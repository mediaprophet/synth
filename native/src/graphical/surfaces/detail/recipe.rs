use qualia_core_db::hypermedia_authoring::image::{filter_plan, ImageFilterOp, ImageLimits};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use vibe::Value;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum DetailRecipe {
    Paint(PaintRecipe),
    Procedural(ProceduralRecipe),
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PaintRecipe {
    pub version: u32,
    pub side: u32,
    pub base_sha256: String,
    pub layers: Vec<PaintLayer>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub effects: Vec<TextureEffectStep>,
}
/// Authored post-composite step; disabled parameters remain validated and retained.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TextureEffectStep {
    pub enabled: bool,
    pub effect: TextureEffect,
}
/// Bounded real Core filters in encoded RGBA8 space. Grayscale/Invert are full strength.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum TextureEffect {
    Grayscale,
    Invert,
    Sepia { intensity: f32 },
    GaussianBlur { sigma: f32 },
    Sharpen { sigma: f32, amount: f32 },
    HueRotate { degrees: f32 },
    Emboss { azimuth_degrees: f32, strength: f32 },
}
impl<'de> Deserialize<'de> for TextureEffect {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // Internally tagged unit variants otherwise accept unknown payload fields.
        // Empty struct variants enforce the same strict object schema without
        // changing the public unit API or its serialized representation.
        #[derive(Deserialize)]
        #[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
        enum StrictEffect {
            Grayscale {},
            Invert {},
            Sepia { intensity: f32 },
            GaussianBlur { sigma: f32 },
            Sharpen { sigma: f32, amount: f32 },
            HueRotate { degrees: f32 },
            Emboss { azimuth_degrees: f32, strength: f32 },
        }
        Ok(match StrictEffect::deserialize(deserializer)? {
            StrictEffect::Grayscale {} => Self::Grayscale,
            StrictEffect::Invert {} => Self::Invert,
            StrictEffect::Sepia { intensity } => Self::Sepia { intensity },
            StrictEffect::GaussianBlur { sigma } => Self::GaussianBlur { sigma },
            StrictEffect::Sharpen { sigma, amount } => Self::Sharpen { sigma, amount },
            StrictEffect::HueRotate { degrees } => Self::HueRotate { degrees },
            StrictEffect::Emboss {
                azimuth_degrees,
                strength,
            } => Self::Emboss {
                azimuth_degrees,
                strength,
            },
        })
    }
}
/// Profile admission shared by source, native controls, and pre-context MCP checks.
pub fn validate_effects(effects: &[TextureEffectStep]) -> Result<(), String> {
    if effects.len() > 8 {
        return Err("Texture effect stack admits at most 8 steps".into());
    }
    for step in effects {
        match step.effect {
            TextureEffect::Sepia { intensity } if !unit(intensity) => {
                return Err("Texture effect Sepia intensity must be finite 0..1".into())
            }
            TextureEffect::GaussianBlur { sigma }
                if !sigma.is_finite() || !(0.0..=16.0).contains(&sigma) =>
            {
                return Err("Gaussian sigma must be finite 0..16 pixels".into())
            }
            TextureEffect::Sharpen { sigma, amount }
                if !sigma.is_finite()
                    || !(0.0..=16.0).contains(&sigma)
                    || !amount.is_finite()
                    || !(0.0..=4.0).contains(&amount) =>
            {
                return Err("Sharpen sigma/amount outside 0..16 pixels/0..4".into())
            }
            TextureEffect::HueRotate { degrees }
                if !degrees.is_finite() || !(-360.0..=360.0).contains(&degrees) =>
            {
                return Err("Hue rotation degrees outside -360..360".into())
            }
            TextureEffect::Emboss {
                azimuth_degrees,
                strength,
            } if !azimuth_degrees.is_finite()
                || !(-360.0..=360.0).contains(&azimuth_degrees)
                || !unit(strength) =>
            {
                return Err("Emboss azimuth/strength outside -360..360 degrees/0..1".into())
            }
            _ => {}
        }
    }
    Ok(())
}
/// Ordered normal source-over layer in canonical encoded RGBA8 working space.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PaintLayer {
    pub name: String,
    pub visible: bool,
    pub opacity: f32,
    pub strokes: Vec<PaintStroke>,
    #[serde(default, skip_serializing_if = "is_normal_blend")]
    pub blend_mode: LayerBlendMode,
    #[serde(default, skip_serializing_if = "is_zero_offset")]
    pub offset: [i32; 2],
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mask: Option<LayerMask>,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LayerBlendMode {
    #[default]
    Normal,
    Multiply,
    Screen,
    Overlay,
    SoftLight,
    HardLight,
    ColorDodge,
    ColorBurn,
    Difference,
    Exclusion,
}
fn is_normal_blend(mode: &LayerBlendMode) -> bool {
    *mode == LayerBlendMode::Normal
}
fn is_zero_offset(offset: &[i32; 2]) -> bool {
    *offset == [0, 0]
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MaskChannel {
    Red,
    Alpha,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum LayerMask {
    Rectangle {
        x: u32,
        y: u32,
        width: u32,
        height: u32,
        coverage: u8,
    },
    Resource {
        sha256: String,
        channel: MaskChannel,
    },
}
impl Default for PaintLayer {
    fn default() -> Self {
        Self {
            name: "Layer".into(),
            visible: true,
            opacity: 1.0,
            strokes: Vec::new(),
            blend_mode: LayerBlendMode::Normal,
            offset: [0, 0],
            mask: None,
        }
    }
}
pub fn required_paint_version(p: &PaintRecipe) -> u32 {
    if p.layers
        .iter()
        .any(|l| l.blend_mode != LayerBlendMode::Normal || l.offset != [0, 0] || l.mask.is_some())
        || p.effects.iter().any(|s| {
            matches!(
                s.effect,
                TextureEffect::GaussianBlur { .. }
                    | TextureEffect::Sharpen { .. }
                    | TextureEffect::HueRotate { .. }
                    | TextureEffect::Emboss { .. }
            )
        })
    {
        3
    } else if !p.effects.is_empty() {
        2
    } else {
        1
    }
}
/// Discrete canonical brush dabs in image-pixel coordinates, +Y down.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PaintStroke {
    pub size: f32,
    pub colour: [u8; 4],
    pub opacity: f32,
    pub hardness: f32,
    pub points: Vec<[f32; 2]>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProceduralRecipe {
    pub version: u32,
    pub side: u32,
    pub seed: u32,
    pub tint: [f32; 3],
    pub contrast: f32,
    pub relief: f32,
    pub bands: [u32; 2],
    pub band_strength: f32,
    pub roughness: f32,
    pub metallic: f32,
}
#[derive(Clone, Copy, Debug)]
pub struct DetailLimits {
    pub working_bytes: usize,
    pub output_bytes: usize,
    pub pixel_visits: usize,
    pub layers: usize,
    pub strokes: usize,
    pub points: usize,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DetailCost {
    pub working_bytes: usize,
    pub output_bytes: usize,
    pub pixel_visits: usize,
}
impl Default for DetailLimits {
    fn default() -> Self {
        Self {
            working_bytes: 128 * 1024 * 1024,
            output_bytes: 16 * 1024 * 1024,
            pixel_visits: 128 * 1024 * 1024,
            layers: 8,
            strokes: 128,
            points: 4096,
        }
    }
}
fn unit(v: f32) -> bool {
    v.is_finite() && (0.0..=1.0).contains(&v)
}
fn charge(total: &mut usize, n: usize, bound: usize) -> Result<(), String> {
    *total = total
        .checked_add(n)
        .filter(|v| *v <= bound)
        .ok_or("Texture detail work budget exceeded")?;
    Ok(())
}
impl DetailRecipe {
    pub fn side(&self) -> u32 {
        match self {
            Self::Paint(p) => p.side,
            Self::Procedural(p) => p.side,
        }
    }
    pub fn from_value(v: &Value) -> Result<Self, String> {
        fn json(v: &Value, depth: usize, nodes: &mut usize) -> Result<serde_json::Value, String> {
            if depth > 8 || *nodes >= 32768 {
                return Err("Texture recipe source bound exceeded".into());
            }
            *nodes += 1;
            Ok(match v {
                Value::Bool(v) => serde_json::Value::Bool(*v),
                Value::I64(v) => (*v).into(),
                Value::U64(v) => (*v).into(),
                Value::F64(v) => serde_json::Number::from_f64(*v)
                    .ok_or("Nonfinite recipe number")?
                    .into(),
                Value::String(v) if v.len() <= 256 => v.clone().into(),
                Value::List(v) if v.len() <= 4096 => serde_json::Value::Array(
                    v.iter()
                        .map(|v| json(v, depth + 1, nodes))
                        .collect::<Result<_, _>>()?,
                ),
                Value::Record(v) if v.len() <= 32 => serde_json::Value::Object(
                    v.iter()
                        .map(|(k, v)| {
                            if k.len() > 64 {
                                return Err("Texture recipe field bound exceeded".into());
                            }
                            Ok((k.clone(), json(v, depth + 1, nodes)?))
                        })
                        .collect::<Result<_, String>>()?,
                ),
                _ => return Err("Unsupported texture recipe value".into()),
            })
        }
        let result: Self = serde_json::from_value(json(v, 0, &mut 0)?)
            .map_err(|e| format!("Texture detail recipe: {e}"))?;
        result.validate(DetailLimits::default())?;
        Ok(result)
    }
    /// Marshal only this strict schema into pinned Vibe record/list expressions.
    pub fn source_expression(&self) -> Result<String, String> {
        self.validate(DetailLimits::default())?;
        fn expression(v: serde_json::Value) -> Result<String, String> {
            Ok(match v {
                serde_json::Value::Object(r) => format!(
                    "{{{}}}",
                    r.into_iter()
                        .map(|(k, v)| {
                            if !k.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_') {
                                return Err("Invalid texture recipe field".into());
                            }
                            Ok(format!("{k}:{}", expression(v)?))
                        })
                        .collect::<Result<Vec<_>, String>>()?
                        .join(",")
                ),
                serde_json::Value::Array(v) => format!(
                    "[{}]",
                    v.into_iter()
                        .map(expression)
                        .collect::<Result<Vec<_>, _>>()?
                        .join(",")
                ),
                v => serde_json::to_string(&v).map_err(|e| e.to_string())?,
            })
        }
        let source = expression(serde_json::to_value(self).map_err(|e| e.to_string())?)?;
        if source.len() > 48 * 1024 {
            return Err("Texture recipe expression exceeds 48 KiB".into());
        }
        Ok(source)
    }
    pub fn digest(&self) -> Result<[u8; 32], String> {
        Ok(Sha256::digest(self.source_expression()?.as_bytes()).into())
    }
    pub fn validate(&self, l: DetailLimits) -> Result<(), String> {
        self.budget_plan(l).map(|_| ())
    }
    /// Same admission estimate for individual validation and root aggregation.
    pub fn budget_plan(&self, l: DetailLimits) -> Result<DetailCost, String> {
        let side = self.side();
        if !(8..=1024).contains(&side) {
            return Err("Texture detail side must be 8..1024".into());
        }
        let pixels = side as usize * side as usize;
        let mut visits = 0;
        let output_count = match self {
            Self::Paint(p) => {
                if !matches!(p.version, 1 | 2 | 3)
                    || p.version < required_paint_version(p)
                    || p.layers.len() > l.layers
                    || p.base_sha256.len() != 64
                    || !p
                        .base_sha256
                        .bytes()
                        .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
                    || p.base_sha256.bytes().all(|b| b == b'0')
                {
                    return Err("Invalid texture paint version/base/layer count".into());
                }
                validate_effects(&p.effects)?;
                let mut strokes = 0;
                let mut points = 0;
                // These passes also run with zero authored paint layers.
                charge(
                    &mut visits,
                    pixels.checked_mul(4).ok_or("Detail work overflow")?,
                    l.pixel_visits,
                )?;
                for layer in &p.layers {
                    if layer.name.is_empty()
                        || layer.name.len() > 80
                        || layer.name.chars().any(char::is_control)
                        || !unit(layer.opacity)
                    {
                        return Err("Invalid texture paint layer".into());
                    }
                    if let Some(mask) = &layer.mask {
                        match mask {
                            LayerMask::Rectangle {
                                x,
                                y,
                                width,
                                height,
                                ..
                            } => {
                                if *width == 0
                                    || *height == 0
                                    || x.checked_add(*width).is_none_or(|v| v > side)
                                    || y.checked_add(*height).is_none_or(|v| v > side)
                                {
                                    return Err("Paint rectangle mask exceeds layer extent".into());
                                }
                            }
                            LayerMask::Resource { sha256, .. } => {
                                if sha256.len() != 64
                                    || !sha256
                                        .bytes()
                                        .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
                                    || sha256.bytes().all(|b| b == b'0')
                                {
                                    return Err(
                                        "Paint mask resource requires nonzero lowercase SHA-256"
                                            .into(),
                                    );
                                }
                            }
                        }
                    }
                    charge(&mut strokes, layer.strokes.len(), l.strokes)?;
                    // Initialization/composite passes including hidden layers.
                    charge(
                        &mut visits,
                        pixels.checked_mul(3).ok_or("Detail work overflow")?,
                        l.pixel_visits,
                    )?;
                    for s in &layer.strokes {
                        if !s.size.is_finite()
                            || !(1.0..=side as f32 * 2.0).contains(&s.size)
                            || !unit(s.opacity)
                            || !unit(s.hardness)
                            || s.colour[3] != 255
                            || s.points.is_empty()
                            || s.points
                                .iter()
                                .flatten()
                                .any(|v| !v.is_finite() || !(0.0..side as f32).contains(v))
                        {
                            return Err("Invalid opaque discrete paint stroke".into());
                        }
                        charge(&mut points, s.points.len(), l.points)?;
                        charge(
                            &mut visits,
                            pixels
                                .checked_mul(s.points.len())
                                .ok_or("Detail work overflow")?,
                            l.pixel_visits,
                        )?;
                    }
                }
                if !p.effects.is_empty() {
                    // Initialization/copy plus every authored step, including disabled ones.
                    charge(
                        &mut visits,
                        pixels
                            .checked_mul(p.effects.len() + 2)
                            .ok_or("Detail work overflow")?,
                        l.pixel_visits,
                    )?;
                }
                if p.version == 3 {
                    for layer in &p.layers {
                        if layer.mask.is_some() {
                            charge(&mut visits, pixels, l.pixel_visits)?;
                        }
                    }
                    for step in &p.effects {
                        let op = match step.effect {
                            TextureEffect::GaussianBlur { sigma } => {
                                Some(ImageFilterOp::GaussianBlur { sigma })
                            }
                            TextureEffect::Sharpen { sigma, amount } => {
                                Some(ImageFilterOp::Sharpen { sigma, amount })
                            }
                            TextureEffect::HueRotate { degrees } => {
                                Some(ImageFilterOp::HueRotate { degrees })
                            }
                            TextureEffect::Emboss {
                                azimuth_degrees,
                                strength,
                            } => Some(ImageFilterOp::Emboss {
                                azimuth_degrees,
                                strength,
                            }),
                            _ => None,
                        };
                        if let Some(op) = op {
                            let plan = filter_plan(side, side, op, ImageLimits::default())
                                .map_err(|e| format!("Core image filter admission: {e:?}"))?;
                            charge(&mut visits, plan.work, l.pixel_visits)?;
                        }
                    }
                }
                1usize
            }
            Self::Procedural(p) => {
                if p.version != 1
                    || !p
                        .tint
                        .iter()
                        .copied()
                        .chain([
                            p.contrast,
                            p.relief,
                            p.band_strength,
                            p.roughness,
                            p.metallic,
                        ])
                        .all(unit)
                    || p.bands.iter().any(|v| *v > 128)
                {
                    return Err("Invalid procedural texture detail recipe".into());
                }
                charge(
                    &mut visits,
                    pixels.checked_mul(32).ok_or("Detail work overflow")?,
                    l.pixel_visits,
                )?;
                3
            }
        };
        let rgba = pixels * 4;
        let png = rgba + side as usize + 4096;
        // Includes opaque snapshots, canonical composite temporaries, decode copy,
        // encoded outputs, and conservative 8 MiB canonical PNG encoder scratch.
        let mut working = match self {
            Self::Paint(p) => {
                // One post-composite RGBA layer in addition to legacy temporaries.
                rgba.checked_mul(p.layers.len() + 6 + usize::from(!p.effects.is_empty()))
            }
            Self::Procedural(_) => rgba.checked_mul(4),
        }
        .and_then(|n| n.checked_add(png.checked_mul(output_count)?))
        .and_then(|n| n.checked_add(8 * 1024 * 1024))
        .ok_or("Detail memory overflow")?;
        if let Self::Paint(p) = self {
            if p.version == 3 {
                let mask_count = p.layers.iter().filter(|layer| layer.mask.is_some()).count();
                let mask_bytes = pixels
                    .checked_mul(mask_count)
                    .ok_or("Detail mask memory overflow")?;
                let filter_scratch = p
                    .effects
                    .iter()
                    .filter_map(|s| match s.effect {
                        TextureEffect::GaussianBlur { sigma } => {
                            Some(ImageFilterOp::GaussianBlur { sigma })
                        }
                        TextureEffect::Sharpen { sigma, amount } => {
                            Some(ImageFilterOp::Sharpen { sigma, amount })
                        }
                        TextureEffect::HueRotate { degrees } => {
                            Some(ImageFilterOp::HueRotate { degrees })
                        }
                        TextureEffect::Emboss {
                            azimuth_degrees,
                            strength,
                        } => Some(ImageFilterOp::Emboss {
                            azimuth_degrees,
                            strength,
                        }),
                        _ => None,
                    })
                    .map(|op| filter_plan(side, side, op, ImageLimits::default()))
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(|e| format!("Core image filter admission: {e:?}"))?
                    .iter()
                    .map(|plan| plan.bytes)
                    .max()
                    .unwrap_or(0);
                working = working
                    .checked_add(mask_bytes)
                    .and_then(|n| n.checked_add(filter_scratch))
                    .ok_or("Detail v3 memory overflow")?;
            }
        }
        if working > l.working_bytes || png * output_count > l.output_bytes {
            return Err("Texture detail byte budget exceeded".into());
        }
        Ok(DetailCost {
            working_bytes: working,
            output_bytes: png * output_count,
            pixel_visits: visits,
        })
    }
}
