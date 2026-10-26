//! Source-bound SKL1 admission for native Poet's animated preview.
//!
//! The source descriptor is durable authoring data. `BoundRig` and the Core
//! player are disposable projections rebuilt from the selected sealed `.10d`.
//! The current profile admits exact vertex correspondence only. A changed
//! topology, including flat-normal or atlas seam expansion, requires an
//! explicit qualified weight/normal/UV remap before it can be admitted.
mod controls;
mod native_controls;
#[cfg(test)]
mod tests;

pub use controls::{apply_control, descriptor_source_literal, parse_descriptor_source_literal, RigControl};
pub use native_controls::RigControls;

use super::preview::SourceRef;
use qualia_core_db::container_10d::{
    self, skeletal_container::read_skeletal_section,
    skeletal_section::{ClipRecord, Counts}, Container10dHeader, SectionDescriptor, SectionType,
    MAX_SECTION_COUNT,
};
use qualia_core_db::render::skeletal::{
    authoring::remap_skin_weights_into, Joint, Key, SkeletalPlayer, SkinWeight, Track,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Durable, source-owned binding. Digests are lowercase SHA-256 hex strings.
/// `resource_key` names an immutable selected HMC resource, not a path to open.
/// Local part and bind IDs remain scoped to this source; joint/clip IDs are
/// the exact SKL1 values and are not universal referent equivalences.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RigSourceDescriptor {
    pub schema: String,
    pub source_artifact_id: String,
    pub part_id: String,
    pub bind_id: String,
    pub resource_key: String,
    pub resource_sha256: String,
    pub bind_mesh_sha256: String,
    pub normal_uv_sha256: String,
    pub joint_ids: Vec<u64>,
    pub clip_ids: Vec<u64>,
    pub selected_clip_id: u64,
    pub seek_seconds: f64,
    pub joint_deltas: Vec<JointDeltaSource>,
    /// Output vertex -> SKL1 bind vertex. This first profile requires identity.
    pub vertex_lineage: Vec<u32>,
}

/// Serialized authored control, reconstructed into Core `Motor` on admission.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct JointDeltaSource {
    pub joint_id: u64,
    pub motor: [f64; 8],
}

/// Exact rest mesh visible to the native viewport. `indices` are local to the
/// selected part and match `.10d` triangle order, not scene-global indices.
pub struct BindMesh<'a> {
    pub positions: &'a [[f32; 3]],
    pub normals: &'a [[f32; 3]],
    pub uv0: &'a [[f32; 2]],
    pub indices: &'a [u32],
}

/// A native viewport range must be independently checked against its compiled
/// scene by the shared renderer before calling `write_mesh_animated_frames`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ViewportSpan {
    pub first_vertex: u32,
    pub vertex_count: usize,
}

pub struct BoundRig {
    source: SourceRef,
    descriptor: RigSourceDescriptor,
    span: ViewportSpan,
    player: SkeletalPlayer,
}

impl BoundRig {
    pub fn descriptor(&self) -> &RigSourceDescriptor { &self.descriptor }
    pub fn span(&self) -> ViewportSpan { self.span }
    pub fn source(&self) -> &SourceRef { &self.source }
    pub fn player(&self) -> &SkeletalPlayer { &self.player }
    pub fn player_mut(&mut self) -> &mut SkeletalPlayer { &mut self.player }
    pub fn check_current(&self) -> Result<(), String> { self.source.check_current() }
}

/// Hash exactly the decoded bind coordinates and local triangle order that the
/// player and viewport consume. Canonical bit encoding avoids display rounding.
pub fn bind_mesh_sha256(positions: &[[f32; 3]], indices: &[u32]) -> String {
    let mut h = Sha256::new();
    h.update((positions.len() as u64).to_le_bytes());
    h.update((indices.len() as u64).to_le_bytes());
    for point in positions { for value in point { h.update(value.to_bits().to_le_bytes()); } }
    for index in indices { h.update(index.to_le_bytes()); }
    format!("{:x}", h.finalize())
}

pub fn normal_uv_sha256(normals: &[[f32; 3]], uv0: &[[f32; 2]]) -> String {
    let mut h = Sha256::new();
    h.update((normals.len() as u64).to_le_bytes());
    h.update((uv0.len() as u64).to_le_bytes());
    for point in normals { for value in point { h.update(value.to_bits().to_le_bytes()); } }
    for point in uv0 { for value in point { h.update(value.to_bits().to_le_bytes()); } }
    format!("{:x}", h.finalize())
}

pub fn resource_sha256(bytes: &[u8]) -> String { format!("{:x}", Sha256::digest(bytes)) }

/// Conservative peak working-set preflight for this cold admission pass.
/// Includes borrowed selected resource, decoded mesh, two weight vectors,
/// section table, descriptor lineage and Core player's documented estimate.
/// Caller supplies the limit; this is not a production-master quality cap.
fn cold_budget_required(resource_len: usize, counts: Counts, indices: usize, lineage: usize) -> Option<(usize, usize)> {
    let n = counts.vertices;
    let player = std::mem::size_of::<SkeletalPlayer>()
        .checked_add(n.checked_mul(120)?)?
        .checked_add(indices.checked_mul(4)?)?
        .checked_add(counts.joints.checked_mul(
            std::mem::size_of::<Joint>() + 8 + 8 * 64,
        )?)?
        .checked_add(counts.clips.checked_mul(std::mem::size_of::<ClipRecord>())?)?
        .checked_add(counts.tracks.checked_mul(std::mem::size_of::<Track>())?)?
        .checked_add(counts.keys.checked_mul(std::mem::size_of::<Key>())?)?;
    let decoded_mesh = n.checked_mul(std::mem::size_of::<[f32; 3]>())?
        .checked_add(indices.checked_div(3)?.checked_mul(std::mem::size_of::<[u32; 3]>())?)?;
    let skin_work = n.checked_mul(std::mem::size_of::<SkinWeight>())?.checked_mul(2)?;
    let total = resource_len
        .checked_add(player)?
        .checked_add(decoded_mesh)?
        .checked_add(skin_work)?
        .checked_add(lineage.checked_mul(std::mem::size_of::<u32>())?)?
        .checked_add((MAX_SECTION_COUNT as usize).checked_mul(std::mem::size_of::<SectionDescriptor>())?)?
        .checked_add(4096)?;
    Some((total, player))
}

/// This operation does not read resources itself: the authorized HMC resolver
/// must supply the bytes for exactly `resource_key` and verify that key's
/// rights/selection. The process-local source stamp refuses stale publication.
pub fn admit_bound_rig(
    source: SourceRef,
    descriptor: RigSourceDescriptor,
    selected_resource_key: &str,
    resource_bytes: &[u8],
    mesh: BindMesh<'_>,
    span: ViewportSpan,
    resident_budget: usize,
) -> Result<BoundRig, String> {
    source.check_current()?;
    if descriptor.schema != "poet.bound-rig/v1"
        || descriptor.source_artifact_id != source.artifact_id
        || descriptor.part_id.is_empty()
        || descriptor.bind_id.is_empty()
        || descriptor.resource_key.is_empty()
        || descriptor.resource_key != selected_resource_key
        || descriptor.resource_sha256 != resource_sha256(resource_bytes)
    { return Err("selected rig source/resource binding does not match".into()); }
    let section_table_work = (MAX_SECTION_COUNT as usize)
        .checked_mul(std::mem::size_of::<SectionDescriptor>())
        .and_then(|bytes| bytes.checked_add(512))
        .ok_or("rig section table budget overflow")?;
    if resource_bytes.len().checked_add(section_table_work)
        .is_none_or(|bytes| bytes > resident_budget)
    {
        return Err("selected rig resource and section table exceed caller working budget".into());
    }

    let section = read_skeletal_section(resource_bytes)
        .map_err(|e| format!("sealed SKL1 resource refused: {e:?}"))?;
    let counts = section.counts();
    if mesh.positions.len() != counts.vertices || mesh.normals.len() != counts.vertices
        || mesh.uv0.len() != counts.vertices || span.vertex_count != counts.vertices
        || mesh.indices.is_empty() || mesh.indices.len() % 3 != 0
        || span.first_vertex.checked_add(span.vertex_count as u32).is_none()
        || descriptor.vertex_lineage.len() != counts.vertices
        || descriptor.vertex_lineage.iter().enumerate().any(|(i, v)| *v as usize != i)
    { return Err("rig vertex count or exact lineage mismatch; remeshing requires qualified remap".into()); }
    if descriptor.joint_ids.len() != counts.joints
        || descriptor.clip_ids.len() != counts.clips
        || descriptor.joint_ids.iter().enumerate().any(|(i, id)| section.joint_id(i) != Some(*id))
        || descriptor.clip_ids.iter().enumerate().any(|(i, id)| section.clip(i).map(|c| c.id) != Some(*id))
    { return Err("rig joint/clip identifiers differ from selected SKL1".into()); }
    if !descriptor.clip_ids.contains(&descriptor.selected_clip_id)
        || !descriptor.seek_seconds.is_finite()
        || descriptor.seek_seconds.abs() > 1e12
        || descriptor.joint_deltas.len() > counts.joints
    { return Err("rig source clip, seek or control count invalid".into()); }

    let (cold, player_estimate) = cold_budget_required(
        resource_bytes.len(), counts, mesh.indices.len(), descriptor.vertex_lineage.len(),
    ).ok_or("rig cold admission size overflow")?;
    if cold > resident_budget {
        return Err("rig cold admission exceeds caller working budget".into());
    }

    let header = Container10dHeader::parse(resource_bytes)
        .map_err(|e| format!("rig container header: {e:?}"))?;
    let sections = container_10d::parse_section_table(resource_bytes, &header)
        .map_err(|e| format!("rig section table: {e:?}"))?;
    let mesh_section = sections.iter().find(|s| s.typ() == Some(SectionType::QuantizedMesh))
        .ok_or("rig resource lacks QuantizedMesh")?;
    let start = mesh_section.byte_offset as usize;
    let end = start.checked_add(mesh_section.byte_length as usize).ok_or("mesh range overflow")?;
    let encoded_mesh = resource_bytes.get(start..end).ok_or("mesh range outside resource")?;
    let decoded = container_10d::decode_mesh_section(encoded_mesh)
        .map_err(|e| format!("rig bind mesh refused: {e:?}"))?;
    if mesh.indices.len() != decoded.triangles.len() * 3
        || mesh.positions != decoded.positions.as_slice()
        || mesh.indices.chunks_exact(3).zip(decoded.triangles.iter())
            .any(|(actual, expected)| actual != expected.as_slice())
        || descriptor.bind_mesh_sha256 != bind_mesh_sha256(mesh.positions, mesh.indices)
        || descriptor.normal_uv_sha256 != normal_uv_sha256(mesh.normals, mesh.uv0)
    { return Err("bind position, topology, normal or UV lineage differs from selected resource".into()); }

    // Route even identity correspondence through Core's checked weight remapper.
    // The section is the sole weight authority; no independent skin codec exists.
    let source_weights: Vec<SkinWeight> = (0..counts.vertices)
        .map(|i| section.skin(i).expect("validated section count"))
        .collect();
    let mut mapped = source_weights.clone();
    remap_skin_weights_into(&source_weights, &descriptor.vertex_lineage, &mut mapped)
        .map_err(|e| format!("skin lineage refused: {e:?}"))?;
    if mapped.iter().zip(source_weights.iter()).any(|(a, b)| a.joints != b.joints || a.weights != b.weights) {
        return Err("skin weights changed during exact lineage mapping".into());
    }

    let mut player = SkeletalPlayer::from_section_with_budget(
        &section, mesh.positions, mesh.normals, mesh.uv0, mesh.indices,
        resident_budget - (cold - player_estimate),
    ).map_err(|e| format!("rig player admission refused: {e:?}"))?;
    controls::restore_source_controls(&descriptor, &mut player)?;
    source.check_current()?;
    Ok(BoundRig { source, descriptor, span, player })
}
