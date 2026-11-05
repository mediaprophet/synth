//! Clean-room reference-image to calibrated MAT4 PBR map synthesis (PA-03).
//!
//! Synthesizes delighting albedo base color, luminance heightfield, Sobel/Scharr
//! tangent-space surface normals, micro-variance roughness, and ambient occlusion.

/// Configuration parameters for PBR map extraction.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PbrExtractionConfig {
    pub normal_strength: f32,
    pub roughness_base: f32,
    pub delight_strength: f32,
    pub ao_radius: u32,
    pub max_pixel_budget: u32,
}

impl Default for PbrExtractionConfig {
    fn default() -> Self {
        Self {
            normal_strength: 2.5,
            roughness_base: 0.35,
            delight_strength: 0.6,
            ao_radius: 2,
            max_pixel_budget: 4096 * 4096,
        }
    }
}

/// Extracted and calibrated PBR surface maps.
#[derive(Debug, Clone, PartialEq)]
pub struct SynthesizedPbrMaps {
    pub width: u32,
    pub height: u32,
    pub albedo: Vec<u8>,               // Delighted RGBA8 (width * height * 4)
    pub normal_rgba8: Vec<u8>,         // Tangent-space normal map RGBA8
    pub normal_vectors: Vec<[f32; 3]>, // Normalized [nx, ny, nz] with nz > 0.0
    pub roughness: Vec<f32>,           // Clamped [0.04..=1.0]
    pub roughness_u8: Vec<u8>,         // [0..=255]
    pub heightfield: Vec<f32>,         // Normalized [0.0..=1.0]
    pub ambient_occlusion: Vec<f32>,   // Normalized [0.0..=1.0]
    pub ao_u8: Vec<u8>,                // [0..=255]
}

/// Errors occurring during PBR map extraction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RawToPbrError {
    ZeroDimensions,
    MismatchedBufferLength { expected: usize, actual: usize },
    BudgetExceeded { max_budget: u32, requested: u32 },
    InvalidConfig(String),
}

impl std::fmt::Display for RawToPbrError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ZeroDimensions => write!(f, "image dimensions must be non-zero"),
            Self::MismatchedBufferLength { expected, actual } => {
                write!(f, "mismatched buffer length: expected {expected}, actual {actual}")
            }
            Self::BudgetExceeded { max_budget, requested } => {
                write!(f, "pixel budget exceeded: max {max_budget}, requested {requested}")
            }
            Self::InvalidConfig(msg) => write!(f, "invalid PBR configuration: {msg}"),
        }
    }
}
impl std::error::Error for RawToPbrError {}

/// Synthesizes calibrated PBR maps from a raw/raster RGBA8 image buffer.
pub fn synthesize_pbr_maps_from_rgba8(
    rgba8: &[u8],
    width: u32,
    height: u32,
    config: &PbrExtractionConfig,
) -> Result<SynthesizedPbrMaps, RawToPbrError> {
    if width == 0 || height == 0 {
        return Err(RawToPbrError::ZeroDimensions);
    }
    let pixel_count = (width as u64) * (height as u64);
    if pixel_count > config.max_pixel_budget as u64 {
        return Err(RawToPbrError::BudgetExceeded {
            max_budget: config.max_pixel_budget,
            requested: pixel_count as u32,
        });
    }
    let expected_len = (pixel_count as usize) * 4;
    if rgba8.len() != expected_len {
        return Err(RawToPbrError::MismatchedBufferLength {
            expected: expected_len,
            actual: rgba8.len(),
        });
    }
    if config.normal_strength <= 0.0 || !config.normal_strength.is_finite() {
        return Err(RawToPbrError::InvalidConfig("normal_strength must be positive finite".into()));
    }
    if config.roughness_base < 0.0 || config.roughness_base > 1.0 {
        return Err(RawToPbrError::InvalidConfig("roughness_base must be in 0.0..=1.0".into()));
    }

    let w = width as usize;
    let h = height as usize;
    let total_pixels = w * h;

    // 1. Luminance heightfield estimation
    let mut heightfield = Vec::with_capacity(total_pixels);
    let mut sum_lum = 0.0f32;
    for p in 0..total_pixels {
        let r = rgba8[p * 4] as f32 / 255.0;
        let g = rgba8[p * 4 + 1] as f32 / 255.0;
        let b = rgba8[p * 4 + 2] as f32 / 255.0;
        let lum = 0.2126 * r + 0.7152 * g + 0.0722 * b;
        heightfield.push(lum);
        sum_lum += lum;
    }
    let mean_lum = (sum_lum / total_pixels as f32).max(0.01);

    // 2. Delighted albedo normalization
    let mut albedo = Vec::with_capacity(total_pixels * 4);
    let rad = config.ao_radius as i32;
    for y in 0..h {
        for x in 0..w {
            let mut local_lum_sum = 0.0f32;
            let mut local_lum_count = 0;
            for dy in -rad..=rad {
                for dx in -rad..=rad {
                    let nx = (x as i32 + dx).clamp(0, w as i32 - 1) as usize;
                    let ny = (y as i32 + dy).clamp(0, h as i32 - 1) as usize;
                    local_lum_sum += heightfield[ny * w + nx];
                    local_lum_count += 1;
                }
            }
            let local_lum = (local_lum_sum / local_lum_count as f32).max(0.01);
            let delight_scale = (mean_lum / local_lum).clamp(0.5, 2.0);
            let blend = 1.0 + (delight_scale - 1.0) * config.delight_strength;

            let idx = (y * w + x) * 4;
            let dr = ((rgba8[idx] as f32 * blend).clamp(0.0, 255.0)) as u8;
            let dg = ((rgba8[idx + 1] as f32 * blend).clamp(0.0, 255.0)) as u8;
            let db = ((rgba8[idx + 2] as f32 * blend).clamp(0.0, 255.0)) as u8;
            let da = rgba8[idx + 3];

            albedo.extend_from_slice(&[dr, dg, db, da]);
        }
    }

    // 3. Sobel/Scharr tangent-space normal map
    let mut normal_rgba8 = Vec::with_capacity(total_pixels * 4);
    let mut normal_vectors = Vec::with_capacity(total_pixels);

    let get_h = |px: i32, py: i32| -> f32 {
        let cx = px.clamp(0, w as i32 - 1) as usize;
        let cy = py.clamp(0, h as i32 - 1) as usize;
        heightfield[cy * w + cx]
    };

    for y in 0..h as i32 {
        for x in 0..w as i32 {
            let dx = (-3.0 * get_h(x - 1, y - 1) + 3.0 * get_h(x + 1, y - 1)
                - 10.0 * get_h(x - 1, y) + 10.0 * get_h(x + 1, y)
                - 3.0 * get_h(x - 1, y + 1) + 3.0 * get_h(x + 1, y + 1))
                / 32.0;

            let dy = (-3.0 * get_h(x - 1, y - 1) - 10.0 * get_h(x, y - 1) - 3.0 * get_h(x + 1, y - 1)
                + 3.0 * get_h(x - 1, y + 1) + 10.0 * get_h(x, y + 1) + 3.0 * get_h(x + 1, y + 1))
                / 32.0;

            let vx = -dx * config.normal_strength;
            let vy = -dy * config.normal_strength;
            let vz = 1.0f32;
            let len = (vx * vx + vy * vy + vz * vz).sqrt();

            let nx = vx / len;
            let ny = vy / len;
            let nz = vz / len;

            normal_vectors.push([nx, ny, nz]);

            let nr = (((nx * 0.5 + 0.5) * 255.0).clamp(0.0, 255.0)) as u8;
            let ng = (((ny * 0.5 + 0.5) * 255.0).clamp(0.0, 255.0)) as u8;
            let nb = (((nz * 0.5 + 0.5) * 255.0).clamp(0.0, 255.0)) as u8;
            normal_rgba8.extend_from_slice(&[nr, ng, nb, 255]);
        }
    }

    // 4. Micro-variance roughness map
    let mut roughness = Vec::with_capacity(total_pixels);
    let mut roughness_u8 = Vec::with_capacity(total_pixels);

    for y in 0..h as i32 {
        for x in 0..w as i32 {
            let mut mean = 0.0f32;
            let mut mean_sq = 0.0f32;
            let mut count = 0;
            for dy in -1..=1 {
                for dx in -1..=1 {
                    let v = get_h(x + dx, y + dy);
                    mean += v;
                    mean_sq += v * v;
                    count += 1;
                }
            }
            mean /= count as f32;
            mean_sq /= count as f32;
            let variance = (mean_sq - mean * mean).max(0.0);
            let std_dev = variance.sqrt();
            let rough = (config.roughness_base + std_dev * 2.5).clamp(0.04, 1.0);
            roughness.push(rough);
            roughness_u8.push(((rough * 255.0).clamp(0.0, 255.0)) as u8);
        }
    }

    // 5. Ambient occlusion estimation
    let mut ambient_occlusion = Vec::with_capacity(total_pixels);
    let mut ao_u8 = Vec::with_capacity(total_pixels);

    for y in 0..h as i32 {
        for x in 0..w as i32 {
            let center_h = get_h(x, y);
            let mut occ_sum = 0.0f32;
            let mut sample_count = 0;
            for dy in -rad..=rad {
                for dx in -rad..=rad {
                    if dx == 0 && dy == 0 {
                        continue;
                    }
                    let dist = ((dx * dx + dy * dy) as f32).sqrt();
                    let diff = (get_h(x + dx, y + dy) - center_h).max(0.0);
                    occ_sum += diff / dist;
                    sample_count += 1;
                }
            }
            let avg_occ = if sample_count > 0 { occ_sum / sample_count as f32 } else { 0.0 };
            let ao = (1.0 - avg_occ * 2.0).clamp(0.0, 1.0);
            ambient_occlusion.push(ao);
            ao_u8.push(((ao * 255.0).clamp(0.0, 255.0)) as u8);
        }
    }

    Ok(SynthesizedPbrMaps {
        width,
        height,
        albedo,
        normal_rgba8,
        normal_vectors,
        roughness,
        roughness_u8,
        heightfield,
        ambient_occlusion,
        ao_u8,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pbr_synthesis_invariants() {
        let (w, h) = (8u32, 8u32);
        let mut rgba = Vec::with_capacity((w * h * 4) as usize);
        for y in 0..h {
            for x in 0..w {
                let v = ((x + y) * 16).min(255) as u8;
                rgba.extend_from_slice(&[v, v, v, 255]);
            }
        }

        let config = PbrExtractionConfig::default();
        let pbr = synthesize_pbr_maps_from_rgba8(&rgba, w, h, &config).expect("pbr extraction");

        assert_eq!(pbr.width, w);
        assert_eq!(pbr.height, h);
        assert_eq!(pbr.normal_vectors.len(), (w * h) as usize);
        assert_eq!(pbr.roughness.len(), (w * h) as usize);

        for &[nx, ny, nz] in &pbr.normal_vectors {
            let len = (nx * nx + ny * ny + nz * nz).sqrt();
            assert!((len - 1.0).abs() < 1e-4, "normal vector must be unit length: {len}");
            assert!(nz > 0.0, "tangent normal nz must be positive: {nz}");
        }

        for &r in &pbr.roughness {
            assert!(r >= 0.04 && r <= 1.0, "roughness must be in [0.04..=1.0], got {r}");
        }

        for &ao in &pbr.ambient_occlusion {
            assert!(ao >= 0.0 && ao <= 1.0, "ao must be in [0.0..=1.0], got {ao}");
        }
    }

    #[test]
    fn test_negative_buffer_mismatch_and_bounds() {
        let config = PbrExtractionConfig::default();

        // Mismatched buffer length
        let err1 = synthesize_pbr_maps_from_rgba8(&[255, 255, 255], 2, 2, &config);
        assert!(matches!(err1, Err(RawToPbrError::MismatchedBufferLength { .. })));

        // Zero dimensions
        let err2 = synthesize_pbr_maps_from_rgba8(&[], 0, 10, &config);
        assert!(matches!(err2, Err(RawToPbrError::ZeroDimensions)));

        // Budget exceeded
        let mut small_cfg = config;
        small_cfg.max_pixel_budget = 16;
        let dummy = vec![0u8; 32 * 4];
        let err3 = synthesize_pbr_maps_from_rgba8(&dummy, 8, 4, &small_cfg);
        assert!(matches!(err3, Err(RawToPbrError::BudgetExceeded { .. })));
    }
}
