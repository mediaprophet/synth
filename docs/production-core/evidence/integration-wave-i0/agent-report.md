# Packet I0 — coordinator pin bump

Packet: **I0**. Date: 17 November 2026. Coordinator. Not a subagent.
Status: manifests and locks rewritten. Cargo check not run (I5 holds the rustc slot).
Not qualified. P00 remains the only accepted packet.

## Pin

`143a1d7538ceae867ceab1dc7e43126bc688752f` (etalon `0.0.41.3`, includes `4c99669`).
Fetched into cargo git db from local `C:\github\etalon` (offline). Did not checkout
`origin/0.0.41.2`.

## Paths written

- `C:\github\qualiaDB\Cargo.toml` — 19 workspace etalon/naga revs `1cb112d` → `143a1d7`
- `C:\github\qualiaDB\Cargo.lock` — etalon packages now git source `rev=143a1d7`
- `C:\github\qualiaDB\tools\accelerator-harness\Cargo.toml`
- `C:\github\qualiaDB\crates\qualia-etalon-sensing-bridge\Cargo.toml`
- Hypermedia crate `Cargo.toml` files (15 rev replacements `a326f82` → `143a1d7`)
- `C:\github\hypermedia-production\Cargo.lock` — 29 `143a1d7` sources; `1cb112d` = 0; `a326f82` = 0

Synth does not git-pin etalon (it pins QualiaDB / Hypermedia). Left unchanged.

## Remaining gaps

- QualiaDB/Hypermedia `cargo check` of the new git pin not executed this wave slot.
- Sibling overlay `.cargo/production-core.toml` still path-patches to `C:\github\etalon`.
- Website still path-deps `_worktrees/qualiadb-webgl2` until I5.
