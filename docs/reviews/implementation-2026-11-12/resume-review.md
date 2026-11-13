# Independent resume review — 12 November 2026

Status: read-only source/specification review of intervening format work. No Cargo, tests, renderer runs, streaming implementation, or format edits performed by this reviewer. Only this review note is written. The user resumed the task and clarified M64 is current; E64 is legacy compatibility.

## What changed and is implemented

The current references are in `C:/github/qualiaDB/docs/manuals/standards`, not `docs/standards`. The overview, six format standards and conformance register are current. `file-formats-standards-work-in-progress.md` explicitly labels its earlier research historical; do not use those older paragraphs to override the repair.

- `q42/chunk_integrity/mod.rs` constructs and verifies a 64-byte trailer containing all 32 BLAKE3 digest bytes followed by 32 required zero bytes. It hashes exact record bytes, not decoded floats, directory metadata or the trailer. All digest/tail bytes are checked.
- EMF writers emit version 2. Each chunk is `cell_count * 64 + 64` bytes, directory flag `0x0004` is required, old 16-byte digest slot and directory reserved bytes are zero, and the old header/prefix flags are cleared/refused. Header/directory/cells remain 64 bytes. `chunk_payload` and `cell` exclude the trailer.
- Default EMF parsing rejects version 1. `parse_legacy_with_limits` is explicitly migration-only; no failed v2 verification falls back to the old prefix check. Header validation alone is not full container admission.
- M64/E64 codec 3 uses the same full trailer with zero old 12-byte prefix. `M64Verification` separately reports `full_hash_chunks` and `legacy_prefix_chunks`; payload/record iteration excludes the trailer. Old codec 2 or nonzero-prefix chunks require explicit migration admission.
- Raw M64 codec 0 remains supported without a full chunk digest. GRM3 v3 stays its raw-only profile with record/metadata checks. The physical heightfield producer must not be advertised as full-BLAKE3 solely because other M64 producers are.
- Hypermedia `parse_sealed_emf`, M64 occupancy authoring, self-verification and occupancy admission were updated. Occupancy now appends the trailer and requires codec 3; the former nonzero-prefix self-check is gone. These are real implementation changes, not just documentation.
- E64 remains M64 v1/v2 compatibility spelling, not an environmental-field replacement. E64 GRM3 is not admitted. Renaming the extension/magic does not migrate integrity or add geospatial metadata.

The shared/cross-format tests exercise all 64 trailer bytes, record mutation, flags, count/length mismatch, payload/trailer separation and explicit legacy resealing. The conformance document reports 61 production-module harness tests, 7 Hypermedia electromagnetics tests and 5 geodesy tests passed in the external work. Those are reported evidence, not independently rerun results from this resumed task.

## Streaming boundary

`qdnf-imp/qbdl-streaming/README.md` identifies an implementation programme, not completed protocol conformance. Its planned descriptor, SHA-384 range tree and paired PQ/classical authority are separate from inner-format BLAKE3. Existing QBDL SHA-256 entry digests and CRC stay unchanged; ZIP HMC is a separate encoding despite the same suffix. No new alignment flags, signatures or proof trees may be inferred from the inner repair. Streaming work belongs to its existing owner; this task must not duplicate it.

The standards identify existing streaming admission/alignment weaknesses as work for that programme. They do not make those weaknesses approved modes. No streaming implementation was changed here.

## Concrete caller findings and required adjustments

1. **Incorrect presentation claim:** `Synth/src/present_3d.rs` displayed unconditional `BLAKE3-Bao + CRC-32C Sealed` at the reviewed line 520. It is false for the repaired EMF digest (no Bao proof), CRC-based 10D, and raw M64 profiles. Root owns correction to actual verified profile information or neutral status.
2. **Rebuild/requalify consumers:** the frozen pre-pause Synth executable is old. New EMF v2 and M64 occupancy bytes differ. Native and Portal consumers must use the new readers; do not silently retry legacy parsing. Browser/WASM qualification must exercise the actual served package, not infer success from Rust source or native tests.
3. **Regenerate affected artifact identities:** each new trailer changes file size, inner digests and enclosing HMC SHA-256. Compare deterministic replay using the same rebuilt executable and exact source/input bytes. An old package's source can execute new morphism semantics and therefore fail strict byte replay; migration must be explicit, not accomplished by relaxing equality or pretending the old package was upgraded.
4. **Receipt precision:** report format version and selected integrity profile. For M64 include full-hash/legacy/raw chunk counts and record-CRC counts. For EMF v2 record complete-BLAKE3 verification, not historical `bao_hash_prefix != 0`. Container corruption checking is established; publisher authority remains an independently trusted outer mechanism.
5. **Focused resumed checks:** rerun the source harness and producer tests after the source assembly gate, then exercise production → HMC reopen → native/Portal load. Include last digest byte and reserved trailer mutation, wrong/missing flags, legacy opt-in/reseal, raw/GRM3 M64 admission, and exact record counts with trailer excluded. Preserve current M64 and E64 compatibility fixtures. Do not reintroduce assertions requiring a nonzero old prefix.

No new defect in the inspected full-digest verification path was established. Known documented constraints—EMF inferred Hilbert order, full-buffer admission, bounded dense conversion, and application-level scientific validation—remain explicit constraints rather than missing-algorithm claims.

## Pre-pause colour/audio request remains separate

EMF v2 changes integrity, not the scientific cell schema or multimodal metadata. It still has no serialized colour interpretation or timed audio-track directory/payload. Epoch plus integer time-slice does not specify timestep/time scale. The `effective_sigma` visible-band mapping is a derived consumer convention; it does not preserve a full spectrum or recorded audio.

The older `emf-contract.md` proposal is superseded where it proposed using v2 or default v1 compatibility. Root assigned interfaces an opt-in v3 extension: preserve v2 reading/full trailers and migration-only v1; specify bounded colour interpretation, physical clock/timebase, typed audio descriptors and payloads with complete integrity; retain 64-byte field cells; expose metadata through compile/verify and Portal together. Unsupported display/listening interpretation must refuse explicitly, not silently assume sRGB or normalize physical samples. Do not claim separate spectral bands survive the current weighted-mean source representation.

The extension owner has received these compatibility/API constraints directly. Independent review resumes after the implementation is assembled and before final acceptance.
