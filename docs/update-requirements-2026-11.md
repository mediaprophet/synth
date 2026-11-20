# Webizen AI Synth — Update Requirements & Technical Analysis (2026-11-20)

> **Scope & Status:** Requirements documentation and root-cause analysis only. No builds, binary swaps, or process terminations are executed in this pass while the concurrent background process is running.

---

## 1. CLI vs. MCP Version & Binary Drift

### Current Behavior & Root Cause
There are three distinct reasons the CLI and MCP report or behave as different versions:

1. **Dual-Binary Locking (`synth.exe` vs. `synth-mcp.exe`):**
   - Because Windows locks running executables, Antigravity's MCP configuration ([.agents/mcp_config.json](file:///c:/github/synth/.agents/mcp_config.json) and [~/.gemini/config/mcp_config.json](file:///C:/Users/Hp/.gemini/config/mcp_config.json)) spawns a sidecar copy (`C:\github\synth\target-prod\debug\synth-mcp.exe`), while CLI commands invoke `C:\github\synth\target-prod\debug\synth.exe` (or `target-studio\debug\synth.exe`).
   - When `cargo build` updates `synth.exe`, `synth-mcp.exe` remains on the older binary until the MCP stdio process is stopped, `synth.exe` is copied over `synth-mcp.exe`, and MCP is reloaded.
2. **Stale Cached MCP Schemas in Antigravity:**
   - Even after `synth-mcp.exe` is updated, Antigravity caches tool schemas under `C:\Users\Hp\.gemini\antigravity\mcp\synth\*.json`.
   - Inspection of the currently cached [synth_produce_present.json](file:///C:/Users/Hp/.gemini/antigravity/mcp/synth/synth_produce_present.json) shows it is still advertising an older schema (e.g., `quality_profile` enum `["preview", "production16"]` with 64/256/320 MiB descriptions, missing `authoring`, `domain`, `taxon`, `fiction`, `evidence_q42`, and missing the newer tools `synth_find`, `synth_operator_status`, `synth_production_rules`, `synth_bookmarks`, `synth_jobs`, `synth_update`).
3. **Dual Version Strings & Duplicate MCP Server Registration:**
   - [Cargo.toml](file:///c:/github/synth/Cargo.toml#L3) declares `version = "0.1.0"` (required by Cargo SemVer), whereas [src/agent_index.rs](file:///c:/github/synth/src/agent_index.rs#L12) defines `RELEASE_VERSION = "0.0.41.12"`. Furthermore, during MCP `initialize`, `qualia-mcp` reports `serverInfo.version` from its own `CARGO_PKG_VERSION` (`0.0.41`) in [protocol.rs](file:///c:/github/qualiaDB/crates/qualia-mcp/src/authoring/protocol.rs#L192).
   - Both `synth` and `synth-qualia_synth` are registered simultaneously in Antigravity, spawning two separate stdio processes that can drift out of sync.

### Update Requirements
- **REQ-1.1 (Unified Binary & Parity Check):** Expose a single canonical build identity (`RELEASE_VERSION`, git commit SHA, build timestamp, and enabled Cargo features such as `preview`, `native-ui`, `cuda`) in `synth --version`, `synth capabilities`, `synth_operator_status`, and the MCP `initialize` `serverInfo` / `instructions` payload.
- **REQ-1.2 (Automated Sidecar Sync & Drift Warning):** In `synth_capabilities` and `synth_operator_status`, compare the running `current_exe()` SHA-256 against sibling `synth.exe` / `synth-mcp.exe` in `target-prod/debug`. If they differ, emit an explicit `binary_drift_detected: true` warning in every MCP receipt so the agent and operator immediately know `synth-mcp.exe` needs a reload.
- **REQ-1.3 (Single MCP Server Registration):** Consolidate the duplicate `synth` and `synth-qualia_synth` MCP entries in Antigravity so only one stdio daemon runs and its cached tool schemas stay synchronized with [src/mcp_full.rs](file:///c:/github/synth/src/mcp_full.rs).

---

## 2. CLI Preview, Motion Preview, Single-Asset Browser View & Annotations

### Current Behavior & Gaps
1. **Why CLI `preview` Appears Missing (`--features preview` Gate):**
   - In [Cargo.toml](file:///c:/github/synth/Cargo.toml#L43-L44), `default = []` and `preview = ["dep:webizen-render"]`.
   - If `synth` or `synth-mcp` is compiled without `--features preview`, running `synth preview` (or passing `preview_assets` to `synth_produce_present`) fails with: `"Native .10d preview requires a native build with --features preview"` ([src/cli.rs](file:///c:/github/synth/src/cli.rs#L157)).
2. **Motion Preview Limitations:**
   - [src/native_preview.rs](file:///c:/github/synth/src/native_preview.rs#L146-L193) and [volumetric_shot.rs](file:///c:/github/qualiaDB/crates/webizen-render/src/volumetric_shot.rs#L20-L68) implement `synth preview --asset FILE.10d --out NEW_DIR --shot SHOT.json`, which steps skeletal animation (`clip_id` + `times`), renders frames (`frame-0000.png`..), and writes `index.html`.
   - **Gaps:**
     - `NativeShot` caps `times.len() <= 16` and uncompressed frame budget to 64 MiB.
     - `render_10d_preview_html` ([src/present.rs](file:///c:/github/synth/src/present.rs#L338-L379)) only renders static angle-switching buttons (`selectView(idx)`), with no play/pause/scrub timeline, FPS playback control, or `.mp4` / animated WebP / sprite-strip playback in the preview HTML.
     - Motion from non-`SKL1` kinetics morphisms (e.g. `step_island_xpbd`, `evaluate_nla_channelbag`, `interpolate_motor_fti`) cannot be previewed directly from CLI `synth preview` without manually authoring a movie capture vibe (`hypermedia.chronicle.capture_deterministic_movie`).
3. **Single-Asset Browser Preview:**
   - Currently, `synth serve` serves the 5-tab demo workbench at `/` and the multi-file project workbench at `/project` ([src/present_project.rs](file:///c:/github/synth/src/present_project.rs)).
   - There is no dedicated single-asset focus route (e.g., `GET /preview/<lane>/<file>` or `synth preview --asset FILE.10d --serve --open`) that opens just that single asset full-viewport in the browser with QualiaPortal WASM / native GPU turntable, lighting/camera controls, motion timeline scrubbing, and contact-sheet comparison.
4. **Browser Annotations Back to the Agent:**
   - [src/project.rs](file:///c:/github/synth/src/project.rs#L53-L68) and [src/present_project.rs](file:///c:/github/synth/src/present_project.rs#L123-L238) implement `POST /api/project/annotations` saving to `{project}/.synth/annotations.json`.
   - **Gaps:**
     - Annotations only work when `synth serve --project DIR` is attached to a project directory; standalone `synth preview` (`index.html`) or `present_html` files opened via `file://` or `<agent-embed>` have **no annotation UI or persistence endpoint**.
     - Annotations lack spatial/temporal pinning: you cannot click a 3D point/camera angle (`yaw`, `pitch`, `zoom`, `2D/3D pin coordinates`) or a specific motion frame (`time_seconds` / `clip_id`) and attach a note directly on the viewport.
     - There is no dedicated MCP tool (e.g. `synth_annotations` with `action=list|resolve|reply`) to surface pending user annotations directly to the agent or automatically inject unread annotations into `synth_produce_present` receipts.

### Update Requirements
- **REQ-2.1 (Enable `preview` by Default):** Move `preview` (`dep:webizen-render`) into `default` features in [Cargo.toml](file:///c:/github/synth/Cargo.toml#L43) so every CLI and MCP build supports `synth preview` out of the box.
- **REQ-2.2 (Interactive Motion Preview in CLI & Browser):**
  - Upgrade `synth preview` and `render_10d_preview_html` to support timeline playback controls (Play/Pause, FPS selector, frame scrubber, loop toggle, onion-skin/frame compare) across shot frames.
  - Add a CLI shorthand `synth preview --asset FILE.10d --out DIR --motion [--clip ID] [--fps N] [--duration S]` that auto-generates the shot timestamps without requiring a hand-written `SHOT.json`, and supports both `SKL1` skeletal clips and PGA motor/NLA trajectories.
- **REQ-2.3 (Dedicated Single-Asset Browser Preview):**
  - Add a single-asset browser view (`GET /asset?path=...` or `GET /project/inspect/<lane>/<file>`, plus CLI `synth preview --asset FILE --open`) that renders a focused, full-viewport inspector for one `.10d` / `.hmc` / `.emf` / `.mp4` / `.png` / `.svg` / `.qac` asset with camera presets, turntable/motion scrubber, wireframe/normal/PBR channel toggles, and side-by-side contact sheet views.
- **REQ-2.4 (In-Viewport Spatial/Temporal Annotations & MCP Feedback Loop):**
  - Embed an annotation drawer directly into the single-asset preview and standalone preview pages (connecting to `http://127.0.0.1:8787/api/annotations` when the local server is running, with local JSON export fallback).
  - Capture exact viewport state on each annotation: `asset_path`, `asset_sha256`, `camera: {yaw, pitch, distance, target}`, `time_seconds`, `frame_index`, `screen_pin: [u, v]`, `aspect`, `rating`, `tags`, `comment`, and `links`.
  - Add an MCP tool `synth_annotations` (`list`, `get`, `mark_addressed`) and include `open_annotations` counts in `synth_produce_present` and `synth_operator_status` responses so the agent automatically sees user feedback before its next edit.

---

## 3. MCP / Antigravity Resetting & Stopping Jobs

### Current Behavior & Root Cause Analysis
Inspection of the MCP stdio transport ([qualia-mcp/src/authoring/stdio.rs](file:///c:/github/qualiaDB/crates/qualia-mcp/src/authoring/stdio.rs) and [src/mcp_full.rs](file:///c:/github/synth/src/mcp_full.rs)) reveals four concrete failure modes that cause Antigravity's MCP client or extension host to reset or drop an active job:

1. **Duplicate Heavy MCP Daemons (`synth` + `synth-qualia_synth`):**
   - Antigravity currently loads **two** identical MCP servers pointing to `synth-mcp.exe` (one from `.agents/mcp_config.json` / `~/.gemini/config/mcp_config.json` as `synth`, and one from `.agents/plugins/synth-qualia` as `synth-qualia_synth`), each registering ~73 tools and initializing heavy QualiaDB/Etalon state.
2. **Synchronous Blocking on the Stdio Loop During Heavy Production & Subprocess Previews:**
   - [qualia-mcp/src/authoring/stdio.rs](file:///c:/github/qualiaDB/crates/qualia-mcp/src/authoring/stdio.rs#L19-L44) processes JSON-RPC messages synchronously on a single thread.
   - While `synth_produce_present` is running heavy 3D boolean/loft/SDF/PBR evaluations (up to 2 billion logical work units in `production16`) plus spawning `synth preview` child processes ([src/mcp_full/contracts.rs](file:///c:/github/synth/src/mcp_full/contracts.rs#L85-L120), which allows up to **180 seconds per preview asset**), the stdio server cannot answer MCP `ping` / heartbeat requests or cancellation notifications. Many MCP hosts treat an unresponsive stdio pipe (30–60s timeout) as a hung server and reset the connection, killing the in-flight job.
3. **Large Inline Base64 Payloads & Stdout/Memory Spikes:**
   - `comprehensive_protocol_limits()` ([src/mcp_full.rs](file:///c:/github/synth/src/mcp_full.rs#L449-L455)) allows 4 MiB wire messages and 512 KiB tool results, and [protocol.rs](file:///c:/github/qualiaDB/crates/qualia-mcp/src/authoring/protocol.rs#L263-L267) serializes the entire JSON result **twice** in the same response frame (`content[0].text` as an escaped JSON string *plus* `structuredContent` as a full JSON object), doubling wire size.
   - When `present_html` is generated, [src/present.rs](file:///c:/github/synth/src/present.rs#L150-L189) base64-inlines `.10d`, `.emf`, `.qac`, and all sibling `.png` files *plus* a full capabilities snapshot (`morphisms`, `mcp_tools`, `vibe_sdk`, `living_world`) into a single multi-megabyte HTML file. When Antigravity reads or embeds that huge self-contained HTML in `<agent-embed>`, webview memory can spike and reset the session.
4. **Uncaught Panics or Stack Overflows in Heavy Geometry/Codec Kernels:**
   - Because `backend.call(name, args)` in [protocol.rs](file:///c:/github/qualiaDB/crates/qualia-mcp/src/authoring/protocol.rs#L226) is not wrapped in `std::panic::catch_unwind`, any panic inside a geometry/boolean/mesh kernel or thread stack exhaustion immediately terminates `synth-mcp.exe`, causing an EOF on Antigravity's stdio pipe and resetting the agent turn.

### Update Requirements
- **REQ-3.1 (`catch_unwind` & Dedicated Worker Thread Stack Size):**
  - Wrap every tool execution in `StandaloneSynthMcp::call` inside `std::panic::catch_unwind` on a dedicated worker thread with an explicit large stack (e.g., 32–64 MiB stack on Windows, where the default thread stack is only 1 MiB and deep CSG/BVH/N3 recursion can overflow it). Convert any panic into a structured `ToolError` (`"internal_panic"`) so the stdio server never crashes.
- **REQ-3.2 (Non-Blocking Heartbeat / Async Job Offload for Long Productions):**
  - Allow the stdio reader loop to respond to `ping` requests immediately even while a `synth_produce_present` worker thread is executing, and support an async/job mode (or tight bounded preview timeout with background completion via the local operator daemon) so tool calls never exceed Antigravity's MCP call timeout.
- **REQ-3.3 (Compact MCP Responses & Lightweight Present HTML):**
  - Keep `synth_produce_present` MCP responses compact (hashes, file paths, resource summaries, and annotation counts—never large inline blobs).
  - When the local HTTP server (`127.0.0.1:8787`) or Webizen runtime (`127.0.0.1:4242`) is active, serve `.10d`, `.emf`, `.qac`, and `.png` assets via localhost URLs rather than base64-inlining tens of megabytes into `present_html`.

---

## 4. Local GPU & Local Runtime Profile (Port `4242` Discovery, Vulkan / CUDA / Metal + WASM / WebGL2)

### Current Architecture in the Codebase
1. **Port `4242` / `8080` Probing Already Exists for Status, But Not Execution Offload:**
   - [src/tray_env.rs](file:///c:/github/synth/src/tray_env.rs#L119-L131) already defines `probe_webizen_desktop()`, which checks `127.0.0.1:8080` (HTTP) and `127.0.0.1:4242` (Webizen Desktop / local runtime WS) with an 80ms timeout and switches `HostRuntimeMode` between `StandaloneTray` and `AttachedToWebizenDesktop`.
   - However, currently this probe only changes a badge label and tray icon (`icon_attached.png`). Neither the browser viewer ([src/present_portal.rs](file:///c:/github/synth/src/present_portal.rs)) nor the production/preview pipeline dispatches assets or compute jobs to port `4242` when it is online.
2. **Current GPU Backend Wiring across Synth, QualiaDB (`webizen-render`), and Etalon:**
   - **Etalon** ([c:\github\etalon\Cargo.toml](file:///c:/github/etalon/Cargo.toml#L89-L130)) supports native backends (`vulkan`, `metal`, `dx12`, `gles`, plus `etalon-cuda` for CUDA compute/inference) as well as `webgpu` and `webgl2`.
   - **`webizen-render`** ([c:\github\qualiaDB\crates\webizen-render\Cargo.toml](file:///c:/github/qualiaDB/crates/webizen-render/Cargo.toml#L9-L19)) enables `etalon` with `["std", "metal", "gles", "vulkan", "wgsl", "webgpu", "spirv"]` and `qualia-core-db` with `["gpu-runtime", "gpu-native-portable", "wgsl-forge"]`.
   - **Gaps:**
     - `cuda` (`wgsl-forge-cuda`, `etalon-cuda`, `qualia-core-db/cuda`) and `nvml` are feature-gated in `qualia-core-db` ([Cargo.toml](file:///c:/github/qualiaDB/crates/qualia-core-db/Cargo.toml#L55-L61)) and are **not** exposed or enabled in `synth/Cargo.toml`.
     - In `synth`, `webizen-render` is only pulled in when `--features preview` is enabled, and `aether-native` ([synth/Cargo.toml](file:///c:/github/synth/Cargo.toml#L36)) is currently configured with `features = ["vello-cpu-softbuffer"]` rather than Etalon native GPU surface rendering.
     - In the browser presentation layer ([src/present_portal.rs](file:///c:/github/synth/src/present_portal.rs#L357-L434)), `bootSynthPortals()` only attempts to load WASM (`/qualia/js/qualia-shell.js` using browser WebGPU or WebGL2 fallback) and falls back to a degraded 2D canvas if browser WASM/WebGPU is unavailable—even when Synth or Webizen Desktop is running locally on the machine with a native Vulkan/CUDA/Metal GPU.

### Update Requirements
- **REQ-4.1 (Tiered Runtime & GPU Backend Selection — Native Local vs. Browser WASM):**
  - Formalize a unified runtime profile selector across Synth CLI, MCP, and Browser Studio with automatic capability negotiation:
    1. **Attached Local Runtime (`127.0.0.1:4242` / `8080` or local Synth Daemon `127.0.0.1:8787`):**
       - Following the LLM & WebizenDesktop pattern, both the Synth host and the browser presentation shell (`PORTAL_BOOT_JS` in [src/present_portal.rs](file:///c:/github/synth/src/present_portal.rs)) must probe `127.0.0.1:4242` (and the local Synth daemon native render endpoint).
       - When the local runtime is detected, send `.10d` / `.hmc` / `.emf` / `.q42` assets and interactive camera/frame requests to execute on the **local native GPU runtime** (streaming native Vulkan/Metal/CUDA-rendered frames or shared GPU surface output) rather than forcing heavy payloads through browser WASM.
    2. **Local Native Standalone Profile (Vulkan / Metal / CUDA):**
       - When Synth runs locally (`synth produce`, `synth preview`, `synth serve`, `synth operator`), automatically detect and initialize the best native Etalon hardware backend for the host OS:
         - **Windows / Linux:** Vulkan (`etalon/vulkan`) + optional CUDA (`qualia-core-db/cuda`, `wgsl-forge-cuda`, `etalon-cuda`) for compute, simulation, SDF/radiance baking, and local LLM/Vibe inference.
         - **macOS:** Metal (`etalon/metal`, `metal-native`) + Vulkan Portability where applicable.
       - Expose explicit Cargo features and runtime profile flags (`--gpu-backend auto|vulkan|cuda|metal|webgl2|wasm`, plus `local-runtime` profile in `synth capabilities` and `synth_produce_present`).
    3. **Portable Web / Visitor Fallback (WASM + WebGPU / WebGL2):**
       - When no local runtime is reachable on port `4242` or `8787` (or when exporting standalone visitor packages for QualiaPortal), seamlessly run via the existing WASM + WebGPU / WebGL2 pipeline (`loadQualiaPortal(canvas, { allowWebGl2: true })`) with PNG contact sheets preserved as the zero-GPU baseline.
- **REQ-4.2 (Pipeline & Receipt Telemetry for Active GPU Backend):**
  - Record the actual executed hardware backend (`backend: "vulkan" | "cuda" | "metal" | "dx12" | "webgpu-wasm" | "webgl2-wasm" | "cpu"`, adapter name, driver, and whether port `4242` local runtime delegation was used) in `synth.production-receipt/v1`, `synth.native-10d-preview/v1`, `synth capabilities`, and the browser `#webgpu-snapshot` banner.
- **REQ-4.3 (Pipeline Compute Offload for Asset Production):**
  - Ensure heavy production morphisms (SDF brick evaluation, raymarched `.emf` slices, PBR AO/normal baking, XPBD physics stepping, and `capture_deterministic_movie`) can use the local Etalon GPU (Vulkan/CUDA/Metal) during `synth_produce_present` while preserving deterministic replay receipts.

---

## 5. Multi-Agent Concurrent Sessions, Resource Admission & Slot Queueing

### Current Behavior & Gaps
1. **No Cross-Agent or Cross-Process Resource Admission Guard:**
   - Currently, each MCP client (`Antigravity`, `Grok`, `Codex`, or subagents in a swarm) either spawns its own `synth-mcp.exe` stdio process or hits `POST /api/mcp/call` / `POST /api/produce` on the HTTP server.
   - Inside `synth_produce_present` ([src/mcp_full/production.rs](file:///c:/github/synth/src/mcp_full/production.rs#L178-L180)), a fresh `ProductionSession` is created on every call with a **2 GiB aggregate / 2.25 GiB package budget** (`production16` / `authoring` profile in [src/production/mod.rs](file:///c:/github/synth/src/production/mod.rs#L108-L113)), and explicitly notes:
     > `"accounting_scope": "per-revision logical component allowances; overlapping candidates and clones are not a process-peak or allocator bound"`
   - If 3 or 4 agents invoke `synth_produce_present` or `synth preview` at the same time, each independently claims up to ~2.25 GiB of host RAM plus an offscreen 2048×2048 `VolumetricRenderer` GPU context—easily exhausting system RAM/VRAM and causing OOM kills or Antigravity resets.
2. **Single-Mutex Contention in Authenticated `SynthAgentBackend` & Single-Threaded HTTP Server:**
   - In the authenticated host path ([src/agent.rs](file:///c:/github/synth/src/agent.rs#L12-L20)), `SynthAgentBackend` holds a single `Arc<Mutex<SynthHost>>` and uses `try_lock()`, returning `ToolError("busy", "Synth operation already running")` if another operation holds the lock—without any multi-session registry, slot count, estimated wait time, or resource breakdown.
   - Meanwhile, [src/web_server.rs](file:///c:/github/synth/src/web_server.rs#L138-L140) processes incoming HTTP connections **serially** (`for stream in listener.incoming().flatten() { let _ = handle_connection(stream); }`), and [src/project.rs](file:///c:/github/synth/src/project.rs#L16) uses a single global `static PROJECT: OnceLock<Project>`, preventing multiple agents from working on separate sessions or projects concurrently through the daemon.
3. **Ephemeral vs. Persistent Multi-Agent Sessions:**
   - `synth_produce_present` creates a one-shot `ProductionSession` and drops it immediately after evaluation (`"handle_lifetime": "one source evaluation"`). Agents cannot hold named, isolated multi-step sessions (`session_id`, `agent_id`) across turns with incremental cells while sharing a coordinated system resource budget.

### Update Requirements
- **REQ-5.1 (Multi-Agent Session Registry & Isolated Session Handles):**
  - Replace the single implicit session model with a concurrent `SessionRegistry` keyed by `session_id` and tagged with `agent_id` (or principal DID/subject), `project_root`, `quality_profile`, `created_at`, `last_active_at`, and `reserved_bytes`.
  - Allow multiple agents to create and work in independent `ProductionSession` instances simultaneously (both stateless one-shot `synth_produce_present` sessions and stateful multi-turn sessions via `session_id`), without overwriting each other's working state or colliding on `OnceLock<Project>`.
- **REQ-5.2 (Cross-Process / Cross-Agent Resource & Slot Admission Controller):**
  - Introduce a machine-wide `ResourceAdmissionController` (coordinated via the local Synth Operator / Daemon on `127.0.0.1:8787` or a shared lockfile/shared-memory ledger in `SYNTH_OUTPUT_ROOT/.synth/slots.json` so multiple `synth-mcp.exe` stdio processes and CLI invocations share the same truth):
    - **Probed System Metrics:** Before admitting a new session or heavy production/preview job, inspect:
      1. Active concurrent sessions / in-flight production slots (`active_slots / max_slots`, configurable via `SYNTH_MAX_CONCURRENT_SESSIONS`, e.g., default `2` for `production16` or dynamically scaled by RAM/VRAM).
      2. Available system RAM and committed session budgets (ensuring `sum(active_session_budgets) + requested_profile_budget <= safe_ram_ceiling`).
      3. Available GPU VRAM / active offscreen renderers (via Etalon / NVML / WebGPU adapter limits) before spawning `VolumetricRenderer` or CUDA/Vulkan compute passes.
- **REQ-5.3 (Explicit `wait_for_slot` / `resource_busy` Contract for Agents):**
  - When an agent requests a new session or calls `synth_produce_present` / `synth preview` and resources or slots are currently saturated:
    - **Optional Blocking Wait (`wait_ms`):** Support an optional `wait_ms` (e.g., up to 15,000 ms) so short slot contentions resolve transparently in queue order (FIFO).
    - **Structured Wait / Retry Directive:** If no slot opens within `wait_ms` (or `wait_ms = 0`), return a deterministic, agent-actionable response (`code: "resource_slot_unavailable"` / `status: "wait_for_slot"`) instead of crashing or thrashing:
      ```json
      {
        "status": "wait_for_slot",
        "code": "resource_slot_unavailable",
        "retry_after_ms": 5000,
        "slots": {
          "active": 2,
          "max": 2,
          "queued": 1,
          "active_agents": [
            {"agent_id": "agent-alpha", "session_id": "sess-01", "profile": "production16", "elapsed_ms": 4200}
          ]
        },
        "resources": {
          "available_ram_mib": 1840,
          "required_ram_mib": 2048,
          "available_vram_mib": 1120,
          "gpu_render_slots_busy": 1
        },
        "advice": "Wait retry_after_ms and retry, or downgrade quality_profile to 'preview' (64 MiB)."
      }
      ```
- **REQ-5.4 (Automatic Lease Expiry & Stale-Session Eviction):**
  - Assign every active session slot a heartbeat/lease TTL (e.g., 180 seconds for active evaluation, 10 minutes for idle retained sessions) and record the owning OS `pid`.
  - If an agent or `synth-mcp.exe` process exits or crashes, automatically reclaim its slot and memory reservation so dead sessions never permanently block the pool.
- **REQ-5.5 (Threaded / Pooled HTTP Server for Multi-Agent Daemon Mode):**
  - Upgrade [src/web_server.rs](file:///c:/github/synth/src/web_server.rs#L138-L167) from a single-threaded blocking accept loop to a bounded worker thread pool governed by the `ResourceAdmissionController`, so UI polling (`/api/operator`, `/project`, annotations) and light queries (`synth_find`, `query_sparql`) never block behind an agent running a heavy `synth_produce_present` job.

---

## 6. Asset Changelog, Multi-Actor SVN-Style Revisions, Provenance Signatures, Economics & Dual-Regime Licensing Contracts

### Current Behavior & Existing Foundations in the Ecosystem
1. **What Synth Currently Packages (`synth/production.json`):**
   - In [src/production/package.rs](file:///c:/github/synth/src/production/package.rs#L5-L22), HMC bundles currently contain only a minimal `synth/production.json` (`schema`, `producer`, `version`, `sources`, `inputs`) and `outputs/*`.
   - When exported to a directory, [src/mcp_full/production.rs](file:///c:/github/synth/src/mcp_full/production.rs#L210-L248) writes `.synth/source.vibe` and `.synth/production-receipt.json`.
   - **Gaps in Synth:** There is no append-only asset changelog (`CHANGELOG.q42.yaml` / `synth/changelog.json` / `.q42j`), no cryptographic signature over each revision entry, no separation between the responsible **Human User** vs. **Software Agent** vs. **Harness**, no economics/compute/token telemetry per revision, no SVN-style revision tree for multi-agent collaborative check-ins, and no attached licensing/obligation contract in the asset package.
2. **Existing Canonical Owners in `qualia-cooperative-core` and `qualia-core-db` (Must Be Reused, Not Re-Invented):**
   - **Human vs. Agent Provenance DAG:** [qualia-cooperative-core/src/provenance.rs](file:///c:/github/qualiaDB/crates/qualia-cooperative-core/src/provenance.rs#L33-L233) defines `AgentType` (`NaturalPerson`, `SoftwareAgent`, `Organization`, `Instrument`, `Dataset`), `AgentRef`, `Reliance`, and `JudgementProvenance`. It enforces the core rule: **authorship and legal/moral responsibility stay with the `NaturalPerson` (`responsible_agent`), while `SoftwareAgent` and harness tooling are disclosed in `informed_by` (`Reliance` edges) with exact versions and epistemic horizon hashes.**
   - **Authoring & Contribution Chains:** [qualia-cooperative-core/src/authoring/records.rs](file:///c:/github/qualiaDB/crates/qualia-cooperative-core/src/authoring/records.rs#L33-L105) and [projects.rs](file:///c:/github/qualiaDB/crates/qualia-cooperative-core/src/projects.rs#L136-L223) define `DocumentRevision`, `Contribution`, `ContributionRecord`, `ReviewRecord`, `IdentifierAssociationEvidence`, and `WorkStatus` (`Unknown`, `Unpaid`, `SelfFunded`, `PartiallyCompensated`, `Compensated`, `Disputed`), linked via append-only `predecessor_id` chains and merged deterministically by stable ID (`merge_contributions`).
   - **9-Stage Contribution & Obligation Lifecycle:** [qualia-cooperative-core/src/contribution_lifecycle.rs](file:///c:/github/qualiaDB/crates/qualia-cooperative-core/src/contribution_lifecycle.rs#L1-L480) implements `RequestAndAgreement` -> `ActivityAndResources` -> `OutputOrService` -> `SubmissionOrClaim` -> `AcceptanceDecision` -> `Valuation` -> `LifecycleObligation` -> `SettlementAllocation` -> `TransitionDecision`.
   - **Permissive Commons & Finite Obligation Discharge:**
     - [qualiaDB/core-ontologies/permissive-commons.n3](file:///c:/github/qualiaDB/core-ontologies/permissive-commons.n3) and [social-permissive-commons.n3](file:///c:/github/qualiaDB/core-ontologies/social-permissive-commons.n3) define `pc:Work`, `pc:productionCost`, `pc:roiMultiplier` (SHACL-capped by `pc:RoiMultiplierCapShape`), `pc:obligationCost`, `pc:amountCompensated`, `pc:ObligationStatus` (`pc:Outstanding` -> `pc:Discharged`), and `pc:attributedTo` (permanent moral attribution that never extinguishes).
     - [qualia-core-db/src/net/qdnf/economics/obligation.rs](file:///c:/github/qualiaDB/crates/qualia-core-db/src/net/qdnf/economics/obligation.rs#L42-L168) and [modalities/socioeconomics.rs](file:///c:/github/qualiaDB/crates/qualia-core-db/src/modalities/socioeconomics.rs#L13-L234) implement finite obligation accounting (`T = target`, `S = settled`, `W = authorised non-cash discharge`, `H = holds`, invariant `S + W + H <= T`) and 48-byte `NQuin` `.q42j` journal checkpoints (`OP_COMMONS_OBLIGATION_POOL = 0x58`).

### Update Requirements

#### REQ-6.1: Signed Asset Changelog (`synth/changelog.q42.yaml` & `synth/changelog.json`)
Every produced or updated asset package (`.hmc`) and exported asset directory (`.synth/changelog.q42.yaml` + human-readable `CHANGELOG.md`) must include an append-only, cryptographically signed revision log where each revision entry (`rev: r1, r2, ...`) records:
1. **Human Actor (`user` / `responsible_agent`):**
   - Mapped to `qualia_cooperative_core::provenance::AgentRef` (`AgentType::NaturalPerson`) and `AuthoringPermit`.
   - Records qualified identifiers (`did`, `subject`, `display_name`, `role: Steward | Contributor | Reviewer`, `capacity`, `organization`, `jurisdiction`), keeping role, authority, authorship, and paid/unpaid `WorkStatus` distinct.
2. **Software Agent (`agent`):**
   - Mapped to `AgentType::SoftwareAgent` inside `JudgementProvenance::informed_by`.
   - Records `agent_id`, `model_name` / `model_family` (e.g., `Gemini 4 Argon`, `Gemma 4`, `Qwen`, `Grok`, `Codex`), `model_version` / weight digest, `role` (e.g. `authoring`, `research`, `review`), and `within_validated_scope`.
3. **Harness & Execution Environment (`harness`):**
   - Records `harness_name` (`Antigravity IDE`, `Webizen AI Synth CLI`, `Webizen Desktop`, `Grok CLI`, `Codex`), `harness_version`, `synth_version` (`0.0.41.12`), `synth_binary_sha256`, `mcp_protocol_version`, `gpu_backend` (`vulkan`, `cuda`, `metal`, `webgpu-wasm`, `cpu`), and `quality_profile`.
4. **Timestamps & Content Hashes (`temporal` & `integrity`):**
   - High-resolution UTC timestamp (`created_at_rfc3339` + `InstantBridge` unix nanos), `lamport_clock`, `source_vibe_sha256`, `inputs_merkle_root`, `outputs_merkle_root`, `hmc_sha256`, and `epistemic_horizon` (digest of the `.q42` evidence and ontology graph active at production time).
5. **Cryptographic Signatures (`signatures`):**
   - Uses QualiaDB Ed25519 `sign_with_context` (and optional post-quantum / DID key bindings) with domain separation (`synth.asset.changelog.rev.v1`).
   - Binds `(revision_id, predecessor_rev_sha256, user_did, agent_id, harness_id, timestamp, outputs_merkle_root, economics_digest, license_contract_digest)` so any tampering invalidates the revision chain.
   - Distinguishes a **host/build attestation signature** from a **human review/adoption signature** (a software agent signature never impersonates human acceptance).

#### REQ-6.2: Economics, Compute & Token Telemetry per Revision (`economics`)
Each changelog revision entry must include an optional, unit-safe `economics` block (never conflating effort minutes, compute ticks, token counts, and fiat/currency minor units):
- **Human Effort & Capital (`human_contribution`):**
  - `effort_minutes: u32` (integer minutes, mapped to `Contribution::effort_minutes` / `ActivityAndResources`), `capital_advanced_cents: i64`, `reimbursable_expense_cents: i64`, `currency: "AUD" | "USD" | ...`, and `work_status: Unpaid | SelfFunded | PartiallyCompensated | Compensated`.
- **AI / Harness Token & Inference Telemetry (`ai_telemetry`):**
  - `input_tokens: Option<u64>`, `output_tokens: Option<u64>`, `reasoning_tokens: Option<u64>`, `cached_tokens: Option<u64>`, `provider_cost_micro_cents: Option<u64>`, `local_inference: bool` (whether run locally on `C:\LLM_files` vs. paid external API).
- **Compute & Energy Telemetry (`compute_telemetry`):**
  - `wall_time_ms: u64`, `vibe_evaluation_ticks: u64`, `asset_work_units: u64`, `peak_ram_bytes: u64`, `gpu_time_ms: Option<u64>`, `gpu_vram_bytes: Option<u64>`, `estimated_energy_joules: Option<f64>` (feeding `socioeconomics::eroi` thermodynamic viability checks).

#### REQ-6.3: SVN-Style Multi-Agent / Multi-Actor Cooperative Revision Graph
To support collaborative projects involving multiple humans and multiple agents working across revisions:
- **Monotonic Revision Numbers & Predecessor DAG (`rev: r1..rN`):**
  - Support SVN-style monotonic integer revisions (`r1`, `r2`, `r3`...) within a project authority trunk, plus content-addressed `revision_id` and `predecessor_ids: Vec<String>` (supporting both linear check-ins and multi-agent branch/merge check-ins).
- **Per-Revision Action & Delta Summary:**
  - Record `action: Checkout | Commit | Branch | Merge | Review | LicenceUpdate`, `changed_resources: Vec<{key, op: Added|Modified|Deleted, old_sha256, new_sha256}>`, and `source_cell_diff_sha256`.
- **Idempotent Add-Wins Contribution Merging:**
  - Reuse `qualia_cooperative_core::projects::merge_contributions` so when multiple agents or workers submit work packets into a shared asset/project, their changelog records and effort/compute ledgers merge deterministically by stable entry ID without double-counting effort or losing attribution.
- **Keep Strict Replay Intact:**
  - Because `ProductionSession` strict replay requires deterministic comparison of generated geometry/media outputs, keep `synth/production.json` (or the deterministic output-manifest digest) separate from the append-only `synth/changelog.json` / `.q42j` metadata stream so adding a review signature, annotation, or settlement receipt updates the changelog envelope without falsifying geometry determinism.

#### REQ-6.4: Dual-Regime Licensing & Obligation Contracts Attached to Assets (`synth/license-contract.q42.yaml`)
Every asset (and sub-asset in an HMC bundle) must be able to carry a machine-readable semantic licensing & obligation contract (`synth/license-contract.q42.yaml` + compiled `.q42` / `.q42j` `NQuin` rules) wired to `qualia-cooperative-core` and `qualia-core-db/src/net/qdnf/economics/obligation.rs`, explicitly supporting **two distinct economic/licensing regimes** (plus composite bundles that mix them):

1. **Regime A — Pay-Down to Commons / Humanitarian ICT (`PermissiveCommonsPaydown`):**
   - **Target Domain:** Biosphere, flora/fauna/anatomy models, scientific/universe knowledge, civics scenarios, SDGs, and humanitarian ICT (`pc:Work` under `permissive-commons.n3` + `humanitarian-equity-framework`).
   - **Mechanics:**
     - Defines a finite, audited obligation pool (`target_t` / `pc:obligationCost = productionCost * roiMultiplier`, where `roiMultiplier` is SHACL-capped inside `[1.0, pc:MAX_ROI]`).
     - Tracks `settled_s` (verified compensation paid to contributors), `discharged_w` (authorized non-cash discharge), `holds_h` (active holds), and `remaining_recovery = max(0, T - S - W - H)`.
     - **Human-Rights / Humanitarian No-Charge Eligibility:** Qualifying human-rights, educational, or non-commercial civic uses may be permitted at no charge (`OP_PERMIT`) before discharge—Explicitly recorded as a **permitted no-charge use**, *never* as a false payment, contributor waiver, or pool discharge (`REQ-AGR-005`, `REQ-AGR-009`).
     - **Commercial Recovery & Self-Extinguishing Discharge:** Qualifying commercial uses pay into the bounded obligation pool (scaled by agent category via `pc:LicenseTerm`). The moment accumulated verified settlements reach `target_t` (`is_commons_discharged(pool, cost) == true`), the economic obligation transitions from `pc:Outstanding` (`METADATA_LANE_OBLIGATION_RECOVERY`) to `pc:Discharged` (`METADATA_LANE_PERMISSIVE_COMMONS`), freeing the asset into the true commons globally while **permanently preserving contributor attribution (`pc:attributedTo`)**.

2. **Regime B — Perpetual Royalty / Ongoing Revenue (`PerpetualCreativeRoyalty`):**
   - **Target Domain:** Cultural and artistic creative works such as an artist's song (`.qac`/`.wav`), a film (`.mp4`), fiction/art, or proprietary commercial creations (`schema:CreativeWork` / `hro:` rights instruments).
   - **Mechanics:**
     - Does **not** extinguish into zero-cost public domain upon reaching initial production cost (`extinguishes_on_cost_recovery: false`).
     - Defines ongoing royalty terms (`per_stream`, `per_seat`, `commercial_sync`, `derivative_revenue_share_bps`), duration/term (`perpetual` or statutory copyright term), and recursive upstream ancestor splits (`socioeconomics::ancestor_royalty` / `royalty_tree_total`) so contributors and originating artists continue to receive revenue share across the lifetime of the work and its derivatives.
     - Retains `spc:ArtifactAttachedRule` (`travelsWithArtifact: true`) for moral rights, attribution, and prohibited uses (e.g., non-consensual synthetic voice/likeness cloning or prohibited repurposing).

3. **Guardrails Across Both Regimes (`REQ-AGR-003` – `REQ-AGR-010`):**
   - Keep **draft/proposed contract terms**, **human-adopted agreement releases**, **valued obligations**, and **verified settlement receipts** strictly distinct (`Activity != Payable Debt` and `Pool Funded != Contributor Paid`, per `contribution_lifecycle.rs`).
   - Expose CLI and MCP inspection/authoring commands (`synth changelog`, `synth license`) so agents and humans can inspect an asset's contributor DAG, revision history, compute/token economics, licensing regime (`PermissiveCommonsPaydown` vs. `PerpetualCreativeRoyalty`), and remaining obligation balance directly from the `.hmc` or project workbench.

---

## 7. Articulation, Joint Budgets, Multi-Regime Flexibility (Trees, Leaves, Water, Fauna Tails, Human Musculoskeletal) & Graceful Degradation

### Current Architecture & Why the "Joint Limit" Debate Happened
Inspection of `qualia-core-db` and `etalon-render` reveals the exact constants and architectural split that caused confusion around joint counts:

1. **Three Different Fixed Joint Ceilings in the Stack:**
   - **`FAUNA_MAX_JOINTS = 24`** ([qualia-core-db/src/render/skeletal/fauna_rig.rs](file:///c:/github/qualiaDB/crates/qualia-core-db/src/render/skeletal/fauna_rig.rs#L34)): The built-in 13-archetype fauna generator uses a fixed 24-joint cardinal rig (`pelvis_root`, `spine`, `chest`, `neck`, `head`, `jaw`, 4×3 limb joints, 4 tail joints `tail_base`..`tail_distal_tip`, `mid_thoracic`, `jaw_mandible`).
   - **`MAX_JOINTS = 128` / `MAX_GPU_RIG_JOINTS = 128` / `MAX_DCA_BODIES = 128`** ([etalon-render/src/skeletal/mod.rs](file:///c:/github/etalon/crates/etalon-render/src/skeletal/mod.rs#L125) and [gpu_skin.rs](file:///c:/github/etalon/crates/etalon-render/src/skeletal/gpu_skin.rs#L15)): The `SKL1` rigid motor skinning palette and Featherstone Divide-and-Conquer Articulated-Body (`dca_multibody.rs`) GPU buffers cap rigid skeletal palettes at **128 joints** (`8 KiB` per instance, 4 bone weights per vertex).
   - **`SKL2_MAX_JOINTS = 256` & `MAX_COSSERAT_NODES = 256`** ([skeletal_section/skl2.rs](file:///c:/github/qualiaDB/crates/qualia-core-db/src/container_10d/skeletal_section/skl2.rs#L30) and [cosserat_rod.rs](file:///c:/github/etalon/crates/etalon-render/src/skeletal/cosserat_rod.rs#L10)): The newer `SKL2` biological skeletal container supports **256 joints** per section and already defines **5 distinct solver regimes** (`KinematicSolverRegime` in [skl2.rs](file:///c:/github/qualiaDB/crates/qualia-core-db/src/container_10d/skeletal_section/skl2.rs#L114-L121)) plus `ActuationPathwayRecord` (musculotendon force, fiber length, tendon slack length, pennation angle, moment arm, activation $\tau$) and `LoopClosureConstraintRecord`.
2. **Why Trees, Leaves, Water, and Muscular Continuum Should Not All Be Crammed into a 24- or 128-Bone Rigid `SKL1` Rig:**
   - A eucalypt or ghost gum (`Corymbia aparrerinja`) has a trunk, primary/secondary/tertiary branches, thousands of twigs, and tens of thousands of leaves. Trying to assign an individual `SKL1` rigid bone to every twig and leaf hits the 128/256 joint ceiling immediately.
   - Conversely, a kangaroo's muscular tail (`Macropus giganteus`), a human arm/neck/spine with soft tissue and fascia, fluttering leaves, and flowing water operate under different physical regimes—all five of which **already exist in `etalon-render/src/skeletal/` and `qualia-core-db/src/container_10d/`**, but need unified hierarchical composition and LOD degradation in Synth:
     - **Regime I (`ArticulatedRigidBodyDca`, `dca_multibody.rs`):** Endoskeletal bones (human arm, leg, neck, thoracic cage) with BMD-scaled 6×6 Featherstone spatial inertia, closed-loop constraints (e.g., radius–ulna pronation/supination, pelvis–sacrum), and dual-quaternion skinning (`gpu_skin.rs`).
     - **Regime II (`CosseratRodContinuum`, `cosserat_rod.rs`):** Compliant axial spines, kangaroo tails (cantilevered pentapedal propulsion + fast hopping counterbalance), vertebrate necks, stems, and tree branches modeled as geometrically exact Cosserat rods with Darboux bending/twist strain ($k_{\text{bend}}, k_{\text{twist}}$) and anisotropic wind/fluid drag.
     - **Regime III (`ExoskeletalCuticleShell`, `exoskeletal_cuticle.rs`):** Arthropod sclerites, bicondylar hinges, apodeme actuators, and hemolymph hydraulics.
     - **Regime IV (`HydrostaticMuscularContinuum`, `vbd_soft_tissue.rs` + `correspondence_section.rs` `FEMC`):** Volumetric soft tissue, active Hill-type muscle contraction along fiber direction $\mathbf{a}_0$ (`sigma0_max_pa`, `activation`, `muscle_quality_eta`), Ogden-Holzapfel collagen fibers, Gent dermal strain-hardening, and fascia sliding planes—bound to the high-res visual surface mesh via `FEMC` barycentric tetrahedral bindings (`VertexTetBinding`).
     - **Regime V (`TurgorAuxinGrowth` + `PROC1` Procedural Motor Oscillations + Continuum Fluid/Water):** Botanical FSPM metamers with turgor cavity pressure ($P_{\text{cav}}\pi r^2$), phototropic/gravitropic auxin bending (`cosserat_rod.rs`), coupled with `PROC1` GPU vertex/motor harmonic oscillations ([procedural_synthesis.rs](file:///c:/github/qualiaDB/crates/qualia-core-db/src/container_10d/procedural_synthesis.rs#L208-L224)) for thousands of leaves/twigs, and `solve_continuum_fem_lbm` (D3Q19 Lattice-Boltzmann / shallow-water wave fields) for water.

### Update Requirements for Multi-Regime Flexibility & Graceful Degradation

- **REQ-7.1 (Hierarchical Multi-Regime Motion Architecture — Macro Rig + Cosserat Chains + GPU Procedural Field):**
  - Instead of forcing all motion through a single flat bone table, Synth's rigging and kinetics morphisms must compose three coupled layers inside `.10d`:
    1. **Primary Structural Skeleton (`SKL2` / `DCA`, up to 256 macro joints):** Trunk & primary boughs for trees; primary endoskeleton for human/fauna (spine, neck, limbs, digits, jaw).
    2. **Compliant Continuum & Muscular Chains (`CosseratRod` + `ActuationPathwayRecord` + `VbdSoftTissue` / `FEMC`):**
       - **Kangaroo Tail / Flexible Spine & Neck:** Multi-segment Cosserat rod coupled with Hill-type musculotendon `ActuationPathwayRecord`s (capturing elastic tendon energy return in macropod tails/Achilles tendons and cervical/lumbar flexion in humans).
       - **Muscular & Soft-Tissue Deformation:** `VbdSoftTissue` tetrahedral cages + `FEMC` (`SectionType::CorrespondenceMap`) and `MRP1` pose-space corrective morph targets for biceps/quadriceps bulging, fascia sliding, and breathing/cardiac wall motion.
       - **Tree Branches:** Hierarchical Cosserat rods parameterized by Murray's law radii ($r^3$ branching) and wood Young's/shear modulus ($E, G$).
    3. **Instanced / Vertex-Field Micro-Motion (`PROC1` + Pivot-Baked Foliage & Continuum Water):**
       - **Leaves, Petioles & Twigs:** Do not consume `SKL2` joint slots. Encode branch-level hierarchy pivot phases in vertex attributes / instance overrides (`ArchetypeRefSection`) driven by `PROC1` (`evaluate_pga_motor_oscillation`) + wind curl-noise (`step_simulation_zone`), giving independent bending and flutter to 50,000+ leaves in zero heap on the GPU.
       - **Water & Fluids:** Drive water via `hypermedia.kinetics.solve_continuum_fem_lbm` (D3Q19 LBM) / spectral Gerstner wave heightfields and `.emf` / `.m64` advection rather than skeletal bones.

- **REQ-7.2 (Graceful Degradation — 4-Tier Kinematic & Structural LOD Budget):**
  - Tag every `SKL2` joint, `CosseratElement`, and `VbdTetElement` with a topological **Kinematic LOD Tier (`KLOD0`..`KLOD3`)** and a parent collapse rule so resource-constrained devices (or distant camera frustums / low RAM/VRAM profiles) degrade smoothly without breaking the asset:
    - **`KLOD0` (Hero / Cinema / Full Local GPU — Native Vulkan/CUDA/Metal):** Full 256 `SKL2` joints + active `VbdSoftTissue` tetrahedral muscle/fascia solver (`FEMC` correspondence) + full 256-node `CosseratRod` solver + `MRP1` corrective morphs + per-leaf `PROC1` wind flutter.
    - **`KLOD1` (High / Interactive Native & Desktop WebGPU):** Up to 128 active `GpuJointMotorPair` dual-quaternion joints + reduced Cosserat chains (e.g., 8–12 active tail/branch rod segments, child twigs merged into parent bone weights) + 8-channel GPU morph targets (`GpuInstanceMorphWeights`) approximating muscle bulge + `PROC1` foliage wave.
    - **`KLOD2` (Medium / Constrained WASM / WebGL2 / Mid-Distance):** Automatic topological bone pruning to the 24–32 cardinal joints (`FAUNA_MAX_JOINTS = 24` for fauna; trunk + 8 primary boughs for trees). Distal joints (fingers, distal tail tips, secondary branches) accumulate their skinning weights onto their nearest active ancestor joint (`parent_index`) at load time; soft-tissue VBD is bypassed in favor of cheap analytical DQS + `PROC1` vertex sway.
    - **`KLOD3` (Low Proxy / Crowd / Distant Horizon / Degraded 2D):** Root transform + single harmonic `PROC1` motor oscillation or `IMPO` (`impostor_section.rs` hemi-octahedral impostor atlas) with zero per-frame skeletal hierarchy traversal.

---

## 8. Comprehensive Computational Model: "Form Validation" of Scientific/Structural Values, Classification & Reference Resources

### Existing Foundations in `qualia-core-db` and `qualia-cooperative-core`
1. **Multi-Scale Computational Strata (`MSTR` `L0`..`L6` & `CBLD`):**
   - [multiscale_strata_section.rs](file:///c:/github/qualiaDB/crates/qualia-core-db/src/container_10d/multiscale_strata_section.rs#L48-L206) already defines a 7-layer computational hierarchy (`L0` Macro Organism/Assembly -> `L1` Organ/Subsystem -> `L2` Tissue/Follicle/Stomata -> `L3` Harboured Consortia/Microbiome -> `L4` Cellular -> `L5` Molecular/Genomic -> `L6` Quantum Chemistry) and `EmbodimentResolutionState` (`ComputationalOnly`, `ProceduralSynthesized`, `ArchetypeReferenced`, `InlineEmbodied`).
2. **Epistemic & Evidential Classification (`ClaimBasis` & `EpistemicTier`):**
   - [contextual_evidence.rs](file:///c:/github/qualiaDB/crates/qualia-cooperative-core/src/contextual_evidence.rs#L19-L115) and [corpus_envelope.rs](file:///c:/github/qualiaDB/crates/qualia-core-db/src/container_10d/corpus_envelope.rs#L22-L101) already classify every parameter's basis (`Measured`, `Described`, `Derived`, `ModelAssumption`, `ArtisticChoice`, `Unknown`) and 7-tier epistemic status (`DirectlyObserved` down to `PhylogeneticallyBracketed` and `AuthorialIntent`), plus `ObservationMethod` (`q42m:*`) and `BoundCitation` (`assertion_ids`).
3. **Dimensional & Metric Trait Gate:**
   - [metric_trait_gate.rs](file:///c:/github/qualiaDB/crates/qualia-core-db/src/container_10d/metric_trait_gate.rs#L18-L250) validates AABB dimensions and mass against species/artefact QUDT trait envelopes (`AusTraits`, `AFD`, `AVONET`, `made:Artefact`).

### Update Requirements for Computational Model "Form Validation"
- **REQ-8.1 (Domain-Specific Computational Schema / "Form Validator" for Assets):**
  - Introduce an explicit, machine-checkable **Asset Computational Form Validator** (`synth validate-model` / MCP `synth_validate_model`, backed by SHACL shapes + QUDT unit checks + `corpus_envelope.rs`) that treats an asset's computational parameters like a strict validated form before declaring a model scientifically or structurally complete:
    - **Required Field Presence by Subject Class:** Based on the entity's ontological class (e.g., `VascularPlant`, `VertebrateFauna`, `HumanAnatomy`, `FluidBody`, `MechanicalVehicle`), check whether all required physical, anatomical, and kinetic parameters have been populated:
      - *Dimensions & Mass:* Length, width, height, total mass, center of mass, segment densities (`cortical_bmd`, `trabecular_bv_tv`, wood/leaf density, fluid viscosity/density).
      - *Structural & Muscular Mechanics:* Solver regime per part (`KinematicSolverRegime`), joint swing-twist cone limits (`SwingTwistConeLimits`, safe range-of-motion vs. `.q42` injury thresholds), muscle/tendon actuation pathways (`max_force_n`, `optimal_fiber_length_m`, `tendon_slack_length_m`, `pennation_angle_rad`), or bending/turgor moduli ($E, G, P_{\text{cav}}$).
      - *Multi-Scale Strata (`MSTR`):* Which strata (`L0`..`L6`) are populated and their `EmbodimentResolutionState`.
- **REQ-8.2 (Mandatory Value Classification & Multi-Reference Citation Binding):**
  - Every populated value in the computational model form must be classified along three axes—never allowing an unlabelled guess to masquerade as an empirical measurement:
    1. **Value Basis (`ClaimBasis`) & Epistemic Tier (`EpistemicTier`):** Explicitly classify each parameter as `Measured`, `Described`, `Derived`, `StatisticallyCalibrated`, `PhylogeneticallyBracketed`, `ModelAssumption`, `ArtisticChoice`, or `Unknown` (gap).
    2. **Reference Resources (`1..N` Citations per Value):** Allow each field (or field group) to bind one or more reference resources (`reference_resources: Vec<ReferenceBinding>`):
       - Sealed `.q42` corpus assertion IDs (`assertion_ids` from `ecopunk/entities/taxa/{taxon}/*.q42`).
       - Semantic bookmarks / external dataset IRIs (`AusTraits`, `ALA`, `GBIF`, `AVONET`, `OpenSim` biomechanical tables, `IFC4` engineering standards, anatomical atlases, or reference photographs/videos/DOIs stored via `synth_bookmarks`).
       - Observation method (`q42m:*`) and uncertainty/range (`min`, `max`, `std_dev`, `qudt:unit`).
    3. **Completeness & Confidence Scorecard:** Produce a structured form-validation report showing:
       - `populated_required_fields / total_required_fields`
       - Breakdown by classification (`% Measured/Evidenced`, `% Derived/Bracketed`, `% ModelAssumption`, `% ArtisticChoice`, `% Missing/Unknown`)
       - Explicit list of any out-of-range QUDT violations (e.g. `HeightOutOfBounds`, `NonUnitMotor`, impossible swing-twist limits, or zero/missing muscle force on an active joint).

---

## 9. Multi-Format Render & Output Verification Gate (`pass` / `fail` / `error` Matrix for Agents)

### Current Behavior & Gaps
1. **Format Integrity vs. Actual Render Verification:**
   - Currently, `present_3d_integrity.rs` ([src/present_3d_integrity.rs](file:///c:/github/synth/src/present_3d_integrity.rs#L30-L72)) checks binary container headers and chunk CRCs/BLAKE3 digests for `.10d`, `.m64`, and `.emf`, and `ProductionSession` verifies deterministic HMC byte replay.
   - However, **binary container validity does not prove that every intended output format and target profile actually renders or plays back.** For example:
     - A `.10d` mesh might have valid CRC-32C headers but render as a blank/black frame, clip outside the camera frustum, have inverted normals, fail texture budget admission (`HmcTextureAdmissionReport`), or fail to animate its `SKL1`/`SKL2` clip.
     - Required target profiles (**Film**, **Open-World Game**, **Cartoon** per [model-completion-claims.md](file:///c:/github/ecopunk/docs/planning/tech/assets/model-completion-claims.md#L33-L61)) or required runtime targets (**Native GPU offscreen**, **QualiaPortal WASM/WebGPU**, **WebGL2 fallback**, **PNG contact sheet**, **SVG**, **MP4 video**, **WAV/QAC audio**, **DXF/IFC4**) are not automatically smoke-tested with a unified per-format `pass | fail | error` matrix returned to the agent.

### Update Requirements for Multi-Format Render & Output Verification

- **REQ-9.1 (Automated Multi-Format Render & Playback Verification Harness):**
  - Add a unified verification pass (`synth verify-outputs` / MCP `synth_verify_outputs`, and optional `verify_outputs: true` in `synth_produce_present`) that tests the produced asset across **all declared/intended output formats and runtime targets** and returns a structured, machine-readable `pass | fail | error | skipped` matrix to the agent:
    1. **`.10d` Spatial / 3D Manifold & Materials (`native-gpu` & `portal-wasm`):**
       - Parse header, section table, whole-file CRC-32C, `QMSH`, `FNRM`, `MAT4`, `UV01`, `SKL1`/`SKL2`, `FEMC`, `MSTR`.
       - Execute native offscreen GPU render across canonical camera views (`front`, `side`, `three-quarter`, `top`) and verify **non-empty visual metrics**: bounding-box frustum occupancy (`fill_ratio` in `[0.05, 0.95]`, not a blank frame or microscopic dot), non-degenerate normal/depth variance, and zero deferred/refused textures in `HmcTextureAdmissionReport`.
       - Verify QualiaPortal WASM (`load_10d`) compatibility and `KLOD0`..`KLOD3` degradation readiness.
    2. **Motion & Kinetics Verification (`motion-shots` & `.mp4`):**
       - When the asset has `SKL1`/`SKL2` clips, Cosserat rods, or `PROC1` dynamics, sample at least $N \ge 4$ distinct timestamps ($t_0, t_1, t_2, t_3$) and verify that vertex positions/frames actually change over time (`frame_delta_l2 > epsilon`, catching frozen rigs) while staying within QUDT trait envelope bounds (catching exploding meshes/NaNs).
       - For `.mp4` (`capture_deterministic_movie`), verify ISO-BMFF / H.264 macroblock decodability, frame count, and non-black frame luma variance.
    3. **Volumetric & Field Formats (`.emf` & `.m64`):**
       - Verify `Q42EMF64` v4 / `M64` chunk BLAKE3 digests *plus* execute a spectral slice / DDA raymarch probe (`render_spectral_slice` / `raymarch_vox0_dda`) to confirm non-zero field energy and finite voxel bounds.
    4. **2D Raster & Vector Formats (`.png`, `.psd`, `.svg`, `.dxf`):**
       - Verify image dimensions, color depth (`RGBA8` / `RGBA16`), non-zero alpha/color entropy (not all-transparent or solid-color failure), and valid XML/SVG/DXF geometry primitives.
    5. **Acoustic & Telemetry Formats (`.wav`, `.aac`, `.qac`):**
       - Verify container headers, sample rate, duration, non-zero RMS/LUFS loudness (catching silent audio buffers), and zero NaN/clipped samples.
    6. **Semantic & Dossier Formats (`.q42`, `.hmc`, `claims.json`, `changelog.q42.yaml`, `license-contract.q42.yaml`):**
       - Verify N3/SHACL conformance (`validate_shacl`), 48-byte `NQuin` parity, Ed25519 changelog signature verification, and `claims.json` deliverable coverage across **Film**, **Open-World Game**, and **Cartoon** profiles.

- **REQ-9.2 (Structured Agent-Facing Verification Matrix Receipt):**
  - Return a clear, actionable JSON summary in the MCP response and studio UI so the agent immediately sees what passed, what failed, and why—without falsely claiming human visual acceptance:
    ```json
    {
      "schema": "synth.output-verification/v1",
      "overall_technical_status": "fail",
      "human_visual_acceptance": "pending_human_review",
      "form_validation": {
        "status": "pass",
        "populated_required": 28,
        "total_required": 28,
        "classification_summary": {
          "measured_or_described": 19,
          "derived_or_bracketed": 6,
          "model_assumption": 3,
          "artistic_choice": 0,
          "unknown_gaps": 0
        },
        "reference_sources_bound": 4
      },
      "format_checks": [
        {
          "target": "outputs/tree.10d",
          "format": "10d",
          "runtime_profile": "native-vulkan-klod0",
          "status": "pass",
          "metrics": { "vertices": 48200, "triangles": 91400, "frustum_fill_ratio": 0.42, "crc32c_valid": true }
        },
        {
          "target": "outputs/tree.10d",
          "format": "10d-motion",
          "runtime_profile": "cosserat-wind-sway-klod0..klod3",
          "status": "fail",
          "code": "zero_motion_delta",
          "message": "Frames t=0.0s and t=0.5s produced identical vertex positions (L2 delta = 0.0); verify Cosserat/PROC1 wind drive is non-zero."
        },
        {
          "target": "outputs/tree_preview.mp4",
          "format": "mp4",
          "runtime_profile": "chronicle-h264",
          "status": "pass",
          "metrics": { "frames": 24, "width": 1024, "height": 1024 }
        }
      ]
    }
    ```



