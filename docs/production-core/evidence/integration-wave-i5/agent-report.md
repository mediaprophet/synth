# Packet I5 — agent report

Packet: **I5**. Date: 17 November 2026.
Workspace: `C:\github\webizen-website`. Isolation `none`.
Status: path-deps retargeted; WASM rebuilt; receipt written; `copy_aaa_review.py --check` passed.
Not qualified. P00 is the only accepted packet.

## Objective

Retarget `native/world-host` from the stale `C:/github/_worktrees/qualiadb-webgl2` worktree to github QualiaDB, assert Etalon `143a1d7538ceae867ceab1dc7e43126bc688752f`, rebuild `site/pkg/native-world/`, and rewrite the WebGL2 handover to current reality.

## Path-deps (new)

`native/world-host/Cargo.toml` (lock rewritten by `cargo metadata --offline`):

| Crate | Path |
|---|---|
| `qualia-core-db` | `C:/github/qualiaDB/crates/qualia-core-db` (`default-features = false`, `features = ["portal"]`) |
| `vibe` | `C:/github/qualiaDB/crates/vibe` |
| `imap-proto` | `C:/github/qualiaDB/vendor/imap-proto` (`[patch.crates-io]`) |
| `vibescript-abi` | `C:/github/qualiaDB/crates/vibescript-abi` (`[patch."https://github.com/mediaprophet/vibe-script.git"]`) |

Github QualiaDB HEAD compiled against: `7296c131a410831a143165999b329ba74abd123e` (`merge(webgl2): fold webgl2-portal into 0.0.41.3`). Working tree also included I2A (`portal_gpu_backend`) and I4Q (`set_adapter_info` allow). No remaining `_worktrees/qualiadb-webgl2` references in `Cargo.toml`, `Cargo.lock`, `receipt.py`, or the handover.

## Etalon rev asserted

`143a1d7538ceae867ceab1dc7e43126bc688752f` (0.0.41.3).

Measured with `cargo metadata --offline --format-version 1 --filter-platform wasm32-unknown-unknown` from `native/world-host` (2026-11-17). Not invented:

- resolved wasm provider crates (`etalon*` / `naga` / `naga-types`): **24**, all `source` ending `#143a1d7538ceae867ceab1dc7e43126bc688752f`
- `Cargo.lock` `etalon.git` providers: **31**, same pin

`receipt.py` now fingerprints `core=Path('C:/github/qualiaDB/crates/qualia-core-db')` and asserts those counts + pin.

## WASM rebuild

Dedicated target dir: `C:\github\webizen-website\target-production-core-wasm`.
`CARGO_BUILD_JOBS=1`. Did not use `tooling/native-host/build.ps1`. Did not write `C:\github\qualiaDB\target`, `C:\github\synth\target`, or `target-studio`.

1. First `cargo build --target wasm32-unknown-unknown --release --offline --locked` with RUSTFLAGS unset failed: `getrandom 0.3.3` `compile_error!` (wasm32-unknown-unknown needs the `wasm_js` cfg; QualiaDB `.cargo/config.toml` does not apply to this crate).
2. Bounded repair 1: `RUSTFLAGS='--cfg getrandom_backend="wasm_js"'`. Rebuild succeeded in 36m 28s (`rustc 1.99.1`).
3. wasm-bindgen **0.2.129** (`C:/Users/Hp/AppData/Local/.wasm-pack/wasm-bindgen-06a9b89f25ab66ba/wasm-bindgen.exe`) `--target web --out-name webizen_world`.
4. `python tooling/native-host/receipt.py --output site/pkg/native-world --bindgen <that exe>`.

### WASM identity

| Field | Value |
|---|---|
| bytes | 21,614,966 |
| sha256 | `0086f172d33bbf726b437a4a7e66282eda370b5c6e8244ee79544c6f64dd72f8` |
| loader sha256 | `ea4c41ef486e44e4e1d9219714edc84b5687a3949bcde2721aa6cd01888f4d63` |
| cargo_lock sha256 | `d6b42f09e19cd33ab530977b516520b640ec5f52e23fdc5862bf3e81b991d95a` |
| QualiaDB revision | `7296c131a410831a143165999b329ba74abd123e` |
| QualiaDB tracked dirty | `gpu_timing.rs`, `constructors.rs`, `capabilities.rs`, `portal/mod.rs` |
| compiler | `rustc 1.99.1 (58cf8f9c3 2026-10-13)` |

Served file: `site/pkg/native-world/runtime-identity.json`.

## copy_aaa result

```
python tooling/copy_aaa_review.py --check
{"preflight": true, "files": 2161, "bytes": 438965501}
```

Exit 0.

## Handover

`plans/WEBGL2-MOBILE-HANDOVER.md` rewritten to current reality:

- WebGL2 worktrees merged into QualiaDB / Etalon `0.0.41.3`.
- Website consumes github QualiaDB.
- WebGPU preferred; one GL retry; `?webgl2=1` is a force-GL probe.
- Honest `selected_backend` from `portal_gpu_backend()` (I2A/I2B).
- Pixel 10 Pro XL Chrome remains the **I6** mobile evidence target.

## Paths written

| Path | Role |
|---|---|
| `C:\github\webizen-website\native\world-host\Cargo.toml` | github QualiaDB path-deps |
| `C:\github\webizen-website\native\world-host\Cargo.lock` | etalon `143a1d7`; path-deps |
| `C:\github\webizen-website\tooling\native-host\receipt.py` | core path + etalon pin + measured counts |
| `C:\github\webizen-website\plans\WEBGL2-MOBILE-HANDOVER.md` | current reality |
| `C:\github\webizen-website\site\pkg\native-world\` | rebuilt WASM + identity |
| `C:\github\synth\docs\production-core\evidence\integration-wave-i5\agent-report.md` | this report |

Not edited: QualiaDB sources, synth `target` / `target-studio`, `C:\github\qualiaDB\target`, `tooling/native-host/build.ps1`. Not committed. Not pushed.

## Remaining gaps

- Packet I5 is **not qualified**. P00 only. `receipt.py` still writes a local `qualified/<identity>/` directory (pre-existing behaviour); that is not packet qualification.
- I6 (Pixel 10 Pro XL Chrome) has not run. Live WebGL2 render evidence is still pending.
- QualiaDB working tree is dirty (I0 pin, I2A getter, I4Q allow). Identity records that honestly. A clean QualiaDB commit of those files is coordinator work.
- `imap-proto` path patch is unused in the wasm graph (`Cargo.lock` `[[patch.unused]]`). Warning only.
- `tooling/native-host/build.ps1` still points `CARGO_TARGET_DIR` at `C:/github/qualiaDB/target`. Do not use it.
- world-host has no `.cargo/config.toml`; wasm builds need `RUSTFLAGS` with `--cfg getrandom_backend="wasm_js"` (repair 1). Exclusive RUSTFLAGS must keep that cfg.
- Default boot / `?webgl2=1` probe / Pixel 10 Pro XL evidence belong to I6, not this packet.
