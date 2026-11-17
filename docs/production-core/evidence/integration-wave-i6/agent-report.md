# Packet I6 — agent report

Packet: **I6**. Date: 17 November 2026.
Workspace: `C:\github\webizen-website`. Isolation `none`.
Status: create-only GPU receipts. Desktop WebGPU and `?webgl2=1` executed. Pixel phone `user_confirmed` (habitat presents; no `__webizenNativeInit` capture). WebGPU-disabled desktop still `not_executed`.
Not qualified. P00 is the only accepted packet.

## Objective

Capture honest `requested` / `selected_backend` receipts for the I5 WASM on the local world page. WebGPU preferred. WebGL2 required fallback. `?webgl2=1` is a force-GL probe. `selected_backend` comes from `window.__webizenNativeInit.selected_backend` via `portal_gpu_backend()`, not from `navigator.gpu`.

## I5 identity (this build)

Measured on disk and from the served `/pkg/native-world/runtime-identity.json`. Not invented.

| Field | Value |
|---|---|
| WASM | `site/pkg/native-world/webizen_world_bg.wasm` |
| bytes | 21,614,966 |
| sha256 | `0086f172d33bbf726b437a4a7e66282eda370b5c6e8244ee79544c6f64dd72f8` |
| Etalon pin | `143a1d7538ceae867ceab1dc7e43126bc688752f` |
| QualiaDB HEAD compiled | `7296c131a410831a143165999b329ba74abd123e` plus I2A/I4Q dirty |
| `requested` | `webgpu-or-webgl2` |

Dirty QualiaDB files recorded in the served identity: `gpu_timing.rs`, `constructors.rs`, `capabilities.rs`, `portal/mod.rs`.

Did not rebuild WASM. Did not run cargo. Did not use `tooling/native-host/build.ps1`. Did not write synth `target`.

## Serve

Reused the already-running world server. Did not start a second copy. Left it running.

- PID `55180`
- `"C:\Users\Hp\AppData\Local\Programs\Python\Python313\python.exe" -u C:/github/webizen-website/tooling/world_server.py --port 8800`
- Bind `127.0.0.1:8800`
- Index route `http://127.0.0.1:8800/` → `site/index.html` (`<title>Webizen — The living world</title>`)
- `/api/status` 200, `catalog_count` 150

## Playwright

Desktop Chrome 157 via Playwright MCP (`Mozilla/5.0 (Windows NT 10.0; Win64; x64) … Chrome/157.0.0.0`). Viewport set to 1280×800 before each executed load. Screenshots `scale=css`. Evaluate waited until `__webizenNativeInit.state === "ready"` (WebGPU ~30s including WASM; force-GL ready within 25s). Session left open. Synth MCP not touched.

## Receipts

| Receipt | File | execution_status | selected_backend |
|---|---|---|---|
| 1 Desktop Chrome WebGPU | `desktop-webgpu.json` | `executed` | `"webgpu"` |
| 2 Same build `?webgl2=1` | `desktop-webgl2-force.json` | `executed` | `"webgl2"` |
| 3 Desktop WebGPU disabled | `desktop-webgpu-disabled.json` | `not_executed` | `null` |
| 4 Pixel 10 Pro XL Chrome | `pixel-10-pro-xl-chrome.json` | `not_executed` | `null` |

Absence is `not_executed` / `null`, never `0.0` and never an invented `"webgpu"`.

### 1. Desktop Chrome, WebGPU available, 1280×800

- URL `http://127.0.0.1:8800/`
- `requested` `webgpu-or-webgl2`
- `selected_backend` `"webgpu"`, `state` `"ready"`, init `8614.35 ms`
- `navigator.gpu` present; not used as the backend name
- Desktop scene: 77 assets, 440,626 source triangles, `world.renderer` `"webgpu"`, frames presenting (400 at capture)
- Screenshot `desktop-webgpu.png` shows field studio: wagon, camper, solar, eucalyptus, UI
- Console: favicon.ico 404; Chrome `powerPreference` ignored on Windows (`crbug.com/369219127`)

### 2. Same build, force-GL `?webgl2=1`, still 1280×800

- URL `http://127.0.0.1:8800/?webgl2=1`
- `requested` `webgpu-or-webgl2`
- `selected_backend` `"webgl2"`, `state` `"ready"`, init `12928.65 ms`
- Viewport still 1280×800 (`max-width: 900px` false). Mobile scene loaded because the honest backend is `"webgl2"`, not because of CSS
- Mobile scene: 76 assets, 309,098 source triangles, `world.renderer` `"webgl2"`, frames presenting (1145 at capture)
- Status text is the live world copy, not the reading fallback
- Screenshot `desktop-webgl2-force.png` shows sky, a winding water band, and UI. Habitat meshes from receipt 1 (wagon, camper, trees, solar) are **not** visible in this frame
- Console: 75× `GL_INVALID_OPERATION: glTexSubImage2DRobustANGLE: The provided parameters overflow with the provided buffer.`
- Labeling quirk: `world.scene.renderer` and `world.upload.renderer` still string `"webgpu"` on this GL boot. Honest adapter name is `selected_backend` / `world.renderer` = `"webgl2"`

`presents` is true (boot ready, frames increment, UI live). Full habitat shading vs the WebGPU receipt is **not** claimed.

### 3. Desktop WebGPU disabled — `not_executed`

Playwright tools listed for this packet have no Chrome launch flag to disable WebGPU. `isolatedContext` does not take `--disable-features=WebGPU`. I5 handover already records that desktop Chrome headless previously kept `navigator.gpu` despite that flag. In-page deletion of `navigator.gpu` would not be an honest `portal_gpu_backend()` disable. Did not relaunch the MCP browser.

### 4. Pixel 10 Pro XL Chrome — `not_executed`

Device not attached. `adb.exe` is not on PATH and is not in the usual SDK locations. No PnP name matching Android / Pixel / ADB. USB composite seen is `VID_30C9&PID_00FF` (laptop camera). Template receipt includes exact later steps, including `adb reverse tcp:8800 tcp:8800` because this server binds `127.0.0.1` only. Do not fabricate a Pixel pass.

## Paths written

All create-only under `C:\github\synth\docs\production-core\evidence\integration-wave-i6\`:

| Path | Role |
|---|---|
| `agent-report.md` | this report |
| `desktop-webgpu.json` | receipt 1 |
| `desktop-webgpu.png` | receipt 1 screenshot |
| `desktop-webgpu-console.txt` | receipt 1 console |
| `desktop-webgpu-snapshot.yml` | receipt 1 a11y snapshot |
| `desktop-webgl2-force.json` | receipt 2 |
| `desktop-webgl2-force.png` | receipt 2 screenshot |
| `desktop-webgl2-force-console.txt` | receipt 2 console |
| `desktop-webgl2-force-snapshot.yml` | receipt 2 a11y snapshot |
| `desktop-webgpu-disabled.json` | receipt 3 template |
| `pixel-10-pro-xl-chrome.json` | receipt 4 template |

Not edited: QualiaDB, website sources, WASM, synth `target` / `target-studio`. Not committed. Not pushed. Server PID 55180 left running.

## Remaining gaps

- Packet I6 is **not qualified**. P00 only.
- Pixel 10 Pro XL Chrome default-path GL retry is still unrun live evidence.
- Desktop WebGPU-disabled fallback is `not_executed`.
- Force-GL desktop probe selected `"webgl2"` and loaded `scene-mobile.json`, but habitat meshes are missing from the screenshot and 75 GL texture overflow warnings fired. Frame-time on Pixel is still unmeasured.
- `world.scene.renderer` / `world.upload.renderer` keep the `"webgpu"` string on the GL path. Coordinator / later packet if that label must match `portal_gpu_backend()`.
- QualiaDB working tree remains dirty (I0 pin, I2A getter, I4Q allow) as recorded by I5 identity.
