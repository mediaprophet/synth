# Agentic category instructions

Read [`../AGENTS.md`](../AGENTS.md) first. This directory is the **per-category handbook** so agents load only the morphisms they need. `AGENTS.md` is the index (house rules, MCP tools, DCC mapping). `GEMINI.md` is the Antigravity always-on digest.

`OPERATIONS` and `ontology::MORPHISMS` must contain the same IDs (`400` morphisms = `150` baseline + `250` expansion, backed by `1,200` `NQuin48` triples across `58` categories). The per-category tables below describe the 150-operation baseline, and the **Expansion Discovery** section below indexes the 250 expansion morphisms (`WP-01`..`WP-40`). Use `synth_find` (`q=<keyword>`) or `synth_operations` for live discovery of all 400 morphisms and the [implementation ledger](../docs/CODEX-SWARM-IMPLEMENTATION-LEDGER.md) for expansion and audit status.

## Protocol

1. Call MCP `synth_find` / CLI `synth find <keyword>`.
2. Open **only** the matching file in this directory.
3. Read that file's ID + args table, crates, and produce-path notes.
4. Call `synth_operation_contracts` (`id=` or `q=`) for the typed output recipe.
5. Invoke every id with `capability.invoke` inside `synth_produce_present`. `vibe_invoke` is a different Poet snapshot.

## What each file contains

Every category file has the same shape:

- Title with morphism count
- Find keywords (`synth_find q=...`)
- Implementing Hypermedia crates
- Full morphism ID + argument table
- Caps, produce-path notes, and familiar-name aliases where they matter
- Cross-links to sibling categories
- Link back to this README and `../AGENTS.md`

Do not paste these tables into `AGENTS.md`. That file must stay an index so Antigravity's 24KB `GEMINI.md` budget is not blown by 150 ids.

## Files

| File | Category | Morphisms | Open when |
|---|---|---|---|
| [locus.md](locus.md) | locus | 15 | 2D profiles, splines, booleans, GIS vectors, WGS84 sketch |
| [manifold.md](manifold.md) | manifold | 27 | 3D CAD, `.10d`, booleans, loft/revolve, Camera3D albedo |
| [surface.md](surface.md) | surface | 8 | UV, MikkT, BSDF, PBR bake, MaterialX |
| [radiance.md](radiance.md) | radiance | 30 | PNG/PSD, grade, LUT, sprites, SDFGI |
| [glyph.md](glyph.md) | glyph | 7 | HarfBuzz, MSDF, Knuth-Plass |
| [layout.md](layout.md) | layout | 10 | Flex/grid, diegetic HUD, InputMap |
| [kinetics.md](kinetics.md) | kinetics | 16 | XPBD, DirectSpace, funnel+ORCA, NLA, IK |
| [acoustics.md](acoustics.md) | acoustics | 9 | Tone, WAV/AAC, spatial PGA audio |
| [electromagnetics.md](electromagnetics.md) | electromagnetics | 6 | `.emf` Q42EMF64, spectral slice, Wi-Fi CSI |
| [geodesy.md](geodesy.md) | geodesy | 7 | Terrain, `.m64`, DEM, kriging, GIS bake |
| [discourse.md](discourse.md) | discourse | 9 | PDF/DOCX/PPTX, charts, telemetry, anatomy pin |
| [chronicle.md](chronicle.md) | chronicle | 5 | MP4, spatial motors, fiduciary archive |
| [species.md](species.md) | species | 1 | `qualia.species.assemble_corpus` |

## Expansion discovery (`250` expansion morphisms -> `400 / 400` total)

Use the exact ID with `synth_operation_contracts` for the arguments and output handles. Search with `synth_find q=<keyword>` by `sdfb`, `cutaway`, `skl2`, `pack_and_diff_hmc`, `symbolic_cas`, `econometrics`, `blackboard`, `research_and_grounding`, `corridor`, `q42_journal`, `hbbtv`, `solid_pod`, `gtop`, `homogenize`, `advanced_cg`, `ternary`, `semantic_instruments`, `lifecycle_and_abac`, `cardiac_m64`, `passivity`, `bioacoustics`, `datacubes`, `cohort_access_gate`, `portfolio`, `ddg_poisson`, `super_resolve`, `situated_place_time`, `active_learning`, `decompile_merge`, `reconstruct_sar_isar_tomography`, `measure_semantic_sections`, `compile_scene_focus_graph`, `plan_hmc_semantic_working_set`, `rebase_hmc_review_anchors`, `audit_affordance_equivalence`, `verify_semantic_lod_transition`, `bind_timed_semantic_cues`, `bind_evidence_fragments`, `plan_semantic_viewpoints`, or `retarget_semantic_clip`. The [implementation ledger](../docs/CODEX-SWARM-IMPLEMENTATION-LEDGER.md) records branch coverage and outstanding work.

WP-30 through WP-40 search terms include `fresnel`, `signed_bezier_metrics`, `explicit_truss`, `rivers`, `seam_links`, `slic_superpixels`, `preserve_psd_layer_info`, `yin_midi`, `import_ods_sheet`, `allocate_landed_cost`, `section_metrology`, `focus_graph`, `working_set`, `review_anchors`, `affordance_equivalence`, `semantic_lod`, `timed_semantic_cues`, `evidence_fragments`, `semantic_viewpoints`, and `retarget_semantic_clip`.
