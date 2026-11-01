# Synth

Synth is the distinct agent-driven embodiment/asset-production consumer. It is
not a newer Poet. Poet's reference interface remains
[Tool-Chest & Manifolds](https://mediaprophet.github.io/qualiaDB/poet-app/index.html)
and its broader design and existing source are preserved. This application has
no dependency on Poet's package or shell.

**Current state: source implementation, uncompiled and unexecuted.** No new
native binary, model, audible output or acceptance is claimed. Tests, rebuilding
and production runs remain deferred while the selected coding is incomplete.
P64 inference is future-only.

## Direct agent production command

The registered `synth` binary has a local `produce` command. It runs the existing
QualiaDB/Vibe data-only source owner, Ecopunk generator, motion admission,
native Q42 corpus builder and HMC writer. A script edit does not need a Rust
rebuild after the tool itself has been built. The current command/profile is
the existing authored **r3 wallaby**, not arbitrary species or terrain.

Future qualification recipe, **not executed during this coding stage**:

```powershell
$synthCandidateDir = 'C:/github/ecopunk/assets/production-studies/swamp-wallaby-review-r1/revisions/synth-candidate-r1'
New-Item -ItemType Directory -Force -Path $synthCandidateDir
cargo run --manifest-path C:/github/ecopunk/Cargo.toml -p synth --no-default-features -- produce --source C:/github/ecopunk/assets/production-studies/swamp-wallaby-review-r1/authoring/swamp-wallaby-r3.vibe --source C:/github/ecopunk/crates/ecopunk-terrain/src/animal_motion/study.vibe --out "$synthCandidateDir/candidate.hmc"
```

`--q42 RESOURCE_KEY FILE.q42` and `--m64 RESOURCE_KEY FILE.m64` retain supplied
native resource bytes. `--reference SOURCE_IRI NAMESPACE_IRI rdf|json|csv|html|declared`
records source namespaces; a reference is not acquired data or adopted rights.
Disclosure defaults to restricted. Raw imports have no invented CBOR metadata;
fresh reopen retains the metadata actually present in a saved bundle.

`synth produce --reopen CURRENT.hmc --out NEW.hmc` uses the existing strict
source/resource/model replay. Older schema/generator incompatibility is an
explicit error. It does not silently discard a corpus or relabel a Studio
checkpoint as a species bundle. Output is create-only. The existing bounded
review/rig limits are explicit current profiles, not a cinema quality ceiling.

Local generation uses the caller's ordinary filesystem access. It does not
invent an authenticated project session, contributor signature, review,
scientific acceptance or complete film/game/cartoon package. A candidate must
be presented through the review route before any accepted/completed claim.
Production must bind each new version to the existing full catalogue/dossier
and expanded cases; this construction command does not silently close them.
**REQ-VISREF-020:** production requires at least 20 relevant subject images found
through online search, with source URLs and an actual output-to-reference
quality assessment. The command does not collect those images or certify that
requirement automatically; a generated candidate is not acceptance.

## Native agent project service

`SynthHost::admit` receives an existing opaque QualiaDB `NativeAuthority` session
and immutable source inputs. This profile's native document explicitly binds
the existing **Vibe source-history digest**, which is distinct from the older
Studio artifact digest. They are not interchangeable by filename or hash label.
The factory retains its existing source, model, resource and motion owners.

`SynthAgentBackend` uses QualiaDB's existing MCP authoring transport. It requires
the software agent's own native authenticated session and rejects impersonation
through a human session. Signed project adoption and sessions remain owned by
the existing native services. Neither request JSON nor a loaded HMC grants them.

Implemented operations:

- `synth.inspect`, bounded `synth.source.read`, and guarded `synth.source.submit`.
- `synth.playback.control`: authored clip selection, clock play/pause, exact
  seek, rational rate and loop selection. Current coupled sound rate is 1x.
- `synth.package.save` and `synth.package.reopen`: native-configured output root,
  create-only candidate HMC, exact digest, native package audit and fresh replay.
- `synth.audio.status`; `audio-device` additionally enables play/stop of a
  **native-owned, already prepared/prefilled** sound job. No unimplemented
  decoder/bank-loading tool is advertised.

MCP JSON is existing protocol framing, not a new canonical project/knowledge
format. Wide counters/ticks travel as decimal strings. Actual source/authority,
projection, transport and output-scope context are checked at the domain call.
Source preparation happens outside the authority lock; final source promotion
and audit use the native transaction. Namespace/disclosure policy survives edits.
Partial source/file commit and audit failure are reported rather than implying
rollback. A new source/transport/host incarnation retires older output.

Native embedding outline, not a fabricated standalone login implementation:

```rust,ignore
// `host` has been admitted through existing native project services.
host.configure_output(synth::NativeOutputRoot::admit(output_directory)?)?;
let shared = std::sync::Arc::new(std::sync::Mutex::new(host));
let backend = synth::agent::SynthAgentBackend::bound(shared.clone())?;
let cancellation = backend.cancellation();
// The embedding owns the agent transport/thread lifetime.
qualia_mcp::authoring::serve(reader, writer, backend, Default::default())?;
// An optional native supervision surface can share that same host:
// synth::native_ui::launch_with_cancellation(shared, Some(cancellation))?;
```

The native UI is an optional **agent supervision surface**, not Poet's human
authoring design or an implemented cinema viewport. Its dependencies point at
`C:/github/rust-app-framework` (Aether). The local Cargo package names remain
`dioxus` and `dioxus-native`; their upstream web shell is not selected. The
headless production command needs neither this UI nor an audio device.

## Audio integration and remaining wiring

`pcm-device-output` in the shared hypermedia repository owns CPAL output. It
uses the controlled existing CPAL backend and Core sound-binding/receipt types.
Native `prepare_audio` checks the selected Q42 resource bytes and source tuple;
`render_sound_block` consumes the same admitted MotionPlayer pose and native
clock, renders through the existing Core scheduler and enqueues under final
native authority. Natural tails derive audio source position from the fixed
output/source offset while holding the actual ended motion pose. This does not
advance a second animation clock or claim acoustic presentation timestamps.

Device storage and callback work use fixed, caller-budgeted slots; callback
allocation is not introduced. Starvation/lock contention retires the output
epoch. Stop cancels the lease immediately and waits for actual backend reset
acknowledgment. Failed/blocked shutdown retains the device slot and holds audio;
polling does not prove instantaneous revocation or measured hardware latency.

The native caller must still supply **real admitted sound-bank/decoded media
bindings** through the existing media/codec owners. A Q42 digest alone does not
prove that an arbitrary acoustic graph or sound recording is correctly sourced.
Concrete bank-loading/producer dispatch, native project-launcher mounting and
the continuously animated HMC review viewport remain implementation work.
Original/native media containers must survive that integration; encoded audio
is not renamed to M64 and the limited Q42/M64 review profile is not a full master.

Source/dependency pins and limits are recorded in the
[wave-3 implementation record](../../docs/qa/2026-11-01-production-audit/cinema-implementation-wave3.md).
