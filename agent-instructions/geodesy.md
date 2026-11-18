# geodesy (7)

Terrain, `.m64` matter, advection, continuum/XPBD contact, sector step, kriging. Find: `synth_find q=gis` (or `terrain`, `m64`, `dem`, `osm`).

**Crates:** `terrain-brushes`, `terrain-erosion-solvers`, `geodesy-spatial-analysis`. GIS vectors also: `hypermedia.locus.transcode_geospatial_vector` ([locus.md](locus.md)).

## Morphisms

| ID | Args |
|---|---|
| `hypermedia.geodesy.sculpt_terrain` | `tile_resolution`, `strokes`, `width`, `height` |
| `hypermedia.geodesy.compile_m64` | geodetic origin, strata/holo/point |
| `hypermedia.geodesy.verify_m64` | `m64` |
| `hypermedia.geodesy.advect_m64_grid` | Poisson + MacCormack + Eikonal |
| `hypermedia.geodesy.couple_continuum_xpbd` | Bekker-Wong/Mohr-Coulomb contact |
| `hypermedia.geodesy.step_sector_continuum_3d` | `sector_id`, `steps` |
| `hypermedia.geodesy.kriging_variogram_morphometry` | WGS84 points, variogram, DEM |

## GIS

Produce-time maps bake through these morphisms now. Live OSM / DEM / STAC ingest is opt-in later via `synth_jobs` and `synth_bookmarks` `kind=gis`. Bookmark OSM/opentopography/stac landing pages today; do not invent a host rebuild when a new `.m64` lands.

`sculpt_terrain` is a 2.5D heightfield. Physical terrain records retain metre samples, spacing, origin and coordinate frame independently from relief colours.

Back to [README.md](README.md) · contract [../AGENTS.md](../AGENTS.md)
