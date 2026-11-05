// Clean-room adaptation derived from EffectCraft multi-layer compositor pass concepts
// Copyright (c) 2024 Storytold / EffectCraft contributors
// Licensed under the MIT License.
// Adapted for native Poet / PortalGpu RGBA8 compositor stack without external dependencies.

use std::fmt;

/// Layer blend modes for Porter-Duff compositing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlendMode {
    NormalSourceOver,
    Multiply,
    Screen,
    Overlay,
    AdditiveLinearDodge,
}

/// 2D geometric transform for a compositor layer.
#[derive(Debug, Clone, PartialEq)]
pub struct LayerTransform2D {
    pub translation: [f32; 2],
    pub scale: [f32; 2],
    pub rotation_radians: f32,
    pub anchor_point: [f32; 2],
}

impl Default for LayerTransform2D {
    fn default() -> Self {
        Self {
            translation: [0.0, 0.0],
            scale: [1.0, 1.0],
            rotation_radians: 0.0,
            anchor_point: [0.0, 0.0],
        }
    }
}

/// A single visual layer in the motion-graphics compositor stack.
#[derive(Debug, Clone, PartialEq)]
pub struct CompositorLayer {
    pub layer_id: u64,
    pub name: String,
    pub opacity: f32,
    pub blend_mode: BlendMode,
    pub transform: LayerTransform2D,
    pub in_frame: u32,
    pub out_frame: u32,
    pub muted: bool,
    pub solo: bool,
    pub pixels: Vec<u8>,
}

/// Descriptor defining a complete multi-layer compositor pass.
#[derive(Debug, Clone, PartialEq)]
pub struct CompositorPassDescriptor {
    pub width: u32,
    pub height: u32,
    pub layers: Vec<CompositorLayer>,
}

/// Error kinds returned by the compositor stack evaluator.
#[derive(Debug, Clone, PartialEq)]
pub enum CompositorError {
    InvalidOpacity { layer_id: u64, opacity: f32 },
    InvertedFrameRange { layer_id: u64, in_frame: u32, out_frame: u32 },
    DimensionMismatch { expected: usize, actual: usize },
    InvalidLayerDimensions { layer_id: u64, expected: usize, actual: usize },
    ZeroDimensions,
}

impl fmt::Display for CompositorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidOpacity { layer_id, opacity } => {
                write!(f, "Layer {} has invalid opacity {} (must be 0.0..=1.0)", layer_id, opacity)
            }
            Self::InvertedFrameRange { layer_id, in_frame, out_frame } => {
                write!(f, "Layer {} has inverted frame range [{}, {})", layer_id, in_frame, out_frame)
            }
            Self::DimensionMismatch { expected, actual } => {
                write!(f, "Compositor output buffer mismatch: expected {} bytes, got {}", expected, actual)
            }
            Self::InvalidLayerDimensions { layer_id, expected, actual } => {
                write!(f, "Layer {} pixel buffer mismatch: expected {} bytes, got {}", layer_id, expected, actual)
            }
            Self::ZeroDimensions => write!(f, "Compositor pass dimensions must be non-zero"),
        }
    }
}

impl std::error::Error for CompositorError {}

/// Evaluates a multi-layer compositor stack for a specific timeline frame into an RGBA8 buffer.
/// Blending uses deterministic Porter-Duff premultiplied alpha math with zero heap allocation in loop.
pub fn evaluate_compositor_stack_rgba8(
    pass: &CompositorPassDescriptor,
    current_frame: u32,
    out_buffer: &mut [u8],
) -> Result<(), CompositorError> {
    if pass.width == 0 || pass.height == 0 {
        return Err(CompositorError::ZeroDimensions);
    }
    let pixel_count = (pass.width as usize) * (pass.height as usize);
    let expected_len = pixel_count * 4;
    if out_buffer.len() != expected_len {
        return Err(CompositorError::DimensionMismatch {
            expected: expected_len,
            actual: out_buffer.len(),
        });
    }

    // Step 1: Validate all layer invariants
    for layer in &pass.layers {
        if !layer.opacity.is_finite() || layer.opacity < 0.0 || layer.opacity > 1.0 {
            return Err(CompositorError::InvalidOpacity {
                layer_id: layer.layer_id,
                opacity: layer.opacity,
            });
        }
        if layer.in_frame >= layer.out_frame {
            return Err(CompositorError::InvertedFrameRange {
                layer_id: layer.layer_id,
                in_frame: layer.in_frame,
                out_frame: layer.out_frame,
            });
        }
        if layer.pixels.len() != expected_len {
            return Err(CompositorError::InvalidLayerDimensions {
                layer_id: layer.layer_id,
                expected: expected_len,
                actual: layer.pixels.len(),
            });
        }
    }

    // Step 2: Determine if any active layer is in solo mode
    let any_solo = pass.layers.iter().any(|l| {
        !l.muted && l.solo && current_frame >= l.in_frame && current_frame < l.out_frame
    });

    // Step 3: Filter indices of participating layers (bottom-to-top order)
    let active_indices: Vec<usize> = pass.layers.iter().enumerate().filter_map(|(idx, l)| {
        if l.muted || current_frame < l.in_frame || current_frame >= l.out_frame {
            return None;
        }
        if any_solo && !l.solo {
            return None;
        }
        if l.opacity <= 0.0 {
            return None;
        }
        Some(idx)
    }).collect();

    // Fast path: no participating layers -> clear to transparent black
    if active_indices.is_empty() {
        out_buffer.fill(0);
        return Ok(());
    }

    // Step 4: Per-pixel Porter-Duff compositing (zero heap inside loop)
    for p in 0..pixel_count {
        let offset = p * 4;
        let mut acc_c = [0.0f32; 3]; // premultiplied RGB
        let mut acc_a = 0.0f32;      // alpha

        for &idx in &active_indices {
            let layer = &pass.layers[idx];
            let src_r = layer.pixels[offset] as f32 / 255.0;
            let src_g = layer.pixels[offset + 1] as f32 / 255.0;
            let src_b = layer.pixels[offset + 2] as f32 / 255.0;
            let src_a = (layer.pixels[offset + 3] as f32 / 255.0) * layer.opacity;

            if src_a <= 0.0 {
                continue;
            }

            let cs = [src_r * src_a, src_g * src_a, src_b * src_a];
            let cd = acc_c;
            let ad = acc_a;

            let (out_c, out_a) = match layer.blend_mode {
                BlendMode::NormalSourceOver => {
                    let out_r = cs[0] + cd[0] * (1.0 - src_a);
                    let out_g = cs[1] + cd[1] * (1.0 - src_a);
                    let out_b = cs[2] + cd[2] * (1.0 - src_a);
                    let out_a = src_a + ad * (1.0 - src_a);
                    ([out_r, out_g, out_b], out_a)
                }
                BlendMode::Multiply => {
                    let out_r = (1.0 - ad) * cs[0] + (1.0 - src_a) * cd[0] + cd[0] * cs[0];
                    let out_g = (1.0 - ad) * cs[1] + (1.0 - src_a) * cd[1] + cd[1] * cs[1];
                    let out_b = (1.0 - ad) * cs[2] + (1.0 - src_a) * cd[2] + cd[2] * cs[2];
                    let out_a = src_a + ad * (1.0 - src_a);
                    ([out_r, out_g, out_b], out_a)
                }
                BlendMode::Screen => {
                    let out_r = cs[0] + cd[0] - cd[0] * cs[0];
                    let out_g = cs[1] + cd[1] - cd[1] * cs[1];
                    let out_b = cs[2] + cd[2] - cd[2] * cs[2];
                    let out_a = src_a + ad * (1.0 - src_a);
                    ([out_r, out_g, out_b], out_a)
                }
                BlendMode::Overlay => {
                    let blend_channel = |cs_k: f32, cd_k: f32| -> f32 {
                        if ad <= 0.0 {
                            cs_k
                        } else if src_a <= 0.0 {
                            cd_k
                        } else {
                            let cb = (cd_k / ad).clamp(0.0, 1.0);
                            let c_s = (cs_k / src_a).clamp(0.0, 1.0);
                            let f = if cb <= 0.5 {
                                2.0 * cb * c_s
                            } else {
                                1.0 - 2.0 * (1.0 - cb) * (1.0 - c_s)
                            };
                            (1.0 - ad) * cs_k + (1.0 - src_a) * cd_k + src_a * ad * f
                        }
                    };
                    let out_r = blend_channel(cs[0], cd[0]);
                    let out_g = blend_channel(cs[1], cd[1]);
                    let out_b = blend_channel(cs[2], cd[2]);
                    let out_a = src_a + ad * (1.0 - src_a);
                    ([out_r, out_g, out_b], out_a)
                }
                BlendMode::AdditiveLinearDodge => {
                    let out_r = (cs[0] + cd[0]).min(1.0);
                    let out_g = (cs[1] + cd[1]).min(1.0);
                    let out_b = (cs[2] + cd[2]).min(1.0);
                    let out_a = (src_a + ad).min(1.0);
                    ([out_r, out_g, out_b], out_a)
                }
            };

            acc_c = out_c;
            acc_a = out_a.min(1.0);
        }

        // Convert premultiplied RGBA back to straight RGBA8
        if acc_a > 1e-5 {
            let inv_a = 1.0 / acc_a;
            let final_r = ((acc_c[0] * inv_a).clamp(0.0, 1.0) * 255.0).round() as u8;
            let final_g = ((acc_c[1] * inv_a).clamp(0.0, 1.0) * 255.0).round() as u8;
            let final_b = ((acc_c[2] * inv_a).clamp(0.0, 1.0) * 255.0).round() as u8;
            let final_a = (acc_a * 255.0).round().clamp(0.0, 255.0) as u8;
            out_buffer[offset] = final_r;
            out_buffer[offset + 1] = final_g;
            out_buffer[offset + 2] = final_b;
            out_buffer[offset + 3] = final_a;
        } else {
            out_buffer[offset] = 0;
            out_buffer[offset + 1] = 0;
            out_buffer[offset + 2] = 0;
            out_buffer[offset + 3] = 0;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_layer(
        id: u64,
        rgba: [u8; 4],
        opacity: f32,
        blend_mode: BlendMode,
        in_frame: u32,
        out_frame: u32,
    ) -> CompositorLayer {
        CompositorLayer {
            layer_id: id,
            name: format!("layer_{}", id),
            opacity,
            blend_mode,
            transform: LayerTransform2D::default(),
            in_frame,
            out_frame,
            muted: false,
            solo: false,
            pixels: rgba.to_vec(),
        }
    }

    #[test]
    fn test_normal_source_over_blending() {
        let bottom = make_test_layer(1, [255, 0, 0, 255], 1.0, BlendMode::NormalSourceOver, 0, 10);
        let top = make_test_layer(2, [0, 0, 255, 128], 1.0, BlendMode::NormalSourceOver, 0, 10);
        let pass = CompositorPassDescriptor { width: 1, height: 1, layers: vec![bottom, top] };
        let mut out = [0u8; 4];
        evaluate_compositor_stack_rgba8(&pass, 5, &mut out).unwrap();
        assert!(out[0] > 100);
        assert_eq!(out[1], 0);
        assert!(out[2] > 100);
        assert_eq!(out[3], 255);
    }

    #[test]
    fn test_opacity_scaling() {
        let layer = make_test_layer(1, [200, 200, 200, 255], 0.5, BlendMode::NormalSourceOver, 0, 10);
        let pass = CompositorPassDescriptor { width: 1, height: 1, layers: vec![layer] };
        let mut out = [0u8; 4];
        evaluate_compositor_stack_rgba8(&pass, 0, &mut out).unwrap();
        assert_eq!(out[0], 200);
        assert_eq!(out[1], 200);
        assert_eq!(out[2], 200);
        assert!((out[3] as i32 - 128).abs() <= 1);
    }

    #[test]
    fn test_additive_linear_dodge() {
        let layer1 = make_test_layer(1, [100, 50, 0, 255], 1.0, BlendMode::NormalSourceOver, 0, 10);
        let layer2 = make_test_layer(2, [100, 50, 0, 255], 1.0, BlendMode::AdditiveLinearDodge, 0, 10);
        let pass = CompositorPassDescriptor { width: 1, height: 1, layers: vec![layer1, layer2] };
        let mut out = [0u8; 4];
        evaluate_compositor_stack_rgba8(&pass, 0, &mut out).unwrap();
        assert_eq!(out[0], 200);
        assert_eq!(out[1], 100);
        assert_eq!(out[2], 0);
    }

    #[test]
    fn test_solo_and_mute_flags() {
        let mut l1 = make_test_layer(1, [255, 0, 0, 255], 1.0, BlendMode::NormalSourceOver, 0, 10);
        let mut l2 = make_test_layer(2, [0, 255, 0, 255], 1.0, BlendMode::NormalSourceOver, 0, 10);
        l2.solo = true;
        let pass = CompositorPassDescriptor { width: 1, height: 1, layers: vec![l1.clone(), l2] };
        let mut out = [0u8; 4];
        evaluate_compositor_stack_rgba8(&pass, 0, &mut out).unwrap();
        assert_eq!(out, [0, 255, 0, 255]);

        l1.muted = true;
        let pass_muted = CompositorPassDescriptor { width: 1, height: 1, layers: vec![l1] };
        evaluate_compositor_stack_rgba8(&pass_muted, 0, &mut out).unwrap();
        assert_eq!(out, [0, 0, 0, 0]);
    }

    #[test]
    fn test_frame_range_visibility() {
        let layer = make_test_layer(1, [10, 20, 30, 255], 1.0, BlendMode::NormalSourceOver, 5, 15);
        let pass = CompositorPassDescriptor { width: 1, height: 1, layers: vec![layer] };
        let mut out = [0u8; 4];
        evaluate_compositor_stack_rgba8(&pass, 4, &mut out).unwrap();
        assert_eq!(out, [0, 0, 0, 0]);
        evaluate_compositor_stack_rgba8(&pass, 5, &mut out).unwrap();
        assert_eq!(out, [10, 20, 30, 255]);
        evaluate_compositor_stack_rgba8(&pass, 14, &mut out).unwrap();
        assert_eq!(out, [10, 20, 30, 255]);
        evaluate_compositor_stack_rgba8(&pass, 15, &mut out).unwrap();
        assert_eq!(out, [0, 0, 0, 0]);
    }

    #[test]
    fn test_negative_invalid_opacity_rejected() {
        let mut layer = make_test_layer(1, [0, 0, 0, 255], 1.5, BlendMode::NormalSourceOver, 0, 10);
        layer.opacity = 1.5;
        let pass = CompositorPassDescriptor { width: 1, height: 1, layers: vec![layer] };
        let mut out = [0u8; 4];
        let err = evaluate_compositor_stack_rgba8(&pass, 0, &mut out).unwrap_err();
        assert_eq!(err, CompositorError::InvalidOpacity { layer_id: 1, opacity: 1.5 });
    }

    #[test]
    fn test_negative_inverted_frame_range_rejected() {
        let layer = make_test_layer(1, [0, 0, 0, 255], 1.0, BlendMode::NormalSourceOver, 20, 10);
        let pass = CompositorPassDescriptor { width: 1, height: 1, layers: vec![layer] };
        let mut out = [0u8; 4];
        let err = evaluate_compositor_stack_rgba8(&pass, 0, &mut out).unwrap_err();
        assert_eq!(err, CompositorError::InvertedFrameRange { layer_id: 1, in_frame: 20, out_frame: 10 });
    }

    #[test]
    fn test_negative_dimension_mismatch_rejected() {
        let layer = make_test_layer(1, [0, 0, 0, 255], 1.0, BlendMode::NormalSourceOver, 0, 10);
        let pass = CompositorPassDescriptor { width: 2, height: 2, layers: vec![layer] };
        let mut out = [0u8; 4]; // expected 2*2*4 = 16
        let err = evaluate_compositor_stack_rgba8(&pass, 0, &mut out).unwrap_err();
        assert_eq!(err, CompositorError::DimensionMismatch { expected: 16, actual: 4 });
    }
}
