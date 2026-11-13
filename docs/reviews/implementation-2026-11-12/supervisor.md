# Independent integration review — 12 November 2026

Status: source review in progress. No builds/tests executed by supervisor; no visual acceptance. Findings describe the working tree at review time and require owner repair plus verification after assembly.

## Findings and ownership

- S01 (P1, A02/A06/A11): MCP production native review invokes `contracts::native_preview(asset, destination)` with no HMC or asset key. The compiled MAT1 texture digest cannot resolve its files/v2 dependency payloads in that subprocess. Repair both MCP argument propagation and default three-view renderer ingestion, not only explicit `--shot`. Owners interfaces + fields_animation notified. Regression: actual textured GLB through MCP export/review, verify dependency digests/upload and inspect geometry-specific texture/normal map.
- S02 (P1, A01/A04): imported attributes live in a parallel immutable sidecar; mesh-edit operations creating a new geometry handle do not automatically preserve it. Require native vertex/face mapping propagation where available or an explicit refusal for attributed inputs; never silently degrade import → edit → compile. Root notified and assigning owner. Regression: exercise supported edit chains and expected refusals with normals/UV/tangents/materials/dependencies.
- S03 (P1, A02/A03): `unwrap_atlas_parameterize` replaces UV0 while copying original imported material texture digests. Original texture pixels cannot remain correctly mapped after arbitrary reparameterization without baking/remapping. Refuse this combination or implement texture remapping with explicit semantics. Root notified. Regression: textured checker/seam fixture before/after atlas, inspect output, and preserve normal-map basis requirements.
- S04 (compatibility, A06/P02/P03): package `build()` globalizes `textures/sha256/*` image resources regardless producer version; restrict canonical placement to files/v2. Interfaces repairing. Nuance: legacy source `kind` label validation disallows MIME slash, so a direct legacy package-builder fixture covers namespace semantics; do not claim the impossible explicit-source replay was reproduced. Existing ordinary files/v1 strict replay still required.

## Review checklist

- S05 (P1, A01/A12): static GLB feature validation ignored unsupported material/extensions, including transmission/unlit. Supervisor repaired declared and undeclared extension checks, constrained supported texture extensions to consumed locations, and refused duplicate JSON documents/version-length ambiguity. Focused regression added; not run.
- S06 (P2, A15): SVG added opacity/hover/focus filters on already computed states while raster did not. Supervisor removed duplicate visual effects while preserving disabled pointer behavior, aligned hover/focus geometry, and fixed raster corner-radius double scaling. Regression added; no visual acceptance yet.

Repairs awaiting verification: interfaces owns S01/S04 and WebP dependency MIME transport; root owns S02/S03 via central explicit attribute-edit policy. Supervisor owns only S05/S06 source changes, avoiding simultaneous file ownership.

Follow-up guard review: `mesh_to_sdf_brick` also needs explicit attributed-input discard (root added it). Untextured `mode=slim` output was still sealing smooth normals/geometry/UV only; the atlas exception cannot cover this until the owner repairs preserved material/normal sealing. Root has assigned that repair. Instancing retains its prototype handle so the recursive guard at realization can see its attributes. Ordinary geometry-only inputs remain admissible.

P08 metadata finding: PNG16 imports with unhandled gAMA/ICC/cICP cannot be silently classified as unspecified and graded as sRGB. Interfaces accepted this and is implementing explicit unsupported-metadata refusal, explicit transfer declaration for untagged grading, and congruent sequence transfer metadata.

## Pre-build assessment of scope

Finish the concrete owner repairs, then assemble/build and run the integration checks. There is no source-review acceptance yet. Current precision delivery is bounded UNORM16 (not unrestricted HDR), with a lossless rational-timed PNG sequence; tiled processing/seam continuity remains unimplemented/unproven. Native shots target one preassembled `.10d`, up to 2048 square and 16 timestamps, with camera/key-light/AO/shadow/prefiltered-probe controls. Arbitrary skeletal transfer must be demonstrated from an actual authored input, not inferred from section attachment. Live three-client discovery/reconnection and all seven domain visual reviews remain acceptance work. These are explicit plan boundaries, not missing algorithm claims.

- [ ] A01–A04: real GLB import, hierarchy/reflection, seams/tangents/material assignment/map transport through HMC/reload and geometry-edit behavior.
- [ ] A05–A09: MCP host/error/schema/input scope, durable receipts, v1 compatibility, actual client registration/reconnection versus templates.
- [ ] A10: arbitrary skeleton + inverse binds + complete weights → SKL1 reload → rest/two bent poses/sequence.
- [ ] A11–A12: replayable shot/texture/camera/light settings; accurately bounded specialist material support.
- [ ] A13: authoritative f32 physical terrain, scale/origin/no-data → erosion → compile, preview-independent samples.
- [ ] A14: true retained precision, tile continuity, rational sequence time/color metadata.
- [ ] A15: shared glyph outlines, retained-font shaping, mixed-script/wrapping/state parity.
- [ ] A16: seven actual reference outputs with exact artifact identities and inspected views. File existence, green tests and counts do not constitute acceptance.

## Deferred verification

Root will supply assembled build/test results and rendered run paths. Inspect actual PNG/SVG and relevant front/side/three-quarter views; inspect motion across frames. Preserve explicit limitations rather than making a blanket film-quality claim.

## Fresh candidate fixture visual review — 12 November 2026, 21:26 run

Independently opened the actual `front-view-assumed.png`, `side-view-assumed.png`, and `three-quarter-view-assumed.png` in `C:/github/synth/production/client-parity-20261112-212641-dcde5373/mcp-review-0`, plus exported `mcp/checker.png`.

- Fixture-quality visual pass: blue/yellow checker placement covers the visible cube faces at consistent scale. Grid rows meet the visible vertical edge without cracks or blank texture bands; no diagonal triangle seam, missing face, or visibly inverted surface appears. The three-quarter image shows a closed cube with top and both adjoining faces textured.
- Evidence limit: front and side look identical because the cube and checker are symmetric. These images cannot establish unique face identity or rule out UV mirroring/quarter-turn rotation. They do not qualify more complex seams, normal maps, anatomical assets, or film-ready material/lighting quality.
- Label visual acceptance remains pending. `mcp/label.svg` exists, but no paired label PNG was exported. SVG source contains the `LOCAL ASSET REVIEW` glyph path and accessibility label; source inspection does not establish readability, clipping, or focus-state appearance. The image viewer cannot decode SVG, no browser viewer is available, and available local renderer checks did not yield a working renderer. No installations were made. Root will inspect the separate paired interface PNG/SVG or add the PNG to a later fixture run.

This is bounded visual acceptance of the fresh cube fixture only. Root-reported client parity/refusal results are separate evidence; the supervisor did not rerun them. Full integration acceptance remains pending the integrated tests and remaining domain/output gates. No Cargo or implementation-source edits were performed for this review.

## Fresh flora visual review — 12 November 2026, 21:32 run

Run: `C:/github/synth/production/quality-20261112-213248-b386b5d8`. Opened every front/side/three-quarter PNG in `flora-native-0`, `flora-branch-native-0`, and `flora-branch-native-1`; `frame-0000.png` in all three corresponding shot directories; and exported `flora-branch-canopy-turnaround.png` plus `flora-flower-turnaround.png`. The `flora` export contains no turnaround PNG. Receipts and outputs were preserved.

- **Leaf / tissue study (`flora`) — diagnostic asset.** The silhouette and petiole are visible, but the native front and authored shot show a largely flat leaf with an opaque pale rectangular patch masking the tissue region and a protruding regular cell grid. The side view exposes the separated layers, which explains the study construction but is nearly edge-on. For a reusable teaching illustration, use an oblique tissue-detail camera, separate/label the layers clearly, and avoid the opaque panel hiding the structure being explained. For an intact reusable leaf, export the lamina separately from this diagnostic inset. Flat untextured appearance and low detail contrast remain visible; no close-up leaf-material acceptance.
- **Canopy (`flora-branch-native-0`) — canopy-volume proxy.** Trunk, branching silhouette, and roots are legible in all views. However, repeated shiny foliage lobes have conspicuous triangular ridges/pinches and fine banding, especially in the three-quarter native image and authored shot. The foliage reads as overlapping solid masses rather than a resolved leaf canopy. First inspect those surface ridges/bands for overlapping surfaces or normal discontinuities; image evidence alone does not establish their cause. Then improve leaf/cluster silhouette and material roughness variation while retaining the authored pale trunk. Suitable as a structural/blockout tree; not accepted as a finished close-up tree asset.
- **Flower/dissection (`flora-branch-native-1`) — reusable schematic study with presentation repairs needed.** Buds, open stamens, central cutaway, raised cap, and separate capsule are distinguishable in front/three-quarter views. The source explicitly requests anthesis and peel, so the raised cap and separate fruit are intentional study elements, not evidence of accidental disconnected geometry. In the side view the capsule occludes the central flower/cutaway; move the study elements or choose a dedicated dissection view. Fine filaments become fragmented/subpixel at authored-shot and turnaround size, so closer framing or higher-resolution/supersampled presentation is warranted before claiming readable fine anatomy. Simple cups and stems remain visibly schematic.

The authored shots improve background contrast but do not resolve these geometry/detail issues. No obvious frame clipping was observed. These outputs demonstrate functioning botanical generation and native transport; none establishes film-ready botanical quality. Root retains remaining domain, runtime, and full integration acceptance gates. No Cargo or implementation-source edits were performed.

## Flora repair source wave — awaiting fresh execution and visual review

Root authorized repairs following the visual findings above. Source diagnosis confirmed that Whitegum lobe longitude modulation gave different heights to coincident pole vertices, 5/9-cycle modulation was undersampled by 7/8 longitude segments, winding faced inward, and supplied sphere normals ignored ellipsoid scaling/scalloping/droop. The existing lobe method is now in focused `whitegum_foliage.rs`: bounded 12..24 latitude / 36..72 longitude resolution, one vertex per pole, outward nondegenerate faces, exact wrap positions, seam-shared area-weighted normals. Crown-volume roughness is 0.68 instead of the individual waxy-leaf default; pale bark is preserved. This repairs the surface construction, not a claim of resolved individual-leaf or film-quality canopy detail.

The leaf's source explicitly called its cuticle plates translucent but sealed every material as Opaque. New `botanical_materials.rs` partitions indexed triangles by their authored vertex coverage, keeping the solid lamina in the opaque pass and only translucent sheet geometry in Blend; semantic IDs and index coverage remain intact. Translucent sheets do not cast an opaque shadow rectangle. Sheet vertex alpha changes from 225/255 to 100/255 so its two faces do not almost completely hide the cells; the CPU turnaround also classified the former value as opaque. **This alpha is authored diagnostic illustration coverage, not measured optical transmission or physical calibration.** The layer's existing material alpha is retained.

The intentionally separate flower capsule is now offset in Z as well as X, avoiding side-view occlusion of the central flower. The leaf recipe requests a larger cutaway and exports its turnaround. The quality harness retains all standard native views, adds 2048-pixel authored flora/flower shots, and adds a separate oblique tissue-detail shot. Existing captured runs and receipts were not edited.

Meaningful regressions added: lobe outward/nondegenerate faces, finite unit normals, poles and wrap continuity; actual sealed leaf MAT1 separates opaque and translucent triangle ranges with full index coverage; flower/capsule projected bounds remain separate in front and side views. A focused core helper test covers mixed-coverage range splitting. Rustfmt parsed the new files, Python AST accepted the harness, and scoped diff whitespace checks passed. **No Cargo or runtime execution was performed by supervisor; Rust regression results and fresh output acceptance remain pending root's coordinated build and independent review.**

Owned source files: Hypermedia `living_world_anatomy/whitegum_canopy.rs`, new `whitegum_foliage.rs`, new `botanical_visual_tests.rs`, and the test-module declaration in `living_world_anatomy/mod.rs`; Qualia `render/botanical_anatomy.rs` and new `botanical_materials.rs`; Synth `examples/quality/flora.vibe` and `scripts/verify-quality-scenes.py`. Pre-existing unrelated changes, especially streaming/storage work, were preserved.


## Portal EMF retention and actual field-preview source wave

Normal qualia-shell startup already awaited WebGPU before constructing Portal; the confirmed gap was early/direct calls, WebGL2 fallback, and device recovery. One bounded accepted LoadedEmf now survives upload and recovery. Dense shape admission uses the existing MAX_DENSE_CELLS. Metadata and physical decoding complete before replacement; failed upload leaves the previous accepted field/status intact. Initialization outcomes and explicit aborts reach frame-boundary adoption, including no-renderer fallback. Display states are empty/pending/uploaded/unavailable; upload is not a claim of presented field pixels. Both Synth receipt branches share the same status handler, including stale API unverified status.

The existing normal Portal frame never drew the loaded EMF field. A focused wasm-only render_emf_preview(amplitude_gain) adapter now submits the existing Etalon phase/sigma slice shader to the WebGPU surface, capped at 1024 pixels per side. Gain must be finite in (0, 1e6]; only its shader uniform changes. The receipt declares gain, z=0/t=0, conversion=legacy-phase-sigma-diagnostic, submitted=true, and pixel_readback=not-performed. load_emf wording now names the actual phase/sigma diagnostic rather than falsely calling it a wavelength visualization. No calibrated RGB or complete-spectrum claim is made; the existing H-produced wavelength PNG remains a distinct reference.

prepare-portal-emf-review.py now uses a separate CPU bootstrap canvas for early load/explicit abort, then awaits initialization on the visible context-free WebGPU canvas. It checks retained-field replay, a ready replacement, and corrupted load refusal. Each pixel probe calls the explicit field draw and immediately copies that actual browser canvas into a visible proof canvas, reads real pixels, checks visible variation, and hashes them. Replayed/ready/refused-replacement captures must agree. The original PCM byte and signed-zero checks remain. Exact timebase comparison preserves numeric tokens rather than rounding them through JS Number; the current physical fixture itself uses a small relative epoch. No native PNG or mocked pixel substitute is accepted as GPU proof.

LoadedEmf's unchanged struct moved into the portable ownership helper and is publicly re-exported at its former portal_load_emf path. The standalone container-format-conformance harness imports the actual state source. Its four new tests use the production Writer/Reader/dense unpack for physical fixtures and exercise pending adoption, recovery, refused replacement, unavailable backend, failed replay retry, and explicit initialization refusal. They do not qualify acoustic conversion; the fixture's incidental frequency value is not tested as derived audio. Runnable filter: cargo test --manifest-path C:/github/qualiaDB/tools/container-format-conformance/Cargo.toml production_emf_state::tests. Parent coordinates execution; no Cargo was run by this worker.

Source checks completed: new Rust helper formatting and syntax parsing of edited facades; scoped diff --check; existing portal-emf-status-test.cjs with both inline/fetched receipts and state refresh; Python staging/generator syntax; actual emitted browser module and qualia-shell JavaScript syntax; exact-clock comparator source checks distinguish adjacent integers above 2^53, preserve sign, and distinguish integer tokens from strings. See portal-emf-source-checks.log. Rust tests, WASM build, real browser capture and independent visual acceptance remain parent execution gates. The flora generator was synchronized only in its two flora lines (peel_01=0.9 and flora-turnaround.png), preserving parent terrain/cosmos edits.

Exact lifecycle/preview source files:
- C:/github/qualiaDB/crates/qualia-core-db/src/render/portal_emf_state.rs (new)
- C:/github/qualiaDB/crates/qualia-core-db/src/render/portal_load_emf.rs
- C:/github/qualiaDB/crates/qualia-core-db/src/render/mod.rs
- C:/github/qualiaDB/crates/qualia-core-db/src/render/portal/mod.rs
- C:/github/qualiaDB/crates/qualia-core-db/src/render/portal/recovery.rs
- C:/github/qualiaDB/crates/qualia-core-db/src/render/portal/emf_preview.rs (new)
- C:/github/qualiaDB/crates/qualia-core-db/src/render/gpu/mod.rs
- C:/github/qualiaDB/crates/qualia-core-db/src/render/gpu/emf_present.rs (new)
- C:/github/qualiaDB/docs/js/qualia-shell.js
- C:/github/qualiaDB/tools/container-format-conformance/lib.rs
- C:/github/synth/src/present_portal.rs
- C:/github/synth/scripts/prepare-portal-emf-review.py
- C:/github/synth/examples/quality/generate_fixtures.py (flora-only two-line synchronization)
- C:/github/synth/docs/reviews/implementation-2026-11-12/portal-emf-status-test.cjs (new source-level host contract check)

Earlier flora source files remain as recorded in the preceding repair section. Fresh output acceptance is still pending; authored cuticle coverage is diagnostic presentation, not measured physical calibration.
