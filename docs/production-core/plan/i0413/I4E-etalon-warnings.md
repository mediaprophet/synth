# Packet I4E — Clear Etalon GPU-check warnings

You are a source-only implementer on github roots. Isolation `none`.
Cwd: `C:\github\etalon`. Packet: I4E. Do not run cargo.

## Constraints

- Do not commit, push, kill Synth MCP, or rebuild synth `target` / `target-studio`.
- Do not run cargo. Do not edit `Cargo.lock` or naga except the listed files.
- P00 is the only accepted packet. Do not mark qualified.
- After two failed bounded repairs, stop and report.

## Write lease (only these)

- `C:\github\etalon\crates\etalon-wgsl-forge\src\numerical\physics\acoustic_fdtd.rs`
- `C:\github\etalon\crates\etalon-wasm\src\worker_steward.rs`
- `C:\github\etalon\crates\etalon-inference\src\gpu_resources\gemv.rs`

Create-only report:
`C:\github\synth\docs\production-core\evidence\integration-wave-i4e\agent-report.md`

## Required change

From QualiaDB GPU check log `C:\github\qualiaDB\target-production-core\logs\cargo-check-qdb-0413-gpu.log` and the wasm I1 log:

1. `acoustic_fdtd.rs`: unused imports `BindingUsage`, `QualiaCompute`, `WgpuComputeContext`, `WgpuPipeline`, `Schedule`. Remove or use them. Prefer remove if the GPU dispatch helpers live in the helper module.
2. `worker_steward.rs` around line 109: `AtomicU64::fetch_update` is deprecated. Switch to `try_update` with the same `Ordering::Relaxed` pair and the same `(id != 0).then(|| id.checked_add(1)).flatten()` logic. Keep exhaustion as `WorkerError::IdentityExhausted`.
3. `gemv.rs` `build(..., options: super::PipelineOptions)`: `options` is unused. Prefix `_options` or use it. Do not drop the parameter if call sites pass it.

Do not `cargo fix` the workspace. Leave Hypermedia codec warnings to I3's out-of-scope list.

## Return

Packet id, each warning site and the fix, paths written, remaining gaps.
