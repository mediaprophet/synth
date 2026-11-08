# General-Purpose Categorical & Ontological Asset Operations

Synth exposes shared production libraries across **Etalon**, **QualiaDB**, and **Hypermedia** through Vibe's existing host capability ABI.
- `synth ontology` inspects the **12 canonical mathematical categories** and their **342 parity-verified `NQuin48` ontological triples** (`hasPrecondition`, `hasPostcondition`, `preservesInvariant`).
- `synth operations` lists all **114 admitted morphisms** (`hypermedia.<category>.<morphism>`), their implementation owners, arguments, and agentic utility specifications.

This is a general-purpose asset-production catalogue consumed by Synth and ready for Poet without modifying Poet's application, UI, or document model.

```vibe
requires [capability("capability.invoke")];
let profile = capability.invoke("hypermedia.locus.construct", {
    shape: "rectangle", width: 40, height: 24, radius: 4
});
let mesh = capability.invoke("hypermedia.manifold.extrude", {
    profile: profile, path: [[0, 0, 0], [0, 0, 8]], caps: true
});
let native = capability.invoke("hypermedia.manifold.compile_10d", {mesh: mesh});
let svg = capability.invoke("hypermedia.locus.project_svg", {profile: profile});
let mut asset = {files: [
    {key: "model.10d", resource: native},
    {key: "outline.svg", resource: svg}
]};
```

Geometry, image, and audio operations return immutable, session-local handles. Encoded-resource handles can be used in `{key, resource}` file declarations; the encoder supplies their format kind. Historical `synth.*` and legacy `hypermedia.*` operation identifiers continue to resolve losslessly via `ontology::canonical_morphism_id` for strict `.hmc` replay compatibility, while all canonical operation identifiers use category-theoretic naming with zero `-craft` legacy names.

## 1. The 12 Canonical Categories

| Sub-NS ID | Category (`hypermedia.<cat>`) | Mathematical Object Space | Morphisms |
|---:|---|---|---:|
| `1` | `locus` | $\mathcal{L}_2$: 1D/2D parametric curves, B-splines, offsets, booleans, fillets, cyclic Bishop RMFs, and geodesic paths | 12 |
| `2` | `manifold` | $\mathcal{M}_3$: 2-manifolds, CSG solids, Radial-Edge B-Reps, SubD, lazy fields, SDF bricks, FSPM botany, and `.10d` PBR containers | 21 |
| `3` | `surface` | $\mathcal{S}_{\text{PBR}}$: Tangential material sheaves, SLIM UV charts, MikkTSpace tangents, Cycles BSDF closures, and procedural fields | 5 |
| `4` | `radiance` | $\mathcal{R}_2$: 2D colorimetric & alpha-composited lattices, masks, 3D LUTs, PSDs, Kuwahara, Cryptomatte, SDFGI/Froxel, and Light Trees | 25 |
| `5` | `glyph` | $\mathcal{G}$: Typographic glyph outlines, UAX #9 BiDi/HarfBuzz shaped text runs, and packed MSDF/vector text projections | 6 |
| `6` | `layout` | $\mathcal{B}_2$: Hierarchical spatial box trees, scoped Webizen UI tokens, 3D diegetic viewports, and 4-stage fiduciary input routing | 6 |
| `7` | `kinetics` | $\mathcal{K}_t$: PGA MotorFTI, two-bone IK, Island XPBD, dual-BVH `DirectSpaceState`, 3D NavMesh Funnel+ORCA, NLA, and Lottie | 10 |
| `8` | `acoustics` | $\mathcal{A}_t$: 1D pressure-wave buffers, RBJ biquad IIR filters, 32-bus `AudioServer` DAGs, PGA spatialization, and WAV/AAC bitstreams | 8 |
| `9` | `electromagnetics` | $\mathcal{E}_5$: 5D wave-optical & electromagnetic field volumes (`.emf` / `Q42EMF64`), Amanatides-Woo DDA raymarches, and 38D metamer null-spaces | 5 |
| `10` | `geodesy` | $\mathcal{T}_{\oplus}$: Planetary heightfields, `.m64` (`m64\0`) subsurface strata, Poisson/MacCormack advection, and Bekker/Mohr-Coulomb coupling | 5 |
| `11` | `discourse` | $\mathcal{D}$: Foliated documents (`.pdf`), prose (`.docx`), slide decks (`.pptx`), charts (`.svg`), galleries (`.html`), telemetry & fiduciary RIDs | 7 |
| `12` | `chronicle` | $\mathcal{C}_t$: Multiplexed audiovisual time-streams, deterministic `ManifoldClock` `.mp4` capture, and 60Hz spatial motor packets | 4 |

## 2. Admitted Morphisms by Category (114 Total)

### `hypermedia.locus.*` (12 Morphisms)
- `construct`, `transform`, `project_svg`, `spline`, `offset`, `boolean`, `stroke_to_locus`, `fillet`, `resample`, `trim`, `bishop_rmf_cyclic`, `shortest_edge_path`

### `hypermedia.manifold.*` (21 Morphisms)
- `extrude`, `revolve`, `loft`, `compile_10d`, `boolean_3d`, `simplify`, `import_mesh`, `project_dxf`, `compile_10d_pbr`, `radial_euler`, `bevel`, `inset`, `solidify`, `bisect_plane`, `subdivide_catmull_clark`, `evaluate_field`, `mesh_to_sdf_brick`, `sdf_brick_to_mesh`, `instance_on_points`, `realize_instances`, `fspm_vascular_grow`

### `hypermedia.surface.*` (5 Morphisms)
- `synthesize_maps`, `unwrap_slim`, `compute_mikktspace`, `compile_bsdf_graph`, `sample_procedural_field`

### `hypermedia.radiance.*` (25 Morphisms)
- `rasterize_loci`, `convolve_blur`, `blend_rgb`, `transform_exposure`, `morphology_mask`, `encode_rgb_png`, `import_rgb_png`, `render_vector`, `create_rgba`, `compose_rgba`, `filter_rgba`, `encode_rgba_png`, `import_rgba_png`, `modify_mask`, `apply_lut`, `grade_color`, `convert_transfer`, `develop_photo`, `encode_psd`, `import_psd`, `filter_kuwahara_anisotropic`, `decode_cryptomatte_q42`, `vector_motion_blur`, `solve_sdfgi_froxel`, `build_light_tree`

### `hypermedia.glyph.*` (6 Morphisms)
- `measure_bounds`, `shape_to_svg_path`, `project_svg`, `rasterize_rgba`, `shape_harfbuzz_bidi`, `pack_msdf_font_atlas`

### `hypermedia.layout.*` (6 Morphisms)
- `solve_box_tree`, `bind_theme_scope`, `render_ui_sheet`, `project_ui_svg`, `project_diegetic_surface`, `route_input_action`

### `hypermedia.kinetics.*` (10 Morphisms)
- `sample_frames`, `solve_two_bone_ik`, `interpolate_keyframes`, `export_lottie`, `interpolate_motor_fti`, `step_island_xpbd`, `query_direct_space`, `navigate_funnel_orca`, `evaluate_nla_channelbag`, `step_simulation_zone`

### `hypermedia.acoustics.*` (8 Morphisms)
- `synthesize_tone`, `scale_gain`, `encode_wav`, `filter_biquad`, `mix_tracks`, `encode_aac`, `route_bus_dag`, `spatialize_pga_zone`

### `hypermedia.electromagnetics.*` (5 Morphisms)
- `compile_emf`, `verify_emf`, `render_spectral_slice`, `raymarch_vox0_dda`, `project_metamer_nullspace`

### `hypermedia.geodesy.*` (5 Morphisms)
- `sculpt_terrain`, `compile_m64`, `verify_m64`, `advect_m64_grid`, `couple_continuum_xpbd`

### `hypermedia.discourse.*` (7 Morphisms)
- `review_gallery`, `compile_pdf`, `compile_prose_docx`, `compile_deck_pptx`, `render_chart_svg`, `inspect_engine_telemetry`, `verify_fiduciary_rid`

### `hypermedia.chronicle.*` (4 Morphisms)
- `mux_mp4`, `decode_mp4_frame`, `capture_deterministic_movie`, `sync_spatial_motors`

## 3. Bounds and Transactional Behavior

Default limits allow 128 retained intermediate artifacts, 16 MiB per artifact, 64 MiB combined artifact payload, and 50 million logical operation work units per source cell. These use the session's `resources`, `resource_bytes`, `total_resource_bytes`, and `asset_work_units` settings. Output resources and HMC size are checked separately using the existing production limits.

Each cell executes against a tentative artifact workspace. A capability error, invalid output, package failure, or cancellation discards that workspace along with the tentative source bindings. Earlier handles and candidate bytes remain unchanged.

The HMC retains source cells and input bytes/hashes. Strict reopen reruns the same producer and operation calls and requires exact package bytes.

## 4. End-to-End Verification

Run the CLI inspection and production workflows with:

```powershell
cargo build --locked --bin synth -j 1
./target/debug/synth.exe ontology
./target/debug/synth.exe operations
./target/debug/synth.exe produce --source examples/production.vibe --out target/production.hmc --export-dir target/production-export
./target/debug/synth.exe produce --reopen target/production.hmc --out target/production-reopened.hmc
```
