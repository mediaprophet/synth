use super::*;
use qualia_core_db::q42::m64_matter::{M64RasterEdges, M64RasterRows, M64RasterSample};
use qualia_geospatial::GeodeticFrame;

fn fixture() -> Vec<u8> {
    let raster = ElevationRaster {
        width: 2,
        height: 2,
        cell_size_mm: 30_001,
        elevations_mm: &[100_001, i32::MIN, -1_203, 4_005],
    };
    let frame = GeodeticFrame {
        srid: 7844,
        vertical_datum: 5711,
        epoch_year: 2020.25,
        origin_lat_deg: -35.7812345678901,
        origin_lon_deg: 148.298765432109,
        origin_elevation_mm: 350_001,
    };
    let policy = RasterM64Policy {
        rows: M64RasterRows::South,
        edges: M64RasterEdges::RepeatNearest,
        nodata_sentinel_mm: Some(i32::MIN),
    };
    let mut out = vec![0; 192 + 64 + 4096 * 64];
    let n = compile_raster_to_m64_v3(
        &raster,
        &frame,
        7,
        policy,
        RasterM64Limits {
            max_output_bytes: out.len(),
            max_chunks: 1,
            max_records: 4096,
        },
        &AtomicBool::new(false),
        &mut out,
    )
    .unwrap();
    out.truncate(n);
    out
}
fn request() -> GeoRasterEditRequest {
    GeoRasterEditRequest {
        resource: ResourceInspectionRequest {
            filename: "fixture.hmc".into(),
            package_digest: [0; 32],
            key: "raster.m64".into(),
            resource_digest: [0; 32],
        },
        origin_elevation_mm: 555_111,
        cell_size_mm: 15_003,
        sample_x: 1,
        sample_y: 0,
        sample_mm: -42_005,
    }
}

#[test]
fn georaster_edit_canonical_samples_metadata_and_padding_roundtrip() {
    let bytes = fixture();
    let request = request();
    let mut scratch = [M64PayloadInterval::default(); 1];
    let out = replace_payload(
        &bytes,
        &request,
        ui_georaster_edit_limits(),
        &AtomicBool::new(false),
        &mut scratch,
    )
    .unwrap();
    let reader = M64Reader::parse(&out, ui_inspection_limits(), &mut scratch).unwrap();
    let meta = reader.georaster().unwrap();
    assert_eq!(meta.origin_elevation_mm, 555_111);
    assert_eq!(meta.cell_size_mm, 15_003);
    assert_eq!(
        meta.origin_lat_deg.to_bits(),
        (-35.7812345678901f64).to_bits()
    );
    assert_eq!(
        meta.origin_lon_deg.to_bits(),
        (148.298765432109f64).to_bits()
    );
    assert_eq!((meta.srid, meta.vertical_datum), (7844, 5711));
    assert_eq!(meta.epoch_year.to_bits(), 2020.25f32.to_bits());
    assert_eq!(meta.nodata_sentinel_mm, Some(i32::MIN));
    assert_eq!(meta.rows, M64RasterRows::South);
    assert_eq!(
        reader.chunk(0).unwrap().directory_entry().lut_bank_epoch_id,
        7
    );
    let mut samples = [0; 4];
    decode_raster_from_m64_v3(&reader, &AtomicBool::new(false), &mut samples).unwrap();
    assert_eq!(samples, [100_001, -42_005, -1_203, 4_005]);
    assert_eq!(reader.raster_sample(2, 0), None);
    let mut nodata = request;
    nodata.sample_mm = i32::MIN;
    let out = replace_payload(
        &bytes,
        &nodata,
        ui_georaster_edit_limits(),
        &AtomicBool::new(false),
        &mut scratch,
    )
    .unwrap();
    let reader = M64Reader::parse(&out, ui_inspection_limits(), &mut scratch).unwrap();
    assert_eq!(
        reader.raster_sample(1, 0),
        Some(M64RasterSample::Nodata(i32::MIN))
    );
}

#[test]
fn georaster_edit_cancellation_budgets_and_coordinates_fail_closed() {
    let bytes = fixture();
    let before = bytes.clone();
    let request = request();
    let mut scratch = [M64PayloadInterval::default(); 1];
    assert!(replace_payload(
        &bytes,
        &request,
        ui_georaster_edit_limits(),
        &AtomicBool::new(true),
        &mut scratch
    )
    .unwrap_err()
    .contains("cancelled"));
    let mut limits = ui_georaster_edit_limits();
    limits.max_working_bytes = bytes.len() + 15;
    assert!(replace_payload(
        &bytes,
        &request,
        limits,
        &AtomicBool::new(false),
        &mut scratch
    )
    .unwrap_err()
    .contains("working budget"));
    limits = ui_georaster_edit_limits();
    limits.read.max_records = 4095;
    assert!(replace_payload(
        &bytes,
        &request,
        limits,
        &AtomicBool::new(false),
        &mut scratch
    )
    .is_err());
    limits = ui_georaster_edit_limits();
    limits.read.max_file_bytes = bytes.len() - 1;
    assert!(replace_payload(
        &bytes,
        &request,
        limits,
        &AtomicBool::new(false),
        &mut scratch
    )
    .is_err());
    assert!(replace_payload(
        &bytes,
        &request,
        ui_georaster_edit_limits(),
        &AtomicBool::new(false),
        &mut []
    )
    .is_err());
    let mut invalid = request;
    invalid.sample_x = 2;
    assert!(replace_payload(
        &bytes,
        &invalid,
        ui_georaster_edit_limits(),
        &AtomicBool::new(false),
        &mut scratch
    )
    .unwrap_err()
    .contains("source extent"));
    invalid.cell_size_mm = 0;
    assert!(invalid.validate_bounds().is_err());
    assert_eq!(bytes, before);
}

#[test]
fn georaster_edit_legacy_corrupt_and_richer_payloads_refuse_loss() {
    use qualia_core_db::container_10d::crc32c::crc32c;
    use qualia_geospatial::compile_raster_to_m64;
    let bytes = fixture();
    let mut scratch = [M64PayloadInterval::default(); 1];
    let request = request();
    let frame = GeodeticFrame::default();
    let raster = ElevationRaster {
        width: 1,
        height: 1,
        cell_size_mm: 10,
        elevations_mm: &[1],
    };
    let mut legacy = vec![0; 64 + 64 + 4096 * 64];
    let written = compile_raster_to_m64(&raster, &frame, 0, &mut legacy).unwrap();
    legacy.truncate(written);
    assert!(replace_payload(
        &legacy,
        &request,
        ui_georaster_edit_limits(),
        &AtomicBool::new(false),
        &mut scratch
    )
    .unwrap_err()
    .contains("automatic upgrade refused"));
    legacy[4..8].copy_from_slice(&2u32.to_le_bytes());
    let crc = crc32c(&legacy[..56]);
    legacy[56..60].copy_from_slice(&crc.to_le_bytes());
    assert!(replace_payload(
        &legacy,
        &request,
        ui_georaster_edit_limits(),
        &AtomicBool::new(false),
        &mut scratch
    )
    .unwrap_err()
    .contains("automatic upgrade refused"));
    let mut richer = bytes.clone();
    // Test-only add an inert non-elevation strata byte and renew its record CRC.
    // Fixture construction remains canonical; production edits never encode records.
    let start = 256;
    richer[start + 4] = 1;
    let crc = crc32c(&richer[start..start + 60]);
    richer[start + 60..start + 64].copy_from_slice(&crc.to_le_bytes());
    assert!(replace_payload(
        &richer,
        &request,
        ui_georaster_edit_limits(),
        &AtomicBool::new(false),
        &mut scratch
    )
    .is_err());
    let mut alias = bytes.clone();
    alias[..4].copy_from_slice(b"e64\0");
    let crc = crc32c(&alias[..56]);
    alias[56..60].copy_from_slice(&crc.to_le_bytes());
    assert!(replace_payload(
        &alias,
        &request,
        ui_georaster_edit_limits(),
        &AtomicBool::new(false),
        &mut scratch
    )
    .is_err());
    let mut noncanonical = bytes.clone();
    noncanonical[192] ^= 1;
    let crc = crc32c(&noncanonical[192..252]);
    noncanonical[252..256].copy_from_slice(&crc.to_le_bytes());
    assert!(replace_payload(
        &noncanonical,
        &request,
        ui_georaster_edit_limits(),
        &AtomicBool::new(false),
        &mut scratch
    )
    .unwrap_err()
    .contains("not preserved"));
    let mut corrupt = bytes;
    corrupt[256] ^= 1;
    assert!(replace_payload(
        &corrupt,
        &request,
        ui_georaster_edit_limits(),
        &AtomicBool::new(false),
        &mut scratch
    )
    .unwrap_err()
    .contains("admission refused"));
}
