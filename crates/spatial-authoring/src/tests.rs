use super::*;
use qualia_core_db::{container_10d::MaterialRecord, render::assets::Mesh};
fn triangle() -> Mesh {
    Mesh {
        positions: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
        triangles: vec![[0, 1, 2]],
        min: [0.0; 3],
        max: [1.0, 1.0, 0.0],
    }
}
fn part(key: &str, id: u64) -> AssemblyPart {
    let mut material = MaterialRecord::legacy_default();
    material.id = 7;
    AssemblyPart {
        key: key.into(),
        parent: None,
        semantic_id: id,
        mesh: triangle(),
        material,
        trs: Transform::default(),
        normals: None,
        tangents: None,
        uv0: None,
    }
}
fn volume(mesh: &Mesh) -> f64 {
    use poet_geometry::Vec3;
    let mut sum = 0.0;
    for t in &mesh.triangles {
        let p = t.map(|i| {
            let p = mesh.positions[i as usize];
            Vec3::new(p[0] as f64, p[1] as f64, p[2] as f64)
        });
        sum += p[0].dot(p[1].cross(p[2])) / 6.0;
    }
    sum
}
#[test]
fn concave_channel_caps_preserve_wall_void_and_volume() {
    let profile = Profile::OpenChannel {
        width: 0.4,
        height: 0.2,
        wall: 0.02,
        corner_radius: 0.0,
        segments: 4,
    };
    let mesh = profile_sweep(
        &profile,
        &[[0.0; 3], [0.0, 0.0, 2.0]],
        true,
        GeometryLimits::default(),
    )
    .unwrap();
    let section = 0.4 * 0.2 - (0.4 - 0.04) * (0.2 - 0.02);
    assert!((volume(&mesh) - section * 2.0).abs() < 1e-7);
    assert!((mesh.min[0] + 0.2).abs() < 1e-6 && (mesh.max[1] - 0.1).abs() < 1e-6);
    let mut edges = std::collections::BTreeMap::new();
    for t in &mesh.triangles {
        for (a, b) in [(t[0], t[1]), (t[1], t[2]), (t[2], t[0])] {
            *edges.entry((a.min(b), a.max(b))).or_insert(0) += 1;
        }
    }
    assert!(edges.values().all(|n| *n == 2));
}
#[test]
fn changed_profile_parameters_change_real_geometry() {
    let p = Profile::RoundedRectangle {
        width: 1.0,
        height: 0.3,
        radius: 0.05,
        segments: 8,
    };
    let changed = Profile::RoundedRectangle {
        width: 1.2,
        height: 0.3,
        radius: 0.05,
        segments: 8,
    };
    let original = profile_sweep(
        &p,
        &[[0.0; 3], [0.0, 0.0, 1.0]],
        true,
        GeometryLimits::default(),
    )
    .unwrap();
    let edited = profile_sweep(
        &changed,
        &[[0.0; 3], [0.0, 0.0, 1.0]],
        true,
        GeometryLimits::default(),
    )
    .unwrap();
    assert!(edited.max[0] > original.max[0]);
    assert!(original.positions.len() > 8);
}
#[test]
fn parent_relative_parts_retain_semantics_material_ranges_and_flat_frames() {
    let mut parent = part("frame", 11);
    parent.trs.translation = [1.0, 0.0, 0.0];
    let mut child = part("channel", 22);
    child.parent = Some("frame".into());
    child.trs.translation = [0.0, 2.0, 0.0];
    let compiled = compile_assembly(&[child, parent], AssemblyLimits::default()).unwrap();
    assert_eq!(compiled.mesh.positions[0], [1.0, 2.0, 0.0]);
    assert_eq!(compiled.parts[0].semantic_id, 22);
    assert_eq!(compiled.ranges[1].semantic_id, 11);
    assert_eq!(compiled.normals.len(), compiled.mesh.positions.len());
    assert_eq!(compiled.uv0.len(), compiled.mesh.positions.len());
    assert_eq!(compiled.materials.len(), 1);
    assert!(compiled.normals.iter().all(|n| *n == [0.0, 0.0, 1.0]));
}
#[test]
fn authored_normals_use_full_world_inverse_transpose_and_uvs_survive() {
    let mut parent = part("frame", 11);
    parent.trs.scale = [2.0, 1.0, 0.5];
    let mut child = part("channel", 22);
    child.parent = Some("frame".into());
    child.trs.rotation = [0.4, 0.7, 0.3];
    child.normals = Some(vec![[0.0, 0.0, 1.0]; 3]);
    child.uv0 = Some(vec![[0.2, 0.3], [0.7, 0.3], [0.2, 0.9]]);
    let compiled = compile_assembly(&[child, parent], AssemblyLimits::default()).unwrap();
    let p = &compiled.mesh.positions;
    let n = compiled.normals[0];
    for i in [1, 2] {
        let dot: f64 = (0..3)
            .map(|a| (p[i][a] - p[0][a]) as f64 * n[a] as f64)
            .sum();
        assert!(dot.abs() < 1e-6);
    }
    assert_eq!(compiled.uv0[1], [0.7, 0.3]);
}

fn framed_part(key: &str, id: u64) -> AssemblyPart {
    let mut p = part(key,id);
    p.normals = Some(vec![[0.0,0.0,1.0];3]);
    p.uv0 = Some(vec![[0.0,0.0],[1.0,0.0],[0.0,1.0]]);
    p.tangents = Some(vec![[1.0,0.0,0.0,1.0];3]); p
}
#[test]
fn retained_tangents_use_linear_world_transform_and_keep_handedness() {
    let mut p = framed_part("surface",1);
    let diagonal = 0.5f32.sqrt();
    p.tangents = Some(vec![[diagonal,diagonal,0.0,-1.0];3]);
    p.trs.scale = [2.0,1.0,0.5];
    let result = compile_assembly(&[p],AssemblyLimits::default()).unwrap();
    for t in result.tangents.unwrap() {
        assert!((t[0]-2.0/5.0f32.sqrt()).abs()<1e-6);
        assert!((t[1]-1.0/5.0f32.sqrt()).abs()<1e-6);
        assert_eq!(t[2],0.0); assert_eq!(t[3],-1.0);
    }
}
#[test]
fn mixed_parts_derive_missing_frames_only_from_valid_existing_uvs() {
    let a = framed_part("framed",1); let mut b = part("legacy",2);
    b.uv0 = a.uv0.clone();
    let result = compile_assembly(&[a.clone(),b.clone()],AssemblyLimits::default()).unwrap();
    assert_eq!(result.tangents.unwrap(),vec![[1.0,0.0,0.0,1.0];6]);
    b.uv0 = None;
    assert!(compile_assembly(&[a.clone(),b.clone()],AssemblyLimits::default()).is_err());
    b.uv0 = Some(vec![[0.0,0.0];3]);
    assert!(compile_assembly(&[a,b],AssemblyLimits::default()).is_err());
}
#[test]
fn explicit_invalid_frames_and_derivation_budget_refuse() {
    for tangent in [[0.0,0.0,0.0,1.0],[0.0,0.0,1.0,1.0],[1.0,0.0,0.0,0.0],[f32::NAN,0.0,0.0,1.0]] {
        let mut p=framed_part("surface",1); p.tangents=Some(vec![tangent;3]);
        assert!(compile_assembly(&[p],AssemblyLimits::default()).is_err());
    }
    let p=framed_part("surface",1);
    assert!(assembly::derive_tangent_frames(&p.mesh,p.normals.as_ref().unwrap(),p.uv0.as_ref().unwrap(),1).is_err());
    let legacy = compile_assembly(&[part("legacy",1)],AssemblyLimits::default()).unwrap();
    assert!(legacy.tangents.is_none());
}
#[test]
fn invalid_assemblies_and_expanded_output_budget_fail_without_mutating_inputs() {
    let mut a = part("a", 1);
    let mut b = part("b", 2);
    a.parent = Some("b".into());
    b.parent = Some("a".into());
    assert!(compile_assembly(&[a.clone(), b.clone()], AssemblyLimits::default()).is_err());
    a.parent = None;
    b.parent = None;
    b.semantic_id = 1;
    assert!(compile_assembly(&[a.clone(), b], AssemblyLimits::default()).is_err());
    let before = a.mesh.clone();
    let mut tiny = AssemblyLimits::default();
    tiny.vertices = 2;
    assert!(compile_assembly(&[a.clone()], tiny).is_err());
    assert_eq!(a.mesh, before);
    a.trs.scale = [1.0, 0.0, 1.0];
    assert!(compile_assembly(&[a], AssemblyLimits::default()).is_err());
}
#[test]
fn malformed_profiles_and_primitive_budgets_are_rejected() {
    let invalid = Profile::OpenChannel {
        width: 1.0,
        height: 0.2,
        wall: 0.3,
        corner_radius: 0.01,
        segments: 4,
    };
    assert!(invalid.points(ProfileLimits::default()).is_err());
    let p = Profile::Circle {
        radius: f64::NAN,
        segments: 16,
    };
    assert!(p.points(ProfileLimits::default()).is_err());
    let mut tiny = GeometryLimits::default();
    tiny.vertices = 2;
    assert!(sphere(1.0, 16, 32, tiny).is_err());
    assert!(torus(1.0, 0.2, 32, 16, tiny).is_err());
}
