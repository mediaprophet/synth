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
    fs::File::open(path).map_err(|e| e.to_string())?
        .take(MAX_ASSET_BYTES as u64 + 1).read_to_end(&mut bytes).map_err(|e| e.to_string())?;
    if bytes.is_empty() || bytes.len() > MAX_ASSET_BYTES {
        return Err("asset grew beyond the selected 42 MiB profile".into());
    }
    Ok(bytes)
}

fn hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes { out.push(DIGITS[(byte >> 4) as usize] as char); out.push(DIGITS[(byte & 15) as usize] as char); }
    out
}

fn write_new(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let mut file = OpenOptions::new().write(true).create_new(true).open(path)
        .map_err(|e| format!("create-only output {}: {e}", path.display()))?;
    file.write_all(bytes).map_err(|e| e.to_string())?;
    file.sync_all().map_err(|e| e.to_string())
}

pub fn run(args: &[String]) -> Result<(), String> {
    if args.len() != 4 || args[0] != "--asset" || args[2] != "--out" {
        return Err("Usage: synth preview --asset FILE.10d --out NEW_DIR".into());
    }
    let requested_asset_path = PathBuf::from(&args[1]);
    if requested_asset_path.extension().and_then(|s| s.to_str()) != Some("10d") {
        return Err("--asset must use the `.10d` extension".into());
    }
    let bytes = read_asset(&requested_asset_path)?;
    let asset_path = requested_asset_path.canonicalize().map_err(|e| format!("asset path: {e}"))?;
    let source_sha256 = hex(&Sha256::digest(&bytes));
    let mut renderer = VolumetricRenderer::new_offscreen(PREVIEW_SIDE, PREVIEW_SIDE, 256)
        .map_err(|e| format!("native offscreen renderer: {e}"))?;
    let (geometry, views) = renderer.render_10d_review_pngs(&bytes, PREVIEW_SIDE)
        .map_err(|e| format!("native `.10d` review render: {e}"))?;
    let total = views.iter().try_fold(0usize, |sum, (_, png)| sum.checked_add(png.len()))
        .ok_or("preview output size overflow")?;
    if total > MAX_TOTAL_OUTPUT_BYTES { return Err("preview PNG set exceeds 64 MiB".into()); }
    let output_dir = PathBuf::from(&args[3]);
    if fs::symlink_metadata(&output_dir).is_ok() { return Err("--out must name a new directory".into()); }
    let parent = output_dir.parent().filter(|p| !p.as_os_str().is_empty()).unwrap_or(Path::new("."))
        .canonicalize().map_err(|_| "--out parent must already exist")?;
    let name = output_dir.file_name().ok_or("--out needs a directory name")?;
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
        "schema": "ecopunk.native-10d-preview/v1",
        "scope": "native readback PNG projection; not model acceptance or an adapter capability receipt",
        "source": {"path": asset_path, "sha256": source_sha256, "bytes": bytes.len()},
        "renderer": "QualiaDB webizen-render::VolumetricRenderer / PortalGpu native offscreen path",
        "projection": geometry,
        "views": records,
    });
    let receipt_bytes = serde_json::to_vec_pretty(&receipt).map_err(|e| e.to_string())?;
    if total.checked_add(receipt_bytes.len()).ok_or("output size overflow")? > MAX_TOTAL_OUTPUT_BYTES {
        return Err("preview package exceeds 64 MiB".into());
    }
    fs::create_dir(&destination).map_err(|e| format!("create-only preview directory: {e}"))?;
    for (view, png) in views {
        write_new(&destination.join(format!("{}.png", view.name)), &png)?;
    }
    write_new(&destination.join("review-receipt.json"), &receipt_bytes)?;
    println!("Native review preview written: {}", destination.display());
    Ok(())
}
