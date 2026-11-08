# General-purpose Synth validation — 7 November 2026

## Architecture and scope

Synth's production library and authenticated host are now asset-neutral. The
public `AssetProducer` interface accepts typed Vibe values and input resources;
the common pipeline owns source history, validation/promotion, resource budgets,
provenance binding, HMC packaging and strict reopen. The host and agent operate
on this common session, without species, rig, timeline or audio-device types.

Ecopunk was removed from Synth's manifest and resolved graph. The species-only
production/export/corpus/audio adapters were removed from active source. There
is no wallaby feature or alternative wallaby host. The optional monitor displays
producer/resources; the optional `.10d` preview remains a format consumer.

No Poet application, QualiaDB, Etalon, Aether or Hypermedia source was modified.
QualiaDB authority APIs retain their upstream `poet_host` namespace; using those
shared owners does not adopt Poet's application or document model.

## Runtime pins

- QualiaDB 0.0.41: `f41aacf9ba6c754339cd5dbc0910151725c16eaa`.
- Etalon: `969f32fc745dcffe89d996c1b4b96f285839e946`.
- Hypermedia: `a46b4e2ba5a9d57fc1a68e20ae87ce4bc2d076cf`.
- One shared `vibescript-abi` 0.0.41 in the default runtime graph.
- Rust `1.95.0 (59807616e 2026-04-14)`, Windows x86_64 MSVC.

The QualiaDB checkout advanced during this work. Its first update changed the
Etalon pin for the macOS metal-lane fix; the next changed two CI workflow lines.
The dependency lock was refreshed to the new Etalon pin. Final source pins are
recorded in `provenance/current-dependencies.json`. `target/general-runtime-dependencies.txt`
records the earlier architecture-only graph. The current production graph is
`target/assets-runtime-dependencies.txt`: it includes the shared Hypermedia
libraries and one unified VibeScript ABI, without Ecopunk or the Poet application.

## Shared-library production verification

The default build now binds 18 production operations through Vibe's host ABI.
`examples/production.vibe` generates nine outputs: SVG, three `.10d` meshes,
two rendered PNG frames, processed and mask PNGs, and a mono PCM16 WAV. Etalon
provider dispatch supplies a parameter; PCM conversion also uses Etalon through
Qualia Audio. Geometry, image, mask, rendering and timing operations reuse their
existing QualiaDB and Hypermedia owners.

`cargo test --locked --offline -p synth --lib --tests -j 1` passed all 13 tests
in `target/assets-tests-final.stdout.log`, including the final tick-overflow and
audio conversion guards. The three new integration tests cover
actual decoded pixel values, changed frame/effect output, mesh indices and finite
bounds, closed extrusion topology, frame displacement, audio sample count/rate/amplitude,
SVG output, retained PNG processing,
exact replay, edits, cancellation, operation bounds, unknown capabilities and
artifact rollback. The authenticated-host test now produces real media, edits
generated SVG, audits save/reopen, and submits a generated PNG through the agent.
Agent inspection includes the production operation catalogue and argument help.

The first integration run caught an incorrect ordering of `requires` before
`import` in the example. Correcting the source preamble resolved that failure.
The shared libraries compiled without source changes.

`scripts/test-asset-operations.ps1 -SkipBuild -OutputDirectory
C:/Projects/synth/target/production-tests/assets-20261107-r2` passed source
production, nine-file export, fresh strict reopen, independent identical
generation, retained PNG decoding/encoding and strict replay of the input-based
package. The generated frame and morphological border mask were visually checked.
The earlier generic file/overwrite/corruption workflow also passed again in
`target/production-tests/general-20261107-r2`.

Asset package SHA-256:
`4090d399803e676f24c96eed743313b3e0e0e837e199998bf8c3c4a22337a130`.
Per-file hashes, byte sizes, tested binary and lock hashes are recorded in
`provenance/shared-library-production-validation-20261107.json`.
The exported operation catalogue is `provenance/asset-operations-20261107.json`.
See `asset-operations.md` for exact format contracts and bounds.

The optional preview build also passed (`target/assets-preview-build.stderr.log`).
The three generated `.10d` files were loaded by the existing native offscreen
renderer, producing nine 2048×2048 PNG views. Three-quarter views of the extrusion,
revolution and loft were visually inspected. Receipts record mesh counts, bounds,
source digests, camera projection and output hashes; consolidated evidence is in
`provenance/native-asset-preview-validation-20261107.json`. These meshes use the
renderer-default material and derived normals; this integration does not author
UVs, material graphs or custom normals. Native previews are under
`target/production-tests/assets-20261107-r1/previews`.

## Earlier architecture-refactor tests

`cargo test --locked --offline -p synth --lib --tests -j 1` passed:

- Seven library tests: SDK/provider dispatch; missing-provider failure; multiple
  resource types and edits; custom producer/version matching; failed-edit and
  cancellation rollback; forged/truncated package rejection; source/evaluation/
  resource/package budgets and separate setup cells.
- Two external-consumer tests: SVG/JSON/Etalon-derived binary production, and an
  independently registered procedural PPM raster producer. Both use the public
  API to author, edit, package and strictly reopen unrelated asset content.
- One authenticated-host test using explicit signed software-agent test identities:
  binding rejection, inspection, source reading, failed/successful edits, stale
  contexts, audited create-only save, strict reopen and revocation.

The default native build and these ten tests passed without compiler warnings in
`target/general-tests-final.stderr.log`. The authenticated-host test was then
extended to exercise agent JSON inspect/read/submit calls, stale token rejection
and the resulting generic resource inventory. Its final targeted rerun passed
(`target/general-agent-test.stdout.log`).

`cargo check --locked --features native-ui -p synth -j 1` passed for the asset-neutral
native monitor (`target/general-ui-check.stderr.log`). This was a compile check;
no live window was launched. The first check compiled the UI dependency closure
and took 13 minutes.

## Earlier file-only CLI workflow

`scripts/test-production.ps1 -SkipBuild -OutputDirectory
C:/Projects/synth/target/production-tests/general-20261107-r1` passed:

- Actual SDK catalogue and profile inspection.
- Source-driven SVG, JSON and binary output (including an Etalon NQuin result).
- Export and format/content checks for all three files.
- Fresh strict HMC reopen, preserving exact bytes.
- Independent generation from identical source, producing identical package bytes.
- Retained external input preserved exactly in generated output.
- Existing-output overwrite rejection and truncated-package rejection.

Candidate SHA-256:
`2e4cb8b1dc8c81c3f80679ed188fd30c746cf5a70ff45537dba8631945a670f9`.
Binary SHA-256:
`b3ee63cb07008fbdaaa125efb71891b4c42e0e5933e5e88456d71965456f6188`.
Lockfile SHA-256:
`890cd0191a09dd480ab264ef54d70028b8a2de9bee448ca42511938d66be66b7`.

## Build issues and qualification boundaries

The first reduced-dependency build hit Windows linker error LNK1114/error 5 when
replacing an existing compiler artifact. Running Cargo with access to the earlier
elevated build outputs resolved this. An offline retry then exposed the upstream
Etalon revision change; fetching and locking the required revision resolved it.
Neither failure was counted as a passing build.

The `files` producer authors or retains intact files. Its `kind` labels do not
certify arbitrary media encodings. Production implementations own their domain
semantics and should reuse the corresponding QualiaDB/Etalon format owners.
The PPM producer is an external-consumer test fixture, not a built-in domain or a
claim of complete mesh, audio, animation or inference generation coverage.

Producers are trusted in-process implementations and must be deterministic,
side-effect free and cooperative about budgets/cancellation. The library is not
an OS memory sandbox. Strict replay requires the original producer identity and
version; it rejects changed results. Complete source history and input hashes are
bound to native authority before authenticated host admission.

This is a breaking change from the former species host API and package schema.
Historical species HMCs are preserved and are not silently migrated. No live
window, audio device or WASM qualification is implied by these checks. The native
rendering evidence is limited to the three generated mesh assets described above.
General production does not create subject-specific quality acceptance.
No changes were committed or pushed.

The previous species workflow report is retained at
`docs/history/20261107-species-integration.md` and applies only to that former build.
