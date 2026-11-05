//! Bounded cold native raster editing; canonical Core/geospatial codecs only.
use super::*;
use qualia_core_db::q42::m64_matter::{M64PayloadInterval, M64ReadLimits, M64Reader};
use qualia_geospatial::{
    raster_m64::{
        compile_raster_to_m64_v3, decode_raster_from_m64_v3, RasterM64Limits, RasterM64Policy,
    },
    ElevationRaster,
};
use std::sync::atomic::{AtomicBool, Ordering};
mod terrain;
pub use terrain::GeoRasterBrushRequest;
mod producer;
pub use producer::GeoRasterCreateRequest;
mod source;
pub use source::{SourceAnnotation, SourceDeclaration, SourceQualification, SourceStatus};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GeoRasterEditRequest {
    pub resource: ResourceInspectionRequest,
    /// Absolute origin declaration, never added to the integer sample values.
    pub origin_elevation_mm: i32,
    pub cell_size_mm: u32,
    pub sample_x: usize,
    pub sample_y: usize,
    /// Absolute mm; equal to the persisted nodata sentinel explicitly means nodata.
    pub sample_mm: i32,
}
impl GeoRasterEditRequest {
    pub fn validate_bounds(&self) -> Result<(), String> {
        self.resource.validate_bounds()?;
        if self.cell_size_mm == 0 {
            return Err("Raster cell size must be positive integer millimetres".into());
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug)]
pub struct GeoRasterEditLimits {
    pub read: M64ReadLimits,
    /// Sum of sample and output capacities. HMC load/save has separate host limits.
    pub max_working_bytes: usize,
}
pub fn ui_georaster_edit_limits() -> GeoRasterEditLimits {
    GeoRasterEditLimits {
        read: ui_inspection_limits(),
        max_working_bytes: 32 * 1024 * 1024,
    }
}

fn live(cancelled: &AtomicBool) -> Result<(), String> {
    if cancelled.load(Ordering::Relaxed) {
        Err("Raster edit cancelled before package mutation".into())
    } else {
        Ok(())
    }
}

/// Caller-owned scratch/output is used by each codec. Only this cold adapter
/// allocates bounded, fallible sample/output buffers; no per-record heap work.
/// Existing single-cell edit over the shared canonical payload mutation owner.
pub(super) fn replace_payload(
    bytes: &[u8],
    request: &GeoRasterEditRequest,
    limits: GeoRasterEditLimits,
    cancelled: &AtomicBool,
    scratch: &mut [M64PayloadInterval],
) -> Result<Vec<u8>, String> {
    request.validate_bounds()?;
    mutate_elevation_payload(bytes, limits, cancelled, scratch, |samples, frame, cell, meta| {
        let shape = meta.shape().map_err(|e| format!("Raster shape refused: {e:?}"))?;
        if request.sample_x >= shape.width || request.sample_y >= shape.height {
            return Err("Raster edit coordinate lies outside persisted source extent; padding is not source".into());
        }
        samples[request.sample_y * shape.width + request.sample_x] = request.sample_mm;
        frame.origin_elevation_mm = request.origin_elevation_mm;
        *cell = request.cell_size_mm;
        Ok(())
    })
}

fn mutate_elevation_payload<F>(
    bytes: &[u8],
    limits: GeoRasterEditLimits,
    cancelled: &AtomicBool,
    scratch: &mut [M64PayloadInterval],
    mutate: F,
) -> Result<Vec<u8>, String>
where
    F: FnOnce(&mut [i32], &mut qualia_geospatial::GeodeticFrame, &mut u32,
        &qualia_core_db::q42::m64_matter::M64GeoRasterMetadata) -> Result<(), String>,
{
    live(cancelled)?;
    let reader = M64Reader::parse(bytes, limits.read, scratch)
        .map_err(|e| format!("Canonical M64 edit admission refused: {e}"))?;
    let meta = reader.georaster().ok_or("Legacy M64 v1/v2 lacks required source metadata; automatic upgrade refused. Supply explicit source values through a separately qualified producer.")?;
    let shape = meta
        .shape()
        .map_err(|e| format!("Raster shape refused: {e:?}"))?;
    let count = shape
        .width
        .checked_mul(shape.height)
        .ok_or("Raster sample count overflow")?;
    let working = count
        .checked_mul(4)
        .and_then(|n| n.checked_add(shape.file_bytes))
        .ok_or("Raster working byte budget overflow")?;
    if working > limits.max_working_bytes {
        return Err(format!(
            "Raster cold working budget requires {working}; limit {}. No package mutation",
            limits.max_working_bytes
        ));
    }
    let mut samples = Vec::new();
    samples
        .try_reserve_exact(count)
        .map_err(|_| "Raster sample allocation refused")?;
    samples.resize(count, 0);
    let (_, mut frame) = decode_raster_from_m64_v3(&reader, cancelled, &mut samples)
        .map_err(|e| format!("Canonical raster recovery refused: {e:?}"))?;
    let policy = RasterM64Policy {
        rows: meta.rows,
        edges: meta.edges,
        nodata_sentinel_mm: meta.nodata_sentinel_mm,
    };
    let lut_epoch = reader
        .chunk(0)
        .ok_or("Raster contains no source chunk")?
        .directory_entry()
        .lut_bank_epoch_id;
    let writer_limits = RasterM64Limits {
        max_output_bytes: limits.read.max_file_bytes,
        max_chunks: limits.read.max_chunks,
        max_records: usize::try_from(limits.read.max_records)
            .map_err(|_| "Record budget overflow")?,
    };
    let mut out = Vec::new();
    out.try_reserve_exact(shape.file_bytes)
        .map_err(|_| "Raster output allocation refused")?;
    out.resize(shape.file_bytes, 0);
    let original = ElevationRaster {
        width: shape.width,
        height: shape.height,
        cell_size_mm: meta.cell_size_mm,
        elevations_mm: &samples,
    };
    let written = compile_raster_to_m64_v3(
        &original,
        &frame,
        lut_epoch,
        policy,
        writer_limits,
        cancelled,
        &mut out,
    )
    .map_err(|e| format!("Canonical original raster regeneration refused: {e:?}"))?;
    // Refuse richer strata, unknown fields, aliases or noncanonical producer
    // layouts rather than erasing them with the elevation-only writer.
    if out[..written] != *bytes {
        return Err("Selected M64 contains fields/layout not preserved by the canonical elevation-only producer; edit refused without losing content".into());
    }
    let mut cell_size_mm = meta.cell_size_mm;
    mutate(&mut samples, &mut frame, &mut cell_size_mm, meta)?;
    let edited = ElevationRaster {
        width: shape.width,
        height: shape.height,
        cell_size_mm,
        elevations_mm: &samples,
    };
    let written = compile_raster_to_m64_v3(
        &edited,
        &frame,
        lut_epoch,
        policy,
        writer_limits,
        cancelled,
        &mut out,
    )
    .map_err(|e| format!("Canonical raster edit compilation refused: {e:?}"))?;
    out.truncate(written);
    // Verify the emitted bytes using the same public checked reader before save.
    M64Reader::parse(&out, limits.read, scratch)
        .map_err(|e| format!("Canonical edited raster verification refused: {e}"))?;
    live(cancelled)?;
    Ok(out)
}

impl CheckpointService {
    /// The outer AuthorizedEditorHost must retain write_package authorization.
    /// No package contents, request field or digest grants write authority.
    pub fn edit_georaster(
        &self,
        request: GeoRasterEditRequest,
        expected: &StoreStamp,
        mut limits: GeoRasterEditLimits,
        cancelled: &AtomicBool,
        scratch: &mut [M64PayloadInterval],
    ) -> Result<SaveReceipt, String> {
        request.validate_bounds()?;
        live(cancelled)?;
        let selected = &request.resource;
        let expected_file = ExpectedFile::Sha256(selected.package_digest);
        if !studio_store::artifact_stamp_is_current(&self.binding.source_id, expected) {
            return Err("Raster edit source changed before read; no write".into());
        }
        self.preflight_resource_edit(&selected.filename, expected_file)?;
        limits.read.max_file_bytes = limits.read.max_file_bytes.min(self.limits.file_bytes);
        let checkpoint = core::load(&self.path(&selected.filename)?, self.limits)?;
        self.matches(checkpoint.artifact())?;
        if checkpoint.file_digest() != Some(selected.package_digest) {
            return Err("Raster package digest changed before read; refresh inventory".into());
        }
        let resource = checkpoint
            .resources()
            .iter()
            .find(|r| r.key == selected.key)
            .ok_or("Selected raster resource is absent")?;
        if resource.summary().digest != selected.resource_digest {
            return Err("Selected raster digest changed; refresh inventory".into());
        }
        source::ensure_editable_annotation(resource.meta.as_deref())?;
        let bytes = replace_payload(&resource.bytes, &request, limits, cancelled, scratch)?;
        drop(checkpoint);
        live(cancelled)?;
        self.save_with_resource_edits(
            expected,
            &selected.filename,
            expected_file,
            Some(selected.package_digest),
            vec![],
            vec![ResourceEdit::ReplaceBytes {
                key: selected.key.clone(),
                expected: selected.resource_digest,
                bytes,
            }],
        )
        .map(|saved| saved.receipt)
    }
}

#[cfg(test)]
mod tests;
