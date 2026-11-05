//! Bounded local terrain brush adapter over the existing canonical M64 edit owner.
//! No codec, datum conversion, source format or package authority is introduced.
use super::*;
use poet_spatial_authoring::geo_studio::{CowTerrainTileGrid, StrokeSample};
use poet_spatial_authoring::geo_studio::cow_delta_undo::NeighborhoodSampling;
use qualia_core_db::q42::m64_matter::M64GeoRasterMetadata;

pub const MAX_BRUSH_WINDOW_SIDE: usize = 64;
pub const MAX_BRUSH_REQUEST_SAMPLES: usize = 64;

/// Local metre XY starts at (window_x,window_y) in persisted raster row order.
/// Height/flatten target remain absolute metres in the persisted vertical datum;
/// origin_elevation_mm is a declaration and is never added to source samples.
#[derive(Clone, Debug, PartialEq)]
pub struct GeoRasterBrushRequest {
    pub resource: ResourceInspectionRequest,
    pub window_x: usize,
    pub window_y: usize,
    pub window_side: usize,
    pub samples: Vec<StrokeSample>,
}

impl GeoRasterBrushRequest {
    pub fn validate_bounds(&self) -> Result<(), String> {
        self.resource.validate_bounds()?;
        if !(1..=MAX_BRUSH_WINDOW_SIDE).contains(&self.window_side)
            || !(1..=MAX_BRUSH_REQUEST_SAMPLES).contains(&self.samples.len())
        {
            return Err("Terrain brush exceeds the native square-window/sample profile".into());
        }
        self.window_x.checked_add(self.window_side).ok_or("Brush X extent overflow")?;
        self.window_y.checked_add(self.window_side).ok_or("Brush Y extent overflow")?;
        Ok(())
    }
}

fn millimetres(value: f32, nodata: Option<i32>) -> Result<i32, String> {
    if !value.is_finite() { return Err("Brush height is not finite".into()); }
    // Declared projection: nearest integer millimetre, ties away from zero.
    let rounded = (f64::from(value) * 1000.0).round();
    if rounded < i32::MIN as f64 || rounded > i32::MAX as f64 {
        return Err("Brush height is outside the existing i32 millimetre range".into());
    }
    let mm = rounded as i32;
    if nodata == Some(mm) {
        return Err("Brush result collides with persisted nodata; no nodata conversion is implicit".into());
    }
    Ok(mm)
}

fn brush_samples(
    samples_mm: &mut [i32],
    meta: &M64GeoRasterMetadata,
    request: &GeoRasterBrushRequest,
    brush_payload_bytes: usize,
    cancelled: &AtomicBool,
) -> Result<(), String> {
    let shape = meta.shape().map_err(|e| format!("Brush raster shape refused: {e:?}"))?;
    if request.window_x + request.window_side > shape.width || request.window_y + request.window_side > shape.height {
        return Err("Brush window exceeds persisted source extent; padding is not editable source".into());
    }
    let cell_m = (f64::from(meta.cell_size_mm) / 1000.0) as f32;
    if !cell_m.is_finite() || cell_m <= 0.0 {
        return Err("Persisted cell size cannot be represented by the local brush".into());
    }
    // Bound f32 local cell placement error to half a millimetre throughout this window.
    for offset in 0..request.window_side {
        let expected_m = offset as f64 * f64::from(meta.cell_size_mm) / 1000.0;
        let brush_m = f64::from(offset as f32 * cell_m);
        if (brush_m - expected_m).abs() > 0.0005 {
            return Err("Cell placement exceeds the admitted half-millimetre local projection; edit refused".into());
        }
    }
    let count = request.window_side * request.window_side;
    let mut projected = Vec::new();
    projected.try_reserve_exact(count).map_err(|_| "Brush source projection allocation refused")?;
    for y in 0..request.window_side {
        live(cancelled)?;
        for x in 0..request.window_side {
            let index = (request.window_y + y) * shape.width + request.window_x + x;
            let mm = samples_mm[index];
            if meta.nodata_sentinel_mm == Some(mm) {
                return Err("Selected brush window contains nodata; masked brush semantics are unqualified".into());
            }
            let value = (f64::from(mm) / 1000.0) as f32;
            if millimetres(value, meta.nodata_sentinel_mm)? != mm {
                return Err("Source millimetres do not round-trip through the local f32 brush; no precision loss is admitted".into());
            }
            projected.push(value);
        }
    }
    // This adapter saves the edited M64 raster, not the local stroke journal.
    // Opt new native edits into unbiased per-sample neighbourhood reads while
    // leaving the shared grid's legacy journal replay default unchanged.
    let mut grid = CowTerrainTileGrid::new_with_neighborhood_sampling(request.window_side, cell_m, brush_payload_bytes, NeighborhoodSampling::SnapshotV1)
        .map_err(|e| format!("Brush grid admission refused: {e}"))?;
    grid.load_tile(0, 0, &projected).map_err(|e| format!("Brush source import refused: {e}"))?;
    grid.begin_stroke().map_err(|e| format!("Brush preview refused: {e}"))?;
    for sample in &request.samples {
        live(cancelled)?;
        grid.apply_stroke_sample(*sample).map_err(|e| format!("Brush sample refused: {e}"))?;
    }
    grid.commit_stroke("native M64 brush preview").map_err(|e| format!("Brush local commit refused: {e}"))?;
    let tile = grid.tile_arc(0, 0).ok_or("Brush preview tile absent")?;
    let mut changed = 0usize;
    for y in 0..request.window_side {
        live(cancelled)?;
        for x in 0..request.window_side {
            let local = y * request.window_side + x;
            if tile[local].to_bits() == projected[local].to_bits() { continue; }
            let mm = millimetres(tile[local], meta.nodata_sentinel_mm)?;
            let index = (request.window_y + y) * shape.width + request.window_x + x;
            if samples_mm[index] != mm {
                samples_mm[index] = mm;
                changed += 1;
            }
        }
    }
    if changed == 0 { return Err("Brush produced no changed integer source samples; no package write".into()); }
    Ok(())
}

/// Detached payload preparation; ordinary errors never mutate input package bytes.
pub(super) fn brush_payload(
    bytes: &[u8], request: &GeoRasterBrushRequest, mut limits: GeoRasterEditLimits,
    cancelled: &AtomicBool, scratch: &mut [M64PayloadInterval],
) -> Result<Vec<u8>, String> {
    request.validate_bounds()?;
    let count = request.window_side * request.window_side;
    // One imported tile + one preview + conservative history-reference charge,
    // plus the local journal; projection Vec is separately reserved below.
    let brush_bytes = count.checked_mul(12)
        .and_then(|b| request.samples.len().checked_mul(core::mem::size_of::<StrokeSample>()).and_then(|s| b.checked_add(s)))
        .ok_or("Brush payload budget overflow")?;
    let extra = count.checked_mul(4).and_then(|b| b.checked_add(brush_bytes))
        .ok_or("Brush working budget overflow")?;
    limits.max_working_bytes = limits.max_working_bytes.checked_sub(extra)
        .ok_or("Brush buffers exceed the caller working-byte profile")?;
    mutate_elevation_payload(bytes, limits, cancelled, scratch, |samples, _frame, _cell, meta| {
        brush_samples(samples, meta, request, brush_bytes, cancelled)
    })
}

impl CheckpointService {
    /// The existing AuthorizedEditorHost must retain WritePackage authority.
    /// A digest, incoming stroke, annotation or package cannot grant that authority.
    pub(crate) fn brush_georaster(
        &self, request: GeoRasterBrushRequest, expected: &StoreStamp,
        mut limits: GeoRasterEditLimits, cancelled: &AtomicBool,
        scratch: &mut [M64PayloadInterval],
    ) -> Result<SaveReceipt, String> {
        request.validate_bounds()?;
        live(cancelled)?;
        let selected = &request.resource;
        let expected_file = ExpectedFile::Sha256(selected.package_digest);
        if !studio_store::artifact_stamp_is_current(&self.binding.source_id, expected) {
            return Err("Terrain brush source changed before read; no write".into());
        }
        self.preflight_resource_edit(&selected.filename, expected_file)?;
        limits.read.max_file_bytes = limits.read.max_file_bytes.min(self.limits.file_bytes);
        let checkpoint = core::load(&self.path(&selected.filename)?, self.limits)?;
        self.matches(checkpoint.artifact())?;
        if checkpoint.file_digest() != Some(selected.package_digest) {
            return Err("Terrain brush package digest changed; refresh inventory".into());
        }
        let resource = checkpoint.resources().iter().find(|r| r.key == selected.key)
            .ok_or("Selected terrain raster resource is absent")?;
        if resource.summary().digest != selected.resource_digest {
            return Err("Selected terrain raster digest changed; refresh inventory".into());
        }
        source::ensure_editable_annotation(resource.meta.as_deref())?;
        let bytes = brush_payload(&resource.bytes, &request, limits, cancelled, scratch)?;
        drop(checkpoint);
        live(cancelled)?;
        self.save_with_resource_edits(expected, &selected.filename, expected_file,
            Some(selected.package_digest), vec![], vec![ResourceEdit::ReplaceBytes {
                key: selected.key.clone(), expected: selected.resource_digest, bytes,
            }]).map(|saved| saved.receipt)
    }
}

#[cfg(test)]
mod tests;
