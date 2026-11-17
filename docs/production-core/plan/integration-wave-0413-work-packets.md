# Integration wave 0.0.41.3 — work packets

Date: 17 November 2026. Coordinator: Grok. Machine register:
`integration-wave-0413-work-packets.json`. Prompts: `plan/i0413/I*.md`.
Leases: `../integration-wave-0413-leases.md`.

These packets land the intended WebGPU-preferred / WebGL2-fallback benefit
on github `0.0.41.3` roots after production-core and webgl2 merges. They
are not P00–P23 and they do not accept those packets.

P00 remains the only accepted production-core packet. Do not invent
qualification. Isolation `none`. Source packets do not run cargo.

## I1 receipt (do not dispatch)

Portal wasm32 check is green.

- Log: `C:\github\qualiaDB\target-production-core-wasm\logs\cargo-check-qdb-portal-wasm-i1.log`
- `Checking getrandom v0.2.17`, `Compiling getrandom v0.4.3`, `Compiling getrandom v0.3.3`, `Finished`, `EXIT=0`
- Durable cfg: QualiaDB `.cargo/config.toml`, `.cargo/production-core.toml`, workspace `getrandom` `wasm_js`; Etalon `.cargo/config.toml` wasm rustflags
- Do not set exclusive `RUSTFLAGS=-C debuginfo=0` (it drops the cfg)

## Dispatch DAG

```
I1 executed
    ├── S1 parallel, no cargo (max 3): I2A, I3, I4E
    ├── S2: coordinator I0; then I2B (website) and I4Q (gpu_timing)
    ├── S3 cargo: I5 website retarget + WASM rebuild
    ├── S4: I6 create-only receipts
    └── S5: I7 independent review (different agent, read-only)
```

Spawn with `isolation: none` and `cwd` = the packet repo. Paste the matching
`plan/i0413/I*.md` as the full prompt.

## Packet index

| ID | Title | Repo | Cargo | Spawnable | Depends | Status |
|---|---|---|---|---|---|---|
| I1 | Portal wasm getrandom cfg | QualiaDB+Etalon | coordinator (done) | no | — | executed EXIT=0 |
| I0 | Etalon pin `143a1d7538ceae867ceab1dc7e43126bc688752f` | QDB/HM/Synth manifests | coordinator | no (coordinator) | I1 | ready |
| I2A | WebGPU then GL retry + expose live backend | QualiaDB | no | yes | I1 | ready |
| I2B | Honest `selected_backend` | website | no | yes | I2A | ready |
| I3 | Use `AdmittedSimplify` fields | Hypermedia | no | yes | — | ready |
| I4E | Etalon GPU-check warnings | Etalon | no | yes | — | ready |
| I4Q | Native unused `set_adapter_info` | QualiaDB | no | yes | I2A | ready |
| I5 | Retarget website off worktree; rebuild WASM | website | yes, this packet | yes | I0,I2A,I2B | blocked |
| I6 | Desktop + Pixel receipts | evidence dir | no | yes | I5 | blocked |
| I7 | Independent review | none | no | yes | I6 | blocked |

## Spawn payload (fill per packet)

```json
{
  "packet_id": "I2A",
  "cwd": "C:\\github\\qualiaDB",
  "isolation": "none",
  "prompt_path": "C:\\github\\synth\\docs\\production-core\\plan\\i0413\\I2A-portal-fallback.md",
  "allowed_write_paths": ["exact paths in the prompt"],
  "cargo": "none",
  "required_return": "agent-report.md under the packet evidence dir"
}
```
