# Synth + Hypermedia production -- agent contract

Read this file before claiming a tool is missing, before re-auditing the kernel, and before writing a `.vibe`. Antigravity also loads `GEMINI.md` (short always-on digest).

**Category morphism tables live in `agent-instructions/<category>.md`.** This file is the index. Open only the category file you need. Directory index: [`agent-instructions/README.md`](agent-instructions/README.md).

**Webizen AI Synth 0.0.41.12 is the development tool** (produce, preview, compile). QualiaPortal plus the world host is the visitor runtime. Do not mint a per-scene Rust file. Do not treat Synth as the player.

**Find work with `synth_find`.** Pass `q=xpbd`, `q=nla`, `q=ik`, `q=navmesh`, `q=camera`, `q=input`, `q=gis`, `q=osm`, `q=n3`, `q=sparql`, `q=hmc`. CLI: `synth find xpbd`. MCP: `synth_find` (Grok names it `synth__synth_find`). The 150 morphisms are QualiaDB + Etalon + Hypermedia; they are not a third engine.

Synth (`C:\github\synth`) is the producer. Hypermedia (`C:\github\hypermedia-production`) is the studio. QualiaDB owns codecs, `.10d` / `.emf` / `.m64`, SPARQL, N3Logic, SHACL, and CPU geometry libraries. Etalon supplies render/signal/sim providers. A `.vibe` names the work. Author logic as **N3** (N3Logic + SHACL); `synth_compile_q42` seals `.q42`; `query_sparql` runs over admitted Q42 ontologies. Add scene assets as **external HMC / `.10d` / `.q42` files** -- the host does not rebuild. GIS maps bake through `hypermedia.geodesy.*` now; OSM/DEM/STAC ingest is opt-in later via `synth_jobs` and bookmarks `kind=gis`.

**The 150 catalog morphisms exist and dispatch.** They are not stubs. `unimplemented!` / `todo!` are absent from `asset-production-vibe`. If output looks poor, call the morphism that already does that job, then inspect PNG / SVG / `.10d` previews.

Do not tell the user a catalog morphism or a rust-audit-indexed function does not exist. Call `synth_find` first, then read `agent-instructions/<category>.md` and grep `C:\github\hypermedia-production\audit-docs\FUNCTIONS.md`.

---

## First session

1. Confirm product: MCP `synth_capabilities` or CLI `synth capabilities`. Role is development tool, version `0.0.41.12`, 150 morphisms.
2. Search: `synth_find` / `synth find <keyword>`.
3. Open `agent-instructions/<category>.md` for that id (table below). Mapping table in this file is the DCC-name index.
4. Check `synth_operation_contracts` (`id=` or `q=`) for the typed output recipe. Do not guess encoders from a namespace.
5. Author a `.vibe` that calls `capability.invoke("<id>", { ... })`. Produce with `synth_produce_present` (MCP) or `synth produce` (CLI). Default `quality_profile=production16`.
6. Bind retained inputs as `inputs:[{key,kind,path}]` under `SYNTH_INPUT_ROOT`. Outputs are create-only names under `SYNTH_OUTPUT_ROOT`.
7. Inspect PNG / SVG / `front-view-assumed.png` / `side-view-assumed.png` / `three-quarter-view-assumed.png`. Read `{project}/.synth/annotations.json` before changing a model.

`vibe_invoke` / `vibe_catalog` use a generic Poet snapshot. Hypermedia morphisms on that host return `wrong_execution_host`. Use `synth_produce_present`.

## House rules

- Do not kill `C:\github\synth\target-studio\debug\synth.exe` or world_server on port 8800.
- Do not rebuild into `C:\github\synth\target` or `target-studio` (those binaries stay locked). Produce and spawned MCP use `C:\github\synth\target-prod\debug\synth.exe` / `synth-mcp.exe`.
- `--out`, `--export-dir`, `--present`, `preview --out`, MCP `out_hmc` / `export_dir` / `present_html` / `out_q42` are create-only. Unique names every run.
- Two `synth produce` HMCs of the same source must match SHA-256.
- Unknown argument keys fail closed.
- Non-fiction biosphere taxa (`domain=biosphere`, `fiction=false`) need evidence `.q42` before produce (`synth_production_rules`). Encyclopedia sources live in Ecopunk (`C:\github\ecopunk`).
- A named organ is a vibe that lofts / revolves / booleans parts. `hypermedia.manifold.compile_vertebrate_anatomy_10d` emits a comparative ellipsoid.

## MCP tools

Grok prefixes names `synth__`. Antigravity, Codex, and `synth mcp-tools` use the bare names below. QualiaDB graph tools ride the same `synth mcp` process.

| Tool | Use |
|---|---|
| `synth_find` | Keyword search over 150 morphisms + MCP tools + notes. Required `q`. |
| `synth_produce_present` | Production AssetHost. `capability.invoke` inside `source` / `source_path`. |
| `synth_compile_q42` | N3 (preferred) / Turtle / NT -> create-only `.q42`. Then `query_sparql` `vault_path`. |
| `synth_operations` | Paged catalog. `category`, `id`, `q`, `offset`, `limit`, `detail=ids\|summary\|full`. Default 40 summaries. `count` is always 150. |
| `synth_ontology_morphisms` | Same 150 with teleology. Same paging. |
| `synth_ontology_categories` | Twelve categories plus example ids. |
| `synth_operation_contracts` | Typed output recipes / execution host / handle lifetime. |
| `synth_capabilities` | Product card + Vibe/Etalon providers. |
| `synth_profiles` | AssetProducer profiles. |
| `synth_bookmarks` | `kind=sparql\|rdf\|gis\|rest\|csv\|page`. Secrets via `action=set_secret` -> gitignored `.synth/secrets.json`. |
| `synth_jobs` | `kind=fetch\|ingest\|compile` into `.synth/inbox`. Operator ticks due jobs; `synth mcp` does not start the scheduler. |
| `synth_operator_status` | Tray/operator snapshot. `fetch=true` ls-remote tags. Browser `GET /operator`. |
| `synth_production_rules` | Biosphere evidence gate. |
| `synth_update` | `check\|upgrade\|verify\|apply` signed GitHub Releases. Never overwrites a locked exe. |
| `synth_environment_bootstrap` | Tray/studio bootstrap + LLM vault scan. |
| `query_sparql` / `query_graph` / `ingest_ontology` / `parse_rdf` / `parse_json` / `parse_csv` / `serialize_rdf` / `get_graph_stats` / `list_ontologies` | QualiaDB graph on this same server. Internal SPARQL over admitted Q42 ontologies. No public SPARQL HTTP server. |
| `vibe_invoke` / `vibe_catalog` | Generic Poet snapshot. Not the Hypermedia production host. |

`synth mcp` stdio writes need `SYNTH_OUTPUT_ROOT`. HTTP `POST /api/mcp/call` uses the operator process env.

## Client adapters

| Client | Config | Skill |
|---|---|---|
| Grok | `vendor/grok/config.toml` -> project `.grok/config.toml`. Tools `synth__*`. | `vendor/grok/skills/synth-studio/SKILL.md` |
| Antigravity IDE / CLI / App | `.agents/mcp_config.json`, `.agents/plugins/synth-qualia/`, `~/.gemini/config/mcp_config.json`. SDK: `vendor/antigravity/synth_local.py` (`McpStdioServer`). | `.agents/skills/synth-studio/SKILL.md` |
| Codex | `vendor/codex/config.toml`, `vendor/codex/synth-qualia.mcp_config.json` | `vendor/codex/synth-studio.SKILL.md` |

Default command: `C:\github\synth\target-prod\debug\synth-mcp.exe` args `mcp`. Env `SYNTH_INPUT_ROOT=C:\github`, `SYNTH_OUTPUT_ROOT=C:\github\synth\production`. Present HTML uses Antigravity allowlisted Tailwind and `<agent-embed>`.

## CLI

```text
synth find <keyword>
synth capabilities | ontology | operations | profiles | mcp-tools
synth mcp                          # stdio JSON-RPC 2.0
synth produce --profile files --quality-profile production16 --source FILE.vibe --out NEW.hmc --export-dir NEWDIR --present NEW.html
synth preview --asset MESH.10d --out NEWDIR
synth compile is synth_compile_q42 on MCP; CLI uses jobs / operator for N3
synth bookmark add URL [--title T] [--id ID]
synth job add URL | run ID | run-due
synth operator                     # http://127.0.0.1:8787/operator  (tray with --features native-ui)
synth serve --port 8787 --project DIR
synth update check | upgrade | verify | apply
```

This machine:

```powershell
$synth = "C:\github\synth\target-prod\debug\synth.exe"
$stamp = "$(Get-Date -Format 'yyyyMMdd-HHmmss-ffffff')-$PID-$([guid]::NewGuid().ToString('N').Substring(0,8))"
$run = "C:\github\synth\target-prod\runs\$stamp"
& $synth produce --profile files --quality-profile production16 --source <FILE.vibe> --out "$run.hmc" --export-dir $run --present "$run.html"
C:\github\synth\scripts\produce-preview.ps1 -Source <FILE.vibe> -Name revolve-demo
C:\github\synth\vendor\grok\scripts\invoke.ps1 -Morphism hypermedia.manifold.revolve -ArgsJson '{"points":[[14,0],[22,12],[16,30]]}' -Encode mesh -Name revolve-demo
```

## Quality profiles

| Profile | Packed `.10d` ring | Aggregate | Package | Work units |
|---|---|---|---|---|
| `production16` / `authoring` (default) | 128 MiB | 2 GiB | 2304 MiB | 2e9 |
| `preview` | leftover ~64 MiB class | 64 MiB | 80 MiB | conservative |

The 128 MiB ring is the graphics-container exception. It is not the 42 MiB Q42 Sentinel and it is not the p64/LLM VRAM ledger (6 GiB default). Mesh size caps stay 200k verts / 400k tris. Runtime cost uses `hypermedia.manifold.simplify` `mode=optimize`.

---

## Where the inventory lives

| What | Path |
|---|---|
| **Category agent instructions** | [`agent-instructions/`](agent-instructions/README.md) -- one markdown file per category (locus through species). This is the morphism-table source of truth for agents. |
| Vibe-callable catalog (150) | `C:\github\hypermedia-production\crates\asset-production-vibe\src\operations_catalog.rs` (`OPERATIONS`) |
| Ontology, arguments, aliases | `...\src\ontology\morphisms_01.rs` ... `morphisms_09.rs` (`ALL_MORPHISMS`) |
| Dispatch | `...\src\lib.rs` `invoke_inner` |
| Function registry from source | `C:\github\hypermedia-production\audit-docs\` -- **903 files, 8745 functions, 69 modules** (generated 2026-11-10T12:07:10Z by rust-audit `docs`) |
| Crate list | `C:\github\hypermedia-production\CRATES.md` (partial; rust-audit `modules/*.md` is complete) |

Counts must match: `OPERATIONS.len() == ontology::MORPHISMS.len() == 150`.

```
locus 15 + manifold 27 + surface 8 + radiance 30 + glyph 7 + layout 10
+ kinetics 16 + acoustics 9 + electromagnetics 6 + geodesy 7
+ discourse 9 + chronicle 5 + qualia.species.assemble_corpus 1
= 150
```

### rust-audit (local tool, not a crates.io package)

`C:\github\rust-audit` scans a tree into a greppable function registry. It is a workspace on this machine. Do not `cargo publish` it. Building it fetches *dependencies* (`ignore`, `clap`, tree-sitter) from crates.io; that is not publishing rust-audit.

Regenerate Hypermedia docs (writes `audit-docs/INDEX.md`, `FUNCTIONS.md`, `CAPABILITIES.md`, `modules/*.md`):

```powershell
$env:CARGO_TARGET_DIR = 'C:\github\_swarm\target-rust-audit'
$audit = 'C:\github\_swarm\target-rust-audit\debug\audit.exe'
& $audit docs C:\github\hypermedia-production --out C:\github\hypermedia-production\audit-docs
```

**Agent protocol:** before implementing, renaming, or claiming a function does not exist, grep `audit-docs/FUNCTIONS.md` and the matching `audit-docs/modules/*.md`. If it appears, it exists at the listed `path:line` -- read that source. Absence from the index means absence from the tree as of the generation timestamp.

Live list from a built Synth binary: `synth find <keyword>`, `synth operations`, MCP `synth__synth_find`, `synth__synth_operations`, `synth__synth_ontology_morphisms`. Catalog tools are paged (default 40 summaries; pass `category`, `id`, `q`, `offset`, `limit`, `detail=ids|summary|full`). A running MCP binary can be older than this catalog; trust this file and `operations_catalog.rs` over a stale tool description that still says 114. After rebuilding `target-prod`, reload `/mcps` so Grok picks up `synth_find`, `synth_compile_q42`, paged contracts, `synth_operator_status`, `synth_production_rules`, `synth_bookmarks`, `synth_jobs`, and `synth_update`. Operator workbench: `synth operator` (Webizen AI Synth system tray on Windows/macOS/Linux with `--features native-ui`). Aether is the Dioxus-fork toolkit; `Aether.toml` + `src/operator/aether_tray.rs` define the app. `GET /operator`. Tagged installs: `docs/install.md` and GitHub Releases (`synth update verify` / `apply`).

Stale Synth docs that under-count and caused the missing-tool loop:

- `docs/capabilities.md` (7 Nov 2026) -- 18-op host. Superseded.
- `docs/asset-operations.md` -- 114 morphisms. Superseded.

---

## How to invoke

```vibe
requires [capability("capability.invoke")];
let mesh = capability.invoke("hypermedia.manifold.loft", { sections: [...], caps: true });
```

Historical `synth.*` and short `manifold.*` names resolve through `canonical_morphism_id`. CLI, MCP, roots, and create-only rules are in **First session** / **CLI** above.

---

## Familiar DCC names -> morphisms that are already there

| Familiar op | Call this |
|---|---|
| Extrude | `hypermedia.manifold.extrude` |
| Spin / revolve | `hypermedia.manifold.revolve` |
| Loft / bridge | `hypermedia.manifold.loft` (`interpolation=linear\|cubic\|nurbs`) |
| Boolean | `hypermedia.manifold.boolean_3d` (`union\|intersection\|difference`) |
| Bevel | `hypermedia.manifold.bevel` |
| Inset | `hypermedia.manifold.inset` |
| Solidify | `hypermedia.manifold.solidify` |
| Knife / bisect | `hypermedia.manifold.bisect_plane` |
| Clay sculpt / displace | `hypermedia.manifold.evaluate_field` (`expression=clay`, `clay_mode=add\|subtract\|smooth\|pinch`, `brush_center`, `radius`, `strength`, `falloff`) |
| Catmull-Clark | `hypermedia.manifold.subdivide_catmull_clark` |
| Decimate / remesh / optimise | `hypermedia.manifold.simplify` (`mode=decimate\|remesh\|anisotropic\|semantic\|optimize`) |
| Import OBJ/STL/GLB | `hypermedia.manifold.import_mesh` |
| Export DXF | `hypermedia.manifold.project_dxf` |
| Sweep along spine | `hypermedia.manifold.sweep_assembly_profile` |
| SDF remesh | `hypermedia.manifold.mesh_to_sdf_brick` then `sdf_brick_to_mesh` |
| Dupliverts / instances | `hypermedia.manifold.instance_on_points` + `realize_instances` |
| Unwrap / UV | `hypermedia.surface.unwrap_slim` (`mode=slim\|atlas`) |
| MikkT tangents | `hypermedia.surface.compute_mikktspace` |
| Principled BSDF | `hypermedia.surface.compile_bsdf_graph` |
| Bake AO/normal + MaterialX | `hypermedia.surface.bake_pbr_materialx` |
| Photo -> mesh albedo | `hypermedia.manifold.compile_10d_pbr` with `albedo` + `camera_position` |
| UV-space albedo | `compile_10d_pbr` with `albedo` + `source_10d` from `unwrap_slim` |
| Procedural PBR maps | `hypermedia.surface.synthesize_maps` |
| 2D boolean / offset / fillet | `hypermedia.locus.boolean`, `offset`, `fillet` |
| B-spline | `hypermedia.locus.spline` |
| PNG / PSD | `hypermedia.radiance.encode_rgba_png`, `import_rgba_png`, `encode_psd`, `import_psd` |
| Sprite sheet / autotile / 9-slice | `hypermedia.radiance.pack_sprite_sheet` (`mode=grid\|autotile`, `wrap=clamp\|repeat\|mirror`), `pack_nine_slice` |
| PBR set pack | `hypermedia.surface.pack_pbr_set` |
| Grade / LUT / develop | `grade_color`, `apply_lut`, `develop_photo`, `grade_cdl_scopes` |
| EMF volume | `hypermedia.electromagnetics.compile_emf` |
| EMF preview PNG | `render_spectral_slice` or `raymarch_vox0_dda` then `encode_rgba_png` |
| Terrain | `hypermedia.geodesy.sculpt_terrain`, `compile_m64` |
| IK / NLA / XPBD | `hypermedia.kinetics.solve_two_bone_ik`, `evaluate_nla_channelbag`, `step_island_xpbd` |
| Ray / sweep / hit | `hypermedia.kinetics.query_direct_space` |
| NavMesh / ORCA | `hypermedia.kinetics.navigate_funnel_orca` |
| Input map | `hypermedia.layout.route_input_action` |
| Diegetic HUD | `hypermedia.layout.project_diegetic_surface` |
| Camera look-at albedo | `hypermedia.manifold.compile_10d_pbr` `camera_position` / `camera_target` |
| GIS / terrain / m64 | `hypermedia.geodesy.sculpt_terrain`, `compile_m64`, `hypermedia.locus.transcode_geospatial_vector` |
| N3 -> Q42 | MCP `synth_compile_q42` then `query_sparql` |

---

## Produce-path facts

Full produce-path notes, caps, and crate lists live in the category files:

- Camera3D albedo / loft / boolean / clay / simplify -> [agent-instructions/manifold.md](agent-instructions/manifold.md)
- UV / MikkT / MaterialX -> [agent-instructions/surface.md](agent-instructions/surface.md)
- `.emf` Hilbert bricks -> [agent-instructions/electromagnetics.md](agent-instructions/electromagnetics.md)

---

## QualiaDB libraries that exist and are not separate catalog morphisms

Do not tell the user these algorithms are missing from the tree. Do not add new `hypermedia.*` names for them unless the catalog is updated in lockstep (ontology + nquins + `OPERATIONS`).

Under `C:\github\qualiaDB\crates\qualia-core-db\src\specialized_libs\computational_geometry\`:

- `uv_atlas` harmonic charts, `harmonic_parameterize_disk`
- `nurbs_fitting`, `parametric_cad` Bezier / NURBS / SDF primitives
- `remesh_3`, `anisotropic_remesh`
- `heat_geodesics`, `screened_poisson`, `reconstruct_3d`
- `curve_skeleton`, `medial_axis_3d`
- `dual_quaternion_skinning`, `pbd_softbody`
- `boolean_3`, `decimate_3`, `sweep_extrusion`, `authoring` (box/sphere/cylinder/torus)
- `sdf_ops`, `isosurface`, `delaunay_2`, `delaunay_3`, `constrained_delaunay`, `voronoi_2`
- `hull`, `hull_3`, `cdt_retriangulation`, `corefine_3d`, `nary_boolean`, `nary_csg`

Wire those through an **existing** morphism if a vibe needs them.

---

## Hypermedia crates

Catalog morphisms are hosted by `asset-production-vibe`. Each [agent-instructions](agent-instructions/README.md) category file lists the crates that implement it. rust-audit `modules/*.md` is the function-level index.

---

## Category instruction files

`agent-instructions/` is the per-category handbook. Keep this index small (Antigravity always-on digest is `GEMINI.md`, 24KB cap). Do not paste the 150-id tables back into this file.

**How to use them**

1. `synth_find` / `synth find <keyword>` returns ids and a category.
2. Open **only** `agent-instructions/<category>.md`.
3. Read that file's ID + args table, produce-path notes, and crate list.
4. Call `synth_operation_contracts` (`id=` or `q=`) for the typed output recipe.
5. Invoke with `capability.invoke` inside `synth_produce_present`.

**What each file contains**

- Find keywords (`synth_find q=...`)
- Implementing Hypermedia crates
- Full morphism ID + argument table for that category
- Caps, produce-path notes, and familiar-name aliases where they matter
- Cross-links to sibling categories (for example manifold <-> surface for albedo)

Handles are session-local unless noted as retained input keys or encoded resources.

| Category | n | File | Open when | `synth_find` |
|---|---|---|---|---|
| locus | 15 | [agent-instructions/locus.md](agent-instructions/locus.md) | 2D profiles, splines, booleans, GIS vectors, WGS84 sketch | `q=locus` / `gis` / `spline` |
| manifold | 27 | [agent-instructions/manifold.md](agent-instructions/manifold.md) | 3D CAD, `.10d`, booleans, loft/revolve, Camera3D albedo | `q=loft` / `boolean` / `10d` / `camera` |
| surface | 8 | [agent-instructions/surface.md](agent-instructions/surface.md) | UV, MikkT, BSDF, PBR bake, MaterialX | `q=uv` / `pbr` / `bsdf` |
| radiance | 30 | [agent-instructions/radiance.md](agent-instructions/radiance.md) | PNG/PSD, grade, LUT, sprites, SDFGI | `q=png` / `lut` / `sprite` |
| glyph | 7 | [agent-instructions/glyph.md](agent-instructions/glyph.md) | HarfBuzz, MSDF, Knuth-Plass | `q=font` / `harfbuzz` / `msdf` |
| layout | 10 | [agent-instructions/layout.md](agent-instructions/layout.md) | Flex/grid, diegetic HUD, InputMap | `q=input` / `hud` / `flexbox` |
| kinetics | 16 | [agent-instructions/kinetics.md](agent-instructions/kinetics.md) | XPBD, DirectSpace, funnel+ORCA, NLA, IK | `q=xpbd` / `nla` / `ik` / `navmesh` |
| acoustics | 9 | [agent-instructions/acoustics.md](agent-instructions/acoustics.md) | Tone, WAV/AAC, spatial PGA audio | `q=wav` / `aac` / `lufs` |
| electromagnetics | 6 | [agent-instructions/electromagnetics.md](agent-instructions/electromagnetics.md) | `.emf` Q42EMF64, spectral slice, Wi-Fi CSI | `q=emf` / `csi` |
| geodesy | 7 | [agent-instructions/geodesy.md](agent-instructions/geodesy.md) | Terrain, `.m64`, DEM, kriging, GIS bake | `q=gis` / `m64` / `terrain` / `dem` |
| discourse | 9 | [agent-instructions/discourse.md](agent-instructions/discourse.md) | PDF/DOCX/PPTX, charts, telemetry, anatomy pin | `q=pdf` / `pptx` |
| chronicle | 5 | [agent-instructions/chronicle.md](agent-instructions/chronicle.md) | MP4, spatial motors, fiduciary archive | `q=mp4` / `movie` |
| species | 1 | [agent-instructions/species.md](agent-instructions/species.md) | `qualia.species.assemble_corpus` | `q=corpus` |

Twelve Hypermedia categories plus `qualia.species.assemble_corpus`. `locus 15 + manifold 27 + surface 8 + radiance 30 + glyph 7 + layout 10 + kinetics 16 + acoustics 9 + electromagnetics 6 + geodesy 7 + discourse 9 + chronicle 5 + species 1 = 150`.

Paged live list: MCP `synth_operations category=kinetics` (or `q=xpbd`). Default 40 summaries.

---

## Present, QualiaPortal, COEP, wasm-lite

- Present HTML inlines `.10d` and sibling `.emf` as `data-mesh-b64` / `data-emf-b64` so QualiaPortal `load_10d` / `load_emf` do not `fetch("heart.10d")` (that 404s). PNG contact sheets stay data URIs. `related_png` matches a stem only when the rest is empty or starts with `/`, `_`, `.`, or `-` (`working/heart` does not match `working/heartbeat.png`); directory match requires `dir + "/"`.
- Studio static: CORP is always `same-origin`. COOP `same-origin` + COEP `require-corp` apply to HTML, JavaScript, and WASM (SharedArrayBuffer needs COEP on the JS/WASM responses, not only the document).
- `GET /qualia`, `/qualia/`, `/vendor/qualia`, `/vendor/qualia/` serve a QualiaPortal docs index (bounded listing, COI headers). Files under `qualia_docs_root()` (`C:\github\qualiaDB\docs`) at `/qualia/...`. Shell: `/qualia/js/qualia-shell.js`. WASM package: `pkg/qualia/qualia.js`.
- wasm-lite: `GET /synth/wasm-lite.js` (`window.SynthWasmLite`: `composeToOpfs`, `ingestHmc`, `opfsReadPng`; uses `navigator.storage.getDirectory()` when present). `POST /api/radiance/compose` returns PNG via `import_rgba_png` + `compose_rgba` + `encode_rgba_png` (1..=8 layers, 1024^2, ~16 MB body). `POST /api/hmc/bridge` packs in-memory PNGs into an HMC candidate.

## Review before claiming the shape is right

1. Read exported `.png` / `.svg`.
2. Read `front-view-assumed.png`, `side-view-assumed.png`, `three-quarter-view-assumed.png` from each preview dir.
3. Serve the project workbench: `http://127.0.0.1:8787/project`. Keep `http://127.0.0.1:8787/` as the QualiaDB demo studio. QualiaPortal WASM is `http://127.0.0.1:8787/qualia/js/qualia-shell.js`.
4. Read `{project}/.synth/annotations.json` before changing a model.
5. A file existing, HTTP 200, or a triangle count is not a visual review.
