# Packet P22 / P03 remaining — agent report

Agent: `p22-campaign`. Date: 16 November 2026.
Workspace: `C:\github\synth` (`discover_worktrees()` parents resolve to `C:\github`).
Status: remaining oracle evaluation and campaign helpers executed; packet **not accepted**; campaign **not qualified**.

This is a Python-harness report. It is not P03/P22 acceptance.

## Objective coverage

1. Known-bad fixtures stay in the denominator. Published **candidates** were evaluated with `oracle_harness.observe_known_bad_gates` / `evaluate_fixture` (`must_fail` / `failing_tests`). `production_core_geometry.exe` was **not** invoked on a welded candidate as if it were an input mesh. A passing exe without failed candidate gates remains a corpus defect. Observed gates are recorded honestly. Missing fields stay `unevaluated`, never a fake fail and never `0.0`.
2. B1 geometric QEM: campaign accepts extra argv `--driver geometric` for mesh corpus JSON **only when** the example argv parser contains `--driver`. Live example source now parses `--driver admitted|geometric`. GitHub-root `production_core_geometry.exe` is **absent**, so B1 stays `not_executed`. Cargo was not run. The flag was not passed to a missing or stale binary.
3. Provenance inspect still prefers `collapse-provenance.json`. Existing evidence directories and this inspect out-dir have no snapshot; status is `not_supplied` with `sha256=null`, not `0.0`.
4. `INCOMPLETE_CORE_CHECKLIST` updated only with honest states. None are `qualified`. `whole_surface_bound` remains `not_measured`: six existing geometry-receipt files were read-only inspected and none report `enclosing_bound` class (they still have `symmetric_surface_deviation.status: not_measured`). Those evidence directories were not overwritten.
5. Ablation B0 is inspect/CPU (`executed`, `qualified=false`, `error`/`cost` null). B1–B5 `not_executed` except B0. B3–B5 were not invoked.
6. Unit tests extended and run from `C:\github\synth\scripts\production_core`.

## Paths written

| Path | Role |
|---|---|
| `C:\github\synth\scripts\production_core\oracle_harness.py` | `observe_known_bad_gates` on published candidates; unobserved gates stay unevaluated |
| `C:\github\synth\scripts\production_core\run_qualification_campaign.py` | Wire oracle gates, `--driver geometric` plan, B1 slot, enclosing-bound inspect |
| `C:\github\synth\scripts\production_core\admitted_geometry_probe.py` | Detect `--driver` / `decimate_qem`; `geometric_qem_driver` capability |
| `C:\github\synth\scripts\production_core\production_core_artifacts.py` | `inspect_whole_surface_bound`; honest checklist |
| `C:\github\synth\scripts\production_core\test_oracle_harness.py` | New known-bad candidate tests |
| `C:\github\synth\scripts\production_core\test_qualification_campaign.py` | Campaign B1 / known-bad / enclosing-bound tests |
| `C:\github\synth\scripts\production_core\test_decode_10d_harness.py` | Provenance first-name `collapse-provenance.json` |
| `C:\github\synth\scripts\production_core\test_production_core_artifacts.py` | Checklist and enclosing-bound honesty |
| `C:\github\synth\docs\production-core\evidence\integration-wave-p22\campaign-inspect-01\` | Create-only inspect campaign receipt |
| `C:\github\synth\docs\production-core\evidence\integration-wave-p22\oracle-known-bad.json` | Four published candidates, observed gates |
| `C:\github\synth\docs\production-core\evidence\integration-wave-p22\agent-report.md` | This report |

Not edited: Rust, Cargo.*, swarm-board, leases, p23, existing evidence directories, catalogs.

## Known-bad oracle (published candidates, no exe)

`python oracle_harness.py --corpus C:\github\synth\docs\production-core\corpus` → `fixture_publication_consistent`, `known_bad_failed_as_required=true`, `qualification_status=not_executed`, `production_implementation_compared=false`.

| Fixture | Observed failed gates | Unevaluated | Exe |
|---|---|---|---|
| `known-bad-cross-domain-weld` | `forbidden_semantic_merges`, `no_cross_domain_support`, `preserve_corner_chart`, `preserve_tangent_handedness`, `coincident_pairs_are_not_welded` | none | not invoked |
| `known-bad-poisoned-affine` | `independent_affine_oracle`, `k1_abs_rel_tolerance` | none | not invoked |
| `known-bad-tetra-inversion` | `accept_when_inversion_forbidden` | none | not invoked |
| `known-bad-infeasible-contact` | `contact_feasibility`, `authoritative_full_model_required` | none | not invoked |

All four remain `in_denominator=true`, `qualified=false`, `execution_status=not_executed`. `corpus_success=true` means the published candidate failed named oracle gates, not production qualification.

## B1 geometric QEM

- Example source (`C:\github\qualiaDB\crates\qualia-core-db\examples\production_core_geometry.rs`) now parses `--driver admitted|geometric` and has `Driver::GeometricQem` / `decimate_qem`.
- GitHub-root executable `C:\github\qualiaDB\target-production-core\debug\examples\production_core_geometry.exe` is **absent**. Coordinator cargo into `target-production-core` has not landed on this root.
- Campaign CLI accepts `--driver geometric` but does not append it unless the example parses `--driver` **and** the exe is present. This inspect run requested no driver and invoked nothing.
- B1: `execution_status=not_executed`, `planned_argv=["--driver","geometric"]`, `error=null`, `cost=null`, `qualified=false`.

## Provenance and whole-surface bound

- `inspect_collapse_provenance` search order starts with `collapse-provenance.json`. Inspect out-dir and four existing evidence dirs: `not_supplied`, `sha256=null`, not `0.0`.
- Six existing `geometry-receipt.json` files read-only: no `enclosing_bound` class. Checklist `whole_surface_bound=not_measured`. Example unit tests now *author* enclosing G1, but those receipts do not exist on disk here. Execution of that example was not performed (no cargo, no exe).

## Ablations

| Slot | Status | Notes |
|---|---|---|
| B0 | executed (CPU inspect) | five identity fixtures `identity_preserved=true`; production no-reduction path not invoked; `qualified=false` |
| B1 | not_executed | planned `--driver geometric`; exe absent |
| B2 | not_executed | inspect-only; admitted exe not invoked |
| B3 | not_executed | remesh/anisotropic not invoked |
| B4 | not_executed | Forge not invoked; no GPU timing |
| B5 | not_executed | physical/operator not invoked |

## Campaign inspect

`--inspect-only --out .../integration-wave-p22/campaign-inspect-01`

- status: `inspected_not_executed` (not qualified)
- 37 fixtures, 37 in denominator (4 must_fail, 4 holdout, 1 descriptor_only)
- holdout not used for thresholds
- maximum-size not materialized
- invented passing geometry receipt: false
- GPU identical timing claimed: false

## Tests actually run

Working directory: `C:\github\synth\scripts\production_core`.

```
python -m unittest
```

Result: **62 tests, 0 failed, 0 skipped, exit 0**.

Re-verified after continuation: `python -m unittest -v` from the same directory, **62 tests, 0 failed, 0 skipped, exit 0** (1.146s). GitHub-root `production_core_geometry.exe` remains absent; example source still parses `--driver admitted|geometric`. Cargo was not run. Geometry exe was not invoked.

Also: `python oracle_harness.py --corpus C:\github\synth\docs\production-core\corpus` (publication consistent, not executed).

Not run: cargo, geometry exe, Forge GPU probe, synth produce, maximum-size materializer, `--driver geometric` invoke.

## Incomplete-core checklist (honest)

| Item | Status |
|---|---|
| geometry_example_rebuilt_after_lowerhex_repair | done |
| admitted_geometry_executable | present (source; github-root exe absent this inspect) |
| joined_tracer_execution | executed_not_qualified |
| p01_p03_accepted | open |
| public_hypermedia_simplify_semantic | compiled_unit_tested |
| whole_surface_bound | not_measured |
| collapse_provenance_qualification | inspected_not_supplied |
| operator_or_physical_qualification | not_executed |
| final_10d_sealing_independently_decoded | executed_not_qualified |
| gpu_timing_byte_identical_across_repeats | not_claimed |
| cached_synth_exe_source_equivalence | not_established |
| p02_p05_p06_p07_compiled_together | compiled |

None of these is `qualified`.

## Remaining gaps

- P03/P22 are not accepted. Oracle evaluation of published candidates is not production qualification.
- GitHub-root geometry exe is missing; B1 `--driver geometric` cannot be invoked until coordinator cargo lands a binary that parses `--driver`.
- Collapse provenance snapshot still `not_supplied` next to existing evidence `.10d` files. Example source now authors QCLN0001, but that output was not produced here.
- Whole-surface enclosing bound remains `not_measured` on existing receipts. Do not treat example-test authorship of `enclosing_bound` as a measured receipt.
- B2 semantic exe path, B3 remesh, B4 Forge, B5 physical/operator stay `not_executed`.
- `p01_p03_accepted` remains `open`. P00 is still the only accepted packet.

Do not treat this inspect receipt as a passing qualification campaign.
