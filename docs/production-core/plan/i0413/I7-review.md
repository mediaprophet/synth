# Packet I7 — Independent review of I1–I6

You are a read-only reviewer. Different agent from the implementers.
Isolation `none`. Cwd: `C:\github\synth`. Packet: I7. Do not edit product code.

## Constraints

- Write only
  `C:\github\synth\docs\production-core\evidence\integration-wave-i7\review.md`
  (create-only).
- Do not commit. Do not accept P22/P23. Do not mark production-core qualified.
- P00 remains the only accepted packet.
- Do not run cargo that rebuilds synth `target` / `target-studio`.
- Do not kill MCP.

## Read

- `C:\github\synth\docs\production-core\integration-wave-0413-swarm-plan.md`
- `C:\github\synth\docs\production-core\plan\integration-wave-0413-work-packets.md`
- I1 log `C:\github\qualiaDB\target-production-core-wasm\logs\cargo-check-qdb-portal-wasm-i1.log`
- Agent reports under `evidence/integration-wave-i{2a,2b,3,4e,4q,5,6}/`
- QualiaDB `constructors.rs` `try_new_async` / `adapter_responds`
- Website `graphics.rs` `selected_backend`
- Website `native/world-host/Cargo.toml` path-dep

## Confirm

1. I1: getrandom 0.3.3 compiled; RUSTFLAGS was empty; cfg is durable.
2. I2A: WebGPU first, GL retry on device/surface failure, `?webgl2=1` still force-GL.
3. I2B: `selected_backend` comes from the live adapter, not `navigator.gpu`.
4. I0: git pin is `143a1d7538ceae867ceab1dc7e43126bc688752f` or explicitly deferred.
5. I5: website path-deps `C:/github/qualiaDB`, not `_worktrees/qualiadb-webgl2`.
6. I6: desktop WebGPU and GL receipts exist or are honest `not_executed`.
7. Bugs only. Suggestions go in a separate nits list. 0 bugs to accept this wave as *landed*, still not qualified.

## Return

Pass/fail per packet, bugs, nits, whether intended WebGPU-then-GL benefit is evidenced.
