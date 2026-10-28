use super::*;
use qualia_core_db::container_10d::field_section::{
    encode_field_section_with_rasters, ContinuousRasterField,
};
use qualia_core_db::container_10d::section::inspect_opaque_container;
use qualia_core_db::container_10d::section::OpaquePreservationBudget;
use qualia_core_db::container_10d::*;
use qualia_core_db::render::assets::Mesh;
use std::sync::atomic::AtomicU64;
static NEXT: AtomicU64 = AtomicU64::new(1);
mod admission;
fn input(section_type: SectionType, payload: &[u8]) -> SectionInput<'_> {
    SectionInput {
        section_type,
        alignment_tier: AlignmentTier::Page,
        stride: 0,
        element_count: 0,
        payload,
    }
}

fn fixture() -> Vec<u8> {
    let mesh = Mesh {
        positions: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
        triangles: vec![[0, 1, 2]],
        min: [0.0; 3],
        max: [1.0, 1.0, 0.0],
    };
    let mut mesh_bytes = vec![0; 1024];
    let n = encode_mesh_section(&mesh, &mut mesh_bytes).unwrap();
    mesh_bytes.truncate(n);
    let mut material = MaterialRecord::legacy_default();
    material.id = 17;
    let mut mat = vec![0; material_section_encoded_len(1, 1).unwrap()];
    encode_material_section(
        &[material],
        &[SubmeshRange {
            first_index: 0,
            index_count: 3,
            material_id: 17,
            semantic_id: 23,
        }],
        3,
        &mut mat,
    )
    .unwrap();
    let mut sidecar = vec![0; 1024];
    let frames = encode_mesh_frames(
        Some(&[[0.0, 0.0, 1.0]; 3]),
        Some(&[[1.0, 0.0, 0.0, 1.0]; 3]),
        &mut sidecar,
    )
    .unwrap();
    let science = encode_field_section_with_rasters(
        &[ContinuousRasterField {
            name_hash: 31,
            unit_hash: 41,
            width: 2,
            height: 1,
            samples: &[2.25, 3.5],
        }],
        &mut sidecar[frames..],
    )
    .unwrap();
    sidecar.truncate(frames + science);
    let fields = [
        TypedFieldRecord {
            field_id: 1,
            kind: FIELD_KIND_SURFACE_FRAMES,
            flags: 0,
            unit_id: 0,
            payload_section_id: 8,
            payload_offset: 0,
            payload_length: frames as u64,
        },
        TypedFieldRecord {
            field_id: 2,
            kind: FIELD_KIND_SCALAR,
            flags: 0,
            unit_id: 41,
            payload_section_id: 8,
            payload_offset: frames as u64,
            payload_length: science as u64,
        },
    ];
    let inputs = [
        input(SectionType::QuantizedMesh, &mesh_bytes),
        input(SectionType::Materials, &mat),
        input(SectionType::FieldSidecar, &sidecar),
    ];
    let identity = ManifoldIdentityV2 {
        entity_id: StableEntityId([7; 16]),
        profile_id: 3,
        domain_id: 5,
        coordinate_convention_id: 7,
        coordinates: [0.0; 10],
        external_digest: None,
    };
    let mut manifest = [0; MANIFOLD_IDENTITY_V2_MAX_BYTES];
    let mut sections = [input(SectionType::QuantizedMesh, &[]); 4];
    let mut out = vec![0; 4096];
    let n = encode_container_with_manifold_identity(
        &Container10dHeader::proposed(),
        &inputs,
        &identity,
        &fields,
        &mut manifest,
        &mut sections,
        &mut out,
    )
    .unwrap();
    out.truncate(n);
    seal_whole_file_crc32c(&mut out);
    out
}
fn request() -> Typed10dEditRequest {
    Typed10dEditRequest {
        resource: ResourceInspectionRequest {
            filename: "typed.hmc".into(),
            package_digest: [1; 32],
            key: "spatial/selected.10d".into(),
            resource_digest: [2; 32],
        },
        source_digest: [3; 32],
        material_id: 17,
        base_color: [0.2, 0.4, 0.6, 1.0],
        emissive: [0.5, 0.0, 0.0],
        metallic: 0.75,
        roughness: 0.25,
    }
}
fn budget() -> OpaquePreservationBudget {
    OpaquePreservationBudget {
        max_bytes: 4096,
        max_sections: 8,
    }
}
#[test]
fn typed_10d_material_delta_preserves_frames_science_geometry_and_every_unrelated_byte() {
    let bytes = fixture();
    let req = request();
    let out =
        payload::replace(&bytes, &req, ui_typed_10d_limits(), &AtomicBool::new(false)).unwrap();
    assert_ne!(bytes, out);
    let before = inspect_opaque_container(&bytes, budget()).unwrap();
    let after = inspect_opaque_container(&out, budget()).unwrap();
    let index = before
        .sections()
        .iter()
        .position(|d| d.typ() == Some(SectionType::Materials))
        .unwrap();
    for i in 0..before.sections().len() {
        if i != index {
            assert_eq!(before.sections()[i], after.sections()[i]);
            assert_eq!(before.payload(i), after.payload(i));
        }
    }
    let descriptor = before.sections()[index];
    let mat_range = descriptor.byte_offset as usize
        ..descriptor.byte_offset as usize + descriptor.byte_length as usize;
    let header_crc = std::mem::offset_of!(Container10dHeader, header_crc32c);
    let descriptor_crc = before.header().section_table_offset as usize
        + index * SECTION_DESCRIPTOR_SIZE
        + std::mem::offset_of!(SectionDescriptor, crc32c);
    for i in 0..bytes.len() {
        if !mat_range.contains(&i)
            && !(header_crc..header_crc + 4).contains(&i)
            && !(descriptor_crc..descriptor_crc + 4).contains(&i)
        {
            assert_eq!(bytes[i], out[i], "byte {i}");
        }
    }
    let p = payload::inspect(&out, ui_typed_10d_limits()).unwrap();
    assert_eq!(p.materials[0].base_color, req.base_color);
    assert_eq!(p.materials[0].metallic, 0.75);
    assert_eq!(p.ranges[0].semantic_id, 23);
    assert_eq!(p.fields.len(), 2);
    assert!(p.frames.unwrap().has_tangents);
}
#[test]
fn typed_10d_corrupt_limits_cancel_and_nonfinite_refuse_without_mutating_input() {
    let bytes = fixture();
    let original = bytes.clone();
    let mut req = request();
    for v in [f32::NAN, f32::INFINITY, -0.1, 1.1] {
        req.roughness = v;
        assert!(
            payload::replace(&bytes, &req, ui_typed_10d_limits(), &AtomicBool::new(false)).is_err()
        );
    }
    req = request();
    assert!(payload::replace(&bytes, &req, ui_typed_10d_limits(), &AtomicBool::new(true)).is_err());
    for limits in [
        Typed10dLimits {
            max_file_bytes: bytes.len() - 1,
            ..ui_typed_10d_limits()
        },
        Typed10dLimits {
            max_materials: 0,
            ..ui_typed_10d_limits()
        },
        Typed10dLimits {
            max_ranges: 0,
            ..ui_typed_10d_limits()
        },
        Typed10dLimits {
            max_vertices: 2,
            ..ui_typed_10d_limits()
        },
        Typed10dLimits {
            max_working_bytes: 1,
            ..ui_typed_10d_limits()
        },
    ] {
        assert!(payload::replace(&bytes, &req, limits, &AtomicBool::new(false)).is_err());
    }
    let mut corrupt = bytes.clone();
    let last = corrupt.len() - 1;
    corrupt[last] ^= 1;
    assert!(payload::replace(
        &corrupt,
        &req,
        ui_typed_10d_limits(),
        &AtomicBool::new(false)
    )
    .is_err());
    let mut tail = bytes.clone();
    tail.push(1);
    seal_whole_file_crc32c(&mut tail);
    assert!(payload::replace(&tail, &req, ui_typed_10d_limits(), &AtomicBool::new(false)).is_err());
    req.material_id = 999;
    assert!(
        payload::replace(&bytes, &req, ui_typed_10d_limits(), &AtomicBool::new(false)).is_err()
    );
    assert_eq!(bytes, original);
}

fn saved(
    root: &Path,
    meta: Option<Vec<u8>>,
) -> (CheckpointService, SourceRef, Typed10dEditRequest) {
    let id = format!(
        "urn:typed10d-test:{}:{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    );
    let (artifact, stamp) = studio_store::create_artifact_if_absent(StudioArtifact::new(
        &id,
        "Geometry",
        "typed resource fixture",
    ))
    .unwrap();
    let limits = CheckpointLimits::default();
    let mut req = request();
    let source = SourceRef {
        artifact_id: id.clone(),
        stamp: stamp.clone(),
    };
    req.source_digest = crate::graphical::cooperation::source_digest(
        &artifact,
        crate::graphical::cooperation::GatewayLimits::default(),
    )
    .unwrap();
    let checkpoint = core::capture(&id, &stamp, limits)
        .unwrap()
        .apply_resource_edits(
            vec![
                ResourceEdit::Add(ResourceData {
                    key: req.resource.key.clone(),
                    kind: "10d".into(),
                    bytes: fixture(),
                    meta,
                }),
                ResourceEdit::Add(ResourceData {
                    key: "unknown/retained".into(),
                    kind: "future".into(),
                    bytes: vec![4, 5, 6],
                    meta: Some(vec![0xa1, 0x61, b'x', 1]),
                }),
            ],
            limits,
        )
        .unwrap();
    req.resource.resource_digest = checkpoint
        .inventory()
        .into_iter()
        .find(|r| r.key == req.resource.key)
        .unwrap()
        .digest;
    let bytes = core::encode(&checkpoint, vec![], limits).unwrap();
    req.resource.package_digest = core::save(
        &root.join(&req.resource.filename),
        &bytes,
        ExpectedFile::Absent,
        limits,
    )
    .unwrap()
    .digest;
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
    (service, source, req)
}
#[test]
fn typed_10d_hmc_save_reopen_retains_unknown_resource_and_changes_selected_material() {
    let root = tempfile::tempdir().unwrap();
    let (service, source, req) = saved(root.path(), None);
    let inspected = service
        .inspect_typed_10d(req.resource.clone(), &source, ui_typed_10d_limits())
        .unwrap();
    assert!(inspected.edit_blocker.is_none());
    let receipt = service
        .edit_typed_10d(
            req.clone(),
            &source,
            ui_typed_10d_limits(),
            &AtomicBool::new(false),
        )
        .unwrap();
    assert_ne!(receipt.digest, req.resource.package_digest);
    let reopened = core::load(
        &root.path().join(&req.resource.filename),
        CheckpointLimits::default(),
    )
    .unwrap();
    assert_eq!(reopened.parent_digest(), Some(req.resource.package_digest));
    let retained = reopened
        .resources()
        .iter()
        .find(|r| r.key == "unknown/retained")
        .unwrap();
    assert_eq!(retained.bytes, vec![4, 5, 6]);
    assert_eq!(retained.meta, Some(vec![0xa1, 0x61, b'x', 1]));
    let resource = reopened
        .resources()
        .iter()
        .find(|r| r.key == req.resource.key)
        .unwrap();
    assert_eq!(resource.kind, "10d");
    assert!(resource.meta.is_none());
    assert_eq!(
        payload::inspect(&resource.bytes, ui_typed_10d_limits())
            .unwrap()
            .materials[0]
            .metallic,
        0.75
    );
}
#[test]
fn typed_10d_stale_source_package_resource_and_opaque_metadata_leave_hmc_unchanged() {
    let root = tempfile::tempdir().unwrap();
    let (service, source, req) = saved(root.path(), None);
    let path = root.path().join(&req.resource.filename);
    let before = std::fs::read(&path).unwrap();
    for which in 0..3 {
        let mut stale = req.clone();
        match which {
            0 => stale.source_digest[0] ^= 1,
            1 => stale.resource.package_digest[0] ^= 1,
            _ => stale.resource.resource_digest[0] ^= 1,
        }
        assert!(service
            .edit_typed_10d(
                stale,
                &source,
                ui_typed_10d_limits(),
                &AtomicBool::new(false)
            )
            .is_err());
        assert_eq!(std::fs::read(&path).unwrap(), before);
    }
    let (_, new_stamp) =
        studio_store::mutate_artifact_stamped(&source.artifact_id, &source.stamp, |artifact| {
            artifact.title = "changed fixture source".into();
            Ok(())
        })
        .unwrap();
    let current_source = SourceRef {
        artifact_id: source.artifact_id.clone(),
        stamp: new_stamp,
    };
    assert!(service
        .inspect_typed_10d(req.resource.clone(), &current_source, ui_typed_10d_limits())
        .is_err());
    assert!(service
        .edit_typed_10d(req, &source, ui_typed_10d_limits(), &AtomicBool::new(false))
        .is_err());
    assert_eq!(std::fs::read(path).unwrap(), before);
    let root = tempfile::tempdir().unwrap();
    let (service, source, req) = saved(root.path(), Some(vec![0xa0]));
    let path = root.path().join(&req.resource.filename);
    let before = std::fs::read(&path).unwrap();
    assert!(service
        .edit_typed_10d(req, &source, ui_typed_10d_limits(), &AtomicBool::new(false))
        .is_err());
    assert_eq!(std::fs::read(path).unwrap(), before);
}

/// Synthetic engineering fixture: no biological or rights claim.
fn skeletal_fixture() -> Vec<u8> {
    use qualia_core_db::container_10d::{
        skeletal_container::attach_skeletal_section_into,
        skeletal_section::{encode_skeletal_section_into, ClipRecord, SkeletalData},
    };
    use qualia_core_db::render::skeletal::{Joint, Key, Motor, SkinWeight, Track};
    let joints = [Joint { parent: None, rest_local: Motor::identity(), inverse_bind: Motor::identity() }];
    let joint_ids = [qualia_core_db::q_hash("urn:test:inspection:root")];
    let skins = [SkinWeight { joints: [0; 4], weights: [1.0, 0.0, 0.0, 0.0] }; 3];
    let clips = [ClipRecord { id: qualia_core_db::q_hash("urn:test:inspection:clip"), duration: 1.0, first_track: 0, track_count: 1, looping: false }];
    let tracks = [Track { joint: 0, first_key: 0, key_count: 1 }];
    let keys = [Key { time: 0.0, local: Motor::identity() }];
    let data = SkeletalData { joints: &joints, joint_ids: &joint_ids, skin: &skins, clips: &clips, tracks: &tracks, keys: &keys };
    let mut skeletal = vec![0; 1024];
    let n = encode_skeletal_section_into(&data, &mut skeletal).unwrap();
    skeletal.truncate(n);
    let mut out = vec![0; 32768];
    let n = attach_skeletal_section_into(&fixture(), &skeletal, &mut out).unwrap();
    out.truncate(n);
    out
}

#[test]
fn skeletal_inspection_distinguishes_present_absent_and_unassessed() {
    assert_eq!(payload::inspect_skeletal(&fixture()), SkeletalInspection::Absent);
    let bytes = skeletal_fixture();
    assert_eq!(payload::inspect_skeletal(&bytes), SkeletalInspection::Present(
        skeletal_section::Counts { joints: 1, vertices: 3, clips: 1, tracks: 1, keys: 1 }));
    let mut corrupt = bytes;
    let end = corrupt.len() - 1;
    corrupt[end] ^= 1;
    assert!(matches!(payload::inspect_skeletal(&corrupt), SkeletalInspection::Unassessed(_)));
}

#[test]
fn typed_10d_selected_package_reports_skeletal_counts_and_preserves_edit_blocker() {
    let root = tempfile::tempdir().unwrap();
    let (service, source, mut req) = saved(root.path(), None);
    let path = root.path().join(&req.resource.filename);
    let limits = CheckpointLimits::default();
    let checkpoint = core::load(&path, limits).unwrap().apply_resource_edits(
        vec![ResourceEdit::ReplaceBytes { key: req.resource.key.clone(), expected: req.resource.resource_digest,
            bytes: skeletal_fixture() }], limits).unwrap();
    req.resource.resource_digest = checkpoint.inventory().into_iter()
        .find(|r| r.key == req.resource.key).unwrap().digest;
    let bytes = core::encode(&checkpoint, vec![], limits).unwrap();
    req.resource.package_digest = core::save(&path, &bytes,
        ExpectedFile::Sha256(req.resource.package_digest), limits).unwrap().digest;
    let result = service.inspect_typed_10d(req.resource, &source, ui_typed_10d_limits()).unwrap();
    assert_eq!(result.skeletal, SkeletalInspection::Present(
        skeletal_section::Counts { joints: 1, vertices: 3, clips: 1, tracks: 1, keys: 1 }));
    assert!(result.edit_blocker.is_some(), "material derivative must not silently discard dependent skeletal data");
}
