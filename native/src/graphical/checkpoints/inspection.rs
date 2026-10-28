//! Read-only selected-resource decoding inside the existing checkpoint owner.
use super::*;
use qualia_core_db::q42::m64_matter::{
    has_m64_magic, M64GeoRasterMetadata, M64Header, M64PayloadInterval, M64RasterSample,
    M64ReadError, M64ReadLimits, M64Reader, M64Verification,
};

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResourceInspectionRequest {
    pub filename: String,
    pub package_digest: [u8; 32],
    pub key: String,
    pub resource_digest: [u8; 32],
}
impl ResourceInspectionRequest {
    pub fn validate_bounds(&self) -> Result<(), String> {
        if self.filename.is_empty()
            || self.filename.len() > 128
            || self.filename.contains('\0')
            || self.key.is_empty()
            || self.key.len() > 4096
            || self.key.contains('\0')
        {
            return Err("Resource inspection request exceeds native filename/key bounds".into());
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct M64PayloadInspection {
    pub header: M64Header,
    pub verification: M64Verification,
    pub georaster: Option<M64GeoRasterMetadata>,
    pub first_source_sample: Option<M64RasterSample>,
}
#[derive(Clone, Debug, PartialEq)]
pub struct ResourceInspection {
    pub request: ResourceInspectionRequest,
    pub saved_source_revision: u32,
    pub payload: M64PayloadInspection,
}
impl ResourceInspection {
    pub fn matches_inventory(&self, filename: &str, inventory: &PackageInventory) -> bool {
        self.request.filename == filename
            && self.request.package_digest == inventory.digest
            && inventory.resources.iter().any(|resource| {
                resource.key == self.request.key && resource.digest == self.request.resource_digest
            })
    }
}

/// A small UI inspection profile, not a format limit or master-quality ceiling.
pub const UI_INSPECTION_CHUNKS: usize = 64;
pub fn ui_inspection_limits() -> M64ReadLimits {
    M64ReadLimits {
        max_file_bytes: 192 + UI_INSPECTION_CHUNKS * (64 + 4096 * 64),
        max_chunks: UI_INSPECTION_CHUNKS,
        max_records: (UI_INSPECTION_CHUNKS * 4096) as u64,
    }
}

pub(super) fn inspect_payload(
    bytes: &[u8],
    limits: M64ReadLimits,
    scratch: &mut [M64PayloadInterval],
) -> Result<M64PayloadInspection, String> {
    if bytes.len() > limits.max_file_bytes {
        return Err(format!("M64 inspection byte budget: requires {}; limit {}. Content unassessed under this inspection profile.", bytes.len(), limits.max_file_bytes));
    }
    if !has_m64_magic(bytes) {
        return Err(
            "Unsupported/unknown resource payload; left unassessed without reinterpretation".into(),
        );
    }
    // Preflight advertised chunk needs before allocating or scanning payloads.
    if let Some(raw) = bytes.get(48..56) {
        let required = u64::from_le_bytes(raw.try_into().unwrap());
        if required > limits.max_chunks as u64 || required > scratch.len() as u64 {
            return Err(format!("M64 inspection chunk/scratch budget: requires {required}; chunk limit {}; supplied scratch {}. Content unassessed under this inspection profile.", limits.max_chunks, scratch.len()));
        }
    }
    let reader = M64Reader::parse(bytes, limits, scratch).map_err(|error| {
        if matches!(error, M64ReadError::RecordLimit) {
            format!("M64 inspection record budget: requires more than {}; limit {}. Content unassessed under this inspection profile.", limits.max_records, limits.max_records)
        } else {
            format!("Canonical M64 inspection refused: {error}. No payload reinterpretation or content acceptance.")
        }
    })?;
    Ok(M64PayloadInspection {
        header: *reader.header(),
        verification: reader.verification(),
        georaster: reader.georaster().copied(),
        first_source_sample: reader.raster_sample(0, 0),
    })
}

impl CheckpointService {
    /// The caller must hold the current ReadSource guard throughout this call.
    /// The result is immutable bytes for the exact selected package/resource;
    /// it is not an SKL1 qualification or a grant to edit or publish anything.
    pub fn read_selected_resource(
        &self,
        request: ResourceInspectionRequest,
        expected: &crate::graphical::preview::SourceRef,
        max_resource_bytes: usize,
        cancelled: &std::sync::atomic::AtomicBool,
    ) -> Result<Vec<u8>, String> {
        let live = || if cancelled.load(std::sync::atomic::Ordering::Acquire) { Err("Rig resource read cancelled") } else { Ok(()) };
        live()?;
        request.validate_bounds()?;
        expected.check_current()?;
        if expected.artifact_id != self.binding.source_id {
            return Err("Selected resource belongs to another source".into());
        }
        if max_resource_bytes == 0 || max_resource_bytes > self.limits.file_bytes {
            return Err("Selected resource byte budget is invalid".into());
        }
        let (artifact, stamp) = studio_store::get_artifact_stamped(&expected.artifact_id)
            .ok_or("Selected resource source disappeared")?;
        if stamp != expected.stamp { return Err("Selected resource source changed".into()); }
        let digest = crate::graphical::cooperation::source_digest(
            &artifact, crate::graphical::cooperation::GatewayLimits::default())?;
        let path = self.path(&request.filename)?;
        let checkpoint = core::load(&path, self.limits)?;
        live()?;
        self.matches(checkpoint.artifact())?;
        if crate::graphical::cooperation::source_digest(
            checkpoint.artifact(), crate::graphical::cooperation::GatewayLimits::default())? != digest
        { return Err("Selected resource package source differs from current source; save/reopen a coherent base first".into()); }
        if checkpoint.file_digest() != Some(request.package_digest) {
            return Err("Selected resource package changed; refresh inventory".into());
        }
        self.check_release_input(&request.filename, request.package_digest)?;
        let selected = checkpoint.resources().iter().find(|r| r.key == request.key)
            .ok_or("Selected resource is absent; refresh inventory")?;
        if selected.kind != "10d" || selected.summary().digest != request.resource_digest {
            return Err("Selected resource kind or digest changed; refresh inventory".into());
        }
        if selected.bytes.len() > max_resource_bytes {
            return Err("Selected resource exceeds the caller byte budget".into());
        }
        let mut bytes = Vec::new();
        bytes.try_reserve_exact(selected.bytes.len()).map_err(|_| "Selected resource allocation refused")?;
        bytes.extend_from_slice(&selected.bytes);
        drop(checkpoint);
        expected.check_current()?;
        let current = core::load(&self.path(&request.filename)?, self.limits)?;
        live()?;
        self.matches(current.artifact())?;
        self.check_release_input(&request.filename, request.package_digest)?;
        if current.file_digest() != Some(request.package_digest) {
            return Err("Selected resource package changed during read; result withheld".into());
        }
        expected.check_current()?;
        live()?;
        Ok(bytes)
    }

    /// Caller retains the current ReadSource owner guard; this service supplies
    /// path/source/digest consistency, never authority from package contents.
    pub fn inspect_resource(
        &self,
        request: ResourceInspectionRequest,
        mut limits: M64ReadLimits,
        scratch: &mut [M64PayloadInterval],
    ) -> Result<ResourceInspection, String> {
        request.validate_bounds()?;
        limits.max_file_bytes = limits.max_file_bytes.min(self.limits.file_bytes);
        let checkpoint = core::load(&self.path(&request.filename)?, self.limits)?;
        self.matches(checkpoint.artifact())?;
        if checkpoint.file_digest() != Some(request.package_digest) {
            return Err(
                "Inspected package digest changed; refresh inventory before decoding".into(),
            );
        }
        self.check_release_input(&request.filename, request.package_digest)?;
        let selected = checkpoint
            .resources()
            .iter()
            .find(|resource| resource.key == request.key)
            .ok_or("Selected resource is absent; refresh inventory")?;
        if selected.summary().digest != request.resource_digest {
            return Err(
                "Selected resource digest changed; refresh inventory before decoding".into(),
            );
        }
        let payload = inspect_payload(&selected.bytes, limits, scratch)?;
        let saved_source_revision = checkpoint.artifact().lamport_clock;
        drop(checkpoint);
        // Do not cache a second carrier. Reuse canonical bounded load to refuse
        // a package swapped during scanning, before returning derived details.
        let current = core::load(&self.path(&request.filename)?, self.limits)?;
        self.matches(current.artifact())?;
        self.check_release_input(&request.filename, request.package_digest)?;
        if current.file_digest() != Some(request.package_digest) {
            return Err("Package changed during inspection; result withheld".into());
        }
        Ok(ResourceInspection {
            request,
            saved_source_revision,
            payload,
        })
    }
}

#[cfg(test)]
#[path = "inspection_tests.rs"]
mod tests;
