# manifold (27)

3D CAD, `.10d` seal, booleans, SubD, SDF, instances, BIM, botanical/vertebrate scaffolds. Find: `synth_find q=manifold` (or `loft`, `boolean`, `10d`, `camera`).

**Crates:** `asset-production-vibe` (`manifold_ops.rs`), `manifold-bim-csg`. QualiaDB computational geometry (boolean/remesh/NURBS/SDF) is wired through these ids — do not mint new `hypermedia.*` names.

## Morphisms

| ID | Args |
|---|---|
| `hypermedia.manifold.extrude` | `profile`, `path:[[x,y,z],...]`, `caps` |
| `hypermedia.manifold.revolve` | `points:[[radius,height],...]`, `segments`, optional `interpolation=linear\|nurbs`, optional `caps` |
| `hypermedia.manifold.loft` | `a`,`b` or `sections:[[ring],...]`, `interpolation=linear\|cubic\|nurbs`, `samples`, `caps` |
| `hypermedia.manifold.compile_10d` | `mesh` |
| `hypermedia.manifold.compile_10d_pbr` | `mesh`, optional `albedo`, `source_10d`, `topology`, `spatial_index`, `camera_*` |
| `hypermedia.manifold.boolean_3d` | `a`, `b`, `op=union\|intersection\|difference` |
| `hypermedia.manifold.simplify` | `mesh`, `target_triangles`, `max_error`, `mode=decimate\|remesh\|anisotropic\|semantic\|optimize`, optional `target_edge` or `metric:[6]` |
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

## Produce-path: `compile_10d_pbr`

Seals quantized mesh, area-weighted normals, optional SurfaceReading albedo, topology, BVH, UV01.

Albedo sampling order:

1. QualiaDB `Camera3D` look-at (`camera_position`, `camera_target`, `camera_up`, `camera_fov`, `camera_near`) for facing, in-frame pixels. Use this for a 3/4 photograph.
2. Authored `source_10d` UV01 (from `unwrap_slim`) when present. Vertex-count mismatch **fails closed**.
3. Auto-`unwrap_slim` when albedo is present and `source_10d` is absent.
4. Spherical centroid UV only when there is no camera and no atlas.

## Caps

- `evaluate_field` clay: `expression=clay` with `clay_mode=add|subtract|smooth|pinch`. Other expressions: `radial_wave|height_gradient|curvature_proxy|normal_displace`.
- `simplify` `mode=optimize` (alias `optimisation`) is runtime cost, not source degradation. `mode=semantic` / `admitted` stay fail-closed.
- `boolean_3d` 200k verts / 400k tris. Default `quality_profile=production16` admits a 128 MiB packed `.10d` ring.
- loft 2..=256 sections, identical point counts; `interpolation=linear|cubic|nurbs`
- revolve ≤256 segments; optional `interpolation=nurbs`; `caps=true` for a closed solid
- `subdivide_catmull_clark` levels 1..=3, abort above 200k tris
- `fspm_vascular_grow` is botanical Murray-law trees, not human coronaries
- `compile_vertebrate_anatomy_10d` is an ellipsoid comparative scaffold. Named organs are loft/revolve/boolean vibes.

A named organ (heart, lung, eye) is a vibe that lofts / revolves / booleans parts.

Back to [README.md](README.md) · UV/PBR [surface.md](surface.md) · contract [../AGENTS.md](../AGENTS.md)
