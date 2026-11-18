# surface (8)

UV, tangents, BSDF, procedural maps, PBR bake, MaterialX, brushes. Find: `synth_find q=surface` (or `uv`, `pbr`, `bsdf`, `mikkt`).

**Crates:** `surface-pbr-studio`, `surface-layered-codecs`, `vector-surface-brush`. QualiaDB `uv_atlas` is reached through `unwrap_slim` `mode=atlas`. Do not invent `hypermedia.surface.uv_atlas`.

## Morphisms

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

## Produce-path

### `unwrap_slim`

Flip-avoiding Symmetric Dirichlet / Tutte + local SLIM. Returns `uvs`, `container_10d` (UV01-sealed), SVG chart. Open meshes get a boundary loop; closed meshes start from a spherical chart then SLIM.

- `mode=slim` (default) keeps 1:1 vertex UV01.
- `mode=atlas` calls QualiaDB `uv_atlas::unwrap_atlas_into` (harmonic disk charts, **explicit seams**, may split vertices). Atlas fail-closes when a 1:1 UV01 consume is required.

### `bake_pbr_materialx`

BVH ray-cast AO + world-normal maps, OpenPBR MaterialX, PSD roundtrip. Uses authored `uvs` or auto-`unwrap_slim`; planar XY is last fallback.

`synthesize_maps` max 1024².

Photo → mesh albedo is `hypermedia.manifold.compile_10d_pbr` with `albedo` + `camera_position` ([manifold.md](manifold.md)). UV-space albedo needs `source_10d` from `unwrap_slim`.

Back to [README.md](README.md) · contract [../AGENTS.md](../AGENTS.md)
