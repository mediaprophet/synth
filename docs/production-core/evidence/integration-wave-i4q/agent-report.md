# Packet I4Q — Native unused `set_adapter_info`

Packet: **I4Q**. Date: 17 November 2026.
Workspace: `C:\github\qualiaDB`. Isolation `none`. Source-only; cargo not run.
Status: source implemented. Not compiled. Not qualified. P00 is the only accepted packet.

## Objective

Native GPU check (`--features` default, not wasm) warns `unused set_adapter_info`
because the only caller is `#[cfg(all(target_arch = "wasm32", feature = "portal"))] try_new_async`.
Keep the method. Silence the native-only unused warning. Do not wrap the method in `cfg(wasm32)`.

## Cfg / allow used

```rust
#[cfg_attr(not(all(target_arch = "wasm32", feature = "portal")), allow(dead_code))]
pub fn set_adapter_info(&mut self, info: etalon::AdapterInfo)
```

- Applied on `PassTimings::set_adapter_info` only.
- Method body unchanged. Not wrapped in `cfg(wasm32)`. Wasm portal constructors still compile the setter.
- `adapter_info` field is not allowed: it remains read by public `PortalGpu::gpu_timing_adapter_info()`.
- `gpu_timing_adapter_info` is public on `PortalGpu` and is not dead on native.

Native GPU log named this site as:

```
warning: method `set_adapter_info` is never used
 --> crates\qualia-core-db\src\render\gpu\gpu_timing.rs:114:12
```

(`C:\github\qualiaDB\target-production-core\logs\cargo-check-qdb-0413-gpu.log`, lines 162–173.)

## Paths written

| Path | Role |
|---|---|
| `C:\github\qualiaDB\crates\qualia-core-db\src\render\gpu\gpu_timing.rs` | Native-only `allow(dead_code)` on `set_adapter_info` |
| `C:\github\synth\docs\production-core\evidence\integration-wave-i4q\agent-report.md` | This report |

Not edited: `constructors.rs` (I2A), `portal/mod.rs`, `adapter_info` field, `gpu_timing_adapter_info`, etalon pin, `Cargo.*`.

## Remaining gaps

- Cargo / rustc not run (lease). Coordinator owns the native GPU re-check.
- Native constructors still do not call `set_adapter_info`. The warning is silenced, not given a native caller.
- After QualiaPortal adopt, live backend still depends on I2A stash-time label. This packet does not change that.
- P00 only. Not qualified.
