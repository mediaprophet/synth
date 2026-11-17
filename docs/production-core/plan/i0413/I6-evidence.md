# Packet I6 — Create-only WebGPU and WebGL2 receipts

You are an evidence agent. Isolation `none`. Packet: I6.
Cwd: `C:\github\webizen-website`. Do not rebuild synth `target`.

## Constraints

- Create-only writes under
  `C:\github\synth\docs\production-core\evidence\integration-wave-i6\`.
- Do not commit. Do not mark qualified. P00 remains the only accepted packet.
- Do not kill Synth MCP.
- Record what actually happened. Absence is `not_executed` / `unsupported`, never `0.0`.

## Depends

I5 (website WASM rebuilt from github QualiaDB with honest `selected_backend`).

## Required receipts

1. Desktop Chrome, WebGPU available: load the local world page, record
   `window.__webizenNativeInit.selected_backend` (expect `"webgpu"` when
   the adapter is WebGPU), screenshot/console, no qualification claim.
2. Same build with WebGPU disabled or `?webgl2=1`: record
   `selected_backend` (expect `"webgl2"`), that the scene still presents
   (mobile scene is allowed). Force-GL URL is a probe; also try the
   WebGPU-disabled browser path if available.
3. Pixel 10 Pro XL Chrome: if the device is not attached, write a
   template receipt with `execution_status=not_executed` and instructions.
   Do not fabricate a Pixel pass.

Use Playwright MCP when desktop Chrome is available. If it is not, say so.

## Report

`agent-report.md` in the evidence dir plus JSON receipts. Include
`requested`, `selected_backend`, URL, viewport, and whether this is
create-only (not qualified).
