//! Inspector labels are derived from bounded readers, never from file suffixes.
use qualia_core_db::container_10d::header::Container10dHeader;
use qualia_core_db::container_10d::section::parse_section_table;
use qualia_core_db::q42::field_envelope::{Family, Limits, StructuralView};
use qualia_core_db::q42::{emf_container as emf, m64_matter as m64};

pub struct Inspection {
    pub valid: bool,
    pub status: String,
    pub records: String,
    pub layout: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BundleIntegrity {
    pub valid: bool,
    pub ten_d_verified: bool,
    pub m64_verified: bool,
    pub emf_verified: bool,
    pub emf_entry_count: u32,
    pub emf_record_count: u64,
    pub status: String,
}

/// Verifies a current EMF v4 (`Q42EMF64` v4) envelope and every chunk's BLAKE3 digest.
pub fn verify_emf(bytes: &[u8]) -> Result<(), String> {
    verify_emf_v4_summary(bytes).map(|_| ())
}

/// Verifies a `.10d` mesh container with optional `.m64` and `.emf` siblings before portal embedding.
pub fn inspect_with_emf(
    ten_d: &[u8],
    m64_bytes: Option<&[u8]>,
    emf_bytes: Option<&[u8]>,
) -> Result<BundleIntegrity, String> {
    let header = Container10dHeader::parse(ten_d)
        .map_err(|e| format!("10d header verification refused: {e:?}"))?;
    let _sections = parse_section_table(ten_d, &header)
        .map_err(|e| format!("10d section table verification refused: {e:?}"))?;

    if let Some(m64_slice) = m64_bytes {
        inspect_m64(m64_slice)?;
    }

    let (emf_verified, emf_entry_count, emf_record_count) = match emf_bytes {
        Some(slice) => {
            let (entries, records) = verify_emf_v4_summary(slice)?;
            (true, entries, records)
        }
        None => (false, 0, 0),
    };

    let status = if emf_verified {
        format!(
            "10d + EMF v4 verified ({emf_entry_count} entries · {emf_record_count} records)"
        )
    } else if m64_bytes.is_some() {
        "10d + M64 verified".into()
    } else {
        "10d verified".into()
    };

    Ok(BundleIntegrity {
        valid: true,
        ten_d_verified: true,
        m64_verified: m64_bytes.is_some(),
        emf_verified,
        emf_entry_count,
        emf_record_count,
        status,
    })
}

/// Backward-compatible wrapper preserving `.10d` + optional `.m64` inspection without `.emf`.
pub fn inspect(ten_d: &[u8], m64_bytes: Option<&[u8]>) -> Result<BundleIntegrity, String> {
    inspect_with_emf(ten_d, m64_bytes, None)
}

pub(super) fn inspect_container(bytes: &[u8], matter: bool) -> Inspection {
    match inspect_checked(bytes, matter) {
        Ok(info) => info,
        Err(_) => Inspection {
            valid: false,
            status: "Verification refused by format reader".into(),
            records: "Not verified".into(),
            layout: "Not verified".into(),
        },
    }
}

fn verify_emf_v4_summary(bytes: &[u8]) -> Result<(u32, u64), String> {
    let view = StructuralView::open(bytes, Family::EmfV4Candidate, Limits::default())
        .map_err(|e| format!("EMF v4 container verification refused: {e:?}"))?;
    let entry_count = view.entry_count();
    let mut record_count = 0u64;
    for index in 1..=entry_count {
        let entry = view
            .entry(index)
            .map_err(|e| format!("EMF v4 entry {index} refused: {e:?}"))?;
        let _verified = view
            .verify_chunk(index)
            .map_err(|e| format!("EMF v4 chunk {index} verification refused: {e:?}"))?;
        record_count = record_count
            .checked_add(entry.record_count())
            .ok_or_else(|| "EMF v4 record count overflow".to_string())?;
    }
    Ok((entry_count, record_count))
}

fn inspect_m64(bytes: &[u8]) -> Result<Inspection, String> {
    let limits = m64::M64ReadLimits {
        max_file_bytes: 64 * 1024 * 1024,
        max_chunks: 4096,
        max_records: 1_048_576,
    };
    let mut scratch = vec![m64::M64PayloadInterval::default(); limits.max_chunks];
    let reader = m64::M64Reader::parse(bytes, limits, &mut scratch)
        .map_err(|e| e.to_string())?;
    let verified = reader.verification();
    Ok(Inspection {
        valid: true,
        status: format!(
            "{} full BLAKE3 chunks · {} record CRCs verified",
            verified.full_hash_chunks, verified.record_crc_checked
        ),
        records: format!(
            "{} records in {} chunks",
            verified.total_records, verified.chunks
        ),
        layout: if reader.georaster().is_some() {
            "GRM3 georaster"
        } else if reader.lrm3().is_some() {
            "LRM3 local metric"
        } else {
            "M64 directory indexed"
        }
        .into(),
    })
}

fn is_emf_v4_prefix(bytes: &[u8]) -> bool {
    bytes.len() >= 10
        && &bytes[..8] == b"Q42EMF64"
        && u16::from_le_bytes([bytes[8], bytes[9]]) == 4
}

fn inspect_checked(bytes: &[u8], matter: bool) -> Result<Inspection, String> {
    if matter {
        inspect_m64(bytes)
    } else if is_emf_v4_prefix(bytes) {
        let (entries, records) = verify_emf_v4_summary(bytes)?;
        Ok(Inspection {
            valid: true,
            status: format!("EMF v4 · {entries} BLAKE3 chunk(s) verified"),
            records: format!("{records} records in {entries} entries"),
            layout: "Q42EMF64 v4 field_envelope".into(),
        })
    } else {
        let reader = emf::EmfContainerReader::parse(bytes).map_err(|e| e.to_string())?;
        let mut records = 0usize;
        for i in 0..reader.chunk_count() {
            let chunk = reader
                .chunk_payload(i)
                .ok_or_else(|| format!("Legacy EMF chunk {i} missing or out of bounds"))?;
            records += chunk.len() / 64;
        }
        Ok(Inspection {
            valid: true,
            status: format!(
                "EMF v{} · {} full BLAKE3 chunks + CRC-32C verified",
                reader.header().version,
                reader.chunk_count()
            ),
            records: format!("{records} field cells in {} chunks", reader.chunk_count()),
            layout: "8×8×8 Hilbert field bricks".into(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use qualia_core_db::q42::field_envelope::{EntryInput, EnvelopeInput};

    fn sample_10d_bytes() -> Vec<u8> {
        use qualia_core_db::render::assets::Mesh;
        use qualia_core_db::render::compile_10d::compile_mesh_to_10d;
        let mesh = Mesh {
            positions: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            triangles: vec![[0, 1, 2]],
            min: [0.0, 0.0, 0.0],
            max: [1.0, 1.0, 0.0],
        };
        compile_mesh_to_10d(&mesh).expect("compile sample 10d")
    }

    fn sample_emf_v4_bytes() -> Vec<u8> {
        let mut payload = vec![0u8; 128];
        payload[..8].copy_from_slice(&1.0_f64.to_le_bytes());
        let entry = EntryInput {
            mode: 0,
            revision: 1,
            flags: 0,
            stream_id: 0,
            frame_id: 1,
            start_tick: 0,
            time_count: 0,
            spatial_key: 0,
            record_count: 1,
            stride: 128,
            codec: 0,
            descriptor_id: 1,
            payload: &payload,
        };
        let input = EnvelopeInput {
            family: Family::EmfV4Candidate,
            flags: 0,
            metadata: &[],
            entries: &[entry],
        };
        let mut out = vec![0u8; input.encoded_len(Limits::default()).expect("emf v4 size")];
        input
            .encode_into(&mut out, Limits::default())
            .expect("encode emf v4");
        out
    }

    #[test]
    fn suffix_or_magic_alone_cannot_claim_verification() {
        for matter in [false, true] {
            let inspected = inspect_container(b"Q42EMF64", matter);
            assert!(inspected.status.starts_with("Verification refused"));
            assert_eq!(inspected.records, "Not verified");
        }
    }

    #[test]
    fn emf_counts_cells_without_directory_or_hash_trailer() {
        let mut writer = emf::EmfContainerWriter::new([0.0; 3], 1.0, [1; 3], 0.0);
        let cells = [emf::EmfCellRecord64::default(); emf::EMF_BRICK_CELLS];
        writer.add_brick_chunk(0, &cells, 0, 1, 0).unwrap();
        let inspected = inspect_container(&writer.to_bytes().unwrap(), false);
        assert_eq!(inspected.records, "512 field cells in 1 chunks");
        assert!(inspected.status.contains("1 full BLAKE3 chunks"));
        assert!(!inspected.status.contains("Bao"));
    }

    #[test]
    fn verify_emf_accepts_valid_v4_and_rejects_corrupt_bytes() {
        let valid = sample_emf_v4_bytes();
        assert!(verify_emf(&valid).is_ok());

        let container_info = inspect_container(&valid, false);
        assert!(container_info.valid);
        assert!(container_info.status.contains("EMF v4"));

        // Corrupt chunk payload byte -> fails BLAKE3 chunk verification.
        let view = StructuralView::open(&valid, Family::EmfV4Candidate, Limits::default()).unwrap();
        let payload_at = view.entry(1).unwrap().payload_offset() as usize;
        let mut tampered_payload = valid.clone();
        tampered_payload[payload_at] ^= 0x01;
        assert!(verify_emf(&tampered_payload).is_err());
        assert!(!inspect_container(&tampered_payload, false).valid);

        // Corrupt header / magic / truncation -> fail closed.
        let mut bad_header = valid.clone();
        bad_header[8] ^= 0x01;
        assert!(verify_emf(&bad_header).is_err());
        assert!(verify_emf(b"Q42EMF64").is_err());
        assert!(verify_emf(&valid[..valid.len() - 4]).is_err());
    }

    #[test]
    fn inspect_with_emf_verifies_bundle_and_fails_closed_on_corrupt_emf() {
        let ten_d = sample_10d_bytes();
        let emf_v4 = sample_emf_v4_bytes();

        let mesh_only = inspect(&ten_d, None).expect("10d inspect");
        assert!(mesh_only.valid);
        assert!(mesh_only.ten_d_verified);
        assert!(!mesh_only.emf_verified);

        let paired = inspect_with_emf(&ten_d, None, Some(&emf_v4)).expect("10d + emf v4 inspect");
        assert!(paired.valid);
        assert!(paired.ten_d_verified);
        assert!(paired.emf_verified);
        assert_eq!(paired.emf_entry_count, 1);
        assert_eq!(paired.emf_record_count, 1);

        let mut corrupt_emf = emf_v4.clone();
        let view =
            StructuralView::open(&corrupt_emf, Family::EmfV4Candidate, Limits::default()).unwrap();
        let at = view.entry(1).unwrap().payload_offset() as usize;
        corrupt_emf[at] ^= 0xff;
        assert!(inspect_with_emf(&ten_d, None, Some(&corrupt_emf)).is_err());
        assert!(inspect_with_emf(b"Q42TEN10\x00bad", None, Some(&emf_v4)).is_err());
    }
}
