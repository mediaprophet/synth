use super::*;
use crate::graphical::preview::PreviewLimits;
use qualia_core_db::container_10d::{self, MaterialRecord, SubmeshRange};
use std::collections::BTreeMap;
use vibe::Value;

fn rec(values: impl IntoIterator<Item = (&'static str, Value)>) -> Value {
    Value::Record(values.into_iter().map(|(k, v)| (k.into(), v)).collect())
}
fn list(values: &[f64]) -> Value {
    Value::List(values.iter().map(|v| Value::F64(*v)).collect())
}
fn material(id: u64) -> Value {
    rec([
        ("id", Value::U64(id)),
        ("colour_space", Value::String("linear".into())),
        ("base_color", list(&[0.3, 0.4, 0.5, 1.0])),
        ("emissive", list(&[0.0; 3])),
        ("metallic", Value::F64(0.8)),
        ("roughness", Value::F64(0.3)),
        ("specular", Value::F64(1.0)),
        ("alpha_cutoff", Value::F64(0.5)),
        ("opacity_mode", Value::String("Opaque".into())),
        ("double_sided", Value::Bool(false)),
    ])
}
fn part(id: &str, semantic: u64, parent: Option<&str>, translation: [f64; 3]) -> Value {
    let geometry = rec([
        (
            "vertices",
            Value::List(vec![
                list(&[0.0, 0.0, 0.0]),
                list(&[1.0, 0.0, 0.0]),
                list(&[0.0, 1.0, 0.0]),
            ]),
        ),
        (
            "triangles",
            Value::List(vec![Value::U64(0), Value::U64(1), Value::U64(2)]),
        ),
        (
            "uv0",
            Value::List(vec![
                list(&[0.0, 0.0]),
                list(&[1.0, 0.0]),
                list(&[0.0, 1.0]),
            ]),
        ),
    ]);
    let mut result = rec([
        ("id", Value::String(id.into())),
        ("semantic_id", Value::U64(semantic)),
        ("geometry", geometry),
        ("material", material(9)),
        ("translation", list(&translation)),
        ("rotation", list(&[0.0; 3])),
        ("scale", list(&[1.0; 3])),
    ]);
    if let Some(parent) = parent {
        record_mut(&mut result).insert("parent".into(), Value::String(parent.into()));
    }
    result
}
fn record_mut(value: &mut Value) -> &mut BTreeMap<String, Value> {
    let Value::Record(record) = value else {
        panic!("record fixture")
    };
    record
}
fn fixture() -> Value {
    rec([(
        "parts",
        Value::List(vec![
            part("body", 101, None, [2.0, 0.0, 0.0]),
            part("lever", 102, Some("body"), [0.0, 3.0, 0.0]),
        ]),
    )])
}
fn parts_mut(value: &mut Value) -> &mut Vec<Value> {
    let Value::List(parts) = record_mut(value).get_mut("parts").unwrap() else {
        panic!("parts fixture")
    };
    parts
}

#[test]
fn actual_native_hierarchy_retains_part_ranges_materials_normals_and_uv() {
    let value = fixture();
    let a = admit_assembly(&value, PreviewLimits::default()).unwrap();
    assert_eq!(a.mesh.positions.len(), 6);
    assert_eq!(a.mesh.triangles.len(), 2);
    assert_eq!(a.mesh.min, [2.0, 0.0, 0.0]);
    assert_eq!(a.mesh.max, [3.0, 4.0, 0.0]);
    assert_eq!(
        a.parts.iter().map(|p| p.key.as_str()).collect::<Vec<_>>(),
        vec!["body", "lever"]
    );
    assert_eq!(a.parts[1].parent.as_deref(), Some("body"));
    assert_eq!(
        a.ranges.iter().map(|r| r.semantic_id).collect::<Vec<_>>(),
        vec![101, 102]
    );
    assert_eq!(a.materials.len(), 1);
    assert_eq!(a.ranges[1].first_index, 3);
    for n in &a.normals {
        assert!((n[2] - 1.0).abs() < 1e-6);
    }
    assert_eq!(a.uv0[1], [1.0, 0.0]);
    assert_eq!(
        compile_assembly(&a).unwrap(),
        compile_assembly(&admit_assembly(&value, PreviewLimits::default()).unwrap()).unwrap()
    );
}

#[test]
fn malformed_ambiguous_or_unbounded_projection_is_refused() {
    for case in 0..9 {
        let mut value = fixture();
        let parts = parts_mut(&mut value);
        match case {
            0 => {
                record_mut(&mut parts[1]).insert("id".into(), Value::String("body".into()));
            }
            1 => {
                record_mut(&mut parts[1]).insert("semantic_id".into(), Value::U64(101));
            }
            2 => {
                record_mut(&mut parts[0]).insert("parent".into(), Value::String("lever".into()));
            }
            3 => {
                record_mut(&mut parts[1]).insert("parent".into(), Value::String("missing".into()));
            }
            4 => {
                record_mut(&mut parts[1]).insert("scale".into(), list(&[-1.0, 1.0, 1.0]));
            }
            5 => {
                record_mut(&mut parts[1]).insert("rotation".into(), list(&[f64::NAN, 0.0, 0.0]));
            }
            6 => {
                record_mut(&mut parts[1])
                    .insert("permission".into(), Value::String("steward".into()));
            }
            7 => {
                let mat = record_mut(&mut parts[1]).get_mut("material").unwrap();
                record_mut(mat).insert("roughness".into(), Value::F64(0.9));
            }
            _ => {
                let geometry = record_mut(&mut parts[1]).get_mut("geometry").unwrap();
                record_mut(geometry).insert(
                    "normals".into(),
                    Value::List(vec![list(&[0.0, 0.0, 0.0]); 3]),
                );
            }
        }
        assert!(
            admit_assembly(&value, PreviewLimits::default()).is_err(),
            "case {case}"
        );
    }
    assert!(admit_assembly(
        &fixture(),
        PreviewLimits {
            side: 64,
            vertices: 5,
            indices: 6
        }
    )
    .is_err());
    let mut wrong = fixture();
    record_mut(&mut wrong).insert("camera".into(), Value::Null);
    assert!(admit_assembly(&wrong, PreviewLimits::default()).is_err());
}

#[test]
fn native_container_roundtrip_preserves_semantic_ranges_normal_and_uv_streams() {
    let a = admit_assembly(&fixture(), PreviewLimits::default()).unwrap();
    let bytes = compile_assembly(&a).unwrap();
    let mesh = qualia_core_db::render::compile_10d::decode_10d_mesh(&bytes).unwrap();
    assert_eq!(mesh.triangles, a.mesh.triangles);
    for (before, after) in a.mesh.positions.iter().zip(&mesh.positions) {
        for axis in 0..3 {
            assert!((before[axis] - after[axis]).abs() <= 4.0 / 65535.0 + 1e-6);
        }
    }
    let mut materials = vec![MaterialRecord::legacy_default(); a.materials.len()];
    let mut ranges = vec![
        SubmeshRange {
            first_index: 0,
            index_count: 0,
            material_id: 0,
            semantic_id: 0
        };
        a.ranges.len()
    ];
    assert_eq!(
        container_10d::read_material_section_into(
            &bytes,
            (a.mesh.triangles.len() * 3) as u32,
            &mut materials,
            &mut ranges
        )
        .unwrap(),
        (1, 2)
    );
    assert_eq!(materials, a.materials);
    assert_eq!(ranges, a.ranges);
    let frames =
        qualia_core_db::render::compile_10d::decode_10d_mesh_frames(&bytes, a.mesh.positions.len())
            .unwrap()
            .unwrap();
    assert!(frames.normals.is_some());
    let mut uv = vec![[0.0; 2]; a.uv0.len()];
    container_10d::read_texture_coordinate_section_into(&bytes, a.uv0.len(), &mut uv).unwrap();
    assert_eq!(uv, a.uv0);
    let mut invalid = a;
    invalid.ranges[0].semantic_id = 999;
    assert!(compile_assembly(&invalid).is_err());
}

fn smooth_atlas() -> Value {
    rec([("version",Value::I64(2)),("seams",Value::List(vec![])),
        ("atlas_size",Value::I64(1024)),("gutter_pixels",Value::I64(4)),
        ("texels_per_metre",Value::F64(64.0)),("iterations",Value::I64(64))])
}
#[test]
fn atlas_frames_reach_native_container_with_complete_mixed_part_correspondence() {
    let mut value = fixture();
    record_mut(&mut parts_mut(&mut value)[0]).insert("uv_atlas".into(),smooth_atlas());
    let before=value.clone();
    let assembly=admit_assembly(&value,PreviewLimits::default()).unwrap();
    let tangents=assembly.tangents.as_ref().unwrap();
    assert_eq!(tangents.len(),assembly.mesh.positions.len());
    poet_spatial_authoring::assembly::validate_tangent_frames(&assembly.normals,tangents).unwrap();
    let bytes=compile_assembly(&assembly).unwrap();
    let decoded=qualia_core_db::render::compile_10d::decode_10d_mesh_frames(&bytes,assembly.mesh.positions.len()).unwrap().unwrap();
    assert_eq!(decoded.tangents.as_ref(),Some(tangents));
    assert_eq!(value,before);
}
#[test]
fn mixed_atlas_and_uvless_part_refuses_without_touching_source() {
    let mut value=fixture();
    let parts=parts_mut(&mut value);
    record_mut(&mut parts[0]).insert("uv_atlas".into(),smooth_atlas());
    record_mut(record_mut(&mut parts[1]).get_mut("geometry").unwrap()).remove("uv0");
    let before=value.clone();
    assert!(admit_assembly(&value,PreviewLimits::default()).unwrap_err().contains("real UV0"));
    assert_eq!(value,before);
}
#[test]
fn malformed_compiled_frame_never_reaches_native_container() {
    let mut a=admit_assembly(&fixture(),PreviewLimits::default()).unwrap();
    a.tangents=Some(vec![[1.0,0.0,0.0,0.0];a.mesh.positions.len()]);
    assert!(compile_assembly(&a).is_err());
    a.tangents=Some(vec![[1.0,0.0,0.0,1.0];1]);
    assert!(compile_assembly(&a).is_err());
}
