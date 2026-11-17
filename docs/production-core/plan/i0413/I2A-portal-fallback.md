# Packet I2A — WebGPU then GL retry; expose live backend

You are a source-only implementer on github roots. Isolation `none`.
Cwd: `C:\github\qualiaDB`. Packet: I2A. Do not run cargo.

## Constraints

- Roots: `C:\github\{qualiaDB,etalon,hypermedia-production,synth,webizen-website}`.
- Do not commit, push, force-push, kill Synth MCP, or rebuild `synth/target` / `target-studio`.
- Do not run cargo. Coordinator owns rustc.
- P00 is the only accepted packet. Do not mark qualified.
- Do not checkout etalon `origin/0.0.41.2`. Do not bump etalon git `rev`.
- WebGPU is preferred. WebGL2 is the required fallback. `?webgl2=1` is force-GL, not the default.
- Do not edit EMF/QAC pipelines except through `effective_readonly_storage_binding_size` already landed.
- After two failed bounded repairs, stop and report.

## Write lease (only these)

- `C:\github\qualiaDB\crates\qualia-core-db\src\render\gpu\portal_gpu\constructors.rs`
- `C:\github\qualiaDB\crates\qualia-core-db\src\render\portal\capabilities.rs`
- `C:\github\qualiaDB\crates\qualia-core-db\src\render\portal\mod.rs` — only `portal_init_webgpu` / `portal_abort_webgpu` neighbourhood and a new wasm_bindgen getter. Do not reformat the rest of `mod.rs`.

Create-only report:
`C:\github\synth\docs\production-core\evidence\integration-wave-i2a\agent-report.md`

## Current behaviour

`PortalGpu::try_new_async` (`constructors.rs`):

1. `?webgl2=1` forces `Backends::GL`.
2. Else `BROWSER_WEBGPU | GL` via `new_instance_with_webgpu_detection`.
3. One adapter/device/surface attempt. If a WebGPU adapter is chosen and `request_device` or surface configure fails, there is no GL retry.
4. `adapter_responds()` probes `BROWSER_WEBGPU` only.
5. `portal_init_webgpu` returns `bool`. Website `graphics.rs` then guesses `selected_backend` from `navigator.gpu` presence.

`gpu_timing_adapter_info()` already exists and constructors already call `set_adapter_info(adapter.get_info())`.

## Required change

1. Keep `?webgl2=1` as GL-only. Do not retry WebGPU in that mode.
2. Default path: prefer WebGPU. If adapter/device/surface fails on a WebGPU adapter (or mixed instance that came up WebGPU), retry once on a GL-only instance with `adapter.limits()` and the existing `READ_ONLY_STORAGE_EMULATION` pairing. Abort flags still win.
3. `adapter_responds()`: true when a WebGPU adapter or a GL adapter answers. Probe WebGPU first, then GL. No canvas claim, no `request_device`.
4. Add `#[wasm_bindgen] pub fn portal_gpu_backend() -> Option<String>` (name may be `portal_selected_backend`) that reads the live pending/adopted `PortalGpu` via `gpu_timing_adapter_info()` and returns `"webgpu"` for `Backend::BrowserWebGpu` / `Backend::Vulkan` on wasm, `"webgl2"` for `Backend::Gl`, `None` if no device. Do not use `navigator.gpu`.
5. Keep `portal_init_webgpu` returning `bool` so existing JS callers compile. The getter is the honest receipt.
6. Do not touch `gpu_timing.rs` (I4Q). Do not touch website files (I2B).

## Done when

- Source implements retry and getter.
- Agent report lists the retry order, getter name/signature, and files touched.
- No cargo, no commit.

## Return

Packet id, paths written, retry sequence, getter signature, remaining gaps.
