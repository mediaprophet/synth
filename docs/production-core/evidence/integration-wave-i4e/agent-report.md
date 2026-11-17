# Packet I4E — Clear Etalon GPU-check warnings

Agent: `i4e-gpu-warnings`. Date: 17 November 2026.
Workspace: `C:\github\etalon`. Isolation: `none`. Source-only; cargo was not run.
Status: leased warning sites edited. Packet **not accepted**. **Not qualified**. P00 is the only accepted packet.

## Objective

Clear the Etalon GPU-check warnings named in `C:\github\qualiaDB\target-production-core\logs\cargo-check-qdb-0413-gpu.log` and the wasm I1 unused-`options` site. Hypermedia codec warnings stay on I3's out-of-scope list. `cargo fix` was not applied.

## Warning sites and fixes

| Site | Log | Fix |
|---|---|---|
| `crates/etalon-wgsl-forge/src/numerical/physics/acoustic_fdtd.rs:10` | unused imports `BindingUsage`, `QualiaCompute`, `WgpuComputeContext`, `WgpuPipeline` | Removed the whole `crate::numerical::execute::{...}` import. GPU dispatch helpers already live in `acoustic_fdtd_helpers.rs`, which imports those types itself. |
| `crates/etalon-wgsl-forge/src/numerical/physics/acoustic_fdtd.rs:11` | unused import `Schedule` | Dropped `Schedule`. Kept `use crate::numerical::ForgeError;` because `validate` still returns `ForgeError`. |
| `crates/etalon-wasm/src/worker_steward.rs:109` | deprecated `AtomicU64::fetch_update` | Replaced with `try_update(Ordering::Relaxed, Ordering::Relaxed, \|id\| { (id != 0).then(\|\| id.checked_add(1)).flatten() })`. Exhaustion remains `WorkerError::IdentityExhausted`. |
| `crates/etalon-inference/src/gpu_resources/gemv.rs:35` | wasm I1 unused `options: super::PipelineOptions` | Renamed the parameter to `_options` (call sites still pass it). Native `#[cfg(not(target_arch = "wasm32"))]` path still reads `_options.subgroups`. |

## Paths written

| Path | Role |
|---|---|
| `C:\github\etalon\crates\etalon-wgsl-forge\src\numerical\physics\acoustic_fdtd.rs` | Removed unused GPU/schedule imports |
| `C:\github\etalon\crates\etalon-wasm\src\worker_steward.rs` | `fetch_update` → `try_update` |
| `C:\github\etalon\crates\etalon-inference\src\gpu_resources\gemv.rs` | `_options` prefix; native subgroup use retained |
| `C:\github\synth\docs\production-core\evidence\integration-wave-i4e\agent-report.md` | This report |

Not edited: `Cargo.lock`, naga, Hypermedia codec, `qualia-core-db`, swarm-board, leases, other inference GPU builders.

## Remaining gaps

- Cargo was not run. The GPU log and wasm I1 warning sites are source-cleared; rustc has not re-checked them.
- Hypermedia codec warnings are out of scope (I3).
- GPU log still records `qualia-core-db` `PassTimings::set_adapter_info` dead_code (`gpu_timing.rs:114`). That crate is outside this write lease.
- `etalon-inference` `gpu_resources/ffn.rs` still names `options: super::PipelineOptions` and only reads `options.subgroups` under `#[cfg(not(target_arch = "wasm32"))]`. Same wasm unused-parameter shape as gemv, but the packet named only `gemv.rs`.
- Packet I4E is **not qualified**.
