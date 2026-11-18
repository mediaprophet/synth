# radiance (30)

Raster, PNG/PSD, grade/LUT, sprites, Cryptomatte, SDFGI, camera RAW. Find: `synth_find q=radiance` (or `png`, `lut`, `sprite`, `grade`).

**Crates:** `vector-cpu-render-adapter`, `vector-radiance-trace`, `color-transforms`, `radiance-camera-raw`, `radiance-color-grading`, `raster-blend-operators`, `raster-layer-mask-adapter`, `raster-selection-morphology`, `raster-spatial-operators`.

## Morphisms

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
| `hypermedia.radiance.solve_sdfgi_froxel` | `image`, cascades, fog, sun, sky, `backend=cpu\|gpu` (gpu = Etalon/wgpu; fail-closed without adapter) |
| `hypermedia.radiance.build_light_tree` | `lights:[{position,direction,color,power,angle_rad}]`, `width`, `height` |
| `hypermedia.radiance.develop_camera_raw_hdr` | Bayer/X-Trans CFA + exposure brackets |
| `hypermedia.radiance.grade_cdl_scopes` | ASC CDL v1.2, 3-way wheels, LogC4, LUT, waveform/vectorscope |
| `hypermedia.radiance.trace_vector_contours` | RGBA8 → cubic Bézier contours, seam carving, skeleton |
| `hypermedia.radiance.pack_sprite_sheet` | `tiles`, `columns`, `padding`, optional `names`, `mode=grid\|autotile`, `wrap=clamp\|repeat\|mirror` → `{image,png,frames,mode,wrap}` |
| `hypermedia.radiance.pack_nine_slice` | `image`, `left`, `right`, `top`, `bottom`, `dest_width`, `dest_height` |

Typical radiance 1024²; envelopes ~16 MB. EMF preview PNGs are [electromagnetics.md](electromagnetics.md) (`render_spectral_slice` / `raymarch_vox0_dda` then `encode_rgba_png`).

Back to [README.md](README.md) · contract [../AGENTS.md](../AGENTS.md)
