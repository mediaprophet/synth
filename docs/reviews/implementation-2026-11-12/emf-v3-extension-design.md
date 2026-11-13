# Opt-in EMF v3 extension design

Design coordinated with root; not implemented or qualified yet. Repaired v2 remains default output. The authoritative v2 standard and external full-digest implementation remain intact. Source edits wait for root's core compile edit window.

## Compatibility and integrity boundary

- Keep magic Q42EMF64, header/directory/cell size 64 bytes and existing v2 raw-cell BLAKE3 trailers. Keep old 16-byte prefix zero and legacy Bao flag clear.
- Keep EMF_CONTAINER_VERSION=2 as default writer version; add EMF_EXTENDED_VERSION=3. Default readers admit repaired v2 and fully validated v3. v1 remains explicit migration-only, never a fallback after v2/v3 verification failure.
- v3 appends an aligned extension after all field chunks. Chunk ranges must end before extension_offset; directory and extension cannot overlap. chunk_payload/cell expose only the original records, never chunk trailers, metadata, audio, extension trailer or footer.
- Extension payload consists of bounded strict UTF-8 JSON metadata, zero padding to 64 bytes, then canonically ordered PCM-f32LE track extents with zero alignment padding. It has an existing chunk_integrity 64-byte full-BLAKE3 trailer covering every original metadata/padding/audio byte. Each track descriptor also records and verifies all 32 BLAKE3 digest bytes of its exact PCM bytes.
- Final 64-byte locator has magic Q42EMFX3, extension offset, JSON byte count, PCM section byte count, payload byte count, track count, extension schema version, reserved zeros and CRC-32C over bytes0..60. All extents/lengths/arithmetic are checked, exact file end is required, unknown flags/enums/struct fields refuse. Locator CRC and full extension digest are corruption checks, not publisher/PQ authenticity or Bao proofs.

## Typed public API (planned names)

- EmfContainerWriter::to_bytes_v3(field: &EmfFieldInterpretation, tracks: &[EmfAudioTrack<'_>], limits: EmfExtensionLimits) -> Result<Vec<u8>, EmfError>. Existing to_bytes/write_to_slice stay v2 and byte-compatible with repaired v2.
- EmfContainerReader::extension() -> Option<&EmfExtensionView<'a>>; reader.header().version reports2/3. EmfExtensionView::metadata() returns a serializable EmfExtensionMetadata; audio_payload(index) returns original borrowed LE bytes and sample access decodes f32 without alignment assumptions.
- parse_with_extension_limits adds explicit metadata/track/audio limits without adding fields to existing EmfReadLimits literals. Existing parse/parse_with_limits use conservative extension defaults in addition to their total file/chunk limit.
- EmfExtensionMetadata contains field interpretation plus full generated audio descriptors. Core uses its existing serde/serde_json and chunk_integrity dependencies only. Portal/root can retain/serialize metadata while separately exposing checked track bytes; no model/provider/high-level audio library is needed in core.

## Field interpretation

EmfFieldInterpretation declares colour, exact timebase, Hilbert order and spectral semantics.

- Colour is tagged SpectralOnly or RgbPreview with named Rec709/Rec2020/DisplayP3 primaries, D65/D50 white point, Linear/Srgb/Bt709/Pq/Hlg transfer, explicit CIE1931 two-degree observer and positive finite reference-white luminance. These describe a declared derivative RGB interpretation, not extra physical E/B cell channels or proof a renderer implements that conversion. Unsupported display interpretation must be reported/refused by the consumer.
- Timebase carries positive bounded rational ticks-per-second numerator/denominator, signed slice_start_tick, positive slice_step_ticks and an exact epoch {scale,seconds,nanoseconds}. Epoch scales initially Relative, UnixPosix and TAI have explicit semantics; UnixPosix does not model leap seconds. The header f64 reference_epoch_s is the nearest compatible value; v3 exact epoch/time metadata is authoritative. Writer/reader check their agreement and checked time arithmetic.
- Field time slice i is at epoch + (slice_start_tick + i*slice_step_ticks)*ticks_per_second_den/ticks_per_second_num seconds. Audio starts use the same tick origin. Exact rational seconds remain authoritative, not rounded wall-clock playback values.
- Hilbert order is explicitly stored and bounded1..8, removing v3's inference ambiguity; v2 inference remains compatibility behavior. Existing dense allocation cap remains1,048,576 cells.
- Spectral semantics explicitly distinguishes authored physical wavelength_nm plus legacy authored sigma, and compile_emf's amplitude-weighted wavelength centroid plus legacy display sigma. The scalar is not a full spectrum/distribution. Multiple source wavelengths currently collapse to a centroid; no arbitrary-spectrum completeness claim is made. Physical wavelength/E/B remain distinct from visible colour and audible sonification.

## Audio tracks

Initial codec is explicitly PCM-f32LE interleaved. Each descriptor carries unique bounded ID, purpose, channels, channel layout, rational sample rate, signed start_tick, signal units/calibration, frame count, relative PCM offset, exact byte length and full BLAKE3 digest.

- Track sample i occurs at the track start plus i*sample_rate_den/sample_rate_num seconds; timestamps and products use checked integer/rational arithmetic.
- Preserve every finite f32 bit pattern (including signed zero and finite values outside[-1,1]) when units are physical pressure or arbitrary signal units. Only an explicitly normalized-amplitude unit imposes[-1,1]. Do not normalize through PCM16/WAV.
- Initial units: normalized amplitude, pascals, or explicitly labelled arbitrary signal units with finite positive calibration. Channel count/layout must agree; unknown codecs, roles or enum values refuse. Track/sample/audio/file limits are explicit before allocation.
- Time/sample-rate support is not limited to human-audible frequencies. Playback devices may impose separate supported rates; refusal to play must not delete the stored track or reinterpret its units. Independent recorded/authored audio is not inferred from the EM cells or visible sigma.

## Hypermedia integration

Extend existing compile_emf with an optional v3 field interpretation and audio_tracks. Without extension arguments, preserve repaired v2 production. Use existing audio handles or explicit retained PCM-f32LE input bindings, preserving original finite samples. Add bounded argument parsing in a focused helper module; do not grow unrelated geometry/codec code. Keep all148 catalog IDs/dispatch entries.

verify_emf reports container version, full-BLAKE3 cell integrity, optional extension integrity, serialized colour/time/spectral declarations and actual track descriptors/count/bytes. It must not claim Bao verification. Rendering/sonification remain derived operations and must not silently discard or reinterpret extension metadata. Root owns Portal/present retention, track access and actual WASM rebuild/qualification.

## Required tests before qualification

v2 byte compatibility/read; v1 current-reader refusal + migration-only path; v3 colour/time/multichannel PCM exact roundtrip including signed-zero/out-of-normalized-range samples; raw field payload unchanged; tamper every extension/track digest and reserved field; unknown versions/enums/flags; malformed extents, overlaps, integer overflow, audio byte/frame/channel mismatch, nonfinite samples and resource caps. Verify out-of-visible physical wavelengths remain distinct from human previews. Exercise compile/verify and Portal consumer retention, then rebuild actual served WASM before claiming browser support.

This design does not alter QBDL, M64/E64, 10d integrity, streaming commitments or external epoch metadata owners.
