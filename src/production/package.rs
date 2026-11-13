use super::*;
use qualia_core_db::bundle::{BundleReadLimits, BundleReader, BundleWriter};
use serde::{Deserialize, Serialize};

const MANIFEST: &str = "synth/production.json";
const SCHEMA: &str = "synth.production/v1";
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    schema: String,
    producer: String,
    version: String,
    sources: Vec<String>,
    inputs: Vec<Entry>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Entry {
    key: String,
    kind: String,
    sha256: String,
}

pub(super) fn read_limits(limits: &ProductionLimits) -> Result<BundleReadLimits, String> {
    Ok(BundleReadLimits {
        file_bytes: limits.package_bytes,
        entries: limits
            .resources
            .checked_mul(2)
            .and_then(|n| n.checked_add(1))
            .ok_or("Entry count overflow")?,
        key_bytes: 1040,
        ..BundleReadLimits::default()
    })
}
fn manifest_limit(limits: &ProductionLimits) -> usize {
    limits
        .source
        .retained_source_bytes
        .saturating_mul(6)
        .saturating_add(limits.resources.saturating_mul(2048))
        .saturating_add(8192)
        .min(limits.package_bytes)
}
pub(super) fn build(
    producer: &dyn AssetProducer,
    sources: &[String],
    inputs: &[Resource],
    outputs: &[Resource],
    limits: &ProductionLimits,
) -> Result<Vec<u8>, String> {
    let manifest = Manifest {
        schema: SCHEMA.into(),
        producer: producer.id().into(),
        version: producer.version().into(),
        sources: sources.to_vec(),
        inputs: inputs
            .iter()
            .map(|r| Entry {
                key: r.key.clone(),
                kind: r.kind.clone(),
                sha256: digest(&r.bytes),
            })
            .collect(),
    };
    let manifest = serde_json::to_vec(&manifest).map_err(|e| e.to_string())?;
    if manifest.len() > manifest_limit(limits) {
        return Err("Manifest exceeds byte budget".into());
    }
    let total = inputs
        .iter()
        .chain(outputs)
        .try_fold(0usize, |n, r| n.checked_add(r.bytes.len()))
        .ok_or("Payload size overflow")?;
    if total > limits.total_resource_bytes {
        return Err("Combined input/output bytes exceed budget".into());
    }
    // Reserve conservative padding and index overhead before BundleWriter allocates.
    let reserve = inputs
        .len()
        .checked_add(outputs.len())
        .and_then(|n| n.checked_add(2))
        .and_then(|n| n.checked_mul(4096))
        .and_then(|n| n.checked_add(1024 * 1024))
        .and_then(|n| n.checked_add(manifest.len()))
        .and_then(|n| n.checked_add(total))
        .ok_or("Package size overflow")?;
    if reserve > limits.package_bytes {
        return Err("Package reservation exceeds byte budget".into());
    }
    let mut writer = BundleWriter::new();
    writer
        .add_file(MANIFEST, "json", manifest, None)
        .map_err(|e| e.to_string())?;
    for (prefix, resources) in [("inputs", inputs), ("outputs", outputs)] {
        for r in resources {
            writer
                .add_file(
                    if producer.id() == "files" && producer.version() == "2" && prefix == "outputs" && r.key.starts_with("textures/sha256/") && r.kind.starts_with("image/") {
                        r.key.clone()
                    } else { format!("{prefix}/{}", r.key) },
                    &r.kind,
                    r.bytes.clone(),
                    None,
                )
                .map_err(|e| e.to_string())?;
        }
    }
    let bytes = writer.build().map_err(|e| e.to_string())?;
    BundleReader::parse_with_limits(&bytes, read_limits(limits)?).map_err(|e| e.to_string())?;
    Ok(bytes)
}

pub(super) fn reopen(
    registry: &ProducerRegistry,
    bytes: &[u8],
    limits: ProductionLimits,
    cancel: &AtomicBool,
) -> Result<ProductionSession, String> {
    if cancel.load(Ordering::Acquire) {
        return Err("Production cancelled".into());
    }
    let reader =
        BundleReader::parse_with_limits(bytes, read_limits(&limits)?).map_err(|e| e.to_string())?;
    if !reader.entries().iter().all(|e| reader.verify_entry(&e.key)) {
        return Err("HMC resource digest mismatch".into());
    }
    let raw = reader
        .get(MANIFEST)
        .ok_or("Not a Synth general production package")?;
    let manifest_ceiling = limits.for_retained_inputs(&[])?;
    if raw.len() > manifest_limit(&manifest_ceiling) {
        return Err("Manifest exceeds byte budget".into());
    }
    let manifest: Manifest =
        serde_json::from_slice(raw).map_err(|e| format!("Production manifest: {e}"))?;
    if manifest.schema != SCHEMA {
        return Err("Unsupported production manifest version".into());
    }
    let producer = registry
        .producers
        .get(&manifest.producer)
        .ok_or_else(|| format!("Unknown producer: {}", manifest.producer))?;
    let producer: Arc<dyn AssetProducer> = if producer.version() == manifest.version {
        producer.clone()
    } else if manifest.producer == "files" && manifest.version == "1" {
        Arc::new(super::files::LegacyFilesProducer)
    } else {
        return Err("Producer version does not match saved package".into());
    };
    if manifest.sources.is_empty()
        || manifest.sources.len() as u64 > limits.source.revisions
        || manifest.inputs.len() > limits.resources
    {
        return Err("Manifest count exceeds source/resource limits".into());
    }
    let mut inputs = Vec::new();
    let mut total = 0usize;
    for input in manifest.inputs {
        valid_key(&input.key)?;
        let key = format!("inputs/{}", input.key);
        let entry = reader.entry(&key).ok_or("Missing retained input")?;
        if entry.kind != input.kind {
            return Err("Input kind differs from manifest".into());
        }
        let bytes = reader.get(&key).ok_or("Missing input bytes")?;
        if digest(bytes) != input.sha256 {
            return Err("Retained input digest differs from manifest".into());
        }
        total = total
            .checked_add(bytes.len())
            .ok_or("Input size overflow")?;
        if bytes.len() > limits.resource_bytes || total > limits.total_resource_bytes {
            return Err("Retained input exceeds limits".into());
        }
        inputs.push(Resource {
            key: input.key,
            kind: input.kind,
            bytes: bytes.to_vec(),
        });
    }
    validate_resources(&inputs, &limits)?;
    let limits = limits.for_retained_inputs(&inputs)?;
    let mut source_bytes = 0usize;
    for source in &manifest.sources {
        source_bytes = source_bytes.checked_add(source.len()).ok_or("Source bytes overflow")?;
        if source.len() > limits.source.submission_bytes || source_bytes > limits.source.retained_source_bytes {
            return Err(format!("Saved source admission refused: submission {} bytes (available {}), history {} bytes (available {})", source.len(), limits.source.submission_bytes, source_bytes, limits.source.retained_source_bytes));
        }
    }
    let mut session = ProductionSession::new(producer, inputs, limits);
    for source in &manifest.sources {
        session.submit(source, cancel)?;
    }
    if session
        .candidate()
        .ok_or("Replay produced no candidate")?
        .bundle()
        != bytes
    {
        return Err("Saved package differs from strict source/producer replay".into());
    }
    Ok(session)
}
