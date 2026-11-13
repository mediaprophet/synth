//! Bounded native `.10d` visual projection for candidate inspection.
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{
    fs::{self, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
};
use webizen_render::VolumetricRenderer;

const MAX_ASSET_BYTES: usize = 42 * 1024 * 1024;
const PREVIEW_SIDE: u32 = 2048;
const MAX_TOTAL_OUTPUT_BYTES: usize = 64 * 1024 * 1024;

fn read_asset(path: &Path) -> Result<Vec<u8>, String> {
    let meta = fs::symlink_metadata(path).map_err(|e| format!("asset metadata: {e}"))?;
    if !meta.is_file() || meta.file_type().is_symlink() || meta.len() > MAX_ASSET_BYTES as u64 {
        return Err("--asset must be an ordinary `.10d` file within the 42 MiB bound".into());
    }
    let mut bytes = Vec::new();
    fs::File::open(path)
        .map_err(|e| e.to_string())?
        .take(MAX_ASSET_BYTES as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.is_empty() || bytes.len() > MAX_ASSET_BYTES {
        return Err("asset grew beyond the selected 42 MiB profile".into());
    }
    Ok(bytes)
}

fn hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(DIGITS[(byte >> 4) as usize] as char);
        out.push(DIGITS[(byte & 15) as usize] as char);
    }
    out
}

fn write_new(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|e| format!("create-only output {}: {e}", path.display()))?;
    file.write_all(bytes).map_err(|e| e.to_string())?;
    file.sync_all().map_err(|e| e.to_string())
}

pub fn run(args: &[String]) -> Result<(), String> {
    if args.len() == 6 || args.len() == 10 { return run_shot(args); }
    if (args.len() != 4 && args.len() != 8) || args[0] != "--asset" || args[2] != "--out" || (args.len() == 8 && (args[4] != "--hmc" || args[6] != "--asset-key")) {
        return Err("Usage: synth preview --asset FILE.10d --out NEW_DIR [--hmc FILE.hmc --asset-key KEY] [or --shot SHOT.json before HMC options]".into());
    }
    let requested_asset_path = PathBuf::from(&args[1]);
    if requested_asset_path.extension().and_then(|s| s.to_str()) != Some("10d") {
        return Err("--asset must use the `.10d` extension".into());
    }
    let bytes = read_asset(&requested_asset_path)?;
    let asset_path = requested_asset_path
        .canonicalize()
        .map_err(|e| format!("asset path: {e}"))?;
    let source_sha256 = hex(&Sha256::digest(&bytes));
    let mut renderer = VolumetricRenderer::new_offscreen(PREVIEW_SIDE, PREVIEW_SIDE, 256)
        .map_err(|e| format!("native offscreen renderer: {e}"))?;
    let texture_receipt = if args.len() == 8 { load_shot_textures(&mut renderer, &bytes, &args[5], &args[7])? }
        else { json!({"status":"no HMC supplied; renderer fallback textures may be used"}) };
    let (geometry, views) = renderer
        .render_10d_review_pngs(&bytes, PREVIEW_SIDE)
        .map_err(|e| format!("native `.10d` review render: {e}"))?;
    let total = views
        .iter()
        .try_fold(0usize, |sum, (_, png)| sum.checked_add(png.len()))
        .ok_or("preview output size overflow")?;
    if total > MAX_TOTAL_OUTPUT_BYTES {
        return Err("preview PNG set exceeds 64 MiB".into());
    }
    let output_dir = PathBuf::from(&args[3]);
    if fs::symlink_metadata(&output_dir).is_ok() {
        return Err("--out must name a new directory".into());
    }
    let parent = output_dir
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."))
        .canonicalize()
        .map_err(|_| "--out parent must already exist")?;
    let name = output_dir
        .file_name()
        .ok_or("--out needs a directory name")?;
    let destination = parent.join(name);
    let mut records = Vec::with_capacity(views.len());
    for (view, png) in &views {
        records.push(json!({
            "file": format!("{}.png", view.name),
            "camera_yaw": view.yaw,
            "camera_pitch": view.pitch,
            "camera_target": geometry.camera_target,
            "camera_distance_requested": geometry.requested_camera_distance,
            "camera_distance_applied": geometry.camera_distance,
            "sha256": hex(&Sha256::digest(png)),
            "bytes": png.len(),
            "width": PREVIEW_SIDE,
            "height": PREVIEW_SIDE,
        }));
    }
    let receipt = json!({
        "schema": "synth.native-10d-preview/v1",
        "scope": "native readback PNG projection; not model acceptance or an adapter capability receipt",
        "source": {"path": asset_path, "sha256": source_sha256, "bytes": bytes.len()},
        "renderer": "QualiaDB webizen-render::VolumetricRenderer / PortalGpu native offscreen path",
        "projection": geometry,
        "textures": texture_receipt,
        "views": records,
    });
    let receipt_bytes = serde_json::to_vec_pretty(&receipt).map_err(|e| e.to_string())?;
    let named_views: Vec<(String, Vec<u8>)> = views
        .iter()
        .map(|(v, png)| (v.name.to_string(), png.clone()))
        .collect();
    let html_bytes = synth::present::render_10d_preview_html(&receipt, &named_views)?
        .into_bytes();
    if total
        .checked_add(receipt_bytes.len())
        .and_then(|sum| sum.checked_add(html_bytes.len()))
        .ok_or("output size overflow")?
        > MAX_TOTAL_OUTPUT_BYTES
    {
        return Err("preview package exceeds 64 MiB".into());
    }
    fs::create_dir(&destination).map_err(|e| format!("create-only preview directory: {e}"))?;
    for (view, png) in views {
        write_new(&destination.join(format!("{}.png", view.name)), &png)?;
    }
    write_new(&destination.join("review-receipt.json"), &receipt_bytes)?;
    write_new(&destination.join("index.html"), &html_bytes)?;
    println!("Native review preview written: {}", destination.display());
    Ok(())
}

/// Render a caller-authored shot and optional digest-verified HMC textures.
fn run_shot(args: &[String]) -> Result<(), String> {
    if args[0] != "--asset" || args[2] != "--out" || args[4] != "--shot" || (args.len() == 10 && (args[6] != "--hmc" || args[8] != "--asset-key")) {
        return Err("Usage: synth preview --asset FILE.10d --out NEW_DIR --shot SHOT.json [--hmc FILE.hmc --asset-key KEY]".into());
    }
    let asset_path = PathBuf::from(&args[1]);
    if asset_path.extension().and_then(|s| s.to_str()) != Some("10d") { return Err("--asset must use .10d".into()); }
    let bytes = read_asset(&asset_path)?;
    let shot_path = PathBuf::from(&args[5]);
    let meta = fs::symlink_metadata(&shot_path).map_err(|e| format!("shot metadata: {e}"))?;
    if !meta.is_file() || meta.file_type().is_symlink() || meta.len() > 65536 { return Err("shot must be an ordinary JSON file at most 64 KiB".into()); }
    let shot_bytes = fs::read(&shot_path).map_err(|e| format!("shot read: {e}"))?;
    if shot_bytes.len() > 65536 { return Err("shot exceeds 64 KiB".into()); }
    let shot: webizen_render::NativeShot = serde_json::from_slice(&shot_bytes).map_err(|e| format!("shot JSON: {e}"))?;
    shot.validate()?;
    let work = (shot.side as usize).checked_mul(shot.side as usize).and_then(|n| n.checked_mul(4)).and_then(|n| n.checked_mul(shot.times.len())).ok_or("shot output size overflow")?;
    if work > MAX_TOTAL_OUTPUT_BYTES { return Err("shot frames exceed 64 MiB uncompressed budget".into()); }
    let output_dir = PathBuf::from(&args[3]);
    if fs::symlink_metadata(&output_dir).is_ok() { return Err("--out must name a new directory".into()); }
    let parent = output_dir.parent().filter(|p| !p.as_os_str().is_empty()).unwrap_or(Path::new(".")).canonicalize().map_err(|_| "--out parent must already exist")?;
    let destination = parent.join(output_dir.file_name().ok_or("--out needs a directory name")?);
    let mut renderer = VolumetricRenderer::new_offscreen(shot.side, shot.side, 256).map_err(|e| format!("native renderer: {e}"))?;
    let texture_receipt = if args.len() == 10 { load_shot_textures(&mut renderer, &bytes, &args[7], &args[9])? }
        else { json!({"status":"no HMC supplied; renderer fallback textures may be used"}) };
    let (geometry, frames) = renderer.render_10d_shot(&bytes, &shot)?;
    let mut records = Vec::new();
    let mut named = Vec::new();
    let mut total = 0usize;
    for (i, png) in frames.into_iter().enumerate() {
        total = total.checked_add(png.len()).ok_or("shot output overflow")?;
        if total > MAX_TOTAL_OUTPUT_BYTES { return Err("shot PNGs exceed output budget".into()); }
        let name = format!("frame-{i:04}");
        records.push(json!({"file":format!("{name}.png"), "time_seconds":shot.times[i], "sha256":hex(&Sha256::digest(&png)), "bytes":png.len(), "width":shot.side, "height":shot.side}));
        named.push((name, png));
    }
    let receipt = json!({"schema":"synth.native-shot/v1", "source":{"path":asset_path.canonicalize().map_err(|e| e.to_string())?, "sha256":hex(&Sha256::digest(&bytes)), "bytes":bytes.len()}, "shot":shot, "projection":geometry, "textures":texture_receipt, "views":records});
    let receipt_bytes = serde_json::to_vec_pretty(&receipt).map_err(|e| e.to_string())?;
    // HTML embeds all PNGs as base64; admit their maximum combined size before allocation.
    let embedded = total.checked_add(2).and_then(|n| n.checked_div(3)).and_then(|n| n.checked_mul(4)).ok_or("shot HTML size overflow")?;
    if total.checked_add(embedded).and_then(|n| n.checked_add(receipt_bytes.len() * 8)).and_then(|n| n.checked_add(1024 * 1024)).ok_or("shot package overflow")? > MAX_TOTAL_OUTPUT_BYTES { return Err("shot package exceeds embedded HTML budget".into()); }
    let html = synth::present::render_10d_preview_html(&receipt, &named)?.into_bytes();
    if total.checked_add(receipt_bytes.len()).and_then(|n| n.checked_add(html.len())).ok_or("shot package overflow")? > MAX_TOTAL_OUTPUT_BYTES { return Err("shot package exceeds output budget".into()); }
    fs::create_dir(&destination).map_err(|e| format!("create-only preview directory: {e}"))?;
    for (name, png) in named { write_new(&destination.join(format!("{name}.png")), &png)?; }
    write_new(&destination.join("review-receipt.json"), &receipt_bytes)?;
    write_new(&destination.join("index.html"), &html)?;
    println!("Native shot written: {}", destination.display());
    Ok(())
}

fn load_shot_textures(renderer: &mut VolumetricRenderer, bytes: &[u8], hmc_path: &str, asset_key: &str) -> Result<serde_json::Value, String> {
    use qualia_core_db::render::texture_stream_policy::TextureStreamBudget;
    let hmc = read_asset(Path::new(hmc_path))?;
    let bundle = qualia_core_db::bundle::BundleReader::parse(&hmc).map_err(|e| format!("HMC: {e}"))?;
    if !bundle.verify_entry(asset_key) || bundle.get(asset_key) != Some(bytes) { return Err("HMC asset key must resolve to the exact supplied .10d".into()); }
    let (_, report) = renderer.load_hmc_asset_with_texture_budget(&hmc, asset_key, TextureStreamBudget { max_resident_bytes: 64 * 1024 * 1024, max_upload_bytes: 64 * 1024 * 1024 }).map_err(|e| format!("HMC texture loading: {e}"))?;
    if report.deferred_interpretations != 0 || report.capability_refusals != 0 || report.requested_interpretations != report.resident_interpretations { return Err("shot texture admission deferred or refused a material texture".into()); }
    Ok(json!({"hmc_sha256": hex(&Sha256::digest(&hmc)), "asset_key": asset_key, "requested": report.requested_interpretations, "resident": report.resident_interpretations}))
}
