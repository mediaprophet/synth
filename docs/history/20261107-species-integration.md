# Production integration validation — 7 November 2026

## Scope and source

Native headless wallaby candidate generation, retained Vibe provider expression,
Q42/HMC packaging, strict fresh reopen, authored pose exports and native PNG review.
The source pins are in `provenance/current-dependencies.json`; the actual resolved
package graph is in `Cargo.lock`. Rust: `rustc 1.95.0 (59807616e 2026-04-14)`;
host: Windows x86_64 MSVC. Cargo ran serially with debug info and incremental
compilation disabled for the development build.

QualiaDB advanced from `a88d573d6` to `398f88b52` during the initial build. The
six-file diff was inspected: it changes CI/tooling configuration, not the Rust
engine source. The final recorded revision and preparation checks use `398f88b52`.
Sibling QualiaDB, Hypermedia and Aether working trees were clean when inspected.

## Executed checks

- Standalone locked native Synth build passed.
- `cargo test --locked --offline -p synth --lib -j 1`: **3 passed, 0 failed**.
  Checks cover all eleven provider registrations, failure on a missing provider,
  SDK-built five-field parity through actual Etalon dispatch and receipts, and
  new render/NQuin operations evaluated by the production species source session.
- Runtime dependency inspection found one `vibescript-abi` 0.0.41 and one
  `qualia-core-db` 0.0.41 in the headless graph. Its Etalon/etalon-vibe revision is
  `22053f61586c31c633696d0736f6e26d30971707`. Older Etalon entries in the lockfile
  belong to optional UI dependencies, not this tested headless graph.
- `synth capabilities` returned **11 providers / 37 operations**, with a passing
  SDK parity probe using nonzero metadata and non-simulated pure dispatch.
- Initial workflow run `target/production-tests/sdk-20261107-r1` passed generation,
  review export, exact fresh reopen, three native PNG views, output digest checks,
  refusal to overwrite an existing candidate and rejection of a truncated HMC.
- Expanded run `target/production-tests/sdk-20261107-r2` passed those checks plus
  verification of all 24 indexed review files, ten motion samples, distinct poses
  in both clips, and three additional native renders of hop sample 4 at 0.9 s.
  Both runs used the same binary and lockfile. The final result is also retained
  in `provenance/production-validation-20261107.json`.
- PowerShell script syntax validation and `git diff --check` passed.

The fixture retains `render.spatial.skeletal_lerp_joint(...)` in the authored coat
length expression. It is evaluated from source without a per-script Rust rebuild,
then replayed on fresh reopen. The original Ecopunk fixture is preserved.

Final candidate (r2): **10,472,513 bytes**, SHA-256
`a9e9ce84181f1ef9b477e53b6f4d6e37b319b01c3953a279b6dc40e5f74d46e4`.
The review export contains 25 files, including the editable master, copied HMC,
package projections and ten sampled poses from two authored motion clips.
The master preview reports 36,221 vertices and 60,578 triangles. Three 2048×2048
PNG views of each of the master and sampled hop pose were produced through the
native `webizen-render` / PortalGpu path (six PNGs in the final run).
This receipt does not identify/qualify a particular adapter or backend.

The master three-quarter and side views and the sampled hop side view were
visually inspected. They contain framed, nonblank geometry and the sampled hop
visibly differs from the rest pose. The inherited r3 asset has segmented joints and basic
surface detail; this test does not establish biological or cinema-quality acceptance.

## Reopen identity versus independent generation

An additional cross-run byte-identity check **failed**: r1 and r2 HMC hashes
differ. Comparison of their package resource indexes isolated the difference to
`resources/knowledge/authored-model-r2-215cc8bb69746048e719deed2d5e326ac19a0e96b27491f6b56e83ed430aeff8.q42`.
All other embedded resource digests match. The source fixture and master manifest
also match; both master `.10d` files have SHA-256
`1d83009c6136beaff77ef9caaffdb16b226b0478d88addf21f8bba25c0335ec1`.

The production knowledge writer calls QualiaDB's `UnifiedVolumeBuilder`, whose
`push_block` and `finish_to_bytes` record wall-clock timestamps in Merkle-DAG
commits and the assertion header respectively (`q42_volume.rs`, lines 931 and
990 at the recorded revision). This makes independent byte-identical generation
unavailable with the current writer. No provenance timestamps were suppressed or
rewritten. The observed resource difference is consistent with this behavior;
the embedded Q42 bytes were not independently decoded and semantically compared.
Each run's strict fresh reopen preserved its own complete HMC bytes exactly.

## Failures encountered and corrections

The initial standalone manifest could not resolve the old sibling paths. Synth
now uses the actual local checkout names and the current Aether package names.
The pinned Ecopunk production source is prepared with a sparse checkout, compiled
through a small manifest adapter, and kept separate from Synth's implementation.

The first complete dependency build reached Ecopunk but failed on 27 embedded
file lookups: old QualiaDB paths and omitted catalogue/terrain fixtures. The
preparation script now includes those fixtures. The checked-in workspace-path
patch changes only source fingerprint paths and manifest/lock embedding; geometry
and motion algorithms are unchanged. Asset fingerprints now use the current
Qualia source and Synth's actual dependency manifest/lock instead of historical
Ecopunk build records. Unexpected changes to the prepared source are rejected.

The successful build retains warnings about unused imports/variables in Ecopunk,
feature-dependent fields/mutability in Synth and an unused imap-proto patch. It is
not a warning-free build. The independent-generation identity limitation above
remains open; the workflow result covers exact reopen, not cross-run byte identity.

## Reproduction and limits

Run `scripts/prepare-dependencies.ps1`,
`cargo test --locked -p synth --lib -j 1`, and `scripts/test-production.ps1`.
Every workflow run uses a new output directory and retains logs, dependency pins,
source fixture, HMCs, review exports, preview receipts and result hashes.

This qualifies the existing native headless wallaby workflow with current Vibe
providers. Optional live native UI, audio-device playback/bank loading,
authenticated agent-host transactions, WASM parity and arbitrary asset profiles
were not exercised. The original film/game/cartoon deliverables remain incomplete
and unaccepted. Scientific, rights, reference-image and complete model-dossier
review are not replaced by build or workflow success. No source was committed,
pushed or published by this task.
