//! Atomic pre-commit HMC resource batch verifier for Poet checkpoints.
//!
//! Validates .10d, .q42, and .m64 v3 containers prior to disk or carrier mutation.
//! Rejects duplicate keys, empty keys, NUL bytes, and budget exhaustion.
//! Reused MIT-licensed Craft-origin capabilities become internal Poet functionality,
//! preserving required notices and native IntentBus/NQuin/HMC contracts.

use super::*;
use qualia_core_db::container_10d::{
    decode_container_manifold_identity, parse_section_table,
    section::{inspect_opaque_container, OpaquePreservationBudget},
    SectionType, TypedFieldRecord, MAGIC_10D, MANIFOLD_IDENTITY_V2_MAX_FIELDS,
};
use qualia_core_db::q42::m64_matter::{
    has_m64_magic, M64PayloadInterval, M64ReadLimits, M64Reader,
};
use std::collections::BTreeSet;

/// Structured report returned on successful pre-commit batch verification.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AtomicResourceVerificationReport {
    pub total_edits: usize,
    pub verified_resources: usize,
    pub m64_verified: usize,
    pub q42_verified: usize,
    pub ten_d_verified: usize,
    pub opaque_passed: usize,
    pub total_bytes_checked: usize,
}

impl AtomicResourceVerificationReport {
    pub fn is_empty(&self) -> bool {
        self.total_edits == 0
    }
}

/// Atomically verify an entire batch of resource edits before disk commit or carrier mutation.
pub fn verify_atomic_resource_batch(
    edits: &[ResourceEdit],
    max_batch_bytes: usize,
) -> Result<AtomicResourceVerificationReport, String> {
    verify_atomic_resource_batch_inner(edits, max_batch_bytes, false)
}

/// Strict mode: enforces typed container verification on all typed containers.
pub fn verify_strict_typed_containers(
    edits: &[ResourceEdit],
    max_batch_bytes: usize,
) -> Result<AtomicResourceVerificationReport, String> {
    verify_atomic_resource_batch_inner(edits, max_batch_bytes, true)
}

fn verify_atomic_resource_batch_inner(
    edits: &[ResourceEdit],
    max_batch_bytes: usize,
    strict: bool,
) -> Result<AtomicResourceVerificationReport, String> {
    let mut report = AtomicResourceVerificationReport {
        total_edits: edits.len(),
        ..Default::default()
    };
    let mut seen_keys = BTreeSet::new();
    let mut total_batch_bytes: usize = 0;

    for edit in edits {
        let (key, kind, bytes) = match edit {
            ResourceEdit::Add(d) => (&d.key, Some(d.kind.as_str()), Some(d.bytes.as_slice())),
            ResourceEdit::Replace { resource, .. } => (
                &resource.key,
                Some(resource.kind.as_str()),
                Some(resource.bytes.as_slice()),
            ),
            ResourceEdit::ReplaceBytes { key, bytes, .. } => (key, None, Some(bytes.as_slice())),
            ResourceEdit::Remove { key, .. } => (key, None, None),
        };

        if key.is_empty() || key.as_bytes().contains(&0) {
            return Err("Resource edit key cannot be empty or contain NUL bytes".into());
        }
        if !seen_keys.insert(key.clone()) {
            return Err(format!("Duplicate resource edit key in batch: '{key}'"));
        }

        if let Some(payload) = bytes {
            if payload.len() > max_batch_bytes {
                return Err(format!("Resource '{key}' exceeds caller budget {max_batch_bytes}"));
            }
            total_batch_bytes = total_batch_bytes
                .checked_add(payload.len())
                .ok_or_else(|| "Total resource batch bytes overflow".to_string())?;
            if total_batch_bytes > max_batch_bytes {
                return Err(format!(
                    "Total batch bytes {total_batch_bytes} exceed caller budget {max_batch_bytes}"
                ));
            }

            let is_m64 = key.ends_with(".m64")
                || kind == Some("m64")
                || (strict && key.contains(".m64"));
            let is_q42 = key.ends_with(".q42")
                || kind == Some("q42")
                || (strict && key.contains(".q42"));
            let is_10d = key.ends_with(".10d")
                || kind == Some("10d")
                || (strict && key.contains(".10d"));

            if is_m64 {
                verify_m64_payload(key, payload, max_batch_bytes)?;
                report.m64_verified += 1;
            } else if is_q42 {
                let _ = verify_q42_payload(key, payload)?;
                report.q42_verified += 1;
            } else if is_10d {
                verify_10d_payload(key, payload, max_batch_bytes)?;
                report.ten_d_verified += 1;
            } else {
                report.opaque_passed += 1;
            }
            report.verified_resources += 1;
            report.total_bytes_checked += payload.len();
        }
    }

    Ok(report)
}

/// Verify an .m64 payload using M64Reader::parse and CRC-32C record verification.
pub fn verify_m64_payload(key: &str, bytes: &[u8], max_bytes: usize) -> Result<(), String> {
    if bytes.len() > max_bytes || bytes.len() < 64 || !has_m64_magic(bytes) {
        return Err(format!("M64 resource '{key}' invalid size, magic or truncated header"));
    }
    let required_chunks = u64::from_le_bytes(bytes[48..56].try_into().unwrap());
    if required_chunks == 0 || required_chunks > 4096 || required_chunks as usize > bytes.len() / 64
    {
        return Err(format!("M64 resource '{key}' chunk count {required_chunks} invalid"));
    }
    let chunk_count = required_chunks as usize;
    let mut scratch = vec![M64PayloadInterval::default(); chunk_count];
    let limits = M64ReadLimits {
        max_file_bytes: max_bytes,
        max_chunks: chunk_count,
        max_records: (chunk_count as u64).saturating_mul(4096),
    };
    let reader = M64Reader::parse(bytes, limits, &mut scratch)
        .map_err(|e| format!("M64 resource '{key}' parse verification failed: {e}"))?;
    let v = reader.verification();
    if v.record_crc_checked != v.total_records || v.records_without_checksum != 0 {
        return Err(format!(
            "M64 resource '{key}' CRC-32C failed: {}/{} checked, {} unchecksummed",
            v.record_crc_checked, v.total_records, v.records_without_checksum
        ));
    }
    Ok(())
}

/// Verify a .q42 payload enforcing 48-byte NQuin alignment and XOR parity fold.
pub fn verify_q42_payload(key: &str, bytes: &[u8]) -> Result<usize, String> {
    if bytes.len() % 48 != 0 {
        return Err(format!(
            "Q42 resource '{key}' length {} is not a multiple of 48 bytes (NQuin alignment)",
            bytes.len()
        ));
    }
    for (idx, slot) in bytes.chunks_exact(48).enumerate() {
        let s = u64::from_le_bytes(slot[0..8].try_into().unwrap());
        let p = u64::from_le_bytes(slot[8..16].try_into().unwrap());
        let o = u64::from_le_bytes(slot[16..24].try_into().unwrap());
        let c = u64::from_le_bytes(slot[24..32].try_into().unwrap());
        let m = u64::from_le_bytes(slot[32..40].try_into().unwrap());
        let parity = u64::from_le_bytes(slot[40..48].try_into().unwrap());
        let expected = s ^ p ^ o ^ c;
        if parity != expected && parity != (expected ^ m) {
            return Err(format!(
                "Q42 resource '{key}' slot {idx} invalid XOR parity fold: got {parity:#018x}, expected {expected:#018x}"
            ));
        }
    }
    Ok(bytes.len() / 48)
}

/// Verify a .10d payload checking 10D magic/header bounds and MID2 sidecar integrity if present.
pub fn verify_10d_payload(key: &str, bytes: &[u8], max_bytes: usize) -> Result<(), String> {
    if bytes.len() > max_bytes || bytes.len() < 64 || !bytes.starts_with(&MAGIC_10D) {
        return Err(format!(".10d resource '{key}' invalid size, magic or truncated header"));
    }
    let budget = OpaquePreservationBudget {
        max_bytes,
        max_sections: 64,
    };
    let view = inspect_opaque_container(bytes, budget)
        .map_err(|e| format!(".10d structure/integrity invalid for '{key}': {e}"))?;
    let sections = parse_section_table(bytes, view.header())
        .map_err(|e| format!(".10d section table invalid for '{key}': {e}"))?;

    let has_mid2 = sections
        .iter()
        .any(|s| s.typ() == Some(SectionType::ManifoldIdentityV2));
    if has_mid2 {
        let mut scratch = [TypedFieldRecord::default(); MANIFOLD_IDENTITY_V2_MAX_FIELDS];
        let mut fields = [TypedFieldRecord::default(); MANIFOLD_IDENTITY_V2_MAX_FIELDS];
        let (_, field_count) = decode_container_manifold_identity(
            bytes,
            view.header(),
            &[],
            &mut scratch,
            &mut fields,
        )
        .map_err(|e| format!(".10d MID2 manifest invalid for '{key}': {e:?}"))?;

        let sidecar_opt = sections
            .iter()
            .position(|s| s.typ() == Some(SectionType::FieldSidecar))
            .and_then(|i| view.payload(i));

        let active_fields = &fields[..field_count];
        for field in active_fields {
            if field.payload_section_id == SectionType::FieldSidecar as u32 && sidecar_opt.is_none()
            {
                return Err(format!(".10d MID2 FieldSidecar section absent in '{key}'"));
            }
        }

        if let Some(sidecar) = sidecar_opt {
            let mut covered = 0u64;
            for field in active_fields {
                if active_fields.iter().any(|other| {
                    other.field_id != field.field_id
                        && field.payload_offset < other.payload_offset + other.payload_length
                        && other.payload_offset < field.payload_offset + field.payload_length
                }) {
                    return Err(format!(".10d MID2 overlapping field declarations in '{key}'"));
                }
                if field.payload_section_id == SectionType::FieldSidecar as u32 {
                    let end = field
                        .payload_offset
                        .checked_add(field.payload_length)
                        .ok_or_else(|| format!(".10d MID2 offset overflow in '{key}'"))?;
                    if end > sidecar.len() as u64 {
                        return Err(format!(".10d MID2 field extends beyond sidecar in '{key}'"));
                    }
                    covered = covered
                        .checked_add(field.payload_length)
                        .ok_or_else(|| format!(".10d MID2 coverage overflow in '{key}'"))?;
                }
            }
            if covered != sidecar.len() as u64 {
                return Err(format!(
                    ".10d MID2 sidecar coverage mismatch in '{key}': covered {covered}, sidecar has {}",
                    sidecar.len()
                ));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_q42_quin(seed: u64) -> [u8; 48] {
        let (s, p, o, c, m) = (
            0x1000_0000_0000_0000 | seed,
            0x2000_0000_0000_0000,
            0x3000_0000_0000_0000,
            0x4000_0000_0000_0000,
            0u64,
        );
        let parity = s ^ p ^ o ^ c;
        let mut slot = [0u8; 48];
        for (i, v) in [s, p, o, c, m, parity].iter().enumerate() {
            slot[i * 8..(i + 1) * 8].copy_from_slice(&v.to_le_bytes());
        }
        slot
    }

    fn sample_valid_m64() -> Vec<u8> {
        let raster = qualia_geospatial::ElevationRaster {
            width: 2,
            height: 2,
            cell_size_mm: 30_000,
            elevations_mm: &[100, 200, 300, 400],
        };
        let frame = qualia_geospatial::GeodeticFrame {
            srid: 7844,
            vertical_datum: 5711,
            epoch_year: 2020.0,
            origin_lat_deg: -35.0,
            origin_lon_deg: 148.0,
            origin_elevation_mm: 350_000,
        };
        let policy = qualia_geospatial::raster_m64::RasterM64Policy {
            rows: qualia_core_db::q42::m64_matter::M64RasterRows::South,
            edges: qualia_core_db::q42::m64_matter::M64RasterEdges::RepeatNearest,
            nodata_sentinel_mm: Some(i32::MIN),
        };
        let mut out = vec![0u8; 192 + 64 + 4096 * 64];
        let n = qualia_geospatial::raster_m64::compile_raster_to_m64_v3(
            &raster,
            &frame,
            1,
            policy,
            qualia_geospatial::raster_m64::RasterM64Limits {
                max_output_bytes: out.len(),
                max_chunks: 1,
                max_records: 4096,
            },
            &std::sync::atomic::AtomicBool::new(false),
            &mut out,
        )
        .unwrap();
        out.truncate(n);
        out
    }

    fn sample_valid_10d() -> Vec<u8> {
        let asset = qualia_core_db::render::compile_10d::compile_asset(
            b"v 0 0 0\nv 1 0 0\nv 0 1 0\nf 1 2 3\n",
            Some("obj"),
            "urn:test:geometry",
            "obj",
        )
        .unwrap();
        asset.container_10d
    }

    #[test]
    fn valid_m64_q42_and_10d_batch_passes() {
        let q42_slot = sample_q42_quin(42);
        let m64_bytes = sample_valid_m64();
        let ten_d_bytes = sample_valid_10d();

        let edits = vec![
            ResourceEdit::Add(ResourceData {
                key: "terrain/elevation.m64".into(),
                kind: "m64".into(),
                bytes: m64_bytes,
                meta: None,
            }),
            ResourceEdit::Add(ResourceData {
                key: "graphs/lineage.q42".into(),
                kind: "q42".into(),
                bytes: q42_slot.to_vec(),
                meta: None,
            }),
            ResourceEdit::Add(ResourceData {
                key: "models/mesh.10d".into(),
                kind: "10d".into(),
                bytes: ten_d_bytes,
                meta: None,
            }),
            ResourceEdit::Add(ResourceData {
                key: "textures/albedo.bin".into(),
                kind: "binary".into(),
                bytes: vec![1, 2, 3, 4],
                meta: None,
            }),
            ResourceEdit::Remove {
                key: "deprecated/old.txt".into(),
                expected: [0; 32],
            },
        ];

        let report = verify_atomic_resource_batch(&edits, 10_000_000).unwrap();
        assert_eq!(report.total_edits, 5);
        assert_eq!(report.verified_resources, 4);
        assert_eq!(report.m64_verified, 1);
        assert_eq!(report.q42_verified, 1);
        assert_eq!(report.ten_d_verified, 1);
        assert_eq!(report.opaque_passed, 1);
    }

    #[test]
    fn corrupt_m64_chunk_crc_fails_closed() {
        let mut m64_bytes = sample_valid_m64();
        m64_bytes[300] ^= 0xFF;

        let edits = vec![ResourceEdit::Add(ResourceData {
            key: "corrupt.m64".into(),
            kind: "m64".into(),
            bytes: m64_bytes,
            meta: None,
        })];

        let err = verify_atomic_resource_batch(&edits, 10_000_000).unwrap_err();
        assert!(err.contains("M64") || err.contains("CRC"));
    }

    #[test]
    fn corrupt_q42_length_and_parity_fail_closed() {
        let edits_len = vec![ResourceEdit::Add(ResourceData {
            key: "bad_len.q42".into(),
            kind: "q42".into(),
            bytes: vec![0u8; 20],
            meta: None,
        })];
        assert!(verify_atomic_resource_batch(&edits_len, 1000).is_err());

        let mut quin = sample_q42_quin(1);
        quin[47] ^= 0xFF;
        let edits_parity = vec![ResourceEdit::Add(ResourceData {
            key: "bad_parity.q42".into(),
            kind: "q42".into(),
            bytes: quin.to_vec(),
            meta: None,
        })];
        let err = verify_atomic_resource_batch(&edits_parity, 1000).unwrap_err();
        assert!(err.contains("invalid XOR parity fold"));
    }

    #[test]
    fn corrupt_10d_fails_closed() {
        let mut ten_d = sample_valid_10d();
        let mid = ten_d.len() - 10;
        ten_d[mid] ^= 0xAA;

        let edits = vec![ResourceEdit::Add(ResourceData {
            key: "corrupt.10d".into(),
            kind: "10d".into(),
            bytes: ten_d,
            meta: None,
        })];
        assert!(verify_atomic_resource_batch(&edits, 10_000_000).is_err());
    }

    #[test]
    fn budget_exhaustion_fails_closed() {
        let q42_slot = sample_q42_quin(7);
        let edits = vec![ResourceEdit::Add(ResourceData {
            key: "data.q42".into(),
            kind: "q42".into(),
            bytes: q42_slot.to_vec(),
            meta: None,
        })];
        let err = verify_atomic_resource_batch(&edits, 30).unwrap_err();
        assert!(err.contains("budget"));
    }

    #[test]
    fn duplicate_empty_and_nul_keys_fail_closed() {
        let edits_dup = vec![
            ResourceEdit::Remove {
                key: "same.bin".into(),
                expected: [0; 32],
            },
            ResourceEdit::Remove {
                key: "same.bin".into(),
                expected: [1; 32],
            },
        ];
        assert!(verify_atomic_resource_batch(&edits_dup, 1000).is_err());

        let edits_empty = vec![ResourceEdit::Remove {
            key: "".into(),
            expected: [0; 32],
        }];
        assert!(verify_atomic_resource_batch(&edits_empty, 1000).is_err());

        let edits_nul = vec![ResourceEdit::Remove {
            key: "bad\0key.bin".into(),
            expected: [0; 32],
        }];
        assert!(verify_atomic_resource_batch(&edits_nul, 1000).is_err());
    }
}
