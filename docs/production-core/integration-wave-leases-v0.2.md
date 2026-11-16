# Integration wave leases v0.2

Date: 16 November 2026. Coordinator: Grok 4.6.

Worktree production-core source was overlaid onto updated `C:\github`
roots. No commits. Optional R01–R08 stay uncommissioned. P00 remains the
only accepted packet. Do not accept P22/P23. Do not invent qualification.

## Root vs worktree (preserved)

| Tree | Root HEAD | Worktree HEAD | Root-only commit kept |
|---|---|---|---|
| qualiaDB | `cb8c731c2` native website scenes + Etalon pin | `7f6057a74` | render/gpu, portal, geospatial |
| etalon | `a326f82` AO/material maps | `48af563` | `etalon-render` AO/mesh/CHANGELOG |
| hypermedia | `fbb7bcb` pin Etalon `a326f82` | `8321792` | published Etalon pin; local QualiaDB path-dep added for admitted driver |
| synth | `54dc28a3` website checkpoint | `0d088846` | website checkpoint files |

Overlay copied production-core modules, examples, docs, scripts, and
`qualiaDB/.cargo/production-core.toml`. Cargo.lock files were not copied.
`target-production-core` was not copied. Hypermedia `Cargo.toml` keeps
`etalon` rev `a326f829…` and uses sibling `qualia-core-db` path.

## Shared files (coordinator only)

- `qualiaDB/.../computational_geometry/mod.rs`
- `etalon/crates/etalon-wgsl-forge/src/lib.rs`
- `hypermedia-production/crates/asset-production-vibe/src/operations_catalog.rs`
- `hypermedia-production/crates/asset-production-vibe/src/ontology/morphisms_03.rs`
- `hypermedia-production/crates/asset-production-vibe/Cargo.toml`
- any `Cargo.lock`, `.cargo/production-core.toml`
- this lease file, `swarm-board-grok.md`, `p23-*.md`

## Wave leases (disjoint; source only; no cargo)

Agents must not run `cargo`, must not rebuild synth `target`/`target-studio`,
must not kill MCP, must not commit. One writer per exact path. Create-only
evidence under a new directory. Absence is Null/Unsupported, never 0.0.

| Agent | Packets | Write paths | Objective |
|---|---|---|---|
| p08-enclosing | P08 remaining | `qualiaDB/.../decoded_error_ledger/**`, `qualiaDB/crates/qualia-core-db/examples/production_core_geometry.rs` | Wire `certify_decoded_output` into the geometry receipt. Whole-surface enclosing bound measured or honest inconclusive. Physical stays NotSupplied unless P10 evidence is passed in. |
| p07-provenance | P07 remaining | `qualiaDB/.../attribute_transfer/**` | Collapse provenance record: original-support rows, stable IDs, protected domains, content identity. Export a hashable lineage snapshot. No silent domain crossing. |
| p20-lifecycle | P20 remaining | `qualiaDB/.../production_lifecycle/**` | Finish activation/rollback/fault accounting. Live device token still does not dispatch GPU work. CPU time cannot be GpuTimestamp. |
| p19-quality | P19 remaining | `etalon/.../production_quality/**` | Invalid fast candidate cannot win; tamper rejected; budget exhaustion keeps qualified baseline; occupancy is admission not timing. |
| p21-simplify | P21 remaining | `hypermedia-production/.../production_simplify.rs` | Independent decode of produced `.10d` via existing `decode_10d_mesh`. Keep default `mode=decimate` as QEM. Do not edit catalogs. |
| p03-decode | P03/P22 harness | `synth/scripts/production_core/**` except do not edit this lease/board; `synth/docs/production-core/corpus/**` only if adding fixtures; evidence only under a new create-only dir | Independent sealed `.10d` decode harness; B0 identity ablation as inspect/CPU; known-bad still skip invocation. Never mark qualified. |

## Honesty

Capability ladder: present / callable / executed / qualified. Source is not
execution. Execution is not qualification. GPU unavailability is
unsupported/skipped. Do not rebuild live synth binaries.
