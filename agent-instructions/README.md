# Agentic category instructions

Read [`../AGENTS.md`](../AGENTS.md) first. This directory is the **per-category handbook** so agents load only the morphisms they need. `AGENTS.md` is the index (house rules, MCP tools, DCC mapping). `GEMINI.md` is the Antigravity always-on digest.

Counts must match `OPERATIONS.len() == ontology::MORPHISMS.len() == 150`.

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

Twelve Hypermedia categories plus one `qualia.*` corpus morphism. `15+27+8+30+7+10+16+9+6+7+9+5+1 = 150`.
