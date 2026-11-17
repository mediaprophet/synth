# Integration wave 0.0.41.3 — swarm plan

Date: 17 November 2026. Coordinator: Grok. Roots: `C:\github`.

P00 remains the only accepted packet. Do not invent qualification. One
rustc at a time into `target-production-core` (wasm into
`target-production-core-wasm`). Never rebuild synth `target` /
`target-studio`. Do not kill live MCP. R01–R08 uncommissioned.

**Policy:** WebGPU is preferred. WebGL2 is the required fallback when
WebGPU is absent or unusable. `?webgl2=1` is a force-GL probe, not the
default.

## Incorporation (git)

Worktree HEADs are ancestors of `0.0.41.3`. No leftover dirty overlay.

| Worktree | Branch | Tip | In `0.0.41.3` |
|---|---|---|---|
| production-core QualiaDB | `codex/geometry-production-core` | `2be65bb53` | yes |
| production-core Etalon | `codex/geometry-production-core` | `421af1b` | yes |
| production-core Hypermedia | `codex/geometry-production-core` | `9ab4b4c` | yes |
| production-core Synth | `codex/geometry-production-core` | `3b46e3eb` | yes |
| WebGL2 QualiaDB | `webgl2-portal` | `de125a7b2` | yes |
| WebGL2 Etalon | `webgl2-storage-emulation` | `4c99669` | yes |

Hypermedia swarm branches and `swarm/c-synth` have zero unique commits
versus `0.0.41.3`. `_audit` detached HEADs are older ancestors.

**Not yet product-complete**

- QualiaDB `Cargo.toml` still git-pins Etalon `1cb112d`. Local
  `.cargo/production-core.toml` uses sibling Etalon `143a1d7` (includes
  `4c99669` SSBO-align fix and Forge production-core). Git-dep consumers
  (Hypermedia/Synth default graphs) compile Etalon `1cb112d` and, in
  Hypermedia, also `a326f82`.
- Website `main` `f6bccf1` still path-deps
  `C:/github/_worktrees/qualiadb-webgl2`, with uncommitted WASM rebuild
  and receipt pin `1cb112d`.
- `selected_backend` in website `graphics.rs` reports `navigator.gpu`
  presence, not the adapter that actually came up.

## Cargo receipts (this session)

All logs UTF-8 under each repo `target-production-core/logs/` except wasm.

| Check | Command | Exit | Errors | Warnings of note |
|---|---|---|---|---|
| QualiaDB lib, no default features | `cargo --config .cargo/production-core.toml check --offline --no-default-features -p qualia-core-db -j 1` | 0 | 0 | 0 (GPU/portal not built) |
| Etalon native | `cargo check --offline -j 1 -p etalon -p etalon-hal -p etalon-render -p etalon-wgsl-forge -p naga` | 0 | 0 | 0 |
| Hypermedia | `cargo check --offline -j 1 -p asset-production-vibe` | 0 | 0 | `production_simplify` dead fields; pre-existing `video-h264-codec` / `radiance-color-grading` unused |
| Synth | `cargo check --offline -j 1 -p synth --no-default-features` into `synth/target-production-core` | 0 | 0 | same Hypermedia warnings + `set_adapter_info` unused |
| QualiaDB GPU (default features, sibling Etalon) | same config, **with** default features | 0 | 0 | Forge unused imports in `acoustic_fdtd.rs`; `etalon-wasm` `fetch_update` deprecated; `set_adapter_info` unused |
| QualiaDB portal wasm32 | `--target wasm32-unknown-unknown --no-default-features --features portal` into `target-production-core-wasm` | **0** (I1 re-run, no exclusive RUSTFLAGS) | 0 | getrandom 0.2.17 / 0.4.3 / 0.3.3 compiled; log `cargo-check-qdb-portal-wasm-i1.log` |

## Fallback path as landed

Preferred path in `PortalGpu::try_new_async`:

1. Default backends `BROWSER_WEBGPU \| GL`.
2. `new_instance_with_webgpu_detection` strips WebGPU when
   `request_adapter` on `navigator.gpu` fails, leaving GL.
3. GL adapters request `adapter.limits()` and pair with
   `READ_ONLY_STORAGE_EMULATION`.
4. `?webgl2=1` forces GL only.

Gaps versus the intended benefit (WebGPU preferred, WebGL2 must work):

- If a WebGPU adapter is chosen and `request_device` / surface fails,
  there is no second attempt on GL-only.
- `adapter_responds()` still probes `BROWSER_WEBGPU` only.
- Website `selected_backend` can say `webgpu` while the device is GL.
- Portal wasm check now compiles (`EXIT=0`, I1). Device/surface GL retry
  and honest `selected_backend` remain I2.

Subagent packets (exact leases and prompts):
`plan/integration-wave-0413-work-packets.md`,
`plan/i0413/*.md`, `integration-wave-0413-leases.md`.

## Swarm packets

Coordinator owns pins (`Cargo.toml` / `Cargo.lock`), `portal_gpu/constructors.rs`
backend policy, and this plan. One rustc. Independent review is a
different agent.

| ID | Owner | Write boundary | Objective | Depends |
|---|---|---|---|---|
| I0 | Coordinator | QualiaDB/Hypermedia/Synth etalon `rev` + lock | After I1 compiles, bump git pin to Etalon `143a1d7538ceae867ceab1dc7e43126bc688752f` so git-dep graphs match sibling `0.0.41.3`. Do not pin `1cb112d`. | I1 green (done) |
| I1 | QualiaDB + Etalon | getrandom/`RUSTFLAGS`/`Cargo.toml` wasm cfg only | **Executed.** Portal wasm check EXIT=0. Log `cargo-check-qdb-portal-wasm-i1.log`. | none |
| I2A | QualiaDB portal | `constructors.rs`, `capabilities.rs`, `portal/mod.rs` getter | WebGPU first; on device/surface failure retry GL. Expose live backend getter. Keep `?webgl2=1` as force-GL. | I1 |
| I2B | website graphics | `graphics.rs`, `boot.rs` if needed | `selected_backend` from I2A getter, not `navigator.gpu`. | I2A |
| I3 | Hypermedia | `production_simplify.rs` only | Use unread `AdmittedSimplify` fields on the production path. | none |
| I4E | Etalon | `acoustic_fdtd.rs`, `worker_steward.rs`, `gemv.rs` | Clear listed GPU-check warnings. | none |
| I4Q | QualiaDB | `gpu_timing.rs` only | Native unused `set_adapter_info`; do not delete (wasm I2A uses it). | I2A |
| I5 | Website | `native/world-host/Cargo.toml` path, receipt, WASM bundle | Point at `C:/github/qualiaDB`. Rebuild `site/pkg/native-world/`. `copy_aaa_review.py --check`. | I0, I2A, I2B |
| I6 | Evidence | create-only receipts | Desktop Chrome WebGPU; same build with WebGPU disabled (or `?webgl2=1`); record `selected_backend`. Pixel 10 Pro XL Chrome fallback. Not qualification. | I5 |
| I7 | Independent review | none (read-only) | Confirm I1–I6 evidence. Do not accept P22/P23. | I6 |

## Dispatch rules

- I1 is done. S1 spawn (parallel, no cargo, isolation none): I2A, I3, I4E.
- S2: coordinator I0; I2B and I4Q after I2A.
- I0 is coordinator-only, because pin/lock is shared.
- I2A must not change EMF/QAC pipelines except through
  `effective_readonly_storage_binding_size` already landed.
- No `cargo fix` across whole workspaces.
- Website remains the mobile benefit gate; worktree path-dep is stale.
- Prompts: `synth/docs/production-core/plan/i0413/`.

## Out of scope this wave

- Rewinding `origin/0.0.41.2`.
- Checking out Etalon `origin/0.0.41.2` (`057545fc`).
- Marking production-core qualified.
- Public Hypermedia `mode=semantic` through cached `synth.exe`.
