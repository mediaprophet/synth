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
