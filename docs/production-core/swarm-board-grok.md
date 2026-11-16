# Grok production-core swarm board

Date: 16 November 2026. Coordinator: Grok 4.6. Worktrees stay at
`C:/Users/Hp/.codex/worktrees/qualia-etalon-production-core`.

Codex 4-slot / 20-day estimate is discarded as a session constraint, not a
physical limit. Packets P00–P23 remain the work list. Optional R01–R08 stay
uncommissioned. No commits, no MCP kill, no rebuild into synth `target` or
`target-studio`. Implement source first; one later consolidated cargo pass.

## Wave 1 leases (disjoint)

| Agent | Packets | Write paths | Subagent | Status |
|---|---|---|---|---|
| p07-transfer | P07 + example wiring | `qualiaDB/.../attribute_transfer/**`, `examples/production_core_geometry.rs` | `01a1e368-1ed0-7e02-b44e-8d80e8cf86e9` | source rewritten; duplicates gone; example on admitted driver; cargo unrun |
| p08-bounds | P08 | `qualiaDB/.../decoded_error_ledger/**` NEW | `01a1e368-1ed1-7bd1-a1b6-a2749dd0b7c4` | source written; `pub mod decoded_error_ledger` wired; physical channel still unsupported placeholder; cargo unrun |
| p09-ddg | P09 | `qualiaDB/.../ddg_production/**` NEW | `01a1e368-1ed1-7bd1-a1b6-a2875e8366a1` | source written; `pub mod ddg_production` wired; cargo unrun |
| p10-physics | P10 | `qualiaDB/.../physical_authority/**` NEW | `01a1e368-1ed1-7bd1-a1b6-a29a7598ea83` | source written; `pub mod physical_authority` wired; cargo unrun |
| p11-runtime | P11, P16, P17, P18, P20 | `qualiaDB/.../production_runtime/**` NEW | `01a1e368-1ed1-7bd1-a1b6-a2a895afc94b` | source written; `pub mod production_runtime` wired; cargo unrun |
| p03-corpus | P03 | `synth/scripts/production_core/qualification_corpus.py`, `synth/docs/production-core/corpus/**` | `01a1e368-1ed1-7bd1-a1b6-a2bc64fc921e` | 37 fixtures + manifest published; qualification not executed |
| p12-forge | P12, P15 | `etalon/crates/etalon-wgsl-forge/src/production_ir/**` NEW | `01a1e368-1ed1-7bd1-a1b6-a2cb770c8b2b` | source written; `pub mod production_ir` wired; cargo unrun |

Coordinator owns `computational_geometry/mod.rs`, `etalon-wgsl-forge/src/lib.rs`,
Cargo.toml/lock, catalogs. Agents must not edit those shared files.

## Wave 2 leases (started while P08 finishes)

| Agent | Packets | Write paths | Status |
|---|---|---|---|
| p13-lowering | P13, P14 | `etalon/.../production_lowering/**` NEW | `01a1e375-f921-77c2-9109-c25fe69575cd` | source written; `pub mod production_lowering` wired; cargo unrun |
| p21-simplify | P21 | hypermedia `manifold_ops.rs` + optional `production_simplify.rs` | `01a1e375-f922-7e60-bcad-ae6bc60a0409` | `mode=semantic|admitted` wired; coordinator path-dep to sibling QualiaDB; cargo unrun |
| p04-tracer | P04 | synth `scripts/production_core/*tracer*` | `01a1e375-f923-7d93-a469-bd3ace02d585` | inspect-only tracer + admitted probe; 12 python tests passed; no fake GPU receipts |

## Compile

QualiaDB `production_core_geometry` **compiled and executed**.

- First cargo: `etalon-wgsl-forge` Eq/serde errors (fixed).
- Second cargo: QualiaDB Copy/float/borrow errors (fixed).
- Third cargo: `CARGO_EXIT=0`, `Finished dev` in 8m 00s.
- Binary: `qualiaDB/target-production-core/debug/examples/production_core_geometry.exe`
- Run receipt: `synth/docs/production-core/evidence/admitted-geometry-run-01/geometry-receipt.json` status `passed` (seam fixture, admitted driver). This is a tracer run, not P22 qualification.
- Empty fixture: exit 2, `invalid_input`, no output directory.
- Impossible target 33: exit 2, `invalid_input`, no output directory.
- Zero target: exit 2, `unsatisfied_constraint`, committed prefix artifacts (ConstraintLimited).
- Forge lib tests: 24 passed (production_ir / production_lowering / production_quality).

## Continue-to-finish (same day)

QualiaDB new-module lib tests: **87 passed**, `qdb-module-tests-03.log`, `CARGO_EXIT=0`.
Geometry example tests: **5 passed** including corpus JSON loader.
Geometry exe rebuilt with `--corpus-json`. Corpus runs:

- `evidence/geometry-corpus-analytic-plane-01` analytic-plane-scale-1, 4v/2f, consumes_corpus_json true, passed
- `evidence/geometry-corpus-identity-seam-01` identity-two-subpart-seam, 6v/2f, consumes_corpus_json true, passed

P08 physical channel: P10 is present. Absence is `NotSupplied` / Unsupported, never a silent 0.0. Supplied `PhysicalEvidence` is mapped.

P21 Hypermedia: `asset-production-vibe` `--lib` tests `production_simplify` **7 passed** in `target-production-core` (`hypermedia-semantic-03.log`). Cube target-8 reduction exercised. Default `mode=decimate` remains geometric QEM. Public morphism was not invoked through cached `synth.exe`.

P22 campaign run-02: 37 fixtures in denominator, 21 executed, 19 consumed corpus JSON, 1 corpus_success (identity-two-subpart-seam must_pass on corpus bytes). Status `executed_partial_not_qualified`. Never qualified.

P04 joined tracer: `evidence/joined-tracer-01` status `passed_numeric_and_packaging_pending_visual_review`. RTX 5070 DX12 AffineF32, three geometry + three HMC hashes, visual rendered pending independent inspection. GPU-identical timing is not claimed.

P00 remains the only accepted packet. R01–R08 uncommissioned. GPU lifecycle is still a GPU-less ledger. Whole-surface bound is not measured.

## Frozen facts from review

- P00 accepted (scope only). 23 packets unaccepted.
- P02/P05/P06 source exists and looks usable; uncompiled as a set.
- P07 `attribute_transfer/mod.rs` is a concatenated draft: duplicate
  `TransferError`, duplicate `source_frame`/`target_frame` fields, missing
  `tests.rs`. It cannot compile.
- Protected-domain collapse rejection must be `UnsatisfiedConstraint` for P06.
- `DdgReport.residual` is iterate change, not `||b-Ax||`.
- Hypermedia `simplify` still calls geometric `decimate_qem`.
- Existing CDAG, FEMC, Forge IR, BVH/distance stay authoritative.
