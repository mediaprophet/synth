use super::*;
use qualia_core_db::q42::m64_matter::{M64RasterEdges, M64RasterRows};
use qualia_geospatial::GeodeticFrame;

fn fixture(first_mm: i32) -> Vec<u8> {
    let values = [first_mm, i32::MIN, -1_203, 4_005];
    let raster = ElevationRaster { width: 2, height: 2, cell_size_mm: 30_001, elevations_mm: &values };
    let frame = GeodeticFrame {
        srid:7844,vertical_datum:5711,epoch_year:2020.25,
        origin_lat_deg:-35.7812345678901,origin_lon_deg:148.298765432109,origin_elevation_mm:350_001,
    };
    let mut bytes = vec![0; 192 + 64 + 4096 * 64];
    let length = compile_raster_to_m64_v3(&raster, &frame, 7, RasterM64Policy {
        rows:M64RasterRows::South,edges:M64RasterEdges::RepeatNearest,nodata_sentinel_mm:Some(i32::MIN),
    }, RasterM64Limits {max_output_bytes:bytes.len(),max_chunks:1,max_records:4096},
        &AtomicBool::new(false), &mut bytes).unwrap();
    bytes.truncate(length);
    bytes
}

fn request() -> GeoRasterBrushRequest {
    GeoRasterBrushRequest {
        resource:ResourceInspectionRequest {filename:"fixture.hmc".into(),package_digest:[0;32],key:"raster.m64".into(),resource_digest:[0;32]},
        window_x:0,window_y:0,window_side:1,
        samples:vec![StrokeSample {
            stroke_id:1,timestamp_ms:0,center_x_m:0.0,center_y_m:0.0,radius_m:1.0,strength:2.0,
            target_height_m:0.0,step_m:0.0,op_code:0,falloff_code:0,layer_index:0,flags:0,seed:42,pad:[0;2],
        }],
    }
}

#[test]
fn brush_retains_exact_m64_frame_policy_lut_nodata_and_unselected_samples() {
    let bytes = fixture(100_001);
    let mut scratch = [M64PayloadInterval::default();1];
    let original = M64Reader::parse(&bytes,ui_georaster_edit_limits().read,&mut scratch).unwrap();
    let original_meta = *original.georaster().unwrap();
    let out = brush_payload(&bytes,&request(),ui_georaster_edit_limits(),&AtomicBool::new(false),&mut scratch).unwrap();
    let edited = M64Reader::parse(&out,ui_georaster_edit_limits().read,&mut scratch).unwrap();
    assert_eq!(edited.georaster(),Some(&original_meta));
    assert_eq!(edited.georaster().unwrap().origin_lat_deg.to_bits(),original_meta.origin_lat_deg.to_bits());
    assert_eq!(edited.georaster().unwrap().origin_lon_deg.to_bits(),original_meta.origin_lon_deg.to_bits());
    assert_eq!(edited.georaster().unwrap().epoch_year.to_bits(),original_meta.epoch_year.to_bits());
    assert_eq!(edited.chunk(0).unwrap().directory_entry().lut_bank_epoch_id,7);
    let mut recovered = [0;4];
    decode_raster_from_m64_v3(&edited,&AtomicBool::new(false),&mut recovered).unwrap();
    // Absolute millimetres: the origin declaration is never added.
    assert_eq!(recovered,[102_001,i32::MIN,-1_203,4_005]);
}

#[test]
fn nodata_masked_extent_cancellation_budget_and_unknown_operation_refuse() {
    let bytes = fixture(100_001);
    let before = bytes.clone();
    let mut scratch = [M64PayloadInterval::default();1];
    let mut r = request();
    r.window_side = 2;
    assert!(brush_payload(&bytes,&r,ui_georaster_edit_limits(),&AtomicBool::new(false),&mut scratch).unwrap_err().contains("nodata"));
    r = request(); r.window_x = 2;
    assert!(brush_payload(&bytes,&r,ui_georaster_edit_limits(),&AtomicBool::new(false),&mut scratch).is_err());
    r = request(); r.samples[0].op_code = 99;
    assert!(brush_payload(&bytes,&r,ui_georaster_edit_limits(),&AtomicBool::new(false),&mut scratch).is_err());
    assert!(brush_payload(&bytes,&request(),ui_georaster_edit_limits(),&AtomicBool::new(true),&mut scratch).is_err());
    let mut small = ui_georaster_edit_limits(); small.max_working_bytes = 79;
    assert!(brush_payload(&bytes,&request(),small,&AtomicBool::new(false),&mut scratch).is_err());
    assert_eq!(bytes,before);
}

#[test]
fn precision_range_sentinel_and_noop_refuse_without_implicit_conversion() {
    let mut scratch = [M64PayloadInterval::default();1];
    assert!(brush_payload(&fixture(i32::MAX),&request(),ui_georaster_edit_limits(),&AtomicBool::new(false),&mut scratch).is_err());
    assert!(millimetres(f32::MAX,None).is_err());
    assert!(millimetres(-1.0,Some(-1000)).is_err());
    assert_eq!(millimetres(0.0005,None).unwrap(),1);
    assert_eq!(millimetres(-0.0005,None).unwrap(),-1);
    let mut r = request(); r.samples[0].strength = 0.0;
    assert!(brush_payload(&fixture(100_001),&r,ui_georaster_edit_limits(),&AtomicBool::new(false),&mut scratch).unwrap_err().contains("no changed"));
}

#[test]
fn bounded_request_refuses_unbounded_window_and_journal() {
    let mut r = request();
    r.window_side = MAX_BRUSH_WINDOW_SIDE + 1;
    assert!(r.validate_bounds().is_err());
    r = request();
    r.samples = vec![r.samples[0];MAX_BRUSH_REQUEST_SAMPLES + 1];
    assert!(r.validate_bounds().is_err());
}
