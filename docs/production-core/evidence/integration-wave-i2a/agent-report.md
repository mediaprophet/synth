# Packet I2A — agent report

Packet: **I2A**. Date: 17 November 2026.
Workspace: `C:\github\qualiaDB`. Isolation `none`. Source-only; cargo not run.
Status: source implemented. Not compiled. Not qualified. P00 is the only accepted packet.

## Objective

WebGPU is preferred. WebGL2 is the required fallback. `?webgl2=1` remains force-GL.
Expose a live backend getter from the pending/adopted `PortalGpu` adapter info, not `navigator.gpu`.

## Retry sequence

`PortalGpu::try_new_async` (`constructors.rs`):

1. **`?webgl2=1`** → GL-only instance. No WebGPU attempt and no WebGPU retry.
2. **Default** → WebGPU-only instance (`Backends::BROWSER_WEBGPU`):
   1. abort check
   2. `request_adapter` without a canvas (HighPerformance → None → LowPower)
   3. abort check
   4. `request_device` (`webgpu_minimum_limits()`)
   5. abort check
   6. claim canvas, `create_surface`, configure, `from_device`
3. If that WebGPU adapter/device/surface path fails **and** abort is not set → **one** GL-only retry (`Backends::GL`):
   1. abort check
   2. claim canvas, `create_surface` (WebGL2 requires a surface before adapter)
   3. `request_adapter` with that surface
   4. `request_device` with `adapter.limits()` (pairs with `DownlevelFlags::READ_ONLY_STORAGE_EMULATION`)
   5. configure, `from_device`
4. Abort flags still win at every step. GL retry errors that are not abort are combined: `{webgpu_error}; gl retry: {gl_error}`.

Adapter/device for WebGPU run **before** `getContext("webgpu")` so a failed device request leaves the presentation canvas free for the GL retry.

## Getter

```rust
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn portal_gpu_backend() -> Option<String>
```

- Reads the pending `PortalGpu` via `gpu_timing_adapter_info()`.
- After adopt, `PENDING_GPU` is taken; a thread-local label captured at stash time keeps the last armed backend.
- `"webgpu"` for `Backend::BrowserWebGpu` and `Backend::Vulkan`.
- `"webgl2"` for `Backend::Gl`.
- `None` if no device is armed. Does not use `navigator.gpu`.
- `portal_init_webgpu` still returns `Result<bool, JsValue>`.

## `adapter_responds()`

True when a WebGPU adapter **or** a GL adapter answers. Probe order: WebGPU (no canvas, no device), then GL (detached canvas surface only, no presentation claim, no device). Capability field `webgpu_adapter_responded` still uses `webgpu_adapter_responds()` so a GL-only answer cannot be reported as a WebGPU adapter.

## Files touched

| Path | Role |
|---|---|
| `C:\github\qualiaDB\crates\qualia-core-db\src\render\gpu\portal_gpu\constructors.rs` | WebGPU-first init, one GL retry, adapter probes |
| `C:\github\qualiaDB\crates\qualia-core-db\src\render\portal\capabilities.rs` | Honest WebGPU adapter probe; live receipt is `portal_gpu_backend` |
| `C:\github\qualiaDB\crates\qualia-core-db\src\render\portal\mod.rs` | `portal_init_webgpu` / `portal_abort_webgpu` neighbourhood; `portal_gpu_backend` |
| `C:\github\synth\docs\production-core\evidence\integration-wave-i2a\agent-report.md` | This report |

Not edited: `gpu_timing.rs` (I4Q), website files (I2B), EMF/QAC pipelines, etalon git rev, `Cargo.*`.

## Remaining gaps

- Cargo / wasm32 rustc not run (lease). Coordinator owns compile.
- If WebGPU `request_device` succeeds and **then** surface configure / `from_device` fails, the presentation canvas already has a WebGPU context. The GL retry still runs and will typically fail (`getContext("webgl2")` null). Adapter/device failure before claim is the retried case.
- After `QualiaPortal` adopt, the getter uses the stash-time label. Recovery that drops `self.gpu` without `portal_abort_webgpu` can still report the last armed backend. `adopt_pending_gpu` / `recovery.rs` were outside the write neighbourhood.
- I2B must consume `portal_gpu_backend` after `portal_init_webgpu` returns true. This packet does not edit website `graphics.rs`.
- P00 only. Not qualified.
