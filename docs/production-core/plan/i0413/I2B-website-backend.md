# Packet I2B — Honest selected_backend

You are a source-only implementer on github roots. Isolation `none`.
Cwd: `C:\github\webizen-website`. Packet: I2B. Do not run cargo.

## Constraints

- Do not commit, push, kill Synth MCP, or rebuild synth `target` / `target-studio`.
- Do not run cargo (I5 rebuilds WASM).
- Do not retarget `native/world-host/Cargo.toml` (I5 / I0).
- P00 is the only accepted packet. Do not mark qualified.
- WebGPU preferred; WebGL2 required fallback. `?webgl2=1` is force-GL.
- After two failed bounded repairs, stop and report.

## Depends

I2A must have landed `portal_gpu_backend` / `portal_selected_backend` on QualiaDB
`portal/mod.rs`. Read that getter. If it is missing, stop and report; do not invent
`navigator.gpu` as a substitute.

## Write lease (only these)

- `C:\github\webizen-website\native\world-host\src\runtime\graphics.rs`
- `C:\github\webizen-website\native\world-host\src\runtime\boot.rs` only if it must consume the getter (today it reads `graphics_init["selected_backend"]`).

Create-only report:
`C:\github\synth\docs\production-core\evidence\integration-wave-i2b\agent-report.md`

## Current defect

`graphics.rs` `initialise` sets
`selected_backend` to `"webgpu"` when `navigator.gpu` exists, even if the
adapter that came up is GL.

## Required change

After `portal_init_webgpu` returns `Ok(true)`, set `selected_backend` from the
I2A getter (live `adapter.get_info().backend`). Map GL to `"webgl2"`, WebGPU
to `"webgpu"`. On `Ok(false)` / error keep `selected_backend` null.
Keep `requested: "webgpu-or-webgl2"`. Keep `data-scene-mobile` selection in
`boot.rs` keyed off the honest backend or viewport `<= 900px`.

Do not rebuild `site/pkg/native-world/`.

## Return

Packet id, paths written, how `selected_backend` is sourced, remaining gaps.
