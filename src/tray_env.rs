//! Self-hosted Synth Tray & Studio environment produced by Synth itself.
//!
//! When Webizen Desktop (`127.0.0.1:8080` / `ws://127.0.0.1:4242`) is not running,
//! Synth runs locally as a resident Webizen AI Synth tray & studio host, using its own
//! VibeScript asset production pipeline (`examples/tray_environment.vibe`) to
//! construct its tray icons, vector emblem, radiance banner, 10D spatial beacon,
//! and acoustic readiness cues.

use crate::production::{ProducerRegistry, ProductionLimits, Resource};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    fs,
    net::{SocketAddr, TcpStream},
    path::{Path, PathBuf},
    sync::atomic::AtomicBool,
    time::Duration,
};

pub const TRAY_ENVIRONMENT_SOURCE: &str = include_str!("../examples/tray_environment.vibe");
pub const SWAMP_WALLABY_SOURCE: &str = include_str!("../examples/swamp_wallaby.vibe");

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum HostRuntimeMode {
    StandaloneTray,
    AttachedToWebizenDesktop,
}

impl HostRuntimeMode {
    pub fn label(self) -> &'static str {
        match self {
            Self::StandaloneTray => "Webizen AI Synth tray",
            Self::AttachedToWebizenDesktop => "Attached to Webizen Desktop",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiscoveredModel {
    pub name: String,
    pub family: String,
    pub format: String,
    pub path: String,
    pub size_bytes: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EcosystemStatus {
    pub total_mcp_tools: usize,
    pub vibe_sdk_families: usize,
    pub living_world_entries: usize,
    pub celestial_reference: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProducedEnvironment {
    pub schema: String,
    pub mode: HostRuntimeMode,
    pub webizen_http_online: bool,
    pub webizen_ws_online: bool,
    pub vault_root: String,
    pub candidate_digest_hex: String,
    pub document_digest_hex: String,
    pub resources: Vec<EnvironmentResourceSummary>,
    pub emblem_svg_data_uri: String,
    pub banner_png_data_uri: String,
    pub active_tray_icon_data_uri: String,
    pub ready_wav_data_uri: String,
    pub showcase_html: String,
    pub discovered_models: Vec<DiscoveredModel>,
    pub ecosystem: EcosystemStatus,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnvironmentResourceSummary {
    pub key: String,
    pub kind: String,
    pub bytes: usize,
    pub sha256: String,
}

fn hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        out.push(DIGITS[(b >> 4) as usize] as char);
        out.push(DIGITS[(b & 15) as usize] as char);
    }
    out
}

fn base64_encode(bytes: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = chunk.get(1).copied().unwrap_or(0) as u32;
        let b2 = chunk.get(2).copied().unwrap_or(0) as u32;
        let triple = (b0 << 16) | (b1 << 8) | b2;
        out.push(TABLE[((triple >> 18) & 0x3f) as usize] as char);
        out.push(TABLE[((triple >> 12) & 0x3f) as usize] as char);
        if chunk.len() > 1 {
            out.push(TABLE[((triple >> 6) & 0x3f) as usize] as char);
        } else {
            out.push('=');
        }
        if chunk.len() > 2 {
            out.push(TABLE[(triple & 0x3f) as usize] as char);
        } else {
            out.push('=');
        }
    }
    out
}

/// Probes whether Webizen Desktop (`127.0.0.1:8080` HTTP daemon or `127.0.0.1:4242` native WS)
/// is currently listening on localhost.
pub fn probe_webizen_desktop() -> (HostRuntimeMode, bool, bool) {
    let timeout = Duration::from_millis(80);
    let http_addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    let ws_addr = SocketAddr::from(([127, 0, 0, 1], 4242));
    let http_online = TcpStream::connect_timeout(&http_addr, timeout).is_ok();
    let ws_online = TcpStream::connect_timeout(&ws_addr, timeout).is_ok();
    let mode = if http_online || ws_online {
        HostRuntimeMode::AttachedToWebizenDesktop
    } else {
        HostRuntimeMode::StandaloneTray
    };
    (mode, http_online, ws_online)
}

/// Resolves the active local LLM model vault root, prioritizing `QUALIA_LLM_VAULT`,
/// then `C:\LLM_files`, then `C:\LLM_Models`.
pub fn resolve_vault_root() -> PathBuf {
    if let Ok(val) = std::env::var("QUALIA_LLM_VAULT") {
        let trimmed = val.trim();
        if !trimmed.is_empty() {
            return PathBuf::from(trimmed);
        }
    }
    let llm_files = PathBuf::from(r"C:\LLM_files");
    if llm_files.exists() {
        return llm_files;
    }
    let llm_models = PathBuf::from(r"C:\LLM_Models");
    if llm_models.exists() {
        return llm_models;
    }
    llm_files
}

fn classify_model_family(name: &str, path: &str) -> &'static str {
    let lower = format!("{name} {path}").to_ascii_lowercase();
    if lower.contains("qwen") {
        "qwen"
    } else if lower.contains("gemma") {
        "gemma4"
    } else if lower.contains("expert") || lower.contains("q64") {
        "expert"
    } else {
        "general"
    }
}

fn scan_vault_dir(dir: &Path, depth_remaining: usize, max: usize, out: &mut Vec<DiscoveredModel>) {
    if out.len() >= max || !dir.is_dir() {
        return;
    }
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        if out.len() >= max {
            break;
        }
        let path = entry.path();
        if path.is_dir() {
            if depth_remaining > 0 {
                scan_vault_dir(&path, depth_remaining - 1, max, out);
            }
            continue;
        }
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        let format = match ext.as_str() {
            "p64" => "p64",
            "q64" => "q64",
            "gguf" => "gguf",
            "safetensors" => "safetensors",
            _ => continue,
        };
        let size_bytes = entry.metadata().map(|m| m.len()).unwrap_or(0);
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("model")
            .to_owned();
        let path_str = path.to_string_lossy().into_owned();
        let family = classify_model_family(&name, &path_str).to_owned();
        out.push(DiscoveredModel {
            name,
            family,
            format: format.to_owned(),
            path: path_str,
            size_bytes,
        });
    }
}

/// Scans `C:\LLM_files` (and any configured `QUALIA_LLM_VAULT` / `C:\LLM_Models`)
/// up to 3 subfolder levels deep for `.p64`, `.q64`, `.gguf`, and `.safetensors` models.
pub fn discover_local_models() -> (PathBuf, Vec<DiscoveredModel>) {
    const MAX_MODELS: usize = 64;
    let primary = resolve_vault_root();
    let mut models = Vec::new();
    scan_vault_dir(&primary, 3, MAX_MODELS, &mut models);
    let fallback = PathBuf::from(r"C:\LLM_Models");
    if fallback != primary && fallback.is_dir() {
        scan_vault_dir(&fallback, 3, MAX_MODELS, &mut models);
    }
    models.sort_by(|a, b| a.name.cmp(&b.name));
    (primary, models)
}

/// Queries the integrated QualiaDB 0.0.41 MCP tools (`etalon_vibe_sdk`,
/// `living_world_anatomy`, `cosmic_coordinates`) to verify live environment readiness.
pub fn query_ecosystem_summary() -> EcosystemStatus {
    use qualia_mcp::authoring::AuthoringBackend;
    let mut mcp = crate::mcp_full::StandaloneSynthMcp::default();
    let total_mcp_tools = mcp.tools().len();

    let sdk_families = mcp
        .call("etalon_vibe_sdk", json!({}))
        .ok()
        .and_then(|res| {
            res["provider_count"]
                .as_u64()
                .map(|n| n as usize)
                .or_else(|| res["providers"].as_array().map(Vec::len))
        })
        .unwrap_or(0);

    let living_entries = mcp
        .call("living_world_anatomy", json!({ "op": "catalog" }))
        .ok()
        .and_then(|res| {
            res["pin_count"]
                .as_u64()
                .map(|n| n as usize)
                .or_else(|| res["pins"].as_array().map(Vec::len))
        })
        .unwrap_or(0);

    let celestial_reference = mcp
        .call(
            "cosmic_coordinates",
            json!({ "op": "celestial_profile", "body": "earth" }),
        )
        .ok()
        .and_then(|res| res["body"].as_str().map(str::to_owned))
        .unwrap_or_else(|| "earth".to_owned());

    EcosystemStatus {
        total_mcp_tools,
        vibe_sdk_families: sdk_families,
        living_world_entries: living_entries,
        celestial_reference,
    }
}

fn find_resource<'a>(resources: &'a [Resource], key: &str) -> Result<&'a Resource, String> {
    resources
        .iter()
        .find(|r| r.key == key)
        .ok_or_else(|| format!("Missing environment resource: {key}"))
}

/// Runs `synth` on `examples/tray_environment.vibe` to bootstrap its own visual,
/// spatial, and acoustic environment, returning the verified `ProducedEnvironment`.
pub fn bootstrap_environment() -> Result<ProducedEnvironment, String> {
    let registry = ProducerRegistry::with_builtins();
    let cancel = AtomicBool::new(false);
    let mut limits = ProductionLimits::default();
    limits.resources = 512;
    limits.total_resource_bytes = 128 * 1024 * 1024;
    limits.package_bytes = 160 * 1024 * 1024;
    let mut session = registry.session("files", vec![], limits)?;
    session.submit(TRAY_ENVIRONMENT_SOURCE, &cancel)?;

    let candidate = session
        .candidate()
        .ok_or("Environment candidate was not produced")?;
    let outputs = candidate.outputs();
    let doc_digest = session.document_digest()?;
    let candidate_digest = candidate.sha256();

    let (mode, webizen_http_online, webizen_ws_online) = probe_webizen_desktop();
    let (vault_path, discovered_models) = discover_local_models();
    let ecosystem = query_ecosystem_summary();

    let emblem = find_resource(outputs, "studio/emblem.svg")?;
    let banner = find_resource(outputs, "studio/banner.png")?;
    let standalone_icon = find_resource(outputs, "tray/icon_standalone.png")?;
    let attached_icon = find_resource(outputs, "tray/icon_attached.png")?;
    let ready_wav = find_resource(outputs, "audio/tray_ready.wav")?;
    let _complete_wav = find_resource(outputs, "audio/job_complete.wav")?;
    let _beacon = find_resource(outputs, "studio/beacon.10d")?;
    let _manifest = find_resource(outputs, "environment/manifest.json")?;

    let active_icon = match mode {
        HostRuntimeMode::StandaloneTray => standalone_icon,
        HostRuntimeMode::AttachedToWebizenDesktop => attached_icon,
    };

    let summaries = outputs
        .iter()
        .map(|r| EnvironmentResourceSummary {
            key: r.key.clone(),
            kind: r.kind.clone(),
            bytes: r.bytes.len(),
            sha256: hex(&Sha256::digest(&r.bytes)),
        })
        .collect();

    let showcase_html = crate::present::render_session_html(&session)?;

    Ok(ProducedEnvironment {
        schema: "synth.tray-environment/v1".to_owned(),
        mode,
        webizen_http_online,
        webizen_ws_online,
        vault_root: vault_path.to_string_lossy().into_owned(),
        candidate_digest_hex: candidate_digest,
        document_digest_hex: hex(&doc_digest),
        resources: summaries,
        emblem_svg_data_uri: format!(
            "data:image/svg+xml;base64,{}",
            base64_encode(&emblem.bytes)
        ),
        banner_png_data_uri: format!("data:image/png;base64,{}", base64_encode(&banner.bytes)),
        active_tray_icon_data_uri: format!(
            "data:image/png;base64,{}",
            base64_encode(&active_icon.bytes)
        ),
        ready_wav_data_uri: format!("data:audio/wav;base64,{}", base64_encode(&ready_wav.bytes)),
        showcase_html,
        discovered_models,
        ecosystem,
    })
}

/// JSON summary of the self-produced Synth Tray & Studio environment (for CLI/MCP inspection).
pub fn environment_report_json() -> Result<Value, String> {
    let env = bootstrap_environment()?;
    Ok(json!({
        "schema": env.schema,
        "mode": env.mode.label(),
        "webizen_http_online": env.webizen_http_online,
        "webizen_ws_online": env.webizen_ws_online,
        "vault_root": env.vault_root,
        "candidate_digest_sha256": env.candidate_digest_hex,
        "document_digest_sha256": env.document_digest_hex,
        "resources": env.resources,
        "discovered_models": env.discovered_models,
        "ecosystem": env.ecosystem,
    }))
}
