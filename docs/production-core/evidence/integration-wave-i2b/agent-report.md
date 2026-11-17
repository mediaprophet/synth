# Packet I2B — agent report

Packet: **I2B**. Date: 17 November 2026.
Workspace: `C:\github\webizen-website`. Isolation `none`. Source-only; cargo not run.
Status: source implemented. Not compiled. Not qualified. P00 is the only accepted packet.

## Objective

`selected_backend` must name the adapter that actually came up, not whether
`navigator.gpu` exists. WebGPU preferred; WebGL2 required fallback.
`?webgl2=1` remains force-GL (Qualia I2A).

## Getter consumed

I2A getter is present at
`C:\github\qualiaDB\crates\qualia-core-db\src\render\portal\mod.rs`:

```rust
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn portal_gpu_backend() -> Option<String>
```

It reads pending/adopted `PortalGpu` via `gpu_timing_adapter_info()`.
Returns `"webgpu"` for `Backend::BrowserWebGpu` / `Backend::Vulkan`,
`"webgl2"` for `Backend::Gl`, `None` if no device. Does not use
`navigator.gpu`. `portal_init_webgpu` still returns `Result<bool, JsValue>`.

`navigator.gpu` was not used as a substitute.

## How `selected_backend` is sourced

`native/world-host/src/runtime/graphics.rs` `initialise`:

1. Publish `requested: "webgpu-or-webgl2"` (unchanged).
2. Await `portal_init_webgpu(canvas)`.
3. **`Ok(true)`** — `state: "ready"`. `selected_backend` is taken from
   `portal_gpu_backend()` after that success:
   - `"webgpu"` → `"webgpu"`
   - `"webgl2"` → `"webgl2"`
   - anything else / `None` → JSON `null`
4. **`Ok(false)` / `Err`** — `state: "refused"`, `selected_backend: null`
   (unchanged failure path, including abort + capability probe).

`navigator.gpu` is no longer read.

`boot.rs` was not edited. It already consumes
`graphics_init["selected_backend"]` and loads `data-scene-mobile` when the
honest backend is `"webgl2"` **or** viewport `max-width: 900px`.

## Paths written

| Path | Role |
|---|---|
| `C:\github\webizen-website\native\world-host\src\runtime\graphics.rs` | Honest `selected_backend` from `portal_gpu_backend` |
| `C:\github\synth\docs\production-core\evidence\integration-wave-i2b\agent-report.md` | This report |

Not edited: `boot.rs` (already keyed off the report field),
`native/world-host/Cargo.toml` (I5 / I0), `site/pkg/native-world/` (I5),
QualiaDB, synth `target` / `target-studio`.

## Remaining gaps

- Cargo / wasm32 rustc not run (lease). Coordinator / I5 owns compile and
  WASM rebuild.
- `native/world-host/Cargo.toml` still points at
  `C:/github/_worktrees/qualiadb-webgl2/crates/qualia-core-db`, which does
  **not** export `portal_gpu_backend`. I2A landed the getter on
  `C:\github\qualiaDB`. I5 must retarget before this crate compiles.
- Served `site/pkg/native-world/` still contains the previous WASM (navigator
  heuristic). Live receipts stay dishonest until I5 rebuilds.
- Coordinator follow-up: `boot.rs` now uses `unwrap_or("")`, so a null
  getter does not name WebGPU. Mobile scene still keys off `backend ==
  "webgl2"` or viewport `<= 900px`.
- P00 only. Not qualified.
