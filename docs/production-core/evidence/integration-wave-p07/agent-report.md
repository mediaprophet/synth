# P07 remaining — collapse provenance (agent report)

Date: 16 November 2026. Agent: p07-provenance. Lease: `attribute_transfer/**` only.

Capability: **present / callable source**. Not executed. Not qualified.

## Objective

Collapse provenance was `not_executed`. Implement a hashable lineage snapshot over existing `CollapseTransfer` original-support rows: live original vertex slots, stable IDs, protected domains, sparse positive-weight support, and ContentId identity (source/target/run/policy plus `committed_collapses`).

## Written paths

| Path | Change |
|---|---|
| `C:\github\qualiaDB\crates\qualia-core-db\src\specialized_libs\computational_geometry\attribute_transfer\provenance.rs` | New. Canonical LE snapshot `QCLN0001`, JSON export with lexical keys, SHA-256 identity. |
| `C:\github\qualiaDB\crates\qualia-core-db\src\specialized_libs\computational_geometry\attribute_transfer\collapse.rs` | Getters only: `run`, `source`, `policy`, `original_count`, `stable_id`, `protected_domain`. No row growth, no commit rewrite. |
| `C:\github\qualiaDB\crates\qualia-core-db\src\specialized_libs\computational_geometry\attribute_transfer\mod.rs` | `mod provenance; pub use provenance::*;` |
| `C:\github\qualiaDB\crates\qualia-core-db\src\specialized_libs\computational_geometry\attribute_transfer\tests.rs` | Identity rows, one lawful collapse, domain-crossing rejected, snapshot hash tracks weight/id. |

Not edited: `computational_geometry/mod.rs`, `production_core_geometry.rs`, `Cargo.*`, decoded_error_ledger, catalogs, production_contracts.

## Snapshot contract

- Schema: `qualia.collapse-lineage/QCLN0001`.
- Identity: SHA-256 of canonical 160-byte LE header plus live-row (56) and support-entry (16) records. CRC is not computed and is not a signature.
- Header field order: magic, live, nnz, `committed_collapses`, reserved-zero, run/source/target/policy digests.
- Each live original slot: `slot`, `stable_id`, domain `(material, subpart, provenance, chart, protected)`, then sparse `(col, positive weight)`.
- Dead slots are omitted. Absence is Null (omission), never a 0.0 weight.
- JSON: compact UTF-8, lexical object keys (`committed_collapses`, `live`, `policy`, `run`, `schema`, `source`, `target`; domain `chart`…`subpart`; support `col`, `weight_le`). Weights are IEEE-754 LE hex so a bit change is exact.
- Resource kind is `Evidence` + `ExternalTyped` schema. This is not a FEMC record and not a surface-to-surface FEMC map. FEMC remains surface-to-tetra only.
- Caller buffers only. Encode/decode never allocate row storage.

Domain crossing remains `TransferError::DomainCrossing` → `ReasonCode::UnsatisfiedConstraint`. Affine/collapse reproduction failure remains `Unsupported("collapse_affine_reproduction")` → `UnsatisfiedConstraint`.

## Authored tests (unrun)

Lease forbids `cargo`. Tests exist in `tests.rs` and were not executed.

- `identity_lineage_snapshot_has_identity_rows` — no collapses → one `(col=i, weight=1.0)` row per original slot; JSON key order; no zero-weight encoding.
- `lawful_collapse_updates_original_support` — one admitted midpoint collapse merges support on the kept slot, drops the removed slot, leaves the third identity row; snapshot `committed_collapses=1`.
- `lineage_domain_crossing_is_rejected_without_commit` — mixed domains fail prepare as `UnsatisfiedConstraint`; sequence stays 0; identity rows unchanged.
- `lineage_snapshot_hash_tracks_weight_and_identity` — hash changes on target id, stable id, tampered byte, and `t=0.25` weights; wrong expected target is `IdentityMismatch`.

## Remaining gaps

- Tests compiled/executed: **no** (no cargo). Packet evidence stays source-only until an allowed test run.
- Geometry example / receipts do not emit this snapshot (`production_core_geometry.rs` is outside this lease).
- Combined P06 admission + P07 snapshot on a real simplify run: not wired, not executed.
- Broader P07 acceptance still open: virtual-work transpose, conservation claims, checker/UV/tangent/skin encode-decode qualification. Those adapters already exist; they are not this provenance remaining item and were not re-qualified here.
- Smooth/MikkTSpace, nonrigid frames, signed/extrapolating maps, >4 skin influences remain explicitly unsupported.
- Qualification campaign item `collapse_provenance_qualification` remains `not_executed`.

## Honesty

Source is not execution. Execution is not qualification. This packet remaining is implemented as callable source with authored unit tests. Do not accept P07.
