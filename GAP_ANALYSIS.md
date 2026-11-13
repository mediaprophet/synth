# Synth and Hypermedia production: gap analysis

Review date: 12 November 2026, Australia/Sydney.

## Assessment

The repositories contain a substantial implemented production system. The immediate constraint is the distance between **dispatchable operations**, **operations that solve the caller's actual problem**, and **independently qualified production results**. Expanding the catalogue is a lower priority than closing that distance.

The governing fidelity requirement, clarified during this review, is **EMF-native computation with explicit consumer projections**. Human-visible colour and human-audible audio must not define the retained computational spectrum. Objects may emit or receive components that no human display or speaker renders. Preserving those components is a first-class acceptance requirement, including immersive scenes.

Synth has a useful asset-neutral session/package/authority architecture. Hypermedia supplies substantial reusable implementations. The catalogue and ontology agree on 148 unique operation IDs. However, some adapters validate an input handle and then solve a fixed example, some outputs imply verification without the necessary evidence, and important application paths still differ in persistence, replay and resource policy.

This is a source and workflow review with a fresh native test-build attempt, not a certification of every function, every device/backend, scientific accuracy, or model appearance. No production code was changed to address findings during this review.

## Scope and evidence

`S` means `C:\github\synth`; `H` means `C:\github\hypermedia-production`.

| Item | Reviewed state |
|---|---|
| Synth | `4379de3d8a018979c51e4d2b72306a88dc7d8182`, branch `fix/kernel-produce-safe-containers` |
| Synth working changes at review start | `Cargo.toml`, `Cargo.lock`, `src/present.rs`, `src/present_3d.rs`, `src/project.rs`, `src/web_server.rs`; existing untracked build/adaptor directories preserved |
| Hypermedia | HEAD recorded in its reflog as `75e9007bfc7e06633a94548e8b02e233381c640b`, branch `fix/kernel-produce-safe-containers` |
| Hypermedia working-tree status | Not established: `git -C` access was denied by the execution environment; source files and Git HEAD/reflog were readable |
| Function inventory | Existing rust-audit index, generated 10 November 2026: 903 indexed files, 8,745 functions, 69 modules. The index and matching module records were consulted before tracing functions |
| Direct source inventory | 35 Rust files / 12,725 lines in Synth's `src` and `tests`; 867 Rust files / 240,059 lines across Hypermedia's `crates` and `checks` |
| Hypermedia crate directories | 68 manifests, plus the separate consumer-smoke workspace member; CPAL is explicitly excluded from default workspace membership |
| Catalogue consistency | 148 unique catalogue IDs, 148 unique ontology records, no ID differences |
| Static integration references | 146 of 148 operations referenced by canonical ID or declared alias in Synth integration-test source or its included Vibe examples |
| Authored test attributes | 62 `#[test]` attributes in surveyed Synth Rust files; 1,146 in Hypermedia crates. These are source counts, not executed-test counts or coverage percentages |

The operation inventory CSV records all 148 IDs, owners, argument descriptions, aliases, ontology source and static integration references. The crate inventory CSV records all 68 crate manifests, source size, test attributes, features and internal path declarations. A static reference is not proof that an operation executed, passed, or produced an acceptable result. `import_rgba_png` also appears in runtime bridge source; its absence from the integration-reference column is not an assertion that it is unused or untested everywhere.

### Validation performed

- `cargo metadata --locked --offline --no-deps --format-version 1`: passed for Synth. This verifies the top-level package metadata, not dependency compilation or fresh-clone reproducibility.
- `scripts/prepare-dependencies.ps1 -Offline`: returned success. Inspection established that it selected **zero** of the five provenance entries for revision verification; see G02.
- Catalogue/ontology ID set comparison and alias-aware static integration-reference inventory: passed consistency checks.
- Fresh native command started: `cargo test --locked --offline -p synth --lib --tests --target-dir C:\github\synth\target-gap-review -j 1`.
- Fresh native build completed in 19m 58s in this environment, with warnings. The library run reported **32 passed, 10 failed**; all ten failures were loopback client connections denied with Windows error 10013. The aggregate command therefore did not pass and stopped before integration execution.
- Separate Synth asset-operation integration run: **17 passed**. Authenticated-host integration reached a denied loopback connection at `tests/authenticated_host.rs:319` and failed for the same environment reason. Separate generic-production run: **2 passed**. No full authenticated-host pass is claimed for this review.
- Hypermedia `asset-production-vibe` unit tests, selected through the current Synth dependency closure: **189 passed**. This is not a full 68-package workspace/feature-matrix run.
- Broader selected Hypermedia library-test commands exposed **real test-target compile failures**: E0308 in `kinetics-continuum-multiphysics/src/solver/nonlinear.rs:82`, and 52 compiler diagnostics from stale `radiance-camera-raw` tests; see G34. These are separate from the sandbox's socket denial. `media-isobmff` could not be selected for testing from the Synth consumer workspace because its dev-dependencies require its owning workspace; no pass is claimed for that target.
- Eleven other Hypermedia library targets were run individually: **91 passed, 2 failed**. The failures were the shared loudness meter (G35) and an invalid PAdES test fixture (G34). Two selected `--lib` targets contain zero executed tests; they do not establish acceptance of their integration suites. Combined with the adapter crate, this review executed **280 passing Hypermedia tests and 2 failing Hypermedia tests**, plus the two compile-blocked test targets.
- Hypermedia top-level metadata also resolved offline: 68 workspace members, accounting for the excluded CPAL crate and included consumer-smoke package.
- Native CLI probes reproduced the frame-rate, ignored-audio and 16-bit interchange findings while the standard CLI strict-replay check succeeded; details below.

Build identity for these runs: Cargo `1.99.1`; effective local path overrides as described in G01; default Synth feature set. `Cargo.lock` SHA-256: `524ccb6deefd56b1b90a18d86245f6c49c9a5a1a7af505a50a2ba62d273116e1`. Built `target-gap-review/debug/synth.exe` SHA-256: `57633039037c93af1daf26e4845d86cf80a1a939b48b2ccc1e9f6ac3e32013bc`. These hashes identify this local run; they do not certify an unmodified or fresh-clone dependency closure.

All prior validation records retain their original scope. In particular, the 7 November report's 13-test/18-operation state and the 2 November source-publication handoff are historical evidence, not current acceptance of the 148-operation tree.

### Priority definitions

- **P1:** fix before presenting the affected behavior as reliable production functionality, actual measurement, verification, or a replayable release.
- **P2:** important integration, resilience, scale or usability work; can be explicitly scoped out of a restricted release.
- **P3:** maintenance and discoverability debt.

These priorities apply to the affected feature. A P1 finding in a demonstration adapter does not mean every unrelated producer is unusable. No P0 incident or repository-wide loss of integrity was established.

## What is already in place

1. **Shared ownership boundaries.** Synth handles sessions, sources, persistence and presentation; Hypermedia adapts algorithms; QualiaDB retains codecs, authority and numerical/geometry owners; Etalon supplies execution providers. Reimplementing these owners would increase the gap.
2. **Real catalogue and dispatch.** All 148 operations are represented in both catalogue and ontology. The `asset-production-vibe` source contains no `todo!()` or `unimplemented!()` calls. The findings below concern semantics and qualification, not a claim that catalogue morphisms are absent.
3. **Rollback and deterministic packaging.** Source submission prepares candidate state before promotion; artifact insertion rolls back on failed invocation; output keys are validated; resources are sorted; HMCs are checked and strictly replayed on principal production paths.
4. **A substantive authenticated-host path.** `SynthHost` uses native authority/context generations, revision checks, revocation and audited package saving. It should remain the reference when assessing the less-guarded standalone/HTTP paths.
5. **Broad native integration test scenarios.** Existing tests cover generated media, retained resources, custom producers, edits, invalid calls, tampering, budgets, authority and multiple produce-to-reopen flows. Some assert actual pixels, topology or signal content, not merely file signatures.
6. **Useful native formats and previews.** `.10d`, `.emf`, `.m64`, UV/topology/BVH handling, PBR resources, PNG contact sheets and QualiaPortal integration are concrete implementations. The `.emf` field/mesh separation is intentional and should be preserved.
7. **Honest bounds in several implementations.** Mesh, image, frame-count and logical resource caps exist; GPU GI refuses an unavailable requested backend instead of quietly relabeling CPU execution as GPU.

## Priority findings

### G01 — P1 — Release dependencies and provenance describe different builds

**Evidence:** `S/Cargo.toml:22` and `S/Cargo.toml:27` use QualiaDB `cfc063c...` and Hypermedia `75e9007...`. `S/provenance/current-dependencies.json` still records `ce88c2a...` and `929c49a...`, including a 144-operation description. The manifest also patches many dependencies to mutable sibling checkout paths. Hypermedia has its own sibling overrides in `H/.cargo/config.toml`.

**Impact:** a manifest Git pin does not identify the code actually built when a path override is active. A copied provenance file can attest to a different closure. The README's sibling names also disagree with current local layout.

**Close when:** one generated build receipt records the effective resolved package graph, source revisions, dirty-file hashes, enabled features, compiler/target, catalogue digest and executable hash. Qualify a clean checkout with no developer-only override dependency. Keep an explicit development-override mode for local work.

### G02 — P1 — Dependency preparation reports verification after checking nothing

**Evidence:** `S/scripts/prepare-dependencies.ps1:5` filters entries with `path -like '../*'`. None of the five current provenance entries has that path field. The script therefore runs zero revision checks and prints that dependencies were verified. This behavior was reproduced during review. Its `-Offline` parameter does not change this logic.

**Impact:** both workflow scripts rely on a preflight that can provide false confidence in the source closure.

**Close when:** validate the provenance schema and expected repositories first; reject missing paths or a zero-check result; record checked repository counts and actual revisions. Include a test where a required path/revision is omitted or wrong.

### G03 — P1 — MP4 timestamps do not preserve requested common frame rates

**Evidence:** `H/crates/asset-production-vibe/src/chronicle_ops.rs:87` chooses timescale 1,000 and integer `sample_duration = timescale / fps`. For requested 60 fps this produces 16 ticks per frame, equivalent to **62.5 fps**. For 24 fps, 41 ticks gives approximately **24.39024 fps**. Temporal sidecars use their own rational frame-rate path.

**Impact:** clip duration and sidecar timing can disagree even though byte-for-byte replay passes. This matters before adding companion audio.

**Reproduced:** a requested 60 fps native CLI capture exported an MP4 with track timescale 1,000 and `stts` sample duration 16, i.e. 62.5 fps. The CLI's strict replay succeeded.

**Close when:** use a timebase exactly representing supported rates or a correct variable-duration schedule. Independently demux the result and verify all sample timestamps, overall duration and sidecar alignment at 24, 25, 30, 50 and 60 fps. Define rational-rate admission explicitly rather than inferring it from integer fps.

### G04 — P1 — Movie capture accepts audio but does not capture it

**Evidence:** `H/crates/asset-production-vibe/src/acoustics_discourse_chronicle_ext/discourse_chronicle.rs:275` accepts an `audio` handle. It calculates actual and expected sample counts, then discards both at line 297. Only frame handles and fps are passed to the video muxer and locked-dt sidecar generation.

**Impact:** a successful audiovisual-looking invocation can silently emit a silent movie; duration mismatches are also not rejected.

**Reproduced:** otherwise identical one-frame captures with 440 Hz and 880 Hz companion tones produced different WAV members but byte-identical MP4 members (`2f8d1097e3e64942c477edf468af87ce6aa9e204e6f63f39dcc3f204d7734679`). Both passed the CLI's strict replay.

**Close when:** mux or explicitly export the companion audio through existing media owners with a declared synchronization policy. Until then, reject `audio` with an explicit unsupported-profile error. Test that changing only the input audio changes the appropriate output audio bytes/track and that timestamps align.

### G05 — P1 — FEM/LBM adapter does not solve the supplied mesh

**Evidence:** `H/crates/asset-production-vibe/src/kinetics_fem_garment_blocks.rs:9` admits and type-checks `mesh`, but does not use its geometry. It solves a hard-coded 2×2 matrix, advances a 4×4×4 LBM grid, and generates a fixed 2×1×1 hex box at line 40 for export.

**Impact:** valid results from existing numerical implementations can be mistaken for a solution on the caller's geometry. Altering that geometry does not define a new finite-element problem.

**Close when:** connect caller geometry, material assignments, boundary conditions, loads and fluid-domain state to the existing libraries; report their identity in the result. Until then, explicitly identify this as an example problem and reject input fields that imply unsupported behavior. Use analytic benchmarks and mesh-refinement/conservation checks.

### G06 — P1 — Garment adapter transfers one-triangle displacement to arbitrary input vertices

**Evidence:** the same file at line 105 builds a fixed triangle, pins its first two vertices, and solves it with the first standard fabric. At line 159, an input mesh is handled by adding the solved triangle's single `dy` to every input vertex after index 1. Caller topology is not used to build the cloth system.

**Impact:** exported geometry changes, but it is not an orthotropic solve of that garment. Vertex ordering controls the apparent pinning.

**Close when:** construct the cloth topology and rest state from the actual mesh; admit explicit pins, fabric and collision/body inputs; solve each relevant degree of freedom. Verify behavior under vertex reordering, different topology, different pins and materials.

### G07 — P1 — Typography output exceeds the demonstrated shaping/font contract

**Evidence:** `H/crates/asset-production-vibe/src/glyph_layout_diegetic/bidi_msdf.rs:161` performs custom BiDi classification/reordering and emits per-character clusters with hand-authored advances. Lines 291 onward use built-in `glyph_advance`/`pair_kerning` and zero offsets; the adapter does not receive a font resource for OpenType shaping. `H/crates/asset-production-vibe/src/glyph_knuth_plass.rs:283` creates a `.ttf` with a single zero-filled `head` table. Existing tests explicitly expect one SFNT table.

**Impact:** BiDi ordering and deterministic stroke glyphs are useful, but they do not demonstrate full font-driven Arabic/Indic shaping or a usable subset font. A directory-parser round trip does not establish font validity/renderability.

**Close when:** wire actual retained font bytes and the existing relevant shaping owner into the operation; test contextual forms, ligatures, combining marks, cluster mapping and positioning. Produce a renderable font/subset with required tables or label the current SFNT artifact as diagnostic metadata rather than a font. Validate with an independent font consumer.

### G08 — P1 — Engine telemetry is illustrative, not measured

**Evidence:** `H/crates/asset-production-vibe/src/acoustics_discourse_chronicle_ext/discourse_chronicle.rs:98` computes timing values from constants and frame-index arithmetic, for example render time `1100 * scale + ((i * 29) % 150)`. It derives p99 and `budget_compliant` from those values and labels CPU workspace byte counts as a VRAM ledger.

**Impact:** the generated JSON is deterministic, but cannot substantiate actual frame-budget or GPU-memory compliance.

**Close when:** consume a retained, versioned real telemetry capture or a separately measured live-host snapshot. Record measured/illustrative status, source and measurement conditions. Keep demonstration data available only with an unmistakable synthetic-data marker; never derive a real compliance verdict from it.

### G09 — P1 — Fiduciary RID verification does not establish native authority

**Evidence:** the same file at line 205 returns `permitted` directly from caller-provided `deontic_mode` (`permit`/`oblige` versus `forbid`), echoes DID strings, counts dependencies and sets generation to 1. It does not consult a DID resolver, signed agreement, policy graph or the native authoring authority on this path.

**Impact:** this is policy-record evaluation/serialization, not evidence that a caller has rights over a resource. It must not be confused with SynthHost's real admission and revocation checks.

**Close when:** use the existing authority/policy owners with explicit evidence inputs, or rename/relabel the result as illustrative policy evaluation while preserving aliases. Unknown, expired, revoked and forged evidence must not yield a verified permission result.

### G10 — P1 — PDF/UA audit evaluates a constructed good snapshot

**Evidence:** `H/crates/asset-production-vibe/src/discourse_chronicle_fiduciary.rs:15` accepts `resource` but only checks that the handle exists. At line 44 it constructs an accessibility snapshot with `is_tagged`, valid encoding, valid links and other favorable flags already set. It then reports passed rules. Its math-layout fragment is also fixed to `E=mc^2` rather than extracted from the supplied document.

**Impact:** checker code exists and executes, but the verdict does not verify the document named by the resource argument. The PII text path has real processing and should be qualified separately rather than allowing it to validate the broader audit claim.

**Close when:** derive the snapshot from parsed input document structure and content; bind the report to the exact resource digest; distinguish supported checks, failures and unknowns. Run independent PDF validation and intentionally malformed/tagging/reading-order fixtures.

### G11 — P1 — PAdES helper can say integrity is verified without a comparison digest

**Evidence:** `H/crates/discourse-pdf-compliance/src/sign/pades.rs:77` accepts an optional expected digest. `None` sets `digest_matches = true` at line 98 and returns the message that content is intact since signing. This helper computes byte-range hashing; its inputs do not establish CMS signature, certificate trust or signing time.

**Impact:** hash computation and range admission can be mistaken for verification of an actual signature. This finding concerns this helper's result contract; it is not a claim that all signing-related algorithms are absent elsewhere.

**Close when:** report `computed/not-compared` when there is no expected imprint; reserve verified status for an authenticated comparison. Keep byte-range integrity, cryptographic signature validity and trust/time/revocation outcomes separate. Add malformed-range/overflow tests and tampered/missing-signature cases.

### G12 — P1 — Empty SDF extraction silently becomes a proxy box

**Evidence:** `H/crates/asset-production-vibe/src/manifold_fields_sdf/sdf_botany.rs:239` replaces an empty vertex or triangle result with an eight-vertex, twelve-triangle box, then returns it through the normal mesh storage path.

**Impact:** a no-surface result can look like successful geometry reconstruction. A nonempty-mesh assertion can pass on invented geometry.

**Close when:** return an explicit empty/no-isosurface result or a checked error. If an intentional preview proxy is requested, mark it separately and prevent it from being mistaken for the extracted master. Test all-positive/all-negative fields, out-of-range iso values and boundary intersections.

### G13 — P1 — Format inspectors display verification badges without running verification

**Evidence:** `S/src/present_3d.rs:314` classifies resources by magic, kind or extension. The Matter-64 branch displays `CRC-32C Verified` at line 484, and the EMF branch displays `BLAKE3-Bao + CRC-32C Sealed` at line 520. These inspector branches do not derive those labels from the corresponding checked-reader/verifier outcome. Newly authored tests pass zero-filled buffers to the card renderer and assert classification text.

**Impact:** successful HTML rendering is conflated with container authenticity. HMC integrity and member-format integrity are different properties; `kind` is explicitly only a label elsewhere in Synth.

**Close when:** consume checked-reader receipts for each format and show verified/invalid/not-checked states with the relevant digest. Test truncation, corrupt checksums, mismatched extension/magic and unsupported versions.

### G14 — P1 — HTTP workbench needs an explicit local trust boundary

**Evidence:** `S/src/web_server.rs:600` creates a default standalone MCP backend for arbitrary requested tool names. Request parsing at line 617 does not validate Origin, Host or a session token before dispatching POST routes. JSON parsing is independent of Content-Type. `serve_project_file` at line 835 serves project HTML on the same origin as those APIs. Loopback binding is present and is a useful restriction. Native MCP output writes are refused without a configured output root, and engine-tool dispatch retains its own enforcement; those protections should not be erased in describing this finding.

**Impact:** visiting active project HTML grants its script the workbench origin. Cross-origin request rejection and capability scoping are not explicit in this HTTP layer. CORS response headers alone do not authenticate a request or prevent all POST delivery. The exact reach of downstream engine tools requires a separate dispatch-by-dispatch authorization test; this review did not establish a successful remote exploit.

**Close when:** require explicit session/capability authorization for stateful/engine routes, validate expected origin and host, enforce the intended request content type, and isolate active project documents from the privileged API origin. Exercise hostile-origin, active-document, unconfigured-root and denied-engine-intent cases.

### G15 — P1 — `env --present` bypasses create-only presentation saving

**Evidence:** `S/src/main.rs:54` uses `std::fs::write` for the output of `synth env --present OUT.html`. Unlike the main produce/present path, this overwrites an existing destination.

**Impact:** an operator following the project's create-only output convention can overwrite an existing file on this command path.

**Close when:** route it through the existing create-only presentation writer with equivalent budget and durability reporting. Verify that an existing destination's bytes survive a rejected attempt. This is a narrow consistency fix, not a reason to change the package writer.

### G16 — P1 — Packages do not identify the effective production implementation

**Evidence:** `S/src/production/package.rs:9` persists schema, producer ID/version, source history and input hashes. `FilesProducer::version()` remains `1` in `S/src/production/files.rs:12`. Actual geometry/codec/provider output semantics can evolve independently of that producer version. Strict reopen correctly rejects changed resulting bytes, but the manifest does not tell the reader which effective dependency/operation implementation to recover.

**Impact:** preserving source plus input hashes is insufficient to select a reproducible historical runtime. An HMC can remain intact while the latest strict-reopen path no longer reproduces it.

**Close when:** record a versioned runtime/provider/catalogue build identity, resource profiles and relevant environment semantics in provenance; define how semantic changes bump contracts. Preserve a checked read-only inspection/export route for intact historical packages separately from execution of their source. Test old/new implementation compatibility deliberately.

## Integration and product gaps

### G17 — P2 — Several domain adapters expose parameterized examples rather than complete caller-authored problems

The algorithms exist; the adapter schemas and bindings determine how much of them an agent can actually use. These are concrete examples, not an assertion that every operation in these domains is fixed:

| Existing adapter | Observed binding | Required integration |
|---|---|---|
| `schedule_cpm_evm_montecarlo` | Optional resource is resolved, then `Project::sample_launch_program()` is scheduled (`acoustics_rf_kriging_cpm.rs:63`) | Accept actual task/dependency/resource/cost networks and bind output to their digest |
| `solve_flexbox_grid_constraints` | Fixed two-child flex and two-cell grid; caller changes container dimensions (`layout_autolayout_foliation_nodegraph.rs:34`) | Bind arbitrary layout children/constraints, or explicitly keep this as a demonstration beside the existing `solve_box_tree` path |
| `simulate_causal_block_diagram` | Fixed constant → integrator → scope model (`kinetics_fem_garment_blocks.rs:208`) | Admit actual blocks, ports, wires, solver policy and initial state |
| `kriging_variogram_morphometry` | Fixed sample-point set; optional image supplies a first-pixel bias (`acoustics_rf_kriging_cpm.rs:237`) | Accept caller sample coordinates, values, variogram and raster/DEM metadata |
| `develop_camera_raw_hdr` | Derives a Bayer-like buffer from RGBA, or synthesizes one; constructs brackets from one developed buffer (`radiance_raw_cdl_trace.rs:8`) | Admit actual sensor samples, CFA/black-white levels and real bracket inputs; preserve linear master precision |
| `sense_wifi_csi_vitals_aoa` | Generates CSI phases and breathing/heart sinusoids; optional audio is mixed into synthetic data (`electromagnetics_ops.rs:489`) | Admit retained real CSI frames/timebase/calibration and clearly label simulated receipts |
| `recalculate_tabular_fiduciary_archive` | Creates a three-cell SUM example and a fixed journal with a parameterized amount (`discourse_chronicle_fiduciary.rs:178`) | Accept actual workbook formulas, ledger accounts/entries, timeline and provenance |
| `solve_parametric_sketch` | Constructs a fixed line and constraint set (`locus_sketch_warp_geospatial.rs`) | Bind caller-authored entities, constraints and expressions to the existing solver |

For each adapter, close the gap with explicit authored-input schemas and tests in which changing the supplied problem changes the result for the correct reason. If a fixed example is intentionally retained, expose its illustrative scope in discovery and output receipts. Do not add duplicate catalogue names or copy underlying libraries to accomplish this.

### G18 — P2 — Loudness analysis changes short input before measuring it

**Evidence:** `H/crates/asset-production-vibe/src/acoustics_rf_kriging_cpm.rs:148` repeats short input samples until a minimum duration is reached, measures the repeated buffer, and exports it as audio. Pitch fallback and the fixed Camelot-key comparison also need explicit interpretation.

**Gap:** an analysis tool should not silently turn a short source into a periodic longer signal and present the result as a measurement of the original. Report insufficient duration/coverage, or make repetition a named transform and retain both identities. Validate loudness against independent reference fixtures at multiple sample rates and durations.

### G19 — P2 — HTTP production blocks all other workbench requests

**Evidence:** `S/src/web_server.rs:126` and `:151` handle accepted connections serially. Production and replay happen synchronously inside the request. Read/write socket timeouts do not bound CPU evaluation. `/` reconstructs the tray environment through production at line 542. `ProductionSession::prepare_edit` also replays history from scratch at `S/src/production/source.rs:58`.

**Gap:** a long produce/replay or repeated expensive root request can prevent file serving, UI status and annotations from progressing. Add a bounded job queue, cancellation/progress, admission/backpressure and safe read concurrency. Cache immutable environment results by build/source identity. Measure edit/replay scaling without weakening deterministic validation.

### G20 — P2 — Browser HMC bridge returns a digest but no usable HMC artifact

**Evidence:** `S/src/wasm_lite.rs:271` constructs an HMC candidate. Its JSON response at line 340 returns digests and member metadata, but neither package bytes nor a retrievable/persisted package reference. This path also does not perform the explicit fresh strict reopen used by `/api/produce` and standalone produce-present. Current bridge tests exercise invalid/empty input and JS string presence, not a successful browser-to-HMC round trip.

**Gap:** a caller cannot recover the newly constructed package through this response. Return bytes or an authorized opaque artifact reference/download route; verify replay before promotion; add a successful PNG compose → OPFS/read → HMC/download → strict-reopen test. The browser helper is a JS bridge to native HTTP production, not evidence of a full standalone WASM producer.

### G21 — P2 — Annotation storage and project schema need durable revision semantics

**Evidence:** `S/src/project.rs:273` rewrites `annotations.json` directly after read-modify-write. It has no atomic replacement/backup or optimistic version check. `parsed.schema` is ignored at line 146; the timestamp helper named `now_rfc3339` emits Unix seconds at line 426. Notes name paths/aspects rather than resource digest/model revision/stable geometry element.

**Gap:** interrupted writes or multiple processes can lose notes; future schemas are silently treated as current. A note may drift when a file is regenerated. Use an atomic versioned store, validate supported schemas, preserve timestamp semantics, and bind review notes to a resource/model revision. The serial current HTTP loop reduces same-process concurrency but does not make filesystem writes atomic.

### G22 — P2 — Project discovery traversal is not bounded by depth or visited directories

**Evidence:** `S/src/project.rs:335` recursively lists directories using `path.is_dir()` and has only a maximum listed-file count. It does not reject directory links/junctions or track visited canonical directories. In contrast, `resolve_file` does canonical containment checking before serving a file.

**Gap:** a link cycle with few/no files can recurse without reaching the file-count cap; linked directories can also be listed before serving later rejects them. Bound depth, avoid or explicitly authorize link traversal, and track visited identities. Test Windows junctions as well as ordinary path traversal.

### G23 — P2 — Machine-readable contracts lag actual argument/resource semantics

**Evidence:** catalogue discovery provides free-form `arguments` strings, while the ABI uses a generic byte-slice argument envelope (`H/crates/asset-production-vibe/src/ontology/mod.rs:156`). The same default envelope is used across operations. The macro declares `PureNoState`, but JSON serialization reports `Irreversible` at line 318. JSON also uniformly claims `BitExact`, including paths needing separate GPU/platform qualification. Several document record parsers silently skip wrongly typed nested elements rather than rejecting them, despite strict top-level field checking.

**Gap:** agents cannot reliably prevalidate nested inputs or distinguish exact, platform-qualified and illustrative outputs from the discovery contract. Generate typed argument/result schemas from the admitted parser contract; include defaults, units, enums, bounds, target requirements and verification level. Serialize determinism/reversibility from actual descriptor data and qualify backend-specific guarantees empirically.

### G24 — P2 — Backend support needs capability admission instead of process panics

**Evidence:** outside `asset-production-vibe`, the surveyed Hypermedia source contains 6 textual `todo!()` and 16 textual `unimplemented!()` occurrences; these include platform-specific and commented code and are not 22 proven reachable catalogue failures. Concrete active paths include unsupported CPU image formats at `H/crates/vector-cpu-render-adapter/src/image_convert.rs:36` and mask layers at `H/crates/vector-hybrid-renderer/src/scene.rs:645`.

**Gap:** a selected backend can panic on an unsupported input. Advertise per-backend support and reject unsupported formats/features before execution. Qualify reachable paths on Windows/native/WebGPU/WebGL/WASM separately. Do not generalize these backend limitations into an assertion that the 148 catalogue morphisms are stubs.

### G25 — P2 — Current validation is broad but not a production acceptance system

**Evidence:** no checked-in CI workflow was found in either inspected repository. There are source tests, local scripts, generated audit docs and selected codec-oracle helpers. The two main Synth workflow scripts rebuild/use `target/debug/synth.exe`, conflicting with the current rule to leave the live MCP target untouched. There is no observed single current receipt tying all catalogue cases, source/build identities, browser behavior and reviewer acceptance together. GPU tests can legitimately pass by proving refusal when no adapter exists; that does not demonstrate successful GPU execution.

**Gap:** separate gates for (1) schema/dispatch parity, (2) input-sensitive behavior, (3) independent numerical/codec conformance, (4) replay/tampering/cancellation, (5) browser/native feature matrices, and (6) human output acceptance. Run safe isolated targets and record exact outcomes. Add successful compose/bridge and multi-canvas portal tests; exercise all 148 operations, aliases and failure boundaries without equating call coverage to quality.

### G26 — P2 — Production profiles and multi-output publication need explicit policy

**Evidence:** the video path is a 1..64-frame, ≤256×256 preview profile using H.264 I_PCM; geometry, raster and resource limits are intentionally bounded. CLI production saves the HMC before completing all exports/presentation, and standalone MCP can save HMC before HTML fails. Preview/presentation paths can duplicate payloads into base64 and decoded browser/GPU buffers.

**Gap:** short bounded previews are useful, but not a sustained final-media pipeline. Define preview/master/device profiles, chunked/streamed jobs, memory accounting and accepted lossy conversions. Treat a run as staged until all requested outputs and receipts succeed; report partial artifacts precisely or publish via a final manifest/commit marker. Do not raise bounds blindly or claim logical budgets are an OS sandbox.

### G27 — P2 — Visual/scientific acceptance and delivery promotion are not established

**Evidence:** the current workbench lists working/delivered lanes and collects notes. Catalogue existence, file hashes, healthy HTTP responses and topology counts cannot establish a recognizable organ, correct cloth, acceptable typography or cinema-quality asset. The vertebrate compiler's comparative primitive scaffold is explicitly scoped in the project contract and tests. This review did not run a new multi-view model acceptance session.

**Gap:** define subject-specific reference sets, dimensional/unit checks, silhouette/landmark tests, material/color checks and expert/reviewer acceptance. Bind front/side/three-quarter previews and annotations to the exact asset revision. Make delivery promotion a checked event with required artifacts/approvals rather than only a directory convention. Author named organs/models in `.vibe` using existing geometry operations; do not substitute the comparative scaffold or add a Rust compiler per model.

### G28 — P3 — Documentation and operational guidance have multiple stale sources of truth

**Evidence:** `S/README.md` and the local authoring skill still describe 114 morphisms and older paths/tool counts; `S/HANDOFF.md` is dated 2 November; the main validation narrative is dated 7 November and describes 18 operations/13 tests; current provenance says 144; `H/CRATES.md` lists 31 packages while 68 crate manifests exist. Generated module docs remain a much broader inventory. The existing documentation coverage report identifies 153 undocumented public functions in `asset-production-vibe` and 195 in `chronicle-tabular-engine` at its generation timestamp.

**Gap:** generate inventory/counts/build commands from current contracts and maintain one dated acceptance/status document with explicit historical links. Retain old validation as history. The 54 crates without a root README are a discoverability consideration, not proof of missing documentation: many are covered by inline and generated module docs.

### G29 — P2 — PBR bake outputs do not form a self-contained material asset

**Evidence:** `H/crates/asset-production-vibe/src/surface_pbr_brush.rs:103` genuinely bakes AO and world-normal images from the mesh. The MaterialX map then references fixed `bronze_albedo.png`, `bronze_normal.png` and `bronze_ao.png` filenames. The adapter returns AO/normal image handles and MaterialX, but no corresponding albedo resource or resource-name manifest. Its PSD is a newly created empty `PsdDocument` that is encoded/decoded; the baked images are not inserted as PSD layers on this path.

**Gap:** a structurally valid MaterialX/PSD file is not a complete transferable baked material. Admit map names/resources explicitly or return a closed material package whose references resolve; include actual bake layers/composite in PSD when offered as the material deliverable. State world-versus-tangent normal conventions and validate them in an external material consumer. The caller can export the AO/normal handles today; this finding concerns the incomplete reference/presentation closure, not absence of baking algorithms.

### G30 — P1 — Sixteen-bit codec profiles do not preserve sixteen-bit processing precision

**Evidence:** `H/crates/asset-production-vibe/src/graphics.rs:55` maps 8-bit samples to 16-bit using multiplication by 257; line 60 maps 16-bit samples to 8-bit by discarding the low byte. `Rgba` stores `Vec<[u8; 4]>` at line 68. PNG import accepts 16-bit data but converts into that 8-bit lattice; 16-bit export expands those 8-bit samples. The profile and comments correctly note RGBA8 intermediates in places, but the existence of a 16-bit codec/profile is not a high-precision edit pipeline.

**Impact:** a 16-bit source can produce a valid 16-bit PNG with only 256 distinct levels per channel. Low-byte information is lost in the derived result even during import → export. Retained original input bytes can remain intact in the HMC; the loss concerns generated lattice results, not necessarily the retained input member.

**Reproduced:** a retained RGBA16 PNG pixel `[0x1234, 0x5678, 0x9ABC, 0xDEF0]` became `[0x1212, 0x5656, 0x9A9A, 0xDEDE]` after import/export with `profile:"16-bit"`. The exported header remained 16-bit, and strict replay passed.

**Close when:** distinguish container bit depth from working precision in discovery and receipts. Preserve the **existing native EMF representation and spectral meaning** through internal computation; increasing an RGBA buffer to u16/float is not a substitute for retaining the underlying spectrum. Where lossless 16-bit external interchange is promised, preserve those source samples or declare the quantization boundary. Keep RGBA8-only operators as explicit display/device transforms with lineage to the native master. Header-depth checks and upconverted 8-bit round trips are insufficient.

### G31 — P1 — Native EMF preservation is not an end-to-end adapter contract

**Evidence:** Etalon already has native spectral computation and explicit EMF-to-display projections: `C:\github\etalon\crates\etalon-render\src\spectral\colour_kernel.rs:39` consumes `[alpha, mu, sigma]` payloads and produces RGB8 for display. Its spectral basis and deferred resolve are real existing implementations. QualiaDB's `q42/emf_container/header.rs:172` defines the separate durable `EmfCellRecord64` with E/B, wavelength, phase, Stokes, sigma and confidence. These representations serve different contracts and must not be confused merely because both are called EMF.

Hypermedia preserves encoded `.emf` bytes, but the general artifact enum in `H/crates/asset-production-vibe/src/lib.rs` exposes `Image`, `Rgba` and sample-buffer `Audio` as its working modalities. Spectral slice/raymarch operations deliberately project the field to RGBA. The resulting generic raster handles do not carry a typed link declaring which native spectral state, bands and conversion policy they represent. Keeping the original `.emf` handle separately is possible today; it is not an automatic spectrum-preserving processing contract.

The albedo path also needs that distinction: `H/crates/asset-production-vibe/src/manifold_ops.rs:793` passes sampled colour to `compile_mesh_to_10d_with_extras_and_frames`; QualiaDB's existing `container_10d/surface_reading.rs` explicitly encodes its `SRD1` surface-reading section as RGBA8 per vertex. That section is a valid authored colour projection, not a replacement for the spectral material/field master or proof that all native EMF state survived.

**Impact:** an agent can follow a successful preview-processing/export path while dropping the native field from the delivered package or substituting an RGB-derived result for a field result. A valid PNG, immersive sound preview, material or replay hash does not prove preservation of invisible/inaudible computational state.

**Close when:** make native field identity, units, supported spectrum/profile, timebase, confidence and explicit loss/conversion metadata part of the production contract. Preserve the existing native EMF master through scene/material/field processing and HMC delivery; link display/audio derivatives to it. Reuse QualiaDB/Etalon representations and codecs rather than creating another EMF format. Out-of-band data must remain queryable and influence relevant computational receivers even when it contributes nothing to human output.

### G32 — P1 — EMF compilation reduces heterogeneous emissions to one effective wavelength

**Evidence:** `H/crates/asset-production-vibe/src/electromagnetics_ops.rs:213` accumulates weighted wavelengths and Stokes values from all emitters. It stores their averages in a single cell record, sets `spectral_sigma` from that averaged wavelength with a 400–700 nm clamp at line 246, and emits each spatial brick with `time_slice_index=0` and `spectral_band_mask=1`. The existing QualiaDB writer already accepts explicit time-slice and spectral-band metadata. The current producer admits wavelengths 1..1e9 nm; this is a bounded profile, not all possible emission bands.

The same producer sets `refractive_n=1`, `extinction_k=0` and `confidence_alpha=1` for every generated cell. Its input schema describes source emitters and a grid, not caller-defined interacting object materials/media or receiver responses. Those constants are reasonable for an explicitly scoped vacuum/example profile, but are not qualification of arbitrary multispectral object interactions or uncertainty propagation.

**Impact:** a multi-band emitter mixture is not retained as its constituent spectrum. A weighted mean wavelength can represent a different spectral state from the original mixture; different out-of-visible bands can also collapse to the same sigma endpoint. This is a native-field fidelity issue before display conversion, distinct from acceptable colour gamut mapping. The stored wavelength is not itself replaced by RGB, but the component distribution is already reduced.

**Close when:** retain band-resolved contributions or the appropriate existing native spectral representation, including per-band/time metadata and declared approximation limits. Do not use a visible-range sigma projection as the sole computational identity of out-of-band radiation. Verify two spectra that share a visible preview but differ outside the visible band remain distinguishable after compile, seal, reopen and computation. Explicitly qualify supported emission/receiver band profiles and dynamic evolution rather than implying unrestricted Maxwell simulation.

### G33 — P2 — Immersive output and non-human receivers need richer retained scene/signal binding

**Evidence:** `H/crates/asset-production-vibe/src/acoustics_discourse_chronicle_ext/acoustics_bus_spatial.rs:339` applies scalar distance/Doppler/occlusion/reverb processing and returns the same sample-buffer `Audio {rate,samples}` shape. This path does not encode channel layout, listener orientation, object identity, directional receiver response or a retained acoustic/EMF scene. Existing owners already provide more: `C:\github\qualiaDB\crates\qualia-audio\src\spatial.rs` re-exports Etalon's spatial emitter/listener and stereo mixing APIs; `H/crates/acoustics-daw-session/src/dsp/pan.rs:95` supplies speaker gains. The finding is the adapter/scene contract, not absence of spatial-audio algorithms.

**Gap:** carry object emitters, poses/velocities, signal-domain units, propagation/interaction state and receiver responses independently of the listener's device mix. Admit explicit channel/layout or object-audio output profiles, and bind them to retained native scene state. Reuse the existing sigma-to-audio conversion where intended as a declared sonification/acoustic projection; record any physical coupling separately. Do not discard ultrasound/infrasound or invisible EMF contributions simply because a human-facing output does not reproduce them. Test head rotation, multiple listeners, directional sources, occlusion, time alignment and a non-human receiver sensitive to a band absent from the human preview.

### G34 — P1 — Domain library test targets have drifted from their APIs and do not compile

**Reproduced:** the selected Hypermedia library-test command failed with Rust E0308 at `H/crates/kinetics-continuum-multiphysics/src/solver/nonlinear.rs:82`. `newton_raphson_solve` requires `tangent_k: &dyn Fn(&[f64]) -> Vec<f64>` at line 22, but `newton_raphson_linear_spring_convergence` passes an unborrowed closure. The production library had compiled successfully as a dependency; the failure is in the `#[cfg(test)]` target. It is not a denied socket or an absent solver.

A subsequent selected run, with that crate excluded, failed compiling `radiance-camera-raw`'s unit tests with 52 diagnostics. Examples are the old eight-argument `demosaic_raw` call at `H/crates/radiance-camera-raw/src/lib.rs:57` against a seven-argument function, obsolete `DevelopSettings` fields, and `PhotoRecord`/`SmartRule` usage that does not match the current metadata API. The diagnostic count includes cascading errors and should not be read as 52 separate product defects. Its production library also compiled as a dependency.

The separate `discourse-pdf-compliance` target executed four passing tests and one failing signature/timestamp test. At `src/tests.rs:134`, the fixture's `ByteRange::new(0,49,97,30)` extends to byte 127 of a 122-byte buffer. The checked validator correctly rejects it; the fixture's success expectation is invalid. This is a test-fixture defect, separate from G11's verification-result semantics.

**Close when:** reconcile these tests with the intended current APIs, preserving meaningful numerical/image assertions, and run the test targets in their supported source/build closure. Then execute the broader domain test matrix so a successful application build cannot conceal broken test targets. No production-source fix was applied during this analysis.

### G35 — P1 — Shared loudness meter uses last-frame energy instead of block energy

**Reproduced:** `acoustics-daw-session` ran five passing tests and one failure. `test_ebu_r128_metering` supplies one second of a 1 kHz, amplitude-0.1 tone at 48 kHz but receives -70 LUFS instead of its expected range, at `H/crates/acoustics-daw-session/src/lib.rs:55`.

**Source:** `H/crates/acoustics-daw-session/src/dsp/meter.rs:78` resets `weighted_sum` for each frame. At the 400 ms boundary, line 91 divides only that final frame's energy by the number of samples rather than using the accumulated block/channel energy. The producer adapter's `integrated_lufs_bs1770` at `H/crates/asset-production-vibe/src/acoustics_rf_kriging_cpm.rs:8` explicitly works around the shared meter by implementing a separate accumulation path. This explains how adapter-level tests can pass while the shared library fails.

**Close when:** repair and independently qualify the shared meter, including channel weights/layout, block overlap/gating and sample-rate fixtures, then route the adapter through the corrected owner instead of maintaining divergent implementations. Keep G18's source-duration policy separate from this arithmetic defect.

### Executed Hypermedia test detail

All package selections below used the effective Synth dependency closure and isolated target directory. They are not a clean-clone standalone Hypermedia release qualification.

| Package / target | Passed | Failed | Qualification |
|---|---:|---:|---|
| `asset-production-vibe --lib` | 189 | 0 | Adapter unit scenarios passed; GPU test success alone may include a valid refusal path |
| `kinetics-garment-draping --lib` | 5 | 0 | Existing library scenarios |
| `kinetics-schedule-cpm --lib` | 12 | 0 | Existing library scenarios |
| `acoustics-daw-session --lib` | 5 | 1 | Loudness failure, G35 |
| `geodesy-spatial-analysis --lib` | 5 | 0 | Existing library scenarios |
| `vector-glyph-shaping --lib` | 15 | 0 | Existing library scenarios; does not establish G07 font-shaping/subset acceptance |
| `surface-pbr-studio --lib` | 6 | 0 | Existing library scenarios; G29 package closure remains separate |
| `chronicle-tabular-engine --lib` | 0 | 0 | No tests executed by this selected target; integration suites not implied |
| `chronicle-fiduciary-ledger --lib` | 16 | 0 | Existing library scenarios |
| `chronicle-office-codecs --lib` | 0 | 0 | No tests executed by this selected target; integration suites not implied |
| `discourse-pdf-compliance --lib` | 4 | 1 | Invalid ByteRange fixture, G34 |
| `audio-timebase --lib` | 23 | 0 | Existing library scenarios |
| `kinetics-continuum-multiphysics --lib` | — | — | Compile blocked, G34 |
| `radiance-camera-raw --lib` | — | — | Compile blocked, G34 |
| `media-isobmff --lib` | — | — | Consumer-workspace selection refused because dev-dependencies require the owning workspace |

### Required spectral and immersive acceptance cases

| Case | Computational requirement | Human output may legitimately do |
|---|---|---|
| Two EMF spectra with the same visible appearance | Retain distinguishable native spectra and receiver responses | Render the same RGB preview |
| Object emitting only outside visible bands | Preserve emission, propagation and interaction data through packaging/reopen | Render no visible emission, or a clearly labeled false-colour lens |
| Signal outside a human listener/device's audible band | Preserve the declared native signal and applicable interactions/receivers | Produce no audible contribution, or use an explicitly selected sonification |
| Immersive scene with two different listeners | Compute from the same retained source scene and individual receiver poses/profiles | Produce different device mixes without rewriting the scene master |
| Display/audio export round trip | Preserve native EMF/source references and conversion receipts | Quantize a derivative according to its declared output profile |
| Spectral-band/time-slice transport | Preserve band identity, temporal ordering, units and uncertainty | Select only the bands/time slice relevant to the chosen preview |

## Domain-by-domain readiness matrix

Counts below follow catalogue namespaces. The separate `qualia.species.assemble_corpus` operation is assigned to `manifold` in the ontology, so ontology category counts report 27 there while the `hypermedia.manifold.*` namespace has 26.

| Domain | Operations | Implemented foundation | Main remaining gap / required acceptance |
|---|---:|---|---|
| Locus | 15 | Profiles, transforms, paths, offsets, booleans, fillets, resampling, RMF, sketch/geospatial adapters | General authored sketch constraints; degenerate/self-intersection/tolerance tests; units and large-coordinate fixtures; independent SVG/DXF/geospatial interpretation |
| Manifold | 26 | Extrude/revolve/loft, mesh booleans, bevel/inset/solidify/bisect, subdivision, fields, SDF, instances, import/export, `.10d` sealing | G12 empty-isosurface policy; numerical/topological benchmark corpus; attribute preservation across topology changes; distinguish model recipes from comparative/botanical scaffolds; qualify stated size bounds |
| Surface | 8 | SLIM/atlas, tangents, BSDF descriptions, procedural maps, AO/normal bake, MaterialX, PBR packing | G29 material reference closure and actual PSD bake layers; preserve authored UV/material identity through mesh edits and seam splits; validate tangent conventions and external MaterialX consumers; reference renders and seam/distortion metrics |
| Radiance | 30 | RGBA composition, vector rasterization, masks, transfer/grade/LUT, PNG/PSD, GI, sprite/nine-slice packing | G30 true working precision versus codec depth; actual sensor/bracket admission; color/alpha/ICC reference cases; CPU/GPU target qualification; successful browser bridge tests; explicit fidelity profiles |
| Glyph | 7 | Deterministic built-in strokes/metrics, custom BiDi, atlas generation and paragraph composition | G07 actual font-driven complex shaping and renderable subset; Unicode conformance and fallback; independent font/render checks |
| Layout | 10 | Generic box tree, theme/UI sheets, diegetic picking/input, frustum, paged/nodegraph adapters | General input binding for newer adapters; font-accurate pagination/overflow; responsive/accessibility checks; structured authoring and stable selection semantics |
| Kinetics | 15 | IK, interpolation, motors, XPBD, queries/navigation, NLA, simulation, archetype rigs and numerical adapters | G05/G06 real problem binding; arbitrary block/task inputs; conservation/constraint tests and collision benchmarks; sampled motion acceptance |
| Acoustics | 9 | Tones, gain, WAV/AAC, biquad/mixing, buses, spatialization and analysis | G18 source-preserving analysis; G33 native scene/object/receiver binding and immersive device projections; out-of-audible-band preservation; route companion audio into chronicle; separately qualify device I/O |
| Electromagnetics | 6 | Sealed `.emf`, verification, spectral slices/raymarching, metamer projection and RF adapter | G31/G32 EMF-native multiband/time preservation before display/audio conversion; distinguish synthetic RF from measurement; real capture input; uncertainty and receiver validation |
| Geodesy | 7 | Heightfield sculpt/erosion, `.m64`, advection/contact/sector operations and spatial analysis | Actual observation/DEM inputs for kriging adapter; coordinate/unit/nodata/dateline tests; preserve uncertainty; respect 2.5D sculpt scope rather than promising volumetric geology editing |
| Discourse | 9 | PDF/DOCX/PPTX/chart/gallery generation plus telemetry/policy/audit/picking adapters | G08–G11 evidence semantics; external document rendering/accessibility checks; independent table/reading-order/Unicode validation; truthful inspection badges |
| Chronicle | 5 | MP4 frame encode/decode, locked-dt sidecars, spatial packets, archive integration | G03/G04 exact timing and audio; authored timeline/workbook/ledger input; sustained media profiles; independent decode and AV sync |
| Species corpus extension | 1 | Existing `qualia.species.assemble_corpus` dispatch and ontology entry | No literal reference in surveyed Synth integration tests; qualify real corpus input, sealed output and replay through Synth, retaining the existing owner |

## Recommended implementation sequence

| Work package | Scope | Exit evidence |
|---|---|---|
| 1. Establish an auditable baseline | G01, G02, G16, G25, G28, G34, G35 | Effective dependency receipt; non-vacuous preflight; compiling and passing numerical/codec test targets; shared meter correction; isolated feature-matrix receipts; current status docs; historical package selection policy |
| 2. Correct observable output and trust claims | G03, G04, G08–G15, G30–G32 | Exact sample timelines; explicit audio support/refusal; empty SDF behavior; checked-reader badges; illustrative-versus-measured labels; no unsigned verification; create-only env output; scoped HTTP actions; native EMF and multiband-preservation contracts |
| 3. Connect caller-authored problems | G05–G07, G17, G18, G33 | Mesh/cloth/font/task/RAW/CSI/workbook/scene input schemas, retained emitters and receiver models, immersive output profiles, and input-sensitive tests against independent references |
| 4. Complete durable studio workflows | G19–G23, G26 | Recoverable HMC bridge artifact, replay before promotion, bounded jobs/progress/cancel, atomic notes and run publication, schema versions and backend admission |
| 5. Qualify production use cases | G24–G27 and domain matrix | Browser/native/device matrix; reference renders and codec/numerical fixtures; revision-bound human acceptance; approved delivery manifests |

Work packages can contain independent changes, but each should preserve the existing source/package/authority owners and include concrete acceptance evidence. Update parser, ontology argument/result contract, NQuin spans/contract identity, catalogue and compatibility tests together when a contract changes. Reuse existing operation IDs where the intended capability already belongs; do not invent parallel morphisms or make catalogue expansion a substitute for closing input-binding and verification gaps.

## Boundaries of this review

- Reviewed code is the current readable local working tree, including pre-existing uncommitted changes. No remote fetch/publication status was inferred from local Git tracking references.
- Existing function and module registries were used; they were not regenerated or treated as acceptance evidence.
- Test-attribute and literal-reference counts are inventory aids, not branch coverage, conformance coverage, or a percentage-complete estimate.
- QualiaDB/Etalon internals, complete third-party decoder correctness, scientific validation and external signature trust infrastructure were not exhaustively audited here.
- No new model appearance, GPU output quality, audio-device behavior or end-user browser session is claimed as accepted.
- User source changes were preserved. This review adds the report and two inventories; findings require a separate implementation pass.

## Companion inventories

- [All 148 operation contracts and static integration references](docs/reviews/2026-11-12-operation-inventory.csv)
- [All 68 Hypermedia crate manifests and source-test inventory](docs/reviews/2026-11-12-crate-inventory.csv)
