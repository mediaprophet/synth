use super::*;
use crate::graphical::{
    assembly::{AssemblyRenderSettings, CompiledAssembly, CompiledPart},
    preview::{PreviewCamera, PreviewLimits, SourceRef},
};
use qualia_core_db::{
    container_10d::{MaterialRecord, OpacityMode, SubmeshRange},
    poet_host::invoke::studio_store::{self, StudioArtifact},
    render::{assets::Mesh, temporal_producers::Mat4ColumnMajor},
};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

static NEXT: AtomicU64 = AtomicU64::new(1);
pub(crate) fn source() -> SourceRef {
    let id = format!(
        "urn:poet:r9:viewport-test:{}:{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    );
    let (_, stamp) = studio_store::create_artifact_if_absent(StudioArtifact::new(
        &id,
        "Geometry",
        "viewport calibration",
    ))
    .unwrap();
    SourceRef {
        artifact_id: id,
        stamp,
    }
}
pub(crate) fn camera() -> PreviewCamera {
    PreviewCamera {
        yaw: 0.0,
        pitch: 0.0,
        distance: 4.0,
        target: [0.0; 3],
    }
}
pub(crate) fn scene() -> CompiledAssembly {
    let mut material = MaterialRecord::legacy_default();
    material.base_color = [0.8, 0.15, 0.08, 1.0];
    // Front and back overlap in the image; the visible nearer part is not first.
    CompiledAssembly {
        mesh: Mesh {
            positions: vec![
                [-1.0, -1.0, 0.0],
                [1.0, -1.0, 0.0],
                [0.0, 1.0, 0.0],
                [-1.0, -1.0, 1.0],
                [1.0, -1.0, 1.0],
                [0.0, 1.0, 1.0],
            ],
            triangles: vec![[0, 1, 2], [3, 4, 5]],
            min: [-1.0, -1.0, 0.0],
            max: [1.0, 1.0, 1.0],
        },
        materials: vec![material],
        ranges: (0..2)
            .map(|i| SubmeshRange {
                first_index: i * 3,
                index_count: 3,
                material_id: material.id,
                semantic_id: 101 + i as u64,
            })
            .collect(),
        parts: (0..2)
            .map(|i| CompiledPart {
                key: if i == 0 { "rear" } else { "front" }.into(),
                parent: None,
                semantic_id: 101 + i as u64,
                first_index: i * 3,
                index_count: 3,
            })
            .collect(),
        normals: vec![[0.0, 0.0, 1.0]; 6],
        tangents: None,
        uv0: vec![[0.0, 0.0]; 6],
    }
}

#[test]
fn viewport_gestures_use_canonical_clamps_and_projection_axes() {
    let moved = apply_gesture(
        camera(),
        ViewportGesture::Orbit {
            dx: 20.0,
            dy: 8192.0,
        },
        512,
        512,
    )
    .unwrap();
    assert_eq!(moved.pitch, 1.45);
    let zoom = apply_gesture(camera(), ViewportGesture::Zoom { steps: 100.0 }, 512, 512).unwrap();
    assert_eq!(
        zoom.distance,
        qualia_core_db::render::camera::CAMERA_MIN_ZOOM
    );
    let pan = apply_gesture(
        camera(),
        ViewportGesture::Pan { dx: 64.0, dy: 0.0 },
        512,
        512,
    )
    .unwrap();
    assert!(pan.target[0] < 0.0);
    assert_eq!(pan.target[1], 0.0);
    assert_eq!(pan.target[2], 0.0);
    assert!(apply_gesture(
        camera(),
        ViewportGesture::Zoom { steps: f32::NAN },
        512,
        512
    )
    .is_err());
}

#[test]
fn viewport_framing_accounts_for_portrait_aspect_and_off_origin_parts() {
    let bounds = ([9.0, -1.0, -1.0], [11.0, 1.0, 1.0]);
    let wide = frame_bounds(camera(), bounds.0, bounds.1, 1024, 512).unwrap();
    let portrait = frame_bounds(camera(), bounds.0, bounds.1, 256, 512).unwrap();
    assert_eq!(wide.target, [10.0, 0.0, 0.0]);
    assert!(portrait.distance > wide.distance);
    for (c, aspect) in [(wide, 2.0), (portrait, 0.5)] {
        let matrix = Mat4ColumnMajor::from_cols(camera_state(c).unwrap().view_projection(aspect));
        for x in [9.0, 11.0] {
            for y in [-1.0, 1.0] {
                for z in [-1.0, 1.0] {
                    let p = picking::project(&matrix, [x, y, z]).unwrap();
                    assert!(p[0].abs() <= 1.0 && p[1].abs() <= 1.0);
                }
            }
        }
    }
    assert!(frame_bounds(camera(), [-f32::MAX; 3], [f32::MAX; 3], 512, 512).is_err());
}

#[test]
fn viewport_pick_maps_nearest_actual_triangle_to_exact_source_part() {
    let source = source();
    let scene = scene();
    let cancel = AtomicBool::new(false);
    let hit = picking::pick_part(&source, 7, &scene, camera(), 64, 64, 32.0, 32.0, 2, &cancel)
        .unwrap()
        .unwrap();
    assert_eq!(hit.part_key, "front");
    assert_eq!(hit.semantic_id, 102);
    assert_eq!(hit.triangle_index, 1);
    assert_eq!(hit.frame_id, 7);
    assert!(
        picking::pick_part(&source, 7, &scene, camera(), 64, 64, 0.0, 0.0, 2, &cancel)
            .unwrap()
            .is_none()
    );
    assert!(
        picking::pick_part(&source, 7, &scene, camera(), 64, 64, 64.0, 32.0, 2, &cancel).is_err()
    );
    assert!(
        picking::pick_part(&source, 7, &scene, camera(), 64, 64, 32.0, 32.0, 1, &cancel).is_err()
    );
}

#[test]
fn viewport_pick_uses_same_non_square_projection_and_pixel_centers() {
    let source = source();
    let scene = scene();
    let cancel = AtomicBool::new(false);
    for (width, height) in [(128, 64), (64, 128)] {
        let hit = picking::pick_part(
            &source,
            1,
            &scene,
            camera(),
            width,
            height,
            width as f32 / 2.0,
            height as f32 / 2.0,
            2,
            &cancel,
        )
        .unwrap()
        .unwrap();
        assert_eq!(hit.part_key, "front");
    }
}

#[test]
fn viewport_alpha_wind_cancel_and_late_source_changes_fail_closed() {
    let source = source();
    let mut scene = scene();
    let cancel = AtomicBool::new(false);
    scene.materials[0].opacity_mode = OpacityMode::Mask;
    assert!(
        picking::pick_part(&source, 1, &scene, camera(), 64, 64, 32.0, 32.0, 2, &cancel).is_err()
    );
    scene.materials[0].opacity_mode = OpacityMode::Opaque;
    scene.materials[0].wind_animated = true;
    assert!(
        picking::pick_part(&source, 1, &scene, camera(), 64, 64, 32.0, 32.0, 2, &cancel).is_err()
    );
    scene.materials[0].wind_animated = false;
    cancel.store(true, Ordering::Release);
    assert!(
        picking::pick_part(&source, 1, &scene, camera(), 64, 64, 32.0, 32.0, 2, &cancel).is_err()
    );
    cancel.store(false, Ordering::Release);
    studio_store::mutate_artifact_stamped(&source.artifact_id, &source.stamp, |artifact| {
        artifact.title = "changed".into();
        Ok(())
    })
    .unwrap();
    assert!(
        picking::pick_part(&source, 1, &scene, camera(), 64, 64, 32.0, 32.0, 2, &cancel).is_err()
    );
}

#[test]
fn viewport_empty_and_byte_budgets_reject_before_device_creation() {
    let limits = PreviewLimits {
        side: 64,
        vertices: 16,
        indices: 16,
    };
    let mut view = RetainedViewport::new(
        limits,
        ViewportBudget {
            working_bytes: 1,
            ..ViewportBudget::default()
        },
    )
    .unwrap();
    assert!(view
        .replace_scene(
            source(),
            scene(),
            AssemblyRenderSettings::for_camera(camera()),
            None,
            None,
            &AtomicBool::new(false)
        )
        .err()
        .unwrap()
        .contains("byte budget"));
    assert_eq!(view.counters().device_creations, 0);
    view.invalidate();
    assert_eq!(view.frame_id(), None);
}

/// Explicit native headless GPU qualification; not native window/input/human acceptance.
/// Run separately with --ignored --exact graphical::viewport::tests::r9_native_gpu_retains_device_scene_and_picks_visible_part
#[test]
#[ignore = "requires explicit native GPU qualification"]
fn r9_native_gpu_retains_device_scene_and_picks_visible_part() {
    let source = source();
    let cancel = AtomicBool::new(false);
    let mut view = RetainedViewport::new(
        PreviewLimits {
            side: 64,
            vertices: 16,
            indices: 16,
        },
        ViewportBudget::default(),
    )
    .unwrap();
    let mut settings = AssemblyRenderSettings::for_camera(camera());
    settings.shadows = false;
    settings.screen_space_ao = false;
    let first = view
        .replace_scene(source.clone(), scene(), settings, None, None, &cancel)
        .unwrap();
    let id = view.frame_id().unwrap();
    let selected = view
        .pick(source.clone(), id, 32.0, 32.0, &cancel)
        .unwrap()
        .unwrap();
    assert_eq!(
        (
            selected.part_key.as_str(),
            selected.semantic_id,
            selected.triangle_index
        ),
        ("front", 102, 1)
    );
    // Actual GPU pixels at projected interior differ from clear background, not just a histogram.
    let center = (32 * 64 + 32) * 4;
    assert_ne!(&first.rgba8[center..center + 3], &first.rgba8[0..3]);
    let highlighted = view.render(source.clone(), camera(), &cancel).unwrap();
    assert!(highlighted
        .rgba8
        .chunks_exact(4)
        .any(|p| p == [255, 194, 45, 255]));
    assert!(view.pick(source.clone(), id, 32.0, 32.0, &cancel).is_err());
    view.clear_selection();
    let moved = apply_gesture(
        camera(),
        ViewportGesture::Orbit { dx: 5.0, dy: 2.0 },
        64,
        64,
    )
    .unwrap();
    let second = view.render(source.clone(), moved, &cancel).unwrap();
    assert_ne!(first.rgba8, second.rgba8);
    let matrix = Mat4ColumnMajor::from_cols(camera_state(moved).unwrap().view_projection(1.0));
    let projected = picking::project(&matrix, [0.0, -1.0 / 3.0, 1.0]).unwrap();
    let x = (projected[0] * 0.5 + 0.5) * 64.0;
    let y = (0.5 - projected[1] * 0.5) * 64.0;
    let selected = view
        .pick(source.clone(), view.frame_id().unwrap(), x, y, &cancel)
        .unwrap()
        .unwrap();
    assert_eq!(selected.part_key, "front");
    assert_eq!(
        view.counters(),
        ViewportCounters {
            device_creations: 1,
            scene_uploads: 1,
            frames: 3
        }
    );
    let old_id = view.frame_id().unwrap();
    view.invalidate();
    assert!(view.pick(source.clone(), old_id, x, y, &cancel).is_err());
    let _ = view
        .replace_scene(source.clone(), scene(), settings, None, None, &cancel)
        .unwrap();
    assert!(view.frame_id().unwrap() > old_id);
    assert_eq!(view.counters().device_creations, 1);
    studio_store::mutate_artifact_stamped(&source.artifact_id, &source.stamp, |a| {
        a.title = "retired GPU source".into();
        Ok(())
    })
    .unwrap();
    assert!(view.render(source, camera(), &cancel).is_err());
}


#[test]
fn selected_rig_span_requires_exact_unshared_compiled_geometry() {
    use crate::graphical::rig_binding::{BindMesh, ViewportSpan};
    let source = source();
    let mut compiled = scene();
    let selected = PartSelection { source, frame_id: 1, part_key: "front".into(), semantic_id: 102, triangle_index: 1 };
    let positions = compiled.mesh.positions[3..6].to_vec();
    let normals = compiled.normals[3..6].to_vec();
    let uv = compiled.uv0[3..6].to_vec();
    let indices = [0, 1, 2];
    let bind = || BindMesh { positions: &positions, normals: &normals, uv0: &uv, indices: &indices };
    assert_eq!(renderer::selected_rig_span(&compiled, &selected, &bind()).unwrap(),
        ViewportSpan { first_vertex: 3, vertex_count: 3 });
    compiled.mesh.positions[3][0] += 0.25;
    assert!(renderer::selected_rig_span(&compiled, &selected, &bind()).is_err());
    compiled.mesh.positions[3][0] = positions[0][0];
    compiled.mesh.triangles[0][0] = 3;
    assert!(renderer::selected_rig_span(&compiled, &selected, &bind()).is_err());
    compiled.mesh.triangles[0][0] = 0;
    compiled.mesh.triangles[1].swap(0, 1);
    assert!(renderer::selected_rig_span(&compiled, &selected, &bind()).is_err());
    compiled.mesh.triangles[1].swap(0, 1);
    compiled.normals[3][1] = 1.0;
    assert!(renderer::selected_rig_span(&compiled, &selected, &bind()).is_err());
}

// Real retained-viewport adoption logic with seeded CPU state and no GPU.
// Does not establish native rendering, signed admission or human usability.
#[test]
fn selected_rig_adoption_derives_binding_and_refuses_mismatches() {
    use crate::graphical::rig_binding::{RigControl, RigSourceDescriptor};
    use qualia_core_db::{
        container_10d::{self, Container10dHeader, SectionType,
            skeletal_container::attach_skeletal_section_into,
            skeletal_section::{encode_skeletal_section_into, ClipRecord, SkeletalData}},
        render::{compile_10d::compile_mesh_to_10d_with_nodes, skeletal::{Joint, Key, Motor, SkinWeight, Track}},
    };
    let source = source();
    let mut compiled = scene();
    let bind_mesh = Mesh {
        positions: compiled.mesh.positions[3..6].to_vec(), triangles: vec![[0,1,2]],
        min: [-1.0,-1.0,1.0], max: [1.0,1.0,1.0],
    };
    let base = compile_mesh_to_10d_with_nodes(&bind_mesh, &[]).unwrap();
    let header = Container10dHeader::parse(&base).unwrap();
    let sections = container_10d::parse_section_table(&base, &header).unwrap();
    let mesh_section = sections.iter().find(|section| section.typ() == Some(SectionType::QuantizedMesh)).unwrap();
    let start = mesh_section.byte_offset as usize;
    let end = start + mesh_section.byte_length as usize;
    let decoded = container_10d::decode_mesh_section(&base[start..end]).unwrap();
    compiled.mesh.positions[3..6].copy_from_slice(&decoded.positions);
    let joint_id = qualia_core_db::q_hash("urn:test:adoption:root");
    let first_clip = qualia_core_db::q_hash("urn:test:adoption:first");
    let second_clip = qualia_core_db::q_hash("urn:test:adoption:second");
    let joints = [Joint { parent:None, rest_local:Motor::identity(), inverse_bind:Motor::identity() }];
    let joint_ids = [joint_id];
    let skins = [SkinWeight { joints:[0;4], weights:[1.0,0.0,0.0,0.0] };3];
    let clips = [ClipRecord { id:first_clip, duration:1.0, first_track:0, track_count:1, looping:false },
                 ClipRecord { id:second_clip, duration:1.0, first_track:1, track_count:1, looping:false }];
    let tracks = [Track { joint:0, first_key:0, key_count:2 }, Track { joint:0, first_key:2, key_count:2 }];
    let keys = [Key { time:0.0, local:Motor::identity() }, Key { time:1.0, local:Motor::identity() },
                Key { time:0.0, local:Motor::identity() }, Key { time:1.0, local:Motor::identity() }];
    let data = SkeletalData { joints:&joints, joint_ids:&joint_ids, skin:&skins, clips:&clips, tracks:&tracks, keys:&keys };
    let mut payload = vec![0;2048];
    let n = encode_skeletal_section_into(&data, &mut payload).unwrap();
    payload.truncate(n);
    let mut bytes = vec![0;8192];
    let n = attach_skeletal_section_into(&base, &payload, &mut bytes).unwrap();
    bytes.truncate(n);
    let selected = PartSelection { source:source.clone(), frame_id:1, part_key:"front".into(), semantic_id:102, triangle_index:1 };
    let viewport = renderer::RetainedViewport::rig_test_seed(source.clone(), compiled.clone(), selected.clone());
    let seed = RigSourceDescriptor {
        schema:"poet.bound-rig/v1".into(), source_artifact_id:source.artifact_id.clone(), part_id:"front".into(), bind_id:"local-bind".into(), resource_key:"synthetic.10d".into(),
        resource_sha256:String::new(), bind_mesh_sha256:String::new(), normal_uv_sha256:String::new(),
        joint_ids:vec![], clip_ids:vec![], selected_clip_id:0, seek_seconds:0.0, joint_deltas:vec![], vertex_lineage:vec![],
    };
    let action = RigControl::Adopt { part_id:"front".into(), bind_id:"local-bind".into() };
    let cancelled = AtomicBool::new(false);
    let candidate = viewport.stage_selected_rig(source.clone(), seed.clone(), &bytes, action.clone(), &cancelled).unwrap();
    let actual = candidate.after();
    assert_eq!(actual.source_artifact_id, source.artifact_id);
    assert_eq!(actual.part_id, "front");
    assert_eq!(actual.bind_id, "local-bind");
    assert_eq!(actual.resource_key, "synthetic.10d");
    assert_eq!(actual.resource_sha256, crate::graphical::rig_binding::resource_sha256(&bytes));
    assert_eq!(actual.bind_mesh_sha256, crate::graphical::rig_binding::bind_mesh_sha256(&decoded.positions, &[0,1,2]));
    assert_eq!(actual.normal_uv_sha256, crate::graphical::rig_binding::normal_uv_sha256(&compiled.normals[3..6], &compiled.uv0[3..6]));
    assert_eq!(actual.joint_ids, vec![joint_id]);
    assert_eq!(actual.clip_ids, vec![first_clip, second_clip]);
    assert_eq!(actual.selected_clip_id, first_clip);
    assert_eq!(actual.seek_seconds, 0.0);
    assert_eq!(actual.vertex_lineage, vec![0,1,2]);
    assert!(viewport.stage_selected_rig(source.clone(), seed.clone(), &bytes,
        RigControl::Adopt { part_id:"rear".into(), bind_id:"local-bind".into() }, &cancelled).is_err());
    assert!(viewport.stage_selected_rig(source.clone(), seed.clone(), &bytes,
        RigControl::Adopt { part_id:"front".into(), bind_id:"".into() }, &cancelled).is_err());
    let mut corrupted = bytes.clone();
    let last = corrupted.len()-1; corrupted[last] ^= 1;
    assert!(viewport.stage_selected_rig(source.clone(), seed.clone(), &corrupted, action.clone(), &cancelled).is_err());
    let mut mismatched = compiled.clone(); mismatched.mesh.positions[3][0] += 0.25;
    let wrong_viewport = renderer::RetainedViewport::rig_test_seed(source.clone(), mismatched, selected);
    assert!(wrong_viewport.stage_selected_rig(source.clone(), seed.clone(), &bytes, action.clone(), &cancelled).is_err());
    cancelled.store(true, Ordering::Release);
    assert!(viewport.stage_selected_rig(source, seed, &bytes, action, &cancelled).is_err());
}

#[test]
fn rig_pose_updates_cpu_picking_geometry_and_bounds() {
    let mut assembly = scene();
    assembly.tangents = Some(vec![[1.0,0.0,0.0,1.0]; 6]);
    let span = crate::graphical::rig_binding::ViewportSpan { first_vertex: 3, vertex_count: 3 };
    let positions = [[2.0, 0.0, 2.0], [3.0, 0.0, 2.0], [2.0, 2.0, 2.0]];
    let normals = [[0.0, 1.0, 0.0]; 3];
    let tangents = [[1.0, 0.0, 0.0, 1.0]; 3];
    super::renderer::validate_rig_pose(&assembly, span, &positions, &normals, &tangents).unwrap();
    super::renderer::apply_rig_pose(&mut assembly, span, &positions, &normals, &tangents).unwrap();
    assert_eq!(&assembly.mesh.positions[3..], &positions);
    assert_eq!(&assembly.normals[3..], &normals);
    assert_eq!(&assembly.tangents.as_ref().unwrap()[3..], &tangents);
    assert_eq!(assembly.mesh.min, [-1.0, -1.0, 0.0]);
    assert_eq!(assembly.mesh.max, [3.0, 2.0, 2.0]);
}

#[test]
fn invalid_rig_pose_is_rejected_before_cpu_mutation() {
    let assembly = scene();
    let span = crate::graphical::rig_binding::ViewportSpan { first_vertex: 3, vertex_count: 3 };
    let mut positions = [[0.0, 0.0, 0.0]; 3];
    positions[1][0] = f32::NAN;
    assert!(super::renderer::validate_rig_pose(&assembly, span, &positions, &[[0.0, 0.0, 1.0]; 3], &[[1.0, 0.0, 0.0, 1.0]; 3]).is_err());
    assert!(super::renderer::validate_rig_pose(&assembly, span, &positions[..2], &[[0.0, 0.0, 1.0]; 3], &[[1.0, 0.0, 0.0, 1.0]; 3]).is_err());
    assert_eq!(assembly.mesh.max, [1.0, 1.0, 1.0]);
}

#[test]
fn rig_tangent_rejection_preserves_the_entire_cpu_pose() {
    let mut assembly=scene();
    assembly.tangents=Some(vec![[1.0,0.0,0.0,1.0];6]);
    let before=assembly.clone();
    let span=crate::graphical::rig_binding::ViewportSpan {first_vertex:3,vertex_count:3};
    let positions=[[1.0,0.0,0.0];3]; let normals=[[0.0,0.0,1.0];3];
    let invalid=[[0.0,0.0,1.0,1.0];3];
    assert!(super::renderer::apply_rig_pose(&mut assembly,span,&positions,&normals,&invalid).is_err());
    assert_eq!(assembly.mesh.positions,before.mesh.positions);
    assert_eq!(assembly.normals,before.normals); assert_eq!(assembly.tangents,before.tangents);
    assembly.tangents=None;
    assert!(super::renderer::apply_rig_pose(&mut assembly,span,&positions,&normals,&[[1.0,0.0,0.0,1.0];3]).is_err());
    assert_eq!(assembly.mesh.positions,before.mesh.positions);
}
