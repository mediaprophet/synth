# P23 independent review — QualiaDB / Etalon production-core

Date: 16 November 2026. Reviewer: independent P23 agent (Grok). Isolation: none.
This file is a review artifact only. Implementation crates were not edited. Cargo
was not run. Nothing was committed. No processes were killed.

**Release verdict: do not accept.** P00 is the only packet that can be treated
as accepted, and that acceptance is a scope freeze, not a production-core
release. Packets P01–P23 are unaccepted. Optional research packets R01–R08
remain uncommissioned. Feature-complete source is not independently accepted
evidence.

`packet-state.json` and `ROUND-HANDOFF.md` are stale relative to later compile
and tracer work recorded in `swarm-board-grok.md` and in
`docs/production-core/evidence/`. This review prefers source plus receipts over
those earlier boards.

## Method

Four states are recorded independently. Presence of source is not compilation.
Compilation is not execution. Execution is not qualification.

| State | Meaning used here |
|---|---|
| implemented | Production source for the packet exists on disk and is wired into a parent module |
| compiled | Non-test production code was compiled by a recorded cargo command |
| executed | A recorded process actually ran that path and left a receipt |
| qualified | Independent acceptance evidence exists against the packet gates |

Qualification is almost never true. A geometry executable that prints
`"status": "passed"` on a builtin seam fixture is a tracer run, not P22/P23
qualification. Forge unit tests are not GPU backend qualification. Python
campaign status `executed_partial_not_qualified` is the honest label.

Evidence roots:

- `C:/Users/Hp/.codex/worktrees/qualia-etalon-production-core/synth/docs/production-core/`
- Packet register: `C:/Users/Hp/.codex/visualizations/2026/11/16/01a1e2e0-ddfa-7c43-9fa6-630bfdb497ac/swarm-packet-register-v0.1.md`
- QualiaDB: `.../qualiaDB/crates/qualia-core-db/src/specialized_libs/computational_geometry/`
- Etalon: `.../etalon/crates/etalon-wgsl-forge/src/`
- Hypermedia: `.../hypermedia-production/crates/asset-production-vibe/src/production_simplify.rs`

## Packet matrix P00–P23

### P00 — Freeze the release contract and workspace baseline

| implemented | compiled | executed | qualified |
|---|---|---|---|
| yes | n/a (planning) | yes | **scope freeze only** |

Evidence: `docs/production-core/scope-v0.1.md`, `acceptance-matrix.json`,
`baseline-review.md`, `evidence/workspace-baseline.json`,
`evidence/forge-native-baseline/probe.json`,
`evidence/browser-baseline-20261116-065406-847337.json`. Native DX12 and
browser WebGPU routes were recorded. The 38-row matrix still has
`execution_status: not_executed` on requirement rows. Accepted as a contract
freeze, not as implemented production-core.

### P01 — Qualify existing mathematical and production capabilities

| implemented | compiled | executed | qualified |
|---|---|---|---|
| partial | partial | partial | **no** |

Executed: fixed AffineF32 native probe
(`evidence/forge-native-baseline/probe.json`, 4099 values, zero mismatches,
NVIDIA RTX 5070 Laptop GPU / DX12) and browser probe
(`evidence/browser-baseline-20261116-065406-847337.json`, Chrome / Intel
gen-12lp). Negative Forge checks:
`evidence/forge-negative-checks.json`. Later geometry example compile/run is
P04/P06 evidence, not a completed P01 math/codec/DDG residual campaign.
Semantic QEM, transfer codec, DDG residual, and adapter-matrix probes required
by the register were not independently qualified.

### P02 — Versioned contracts, identities and result semantics

| implemented | compiled | executed | qualified |
|---|---|---|---|
| yes | yes (lib, via example) | no dedicated run | **no** |

Source: `computational_geometry/production_contracts/` (`mod.rs`, `identity.rs`,
`records.rs`, `transfer.rs`, `collapse.rs`, `validation.rs`, `tests.rs`).
Exported from `computational_geometry/mod.rs`. Compiled as QualiaDB library
code during
`qualiaDB/target-production-core/geometry-build-grok-03.log` (`Finished dev`,
8m 00s). Ten unit tests exist and were **not** run: there is no
`cargo test -p qualia-core-db --lib` log. Schema/identity gates remain
unexecuted.

### P03 — Independent corpus and oracle harness

| implemented | compiled | executed | qualified |
|---|---|---|---|
| yes (fixtures + harness) | n/a (Python) | fixtures published | **no** |

Evidence: `docs/production-core/corpus/manifest.json` (37 fixtures, 11
families, blake3+sha256). Scripts:
`scripts/production_core/qualification_corpus.py`, `oracle_harness.py`.
Maximum-size remains `descriptor_only`. Production comparison is not executed.
The geometry executable does not load these JSON files (see P22). Known-bad
and identity oracles were not applied to production outputs.

### P04 — Early end-to-end tracer using existing paths

| implemented | compiled | executed | qualified |
|---|---|---|---|
| yes | yes (geometry example) | partial | **no** |

Compiled: `qualiaDB/target-production-core/debug/examples/production_core_geometry.exe`
(`geometry-build-grok-03.log`). Executed: admitted seam tracer
`evidence/admitted-geometry-run-01/geometry-receipt.json` (`status: passed`,
termination `ReachedTarget`, 32→24 faces, builtin
`normalized-tilted-two-chart-grid`). Empty input: campaign
`geometry-empty-mesh` exit 2, `invalid_input`, no output directory. Zero
target: `evidence/admitted-geometry-zero-target-01/` and campaign
`geometry-impossible-ratio` (`unsatisfied_constraint`, committed prefix).
Example tests: `geometry-example-tests-01.log`, 4 passed.

Not executed: joined geometry → Forge GPU → Synth HMC tracer. Receipt
limitations include `no GPU stage in this executable`,
`no independent whole-surface distance certificate`,
`operator_residual.status: not_measured`,
`physical_response.status: not_measured`,
`symmetric_surface_deviation.status: not_measured`. This is an integration
tracer, not acceptance.

### P05 — Budget, workspace and cancellation enforcement

| implemented | compiled | executed | qualified |
|---|---|---|---|
| yes | yes (lib, via example) | indirect only | **no** |

Source: `geometry_workspace/production_budget.rs` and
`production_budget_tests.rs` (3 tests, unrun). Geometry example charges a
budget (`resources.work_used` in the tracer receipt). That does not prove
downstream allocation accounting, zero/max/exhausted capacities, or
cancellation disposal across P11–P20.

### P06 — Hard semantic and topology collapse admission

| implemented | compiled | executed | qualified |
|---|---|---|---|
| yes | yes | tracer only | **no** |

Source: `decimate_3_semantic/{admission.rs,bounded_driver.rs,topology.rs,normalized_cost.rs,admission_tests.rs}`.
Public API `decimate_semantic_admitted_3` is used by the geometry example.
Tracer run reached target with `forbidden_label_vertices: 0` and
`mixed_domain_triangles: 0` on the builtin two-chart grid. Three admission
unit tests and parent infinity/domain regressions are authored and **unrun**.
Adversarial corpus, scale/permutation metamorphic tests, and combined P08
transfer+bound evidence are absent.

### P07 — Typed attribute and correspondence transfers

| implemented | compiled | executed | qualified |
|---|---|---|---|
| yes | yes | tracer only | **no** |

Source: `attribute_transfer/` (`mod.rs`, `sparse.rs`, `frames.rs`, `corner.rs`,
`skin.rs`, `femc.rs`, `resource.rs`, `collapse.rs`, `tests.rs`). Earlier
handover duplicate-`TransferError` / missing-`tests.rs` findings are stale:
`tests.rs` is present and `mod.rs` defines a single `TransferError`. Geometry
example uses `CollapseTransfer`. Six unit tests authored, **unrun**. Virtual
work, conservation, checker/tangent/skin fixtures, and encode/decode
qualification were not executed. Smooth/MikkTSpace, nonrigid frames, signed
maps, and >4 skin influences remain explicitly unsupported.

### P08 — Decoded-output geometry bounds and error ledger

| implemented | compiled | executed | qualified |
|---|---|---|---|
| yes | yes (lib) | **no** | **no** |

Source: `decoded_error_ledger/{mod.rs,ledger.rs,surface_bracket.rs,tests.rs}`.
Compiled as QualiaDB lib. Geometry receipt still records
`symmetric_surface_deviation.status: not_measured`. Six unit tests authored,
**unrun**.

**Physical channel remains an unsupported placeholder**, even though P10 source
now exists. `decoded_error_ledger/ledger.rs` hard-codes:

```text
"P10 physical authority is not present; response is unsupported, not a zero error"
PhysicalRecord { status: LedgerStatus::Unsupported, ... }
```

G1 enclosing-bound acceptance (`U <= 1e-3 D`, `U-L <= 1e-5 D`) was not
executed on decoded final assets. Sampled estimates must stay labelled
estimates; they were not promoted, which is honest, but the packet is not
done.

### P09 — Surface operator conventions and true residuals

| implemented | compiled | executed | qualified |
|---|---|---|---|
| yes | yes (lib) | **no** | **no** |

Source: `ddg_production/{mod.rs,algebraic_residual.rs,conventions.rs,tests.rs}`.
Correctly keeps `DdgReport.residual` as iterate change and adds a separate
`algebraic_residual` (`||b − A x||`). Fourteen unit tests authored, **unrun**.
Geometry receipt `operator_residual.status: not_measured`. Analytic/nullspace
fixtures and heat-route stability were not executed.

### P10 — Authoritative physical-state interface and evidence

| implemented | compiled | executed | qualified |
|---|---|---|---|
| yes | yes (lib) | **no** | **no** |

Source: `physical_authority/{mod.rs,state.rs,transfer.rs,tests.rs}` over
existing `pbd_softbody`. Independent-tetrahedron XPBD is labelled as such, not
as a race-free global solver. Twelve unit tests authored, **unrun**. Geometry
receipt `physical_response.status: not_measured`. P08 does not call this
module. Contact remains explicit `unsupported`.

### P11 — Coherent cluster hierarchy and cut validator

| implemented | compiled | executed | qualified |
|---|---|---|---|
| yes (adapter) | yes (lib) | **no** | **no** |

Source: `production_runtime/hierarchy.rs` plus tests in
`production_runtime/tests.rs`. Trees/forests only; existing 64-vertex /
124-triangle CDAG node layout remains authoritative. Coverage is an external
typed dependency, not a new CDAG field. Unit tests authored, **unrun**. No
codec roundtrip, no random legal/illegal cuts executed.

### P12 — Semantic numerical IR and guarded rewrites

| implemented | compiled | executed | qualified |
|---|---|---|---|
| yes | yes | unit tests only | **no** |

Source: `etalon-wgsl-forge/src/production_ir/`. Wired in `lib.rs`. Forge lib
tests: `etalon/target-production-core/forge-tests-grok-01.log`, **24 passed /
22 filtered out**. IR tests cover invalid dimensions, aliasing, bounded
rewrites, and fma assumption traces. This is not production-kernel
qualification and not a GPU rewrite campaign.

### P13 — Bounded portable WGSL lowering

| implemented | compiled | executed | qualified |
|---|---|---|---|
| yes | yes | unit tests only | **no** |

Source: `production_lowering/{mod.rs,admission.rs,wgsl.rs,tests.rs}`. Unit
tests emit geometry WGSL and run Naga parse/validate on the host. No
generated → pipeline → oracle GPU execution of this new lowering path. The
historical AffineF32 probe is a different kernel. GPU unavailability was
honestly recorded as unsupported Metal in `forge-negative-checks.json`; that
is not this packet's execution receipt.

### P14 — Precision routes and backend qualification

| implemented | compiled | executed | qualified |
|---|---|---|---|
| partial | yes | unit + old affine baseline | **no** |

`df64_is_not_native_wgsl_f64` passed as a unit test. Native/browser f32
AffineF32 baselines exist (P01 evidence) and are not a production_lowering
precision matrix. Paired-f32 oracle, ill-conditioned refusal envelopes, and
frozen native+browser route qualification for the new IR were not executed.

### P15 — Evidence stages and dependency-aware cache

| implemented | compiled | executed | qualified |
|---|---|---|---|
| yes | yes | unit tests only | **no** |

Source: `production_ir/evidence.rs`. Unit tests cover canonical keys,
transitive invalidation, replay of missing hashes/unsupported versions, and
refusal to treat CRC as a signature or synthetic time as GPU time. No
end-to-end stage receipts tied to actual geometry/GPU artifacts.

### P16 — Conservative view and deformation selection

| implemented | compiled | executed | qualified |
|---|---|---|---|
| yes (CPU adapter) | yes (lib) | **no** | **no** |

Source: `production_runtime/selection.rs`. Tests authored, **unrun**. No
CPU/GPU selection comparison, no camera/near-plane/occlusion fixtures, no
legal-cut binding executed.

### P17 — Residency, paging and complete fallback cuts

| implemented | compiled | executed | qualified |
|---|---|---|---|
| yes (CPU adapter) | yes (lib) | **no** | **no** |

Source: `production_runtime/residency.rs`. Tests authored, **unrun**. Device
loss / missing-page / exhaustion lifecycle is GPU-less state-machine source
only (see P20/P22).

### P18 — Compaction and complete bounded draw generation

| implemented | compiled | executed | qualified |
|---|---|---|---|
| yes (CPU adapter) | yes (lib) | **no** | **no** |

Source: `production_runtime/draw.rs`. Tests authored, **unrun**. No actual
indexed draw, no GPU indirect arguments, no rendered-coverage comparison.

### P19 — Immutable quality gates and bounded tuning

| implemented | compiled | executed | qualified |
|---|---|---|---|
| yes | yes | unit tests only | **no** |

Source: `production_quality/{mod.rs,schedule_gate.rs,tests.rs}`. Included in
the 24 passing Forge lib tests (occupancy, labeled timing, fail-closed search).
No equal-budget kernel-schedule campaign, no GPU timing qualification, no
tamper test against frozen thresholds on a real adapter.

### P20 — Atomic activation, rollback and state migration

| implemented | compiled | executed | qualified |
|---|---|---|---|
| yes (CPU adapter) | yes (lib) | **no** | **no** |

Source: `production_runtime/activation.rs` and
`production_lifecycle/{mod.rs,faults.rs,phases.rs,tests.rs}`. Ten lifecycle
tests authored, **unrun**. The module never constructs `wgpu::Device` and does
not dispatch GPU work. A live device token without a driver execution is
`inconclusive`, never `passed` (`production_lifecycle/phases.rs`
`GPU_NO_DISPATCH_NOTE`). GPU lifecycle was not actually exercised.

### P21 — Existing morphism and deterministic production integration

| implemented | compiled | executed | qualified |
|---|---|---|---|
| source wired | **no** (Hypermedia cargo unrun) | historical smoke only | **no** |

Source: `hypermedia-production/crates/asset-production-vibe/src/production_simplify.rs`
and `manifold_ops.rs` (`mode=semantic|admitted` →
`decimate_semantic_admitted_3` + P07 `CollapseTransfer`). Ontology/catalog
strings updated; inventory still claimed 150. Default `mode=decimate` remains
geometric `decimate_qem`. No Hypermedia `target-production-core` tree exists.
Qualification campaign checklist:
`public_hypermedia_simplify_semantic: source_wired_uncompiled`.

Historical packaging smoke (`evidence/attributed-handoff-smoke/receipt.json`)
used cached `C:/github/synth/target-prod/debug/synth.exe` on a six-vertex
fixture. That is not semantic simplify, not three independent GPU runs, and
not source-equivalent to the isolated worktree.

### P22 — Independent ablations and qualification campaign

| implemented | compiled | executed | qualified |
|---|---|---|---|
| runner + inspect/run receipts | n/a (Python) | **partial** | **no** |

Receipts:

- `evidence/qualification-campaign-inspect-01/qualification-receipt.json`
  (`inspected_not_executed`)
- `evidence/qualification-campaign-run-01/qualification-receipt.json`
  (`executed_partial_not_qualified`)

Campaign facts: 25 selected fixtures in the denominator; 6 invoked; 19 not
executed; 0 qualified; 0 corpus_success. Geometry exe
`consumes_corpus_json: false`; builtin fixture
`normalized-tilted-two-chart-grid` reused for analytic-plane, mixed-provenance,
uv-seam, and identity-two-subpart-seam. Known-bad fixtures were skipped, not
failed. Ablations B0, B1, B3–B5 `not_executed`; B2 executed-not-qualified.
Visual capture is `recipe_only_not_executed`. No 5 warmup / 30 GPU samples
for this campaign. No decode/upload/compaction/draw cost confirmation.

Python helper tests exist in `scripts/production_core/test_qualification_campaign.py`
and `test_production_core_artifacts.py`. Swarm-board claim of 12 Python tracer
tests is source, not this reviewer's re-run.

### P23 — Release gate and reproducible handoff

| implemented | compiled | executed | qualified |
|---|---|---|---|
| this review only | n/a | n/a | **no** |

Requirement/test/evidence/reviewer matrix is not complete. No frozen release
candidate hashes, no independent clean reproduction of a representative
end-to-end result, no rollback runbook for an accepted candidate. This
document is a development-handoff review, not a release.

## Optional research packets

| Packet | Status |
|---|---|
| R01 Intrinsic coarsening | uncommissioned |
| R02 Spectral preservation | uncommissioned |
| R03 Physical reduction / hyperreduction | uncommissioned |
| R04 Appearance / deformation-aware optimization | uncommissioned |
| R05 Quad-preserving authoring | uncommissioned |
| R06 Learned proposal search | uncommissioned |
| R07 Native backend specialization | uncommissioned |
| R08 Generated adaptive predicates | uncommissioned |

Do not silently fold any of these into the 24 core packets.

## Remaining compile / test gaps (must not be papered over)

1. **Hypermedia semantic mode uncompiled.** `production_simplify.rs` and
   `manifold_ops.rs` are source-wired to sibling QualiaDB
   (`asset-production-vibe/Cargo.toml` path dep). No cargo log, no target
   directory, no morphism execution. Default simplify is still geometric QEM.

2. **QualiaDB unit tests of new modules unrun.** Authored tests exist in
   `production_contracts` (10), `production_budget_tests` (3),
   `admission_tests` (3), `attribute_transfer` (6), `decoded_error_ledger` (6),
   `ddg_production` (14), `physical_authority` (12), `production_runtime`
   (16), `production_lifecycle` (10). The only recorded QualiaDB test command
   is `cargo test --example production_core_geometry` (4 passed). Lib tests
   including `#[cfg(test)]` modules were not compiled or executed.

3. **GPU lifecycle not actually exercised.** `production_lifecycle` is a
   GPU-less ledger. It never constructs `wgpu::Device` and does not dispatch.
   P22 campaign did not run Forge, residency, draw, or device-loss on a live
   adapter. Native AffineF32 probe is a different, earlier kernel.

4. **P08 physical channel unsupported.** Ledger still returns
   `LedgerStatus::Unsupported` / `AbsenceReason::UnsupportedCapability` for
   physical response, with a note that P10 is not present. P10 source exists
   and is not wired. Geometry receipts correctly leave
   `physical_response` / `symmetric_surface_deviation` / `operator_residual`
   as `not_measured`.

5. **Corpus executable does not consume fixture JSON.**
   `examples/production_core_geometry.rs` CLI is
   `--out [--target-faces N] [--empty] [--output-byte-budget N]`. Campaign
   receipt: `"consumes_corpus_json": false`. Invoking
   `analytic-plane-scale-1` still runs the builtin two-chart grid. That is
   not corpus qualification.

Additional honesty items:

- Stale frozen-facts in `swarm-board-grok.md` (P07 cannot compile; P02/P05/P06
  uncompiled as a set) were superseded by later cargo passes. Do not treat
  either the stale handover or the later compile as qualification.
- `DdgReport.residual` is still iterate change; P09 adds a separate field but
  did not execute it.
- Existing CDAG, FEMC, Forge IR, and BVH/distance remain authoritative; new
  adapters must not be described as replacements.
- Cached Synth producer SHA256
  `62095c5c1c126bfade382c1251a6293be9c87b41abb88902dedc834ff742618e` is not
  source-equivalent to the isolated worktree HEAD.

## Smallest remaining work for a credible engineering handoff

This is not a release checklist. It is the smallest set that would let a later
engineer take the tree without guessing.

1. **Compile Hypermedia semantic simplify** in an isolated target
   (`target-production-core`, not synth `target` / `target-studio`). Record
   cargo log and one `mode=semantic` / `mode=admitted` invoke plus empty and
   impossible-target negatives. Keep default `decimate` on QEM.

2. **Run QualiaDB lib tests** for the new modules only
   (`cargo test -p qualia-core-db --lib` filtered to
   `production_contracts`, `attribute_transfer`, `decoded_error_ledger`,
   `ddg_production`, `physical_authority`, `production_runtime`,
   `production_lifecycle`, `decimate_3_semantic`, `production_budget`).
   Keep the log. Do not claim qualification from a green unit-test run.

3. **Make the geometry example consume a documented corpus JSON subset**
   (or add a thin loader). Stop invoking four different fixture IDs against
   the same builtin mesh. Keep `--empty` / `--target-faces` negatives.

4. **Either wire P08 physical records to P10 or keep them unsupported
   forever with a current note.** Do not leave “P10 is not present” in the
   ledger after P10 source exists. Do not invent a zero physical error.

5. **Run one joined tracer** after (1)–(3): geometry exe → decoded vertices →
   existing Forge AffineF32 probe with `--vertices-json` → cached or rebuilt
   Synth HMC. Create-only output. If GPU is unavailable, record unsupported.
   Do not claim three GPU-identical timings from three HMC hashes.

6. **Document GPU lifecycle as unexercised** unless a later owner supplies a
   live device and actually dispatches. Do not promote
   `production_lifecycle` CPU injection to P22 GPU evidence.

7. **Refresh `packet-state.json` and `ROUND-HANDOFF.md`** from this review so
   the next agent does not trust the 08:03 UTC snapshot. Freeze file hashes
   after the compile/test commands above, then invalidate any receipt whose
   source identity changed.

Do not accept P23, do not publish, and do not start R01–R08 until that handoff
exists. The current tree is a coherent source snapshot with one compiled
QualiaDB tracer, Forge unit tests, a partial Python campaign, and an
uncompiled Hypermedia semantic path. That is development evidence, not a
release candidate.
