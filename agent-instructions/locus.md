# locus (15)

2D profiles, splines, offsets, booleans, RMF frames, SVG, and geospatial vectors. Find: `synth_find q=locus` (or `gis`, `spline`, `wgs84`).

**Crates:** `profile-authoring`, `drafting-geometry`, `vector-locus-geometry`, `vector-locus-pathops`, `locus-parametric-solver`, `locus-geospatial-codecs`.

## Morphisms

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

## Notes

- Profiles feed `hypermedia.manifold.extrude` / `revolve` / `sweep_assembly_profile`.
- GIS vectors: `transcode_geospatial_vector` plus [geodesy.md](geodesy.md). Live OSM/DEM/STAC ingest is later (`synth_jobs`, bookmarks `kind=gis`).
- 2D boolean / offset / fillet / B-spline are these ids, not Blender names.

Back to [README.md](README.md) · contract [../AGENTS.md](../AGENTS.md)
