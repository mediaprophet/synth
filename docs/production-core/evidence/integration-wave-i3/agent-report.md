# Packet I3 — agent report

Packet: **I3**. Date: 17 November 2026.
Workspace: `C:\github\hypermedia-production`. Isolation `none`. Cargo: not run.
Status: source change landed. Not accepted. Not qualified. P00 remains the only accepted packet.

## Objective

`AdmittedSimplify` fields `container_10d`, `admitted`, and `decoded` were written by
`decimate_semantic_admitted_10d` and read only under `#[cfg(test)]`. Production
`decimate_semantic_admitted` returned `(sealed.vertices, sealed.triangles)` and
left the three fields unread, which is the
`fields container_10d, admitted, and decoded are never read` warning
in `target-production-core/logs/cargo-check-hm-0413.log` (line 496,
`production_simplify.rs:276`).

## How the three fields are now read

Production-path helper `verify_sealed_container` reads all three fields on
`&AdmittedSimplify`:

- `container_10d` — fail closed if empty (`semantic simplify failed: invalid_input: empty sealed container_10d`).
- `decoded` and `admitted` — fail closed unless vertex/face counts match, using the existing
  `verify_decoded_matches_admission` message
  (`semantic simplify failed: unsatisfied_constraint: independent decode_10d_mesh … != admission …`).

Call sites (always compiled, not test-only):

1. `decimate_semantic_admitted_10d` — after constructing `AdmittedSimplify` from
   `seal_admitted_10d`, before returning the sealed value.
2. `decimate_semantic_admitted` — after `decimate_semantic_admitted_10d`, before
   returning vertices/triangles to `manifold_ops::manifold_simplify_semantic`.

Default `mode=decimate` QEM is unchanged. No `#[allow(dead_code)]`. Catalogs,
`Cargo.toml`, and `operations_catalog.rs` were not edited.

## Paths written

| Path | Role |
|---|---|
| `C:\github\hypermedia-production\crates\asset-production-vibe\src\production_simplify.rs` | Lease. Production-path reads of the three fields. |
| `C:\github\synth\docs\production-core\evidence\integration-wave-i3\agent-report.md` | This create-only report. |

No other files. No commit, push, cargo, synth rebuild, or cached `synth.exe`.

## Remaining gaps

- Cargo was not run. The `never read` warning is addressed in source only; rustc
  clearance is unverified until an allowed check.
- Independent decode still happens inside `seal_admitted_10d`. The new helper
  re-reads the stored counts and emptiness; it does not re-decode bytes.
- Live `hypermedia.manifold.simplify` with `mode=semantic|admitted` was not
  executed. `manifold_ops.rs` is outside the write lease and still consumes only
  vertices/triangles after the wrapper check.
- Authored unit tests for empty `container_10d` and count mismatch were not
  executed (no cargo).
- Packet I3 is not accepted. This is not production-core qualification.
  `video-h264-codec` / `radiance-color-grading` warnings were left alone.
