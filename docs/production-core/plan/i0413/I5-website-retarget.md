# Packet I5 — Retarget website to github QualiaDB; rebuild WASM

You are the website retarget implementer. Isolation `none`.
Cwd: `C:\github\webizen-website`. Packet: I5.

## Constraints

- Do not commit unless the coordinator later asks. Do not push.
- Do not kill Synth MCP. Do not rebuild `C:\github\synth\target` or `target-studio`.
- One rustc. Use a dedicated target dir:
  `C:\github\webizen-website\target-production-core-wasm`
  `CARGO_BUILD_JOBS=1`. Do not set exclusive `RUSTFLAGS=-C debuginfo=0`.
  QualiaDB `.cargo/config.toml` already has `getrandom_backend="wasm_js"`.
- P00 is the only accepted packet. Do not mark qualified.
- WebGPU preferred; WebGL2 required fallback.
- After two failed bounded repairs, stop and report.

## Depends

I0 (etalon pin `143a1d7538ceae867ceab1dc7e43126bc688752f` on QualiaDB),
I2A (portal retry + getter), I2B (honest `selected_backend` source).

## Write lease

- `C:\github\webizen-website\native\world-host\Cargo.toml` (and lock if cargo updates it)
- `C:\github\webizen-website\tooling\native-host\receipt.py`
- `C:\github\webizen-website\plans\WEBGL2-MOBILE-HANDOVER.md`
- `C:\github\webizen-website\site\pkg\native-world\` (rebuild after I2B source is honest)
- patch paths that still point at `C:/github/_worktrees/qualiadb-webgl2`

Create-only report:
`C:\github\synth\docs\production-core\evidence\integration-wave-i5\agent-report.md`

## Required change

1. Point `qualia-core-db`, `vibe`, `vibescript-abi`, `imap-proto` at
   `C:/github/qualiaDB/...` (github `0.0.41.3` / HEAD `7296c131a…`), not the
   stale worktree.
2. `receipt.py`: `core=Path('C:/github/qualiaDB/crates/qualia-core-db')`.
   Expected etalon revision `143a1d7538ceae867ceab1dc7e43126bc688752f`
   (not `4c99669` alone and not `1cb112d`). Adjust resolved-crate counts
   only if cargo metadata disagrees; record actual counts, do not invent.
3. Rebuild `site/pkg/native-world/` with the existing native-host recipe.
   Run `python tooling/copy_aaa_review.py --check`.
4. Rewrite `WEBGL2-MOBILE-HANDOVER.md` to current reality: webgl2 worktrees
   are merged into `0.0.41.3`; website consumes github QualiaDB; WebGPU
   preferred with GL fallback; `?webgl2=1` is a probe. Keep Pixel 10 Pro XL
   as the mobile evidence target for I6.

## Return

Packet id, new path-deps, etalon rev asserted, wasm sha256, copy_aaa result, remaining gaps.
