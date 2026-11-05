//! Resource-backed mask morphology for the existing Poet paint source.
//!
//! This module changes only a cloned PaintRecipe and a cloned immutable HMC
//! resource snapshot. The caller submits the recipe through NativeProject's
//! existing authorized SourceRef/history operation before retaining the new
//! resource snapshot.

use super::{required_paint_version, DetailLimits, DetailRecipe, LayerMask, MaskChannel, PaintRecipe};
use crate::graphical::surfaces::TextureResources;
use qualia_core_db::render::{
    assets::TextureDependency,
    surface_authoring::encode_surface_png_into,
};
use raster_layer_mask_adapter::{modify_layer_mask, preflight_mask, AdapterLimits};
use raster_selection_morphology::{Limits as MorphologyLimits, Operation};
use sha2::{Digest, Sha256};

/// User-facing choice supported by the shared source-derived morphology operations.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MaskMorphologyOperation {
    Expand,
    Contract,
    Border,
}

impl MaskMorphologyOperation {
    fn adapter(self) -> Operation {
        match self {
            Self::Expand => Operation::Expand,
            Self::Contract => Operation::Contract,
            Self::Border => Operation::Border,
        }
    }
}

fn adapter_limits(side: usize, limits: DetailLimits) -> AdapterLimits {
    AdapterLimits {
        morphology: MorphologyLimits {
            max_pixels: side.saturating_mul(side),
            max_work_units: limits.pixel_visits,
            max_edt_scratch_bytes: limits.working_bytes,
            max_algorithm_payload_bytes: limits.working_bytes,
        },
        max_adapter_payload_bytes: limits.working_bytes,
        max_total_payload_bytes: limits.working_bytes,
        max_total_work_units: limits.pixel_visits,
    }
}

fn checked_rgba_len(side: usize) -> Result<usize, String> {
    side.checked_mul(side)
        .and_then(|pixels| pixels.checked_mul(4))
        .ok_or_else(|| "Mask morphology RGBA dimensions overflow".into())
}

fn reserve_vec<T>(count: usize, message: &'static str) -> Result<Vec<T>, String> {
    let mut out = Vec::new();
    out.try_reserve_exact(count).map_err(|_| message.to_owned())?;
    Ok(out)
}

/// Prepare a morphology result for one existing editable paint layer.
///
/// All dimensions and work/payload limits are admitted before mask extraction.
/// Resource input uses the existing integrity-checked HMC decoder; output uses
/// Core's RGBA PNG encoder and the existing immutable HMC dependency merger.
/// Neither `paint` nor `resources` is mutated on success or refusal.
pub(crate) fn prepare(
    paint: &PaintRecipe,
    layer_index: usize,
    operation: MaskMorphologyOperation,
    radius: f32,
    resources: &TextureResources,
    limits: DetailLimits,
) -> Result<(PaintRecipe, TextureResources, [u8; 32]), String> {
    // The controller passes only the strict recipe read from the current
    // accepted source. Revalidate the detached successor after constructing it.
    let layer = paint
        .layers
        .get(layer_index)
        .ok_or("Selected paint layer is unavailable")?;
    let side = usize::try_from(paint.side).map_err(|_| "Paint side is not addressable")?;
    let pixels = side
        .checked_mul(side)
        .ok_or("Mask morphology pixel count overflow")?;
    let rgba_len = checked_rgba_len(side)?;
    let adapter_operation = operation.adapter();
    let adapter_limits = adapter_limits(side, limits);

    // Structural, radius, pixel, work, scratch and morphology payload refusal
    // precedes the first decoded/channel/output allocation.
    let plan = preflight_mask(
        None,
        side,
        side,
        adapter_operation,
        radius,
        adapter_limits,
    )
    .map_err(|error| format!("Mask morphology preflight refused: {error:?}"))?;
    let png_capacity = rgba_len
        .checked_add(side)
        .and_then(|bytes| bytes.checked_add(4096))
        .ok_or("Mask PNG capacity overflow")?;
    let output_reservation = plan
        .total_payload_bytes_upper_bound
        .checked_add(pixels)
        .and_then(|bytes| bytes.checked_add(rgba_len))
        .and_then(|bytes| bytes.checked_add(png_capacity))
        .ok_or("Mask morphology aggregate payload overflow")?;
    if output_reservation > limits.working_bytes {
        return Err("Mask morphology aggregate logical payload exceeds detail working budget".into());
    }
    let decode_budget = limits.working_bytes - output_reservation;

    let mut channel = None::<Vec<u8>>;
    let mut output_channel = match &layer.mask {
        None => MaskChannel::Alpha,
        Some(LayerMask::Rectangle { x, y, width, height, coverage }) => {
            let right = x.checked_add(*width).filter(|right| *right <= paint.side)
                .ok_or("Rectangle mask exceeds paint extent")?;
            let bottom = y.checked_add(*height).filter(|bottom| *bottom <= paint.side)
                .ok_or("Rectangle mask exceeds paint extent")?;
            let mut values = reserve_vec(pixels, "Rectangle mask allocation refused")?;
            values.resize(pixels, 0);
            for row in *y..bottom {
                let start = (row as usize)
                    .checked_mul(side)
                    .and_then(|offset| offset.checked_add(*x as usize))
                    .ok_or("Rectangle mask offset overflow")?;
                let end = start.checked_add(*width as usize).ok_or("Rectangle mask end overflow")?;
                values[start..end].fill(*coverage);
            }
            channel = Some(values);
            MaskChannel::Alpha
        }
        Some(LayerMask::Resource { sha256, channel: selected }) => {
            let digest = crate::graphical::surfaces::binding::parse_digest(sha256)?;
            let encoded_len = resources.source_byte_len(digest)?;
            let snapshot_len = resources.snapshot_bytes();
            let admitted_decode = encoded_len
                .checked_add(snapshot_len)
                .and_then(|bytes| bytes.checked_add(rgba_len.checked_mul(2)?))
                .and_then(|bytes| bytes.checked_add(pixels))
                .ok_or("Mask decode working bound overflow")?;
            if admitted_decode > decode_budget {
                return Err("Mask decode plus morphology exceeds detail working budget".into());
            }
            let input = resources.mask_input_with_budget(digest, paint.side, decode_budget)?;
            if input.width != paint.side || input.height != paint.side || input.rgba.len() != rgba_len {
                return Err("Selected mask resource extent differs from paint recipe".into());
            }
            let channel_index = if *selected == MaskChannel::Red { 0 } else { 3 };
            let mut values = reserve_vec(pixels, "Resource mask channel allocation refused")?;
            values.extend(input.rgba.chunks_exact(4).map(|rgba| rgba[channel_index]));
            channel = Some(values);
            *selected
        }
    };

    let replacement = modify_layer_mask(
        channel.as_deref(),
        side,
        side,
        adapter_operation,
        radius,
        adapter_limits,
    )
    .map_err(|error| format!("Mask morphology refused: {error:?}"))?;
    if replacement.len() != pixels {
        return Err("Mask morphology returned a mismatched pixel count".into());
    }
    drop(channel);

    let mut rgba = reserve_vec(rgba_len, "Derived mask RGBA allocation refused")?;
    rgba.resize(rgba_len, 255);
    for (pixel, coverage) in rgba.chunks_exact_mut(4).zip(&replacement) {
        match output_channel {
            MaskChannel::Red => {
                pixel[0] = *coverage;
                pixel[1] = 0;
                pixel[2] = 0;
                pixel[3] = 255;
            }
            MaskChannel::Alpha => pixel[3] = *coverage,
        }
    }
    let mut png = reserve_vec(png_capacity, "Derived mask PNG allocation refused")?;
    png.resize(png_capacity, 0);
    let used = encode_surface_png_into(paint.side, &rgba, &mut png)
        .map_err(|error| format!("Derived mask PNG encode refused: {error:?}"))?;
    png.truncate(used);
    drop(rgba);
    drop(replacement);

    let digest: [u8; 32] = Sha256::digest(&png).into();
    let dependency = TextureDependency {
        digest,
        mime_type: "image/png".into(),
        bytes: png,
    };
    let candidate_resources = resources.with_dependencies_with_budget(
        &[dependency],
        limits.working_bytes,
    )?;

    let mut next = paint.clone();
    next.layers[layer_index].mask = Some(LayerMask::Resource {
        sha256: crate::graphical::surfaces::binding::digest_hex(&digest),
        channel: output_channel,
    });
    next.version = next.version.max(required_paint_version(&next));
    DetailRecipe::Paint(next.clone()).validate(limits)?;
    Ok((next, candidate_resources, digest))
}

