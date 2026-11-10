use qualia_core_db::poet_host::invoke::studio_store::checkpoint::{self as io, ExpectedFile};
use std::{
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::atomic::AtomicBool,
};
use synth::production::{Candidate, ProducerRegistry, ProductionLimits, ProductionSession, Resource};

const USAGE: &str = "synth produce [--profile files] --source FILE.vibe [--source EDIT.vibe ...] [--input KEY KIND FILE ...] [--resource-mib N] [--total-mib N] [--package-mib N] --out NEW.hmc [--export-dir NEW_DIR] [--present NEW.html]\nsynth produce --reopen SAVED.hmc --out NEW.hmc [--export-dir NEW_DIR] [--present NEW.html]\nsynth present [--profile files] --source FILE.vibe [--source EDIT.vibe ...] [--input KEY KIND FILE ...] --out NEW.html\nsynth present --reopen SAVED.hmc --out NEW.html";

enum CommandMode {
    Produce,
    PresentOnly,
}

struct Options {
    mode: CommandMode,
    profile: Option<String>,
    sources: Vec<PathBuf>,
    inputs: Vec<(String, String, PathBuf)>,
    reopen: Option<PathBuf>,
    out: PathBuf,
    export: Option<PathBuf>,
    present: Option<PathBuf>,
    resource_mib: Option<u64>,
    total_mib: Option<u64>,
    package_mib: Option<u64>,
}

fn parse(args: &[String]) -> Result<Options, String> {
    let mode = match args.first().map(String::as_str) {
        Some("produce") => CommandMode::Produce,
        Some("present") => CommandMode::PresentOnly,
        _ => return Err(USAGE.into()),
    };
    let mut profile = None;
    let mut sources = Vec::new();
    let mut inputs = Vec::new();
    let mut reopen = None;
    let mut out = None;
    let mut export = None;
    let mut present = None;
    let mut resource_mib = None;
    let mut total_mib = None;
    let mut package_mib = None;
    let mut i = 1;
    let value = |i: usize| {
        args.get(i)
            .filter(|s| !s.is_empty() && !s.starts_with("--"))
            .cloned()
            .ok_or_else(|| USAGE.to_owned())
    };
    while i < args.len() {
        match args[i].as_str() {
            "--source" => {
                sources.push(value(i + 1)?.into());
                i += 2;
            }
            "--profile" if profile.is_none() => {
                profile = Some(value(i + 1)?);
                i += 2;
            }
            "--reopen" if reopen.is_none() => {
                reopen = Some(value(i + 1)?.into());
                i += 2;
            }
            "--out" if out.is_none() => {
                out = Some(value(i + 1)?.into());
                i += 2;
            }
            "--export-dir" if matches!(mode, CommandMode::Produce) && export.is_none() => {
                export = Some(value(i + 1)?.into());
                i += 2;
            }
            "--present" if matches!(mode, CommandMode::Produce) && present.is_none() => {
                present = Some(value(i + 1)?.into());
                i += 2;
            }
            "--input" => {
                inputs.push((value(i + 1)?, value(i + 2)?, value(i + 3)?.into()));
                i += 4;
            }
            "--resource-mib" if resource_mib.is_none() => {
                resource_mib = Some(parse_mib(&value(i + 1)?)?);
                i += 2;
            }
            "--total-mib" if total_mib.is_none() => {
                total_mib = Some(parse_mib(&value(i + 1)?)?);
                i += 2;
            }
            "--package-mib" if package_mib.is_none() => {
                package_mib = Some(parse_mib(&value(i + 1)?)?);
                i += 2;
            }
            _ => return Err(USAGE.into()),
        }
    }
    if reopen.is_some() && (profile.is_some() || !sources.is_empty() || !inputs.is_empty())
        || reopen.is_none() && sources.is_empty()
    {
        return Err(
            "Supply source/input/profile for generation, or only --reopen for strict replay".into(),
        );
    }
    Ok(Options {
        mode,
        profile,
        sources,
        inputs,
        reopen,
        out: out.ok_or(USAGE)?,
        export,
        present,
        resource_mib,
        total_mib,
        package_mib,
    })
}

fn parse_mib(text: &str) -> Result<u64, String> {
    text.parse::<u64>()
        .map_err(|_| format!("Byte budget must be a whole number of MiB: {text}"))
}

fn mib_bytes(mib: u64) -> Result<usize, String> {
    let bytes = mib
        .checked_mul(1024 * 1024)
        .ok_or("Byte budget overflow")?;
    usize::try_from(bytes).map_err(|_| "Byte budget overflow".to_owned())
}

fn read(path: &Path, max: usize) -> Result<Vec<u8>, String> {
    let meta = fs::symlink_metadata(path).map_err(|e| e.to_string())?;
    if !meta.is_file() || meta.file_type().is_symlink() || meta.len() > max as u64 {
        return Err("Input must be a regular file within budget".into());
    }
    let mut bytes = Vec::new();
    fs::File::open(path)
        .map_err(|e| e.to_string())?
        .take(max as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() > max {
        return Err("Input grew beyond budget".into());
    }
    Ok(bytes)
}

fn new_destination(path: &Path) -> Result<(), String> {
    if fs::symlink_metadata(path).is_ok() {
        return Err(format!("Output already exists: {}", path.display()));
    }
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    if !parent.is_dir() || path.file_name().is_none() {
        return Err("Output parent must exist".into());
    }
    Ok(())
}

fn export(candidate: &Candidate, directory: &Path) -> Result<(), String> {
    fs::create_dir(directory).map_err(|e| e.to_string())?;
    for resource in candidate.outputs() {
        let path = directory.join(&resource.key);
        fs::create_dir_all(path.parent().ok_or("Missing export parent")?)
            .map_err(|e| e.to_string())?;
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .map_err(|e| e.to_string())?;
        file.write_all(&resource.bytes)
            .and_then(|_| file.sync_all())
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn prepare_session(
    options: &Options,
    registry: &ProducerRegistry,
    limits: &ProductionLimits,
    cancel: &AtomicBool,
) -> Result<ProductionSession, String> {
    if let Some(path) = &options.reopen {
        registry.reopen(&read(path, limits.package_bytes)?, limits.clone(), cancel)
    } else {
        if options.inputs.len() > limits.resources
            || options.sources.len() as u64 > limits.source.revisions
        {
            return Err("Input/source count exceeds limits".into());
        }
        let mut inputs = Vec::new();
        let mut total = 0usize;
        for (key, kind, path) in &options.inputs {
            let bytes = read(path, limits.resource_bytes)?;
            total = total
                .checked_add(bytes.len())
                .ok_or("Input size overflow")?;
            if total > limits.total_resource_bytes {
                return Err("Input bytes exceed limits".into());
            }
            inputs.push(Resource {
                key: key.clone(),
                kind: kind.clone(),
                bytes,
            });
        }
        let mut session = registry.session(
            options.profile.as_deref().unwrap_or("files"),
            inputs,
            limits.clone(),
        )?;
        for path in &options.sources {
            let bytes = read(path, limits.source.submission_bytes)?;
            session.submit(
                std::str::from_utf8(&bytes).map_err(|_| "Source must be UTF-8")?,
                cancel,
            )?;
        }
        Ok(session)
    }
}

pub fn run(args: &[String]) -> Result<(), String> {
    if args == ["--help"] || args == ["produce", "--help"] || args == ["present", "--help"] {
        println!("{USAGE}");
        return Ok(());
    }
    let options = parse(args)?;
    new_destination(&options.out)?;
    match options.mode {
        CommandMode::Produce => {
            if options.out.extension().and_then(|v| v.to_str()) != Some("hmc") {
                return Err("Output must use .hmc".into());
            }
        }
        CommandMode::PresentOnly => {
            if options.out.extension().and_then(|v| v.to_str()) != Some("html") {
                return Err("Presentation output must use .html".into());
            }
        }
    }
    if let Some(path) = &options.export {
        new_destination(path)?;
    }
    if let Some(path) = &options.present {
        new_destination(path)?;
        if path.extension().and_then(|v| v.to_str()) != Some("html") {
            return Err("Presentation output must use .html".into());
        }
    }
    let mut limits = ProductionLimits::default();
    if let Some(mib) = options.resource_mib {
        limits.resource_bytes = mib_bytes(mib)?;
    }
    if let Some(mib) = options.total_mib {
        limits.total_resource_bytes = mib_bytes(mib)?;
    }
    if let Some(mib) = options.package_mib {
        limits.package_bytes = mib_bytes(mib)?;
    }
    let cancel = AtomicBool::new(false);
    let registry = ProducerRegistry::with_builtins();
    let session = prepare_session(&options, &registry, &limits, &cancel)?;
    let candidate = session.candidate().ok_or("No candidate")?;
    // Verify a fresh source replay before creating any output.
    registry.reopen(candidate.bundle(), limits.clone(), &cancel)?;

    if matches!(options.mode, CommandMode::PresentOnly) {
        let html = synth::present::render_session_html(&session)?;
        synth::present::write_presentation_file(&options.out, &html, limits.package_bytes)?;
        println!(
            "Synth {}/{} presentation written: {}",
            session.profile().0,
            session.profile().1,
            options.out.display()
        );
        return Ok(());
    }

    let saved = io::save_bundle(
        &options.out,
        candidate.bundle(),
        ExpectedFile::Absent,
        limits.package_bytes,
    )?;
    println!(
        "Synth {}/{} candidate saved: {}",
        session.profile().0,
        session.profile().1,
        saved.path.display()
    );
    println!(
        "SHA-256: {}; outputs: {}; revision: {}",
        candidate.sha256(),
        candidate.outputs().len(),
        candidate.revision()
    );
    if let Some(warning) = saved.durability_warning {
        eprintln!("{warning}");
    }
    if let Some(directory) = options.export {
        export(candidate, &directory)?;
        println!("Exported: {}", directory.display());
    }
    if let Some(present_path) = options.present {
        let html = synth::present::render_session_html(&session)?;
        synth::present::write_presentation_file(&present_path, &html, limits.package_bytes)?;
        println!("Presented: {}", present_path.display());
    }
    Ok(())
}
