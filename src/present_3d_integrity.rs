//! Inspector labels are derived from bounded readers, never from file suffixes.
use qualia_core_db::q42::{emf_container as emf, m64_matter as m64};

pub(super) struct Inspection {
    pub valid: bool,
    pub status: String,
    pub records: String,
    pub layout: String,
}

pub(super) fn inspect(bytes: &[u8], matter: bool) -> Inspection {
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

fn inspect_checked(bytes: &[u8], matter: bool) -> Result<Inspection, String> {
    if matter {
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
            status: format!("{} full BLAKE3 chunks · {} record CRCs verified",
                verified.full_hash_chunks, verified.record_crc_checked),
            records: format!("{} records in {} chunks", verified.total_records, verified.chunks),
            layout: if reader.georaster().is_some() { "GRM3 georaster" }
                else if reader.lrm3().is_some() { "LRM3 local metric" }
                else { "M64 directory indexed" }.into(),
        })
    } else {
        let reader = emf::EmfContainerReader::parse(bytes).map_err(|e| e.to_string())?;
        let records: usize = (0..reader.chunk_count())
            .map(|i| reader.chunk_payload(i).map_or(0, |data| data.len() / 64)).sum();
        Ok(Inspection {
            valid: true,
            status: format!("EMF v{} · {} full BLAKE3 chunks + CRC-32C verified",
                reader.header().version, reader.chunk_count()),
            records: format!("{records} field cells in {} chunks", reader.chunk_count()),
            layout: "8×8×8 Hilbert field bricks".into(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn suffix_or_magic_alone_cannot_claim_verification() {
        for matter in [false, true] {
            let inspected = inspect(b"Q42EMF64", matter);
            assert!(inspected.status.starts_with("Verification refused"));
            assert_eq!(inspected.records, "Not verified");
        }
    }

    #[test]
    fn emf_counts_cells_without_directory_or_hash_trailer() {
        let mut writer = emf::EmfContainerWriter::new([0.0; 3], 1.0, [1; 3], 0.0);
        let cells = [emf::EmfCellRecord64::default(); emf::EMF_BRICK_CELLS];
        writer.add_brick_chunk(0, &cells, 0, 1, 0).unwrap();
        let inspected = inspect(&writer.to_bytes().unwrap(), false);
        assert_eq!(inspected.records, "512 field cells in 1 chunks");
        assert!(inspected.status.contains("1 full BLAKE3 chunks"));
        assert!(!inspected.status.contains("Bao"));
    }
}
