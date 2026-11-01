# Synth handoff manifest and source audit

**Scope:** read-only inspection of `delivery/github-handoff-2026-11-02/synth/Cargo.toml`, its README/NOTICE/provenance, and `src/` against `apps/synth`. No Cargo command, test, build, or external change was made.

## Findings

- All 12 Rust source files enumerated in `provenance/source-parity.json` match `apps/synth` byte-for-byte in this audit. No unexpected Rust source files were present. The parity record correctly limits its claim to static source parity/TOML parsing and explicitly does not claim standalone compilation.
- The manifest keeps Poet separate, defines a one-member standalone workspace, defaults to no optional features, and gives the two optional capabilities explicit gates (`native-ui`, `audio-device`). Etalon dependencies remain pinned to `072b4d9ceeebd423234359dff1dee8fbe9e62b6e`. The new path dependencies are sibling-layout paths for Ecopunk, QualiaDB, hypermedia-libraries and rust-app-framework, consistent with the README's stated checkout layout. They do not resolve from the current nested handoff staging directory; a standalone checkout must place the sibling repositories as documented.
- The handoff has no root `Cargo.lock`. It preserves the prior Ecopunk lock only under `provenance/`; README accurately says this new workspace is uncompiled and its lock must be resolved/reviewed before a locked build is claimed. Do not present the prior Ecopunk lock as the new standalone lock.
- Qualification wording is appropriately bounded: it distinguishes the embedded Native39/WASM40 evidence from standalone compilation, records that final R7F native/WASM exact replay was not executed, and leaves optional UI/audio unqualified. It also says the wallaby is not cinema-grade/accepted and that film/game/cartoon packages remain incomplete. The preservation and non-completion statements are consistent with this audit.
- The package metadata declares Apache-2.0 and `NOTICE` preserves the project-level attribution/rights limitation. The handoff directory itself contains no `LICENSE` file; include the applicable Apache-2.0 license text if the staged package is intended to be independently distributed as a repository.

## Readiness boundary

No material source-parity or sibling-path rewrite defect was found in this narrow pass. This is not compile, lockfile, integration, or reproducibility qualification. The separate Hypermedia handoff still needs its omitted consumer-smoke/check evidence and `docs/migration5` material incorporated as tracked in the coordinator's pending work; this report does not validate that packet. No commit or push was performed.
