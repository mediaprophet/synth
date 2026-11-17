# Packet I4Q — Native unused set_adapter_info

You are a source-only implementer on github roots. Isolation `none`.
Cwd: `C:\github\qualiaDB`. Packet: I4Q. Do not run cargo.

## Constraints

- Do not commit, push, kill Synth MCP, or rebuild synth `target` / `target-studio`.
- Do not run cargo. Do not bump etalon pin.
- P00 is the only accepted packet. Do not mark qualified.
- After two failed bounded repairs, stop and report.

## Depends

I2A. Constructors already call `set_adapter_info` on the wasm portal path.
I2A may add a wasm_bindgen getter that reads `gpu_timing_adapter_info()`.
Do not delete `set_adapter_info` or `adapter_info`.

## Write lease (only this file)

- `C:\github\qualiaDB\crates\qualia-core-db\src\render\gpu\gpu_timing.rs`

Create-only report:
`C:\github\synth\docs\production-core\evidence\integration-wave-i4q\agent-report.md`

## Current defect

Native GPU check (`--features` default, not wasm) warns
`unused set_adapter_info` because the only caller is
`#[cfg(all(target_arch = "wasm32", feature = "portal"))] try_new_async`.

## Required change

Keep the method. Silence or cfg the native-only unused warning, for example
`#[cfg_attr(not(all(target_arch = "wasm32", feature = "portal")), allow(dead_code))]`
on `set_adapter_info` and/or `adapter_info` if they remain unread on native.
Do not wrap the method in `cfg(wasm32)` — wasm constructors need it.

Do not edit `constructors.rs` (I2A).

## Return

Packet id, the cfg/allow used, paths written, remaining gaps.
