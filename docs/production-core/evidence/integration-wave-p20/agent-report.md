# P20 agent report — GPU-less activation / rollback / migration

Date: 16 November 2026. Agent: `p20-lifecycle`. Packet: **P20 remaining**.
This is source accounting, not packet acceptance and not P22 qualification.

## Lease

- Write: `qualiaDB/crates/qualia-core-db/src/specialized_libs/computational_geometry/production_lifecycle/**`
- Evidence create-only: this directory
- Read, not written: `production_runtime/**`, parent `mod.rs`, `Cargo.*`, wgpu, `etalon-render`
- No cargo, no commits, no MCP kill, no synth rebuild

## Source

| Path | Role |
|---|---|
| `production_lifecycle/mod.rs` | P20 adapter surface; `DishonestTiming`; not P22 |
| `production_lifecycle/phases.rs` | Five-phase ledger, live-device token, GPU execution record, migration effects |
| `production_lifecycle/faults.rs` | `run_lifecycle` over existing `QualifiedSet` / `ActivationStore` / `PageState` |
| `production_lifecycle/tests.rs` | 16 authored tests; **unrun** |

Parent `computational_geometry/mod.rs` already declares `pub mod production_lifecycle`. This agent did not edit it.

## Behaviour (source, not execution)

Five independent phase records: decode / upload / compaction / draw / activation. No blended score.

CPU/synthetic durations are `CpuMonotonic` or `NotMeasured`. `PhaseRecord::accounted` returns `LifecycleError::DishonestTiming` for `GpuTimestamp` and `CompletionClock`. Timed phases go through that constructor.

Live GPU device token:

- absent → `gpu_execution.status = Unsupported`, note `GPU_ABSENT_NOTE`, timing `NotMeasured`, `timestamp_ns = None`
- present → `Inconclusive`, note `GPU_NO_DISPATCH_NOTE`, timing `NotMeasured`, `timestamp_ns = None`, **never** `Passed`

The adapter never constructs `wgpu::Device`, never dispatches, and never synthesizes a GPU timestamp. Token presence is identity only.

Fault injection (GPU-less state machine):

- `DeviceLostAtActivation` — `RuntimeError::DeviceLost`; baseline and consumed simulation retained
- `IncompleteReplacement` — missing page / incomplete child residency; `MissingFallback`; last page stays `Inflight`
- `DrawCommandOverflow` — caller command buffer zeroed (`DrawIndexedCommand::EMPTY`)
- `CancelledInflightTransfer` — `PageState::Cancelled`; later phases `Cancelled`
- `MixedGenerationActivation` — mesh/shader/transfer generations disagree → `StaleIdentity`
- `IncompleteAsset` — `MissingFallback`; baseline retained
- `InvalidatedEvidence` — `StaleIdentity`; baseline retained

Rollback restores previous mesh/shader/transfer identities. `ROLLBACK_DOES_NOT_UNDO_SIMULATION` stays true. `LifecycleReport.rollback_restores_simulation` is false. Energy and momentum are `Unsupported` with explicit notes, never a silent `0.0`.

## Tests authored (unrun)

Faults: device lost, missing page, draw overflow, cancelled inflight, mixed generation, incomplete asset, invalidated evidence, rollback-does-not-undo-simulation.

GPU token cases: absent (`Unsupported` + `GPU_ABSENT_NOTE`); present (`Inconclusive` + `GPU_NO_DISPATCH_NOTE`, never `Passed`). Present token does not rescue device-lost activation.

Also: dishonest CPU `GpuTimestamp` / `CompletionClock` labels; five separate phases; caller-buffered capacity plan.

## Honesty / capability

| rung | status |
|---|---|
| present | yes (source under leased path) |
| callable | source only; crate not rebuilt this wave |
| executed | **no** (cargo forbidden) |
| qualified | **no** |

This packet is simulated lifecycle. It does **not** claim P22 device-loss qualification, GPU timestamp queries, or GPU execution `Passed`.

## Remaining gaps

- Lifecycle unit tests not executed in this wave.
- No live `wgpu::Device`, no GPU timestamp query, no driver dispatch.
- Energy/momentum deltas are `Unsupported` / not measured. P10 physical records are not written here and stay unsupported.
- Budget-exhaustion around activation is not a dedicated injected fault; it would surface as `BudgetError` / `Inconclusive` through existing `ProductionBudget` checkpoints.
- Actual GPU lifecycle integration remains P22. P00 is still the only accepted packet.
