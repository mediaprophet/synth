# acoustics (9)

Tone, gain, WAV/AAC, biquad, mix, bus DAG, PGA spatial, EBU R128 / Camelot DAW. Find: `synth_find q=acoustics` (or `wav`, `aac`, `lufs`).

**Crates:** `acoustics-daw-session`, `audio-aac-codec`, `audio-opus-codec`, `audio-device-cpal`, `audio-timebase`, `pcm-device-output`.

## Morphisms

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

EMF σ → Hz sonification is Portal AcousticPlane over `.emf` cells ([electromagnetics.md](electromagnetics.md)). There is no separate EMF-to-WAV store.

Back to [README.md](README.md) · contract [../AGENTS.md](../AGENTS.md)
