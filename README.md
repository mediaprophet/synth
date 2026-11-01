# Synth

Synth is the separate agent-driven asset-authoring application. Poet remains a distinct product and is not replaced by this repository. QualiaDB owns the semantic/runtime state; Synth consumes existing engine services and shared hypermedia libraries. P64 remains future work.

## Preserved implementation and actual qualification

This checkout copies `ecopunk/apps/synth` without deleting or modifying the original. Rust source bytes are unchanged. Only the manifest's relative dependency paths/workspace and this README are adapted. Original manifest/README and the previously resolved Ecopunk lockfile are retained under `provenance`.

The embedded headless producer built successfully (Native39) and generated, saved, strictly reopened and exported the R7F wallaby package (native production06). Corrected WASM40 also built. Final R7F native/WASM exact replay was not executed before this handoff; the preceding R7E check failed on target-dependent accounting, and that failure and repair remain recorded. Optional `native-ui` and `audio-device` capabilities are not qualified by the headless build. The standalone layout has not been compiled. Its final Cargo lock must be resolved and reviewed in the new workspace before a locked build is claimed.

The current wallaby is **not cinema-grade or accepted**. Film/game/cartoon packages remain incomplete. See the accompanying engine handoff and existing complete dossier. A build, a source extraction or an HMC file is not model acceptance.

## Checkout layout

Clone/recover these sibling directories under the same parent: `synth`, `ecopunk`, `qualiaDB`, `hypermedia-libraries`, and `rust-app-framework`. Etalon remains the pinned dependency in the workspace manifest. The accompanying per-repository packets identify exact base commits and uncommitted source dependencies; repository HEAD alone does not include those changes. Apply/review and publish engine changes in their owning repositories before declaring a fresh-clone build reproducible.

No QualiaDB/Etalon/hypermedia engine is copied into Synth. `ecopunk-terrain` remains in Ecopunk and is consumed as a sibling dependency. `native-ui` uses the controlled Aether fork; `audio-device` uses `pcm-device-output`.

## Commands after dependency/lock qualification

```powershell
cargo build --manifest-path C:/github/synth/Cargo.toml --bin synth --locked
C:/github/synth/target/debug/synth.exe produce --source C:/github/ecopunk/assets/production-studies/swamp-wallaby-review-r1/authoring/swamp-wallaby-r3.vibe --source C:/github/ecopunk/crates/ecopunk-terrain/src/animal_motion/study.vibe --out C:/path/to/new-candidate.hmc
```

These are future operator commands, not executions performed by this handoff. Keep Cargo serial and provide bounded target/disk budgets. Review a candidate through the existing asset-review URL before acceptance. Retain at least twenty relevant real subject photographs and actual output comparison; generated pictures do not replace biological references.

The original detailed host/MCP/audio contracts and limitations remain in `provenance/original-embedded-README.md`. Existing Apache-2.0 package declarations and dependency/source notices are preserved; this extraction does not relicense sibling repositories or third-party work.

## Dependency commits

The source handoff is pinned in [published-dependencies.json](provenance/published-dependencies.json). Existing engine commits are now published on the review branches listed below; their default branches do not include these updates. Clone `hypermedia-production` into a directory named `hypermedia-libraries`. A fresh-clone build remains unqualified.

- [qualiaDB 392e53343a3b](https://github.com/mediaprophet/qualiaDB/commit/392e53343a3b3f5b4cade28c4816e4e17a0aef6f) — published review commit.
- [ecopunk c65d76df9ee4](https://github.com/mediaprophet/ecopunk/commit/c65d76df9ee47cc33133fdeb90f5dab5f0aa383d) — published review commit.
- [rust-app-framework ed61a5673965](https://github.com/mediaprophet/rust-app-framework/commit/ed61a567396552fc2076835885ded7ce56a797bb) — published review commit.
- [hypermedia-production 55925b09af4b](https://github.com/mediaprophet/hypermedia-production/commit/55925b09af4b3b174b0df77cc2a8427e04bc3822)
