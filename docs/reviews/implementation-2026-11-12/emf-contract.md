# EMF contract investigation — paused 2026-11-12

No implementation or tests were performed. The following is a proposal, not an accepted format or evidence of support.

## Findings

- QualiaDB `crates/qualia-core-db/src/q42/emf_container/{header,reader,writer,chunk_directory,mod}.rs` owns Q42EMF64. Version 1 has a 64-byte header, 64-byte directory rows and 64-byte cells. The writer hashes each brick with a BLAKE3 digest prefix; the reader verifies directory CRCs and available brick digests, enforces an explicit file/chunk budget, validates bounds and overlap, and exposes borrowed payloads.
- Hypermedia `crates/asset-production-vibe/src/electromagnetics_ops.rs` implements `compile_emf` and `verify_emf`. Sources accept wavelengths 1..=1e9 nm and collapse multiple sources to a weighted mean per cell. The cell stores physical wavelength, E and B; `spectral_sigma` is clamped to a visible-band mapping. `verify_emf` currently reports physical aggregates, not track or colour metadata.
- QualiaDB `crates/qualia-core-db/src/render/portal_load_emf.rs` parses EMF for the portal; consumers must be audited for retaining/reporting extension content and for explicit display behavior.
- Hypermedia `crates/color-transforms/src/spaces.rs` provides named RGB gamuts including BT.709, BT.2020 and P3-D65. `crates/acoustics-daw-session/src/codecs/mod.rs` owns an AudioBuffer and WAV 16-bit PCM encoding / PCM16 or float32 decoding. Qualia core cannot depend backwards on Hypermedia; a cross-boundary representation must be explicitly specified and compatible with existing codecs, not silently normalized through WAV PCM16.
- `audit-docs/FUNCTIONS.md` identifies existing colour and audio functions. The guessed single `audit-docs/modules/asset-production-vibe.md` path was absent; find the correct indexed module path before further source-index work.

## Proposed opt-in contract for review

Preserve v1 serialization and reading. Consider v2 with a bounded, authenticated extension after the brick data, with explicit version and checked extents. Include a colour interpretation (named primaries/white point and transfer, or explicitly defined profile data) and real timed audio tracks containing codec, channels, sample rate, start time relative to the reference epoch, signal units and calibration. PCM float32 interleaved is a candidate initial codec, preserving finite samples outside normalized audible amplitude; track count, sample bytes, total file bytes and numerical ranges must be bounded. Authenticate all metadata and audio bytes and reject unknown/malformed structural fields, overlaps, invalid lengths and unsupported codec interpretation. Ensure v1 readers fail closed on v2 and new readers retain v1 compatibility. Define display behavior for unsupported profiles as an explicit conversion refusal, not an implicit sRGB reinterpretation.

The physical electromagnetic cells, recorded acoustic pressure or declared signal units, and derived display/sonification are distinct. Audio is not inferred from E/B and no physical spectrum should be clamped to human perceptual bands. Existing multi-source weighted-mean representation cannot retain separate spectra; resolve that limitation honestly in the resumed design rather than claiming full spectral preservation.

Extend the existing compile/verify morphisms and ontology in lockstep, with a usable production roundtrip and independent review. Audit portal and all render/sonification paths for metadata reporting and unsupported interpretation. Exercise v1 compatibility, broad-spectrum field values, colour/audio roundtrip, integrity tampering and malformed/budgeted input. Inspect output and listen where meaningful. Root coordinates builds/tests in `C:/github/synth/target-gap-review`; do not use or disturb live targets.

The separate .m64/.e64 boundary specification is pending and must not be modified or redefined here.
