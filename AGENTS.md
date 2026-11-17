# Synth + Hypermedia production — agent contract

Read this file before claiming a tool is missing, before re-auditing the kernel, and before writing a `.vibe`.

Synth (`C:\github\synth`) is the producer. Hypermedia (`C:\github\hypermedia-production`) is the studio. QualiaDB owns codecs, `.10d` / `.emf` / `.m64`, and CPU geometry libraries. Etalon supplies render/signal providers. A `.vibe` names the work.

**The 150 catalog morphisms exist and dispatch.** They are not stubs. `unimplemented!` / `todo!` are absent from `asset-production-vibe`. If output looks poor, call the morphism that already does that job, then inspect PNG / SVG / `.10d` previews.

Do not tell the user a catalog morphism or a rust-audit-indexed function does not exist. Grep this file and `C:\github\hypermedia-production\audit-docs\FUNCTIONS.md` first.

---

## Where the inventory lives

| What | Path |
|---|---|
| Vibe-callable catalog (150) | `C:\github\hypermedia-production\crates\asset-production-vibe\src\operations_catalog.rs` (`OPERATIONS`) |
| Ontology, arguments, aliases | `...\src\ontology\morphisms_01.rs` … `morphisms_09.rs` (`ALL_MORPHISMS`) |
| Dispatch | `...\src\lib.rs` `invoke_inner` |
| Function registry from source | `C:\github\hypermedia-production\audit-docs\` — **903 files, 8745 functions, 69 modules** (generated 2026-11-10T12:07:10Z by rust-audit `docs`) |
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

**Agent protocol:** before implementing, renaming, or claiming a function does not exist, grep `audit-docs/FUNCTIONS.md` and the matching `audit-docs/modules/*.md`. If it appears, it exists at the listed `path:line` — read that source. Absence from the index means absence from the tree as of the generation timestamp.

Live list from a built Synth binary: `synth operations`, MCP `synth__synth_operations`, `synth__synth_ontology_morphisms`. Catalog tools are paged (default 40 summaries; pass `category`, `id`, `offset`, `limit`, `detail=ids|summary|full`). A running MCP binary can be older than this catalog; trust this file and `operations_catalog.rs` over a stale tool description that still says 114. After rebuilding `target-prod`, reload `/mcps` so Grok picks up `synth_compile_q42`, paged contracts, `synth_operator_status`, `synth_production_rules`, `synth_bookmarks`, `synth_jobs`, and `synth_update`. Operator workbench: `synth operator` (Webizen AI Synth system tray on Windows/macOS/Linux with `--features native-ui`). Aether is the Dioxus-fork toolkit; `Aether.toml` + `src/operator/aether_tray.rs` define the app. `GET /operator`. Tagged installs: `docs/install.md` and GitHub Releases (`synth update verify` / `apply`).

Stale Synth docs that under-count and caused the missing-tool loop:

- `docs/capabilities.md` (7 Nov 2026) — 18-op host. Superseded.
- `docs/asset-operations.md` — 114 morphisms. Superseded.

---

## How to invoke

VibeScript:

```vibe
requires [capability("capability.invoke")];
let mesh = capability.invoke("hypermedia.manifold.loft", { sections: [...], caps: true });
```

CLI (this machine). Do not kill live MCP. Do not rebuild into `C:\github\synth\target` or `target-studio`.

```powershell
$synth = "C:\github\synth\target-prod\debug\synth.exe"
$stamp = "$(Get-Date -Format 'yyyyMMdd-HHmmss-ffffff')-$PID-$([guid]::NewGuid().ToString('N').Substring(0,8))"
$run = "C:\github\synth\target-prod\runs\$stamp"
& $synth produce --profile files --source <FILE.vibe> --out "$run.hmc" --export-dir $run --present "$run.html"
C:\github\synth\scripts\produce-preview.ps1 -Source <FILE.vibe> -Name revolve-demo
```

`--out`, `--export-dir`, `--present`, and `preview --out` are create-only. Two `synth produce` HMCs of the same source must match SHA-256. Unknown argument keys fail closed (`fields()`). Historical `synth.*` and short `manifold.*` names resolve through `canonical_morphism_id`.

MCP (when connected): `synth__synth_ontology_morphisms`, `synth__synth_operations`, `synth__synth_operation_contracts` (paged; default 40 summaries), `synth__synth_produce_present`, `synth__synth_compile_q42`, `synth__synth_operator_status`, `synth__synth_production_rules`, `synth__synth_bookmarks`, `synth__synth_jobs`, `synth__synth_capabilities`, plus QualiaDB graph tools on the same server (`query_sparql`, `query_graph`, `ingest_ontology`, `parse_rdf`). Search with `search_tool` for `synth`. Compile sourced Turtle/N3 (file under `SYNTH_INPUT_ROOT` or inline `rdf_data`) with `synth_compile_q42` to a create-only `.q42` under `SYNTH_OUTPUT_ROOT`, then `query_sparql` with `vault_path`. Bookmark RDF dumps, SPARQL endpoints, or landing pages with `synth_bookmarks` (`action=upsert` plus later `data_endpoint` / `requires_api_key` / `agent_notes`). Fetch with `synth_jobs` (`kind=fetch|ingest|compile`) into gitignored `.synth/inbox` (PROV/DCT/DCAT wrap, optional `.q42`, max 16 MiB). CLI: `synth job add URL` / `synth job run ID` / `synth job run-due`. The operator ticks due jobs; `synth mcp` does not start the scheduler. Do not invent an HTTP CAS. Public metadata is `.synth/bookmarks.json` and `.synth/jobs.json`; API keys use `action=set_secret` → `.synth/secrets.json` (gitignored). CLI: `synth bookmark add URL`. Biosphere encyclopedia sources live in Ecopunk (`C:\github\ecopunk`). Non-fiction biosphere taxa (`domain=biosphere`, `fiction=false`) require that evidence `.q42` (or `ecopunk/entities/taxa/{taxon}/*.q42`) before `synth_produce_present`; `compile_q42` and graph queries stay allowed. `synth operator` / `GET /operator` is the Webizen AI Synth tray + browser shell (Windows/macOS/Linux; Aether is the toolkit). `synth update` checks tags, and the operator/tray **Upgrade** button (or `synth update upgrade`) downloads the latest signed GitHub Release portable, verifies Ed25519 over SHA-256 (`synth.release.sha256.v1`), and copies to a sibling `synth.new.exe`; it never overwrites a locked exe. The updater seed is gitignored `.synth/update-seed.hex` / GitHub secret `SYNTH_UPDATE_SEED`; the public key is `.synth/update-public.hex`.

A named organ is a vibe that lofts / revolves / booleans parts. Do not call `hypermedia.manifold.compile_vertebrate_anatomy_10d` for a recognisable heart / lung / eye: that morphism emits a comparative ellipsoid.

---

## Familiar DCC names → morphisms that are already there

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
| Catmull–Clark | `hypermedia.manifold.subdivide_catmull_clark` |
| Decimate / remesh | `hypermedia.manifold.simplify` (`mode=decimate\|remesh\|anisotropic`) |
| Import OBJ/STL/GLB | `hypermedia.manifold.import_mesh` |
| Export DXF | `hypermedia.manifold.project_dxf` |
| Sweep along spine | `hypermedia.manifold.sweep_assembly_profile` |
| SDF remesh | `hypermedia.manifold.mesh_to_sdf_brick` then `sdf_brick_to_mesh` |
| Dupliverts / instances | `hypermedia.manifold.instance_on_points` + `realize_instances` |
| Unwrap / UV | `hypermedia.surface.unwrap_slim` (`mode=slim\|atlas`) |
| MikkT tangents | `hypermedia.surface.compute_mikktspace` |
| Principled BSDF | `hypermedia.surface.compile_bsdf_graph` |
| Bake AO/normal + MaterialX | `hypermedia.surface.bake_pbr_materialx` |
| Photo → mesh albedo | `hypermedia.manifold.compile_10d_pbr` with `albedo` + `camera_position` |
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
| IK / NLA / XPBD | `kinetics.solve_two_bone_ik`, `evaluate_nla_channelbag`, `step_island_xpbd` |

---

## Produce-path facts (10 Nov 2026)

### `hypermedia.manifold.compile_10d_pbr`

Implemented in `manifold_ops.rs`. Seals quantized mesh, area-weighted normals, optional SurfaceReading albedo, topology, BVH, UV01.

Albedo sampling order:

1. Godot-style `Camera3D` look-at (`camera_position`, `camera_target`, `camera_up`, `camera_fov`, `camera_near`) for facing, in-frame pixels. Use this for a 3/4 photograph. Do not resample a photograph through auto-unwrap UVs.
2. Authored `source_10d` UV01 (from `unwrap_slim`) when present. Vertex-count mismatch **fails closed**.
3. Auto-`unwrap_slim` when albedo is present and `source_10d` is absent.
4. Spherical centroid UV only when there is no camera and no atlas.

Compiled `.10d` attaches UV01 when an atlas exists. `consume_unwrap_uv01` in `lib.rs` re-seals authored `source_10d` UVs.

### `hypermedia.surface.unwrap_slim`

Flip-avoiding Symmetric Dirichlet / Tutte + local SLIM in `surface_bsdf_uv/slim_mikktspace.rs`. Returns `uvs`, `container_10d` (UV01-sealed), SVG chart. Open meshes get a boundary loop; closed meshes start from a spherical chart then SLIM. BTree adjacency (produce-safe).

`unwrap_slim` `mode=slim` (default) keeps 1:1 vertex UV01. `mode=atlas` calls QualiaDB `uv_atlas::unwrap_atlas_into` (harmonic disk charts, **explicit seams**, may split vertices). Atlas fail-closes when a 1:1 UV01 consume is required. Do not invent `hypermedia.surface.uv_atlas`.

### `hypermedia.surface.bake_pbr_materialx`

BVH ray-cast AO + world-normal maps, OpenPBR MaterialX, PSD roundtrip (`surface-pbr-studio` + `surface-layered-codecs`). Uses authored `uvs` or auto-`unwrap_slim`; planar XY is last fallback.

### Electromagnetics: values live in `.emf`

The durable field is sibling **`Q42EMF64`** (`.emf`), not a second mesh format and not a new `.10d` magic.

Each cell is 64 bytes: **E** (V/m), **B** (T), Poynting flux, `wavelength_nm`, n/κ, coherence phase, Stokes Q/U/V, `spectral_sigma` (σ), `confidence_alpha`. Chunks are 8³ Hilbert bricks (512 cells, 32 KiB) with directory rows (spectral band mask, time slice, Bao prefix). Writer: `EmfContainerWriter`. Reader: `EmfContainerReader`. Seal path: occupancy → sibling `.m64` + `.emf` + `.10d`.

`.10d` **supports** that field via MID2 `FIELD_KIND_ELECTROMAGNETIC` (`0x0100`) plus FIEL descriptors (`electromagnetic.e_field`, `electromagnetic.spectrum`) in FieldSidecar / HyperspectralCube. Those records name the field; they do not duplicate the brick cells.

RGBA and sonics are **conversions of the same cells**:

- **RGBA** — `hypermedia.electromagnetics.render_spectral_slice` / `raymarch_vox0_dda` (CIE 1931 + Bruton from `wavelength_nm` and vacuum-impedance irradiance `|E|²/Z0`). Encode with `encode_rgba_png`. This is a preview, not a replacement for `.emf`.
- **Sonics** — QualiaPortal AcousticPlane maps the same σ (`spectral_sigma` / Tensor10D σ) to Hz (`sigma_to_center_frequency_hz`: 400–700 nm → 1760–110 Hz). There is no separate EMF-to-WAV store. STFT/CQT sidecars are acoustic bake of that σ, not a third field.

`compile_emf` seals `.emf`. `verify_emf` checks CRC-32C + Bao. Do not invent `.10d` EMF cell sections. Portal `load_10d` is the mesh. Portal `load_emf` unpacks sibling `.emf` Hilbert bricks onto the GPU field and sets AcousticPlane σ from the same cells. `dense_volume()` fail-closes above `MAX_DENSE_CELLS` (1_048_576). The QualiaPortal WASM package (`docs/pkg/qualia/qualia.js`) exports `QualiaPortal.load_emf`.

### Caps (honest bounds, not absences)

- `evaluate_field` clay brush: `expression=clay` with `clay_mode=add|subtract|smooth|pinch` (add/subtract/smooth along normals; pinch lerps toward `brush_center`). Other expressions remain `radial_wave|height_gradient|curvature_proxy|normal_displace`.
- `simplify` `mode=anisotropic` uses a per-vertex isotropic(local mean edge) background metric by default; optional `target_edge` or SPD `metric:[m00,m10,m11,m20,m21,m22]` (indefinite metric fail-closes)
- `boolean_3d` ~50k verts / 100k tris
- loft 2..=256 sections, identical point counts; `interpolation=linear\|cubic\|nurbs`
- revolve ≤256 segments; optional `interpolation=nurbs`
- `subdivide_catmull_clark` levels 1..=3, abort above 200k tris
- `synthesize_maps` max 1024²; typical radiance 1024²; envelopes ~16 MB
- geodesy sculpt is 2.5D heightfield
- `fspm_vascular_grow` is botanical Murray-law trees (generations 1..=6), not human coronaries
- `compile_vertebrate_anatomy_10d` is an ellipsoid comparative scaffold

---

## QualiaDB libraries that exist and are not separate catalog morphisms

Do not tell the user these algorithms are missing from the tree. Do not add new `hypermedia.*` names for them unless the catalog is updated in lockstep (ontology + nquins + `OPERATIONS`).

Under `C:\github\qualiaDB\crates\qualia-core-db\src\specialized_libs\computational_geometry\`:

- `uv_atlas` harmonic charts, `harmonic_parameterize_disk`
- `nurbs_fitting`, `parametric_cad` Bézier / NURBS / SDF primitives
- `remesh_3`, `anisotropic_remesh`
- `heat_geodesics`, `screened_poisson`, `reconstruct_3d`
- `curve_skeleton`, `medial_axis_3d`
- `dual_quaternion_skinning`, `pbd_softbody`
- `boolean_3`, `decimate_3`, `sweep_extrusion`, `authoring` (box/sphere/cylinder/torus)
- `sdf_ops`, `isosurface`, `delaunay_2`, `delaunay_3`, `constrained_delaunay`, `voronoi_2`
- `hull`, `hull_3`, `cdt_retriangulation`, `corefine_3d`, `nary_boolean`, `nary_csg`

Wire those through an **existing** morphism if a vibe needs them.

---

## Hypermedia crates (what is in `C:\github\hypermedia-production\crates`)

Catalog morphisms are hosted by `asset-production-vibe` and implemented across these crates. rust-audit `modules/*.md` is the function-level index.

| Crate | Role |
|---|---|
| `asset-production-vibe` | 150 morphisms, ontology, dispatch, produce-path |
| `profile-authoring` | circle / rounded-rect / channel / polygon profiles |
| `drafting-geometry` | 2D curves, bulged polylines, tessellation |
| `vector-locus-geometry` | locus geometry |
| `vector-locus-pathops` | Pathfinder booleans, Live Corners, warp envelopes |
| `locus-parametric-solver` | sketch constraints, WGS84 ruler |
| `locus-geospatial-codecs` | WKB / KML / LandXML / PMTiles |
| `manifold-bim-csg` | IFC4 STEP, wall/window/stair CSG |
| `surface-pbr-studio` | PBR bake, MaterialX |
| `surface-layered-codecs` | PSD layer roundtrip |
| `vector-surface-brush` | calligraphic / bristle / mixer brush |
| `vector-cpu-render-adapter` | CPU vector raster |
| `vector-gpu-render-adapter` / `vector-hybrid-*` / `vector-compute-*` / `vector-skia-render-adapter` | GPU/hybrid/Skia vector backends |
| `vector-glyph-shaping` | HarfBuzz/BiDi, Knuth–Plass, MSDF |
| `vector-radiance-trace` | contour trace, seam carving |
| `color-transforms` | LUT, grade, transfer |
| `radiance-camera-raw` | CFA demosaic, HDR fusion |
| `radiance-color-grading` | ASC CDL, scopes |
| `raster-blend-operators` | burn/dodge/vivid-light/hard-mix |
| `raster-layer-mask-adapter` | layer masks |
| `raster-selection-morphology` | expand/contract/border |
| `raster-spatial-operators` | spatial raster ops |
| `terrain-brushes` | CoW heightfield strokes |
| `terrain-erosion-solvers` | crater / terrace / hydraulic erosion |
| `geodesy-spatial-analysis` | kriging, morphometry |
| `electromagnetics-rf-sensing` | Wi-Fi CSI vitals / AoA |
| `layout-box-autolayout` | flexbox / grid |
| `layout-paged-typesetter` | paged foliation, math |
| `layout-nodegraph-compiler` | dataflow nodegraph |
| `layout-presentation-deck` | deck morph frames |
| `kinetics-continuum-multiphysics` | FEM / LBM |
| `kinetics-garment-draping` | orthotropic cloth |
| `kinetics-block-simulation` | causal block diagrams |
| `kinetics-motion-compositor` | motion tracks |
| `kinetics-schedule-cpm` | CPM / EVM / Monte Carlo |
| `acoustics-daw-session` | EBU R128 / Camelot DAW |
| `audio-aac-codec` / `audio-opus-codec` / `audio-device-cpal` / `audio-timebase` / `pcm-device-output` | audio codecs and devices |
| `discourse-academic-tex` / `discourse-pdf-compliance` / `discourse-scientific-matlang` | TeX, PDF/UA, MatLang |
| `chronicle-tabular-engine` / `chronicle-fiduciary-ledger` / `chronicle-office-codecs` / `chronicle-timeline-interchange` / `chronicle-broadcast-containers` | spreadsheet, ledger, OTIO, Matroska |
| `video-h264-codec` / `video-h264-preview` / `video-prores-codec` / `media-isobmff` / `media-ogg` | video/media |
| `gpu-device-context` / `gpu-command-profiler` / `pixel-buffer-output` / `pixel-window-render-adapter` | GPU/window |
| `bitstream-codec` / `multimodal-synthesis-schema` / `timeline-editing` | shared codecs / schema / time |

---

## Twelve categories — 150 morphisms

Arguments are the ontology `arguments:` strings. Handles are session-local unless noted as retained input keys or encoded resources.

### locus (15)

| ID | Args |
|---|---|
| `hypermedia.locus.construct` | `shape`: circle(`radius`,`segments`), rectangle(`width`,`height`,`radius`), channel(`width`,`height`,`wall`), polygon(`points`) |
| `hypermedia.locus.transform` | `profile`, `matrix:[a,b,c,d,e,f]` |
| `hypermedia.locus.project_svg` | `profile`, `fill:[r,g,b,a]`, `padding` |
| `hypermedia.locus.spline` | `points` 3..=1024, `closed`, `tolerance` |
| `hypermedia.locus.offset` | `profile`, `distance`, `miter_limit` |
| `hypermedia.locus.boolean` | `a`, `b`, `op=union\|intersection\|difference\|xor` |
| `hypermedia.locus.stroke_to_locus` | `points`, `width`, `cap`, `join` |
| `hypermedia.locus.fillet` | `profile`, `radius`, `segments`, `closed` |
| `hypermedia.locus.resample` | `profile`, `mode=count\|length\|curvature`, `count` |
| `hypermedia.locus.trim` | `profile`, `start`, `end` |
| `hypermedia.locus.bishop_rmf_cyclic` | `path:[[x,y,z],...]`, `closed` |
| `hypermedia.locus.shortest_edge_path` | `mesh`, `start_vertex`, `end_vertex` |
| `hypermedia.locus.solve_parametric_sketch` | sketch constraints + WGS84 ruler (`target_length`, `mtext`, `lat1_deg`, `lon1_deg`, `lat2_deg`, `lon2_deg`) |
| `hypermedia.locus.pathfinder_warp_envelope` | `width`, `height`, `bend`, `corner_radius` |
| `hypermedia.locus.transcode_geospatial_vector` | `lon`, `lat`, `zoom` |

### manifold (26)

| ID | Args |
|---|---|
| `hypermedia.manifold.extrude` | `profile`, `path:[[x,y,z],...]`, `caps` |
| `hypermedia.manifold.revolve` | `points:[[radius,height],...]`, `segments`, optional `interpolation=linear\|nurbs` |
| `hypermedia.manifold.loft` | `a`,`b` or `sections:[[ring],...]`, `interpolation=linear\|cubic\|nurbs`, `samples`, `caps` |
| `hypermedia.manifold.compile_10d` | `mesh` |
| `hypermedia.manifold.compile_10d_pbr` | `mesh`, optional `albedo`, `source_10d`, `topology`, `spatial_index`, `camera_*` |
| `hypermedia.manifold.boolean_3d` | `a`, `b`, `op=union\|intersection\|difference` |
| `hypermedia.manifold.simplify` | `mesh`, `target_triangles`, `max_error`, `mode=decimate\|remesh\|anisotropic`, optional `target_edge` or `metric:[6]` |
| `hypermedia.manifold.import_mesh` | `input` retained key, `format=obj\|stl\|glb` |
| `hypermedia.manifold.project_dxf` | `profile` and/or `mesh`, `layer` |
| `hypermedia.manifold.radial_euler` | `mesh`, Radial-Edge ops SEMV/SFME/JEKV/JVKE/JFKE |
| `hypermedia.manifold.bevel` | `mesh`, `width`, `segments` |
| `hypermedia.manifold.inset` | `mesh`, `thickness`, `depth` |
| `hypermedia.manifold.solidify` | `mesh`, `thickness`, `offset` |
| `hypermedia.manifold.bisect_plane` | `mesh`, `plane_origin`, `plane_normal`, caps |
| `hypermedia.manifold.subdivide_catmull_clark` | `mesh`, `levels` 1..=3, `crease` |
| `hypermedia.manifold.evaluate_field` | `mesh`, `domain`, `expression=radial_wave\|height_gradient\|curvature_proxy\|normal_displace\|clay`, `amplitude`, clay: `clay_mode`, `brush_center`, `radius`, `strength`, `falloff` |
| `hypermedia.manifold.mesh_to_sdf_brick` | `mesh`, `resolution` 8..=32, `padding` |
| `hypermedia.manifold.sdf_brick_to_mesh` | `sdf`, `iso_level` |
| `hypermedia.manifold.instance_on_points` | `prototype`, `points`, `scale` |
| `hypermedia.manifold.realize_instances` | `instances` record |
| `hypermedia.manifold.fspm_vascular_grow` | `generations` 1..=6, Murray-law botanical |
| `hypermedia.manifold.sweep_assembly_profile` | 2D profile + 3D spine, caps |
| `hypermedia.manifold.compile_bim_ifc4` | wall/window/stair → IFC4 STEP |
| `hypermedia.manifold.erode_geomorphic_heightfield` | heightfield erosion |
| `hypermedia.manifold.compile_botanical_organ_10d` | botanical scale 1..=4 (not a human organ) |
| `hypermedia.manifold.compile_vertebrate_anatomy_10d` | ellipsoid comparative scaffold |

### surface (8)

| ID | Args |
|---|---|
| `hypermedia.surface.synthesize_maps` | `side` 8..=1024, `seed`, `tint`, `roughness`, `metallic` → `{colour,normal,orm}` |
| `hypermedia.surface.unwrap_slim` | `mesh`, `iterations` 8..=64, `chart_padding` 0..=0.25, `mode=slim\|atlas` |
| `hypermedia.surface.compute_mikktspace` | `mesh`, optional `uvs` |
| `hypermedia.surface.compile_bsdf_graph` | `closure=principled_v2\|chiang_hair\|fournier_forand_water\|random_walk_sss`, PBR params |
| `hypermedia.surface.sample_procedural_field` | `basis=gabor\|voronoi\|wave\|blackbody\|wavelength` |
| `hypermedia.surface.bake_pbr_materialx` | `mesh`, `resolution`, `samples`, `material_name`, optional `uvs` |
| `hypermedia.surface.expand_procedural_brush` | `stroke_weight`, `bristle_count`, `wetness` |
| `hypermedia.surface.pack_pbr_set` | `colour`, `normal`, `orm` → encoded PNG handles |

### radiance (30)

| ID | Args |
|---|---|
| `hypermedia.radiance.rasterize_loci` | `width`, `height`, `background`, `layers:[{profile,fill,matrix}]` |
| `hypermedia.radiance.convolve_blur` | `image` |
| `hypermedia.radiance.blend_rgb` | `image`, `source`, `mode=burn\|dodge\|vivid-light\|hard-mix` |
| `hypermedia.radiance.transform_exposure` | `image`, `stops` |
| `hypermedia.radiance.morphology_mask` | `image`, `mode=expand\|contract\|border`, `radius`, `threshold` |
| `hypermedia.radiance.encode_rgb_png` | `image` |
| `hypermedia.radiance.import_rgb_png` | `input` retained opaque RGB8 PNG |
| `hypermedia.radiance.render_vector` | `width`, `height`, SVG path layers, fills, strokes, clips |
| `hypermedia.radiance.create_rgba` | `width`, `height`, `color` |
| `hypermedia.radiance.compose_rgba` | `layers` with opacity, blend mode, offset, mask |
| `hypermedia.radiance.filter_rgba` | `image`, `operation=blur\|sharpen\|hue\|emboss` |
| `hypermedia.radiance.encode_rgba_png` | `image`, optional `profile=ruled\|8-bit\|1024\|16-bit\|2k` |
| `hypermedia.radiance.import_rgba_png` | `input`, optional `profile` |
| `hypermedia.radiance.modify_mask` | `image`, `mode=expand\|contract\|border`, `channel=alpha\|luma` |
| `hypermedia.radiance.apply_lut` | `image`, `preset` or `cube_text`, `intensity` |
| `hypermedia.radiance.grade_color` | `image`, ASC-CDL lift/gamma/gain, `working_space` |
| `hypermedia.radiance.convert_transfer` | `image`, `source_transfer`, `target_transfer` |
| `hypermedia.radiance.develop_photo` | `image`, exposure/WB/highlights/shadows/clarity/vibrance/denoise |
| `hypermedia.radiance.encode_psd` | `width`, `height`, `layers:[{name,image,opacity}]` |
| `hypermedia.radiance.import_psd` | `input`, optional `layer_index` |
| `hypermedia.radiance.filter_kuwahara_anisotropic` | `image`, `radius`, `sharpness`, `eccentricity` |
| `hypermedia.radiance.decode_cryptomatte_q42` | `image`, `target_id`, luma range, `feather` |
| `hypermedia.radiance.vector_motion_blur` | `image`, `velocity_px`, `samples`, `shutter` |
| `hypermedia.radiance.solve_sdfgi_froxel` | `image`, cascades, fog, sun, sky, `backend=cpu\|gpu` (gpu = Etalon/wgpu cascade probes + GGX; fail-closed without adapter) |
| `hypermedia.radiance.build_light_tree` | `lights:[{position,direction,color,power,angle_rad}]`, `width`, `height` |
| `hypermedia.radiance.develop_camera_raw_hdr` | Bayer/X-Trans CFA + exposure brackets |
| `hypermedia.radiance.grade_cdl_scopes` | ASC CDL v1.2, 3-way wheels, LogC4, LUT, waveform/vectorscope |
| `hypermedia.radiance.trace_vector_contours` | RGBA8 → cubic Bézier contours, seam carving, skeleton |
| `hypermedia.radiance.pack_sprite_sheet` | `tiles`, `columns`, `padding`, optional `names`, `mode=grid\|autotile`, `wrap=clamp\|repeat\|mirror` → `{image,png,frames,mode,wrap}` |
| `hypermedia.radiance.pack_nine_slice` | `image`, `left`, `right`, `top`, `bottom`, `dest_width`, `dest_height` |

### glyph (7)

| ID | Args |
|---|---|
| `hypermedia.glyph.measure_bounds` | `text`, `font_size`, wrapping |
| `hypermedia.glyph.shape_to_svg_path` | `text`, origin, `font_size`, tracking |
| `hypermedia.glyph.project_svg` | shaped text run → SVG resource |
| `hypermedia.glyph.rasterize_rgba` | shaped text run → RGBA8 |
| `hypermedia.glyph.shape_harfbuzz_bidi` | `text`, `base_direction`, `script` |
| `hypermedia.glyph.pack_msdf_font_atlas` | `charset`, `cell_size`, `px_range` |
| `hypermedia.glyph.compose_knuth_plass_paragraph` | `text`, `frame_width`, `frame_height` |

### layout (10)

| ID | Args |
|---|---|
| `hypermedia.layout.solve_box_tree` | viewport, flex/grid `boxes` |
| `hypermedia.layout.bind_theme_scope` | `scope`, `mode`, accent/surface tokens |
| `hypermedia.layout.render_ui_sheet` | `layout`, `theme` → RGBA8 |
| `hypermedia.layout.project_ui_svg` | `layout`, `theme` → SVG |
| `hypermedia.layout.project_diegetic_surface` | UI RGBA + 3D quad pose + pick ray |
| `hypermedia.layout.route_input_action` | hardware event + HUD rects + InputMap |
| `hypermedia.layout.synchronize_companion_frustum` | dome `rows`×`cols`, FOV, Lamport frame |
| `hypermedia.layout.solve_flexbox_grid_constraints` | flex/grid container + Hug/Fill children |
| `hypermedia.layout.typeset_paged_foliation` | multi-column, LaTeX math, CSL bibliography |
| `hypermedia.layout.compile_dataflow_nodegraph` | typed NodeNetwork + morph frame |

### kinetics (15)

| ID | Args |
|---|---|
| `hypermedia.kinetics.sample_frames` | `numerator`, `denominator`, `count` |
| `hypermedia.kinetics.solve_two_bone_ik` | `root`, `target`, `pole`, bone lengths |
| `hypermedia.kinetics.interpolate_keyframes` | `keyframes`, `fps`, `easing` |
| `hypermedia.kinetics.export_lottie` | vector layers → Bodymovin JSON |
| `hypermedia.kinetics.interpolate_motor_fti` | prev/curr PGA motors, `alpha`, `teleport` |
| `hypermedia.kinetics.step_island_xpbd` | rigid `bodies`, Area3D overrides, substeps |
| `hypermedia.kinetics.query_direct_space` | colliders, ray/sweep, collision mask |
| `hypermedia.kinetics.navigate_funnel_orca` | NavMesh portals, agent, ORCA neighbors |
| `hypermedia.kinetics.evaluate_nla_channelbag` | ActionChannelbag strips, LensTween |
| `hypermedia.kinetics.step_simulation_zone` | point cloud, curl-noise/attractor |
| `hypermedia.kinetics.rig_biomechanical_archetype` | fauna archetype, speed, turn rate |
| `hypermedia.kinetics.solve_continuum_fem_lbm` | elastic params, D3Q19 LBM |
| `hypermedia.kinetics.drape_orthotropic_garment` | textile + pinned vertices |
| `hypermedia.kinetics.simulate_causal_block_diagram` | causal blocks, RK4 |
| `hypermedia.kinetics.schedule_cpm_evm_montecarlo` | task network, trials, Gantt SVG |

### acoustics (9)

| ID | Args |
|---|---|
| `hypermedia.acoustics.synthesize_tone` | `frequency`, `sample_rate`, `frames`, `amplitude` |
| `hypermedia.acoustics.scale_gain` | `audio`, `gain` |
| `hypermedia.acoustics.encode_wav` | `audio` |
| `hypermedia.acoustics.filter_biquad` | `audio`, `mode=lowpass\|highpass\|bandpass\|notch\|peaking` |
| `hypermedia.acoustics.mix_tracks` | `tracks` with gain/offset/fade |
| `hypermedia.acoustics.encode_aac` | `audio`, `bitrate` |
| `hypermedia.acoustics.route_bus_dag` | multi-bus DAG, biquad/compressor/reverb |
| `hypermedia.acoustics.spatialize_pga_zone` | `audio`, source/listener PGA poses |
| `hypermedia.acoustics.analyze_ebu_r128_camelot_daw` | LUFS, YIN/MIDI, VBAP, Camelot, timecode |

### electromagnetics (6)

| ID | Args |
|---|---|
| `hypermedia.electromagnetics.compile_emf` | `origin`, `voxel_size`, `grid_dims`, `sources` |
| `hypermedia.electromagnetics.verify_emf` | `emf` |
| `hypermedia.electromagnetics.render_spectral_slice` | `emf`, `axis`, `slice_index`, `width`, `height`, `exposure` |
| `hypermedia.electromagnetics.raymarch_vox0_dda` | `emf`, `width`, `height`, `deflection_mass`, `step_limit` |
| `hypermedia.electromagnetics.project_metamer_nullspace` | `spd_41`, `nullspace_scale` |
| `hypermedia.electromagnetics.sense_wifi_csi_vitals_aoa` | `breathing_hz`, `heart_hz`, `carrier_ghz` |

### geodesy (7)

| ID | Args |
|---|---|
| `hypermedia.geodesy.sculpt_terrain` | `tile_resolution`, `strokes`, `width`, `height` |
| `hypermedia.geodesy.compile_m64` | geodetic origin, strata/holo/point |
| `hypermedia.geodesy.verify_m64` | `m64` |
| `hypermedia.geodesy.advect_m64_grid` | Poisson + MacCormack + Eikonal |
| `hypermedia.geodesy.couple_continuum_xpbd` | Bekker-Wong/Mohr-Coulomb contact |
| `hypermedia.geodesy.step_sector_continuum_3d` | `sector_id`, `steps` |
| `hypermedia.geodesy.kriging_variogram_morphometry` | WGS84 points, variogram, DEM |

### discourse (9)

| ID | Args |
|---|---|
| `hypermedia.discourse.review_gallery` | `title`, `files:[{name,resource}]` |
| `hypermedia.discourse.compile_pdf` | pages, headings, vector shapes |
| `hypermedia.discourse.compile_prose_docx` | sections, paragraphs, tables |
| `hypermedia.discourse.compile_deck_pptx` | slides with titles and bullets |
| `hypermedia.discourse.render_chart_svg` | `chart_type=bar\|line\|area\|scatter` |
| `hypermedia.discourse.inspect_engine_telemetry` | 256-frame ring-buffer, VRAM ledger |
| `hypermedia.discourse.verify_fiduciary_rid` | resource DID, AgreementDID, deontic mode |
| `hypermedia.discourse.query_anatomical_ontology_pin` | biological scale, picking ray |
| `hypermedia.discourse.audit_tex_pdf_ua_matlang` | TeX, PDF/UA, M-code, PII redaction |

### chronicle (5)

| ID | Args |
|---|---|
| `hypermedia.chronicle.mux_mp4` | `frames`, `fps` |
| `hypermedia.chronicle.decode_mp4_frame` | `source`, `frame_index` |
| `hypermedia.chronicle.capture_deterministic_movie` | RGBA frames + audio + locked-dt |
| `hypermedia.chronicle.sync_spatial_motors` | entity PGA motors, 60 Hz packets |
| `hypermedia.chronicle.recalculate_tabular_fiduciary_archive` | spreadsheet + journal + OTIO/ProRes |

### species (1)

| ID | Args |
|---|---|
| `qualia.species.assemble_corpus` | `corpus` record or `"corpus.q42"` |

---

## Present, QualiaPortal, COEP, wasm-lite

- Present HTML inlines `.10d` and sibling `.emf` as `data-mesh-b64` / `data-emf-b64` so QualiaPortal `load_10d` / `load_emf` do not `fetch("heart.10d")` (that 404s). PNG contact sheets stay data URIs. `related_png` matches a stem only when the rest is empty or starts with `/`, `_`, `.`, or `-` (`working/heart` does not match `working/heartbeat.png`); directory match requires `dir + "/"`.
- Studio static: CORP is always `same-origin`. COOP `same-origin` + COEP `require-corp` apply to HTML, JavaScript, and WASM (SharedArrayBuffer needs COEP on the JS/WASM responses, not only the document).
- `GET /qualia`, `/qualia/`, `/vendor/qualia`, `/vendor/qualia/` serve a QualiaPortal docs index (bounded listing, COI headers). Files under `qualia_docs_root()` (`C:\github\qualiaDB\docs`) at `/qualia/…`. Shell: `/qualia/js/qualia-shell.js`. WASM package: `pkg/qualia/qualia.js`.
- wasm-lite: `GET /synth/wasm-lite.js` (`window.SynthWasmLite`: `composeToOpfs`, `ingestHmc`, `opfsReadPng`; uses `navigator.storage.getDirectory()` when present). `POST /api/radiance/compose` returns PNG via `import_rgba_png` + `compose_rgba` + `encode_rgba_png` (1..=8 layers, 1024², ~16 MB body). `POST /api/hmc/bridge` packs in-memory PNGs into an HMC candidate.

## Review before claiming the shape is right

1. Read exported `.png` / `.svg`.
2. Read `front-view-assumed.png`, `side-view-assumed.png`, `three-quarter-view-assumed.png` from each preview dir.
3. Serve the project workbench: `http://127.0.0.1:8787/project`. Keep `http://127.0.0.1:8787/` as the QualiaDB demo studio. QualiaPortal WASM is `http://127.0.0.1:8787/qualia/js/qualia-shell.js`.
4. Read `{project}/.synth/annotations.json` before changing a model.
5. A file existing, HTTP 200, or a triangle count is not a visual review.
