//! Synthetic engineering fixture only. No species anatomy or human acceptance.
use super::*;
use crate::graphical::animated_rig_preview::AnimatedRigPreview;
use qualia_core_db::{
    container_10d::{
        skeletal_container::attach_skeletal_section_into,
        skeletal_section::{encode_skeletal_section_into, ClipRecord, SkeletalData},
    },
    poet_host::invoke::studio_store::{self, StudioArtifact},
    render::{
        assets::Mesh,
        compile_10d::compile_mesh_to_10d_with_nodes,
        skeletal::{Joint, Key, Motor, Track},
    },
};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT: AtomicU64 = AtomicU64::new(1);

struct Fixture {
    source: SourceRef,
    bytes: Vec<u8>,
    descriptor: RigSourceDescriptor,
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    uv: Vec<[f32; 2]>,
    indices: Vec<u32>,
}

impl Fixture {
    fn new() -> Self {
        let id = format!("rig-fixture-{}", NEXT.fetch_add(1, Ordering::Relaxed));
        let (_, stamp) = studio_store::create_artifact_if_absent(StudioArtifact::new(
            &id,
            "Geometry",
            "synthetic rig source",
        ))
        .unwrap();
        let source = SourceRef {
            artifact_id: id.clone(),
            stamp,
        };
        let positions = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let normals = vec![[0.0, 0.0, 1.0]; 3];
        let uv = vec![[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]];
        let indices = vec![0, 1, 2];
        let mesh = Mesh {
            positions: positions.clone(),
            triangles: vec![[0, 1, 2]],
            min: [0.0; 3],
            max: [1.0, 1.0, 0.0],
        };
        let base = compile_mesh_to_10d_with_nodes(&mesh, &[]).unwrap();
        let joint_id = qualia_core_db::q_hash("urn:test:rig:root");
        let clip_id = qualia_core_db::q_hash("urn:test:rig:translate");
        let joints = [Joint {
            parent: None,
            rest_local: Motor::identity(),
            inverse_bind: Motor::identity(),
        }];
        let skin = [SkinWeight {
            joints: [0; 4],
            weights: [1.0, 0.0, 0.0, 0.0],
        }; 3];
        let clips = [ClipRecord {
            id: clip_id,
            duration: 1.0,
            first_track: 0,
            track_count: 1,
            looping: false,
        }];
        let tracks = [Track {
            joint: 0,
            first_key: 0,
            key_count: 2,
        }];
        let keys = [
            Key {
                time: 0.0,
                local: Motor::identity(),
            },
            Key {
                time: 1.0,
                local: Motor::from_translation(1.0, 0.0, 0.0),
            },
        ];
        let data = SkeletalData {
            joints: &joints,
            joint_ids: &[joint_id],
            skin: &skin,
            clips: &clips,
            tracks: &tracks,
            keys: &keys,
        };
        let mut payload = vec![0; 432];
        let written = encode_skeletal_section_into(&data, &mut payload).unwrap();
        payload.truncate(written);
        let mut output = vec![0; 8192];
        let length = attach_skeletal_section_into(&base, &payload, &mut output).unwrap();
        output.truncate(length);
        let descriptor = RigSourceDescriptor {
            schema: "poet.bound-rig/v1".into(),
            source_artifact_id: id,
            part_id: "synthetic-part".into(),
            bind_id: "rest-v1".into(),
            resource_key: "resources/synthetic-rig.10d".into(),
            resource_sha256: resource_sha256(&output),
            bind_mesh_sha256: bind_mesh_sha256(&positions, &indices),
            normal_uv_sha256: normal_uv_sha256(&normals, &uv),
            joint_ids: vec![joint_id],
            clip_ids: vec![clip_id],
            selected_clip_id: clip_id,
            seek_seconds: 0.0,
            joint_deltas: Vec::new(),
            vertex_lineage: vec![0, 1, 2],
        };
        Self {
            source,
            bytes: output,
            descriptor,
            positions,
            normals,
            uv,
            indices,
        }
    }

    fn admit(&self, descriptor: RigSourceDescriptor, budget: usize) -> Result<BoundRig, String> {
        admit_bound_rig(
            self.source.clone(),
            descriptor,
            "resources/synthetic-rig.10d",
            &self.bytes,
            BindMesh {
                positions: &self.positions,
                normals: &self.normals,
                uv0: &self.uv,
                indices: &self.indices,
            },
            ViewportSpan {
                first_vertex: 4,
                vertex_count: 3,
            },
            budget,
        )
    }
}

#[test]
fn sealed_source_admits_real_player_and_replays_from_bind_mesh() {
    let f = Fixture::new();
    let mut preview =
        AnimatedRigPreview::new(f.admit(f.descriptor.clone(), 42 * 1024 * 1024).unwrap());
    let frame = preview.advance(0.0, false).unwrap();
    assert_eq!(frame.receipt.clip_id, f.descriptor.selected_clip_id);
    assert_eq!(frame.span.first_vertex, 4);
    assert_eq!(frame.positions, f.positions);
    assert_eq!(frame.normals.len(), 3);
    assert_eq!(frame.tangents.len(), 3);
    apply_control(preview.rig_mut(), RigControl::Seek { seconds: 1.0 }).unwrap();
    let moved = preview.advance(0.0, false).unwrap().positions.to_vec();
    assert!((moved[0][0] - 1.0).abs() < 1e-5);
    let clip = f.descriptor.selected_clip_id;
    apply_control(
        preview.rig_mut(),
        RigControl::SelectClip {
            clip_id: clip,
            fade_seconds: 0.0,
        },
    )
    .unwrap();
    let shift = Motor::from_translation(0.0, 0.0, 0.25);
    apply_control(
        preview.rig_mut(),
        RigControl::SetJointDelta {
            joint_id: f.descriptor.joint_ids[0],
            motor: [
                shift.r_w, shift.r_x, shift.r_y, shift.r_z, shift.d_w, shift.d_x, shift.d_y,
                shift.d_z,
            ],
        },
    )
    .unwrap();
    assert!((preview.advance(0.0, false).unwrap().positions[0][2] - 0.25).abs() < 1e-5);
    apply_control(preview.rig_mut(), RigControl::Seek { seconds: 0.0 }).unwrap();
    assert!((preview.advance(0.0, false).unwrap().positions[0][0]).abs() < 1e-5);
    apply_control(preview.rig_mut(), RigControl::Seek { seconds: 1.0 }).unwrap();
    let controlled = preview.advance(0.0, false).unwrap().positions.to_vec();
    assert!((controlled[0][0] - moved[0][0]).abs() < 1e-5);
    let literal = descriptor_source_literal(preview.rig().descriptor()).unwrap();
    let restored = parse_descriptor_source_literal(&literal).unwrap();
    let mut reopened = AnimatedRigPreview::new(f.admit(restored, 42 * 1024 * 1024).unwrap());
    assert_eq!(reopened.advance(0.0, false).unwrap().positions, controlled);
}

#[test]
fn selecting_active_clip_restarts_live_player_and_reopened_source_at_zero() {
    let f = Fixture::new();
    let mut rig = f.admit(f.descriptor.clone(), 42 * 1024 * 1024).unwrap();
    apply_control(&mut rig, RigControl::Seek { seconds: 1.0 }).unwrap();
    assert!((rig.player().positions()[0][0] - 1.0).abs() < 1e-5);
    assert_eq!(rig.descriptor().seek_seconds, 1.0);

    let clip_id = f.descriptor.selected_clip_id;
    apply_control(
        &mut rig,
        RigControl::SelectClip {
            clip_id,
            fade_seconds: 0.0,
        },
    )
    .unwrap();
    assert_eq!(rig.descriptor().selected_clip_id, clip_id);
    assert_eq!(rig.descriptor().seek_seconds, 0.0);
    assert!((rig.player().positions()[0][0]).abs() < 1e-5);
    let live = rig.player().positions().to_vec();
    let receipt = rig.player_mut().advance(0.0, false).unwrap();
    assert_eq!(receipt.time, 0.0);

    let literal = descriptor_source_literal(rig.descriptor()).unwrap();
    let descriptor = parse_descriptor_source_literal(&literal).unwrap();
    let reopened = f.admit(descriptor, 42 * 1024 * 1024).unwrap();
    assert_eq!(reopened.player().positions(), live);
}

#[test]
fn mismatched_source_resource_lineage_and_controls_are_refused() {
    let f = Fixture::new();
    let mut d = f.descriptor.clone();
    d.resource_sha256 = "0".repeat(64);
    assert!(f.admit(d, usize::MAX).is_err());
    let mut d = f.descriptor.clone();
    d.vertex_lineage.swap(0, 1);
    assert!(f.admit(d, usize::MAX).is_err());
    let mut d = f.descriptor.clone();
    d.vertex_lineage.pop();
    assert!(f.admit(d, usize::MAX).is_err());
    let mut d = f.descriptor.clone();
    d.bind_mesh_sha256 = "0".repeat(64);
    assert!(f.admit(d, usize::MAX).is_err());
    let mut d = f.descriptor.clone();
    d.normal_uv_sha256 = "0".repeat(64);
    assert!(f.admit(d, usize::MAX).is_err());
    let mut d = f.descriptor.clone();
    d.clip_ids[0] = 999;
    assert!(f.admit(d, usize::MAX).is_err());
    let mut d = f.descriptor.clone();
    d.joint_deltas.push(JointDeltaSource {
        joint_id: 999,
        motor: [1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    });
    assert!(f.admit(d, usize::MAX).is_err());
    assert!(f.admit(f.descriptor.clone(), 1).is_err());
    // The resource byte bound rejects before section-table/mesh/skin allocation.
    assert!(f
        .admit(f.descriptor.clone(), f.bytes.len() - 1)
        .err()
        .unwrap()
        .contains("resource"));
    // Section counts and mesh/weight working memory reject before cold vectors.
    let counts = qualia_core_db::container_10d::skeletal_container::read_skeletal_section(&f.bytes)
        .unwrap()
        .counts();
    let (cold, _) = cold_budget_required(
        f.bytes.len(),
        counts,
        f.indices.len(),
        f.descriptor.vertex_lineage.len(),
    )
    .unwrap();
    assert!(f
        .admit(f.descriptor.clone(), cold - 1)
        .err()
        .unwrap()
        .contains("cold"));
}

#[test]
fn malformed_mesh_indices_weights_and_player_controls_fail_closed() {
    let mut f = Fixture::new();
    f.indices[2] = 99;
    assert!(f.admit(f.descriptor.clone(), usize::MAX).is_err());
    f.indices[2] = 2;
    let mut changed = f.bytes.clone();
    *changed.last_mut().unwrap() ^= 1; // sealed resource or SKL1 weight corruption
    assert!(admit_bound_rig(
        f.source.clone(),
        f.descriptor.clone(),
        &f.descriptor.resource_key,
        &changed,
        BindMesh {
            positions: &f.positions,
            normals: &f.normals,
            uv0: &f.uv,
            indices: &f.indices
        },
        ViewportSpan {
            first_vertex: 0,
            vertex_count: 3
        },
        usize::MAX
    )
    .is_err());
    let mut rig = f.admit(f.descriptor.clone(), usize::MAX).unwrap();
    assert!(apply_control(
        &mut rig,
        RigControl::SelectClip {
            clip_id: 123,
            fade_seconds: 0.0
        }
    )
    .is_err());
    let before = rig.descriptor().clone();
    assert!(apply_control(
        &mut rig,
        RigControl::SelectClip {
            clip_id: f.descriptor.selected_clip_id,
            fade_seconds: 0.5,
        }
    )
    .is_err());
    assert_eq!(rig.descriptor(), &before);
    assert!(apply_control(&mut rig, RigControl::Seek { seconds: f64::NAN }).is_err());
    assert!(apply_control(
        &mut rig,
        RigControl::SetJointDelta {
            joint_id: 123,
            motor: [1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]
        }
    )
    .is_err());
    assert!(apply_control(
        &mut rig,
        RigControl::SetJointDelta {
            joint_id: f.descriptor.joint_ids[0],
            motor: [2.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]
        }
    )
    .is_err());
}

#[test]
fn sampled_frame_is_hidden_after_source_revision_changes() {
    let f = Fixture::new();
    let mut preview = AnimatedRigPreview::new(f.admit(f.descriptor.clone(), usize::MAX).unwrap());
    assert!(preview.advance(0.0, false).is_ok());
    assert!(preview.frame().is_some());
    studio_store::mutate_artifact_stamped(&f.source.artifact_id, &f.source.stamp, |source| {
        source.title = "changed synthetic source".into();
        Ok(())
    })
    .unwrap();
    assert!(preview.frame().is_none());
    assert!(preview.advance(0.0, false).is_err());
    assert!(f.admit(f.descriptor.clone(), usize::MAX).is_err());
}

#[test]
fn core_codec_refuses_invalid_skin_before_a_resource_can_be_attached() {
    use qualia_core_db::container_10d::skeletal_section::SkeletalSectionError;
    let joints = [Joint {
        parent: None,
        rest_local: Motor::identity(),
        inverse_bind: Motor::identity(),
    }];
    let ids = [qualia_core_db::q_hash("urn:test:invalid-weight-root")];
    let skin = [SkinWeight {
        joints: [0; 4],
        weights: [0.0; 4],
    }];
    let clips = [ClipRecord {
        id: qualia_core_db::q_hash("urn:test:invalid-weight-clip"),
        duration: 1.0,
        first_track: 0,
        track_count: 1,
        looping: false,
    }];
    let tracks = [Track {
        joint: 0,
        first_key: 0,
        key_count: 1,
    }];
    let keys = [Key {
        time: 0.0,
        local: Motor::identity(),
    }];
    let data = SkeletalData {
        joints: &joints,
        joint_ids: &ids,
        skin: &skin,
        clips: &clips,
        tracks: &tracks,
        keys: &keys,
    };
    let mut output = vec![77; 32 + 144 + 32 + 8 + 72 + 24];
    assert_eq!(
        encode_skeletal_section_into(&data, &mut output),
        Err(SkeletalSectionError::InvalidData)
    );
    assert!(output.iter().all(|v| *v == 77));
}

#[test]
fn rejected_rig_advance_retires_previous_frame() {
    let f = Fixture::new();
    let mut preview = AnimatedRigPreview::new(f.admit(f.descriptor.clone(), usize::MAX).unwrap());
    preview.advance(0.0, false).unwrap();
    assert!(preview.frame().is_some());
    assert!(preview.advance(f64::NAN, false).is_err());
    assert!(preview.frame().is_none(), "a failed advance must not expose the prior receipt");
    assert!(preview.advance(0.0, false).is_ok(), "a valid fresh frame can be sampled after refusal");
}

// Include inside rig_binding::tests, alongside its existing Fixture.
#[test]
fn staged_rig_control_keeps_existing_preview_and_refuses_wrong_resource() {
    let f = Fixture::new();
    let mut preview = AnimatedRigPreview::new(f.admit(f.descriptor.clone(), 42 * 1024 * 1024).unwrap());
    let before = preview.advance(0.0, false).unwrap().positions.to_vec();
    let mesh = || BindMesh {
        positions: &f.positions, normals: &f.normals, uv0: &f.uv, indices: &f.indices,
    };
    let span = ViewportSpan { first_vertex: 4, vertex_count: 3 };
    assert!(super::rig_candidate::stage_rig_control(
        f.source.clone(), f.descriptor.clone(), "wrong", &f.bytes, mesh(), span,
        42 * 1024 * 1024, RigControl::Seek { seconds: 1.0 },
    ).is_err());
    let candidate = super::rig_candidate::stage_rig_control(
        f.source.clone(), f.descriptor.clone(), &f.descriptor.resource_key, &f.bytes,
        mesh(), span, 42 * 1024 * 1024, RigControl::Seek { seconds: 1.0 },
    ).unwrap();
    assert_eq!(preview.rig().descriptor(), &f.descriptor);
    assert_eq!(preview.advance(0.0, false).unwrap().positions, before);
    assert_eq!(parse_descriptor_source_literal(candidate.literal()).unwrap(), *candidate.after());
    let mut rebuilt = AnimatedRigPreview::new(f.admit(candidate.after().clone(), 42 * 1024 * 1024).unwrap());
    assert!((rebuilt.advance(0.0, false).unwrap().positions[0][0] - 1.0).abs() < 1e-5);
}

// Include inside rig_binding::tests; requires coordinator registration of
// graphical::authoring::rig_source and rig_binding::rig_candidate.
#[test]
fn rig_source_candidate_evaluates_and_refuses_unsupported_contexts() {
    use crate::graphical::authoring::rig_source;
    use vibe::{session::SourceSession, Budget, LocalHost, Value};
    let f = Fixture::new();
    let descriptor_json = descriptor_source_literal(&f.descriptor).unwrap();
    let encoded = serde_json::to_string(&descriptor_json).unwrap();
    let mut session = SourceSession::default();
    let initial = format!("let mut poet_parts = []; let mut poet_rig = {encoded}; ({{parts:poet_parts}});");
    session.submit(&initial, &mut LocalHost::default(), Budget::default()).unwrap();
    let mesh = BindMesh { positions: &f.positions, normals: &f.normals, uv0: &f.uv, indices: &f.indices };
    let candidate = super::rig_candidate::stage_rig_control(
        f.source.clone(), f.descriptor.clone(), &f.descriptor.resource_key, &f.bytes,
        mesh, ViewportSpan { first_vertex: 4, vertex_count: 3 }, 42 * 1024 * 1024,
        RigControl::Seek { seconds: 1.0 },
    ).unwrap();
    let before = session.value("poet_rig").unwrap().clone();
    assert!(rig_source::edit_source(None, true, &f.source, &candidate).is_err());
    assert!(rig_source::edit_source(Some(&before), false, &f.source, &candidate).is_err());
    assert!(rig_source::edit_source(Some(&Value::Bool(true)), true, &f.source, &candidate).is_err());
    let mut changed = f.descriptor.clone();
    changed.seek_seconds = 0.5;
    let changed = Value::String(descriptor_source_literal(&changed).unwrap());
    assert!(rig_source::edit_source(Some(&changed), true, &f.source, &candidate).is_err());
    assert_eq!(session.value("poet_rig"), Some(&before));
    let cell = rig_source::edit_source(Some(&before), true, &f.source, &candidate).unwrap();
    session.submit(&cell, &mut LocalHost::default(), Budget::default())
        .unwrap_or_else(|e| panic!("rig source cell failed: {e:?}\n{cell}"));
    assert_eq!(rig_source::read_binding(session.value("poet_rig"), &f.source).unwrap(), Some(candidate.after().clone()));
    assert_eq!(session.value("poet_parts"), Some(&Value::List(vec![])));
}

// Exercises the actual canonical source owner with a synthetic selected-resource
// and a no-GPU preview adapter. It is not a project authorization or UI test.
#[test]
fn canonical_rig_edit_updates_source_and_rebuilds_without_losing_binding() {
    use crate::graphical::controller::{CanonicalEditorHost, EditorHost, PreviewProvider};
    use crate::graphical::geometry_host::GeometryHost;
    use crate::graphical::preview::PreviewFrame;
    use vibe::{session::SessionLimits, Value};
    struct FixturePreview { positions: Vec<[f32;3]>, normals: Vec<[f32;3]>, uv: Vec<[f32;2]>, indices: Vec<u32>, cancel_once: bool }
    impl PreviewProvider for FixturePreview {
        fn preview(&mut self, _: SourceRef, _: &Value) -> Result<PreviewFrame,String> { Err("no GPU in this fixture".into()) }
        fn stage_rig_control(&mut self, source: SourceRef, descriptor: RigSourceDescriptor, bytes: &[u8], action: RigControl, cancelled: &std::sync::atomic::AtomicBool) -> Result<super::rig_candidate::StagedRigControl,String> {
            let key=descriptor.resource_key.clone();
            let candidate = super::rig_candidate::stage_rig_control(source,descriptor,&key,bytes,
                BindMesh { positions:&self.positions,normals:&self.normals,uv0:&self.uv,indices:&self.indices },
                ViewportSpan { first_vertex:4,vertex_count:self.positions.len() },42*1024*1024,action)?;
            if self.cancel_once { self.cancel_once=false; cancelled.store(true, std::sync::atomic::Ordering::Release); }
            Ok(candidate)
        }
    }
    let f=Fixture::new();
    let raw=descriptor_source_literal(&f.descriptor).unwrap();
    let quoted=serde_json::to_string(&raw).unwrap();
    let mut host=CanonicalEditorHost::open(f.source.artifact_id.clone(),SessionLimits::default(),
        GeometryHost::new(vec![],200_000),Box::new(FixturePreview{positions:f.positions.clone(),normals:f.normals.clone(),uv:f.uv.clone(),indices:f.indices.clone(),cancel_once:true})).unwrap();
    let initial=format!(r#"let m={{id:1,colour_space:"linear",base_color:[0.3,0.4,0.5,1.0],emissive:[0.0,0.0,0.0],metallic:0.1,roughness:0.5,specular:1.0,alpha_cutoff:0.5,opacity_mode:"Opaque",double_sided:true}}; let tri={{vertices:[[0.0,0.0,0.0],[1.0,0.0,0.0],[0.0,1.0,0.0]],triangles:[[0,1,2]]}}; let mut poet_parts=[{{id:"synthetic-part",semantic_id:101,geometry:tri,material:m,translation:[0.0,0.0,0.0],rotation:[0.0,0.0,0.0],scale:[1.0,1.0,1.0]}}]; let mut poet_rig={quoted}; ({{parts:poet_parts}});"#);
    let accepted=host.submit(&initial,f.source.clone()).unwrap().snapshot;
    let cancelled=std::sync::atomic::AtomicBool::new(false);
    assert!(host.edit_rig_from_selected_resource(&f.descriptor.resource_key,&f.bytes,RigControl::Seek {seconds:1.0},accepted.source.clone(),&cancelled).is_err());
    assert_eq!(host.snapshot().unwrap().source,accepted.source, "cancellation during rig staging must not publish source");
    cancelled.store(false, std::sync::atomic::Ordering::Release);
    let updated=host.edit_rig_from_selected_resource(&f.descriptor.resource_key,&f.bytes,RigControl::Seek {seconds:1.0},accepted.source.clone(),&cancelled).unwrap().snapshot;
    assert_eq!(host.authoring_view(updated.source.clone()).unwrap().rig.unwrap().seek_seconds,1.0);
    assert!(host.edit_rig_from_selected_resource(&f.descriptor.resource_key,&f.bytes,RigControl::Seek {seconds:0.0},accepted.source,&cancelled).is_err());
    let rebuilt=host.rebuild_execution(updated.source).unwrap().snapshot;
    assert_eq!(host.authoring_view(rebuilt.source).unwrap().rig.unwrap().seek_seconds,1.0);
}

// Synthetic source-owner test. The fake preview must not be confused with
// retained viewport correspondence, signed project authority, or HMC reopen.
#[test]
fn canonical_rig_adoption_creates_binding_and_refuses_replacement() {
    use crate::graphical::controller::{CanonicalEditorHost, EditorHost, PreviewProvider};
    use crate::graphical::geometry_host::GeometryHost;
    use crate::graphical::preview::PreviewFrame;
    use vibe::{session::SessionLimits, Value};
    struct FixturePreview { descriptor: RigSourceDescriptor, positions: Vec<[f32;3]>, normals: Vec<[f32;3]>, uv: Vec<[f32;2]>, indices: Vec<u32> }
    impl PreviewProvider for FixturePreview {
        fn preview(&mut self, _: SourceRef, _: &Value) -> Result<PreviewFrame,String> { Err("no GPU in this fixture".into()) }
        fn stage_rig_control(&mut self, source: SourceRef, descriptor: RigSourceDescriptor, bytes: &[u8], action: RigControl, _: &std::sync::atomic::AtomicBool) -> Result<super::rig_candidate::StagedRigControl,String> {
            let RigControl::Adopt { part_id, bind_id } = action else { return Err("expected adoption".into()) };
            if part_id != self.descriptor.part_id || bind_id != self.descriptor.bind_id || descriptor.resource_key != self.descriptor.resource_key { return Err("foreign adoption".into()) }
            let key = self.descriptor.resource_key.clone();
            super::rig_candidate::stage_rig_control(source,self.descriptor.clone(),&key,bytes,
                BindMesh { positions:&self.positions,normals:&self.normals,uv0:&self.uv,indices:&self.indices },
                ViewportSpan { first_vertex:4,vertex_count:self.positions.len() },42*1024*1024,RigControl::Seek { seconds:0.0 })
        }
    }
    let f=Fixture::new();
    let mut host=CanonicalEditorHost::open(f.source.artifact_id.clone(),SessionLimits::default(),
        GeometryHost::new(vec![],200_000),Box::new(FixturePreview { descriptor:f.descriptor.clone(), positions:f.positions.clone(), normals:f.normals.clone(), uv:f.uv.clone(), indices:f.indices.clone() })).unwrap();
    let initial=r#"let m={id:1,colour_space:"linear",base_color:[0.3,0.4,0.5,1.0],emissive:[0.0,0.0,0.0],metallic:0.1,roughness:0.5,specular:1.0,alpha_cutoff:0.5,opacity_mode:"Opaque",double_sided:true}; let tri={vertices:[[0.0,0.0,0.0],[1.0,0.0,0.0],[0.0,1.0,0.0]],triangles:[[0,1,2]]}; let mut poet_parts=[{id:"synthetic-part",semantic_id:101,geometry:tri,material:m,translation:[0.0,0.0,0.0],rotation:[0.0,0.0,0.0],scale:[1.0,1.0,1.0]}]; ({parts:poet_parts});"#;
    let accepted=host.submit(initial,f.source.clone()).unwrap().snapshot;
    let cancelled=std::sync::atomic::AtomicBool::new(false);
    let adopt=RigControl::Adopt { part_id:f.descriptor.part_id.clone(), bind_id:f.descriptor.bind_id.clone() };
    let updated=host.edit_rig_from_selected_resource(&f.descriptor.resource_key,&f.bytes,adopt.clone(),accepted.source.clone(),&cancelled).unwrap().snapshot;
    assert_eq!(host.authoring_view(updated.source.clone()).unwrap().rig.unwrap(), f.descriptor);
    assert!(host.edit_rig_from_selected_resource(&f.descriptor.resource_key,&f.bytes,adopt,updated.source.clone(),&cancelled).is_err());
    assert_eq!(host.snapshot().unwrap().source,updated.source);
}

// Source-owner fixture injects cancellation after a provider has produced a
// frame. This tests retained-state retirement, not an actual GPU or UI race.
#[test]
fn canonical_rig_preview_retires_provider_state_on_late_cancel() {
    use crate::graphical::controller::{CanonicalEditorHost, EditorHost, PreviewProvider};
    use crate::graphical::geometry_host::GeometryHost;
    use crate::graphical::preview::{PreviewCamera, PreviewFrame};
    use crate::graphical::viewport_contract::{ViewportOutput, ViewportState};
    use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
    use vibe::{session::SessionLimits, Value};
    struct CancelAfterFrame { dirty: Arc<AtomicBool>, cancel_once: bool }
    impl PreviewProvider for CancelAfterFrame {
        fn preview(&mut self, _: SourceRef, _: &Value) -> Result<PreviewFrame, String> { Err("no GPU in this fixture".into()) }
        fn preview_selected_rig(&mut self, source: SourceRef, _: RigSourceDescriptor, _: &[u8], _: f64, cancelled: &AtomicBool) -> Result<ViewportOutput, String> {
            self.dirty.store(true, Ordering::Release);
            if self.cancel_once { self.cancel_once = false; cancelled.store(true, Ordering::Release); }
            Ok(ViewportOutput {
                frame: PreviewFrame { source: source.clone(), session_revision: None, execution_epoch: None, side: 1, rgba8: vec![0; 4] },
                state: ViewportState { source, execution_epoch: None, frame_id: 1, selected_part: None, camera: PreviewCamera { yaw: 0.0, pitch: 0.0, distance: 1.0, target: [0.0; 3] } },
            })
        }
        fn invalidate_rig_preview(&mut self) { self.dirty.store(false, Ordering::Release); }
    }
    let f = Fixture::new();
    let dirty = Arc::new(AtomicBool::new(false));
    let mut host = CanonicalEditorHost::open(f.source.artifact_id.clone(), SessionLimits::default(), GeometryHost::new(vec![], 200_000), Box::new(CancelAfterFrame { dirty: dirty.clone(), cancel_once: true })).unwrap();
    let quoted = serde_json::to_string(&descriptor_source_literal(&f.descriptor).unwrap()).unwrap();
    let source = format!("let mut poet_rig={quoted}; 0;");
    let accepted = host.submit(&source, f.source.clone()).unwrap().snapshot.source;
    let cancelled = AtomicBool::new(false);
    assert!(host.preview_rig_from_selected_resource(&f.descriptor.resource_key, &f.bytes, accepted.clone(), 0.0, &cancelled).is_err());
    assert!(cancelled.load(Ordering::Acquire));
    assert!(!dirty.load(Ordering::Acquire), "late cancellation must retire provider state");
    assert_eq!(host.snapshot().unwrap().source, accepted);
    cancelled.store(false, Ordering::Release);
    host.preview_rig_from_selected_resource(&f.descriptor.resource_key, &f.bytes, accepted, 0.0, &cancelled).unwrap();
    assert!(dirty.load(Ordering::Acquire), "successful publication must retain the frame");
}
