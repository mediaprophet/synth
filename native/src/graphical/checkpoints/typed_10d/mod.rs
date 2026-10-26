//! Selected-resource visual material derivatives; canonical Core codecs only.
use super::*;
use crate::graphical::preview::SourceRef;
use qualia_core_db::container_10d::{
    MaterialRecord, MeshFrameLayout, SubmeshRange, TypedFieldRecord,
};
use std::sync::atomic::{AtomicBool, Ordering};
mod payload;
mod service;

#[derive(Clone, Copy, Debug)]
pub struct Typed10dLimits {
    pub max_file_bytes: usize,
    pub max_sections: usize,
    pub max_materials: usize,
    pub max_ranges: usize,
    pub max_vertices: usize,
    pub max_working_bytes: usize,
}
pub fn ui_typed_10d_limits() -> Typed10dLimits {
    Typed10dLimits {
        max_file_bytes: 16 * 1024 * 1024,
        max_sections: 32,
        max_materials: 256,
        max_ranges: 4096,
        max_vertices: 200_000,
        max_working_bytes: 64 * 1024 * 1024,
    }
}
impl Default for Typed10dLimits {
    fn default() -> Self {
        ui_typed_10d_limits()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Typed10dEditRequest {
    pub resource: ResourceInspectionRequest,
    /// Current authored-source association guard, not a regeneration claim.
    pub source_digest: [u8; 32],
    pub material_id: u64,
    /// Linear visual factors. These do not assert physical material properties.
    pub base_color: [f32; 4],
    pub emissive: [f32; 3],
    pub metallic: f32,
    pub roughness: f32,
}
impl Typed10dEditRequest {
    pub fn validate_bounds(&self) -> Result<(), String> {
        self.resource.validate_bounds()?;
        if self.material_id == 0
            || self.source_digest == [0; 32]
            || self
                .base_color
                .iter()
                .chain([&self.metallic, &self.roughness])
                .any(|v| !v.is_finite() || !(0.0..=1.0).contains(v))
            || self
                .emissive
                .iter()
                .any(|v| !v.is_finite() || !(0.0..=65_504.0).contains(v))
        {
            return Err("Material requires an identified selection, current source digest and finite linear factors in range".into());
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Typed10dInspection {
    pub request: ResourceInspectionRequest,
    pub saved_source_revision: u32,
    pub source_digest: [u8; 32],
    pub version: u16,
    pub vertex_count: usize,
    pub index_count: u32,
    pub materials: Vec<MaterialRecord>,
    pub ranges: Vec<SubmeshRange>,
    pub fields: Vec<TypedFieldRecord>,
    pub frames: Option<MeshFrameLayout>,
    pub edit_blocker: Option<String>,
}
impl Typed10dInspection {
    pub fn matches_inventory(&self, filename: &str, inventory: &PackageInventory) -> bool {
        self.request.filename == filename
            && self.request.package_digest == inventory.digest
            && inventory
                .resources
                .iter()
                .any(|r| r.key == self.request.key && r.digest == self.request.resource_digest)
    }
}
fn live(cancelled: &AtomicBool) -> Result<(), String> {
    if cancelled.load(Ordering::Relaxed) {
        Err("Typed .10d edit cancelled before package mutation".into())
    } else {
        Ok(())
    }
}
fn buffer<T: Clone>(count: usize, value: T) -> Result<Vec<T>, String> {
    let mut out = Vec::new();
    out.try_reserve_exact(count)
        .map_err(|_| "Typed .10d bounded cold allocation refused")?;
    out.resize(count, value);
    Ok(out)
}

#[cfg(test)]
mod tests;
