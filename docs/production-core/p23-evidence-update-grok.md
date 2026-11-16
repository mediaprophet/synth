# P23 evidence update (implementer, not a release)

Date: 16 November 2026. This does not replace `p23-independent-review.md` and
does not accept the release. It records work done after that review.

## Closed relative to the review's smallest remaining work

1. Hypermedia `production_simplify` compiled in
   `hypermedia-production/target-production-core`. Log
   `hypermedia-semantic-03.log`. 7 unit tests passed, including empty/impossible
   negatives and a cube target-8 admitted reduction. Default `mode=decimate`
   remains geometric QEM. Cached `synth.exe` was not rebuilt.

2. QualiaDB new-module lib tests: 87 passed, 0 failed
   (`qualiaDB/target-production-core/qdb-module-tests-03.log`).

3. Geometry example consumes `--corpus-json`. Campaign run-02 sets
   `geometry_exe.consumes_corpus_json: true`. 19 fixtures consumed published
   JSON; they are no longer the builtin two-chart grid under four names.

4. P08 physical records call `physical_record(Option<&PhysicalEvidence>)`.
   No evidence → Unsupported / NotSupplied / note that P10 is present.
   Absence is not 0.0. Geometry receipts still label
   `physical_response: not_measured` when no physical fixture is supplied.

5. Joined tracer executed: `evidence/joined-tracer-01`. Status
   `passed_numeric_and_packaging_pending_visual_review`. Forge AffineF32 on
   NVIDIA GeForce RTX 5070 Laptop GPU, DX12. Three geometry builds and three
   HMC builds. Visual preview rendered, pending independent inspection.
   GPU-identical timing is not claimed. Cached synth SHA is not source
   equivalence to the isolated worktree.

6. GPU lifecycle remains unexercised as a live dispatch.
   `production_lifecycle` is still a GPU-less accounting adapter.

## Campaign run-02 (not qualification)

- 37 fixtures in the denominator
- 21 executed, 15 not_executed, 1 descriptor_only
- 1 corpus_success (`identity-two-subpart-seam` must_pass on corpus bytes)
- 0 qualified
- B2 executed-not-qualified; B0, B1, B3–B5 not_executed
- Holdout unused for thresholds
- Status `executed_partial_not_qualified`

## Integration onto updated github roots (same day, later)

Worktree production-core source was overlaid onto `C:\github` without commits
and without copying `Cargo.lock` or `target-production-core`. Root-only
commits kept: qualiaDB `cb8c731c2`, etalon `a326f82` (branch
`codex/production-integration-20261113`), hypermedia `fbb7bcb` Etalon pin
plus sibling QualiaDB path-dep, synth website checkpoint `54dc28a3`.

Integration-wave packets (source + coordinator cargo into each repo's
`target-production-core`):

- P08: geometry example calls `certify_decoded_output` on decoded `.10d`.
  Ledger tests 11 passed. Example tests 6 passed including enclosing G1
  identity. Physical remains NotSupplied / not_measured when no P10 evidence.
- P07: `QCLN0001` collapse provenance snapshot. Attribute-transfer tests 10
  passed. Campaign qualification of provenance is still not_executed.
- P20: GPU-less activation/rollback. Lifecycle tests 16 passed. Live device
  token does not dispatch; gpu_execution never Passed.
- P19: quality gates (invalid-fast cannot win, tamper, budget, occupancy
  admission). Forge `production_*` tests 32 passed.
- P21: admitted success independently decoded via `decode_10d_mesh`. 9
  `production_simplify` tests passed. Cached `synth.exe` not rebuilt.
- P03: pure-Python independent `.10d` decode of 7 existing evidence files,
  0 sidecar mismatches. Python 50 tests passed. Status
  `executed_not_qualified`.

Root QualiaDB production-module lib tests: 95 passed, 0 failed
(`qdb-module-tests-root-02.log`).

## Continue-wave v0.3 (same day, later)

Geometry example now snapshots QCLN0001 lineage, accepts `--driver admitted|geometric`,
and measures constant-Neumann `algebraic_residual` (`||b-Ax||`, not iterate change).

- Geometry example tests: 8 passed. CLI exe present on github root.
- P09 ddg_production tests: 19 passed.
- Identity G1 enclosing_bound passed. Reduction admitted G1 inconclusive.
  Geometric QEM ablation G1 failed. Physical still `not_measured` / `wire: null`.
- Independent decode of four new dirs: 4 executed, 0 sidecar mismatches.
- Campaign-02: 21/37 executed, 16 enclosing_bound, 16 QCLN0001 present, B2 executed,
  known-bad skipped. Status `executed_partial_not_qualified`.
- B1 campaign-b1-02: `--driver geometric` on five analytic fixtures. B1 executed,
  not qualified. Provenance not_supplied (no CollapseTransfer).
- Joined-tracer PNGs inspected by the implementer (two-chart seam, side-view tilt).
  That is not independent P23 visual acceptance.

## Still not a release

P00 is the only accepted packet. R01–R08 stay uncommissioned. GPU lifecycle
on a live device, public `mode=semantic` through cached `synth.exe`,
independent P23 visual acceptance, operator/physical qualification, collapse
provenance qualification, and P22 campaign qualification remain open.
Do not accept this release.
