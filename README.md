# Webizen AI Synth

Webizen AI Synth is a general-purpose VibeScript asset-production library and application.
The desktop operator and studio are Aether applications: Aether is a Dioxus-fork
toolkit (`Aether.toml`, `LaunchBuilder`, `Config`, menus, tray); this repo is the app.
QualiaDB owns the language runtime, native resource formats and authority;
Etalon supplies execution providers; Hypermedia supplies shared geometry,
vector rendering, raster and timeline libraries. Poet remains a separate application.
Synth does not modify or embed Poet's application, UI or document model.

## Download

Tagged, signed installers and portable binaries: <https://github.com/mediaprophet/synth/releases>

- Windows: `WebizenAI-Synth-Setup-windows-x86_64.exe`
- macOS: `WebizenAI-Synth-macos-aarch64.dmg`
- Linux: `webizen-ai-synth_*_amd64.deb`

```text
synth update verify --artifact synth-windows-x86_64.exe
synth update apply --artifact synth-windows-x86_64.exe
```

Apply copies next to the running binary and never overwrites a locked `synth.exe`. See `docs/install.md`.

## Architecture

The same production pipeline handles every registered producer:

```text
VibeScript source + retained input resources
    -> checked source session
    -> chosen AssetProducer (domain validation and generation)
    -> validated output resources
    -> source/provenance manifest + native HMC
    -> strict replay, save and export
```

`production::ProductionSession` owns source revisions, bounded evaluation,
resource limits and candidate promotion. `AssetProducer` receives the typed Vibe
`asset` value and retained resources; it validates its domain and returns intact
files. It does not need a new host, agent backend or container implementation.
The public API accepts any implementation registered by the embedding.

`SynthHost` takes a prepared production session and native authority. It guards
source edits, inspection, saving and reopening without assuming an animal,
rig, timeline, audio device or particular output format. The agent exposes these
same guarded operations. The optional native monitor displays producer/resource
state. Format-specific rendering is a consumer of the resources, not the host's
identity.

There is no Ecopunk dependency, wallaby feature or species production path in
Synth's library, CLI or default dependency graph. Project models and domain
production implementations belong to consumers and producer modules. Historical
species outputs remain evidence of the former implementation, not its current
architecture or API.

Agent adapters live under `vendor/`:

- `vendor/antigravity-sdk-python` — Antigravity IDE MCP + `<agent-embed>` presentation
- `vendor/grok` — Grok MCP project config, skill, and JSON invoke helper

## VibeScript SDK and providers

The shared `vibescript-abi` is unified with QualiaDB's local SDK crate, including
Etalon's dependency on that ABI. Current source pins are recorded in
`provenance/current-dependencies.json`; Cargo's resolved graph is in `Cargo.lock`.

Vibe function providers and asset producers are different extension points:

- QualiaDB registers the built-in Vibe/Etalon function providers. Source imports
  use their existing versioned namespaces, such as `vibe:0.3/nquin`.
- The embedding registers an `AssetProducer` for its output semantics. Adding a
  new producer does not change Synth's host, source session or packaging layer.

`synth capabilities` inspects the actual Vibe catalogue and performs SDK-to-Etalon
parity dispatch. `synth profiles` lists the CLI's available asset producers.
Registration does not mean every possible engine operation is exposed.

`synth ontology` and `synth operations` inspect the category-theoretic ontology
and the **114 general-purpose production morphisms** across **12 canonical
categories** (`locus`, `manifold`, `surface`, `radiance`, `glyph`, `layout`,
`kinetics`, `acoustics`, `electromagnetics`, `geodesy`, `discourse`, and
`chronicle`) bound to the source host via `HypermediaOntologyProvider` (`342`
5-field parity-verified `NQuin48` triples). They generate 2D profiles/splines/
booleans/fillets/RMF frames/SVG, BiDi/MSDF typography and 2D/diegetic Webizen UI
layout sheets, extruded/revolved/lofted/CSG/QEM/Radial-Edge B-Rep/SubD/SDF-brick/
FSPM-botany `.10d` PBR manifolds and `.dxf` drawings, SLIM UV/MikkTSpace/Cycles
BSDF surfaces, rendered/graded/LUT/PSD/Kuwahara/Cryptomatte/SDFGI/Light-Tree
radiance lattices, `.emf` (`Q42EMF64`) electromagnetic field volumes, Amanatides-Woo
DDA raymarches, and 38D metameric null-space projections, `.m64` (`m64\0`)
planetary terrain/strata/continuum/XPBD containers, PGA MotorFTI/two-bone IK/
island XPBD/DirectSpaceState/NavMesh Funnel+ORCA/NLA/Lottie kinetics, 32-bus DAG/
PGA spatialized/biquad-filtered WAV/AAC audio, ISO-BMFF H.264 `.mp4` clips and
60Hz spatial motor packets, and PDF/DOCX/PPTX/chart-SVG/telemetry/fiduciary
discourse documents.
All canonical operation identifiers use category-theoretic naming with zero
`-craft` legacy names while preserving backward-compatible aliases. See
[`docs/asset-operations.md`](docs/asset-operations.md) and
`../hypermedia-production/migration/categorical-ontology-audit-20261107.md` for
full contracts and bounds.

## Build and use

The current workspace uses `../qualia-27062026` for the engine and SDK,
`../hypermedia-production` for shared production libraries, and
`../Aether` for the optional native monitor. The preparation script verifies the
recorded sibling revisions. It does not obtain project-specific source.

```powershell
./scripts/prepare-dependencies.ps1
cargo build --locked --bin synth -j 1
./target/debug/synth.exe capabilities
./target/debug/synth.exe profiles
./target/debug/synth.exe operations
./target/debug/synth.exe produce --source ./examples/production.vibe --out ./target/production.hmc --export-dir ./target/production-export
./target/debug/synth.exe produce --source ./examples/assets.vibe --out ./target/assets.hmc --export-dir ./target/assets-export
./target/debug/synth.exe produce --reopen ./target/assets.hmc --out ./target/assets-reopened.hmc
```

Output parents must exist; output files and export directories must be new.
`--source` can be repeated for edits. `--input KEY KIND FILE` retains external
resources, whose contents are hashed into the production binding.

The built-in `files` producer accepts an `asset` record containing a `files` list.
Each file declares `key`, `kind` and exactly one of `text`, `bytes` or `input`:

```vibe
import "vibe:0.3/nquin" as nquin;
let parity = nquin.wire.compute_wire_parity([1, 2, 4, 8, 16]);
let mut asset = {
    files: [
        { key: "notes.txt", kind: "txt", text: "An authored resource" },
        { key: "data.bin", kind: "bin", bytes: [0, parity, 255] }
    ]
};
```

Generated outputs use `{key, resource}` instead, where `resource` is the handle
returned by an encoder operation. The encoder supplies the kind. A manually
supplied `kind` label still does not validate arbitrary bytes.

## Library API

```rust
use std::sync::atomic::AtomicBool;
use synth::production::{ProducerRegistry, ProductionLimits};

let registry = ProducerRegistry::with_builtins();
let cancel = AtomicBool::new(false);
let mut session = registry.session("files", vec![], ProductionLimits::default())?;
session.submit(source, &cancel)?;
let candidate = session.candidate().ok_or("No candidate")?;
let reopened = registry.reopen(candidate.bundle(), ProductionLimits::default(), &cancel)?;
```

Implement `AssetProducer` and call `registry.register(producer)` to supply a new
generator. See `tests/general_production.rs` for an external-consumer example that
registers a procedural raster producer and uses the same edit/package/reopen API.
The source session passes typed Vibe values directly; no species schema or JSON
conversion is imposed on producers.

Producers are trusted in-process Rust implementations. They must be deterministic,
side-effect free, cancel cooperatively and respect the caller's budgets. Strict
reopen reruns the exact recorded producer version and source history, comparing
the complete package bytes. It rejects unknown versions or changed results.
These are logical limits, not an operating-system memory sandbox.
Capture results from external or nondeterministic generation as retained inputs
before authoring the replayable package. The pure source session does not make
network calls or grant publication authority.

For an authenticated embedding, `SynthHost::admit` requires a native session whose
document binds `ProductionSession::document_digest()`. That digest includes the
producer, source history and retained input hashes. Source text and agent requests
cannot mint authority. Shared QualiaDB authority APIs currently reside under its
`poet_host` namespace; Synth consumes them without changing Poet.

## Verification and migration

```powershell
cargo test --locked -p synth --lib --tests -j 1
./scripts/test-production.ps1
./scripts/test-asset-operations.ps1
```

The workflows generate native SVG, `.10d`, PNG and WAV, and exercise JSON,
Etalon-derived binary data and retained inputs. Tests
also exercise an independently registered raster producer, failed-edit rollback,
budgets, cancellation, altered packages, producer versions and the authenticated
host. Evidence is retained under `target/production-tests` and `target/host-tests`.
See `docs/production-validation.md` for actual executed results and limitations.
See `docs/capabilities.md` for the current asset capability inventory, comparison
with familiar creative workflows, complete SDK extension list and integration gaps.

This is an API and package-schema change from the former species implementation.
Old species HMCs are not silently adopted as general production packages. Their
original source owners remain responsible for migration; prior files are retained
unchanged. The old transport/audio assumptions are removed from the general host;
a media consumer can attach its own playback and output layer. The optional
`preview` feature retains the native `.10d` renderer, and `native-ui` retains the
asset-neutral monitor.

`HANDOFF.md`, `provenance/original-embedded-*` and
`docs/history/20261107-species-integration.md` are historical. They do not qualify
the current implementation. Original notices remain applicable.
