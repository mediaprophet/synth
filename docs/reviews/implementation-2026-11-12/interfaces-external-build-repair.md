# External partial-build diagnosis and minimal repair

Read-only inspection confirmed external streaming/storage edits were in flight: key_management/mod.rs now declares/reexports managed_control; root confirmed x25519 wiring was also saved after build3. No crypto wiring changes by this worker.

A real stale call remained at container_10d/section/opaque_preservation.rs:187. That file was last saved2026-11-08T07:58:29Z; table_validation refactor was2026-11-12T11:48:53Z and metadata.rs added11:53:51Z. Removed table_validation::metadata_range was still called. Root authorized a minimal repair: inline the nonempty payload/header/table exclusion using existing checked payload_end, returning the same SectionOverlapsMetadata error. Do not substitute strict validate_descriptor_metadata: it rejects unknown tags/reserved16 that explicit opaque structural retention intentionally preserves. Strict validator, payload CRCs, whole-file CRC and crypto code remain untouched.

Existing tests/ten_d_opaque_preservation.rs::padding_duplicates_zero_tags_alignment_and_overlaps_refuse already checks this metadata-overlap error (line234), plus existing unknown/reserved retention coverage. No duplicate test was added. Root owns Cargo and source acceptance; production source frozen before root next build.
