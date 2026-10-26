use super::*;
use qualia_core_db::container_10d::crc32c::crc32c;
use qualia_core_db::q42::m64_matter::{M64RasterEdges, M64RasterRows};
use std::sync::atomic::{AtomicUsize, Ordering};
static NEXT: AtomicUsize = AtomicUsize::new(0);

fn fixture(version: u32) -> Vec<u8> {
    let header_size = if version == 3 { 192 } else { 64 };
    let payload_start = header_size + 64;
    let mut bytes = vec![0u8; payload_start + 4096 * 64];
    let total = bytes.len() as u64;
    bytes[..4].copy_from_slice(b"m64\0");
    bytes[4..8].copy_from_slice(&version.to_le_bytes());
    bytes[8..16].copy_from_slice(&(header_size as u64).to_le_bytes());
    bytes[16..24].copy_from_slice(&total.to_le_bytes());
    let meta = M64GeoRasterMetadata {
        width: 1,
        height: 1,
        origin_lat_deg: -35.7812345678901,
        origin_lon_deg: 148.298765432109,
        origin_elevation_mm: 350_001,
        cell_size_mm: 30_001,
        epoch_year: 2020.25,
        srid: 7844,
        vertical_datum: 5711,
        rows: M64RasterRows::South,
        edges: M64RasterEdges::RepeatNearest,
        nodata_sentinel_mm: Some(i32::MIN),
    };
    bytes[24..28].copy_from_slice(&(meta.origin_lat_deg as f32).to_le_bytes());
    bytes[28..32].copy_from_slice(&(meta.origin_lon_deg as f32).to_le_bytes());
    bytes[32..36].copy_from_slice(&(meta.origin_elevation_mm as f32 / 1000.0).to_le_bytes());
    bytes[36..40].copy_from_slice(&1.0f32.to_le_bytes());
    bytes[40..48].copy_from_slice(&(header_size as u64).to_le_bytes());
    bytes[48..56].copy_from_slice(&1u64.to_le_bytes());
    let crc = crc32c(&bytes[..56]);
    bytes[56..60].copy_from_slice(&crc.to_le_bytes());
    if version == 3 {
        meta.encode(&mut bytes[64..192]).unwrap();
    }
    let dir = &mut bytes[header_size..payload_start];
    dir[24..32].copy_from_slice(&(payload_start as u64).to_le_bytes());
    dir[32..36].copy_from_slice(&(4096u32 * 64).to_le_bytes());
    dir[36..40].copy_from_slice(&4096u32.to_le_bytes());
    dir[40..44].copy_from_slice(&meta.srid.to_le_bytes());
    dir[46] = 2;
    let crc = crc32c(&dir[..60]);
    dir[60..64].copy_from_slice(&crc.to_le_bytes());
    for record in bytes[payload_start..].chunks_exact_mut(64) {
        record[..4].copy_from_slice(&i32::MIN.to_le_bytes());
        let crc = crc32c(&record[..60]);
        record[60..64].copy_from_slice(&crc.to_le_bytes());
    }
    bytes
}

#[test]
fn canonical_v3_and_legacy_inspection_are_distinct() {
    let mut scratch = [M64PayloadInterval::default(); 1];
    let result = inspect_payload(&fixture(3), ui_inspection_limits(), &mut scratch).unwrap();
    assert_eq!(result.header.version, 3);
    let meta = result.georaster.unwrap();
    assert_eq!(meta.cell_size_mm, 30_001);
    assert_eq!(meta.vertical_datum, 5711);
    assert_eq!(
        meta.origin_lat_deg.to_bits(),
        (-35.7812345678901f64).to_bits()
    );
    assert_eq!(
        result.first_source_sample,
        Some(M64RasterSample::Nodata(i32::MIN))
    );
    assert_eq!(result.verification.record_crc_checked, 4096);
    for version in [1, 2] {
        let result =
            inspect_payload(&fixture(version), ui_inspection_limits(), &mut scratch).unwrap();
        assert_eq!(result.header.version, version);
        assert!(result.georaster.is_none());
        assert!(result.first_source_sample.is_none());
    }
}

#[test]
fn corruption_unknown_payload_and_budgets_never_fallback() {
    let mut scratch = [M64PayloadInterval::default(); 1];
    let mut bytes = fixture(3);
    let result = inspect_payload(b"unknown bytes", ui_inspection_limits(), &mut scratch);
    assert!(result
        .unwrap_err()
        .contains("unassessed without reinterpretation"));
    let last = bytes.len() - 1;
    bytes[last] ^= 1;
    assert!(
        inspect_payload(&bytes, ui_inspection_limits(), &mut scratch)
            .unwrap_err()
            .contains("Canonical M64 inspection refused")
    );
    bytes = fixture(3);
    let mut limits = ui_inspection_limits();
    limits.max_file_bytes = bytes.len() - 1;
    let error = inspect_payload(&bytes, limits, &mut scratch).unwrap_err();
    assert!(error.contains(&bytes.len().to_string()) && error.contains("limit"));
    assert!(!error.contains("corrupt"));
    let error = inspect_payload(&bytes, ui_inspection_limits(), &mut []).unwrap_err();
    assert!(error.contains("requires 1") && error.contains("scratch 0"));
    limits = ui_inspection_limits();
    limits.max_records = 4095;
    assert!(inspect_payload(&bytes, limits, &mut scratch)
        .unwrap_err()
        .contains("record budget"));
}

fn saved(root: &Path) -> (CheckpointService, ResourceInspectionRequest) {
    let id = format!(
        "urn:poet:m64-inspection:{}:{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    );
    let (_, stamp) = studio_store::create_artifact_if_absent(StudioArtifact::new(
        &id,
        "Geometry",
        "M64 inspection fixture",
    ))
    .unwrap();
    let limits = CheckpointLimits::default();
    let checkpoint = core::capture(&id, &stamp, limits)
        .unwrap()
        .apply_resource_edits(
            vec![
                ResourceEdit::Add(ResourceData {
                    key: "spatial/raster.m64".into(),
                    kind: "m64".into(),
                    bytes: fixture(3),
                    meta: Some(vec![0xa1, 0x61, b'x', 1]),
                }),
                ResourceEdit::Add(ResourceData {
                    key: "unknown/raw".into(),
                    kind: "future/raw".into(),
                    bytes: b"unknown inert payload".to_vec(),
                    meta: Some(vec![0xa1, 0x61, b'z', 2]),
                }),
            ],
            limits,
        )
        .unwrap();
    let resource_digest = checkpoint.inventory()[0].digest;
    let bytes = core::encode(&checkpoint, vec![], limits).unwrap();
    let receipt = core::save(
        &root.join("inspection.hmc"),
        &bytes,
        ExpectedFile::Absent,
        limits,
    )
    .unwrap();
    let service = CheckpointService::new(
        root,
        TrustedBinding {
            project_id: "fixture-project".into(),
            document_id: "fixture-document".into(),
            source_id: id,
            domain: "Geometry".into(),
        },
        limits,
    )
    .unwrap();
    (
        service,
        ResourceInspectionRequest {
            filename: "inspection.hmc".into(),
            package_digest: receipt.digest,
            key: "spatial/raster.m64".into(),
            resource_digest,
        },
    )
}

#[test]
fn selected_hmc_digests_and_unknown_metadata_remain_exact() {
    let root = tempfile::tempdir().unwrap();
    let (service, request) = saved(root.path());
    let before = std::fs::read(root.path().join(&request.filename)).unwrap();
    let mut scratch = [M64PayloadInterval::default(); 1];
    let result = service
        .inspect_resource(request.clone(), ui_inspection_limits(), &mut scratch)
        .unwrap();
    assert_eq!(result.request, request);
    assert!(result.payload.georaster.is_some());
    for field in 0..3 {
        let mut stale = request.clone();
        match field {
            0 => stale.package_digest[0] ^= 1,
            1 => stale.resource_digest[0] ^= 1,
            _ => stale.key = "absent".into(),
        }
        assert!(service
            .inspect_resource(stale, ui_inspection_limits(), &mut scratch)
            .is_err());
    }
    let inventory = service.inventory(&request.filename).unwrap();
    assert!(result.matches_inventory(&request.filename, &inventory));
    assert!(!result.matches_inventory("different.hmc", &inventory));
    let mut stale_inventory = inventory.clone();
    stale_inventory.digest[0] ^= 1;
    assert!(!result.matches_inventory(&request.filename, &stale_inventory));
    let unknown = inventory
        .resources
        .iter()
        .find(|resource| resource.key == "unknown/raw")
        .unwrap();
    let mut unsupported = request.clone();
    unsupported.key = unknown.key.clone();
    unsupported.resource_digest = unknown.digest;
    assert!(service
        .inspect_resource(unsupported, ui_inspection_limits(), &mut scratch)
        .unwrap_err()
        .contains("unknown resource"));
    assert_eq!(
        std::fs::read(root.path().join(&request.filename)).unwrap(),
        before
    );
    let carrier = core::load(
        &root.path().join(&request.filename),
        CheckpointLimits::default(),
    )
    .unwrap();
    assert_eq!(
        carrier
            .resources()
            .iter()
            .find(|resource| resource.key == "unknown/raw")
            .unwrap()
            .meta,
        Some(vec![0xa1, 0x61, b'z', 2])
    );
}

#[test]
fn native_service_preserves_immutable_pin_and_path_refusals() {
    let root = tempfile::tempdir().unwrap();
    let (service, request) = saved(root.path());
    let mut scratch = [M64PayloadInterval::default(); 1];
    let pinned = CheckpointService::new_with_policy(
        root.path(),
        service.binding().clone(),
        CheckpointLimits::default(),
        PackageWritePolicy::ImmutableRelease {
            filename: request.filename.clone(),
            digest: [0; 32],
        },
    )
    .unwrap();
    assert!(pinned
        .inspect_resource(request.clone(), ui_inspection_limits(), &mut scratch)
        .is_err());
    let mut wrong_path = request;
    wrong_path.filename = "../outside.hmc".into();
    assert!(service
        .inspect_resource(wrong_path, ui_inspection_limits(), &mut scratch)
        .is_err());
}
