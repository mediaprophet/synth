//! Browser wasm-lite bridge: OPFS compose + native HMC ingest over the studio HTTP API.
//!
//! `GET /synth/wasm-lite.js` posts PNG layers to `POST /api/radiance/compose` and
//! optional OPFS `synth-compose.png`. `POST /api/hmc/bridge` packs in-memory PNGs
//! through `ProducerRegistry` into an HMC candidate.

use crate::production::{ProducerRegistry, ProductionLimits, Resource};
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::atomic::AtomicBool;

pub const WASM_LITE_JS: &str = r#"
function b64ToU8(b64) {
  const bin = atob(b64);
  const out = new Uint8Array(bin.length);
  for (let i = 0; i < bin.length; i++) out[i] = bin.charCodeAt(i);
  return out;
}
async function composeToOpfs(layers) {
  const body = (layers && typeof layers === 'object' && !Array.isArray(layers) && ('layers' in layers || 'width' in layers))
    ? layers
    : { width: 256, height: 256, layers: layers };
  const res = await fetch('/api/radiance/compose', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(body)
  });
  const ctype = (res.headers.get('content-type') || '').toLowerCase();
  if (ctype.indexOf('image/png') === -1) {
    let data = {};
    try { data = await res.json(); } catch (e) { data = { error: res.statusText }; }
    throw new Error((data && data.error) || 'compose failed');
  }
  const png = new Uint8Array(await res.arrayBuffer());
  if (navigator.storage && typeof navigator.storage.getDirectory === 'function') {
    const root = await navigator.storage.getDirectory();
    const handle = await root.getFileHandle('synth-compose.png', { create: true });
    const writable = await handle.createWritable();
    await writable.write(png);
    await writable.close();
  }
  return png;
}
async function ingestHmc(files) {
  const body = Array.isArray(files) ? { files: files } : files;
  const res = await fetch('/api/hmc/bridge', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(body)
  });
  return res.json();
}
async function opfsReadPng() {
  if (!navigator.storage || typeof navigator.storage.getDirectory !== 'function') {
    throw new Error('OPFS unavailable');
  }
  const root = await navigator.storage.getDirectory();
  const handle = await root.getFileHandle('synth-compose.png');
  const file = await handle.getFile();
  return new Uint8Array(await file.arrayBuffer());
}
window.SynthWasmLite = { composeToOpfs: composeToOpfs, ingestHmc: ingestHmc, opfsReadPng: opfsReadPng };
"#;

pub const MAX_COMPOSE_DIM: u32 = 1024;
pub const MAX_COMPOSE_LAYERS: usize = 8;
pub const MAX_HMC_FILES: usize = 16;
pub const MAX_BODY_BYTES: usize = 16 * 1024 * 1024;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ComposeRequest {
    width: u32,
    height: u32,
    layers: Vec<ComposeLayer>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ComposeLayer {
    png_base64: String,
    #[serde(default = "default_opacity")]
    opacity: f64,
    #[serde(default = "default_blend")]
    blend: String,
}

fn default_opacity() -> f64 {
    1.0
}

fn default_blend() -> String {
    "src_over".into()
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct HmcBridgeRequest {
    files: Vec<HmcFile>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct HmcFile {
    name: String,
    png_base64: String,
}

fn b64_val(c: u8) -> Result<u8, String> {
    match c {
        b'A'..=b'Z' => Ok(c - b'A'),
        b'a'..=b'z' => Ok(c - b'a' + 26),
        b'0'..=b'9' => Ok(c - b'0' + 52),
        b'+' | b'-' => Ok(62),
        b'/' | b'_' => Ok(63),
        _ => Err("Invalid base64".into()),
    }
}

pub fn b64_decode(s: &str) -> Result<Vec<u8>, String> {
    let filtered: Vec<u8> = s
        .bytes()
        .filter(|b| !b.is_ascii_whitespace())
        .collect();
    if filtered.is_empty() {
        return Err("Empty base64".into());
    }
    if filtered.len() % 4 != 0 {
        return Err("Invalid base64 length".into());
    }
    let mut out = Vec::with_capacity(filtered.len() / 4 * 3);
    for chunk in filtered.chunks_exact(4) {
        let pad2 = chunk[2] == b'=';
        let pad3 = chunk[3] == b'=';
        if pad2 && !pad3 {
            return Err("Invalid base64 padding".into());
        }
        if pad2 && chunk[2] != b'=' {
            return Err("Invalid base64 padding".into());
        }
        let a = b64_val(chunk[0])?;
        let b = b64_val(chunk[1])?;
        let c = if pad2 { 0 } else { b64_val(chunk[2])? };
        let d = if pad3 { 0 } else { b64_val(chunk[3])? };
        let triple = ((a as u32) << 18) | ((b as u32) << 12) | ((c as u32) << 6) | (d as u32);
        out.push((triple >> 16) as u8);
        if !pad2 {
            out.push((triple >> 8) as u8);
        }
        if !pad3 {
            out.push(triple as u8);
        }
    }
    Ok(out)
}

fn blend_to_mode(blend: &str) -> Result<&'static str, String> {
    Ok(match blend.to_ascii_lowercase().as_str() {
        "src_over" | "src-over" | "normal" => "normal",
        "multiply" => "multiply",
        "screen" => "screen",
        "overlay" => "overlay",
        "soft-light" | "soft_light" => "soft-light",
        "hard-light" | "hard_light" => "hard-light",
        "dodge" | "color-dodge" | "color_dodge" => "dodge",
        "burn" | "color-burn" | "color_burn" => "burn",
        "difference" => "difference",
        "exclusion" => "exclusion",
        other => return Err(format!("Unknown blend mode: {other}")),
    })
}

fn vibe_f64(n: f64) -> String {
    if n.fract() == 0.0 {
        format!("{n:.1}")
    } else {
        format!("{n}")
    }
}

fn vibe_quoted(s: &str) -> Result<String, String> {
    if s.bytes().any(|b| b < 0x20 || b == 0x7f) {
        return Err("Invalid name".into());
    }
    Ok(format!(
        "\"{}\"",
        s.replace('\\', "\\\\").replace('"', "\\\"")
    ))
}

/// Decode PNG layers, invoke `import_rgba_png` + `compose_rgba` + `encode_rgba_png`.
pub fn compose_png(body: &[u8]) -> Result<Vec<u8>, String> {
    if body.len() > MAX_BODY_BYTES {
        return Err("Request body exceeds 16MB".into());
    }
    let req: ComposeRequest =
        serde_json::from_slice(body).map_err(|_| "Invalid JSON body for /api/radiance/compose")?;
    if req.layers.is_empty() || req.layers.len() > MAX_COMPOSE_LAYERS {
        return Err("layers must contain 1..=8 entries".into());
    }
    if req.width == 0
        || req.height == 0
        || req.width > MAX_COMPOSE_DIM
        || req.height > MAX_COMPOSE_DIM
        || (req.width as u64) * (req.height as u64) > (MAX_COMPOSE_DIM as u64) * (MAX_COMPOSE_DIM as u64)
    {
        return Err("width/height must be 1..=1024 and at most 1024² pixels".into());
    }
    let mut inputs = Vec::with_capacity(req.layers.len());
    let mut layer_meta = Vec::with_capacity(req.layers.len());
    for (i, layer) in req.layers.iter().enumerate() {
        if !(0.0..=1.0).contains(&layer.opacity) {
            return Err("opacity outside 0..1".into());
        }
        let mode = blend_to_mode(&layer.blend)?;
        let bytes = b64_decode(&layer.png_base64)?;
        if bytes.len() > ProductionLimits::default().resource_bytes {
            return Err("Layer PNG exceeds resource byte limit".into());
        }
        let key = format!("layer-{i}.png");
        inputs.push(Resource {
            key: key.clone(),
            kind: "png".into(),
            bytes,
        });
        layer_meta.push((key, layer.opacity, mode));
    }
    let mut source = String::from("requires [capability(\"capability.invoke\")];\n");
    for (i, (key, _, _)) in layer_meta.iter().enumerate() {
        source.push_str(&format!(
            "let l{i} = capability.invoke(\"hypermedia.radiance.import_rgba_png\", {{ input: \"{key}\", profile: \"1024\" }});\n"
        ));
    }
    source.push_str(&format!(
        "let comp = capability.invoke(\"hypermedia.radiance.compose_rgba\", {{ width: {}, height: {}, profile: \"1024\", layers: [",
        req.width, req.height
    ));
    for (i, (_, opacity, mode)) in layer_meta.iter().enumerate() {
        if i > 0 {
            source.push(',');
        }
        source.push_str(&format!(
            "{{ image: l{i}, opacity: {}, mode: \"{mode}\" }}",
            vibe_f64(*opacity)
        ));
    }
    source.push_str(
        "] });\nlet png = capability.invoke(\"hypermedia.radiance.encode_rgba_png\", { image: comp, profile: \"1024\" });\nlet asset = { files: [{ key: \"compose.png\", resource: png }] };\n",
    );
    let registry = ProducerRegistry::with_builtins();
    let cancel = AtomicBool::new(false);
    let mut session = registry.session("files", inputs, ProductionLimits::default())?;
    session.submit(&source, &cancel)?;
    let candidate = session.candidate().ok_or("No candidate produced")?;
    candidate
        .outputs()
        .iter()
        .find(|r| r.key == "compose.png")
        .map(|r| r.bytes.clone())
        .ok_or_else(|| "compose.png missing from candidate".into())
}

/// Pack in-memory PNG members through the files producer into an HMC candidate.
pub fn bridge_hmc(body: &[u8]) -> Value {
    match bridge_hmc_inner(body) {
        Ok(v) => v,
        Err(error) => json!({ "ok": false, "error": error }),
    }
}

fn bridge_hmc_inner(body: &[u8]) -> Result<Value, String> {
    if body.len() > MAX_BODY_BYTES {
        return Err("Request body exceeds 16MB".into());
    }
    let req: HmcBridgeRequest =
        serde_json::from_slice(body).map_err(|_| "Invalid JSON body for /api/hmc/bridge")?;
    if req.files.is_empty() || req.files.len() > MAX_HMC_FILES {
        return Err("files must contain 1..=16 entries".into());
    }
    let mut inputs = Vec::with_capacity(req.files.len());
    let mut keys = Vec::with_capacity(req.files.len());
    for file in &req.files {
        let name = file.name.trim();
        if name.is_empty() {
            return Err("File name is empty".into());
        }
        let key = if name.to_ascii_lowercase().ends_with(".png") {
            name.to_string()
        } else {
            format!("{name}.png")
        };
        let _ = vibe_quoted(&key)?;
        let bytes = b64_decode(&file.png_base64)?;
        if bytes.len() > ProductionLimits::default().resource_bytes {
            return Err("PNG exceeds resource byte limit".into());
        }
        inputs.push(Resource {
            key: key.clone(),
            kind: "png".into(),
            bytes,
        });
        keys.push(key);
    }
    let mut source = String::from("requires [capability(\"capability.invoke\")];\n");
    for (i, key) in keys.iter().enumerate() {
        let quoted = vibe_quoted(key)?;
        source.push_str(&format!(
            "let _chk{i} = capability.invoke(\"hypermedia.radiance.import_rgba_png\", {{ input: {quoted}, profile: \"1024\" }});\n"
        ));
    }
    source.push_str("let asset = { files: [");
    for (i, key) in keys.iter().enumerate() {
        if i > 0 {
            source.push(',');
        }
        let quoted = vibe_quoted(key)?;
        source.push_str(&format!("{{ key: {quoted}, kind: \"png\", input: {quoted} }}"));
    }
    source.push_str("] };\n");
    let registry = ProducerRegistry::with_builtins();
    let cancel = AtomicBool::new(false);
    let mut session = registry.session("files", inputs, ProductionLimits::default())?;
    session.submit(&source, &cancel)?;
    let candidate = session.candidate().ok_or("No candidate produced")?;
    let doc_digest = session
        .document_digest()
        .map(|d| d.iter().map(|b| format!("{b:02x}")).collect::<String>())
        .unwrap_or_default();
    let resources: Vec<Value> = candidate
        .outputs()
        .iter()
        .map(|r| {
            json!({
                "key": r.key,
                "kind": r.kind,
                "bytes": r.bytes.len(),
            })
        })
        .collect();
    Ok(json!({
        "ok": true,
        "hmc_sha256": candidate.sha256(),
        "package_sha256": candidate.sha256(),
        "document_digest": doc_digest,
        "resource_count": candidate.outputs().len(),
        "resources": resources,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wasm_lite_js_exports_compose_and_opfs() {
        assert!(WASM_LITE_JS.contains("composeToOpfs"));
        assert!(WASM_LITE_JS.contains("getDirectory"));
        assert!(WASM_LITE_JS.contains("ingestHmc"));
        assert!(WASM_LITE_JS.contains("opfsReadPng"));
        assert!(WASM_LITE_JS.contains("SynthWasmLite"));
        assert!(
            !WASM_LITE_JS.contains("export {"),
            "classic <script src> cannot parse module export"
        );
    }

    #[test]
    fn compose_rejects_invalid_json() {
        let err = compose_png(b"not-json").unwrap_err();
        assert!(err.to_ascii_lowercase().contains("invalid json"));
    }

    #[test]
    fn compose_rejects_empty_layers() {
        let err = compose_png(br#"{"width":8,"height":8,"layers":[]}"#).unwrap_err();
        assert!(err.contains("1..=8"));
    }

    #[test]
    fn compose_rejects_unknown_fields() {
        let err = compose_png(br#"{"width":8,"height":8,"layers":[],"extra":true}"#).unwrap_err();
        assert!(err.to_ascii_lowercase().contains("invalid json"));
    }

    #[test]
    fn hmc_bridge_empty_files_is_error() {
        let v = bridge_hmc(br#"{"files":[]}"#);
        assert_eq!(v["ok"], false);
        let err = v["error"].as_str().unwrap_or("");
        assert!(err.contains("1..=16"), "got {err}");
    }
}
