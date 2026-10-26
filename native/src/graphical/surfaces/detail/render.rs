use super::{DetailLimits, DetailRecipe, LayerBlendMode, LayerMask, MaskChannel, TextureEffect};
use crate::graphical::surfaces::{binding::digest_hex, TextureRole};
use qualia_core_db::{
    hypermedia_authoring::image::{
        apply_filter, try_apply_image_filter, BlendMode, BrushStroke, ImageDocument, ImageFilter,
        ImageFilterOp, ImageLayer, ImageLimits,
    },
    render::{
        assets::TextureDependency,
        surface_authoring::{encode_surface_png_into, generate_surface_maps_into, SurfaceRecipe},
        texture_decode::TextureColorSpace,
    },
};
use sha2::{Digest, Sha256};

/// Pixels decoded by the existing authorized resource owner, never a source path.
pub struct DetailBase<'a> {
    pub digest: [u8; 32],
    pub width: u32,
    pub height: u32,
    pub rgba: &'a [u8],
    pub colour_space: TextureColorSpace,
}
/// Decoded immutable HMC mask. Red/alpha are raw encoded bytes, with no colour transfer claim.
pub struct DetailMaskBase<'a> {
    pub digest: [u8; 32],
    pub width: u32,
    pub height: u32,
    pub rgba: &'a [u8],
}
#[derive(Debug)]
pub struct DetailOutput {
    pub overrides: Vec<(TextureRole, [u8; 32])>,
    pub dependencies: Vec<TextureDependency>,
    pub base_digest: Option<[u8; 32]>,
    /// Source lineage witness, never authority or permission.
    pub recipe_sha256: [u8; 32],
}
fn encoded(side: u32, pixels: &[u8]) -> Result<TextureDependency, String> {
    let mut png = vec![0; pixels.len() + side as usize + 4096];
    let n = encode_surface_png_into(side, pixels, &mut png)
        .map_err(|e| format!("Texture detail PNG: {e:?}"))?;
    png.truncate(n);
    Ok(TextureDependency {
        digest: Sha256::digest(&png).into(),
        mime_type: "image/png".into(),
        bytes: png,
    })
}
/// Initial imported profile is opaque, sRGB base colour, exact square extent.
/// Canonical paint/composite operates on encoded samples, not linear-light RGB.
pub fn render_detail(
    recipe: &DetailRecipe,
    base: Option<DetailBase<'_>>,
    limits: DetailLimits,
) -> Result<DetailOutput, String> {
    render_detail_with_masks(recipe, base, &[], limits)
}
pub fn render_detail_with_masks(
    recipe: &DetailRecipe,
    base: Option<DetailBase<'_>>,
    masks: &[DetailMaskBase<'_>],
    limits: DetailLimits,
) -> Result<DetailOutput, String> {
    let cost = recipe.budget_plan(limits)?;
    let side = recipe.side();
    let len = side as usize * side as usize * 4;
    let pixels_per_layer = side as usize * side as usize;
    let core_limits = ImageLimits {
        max_pixels: pixels_per_layer,
        max_work: limits.pixel_visits,
        max_bytes: limits.working_bytes,
    };
    let mut maps = Vec::new();
    let base_digest = match recipe {
        DetailRecipe::Paint(p) => {
            let b = base.ok_or("Imported detail needs authorized original base pixels")?;
            if digest_hex(&b.digest) != p.base_sha256
                || b.width != side
                || b.height != side
                || b.rgba.len() != len
                || b.colour_space != TextureColorSpace::Srgb
                || b.rgba.chunks_exact(4).any(|p| p[3] != 255)
            {
                return Err(
                    "Imported detail base digest/extent/sRGB/opaque profile mismatch".into(),
                );
            }
            let required_masks: std::collections::BTreeSet<_> = p
                .layers
                .iter()
                .filter_map(|l| match &l.mask {
                    Some(LayerMask::Resource { sha256, .. }) => Some(sha256.as_str()),
                    _ => None,
                })
                .collect();
            if masks.len() != required_masks.len()
                || masks
                    .iter()
                    .any(|m| !required_masks.contains(digest_hex(&m.digest).as_str()))
            {
                return Err("Paint mask inputs differ from referenced resources".into());
            }
            for l in &p.layers {
                if let Some(LayerMask::Resource { sha256, .. }) = &l.mask {
                    let required = super::super::binding::parse_digest(sha256)?;
                    let found = masks
                        .iter()
                        .find(|m| m.digest == required)
                        .ok_or("Referenced immutable paint mask resource unavailable")?;
                    if found.width != side || found.height != side || found.rgba.len() != len {
                        return Err("Paint mask decoded extent/storage differs from layer".into());
                    }
                }
            }
            if cost
                .working_bytes
                .checked_add(
                    required_masks
                        .len()
                        .checked_mul(len)
                        .ok_or("Paint mask decoded size overflow")?,
                )
                .is_none_or(|n| n > limits.working_bytes)
            {
                return Err("Paint mask decoded working admission exceeded".into());
            }
            let mut document = ImageDocument::new("poet-detail", side, side);
            let original = document.add_layer("Original immutable imported base");
            for (dst, src) in original.pixels.iter_mut().zip(b.rgba.chunks_exact(4)) {
                dst.copy_from_slice(src);
            }
            for l in &p.layers {
                let layer = document.add_layer(&l.name);
                layer.opacity = l.opacity;
                layer.visible = l.visible;
                layer.blend_mode = match l.blend_mode {
                    LayerBlendMode::Normal => BlendMode::Normal,
                    LayerBlendMode::Multiply => BlendMode::Multiply,
                    LayerBlendMode::Screen => BlendMode::Screen,
                    LayerBlendMode::Overlay => BlendMode::Overlay,
                    LayerBlendMode::SoftLight => BlendMode::SoftLight,
                    LayerBlendMode::HardLight => BlendMode::HardLight,
                    LayerBlendMode::ColorDodge => BlendMode::ColorDodge,
                    LayerBlendMode::ColorBurn => BlendMode::ColorBurn,
                    LayerBlendMode::Difference => BlendMode::Difference,
                    LayerBlendMode::Exclusion => BlendMode::Exclusion,
                };
                layer.offset_x = l.offset[0];
                layer.offset_y = l.offset[1];
                layer.mask = match &l.mask {
                    None => None,
                    Some(LayerMask::Rectangle {
                        x,
                        y,
                        width,
                        height,
                        coverage,
                    }) => {
                        let mut dense = vec![0u8; pixels_per_layer];
                        for row in *y..(*y + *height) {
                            for col in *x..(*x + *width) {
                                dense[row as usize * side as usize + col as usize] = *coverage;
                            }
                        }
                        Some(dense)
                    }
                    Some(LayerMask::Resource { sha256, channel }) => {
                        let digest = super::super::binding::parse_digest(sha256)?;
                        let rgba = masks
                            .iter()
                            .find(|m| m.digest == digest)
                            .ok_or("Paint mask unavailable")?
                            .rgba;
                        let index = if *channel == MaskChannel::Red { 0 } else { 3 };
                        Some(rgba.chunks_exact(4).map(|p| p[index]).collect())
                    }
                };
                // Replay hidden layers too, preserving authored operation semantics.
                for s in &l.strokes {
                    let mut stroke = BrushStroke::new(s.size, s.colour);
                    stroke.opacity = s.opacity;
                    stroke.hardness = s.hardness;
                    for point in &s.points {
                        stroke.add_point(point[0], point[1]);
                    }
                    stroke.apply_to(layer);
                }
            }
            let pixels = document
                .try_composite(core_limits)
                .map_err(|e| format!("Canonical paint composite: {e:?}"))?;
            let rgba: Vec<u8> = if p.effects.is_empty() {
                pixels.iter().flatten().copied().collect()
            } else {
                // One temporary layer, never attached to the immutable document/source.
                let mut filtered =
                    ImageLayer::new("poet-effects", "Post-composite effects", side, side);
                filtered.pixels.copy_from_slice(&pixels);
                for step in &p.effects {
                    if !step.enabled {
                        continue;
                    }
                    match step.effect {
                        TextureEffect::Grayscale => {
                            apply_filter(&mut filtered, ImageFilter::Grayscale, 1.0)
                        }
                        TextureEffect::Invert => {
                            apply_filter(&mut filtered, ImageFilter::Invert, 1.0)
                        }
                        TextureEffect::Sepia { intensity } => {
                            apply_filter(&mut filtered, ImageFilter::Sepia, intensity)
                        }
                        TextureEffect::GaussianBlur { sigma } => try_apply_image_filter(
                            &mut filtered,
                            ImageFilterOp::GaussianBlur { sigma },
                            core_limits,
                        )
                        .map_err(|e| format!("Canonical Gaussian blur: {e:?}"))?,
                        TextureEffect::Sharpen { sigma, amount } => try_apply_image_filter(
                            &mut filtered,
                            ImageFilterOp::Sharpen { sigma, amount },
                            core_limits,
                        )
                        .map_err(|e| format!("Canonical Sharpen: {e:?}"))?,
                        TextureEffect::HueRotate { degrees } => try_apply_image_filter(
                            &mut filtered,
                            ImageFilterOp::HueRotate { degrees },
                            core_limits,
                        )
                        .map_err(|e| format!("Canonical hue rotation: {e:?}"))?,
                        TextureEffect::Emboss {
                            azimuth_degrees,
                            strength,
                        } => try_apply_image_filter(
                            &mut filtered,
                            ImageFilterOp::Emboss {
                                azimuth_degrees,
                                strength,
                            },
                            core_limits,
                        )
                        .map_err(|e| format!("Canonical Emboss: {e:?}"))?,
                    }
                }
                filtered.pixels.iter().flatten().copied().collect()
            };
            maps.push((TextureRole::BaseColor, encoded(side, &rgba)?));
            Some(b.digest)
        }
        DetailRecipe::Procedural(p) => {
            if base.is_some() {
                return Err("Procedural detail has no imported base input".into());
            }
            let mut colour = vec![0; len];
            let mut normal = vec![0; len];
            let mut orm = vec![0; len];
            generate_surface_maps_into(
                SurfaceRecipe {
                    seed: p.seed,
                    tint: p.tint,
                    contrast: p.contrast,
                    relief: p.relief,
                    bands: p.bands,
                    band_strength: p.band_strength,
                    roughness: p.roughness,
                    metallic: p.metallic,
                },
                side,
                &mut colour,
                &mut normal,
                &mut orm,
            )
            .map_err(|e| format!("Canonical procedural detail: {e:?}"))?;
            maps.push((TextureRole::BaseColor, encoded(side, &colour)?));
            maps.push((TextureRole::Normal, encoded(side, &normal)?));
            // ORM's R=1 also explicitly supplies the separate AO map role.
            maps.push((TextureRole::MetallicRoughness, encoded(side, &orm)?));
            None
        }
    };
    let mut overrides: Vec<_> = maps
        .iter()
        .map(|(role, dependency)| (*role, dependency.digest))
        .collect();
    if let Some((_, digest)) = overrides
        .iter()
        .find(|(r, _)| *r == TextureRole::MetallicRoughness)
        .copied()
    {
        overrides.push((TextureRole::Occlusion, digest));
    }
    let mut dependencies: Vec<TextureDependency> = Vec::new();
    for (_, dependency) in maps {
        if !dependencies.iter().any(|d| d.digest == dependency.digest) {
            dependencies.push(dependency);
        }
    }
    Ok(DetailOutput {
        overrides,
        dependencies,
        base_digest,
        recipe_sha256: recipe.digest()?,
    })
}
