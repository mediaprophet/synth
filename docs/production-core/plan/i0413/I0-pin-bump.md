# Packet I0 — Coordinator only: bump etalon git pin

Not a subagent packet. Coordinator owns `Cargo.toml` / `Cargo.lock`.

## Depends

I1 green (portal wasm EXIT=0). Log:
`C:\github\qualiaDB\target-production-core-wasm\logs\cargo-check-qdb-portal-wasm-i1.log`.

## Pin

Replace QualiaDB workspace etalon / naga `rev` currently
`1cb112dd59de44330f981f6b2d3195ffa2e8514d` with
`143a1d7538ceae867ceab1dc7e43126bc688752f` (etalon `0.0.41.3`, includes
`4c99669` SSBO-align skip and Forge production-core).

Hypermedia still has `a326f82` on several crates. After QualiaDB pin, bring
Hypermedia/Synth git-dep graphs to the same rev where they pin etalon.
Do not pin `1cb112d`. Do not checkout `origin/0.0.41.2`.

Sibling overlay `.cargo/production-core.toml` already paths to
`C:\github\etalon` at this SHA; I0 makes git-dep consumers match.

## Cargo

Coordinator only, `--offline`, `-j 1`, `CARGO_TARGET_DIR` per repo
`target-production-core`. Never synth `target` / `target-studio`.
Do not set exclusive `RUSTFLAGS=-C debuginfo=0`.

## Do not

Commit unless the user asks. Mark qualified. Spawn a subagent for lock edits.
