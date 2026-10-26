//! Cold authoring policy over checked native format APIs; no binary codec here.
use super::*;
use qualia_core_db::container_10d::field_section::{inspect_field_section, FieldInspectionLimits};
use qualia_core_db::container_10d::{
    section::{
        inspect_opaque_container, replace_known_section_bytes_preserving_layout,
        OpaquePreservationBudget,
    },
    *,
};

pub(super) struct Payload {
    pub version: u16,
    pub vertices: usize,
    pub indices: u32,
    pub materials: Vec<MaterialRecord>,
    pub ranges: Vec<SubmeshRange>,
    pub fields: Vec<TypedFieldRecord>,
    pub frames: Option<MeshFrameLayout>,
    pub blocker: Option<String>,
}
fn budget(limits: Typed10dLimits) -> OpaquePreservationBudget {
    OpaquePreservationBudget {
        max_bytes: limits.max_file_bytes,
        max_sections: limits.max_sections,
    }
}

pub(super) fn inspect(bytes: &[u8], limits: Typed10dLimits) -> Result<Payload, String> {
    let view = inspect_opaque_container(bytes, budget(limits)).map_err(|e| e.to_string())?;
    // Unknowns are retained only through explicit structural copy, never admitted
    // to semantic editing. The typed resolver uses the strict canonical table.
    let sections = parse_section_table(bytes, view.header()).map_err(|e| e.to_string())?;
    let get = |kind| {
        sections
            .iter()
            .position(|d| d.typ() == Some(kind))
            .and_then(|i| view.payload(i))
    };
    let mesh_bytes =
        get(SectionType::QuantizedMesh).ok_or("Selected .10d has no quantized mesh")?;
    let (header, needed) = parse_mesh_header(mesh_bytes).map_err(|e| e.to_string())?;
    if needed != mesh_bytes.len() {
        return Err("Mesh has unassessed trailing bytes".into());
    }
    let vertices = header.vertex_count as usize;
    if vertices > limits.max_vertices {
        return Err("Typed .10d vertex budget exceeded".into());
    }
    let indices = header
        .triangle_count
        .checked_mul(3)
        .ok_or("Mesh index count overflow")?;
    let mat_bytes =
        get(SectionType::Materials).ok_or("Selected .10d has no authored material section")?;
    let (mc, rc) = material_section_counts(mat_bytes).map_err(|e| e.to_string())?;
    if mc > limits.max_materials || rc > limits.max_ranges {
        return Err("Typed .10d material/range budget exceeded".into());
    }
    // Includes live material/range output, decoded mesh + optional UVs, two
    // container buffers and MAT re-encoding scratch. HMC has separate limits.
    let working = vertices
        .checked_mul(20)
        .and_then(|n| n.checked_add(header.triangle_count as usize * 12))
        .and_then(|n| n.checked_add(mc * std::mem::size_of::<MaterialRecord>()))
        .and_then(|n| n.checked_add(rc * std::mem::size_of::<SubmeshRange>()))
        .and_then(|n| n.checked_add(bytes.len().checked_mul(3)?))
        .ok_or("Typed .10d working budget overflow")?;
    if working > limits.max_working_bytes {
        return Err("Typed .10d working budget exceeded".into());
    }
    let mesh = decode_mesh_section(mesh_bytes).map_err(|e| e.to_string())?;
    if mesh
        .triangles
        .iter()
        .flatten()
        .any(|i| *i as usize >= vertices)
    {
        return Err("Canonical mesh contains an index outside the vertex stream".into());
    }
    drop(mesh);
    let mut materials = buffer(mc, MaterialRecord::legacy_default())?;
    let mut ranges = buffer(
        rc,
        SubmeshRange {
            first_index: 0,
            index_count: 0,
            material_id: 0,
            semantic_id: 0,
        },
    )?;
    decode_material_section_into(mat_bytes, indices, &mut materials, &mut ranges)
        .map_err(|e| e.to_string())?;
    let mut scratch = [TypedFieldRecord::default(); MANIFOLD_IDENTITY_V2_MAX_FIELDS];
    let mut fields = [TypedFieldRecord::default(); MANIFOLD_IDENTITY_V2_MAX_FIELDS];
    let resolver = TypedPayloadResolver::new(bytes, &[], &mut scratch, &mut fields)
        .map_err(|e| e.to_string())?;
    // Validate all declarations independently of optional sidecar presence.
    // A generic scientific field may legally reference another envelope
    // section at the format layer; this bounded material editor has no such
    // derivation contract and must not modify bytes it declares authoritative.
    for field in resolver.fields() {
        if field.payload_section_id != SectionType::FieldSidecar as u32 || field.payload_length == 0
        {
            return Err("Typed material profile requires nonempty explicit FieldSidecar bindings for every declaration".into());
        }
    }
    let frame_limits = MeshFrameDecodeLimits {
        max_vertices: limits.max_vertices,
        max_payload_bytes: limits.max_file_bytes,
        max_decoded_bytes: limits.max_working_bytes,
    };
    let frames =
        inspect_container_mesh_frames(bytes, vertices, frame_limits).map_err(|e| e.to_string())?;
    let mut blocker = None;
    if resolver.has_manifest() {
        let (identity, _) = decode_container_manifold_identity(
            bytes,
            view.header(),
            &[],
            &mut scratch,
            &mut [TypedFieldRecord::default(); MANIFOLD_IDENTITY_V2_MAX_FIELDS],
        )
        .map_err(|e| format!("MID2 dependency inspection refused: {e:?}"))?;
        if identity.external_digest.is_some() {
            blocker = Some("MID2 external digest/index hint has no qualified material derivation; use authored source".into());
        }
    }
    if view.header().flags & !FLAG_DEFAULT_DISPOSITION_REFUSE != 0
        || sections.iter().any(|d| d.reserved16 != 0)
    {
        blocker = Some(
            "Unknown .10d flags/descriptor metadata have no material derivation contract".into(),
        );
    }
    for d in sections {
        match d.typ() {
            Some(SectionType::QuantizedMesh | SectionType::Materials | SectionType::FieldSidecar | SectionType::ManifoldIdentityV2 | SectionType::TextureCoordinates) => {}
            _ => blocker = Some("Selected .10d has dependent sections outside this material edit profile; use its authored source".into()),
        }
    }
    if let Some(uv) = get(SectionType::TextureCoordinates) {
        let mut coords = buffer(vertices, [0.0; 2])?;
        decode_texture_coordinates_into(uv, vertices, &mut coords).map_err(|e| e.to_string())?;
    }
    if let Some(sidecar) = get(SectionType::FieldSidecar) {
        if resolver.has_manifest() {
            // Every sidecar byte must have one nonoverlapping, known declaration.
            let mut covered = 0u64;
            for field in resolver.fields() {
                if resolver.fields().iter().any(|other| {
                    other.field_id != field.field_id
                        && field.payload_offset < other.payload_offset + other.payload_length
                        && other.payload_offset < field.payload_offset + field.payload_length
                }) {
                    return Err("Ambiguous overlapping typed sidecar declarations".into());
                }
                let p = resolver
                    .field(field.field_id)
                    .map_err(|e| e.to_string())?
                    .ok_or("Missing typed payload")?;
                if field.kind == FIELD_KIND_SURFACE_FRAMES {
                    resolver.surface_frames().map_err(|e| e.to_string())?;
                } else if matches!(
                    field.kind,
                    FIELD_KIND_SCALAR | FIELD_KIND_VECTOR | FIELD_KIND_TENSOR | FIELD_KIND_SPECTRAL
                ) {
                    let scientific = inspect_field_section(
                        p.bytes,
                        FieldInspectionLimits {
                            max_payload_bytes: limits.max_file_bytes,
                            max_fields: 256,
                            max_samples: limits.max_file_bytes / 4,
                        },
                    )
                    .map_err(|e| e.to_string())?;
                    // Registry kinds and FIEL kinds have distinct numeric tags;
                    // this explicit binding supports one homogeneous FIEL part.
                    let expected_mask = match field.kind {
                        FIELD_KIND_SCALAR => 1,
                        FIELD_KIND_VECTOR => 2,
                        FIELD_KIND_TENSOR => 4,
                        FIELD_KIND_SPECTRAL => 8,
                        _ => unreachable!("matched scientific kinds"),
                    };
                    if scientific.type_mask != expected_mask {
                        return Err(
                            "MID2 kind does not bind a homogeneous matching FIEL schema".into()
                        );
                    }
                } else {
                    blocker = Some("Typed sidecar schema is outside this material edit profile; preserved but editing is held".into());
                }
                covered = covered
                    .checked_add(field.payload_length)
                    .ok_or("Typed sidecar coverage overflow")?;
            }
            if covered != sidecar.len() as u64 {
                return Err("Unclaimed typed sidecar bytes; semantic edit refused".into());
            }
        } else if !is_normal_section(sidecar) {
            inspect_field_section(
                sidecar,
                FieldInspectionLimits {
                    max_payload_bytes: limits.max_file_bytes,
                    max_fields: 256,
                    max_samples: limits.max_file_bytes / 4,
                },
            )
            .map_err(|e| e.to_string())?;
        }
    }
    let mut field_output = buffer(resolver.fields().len(), TypedFieldRecord::default())?;
    field_output.copy_from_slice(resolver.fields());
    if !mat_bytes.starts_with(b"MAT4") {
        blocker = Some("MAT1/MAT2/MAT3 are inspection-only; automatic upgrade is refused".into());
    }
    Ok(Payload {
        version: view.header().version,
        vertices,
        indices,
        materials,
        ranges,
        fields: field_output,
        frames,
        blocker,
    })
}

pub(super) fn replace(
    bytes: &[u8],
    request: &Typed10dEditRequest,
    limits: Typed10dLimits,
    cancelled: &AtomicBool,
) -> Result<Vec<u8>, String> {
    request.validate_bounds()?;
    live(cancelled)?;
    let mut payload = inspect(bytes, limits)?;
    if let Some(blocker) = payload.blocker.take() {
        return Err(blocker);
    }
    let view = inspect_opaque_container(bytes, budget(limits)).map_err(|e| e.to_string())?;
    let index = view
        .sections()
        .iter()
        .position(|d| d.typ() == Some(SectionType::Materials))
        .ok_or("Materials absent")?;
    let original = view.payload(index).ok_or("Materials absent")?;
    let mut encoded = buffer(original.len(), 0u8)?;
    let n = encode_material_section(
        &payload.materials,
        &payload.ranges,
        payload.indices,
        &mut encoded,
    )
    .map_err(|e| e.to_string())?;
    if encoded[..n] != *original {
        return Err(
            "MAT4 source is not byte-exactly preserved by canonical regeneration; edit held".into(),
        );
    }
    let material = payload
        .materials
        .iter_mut()
        .find(|m| m.id == request.material_id)
        .ok_or("Selected material identifier is absent")?;
    material.base_color = request.base_color;
    material.emissive = request.emissive;
    material.metallic = request.metallic;
    material.roughness = request.roughness;
    encode_material_section(
        &payload.materials,
        &payload.ranges,
        payload.indices,
        &mut encoded,
    )
    .map_err(|e| e.to_string())?;
    let mut out = buffer(bytes.len(), 0u8)?;
    replace_known_section_bytes_preserving_layout(
        bytes,
        SectionType::Materials,
        &encoded,
        budget(limits),
        &mut out,
    )
    .map_err(|e| e.to_string())?;
    drop(payload);
    inspect(&out, limits)?;
    live(cancelled)?;
    Ok(out)
}
