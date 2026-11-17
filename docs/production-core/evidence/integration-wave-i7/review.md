# Packet I7 — Independent review of I1–I6

Packet: **I7**. Date: 17 November 2026.
Reviewer: independent of implementers. Isolation `none`. Cwd: `C:\github\synth`.
Write boundary: this file only. Product code not edited. Cargo not run. MCP not killed.
Not committed. **P00 remains the only accepted packet.** P22/P23 are not accepted.
Production-core is **not qualified**.

## Verdict

| Packet | Result | Basis |
|---|---|---|
| I1 | **PASS** | Portal wasm check EXIT=0; getrandom 0.3.3 compiled; RUSTFLAGS empty; cfg durable |
| I0 | **PASS** | Git pin is `143a1d7538ceae867ceab1dc7e43126bc688752f`; cargo re-check explicitly deferred |
| I2A | **PASS** | Source: WebGPU first, one GL retry, `?webgl2=1` force-GL; getter from live adapter |
| I2B | **PASS** | `selected_backend` from `portal_gpu_backend()`, not `navigator.gpu` |
| I3 | **PASS** | Production path reads `container_10d` / `admitted` / `decoded`; rustc not re-run (lease) |
| I4E | **PASS** | Named Etalon warning sites edited; rustc not re-run (lease) |
| I4Q | **PASS** | Native-only `allow(dead_code)` on `set_adapter_info`; method kept |
| I5 | **PASS** | Path-deps `C:/github/qualiaDB`; WASM rebuilt; etalon pin asserted |
| I6 | **PASS** | Honest receipts: WebGPU and force-GL executed; Pixel and WebGPU-disabled `not_executed` |

Wave I1–I6: **not landed** (1 bug). **Not qualified.**

## Confirmations

### 1. I1 getrandom / RUSTFLAGS / durable cfg — confirmed

I1 log `C:\github\qualiaDB\target-production-core-wasm\logs\cargo-check-qdb-portal-wasm-i1.log`:

- Header: `RUSTFLAGS=''`, `CARGO_ENCODED_RUSTFLAGS=''`
- `Checking getrandom v0.2.17`
- `Compiling getrandom v0.4.3`
- `Compiling getrandom v0.3.3`
- `Finished dev profile [unoptimized + debuginfo] target(s) in 3m 24s`
- `EXIT=0`

Durable cfg (independent of the empty process RUSTFLAGS):

- QualiaDB `.cargo/config.toml`: `[target.wasm32-unknown-unknown] rustflags = ['--cfg', 'getrandom_backend="wasm_js"']`
- QualiaDB `.cargo/production-core.toml`: same cfg plus `'-C', 'debuginfo=0'`
- QualiaDB workspace `Cargo.toml`: `getrandom = { version = "0.4.3", features = ["wasm_js"] }`
- Etalon `.cargo/config.toml`: wasm rustflags include the same cfg

**I5 wasm rebuild RUSTFLAGS (honest):** `native/world-host` has **no** `.cargo/` directory and **no** `.cargo/config.toml`. QualiaDB cargo config does not apply to that crate. I5 reports the first `cargo build --target wasm32-unknown-unknown --release --offline --locked` with RUSTFLAGS unset failed on getrandom 0.3.3 `compile_error!`, then succeeded after `RUSTFLAGS='--cfg getrandom_backend="wasm_js"'`. This reviewer did not re-run cargo. No I5 cargo log of that first failure is on disk; the served WASM and `runtime-identity.json` exist. Exclusive RUSTFLAGS on world-host must keep the cfg; it is not durable in that crate the way I1’s QualiaDB check is.

### 2. I2A WebGPU first, GL retry, `?webgl2=1` force-GL — confirmed in source

`PortalGpu::try_new_async` (`constructors.rs`):

1. `location_forces_webgl2()` (`?webgl2=1`) → `try_new_gl_instance` only. No WebGPU attempt, no WebGPU retry.
2. Default → `try_new_webgpu_instance` (`Backends::BROWSER_WEBGPU`), adapter/device **before** canvas claim.
3. On non-abort WebGPU failure → one `try_new_gl_instance` (`Backends::GL`). Combined error `{webgpu_error}; gl retry: {gl_error}`.

`adapter_responds()` probes WebGPU (no canvas, no device) then GL (detached canvas, no presentation claim, no device). Capability field `webgpu_adapter_responded` still uses the WebGPU-only probe.

Getter `portal_gpu_backend()` reads pending/adopted adapter info. Does not use `navigator.gpu`.

Known in-source limit (nit, not a packet fail): if WebGPU `request_device` succeeds and **then** surface configure / `from_device` fails, the presentation canvas already has a WebGPU context; the GL retry still runs and will typically fail.

I2A itself did not run cargo. I5 compiled the dirty `constructors.rs` / `capabilities.rs` / `portal/mod.rs` into the served WASM.

### 3. I2B `selected_backend` from live adapter — confirmed

`native/world-host/src/runtime/graphics.rs`:

- Imports `portal_gpu_backend`.
- After `portal_init_webgpu` `Ok(true)`, `selected_backend` is `"webgpu"` / `"webgl2"` / JSON null from that getter.
- `navigator.gpu` is not read (comment only).
- Failure path: `state: "refused"`, `selected_backend: null`.

`boot.rs` was not edited. It already keys `data-scene-mobile` off `graphics_init["selected_backend"] == "webgl2"` **or** `(max-width: 900px)`.

Live I6 force-GL receipt: `hasNavigatorGpu: true` and `selected_backend: "webgl2"`. That is the honesty check: presence of `navigator.gpu` did not name the backend.

### 4. I0 git pin — confirmed, not deferred

Pin `143a1d7538ceae867ceab1dc7e43126bc688752f` is in:

- QualiaDB workspace `Cargo.toml` (etalon/naga revs)
- QualiaDB workspace `Cargo.lock` (`1cb112d` = 0)
- Hypermedia crate `Cargo.toml` files and `Cargo.lock` (`1cb112d` = 0, `a326f82` = 0)
- Website world-host `Cargo.lock` and served `runtime-identity.json`

Cargo check of the new git pin was **explicitly deferred** (I5 held the rustc slot). Nested QualiaDB locks `crates/qualia-etalon-sensing-bridge/Cargo.lock` and `tools/accelerator-harness/Cargo.lock` still cite `a326f82` (nit). Synth does not git-pin etalon.

### 5. I5 path-deps — confirmed

`native/world-host/Cargo.toml`:

```
qualia-core-db = { path = "C:/github/qualiaDB/crates/qualia-core-db", default-features = false, features = ["portal"] }
vibe = { path = "C:/github/qualiaDB/crates/vibe" }
```

Patches: `C:/github/qualiaDB/vendor/imap-proto`, `C:/github/qualiaDB/crates/vibescript-abi`.

No `_worktrees/qualiadb-webgl2` in world-host `Cargo.toml` / `Cargo.lock` / `receipt.py`. `receipt.py` fingerprints `core=Path('C:/github/qualiaDB/crates/qualia-core-db')` and asserts etalon `143a1d7538ceae867ceab1dc7e43126bc688752f` (24 wasm provider crates, 31 lock providers).

Served identity matches I6 receipts: wasm 21,614,966 bytes, sha256 `0086f172d33bbf726b437a4a7e66282eda370b5c6e8244ee79544c6f64dd72f8`. QualiaDB HEAD compiled `7296c131a410831a143165999b329ba74abd123e` plus tracked dirty I2A/I4Q files.

### 6. I6 receipts — confirmed

| Receipt | execution_status | selected_backend | Independent check |
|---|---|---|---|
| `desktop-webgpu.json` | `executed` | `"webgpu"` | PNG shows field studio: wagon, camper, solar, eucalyptus, UI |
| `desktop-webgl2-force.json` | `executed` | `"webgl2"` | PNG shows sky, winding water band, UI; habitat meshes absent |
| `desktop-webgpu-disabled.json` | `not_executed` | `null` | Honest; Playwright tools had no disable-WebGPU launch flag |
| `pixel-10-pro-xl-chrome.json` | `not_executed` | `null` | Honest; no adb / no Pixel attached |

Absence is `not_executed` / `null`, never `0.0` and never an invented `"webgpu"`. Force-GL URL is `http://127.0.0.1:8800/?webgl2=1` at viewport 1280×800 (`media_max_width_900px: false`); mobile scene loaded because honest backend is `"webgl2"`. Console: **75** `GL_INVALID_OPERATION: glTexSubImage2DRobustANGLE: The provided parameters overflow with the provided buffer.` (`materials_loaded` = 75).

That visual incompleteness is **BUG-1**, not a nit.

### 7. Intended WebGPU-then-GL benefit — only partly evidenced

Policy: WebGPU preferred; WebGL2 required fallback when WebGPU is absent or unusable; `?webgl2=1` is a force-GL probe, not the default.

| Claim | Evidence |
|---|---|
| WebGPU preferred when available | **Yes.** Desktop default boot: `selected_backend=webgpu`, habitat presents |
| Honest live backend name | **Yes.** Force-GL named `webgl2` while `navigator.gpu` remained present |
| Force-GL probe boots GL | **Yes, adapter/device.** `state=ready`, frames increment, live status copy |
| Force-GL presents the world | **No.** Habitat meshes missing; 75 texture overflows (BUG-1) |
| Default-path retry (WebGPU fail → one GL attempt) | **Not live.** Source in I2A. Pixel and WebGPU-disabled are `not_executed`. Force-GL is a different branch (`location_forces_webgl2()` skips WebGPU entirely) |

The intended benefit is therefore **not evidenced end-to-end**. Adapter selection and honest naming are sourced and, on desktop, partly live. Automatic retry on a WebGPU-absent device is unrun. The only executed GL path does not visually present the habitat.

## Per-packet notes

### I1 PASS

Coordinator-executed portal wasm check. Empty RUSTFLAGS did not drop the cfg because QualiaDB cargo config supplies it. I1 log still records the Etalon warnings I4E later edited in source; that log is pre-I4E and is not a re-check.

### I0 PASS

Pin landed on the workspace graphs this wave cares about. Cargo check of the new pin not executed this slot (stated). Nested tool/crate locks still on `a326f82` (nit).

### I2A PASS (source; compiled by I5)

Retry sequence matches the lease. Getter and `adapter_responds` match the reports. Remaining surface-after-claim gap is documented by the implementer and is a nit.

### I2B PASS (source; compiled by I5)

`graphics.rs` consumes the I2A getter. I6 live values match.

### I3 PASS (source-only)

`verify_sealed_container` reads all three `AdmittedSimplify` fields on the production path (`decimate_semantic_admitted_10d` and `decimate_semantic_admitted`). No `#[allow(dead_code)]`. rustc clearance unverified.

### I4E PASS (source-only)

`acoustic_fdtd.rs` unused GPU/schedule imports removed. `worker_steward.rs` uses `try_update`. `gemv.rs` parameter is `_options`. `ffn.rs` still has unused `options` (out of lease; nit). Local etalon edits are **not** in the I5 WASM (identity: git checkout `143a1d7`, “no local provider overrides”).

### I4Q PASS (source; compiled by I5)

```
#[cfg_attr(not(all(target_arch = "wasm32", feature = "portal")), allow(dead_code))]
pub fn set_adapter_info(...)
```

Method body unchanged. Not wrapped in `cfg(wasm32)`.

### I5 PASS

Retarget, lock, receipt, WASM, `copy_aaa_review.py --check` exit 0 as reported. Identity records dirty QualiaDB honestly. world-host wasm cfg is process RUSTFLAGS, not a crate `.cargo/config.toml`.

### I6 PASS (evidence packet)

Create-only receipts exist. Executed vs `not_executed` is honest. I6’s leased job was capture, not fixing GL textures. The product defect it captured is BUG-1 against the wave benefit.

## Bugs

1. **Force-GL visual incompleteness (required WebGL2 fallback does not present the habitat).** Independent review of `desktop-webgl2-force.png` vs `desktop-webgpu.png`: WebGPU shows wagon, camper, solar, trees, ground; force-GL shows sky, a water band, and UI only. Console is 75× `glTexSubImage2DRobustANGLE` overflow, equal to `materials_loaded=75`. Policy and swarm plan require WebGL2 to work as the fallback, not merely to name `selected_backend=webgl2` and increment frames. `presents=true` with missing habitat is not a working fallback. This is a bug, not a nit.

## Nits

1. Default-path WebGPU-then-GL retry is source-only. Pixel 10 Pro XL Chrome and desktop WebGPU-disabled are honest `not_executed`. I6 allowed `?webgl2=1` as the GL probe substitute; that is not the retry branch.
2. On the GL boot, `world.scene.renderer` and `world.upload.renderer` still string `"webgpu"`. Honest names are `selected_backend` / `world.renderer` = `"webgl2"`.
3. I2A: WebGPU device success then surface failure claims the canvas; GL retry will typically fail `getContext("webgl2")`.
4. After QualiaPortal adopt, `portal_gpu_backend` uses the stash-time label. Recovery that drops `self.gpu` without `portal_abort_webgpu` can report the last armed backend.
5. I4E did not touch `ffn.rs` unused `options` (I1 log still names it). Lease listed `gemv.rs` only.
6. I3 / I4E / I4Q rustc not re-run. I1 log warnings are stale relative to current Etalon/Hypermedia source.
7. Nested QualiaDB `Cargo.lock` files (`qualia-etalon-sensing-bridge`, `accelerator-harness`) still pin etalon `a326f82`.
8. world-host has no durable wasm getrandom cfg. I5 had to set process RUSTFLAGS. `tooling/native-host/build.ps1` still points `CARGO_TARGET_DIR` at `C:/github/qualiaDB/target` (do not use it).
9. QualiaDB working tree remains dirty (I0 pin, I2A getter, I4Q allow) as recorded in the served identity.
10. I0 cargo check of the new git pin was not executed.
11. I5 WASM resolves etalon from the cargo git checkout of `143a1d7`, not the local I4E-edited `C:\github\etalon` tree.

## Landed / qualified

- **Landed:** **no.** 0 bugs are required to accept this wave as landed. BUG-1 is open.
- **Qualified:** **no.** This wave does not accept P22/P23 and does not mark production-core qualified.
- **Accepted packet:** **P00 only.**

Packets I1–I6 completed their leased work. The intended WebGPU-preferred / WebGL2-fallback **benefit** is not landed: default-path retry is unrun live, and the only executed GL receipt does not present the habitat.

## Paths read (this review)

- `docs/production-core/integration-wave-0413-swarm-plan.md`
- `docs/production-core/plan/integration-wave-0413-work-packets.md`
- `docs/production-core/plan/i0413/{I2A,I6,I7,shared-constraints}.md`
- `docs/production-core/evidence/integration-wave-i{0,2a,2b,3,4e,4q,5,6}/agent-report.md`
- I6: `desktop-webgpu.json` + `.png` + `-console.txt`; `desktop-webgl2-force.json` + `.png` + `-console.txt`; `desktop-webgpu-disabled.json`; `pixel-10-pro-xl-chrome.json`
- I1 log `C:\github\qualiaDB\target-production-core-wasm\logs\cargo-check-qdb-portal-wasm-i1.log`
- QualiaDB `constructors.rs`, `portal/mod.rs` getter, `capabilities.rs`, `gpu_timing.rs`
- QualiaDB `.cargo/config.toml`, `.cargo/production-core.toml`, workspace `Cargo.toml` / `Cargo.lock`
- Etalon `.cargo/config.toml`; I4E sites `acoustic_fdtd.rs`, `worker_steward.rs`, `gemv.rs`, `ffn.rs`
- Hypermedia `production_simplify.rs`; Hypermedia `Cargo.lock`
- Website `graphics.rs`, `boot.rs`, `native/world-host/Cargo.toml` / `Cargo.lock`, `tooling/native-host/receipt.py`
- Served `site/pkg/native-world/runtime-identity.json`
