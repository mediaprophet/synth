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

/// Persisted GPU Adapter & Backend Policy (`SYNTH_GPU_PREFERENCE`, `QUALIA_RENDER_WGPU_BACKEND`, `SYNTH_REJECT_IGPU`).
/// Defaults to `discrete-only` (`reject_igpu = true`, `backend = vulkan`) so Synth never throttles
/// or locks up the on-board Intel integrated GPU on hybrid-GPU machines.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuSelectionConfig {
    pub schema: String,
    /// `"discrete-only"` (default), `"discrete"`, `"integrated"`, or `"name:<substring>"`
    pub preference: String,
    /// `"vulkan"` (default on Windows/Linux), `"dx12"`, `"metal"` (default on macOS), `"gl"`, or `"auto"`
    pub backend: String,
    /// When `true` (default), refuses `DeviceType::IntegratedGpu` so the Intel on-board GPU is never locked up.
    pub reject_igpu: bool,
    /// Optional adapter name substring filter (e.g. `"RTX"`, `"NVIDIA"`, `"Radeon"`).
    #[serde(default)]
    pub adapter_name_filter: Option<String>,
}

impl Default for GpuSelectionConfig {
    fn default() -> Self {
        Self {
            schema: "synth.gpu-selection-config/v1".into(),
            preference: "discrete-only".into(),
            backend: if cfg!(target_os = "macos") {
                "metal".into()
            } else {
                "vulkan".into()
            },
            reject_igpu: true,
            adapter_name_filter: None,
        }
    }
}

fn gpu_config_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    if let Ok(root) = std::env::var("SYNTH_OUTPUT_ROOT") {
        let trimmed = root.trim();
        if !trimmed.is_empty() {
            let dir = Path::new(trimmed).join(".synth");
            let _ = fs::create_dir_all(&dir);
            paths.push(dir.join("gpu-config.json"));
        }
    }
    let repo_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join(".synth");
    let _ = fs::create_dir_all(&repo_dir);
    let repo_cfg = repo_dir.join("gpu-config.json");
    if !paths.contains(&repo_cfg) {
        paths.push(repo_cfg);
    }
    paths
}

pub fn load_gpu_selection_config() -> GpuSelectionConfig {
    for path in gpu_config_paths() {
        if let Ok(raw) = fs::read_to_string(&path) {
            if let Ok(cfg) = serde_json::from_str::<GpuSelectionConfig>(&raw) {
                return cfg;
            }
        }
    }
    GpuSelectionConfig::default()
}

pub fn ensure_host_cuda_toolkit_path() {
    static ONCE: std::sync::OnceLock<()> = std::sync::OnceLock::new();
    ONCE.get_or_init(|| {
        let mut cuda_dir = std::env::var_os("CUDA_PATH").map(PathBuf::from);
        if cuda_dir.as_ref().is_none_or(|p| !p.is_dir()) {
            #[cfg(windows)]
            {
                let base = Path::new(r"C:\Program Files\NVIDIA GPU Computing Toolkit\CUDA");
                if let Ok(entries) = fs::read_dir(base) {
                    let mut versions: Vec<PathBuf> = entries
                        .flatten()
                        .map(|e| e.path())
                        .filter(|p| p.is_dir())
                        .collect();
                    versions.sort();
                    if let Some(best) = versions.pop() {
                        std::env::set_var("CUDA_PATH", &best);
                        cuda_dir = Some(best);
                    }
                }
            }
            #[cfg(target_os = "linux")]
            {
                let base = Path::new("/usr/local/cuda");
                if base.is_dir() {
                    std::env::set_var("CUDA_PATH", base);
                    cuda_dir = Some(base.to_path_buf());
                }
            }
        }
        if let Some(cuda) = cuda_dir {
            let mut prepend = Vec::new();
            let x64 = cuda.join("bin").join("x64");
            let bin = cuda.join("bin");
            if x64.is_dir() {
                prepend.push(x64);
            }
            if bin.is_dir() {
                prepend.push(bin);
            }
            if !prepend.is_empty() {
                let old = std::env::var_os("PATH").unwrap_or_default();
                prepend.extend(std::env::split_paths(&old));
                if let Ok(joined) = std::env::join_paths(prepend) {
                    std::env::set_var("PATH", joined);
                }
            }
        }
    });
}

pub fn apply_gpu_selection_env(cfg: &GpuSelectionConfig) {
    ensure_host_cuda_toolkit_path();
    let effective_pref = if let Some(ref name) = cfg.adapter_name_filter {
        let trimmed = name.trim();
        if !trimmed.is_empty() {
            format!("name:{trimmed}")
        } else {
            cfg.preference.clone()
        }
    } else {
        cfg.preference.clone()
    };
    std::env::set_var("SYNTH_GPU_PREFERENCE", &effective_pref);
    std::env::set_var("QUALIA_GPU_PREFERENCE", &effective_pref);
    let reject_val = if cfg.reject_igpu || cfg.preference == "discrete-only" {
        "1"
    } else {
        "0"
    };
    std::env::set_var("SYNTH_REJECT_IGPU", reject_val);
    std::env::set_var("QUALIA_REJECT_IGPU", reject_val);
    let backend = cfg.backend.trim();
    if !backend.is_empty() {
        std::env::set_var("SYNTH_GPU_BACKEND", backend);
        std::env::set_var("QUALIA_RENDER_WGPU_BACKEND", backend);
        std::env::set_var("QUALIA_WGPU_BACKEND", backend);
        if backend.eq_ignore_ascii_case("cuda") {
            std::env::set_var("QUALIA_INFERENCE_MODE", "cuda");
            std::env::set_var("QUALIA_PREFER_CUDA", "1");
            #[cfg(not(target_arch = "wasm32"))]
            qualia_core_db::set_inference_mode(qualia_core_db::InferenceMode::CudaTc);
        }
    }
}

pub fn save_and_apply_gpu_selection_config(mut cfg: GpuSelectionConfig) -> Result<Value, String> {
    cfg.schema = "synth.gpu-selection-config/v1".into();
    let pref = cfg.preference.trim().to_ascii_lowercase();
    if !matches!(
        pref.as_str(),
        "discrete-only" | "discrete" | "integrated" | "igpu" | "low-power" | "high-performance"
    ) && !pref.starts_with("name:")
    {
        return Err(format!(
            "Invalid GPU preference '{}'; expected discrete-only, discrete, integrated, or name:<adapter>",
            cfg.preference
        ));
    }
    cfg.preference = pref;
    if cfg.preference == "discrete-only" {
        cfg.reject_igpu = true;
    } else if cfg.preference == "integrated" || cfg.preference == "igpu" {
        cfg.reject_igpu = false;
    }
    let backend = cfg.backend.trim().to_ascii_lowercase();
    if !matches!(
        backend.as_str(),
        "auto" | "cuda" | "vulkan" | "vk" | "dx12" | "d3d12" | "metal" | "gl" | "gles" | "cpu"
    ) {
        return Err(format!(
            "Invalid GPU backend '{}'; expected cuda, vulkan, dx12, metal, gl, auto, or cpu",
            cfg.backend
        ));
    }
    cfg.backend = backend;
    let bytes = serde_json::to_vec_pretty(&cfg).map_err(|e| e.to_string())?;
    for path in gpu_config_paths() {
        let _ = fs::write(&path, &bytes);
    }
    apply_gpu_selection_env(&cfg);
    Ok(detect_runtime_and_gpu_profile(Some(&cfg.backend)))
}

/// Initialize the process environment variables from `.synth/gpu-config.json` (or default `discrete-only`)
/// before any GPU device is created.
pub fn init_gpu_policy_on_startup() {
    let cfg = load_gpu_selection_config();
    apply_gpu_selection_env(&cfg);
}

/// Enumerate physical GPU circuits discovered via `qualia_core_db::gpu_context::device_registry`.
pub fn enumerate_gpu_adapters_json() -> Vec<Value> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        static CACHED_ADAPTERS: std::sync::OnceLock<Vec<Value>> = std::sync::OnceLock::new();
        CACHED_ADAPTERS
            .get_or_init(|| {
                let circuits = qualia_core_db::gpu_context::device_registry::enumerate_circuits();
                let primary = qualia_core_db::gpu_context::device_registry::primary_circuit_index();
                let aux = qualia_core_db::gpu_context::device_registry::auxiliary_circuit_index();
                circuits
                    .iter()
                    .enumerate()
                    .map(|(idx, c)| {
                        let is_igpu = c.caps.is_integrated_gpu();
                        json!({
                            "index": idx,
                            "name": c.caps.name,
                            "vendor_id": format!("0x{:04x}", c.caps.vendor),
                            "device_id": format!("0x{:04x}", c.caps.device),
                            "device_type": format!("{:?}", c.caps.device_type),
                            "backend": c.caps.backend_label(),
                            "driver": c.caps.driver,
                            "is_discrete": format!("{:?}", c.caps.device_type) == "DiscreteGpu",
                            "is_integrated": is_igpu,
                            "role": if Some(idx) == primary {
                                "primary-discrete"
                            } else if Some(idx) == aux {
                                "auxiliary-integrated"
                            } else {
                                "secondary"
                            }
                        })
                    })
                    .collect()
            })
            .clone()
    }
    #[cfg(target_arch = "wasm32")]
    {
        Vec::new()
    }
}

/// Probe NVIDIA CUDA driver (`nvcuda.dll` / `libcuda.so.1`) and Forge NVRTC availability once per process.
pub fn probe_cuda_status_json() -> Value {
    #[cfg(not(target_arch = "wasm32"))]
    {
        static CUDA_STATUS: std::sync::OnceLock<Value> = std::sync::OnceLock::new();
        let mut base = CUDA_STATUS
            .get_or_init(|| {
                ensure_host_cuda_toolkit_path();
                let driver_present = qualia_core_db::etalon_agent::CudaDriverApi::probe().is_some();
                let forge_caps = qualia_core_db::wgsl_forge::dispatch::caps();
                json!({
                    "compiled_with_cuda": cfg!(feature = "cuda"),
                    "driver_available": driver_present,
                    "forge_cuda_ready": forge_caps.cuda,
                    "coopmat_available": forge_caps.coopmat,
                    "rt_cores_available": forge_caps.rt,
                    "cuda_path": std::env::var("CUDA_PATH").ok(),
                })
            })
            .clone();
        if let Value::Object(ref mut map) = base {
            map.insert(
                "active_inference_mode".into(),
                json!(qualia_core_db::active_inference_mode().as_str()),
            );
        }
        base
    }
    #[cfg(target_arch = "wasm32")]
    {
        json!({
            "compiled_with_cuda": false,
            "driver_available": false,
            "forge_cuda_ready": false
        })
    }
}

/// REQ-4.1 & REQ-4.2: Tiered Local GPU & Local Runtime Profile (Port 4242 / 8080 / 8787 + Native Vulkan/CUDA/DX12/Metal).
pub fn detect_runtime_and_gpu_profile(requested_backend: Option<&str>) -> Value {
    let cfg = load_gpu_selection_config();
    static DESKTOP_PROBE: std::sync::OnceLock<(bool, bool)> = std::sync::OnceLock::new();
    let (local_runtime_8080, local_runtime_4242) = *DESKTOP_PROBE.get_or_init(|| {
        let (_, r8080, r4242) = crate::tray_env::probe_webizen_desktop();
        (r8080, r4242)
    });

    let selected_backend = match requested_backend.map(str::trim).filter(|s| !s.is_empty()) {
        Some("auto") | None => cfg.backend.as_str(),
        Some(other) => other,
    };

    let execution_tier = if local_runtime_4242 || local_runtime_8080 {
        "attached-local-runtime-4242"
    } else if matches!(selected_backend, "webgpu-wasm" | "webgl2-wasm" | "wasm") {
        "portable-web-wasm"
    } else {
        "local-native-gpu-standalone"
    };

    let adapters = enumerate_gpu_adapters_json();
    let cuda_status = probe_cuda_status_json();

    json!({
        "schema": "synth.runtime-gpu-profile/v1",
        "execution_tier": execution_tier,
        "selected_gpu_backend": selected_backend,
        "gpu_selection_config": cfg,
        "cuda_runtime": cuda_status,
        "enumerated_adapters": adapters,
        "supported_backends": ["vulkan", "cuda", "dx12", "metal", "gl", "webgpu-wasm", "webgl2-wasm", "cpu"],
        "supported_preferences": ["discrete-only", "discrete", "integrated", "name:<adapter>"],
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
