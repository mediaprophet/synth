# electromagnetics (6)

`.emf` (`Q42EMF64`) field volumes, spectral preview, DDA raymarch, metamer null-space, Wi-Fi CSI. Find: `synth_find q=emf` (or `electromagnetics`, `csi`).

**Crates:** `electromagnetics-rf-sensing`. Writer `EmfContainerWriter`; reader `EmfContainerReader`.

## Morphisms

| ID | Args |
|---|---|
| `hypermedia.electromagnetics.compile_emf` | `origin`, `voxel_size`, `grid_dims`, `sources` |
| `hypermedia.electromagnetics.verify_emf` | `emf` |
| `hypermedia.electromagnetics.render_spectral_slice` | `emf`, `axis`, `slice_index`, `width`, `height`, `exposure` |
| `hypermedia.electromagnetics.raymarch_vox0_dda` | `emf`, `width`, `height`, `deflection_mass`, `step_limit` |
| `hypermedia.electromagnetics.project_metamer_nullspace` | `spd_41`, `nullspace_scale` |
| `hypermedia.electromagnetics.sense_wifi_csi_vitals_aoa` | `breathing_hz`, `heart_hz`, `carrier_ghz` |

## Durable field is `.emf`

Each cell is 64 bytes: **E** (V/m), **B** (T), Poynting flux, `wavelength_nm`, n/κ, coherence phase, Stokes Q/U/V, `spectral_sigma` (σ), `confidence_alpha`. Chunks are 8³ Hilbert bricks (512 cells, 32 KiB). Seal path: occupancy → sibling `.m64` + `.emf` + `.10d`.

`.10d` names the field via MID2 `FIELD_KIND_ELECTROMAGNETIC` (`0x0100`) plus FIEL descriptors. Those records do not duplicate brick cells.

RGBA and sonics are conversions of the same cells:

- **RGBA** — `render_spectral_slice` / `raymarch_vox0_dda` (CIE 1931 + Bruton), then `encode_rgba_png`. Preview, not a replacement for `.emf`.
- **Sonics** — QualiaPortal AcousticPlane maps σ to Hz (`sigma_to_center_frequency_hz`: 400–700 nm → 1760–110 Hz).

`verify_emf` checks CRC-32C + Bao. `dense_volume()` fail-closes above `MAX_DENSE_CELLS` (1_048_576). Portal `load_emf` unpacks Hilbert bricks onto the GPU field. WASM: `QualiaPortal.load_emf`.

Back to [README.md](README.md) · contract [../AGENTS.md](../AGENTS.md)
