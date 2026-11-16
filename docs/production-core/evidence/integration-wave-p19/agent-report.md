# P19 agent report — immutable quality gates and bounded tuning

Date: 16 November 2026. Agent: p19-quality. Packet: **P19 remaining**.

Workspace: `C:\github\etalon` (root HEAD `a326f82` AO/material; not touched).
Lease: `etalon/crates/etalon-wgsl-forge/src/production_quality/**` plus this
create-only evidence file.

No cargo. No commits. No MCP kill. No GPU dispatch. No synth rebuild.
P00 remains the only accepted packet. This packet is **not qualified**.

## Source written

All under `C:\github\etalon\crates\etalon-wgsl-forge\src\production_quality\`:

| Path | Role |
|---|---|
| `mod.rs` | Re-exports gates/search/costs; occupancy types from lowering; `Digest` for CRC refusal |
| `schedule_gate.rs` | Existing admit/label/record; `BudgetSkipped`; search counts; occupancy-as-timing refusal; `gate_tuning` rejects invalid winners |
| `gates.rs` | Frozen production thresholds, tamper rejection, oracle admission, CRC ≠ signature |
| `search.rs` | Equal-budget `gated_search`, baseline preservation, ranking key, Pareto, full attempt record |
| `costs.rs` | Labeled build/compile/upload/steady costs; missing is unsupported, never 0; break-even |
| `tests.rs` | Previous seven tests kept; eight new gate tests authored |

Not edited: `etalon-wgsl-forge/src/lib.rs`, `etalon-render/**`, `production_ir`,
`production_lowering`, `Cargo.*`, `autotune.rs`, `schedule.rs`, `cache.rs`,
public `KernelSpec` / `Schedule` / `tune_with`.

## Admission logic (source, not execution)

- **Invalid fast cannot win.** `tune_with` still rejects oracle failures. `gate_tuning` additionally refuses a `TuningResult` whose winner fails frozen oracle gates or appears in `failures`. `gated_search` records the fast failure as `Rejected` and will not select it.
- **Immutable thresholds.** `FrozenQualityGates::production()` is the only freeze. Loosened mismatch/error/shared-memory/workgroup ceilings, `attempt_retune`, and JSON digest/threshold mismatch are `Validation("immutable quality thresholds reject tampering")`. Constants match `OracleTolerance::default()` and portable shared/workgroup ceilings.
- **Budget exhaustion preserves admitted baseline.** `SearchBudget` evaluation/sample caps skip remaining candidates as `BudgetSkipped`. Winner is the admitted baseline (`WinnerSource::PreservedBaseline`), even if a faster admitted candidate was already seen. No baseline → refuse, not a partial winner.
- **Tie rules.** `ranking_key` is median_ns, then p95_ns, then `Schedule::sort_key()` (same order as `autotune::sort_results`). Two identical runs must agree.
- **Occupancy is admission.** `DeclaredOccupancy` / `OccupancyReport` live on `ScheduleGateReport`. `labeled_timing_from_occupancy` and `occupancy_report_as_gpu_timestamp` always error. `LabeledTiming` has no occupancy fields and cannot be `GpuTimestamp` unless the method label is actually `GpuTimestamp`.
- **CRC is not a signature.** `crc_is_not_a_signature` / `parse_content_digest` reject `crc*` as checksum, not identity or authentication. Blake3 remains the content digest.
- **Synthetic is not GPU timing.** Existing `admit_timing` / `require_timing_label` / `as_gpu_timing` unchanged in policy; break-even refuses mixed methods.
- **Publish all attempts.** `gated_search` records every enumerated schedule as Admitted, Rejected, or BudgetSkipped, with reasons on failures/skips. Record-capacity overflow stays fail-closed. Learned `.p64` search is excluded (`learned_search: false`).
- **Phase costs.** `T_total(n) = T_build + T_compile + T_upload + n T_steady` using labeled medians. Unsupported phases are inconclusive, never zero. Integrated decode/compaction/draw remains P22 (`INTEGRATED_PHASE_COST_NOTE`).

## Tests authored (unrun)

Existing: occupancy admit/undeclared/declared, workgroup/shared/subgroup, labeled timing, fail-closed overflow, autotune record.

New:

- `invalid_fast_candidate_cannot_win`
- `immutable_thresholds_reject_tampering`
- `budget_exhaustion_preserves_admitted_baseline`
- `tie_rules_are_reproducible`
- `occupancy_is_admission_never_labeled_timing`
- `crc_is_not_a_signature_and_synthetic_is_not_gpu`
- `gated_search_publishes_all_attempts_and_failures`
- `break_even_does_not_treat_missing_cost_as_zero`

Cargo was not run (lease). These tests are source, not executed evidence.

## Remaining gaps

- **Not compiled / not executed.** Lease forbids cargo. Previous Forge lib 24-test pass does not include these new tests.
- **No GPU campaign.** No equal-budget kernel-schedule measurement on a real adapter. No timestamp-query samples. Test timings are injected `TimingSource` labels, including `GpuTimestamp` fixtures that are **not** device measurements.
- **Not qualification.** No P22 corpus, no native/browser matrix, no five-warmup / 30-sample design, no visual review.
- **`tune_with` wrapping is lossy.** `TuningResult` omits accepted-then-truncated non-finalists. `record_tuning_result` can only publish what `tune_with` retained. Full attempt publication is `gated_search`. Public `tune_with` was not changed.
- **Integrated decode/upload/compaction/draw costs** belong to P22 after P18. Break-even here is kernel-schedule phase labels only.
- **Optional learned search (R06)** uncommissioned and excluded from core.
- **Tamper test is unit-level** (freeze/retune/JSON). No frozen-threshold campaign against a real adapter (p23 review already noted this).
- **P08/P14/P15 dependencies** are source-present, not accepted packets.

## Capability ladder

| Claim | Status |
|---|---|
| present | yes (source under lease) |
| callable | public module already in `lib.rs`; new items are module-public |
| executed | **no** (cargo unrun this wave) |
| qualified | **no** |
