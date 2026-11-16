# P08 agent report (p08-enclosing)

Date: 16 November 2026. Packet: P08 remaining (decoded-output geometry bounds and error ledger). Source only; **cargo was not run**. This is not qualification.

## Touched paths

- `C:\github\qualiaDB\crates\qualia-core-db\src\specialized_libs\computational_geometry\decoded_error_ledger\tests.rs`
- `C:\github\qualiaDB\crates\qualia-core-db\examples\production_core_geometry.rs`
- `C:\github\synth\docs\production-core\evidence\integration-wave-p08\agent-report.md` (this file; create-only)

Ledger implementation (`mod.rs`, `ledger.rs`, `surface_bracket.rs`) was already present. No edits there. Coordinator-owned files (`computational_geometry/mod.rs`, Cargo.toml/lock) were not touched.

## What is now measured vs still inconclusive

**Measured (in source; unexecuted):**

- Geometry tracer calls `certify_decoded_output` on the **authored frozen triangle domain** vs **decoded final `.10d` buffers** (`decode_10d_mesh` positions/triangles promoted f32→f64). It does **not** certify the pre-quantization simplified mesh.
- Completed enclosing brackets record `L`/`U`, diagonal `D`, slack, method `symmetric_triangle_covering_lipschitz_subdivision`, and G1 decision (`U <= 1e-3 D` and `U-L <= 1e-5 D`).
- Quantization channel uses `Binary32` for decoded `.10d` positions (separate from the surface bracket).
- Forbidden semantic merges are a measured count (builtin checks contribute `0`), never a silent geometric `0.0` error.
- Protected-point drift is measured when locked vertices survive compaction.

**Honest absence / inconclusive:**

- Width unmet or budget exhausted → `LedgerStatus::Inconclusive` and enclosing `Metric::Absent { IncompleteEnclosure }` (or a completed-but-wide bracket that cannot pass G1). Sampled estimates stay `SampledEstimate` and cannot pass G1 (`BoundRoute::SampledVertices` forces overall `Inconclusive`).
- Physical channel: **no P10 `PhysicalEvidence` is passed in**. Receipt keeps `physical_response.status: not_measured` with `wire: null` and reason `p10_physical_evidence_not_supplied`. Ledger `physical_record(None)` remains `Unsupported` / `NotSupplied`, never `0.0`.
- Operator residual remains `not_measured` (P09, out of lease).
- IEEE-754 f64 slack is still a first-order ulp estimate (`outward_rounding: false`).
- Stale limitation `no independent whole-surface distance certificate` is removed because `certify_decoded_output` is actually called. If enclosure does not complete, the receipt adds `whole-surface enclosure incomplete; sampled estimates cannot pass G1`.

## Tests added (unrun)

Ledger unit tests (`decoded_error_ledger/tests.rs`):

- `identity_pair_enclosing_passes_g1` — unit square identity; expects enclosing bracket + G1 Passed; physical Null.
- `identity_pair_decoded_f32_buffers_pass_g1` — authored vs f32-cast decoded buffers + `Binary32` quantization.
- `budget_exhausted_enclosing_is_inconclusive` — work limit `0` → `Inconclusive` / `IncompleteEnclosure` / wire Null.
- Existing (kept): `translated_large_coordinate_fixture_passes_g1`, `thin_feature_fixture_preserves_enclosing_bracket`, `known_distance_lower_bound_exceeds_threshold_fails`, `unmet_width_is_inconclusive` (Absent now asserts `IncompleteEnclosure`), `sampled_estimate_is_labelled_and_cannot_pass_enclosing_profile`, physical NotSupplied / supplied P10 mapping.

Example tests (`production_core_geometry.rs`):

- `identity_decoded_output_reports_enclosing_g1` — identity run must report enclosing G1 from decoded `.10d` buffers; physical stays `not_measured`/null; stale limitation gone; reduction must not stay `not_measured` and must not be labelled a sampled G1 pass.
- Corpus loader test now also asserts surface deviation is no longer `not_measured` and physical stays `not_measured`.

## Remaining gaps

- Tests and the geometry example were **not compiled or executed** (lease: no cargo; coordinator compiles into `target-production-core`).
- Execution is not qualification. G1 on decoded campaign fixtures is still unrun.
- Physical remains unsupported in receipts until a caller actually supplies P10 evidence.
- Reduction (32→24) may complete a wide bracket and stay **inconclusive**; that is not a pass.
- Slack is not outward rounding.
