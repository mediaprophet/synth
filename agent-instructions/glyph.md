# glyph (7)

Text shaping, SVG paths, RGBA raster, HarfBuzz/BiDi, MSDF atlas, paragraph layout. Find: `synth_find q=glyph` (or `font`, `harfbuzz`, `msdf`).

**Crates:** `vector-glyph-shaping`.

## Morphisms

| ID | Args |
|---|---|
| `hypermedia.glyph.measure_bounds` | `text`, `font_size`, wrapping |
| `hypermedia.glyph.shape_to_svg_path` | `text`, origin, `font_size`, tracking |
| `hypermedia.glyph.project_svg` | shaped text run → SVG resource |
| `hypermedia.glyph.rasterize_rgba` | shaped text run → RGBA8 |
| `hypermedia.glyph.shape_harfbuzz_bidi` | `text`, `base_direction`, `script` |
| `hypermedia.glyph.pack_msdf_font_atlas` | `charset`, `cell_size`, `px_range` |
| `hypermedia.glyph.compose_knuth_plass_paragraph` | `text`, `frame_width`, `frame_height` |

Shaped runs feed [layout.md](layout.md) UI sheets and [discourse.md](discourse.md) documents.

Back to [README.md](README.md) · contract [../AGENTS.md](../AGENTS.md)
