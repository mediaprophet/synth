# Integration wave 0.0.41.3 — subagent leases

Date: 17 November 2026. Coordinator: Grok. Packets:
`plan/integration-wave-0413-work-packets.md`.

P00 remains the only accepted packet. Do not invent qualification. No
commits. No MCP kill. No synth `target` / `target-studio` rebuild.
R01–R08 uncommissioned. One rustc. Isolation `none` on github roots.

I1 is executed (portal wasm check EXIT=0). I1 rustc is free.

## Coordinator only

- QualiaDB / Hypermedia / Synth etalon `rev` + `Cargo.lock` (I0)
- `.cargo/production-core.toml` and QualiaDB/Etalon wasm rustflags (I1, landed)
- this lease file, `swarm-board-grok.md`, `integration-wave-0413-swarm-plan.md`
- cargo into `target-production-core` / `target-production-core-wasm`

## Disjoint implementers (source; no cargo)

| Agent | Packet | Write paths | Depends | Spawn |
|---|---|---|---|---|
| i2a-portal | I2A | QualiaDB `constructors.rs`, `capabilities.rs`, `portal/mod.rs` only `portal_init_webgpu` / new backend getter | I1 | spawned `01a1e734-177c-7480-b176-a38fe5e1dc5d` |
| i3-simplify | I3 | Hypermedia `production_simplify.rs` only | none | spawned `01a1e734-177c-7480-b176-a3ae6d9df16d` |
| i4e-warnings | I4E | Etalon `acoustic_fdtd.rs`, `worker_steward.rs`, `gemv.rs` | none | spawned `01a1e734-177c-7480-b176-a3ba542c005e` |
| i2b-site | I2B | Website `graphics.rs`; `boot.rs` only if it must consume the new getter | I2A | S2 |
| i4q-timing | I4Q | QualiaDB `gpu_timing.rs` only | I2A | S2 |

## Serial cargo / evidence

| Agent | Packet | Write paths | Depends | Spawn |
|---|---|---|---|---|
| coordinator | I0 | QualiaDB/Hypermedia/Synth `Cargo.toml` + locks; etalon rev `143a1d7538ceae867ceab1dc7e43126bc688752f` | I1 | S2 |
| i5-retarget | I5 | Website `native/world-host/Cargo.toml`, `receipt.py`, handover, `site/pkg/native-world/` | I0, I2A, I2B | S3 |
| i6-evidence | I6 | create-only `synth/docs/production-core/evidence/integration-wave-i6/` | I5 | S4 |
| i7-review | I7 | none (read-only) | I6 | S5 |
