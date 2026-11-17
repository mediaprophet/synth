# Packet I3 — Use AdmittedSimplify unread fields

You are a source-only implementer on github roots. Isolation `none`.
Cwd: `C:\github\hypermedia-production`. Packet: I3. Do not run cargo.

## Constraints

- Do not commit, push, kill Synth MCP, or rebuild synth `target` / `target-studio`.
- Do not run cargo. Do not edit catalogs, `Cargo.toml`, or `operations_catalog.rs`.
- P00 is the only accepted packet. Do not mark qualified.
- After two failed bounded repairs, stop and report.

## Write lease (only this file)

- `C:\github\hypermedia-production\crates\asset-production-vibe\src\production_simplify.rs`

Create-only report:
`C:\github\synth\docs\production-core\evidence\integration-wave-i3\agent-report.md`

## Current defect

`AdmittedSimplify` fields `container_10d`, `admitted`, and `decoded` are written
and used in `#[cfg(test)]`, but `decimate_semantic_admitted` returns only
`(sealed.vertices, sealed.triangles)`, so the lib warns:

```
warning: fields `container_10d`, `admitted`, and `decoded` are never read
 --> crates/asset-production-vibe/src/production_simplify.rs:276
```

Log: `C:\github\hypermedia-production\target-production-core\logs\cargo-check-hm-0413.log`.

## Required change

Use those fields on the production path. Preferred: after seal, verify
`decoded` counts match `admitted` (and that `container_10d` is non-empty)
inside `decimate_semantic_admitted_10d` or `decimate_semantic_admitted`
before returning vertices/triangles. Fail closed with the existing
`semantic simplify failed:` error style if the sealed container does not
decode to the admitted counts.

Do not `#[allow(dead_code)]` as the only fix. Do not change default
`mode=decimate` QEM. Do not invoke cached `synth.exe`. Leave
`video-h264-codec` / `radiance-color-grading` warnings alone.

## Return

Packet id, how the three fields are now read, paths written, remaining gaps.
