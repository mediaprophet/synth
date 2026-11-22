use super::*;
use qualia_core_db::bundle::{BundleReadLimits, BundleReader, BundleWriter};
use qualia_core_db::bundle::semantic_model::{
    open_semantic_model, resource_identity_meta, SemanticModelBuilder, SEMANTIC_MODEL_KEY,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

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
            .and_then(|n| n.checked_add(2))
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

fn bundle_entry_key(producer: &dyn AssetProducer, prefix: &str, resource: &Resource) -> String {
    if producer.id() == "files"
        && producer.version() == "2"
        && prefix == "outputs"
        && resource.key.starts_with("textures/sha256/")
        && resource.kind.starts_with("image/")
    {
        resource.key.clone()
    } else {
        format!("{prefix}/{}", resource.key)
    }
}

pub(super) fn build(
    producer: &dyn AssetProducer,
    sources: &[String],
    inputs: &[Resource],
    outputs: &[Resource],
    limits: &ProductionLimits,
) -> Result<Vec<u8>, String> {
    build_envelope(producer, sources, inputs, outputs, limits, true)
}

fn build_envelope(
    producer: &dyn AssetProducer,
    sources: &[String],
    inputs: &[Resource],
    outputs: &[Resource],
    limits: &ProductionLimits,
    entry_selectors: bool,
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

    // The source manifest and the intact entry identities determine this package's IRI.
    let mut package_identity = Sha256::new();
    package_identity.update(&manifest);
    for output in outputs {
        for term in [output.key.as_bytes(), output.kind.as_bytes(), &output.bytes] {
            package_identity.update((term.len() as u64).to_le_bytes());
            package_identity.update(term);
        }
    }
    let manifest_iri = format!("urn:sha256:{:x}", package_identity.finalize());
    let mut semantic_model = SemanticModelBuilder::new(
        limits.resource_bytes.min(limits.total_resource_bytes - total),
    )
    .identify_package(&manifest_iri)
    .map_err(|e| format!("HMC semantic model: {e}"))?;
    // Resolve shared terms before admitting any spatial records that reference them.
    for resource in inputs.iter().chain(outputs) {
        semantic_model = match resource.kind.as_str() {
            "q42" => semantic_model.merge_q42(&resource.bytes),
            "cbld" => semantic_model.merge_cbld(&resource.bytes),
            _ => continue,
        }.map_err(|e| format!("HMC companion semantics for {}: {e}", resource.key))?;
    }
    let mut spatial_entries = std::collections::BTreeMap::new();
    for (index, resource) in inputs.iter().chain(outputs).enumerate().filter(|(_, r)| r.kind == "10d") {
        let (next, spatial) = semantic_model
            .merge_10d(&resource.bytes)
            .map_err(|e| format!("HMC semantic model for {}: {e}", resource.key))?;
        semantic_model = next;
        let spatial = spatial.ok_or_else(|| {
            format!("HMC 10d resource has no canonical HOLO identity: {}", resource.key)
        })?;
        spatial_entries.insert(index, spatial);
    }
    let mut entry_meta = Vec::with_capacity(inputs.len().saturating_add(outputs.len()));
    let resources = inputs.iter().map(|resource| ("inputs", resource))
        .chain(outputs.iter().map(|resource| ("outputs", resource)));
    for (index, (prefix, resource)) in resources.enumerate() {
        let content_digest = Sha256::digest(&resource.bytes);
        let content_iri = format!("urn:sha256:{content_digest:x}");
        let content_digest: [u8; 32] = content_digest.into();
        let spatial = spatial_entries.remove(&index);
        let iri = spatial.as_ref().map_or(content_iri.as_str(), |meta| meta.root_entity_iri.as_str());
        semantic_model = semantic_model
            .merge_resource_addressability(&resource.kind, &resource.bytes, iri, &content_digest)
            .map_err(|e| format!("HMC resource semantics for {}: {e}", resource.key))?;
        semantic_model = if entry_selectors {
            let key = bundle_entry_key(producer, prefix, resource);
            semantic_model.add_package_entry_selector(&manifest_iri, &key, iri, &content_digest)
        } else {
            semantic_model.add_package_entry(&manifest_iri, iri)
        }.map_err(|e| format!("HMC semantic model: {e}"))?;
        entry_meta.push(
            match &spatial {
                Some(meta) => meta.encode(),
                None => resource_identity_meta(iri),
            }.map_err(|e| e.to_string())?,
        );
    }
    let semantic_meta = resource_identity_meta(&manifest_iri).map_err(|e| e.to_string())?;
    let metadata_bytes = entry_meta.iter().try_fold(semantic_meta.len(), |size, bytes| size.checked_add(bytes.len()))
        .ok_or("Semantic entry metadata size overflow")?;
    let semantic_model = semantic_model.finish().map_err(|e| format!("HMC semantic model: {e}"))?;
    // Reserve conservative padding and index overhead before BundleWriter allocates.
    let reserve = inputs
        .len()
        .checked_add(outputs.len())
        .and_then(|n| n.checked_add(3))
        .and_then(|n| n.checked_mul(4096))
        .and_then(|n| n.checked_add(1024 * 1024))
        .and_then(|n| n.checked_add(manifest.len()))
        .and_then(|n| n.checked_add(semantic_model.len()))
        .and_then(|n| n.checked_add(metadata_bytes))
        .and_then(|n| n.checked_add(total))
        .ok_or("Package size overflow")?;
    if reserve > limits.package_bytes {
        return Err("Package reservation exceeds byte budget".into());
    }
    let mut writer = BundleWriter::new().with_page_align_4096(true);
    writer
        .add_file(MANIFEST, "json", manifest, None)
        .map_err(|e| e.to_string())?;
    writer
        .add_file(SEMANTIC_MODEL_KEY, "q42", semantic_model, Some(semantic_meta))
        .map_err(|e| e.to_string())?;
    let mut entry_meta = entry_meta.into_iter();
    for (prefix, resources) in [("inputs", inputs), ("outputs", outputs)] {
        for r in resources {
            writer
                .add_file(
                    bundle_entry_key(producer, prefix, r),
                    &r.kind,
                    r.bytes.clone(),
                    entry_meta.next(),
                )
                .map_err(|e| e.to_string())?;
        }
    }
    let bytes = writer.build().map_err(|e| e.to_string())?;
    let bundle =
        BundleReader::parse_with_limits(&bytes, read_limits(limits)?).map_err(|e| e.to_string())?;
    open_semantic_model(&bundle).map_err(|e| format!("HMC semantic model: {e}"))?;
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
    let candidate = session.candidate().ok_or("Replay produced no candidate")?;
    let replay = candidate.bundle();
    if replay == bytes {
        return Ok(session);
    }
    let compatible = if reader.entry(SEMANTIC_MODEL_KEY).is_some() {
        // Reproduce the former root-only semantic envelope byte for byte.
        build_envelope(session.producer.as_ref(), session.sources(), &session.inputs,
            candidate.outputs(), &session.limits, false)? == bytes
    } else {
        matches_legacy_replay(&reader, replay, bytes, &session.limits)?
    };
    if !compatible {
        return Err("Saved package differs from strict source/producer replay".into());
    }
    Ok(session)
}

fn matches_legacy_replay(
    original: &BundleReader<'_>,
    replay: &[u8],
    bytes: &[u8],
    limits: &ProductionLimits,
) -> Result<bool, String> {
    if original.entry(SEMANTIC_MODEL_KEY).is_some() || original.entry("manifest.q42").is_some() {
        return Ok(false);
    }
    // Format migration is admitted only after reproducing the entire historical envelope
    // from the replayed manifest and payloads, including entry order, index and checksums.
    let replay = BundleReader::parse_with_limits(replay, read_limits(limits)?)
        .map_err(|e| e.to_string())?;
    let mut legacy = BundleWriter::new();
    for entry in replay.entries().iter().filter(|entry| entry.key != SEMANTIC_MODEL_KEY) {
        let payload = replay.get(&entry.key).ok_or("Missing replay resource")?;
        legacy.add_file(&entry.key, &entry.kind, payload.to_vec(), None)
            .map_err(|e| e.to_string())?;
    }
    legacy.build().map(|sealed| sealed == bytes).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_legacy_replay_migrates_to_the_authoritative_model() {
        let limits = ProductionLimits::default();
        let cancel = AtomicBool::new(false);
        let registry = ProducerRegistry::with_builtins();
        let mut session = registry.session("files", vec![], limits.clone()).unwrap();
        session.submit(
            r#"let mut asset = {files:[{key:"legacy.txt",kind:"txt",text:"original"}]};"#,
            &cancel,
        ).unwrap();
        let current = session.candidate().unwrap().bundle();
        let current_view = BundleReader::parse_with_limits(current, read_limits(&limits).unwrap()).unwrap();
        let legacy = |payload: Option<&[u8]>| {
            let mut writer = BundleWriter::new();
            for row in current_view.entries().iter().filter(|row| row.key != SEMANTIC_MODEL_KEY) {
                let input = if row.key == "outputs/legacy.txt" {
                    payload.or_else(|| current_view.get(&row.key))
                } else {
                    current_view.get(&row.key)
                }.unwrap();
                writer.add_file(&row.key, &row.kind, input.to_vec(), None).unwrap();
            }
            writer.build().unwrap()
        };
        let opened = registry.reopen(&legacy(None), limits.clone(), &cancel).unwrap();
        assert_eq!(opened.candidate().unwrap().bundle(), current);
        let root_only = build_envelope(
            session.producer.as_ref(),
            session.sources(),
            &session.inputs,
            session.candidate().unwrap().outputs(),
            &limits,
            false,
        ).unwrap();
        assert_ne!(root_only.as_slice(), current);
        let opened = registry.reopen(&root_only, limits.clone(), &cancel).unwrap();
        assert_eq!(opened.candidate().unwrap().bundle(), current);
        assert!(registry.reopen(&legacy(Some(b"changed")), limits, &cancel).is_err());
    }
}
