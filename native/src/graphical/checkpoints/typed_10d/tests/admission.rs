use super::*;
use qualia_core_db::container_10d::section::replace_known_section_bytes_preserving_layout;

fn change_section(bytes: &[u8], kind: SectionType, replacement: &[u8]) -> Vec<u8> {
    let view = inspect_opaque_container(bytes, budget()).unwrap();
    let mut inputs = Vec::new();
    for (i, d) in view.sections().iter().enumerate() {
        inputs.push(input(
            d.typ().unwrap(),
            if d.typ() == Some(kind) {
                replacement
            } else {
                view.payload(i).unwrap()
            },
        ));
    }
    let mut out = vec![0; 4096];
    let n = encode_container(view.header(), &inputs, &mut out).unwrap();
    out.truncate(n);
    seal_whole_file_crc32c(&mut out);
    out
}

#[test]
fn typed_10d_rejects_malformed_science_overlapping_bindings_and_unknown_metadata() {
    let bytes = fixture();
    let req = request();
    let view = inspect_opaque_container(&bytes, budget()).unwrap();
    let side_i = view
        .sections()
        .iter()
        .position(|d| d.typ() == Some(SectionType::FieldSidecar))
        .unwrap();
    let mat_i = view
        .sections()
        .iter()
        .position(|d| d.typ() == Some(SectionType::ManifoldIdentityV2))
        .unwrap();
    let mut fields = [TypedFieldRecord::default(); MANIFOLD_IDENTITY_V2_MAX_FIELDS];
    let (identity, count) =
        decode_manifold_identity_v2(view.payload(mat_i).unwrap(), &[], &mut fields).unwrap();
    let mut side = view.payload(side_i).unwrap().to_vec();
    side[fields[1].payload_offset as usize + 8] = 1; // FIEL reserved header, valid envelope after replacement.
    let malformed = change_section(&bytes, SectionType::FieldSidecar, &side);
    assert!(payload::replace(
        &malformed,
        &req,
        ui_typed_10d_limits(),
        &AtomicBool::new(false)
    )
    .is_err());
    fields[1].payload_offset = 0;
    let mut manifest = vec![0; MANIFOLD_IDENTITY_V2_MAX_BYTES];
    let n = encode_manifold_identity_v2(&identity, &fields[..count], &mut manifest).unwrap();
    manifest.truncate(n);
    let ambiguous = change_section(&bytes, SectionType::ManifoldIdentityV2, &manifest);
    assert!(payload::replace(
        &ambiguous,
        &req,
        ui_typed_10d_limits(),
        &AtomicBool::new(false)
    )
    .is_err());
    let mut metadata = bytes.clone();
    let table = view.header().section_table_offset as usize;
    let at = table + std::mem::offset_of!(SectionDescriptor, reserved16);
    metadata[at] = 1;
    seal_whole_file_crc32c(&mut metadata);
    assert!(payload::replace(
        &metadata,
        &req,
        ui_typed_10d_limits(),
        &AtomicBool::new(false)
    )
    .is_err());
    // A raw preservation operation can copy the original byte-exactly; it is
    // explicitly not a semantic-material admission token.
    let mut out = vec![0; bytes.len()];
    replace_known_section_bytes_preserving_layout(
        &bytes,
        SectionType::Materials,
        view.payload(
            view.sections()
                .iter()
                .position(|d| d.typ() == Some(SectionType::Materials))
                .unwrap(),
        )
        .unwrap(),
        budget(),
        &mut out,
    )
    .unwrap();
    assert_eq!(out, bytes);
}

#[test]
fn typed_10d_legacy_mat3_inspects_without_silent_upgrade_and_external_hint_holds_edit() {
    let bytes = fixture();
    let view = inspect_opaque_container(&bytes, budget()).unwrap();
    let index = view
        .sections()
        .iter()
        .position(|d| d.typ() == Some(SectionType::Materials))
        .unwrap();
    let original = view.payload(index).unwrap();
    // Independent fixture oracle: MAT3 has the shared 448-byte material record,
    // omitting the MAT4 wind/reserved extension, followed by the same range.
    let mut mat3 = original[..MATERIAL_SECTION_HEADER_SIZE + 448].to_vec();
    mat3.extend_from_slice(&original[MATERIAL_SECTION_HEADER_SIZE + MATERIAL_RECORD_SIZE..]);
    mat3[..4].copy_from_slice(b"MAT3");
    mat3[4..6].copy_from_slice(&3u16.to_le_bytes());
    mat3[16..20].copy_from_slice(&448u32.to_le_bytes());
    let legacy = change_section(&bytes, SectionType::Materials, &mat3);
    let inspected = payload::inspect(&legacy, ui_typed_10d_limits()).unwrap();
    assert_eq!(inspected.materials.len(), 1);
    assert!(inspected.blocker.unwrap().contains("inspection-only"));
    assert!(payload::replace(
        &legacy,
        &request(),
        ui_typed_10d_limits(),
        &AtomicBool::new(false)
    )
    .is_err());
    let index = view
        .sections()
        .iter()
        .position(|d| d.typ() == Some(SectionType::ManifoldIdentityV2))
        .unwrap();
    let mut fields = [TypedFieldRecord::default(); MANIFOLD_IDENTITY_V2_MAX_FIELDS];
    let (mut identity, count) =
        decode_manifold_identity_v2(view.payload(index).unwrap(), &[], &mut fields).unwrap();
    identity.external_digest = Some(IntegrityIndexDigest([9; 16]));
    let mut manifest = vec![0; MANIFOLD_IDENTITY_V2_MAX_BYTES];
    let n = encode_manifold_identity_v2(&identity, &fields[..count], &mut manifest).unwrap();
    manifest.truncate(n);
    let hinted = change_section(&bytes, SectionType::ManifoldIdentityV2, &manifest);
    assert!(payload::inspect(&hinted, ui_typed_10d_limits())
        .unwrap()
        .blocker
        .unwrap()
        .contains("external digest"));
    assert!(payload::replace(
        &hinted,
        &request(),
        ui_typed_10d_limits(),
        &AtomicBool::new(false)
    )
    .is_err());
}

#[test]
fn typed_10d_unknown_extensions_and_mismatched_scientific_kinds_refuse() {
    let bytes = fixture();
    let view = inspect_opaque_container(&bytes, budget()).unwrap();
    let index = view
        .sections()
        .iter()
        .position(|d| d.typ() == Some(SectionType::ManifoldIdentityV2))
        .unwrap();
    let mut fields = [TypedFieldRecord::default(); MANIFOLD_IDENTITY_V2_MAX_FIELDS];
    let (identity, count) =
        decode_manifold_identity_v2(view.payload(index).unwrap(), &[], &mut fields).unwrap();
    let mut manifest = vec![0; MANIFOLD_IDENTITY_V2_MAX_BYTES];
    fields[1].kind = FIELD_KIND_VECTOR;
    let n = encode_manifold_identity_v2(&identity, &fields[..count], &mut manifest).unwrap();
    let mismatch = change_section(&bytes, SectionType::ManifoldIdentityV2, &manifest[..n]);
    assert!(payload::replace(
        &mismatch,
        &request(),
        ui_typed_10d_limits(),
        &AtomicBool::new(false)
    )
    .is_err());
    fields[1].kind = 0x8001;
    let n = encode_manifold_identity_v2_with_extensions(
        &identity,
        &fields[..count],
        &[0x8001],
        &mut manifest,
    )
    .unwrap();
    let unknown = change_section(&bytes, SectionType::ManifoldIdentityV2, &manifest[..n]);
    assert!(payload::replace(
        &unknown,
        &request(),
        ui_typed_10d_limits(),
        &AtomicBool::new(false)
    )
    .is_err());
    let mut unknown_section = bytes.clone();
    unknown_section[view.header().section_table_offset as usize] = 200;
    seal_whole_file_crc32c(&mut unknown_section);
    assert!(payload::replace(
        &unknown_section,
        &request(),
        ui_typed_10d_limits(),
        &AtomicBool::new(false)
    )
    .is_err());
}

fn no_sidecar_binding(target: Option<SectionType>) -> Vec<u8> {
    let bytes = fixture();
    let view = inspect_opaque_container(&bytes, budget()).unwrap();
    let index = view
        .sections()
        .iter()
        .position(|d| d.typ() == Some(SectionType::ManifoldIdentityV2))
        .unwrap();
    let mut old_fields = [TypedFieldRecord::default(); MANIFOLD_IDENTITY_V2_MAX_FIELDS];
    let (identity, _) =
        decode_manifold_identity_v2(view.payload(index).unwrap(), &[], &mut old_fields).unwrap();
    let payload_length = target
        .map(|kind| {
            view.sections()
                .iter()
                .find(|d| d.typ() == Some(kind))
                .unwrap()
                .byte_length as u64
        })
        .unwrap_or(0);
    let fields = [TypedFieldRecord {
        field_id: 1,
        kind: FIELD_KIND_SCALAR,
        flags: 0,
        unit_id: 41,
        payload_section_id: target.map(|kind| kind as u32).unwrap_or(0),
        payload_offset: 0,
        payload_length,
    }];
    let mut manifest = vec![0; MANIFOLD_IDENTITY_V2_MAX_BYTES];
    let n = encode_manifold_identity_v2(&identity, &fields, &mut manifest).unwrap();
    manifest.truncate(n);
    let mut inputs = Vec::new();
    for (i, d) in view.sections().iter().enumerate() {
        match d.typ().unwrap() {
            SectionType::FieldSidecar => {}
            SectionType::ManifoldIdentityV2 => {
                inputs.push(input(SectionType::ManifoldIdentityV2, &manifest))
            }
            kind => inputs.push(input(kind, view.payload(i).unwrap())),
        }
    }
    let mut out = vec![0; 4096];
    let n = encode_container(view.header(), &inputs, &mut out).unwrap();
    out.truncate(n);
    seal_whole_file_crc32c(&mut out);
    out
}

#[test]
fn typed_10d_no_sidecar_material_mesh_and_empty_declarations_refuse_without_hmc_mutation() {
    for target in [
        Some(SectionType::Materials),
        Some(SectionType::QuantizedMesh),
        None,
    ] {
        let bytes = no_sidecar_binding(target);
        let original = bytes.clone();
        assert!(payload::replace(
            &bytes,
            &request(),
            ui_typed_10d_limits(),
            &AtomicBool::new(false)
        )
        .is_err());
        assert_eq!(bytes, original);
        // The canonical format permits generic section bindings. Install this
        // inert source fixture through its codec, then exercise editor policy.
        let root = tempfile::tempdir().unwrap();
        let (service, source, mut req) = saved(root.path(), None);
        let limits = CheckpointLimits::default();
        let path = root.path().join(&req.resource.filename);
        let carrier = core::load(&path, limits)
            .unwrap()
            .apply_resource_edits(
                vec![ResourceEdit::ReplaceBytes {
                    key: req.resource.key.clone(),
                    expected: req.resource.resource_digest,
                    bytes,
                }],
                limits,
            )
            .unwrap();
        req.resource.resource_digest = carrier
            .inventory()
            .iter()
            .find(|r| r.key == req.resource.key)
            .unwrap()
            .digest;
        let candidate = core::encode(&carrier, vec![], limits).unwrap();
        req.resource.package_digest = core::save(
            &path,
            &candidate,
            ExpectedFile::Sha256(req.resource.package_digest),
            limits,
        )
        .unwrap()
        .digest;
        let before = std::fs::read(&path).unwrap();
        assert!(service
            .edit_typed_10d(req, &source, ui_typed_10d_limits(), &AtomicBool::new(false))
            .is_err());
        assert_eq!(std::fs::read(&path).unwrap(), before);
    }
}
