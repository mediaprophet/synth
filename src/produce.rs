use ecopunk_terrain::{
    animal_motion::MotionLimits,
    species_knowledge::{KnowledgeDisclosure, KnowledgeLimits, SourceFormat, SourceNamespaceRef},
    species_playback::{NativeProjectionBudget, SpeciesPlaybackSession},
    species_review_resources::{
        CorpusOptions, ResourceData, ReviewResourceLimits, ReviewResources,
    },
    wallaby_rig::RigLimits,
};
use qualia_core_db::poet_host::invoke::studio_store::checkpoint::{self as io, ExpectedFile};
use sha2::{Digest, Sha256};
use std::{
    fs::File,
    io::Read,
    path::{Path, PathBuf},
    sync::atomic::AtomicBool,
};
const USAGE:&str="synth produce --source FILE.vibe [--source EDIT.vibe ...] --out NEW.hmc [--review-dir NEW_DIR] [--q42 RESOURCE_KEY FILE.q42] [--m64 RESOURCE_KEY FILE.m64] [--reference SOURCE_IRI NAMESPACE_IRI rdf|json|csv|html|declared]. Or: synth produce --reopen CURRENT.hmc --out NEW.hmc [--review-dir NEW_DIR]. Current profile: existing authored r3 wallaby; outputs are unaccepted candidates.";
#[path = "review_export.rs"]
mod review_export;
struct Options {
    sources: Vec<PathBuf>,
    reopen: Option<PathBuf>,
    out: Option<PathBuf>,
    review_dir: Option<PathBuf>,
    resources: Vec<(String, String, PathBuf)>,
    references: Vec<(String, String, SourceFormat)>,
}
fn parse(args: &[String]) -> Result<Options, String> {
    if args.first().is_none_or(|a| a != "produce") {
        return Err(USAGE.into());
    }
    let mut o = Options {
        sources: Vec::new(),
        reopen: None,
        out: None,
        review_dir: None,
        resources: Vec::new(),
        references: Vec::new(),
    };
    let mut index = 1;
    let value = |i: usize| {
        args.get(i)
            .filter(|s| !s.is_empty() && !s.starts_with("--"))
            .cloned()
            .ok_or_else(|| USAGE.to_owned())
    };
    while index < args.len() {
        match args[index].as_str() {
            "--source" => {
                o.sources.push(value(index + 1)?.into());
                index += 2;
            }
            "--out" if o.out.is_none() => {
                o.out = Some(value(index + 1)?.into());
                index += 2;
            }
            "--review-dir" if o.review_dir.is_none() => {
                o.review_dir = Some(value(index + 1)?.into());
                index += 2;
            }
            "--reopen" if o.reopen.is_none() => {
                o.reopen = Some(value(index + 1)?.into());
                index += 2;
            }
            "--q42" | "--m64" => {
                o.resources.push((
                    value(index + 1)?,
                    args[index][2..].to_owned(),
                    value(index + 2)?.into(),
                ));
                index += 3;
            }
            "--reference" => {
                let source = value(index + 1)?;
                let namespace = value(index + 2)?;
                let format = match value(index + 3)?.as_str() {
                    "rdf" => SourceFormat::Rdf,
                    "json" => SourceFormat::JsonCompatibility,
                    "csv" => SourceFormat::Csv,
                    "html" => SourceFormat::Html,
                    "declared" => SourceFormat::OtherDeclared,
                    _ => return Err(USAGE.into()),
                };
                o.references.push((source, namespace, format));
                index += 4;
            }
            _ => return Err(USAGE.into()),
        }
    }
    if o.out.is_none()
        || o.sources.len() > 128
        || o.resources.len() > 31
        || o.references.len() > 512
    {
        return Err(USAGE.into());
    }
    if o.reopen.is_some() {
        if !o.sources.is_empty() || !o.resources.is_empty() || !o.references.is_empty() {
            return Err("Reopen retains exact source/resources; combine edits through the guarded agent host".into());
        }
    } else if o.sources.is_empty() {
        return Err(USAGE.into());
    }
    Ok(o)
}
fn read(path: &Path, max: usize) -> Result<Vec<u8>, String> {
    let metadata = std::fs::symlink_metadata(path).map_err(|e| e.to_string())?;
    if !metadata.is_file() || metadata.file_type().is_symlink() || metadata.len() > max as u64 {
        return Err("Input must be a regular file within the selected byte budget".into());
    }
    let mut bytes = Vec::new();
    File::open(path)
        .map_err(|e| e.to_string())?
        .take(max as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() > max {
        return Err("Input grew beyond the selected byte budget".into());
    }
    Ok(bytes)
}
pub fn run(args: &[String]) -> Result<(), String> {
    if args.is_empty() || (args.len() == 1 && args[0] == "--help") {
        println!("{USAGE}");
        return Ok(());
    }
    let options = parse(args)?;
    let destination = options.out.as_ref().ok_or("Output missing")?;
    if std::fs::symlink_metadata(destination).is_ok() {
        return Err("Output already exists; choose an explicit new candidate revision".into());
    }
    destination
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."))
        .canonicalize()
        .map_err(|_| "Output directory must already exist")?;
    if destination.extension().and_then(|s| s.to_str()) != Some("hmc") {
        return Err("Output must use the native .hmc extension".into());
    }
    let review_dir = if let Some(path) = options.review_dir.as_ref() {
        if std::fs::symlink_metadata(path).is_ok() {
            return Err("Review directory already exists; choose a new directory".into());
        }
        let parent = path.parent().filter(|p| !p.as_os_str().is_empty())
            .unwrap_or(Path::new("."))
            .canonicalize().map_err(|_| "Review directory parent must already exist")?;
        let name = path.file_name().ok_or("Review directory needs a name")?;
        Some(parent.join(name))
    } else {
        None
    };
    let limits = ReviewResourceLimits::default();
    let cancel = AtomicBool::new(false);
    let playback = if let Some(path) = options.reopen {
        let bytes = io::load_bundle_cancellable(&path, limits.package_bytes, &cancel)?;
        SpeciesPlaybackSession::reopen(&bytes, limits, MotionLimits::default(), &cancel)?
    } else {
        let mut source_bytes = 0usize;
        let mut cells = Vec::new();
        for path in options.sources {
            let bytes = read(&path, 1024 * 1024)?;
            source_bytes = source_bytes
                .checked_add(bytes.len())
                .ok_or("Source byte count overflow")?;
            if source_bytes > 8 * 1024 * 1024 {
                return Err("Retained source exceeds the existing 8 MiB profile".into());
            }
            cells.push(String::from_utf8(bytes).map_err(|_| "Vibe source must be UTF-8")?);
        }
        let mut entries = Vec::new();
        let mut total = 0usize;
        for (key, kind, path) in options.resources {
            let bytes = read(&path, limits.resource_bytes)?;
            total = total
                .checked_add(bytes.len())
                .ok_or("Resource byte count overflow")?;
            if total > limits.total_bytes {
                return Err("Resource inventory exceeds selected budget".into());
            }
            entries.push(ResourceData {
                key,
                kind,
                bytes,
                meta: None,
            });
        }
        let resources = ReviewResources::admit(entries, limits)?;
        let references: Vec<_> = options
            .references
            .iter()
            .map(|(source, namespace, format)| SourceNamespaceRef {
                source_iri: source,
                namespace_iri: namespace,
                format: *format,
            })
            .collect();
        let corpus = CorpusOptions {
            sources: &references,
            disclosure: KnowledgeDisclosure::Restricted,
            knowledge: KnowledgeLimits::default(),
        };
        SpeciesPlaybackSession::from_authoritative_source_cells(
            &cells,
            resources,
            limits,
            RigLimits::default(),
            MotionLimits::default(),
            &corpus,
            NativeProjectionBudget::default(),
            &cancel,
        )?
    };
    let asset = playback.asset().ok_or("No candidate generated")?;
    let saved = io::save_bundle(
        destination,
        &asset.bundle,
        ExpectedFile::Absent,
        limits.package_bytes,
    )?;
    use std::fmt::Write;
    let mut hash = String::with_capacity(64);
    for byte in saved.digest {
        let _ = write!(&mut hash, "{byte:02x}");
    }
    println!("Synth candidate saved: {}", saved.path.display());
    println!(
        "SHA-256: {hash}; bytes: {}; source revision: {}",
        saved.bytes,
        playback.revision()
    );
    println!("Local generation only. Authenticated project audit and human/domain review are not asserted. Complete film/game/cartoon packages remain unaccepted.");
    if let Some(warning) = saved.durability_warning {
        eprintln!("{warning}");
    }
    if let Some(review_dir) = review_dir {
        let original_bundle = playback.asset().ok_or("Saved candidate missing")?.bundle.clone();
        let saved_bundle = io::load_bundle_cancellable(
            &saved.path, limits.package_bytes, &cancel,
        )?;
        if saved_bundle != original_bundle
            || <[u8; 32]>::from(Sha256::digest(&saved_bundle)) != saved.digest
        {
            return Err("Saved HMC bytes differ from the admitted candidate or save digest".into());
        }
        let source_digest = playback.vibe_source_sha256()
            .ok_or("Saved source-history digest missing")?.to_owned();
        let mut reopened = SpeciesPlaybackSession::reopen(
            &saved_bundle, limits, MotionLimits::default(), &cancel,
        )?;
        if reopened.revision() != playback.revision()
            || reopened.vibe_source_sha256() != Some(source_digest.as_str())
            || reopened.asset().ok_or("Reopened candidate missing")?.bundle != original_bundle
        {
            return Err("Strict fresh HMC reopen changed the source, revision, or package".into());
        }
        let receipt = review_export::write_review_export(
            &mut reopened, &review_dir, &saved.path, limits, &cancel,
        )?;
        println!("Synth review snapshot: {}", serde_json::to_string(&receipt)
            .map_err(|e| format!("Review receipt: {e}"))?);
    }
    Ok(())
}
