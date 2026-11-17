//! Scoped native input admission and durable production receipts for MCP.
use super::*;
use crate::production::Resource;
use sha2::{Digest, Sha256};
use std::{fs, io::{Read, Write}, sync::atomic::AtomicBool};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct InputBinding { key: String, kind: String, path: String }

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ProduceArg {
    source: Option<String>,
    source_path: Option<String>,
    profile: Option<String>,
    quality_profile: Option<String>,
    out_hmc: Option<String>,
    present_html: Option<String>,
    export_dir: Option<String>,
    receipt_json: Option<String>,
    #[serde(default)] inputs: Vec<InputBinding>,
    #[serde(default)] preview_assets: Vec<String>,
    domain: Option<String>,
    taxon: Option<String>,
    fiction: Option<bool>,
    evidence_q42: Option<String>,
}

pub(super) struct InputRoot { configured: PathBuf, canonical: PathBuf }
impl InputRoot {
    pub(super) fn admit(root: &Path) -> Result<Self, String> {
        let meta = fs::symlink_metadata(root).map_err(|_| "Input directory unavailable")?;
        if !meta.is_dir() || meta.file_type().is_symlink() || is_reparse_point(&meta) {
            return Err("Input root must be an existing native directory".into());
        }
        Ok(Self { configured: root.to_owned(), canonical: root.canonicalize().map_err(|_| "Input root cannot be resolved")? })
    }

    pub(super) fn read(&self, requested: &str, max: usize) -> Result<(PathBuf, Vec<u8>), ToolError> {
        let refused = |message| ToolError::new("native_input_refused", message);
        if requested.len() > 4096 || self.configured.canonicalize().ok().as_ref() != Some(&self.canonical) {
            return Err(refused("Input root changed or input path exceeds budget"));
        }
        let requested = Path::new(requested);
        if requested.components().any(|c| matches!(c, std::path::Component::ParentDir)) {
            return Err(refused("Input paths must not contain parent traversal"));
        }
        let path = if requested.is_absolute() { requested.to_owned() } else { self.canonical.join(requested) };
        let meta = fs::symlink_metadata(&path).map_err(|_| refused("Input file unavailable"))?;
        let canonical = path.canonicalize().map_err(|_| refused("Input file cannot be resolved"))?;
        if !canonical.starts_with(&self.canonical) || !meta.is_file() || meta.file_type().is_symlink() {
            return Err(refused("Input must be an ordinary file inside SYNTH_INPUT_ROOT and within budget"));
        }
        if meta.len() > max as u64 {
            return Err(ToolError::new("native_input_refused", format!("Input is {} bytes; available budget is {max} bytes", meta.len())));
        }
        // Reject junctions/symlink ancestors, even those pointing back inside the root.
        let mut ancestor = path.as_path();
        loop {
            let metadata = fs::symlink_metadata(ancestor).map_err(|_| refused("Input path unavailable"))?;
            if metadata.file_type().is_symlink() || is_reparse_point(&metadata) {
                return Err(refused("Input paths must not traverse links or junctions"));
            }
            if ancestor.canonicalize().ok().as_ref() == Some(&self.canonical) { break; }
            ancestor = ancestor.parent().ok_or_else(|| refused("Input path does not reach admitted root"))?;
        }
        let mut bytes = Vec::new();
        fs::File::open(&canonical).map_err(|_| refused("Input file cannot be opened"))?
            .take(max as u64 + 1).read_to_end(&mut bytes).map_err(|_| refused("Input file cannot be read"))?;
        if bytes.len() > max { return Err(ToolError::new("native_input_refused", format!("Input grew to at least {} bytes; available budget is {max} bytes", bytes.len()))); }
        Ok((canonical, bytes))
    }
}

#[cfg(windows)]
fn is_reparse_point(meta: &fs::Metadata) -> bool {
    use std::os::windows::fs::MetadataExt;
    meta.file_attributes() & 0x400 != 0
}
#[cfg(not(windows))]
fn is_reparse_point(_: &fs::Metadata) -> bool { false }

fn digest(bytes: &[u8]) -> String { format!("{:x}", Sha256::digest(bytes)) }
fn binary_identity() -> Result<Value, ToolError> {
    let path = std::env::current_exe().map_err(|e| ToolError::new("identity_failed", e.to_string()))?;
    let mut file = fs::File::open(&path).map_err(|e| ToolError::new("identity_failed", e.to_string()))?;
    let mut hash = Sha256::new();
    let mut buffer = [0u8; 64 * 1024];
    loop {
        let count = file.read(&mut buffer).map_err(|e| ToolError::new("identity_failed", e.to_string()))?;
        if count == 0 { break; }
        hash.update(&buffer[..count]);
    }
    Ok(json!({"version": env!("CARGO_PKG_VERSION"), "path": path, "sha256": format!("{:x}", hash.finalize())}))
}
fn write_new(path: &Path, bytes: &[u8]) -> Result<(), ToolError> {
    let mut file = fs::OpenOptions::new().write(true).create_new(true).open(path)
        .map_err(|e| ToolError::new("native_output_failed", format!("Create-only output failed: {e}")))?;
    file.write_all(bytes).and_then(|_| file.sync_all())
        .map_err(|e| ToolError::new("native_output_failed", e.to_string()))
}
fn destination(mcp: &StandaloneSynthMcp, name: Option<&str>, ext: &str) -> Result<Option<PathBuf>, ToolError> {
    name.map(|name| mcp.confined_destination(name, ext, "Use a visible output name without directory components"))
        .transpose()
}

pub(super) fn produce(mcp: &StandaloneSynthMcp, args: Value) -> Result<Value, ToolError> {
    let parsed: ProduceArg = serde_json::from_value(args)
        .map_err(|e| ToolError::new("invalid_arguments", format!("Invalid synth_produce_present arguments: {e}")))?;
    #[cfg(not(target_arch = "wasm32"))]
    crate::operator::enforce_produce(&crate::operator::RuleRequest {
        domain: parsed.domain.clone(),
        taxon: parsed.taxon.clone(),
        fiction: parsed.fiction,
        action: Some("produce".into()),
        evidence_q42: parsed.evidence_q42.clone(),
    })
    .map_err(|e| ToolError::new("production_rule_blocked", e))?;
    let limits = ProductionLimits::for_quality_profile(parsed.quality_profile.as_deref().unwrap_or("preview"))
        .map_err(|error| ToolError::new("invalid_arguments",error))?;
    if parsed.inputs.len() > limits.resources || parsed.preview_assets.len() > 8 {
        return Err(ToolError::new("invalid_arguments", "Input or preview count exceeds budget"));
    }
    if !parsed.preview_assets.is_empty() && parsed.export_dir.is_none() {
        return Err(ToolError::new("invalid_arguments", "preview_assets requires export_dir"));
    }
    let hmc = destination(mcp, parsed.out_hmc.as_deref(), ".hmc")?;
    let html = destination(mcp, parsed.present_html.as_deref(), ".html")?;
    let export = destination(mcp, parsed.export_dir.as_deref(), "")?;
    let receipt_path = destination(mcp, parsed.receipt_json.as_deref(), ".json")?;
    let mut destinations: Vec<PathBuf> = [&hmc, &html, &export, &receipt_path].into_iter().flatten().cloned().collect();
    let mut previews = Vec::new();
    for (index, key) in parsed.preview_assets.iter().enumerate() {
        if !key.ends_with(".10d") || parsed.preview_assets[..index].contains(key) {
            return Err(ToolError::new("invalid_arguments", "Preview assets must be distinct .10d resource keys"));
        }
        let name = format!("{}-review-{index}", parsed.export_dir.as_deref().unwrap());
        let path = destination(mcp, Some(&name), "")?.unwrap();
        destinations.push(path.clone());
        previews.push((key, path));
    }
    for (index, path) in destinations.iter().enumerate() {
        if fs::symlink_metadata(path).is_ok() || destinations[..index].iter().any(|p| p.to_string_lossy().eq_ignore_ascii_case(&path.to_string_lossy())) {
            return Err(ToolError::new("native_output_refused", "Output already exists or destinations overlap"));
        }
    }
    let mut inputs = Vec::new();
    let mut input_records = Vec::new();
    let mut total = 0usize;
    for binding in &parsed.inputs {
        let root = mcp.input_root.as_ref().ok_or_else(|| ToolError::new("native_input_unconfigured", "Set SYNTH_INPUT_ROOT to admit local inputs"))?;
        let remaining = limits.total_resource_bytes.saturating_sub(total).min(limits.resource_bytes);
        let (path, bytes) = root.read(&binding.path, remaining)?;
        total = total.checked_add(bytes.len()).ok_or_else(|| ToolError::new("input_budget", "Input bytes overflow"))?;
        input_records.push(json!({"key": binding.key, "kind": binding.kind, "path": path, "bytes": bytes.len(), "sha256": digest(&bytes)}));
        inputs.push(Resource { key: binding.key.clone(), kind: binding.kind.clone(), bytes });
    }
    let admitted = limits.for_retained_inputs(&inputs).map_err(|e| ToolError::new("input_budget", e))?;
    let source = match (parsed.source.as_deref(), parsed.source_path.as_deref()) {
        (Some(source), None) => {
            if source.len() > admitted.source.submission_bytes { return Err(ToolError::new("input_budget", format!("Source {} bytes exceeds available {} bytes", source.len(), admitted.source.submission_bytes))); }
            source.to_owned()
        },
        (None, Some(path)) => {
            let root = mcp.input_root.as_ref().ok_or_else(|| ToolError::new("native_input_unconfigured", "Set SYNTH_INPUT_ROOT to admit local source"))?;
            let (_, bytes) = root.read(path, admitted.source.submission_bytes)?;
            String::from_utf8(bytes).map_err(|_| ToolError::new("invalid_arguments", "Source must be UTF-8"))?
        }
        _ => return Err(ToolError::new("invalid_arguments", "Provide exactly one of source or source_path")),
    };
    if source.len() > admitted.source.submission_bytes {
        return Err(ToolError::new("input_budget", format!("Source {} bytes exceeds available {} bytes", source.len(), admitted.source.submission_bytes)));
    }
    let effective = admitted.for_source_usage(source.len()).map_err(|e| ToolError::new("input_budget", e))?;
    let cancel = AtomicBool::new(false);
    let registry = ProducerRegistry::with_builtins();
    let mut session = registry.session(parsed.profile.as_deref().unwrap_or("files"), inputs, limits.clone())
        .map_err(|e| ToolError::new("session_error", e))?;
    session.submit(&source, &cancel).map_err(|e| ToolError::new("production_failed", e))?;
    let candidate = session.candidate().ok_or_else(|| ToolError::new("no_candidate", "No candidate produced"))?;
    registry.reopen(candidate.bundle(), limits.clone(), &cancel).map_err(|e| ToolError::new("replay_failed", e))?;
    for (key, _) in &previews {
        if !candidate.outputs().iter().any(|resource| resource.key == **key && resource.kind == "10d") {
            return Err(ToolError::new("invalid_arguments", "Preview key must identify an exported 10d resource"));
        }
    }
    if export.is_some() && candidate.outputs().iter().any(|r| r.key.eq_ignore_ascii_case(".synth") || r.key.to_ascii_lowercase().starts_with(".synth/")) {
        return Err(ToolError::new("native_output_refused", "The .synth export directory is reserved for source and receipts"));
    }
    // Render HTML before installing any outputs so presentation failures do not leave a half delivery.
    let html_bytes = html.as_ref().map(|_| crate::present::render_session_html(&session)).transpose()
        .map_err(|e| ToolError::new("present_render_failed", e))?;
    if let Some(path) = &hmc {
        qualia_core_db::poet_host::invoke::studio_store::checkpoint::save_bundle(path, candidate.bundle(),
            qualia_core_db::poet_host::invoke::studio_store::checkpoint::ExpectedFile::Absent, limits.package_bytes)
            .map_err(|e| ToolError::new("hmc_save_failed", e))?;
    }
    if let (Some(path), Some(content)) = (&html, html_bytes) {
        crate::present::write_presentation_file(path, &content, limits.package_bytes)
            .map_err(|e| ToolError::new("present_save_failed", e))?;
    }
    if let Some(directory) = &export {
        fs::create_dir(directory).map_err(|e| ToolError::new("export_failed", e.to_string()))?;
        for resource in candidate.outputs() {
            let path = directory.join(&resource.key);
            fs::create_dir_all(path.parent().unwrap()).map_err(|e| ToolError::new("export_failed", e.to_string()))?;
            write_new(&path, &resource.bytes)?;
        }
        fs::create_dir(directory.join(".synth")).map_err(|e| ToolError::new("export_failed", e.to_string()))?;
        write_new(&directory.join(".synth/source.vibe"), source.as_bytes())?;
    }
    let review_hmc = if !previews.is_empty() && hmc.is_none() {
        let path = export.as_ref().unwrap().join(".synth/candidate.hmc");
        write_new(&path, candidate.bundle())?;
        Some(path)
    } else { hmc.clone() };
    let resources: Vec<Value> = candidate.outputs().iter().map(|resource| json!({
        "key": resource.key, "kind": resource.kind, "bytes": resource.bytes.len(),
        "sha256": digest(&resource.bytes), "path": export.as_ref().map(|dir| dir.join(&resource.key))
    })).collect();
    let mut receipt = json!({
        "schema": "synth.production-receipt/v1", "execution_host": "synth.production",
        "handle_lifetime": "one source evaluation; only encoded resources in this receipt are durable",
        "producer": {"id": session.profile().0, "version": session.profile().1},
        "binary": binary_identity()?,
        "catalog": {"count": crate::production::assets::OPERATIONS.len(), "sha256": digest(&serde_json::to_vec(&crate::production::assets::catalogue()).unwrap())},
        "source_sha256": digest(source.as_bytes()), "inputs": input_records,
        "hmc": {"sha256": candidate.sha256(), "bytes": candidate.bundle().len(), "path": review_hmc},
        "quality_profile": parsed.quality_profile.as_deref().unwrap_or("preview"),
        "limits": {"resource_bytes":limits.resource_bytes,"total_resource_bytes":limits.total_resource_bytes,"package_bytes":limits.package_bytes,"asset_work_units":limits.asset_work_units,
            "source_bytes":source.len(),"source_available_bytes":admitted.source.submission_bytes,
            "source_reservation_bytes":limits.total_resource_bytes - effective.total_resource_bytes,
            "source_reservation_kind":"conservative logical estimate, not an allocator bound",
            "evaluation_workspace_bytes":limits.evaluation.workspace_left,
            "combined_logical_budget_bytes":(limits.total_resource_bytes as u64).saturating_add(limits.evaluation.workspace_left),
            "accounting_scope":"per-revision logical component allowances; overlapping candidates and clones are not a process-peak or allocator bound",
            "source_work_units":limits.asset_work_units - effective.asset_work_units,
            "effective_asset_work_units":effective.asset_work_units,
            "evaluation_steps":effective.evaluation.steps_left,
            "effective_host_bytes":effective.total_resource_bytes,
            "standalone_wire_bytes":4*1024*1024},
        "resources": resources, "strict_replay_verified": true,
        "presentation_path": html, "native_reviews": [], "requested_reviews": previews.iter().map(|(key,path)| json!({"resource":key,"path":path,"status":"pending"})).collect::<Vec<_>>(),
        "review_status": if previews.is_empty() {"not_requested"} else {"pending"}, "visual_acceptance": "pending inspection"
    });
    // Persist provenance before GPU work. A failed review does not erase completed exports.
    save_receipt(export.as_deref(), receipt_path.as_deref(), &receipt)?;
    let mut reviews = Vec::new();
    for (key, path) in previews {
        let asset = export.as_ref().unwrap().join(key);
        match super::contracts::native_preview(&asset, &path, review_hmc.as_ref().unwrap(), &format!("outputs/{key}")) {
            Ok(()) => reviews.push(json!({"resource": key, "path": path, "receipt_path": path.join("review-receipt.json")})),
            Err(error) => {
                receipt["native_reviews"] = json!(reviews);
                receipt["review_status"] = json!("failed");
                receipt["review_error"] = json!({"code": error.code, "message": error.message});
                if let Some(directory) = &export {
                    write_new(&directory.join(".synth/review-completion.json"), &serde_json::to_vec_pretty(&receipt).map_err(|e| ToolError::new("receipt_failed", e.to_string()))?)?;
                }
                return Ok(json!({"status": "error", "error": {"code": error.code, "message": error.message}, "receipt": receipt,
                    "completed_outputs_preserved": true}));
            }
        }
    }
    receipt["native_reviews"] = json!(reviews);
    if !parsed.preview_assets.is_empty() { receipt["review_status"] = json!("rendered_pending_inspection"); }
    if let Some(directory) = &export {
        write_new(&directory.join(".synth/review-completion.json"), &serde_json::to_vec_pretty(&receipt).map_err(|e| ToolError::new("receipt_failed", e.to_string()))?)?;
    }
    // The initial durable receipt records requested reviews as pending; review receipts are separate create-only files.
    let mut summary = crate::present::session_presentation_summary(&session, html.as_ref().map(|p| p.to_string_lossy()).as_deref())
        .map_err(|e| ToolError::new("summary_failed", e))?;
    summary["saved_hmc"] = json!(hmc);
    summary["strict_replay_verified"] = json!(true);
    summary["receipt"] = receipt;
    Ok(summary)
}

fn save_receipt(export: Option<&Path>, explicit: Option<&Path>, receipt: &Value) -> Result<(), ToolError> {
    let bytes = serde_json::to_vec_pretty(receipt).map_err(|e| ToolError::new("receipt_failed", e.to_string()))?;
    if let Some(directory) = export { write_new(&directory.join(".synth/production-receipt.json"), &bytes)?; }
    if let Some(path) = explicit { write_new(path, &bytes)?; }
    Ok(())
}
