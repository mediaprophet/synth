# P21 remaining — independent `.10d` decode (agent report)

Packet: **P21 remaining** (`p21-simplify`). Date: 16 November 2026.

Capability ladder for this remaining slice: **source present**. Not compiled, not executed, not qualified in this wave (lease: no cargo).

## Write path

- Edited: `C:\github\hypermedia-production\crates\asset-production-vibe\src\production_simplify.rs`
- Evidence: this file only
- Not edited: `operations_catalog.rs`, `morphisms_03.rs`, `Cargo.toml`, `manifold_ops.rs`, synth `target` / `target-studio`, cached `synth.exe`

Etalon pin `a326f82` in Hypermedia `Cargo.toml` was not touched. `qualia-core-db` remains the sibling path dep.

## What changed

Admitted success now has a producer/reader split:

1. `decimate_semantic_admitted_3` still admits the reduction.
2. Producer: `compile_mesh_to_10d` seals the truncated admitted mesh as in-memory `.10d` bytes.
3. Independent reader: existing `qualia_core_db::render::compile_10d::decode_10d_mesh` reads those bytes.
4. Decoded vertex/triangle counts must equal the admission report (`report.vertices` / `report.faces`). Mismatch or decode failure is an error: **no success mesh**.

`manifold_ops.rs` already calls `production_simplify::decimate_semantic_admitted`. That wrapper now goes through the seal/decode gate, so no catalog or dispatch edit was required.

Default `mode=decimate` is still geometric QEM in `manifold_ops` (`manifold_simplify_decimate` → `cg::decimate_qem`). This module remains `mode=semantic|admitted` only (`is_semantic_mode`).

Empty mesh, impossible `target_triangles`, and capacity exhaustion still return `Err` before any `.10d` is compiled.

## Tests (source-authored, cargo unrun this wave)

Previous 7 tests kept. Added:

- `admitted_success_is_independently_decoded_from_compiled_10d` — cube target-8, then `decode_10d_mesh` on the produced bytes; counts must match admission, decoded mesh, and truncated buffers; faces ≤ 8.
- `independent_decode_10d_mesh_rejects_non_container_and_count_mismatch` — garbage bytes fail through `decode_10d_mesh`; count mismatch is `unsatisfied_constraint`.

Prior wave: 7 `production_simplify` tests passed in `target-production-core` (`hypermedia-semantic-03.log`). This remaining slice was **not** re-run.

## Catalog

No catalog change required. `hypermedia.manifold.simplify` already documents `mode=decimate|remesh|anisotropic|semantic` and semantic/admitted → `decimate_semantic_admitted_3` + P07 `CollapseTransfer`. Coordinator owns catalogs.

## Honesty / remaining gaps

| Item | Status |
|---|---|
| Independent `decode_10d_mesh` of produced in-memory `.10d` | source wired |
| Default `mode=decimate` remains QEM | unchanged in `manifold_ops` |
| Empty / impossible targets produce no success mesh | unchanged, still pre-seal |
| Cargo / lib tests this wave | **not executed** (lease) |
| Public morphism via cached `synth.exe` | **not invoked** (lease) |
| Written on-disk `.10d` from this agent | none; in-memory compiled bytes only |
| Three repeated asset/source builds, `.hmc` packaging | not this remaining slice |
| Front/side/three-quarter, wireframe/normals, error overlays | not executed |
| Inventory 150 agreement | not re-verified here |
| P21 qualification | **no**. Source is not execution. Execution is not qualification. |
| P00-only acceptance | unchanged |

P21 packet acceptance still depends on a later consolidated cargo pass and independent review. This report is a source handoff, not qualification.
