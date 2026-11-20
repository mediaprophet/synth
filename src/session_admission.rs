//! Multi-Agent Session Registry & Cross-Process Resource Admission Controller (`REQ-5.1`..`5.5`),
//! plus Unified Build Identity & Binary Drift Detection (`REQ-1.1`..`1.2`) and Local GPU / Port `4242`
//! Runtime Profile Telemetry (`REQ-4.1`..`4.3`).
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs,
    io::Read,
    path::{Path, PathBuf},
    sync::{Mutex, OnceLock},
    thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

pub const SLOTS_SCHEMA: &str = "synth.resource-admission-slots/v1";
pub const ACTIVE_LEASE_TTL_SECS: u64 = 180;
pub const IDLE_SESSION_TTL_SECS: u64 = 600;
pub const DEFAULT_MAX_CONCURRENT_SESSIONS: usize = 2;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSlotLease {
    pub session_id: String,
    pub agent_id: String,
    pub pid: u32,
    pub project_root: Option<String>,
    pub quality_profile: String,
    pub reserved_ram_mib: u64,
    pub reserved_vram_mib: u64,
    pub created_at_epoch_s: u64,
    pub last_active_epoch_s: u64,
    pub lease_expires_epoch_s: u64,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SlotLedgerFile {
    pub schema: String,
    pub max_slots: usize,
    pub active_leases: Vec<SessionSlotLease>,
}

static IN_PROCESS_SESSIONS: OnceLock<Mutex<BTreeMap<String, SessionSlotLease>>> = OnceLock::new();

fn session_map() -> &'static Mutex<BTreeMap<String, SessionSlotLease>> {
    IN_PROCESS_SESSIONS.get_or_init(|| Mutex::new(BTreeMap::new()))
}

fn now_epoch_s() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn file_sha256(path: &Path) -> Option<String> {
    let mut file = fs::File::open(path).ok()?;
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 64 * 1024];
    loop {
        let n = file.read(&mut buf).ok()?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Some(format!("{:x}", hasher.finalize()))
}

/// REQ-1.1 & REQ-1.2: Unified Build Identity & Sidecar Binary Drift Check.
pub fn build_identity_and_drift_report() -> Value {
    let current_exe = std::env::current_exe().ok();
    let current_sha = current_exe.as_deref().and_then(file_sha256);
    let parent_dir = current_exe
        .as_deref()
        .and_then(|p| p.parent())
        .map(Path::to_path_buf);

    let sibling_synth_sha = parent_dir
        .as_deref()
        .and_then(|d| file_sha256(&d.join("synth.exe")));
    let sibling_mcp_sha = parent_dir
        .as_deref()
        .and_then(|d| file_sha256(&d.join("synth-mcp.exe")));

    let binary_drift_detected = match (&sibling_synth_sha, &sibling_mcp_sha) {
        (Some(a), Some(b)) => a != b,
        _ => false,
    };

    let mut features = vec!["holographic-10d-v2", "cc-by-nc-nd-4.0"];
    #[cfg(feature = "preview")]
    features.push("preview");
    #[cfg(feature = "native-ui")]
    features.push("native-ui");
    #[cfg(feature = "atlas")]
    features.push("atlas");

    json!({
        "schema": "synth.build-identity/v1",
        "product": crate::agent_index::PRODUCT_NAME,
        "release_version": crate::agent_index::RELEASE_VERSION,
        "cargo_pkg_version": env!("CARGO_PKG_VERSION"),
        "license": "CC-BY-NC-ND-4.0",
        "copyright": "Copyright (c) 2026 Timothy Charles Holborn",
        "current_exe": current_exe,
        "current_exe_sha256": current_sha,
        "sibling_synth_exe_sha256": sibling_synth_sha,
        "sibling_synth_mcp_exe_sha256": sibling_mcp_sha,
        "binary_drift_detected": binary_drift_detected,
        "drift_advice": if binary_drift_detected {
            Some("synth.exe and synth-mcp.exe SHA-256 digests differ; copy synth.exe over synth-mcp.exe and reload MCP.")
        } else {
            None
        },
        "enabled_features": features,
    })
}

/// REQ-4.1 & REQ-4.2: Tiered Local GPU & Local Runtime Profile (Port 4242 / 8080 / 8787 + Native Vulkan/CUDA/Metal).
pub fn detect_runtime_and_gpu_profile(requested_backend: Option<&str>) -> Value {
    let (_, local_runtime_8080, local_runtime_4242) = crate::tray_env::probe_webizen_desktop();

    let default_backend = if cfg!(target_os = "macos") {
        "metal"
    } else {
        "vulkan"
    };
    let selected_backend = match requested_backend.map(str::trim).filter(|s| !s.is_empty()) {
        Some("auto") | None => default_backend,
        Some(other) => other,
    };

    let execution_tier = if local_runtime_4242 || local_runtime_8080 {
        "attached-local-runtime-4242"
    } else if matches!(selected_backend, "webgpu-wasm" | "webgl2-wasm" | "wasm") {
        "portable-web-wasm"
    } else {
        "local-native-gpu-standalone"
    };

    json!({
        "schema": "synth.runtime-gpu-profile/v1",
        "execution_tier": execution_tier,
        "selected_gpu_backend": selected_backend,
        "supported_backends": ["vulkan", "cuda", "metal", "dx12", "webgpu-wasm", "webgl2-wasm", "cpu"],
        "webizen_desktop_probe": {
            "port_4242_ws_online": local_runtime_4242,
            "port_8080_http_online": local_runtime_8080,
            "delegation_active": local_runtime_4242,
        },
        "holographic_10d_pipeline": {
            "proxy_lens_triangles": 12,
            "max_raymarch_steps": 32,
            "klod_tiers_supported": ["KLOD0", "KLOD1", "KLOD2", "KLOD3"]
        }
    })
}

fn slots_ledger_path() -> Option<PathBuf> {
    let root = std::env::var("SYNTH_OUTPUT_ROOT").ok()?;
    let trimmed = root.trim();
    if trimmed.is_empty() {
        return None;
    }
    let dir = Path::new(trimmed).join(".synth");
    let _ = fs::create_dir_all(&dir);
    Some(dir.join("slots.json"))
}

fn max_concurrent_sessions() -> usize {
    std::env::var("SYNTH_MAX_CONCURRENT_SESSIONS")
        .ok()
        .and_then(|v| v.trim().parse::<usize>().ok())
        .filter(|&n| n >= 1 && n <= 32)
        .unwrap_or(DEFAULT_MAX_CONCURRENT_SESSIONS)
}

fn profile_ram_vram_mib(quality_profile: &str) -> (u64, u64) {
    match quality_profile {
        "preview" => (80, 128),
        _ => (2048, 512),
    }
}

fn load_and_prune_ledger(now: u64) -> SlotLedgerFile {
    let max_slots = max_concurrent_sessions();
    let Some(path) = slots_ledger_path() else {
        return SlotLedgerFile {
            schema: SLOTS_SCHEMA.into(),
            max_slots,
            active_leases: Vec::new(),
        };
    };
    let mut ledger: SlotLedgerFile = fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_else(|| SlotLedgerFile {
            schema: SLOTS_SCHEMA.into(),
            max_slots,
            active_leases: Vec::new(),
        });
    ledger.schema = SLOTS_SCHEMA.into();
    ledger.max_slots = max_slots;
    // REQ-5.4: Automatic lease expiry & stale-session eviction
    ledger
        .active_leases
        .retain(|lease| lease.lease_expires_epoch_s > now);
    ledger
}

fn save_ledger(ledger: &SlotLedgerFile) {
    if let Some(path) = slots_ledger_path() {
        if let Ok(bytes) = serde_json::to_vec_pretty(ledger) {
            let _ = fs::write(path, bytes);
        }
    }
}

/// REQ-5.1..5.4: Acquire a multi-agent session slot before running a production or preview job.
/// Returns `Ok(SessionSlotLease)` if admitted, or `Err(Value)` with the structured `wait_for_slot` payload.
pub fn acquire_production_slot(
    session_id: Option<&str>,
    agent_id: Option<&str>,
    project_root: Option<&str>,
    quality_profile: &str,
    wait_ms: u64,
) -> Result<SessionSlotLease, Value> {
    let bounded_wait_ms = wait_ms.min(15_000);
    let start = SystemTime::now();
    let (req_ram, req_vram) = profile_ram_vram_mib(quality_profile);
    let sid = session_id
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_owned)
        .unwrap_or_else(|| format!("sess-{}-{}", std::process::id(), now_epoch_s()));
    let aid = agent_id
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_owned)
        .unwrap_or_else(|| format!("agent-pid-{}", std::process::id()));

    loop {
        let now = now_epoch_s();
        let mut ledger = load_and_prune_ledger(now);

        // If this exact session_id already holds a lease, refresh it
        if let Some(existing) = ledger
            .active_leases
            .iter_mut()
            .find(|l| l.session_id == sid)
        {
            existing.last_active_epoch_s = now;
            existing.lease_expires_epoch_s = now + ACTIVE_LEASE_TTL_SECS;
            let lease = existing.clone();
            save_ledger(&ledger);
            if let Ok(mut map) = session_map().lock() {
                map.insert(sid.clone(), lease.clone());
            }
            return Ok(lease);
        }

        if ledger.active_leases.len() < ledger.max_slots {
            let lease = SessionSlotLease {
                session_id: sid.clone(),
                agent_id: aid.clone(),
                pid: std::process::id(),
                project_root: project_root.map(str::to_owned),
                quality_profile: quality_profile.to_owned(),
                reserved_ram_mib: req_ram,
                reserved_vram_mib: req_vram,
                created_at_epoch_s: now,
                last_active_epoch_s: now,
                lease_expires_epoch_s: now + ACTIVE_LEASE_TTL_SECS,
                status: "active".into(),
            };
            ledger.active_leases.push(lease.clone());
            save_ledger(&ledger);
            if let Ok(mut map) = session_map().lock() {
                map.insert(sid, lease.clone());
            }
            return Ok(lease);
        }

        let elapsed_ms = start
            .elapsed()
            .map(|d| d.as_millis() as u64)
            .unwrap_or(bounded_wait_ms);
        if elapsed_ms >= bounded_wait_ms {
            let committed_ram: u64 = ledger
                .active_leases
                .iter()
                .map(|l| l.reserved_ram_mib)
                .sum();
            let active_agents: Vec<Value> = ledger
                .active_leases
                .iter()
                .map(|l| {
                    json!({
                        "agent_id": l.agent_id,
                        "session_id": l.session_id,
                        "pid": l.pid,
                        "profile": l.quality_profile,
                        "elapsed_ms": now.saturating_sub(l.created_at_epoch_s) * 1000,
                    })
                })
                .collect();
            return Err(json!({
                "status": "wait_for_slot",
                "code": "resource_slot_unavailable",
                "retry_after_ms": 5000,
                "slots": {
                    "active": ledger.active_leases.len(),
                    "max": ledger.max_slots,
                    "queued": 1,
                    "active_agents": active_agents
                },
                "resources": {
                    "committed_ram_mib": committed_ram,
                    "required_ram_mib": req_ram,
                    "required_vram_mib": req_vram,
                    "gpu_render_slots_busy": ledger.active_leases.len()
                },
                "advice": "Wait retry_after_ms and retry, or pass wait_ms (up to 15000), or downgrade quality_profile to 'preview' (80 MiB)."
            }));
        }
        thread::sleep(Duration::from_millis(100));
    }
}

/// Release an active production slot upon completion.
pub fn release_production_slot(session_id: &str) {
    let now = now_epoch_s();
    let mut ledger = load_and_prune_ledger(now);
    ledger
        .active_leases
        .retain(|l| l.session_id != session_id);
    save_ledger(&ledger);
    if let Ok(mut map) = session_map().lock() {
        map.remove(session_id);
    }
}

/// Snapshot current multi-agent session slots and admission state.
pub fn session_admission_status() -> Value {
    let now = now_epoch_s();
    let ledger = load_and_prune_ledger(now);
    save_ledger(&ledger);
    let committed_ram: u64 = ledger
        .active_leases
        .iter()
        .map(|l| l.reserved_ram_mib)
        .sum();
    json!({
        "schema": SLOTS_SCHEMA,
        "active_slots": ledger.active_leases.len(),
        "max_slots": ledger.max_slots,
        "committed_ram_mib": committed_ram,
        "active_leases": ledger.active_leases,
    })
}
