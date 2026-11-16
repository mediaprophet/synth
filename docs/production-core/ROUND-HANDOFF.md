# QualiaDB / Etalon production core — round handover

Date: 16 November 2026. This document records the implementation round; it does not replace or amend the accepted specification or 24-packet plan.

## 1. Start here

Continue from the isolated worktrees below. **P00 is the only accepted packet. The other 23 packets remain unaccepted.** New production source has been written, but it has not been compiled or tested as an integrated set. Earlier successful baseline checks apply only to their recorded sources and binaries.

The latest user instruction is to finish this round and provide a comprehensive handover to another agent. Implementation workers are stopped for the snapshot. Do not infer completed validation from source presence or from this handover.

The user explicitly asked that the handover preserve the work accurately. The original specification and plan are retained unchanged under `synth/docs/production-core/plan/`. The final snapshot manifest records their hashes alongside implementation files.

The user’s working preferences are:

- Use coordinated sub-agents for the 24 implementation packets.
- Report active sub-agent count and packets remaining every 15 minutes during an active run. Distinguish implemented source from accepted packets.
- Complete coherent implementation before a consolidated build/test pass; avoid repeated Cargo invocations between small changes.
- Preserve existing QualiaDB mathematical capabilities, existing Etalon providers, and the existing 150-morphism production catalog.
- A three-hour delivery target was requested at 07:41:53 UTC, before the subsequent instruction to close this round and hand over. It was a requested target, not measured completion evidence. The prior planning estimate was 20 working days, range 12–35; no evidence-based replacement estimate was established in this round.

This session provided four concurrent agent slots, including the coordinator. Three workers were used concurrently. This is a property of this session, not a requirement for the next environment.

## 2. Worktrees and source provenance

Base directory:

`C:/Users/Hp/.codex/worktrees/qualia-etalon-production-core`

| Repository | Worktree | Base commit |
|---|---|---|
| Synth | `.../synth` | `0d08884612f387685eb96e5ddd0a2085c1f1ff02` |
| QualiaDB | `.../qualiaDB` | `7f6057a743937173260c1d62d670283214a1d09a` |
| Etalon | `.../etalon` | `48af56392a13ba7734c438887b58db4a1057f1bd` |
| Hypermedia | `.../hypermedia-production` | `8321792cb700cdac90adf1b1ef8accdc039597f1` |

All four branches are `codex/geometry-production-core`. No commits, merges, deployment or publication were performed. Original source checkout modifications were not copied into these worktrees. The baseline records original dirty paths/hashes; preserve them.

Synth was created as a managed worktree. The three siblings are valid Git worktrees; attaching them to the desktop chat failed its ownerless-worktree ownership check. They remain accessible by explicit filesystem path. There was no workaround that modified app ownership metadata.

The session’s filesystem policy required escalated write commands for the newly created worktrees. Authorized worktree writes were approved; no automatic-review rejection remained outstanding. A next session may have different writable roots.

Dependency resolution:

- Synth already resolves sibling QualiaDB/Hypermedia/Etalon paths through its existing configuration.
- Standalone QualiaDB commands must use `--config .cargo/production-core.toml`. This new config patches the Etalon Git source to the isolated sibling worktree.
- QualiaDB `Cargo.lock` changed as metadata resolution replaced 30 Git-source entries with local path packages. No dependency-version upgrade was intended.
- Hypermedia’s existing `.cargo/config.toml` already has sibling patches. Hypermedia source was not modified this round.

## 3. Packet state

“Source written” below is not acceptance. Acceptance requires the original packet gates and independent evidence. `packet-state.json` is the machine-readable board; the source snapshot and this document take precedence over earlier progress prose.

| Packet | Original scope | State at handover |
|---|---|---|
| P00 | Scope, baseline, release contract | Accepted: scope and test contracts, not full implementation |
| P01 | Qualify existing capabilities | Partial: native/browser baselines executed; geometry build failed; remaining math/codec qualification pending |
| P02 | Versioned contracts and identities | Production source + regression source written; uncompiled/unexecuted; review gaps listed below |
| P03 | Independent corpus and oracles | Initial fixture and Python oracle code; eleven-family corpus generator source added, not executed |
| P04 | Joined end-to-end tracer | Runner and component examples written; joined production run not executed |
| P05 | Budgets, workspace, cancellation | Caller-buffer capacity/work adapter written and source-reviewed; full pipeline adoption/qualification pending |
| P06 | Hard semantic/topology admission | Bounded admission source snapshot; final worker report gives exact completion limits |
| P07 | Typed attribute/correspondence transfer | Partial transfer source snapshot; `tests.rs` is absent despite a test-module declaration |
| P08 | Decoded-output bounds/error ledger | Not implemented as a production packet |
| P09 | Surface operators/true residuals | Existing APIs inspected; production adapter not implemented |
| P10 | Physical authority/evidence | Not implemented |
| P11 | Hierarchy and legal cuts | Not implemented; coordinator scratch drafts were not installed/exported |
| P12 | Semantic numerical IR | Not implemented as a new production packet |
| P13 | Bounded portable WGSL lowering | Existing fixed affine path exercised; required production extensions not implemented |
| P14 | Precision/backend qualification | Fixed f32 native/browser baseline only; packet incomplete |
| P15 | Evidence stages/cache | Existing evidence APIs inspected; production integration incomplete |
| P16 | View/deformation selection | Not implemented |
| P17 | Residency/fallback cuts | Not implemented |
| P18 | Compaction/complete draws | Not implemented |
| P19 | Quality gates/bounded tuning | Not implemented |
| P20 | Atomic activation/migration | Not implemented |
| P21 | Existing morphism integration | Smoke packaging executed; public semantic production wiring not implemented |
| P22 | Ablations/qualification | Not executed |
| P23 | Release gate/reproduction | Not achieved; this is a development handover |

Optional R01–R08 remain uncommissioned. Their research scope is not silently included in, or substituted for, these 24 core packets.

## 4. Implementation inventory

### QualiaDB

All paths in this subsection are relative to `qualiaDB/crates/qualia-core-db/`.

- `src/specialized_libs/computational_geometry/production_contracts/`: seven focused files, strict versioned records, typed resource references, identities, dependency checks, transfer contract and collapse protocol. Module exported by the coordinator in computational geometry `mod.rs`.
- `src/specialized_libs/computational_geometry/geometry_workspace/production_budget.rs` and `production_budget_tests.rs`: new capacity/work accounting over existing `Cancellation`; exported from existing `geometry_workspace.rs`.
- `src/specialized_libs/computational_geometry/decimate_3_semantic/`: new admitted, caller-buffered simplification path; exact final modules listed in the worker report and snapshot manifest.
- `src/specialized_libs/computational_geometry/decimate_3_semantic.rs`: narrow legacy infinity/zero cost fix, regression source, and new module wiring. Existing API retained.
- `src/specialized_libs/computational_geometry/attribute_transfer/`: partial typed transfer operations and strict existing FEMC adapters. Exported from computational geometry `mod.rs`. Eight source files exist; `mod.rs` declares a `#[cfg(test)] mod tests`, but `tests.rs` was not written before closeout. This is a known test-compilation blocker to resolve before the consolidated test build.
- `examples/production_core_geometry.rs`: geometry/codec example and regression source. It remains on the legacy semantic-QEM path. Connecting it to the new P06 driver and P07 gate is explicitly pending.

The new production modules have **not** been compiled together. Source formatting checks do not establish type correctness, runtime correctness or acceptance.

### Etalon

`crates/etalon-wgsl-forge/examples/production_core_probe.rs` is the added example. It was built and executed before the later implementation phase. It supports an optional decoded-asset input and records actual kernel execution and hashes. No existing Etalon library implementation was changed this round.

`etalon/target-production-core/` is untracked build/evidence output, not source. Do not accidentally add the entire directory to a commit or portable source package.

### Synth

`scripts/production_core/` contains:

- `production_core_artifacts.py`: independent binary64 affine comparison and bounded attributed GLB construction.
- `run_production_core_tracer.py`: create-only joined geometry → decoded input → generated GPU kernel → GLB → existing Synth production → preview runner, with retained receipts and command logs.
- `test_production_core_artifacts.py`: three executed helper test groups, predating the user’s implementation-first instruction.
- `qualification_corpus.py`: deterministic analytic/adversarial fixture generator for eleven families; source only, not run.
- `requirements.txt`: `blake3==1.0.11`. The local `vendor/` copy is excluded from the portable source package; install it explicitly if absent.

The corpus generator’s maximum-size fixture is explicitly a descriptor requiring an execution-time bounded generator. It must not be counted as an executed maximum-size test. Numerical/reference metadata and fixture generation are not test results.

`docs/production-core/` contains the unchanged plan copies, acceptance matrix, board, baseline reports, retained evidence, browser probe source and this handover.

## 5. Frozen interfaces and ownership

P02 uses `ContentId { namespace: HashNamespace::Sha256V1, digest: [u8;32] }`. Canonical v1 bytes are compact UTF-8 JSON in versioned schema field declaration order; `BTreeMap` parameter keys are lexical and arrays retain normative order. This is **not** a claim of generic RFC JSON canonicalization. Changing this encoding requires versioning.

The seven logical records are Asset, Quality, Candidate, Transfer, Kernel, Variant and Receipt. Existing `.10d`, FEMC, CDAG, numerical Forge schema 2 and Etalon receipt schema 1 remain referenced artifacts; their ABIs were not repurposed.

The collapse/transfer protocol uses:

1. A proposal bound to run/source/policy, original endpoint IDs, generations, target position and interpolation parameter.
2. Borrowed original face/corner deltas and caller-owned transfer scratch.
3. Pure preparation before ranking; winner preparation/revalidation before commit.
4. Capacity reservation before an infallible mesh/transfer/journal commit, with no cancellation checkpoint inside the commit.
5. Distinct lineage, survivor compaction, face compaction and corner maps. A lineage map is not an interpolation operator.

The current `TransferAdmission` trait also requires `additional_storage_bytes`, `scratch_bytes`, `work_bound`, `prepared_identity`, `prepare`, `revalidate` and `commit`. Read the frozen source rather than copying older chat signatures.

P05 exposes `CapacityPlan`, `BufferClass`, `ProductionBudget` and `BudgetError` at `geometry_workspace::production_budget`. It checks typed capacity arithmetic and charged work while borrowing the existing cancellation token. Arena padding is conservatively accounted. It does not intercept allocations or make legacy nested `Vec`/`HashMap` algorithms bounded. Each consuming algorithm must account for every live backing buffer once and charge its real scans.

Last source owners:

| Owner | Files |
|---|---|
| Coordinator `/root` | Shared module exports/configuration, P05 budget files, Synth scripts/docs/integration |
| `/root/baseline_forge` | P02 contracts; earlier Etalon example |
| `/root/baseline_geometry` | Semantic QEM fix, P06 modules, geometry example |
| `/root/production_transfer` | P07 attribute-transfer modules |
| `/root/scope_review` | Read-only independent source/scope review |

These leases end with this round. The next coordinator should assign new non-overlapping leases rather than assume any background agent still owns a file.

## 6. Executed evidence

Evidence root: `synth/docs/production-core/evidence/`.

| Check | Observed result | Scope |
|---|---|---|
| P00 independent review | Accepted | Scope, applicability matrix and baseline routes |
| Native Forge DX12 | Passed: 4,099 values, zero mismatches/max error | Fixed generated affine kernel on recorded NVIDIA adapter |
| Browser WebGPU | Passed: 4,099 values, zero mismatches; poisoned output detected | Same fixed shader, Chrome/Intel route |
| Producer GLB smoke | Passed: import → PBR `.10d` → HMC/export/present | Cached producer binary; six vertices/two triangles |
| Smoke native preview | Passed after correcting canonical asset key; three images inspected | Format/render handshake, not joined semantic result |
| Python helper checks | Three test functions passed | Nonfinite/invalid data, affine corruption, GLB attribute/domain guards |
| Joined tracer | Not executed | Geometry executable unavailable after failed first build |
| New P02/P05/P06/P07 modules | Not compiled/tested together | Source and regression source only |

Native hardware: NVIDIA GeForce RTX 5070 Laptop GPU, DX12, driver `32.0.16.2114`. Five warmups and 30 GPU timestamp samples: median 1,088 ns, p95 4,320 ns, minimum 960 ns. Probe wall time 3.559 s; shell wall time 3.840 s. This is one small kernel measurement, not production throughput or implementation velocity.

Native seeded certification uses an existing CPU **f32 `mul_add`** oracle. The optional decoded-asset phase uses an independent scalar f64 oracle. The Python runner independently checks that asset output again. Do not conflate these oracle derivations.

Browser: installed Chrome `157.0.8103.52`, actual WebGPU adapter reported Intel `gen-12lp`, not fallback. Exact device identity is privacy-limited. JavaScript binary64 oracle compared all 4,099 values. No browser GPU timing was claimed. The ephemeral local probe server was stopped.

The exact shared WGSL SHA256 was `d5a6d8ac4dea817f078ffd0d9f0affeb4b2cdee7254e9b625c4bc6e3ba24bd36`.

Cached producer used read-only:

`C:/github/synth/target-prod/debug/synth.exe`

SHA256: `62095c5c1c126bfade382c1251a6293be9c87b41abb88902dedc834ff742618e`.

Its recorded binary identity is evidence. Equivalence to the isolated source HEAD was **not** established. A reproducible source-qualified producer build remains required for release.

The smoke side view was blank because the authored fixture was planar and viewed edge-on. Front/three-quarter showed the intended two colored triangles. This observation is not a geometry-fidelity review of the intended final seam fixture.

Negative evidence:

- Existing Forge output directory rejected without modifying its receipt.
- Unavailable Metal compiled route returned explicit unsupported before pipeline/certification; this is not device-loss qualification.
- Empty optional vertex input failed without a derived output.
- Known-bad numerical output and malformed/mixed-domain GLB inputs were rejected by the helper checks.

## 7. Failures and repairs retained

1. Initial Forge cold build: 308.018 s; failed because compilation observed intermediate optional-argument wiring. Completed source rebuilt successfully in 19.382 s. The final example then executed successfully.
2. QualiaDB cold build: dependencies/core compilation progressed, but the geometry example failed with `E0277`: SHA2 digest array did not implement `LowerHex` for `format!("{:x}", Sha256::digest(...))`. Source changed to explicit byte formatting. **No subsequent build verified that repair.** Full log: `qualiaDB/target-production-core/geometry-build-01.log`. Its observed file timestamp span was 06:52:05.969–07:32:30.054 UTC (40 minutes 24.085 seconds); this is not an independently measured complete command duration.
3. Existing semantic cost source computes infinity × zero for same-domain transitions under a hard infinite boundary penalty. The minimal source fix uses exact zero when transitions are zero while retaining rejection across protected domains. Regression source was added. **No executable before/after reproduction was obtained.** Do not label it a passed runtime regression.
4. Initial smoke preview used `--asset-key fixture.10d` and failed HMC identity validation. Correct key `outputs/fixture.10d` passed. The joined runner uses `outputs/gpu-derived.10d`.
5. Initial Python test source had an `else202` syntax typo; corrected before the reported successful helper run.
6. An operations-list command piped into an early-closing truncation command caused a broken-pipe panic. This was command-output handling, not evidence of a missing catalog morphism.

The consolidated validation pass must use the final source snapshot; old build outputs cannot validate code written afterward.

## 8. Independent review findings requiring follow-through

These are source-review observations, not executed test failures. They are retained verbatim in substance so the next agent can verify and resolve them without guessing.

- `production_contracts/validation.rs`, `validate_receipt_against_profile`: metrics and target membership are checked, but required stage, profile identity, provenance, resource limits and full profile validity are not collectively enforced. Empty/no-required criteria can pass vacuously. Do not use this helper alone for activation or certification.
- `production_contracts/identity.rs`, dependency graph validation: establishes content hashes, existence and acyclicity. It does not by itself establish compatible record roles, variant/kernel precision agreement, or selected transfer endpoint relationships.
- `production_contracts/records.rs`, `ResourceRef::verify`: verifies byte length and content hash, not codec validity or element counts. Resolve and decode resources under their actual codecs.
- `production_contracts/collapse.rs`, `CollapseProposal::validate`: checks basic numeric/index form. The driver must establish run/source/policy binding, actual endpoint bounds/generations and neighborhood consistency.
- P05 counters only bound work that consumers charge. Source review found no arithmetic or cooperative-cancellation blocker in the ledger; this is not allocation instrumentation or full pipeline qualification.

Other integration facts:

- P07 is a partial, unformatted/uncompiled snapshot. Besides the absent test module, its collapse protected-domain/affine rejection currently returns `UnsupportedCapability`, while P06 expects `UnsatisfiedConstraint` for candidate rejection. Reconcile this before integration. P07 `additional_storage_bytes()` also counts original positions/stable IDs already counted by P06; reconcile backing-buffer accounting. Smooth/MikkTSpace reconstruction, nonrigid frames, signed/extrapolating transfer maps and skin unions above four influences are explicitly unsupported in this new adapter.
- Existing public Hypermedia `simplify` currently uses geometric QEM. P06 library code is not automatically public production exposure.
- Existing FEMC constructor clamps invalid input weights; strict preflight is necessary. Its legacy validation skips tetra index bounds when tetrahedron count is zero. P07 must enforce the stricter admitted domain.
- Existing DDG solver `DdgReport.residual` represents iterate change in the inspected path. A true algebraic residual must be independently computed and labelled.
- Existing CDAG monotonic per-node selection does not establish coherent replacement coverage. The inspected GPU culling ABI contains 12 scalars per node and consumes preprojected errors.
- The culling shader inspected reads coupled parameter/output buffers using a count derived from the cluster input. Future admission/lowering must validate all coupled lengths and dispatch limits.
- The initial runner records that final production `.10d` sealing is not independently decoded/compared. Visual inspection cannot establish exact UV/material retention.

## 9. Next-agent execution sequence

1. Read repository `AGENTS.md`, this handover, the unchanged specification/packet register, and the final worker reports. Verify snapshot hashes. Preserve original checkout changes.
2. Review P02/P05/P06/P07 source together. Resolve the review findings and exact callback compatibility. Check final example integration before scheduling a build. Keep packet acceptance separate from source completion.
3. Complete the remaining implementation in dependency order with exact file leases. Reuse existing mathematics/codecs/providers. Preserve all original gates; unsupported or unmeasured cases remain explicit.
4. Run a consolidated qualification pass once that implementation is ready. The commands below are starting points, not claims that they have passed against this snapshot.
5. Execute the joined example, independently decode final output, verify identities, inspect all required views/overlays, then update the packet board from retained evidence and independent review.
6. Reforecast from accepted work and actual remaining critical-path effort. Do not extrapolate the affine timing or count source files as accepted packets.

Suggested consolidated QualiaDB compilation, in its isolated worktree:

```powershell
$env:CARGO_TARGET_DIR = 'C:\Users\Hp\.codex\worktrees\qualia-etalon-production-core\qualiaDB\target-production-core'
$env:CARGO_BUILD_JOBS = '1'
$env:CARGO_INCREMENTAL = '0'
$env:RUSTFLAGS = '-C debuginfo=0'
cargo --config .cargo/production-core.toml test --offline --no-default-features -p qualia-core-db --lib --no-run
```

After that single test-binary build, run the relevant test filters against the emitted binary (contracts, budget, semantic admission, transfers), retaining exit codes and output. The original build set `CARGO_BUILD_JOBS=1`, `CARGO_INCREMENTAL=0`, `RUSTFLAGS=-C debuginfo=0`, and used explicit `--target-dir target-production-core`. It did not set profile-debug or target-dir environment variables; inherited values were not recorded. The baseline alone does not establish that the above consolidated test binary will compile.

Build the ordinary geometry executable once implementation is ready:

```powershell
cargo --config .cargo/production-core.toml build --offline --no-default-features -p qualia-core-db --example production_core_geometry
```

Etalon example, if rebuilding is required:

```powershell
$env:CARGO_TARGET_DIR = 'C:\Users\Hp\.codex\worktrees\qualia-etalon-production-core\etalon\target-production-core'
cargo build --locked --offline -p etalon-wgsl-forge --example production_core_probe --features gpu-native-dx12
```

Python dependency installation, only if the local vendor directory is absent:

```powershell
python -m pip install --target C:\Users\Hp\.codex\worktrees\qualia-etalon-production-core\synth\scripts\production_core\vendor -r C:\Users\Hp\.codex\worktrees\qualia-etalon-production-core\synth\scripts\production_core\requirements.txt
```

Joined run, only after the geometry executable exists and integration is reviewed; select a new output name:

```powershell
python C:\Users\Hp\.codex\worktrees\qualia-etalon-production-core\synth\scripts\production_core\run_production_core_tracer.py `
  --geometry-exe C:\Users\Hp\.codex\worktrees\qualia-etalon-production-core\qualiaDB\target-production-core\debug\examples\production_core_geometry.exe `
  --forge-exe C:\Users\Hp\.codex\worktrees\qualia-etalon-production-core\etalon\target-production-core\debug\examples\production_core_probe.exe `
  --synth-exe C:\github\synth\target-prod\debug\synth.exe `
  --out C:\Users\Hp\.codex\worktrees\qualia-etalon-production-core\synth\docs\production-core\evidence\joined-tracer-01
```

All producer output paths are create-only. Do not delete existing evidence to reuse a name. Do not rebuild into active Synth `target` or `target-studio`, and do not kill a live MCP process. Keep one heavy build and one GPU lease at a time unless actual machine capacity supports a change.

The joined runner checks three geometry asset builds and three HMC builds with fixed embedded receipts. That is not three independent full GPU runs with byte-identical timing receipts. It records this determinism boundary explicitly.

## 10. Portable package and completeness

The accompanying ZIP contains this handover, final worker reports, a SHA256 manifest, changed/new source files at repository-relative paths, tracked diffs, unchanged specification/plan copies and selected baseline evidence. Build trees, local dependency vendor trees, original credentials and cached producer binaries are excluded.

For another machine, obtain the four repositories at the recorded base commits, preserve sibling layout and applicable instructions, then review/apply the provided source snapshot. Do not apply both the full replacement files and tracked diff blindly; they are alternative review/reconstruction representations of the same changes.

The final snapshot manifest and accompanying `worker-reports.md` supply exact file counts/hashes and final P06/P07 details. No release or production-complete claim is made by packaging this development state.
