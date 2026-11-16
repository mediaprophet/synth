# Executable work-packet register

Version 0.1 · 16 November 2026 · planning only

Read alongside the implementation plan and specification v0.2. Hours include the packet’s independent review allocation (20%); that review must be performed by a different agent. The coordinator has a separate allowance. File scopes below are leases to be narrowed to exact paths at dispatch, not permission for simultaneous edits across entire repositories. Dependencies refer to accepted outputs, not merely a worker reporting completion.

## P00 — Freeze the release contract and workspace baseline

**Owner:** Coordinator · **Dependencies:** none

**Implementation:** `gpt-6.1-sol` / high · **Independent review:** `gpt-6-astra` / high

**Estimated agent-hours O / planning / conservative:** 2.4 / 4 / 7.2. GPU reservation: no.

**Requirements:** REQ-01, REQ-05.

**Write boundary:** Planning artifacts only; record repository heads, dirty-file hashes and proposed packet file leases.

**Deliverables**

- Core domain/hardware/quality profile; 38-row acceptance matrix; ownership and interface decision log.
- Named mandatory and optional scope, baseline inventory and initial worktree/build-target policy.

**Acceptance**

- Every requirement has a test owner and applicability reason; no mandatory feature silently deferred.
- Freeze candidate thresholds before comparing implementations; select one native adapter and one browser WebGPU route, with actual availability recorded.

**Main risk:** The winning specification may change scope. Reconcile competition decisions before dispatch.

## P01 — Qualify existing mathematical and production capabilities

**Owner:** QualiaDB + Etalon · **Dependencies:** P00

**Implementation:** `gpt-6.1-sol` / high · **Independent review:** `gpt-6.1-sol` / high

**Estimated agent-hours O / planning / conservative:** 4.8 / 8 / 14.4. GPU reservation: yes.

**Requirements:** REQ-03, REQ-06.

**Write boundary:** Capability probes and evidence only; no wholesale library audit or rewrite.

**Deliverables**

- Refresh source-evidence hashes; inspect audit registry then source; execute targeted existing-library probes.
- Build/test commands, feature flags, baseline failures and measured tool/build latency.

**Acceptance**

- Distinguish library presence, callable production exposure and executed qualification.
- Probe semantic QEM, transfer codec, DDG residual, Forge pipeline and adapter; retain actual outcomes.

**Main risk:** Dirty workspaces and stale binary/catalog versions invalidate assumptions.

## P02 — Versioned contracts, identities and result semantics

**Owner:** QualiaDB records / Etalon receipts · **Dependencies:** P00

**Implementation:** `gpt-6.1-sol` / high · **Independent review:** `gpt-6.1-sol` / high

**Estimated agent-hours O / planning / conservative:** 4.8 / 8 / 14.4. GPU reservation: no.

**Requirements:** REQ-02, REQ-07, REQ-13, REQ-30, REQ-34, REQ-35.

**Write boundary:** Existing representation/receipt record adapters and dedicated schema tests; coordinator owns shared exports.

**Deliverables**

- Asset/quality/transfer/kernel/variant/receipt mappings onto existing types; freeze collapse-event/remap and transfer-admission interfaces for P06/P07 parallel work.
- Canonical serialization, content identities, dependency graph, typed referenced resources, reason codes.

**Acceptance**

- Reject unknown normative fields or negotiate versions; no absent metric serialized as zero.
- Hash changes when semantics, precision or ABI changes; reject mismatched source/target identities.

**Main risk:** Do not repurpose CorrespondenceMap9, ClusterDag23 or 64-byte metadata records.

## P03 — Independent corpus and oracle harness

**Owner:** Independent QA · **Dependencies:** P00, P02

**Implementation:** `gpt-6.1-terra` / medium · **Independent review:** `gpt-6.1-sol` / high

**Estimated agent-hours O / planning / conservative:** 6.0 / 10 / 18.0. GPU reservation: no.

**Requirements:** REQ-05, REQ-28, REQ-31, REQ-37.

**Write boundary:** New qualification fixture/harness directories; never modify implementation to force oracle agreement.

**Deliverables**

- Hashed, licensed analytic and adversarial fixtures across all eleven corpus families.
- CPU high-precision or analytic references, baseline runner and visual capture recipe; holdout partition.

**Acceptance**

- Known-bad candidate deliberately fails; identity candidate passes relevant tests.
- Oracles differ in derivation or implementation; no same-helper self-comparison.
- Record seeds, scales, nullspaces, conditioning, tolerances and excluded cases.

**Main risk:** No external fixture or reference library is trusted without provenance and an explicit role.

## P04 — Early end-to-end tracer using existing paths

**Owner:** Hypermedia + Synth · **Dependencies:** P01, P02, P03

**Implementation:** `gpt-6.1-sol` / high · **Independent review:** `gpt-6.1-sol` / high

**Estimated agent-hours O / planning / conservative:** 4.8 / 8 / 14.4. GPU reservation: yes.

**Requirements:** REQ-02, REQ-05, REQ-15, REQ-29, REQ-36.

**Write boundary:** Isolated integration example and tracer tests; existing production entry points only.

**Deliverables**

- Small seam-labelled asset through existing simplification, .10d roundtrip, fixed Forge kernel, render and identity-bound receipt.
- Explicit incomplete-core checklist and measured timings for estimate recalibration.

**Acceptance**

- Visually inspect front/side/three-quarter outputs and decoded geometry.
- Observe real validation/pipeline stages; impossible ratio, empty input and unavailable device produce honest status.

**Main risk:** This is an integration tracer, not acceptance of later conservative bounds, streaming or full semantics.

## P05 — Budget, workspace and cancellation enforcement

**Owner:** QualiaDB + Etalon · **Dependencies:** P01, P02

**Implementation:** `gpt-6.1-sol` / high · **Independent review:** `gpt-6.1-sol` / high

**Estimated agent-hours O / planning / conservative:** 4.8 / 8 / 14.4. GPU reservation: no.

**Requirements:** REQ-04, REQ-15, REQ-26.

**Write boundary:** Workspace/admission adapters and bounded queue helpers; no unrelated allocator changes.

**Deliverables**

- Budget accounting for construction, validation, compilation, tuning and runtime.
- Caller scratch/capacity, cancellation checkpoints, deterministic ordering policy and partial-output disposal.

**Acceptance**

- Zero/one/maximum/exhausted capacities; cancellation leaves no success-shaped partial asset.
- Instrument hot allocation where prohibited; default cold geometry budget 256 MiB with named overrides.

**Main risk:** A budget object does not prove all downstream allocation is accounted for.

## P06 — Hard semantic and topology collapse admission

**Owner:** QualiaDB geometry · **Dependencies:** P01, P03, P05

**Implementation:** `gpt-6.1-sol` / high · **Independent review:** `gpt-6-astra` / high

**Estimated agent-hours O / planning / conservative:** 8.4 / 14 / 25.2. GPU reservation: no.

**Requirements:** REQ-09, REQ-10, REQ-11.

**Write boundary:** decimate_3.rs and decimate_3_semantic.rs admission/ranking changes plus dedicated tests.

**Deliverables**

- Explicit protected labels, topology modes and compatibility checks before collapse.
- Dimensionless cost terms, declared scaling, deterministic tie breaks and constraint-limited termination.

**Acceptance**

- Zero forbidden merges; protected features and orientation survive adversarial collapses; reject a collapse with no admissible required transfer under the P02 contract.
- Boundary penalties never substitute for locks; scale/permutation metamorphic tests.
- Unreachable target ratio returns achieved count and reason without weakening constraints; P08 tests combined admission and actual transfer.

**Main risk:** Finite penalties are not hard constraints; exact signs do not imply exact constructions.

## P07 — Typed attribute and correspondence transfers

**Owner:** QualiaDB geometry / codecs · **Dependencies:** P02, P03, P05

**Implementation:** `gpt-6.1-sol` / high · **Independent review:** `gpt-6-astra` / high

**Estimated agent-hours O / planning / conservative:** 8.4 / 14 / 25.2. GPU reservation: no.

**Requirements:** REQ-08, REQ-12.

**Write boundary:** Dedicated transfer adapters and UV/corner/skin remapping; read decimator output contract, request shared edits from coordinator.

**Deliverables**

- Source/target maps with intensive, extensive, categorical, vector-frame and force-transfer rules.
- UV seam/corner split handling, skin weights, normals/tangents and lawful existing codec use.

**Acceptance**

- Constants and rigid motion; virtual-work transpose identity; conservation where claimed.
- No transfer support crosses protected domains; quantized existing tetra bindings sum to 65535.
- Checker texture, tangent handedness and joint influence fixtures survive encode/decode.

**Main risk:** Surface and tetrahedral maps have different schemas; arbitrary sparse maps need their own lawful referenced resource.

## P08 — Decoded-output geometry bounds and error ledger

**Owner:** QualiaDB geometry · **Dependencies:** P03, P06, P07

**Implementation:** `gpt-6.1-sol` / high · **Independent review:** `gpt-6-astra` / high

**Estimated agent-hours O / planning / conservative:** 9.6 / 16 / 28.8. GPU reservation: no.

**Requirements:** REQ-13, REQ-14, REQ-15.

**Write boundary:** Distance/BVH assessment adapters, bounded enclosure route, quantization checks and evidence records.

**Deliverables**

- Separate geometric, attribute, quantization, physical and numerical error records.
- Symmetric triangle-surface deviation bracket on the frozen bounded domain, with conservative arithmetic and explicit inconclusive status.

**Acceptance**

- Compare reference to decoded final asset, not just prequantization mesh; exercise combined P06 admission and P07 transfers.
- Document enclosure arithmetic assumptions. Accept G1 only when U <= 1e-3 D and U-L <= 1e-5 D; reject when L > threshold; otherwise unmet width or exhausted budget is inconclusive.
- Test translated large-coordinate, thin-feature, near-threshold and known-distance fixtures; sampled estimates stay labelled estimates.

**Main risk:** Highest geometry uncertainty: finite interval containers are insufficient for rigorous outward rounding.

## P09 — Surface operator conventions and true residuals

**Owner:** QualiaDB mathematics · **Dependencies:** P01, P03, P07

**Implementation:** `gpt-6.1-sol` / high · **Independent review:** `gpt-6-astra` / high

**Estimated agent-hours O / planning / conservative:** 6.0 / 10 / 18.0. GPU reservation: no.

**Requirements:** REQ-16, REQ-17.

**Write boundary:** DDG/solver adapters and scoped diagnostic fixes; no new intrinsic coarsening engine.

**Deliverables**

- Units, sign, boundary conditions, mass, nullspace and transfer contract.
- True algebraic residual instrumentation and scale-aware convergence/rejection reports.

**Acceptance**

- Analytic/nullspace fixtures; residual is ||b-Ax|| under declared norm, not iterate change.
- Check stability of explicit heat routes and sparse scaling; downstream diffusion test.
- Reject unsupported size/conditioning rather than send unbounded dense problems to a local solver.

**Main risk:** Existing dense eigensolvers and A-transpose-A SVD are not automatic scalable sparse references.

## P10 — Authoritative physical-state interface and evidence

**Owner:** QualiaDB physics adapters · **Dependencies:** P03, P07, P09

**Implementation:** `gpt-6.1-sol` / high · **Independent review:** `gpt-6-astra` / high

**Estimated agent-hours O / planning / conservative:** 4.8 / 8 / 14.4. GPU reservation: no.

**Requirements:** REQ-18, REQ-19.

**Write boundary:** Representation/physical-state adapters and small connected reference fixtures; no new general contact solver.

**Deliverables**

- Constitutive/load/time/units/authority declarations and visual-to-authoritative state transfer.
- Scoped inversion/contact evidence and explicit unsupported-guarantee rejection.

**Acceptance**

- Visual LOD changes do not replace authoritative solver topology.
- Rest plus deformed-state transfer checks; virtual work and prescribed response tests.
- Independent-tetrahedron XPBD never labelled a race-free connected global solver or universal collision guarantee.

**Main risk:** Physical reduction and new feasible nonlinear contact methods belong to optional research.

## P11 — Coherent cluster hierarchy and cut validator

**Owner:** QualiaDB hierarchy · **Dependencies:** P06, P07

**Implementation:** `gpt-6.1-sol` / high · **Independent review:** `gpt-6.1-sol` / high

**Estimated agent-hours O / planning / conservative:** 7.2 / 12 / 21.6. GPU reservation: no.

**Requirements:** REQ-20.

**Write boundary:** ClusterDag construction adapters/validation and compatible codec roundtrip tests.

**Deliverables**

- Hierarchy with replacement-group coverage, seam/border invariants and bounded storage.
- CPU cut reference and explicit parent/child dependency identities.

**Acceptance**

- No holes, duplicate coverage or unsupported sibling replacement in valid cuts.
- Random legal/illegal cuts, disconnected assets and quantized codec roundtrip.
- Existing 64-vertex/124-triangle and node layout limits stay authoritative.

**Main risk:** A per-cluster LOD choice alone is insufficient to prove a valid global cut.

## P12 — Semantic numerical IR and guarded rewrites

**Owner:** Etalon Forge · **Dependencies:** P01, P02, P05

**Implementation:** `gpt-6.1-sol` / high · **Independent review:** `gpt-6-astra` / high

**Estimated agent-hours O / planning / conservative:** 7.2 / 12 / 21.6. GPU reservation: no.

**Requirements:** REQ-24, REQ-25, REQ-27.

**Write boundary:** Existing KernelSpec and numerical graph extensions; preserve public interfaces where possible.

**Deliverables**

- Math/discretization/schedule/ABI/evidence separation over existing IR.
- Units/shapes/assumptions/effects/arithmetic annotations and rewrite proof obligations.

**Acceptance**

- Reject invalid dimensions, undeclared aliasing and domain-invalid simplifications.
- Floating-point rewrite legality distinct from real algebra; record applied rule/assumption trace.
- One reduction, one sparse operator and geometry selection contract exercise the design.

**Main risk:** Do not turn cold CAS into an unbounded hot-path optimizer.

## P13 — Bounded portable WGSL lowering

**Owner:** Etalon Forge · **Dependencies:** P03, P12

**Implementation:** `gpt-6.1-sol` / high · **Independent review:** `gpt-6-astra` / high

**Estimated agent-hours O / planning / conservative:** 7.2 / 12 / 21.6. GPU reservation: yes.

**Requirements:** REQ-26, REQ-28, REQ-29, REQ-31.

**Write boundary:** Existing emit/schedule adapters and kernel execution tests.

**Deliverables**

- Checked indices, tail paths, loops, barriers, alias/effect rules and ABI layouts.
- Generated -> Naga -> pipeline -> oracle stages with real execution receipts.

**Acceptance**

- Empty/ragged/max shapes; odd workgroup tails; divergent-barrier rejection; alias counterexamples.
- Host and WGSL layout byte agreement; CPU/GPU comparison and metamorphic tests.
- GPU unavailability is unsupported/skipped evidence, never a pass.

**Main risk:** Portable syntax does not establish semantic portability across adapters.

## P14 — Precision routes and backend qualification

**Owner:** Etalon numerical execution · **Dependencies:** P13

**Implementation:** `gpt-6.1-sol` / high · **Independent review:** `gpt-6-astra` / high

**Estimated agent-hours O / planning / conservative:** 6.0 / 10 / 18.0. GPU reservation: yes.

**Requirements:** REQ-27, REQ-28, REQ-29.

**Write boundary:** Arithmetic profile/probe and qualified df64 adapter tests; no claim of universal WGSL f64.

**Deliverables**

- Ordinary f32 and optional paired-f32 profile admission, ranges and backend identities.
- Numerical boundaries, cancellation/overflow/underflow and reassociation-sensitive probes.

**Acceptance**

- K1 well-conditioned tests plus explicit ill-conditioned refusal/error envelopes.
- Paired f32 must pass its own oracle; failed route disabled while qualified f32 remains available.
- Qualify frozen native and browser routes; unavailable matrix entries remain missing release evidence.

**Main risk:** A wider effective mantissa is not IEEE binary64 semantics.

## P15 — Evidence stages and dependency-aware cache

**Owner:** Etalon receipts / QualiaDB identities · **Dependencies:** P02, P13

**Implementation:** `gpt-6.1-terra` / medium · **Independent review:** `gpt-6.1-sol` / high

**Estimated agent-hours O / planning / conservative:** 4.8 / 8 / 14.4. GPU reservation: no.

**Requirements:** REQ-29, REQ-30, REQ-34, REQ-35.

**Write boundary:** Manifest/cache adapters and invalidation tests; shared identity schema frozen by P02.

**Deliverables**

- Stage-specific receipts tied to input/source/options/ABI/precision/adapter and measurement method.
- Transitive invalidation and canonical cache keys.

**Acceptance**

- Mutate each dependency and observe correct stage invalidation.
- No stale validation promotion, synthetic timing presented as GPU timing, or CRC presented as signature.
- Replay receipts rejects missing hashes and unsupported versions.

**Main risk:** A cache hit can preserve obsolete assumptions unless its dependency closure is complete.

## P16 — Conservative view and deformation selection

**Owner:** Etalon selection · **Dependencies:** P08, P11, P13

**Implementation:** `gpt-6.1-sol` / high · **Independent review:** `gpt-6.1-sol` / high

**Estimated agent-hours O / planning / conservative:** 6.0 / 10 / 18.0. GPU reservation: yes.

**Requirements:** REQ-21.

**Write boundary:** cull_and_lod adapters and CPU/GPU selection comparisons.

**Deliverables**

- Projection conventions, near-plane rules, transformed/deformed bounds and occlusion history policy.
- Selection result tied to a legal cut rather than unrelated independent masks.

**Acceptance**

- Perspective/orthographic and multiview fixtures; two-sided surfaces; camera jumps, near-plane intersections, nonuniform transforms and stale occlusion.
- CPU/GPU selection comparison, no false disappearance in frozen domain.
- Record difference between CPU projected-error and GPU preprojected-error contracts.

**Main risk:** Inflated rest bounds alone may not cover a claimed deformation family.

## P17 — Residency, paging and complete fallback cuts

**Owner:** Etalon runtime / QualiaDB resources · **Dependencies:** P16

**Implementation:** `gpt-6.1-sol` / high · **Independent review:** `gpt-6.1-sol` / high

**Estimated agent-hours O / planning / conservative:** 4.8 / 8 / 14.4. GPU reservation: no.

**Requirements:** REQ-22.

**Write boundary:** Residency state machine, bounded queues, fallback selection and tests.

**Deliverables**

- Dependency-aware replacement readiness, pinned coarse fallback and eviction rules.
- Device loss/missing-page/exhaustion lifecycle behavior.

**Acceptance**

- Random arrival/eviction permutations preserve complete renderable coverage.
- Never activate a replacement with incomplete dependencies; fallback itself must be resident.
- Memory cap and cancellation tested during simulated inflight transfers; P22 exercises actual device/runtime integration.

**Main risk:** Partial refinement may make holes even when each page is individually valid.

## P18 — Compaction and complete bounded draw generation

**Owner:** Etalon GPU runtime · **Dependencies:** P13, P17

**Implementation:** `gpt-6.1-sol` / high · **Independent review:** `gpt-6.1-sol` / high

**Estimated agent-hours O / planning / conservative:** 4.8 / 8 / 14.4. GPU reservation: yes.

**Requirements:** REQ-23, REQ-26.

**Write boundary:** Selection-to-worklist/indirect submission adapters and execution tests.

**Deliverables**

- Mask -> bounded scan/compact -> command generation -> indexed draw path.
- Capacity/overflow records and measured phase costs.

**Acceptance**

- Zero/all/alternating selected, exact capacity and one-over-capacity cases.
- Compare CPU reference lists and actual rendered coverage; no stale indirect arguments.
- Respect dispatch limits, alignment and backend requirements.

**Main risk:** Existing mask generation is only the first stage of complete work generation.

## P19 — Immutable quality gates and bounded tuning

**Owner:** Etalon tuning · **Dependencies:** P08, P14, P15

**Implementation:** `gpt-6.1-terra` / medium · **Independent review:** `gpt-6.1-sol` / high

**Estimated agent-hours O / planning / conservative:** 4.8 / 8 / 14.4. GPU reservation: yes.

**Requirements:** REQ-32.

**Write boundary:** Fixed-schedule baseline, bounded deterministic candidate enumeration and Pareto reporting.

**Deliverables**

- Equal-budget kernel-schedule comparison with frozen quality/resource gates.
- Kernel build/compile/upload/steady costs and break-even accounting; integrated decode/compaction/draw confirmation belongs to P22 after P18.

**Acceptance**

- Invalid fast candidate cannot win; immutable thresholds validated against tampering.
- Budget exhaustion preserves qualified baseline; tie rules reproducible.
- Publish all candidate attempts/failures; optional learned search excluded from core.

**Main risk:** Tuning noise and unfair preprocessing can create false wins.

## P20 — Atomic activation, rollback and state migration

**Owner:** Etalon runtime + QualiaDB authority · **Dependencies:** P10, P15, P18, P19

**Implementation:** `gpt-6.1-sol` / high · **Independent review:** `gpt-6-astra` / high

**Estimated agent-hours O / planning / conservative:** 4.8 / 8 / 14.4. GPU reservation: no.

**Requirements:** REQ-33.

**Write boundary:** Variant activation/state migration adapters and fault injection tests.

**Deliverables**

- Qualified dependency-set activation at a declared boundary.
- Retained previous configuration, device-loss recovery and quantified transfer/migration semantics.

**Acceptance**

- State-machine fault injection of incomplete assets, invalidated evidence and device loss around activation; P22 performs actual GPU lifecycle integration.
- No mixed-version mesh/shader/transfer set; irreversible consumer policy explicit.
- Reference state retained or migration fails before publication; energy/momentum effects reported when applicable.

**Main risk:** Rolling back bytes does not undo already-consumed simulation state.

## P21 — Existing morphism and deterministic production integration

**Owner:** Hypermedia + Synth · **Dependencies:** P04, P20

**Implementation:** `gpt-6.1-sol` / high · **Independent review:** `gpt-6.1-sol` / high

**Estimated agent-hours O / planning / conservative:** 4.8 / 8 / 14.4. GPU reservation: yes.

**Requirements:** REQ-02, REQ-15, REQ-36.

**Write boundary:** Existing simplify arguments/result wiring, tests and isolated showcase; coordinator alone edits shared catalogs/manifests.

**Deliverables**

- Actual existing operation exposure, backward compatibility, .hmc packaging and embedded .10d previews.
- Three repeated asset/source builds and documented determinism boundary.

**Acceptance**

- Operation/ontology/dispatch/aliases agree; inventory remains 150 unless explicitly approved change.
- Inspect front/side/three-quarter, wireframe/normals and error overlays.
- Respect create-only outputs, existing format ownership and active MCP build targets.

**Main risk:** A library method is not automatically a callable production operation.

## P22 — Independent ablations and qualification campaign

**Owner:** Independent QA · **Dependencies:** P09, P10, P14, P21

**Implementation:** `gpt-6.1-sol` / high · **Independent review:** `gpt-6-astra` / high

**Estimated agent-hours O / planning / conservative:** 7.2 / 12 / 21.6. GPU reservation: yes.

**Requirements:** REQ-05, REQ-31, REQ-37, REQ-38.

**Write boundary:** Qualification runners, receipts and report artifacts; implementation fixes go back to their packet owners.

**Deliverables**

- B0-B5 relevant comparisons; pinned B6 external reference where feasible.
- Full corpus, heldout poses/views, error-versus-cost curves and ablations.

**Acceptance**

- All failures/exclusions in denominator; identical quality gates and budgets; confirm integrated decode/upload/compaction/draw costs after P18.
- At least five warmups and 30 retained timing samples; more when uncertainty prevents decision.
- Numeric checks plus actual visual review; native/browser feature matrix has no silent skips; exercise real GPU lifecycle/device-loss handling where supported and document injection scope.

**Main risk:** Tail guarantees need a justified sample design, not a 30-sample p95.

## P23 — Release gate and reproducible handoff

**Owner:** Coordinator + independent reviewer · **Dependencies:** P22

**Implementation:** `gpt-6.1-sol` / high · **Independent review:** `gpt-6-astra` / high

**Estimated agent-hours O / planning / conservative:** 6.0 / 10 / 18.0. GPU reservation: no.

**Requirements:** REQ-01, REQ-02, REQ-03, REQ-04, REQ-05, REQ-06, REQ-07, REQ-08, REQ-09, REQ-10, REQ-11, REQ-12, REQ-13, REQ-14, REQ-15, REQ-16, REQ-17, REQ-18, REQ-19, REQ-20, REQ-21, REQ-22, REQ-23, REQ-24, REQ-25, REQ-26, REQ-27, REQ-28, REQ-29, REQ-30, REQ-31, REQ-32, REQ-33, REQ-34, REQ-35, REQ-36, REQ-37, REQ-38.

**Write boundary:** Release evidence/documentation, frozen manifests and reproduction commands; no unrequested publishing.

**Deliverables**

- Complete requirement/test/evidence/reviewer matrix; known limitations and operational runbook.
- Accepted release candidate with exact revisions/file hashes, builds, artifacts and rollback instructions.

**Acceptance**

- All core gates pass on frozen support matrix; no unresolved correctness defects or disguised deferrals.
- Independent clean reproduction of representative end-to-end result.
- Recheck source identity after fixes; invalidate affected evidence and rerun only impacted checks.

**Main risk:** Feature complete is not independently accepted; missing hardware evidence prevents that claim.

## Optional research packets

These are not commissioned by the planning request. Each track begins with a falsifiable experiment, then a separate productionisation decision. Estimates include review but are not guarantees of a successful research result.

### R01 — Intrinsic coarsening

Depends on: P07 P09 P23. Planning allowance: **70 agent-hours**; rough optimistic/conservative envelope 38–133 hours. Sol/high implementation; Astra/high independent design and numerical review.

**Accepted bounded outcome:** Intrinsic triangulation and transfer with explicit representability domains; bounded downstream diffusion/geodesic comparison.

**Stop or re-scope trigger:** No benefit at equal total cost, invalid transfer or scalability failure.

### R02 — Spectral preservation

Depends on: R01. Planning allowance: **55 agent-hours**; rough optimistic/conservative envelope 30–104 hours. Sol/high implementation; Astra/high independent design and numerical review.

**Accepted bounded outcome:** Clustered eigenspace comparison and sparse scalable solver strategy; O1 plus downstream behavior on heldout meshes.

**Stop or re-scope trigger:** Unstable repeated modes, insufficient spectral gaps or preprocessing outweighing benefits.

### R03 — Physical reduction and hyperreduction

Depends on: P10 P23. Planning allowance: **120 agent-hours**; rough optimistic/conservative envelope 66–228 hours. Sol/high implementation; Astra/high independent design and numerical review.

**Accepted bounded outcome:** One declared constitutive/load family, verified full-order reference, reduced basis and bounded cubature; P1 and scoped feasibility.

**Stop or re-scope trigger:** Heldout load/contact response fails; full model remains authoritative. No general contact solver included.

### R04 — Appearance and deformation-aware optimization

Depends on: P07 P08 P23. Planning allowance: **70 agent-hours**; rough optimistic/conservative envelope 38–133 hours. Sol/high implementation; Astra/high independent design and numerical review.

**Accepted bounded outcome:** Fixed renderer/material/lighting/pose domain, rebaking/correspondence and temporal error measures.

**Stop or re-scope trigger:** Heldout views/poses fail or cost exceeds equal-quality baseline.

### R05 — Quad-preserving authoring

Depends on: P06 P23. Planning allowance: **65 agent-hours**; rough optimistic/conservative envelope 36–124 hours. Sol/high implementation; Astra/high independent design and numerical review.

**Accepted bounded outcome:** Bounded cold quad-domain optimizer, singularity/feature policy, downstream triangulation and roundtrip.

**Stop or re-scope trigger:** No robust admitted-domain result within work budget; preserve original quad topology.

### R06 — Learned proposal search

Depends on: P19 P23. Planning allowance: **50 agent-hours**; rough optimistic/conservative envelope 28–95 hours. Sol/high implementation; Astra/high independent design and numerical review.

**Accepted bounded outcome:** Existing .p64 proposer integration, heldout split and invalid-proposal accounting against equal-budget deterministic search.

**Stop or re-scope trigger:** No reproducible total-cost benefit; use fixed/deterministic baseline.

### R07 — Native backend specialization

Depends on: P14 P23. Planning allowance: **75 agent-hours**; rough optimistic/conservative envelope 41–142 hours. Sol/high implementation; Astra/high independent design and numerical review.

**Accepted bounded outcome:** One named extra backend/precision route, same semantic ABI/oracle and isolated fallback.

**Stop or re-scope trigger:** Backend arithmetic or resource contract fails; portable route retained.

### R08 — Generated adaptive predicates

Depends on: P12 P14 P23. Planning allowance: **85 agent-hours**; rough optimistic/conservative envelope 47–162 hours. Sol/high implementation; Astra/high independent design and numerical review.

**Accepted bounded outcome:** One bounded predicate family, filtered arithmetic/escalation worklists, exact-sign oracle and exhaustion behavior.

**Stop or re-scope trigger:** Unbounded escalations, backend transformations or unproven signs invalidate route.
