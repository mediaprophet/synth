//! Google Colab Ultra Heavy-Compute Burst Extension for Webizen AI Synth (`synth_colab` / `synth colab`).
//!
//! Offloads heavy 24 GB–80 GB VRAM / 83 GB High-RAM routines (such as `microsoft/TRELLIS.2-4B`
//! image/multiview-to-3D O-Voxel + 4K PBR generation and massive OWL/RDF/BioPortal -> `.q42`
//! refactoring & volume compilation) to an authenticated Google Colab Ultra worker over:
//!   - Mode A (`transport = "http"`): HTTPS tunnel + Bearer token (`colab-ultra` bookmark/secret)
//!   - Mode B (`transport = "drive_queue"`): Local Google Drive sync folder queue with atomic
//!     `.tmp` -> `receipt.json` publishing.
//!
//! Upon ingestion, all canonical `.10d` v2 (`HOLO` `0x0306` + `VOX0` + `CBLD` + `PROV`),
//! `source-asset-licensing.n3`, `.synth/changelog.json`, and `.q42` volume verification are
//! executed strictly inside Synth's native Rust authority.

use super::*;
use crate::changelog_licensing::{
    HarnessEnvironmentRecord, HumanActorRecord, RevisionEconomicsTelemetry, SoftwareAgentRecord,
};
use crate::model_validation::{convert_3d_with_provenance, Convert3dProvenanceOverrides};
use crate::operator::bookmarks;
use qualia_mcp::authoring::ToolError;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub const COLAB_BOOKMARK_ID: &str = "colab-ultra";
pub const COLAB_TOKEN_ENV: &str = "SYNTH_COLAB_TOKEN";
pub const COLAB_JOB_REQUEST_SCHEMA: &str = "synth.colab-job-request/v1";
pub const COLAB_JOB_RECEIPT_SCHEMA: &str = "synth.colab-job-receipt/v1";
pub const COLAB_CONFIG_FILE: &str = "colab-config.json";

pub fn colab_arg_schema_properties() -> Value {
    let raw = r#"{
        "action": {"type":"string","enum":["status","configure","routines","submit","poll","ingest","list_jobs","cancel","export_notebook","colab_mcp_cells"]},
        "routine": {"type":"string","enum":["trellis2_image_to_3d","trellis2_multiview_to_3d","hunyuan3d_shape_and_pbr","partcrafter_decompose_3d","unirig_auto_skeleton","ontology_compile_q42"]},
        "transport": {"type":"string","enum":["auto","http","drive_queue"]},
        "endpoint_url": {"type":"string","maxLength":2048,"description":"HTTPS base URL for Colab worker (or http://127.0.0.1 for local tests)"},
        "api_token": {"type":"string","maxLength":512,"description":"Only with action=configure; saved to gitignored .synth/secrets.json"},
        "drive_queue_dir": {"type":"string","maxLength":4096,"description":"Local Google Drive sync directory (e.g. G:\\My Drive\\Synth-Colab-Ultra)"},
        "idle_timeout_minutes": {"type":"integer","minimum":1,"maximum":1440,"description":"Colab runtime.unassign() idle watchdog timeout in minutes (default 30)"},
        "job_id": {"type":"string","maxLength":96},
        "wait_secs": {"type":"integer","minimum":0,"maximum":3600},
        "image_path": {"type":"string","maxLength":4096,"description":"Reference image under SYNTH_INPUT_ROOT for trellis2_image_to_3d, hunyuan3d_shape_and_pbr, or partcrafter_decompose_3d"},
        "image_paths": {"type":"array","minItems":2,"maxItems":4,"items":{"type":"string","maxLength":4096},"description":"2..4 reference images under SYNTH_INPUT_ROOT for trellis2_multiview_to_3d"},
        "mesh_path": {"type":"string","maxLength":4096,"description":"Input 3D mesh (.glb/.obj) under SYNTH_INPUT_ROOT for hunyuan3d_shape_and_pbr (texture-only mode), partcrafter_decompose_3d, or unirig_auto_skeleton"},
        "mode": {"type":"string","enum":["shape_and_pbr","pbr_only","decompose","rig"],"description":"Sub-mode for hunyuan3d_shape_and_pbr (shape_and_pbr | pbr_only), partcrafter_decompose_3d, or unirig_auto_skeleton"},
        "target_parts": {"type":"integer","minimum":2,"maximum":64,"description":"Target articulated/semantic part count for partcrafter_decompose_3d"},
        "max_joints": {"type":"integer","minimum":2,"maximum":256,"description":"Maximum skeletal joints for unirig_auto_skeleton (produces SKL1/SKL2 + KLOD0..KLOD3)"},
        "resolution": {"type":"integer","enum":[512,1024,1536]},
        "seed": {"type":"integer","minimum":0},
        "decimation_target": {"type":"integer","minimum":4000,"maximum":350000},
        "texture_size": {"type":"integer","enum":[1024,2048,4096]},
        "remesh": {"type":"boolean"},
        "export_ovoxel_summary": {"type":"boolean"},
        "auto_convert_10d": {"type":"boolean"},
        "out_dir": {"type":"string","maxLength":4096},
        "slug": {"type":"string","maxLength":96,"description":"Optional target slug under poiesis/<slug> or eco-intent/<slug>"},
        "asset_uri": {"type":"string","maxLength":1024},
        "domain": {"type":"string","maxLength":64},
        "domain_id": {"type":"integer","minimum":1},
        "taxon": {"type":"string","maxLength":128},
        "fiction": {"type":"boolean"},
        "evidence_q42": {"type":"string","maxLength":4096},
        "spdx_license": {"type":"string","maxLength":128},
        "creator": {"type":"string","maxLength":512},
        "attribution_notice": {"type":"string","maxLength":2048},
        "origin_uri": {"type":"string","maxLength":2048},
        "embed_full_source_bytes": {"type":"boolean"},
        "acronym": {"type":"string","maxLength":64},
        "source_url": {"type":"string","maxLength":2048},
        "source_path": {"type":"string","maxLength":4096},
        "download_format": {"type":"string","enum":["rdf","owl","ttl","n3","nt","csv"]},
        "context_iri": {"type":"string","maxLength":1024},
        "out_q42": {"type":"string","maxLength":96,"pattern":"^[A-Za-z0-9_-][A-Za-z0-9._-]*\\.q42$"},
        "refactor_owl": {"type":"boolean"},
        "ingest_to_graph": {"type":"boolean"},
        "segment_mib": {"type":"integer","minimum":1},
        "notebook_out_path": {"type":"string","maxLength":4096}
    }"#;
    serde_json::from_str(raw).unwrap_or_else(|_| json!({}))
}

/// Maximum input image size (32 MiB).
const MAX_IMAGE_INPUT_BYTES: usize = 32 * 1024 * 1024;
/// Maximum remote artifact download size (512 MiB per artifact).
const MAX_ARTIFACT_BYTES: u64 = 512 * 1024 * 1024;
/// Default decimation triangle budget for `.glb` export (stays within `production16` 200k/400k mesh cap).
const DEFAULT_DECIMATION_TRIS: u32 = 120_000;
const MIN_DECIMATION_TRIS: u32 = 4_000;
const MAX_DECIMATION_TRIS: u32 = 350_000;

const EMBEDDED_COLAB_WORKER_PY: &str = include_str!("../../vendor/colab/colab_worker.py");
const EMBEDDED_COLAB_NOTEBOOK_IPYNB: &str =
    include_str!("../../vendor/colab/synth_colab_ultra_worker.ipynb");

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default, deny_unknown_fields)]
pub struct ColabArg {
    /// "status" | "configure" | "routines" | "submit" | "poll" | "ingest" | "list_jobs" | "cancel" | "export_notebook"
    pub action: Option<String>,
    /// Routine ID: "trellis2_image_to_3d" | "trellis2_multiview_to_3d" | "ontology_compile_q42"
    pub routine: Option<String>,
    /// Transport mode: "auto" | "http" | "drive_queue"
    pub transport: Option<String>,
    /// Remote Colab worker HTTPS base URL (stored in .synth/bookmarks.json id="colab-ultra")
    pub endpoint_url: Option<String>,
    /// Bearer API token (stored in gitignored .synth/secrets.json id="colab-ultra")
    pub api_token: Option<String>,
    /// Local Google Drive sync root directory (e.g. "G:\\My Drive\\Synth-Colab-Ultra")
    pub drive_queue_dir: Option<String>,
    /// Idle auto-disconnect timeout in minutes on Colab Ultra (default: 30)
    pub idle_timeout_minutes: Option<u32>,

    // Job identification
    pub job_id: Option<String>,
    pub wait_secs: Option<u64>,

    // Routine 1..5 (3D Generation, PBR Retexturing, Multi-Part Decomposition, Auto-Rigging) arguments
    pub image_path: Option<String>,
    pub image_paths: Option<Vec<String>>,
    pub mesh_path: Option<String>,
    pub mode: Option<String>,
    pub target_parts: Option<u32>,
    pub max_joints: Option<u32>,
    pub resolution: Option<u32>,
    pub seed: Option<u64>,
    pub decimation_target: Option<u32>,
    pub texture_size: Option<u32>,
    pub remesh: Option<bool>,
    pub export_ovoxel_summary: Option<bool>,

    // Automatic local synth_convert_3d & domain handoff
    pub auto_convert_10d: Option<bool>,
    pub out_dir: Option<String>,
    pub slug: Option<String>,
    pub asset_uri: Option<String>,
    pub domain: Option<String>,
    pub domain_id: Option<u32>,
    pub taxon: Option<String>,
    pub fiction: Option<bool>,
    pub evidence_q42: Option<String>,
    pub spdx_license: Option<String>,
    pub creator: Option<String>,
    pub attribution_notice: Option<String>,
    pub origin_uri: Option<String>,
    pub embed_full_source_bytes: Option<bool>,

    // Routine 3 (Ontology / Q42) arguments
    pub acronym: Option<String>,
    pub source_url: Option<String>,
    pub source_path: Option<String>,
    pub download_format: Option<String>,
    pub context_iri: Option<String>,
    pub out_q42: Option<String>,
    pub refactor_owl: Option<bool>,
    pub ingest_to_graph: Option<bool>,
    pub segment_mib: Option<u64>,

    // Notebook export path override
    pub notebook_out_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColabConfig {
    pub schema: String,
    pub transport: String,
    pub endpoint_url: Option<String>,
    pub drive_queue_dir: Option<String>,
    pub idle_timeout_minutes: u32,
    pub updated_at_epoch_s: u64,
}

impl Default for ColabConfig {
    fn default() -> Self {
        Self {
            schema: "synth.colab-config/v1".into(),
            transport: "auto".into(),
            endpoint_url: None,
            drive_queue_dir: None,
            idle_timeout_minutes: 30,
            updated_at_epoch_s: now_secs(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColabInputDescriptor {
    pub role: String,
    pub filename: String,
    pub sha256: String,
    pub bytes: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base64_payload: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColabJobRequest {
    pub schema: String,
    pub job_id: String,
    pub routine: String,
    pub created_at_epoch_s: u64,
    pub steward_did: String,
    pub idle_timeout_minutes: u32,
    pub inputs: Vec<ColabInputDescriptor>,
    pub params: Value,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub conversion_context: Option<ColabConversionContext>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColabConversionContext {
    pub auto_convert_10d: bool,
    pub out_dir: Option<String>,
    pub slug: Option<String>,
    pub asset_uri: String,
    pub domain: Option<String>,
    pub domain_id: u32,
    pub taxon: Option<String>,
    pub fiction: Option<bool>,
    pub evidence_q42: Option<String>,
    pub evidence_q42_sha256: Option<String>,
    pub spdx_license: String,
    pub creator: String,
    pub attribution_notice: String,
    pub origin_uri: Option<String>,
    pub embed_full_source_bytes: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ColabHardwareTelemetry {
    pub gpu_name: String,
    pub vram_total_mib: u64,
    pub vram_peak_mib: u64,
    pub system_ram_total_mib: u64,
    #[serde(default)]
    pub system_ram_peak_mib: u64,
    pub cuda_version: String,
    pub torch_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ColabTimingTelemetry {
    pub started_at_epoch_s: u64,
    pub completed_at_epoch_s: u64,
    pub wall_time_ms: u64,
    #[serde(default)]
    pub estimated_energy_joules: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ColabOutputDescriptor {
    pub role: String,
    pub filename: String,
    pub sha256: String,
    pub bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColabJobReceipt {
    pub schema: String,
    pub job_id: String,
    pub routine: String,
    /// "queued" | "running" | "completed" | "failed"
    pub status: String,
    #[serde(default)]
    pub error: Option<String>,
    #[serde(default)]
    pub colab_hardware: ColabHardwareTelemetry,
    #[serde(default)]
    pub timing: ColabTimingTelemetry,
    #[serde(default)]
    pub model_provenance: Value,
    #[serde(default)]
    pub outputs: Vec<ColabOutputDescriptor>,
    #[serde(default)]
    pub ovoxel_summary: Option<Value>,
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

/// Strictly validate a single path segment (`job_id`, `filename`, `slug`) to prevent
/// directory traversal (`..`), slashes, colons, leading dots, or excessive length.
pub fn validate_safe_component(raw: &str, label: &str) -> Result<String, String> {
    let trimmed = raw.trim();
    if trimmed.is_empty()
        || trimmed.len() > 96
        || trimmed.starts_with('.')
        || trimmed.contains("..")
        || !trimmed
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, b'-' | b'_' | b'.'))
    {
        return Err(format!(
            "Invalid {label} `{trimmed}`: must be 1..=96 ASCII alphanumeric, '-', '_', or '.' characters without '..' or path separators"
        ));
    }
    Ok(trimmed.to_owned())
}

/// Validate an HTTP(S) endpoint URL for the Colab worker.
/// Requires `https://` whenever authentication tokens are used, except for local loopback
/// (`127.0.0.1` or `localhost`) during local harness testing.
pub fn validate_worker_endpoint_url(raw: &str) -> Result<String, String> {
    let trimmed = raw.trim().trim_end_matches('/');
    if trimmed.is_empty() || trimmed.len() > 2048 {
        return Err("Colab endpoint_url must be a valid non-empty URL (<= 2048 chars)".into());
    }
    if trimmed.contains('?') || trimmed.contains('#') || trimmed.contains('@') {
        return Err(
            "Colab endpoint_url must not embed query parameters, fragments, or userinfo credentials"
                .into(),
        );
    }
    if let Some(rest) = trimmed.strip_prefix("https://") {
        if rest.is_empty() {
            return Err("Colab endpoint_url is missing host".into());
        }
        return Ok(trimmed.to_owned());
    }
    if let Some(rest) = trimmed.strip_prefix("http://") {
        let host_port = rest.split('/').next().unwrap_or("");
        let host = host_port.split(':').next().unwrap_or("");
        if host == "127.0.0.1" || host.eq_ignore_ascii_case("localhost") {
            return Ok(trimmed.to_owned());
        }
        return Err(
            "Remote Colab endpoint_url must use https:// to protect Bearer tokens in transit (http:// is only permitted for 127.0.0.1/localhost)"
                .into(),
        );
    }
    Err("Colab endpoint_url must start with https:// (or http://127.0.0.1 for local tests)".into())
}

/// Resolve `.synth/colab-config.json` inside `bookmarks::store_dir()`.
fn config_path() -> Result<PathBuf, String> {
    Ok(bookmarks::store_dir()?.join(COLAB_CONFIG_FILE))
}

/// Resolve `.synth/inbox/colab/` inside `bookmarks::store_dir()` (never under `SYNTH_INPUT_ROOT`).
pub fn colab_inbox_root() -> Result<PathBuf, String> {
    let dir = bookmarks::store_dir()?.join("inbox").join("colab");
    fs::create_dir_all(&dir).map_err(|e| format!("Failed to create .synth/inbox/colab: {e}"))?;
    Ok(dir)
}

pub fn load_colab_config() -> ColabConfig {
    let Ok(path) = config_path() else {
        return ColabConfig::default();
    };
    let Ok(bytes) = fs::read(&path) else {
        return ColabConfig::default();
    };
    serde_json::from_slice::<ColabConfig>(&bytes).unwrap_or_default()
}

pub fn save_colab_config(cfg: &ColabConfig) -> Result<(), String> {
    let path = config_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let bytes = serde_json::to_vec_pretty(cfg).map_err(|e| e.to_string())?;
    fs::write(&path, bytes).map_err(|e| format!("Failed to save colab-config.json: {e}"))
}

/// Resolve the Colab Bearer token from `.synth/secrets.json` (`colab-ultra`) or `SYNTH_COLAB_TOKEN`.
pub fn resolve_colab_api_token() -> Option<String> {
    if let Ok(Some(sec)) = bookmarks::secret_value(COLAB_BOOKMARK_ID) {
        if !sec.trim().is_empty() {
            return Some(sec.trim().to_owned());
        }
    }
    if let Ok(env_val) = std::env::var(COLAB_TOKEN_ENV) {
        if !env_val.trim().is_empty() {
            return Some(env_val.trim().to_owned());
        }
    }
    None
}

/// Store the Colab endpoint and optional secret in `.synth/bookmarks.json` (`kind = "rest"`)
/// and `.synth/secrets.json` (`id = "colab-ultra"`). Always upserts the bookmark first so
/// `bookmarks::set_secret` succeeds.
pub fn store_colab_bookmark_and_secret(
    endpoint_url: Option<&str>,
    api_token: Option<&str>,
) -> Result<(), String> {
    let url_to_store = endpoint_url.unwrap_or("https://colab.research.google.com");
    let validated_url = if endpoint_url.is_some() {
        validate_worker_endpoint_url(url_to_store)?
    } else {
        url_to_store.to_owned()
    };

    let _ = bookmarks::dispatch(bookmarks::BookmarkArg {
        action: Some("upsert".into()),
        id: Some(COLAB_BOOKMARK_ID.into()),
        url: Some(validated_url.clone()),
        data_endpoint: endpoint_url.map(|_| validated_url),
        kind: Some("rest".into()),
        title: Some("Google Colab Ultra Heavy-Compute Burst Worker".into()),
        requires_api_key: Some(true),
        api_key_env: Some(COLAB_TOKEN_ENV.into()),
        auth: Some("bearer".into()),
        domain: Some("compute_burst".into()),
        formats: Some(vec!["glb".into(), "q42".into(), "json".into()]),
        ..bookmarks::BookmarkArg::default()
    })?;

    if let Some(secret) = api_token {
        let trimmed = secret.trim();
        if trimmed.is_empty() {
            return Err("Colab api_token must not be empty".into());
        }
        let _ = bookmarks::dispatch(bookmarks::BookmarkArg {
            action: Some("set_secret".into()),
            id: Some(COLAB_BOOKMARK_ID.into()),
            secret: Some(trimmed.to_owned()),
            api_key_env: Some(COLAB_TOKEN_ENV.into()),
            auth: Some("bearer".into()),
            ..bookmarks::BookmarkArg::default()
        })?;
    }
    Ok(())
}

/// Catalog of supported Google Colab Ultra heavy-compute burst routines with decision guidance.
pub fn routines_catalog_json() -> Value {
    json!({
        "schema": "synth.colab-routines/v1",
        "decision_matrix": {
            "complex_topology_cavities_and_open_surfaces": "trellis2_image_to_3d (or trellis2_multiview_to_3d when 2..4 views exist)",
            "retexture_existing_synth_vibe_mesh_or_clean_cad_surface": "hunyuan3d_shape_and_pbr (pass mesh_path + image_path with mode='pbr_only' to bake delighted 4K PBR maps onto an existing Synth mesh, or mode='shape_and_pbr' for full DiT generation)",
            "articulated_or_multi_part_scene_assembly_aref_inst": "partcrafter_decompose_3d (decomposes a concept image or mesh into distinct semantic part nodes -> Synth Mode-A Archetype .10d + Mode-B instances/*.10d AREF/INST)",
            "automatic_skeletal_rigging_and_skinning_skl1_klod": "unirig_auto_skeleton (predicts kinematic bone tree + per-vertex skinning weights -> .10d SKL1/SKL2 + KLOD0..KLOD3)",
            "online_or_massive_ontology_to_q42_derivative": "ontology_compile_q42 (runs QualiaDB/Synth CLI on 83 GB High-RAM + NVMe /content/scratch to fetch online OWL/RDF/N3 or BioPortal acronym, refactor, and compile into verified .q42 + _refs.ttl + _shapes.shacl.ttl + _rules.n3)"
        },
        "routines": [
            {
                "id": "trellis2_image_to_3d",
                "title": "Microsoft TRELLIS.2-4B Single-Image to 3D O-Voxel & 4K PBR",
                "model_id": "microsoft/TRELLIS.2-4B",
                "model_license": "MIT",
                "when_to_use": "Best for complex non-manifold/open-surface topology, internal cavities, botanical foliage, perforated anatomy, and volumetric O-Voxel + 4K PBR generation from a single reference image.",
                "recommended_gpu": "NVIDIA A100 (40GB/80GB) or NVIDIA L4 (24GB)",
                "min_vram_gib": 24,
                "inputs": ["image_path (under SYNTH_INPUT_ROOT, <= 32 MiB)"],
                "outputs": ["output.glb", "output.ovoxel.json (multi-segment M64HoloRecord clusters + VOX0 DAG nodes)", "receipt.json"],
                "local_handoff": "convert_3d_with_provenance -> {stem}-archetype.10d (HOLO 0x0306 + CBLD + PROV) + {stem}-provenance.n3 + manifest.q42.yaml (L0-L6) + .synth/changelog.json"
            },
            {
                "id": "trellis2_multiview_to_3d",
                "title": "Microsoft TRELLIS.2-4B Multi-View (2..4 Images) to 3D O-Voxel & 4K PBR",
                "model_id": "microsoft/TRELLIS.2-4B",
                "model_license": "MIT",
                "when_to_use": "Best when 2..4 orthographic or multi-angle reference views exist and strict 3D silhouette/proportion fidelity is required alongside O-Voxel volumetric field extraction.",
                "recommended_gpu": "NVIDIA A100 (40GB/80GB) or NVIDIA L4 (24GB)",
                "min_vram_gib": 24,
                "inputs": ["image_paths (2..=4 images under SYNTH_INPUT_ROOT, each <= 32 MiB)"],
                "outputs": ["output.glb", "output.ovoxel.json", "receipt.json"],
                "local_handoff": "convert_3d_with_provenance -> {stem}-archetype.10d (HOLO 0x0306 + CBLD + PROV) + {stem}-provenance.n3 + manifest.q42.yaml (L0-L6) + .synth/changelog.json"
            },
            {
                "id": "hunyuan3d_shape_and_pbr",
                "title": "Tencent Hunyuan3D-2.1 Decoupled DiT Shape Generation & Delighted PBR Texture Baking",
                "model_id": "tencent/Hunyuan3D-2.1",
                "model_license": "Apache-2.0",
                "when_to_use": "Best for (1) mode='pbr_only': baking delighted 4K PBR albedo/normal/roughness/metallic maps onto an existing parametric Synth .vibe mesh (`mesh_path`), or (2) mode='shape_and_pbr': crisp hard-surface/organic watertight shape generation with illumination-invariant PBR maps.",
                "recommended_gpu": "NVIDIA A100 (40GB/80GB) or NVIDIA L4 (24GB)",
                "min_vram_gib": 20,
                "inputs": ["image_path (reference appearance image)", "optional mesh_path (.glb/.obj under SYNTH_INPUT_ROOT when mode='pbr_only')"],
                "outputs": ["output.glb", "receipt.json"],
                "local_handoff": "convert_3d_with_provenance -> {stem}-archetype.10d (HOLO 0x0306 + MAT1 PBR + CBLD + PROV) + {stem}-provenance.n3 + manifest.q42.yaml (L0-L6) + .synth/changelog.json"
            },
            {
                "id": "partcrafter_decompose_3d",
                "title": "PartCrafter / HoloPart Compositional Multi-Part 3D Decomposition (Two-Tier AREF/INST)",
                "model_id": "wgsxm/PartCrafter",
                "model_license": "MIT",
                "when_to_use": "Best for machinery, furniture, buildings, or articulated assemblies that must be decomposed into N distinct semantic sub-parts (`target_parts`: 2..64) so Synth's converter seals Mode-A Archetype `.10d` + Mode-B `instances/*.10d` (`AREF`/`INST` PGA motor transforms) instead of a monolithic triangle soup.",
                "recommended_gpu": "NVIDIA A100 (40GB/80GB) or NVIDIA L4 (24GB)",
                "min_vram_gib": 24,
                "inputs": ["image_path or mesh_path (under SYNTH_INPUT_ROOT)", "optional target_parts (2..=64, default 8)"],
                "outputs": ["output.glb (multi-node hierarchical glTF 2.0 scene)", "output.parts.json", "receipt.json"],
                "local_handoff": "convert_3d_with_provenance -> Mode-A {stem}-archetype.10d + Mode-B instances/*.10d (AREF/INST) + {stem}-provenance.n3 + manifest.q42.yaml (L0-L6)"
            },
            {
                "id": "unirig_auto_skeleton",
                "title": "UniRig / RigAnything Autoregressive Skeletal Hierarchy & Skinning Weight Prediction",
                "model_id": "VAST-AI-Research/UniRig",
                "model_license": "MIT",
                "when_to_use": "Best when a static 3D mesh (`mesh_path`) of a creature, human, plant, or articulated mechanism needs an automatic topological bone hierarchy and per-vertex skinning weights (`JOINTS_0`, `WEIGHTS_0`) for `.10d` `SKL1`/`SKL2` and `KLOD0`..`KLOD3`.",
                "recommended_gpu": "NVIDIA L4 (24GB) or NVIDIA A100 (40GB/80GB)",
                "min_vram_gib": 16,
                "inputs": ["mesh_path (.glb/.obj under SYNTH_INPUT_ROOT)", "optional max_joints (2..=256, default 64)"],
                "outputs": ["output.glb (rigged glTF 2.0 with skin + joints + weights)", "output.skeleton.json", "receipt.json"],
                "local_handoff": "convert_3d_with_provenance -> {stem}-archetype.10d (HOLO 0x0306 + SKL1/SKL2 + CBLD + PROV) + {stem}-provenance.n3 + manifest.q42.yaml"
            },
            {
                "id": "ontology_compile_q42",
                "title": "QualiaDB / Synth CLI High-RAM (83 GB) Online Ontology Fetch, OWL Refactoring & .q42 Compilation",
                "model_id": "qualia-core-db/q42-compiler",
                "model_license": "CC-BY-NC-ND-4.0",
                "when_to_use": "Best for fetching large online ontologies (via `source_url` or BioPortal `acronym` such as FMA, CHEBI, GO, ENVO, SNOMED) or local multi-GB RDF/OWL/N3 dumps and running the Linux `synth` / QualiaDB CLI on Colab's 83 GB System RAM + NVMe `/content/scratch` to produce the canonical `.q42` derivative (`{out_q42}`, `_refs.ttl`, `_shapes.shacl.ttl`, `_rules.n3`).",
                "recommended_gpu": "Colab High-RAM CPU or GPU Runtime (~83 GB System RAM + NVMe /content/scratch)",
                "min_vram_gib": 0,
                "inputs": ["acronym (BioPortal), source_url (online OWL/RDF/TTL/N3 URL), or source_path (under SYNTH_INPUT_ROOT)"],
                "outputs": ["{out_q42} (or {stem}-root.q42 + {stem}-0000.q42 shard set)", "_refs.ttl", "_shapes.shacl.ttl", "_rules.n3", "receipt.json"],
                "local_handoff": "verify_volume_set_from_root / Q42 header & SHA-256 verification -> SYNTH_OUTPUT_ROOT + optional daemon_graph ingestion"
            }
        ]
    })
}

pub fn dispatch_mcp(mcp: &StandaloneSynthMcp, args: Value) -> Result<Value, ToolError> {
    let parsed: ColabArg = serde_json::from_value(args)
        .map_err(|e| ToolError::new("invalid_arguments", format!("Invalid synth_colab arguments: {e}")))?;
    dispatch(Some(mcp), parsed).map_err(|e| ToolError::new("colab_failed", e))
}

pub fn dispatch(mcp: Option<&StandaloneSynthMcp>, arg: ColabArg) -> Result<Value, String> {
    let action = arg
        .action
        .as_deref()
        .unwrap_or("status")
        .trim()
        .to_ascii_lowercase();
    match action.as_str() {
        "routines" | "catalog" => Ok(routines_catalog_json()),
        "configure" | "config" => configure_colab(arg),
        "status" => colab_status(arg),
        "export_notebook" | "notebook" => export_colab_notebook(arg),
        "colab_mcp_cells" | "mcp_cells" => export_colab_mcp_cells(),
        "submit" | "run" => submit_colab_job(mcp, arg),
        "poll" | "get" => poll_colab_job(mcp, arg),
        "ingest" | "fetch" => ingest_colab_job(mcp, arg),
        "list_jobs" | "list" => list_colab_jobs(),
        "cancel" => cancel_colab_job(arg),
        other => Err(format!(
            "Unknown synth_colab action `{other}`; expected status|configure|routines|submit|poll|ingest|list_jobs|cancel|export_notebook|colab_mcp_cells"
        )),
    }
}

fn configure_colab(arg: ColabArg) -> Result<Value, String> {
    let mut cfg = load_colab_config();
    if let Some(t) = arg.transport.as_deref() {
        let mode = t.trim().to_ascii_lowercase();
        if !matches!(mode.as_str(), "auto" | "http" | "drive_queue") {
            return Err("transport must be one of: auto, http, drive_queue".into());
        }
        cfg.transport = mode;
    }
    if let Some(url) = arg.endpoint_url.as_deref() {
        if url.trim().is_empty() || url.trim().eq_ignore_ascii_case("none") {
            cfg.endpoint_url = None;
        } else {
            let valid = validate_worker_endpoint_url(url)?;
            cfg.endpoint_url = Some(valid);
        }
    }
    if let Some(dq) = arg.drive_queue_dir.as_deref() {
        let trimmed = dq.trim();
        if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("none") {
            cfg.drive_queue_dir = None;
        } else {
            if trimmed.contains("..") {
                return Err("drive_queue_dir must not contain '..'".into());
            }
            let p = PathBuf::from(trimmed);
            fs::create_dir_all(p.join("queue").join("pending"))
                .and_then(|_| fs::create_dir_all(p.join("queue").join("running")))
                .and_then(|_| fs::create_dir_all(p.join("queue").join("completed")))
                .map_err(|e| format!("Failed to initialize Drive queue folders under {trimmed}: {e}"))?;
            cfg.drive_queue_dir = Some(trimmed.to_owned());
        }
    }
    if let Some(idle) = arg.idle_timeout_minutes {
        if !(1..=1440).contains(&idle) {
            return Err("idle_timeout_minutes must be between 1 and 1440".into());
        }
        cfg.idle_timeout_minutes = idle;
    }
    cfg.updated_at_epoch_s = now_secs();

    store_colab_bookmark_and_secret(cfg.endpoint_url.as_deref(), arg.api_token.as_deref())?;
    save_colab_config(&cfg)?;

    Ok(json!({
        "schema": "synth.colab-config-receipt/v1",
        "status": "ok",
        "transport": cfg.transport,
        "endpoint_url": cfg.endpoint_url,
        "api_token_configured": resolve_colab_api_token().is_some(),
        "drive_queue_dir": cfg.drive_queue_dir,
        "idle_timeout_minutes": cfg.idle_timeout_minutes,
    }))
}

fn colab_status(arg: ColabArg) -> Result<Value, String> {
    let cfg = load_colab_config();
    let has_token = resolve_colab_api_token().is_some();
    let endpoint = arg.endpoint_url.as_deref().or(cfg.endpoint_url.as_deref());
    let mut remote_health = Value::Null;
    if let Some(url) = endpoint {
        if let Ok(valid_url) = validate_worker_endpoint_url(url) {
            let health_url = format!("{valid_url}/health");
            let mut req = ureq::get(&health_url).timeout(Duration::from_secs(5));
            if let Some(tok) = resolve_colab_api_token() {
                req = req.set("Authorization", &format!("Bearer {tok}"));
            }
            remote_health = match req.call() {
                Ok(resp) => resp
                    .into_string()
                    .ok()
                    .and_then(|s| serde_json::from_str::<Value>(&s).ok())
                    .unwrap_or_else(|| json!({"reachable": true})),
                Err(_) => json!({"reachable": false, "note": "Colab HTTP tunnel not currently reachable"}),
            };
        }
    }

    let drive_status = if let Some(dq) = cfg.drive_queue_dir.as_deref() {
        let p = Path::new(dq);
        let pending_count = fs::read_dir(p.join("queue").join("pending"))
            .map(|rd| rd.flatten().count())
            .unwrap_or(0);
        let completed_count = fs::read_dir(p.join("queue").join("completed"))
            .map(|rd| rd.flatten().count())
            .unwrap_or(0);
        json!({
            "configured": true,
            "root": dq,
            "exists": p.is_dir(),
            "pending_jobs": pending_count,
            "completed_jobs": completed_count,
        })
    } else {
        json!({"configured": false})
    };

    Ok(json!({
        "schema": "synth.colab-status/v1",
        "bookmark_id": COLAB_BOOKMARK_ID,
        "transport": cfg.transport,
        "endpoint_url": cfg.endpoint_url,
        "api_token_configured": has_token,
        "idle_timeout_minutes": cfg.idle_timeout_minutes,
        "drive_queue": drive_status,
        "remote_health": remote_health,
        "notebook_path": r"C:\github\synth\vendor\colab\synth_colab_ultra_worker.ipynb",
        "worker_script_path": r"C:\github\synth\vendor\colab\colab_worker.py",
    }))
}

fn export_colab_notebook(arg: ColabArg) -> Result<Value, String> {
    let target_dir = if let Some(custom) = arg.notebook_out_path.as_deref() {
        if custom.contains("..") {
            return Err("notebook_out_path must not contain '..'".into());
        }
        PathBuf::from(custom)
    } else {
        colab_inbox_root()?.join("exported-notebook")
    };
    fs::create_dir_all(&target_dir).map_err(|e| format!("Failed to create notebook export dir: {e}"))?;
    let nb_path = target_dir.join("synth_colab_ultra_worker.ipynb");
    let py_path = target_dir.join("colab_worker.py");
    fs::write(&nb_path, EMBEDDED_COLAB_NOTEBOOK_IPYNB.as_bytes())
        .map_err(|e| format!("Failed to write notebook: {e}"))?;
    fs::write(&py_path, EMBEDDED_COLAB_WORKER_PY.as_bytes())
        .map_err(|e| format!("Failed to write colab_worker.py: {e}"))?;

    Ok(json!({
        "schema": "synth.colab-notebook-export/v1",
        "status": "ok",
        "notebook_path": nb_path,
        "worker_script_path": py_path,
        "notebook_sha256": sha256_hex(EMBEDDED_COLAB_NOTEBOOK_IPYNB.as_bytes()),
        "worker_script_sha256": sha256_hex(EMBEDDED_COLAB_WORKER_PY.as_bytes()),
    }))
}

/// Emit the ready-to-execute Python code cells and embedded `colab_worker.py` source so
/// an agent connected to `googlecolab/colab-mcp` (`colab-mcp`) can provision and run
/// the Google Colab Ultra A100/L4 worker directly via MCP cell execution without manual upload.
fn export_colab_mcp_cells() -> Result<Value, String> {
    let nb_val: Value =
        serde_json::from_str(EMBEDDED_COLAB_NOTEBOOK_IPYNB).map_err(|e| e.to_string())?;
    let mut code_cells = Vec::new();
    if let Some(cells) = nb_val.get("cells").and_then(Value::as_array) {
        for (idx, cell) in cells.iter().enumerate() {
            let cell_type = cell.get("cell_type").and_then(Value::as_str).unwrap_or("code");
            let source_lines: Vec<&str> = cell
                .get("source")
                .and_then(Value::as_array)
                .map(|arr| arr.iter().filter_map(Value::as_str).collect())
                .unwrap_or_default();
            code_cells.push(json!({
                "index": idx,
                "cell_type": cell_type,
                "source": source_lines.join(""),
            }));
        }
    }
    Ok(json!({
        "schema": "synth.colab-mcp-cells/v1",
        "colab_mcp_server": "uvx git+https://github.com/googlecolab/colab-mcp",
        "usage": "Use `colab-mcp` tools to write `/content/colab_worker.py` (from `worker_script_py`) and execute `cells` in sequence on the Google Colab Ultra A100/L4 kernel.",
        "cells": code_cells,
        "worker_script_py": EMBEDDED_COLAB_WORKER_PY,
    }))
}

/// Resolve a local file strictly through `mcp.input_root` when running under MCP,
/// or fall back to `SYNTH_INPUT_ROOT` when running via CLI.
fn resolve_confined_input_file(
    mcp: Option<&StandaloneSynthMcp>,
    requested: &str,
    max_bytes: usize,
) -> Result<(PathBuf, Vec<u8>), String> {
    if let Some(server) = mcp {
        let root = server
            .input_root
            .as_ref()
            .ok_or_else(|| "Set SYNTH_INPUT_ROOT to admit local input files".to_owned())?;
        return root.read(requested, max_bytes).map_err(|e| e.message);
    }
    let env_root = std::env::var("SYNTH_INPUT_ROOT")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| r"C:\github".to_owned());
    let root = super::production::InputRoot::admit(Path::new(env_root.trim()))?;
    root.read(requested, max_bytes).map_err(|e| e.message)
}

fn resolve_confined_input_path_only(
    mcp: Option<&StandaloneSynthMcp>,
    requested: &str,
    max_bytes: u64,
) -> Result<(PathBuf, u64), String> {
    if let Some(server) = mcp {
        let root = server
            .input_root
            .as_ref()
            .ok_or_else(|| "Set SYNTH_INPUT_ROOT to admit local input paths".to_owned())?;
        return root.resolve_path(requested, max_bytes).map_err(|e| e.message);
    }
    let env_root = std::env::var("SYNTH_INPUT_ROOT")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| r"C:\github".to_owned());
    let root = super::production::InputRoot::admit(Path::new(env_root.trim()))?;
    root.resolve_path(requested, max_bytes).map_err(|e| e.message)
}

/// Enforce Ecopunk biosphere production rules and verify any supplied `evidence_q42`
/// through `InputRoot`.
fn enforce_biosphere_gate_for_routine(
    mcp: Option<&StandaloneSynthMcp>,
    routine: &str,
    domain: Option<&str>,
    taxon: Option<&str>,
    fiction: Option<bool>,
    evidence_q42: Option<&str>,
) -> Result<Option<String>, String> {
    let mut evidence_sha: Option<String> = None;
    let mut resolved_evidence_path: Option<String> = None;

    if let Some(ev_path) = evidence_q42 {
        let (canonical, bytes) =
            resolve_confined_input_file(mcp, ev_path, 256 * 1024 * 1024)?;
        if bytes.len() < 64 || &bytes[0..4] != b"Q42\x00" {
            return Err("evidence_q42 must be a valid sealed .q42 volume (Q42\\0 magic)".into());
        }
        evidence_sha = Some(sha256_hex(&bytes));
        resolved_evidence_path = Some(canonical.to_string_lossy().into_owned());
    }

    let rule_action = if routine == "ontology_compile_q42" {
        "compile_q42"
    } else {
        "produce"
    };

    let req = crate::operator::RuleRequest {
        domain: domain.map(str::to_owned),
        taxon: taxon.map(str::to_owned),
        fiction,
        action: Some(rule_action.into()),
        evidence_q42: resolved_evidence_path,
    };
    crate::operator::enforce_produce(&req)?;
    Ok(evidence_sha)
}

fn encode_base64(bytes: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity((bytes.len() + 2) / 3 * 4);
    for chunk in bytes.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
        let triple = (b0 << 16) | (b1 << 8) | b2;
        out.push(TABLE[((triple >> 18) & 0x3F) as usize] as char);
        out.push(TABLE[((triple >> 12) & 0x3F) as usize] as char);
        if chunk.len() > 1 {
            out.push(TABLE[((triple >> 6) & 0x3F) as usize] as char);
        } else {
            out.push('=');
        }
        if chunk.len() > 2 {
            out.push(TABLE[(triple & 0x3F) as usize] as char);
        } else {
            out.push('=');
        }
    }
    out
}

fn is_3d_routine(routine: &str) -> bool {
    matches!(
        routine,
        "trellis2_image_to_3d"
            | "trellis2_multiview_to_3d"
            | "hunyuan3d_shape_and_pbr"
            | "partcrafter_decompose_3d"
            | "unirig_auto_skeleton"
    )
}

fn model_metadata_for_routine(routine: &str) -> (&'static str, &'static str, &'static str) {
    match routine {
        "trellis2_image_to_3d" | "trellis2_multiview_to_3d" => (
            "microsoft/TRELLIS.2-4B",
            "Microsoft TRELLIS.2 (4B O-Voxel Flow-Matching Transformer)",
            "2.0-4B",
        ),
        "hunyuan3d_shape_and_pbr" => (
            "tencent/Hunyuan3D-2.1",
            "Tencent Hunyuan3D-2.1 (DiT Shape & Delighted PBR Paint)",
            "2.1",
        ),
        "partcrafter_decompose_3d" => (
            "wgsxm/PartCrafter",
            "PartCrafter / HoloPart Compositional Multi-Part 3D Decomposer",
            "1.0",
        ),
        "unirig_auto_skeleton" => (
            "VAST-AI-Research/UniRig",
            "UniRig Autoregressive Skeletal Hierarchy & Skinning Predictor",
            "1.0",
        ),
        _ => (
            "qualia-core-db/q42-compiler",
            "Webizen AI Synth QualiaDB Q42 Streaming Compiler",
            crate::agent_index::RELEASE_VERSION,
        ),
    }
}

fn submit_colab_job(mcp: Option<&StandaloneSynthMcp>, arg: ColabArg) -> Result<Value, String> {
    let routine = arg
        .routine
        .as_deref()
        .ok_or("submit requires routine: trellis2_image_to_3d | trellis2_multiview_to_3d | hunyuan3d_shape_and_pbr | partcrafter_decompose_3d | unirig_auto_skeleton | ontology_compile_q42")?
        .trim()
        .to_ascii_lowercase();

    if !is_3d_routine(&routine) && routine != "ontology_compile_q42" {
        return Err(format!(
            "Unsupported routine `{routine}`; supported: trellis2_image_to_3d, trellis2_multiview_to_3d, hunyuan3d_shape_and_pbr, partcrafter_decompose_3d, unirig_auto_skeleton, ontology_compile_q42"
        ));
    }

    let evidence_sha = enforce_biosphere_gate_for_routine(
        mcp,
        &routine,
        arg.domain.as_deref(),
        arg.taxon.as_deref(),
        arg.fiction,
        arg.evidence_q42.as_deref(),
    )?;

    let cfg = load_colab_config();
    let ts = now_secs();
    let mut seed_hasher = Sha256::new();
    seed_hasher.update(routine.as_bytes());
    seed_hasher.update(ts.to_le_bytes());
    seed_hasher.update(std::process::id().to_le_bytes());
    let short_hash = &format!("{:x}", seed_hasher.finalize())[..8];
    let job_id = if let Some(custom_id) = arg.job_id.as_deref() {
        validate_safe_component(custom_id, "job_id")?
    } else {
        format!("colab-{ts}-{short_hash}")
    };

    let mut inputs = Vec::new();
    let mut raw_input_files: Vec<(String, Vec<u8>)> = Vec::new();
    let params: Value;
    let mut conversion_context: Option<ColabConversionContext> = None;

    if is_3d_routine(&routine) {
        let resolution = arg.resolution.unwrap_or(1024);
        if !matches!(resolution, 512 | 1024 | 1536) {
            return Err("resolution must be 512, 1024, or 1536".into());
        }
        let decimation_target = arg.decimation_target.unwrap_or(DEFAULT_DECIMATION_TRIS);
        if !(MIN_DECIMATION_TRIS..=MAX_DECIMATION_TRIS).contains(&decimation_target) {
            return Err(format!(
                "decimation_target must be between {MIN_DECIMATION_TRIS} and {MAX_DECIMATION_TRIS} triangles to fit Synth's production16 mesh budget"
            ));
        }
        let texture_size = arg.texture_size.unwrap_or(2048);
        if !matches!(texture_size, 1024 | 2048 | 4096) {
            return Err("texture_size must be 1024, 2048, or 4096".into());
        }

        let auto_convert = arg.auto_convert_10d.unwrap_or(true);
        let spdx = arg
            .spdx_license
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty());
        let attribution = arg
            .attribution_notice
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty());

        if auto_convert && (spdx.is_none() || attribution.is_none()) {
            return Err(
                "auto_convert_10d=true requires explicit spdx_license and attribution_notice for the input reference image(s)/mesh to enforce source-asset-licensing.n3 high-water-mark provenance"
                    .into(),
            );
        }

        // Stage optional reference image(s)
        let mut requested_images: Vec<String> = Vec::new();
        if routine == "trellis2_image_to_3d" {
            let p = arg
                .image_path
                .as_deref()
                .ok_or("trellis2_image_to_3d requires image_path under SYNTH_INPUT_ROOT")?;
            requested_images.push(p.to_owned());
        } else if routine == "trellis2_multiview_to_3d" {
            let list = arg
                .image_paths
                .clone()
                .ok_or("trellis2_multiview_to_3d requires image_paths (2..=4 images)")?;
            if !(2..=4).contains(&list.len()) {
                return Err("trellis2_multiview_to_3d requires between 2 and 4 image_paths".into());
            }
            requested_images = list;
        } else if let Some(img) = arg.image_path.as_deref() {
            requested_images.push(img.to_owned());
        }

        for (idx, req_p) in requested_images.iter().enumerate() {
            let (canonical, bytes) =
                resolve_confined_input_file(mcp, req_p, MAX_IMAGE_INPUT_BYTES)?;
            let ext = canonical
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or("png")
                .to_ascii_lowercase();
            if !matches!(ext.as_str(), "png" | "jpg" | "jpeg" | "webp") {
                return Err(format!("Unsupported reference image extension .{ext}; use png, jpg, or webp"));
            }
            let fname = format!("input_{idx:02}.{ext}");
            let sha = sha256_hex(&bytes);
            inputs.push(ColabInputDescriptor {
                role: format!("reference_image_{idx}"),
                filename: fname.clone(),
                sha256: sha,
                bytes: bytes.len(),
                base64_payload: None,
            });
            raw_input_files.push((fname, bytes));
        }

        // Stage optional input 3D mesh (.glb/.obj/.stl) for hunyuan3d_shape_and_pbr (pbr_only), partcrafter_decompose_3d, or unirig_auto_skeleton
        if let Some(req_mesh) = arg.mesh_path.as_deref() {
            let (canonical, bytes) =
                resolve_confined_input_file(mcp, req_mesh, 128 * 1024 * 1024)?;
            let ext = canonical
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or("glb")
                .to_ascii_lowercase();
            if !matches!(ext.as_str(), "glb" | "gltf" | "obj" | "stl") {
                return Err(format!("Unsupported mesh_path extension .{ext}; use glb, obj, or stl"));
            }
            let fname = format!("input_mesh.{ext}");
            let sha = sha256_hex(&bytes);
            inputs.push(ColabInputDescriptor {
                role: "input_mesh".into(),
                filename: fname.clone(),
                sha256: sha,
                bytes: bytes.len(),
                base64_payload: None,
            });
            raw_input_files.push((fname, bytes));
        }

        // Routine-specific validation
        if routine == "hunyuan3d_shape_and_pbr" && requested_images.is_empty() {
            return Err("hunyuan3d_shape_and_pbr requires image_path (and optional mesh_path when mode='pbr_only')".into());
        }
        if routine == "partcrafter_decompose_3d" && requested_images.is_empty() && arg.mesh_path.is_none() {
            return Err("partcrafter_decompose_3d requires image_path or mesh_path under SYNTH_INPUT_ROOT".into());
        }
        if routine == "unirig_auto_skeleton" && arg.mesh_path.is_none() {
            return Err("unirig_auto_skeleton requires mesh_path (.glb/.obj under SYNTH_INPUT_ROOT)".into());
        }

        let (model_id, _, _) = model_metadata_for_routine(&routine);
        params = json!({
            "model_id": model_id,
            "mode": arg.mode.as_deref().unwrap_or(match routine.as_str() {
                "hunyuan3d_shape_and_pbr" => if arg.mesh_path.is_some() { "pbr_only" } else { "shape_and_pbr" },
                "partcrafter_decompose_3d" => "decompose",
                "unirig_auto_skeleton" => "rig",
                _ => "shape_and_pbr",
            }),
            "target_parts": arg.target_parts.unwrap_or(8).clamp(2, 64),
            "max_joints": arg.max_joints.unwrap_or(64).clamp(2, 256),
            "resolution": resolution,
            "seed": arg.seed.unwrap_or(42),
            "decimation_target": decimation_target,
            "texture_size": texture_size,
            "remesh": arg.remesh.unwrap_or(true),
            "export_ovoxel_summary": arg.export_ovoxel_summary.unwrap_or(routine.starts_with("trellis2_")),
        });

        conversion_context = Some(ColabConversionContext {
            auto_convert_10d: auto_convert,
            out_dir: arg.out_dir.clone(),
            slug: arg.slug.clone(),
            asset_uri: arg
                .asset_uri
                .clone()
                .unwrap_or_else(|| format!("urn:qualia:colab:{job_id}")),
            domain: arg.domain.clone(),
            domain_id: arg.domain_id.unwrap_or(1),
            taxon: arg.taxon.clone(),
            fiction: arg.fiction,
            evidence_q42: arg.evidence_q42.clone(),
            evidence_q42_sha256: evidence_sha,
            spdx_license: spdx.unwrap_or("CC-BY-NC-ND-4.0").to_owned(),
            creator: arg
                .creator
                .clone()
                .unwrap_or_else(|| "Timothy Charles Holborn".into()),
            attribution_notice: attribution
                .unwrap_or("Copyright (c) 2026 Timothy Charles Holborn")
                .to_owned(),
            origin_uri: arg.origin_uri.clone(),
            embed_full_source_bytes: arg.embed_full_source_bytes.unwrap_or(false),
        });
    } else {
        // ontology_compile_q42
        let out_q42 = arg
            .out_q42
            .as_deref()
            .ok_or("ontology_compile_q42 requires out_q42 (e.g. fma_refactored.q42)")?;
        let valid_out_q42 = validate_safe_component(out_q42, "out_q42")?;
        if !valid_out_q42.ends_with(".q42") {
            return Err("out_q42 must end with .q42".into());
        }

        if let Some(local_src) = arg.source_path.as_deref() {
            let (canonical, len) =
                resolve_confined_input_path_only(mcp, local_src, 4 * 1024 * 1024 * 1024)?;
            let fname = canonical
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("ontology.owl")
                .to_owned();
            let valid_fname = validate_safe_component(&fname, "source_path filename")?;
            let sha = super::q42::sha256_file(&canonical).map_err(|e| e.message)?;
            inputs.push(ColabInputDescriptor {
                role: "ontology_source".into(),
                filename: valid_fname.clone(),
                sha256: sha,
                bytes: len as usize,
                base64_payload: None,
            });
            if len <= 64 * 1024 * 1024 {
                let bytes = fs::read(&canonical).map_err(|e| e.to_string())?;
                raw_input_files.push((valid_fname, bytes));
            }
        } else if arg.acronym.is_none() && arg.source_url.is_none() {
            return Err(
                "ontology_compile_q42 requires at least one of: acronym (BioPortal), source_url, or source_path"
                    .into(),
            );
        }

        if let Some(url) = arg.source_url.as_deref() {
            if !url.starts_with("https://") && !url.starts_with("http://") {
                return Err("source_url must start with https:// or http://".into());
            }
        }

        params = json!({
            "acronym": arg.acronym,
            "source_url": arg.source_url,
            "download_format": arg.download_format.as_deref().unwrap_or("owl"),
            "context_iri": arg.context_iri,
            "out_q42": valid_out_q42,
            "refactor_owl": arg.refactor_owl.unwrap_or(true),
            "segment_mib": arg.segment_mib,
            "ingest_to_graph": arg.ingest_to_graph.unwrap_or(false),
            "qualia_ingest_scratch": "/content/scratch",
        });
    }

    let job_req = ColabJobRequest {
        schema: COLAB_JOB_REQUEST_SCHEMA.into(),
        job_id: job_id.clone(),
        routine: routine.clone(),
        created_at_epoch_s: ts,
        steward_did: "did:q42:person:timothy-charles-holborn".into(),
        idle_timeout_minutes: arg.idle_timeout_minutes.unwrap_or(cfg.idle_timeout_minutes),
        inputs,
        params,
        conversion_context,
    };

    // Persist local job manifest in `.synth/inbox/colab/{job_id}/job.json`
    let local_job_dir = colab_inbox_root()?.join(&job_id);
    fs::create_dir_all(&local_job_dir).map_err(|e| format!("create job dir: {e}"))?;
    let job_req_bytes = serde_json::to_vec_pretty(&job_req).map_err(|e| e.to_string())?;
    fs::write(local_job_dir.join("job.json"), &job_req_bytes).map_err(|e| e.to_string())?;
    for (fname, bytes) in &raw_input_files {
        fs::write(local_job_dir.join(fname), bytes).map_err(|e| e.to_string())?;
    }

    // Determine effective transport ("http" vs "drive_queue")
    let requested_transport = arg
        .transport
        .as_deref()
        .unwrap_or(&cfg.transport)
        .to_ascii_lowercase();
    let endpoint = arg.endpoint_url.as_deref().or(cfg.endpoint_url.as_deref());
    let drive_dir = arg
        .drive_queue_dir
        .as_deref()
        .or(cfg.drive_queue_dir.as_deref());

    let use_http = match requested_transport.as_str() {
        "http" => true,
        "drive_queue" => false,
        _ => endpoint.is_some(),
    };

    if use_http {
        let raw_url = endpoint.ok_or(
            "Colab HTTP transport selected, but no endpoint_url is configured. Run `synth colab configure --url https://...` or use `transport=\"drive_queue\"`.",
        )?;
        let base_url = validate_worker_endpoint_url(raw_url)?;
        let mut http_req_payload = job_req.clone();
        for inp in &mut http_req_payload.inputs {
            if let Some((_, bytes)) = raw_input_files.iter().find(|(n, _)| *n == inp.filename) {
                inp.base64_payload = Some(encode_base64(bytes));
            }
        }
        let post_url = format!("{base_url}/jobs");
        let mut req = ureq::post(&post_url)
            .timeout(Duration::from_secs(60))
            .set("Content-Type", "application/json");
        if let Some(tok) = resolve_colab_api_token() {
            req = req.set("Authorization", &format!("Bearer {tok}"));
        }
        let body_str = serde_json::to_string(&http_req_payload).map_err(|e| e.to_string())?;
        let resp = req
            .send_string(&body_str)
            .map_err(|_| "Failed to submit job to Colab HTTP endpoint (verify worker is online and Bearer token matches)".to_owned())?;
        let remote_reply: Value = resp
            .into_string()
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_else(|| json!({"status": "queued"}));

        return Ok(json!({
            "schema": "synth.colab-submit-receipt/v1",
            "status": "submitted",
            "transport": "http",
            "job_id": job_id,
            "routine": routine,
            "local_staging_dir": local_job_dir,
            "remote_response": remote_reply,
        }));
    }

    // Drive Queue transport (Mode B)
    let dq_root = drive_dir.ok_or(
        "Neither HTTP endpoint_url nor drive_queue_dir is configured. Run `synth colab configure` first.",
    )?;
    if dq_root.contains("..") {
        return Err("drive_queue_dir must not contain '..'".into());
    }
    let pending_job_dir = Path::new(dq_root)
        .join("queue")
        .join("pending")
        .join(&job_id);
    fs::create_dir_all(&pending_job_dir)
        .map_err(|e| format!("Failed to create Drive queue pending dir: {e}"))?;
    for (fname, bytes) in &raw_input_files {
        fs::write(pending_job_dir.join(fname), bytes)
            .map_err(|e| format!("Failed to stage {fname} to Drive queue: {e}"))?;
    }
    // Atomic publish: write job.json.tmp first, then rename to job.json so Colab watcher never reads a partial file
    let tmp_job = pending_job_dir.join("job.json.tmp");
    let final_job = pending_job_dir.join("job.json");
    fs::write(&tmp_job, &job_req_bytes).map_err(|e| e.to_string())?;
    fs::rename(&tmp_job, &final_job).map_err(|e| e.to_string())?;

    Ok(json!({
        "schema": "synth.colab-submit-receipt/v1",
        "status": "queued_on_drive",
        "transport": "drive_queue",
        "job_id": job_id,
        "routine": routine,
        "drive_pending_dir": pending_job_dir,
        "local_staging_dir": local_job_dir,
    }))
}

fn poll_colab_job(mcp: Option<&StandaloneSynthMcp>, arg: ColabArg) -> Result<Value, String> {
    let raw_id = arg.job_id.as_deref().ok_or("poll requires job_id")?;
    let job_id = validate_safe_component(raw_id, "job_id")?;
    let cfg = load_colab_config();
    let local_job_dir = colab_inbox_root()?.join(&job_id);

    // 1. Check if receipt.json is already in local staging
    let local_receipt_path = local_job_dir.join("receipt.json");
    if local_receipt_path.is_file() {
        let bytes = fs::read(&local_receipt_path).map_err(|e| e.to_string())?;
        let receipt: ColabJobReceipt =
            serde_json::from_slice(&bytes).map_err(|e| format!("Invalid local receipt.json: {e}"))?;
        if receipt.status == "completed" {
            return Ok(json!({
                "schema": "synth.colab-poll/v1",
                "job_id": job_id,
                "status": "completed",
                "source": "local_staging",
                "receipt": receipt,
            }));
        }
    }

    // 2. Check Drive Queue if configured
    let drive_dir = arg
        .drive_queue_dir
        .as_deref()
        .or(cfg.drive_queue_dir.as_deref());
    if let Some(dq_root) = drive_dir {
        let completed_dir = Path::new(dq_root)
            .join("queue")
            .join("completed")
            .join(&job_id);
        let dq_receipt = completed_dir.join("receipt.json");
        if dq_receipt.is_file() {
            let bytes = fs::read(&dq_receipt).map_err(|e| e.to_string())?;
            let receipt: ColabJobReceipt = serde_json::from_slice(&bytes)
                .map_err(|e| format!("Invalid Drive receipt.json: {e}"))?;
            return Ok(json!({
                "schema": "synth.colab-poll/v1",
                "job_id": job_id,
                "status": receipt.status,
                "source": "drive_queue",
                "receipt": receipt,
            }));
        }
        let running_dir = Path::new(dq_root)
            .join("queue")
            .join("running")
            .join(&job_id);
        if running_dir.is_dir() {
            return Ok(json!({
                "schema": "synth.colab-poll/v1",
                "job_id": job_id,
                "status": "running",
                "source": "drive_queue",
            }));
        }
    }

    // 3. Check HTTP endpoint if configured
    let endpoint = arg.endpoint_url.as_deref().or(cfg.endpoint_url.as_deref());
    if let Some(raw_url) = endpoint {
        let base_url = validate_worker_endpoint_url(raw_url)?;
        let status_url = format!("{base_url}/jobs/{job_id}");
        let mut req = ureq::get(&status_url).timeout(Duration::from_secs(15));
        if let Some(tok) = resolve_colab_api_token() {
            req = req.set("Authorization", &format!("Bearer {tok}"));
        }
        if let Ok(resp) = req.call() {
            let text = resp
                .into_string()
                .map_err(|e| format!("Failed reading receipt from Colab HTTP endpoint: {e}"))?;
            let receipt: ColabJobReceipt = serde_json::from_str(&text)
                .map_err(|e| format!("Invalid receipt JSON from Colab HTTP endpoint: {e}"))?;
            return Ok(json!({
                "schema": "synth.colab-poll/v1",
                "job_id": job_id,
                "status": receipt.status,
                "source": "http",
                "receipt": receipt,
            }));
        }
    }

    let _ = mcp;
    Ok(json!({
        "schema": "synth.colab-poll/v1",
        "job_id": job_id,
        "status": "queued",
    }))
}

/// Download/copy completed artifacts from HTTP or Drive Queue into `.synth/inbox/colab/{job_id}/`,
/// verify all SHA-256 digests and size limits, and execute native Synth post-processing:
///   - `trellis2_*`: runs `convert_3d_with_provenance` -> Holographic `.10d` v2 (`HOLO` `0x0306`) + `.ovoxel.json` + `.synth/changelog.json`
///   - `ontology_compile_q42`: verifies `.q42` (or multi-shard volume set via `verify_volume_set_from_root`) and installs into `SYNTH_OUTPUT_ROOT`.
fn ingest_colab_job(mcp: Option<&StandaloneSynthMcp>, arg: ColabArg) -> Result<Value, String> {
    let raw_id = arg.job_id.as_deref().ok_or("ingest requires job_id")?;
    let job_id = validate_safe_component(raw_id, "job_id")?;
    let cfg = load_colab_config();
    let local_job_dir = colab_inbox_root()?.join(&job_id);
    fs::create_dir_all(&local_job_dir).map_err(|e| e.to_string())?;

    // Load original job.json if available
    let stored_req: Option<ColabJobRequest> = fs::read(local_job_dir.join("job.json"))
        .ok()
        .and_then(|b| serde_json::from_slice(&b).ok());

    let drive_dir = arg
        .drive_queue_dir
        .as_deref()
        .or(cfg.drive_queue_dir.as_deref());
    let endpoint = arg.endpoint_url.as_deref().or(cfg.endpoint_url.as_deref());

    // Resolve receipt.json from local staging, Drive Queue, or HTTP
    let receipt: ColabJobReceipt = if local_job_dir.join("receipt.json").is_file() {
        let bytes = fs::read(local_job_dir.join("receipt.json")).map_err(|e| e.to_string())?;
        serde_json::from_slice(&bytes).map_err(|e| format!("Invalid local receipt.json: {e}"))?
    } else if let Some(dq_root) = drive_dir.filter(|d| {
        Path::new(d)
            .join("queue")
            .join("completed")
            .join(&job_id)
            .join("receipt.json")
            .is_file()
    }) {
        let completed_dir = Path::new(dq_root)
            .join("queue")
            .join("completed")
            .join(&job_id);
        let bytes = fs::read(completed_dir.join("receipt.json")).map_err(|e| e.to_string())?;
        let rec: ColabJobReceipt =
            serde_json::from_slice(&bytes).map_err(|e| format!("Invalid Drive receipt.json: {e}"))?;
        // Copy all outputs from completed_dir into local_job_dir with strict filename validation
        for out_desc in &rec.outputs {
            let safe_fname = validate_safe_component(&out_desc.filename, "output filename")?;
            if out_desc.bytes > MAX_ARTIFACT_BYTES {
                return Err(format!(
                    "Output artifact `{safe_fname}` exceeds maximum allowed size ({MAX_ARTIFACT_BYTES} bytes)"
                ));
            }
            let src_file = completed_dir.join(&safe_fname);
            let meta = fs::metadata(&src_file)
                .map_err(|e| format!("Missing completed artifact `{safe_fname}` in Drive queue: {e}"))?;
            if meta.len() > MAX_ARTIFACT_BYTES {
                return Err(format!("Artifact `{safe_fname}` on disk exceeds budget"));
            }
            let data = fs::read(&src_file).map_err(|e| e.to_string())?;
            let actual_sha = sha256_hex(&data);
            if !actual_sha.eq_ignore_ascii_case(&out_desc.sha256) {
                return Err(format!(
                    "SHA-256 mismatch on Drive artifact `{safe_fname}`: expected {}, got {actual_sha}",
                    out_desc.sha256
                ));
            }
            fs::write(local_job_dir.join(&safe_fname), &data).map_err(|e| e.to_string())?;
        }
        fs::write(local_job_dir.join("receipt.json"), &bytes).map_err(|e| e.to_string())?;
        rec
    } else if let Some(raw_url) = endpoint {
        let base_url = validate_worker_endpoint_url(raw_url)?;
        let status_url = format!("{base_url}/jobs/{job_id}");
        let mut req = ureq::get(&status_url).timeout(Duration::from_secs(30));
        if let Some(tok) = resolve_colab_api_token() {
            req = req.set("Authorization", &format!("Bearer {tok}"));
        }
        let resp = req
            .call()
            .map_err(|_| "Failed to fetch job receipt from Colab HTTP endpoint".to_owned())?;
        let text = resp
            .into_string()
            .map_err(|e| format!("Failed reading receipt from Colab endpoint: {e}"))?;
        let rec: ColabJobReceipt = serde_json::from_str(&text)
            .map_err(|e| format!("Invalid receipt JSON from Colab endpoint: {e}"))?;
        if rec.status == "completed" {
            for out_desc in &rec.outputs {
                let safe_fname = validate_safe_component(&out_desc.filename, "output filename")?;
                if out_desc.bytes > MAX_ARTIFACT_BYTES {
                    return Err(format!("Remote artifact `{safe_fname}` exceeds budget"));
                }
                let art_url = format!("{base_url}/jobs/{job_id}/artifacts/{safe_fname}");
                let mut art_req = ureq::get(&art_url).timeout(Duration::from_secs(120));
                if let Some(tok) = resolve_colab_api_token() {
                    art_req = art_req.set("Authorization", &format!("Bearer {tok}"));
                }
                let art_resp = art_req
                    .call()
                    .map_err(|_| format!("Failed to download artifact `{safe_fname}` from Colab endpoint"))?;
                let mut data = Vec::new();
                art_resp
                    .into_reader()
                    .take(MAX_ARTIFACT_BYTES + 1)
                    .read_to_end(&mut data)
                    .map_err(|e| format!("Failed reading artifact `{safe_fname}`: {e}"))?;
                if data.len() as u64 > MAX_ARTIFACT_BYTES {
                    return Err(format!("Artifact `{safe_fname}` exceeded {MAX_ARTIFACT_BYTES} bytes"));
                }
                let actual_sha = sha256_hex(&data);
                if !actual_sha.eq_ignore_ascii_case(&out_desc.sha256) {
                    return Err(format!(
                        "SHA-256 mismatch on downloaded artifact `{safe_fname}`: expected {}, got {actual_sha}",
                        out_desc.sha256
                    ));
                }
                fs::write(local_job_dir.join(&safe_fname), &data).map_err(|e| e.to_string())?;
            }
            let rec_bytes = serde_json::to_vec_pretty(&rec).map_err(|e| e.to_string())?;
            fs::write(local_job_dir.join("receipt.json"), &rec_bytes).map_err(|e| e.to_string())?;
        }
        rec
    } else {
        return Err(format!(
            "No completed receipt found for job `{job_id}` in local staging, Drive queue, or HTTP endpoint"
        ));
    };

    if receipt.status != "completed" {
        return Err(format!(
            "Job `{job_id}` is not completed (current status: `{}`, error: {:?})",
            receipt.status, receipt.error
        ));
    }

    // Verify all declared outputs in local_job_dir
    for out_desc in &receipt.outputs {
        let safe_fname = validate_safe_component(&out_desc.filename, "output filename")?;
        let local_file = local_job_dir.join(&safe_fname);
        let data = fs::read(&local_file)
            .map_err(|e| format!("Missing staged artifact `{safe_fname}`: {e}"))?;
        let actual_sha = sha256_hex(&data);
        if !actual_sha.eq_ignore_ascii_case(&out_desc.sha256) {
            return Err(format!(
                "SHA-256 integrity failure on staged artifact `{safe_fname}`: expected {}, got {actual_sha}",
                out_desc.sha256
            ));
        }
    }

    // Route post-processing by routine
    if is_3d_routine(&receipt.routine) {
        return ingest_trellis2_3d_outputs(mcp, &job_id, &local_job_dir, &receipt, stored_req.as_ref(), &arg);
    } else if receipt.routine == "ontology_compile_q42" {
        return ingest_ontology_q42_outputs(mcp, &job_id, &local_job_dir, &receipt, stored_req.as_ref(), &arg);
    }

    Ok(json!({
        "schema": "synth.colab-ingest-receipt/v1",
        "status": "ingested",
        "job_id": job_id,
        "routine": receipt.routine,
        "local_staging_dir": local_job_dir,
        "receipt": receipt,
    }))
}

fn ingest_trellis2_3d_outputs(
    mcp: Option<&StandaloneSynthMcp>,
    job_id: &str,
    local_job_dir: &Path,
    receipt: &ColabJobReceipt,
    stored_req: Option<&ColabJobRequest>,
    arg: &ColabArg,
) -> Result<Value, String> {
    let ctx = stored_req.and_then(|r| r.conversion_context.as_ref());
    let domain = arg
        .domain
        .as_deref()
        .or_else(|| ctx.and_then(|c| c.domain.as_deref()));
    let taxon = arg
        .taxon
        .as_deref()
        .or_else(|| ctx.and_then(|c| c.taxon.as_deref()));
    let fiction = arg.fiction.or_else(|| ctx.and_then(|c| c.fiction));
    let evidence_q42 = arg
        .evidence_q42
        .as_deref()
        .or_else(|| ctx.and_then(|c| c.evidence_q42.as_deref()));

    // Re-enforce biosphere evidence gate at ingest time before running 3D conversion
    let evidence_sha = enforce_biosphere_gate_for_routine(
        mcp,
        &receipt.routine,
        domain,
        taxon,
        fiction,
        evidence_q42,
    )?
    .or_else(|| ctx.and_then(|c| c.evidence_q42_sha256.clone()));

    let glb_desc = receipt
        .outputs
        .iter()
        .find(|o| o.filename.ends_with(".glb") || o.role == "mesh_glb")
        .ok_or("Completed TRELLIS.2 job receipt does not contain a .glb output")?;
    let safe_glb_name = validate_safe_component(&glb_desc.filename, "glb filename")?;
    let glb_path = local_job_dir.join(&safe_glb_name);

    // Load optional .ovoxel.json sidecar (containing M64HoloRecord clusters + VOX0 DAG nodes)
    let ovoxel_json: Option<Value> = receipt
        .outputs
        .iter()
        .find(|o| o.filename.ends_with(".ovoxel.json") || o.role == "ovoxel_sidecar")
        .and_then(|desc| validate_safe_component(&desc.filename, "ovoxel filename").ok())
        .and_then(|fname| fs::read(local_job_dir.join(fname)).ok())
        .and_then(|bytes| serde_json::from_slice(&bytes).ok())
        .or_else(|| receipt.ovoxel_summary.clone());

    let auto_convert = arg
        .auto_convert_10d
        .or_else(|| ctx.map(|c| c.auto_convert_10d))
        .unwrap_or(true);

    if !auto_convert {
        return Ok(json!({
            "schema": "synth.colab-ingest-receipt/v1",
            "status": "staged_without_conversion",
            "job_id": job_id,
            "glb_path": glb_path,
            "ovoxel_present": ovoxel_json.is_some(),
            "receipt": receipt,
        }));
    }

    // Resolve target out_dir: explicit out_dir, or domain-routed poiesis/<slug> / eco-intent/<slug>, or SYNTH_OUTPUT_ROOT/<job_id>-10d
    let target_out_dir: PathBuf = if let Some(explicit_out) = arg
        .out_dir
        .as_deref()
        .or_else(|| ctx.and_then(|c| c.out_dir.as_deref()))
    {
        if explicit_out.contains("..") {
            return Err("out_dir must not contain '..'".into());
        }
        PathBuf::from(explicit_out)
    } else if let Some(slug_raw) = arg
        .slug
        .as_deref()
        .or_else(|| ctx.and_then(|c| c.slug.as_deref()))
    {
        let safe_slug = validate_safe_component(slug_raw, "slug")?;
        let base_repo = if domain == Some("biosphere") {
            r"C:\github\eco-intent"
        } else {
            r"C:\github\poiesis"
        };
        Path::new(base_repo).join(&safe_slug).join(&job_id)
    } else {
        let out_root = std::env::var("SYNTH_OUTPUT_ROOT")
            .ok()
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| r"C:\github\synth\production".to_owned());
        Path::new(out_root.trim()).join(format!("{job_id}-10d"))
    };

    let spdx = arg
        .spdx_license
        .as_deref()
        .or_else(|| ctx.map(|c| c.spdx_license.as_str()))
        .ok_or("spdx_license is required for 3D conversion")?;
    let creator = arg
        .creator
        .as_deref()
        .or_else(|| ctx.map(|c| c.creator.as_str()))
        .unwrap_or("Timothy Charles Holborn");
    let attribution = arg
        .attribution_notice
        .as_deref()
        .or_else(|| ctx.map(|c| c.attribution_notice.as_str()))
        .ok_or("attribution_notice is required for 3D conversion")?;
    let asset_uri = arg
        .asset_uri
        .as_deref()
        .or_else(|| ctx.map(|c| c.asset_uri.as_str()))
        .unwrap_or("urn:qualia:colab:asset");
    let domain_id = arg
        .domain_id
        .or_else(|| ctx.map(|c| c.domain_id))
        .unwrap_or(1);

    let gpu_label = if receipt.colab_hardware.gpu_name.is_empty() {
        "Google Colab Ultra GPU".to_owned()
    } else {
        format!(
            "colab-ultra:{} ({} MiB VRAM, CUDA {})",
            receipt.colab_hardware.gpu_name,
            receipt.colab_hardware.vram_total_mib,
            receipt.colab_hardware.cuda_version
        )
    };

    let (default_agent_id, default_model_name, default_model_ver) =
        model_metadata_for_routine(&receipt.routine);
    let agent_id = receipt
        .model_provenance
        .get("model_id")
        .and_then(Value::as_str)
        .unwrap_or(default_agent_id);

    let overrides = Convert3dProvenanceOverrides {
        human_actor: Some(HumanActorRecord {
            did: "did:q42:person:timothy-charles-holborn".into(),
            display_name: creator.to_owned(),
            role: "Steward".into(),
            organization: Some("Webizen / Qualia".into()),
            jurisdiction: Some("AU".into()),
            work_status: "SelfFunded".into(),
        }),
        software_agent: Some(SoftwareAgentRecord {
            agent_id: agent_id.to_owned(),
            model_name: default_model_name.into(),
            model_version: Some(default_model_ver.into()),
            role: "conversion".into(),
            within_validated_scope: true,
        }),
        harness: Some(HarnessEnvironmentRecord {
            harness_name: "Webizen AI Synth Colab Ultra Burst Worker".into(),
            synth_version: crate::agent_index::RELEASE_VERSION.into(),
            synth_binary_sha256: None,
            gpu_backend: gpu_label,
            quality_profile: "production16".into(),
        }),
        economics: Some(RevisionEconomicsTelemetry {
            currency: "AUD".into(),
            work_status: "SelfFunded".into(),
            local_inference: false,
            wall_time_ms: receipt.timing.wall_time_ms,
            peak_ram_bytes: receipt
                .colab_hardware
                .vram_peak_mib
                .max(receipt.colab_hardware.system_ram_peak_mib)
                * 1024
                * 1024,
            estimated_energy_joules: receipt.timing.estimated_energy_joules,
            ..Default::default()
        }),
        epistemic_horizon_q42_sha256: evidence_sha,
        ovoxel_sidecar_json: ovoxel_json,
        summary_override: Some(format!(
            "Colab Ultra {} ({agent_id}) generation & Holographic .10d v2 conversion (job {job_id}) under license {spdx}",
            receipt.routine
        )),
    };

    let convert_args = json!({
        "source_path": glb_path.to_string_lossy(),
        "out_dir": target_out_dir.to_string_lossy(),
        "format": "glb",
        "asset_uri": asset_uri,
        "domain_id": domain_id,
        "spdx_license": spdx,
        "creator": creator,
        "attribution_notice": attribution,
        "origin_uri": arg.origin_uri.as_deref().or_else(|| ctx.and_then(|c| c.origin_uri.as_deref())),
        "embed_full_source_bytes": arg.embed_full_source_bytes.or_else(|| ctx.map(|c| c.embed_full_source_bytes)).unwrap_or(false),
    });

    let convert_receipt = convert_3d_with_provenance(convert_args, overrides)?;

    Ok(json!({
        "schema": "synth.colab-ingest-receipt/v1",
        "status": "converted_10d_v2",
        "job_id": job_id,
        "routine": receipt.routine,
        "colab_hardware": receipt.colab_hardware,
        "timing": receipt.timing,
        "convert_3d_receipt": convert_receipt,
    }))
}

fn ingest_ontology_q42_outputs(
    mcp: Option<&StandaloneSynthMcp>,
    job_id: &str,
    local_job_dir: &Path,
    receipt: &ColabJobReceipt,
    stored_req: Option<&ColabJobRequest>,
    arg: &ColabArg,
) -> Result<Value, String> {
    let q42_outputs: Vec<&ColabOutputDescriptor> = receipt
        .outputs
        .iter()
        .filter(|o| o.filename.ends_with(".q42"))
        .collect();
    if q42_outputs.is_empty() {
        return Err("Completed ontology_compile_q42 receipt contains no .q42 output files".into());
    }

    // Verify Q42\0 magic header on every .q42 shard/file
    for q_out in &q42_outputs {
        let safe_name = validate_safe_component(&q_out.filename, "q42 filename")?;
        let p = local_job_dir.join(&safe_name);
        let bytes = fs::read(&p).map_err(|e| format!("Failed reading {safe_name}: {e}"))?;
        if bytes.len() < 64 || &bytes[0..4] != b"Q42\x00" {
            return Err(format!("Staged file `{safe_name}` is not a valid Q42 volume (missing Q42\\0 magic)"));
        }
    }

    // If a multi-segment root volume (`*-root.q42`) is present, verify the volume shard set
    let mut volume_set_verified = false;
    if let Some(root_desc) = q42_outputs.iter().find(|o| o.filename.ends_with("-root.q42")) {
        let root_path = local_job_dir.join(&root_desc.filename);
        let report = qualia_core_db::q42::q42_volume::verify_volume_set_from_root(
            &root_path,
            qualia_core_db::q42::q42_volume::VerifyLevel::Full,
        )
        .map_err(|e| format!("Multi-shard Q42 volume set verification failed: {e}"))?;
        if report.overall != qualia_core_db::q42::q42_volume::CheckStatus::Pass {
            return Err(format!(
                "Multi-shard Q42 volume set verification did not pass: {:?}",
                report.overall
            ));
        }
        volume_set_verified = true;
    }

    // Copy verified .q42 files (and any companion _refs.ttl, _shapes.shacl.ttl, _rules.n3) into SYNTH_OUTPUT_ROOT
    let out_root: PathBuf = if let Some(server) = mcp {
        if let Some(out) = &server.output_root {
            out. destination_with_ext("colab-probe.q42", ".q42", "probe")
                .ok()
                .and_then(|p| p.parent().map(Path::to_path_buf))
                .unwrap_or_else(|| local_job_dir.to_path_buf())
        } else {
            local_job_dir.to_path_buf()
        }
    } else if let Ok(raw) = std::env::var("SYNTH_OUTPUT_ROOT") {
        PathBuf::from(raw.trim())
    } else {
        local_job_dir.to_path_buf()
    };

    let mut installed_files = Vec::new();
    for out_desc in &receipt.outputs {
        let safe_name = validate_safe_component(&out_desc.filename, "output filename")?;
        let src_p = local_job_dir.join(&safe_name);
        let dest_p = out_root.join(&safe_name);
        if src_p != dest_p {
            if dest_p.exists() {
                return Err(format!(
                    "Create-only destination already exists in SYNTH_OUTPUT_ROOT: {}",
                    dest_p.display()
                ));
            }
            fs::copy(&src_p, &dest_p)
                .map_err(|e| format!("Failed copying {safe_name} to SYNTH_OUTPUT_ROOT: {e}"))?;
        }
        installed_files.push(dest_p);
    }

    let ingest_graph = arg
        .ingest_to_graph
        .or_else(|| {
            stored_req
                .and_then(|r| r.params.get("ingest_to_graph"))
                .and_then(Value::as_bool)
        })
        .unwrap_or(false);

    if ingest_graph {
        for p in &installed_files {
            if p.extension().and_then(|s| s.to_str()) == Some("q42") {
                let quins = qualia_core_db::ontology_loader::load_q42_file(p);
                qualia_core_db::daemon_graph::extend_with_ontology_quins_slice(&quins);
            }
        }
    }

    Ok(json!({
        "schema": "synth.colab-ingest-receipt/v1",
        "status": "verified_q42",
        "job_id": job_id,
        "routine": receipt.routine,
        "multi_shard_volume_set_verified": volume_set_verified,
        "ingested_to_graph": ingest_graph,
        "installed_files": installed_files,
        "colab_hardware": receipt.colab_hardware,
        "timing": receipt.timing,
    }))
}

fn list_colab_jobs() -> Result<Value, String> {
    let inbox = colab_inbox_root()?;
    let mut jobs = Vec::new();
    if let Ok(rd) = fs::read_dir(&inbox) {
        for entry in rd.flatten() {
            let p = entry.path();
            if !p.is_dir() {
                continue;
            }
            let Some(name) = p.file_name().and_then(|s| s.to_str()) else {
                continue;
            };
            if name == "exported-notebook" {
                continue;
            }
            let has_receipt = p.join("receipt.json").is_file();
            let status = if has_receipt {
                fs::read(p.join("receipt.json"))
                    .ok()
                    .and_then(|b| serde_json::from_slice::<Value>(&b).ok())
                    .and_then(|v| v.get("status").and_then(Value::as_str).map(str::to_owned))
                    .unwrap_or_else(|| "unknown".into())
            } else {
                "submitted".into()
            };
            jobs.push(json!({
                "job_id": name,
                "status": status,
                "has_receipt": has_receipt,
                "staging_dir": p,
            }));
        }
    }
    Ok(json!({
        "schema": "synth.colab-jobs-list/v1",
        "count": jobs.len(),
        "jobs": jobs,
    }))
}

fn cancel_colab_job(arg: ColabArg) -> Result<Value, String> {
    let raw_id = arg.job_id.as_deref().ok_or("cancel requires job_id")?;
    let job_id = validate_safe_component(raw_id, "job_id")?;
    let cfg = load_colab_config();
    let mut removed_from_drive = false;
    if let Some(dq) = cfg.drive_queue_dir.as_deref() {
        let pending_dir = Path::new(dq).join("queue").join("pending").join(&job_id);
        if pending_dir.is_dir() {
            let _ = fs::remove_dir_all(&pending_dir);
            removed_from_drive = true;
        }
    }
    Ok(json!({
        "schema": "synth.colab-cancel/v1",
        "job_id": job_id,
        "status": "cancelled",
        "removed_from_drive_pending": removed_from_drive,
    }))
}

/// CLI entry point for `synth colab ...`.
/// Never accepts `--token <secret>` on the command line; reads token from `--token-stdin`
/// or `SYNTH_COLAB_TOKEN` environment variable so secrets never leak to shell history.
pub fn run_cli(args: &[String]) -> Result<(), String> {
    if args.is_empty() || args.iter().any(|a| a == "--help" || a == "-h") {
        println!(
            "Usage: synth colab <subcommand> [options]\n\
             Subcommands:\n\
               status              Check Colab Ultra worker configuration, Drive queue, and HTTP health\n\
               routines            List supported Colab Ultra heavy-compute burst routines\n\
               configure           Configure endpoint URL, Drive queue dir, idle timeout, and Bearer token\n\
                                   Options: [--url HTTPS_URL] [--drive-dir DIR] [--transport auto|http|drive_queue]\n\
                                            [--idle-timeout MINUTES] [--token-stdin]\n\
                                   (Note: Pass Bearer token via --token-stdin or SYNTH_COLAB_TOKEN env var)\n\
               export-notebook     Export synth_colab_ultra_worker.ipynb and colab_worker.py [--out DIR]\n\
               submit              Submit a job: --routine trellis2_image_to_3d|trellis2_multiview_to_3d|ontology_compile_q42\n\
                                   [--image PATH] [--resolution 512|1024|1536] [--license SPDX] [--attribution TEXT]\n\
                                   [--acronym FMA] [--source-url URL] [--out-q42 FILE.q42]\n\
               poll                Poll job status: --job-id ID\n\
               ingest              Verify and ingest completed job outputs into .10d v2 / .q42: --job-id ID [--out-dir DIR]\n\
               list                List staged Colab jobs in .synth/inbox/colab/"
        );
        return Ok(());
    }

    let sub = args[0].as_str();
    let mut colab_arg = ColabArg {
        action: Some(match sub {
            "export-notebook" => "export_notebook".into(),
            "list" => "list_jobs".into(),
            other => other.to_owned(),
        }),
        ..ColabArg::default()
    };

    let mut read_token_stdin = false;
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--token" => {
                return Err(
                    "Refusing `--token` on the CLI to protect shell history and process lists. Use `--token-stdin` or set `SYNTH_COLAB_TOKEN`."
                        .into(),
                );
            }
            "--token-stdin" => {
                read_token_stdin = true;
                i += 1;
            }
            "--url" | "--endpoint" if i + 1 < args.len() => {
                colab_arg.endpoint_url = Some(args[i + 1].clone());
                i += 2;
            }
            "--drive-dir" | "--drive-queue-dir" if i + 1 < args.len() => {
                colab_arg.drive_queue_dir = Some(args[i + 1].clone());
                i += 2;
            }
            "--transport" if i + 1 < args.len() => {
                colab_arg.transport = Some(args[i + 1].clone());
                i += 2;
            }
            "--idle-timeout" if i + 1 < args.len() => {
                colab_arg.idle_timeout_minutes = Some(
                    args[i + 1]
                        .parse::<u32>()
                        .map_err(|_| "--idle-timeout must be a positive integer (minutes)")?,
                );
                i += 2;
            }
            "--routine" if i + 1 < args.len() => {
                colab_arg.routine = Some(args[i + 1].clone());
                i += 2;
            }
            "--job-id" | "--id" if i + 1 < args.len() => {
                colab_arg.job_id = Some(args[i + 1].clone());
                i += 2;
            }
            "--image" | "--image-path" if i + 1 < args.len() => {
                colab_arg.image_path = Some(args[i + 1].clone());
                i += 2;
            }
            "--mesh" | "--mesh-path" if i + 1 < args.len() => {
                colab_arg.mesh_path = Some(args[i + 1].clone());
                i += 2;
            }
            "--mode" if i + 1 < args.len() => {
                colab_arg.mode = Some(args[i + 1].clone());
                i += 2;
            }
            "--target-parts" if i + 1 < args.len() => {
                colab_arg.target_parts = Some(
                    args[i + 1]
                        .parse::<u32>()
                        .map_err(|_| "--target-parts must be an integer (2..=64)")?,
                );
                i += 2;
            }
            "--max-joints" if i + 1 < args.len() => {
                colab_arg.max_joints = Some(
                    args[i + 1]
                        .parse::<u32>()
                        .map_err(|_| "--max-joints must be an integer (2..=256)")?,
                );
                i += 2;
            }
            "--resolution" if i + 1 < args.len() => {
                colab_arg.resolution = Some(
                    args[i + 1]
                        .parse::<u32>()
                        .map_err(|_| "--resolution must be 512, 1024, or 1536")?,
                );
                i += 2;
            }
            "--decimation-target" if i + 1 < args.len() => {
                colab_arg.decimation_target = Some(
                    args[i + 1]
                        .parse::<u32>()
                        .map_err(|_| "--decimation-target must be an integer")?,
                );
                i += 2;
            }
            "--texture-size" if i + 1 < args.len() => {
                colab_arg.texture_size = Some(
                    args[i + 1]
                        .parse::<u32>()
                        .map_err(|_| "--texture-size must be 1024, 2048, or 4096")?,
                );
                i += 2;
            }
            "--license" | "--spdx" if i + 1 < args.len() => {
                colab_arg.spdx_license = Some(args[i + 1].clone());
                i += 2;
            }
            "--attribution" if i + 1 < args.len() => {
                colab_arg.attribution_notice = Some(args[i + 1].clone());
                i += 2;
            }
            "--creator" if i + 1 < args.len() => {
                colab_arg.creator = Some(args[i + 1].clone());
                i += 2;
            }
            "--domain" if i + 1 < args.len() => {
                colab_arg.domain = Some(args[i + 1].clone());
                i += 2;
            }
            "--taxon" if i + 1 < args.len() => {
                colab_arg.taxon = Some(args[i + 1].clone());
                i += 2;
            }
            "--evidence-q42" if i + 1 < args.len() => {
                colab_arg.evidence_q42 = Some(args[i + 1].clone());
                i += 2;
            }
            "--slug" if i + 1 < args.len() => {
                colab_arg.slug = Some(args[i + 1].clone());
                i += 2;
            }
            "--out-dir" | "--out" if i + 1 < args.len() => {
                if colab_arg.action.as_deref() == Some("export_notebook") {
                    colab_arg.notebook_out_path = Some(args[i + 1].clone());
                } else {
                    colab_arg.out_dir = Some(args[i + 1].clone());
                }
                i += 2;
            }
            "--acronym" if i + 1 < args.len() => {
                colab_arg.acronym = Some(args[i + 1].clone());
                i += 2;
            }
            "--source-url" if i + 1 < args.len() => {
                colab_arg.source_url = Some(args[i + 1].clone());
                i += 2;
            }
            "--source-path" if i + 1 < args.len() => {
                colab_arg.source_path = Some(args[i + 1].clone());
                i += 2;
            }
            "--out-q42" if i + 1 < args.len() => {
                colab_arg.out_q42 = Some(args[i + 1].clone());
                i += 2;
            }
            other => {
                return Err(format!(
                    "Unknown `synth colab` option `{other}` (use `synth colab --help`)"
                ));
            }
        }
    }

    if read_token_stdin {
        let mut buf = String::new();
        std::io::stdin()
            .read_to_string(&mut buf)
            .map_err(|e| format!("Failed reading token from stdin: {e}"))?;
        let trimmed = buf.trim().to_owned();
        if trimmed.is_empty() {
            return Err("No token provided on stdin for --token-stdin".into());
        }
        colab_arg.api_token = Some(trimmed);
    }

    let report = dispatch(None, colab_arg)?;
    println!(
        "{}",
        serde_json::to_string_pretty(&report).map_err(|e| e.to_string())?
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn safe_component_blocks_traversal_and_separators() {
        assert!(validate_safe_component("colab-17000000-deadbeef", "job_id").is_ok());
        assert!(validate_safe_component("output_asset.glb", "filename").is_ok());
        assert!(validate_safe_component("../escape.glb", "filename").is_err());
        assert!(validate_safe_component("sub/dir.glb", "filename").is_err());
        assert!(validate_safe_component("sub\\dir.glb", "filename").is_err());
        assert!(validate_safe_component(".hidden", "filename").is_err());
        assert!(validate_safe_component("C:drive.glb", "filename").is_err());
    }

    #[test]
    fn endpoint_url_requires_https_except_loopback() {
        assert!(validate_worker_endpoint_url("https://my-tunnel.trycloudflare.com").is_ok());
        assert!(validate_worker_endpoint_url("http://127.0.0.1:8099").is_ok());
        assert!(validate_worker_endpoint_url("http://localhost:8099").is_ok());
        assert!(validate_worker_endpoint_url("http://remote-host.example.com:8099").is_err());
        assert!(validate_worker_endpoint_url("https://host.example.com?token=secret").is_err());
    }

    #[test]
    fn routines_catalog_lists_all_six_canonical_routines_and_decision_matrix() {
        let cat = routines_catalog_json();
        assert!(cat["decision_matrix"].is_object());
        let arr = cat["routines"].as_array().expect("routines array");
        assert_eq!(arr.len(), 6);
        assert_eq!(arr[0]["id"], "trellis2_image_to_3d");
        assert_eq!(arr[1]["id"], "trellis2_multiview_to_3d");
        assert_eq!(arr[2]["id"], "hunyuan3d_shape_and_pbr");
        assert_eq!(arr[3]["id"], "partcrafter_decompose_3d");
        assert_eq!(arr[4]["id"], "unirig_auto_skeleton");
        assert_eq!(arr[5]["id"], "ontology_compile_q42");
    }

    #[test]
    fn cli_rejects_plaintext_token_flag() {
        let err = run_cli(&[
            "configure".into(),
            "--url".into(),
            "https://example.trycloudflare.com".into(),
            "--token".into(),
            "secret123".into(),
        ])
        .expect_err("must reject --token on CLI");
        assert!(err.contains("--token-stdin"));
    }

    #[test]
    fn biosphere_nonfiction_blocks_trellis2_without_evidence_and_allows_ontology_compile_q42() {
        // Non-fiction biosphere 3D generation without evidence_q42 must be blocked before spending Colab Compute Units
        let err = enforce_biosphere_gate_for_routine(
            None,
            "trellis2_image_to_3d",
            Some("biosphere"),
            Some("nonexistent-taxon-xyz"),
            Some(false),
            None,
        )
        .expect_err("biosphere non-fiction without evidence must block trellis2_image_to_3d");
        assert!(err.contains("requires sourced evidence"));

        // Conversely, ontology_compile_q42 is the evidence path (`compile_q42`) and must be allowed without prior evidence
        let ok = enforce_biosphere_gate_for_routine(
            None,
            "ontology_compile_q42",
            Some("biosphere"),
            Some("nonexistent-taxon-xyz"),
            Some(false),
            None,
        );
        assert!(ok.is_ok());
    }

    #[test]
    fn mock_drive_queue_trellis2_round_trip_seals_holographic_10d_v2_and_changelog() {
        use qualia_core_db::container_10d::{
            decode_container_manifold_identity, header::Container10dHeader,
            integrity::verify_whole_file_crc32c, TypedFieldRecord,
            FIELD_KIND_HOLOGRAPHIC_PHASE_SPACE,
        };

        let stamp = format!("test-colab-dq-{}-{}", std::process::id(), now_secs());
        let temp_base = std::env::temp_dir().join(&stamp);
        let drive_root = temp_base.join("drive");
        let out_10d_dir = temp_base.join("converted-10d");
        fs::create_dir_all(drive_root.join("queue").join("completed")).unwrap();

        // Build a minimal valid ASCII STL mesh saved with .stl extension (or minimal OBJ) so convert_3d_with_provenance parses triangles
        let job_id = format!("colab-test-{}", std::process::id());
        let completed_job_dir = drive_root.join("queue").join("completed").join(&job_id);
        fs::create_dir_all(&completed_job_dir).unwrap();

        // Minimal valid glTF 2.0 binary (.glb) or OBJ/STL; let's build a minimal valid binary GLB with 1 triangle!
        let positions: [f32; 9] = [0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0];
        let indices: [u16; 3] = [0, 1, 2];
        let mut bin_chunk = Vec::new();
        for f in positions {
            bin_chunk.extend_from_slice(&f.to_le_bytes());
        }
        for idx in indices {
            bin_chunk.extend_from_slice(&idx.to_le_bytes());
        }
        while bin_chunk.len() % 4 != 0 {
            bin_chunk.push(0);
        }
        let gltf_json = json!({
            "asset": { "version": "2.0" },
            "scene": 0,
            "scenes": [{ "nodes": [0] }],
            "nodes": [{ "name": "TrellisRoot", "mesh": 0 }],
            "meshes": [{
                "name": "TrellisMesh",
                "primitives": [{
                    "attributes": { "POSITION": 0 },
                    "indices": 1
                }]
            }],
            "buffers": [{ "byteLength": bin_chunk.len() }],
            "bufferViews": [
                { "buffer": 0, "byteOffset": 0, "byteLength": 36 },
                { "buffer": 0, "byteOffset": 36, "byteLength": 6 }
            ],
            "accessors": [
                {
                    "bufferView": 0,
                    "componentType": 5126,
                    "count": 3,
                    "type": "VEC3",
                    "min": [0.0, 0.0, 0.0],
                    "max": [1.0, 1.0, 0.0]
                },
                {
                    "bufferView": 1,
                    "componentType": 5123,
                    "count": 3,
                    "type": "SCALAR"
                }
            ]
        });
        let mut json_bytes = serde_json::to_vec(&gltf_json).unwrap();
        while json_bytes.len() % 4 != 0 {
            json_bytes.push(b' ');
        }
        let total_len = 12 + 8 + json_bytes.len() + 8 + bin_chunk.len();
        let mut glb_bytes = Vec::with_capacity(total_len);
        glb_bytes.extend_from_slice(b"glTF");
        glb_bytes.extend_from_slice(&2u32.to_le_bytes());
        glb_bytes.extend_from_slice(&(total_len as u32).to_le_bytes());
        glb_bytes.extend_from_slice(&(json_bytes.len() as u32).to_le_bytes());
        glb_bytes.extend_from_slice(&0x4E4F534Au32.to_le_bytes()); // JSON
        glb_bytes.extend_from_slice(&json_bytes);
        glb_bytes.extend_from_slice(&(bin_chunk.len() as u32).to_le_bytes());
        glb_bytes.extend_from_slice(&0x004E4942u32.to_le_bytes()); // BIN\0
        glb_bytes.extend_from_slice(&bin_chunk);

        let glb_sha = sha256_hex(&glb_bytes);
        fs::write(completed_job_dir.join("output.glb"), &glb_bytes).unwrap();

        let ovoxel_sidecar = json!({
            "schema": "synth.ovoxel-volumetric-sidecar/v1",
            "target_10d_sections": { "holo_field_kind": "0x0306", "vox0_section_type": 22 },
            "voxel_resolution": 1024,
            "voxel_size_m": 0.0009765625,
            "occupied_voxel_sample_count": 128,
            "aabb_min": [-0.5, -0.5, -0.5],
            "aabb_max": [0.5, 0.5, 0.5],
            "octant_density": [0.125, 0.125, 0.125, 0.125, 0.125, 0.125, 0.125, 0.125],
            "m64_holo_segments": [{
                "segment_index": 0,
                "local_origin": [0.0, 0.0, 0.0],
                "scale_bounds": [0.5, 0.5, 0.5],
                "sdf_harmonics_k0_k11": [1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
                "max_raymarch_steps": 24
            }],
            "vox0_dag_nodes": [{
                "child_mask": 0,
                "leaf_mask": 255,
                "coord": [0.0, 0.0, 0.0],
                "base_color_rgb": [0.8, 0.7, 0.6],
                "roughness": 0.4,
                "metallic": 0.1,
                "opacity": 1.0
            }]
        });
        let ovoxel_bytes = serde_json::to_vec_pretty(&ovoxel_sidecar).unwrap();
        let ovoxel_sha = sha256_hex(&ovoxel_bytes);
        fs::write(completed_job_dir.join("output.ovoxel.json"), &ovoxel_bytes).unwrap();

        let receipt = ColabJobReceipt {
            schema: COLAB_JOB_RECEIPT_SCHEMA.into(),
            job_id: job_id.clone(),
            routine: "trellis2_image_to_3d".into(),
            status: "completed".into(),
            error: None,
            colab_hardware: ColabHardwareTelemetry {
                gpu_name: "NVIDIA A100-SXM4-40GB".into(),
                vram_total_mib: 40960,
                vram_peak_mib: 22400,
                system_ram_total_mib: 85000,
                system_ram_peak_mib: 12000,
                cuda_version: "12.4".into(),
                torch_version: "2.5.1+cu124".into(),
            },
            timing: ColabTimingTelemetry {
                started_at_epoch_s: 1700000000,
                completed_at_epoch_s: 1700000022,
                wall_time_ms: 22000,
                estimated_energy_joules: Some(6600.0),
            },
            model_provenance: json!({
                "model_id": "microsoft/TRELLIS.2-4B",
                "model_license": "MIT",
                "seed": 42,
                "resolution": 1024
            }),
            outputs: vec![
                ColabOutputDescriptor {
                    role: "mesh_glb".into(),
                    filename: "output.glb".into(),
                    sha256: glb_sha,
                    bytes: glb_bytes.len() as u64,
                },
                ColabOutputDescriptor {
                    role: "ovoxel_sidecar".into(),
                    filename: "output.ovoxel.json".into(),
                    sha256: ovoxel_sha,
                    bytes: ovoxel_bytes.len() as u64,
                },
            ],
            ovoxel_summary: Some(ovoxel_sidecar),
        };
        fs::write(
            completed_job_dir.join("receipt.json"),
            serde_json::to_vec_pretty(&receipt).unwrap(),
        )
        .unwrap();

        std::env::set_var("SYNTH_BOOKMARK_ROOT", &temp_base);
        let res = ingest_colab_job(
            None,
            ColabArg {
                action: Some("ingest".into()),
                job_id: Some(job_id.clone()),
                drive_queue_dir: Some(drive_root.to_string_lossy().into_owned()),
                out_dir: Some(out_10d_dir.to_string_lossy().into_owned()),
                spdx_license: Some("CC-BY-NC-ND-4.0".into()),
                creator: Some("Timothy Charles Holborn".into()),
                attribution_notice: Some("Copyright (c) 2026 Timothy Charles Holborn".into()),
                asset_uri: Some("urn:qualia:poiesis:test-colab-trellis2".into()),
                auto_convert_10d: Some(true),
                ..ColabArg::default()
            },
        )
        .expect("ingest_colab_job must succeed");

        assert_eq!(res["status"], "converted_10d_v2");
        let arch_path = PathBuf::from(
            res["convert_3d_receipt"]["archetype_10d_path"]
                .as_str()
                .expect("archetype_10d_path"),
        );
        assert!(arch_path.is_file());
        assert!(out_10d_dir.join("output.ovoxel.json").is_file());
        assert!(out_10d_dir.join("output-provenance.n3").is_file());
        assert!(out_10d_dir.join("manifest.q42.yaml").is_file());

        // Verify sealed .10d v2 CRC32C and HOLO 0x0306 presence
        let mut arch_bytes = fs::read(&arch_path).unwrap();
        assert!(verify_whole_file_crc32c(&mut arch_bytes).is_ok());
        let hdr = Container10dHeader::parse(&arch_bytes).unwrap();
        let mut scratch = [TypedFieldRecord::default(); 32];
        let mut fields = [TypedFieldRecord::default(); 32];
        let (_, fcount) =
            decode_container_manifold_identity(&arch_bytes, &hdr, &[], &mut scratch, &mut fields)
                .unwrap();
        assert!(fields[..fcount]
            .iter()
            .any(|f| f.kind == FIELD_KIND_HOLOGRAPHIC_PHASE_SPACE));

        // Verify .synth/changelog.json records TRELLIS.2-4B SoftwareAgentRecord and Colab A100 HarnessEnvironmentRecord
        let cl_bytes = fs::read(out_10d_dir.join(".synth").join("changelog.json")).unwrap();
        let cl_val: Value = serde_json::from_slice(&cl_bytes).unwrap();
        let rev0 = &cl_val["revisions"][0];
        assert_eq!(rev0["agent"]["agent_id"], "microsoft/TRELLIS.2-4B");
        assert!(rev0["harness"]["gpu_backend"]
            .as_str()
            .unwrap()
            .contains("NVIDIA A100-SXM4-40GB"));
        assert_eq!(rev0["economics"]["wall_time_ms"], 22000);

        let _ = fs::remove_dir_all(&temp_base);
    }
}
