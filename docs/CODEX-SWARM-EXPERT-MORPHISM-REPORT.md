# Expert review of remaining gaps and proposed Synth morphisms

The three specialist reviews confirmed **377 canonical IDs in source, with 13 of the original 390 still unwired** at the start of this review. The connected Synth MCP process reported **150 IDs** in both operation and ontology listings. The source inventory is the basis for the proposal checks. Live exposure still needs to be verified against the coordinator’s build. [Implementation ledger](CODEX-SWARM-IMPLEMENTATION-LEDGER.md)

## What the specialists reviewed

| Specialist | Work and finding |
|---|---|
| Catalog and semantic runtime | Independently checked 377 unique descriptors across 22 batches and 1,131 NQuin facts. Compared #115’s range output, the existing selective streamer, project annotations, and LOD workflows. Proposed entry selection by semantic working set, review-anchor migration, and independent LOD verification. |
| Geometry and interaction | Checked the native cutaway decoder, Portal pick tuples, EONT decoding, bounded graph search, and Etalon IK against the catalog. Proposed cutaway metrology, affordance reachability audits, visibility-based camera planning, and cross-rig retargeting. |
| Accessibility and media | Compared keyboard routing, tagged PDF interchange, caption codecs, whole-artifact lineage, and native time types. Proposed keyboard focus authoring, semantic timeline cues, and evidence-fragment links. Flagged trapping focus, rounding exact timestamps, and rebinding ambiguous annotations as cases to test. |

The specialists used `synth_find` to locate nearby capabilities, then checked the current source ontology and dispatch. The items below are candidates for separate review. The expert consultation did not register them or run builds or tests.

## Recommended first additions

| Proposed ID | Input and output | First acceptance check and reuse |
|---|---|---|
| `hypermedia.manifold.measure_semantic_sections` | Digest-pinned standalone SDFB or flat HMC plus authored metre planes → SVG contours, per-occurrence area and perimeter table, component and hole counts, and Q42 receipt | A synthetic annulus keeps its hole and semantic identity at two grid resolutions. Report the actual sampled measurements and resolution. Reuse #82’s cutaway and Q42 lookup paths. This gives reviewers a measurement on top of the existing visual section. |
| `qualia.accessibility.compile_scene_focus_graph` | Admitted HMC plus localized labels, roles, grouping and authored order → focus graph and Q42 bindings to exact scene occurrences | Tab and Shift+Tab visit each active authored target in the intended order and can leave the scene. Keyboard focus and pointer selection resolve to the same occurrence after a supported cutaway change. Reuse the input router and Portal tuple resolution. [W3C Focus Order guidance](https://www.w3.org/WAI/WCAG22/Understanding/focus-order.html) |
| `qualia.container.plan_hmc_semantic_working_set` | Verified HMC, selected IRIs or occurrence selectors, strata or LOD, and byte budget → offline byte-range plan with required dependencies and reported deferrals | A selected leaf’s semantic root and pinned AREF dependencies fit the budget or produce a clear refusal. Check that planning reads no unrelated payload bodies. Reuse the existing selective streamer and #115’s key and digest facts. |
| `qualia.provenance.rebase_hmc_review_anchors` | Two verified HMC revisions and digest-bound review annotations → unchanged, moved, ambiguous, or missing anchors with Q42 evidence | Reordering or renaming a uniquely identifiable leaf preserves its annotation. Duplicate candidates stay unresolved. Reuse project annotations, HMC entry selectors, and the Portal occurrence model. |

## Further ideas

| Proposed ID | Distinct capability and first acceptance check |
|---|---|
| `qualia.accessibility.audit_affordance_equivalence` | Compare authored pointer, keyboard and switch interaction graphs against required EONT action IRIs. Return a shortest trace to a missing action or dead end. Seed a fixture with a pointer-only action and verify the report identifies it without dispatching live events. Reuse EONT and the bounded #211 search. Pair it with the focus compiler, which produces traversal data. |
| `hypermedia.manifold.verify_semantic_lod_transition` | Compare independently authored LODs and report bidirectional sampled distances, missing or ambiguous bindings, and a Q42 receipt. Reject a swapped wall/lumen label even when geometry matches exactly. Reuse CBLD/MAT1 lookup and existing distance kernels. State the sample spacing and tolerance. |
| `hypermedia.chronicle.bind_timed_semantic_cues` | Bind captions, descriptions, or exhibit chapters to pinned HMC occurrences. Export WebVTT and a Q42 cue index. Preserve rational source time, bound the WebVTT quantization error to 1 ms, and verify that seeking resolves the same occurrence. Reuse the native caption codec. Caption alignment and burn-in already belong to #100 and #178. [W3C WebVTT](https://www.w3.org/TR/webvtt1/) |
| `hypermedia.discourse.bind_evidence_fragments` | Link a logical PDF passage to an exact wall, lumen, or signal interval and expose backlinks in Q42 and an annotation export. Reject stale digests and ambiguous text matches. Reuse #238’s logical reading order and HMC occurrence selectors. This lets a reviewer move from a claim to the supporting scene or media fragment. [W3C Web Annotation Data Model](https://www.w3.org/TR/annotation-model/) |
| `hypermedia.radiance.plan_semantic_viewpoints` | Evaluate authored cameras, report visible pixels per occurrence and rank the views by required coverage. Test that a known occluder reduces coverage to zero and that two instances sharing an IRI are counted separately. Portal already carries the right pick tuple. Bulk ID-buffer collection and camera scoring require new work. |
| `hypermedia.kinetics.retarget_semantic_clip` | Map an animation between two admitted SKL2 rigs using authored joint correspondences and rest-axis alignment. Return a clip and per-joint residuals. Verify a same-rig round trip and reject ambiguous joint correspondence. Reuse Etalon PGA motors and constrained FABRIK. |

## What belongs in the current acceptance work

Close the existing runtime gaps under their current IDs or inspection tools. The specialists highlighted four priorities.

1. Run the native and WASM pick → IRI → physical occurrence round trip for flat and nested assemblies, including SDFB wall/lumen, stale picks, and rejected admission.
2. Complete and independently verify #118’s signed grant binding to the mounted HMC, actor, EONT action and capability, and Aura evidence. A caller assertion cannot carry authority.
3. Exercise CBLD/MSTR, HOLO, and the embedded Q42 model together. Measure the 42 MiB reasoning arena and the directed inter-stratum closure acceptance, for which the ledger records no measurements.
4. Verify the original 390-ID target through the production dispatch and complete the planned interchange profiles. Use a package-inspection mode or an audit fixture for the package-wide addressability checks.

The exact 13-ID queue and detailed in-family coverage are maintained in the [implementation ledger](CODEX-SWARM-IMPLEMENTATION-LEDGER.md#production-catalog-queue) and [review handoff](CODEX-SWARM-REVIEW-HANDOFF.md).

## Reusable source entry points

- [Selective HMC range planner](C:/github/qualiaDB/crates/qualia-core-db/src/q42/selective_streamer.rs:153) and [Q42 entry facts](C:/github/qualiaDB/crates/qualia-core-db/src/bundle/semantic_model/packing.rs:159)
- [HMC cutaway decoder](C:/github/qualiaDB/crates/qualia-core-db/src/render/portal_assembly_mesh.rs:218) and [Portal occurrence lookup](C:/github/qualiaDB/crates/qualia-core-db/src/render/portal/sdf_section.rs:79)
- [EONT decoder](C:/github/qualiaDB/crates/qualia-core-db/src/container_10d/entity_ontology_section.rs:213) and [Synth input router](C:/github/hypermedia-production/crates/asset-production-vibe/src/glyph_layout_diegetic/diegetic_input.rs:217)
- [Synth annotation model](C:/github/synth/src/project.rs:53), [WebVTT writer](C:/github/hypermedia-production/crates/chronicle-timeline-interchange/src/captions/vtt.rs:93), and [tagged PDF reader](C:/github/hypermedia-production/crates/asset-production-vibe/src/swarm_formats/document.rs:108)
- [Etalon constrained FABRIK](C:/github/etalon/crates/etalon-render/src/skeletal/ik_motor.rs:398)
