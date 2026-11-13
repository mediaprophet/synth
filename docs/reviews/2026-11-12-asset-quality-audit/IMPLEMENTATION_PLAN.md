# Local production quality and client parity — implementation plan

12 November 2026. Prepared from the [asset-production audit](C:/github/synth/docs/reviews/2026-11-12-asset-quality-audit/REPORT.md), including the user's request to cover Antigravity, Grok and ChatGPT/Codex. This is a concrete implementation backlog, not a claim that these changes have already shipped.

## Intended result

An agent working on this laptop can discover the correct operation, supply real project inputs, author and revise an asset, preserve its geometry/surface/animation data, produce a shot, inspect actual outputs, and deliver a deterministic package with a quality receipt. Every client should reach that same local workflow.

Reuse the existing 148 morphisms, QualiaDB codecs/geometry and Etalon providers. Extend existing operation arguments and return values where suitable. Any genuinely necessary new catalog entry must update ontology, nquins, dispatch and operations together. A bad output triggers a specific reproduction and correction, not an invented missing-tool claim or a new generic kernel audit.

## Ordered work packages

| Package | Scope and owning files | Completion evidence |
|---|---|---|
| **P01 — Reliable contracts and routing** | Synth `src/mcp_full.rs`; Qualia MCP `src/authoring/protocol.rs`, `src/registry.rs`, `src/tools/vibe.rs`, `src/tools/living_cosmic.rs`; Hypermedia ontology descriptors. Resolve A05/A07/A08: explicit execution host, structured errors, corrected cosmic/geometry schemas, 148-way descriptor accounting where intended, typed resource/record outputs. | The same valid production recipe succeeds through CLI and production MCP; wrong-host invocation identifies the correct route without suggesting absent code; all real failures set `isError`; schema-driven calls use the actual arguments. Enumerations agree and every documented composition has an executable example. |
| **P02 — Local client adapters and input/output parity** | Synth MCP producer, plugin templates, `vendor/grok`, Antigravity integration examples, proposed `vendor/codex` adapter materials. Resolve A06/A09. Add scoped retained inputs, export/review access and common bootstrap instructions. | Antigravity, Grok and Codex can import the same local fixture, produce matching HMC content, export PNG/SVG/`.10d`, inspect the result and receive the same refusal for invalid input. Version/catalog identity appears in the receipt. |
| **P03 — Preserve complete asset data** | Hypermedia `Artifact`/mesh handling, `manifold_ops.rs`, assembly and surface wrappers; QualiaDB existing imported asset and container types. Resolve A01–A04. | An attributed GLB retains authored normals, UV seams, tangents, material assignments and supported textures after import/seal/reload. Deliberately changing material or UVs changes the relevant output data. Unsupported fields produce explicit diagnostics. Closed-box atlas succeeds with supplied seams and returns topology compatible with downstream compilation. |
| **P04 — Bind materials and define shots** | Existing BSDF/material owners, `.10d` material/texture sections, Synth native preview/presentation and Qualia renderer setup. Resolve A02/A11/A12. | An authored mesh uses colour/normal/ORM and a specified closure after reload. A shot supplies camera, lights/environment, scene membership, output profile and frame range. Tests compare geometry-specific normal mapping and material response; sphere previews alone cannot pass. Approximate SSS/hair/water models are labelled accurately. |
| **P05 — Preserve scientific fields and scale** | Terrain sculpt/erosion/geodesy wrappers, existing `.m64`/`.emf`/field readers and relevant cosmic coordinate interfaces. Resolve A13 plus domain data contracts. | A sculpted f32 heightfield reaches erosion and compilation without relief-image conversion. Units, origin, transforms and no-data survive. Field/mesh co-registration and cosmic frame/time conversions have reference checks. Preview colour changes cannot alter physical data. |
| **P06 — Complete deformation and animation** | Existing anatomy finalizer, rig wrapper, skinning/IK/NLA owners and renderer/container consumers. Resolve A10. | A mesh with a nontrivial skeleton, inverse bind transforms and complete vertex weights is sealed, reloaded and animated. Rest pose matches source; at least two bent poses and a short sequence have inspected deformation. Physics and skinning use consistent transforms. No pose-based silhouette trick substitutes for skin evaluation. |
| **P07 — Finished interface rendering** | `layout_ops/sheet_render.rs`, glyph/font/shaping/raster owners and existing UI runtime/input bridges. Resolve A15. | Raster and SVG render the same real labels, including mixed scripts, wrapping and focus/disabled states. Fonts and asset references are reproducible. Where the deliverable is an app, keyboard actions, focus order, resizing and accessibility are tested in the actual runtime. |
| **P08 — Production profiles and sequence delivery** | Raster storage/colour transforms, bake/encode/mux routes, resource budgets and shot rendering. Resolve A14. | Distinct preview and final profiles declare precision, resolution and resource cost. A scene-linear/high-bit-depth path preserves a gradient beyond 8-bit precision; tiled textures/frames cross tile boundaries without seams. Rational frame timing and colour interpretation survive sequence/container reload. |
| **P09 — Reference assets and acceptance** | `.vibe` recipes and project annotations, review tooling and representative fixtures across all seven domains. Resolve A16 and prove the preceding packages together. | Each domain has an inspected reference asset, source, physical/design assumptions, views/sequence, exported package and recorded review. Accepted quality is tied to the actual delivered shots and resolution, not a blanket “film quality” flag. |

P01 starts first. P02 can begin against existing CLI production while the MCP contract is repaired. P03 is prerequisite to trustworthy P04/P06. P05 and P07 can proceed independently after their contracts are clear. P08 depends on the representation/rendering decisions. Start P09 reference briefs immediately and use them throughout implementation, with final acceptance after the required paths work.

## P02 detail: Antigravity, Grok, ChatGPT and Codex

### Shared local adapter contract

1. Select a known, validated Synth executable and record its hash, build identity, catalog count/hash and enabled capabilities. Do not silently select the old `target/debug` binary because a template names it. Keep live MCP binaries intact; use a separate build directory for upgrades and switch clients deliberately.
2. Give each client the same concise server instructions: discover the asset catalog, execute Hypermedia through the production host, bind inputs explicitly, export encoded resources, and review actual images. Declare the generic Poet host separately. Generate counts/examples from the catalog to replace stale 114/145-operation and 63-tool prose.
3. Add project-scoped input bindings to production MCP with retained keys, kind, digest and size. Add create-only export and preview requests with paths restricted to the configured project/output root. Return actionable errors and resource references clients can actually inspect.
4. Preserve source, input digests, package hash, resource digests and review receipt together. Handle lifetimes must be explicit; a new MCP call must not accidentally refer to a destroyed AssetHost workspace.
5. Provide ordinary PNG/SVG/audio/file results and absolute local links in addition to optional HTML embedding. Antigravity-specific `<agent-embed>` presentation remains an adapter feature, not the only usable output mechanism.

### Antigravity

Keep the existing vendored SDK and `McpStdioServer` integration. Add a minimal tested local example using the selected executable and output root, then run the shared parity harness through that client adapter. Reconcile plugin configuration and documentation with current producer behaviour. A vendored SDK and a successful raw MCP call are not, by themselves, proof of an end-to-end Antigravity workflow.

### Grok

Repair the existing helper instead of replacing the integration. Remove namespace-based guesses about return types. Use declared output fields and encoding recipes; handle record results such as `assembly.container_10d`, images requiring PNG encoding, and multi-operation source with retained inputs. Replace instructions to rebuild into a live target with the isolated-build procedure. Add the same bootstrap and parity harness as the other clients.

### Codex / local ChatGPT desktop integration — explicitly added to this plan

Add repository-owned adapter documentation and a configuration template alongside the existing client material, provisionally under `vendor/codex`. Connect the existing stdio MCP server; a new asset engine or full copied SDK is unnecessary for this connection. If later automation needs Codex SDK/App Server orchestration, add that as a separate consumer of the same adapter, with its own lifecycle test.

For a local Codex host, set the MCP environment explicitly to `local`, select the validated executable, provide the project-specific writable output root, and document the supported startup/tool timeouts. Use the client's normal trust/configuration flow; do not edit user-wide settings or launch a remote executor as an incidental side effect of generating an asset. Local stdio and the `environment_id` setting are supported in the current [official MCP configuration documentation](https://learn.chatgpt.com/docs/extend/mcp).

Test discovery **inside the actual Codex chat** after registration, not only by launching the executable manually. Acceptance requires visible Synth tools, a local retained-input import, production/export/review, a meaningful failure response, and client restart/reconnection without losing the durable outputs.

Keep host distinctions explicit. The app's Codex agent shares configuration with the IDE/CLI; managed ChatGPT Work chats do not simply read the laptop's local Codex files. A template for local Codex therefore must not promise universal ChatGPT connectivity. Any separate ChatGPT product integration needs its own documented supported connection and acceptance test. This plan does not introduce a public tunnel or move production to the cloud. See [official developer settings](https://learn.chatgpt.com/docs/developer-settings).

### Common client acceptance fixture

Use one fixture with an attributed GLB, a small reference PNG, an assembly, a texture/material binding and a UI label. Clients must discover it, bind the same inputs, produce and reopen it, and display the same expected resources. Check exact HMC equality where the producer promises determinism; use bounded visual comparisons for GPU readbacks, which need not be byte-identical across devices. Include expected failures for unknown keys, invalid topology, stale handles and output paths outside the configured root.

The adapter is accepted only after the client can complete these steps. Configuration files alone do not close P02.

## Quality gate for the requested domains

Agree a reference brief per asset: intended scale, hero/secondary/background role, close-up distance, target output size, still or motion, physical/anatomical claims, stylistic intent and acceptable approximations. This can begin with the user's existing anatomy annotations and locally available references; artistic decisions must not be converted into invented technical limitations.

| Demonstration | Required inspection |
|---|---|
| Biology | A cell/tissue specimen with declared scale and provenance; data-to-surface/volume correspondence and multi-scale views. |
| Anatomy | A recognisable heart with apex, atria, ventricular masses and great-vessel roots; front/side/three-quarter plus necessary section views; reference review of relations and junctions. |
| Flora | A species-specific branch/leaf/flower assembly; veins, petiole junction, bark and transmission at the agreed viewing distance; convincing silhouette and detail. |
| Fauna | A complete animal with eyes/mouth/pelage, layered anatomy where requested, and bound animation; inspect rest and bent poses, contact and temporal continuity. |
| Terrain | A terrain patch with retained physical heights, overhangs through appropriate geometry where needed, multi-scale surface detail and tile continuity. |
| Cosmos | A framed planetary/astronomical scene with explicit coordinate/time/scale assumptions and physically motivated illumination/field visualization. |
| Software interfaces | A legible finished screen with real content, typography and component states; for an application, verify its interactive behaviour and accessibility. |

For every asset, inspect actual exported PNG/SVG and the relevant `.10d` native views. For animation, inspect the sequence rather than its first frame. Read project annotations before revisions, then record what changed and whether the issue was resolved. A receipt should link the source/inputs, exact output hash, camera/profile, inspected views, outstanding defects and reviewer decision.

Model correctness, technical validity and visual quality are separate checks. A scientifically meaningful object can have poor shading; a convincing image can contain incorrect anatomy; a deterministic package can preserve either error. All required checks must pass for the intended use.

## Implementation boundaries and first delivery

The first implementation delivery should close P01 and the local CLI-backed portion of P02, reproduce A01/A03/A15 in focused regression tests, then repair their owners in P03/P07. That immediately removes misleading discovery, inaccessible inputs, silent attribute loss and placeholder labels. Continue through the material/shot and deformation work using the reference assets as acceptance cases.

Keep changes focused and reviewable. Preserve the user's existing working-tree changes and live servers. Do not replace existing QualiaDB algorithms, create a second EMF format, relax correctness checks to make demos succeed, or report film-quality readiness from tool counts. The final gate is successful production and inspected output, with any remaining limitation named precisely and supported by a reproduction.
