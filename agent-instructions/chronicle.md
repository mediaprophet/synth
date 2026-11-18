# chronicle (5)

MP4 mux/decode, deterministic movie, spatial motor packets, fiduciary tabular archive. Find: `synth_find q=chronicle` (or `mp4`, `otio`, `movie`).

**Crates:** `chronicle-tabular-engine`, `chronicle-fiduciary-ledger`, `chronicle-office-codecs`, `chronicle-timeline-interchange`, `chronicle-broadcast-containers`, `video-h264-codec`, `video-h264-preview`, `video-prores-codec`, `media-isobmff`, `media-ogg`.

## Morphisms

| ID | Args |
|---|---|
| `hypermedia.chronicle.mux_mp4` | `frames`, `fps` |
| `hypermedia.chronicle.decode_mp4_frame` | `source`, `frame_index` |
| `hypermedia.chronicle.capture_deterministic_movie` | RGBA frames + audio + locked-dt |
| `hypermedia.chronicle.sync_spatial_motors` | entity PGA motors, 60 Hz packets |
| `hypermedia.chronicle.recalculate_tabular_fiduciary_archive` | spreadsheet + journal + OTIO/ProRes |

Lossless final sequences: `capture_deterministic_movie` with `profile:"png-sequence"`, rational `fps:{numerator:24000,denominator:1001}`, congruent retained RGBA8/16 frames. That delivers an exact-timing image sequence.

Back to [README.md](README.md) · contract [../AGENTS.md](../AGENTS.md)
