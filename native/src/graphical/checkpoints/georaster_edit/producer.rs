//! Explicit-source developmental v3 creation into an existing guarded HMC.
use super::*;
use qualia_core_db::q42::m64_matter::M64GeoRasterMetadata;
use qualia_geospatial::GeodeticFrame;
use sha2::{Digest, Sha256};

#[derive(Clone, Debug, PartialEq)]
pub struct GeoRasterCreateRequest {
    pub filename: String,
    pub package_digest: [u8; 32],
    pub key: String,
    pub metadata: M64GeoRasterMetadata,
    pub lut_bank_epoch: u16,
    pub samples_mm: Vec<i32>,
    pub source: SourceDeclaration,
}
impl GeoRasterCreateRequest {
    /// Native queue profile, not a format or master-quality limit.
    pub fn validate_bounds(&self) -> Result<(), String> {
        self.source.validate()?;
        if self.filename.is_empty()
            || self.filename.len() > 128
            || self.filename.contains('\0')
            || self.key.is_empty()
            || self.key.len() > 4096
            || self.key.contains('\0')
            || self.key == "authoring/native-source.json"
            || self.samples_mm.len() > UI_INSPECTION_CHUNKS * 4096
        {
            return Err(
                "New raster request exceeds native queue bounds or uses a reserved key".into(),
            );
        }
        let shape = self
            .metadata
            .shape()
            .map_err(|e| format!("Explicit raster metadata refused: {e:?}"))?;
        if shape.width.checked_mul(shape.height) != Some(self.samples_mm.len()) {
            return Err("Explicit raster extent and source sample count differ".into());
        }
        Ok(())
    }
}

pub(super) fn create_payload(
    request: &GeoRasterCreateRequest,
    limits: GeoRasterEditLimits,
    cancelled: &AtomicBool,
    scratch: &mut [M64PayloadInterval],
) -> Result<Vec<u8>, String> {
    request.validate_bounds()?;
    live(cancelled)?;
    let meta = request.metadata;
    let shape = meta
        .shape()
        .map_err(|e| format!("Explicit raster shape refused: {e:?}"))?;
    let working = request
        .samples_mm
        .len()
        .checked_mul(4)
        .and_then(|n| n.checked_add(shape.file_bytes))
        .ok_or("Raster creation working budget overflow")?;
    if working > limits.max_working_bytes
        || shape.file_bytes > limits.read.max_file_bytes
        || shape.chunks > limits.read.max_chunks
        || shape.chunks > scratch.len()
        || shape.records as u64 > limits.read.max_records
    {
        return Err("Raster creation exceeds caller byte/sample/chunk/scratch budget; unassessed under this profile".into());
    }
    let raster = ElevationRaster {
        width: shape.width,
        height: shape.height,
        cell_size_mm: meta.cell_size_mm,
        elevations_mm: &request.samples_mm,
    };
    let frame = GeodeticFrame {
        srid: meta.srid,
        vertical_datum: meta.vertical_datum,
        epoch_year: meta.epoch_year,
        origin_lat_deg: meta.origin_lat_deg,
        origin_lon_deg: meta.origin_lon_deg,
        origin_elevation_mm: meta.origin_elevation_mm,
    };
    let policy = RasterM64Policy {
        rows: meta.rows,
        edges: meta.edges,
        nodata_sentinel_mm: meta.nodata_sentinel_mm,
    };
    let writer_limits = RasterM64Limits {
        max_output_bytes: limits.read.max_file_bytes,
        max_chunks: limits.read.max_chunks,
        max_records: usize::try_from(limits.read.max_records)
            .map_err(|_| "Record budget overflow")?,
    };
    let mut out = Vec::new();
    out.try_reserve_exact(shape.file_bytes)
        .map_err(|_| "Raster creation output allocation refused")?;
    out.resize(shape.file_bytes, 0);
    let written = compile_raster_to_m64_v3(
        &raster,
        &frame,
        request.lut_bank_epoch,
        policy,
        writer_limits,
        cancelled,
        &mut out,
    )
    .map_err(|e| format!("Canonical explicit-source raster compilation refused: {e:?}"))?;
    out.truncate(written);
    let reader = M64Reader::parse(&out, limits.read, scratch)
        .map_err(|e| format!("Canonical produced raster verification refused: {e}"))?;
    if reader.georaster() != Some(&meta) {
        return Err("Canonical produced metadata differs from explicit source declarations".into());
    }
    live(cancelled)?;
    Ok(out)
}

impl CheckpointService {
    /// Outer host holds the existing write_package authority/audit lifecycle.
    pub fn create_georaster(
        &self,
        request: GeoRasterCreateRequest,
        expected: &StoreStamp,
        mut limits: GeoRasterEditLimits,
        cancelled: &AtomicBool,
        scratch: &mut [M64PayloadInterval],
    ) -> Result<SaveReceipt, String> {
        request.validate_bounds()?;
        live(cancelled)?;
        if !studio_store::artifact_stamp_is_current(&self.binding.source_id, expected) {
            return Err("Raster producer source changed before read; no write".into());
        }
        let file = ExpectedFile::Sha256(request.package_digest);
        self.preflight_resource_edit(&request.filename, file)?;
        let inventory = self.inventory(&request.filename)?;
        if inventory.digest != request.package_digest
            || inventory.resources.iter().any(|r| r.key == request.key)
        {
            return Err(
                "Raster producer package changed or target key exists; refresh inventory".into(),
            );
        }
        limits.read.max_file_bytes = limits.read.max_file_bytes.min(self.limits.file_bytes);
        let bytes = create_payload(&request, limits, cancelled, scratch)?;
        let meta =
            super::source::encode_annotation(&request.source, Sha256::digest(&bytes).into())?;
        live(cancelled)?;
        self.save_with_resource_edits(
            expected,
            &request.filename,
            file,
            Some(request.package_digest),
            vec![],
            vec![ResourceEdit::Add(ResourceData {
                key: request.key,
                kind: "m64".into(),
                bytes,
                meta: Some(meta),
            })],
        )
        .map(|saved| saved.receipt)
    }
}
