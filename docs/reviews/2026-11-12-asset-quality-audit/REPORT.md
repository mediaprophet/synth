# Synth / Hypermedia asset-production audit

12 November 2026. Local Windows workspace audit; no project publication or cloud execution. Production code and client configuration were not changed. Companion: [implementation plan](C:/github/synth/docs/reviews/2026-11-12-asset-quality-audit/IMPLEMENTATION_PLAN.md).

## Conclusion

The stack contains substantial, working asset-production machinery. All **148 catalog operations** are present in the current source and reviewed production binary; the local MCP server advertises **94 tools** (7 Synth and 87 Qualia). Claims that lofting, booleans, sculpt displacement, atlas algorithms, materials, anatomy authoring, or terrain algorithms simply do not exist are unsupported.

However, reliable access and lossless composition are not yet sufficient for an unrestricted film-quality production promise. This audit reproduced discarded GLB attributes, an atlas route that asks for seams it cannot accept, divergent MCP invocation hosts, misleading error signaling, and raster UI labels rendered as bars. Source inspection also found gaps between material/tangent/rig computations and the final authored asset. These are specific integration and representation problems, not reasons to rebuild the geometry kernel.

Working deterministic packaging is valuable, but it cannot establish visual fidelity: two visibly different intended materials produced identical `.10d` files through the generic import route, and that loss was itself perfectly reproducible.

## Scope and evidence

The audit inventoried every catalog operation and MCP schema, followed selected production-critical paths through Synth, Hypermedia, QualiaDB and Etalon, exercised local CLI and isolated stdio MCP processes, ran the existing asset integration suite, and inspected exported images. It did **not** individually certify all 8,745 indexed functions or all 148 operations against artistic/scientific references. A successful invocation, valid container, or recognizable silhouette is not film-quality acceptance.

Source inventory was checked against the supplied agent contract and Hypermedia's generated [function registry](C:/github/hypermedia-production/audit-docs/FUNCTIONS.md), dated 10 November. Relevant source files were then read. The registry counts are historical inventory metadata, not fresh coverage metrics.

| Evidence | Result / artifact |
|---|---|
| Current catalog | 148 source IDs = 148 runtime IDs; no differences. [Summary](C:/github/synth/docs/reviews/2026-11-12-asset-quality-audit/inventory-summary.json) |
| Per-operation routes | [148-row route inventory](C:/github/synth/docs/reviews/2026-11-12-asset-quality-audit/operation-routes.csv), [full operation metadata](C:/github/synth/docs/reviews/2026-11-12-asset-quality-audit/operations.json) |
| MCP surface | [94-tool schema inventory](C:/github/synth/docs/reviews/2026-11-12-asset-quality-audit/mcp-interface-inventory.csv), [full schemas](C:/github/synth/docs/reviews/2026-11-12-asset-quality-audit/mcp-tools.json) |
| ABI / SDK discovery | 11 providers; Hypermedia descriptor arrays expose 147 of 148 operations. [Provider metadata](C:/github/synth/docs/reviews/2026-11-12-asset-quality-audit/sdk-providers.json) |
| Runtime probes | [Primary MCP replies](C:/github/synth/docs/reviews/2026-11-12-asset-quality-audit/mcp-probes.json), [additional requests/replies](C:/github/synth/docs/reviews/2026-11-12-asset-quality-audit/additional-mcp-probes.json) |
| Fresh production probes | 8 successful cases and 2 expected argument refusals; [commands, errors and hashes](C:/github/synth/docs/reviews/2026-11-12-asset-quality-audit/run-20261112-103345-dcc594/results.json) |
| Replay and import comparison | Identical repeated HMCs; identical compiled outputs from different GLB attributes. [Checks](C:/github/synth/docs/reviews/2026-11-12-asset-quality-audit/run-20261112-103345-dcc594/checks.json) |
| Existing integration tests | `cargo test --locked --offline -p synth --test asset_operations --target-dir C:\github\synth\target-gap-review -j1`: **17 passed, 0 failed**. [Log](C:/github/synth/docs/reviews/2026-11-12-asset-quality-audit/asset-operations-tests.log) |
| Additional fresh outputs | Native GPU assembly preview, three 2048² views; swamp-wallaby example, 14 outputs. Logs and artifacts in the [run directory](C:/github/synth/docs/reviews/2026-11-12-asset-quality-audit/run-20261112-103345-dcc594) |

Source baseline commits: Synth `4379de3d8a018979c51e4d2b72306a88dc7d8182`; Hypermedia `75e9007bfc7e06633a94548e8b02e233381c640b`; QualiaDB `afbe431195167fde0fadc233ed79fd2fe85f73f0`; Etalon `032837f72559fa7ddf724b2b74d2b474dd5ba724`. Existing working-tree changes are recorded in the inventory summary and were preserved. Runtime binary hashes are in [binary-inventory.json](C:/github/synth/docs/reviews/2026-11-12-asset-quality-audit/binary-inventory.json); different binaries must not be treated as interchangeable.

## Findings

Priority P1 means a reproducible failure or material blocker to the requested production workflow. P2 means a bounded capability, incomplete contract, or missing acceptance evidence. Neither classification means the owning algorithm is absent.

### A01 — P1: generic import drops the authored surface attributes

**Reproduced and source-confirmed.** `import_mesh` imports an `ImportedMesh`, then retains only its geometry-only `mesh`. The richer type already includes surface attributes. Two GLBs with the same geometry but different UVs and base colours became identical `.10d` files through `import_mesh → compile_10d_pbr`. This defeats imported asset fidelity before rendering starts.

Owners: [import route](C:/github/hypermedia-production/crates/asset-production-vibe/src/manifold_ops.rs:267), [Mesh and ImportedMesh](C:/github/qualiaDB/crates/qualia-core-db/src/render/assets/mod.rs:74). Repair the retained representation and preserve supported attributes, with explicit refusal for unsupported attributes. Do not silently reduce a rich asset to positions and triangles.

### A02 — P1: material authoring does not uniformly bind to authored meshes

**Source-confirmed; argument refusal reproduced.** `compile_10d_pbr` accepts mesh, albedo, source UV/container data and camera/topology options, but not a BSDF graph, normal/ORM maps, or roughness. Passing `roughness` fails closed. `compile_bsdf_graph` produces a real shader preview and descriptive graph; that does not establish a material binding in the generic mesh compiler.

The important existing exception is `sweep_assembly_profile`: its parts accept transforms, colour, roughness and metallic, and its compiled container attaches MAT1. Use that route now for supported assemblies. Extend existing representations and consumers instead of declaring that the renderer or `.10d` has no materials.

Owners: [generic compiler](C:/github/hypermedia-production/crates/asset-production-vibe/src/manifold_ops.rs:546), [assembly route](C:/github/hypermedia-production/crates/asset-production-vibe/src/manifold_assembly_bim_erosion.rs:16), [BSDF preview](C:/github/hypermedia-production/crates/asset-production-vibe/src/surface_bsdf_uv/bsdf_procedural.rs:14).

### A03 — P1: atlas seams and split topology are not fully exposed

**Closed-mesh failure reproduced; split-topology consequence inferred from source.** A capped box sent to `unwrap_slim(mode="atlas")` fails with “chart lacks a simple boundary; mark seams to make disks”. The wrapper accepts no seam input and passes an empty seam list to the existing QualiaDB atlas algorithm. On paths that split vertices, it builds an atlas mesh internally but returns UVs/container data without a corresponding mesh handle and topology remap. A downstream 1:1 UV consumer cannot safely pair these with the original mesh.

Owner: [atlas wrapper](C:/github/hypermedia-production/crates/asset-production-vibe/src/surface_bsdf_uv/slim_mikktspace.rs:22), native call around line 379. Expose seam selection and the resulting topology together. Preserve fail-closed vertex-count checks. The native atlas algorithm already exists.

### A04 — P1: tangent computation cannot be carried through the general pipeline

**Source-confirmed.** `compute_mikktspace` computes real tangent frames but returns only a bounded sample, counts/metrics and a sphere preview. The generic `.10d` compile path supplies no tangent stream. A preview of a shader sphere cannot validate tangent-space normal mapping on a seam-bearing authored mesh.

Owners: [tangent wrapper](C:/github/hypermedia-production/crates/asset-production-vibe/src/surface_bsdf_uv/slim_mikktspace.rs:494), [compiler](C:/github/hypermedia-production/crates/asset-production-vibe/src/manifold_ops.rs:793). Retain the complete frame stream or seal it into the existing format; test handedness, mirrored UVs and normal-map orientation after reload.

### A05 — P1: two different Vibe invocation hosts look like one tool family

**Reproduced.** MCP `vibe_catalog(prefix="hypermedia.")` reports no matches in its 1,261 bound Poet entries. `vibe_invoke("hypermedia.locus.construct")` reports an unbound seam and suggests adding code. The same Hypermedia route succeeds when invoked inside `synth_produce_present` production source. The operation exists; the selected host is wrong.

Owners: [generic Poet tools](C:/github/qualiaDB/crates/qualia-mcp/src/tools/vibe.rs:177), [production host](C:/github/synth/src/production/source.rs:179). Discovery must state the execution host, accepted IDs and handle lifetime. Either explicitly route asset invocations through an AssetHost session, or direct callers to production source with a precise error. Do not imply that all Poet operations and production operations share a runtime.

### A06 — P1: MCP production cannot ingest the same inputs as the CLI

**Source-confirmed and invalid-argument refusal reproduced.** `synth_produce_present` creates a session with an empty retained-input list and accepts source/profile/output paths, not input bindings. It lacks CLI-equivalent export and native-review controls. A self-contained revolve succeeds; imported reference images or GLBs require a different route. Saving requires `SYNTH_OUTPUT_ROOT`; absent configuration produces a correct, explicit refusal.

Owner: [MCP producer](C:/github/synth/src/mcp_full.rs:239). Add scoped input bindings, export/review access and reproducible receipts. Preserve the output-root boundary. No need to weaken file admission.

### A07 — P1: runtime failures and argument schemas can mislead clients

**Reproduced.** The unbound `vibe_invoke` response contains `status:"error"` but MCP `isError:false`. Other errors collapse to `unknown_or_failed_tool`. `cosmic_coordinates` advertises both `total_z` and `z`, but its HEALPix operation reads only `z`: changing `total_z` from 3 to 900 leaves the tested index unchanged, while `z=900` changes it. The shared schema does not make this operation-specific distinction clear. `computational_geometry(mesh_topology)` uses `vertex_count` and `triangles`, which are absent from its advertised properties.

Owners: [MCP protocol](C:/github/qualiaDB/crates/qualia-mcp/src/authoring/protocol.rs:213), [tool registry](C:/github/qualiaDB/crates/qualia-mcp/src/registry.rs:117), [cosmic implementation](C:/github/qualiaDB/crates/qualia-mcp/src/tools/living_cosmic.rs:310). Generate or test schemas against dispatch; reject unknown keys where meaningful; preserve structured execution errors. Dotted Synth aliases are also rejected by the protocol's exact-name check before reaching backend aliases; publish the actual underscore names.

### A08 — P2: SDK descriptors and prose contracts are not a complete executable API

**Inventory/source-confirmed.** All 148 catalogue/ontology entries exist, but descriptor arrays omit `qualia.species.assemble_corpus` and total 147. The ontology namespace provider intentionally returns a schema hash; calling it is not asset production. Arguments are primarily prose and a shared argument-record descriptor. All `composable_with` names resolve, but that does not prove compatible runtime values or correct record-field extraction.

Owner: [ontology ABI provider](C:/github/hypermedia-production/crates/asset-production-vibe/src/ontology/vibe_descriptors.rs:258). Establish bijections where intended, distinguish metadata calls from execution, and publish typed inputs, outputs, resource fields and executable composition examples.

### A09 — P1: client wiring can select an old catalog or an unusable save path

**Local configuration and binary evidence.** The Synth plugin points at `target/debug/synth.exe`; that binary advertises 144 operations, missing sprite-sheet packing, nine-slice packing, PBR-set packing and species assembly. Current reviewed production binaries advertise 148. Antigravity integration material and a vendored SDK exist. Grok has configuration, skills and an invocation script, but defaults to `target-studio`; the reviewed client configurations do not set `SYNTH_OUTPUT_ROOT`.

Grok's helper guesses output handling by the `hypermedia.manifold.*` prefix. That is unsafe for operations returning records such as assembly outputs, and for non-manifold operations returning handles that still require encoding. Neither namespace nor a JSON argument string is a substitute for a typed recipe.

Owners: [plugin configuration](C:/github/synth/.agents/plugins/synth-qualia/mcp_config.json), [Grok helper](C:/github/synth/vendor/grok/scripts/invoke.ps1:74). The current Codex chat exposes no Synth MCP tools in its tool inventory, although local CLI execution works. Add the requested local Codex adapter and common parity tests; see plan P02. This is a connection gap, not missing local machinery.

### A10 — P1 for animated assets: rig computation is not a complete arbitrary-mesh binding

**Source-confirmed; mesh-argument refusal reproduced.** `rig_biomechanical_archetype` calculates a fixed archetype rig and behaviour outputs, but accepts no authored mesh/weight binding. Its example skin data is not a full vertex stream for an arbitrary mesh. The richer portable-anatomy route does generate joint assignments; the inspected finalizer retains skin arrays in its returned structure but does not seal those through the inspected container path. Turntable/cinema deformation also includes position-based heuristic motion rather than evaluation of a bound skin.

Owners: [rig wrapper](C:/github/hypermedia-production/crates/asset-production-vibe/src/living_world_anatomy/rig_ontology_continuum_frustum.rs:28), [anatomy finalizer](C:/github/qualiaDB/crates/qualia-core-db/src/render/vertebrate_anatomy.rs:2945), [turntable animation](C:/github/hypermedia-production/crates/asset-production-vibe/src/living_world_anatomy/turntable_render.rs:58). Use the existing skeleton, dual-quaternion, IK/NLA and physics owners, but prove bind → seal → reload → animate for the authored asset.

### A11 — P1 for final shots: review rendering is not yet a complete shot interface

**Source/receipt-confirmed.** Native preview uses a real QualiaDB GPU renderer, authored normals/MAT1, SSAO and shadows. The inspected command exposes asset and output, while its review path fixes three cameras, 2048² output and lighting. The receipt explicitly says it is not model acceptance. Environment probes are unset in this path. `solve_sdfgi_froxel` uses real CPU/GPU facilities but its exposed input is an image, not the authored scene's mesh/depth/material ensemble. `build_light_tree` also has its own preview output.

Owners: [preview CLI](C:/github/synth/src/native_preview.rs:52), [native renderer setup](C:/github/qualiaDB/crates/webizen-render/src/volumetric_native_preview.rs:99), [GI wrapper](C:/github/hypermedia-production/crates/asset-production-vibe/src/radiance_compositor_gi/sdfgi_light_tree.rs:283). Expose a durable shot description with camera, lighting/environment, scene membership, render settings and frame range. Existing real-time facilities must be evaluated against final-shot needs, not renamed “cinema” and assumed sufficient.

### A12 — P2: specialist shader names overstate the demonstrated material behaviour

**Source-confirmed.** The BSDF preview genuinely evaluates rendering code. Its `random_walk_sss` branch is an analytic diffuse/rim approximation in the preview, not a demonstrated spatial random walk through an authored mesh. `energy_conserving` is returned as a fixed true value rather than a measured verification. Hair and water previews likewise do not establish a groom or participating medium in a production scene.

Owner: [BSDF/procedural preview](C:/github/hypermedia-production/crates/asset-production-vibe/src/surface_bsdf_uv/bsdf_procedural.rs:232). Etalon already has spectral/subsurface support, including [Burley diffusion weights](C:/github/etalon/crates/etalon-render/src/spectral/basis.rs:165). Bind and validate the needed model, and label approximation and support levels accurately.

### A13 — P1 for scientific terrain: the sculpt result loses authoritative height samples

**Source-confirmed; relief output produced.** `sculpt_terrain` computes floating-point heights but returns relief pixels and summary statistics. `compile_m64` has a separate sculpt path; the erosion wrapper can derive heights from image luminance. A shaded/coloured relief image is not an interchangeable physical elevation field. This creates a data discontinuity between editing, erosion, physical interpretation and rendering.

Owners: [terrain sculpt](C:/github/hypermedia-production/crates/asset-production-vibe/src/geodesy_ops.rs:325), [erosion wrapper](C:/github/hypermedia-production/crates/asset-production-vibe/src/manifold_assembly_bim_erosion.rs:480). Retain heights, units, transform and no-data semantics; use relief only for preview. The fresh erosion mesh and sculpt relief in this audit are separate demonstrations, not proof of a connected heightfield workflow.

### A14 — P2: precision and resource profiles need production-level differentiation

**Source-confirmed bounds.** General raster profiles can reach 2048²; it is incorrect to call the whole system 1024-limited. Other routes have their own limits: UI sheets and map synthesis 1024², material baking up to 256², native review 2048², and the inspected MP4 preview path up to 256² frames. The retained RGBA type is 8-bit; a 16-bit PNG export does not restore discarded source precision. Mesh/work/envelope and Vibe evaluation budgets also bound composition independently of export size.

Owners: [raster profiles](C:/github/hypermedia-production/crates/asset-production-vibe/src/graphics.rs:10), [PBR bake](C:/github/hypermedia-production/crates/asset-production-vibe/src/surface_pbr_brush.rs:15), [production limits](C:/github/synth/src/production/mod.rs:35), [CLI overrides](C:/github/synth/src/asset_cli.rs:84). The current generic geometry guard is 200k vertices / 400k triangles, while some older prose gives smaller boolean limits. Use current source and tested binary bounds. Plan tiled/streamed, scene-linear and LOD routes rather than indiscriminately raising all caps. Prior [GAP_ANALYSIS](C:/github/synth/GAP_ANALYSIS.md) contains additional codec/timing findings; those were not all re-executed here.

### A15 — P1 for finished UI imagery: raster labels are placeholders

**Visually reproduced and source-confirmed.** The same resolved UI produces actual label strings in SVG but rounded label pills in PNG. `render_ui_sheet` derives pill width from string length instead of shaping and drawing glyphs. Theme states and layout geometry work, but the raster sheet cannot be presented as a finished labelled interface.

Owner: [label rasterization](C:/github/hypermedia-production/crates/asset-production-vibe/src/layout_ops/sheet_render.rs:175). Connect existing glyph shaping/font/raster owners to this route. Separately distinguish rendered sheets from functioning applications: input routing and runtime facilities exist, but a sheet alone does not prove focus order, keyboard operation, accessibility or application behaviour.

### A16 — P1 for quality claims: domain labels and previews are not acceptance criteria

**Visual/source evidence.** The fresh wallaby is recognizable, and the botanical leaf has veins and a sectional/histological display. These demonstrate authored structure. The inspected outputs do not establish film close-up topology, fur/skin/leaf microstructure, seamless materials or reference accuracy. The simple assembly fixture deliberately tests data transport; its coarse geometry must not be mistaken for a renderer-quality benchmark.

The anatomy project's existing annotation rates the delivered heart 1/5 and requests apex, atria, ventricular mass and great-vessel roots. The later authored heart source uses loft/extrude/photo projection, so it would be wrong to describe every heart attempt as an ellipsoid. Its existing front, side, three-quarter and photo-projected reviews still show coarse surfaces and abrupt junctions. Those are pre-existing artifacts inspected during the audit, not newly regenerated medical validation.

Owners: [anatomy annotations](C:/github/anatomy/.synth/annotations.json), [authored heart](C:/github/anatomy/master/production/organs/heart.vibe), [portable scaffold](C:/github/hypermedia-production/crates/asset-production-vibe/src/living_world_anatomy/portable_scaffold.rs:22). `compile_vertebrate_anatomy_10d` also has a richer `spec` route; the default comparative scaffold does not describe that whole implementation. Build named assets with appropriate authoring routes and explicit reference review, not a new compiler for every organ.

## Domain readiness and correct routes

| Domain | Existing route to use | Remaining proof needed |
|---|---|---|
| Biology at multiple scales | Existing reconstruction/segmentation/geometry owners, authored manifold operations, EMF field and species/ontology metadata | Scientific-input ingestion, physical units and scale transforms, data-to-geometry provenance, reference-reviewed cell/tissue examples. No fresh microscopic asset certification in this audit. |
| Anatomy | Loft/revolve/booleans/sculpt + assemblies; photo projection when appropriate; portable anatomy specifications | Recognizable anatomical landmarks in several views, continuous junctions, meaningful thickness/cavities, materials and deformation. Ontology labels alone are insufficient. |
| Flora | Botanical organ/compiler routes, whitegum canopy, Murray-law growth, lofts/sweeps/instances | Species-specific morphology, junctions, leaf transmission and bark detail, scene instancing/LOD. The botanical vascular generator is not a human coronary model. |
| Fauna | Authored species specs, layered anatomy, assembly materials, IK/NLA/skin owners | Complete skin binding, eyes/mouth/pelage details, pose and motion acceptance; fresh wallaby render is a baseline, not film approval. |
| Terrain | Sculpt/erosion, retained geometry, geodesy and `.m64` | Lossless elevation composition, units/origin, multi-tile seams, scale-dependent detail and lighting. Heightfield sculpt is 2.5D; manifold/SDF routes supply overhangs where required. |
| Cosmos | Cosmic coordinate/orbital/math facilities; manifold + radiance + electromagnetic fields | Explicit frame/time/units, large-coordinate handling, authored scene/volume bindings, scientifically grounded appearance and shot control. Scalar cosmic replies are not rendered assets. No fresh cosmos shot was certified. |
| Software interfaces | Box layout/themes, SVG, glyph facilities, sprite/nine-slice packing, diegetic surfaces and input routing | Raster text parity, genuine fonts and layout content, responsive/interactive/accessibility tests where the deliverable is an application. |

EMF retains its existing sibling `.emf` field cells, `.m64` occupancy and `.10d` field descriptors. PNG and sound are conversions/previews. This audit does not propose another field format, and did not newly certify the complete EMF movie/audio path.

## Visual review record

Freshly inspected: assembly front/side/three-quarter native PNGs; wallaby complete turnaround; leaf turnaround; UI raster. Assembly blueprint, botanical SVG and UI SVG were also read as exported vector content. The [native receipt](C:/github/synth/docs/reviews/2026-11-12-asset-quality-audit/run-20261112-103345-dcc594/assembly-preview/review-receipt.json) records canonical MAT1, authored normals, absent texture coordinates and renderer-derived tangents for that deliberately simple assembly fixture.

- [Wallaby baseline](C:/github/synth/docs/reviews/2026-11-12-asset-quality-audit/run-20261112-103345-dcc594/fauna/wallaby/wallabia_bicolor_turnaround.png)
- [Botanical baseline](C:/github/synth/docs/reviews/2026-11-12-asset-quality-audit/run-20261112-103345-dcc594/flora/leaf.png)
- [UI label failure](C:/github/synth/docs/reviews/2026-11-12-asset-quality-audit/run-20261112-103345-dcc594/ui/ui.png)
- [Three-quarter native material transport fixture](C:/github/synth/docs/reviews/2026-11-12-asset-quality-audit/run-20261112-103345-dcc594/assembly-preview/three-quarter-view-assumed.png)

The integration suite and deterministic HMC tests pass. The requested across-domain film-quality acceptance gate remains unmet. The implementation plan assigns concrete repairs and demonstrations without treating existing native algorithms as missing.
