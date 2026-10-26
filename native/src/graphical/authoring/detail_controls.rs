//! Native texture controls emit a source recipe through the existing intent owner.
use crate::graphical::surfaces::detail::{
    required_paint_version, DetailLimits, DetailRecipe, LayerBlendMode, LayerMask, MaskChannel,
    PaintLayer, PaintRecipe, PaintStroke, ProceduralRecipe, TextureEffect, TextureEffectStep,
};
use dioxus::prelude::*;

const STROKE_FIELDS: [&str; 5] = [
    "Diameter in pixels",
    "RGB bytes, comma separated",
    "Stroke opacity",
    "Hardness",
    "Dab points x,y; x,y (+Y down)",
];
const PROCEDURAL_FIELDS: [&str; 12] = [
    "Side",
    "Seed",
    "Tint red",
    "Tint green",
    "Tint blue",
    "Contrast",
    "Relief",
    "Bands U",
    "Bands V",
    "Band strength",
    "Roughness",
    "Metallic",
];
fn f32_value(s: &str) -> Result<f32, String> {
    let v = s
        .trim()
        .parse::<f32>()
        .map_err(|_| "Enter a finite number".to_owned())?;
    if !v.is_finite() {
        return Err("Enter a finite number".into());
    }
    Ok(v)
}
fn stroke(fields: &[String]) -> Result<PaintStroke, String> {
    if fields.len() != 5 || fields[4].len() > 32 * 1024 {
        return Err("Incomplete or oversized stroke fields".into());
    }
    let colour = fields[1]
        .split(',')
        .map(|s| {
            s.trim()
                .parse::<u8>()
                .map_err(|_| "RGB must be three bytes".to_owned())
        })
        .collect::<Result<Vec<_>, _>>()?;
    if colour.len() != 3 {
        return Err("RGB must be three bytes".into());
    }
    let mut points = Vec::new();
    for point in fields[4].split(';') {
        if points.len() >= 4096 {
            return Err("Dab point bound exceeded".into());
        }
        let coordinates = point
            .split(',')
            .map(f32_value)
            .collect::<Result<Vec<_>, _>>()?;
        if coordinates.len() != 2 {
            return Err("Each dab needs x,y".into());
        }
        points.push([coordinates[0], coordinates[1]]);
    }
    Ok(PaintStroke {
        size: f32_value(&fields[0])?,
        colour: [colour[0], colour[1], colour[2], 255],
        opacity: f32_value(&fields[2])?,
        hardness: f32_value(&fields[3])?,
        points,
    })
}
fn stroke_fields(s: &PaintStroke) -> Vec<String> {
    vec![
        s.size.to_string(),
        format!("{},{},{}", s.colour[0], s.colour[1], s.colour[2]),
        s.opacity.to_string(),
        s.hardness.to_string(),
        s.points
            .iter()
            .map(|p| format!("{},{}", p[0], p[1]))
            .collect::<Vec<_>>()
            .join(";"),
    ]
}
fn default_stroke_fields(side: u32) -> Vec<String> {
    stroke_fields(&PaintStroke {
        size: 8.0,
        colour: [255, 64, 16, 255],
        opacity: 1.0,
        hardness: 0.8,
        points: vec![[side as f32 / 2.0, side as f32 / 2.0]],
    })
}
fn procedural(xs: &[String]) -> Result<DetailRecipe, String> {
    if xs.len() != 12 {
        return Err("Incomplete procedural fields".into());
    }
    let integer = |i: usize| {
        xs[i]
            .trim()
            .parse::<u32>()
            .map_err(|_| "Side, seed and bands must be whole unsigned numbers".to_owned())
    };
    let p = DetailRecipe::Procedural(ProceduralRecipe {
        version: 1,
        side: integer(0)?,
        seed: integer(1)?,
        tint: [f32_value(&xs[2])?, f32_value(&xs[3])?, f32_value(&xs[4])?],
        contrast: f32_value(&xs[5])?,
        relief: f32_value(&xs[6])?,
        bands: [integer(7)?, integer(8)?],
        band_strength: f32_value(&xs[9])?,
        roughness: f32_value(&xs[10])?,
        metallic: f32_value(&xs[11])?,
    });
    p.validate(DetailLimits::default())?;
    Ok(p)
}
fn procedural_fields(p: &ProceduralRecipe) -> Vec<String> {
    vec![
        p.side.to_string(),
        p.seed.to_string(),
        p.tint[0].to_string(),
        p.tint[1].to_string(),
        p.tint[2].to_string(),
        p.contrast.to_string(),
        p.relief.to_string(),
        p.bands[0].to_string(),
        p.bands[1].to_string(),
        p.band_strength.to_string(),
        p.roughness.to_string(),
        p.metallic.to_string(),
    ]
}

// Raw numeric text belongs to the local form, not to the accepted recipe. Keeping
// it separately prevents a rerender from replacing invalid text with the previous
// valid float, and lets Apply reject that text before emitting an authoring edit.
#[derive(Clone, Debug, Default, PartialEq)]
struct PaintFieldDraft {
    opacities: Vec<String>,
    intensities: Vec<String>,
    secondaries: Vec<String>,
    layers: Vec<LayerFieldDraft>,
    // Each layer owns its pending stroke form. Switching layers must not let a
    // prior layer's form replace a different layer's accepted stroke.
    stroke_forms: Vec<StrokeFieldDraft>,
}
#[derive(Clone, Debug, PartialEq)]
struct StrokeFieldDraft {
    values: Vec<String>,
    selected: usize,
}
impl StrokeFieldDraft {
    fn from_layer(layer: &PaintLayer, side: u32) -> Self {
        Self {
            values: layer
                .strokes
                .first()
                .map(stroke_fields)
                .unwrap_or_else(|| default_stroke_fields(side)),
            selected: 0,
        }
    }
}
#[derive(Clone, Debug, PartialEq)]
struct LayerFieldDraft {
    blend_mode: String,
    offset_x: String,
    offset_y: String,
    mask_kind: String,
    rectangle: [String; 5],
    resource_sha256: String,
    resource_channel: String,
}
impl LayerFieldDraft {
    fn from_layer(layer: &PaintLayer, side: u32) -> Self {
        let mut out = Self {
            blend_mode: blend_mode_name(layer.blend_mode).into(),
            offset_x: layer.offset[0].to_string(),
            offset_y: layer.offset[1].to_string(),
            mask_kind: "none".into(),
            rectangle: [
                "0".into(),
                "0".into(),
                side.to_string(),
                side.to_string(),
                "255".into(),
            ],
            resource_sha256: String::new(),
            resource_channel: "alpha".into(),
        };
        match &layer.mask {
            Some(LayerMask::Rectangle {
                x,
                y,
                width,
                height,
                coverage,
            }) => {
                out.mask_kind = "rectangle".into();
                out.rectangle = [
                    x.to_string(),
                    y.to_string(),
                    width.to_string(),
                    height.to_string(),
                    coverage.to_string(),
                ];
            }
            Some(LayerMask::Resource { sha256, channel }) => {
                out.mask_kind = "resource".into();
                out.resource_sha256 = sha256.clone();
                out.resource_channel = match channel {
                    MaskChannel::Red => "red",
                    MaskChannel::Alpha => "alpha",
                }
                .into();
            }
            None => {}
        }
        out
    }
    fn parse(&self) -> Result<(LayerBlendMode, [i32; 2], Option<LayerMask>), String> {
        let blend_mode = blend_mode(&self.blend_mode)?;
        let signed = |s: &str| {
            s.trim().parse::<i32>().map_err(|_| {
                "Offset must be a signed whole pixel count within i32 range".to_owned()
            })
        };
        let offset = [signed(&self.offset_x)?, signed(&self.offset_y)?];
        let mask = match self.mask_kind.as_str() {
            "none" => None,
            "rectangle" => {
                let number = |s: &str| {
                    s.trim().parse::<u32>().map_err(|_| {
                        "Rectangle coordinates and size must be unsigned whole pixels".to_owned()
                    })
                };
                let coverage = self.rectangle[4]
                    .trim()
                    .parse::<u8>()
                    .map_err(|_| "Rectangle coverage must be a byte from 0 to 255".to_owned())?;
                Some(LayerMask::Rectangle {
                    x: number(&self.rectangle[0])?,
                    y: number(&self.rectangle[1])?,
                    width: number(&self.rectangle[2])?,
                    height: number(&self.rectangle[3])?,
                    coverage,
                })
            }
            "resource" => {
                if self.resource_sha256.len() != 64
                    || !self.resource_sha256.bytes().all(|b| b.is_ascii_hexdigit())
                {
                    return Err(
                        "Mask HMC resource SHA-256 must be 64 hexadecimal characters".into(),
                    );
                }
                let channel = match self.resource_channel.as_str() {
                    "red" => MaskChannel::Red,
                    "alpha" => MaskChannel::Alpha,
                    _ => return Err("Unknown mask channel".into()),
                };
                Some(LayerMask::Resource {
                    sha256: self.resource_sha256.clone(),
                    channel,
                })
            }
            _ => return Err("Unknown layer mask type".into()),
        };
        Ok((blend_mode, offset, mask))
    }
}
impl PaintFieldDraft {
    fn from_recipe(p: &PaintRecipe) -> Self {
        Self {
            opacities: p.layers.iter().map(|l| l.opacity.to_string()).collect(),
            intensities: p
                .effects
                .iter()
                .map(|s| match &s.effect {
                    TextureEffect::Sepia { intensity } => intensity.to_string(),
                    TextureEffect::GaussianBlur { sigma }
                    | TextureEffect::Sharpen { sigma, .. } => sigma.to_string(),
                    TextureEffect::HueRotate { degrees } => degrees.to_string(),
                    TextureEffect::Emboss {
                        azimuth_degrees, ..
                    } => azimuth_degrees.to_string(),
                    _ => "1".to_owned(),
                })
                .collect(),
            secondaries: p
                .effects
                .iter()
                .map(|s| match &s.effect {
                    TextureEffect::Sharpen { amount, .. } => amount.to_string(),
                    TextureEffect::Emboss { strength, .. } => strength.to_string(),
                    _ => "1".into(),
                })
                .collect(),
            layers: p
                .layers
                .iter()
                .map(|l| LayerFieldDraft::from_layer(l, p.side))
                .collect(),
            stroke_forms: p
                .layers
                .iter()
                .map(|l| StrokeFieldDraft::from_layer(l, p.side))
                .collect(),
        }
    }
}
fn unit_value(text: &str) -> Result<f32, String> {
    let value = f32_value(text)?;
    if !(0.0..=1.0).contains(&value) {
        return Err("Enter a finite number from 0 to 1".into());
    }
    Ok(value)
}
fn texture_effect(kind: &str, intensity: &str) -> Result<TextureEffect, String> {
    texture_effect_values(kind, intensity, "1")
}
fn bounded_value(text: &str, low: f32, high: f32) -> Result<f32, String> {
    let value = f32_value(text)?;
    if !(low..=high).contains(&value) {
        return Err(format!("Enter a finite number from {low} to {high}"));
    }
    Ok(value)
}
fn texture_effect_values(
    kind: &str,
    primary: &str,
    secondary: &str,
) -> Result<TextureEffect, String> {
    match kind {
        "grayscale" => Ok(TextureEffect::Grayscale),
        "invert" => Ok(TextureEffect::Invert),
        "sepia" => Ok(TextureEffect::Sepia {
            intensity: unit_value(primary)?,
        }),
        "gaussian_blur" => Ok(TextureEffect::GaussianBlur {
            sigma: bounded_value(primary, 0.0, 16.0)?,
        }),
        "sharpen" => Ok(TextureEffect::Sharpen {
            sigma: bounded_value(primary, 0.0, 16.0)?,
            amount: bounded_value(secondary, 0.0, 4.0)?,
        }),
        "hue_rotate" => Ok(TextureEffect::HueRotate {
            degrees: bounded_value(primary, -360.0, 360.0)?,
        }),
        "emboss" => Ok(TextureEffect::Emboss {
            azimuth_degrees: bounded_value(primary, -360.0, 360.0)?,
            strength: unit_value(secondary)?,
        }),
        _ => Err("Unknown texture effect".into()),
    }
}
fn texture_effect_kind(effect: &TextureEffect) -> &'static str {
    match effect {
        TextureEffect::Grayscale => "grayscale",
        TextureEffect::Invert => "invert",
        TextureEffect::Sepia { .. } => "sepia",
        TextureEffect::GaussianBlur { .. } => "gaussian_blur",
        TextureEffect::Sharpen { .. } => "sharpen",
        TextureEffect::HueRotate { .. } => "hue_rotate",
        TextureEffect::Emboss { .. } => "emboss",
    }
}
fn effect_primary_label(effect: &TextureEffect) -> &'static str {
    match effect {
        TextureEffect::Sepia { .. } => "Sepia intensity (0 to 1)",
        TextureEffect::GaussianBlur { .. } | TextureEffect::Sharpen { .. } => {
            "Sigma in image pixels (0 to 16)"
        }
        TextureEffect::HueRotate { .. } => "Hue rotation in degrees (-360 to 360)",
        TextureEffect::Emboss { .. } => {
            "Emboss azimuth in degrees (-360 to 360; +X right, +Y down)"
        }
        _ => "",
    }
}
fn effect_primary_key(effect: &TextureEffect) -> &'static str {
    match effect {
        TextureEffect::Sepia { .. } => "sepia intensity",
        TextureEffect::GaussianBlur { .. } | TextureEffect::Sharpen { .. } => "sigma",
        TextureEffect::HueRotate { .. } => "degrees",
        TextureEffect::Emboss { .. } => "azimuth degrees",
        _ => "",
    }
}
const BLEND_MODES: [&str; 10] = [
    "normal",
    "multiply",
    "screen",
    "overlay",
    "soft_light",
    "hard_light",
    "color_dodge",
    "color_burn",
    "difference",
    "exclusion",
];
fn blend_mode_name(mode: LayerBlendMode) -> &'static str {
    match mode {
        LayerBlendMode::Normal => "normal",
        LayerBlendMode::Multiply => "multiply",
        LayerBlendMode::Screen => "screen",
        LayerBlendMode::Overlay => "overlay",
        LayerBlendMode::SoftLight => "soft_light",
        LayerBlendMode::HardLight => "hard_light",
        LayerBlendMode::ColorDodge => "color_dodge",
        LayerBlendMode::ColorBurn => "color_burn",
        LayerBlendMode::Difference => "difference",
        LayerBlendMode::Exclusion => "exclusion",
    }
}
fn blend_mode(name: &str) -> Result<LayerBlendMode, String> {
    match name {
        "normal" => Ok(LayerBlendMode::Normal),
        "multiply" => Ok(LayerBlendMode::Multiply),
        "screen" => Ok(LayerBlendMode::Screen),
        "overlay" => Ok(LayerBlendMode::Overlay),
        "soft_light" => Ok(LayerBlendMode::SoftLight),
        "hard_light" => Ok(LayerBlendMode::HardLight),
        "color_dodge" => Ok(LayerBlendMode::ColorDodge),
        "color_burn" => Ok(LayerBlendMode::ColorBurn),
        "difference" => Ok(LayerBlendMode::Difference),
        "exclusion" => Ok(LayerBlendMode::Exclusion),
        _ => Err("Unknown layer blend mode".into()),
    }
}
fn texture_effect_add(
    p: &mut PaintRecipe,
    fields: &mut PaintFieldDraft,
    kind: &str,
) -> Result<(), String> {
    if p.effects.len() >= 8
        || fields.intensities.len() != p.effects.len()
        || fields.secondaries.len() != p.effects.len()
    {
        return Err("At most eight ordered texture effects are supported".into());
    }
    let effect = texture_effect(kind, "1")?;
    // Adding a step is the explicit schema promotion. Loading, toggling, reordering
    // and removing steps never downgrade or rewrite the imported base/layers.
    p.effects.push(TextureEffectStep {
        enabled: true,
        effect,
    });
    fields.intensities.push("1".into());
    fields.secondaries.push("1".into());
    p.version = p.version.max(required_paint_version(p));
    Ok(())
}
#[cfg(test)]
fn texture_effect_update(
    p: &mut PaintRecipe,
    index: usize,
    kind: &str,
    intensity: &str,
) -> Result<(), String> {
    texture_effect_update_values(p, index, kind, intensity, "1")
}
fn texture_effect_update_values(
    p: &mut PaintRecipe,
    index: usize,
    kind: &str,
    primary: &str,
    secondary: &str,
) -> Result<(), String> {
    let effect = texture_effect_values(kind, primary, secondary)?;
    let step = p
        .effects
        .get_mut(index)
        .ok_or("Texture effect no longer exists")?;
    step.effect = effect;
    p.version = p.version.max(required_paint_version(p));
    Ok(())
}
#[cfg(test)]
fn texture_effect_select(
    p: &mut PaintRecipe,
    index: usize,
    kind: &str,
    intensity_text: &str,
) -> Result<(), String> {
    // Switching back to Sepia must reveal its retained invalid text so the user
    // can repair it. The typed placeholder cannot be submitted: recipe_for_apply
    // reparses the raw form, including disabled steps, before emitting any edit.
    texture_effect_select_values(p, index, kind, intensity_text, "1")
}
fn texture_effect_select_values(
    p: &mut PaintRecipe,
    index: usize,
    kind: &str,
    primary: &str,
    secondary: &str,
) -> Result<(), String> {
    // Keep the raw text unchanged when a different operation reveals invalid fields.
    // A typed placeholder lets the form show that operation; Apply reparses it.
    let effect = texture_effect_values(kind, primary, secondary)
        .or_else(|_| texture_effect_values(kind, "1", "1"))?;
    let step = p
        .effects
        .get_mut(index)
        .ok_or("Texture effect no longer exists")?;
    step.effect = effect;
    p.version = p.version.max(required_paint_version(p));
    Ok(())
}
fn texture_effect_move(
    p: &mut PaintRecipe,
    fields: &mut PaintFieldDraft,
    from: usize,
    to: usize,
) -> Result<(), String> {
    if fields.intensities.len() != p.effects.len()
        || fields.secondaries.len() != p.effects.len()
        || from >= p.effects.len()
        || to >= p.effects.len()
    {
        return Err("Texture effect order no longer matches the draft".into());
    }
    p.effects.swap(from, to);
    fields.intensities.swap(from, to);
    fields.secondaries.swap(from, to);
    Ok(())
}
fn texture_effect_remove(
    p: &mut PaintRecipe,
    fields: &mut PaintFieldDraft,
    index: usize,
) -> Result<(), String> {
    if fields.intensities.len() != p.effects.len()
        || fields.secondaries.len() != p.effects.len()
        || index >= p.effects.len()
    {
        return Err("Texture effect no longer exists".into());
    }
    p.effects.remove(index);
    fields.intensities.remove(index);
    fields.secondaries.remove(index);
    Ok(())
}
fn detail_draft_ready(
    fresh: bool,
    loaded: bool,
    loaded_base: &Option<DetailRecipe>,
    accepted: &Option<DetailRecipe>,
    loaded_context: &str,
    context: &str,
) -> bool {
    fresh && loaded && loaded_base == accepted && loaded_context == context
}
fn paint_origin_fields(p: &PaintRecipe, base: &str, side: &str) -> Result<(), String> {
    let side = side
        .trim()
        .parse::<u32>()
        .map_err(|_| "Enter the exact whole image side")?;
    if side != p.side || base != p.base_sha256 {
        return Err("Imported base/side fields differ from this draft. Restore them or explicitly Start a new imported paint recipe before applying.".into());
    }
    Ok(())
}
fn recipe_for_apply(
    recipe: &DetailRecipe,
    fields: &PaintFieldDraft,
    stroke_values: Option<&[String]>,
    procedural_values: &[String],
) -> Result<DetailRecipe, String> {
    let mut next = recipe.clone();
    match &mut next {
        DetailRecipe::Paint(p) => {
            if fields.opacities.len() != p.layers.len()
                || fields.layers.len() != p.layers.len()
                || fields.stroke_forms.len() != p.layers.len()
                || fields.intensities.len() != p.effects.len()
                || fields.secondaries.len() != p.effects.len()
            {
                return Err("Texture fields no longer match the draft; reload explicitly".into());
            }
            for ((layer, text), raw) in p
                .layers
                .iter_mut()
                .zip(&fields.opacities)
                .zip(&fields.layers)
            {
                layer.opacity = unit_value(text)?;
                let (blend_mode, offset, mask) = raw.parse()?;
                layer.blend_mode = blend_mode;
                layer.offset = offset;
                layer.mask = mask;
            }
            for ((step, primary), secondary) in p
                .effects
                .iter_mut()
                .zip(&fields.intensities)
                .zip(&fields.secondaries)
            {
                // Disabled steps have the same parameter contract.
                step.effect =
                    texture_effect_values(texture_effect_kind(&step.effect), primary, secondary)?;
            }
            p.version = p.version.max(required_paint_version(p));
            if let Some(values) = stroke_values {
                let s = stroke(values)?;
                // Validate the visible stroke form through the domain owner even
                // if the user has not pressed Add/Replace. Never submit old values
                // while an invalid numeric draft is still displayed.
                let mut probe = p.clone();
                probe.layers = vec![PaintLayer {
                    name: "Stroke field validation".into(),
                    visible: true,
                    opacity: 1.0,
                    strokes: vec![s],
                    ..Default::default()
                }];
                DetailRecipe::Paint(probe).validate(DetailLimits::default())?;
            }
        }
        DetailRecipe::Procedural(p) => {
            let DetailRecipe::Procedural(parsed) = procedural(procedural_values)? else {
                unreachable!()
            };
            *p = parsed;
        }
    }
    next.validate(DetailLimits::default())?;
    next.source_expression()?;
    Ok(next)
}

#[component]
pub(super) fn DetailControls(
    detail: Option<DetailRecipe>,
    context_token: String,
    fresh: bool,
    preview_uri: Option<String>,
    onapply: EventHandler<Option<DetailRecipe>>,
    oninspect: EventHandler<()>,
) -> Element {
    let mut loaded = use_signal(|| false);
    let mut loaded_context = use_signal(String::new);
    let mut loaded_base = use_signal(|| None::<DetailRecipe>);
    let mut draft = use_signal(|| None::<DetailRecipe>);
    let mut paint_fields = use_signal(PaintFieldDraft::default);
    let mut add_effect_kind = use_signal(|| "grayscale".to_owned());
    let mut layer_index = use_signal(|| 0usize);
    let mut stroke_index = use_signal(|| 0usize);
    let mut base_digest = use_signal(String::new);
    let mut side = use_signal(|| "64".to_owned());
    let mut fields = use_signal(|| {
        vec![
            "8".into(),
            "255,64,16".into(),
            "1".into(),
            "0.8".into(),
            "32,32".into(),
        ]
    });
    let mut procedural_values = use_signal(|| {
        vec![
            "64", "7", "1", "1", "1", "0.4", "0.01", "8", "1", "0.7", "0.8", "0",
        ]
        .into_iter()
        .map(str::to_owned)
        .collect::<Vec<_>>()
    });
    let mut message = use_signal(String::new);
    let ready = detail_draft_ready(
        fresh,
        loaded(),
        &loaded_base(),
        &detail,
        &loaded_context(),
        &context_token,
    );
    let may_start = fresh && (!loaded() || ready);
    let reload_context = context_token.clone();
    let paint_context = context_token.clone();
    let procedural_context = context_token.clone();
    let reload_detail = detail.clone();
    let paint_base = detail.clone();
    let proc_base = detail.clone();
    rsx! {
        fieldset {style:"margin-top:10px;border:1px solid #405264;",
            legend {"Editable texture and detail source"}
            p {"Paint edits opaque imported base colour at its exact square size. Ordered layers use encoded sRGB sample compositing, discrete pixel dabs and +Y down; this is not 3D painting or linear-light compositing. Procedural maps are artistic colour, tangent normal and packed ORM, not measured physical properties."}
            button {disabled:!fresh,onclick:move |_|{
                loaded_base.set(reload_detail.clone());loaded_context.set(reload_context.clone());draft.set(reload_detail.clone());loaded.set(true);layer_index.set(0);stroke_index.set(0);
                paint_fields.set(match &reload_detail {Some(DetailRecipe::Paint(p))=>{
                    base_digest.set(p.base_sha256.clone());side.set(p.side.to_string());
                    fields.set(p.layers.first().and_then(|l|l.strokes.first()).map(stroke_fields).unwrap_or_else(||default_stroke_fields(p.side)));
                    PaintFieldDraft::from_recipe(p)
                },_=>PaintFieldDraft::default()});
                if let Some(DetailRecipe::Procedural(p))=&reload_detail {procedural_values.set(procedural_fields(p));}
                message.set("Current accepted recipe loaded; local edits are retained until explicit reload.".into());
            },"Load accepted detail recipe"}
            label {"Imported base SHA-256" input {value:"{base_digest}",maxlength:64,oninput:move |e|base_digest.set(e.value())}}
            label {"Exact square side" input {value:"{side}",maxlength:4,oninput:move |e|side.set(e.value())}}
            p {"Base and side fields start a new imported paint draft. Existing paint retains its immutable base; restore changed start fields or explicitly Start before Apply. Use Add/Replace to store stroke fields in the local recipe."}
            button {disabled:!may_start,onclick:move |_|{
                let next=side().parse::<u32>().map_err(|_|"Enter the exact whole image side".to_owned()).and_then(|side|{
                    let p=DetailRecipe::Paint(PaintRecipe {version:1,side,base_sha256:base_digest(),layers:vec![],effects:vec![]});p.validate(DetailLimits::default())?;Ok(p)
                });
                match next {Ok(p)=>{fields.set(default_stroke_fields(p.side()));loaded_base.set(paint_base.clone());loaded_context.set(paint_context.clone());draft.set(Some(p));paint_fields.set(PaintFieldDraft::default());loaded.set(true);layer_index.set(0);stroke_index.set(0);},Err(e)=>message.set(e)}
            },"Start imported paint recipe"}
            button {disabled:!may_start,onclick:move |_|match procedural(&procedural_values()) {
                Ok(p)=>{loaded_base.set(proc_base.clone());loaded_context.set(procedural_context.clone());draft.set(Some(p));paint_fields.set(PaintFieldDraft::default());loaded.set(true);},Err(e)=>message.set(e)
            },"Start procedural detail recipe"}
            if loaded() && !ready {p {"Source or selection changed. Your local draft is retained; reload the current accepted recipe before applying."}}
            if let Some(DetailRecipe::Paint(p))=draft() {
                p {"{p.layers.len()} ordered paint layers; original base remains immutable."}
                button {disabled:!ready||p.layers.len()>=8,onclick:move |_|{
                    if let Some(DetailRecipe::Paint(p))=draft.write().as_mut() {
                        let layer=PaintLayer {name:format!("Paint {}",p.layers.len()+1),visible:true,opacity:1.0,strokes:vec![],..Default::default()};
                        let raw=LayerFieldDraft::from_layer(&layer,p.side);
                        let form=StrokeFieldDraft::from_layer(&layer,p.side);
                        p.layers.push(layer);
                        let mut raw_fields=paint_fields.write();raw_fields.opacities.push("1".into());raw_fields.layers.push(raw);raw_fields.stroke_forms.push(form.clone());
                        layer_index.set(p.layers.len()-1);stroke_index.set(form.selected);fields.set(form.values);
                    }
                },"Add paint layer"}
                for (i,l) in p.layers.iter().enumerate() {
                    div {style:"padding:4px;border:1px solid #405264;",
                        button {disabled:!ready,onclick:move |_|{
                            let form=paint_fields().stroke_forms[i].clone();
                            layer_index.set(i);stroke_index.set(form.selected);fields.set(form.values);
                        },"Select layer {i}"}
                        input {disabled:!ready,value:"{l.name}",maxlength:80,aria_label:"Layer {i} name",oninput:move |e|if let Some(DetailRecipe::Paint(p))=draft.write().as_mut(){p.layers[i].name=e.value();}}
                        label {"Visible" input {r#type:"checkbox",disabled:!ready,checked:l.visible,onchange:move |e|if let Some(DetailRecipe::Paint(p))=draft.write().as_mut(){p.layers[i].visible=e.checked();}}}
                        label {"Layer opacity" input {disabled:!ready,value:"{paint_fields().opacities[i]}",maxlength:32,aria_label:"Layer {i} opacity",oninput:move |e|{
                            let text=e.value();paint_fields.write().opacities[i]=text.clone();
                            match unit_value(&text){Ok(v)=>if let Some(DetailRecipe::Paint(p))=draft.write().as_mut(){p.layers[i].opacity=v;},Err(e)=>message.set(e)}
                        }}}
                        label {"Blend mode" select {disabled:!ready,value:"{paint_fields().layers[i].blend_mode}",aria_label:"Layer {i} blend mode",onchange:move |e|paint_fields.write().layers[i].blend_mode=e.value(),
                            for name in BLEND_MODES {option {value:name,"{name}"}}
                        }}
                        label {"Offset X in pixels (signed)" input {disabled:!ready,value:"{paint_fields().layers[i].offset_x}",maxlength:32,aria_label:"Layer {i} offset X",oninput:move |e|paint_fields.write().layers[i].offset_x=e.value()}}
                        label {"Offset Y in pixels (signed, +Y down)" input {disabled:!ready,value:"{paint_fields().layers[i].offset_y}",maxlength:32,aria_label:"Layer {i} offset Y",oninput:move |e|paint_fields.write().layers[i].offset_y=e.value()}}
                        label {"Layer-local mask" select {disabled:!ready,value:"{paint_fields().layers[i].mask_kind}",aria_label:"Layer {i} mask type",onchange:move |e|paint_fields.write().layers[i].mask_kind=e.value(),
                            option {value:"none","None"} option {value:"rectangle","Rectangle"} option {value:"resource","HMC image resource"}
                        }}
                        if paint_fields().layers[i].mask_kind=="rectangle" {
                            for (j,label) in ["Mask X in pixels", "Mask Y in pixels", "Mask width in pixels", "Mask height in pixels", "Mask coverage byte (0 to 255)"].iter().enumerate() {
                                label {"{label}" input {disabled:!ready,value:"{paint_fields().layers[i].rectangle[j]}",maxlength:32,aria_label:"Layer {i} {label}",oninput:move |e|paint_fields.write().layers[i].rectangle[j]=e.value()}}
                            }
                        }
                        if paint_fields().layers[i].mask_kind=="resource" {
                            label {"Immutable mask HMC resource SHA-256" input {disabled:!ready,value:"{paint_fields().layers[i].resource_sha256}",maxlength:64,aria_label:"Layer {i} mask resource SHA-256",oninput:move |e|paint_fields.write().layers[i].resource_sha256=e.value()}}
                            label {"Mask resource channel" select {disabled:!ready,value:"{paint_fields().layers[i].resource_channel}",aria_label:"Layer {i} mask resource channel",onchange:move |e|paint_fields.write().layers[i].resource_channel=e.value(),option {value:"red","Red"} option {value:"alpha","Alpha"}}}
                            p {"Use an existing immutable HMC texture with this recipe's square extent. Resource resolution and authority are checked by the shared source owner."}
                        }
                        button {disabled:!ready||i==0,onclick:move |_|if let Some(DetailRecipe::Paint(p))=draft.write().as_mut(){
                            p.layers.swap(i,i-1);
                            let mut raw_fields=paint_fields.write();raw_fields.opacities.swap(i,i-1);raw_fields.layers.swap(i,i-1);raw_fields.stroke_forms.swap(i,i-1);
                            let form=raw_fields.stroke_forms[i-1].clone();layer_index.set(i-1);stroke_index.set(form.selected);fields.set(form.values);
                        },"Move layer up"}
                        button {disabled:!ready||i+1==p.layers.len(),onclick:move |_|if let Some(DetailRecipe::Paint(p))=draft.write().as_mut(){
                            p.layers.swap(i,i+1);
                            let mut raw_fields=paint_fields.write();raw_fields.opacities.swap(i,i+1);raw_fields.layers.swap(i,i+1);raw_fields.stroke_forms.swap(i,i+1);
                            let form=raw_fields.stroke_forms[i+1].clone();layer_index.set(i+1);stroke_index.set(form.selected);fields.set(form.values);
                        },"Move layer down"}
                        button {disabled:!ready,onclick:move |_|if let Some(DetailRecipe::Paint(p))=draft.write().as_mut(){
                            p.layers.remove(i);
                            let mut raw_fields=paint_fields.write();raw_fields.opacities.remove(i);raw_fields.layers.remove(i);raw_fields.stroke_forms.remove(i);
                            let form=raw_fields.stroke_forms.first().cloned().unwrap_or_else(||StrokeFieldDraft {values:default_stroke_fields(p.side),selected:0});
                            layer_index.set(0);stroke_index.set(form.selected);fields.set(form.values);
                        },"Remove layer"}
                    }
                }
                if let Some(layer)=p.layers.get(layer_index()) {
                    p {"Editing layer {layer_index}: {layer.name}"}
                    for (i,s) in layer.strokes.iter().enumerate() {
                        div {
                            button {disabled:!ready,onclick:move |_|{
                                if let Some(DetailRecipe::Paint(p))=draft() {if let Some(s)=p.layers.get(layer_index()).and_then(|l|l.strokes.get(i)){
                                    let values=stroke_fields(s);let current=layer_index();
                                    paint_fields.write().stroke_forms[current]=StrokeFieldDraft {values:values.clone(),selected:i};
                                    fields.set(values);stroke_index.set(i);
                                }}
                            },"Load stroke {i} ({s.points.len()} dabs)"}
                            button {disabled:!ready||i==0,onclick:move |_|if let Some(DetailRecipe::Paint(p))=draft.write().as_mut(){
                                let current=layer_index();p.layers[current].strokes.swap(i,i-1);
                                let values=stroke_fields(&p.layers[current].strokes[i-1]);
                                paint_fields.write().stroke_forms[current]=StrokeFieldDraft {values:values.clone(),selected:i-1};
                                fields.set(values);stroke_index.set(i-1);
                                message.set("Stroke moved; the form now shows the moved stored stroke. Pending unstored stroke text was discarded.".into());
                            },"Move stroke up"}
                            button {disabled:!ready||i+1==layer.strokes.len(),onclick:move |_|if let Some(DetailRecipe::Paint(p))=draft.write().as_mut(){
                                let current=layer_index();p.layers[current].strokes.swap(i,i+1);
                                let values=stroke_fields(&p.layers[current].strokes[i+1]);
                                paint_fields.write().stroke_forms[current]=StrokeFieldDraft {values:values.clone(),selected:i+1};
                                fields.set(values);stroke_index.set(i+1);
                                message.set("Stroke moved; the form now shows the moved stored stroke. Pending unstored stroke text was discarded.".into());
                            },"Move stroke down"}
                            button {disabled:!ready,onclick:move |_|if let Some(DetailRecipe::Paint(p))=draft.write().as_mut(){
                                let current=layer_index();p.layers[current].strokes.remove(i);
                                let form=StrokeFieldDraft::from_layer(&p.layers[current],p.side);
                                paint_fields.write().stroke_forms[current]=form.clone();
                                fields.set(form.values);stroke_index.set(form.selected);
                                message.set("Stroke removed; the form now shows the first remaining stored stroke. Pending unstored stroke text was discarded.".into());
                            },"Remove stroke"}
                        }
                    }
                    for (i,label) in STROKE_FIELDS.iter().enumerate() {label {"{label}" input {disabled:!ready,value:"{fields()[i]}",maxlength:32768,aria_label:"{label}",oninput:move |e|{
                        let value=e.value();fields.write()[i]=value.clone();
                        paint_fields.write().stroke_forms[layer_index()].values[i]=value;
                    }}}}
                    button {disabled:!ready||layer.strokes.len()>=128,onclick:move |_|match stroke(&fields()) {
                        Ok(s)=>if let Some(DetailRecipe::Paint(p))=draft.write().as_mut(){
                            let current=layer_index();p.layers[current].strokes.push(s);
                            let selected=p.layers[current].strokes.len()-1;
                            paint_fields.write().stroke_forms[current].selected=selected;stroke_index.set(selected);
                        },Err(e)=>message.set(e)
                    },"Add discrete stroke"}
                    button {disabled:!ready||layer.strokes.is_empty(),onclick:move |_|match stroke(&fields()) {
                        Ok(s)=>if let Some(DetailRecipe::Paint(p))=draft.write().as_mut(){if let Some(slot)=p.layers[layer_index()].strokes.get_mut(stroke_index()){*slot=s;}},Err(e)=>message.set(e)
                    },"Replace loaded stroke"}
                }
                fieldset {
                    legend {"Ordered post-composite texture effects"}
                    p {"Effects run after all paint layers. Grayscale and invert are full strength; sepia intensity is 0 to 1. Gaussian sigma is 0 to 16 image pixels, sharpen amount 0 to 4, hue/azimuth -360 to 360 degrees, and emboss strength 0 to 1. Disabled steps retain their position and parameters. New filters and layer features require version 3; existing version numbers never decrease."}
                    label {"Effect to add" select {disabled:!ready,value:"{add_effect_kind}",aria_label:"Texture effect to add",onchange:move |e|add_effect_kind.set(e.value()),
                        option {value:"grayscale","Grayscale"} option {value:"invert","Invert"} option {value:"sepia","Sepia"}
                        option {value:"gaussian_blur","Gaussian blur"} option {value:"sharpen","Sharpen"} option {value:"hue_rotate","Hue rotation"} option {value:"emboss","Emboss"}
                    }}
                    button {disabled:!ready||p.effects.len()>=8,onclick:move |_|{
                        if let Some(DetailRecipe::Paint(p))=draft.write().as_mut(){
                            match texture_effect_add(p,&mut paint_fields.write(),&add_effect_kind()){
                                Ok(())=>message.set(format!("Effect added to the local version {} paint draft; Apply submits the shared source edit.",p.version)),Err(e)=>message.set(e)
                            }
                        }
                    },if matches!(add_effect_kind().as_str(),"grayscale"|"invert"|"sepia") {"Add effect (paint version 2)"} else {"Add effect (paint version 3)"}}
                    for (i,step) in p.effects.iter().enumerate() {
                        div {style:"padding:4px;border:1px solid #405264;",
                            p {"Effect {i}: {texture_effect_kind(&step.effect)}"}
                            label {"Enabled" input {r#type:"checkbox",disabled:!ready,checked:step.enabled,aria_label:"Texture effect {i} enabled",onchange:move |e|{
                                if let Some(DetailRecipe::Paint(p))=draft.write().as_mut(){p.effects[i].enabled=e.checked();}
                            }}}
                            label {"Operation" select {disabled:!ready,value:texture_effect_kind(&step.effect),aria_label:"Texture effect {i} operation",onchange:move |e|{
                                if let Some(DetailRecipe::Paint(p))=draft.write().as_mut(){
                                    if let Err(e)=texture_effect_select_values(p,i,&e.value(),&paint_fields().intensities[i],&paint_fields().secondaries[i]){message.set(e);}
                                }
                            },option {value:"grayscale","Grayscale"} option {value:"invert","Invert"} option {value:"sepia","Sepia"}
                                option {value:"gaussian_blur","Gaussian blur"} option {value:"sharpen","Sharpen"} option {value:"hue_rotate","Hue rotation"} option {value:"emboss","Emboss"}
                            }}
                            if !matches!(&step.effect,TextureEffect::Grayscale|TextureEffect::Invert) {
                                label {"{effect_primary_label(&step.effect)}" input {disabled:!ready,value:"{paint_fields().intensities[i]}",maxlength:32,aria_label:"Texture effect {i} {effect_primary_key(&step.effect)}",oninput:move |e|{
                                    let text=e.value();paint_fields.write().intensities[i]=text.clone();
                                    if let Some(DetailRecipe::Paint(p))=draft.write().as_mut(){let kind=texture_effect_kind(&p.effects[i].effect);if let Err(e)=texture_effect_update_values(p,i,kind,&text,&paint_fields().secondaries[i]){message.set(e);}}
                                }}}
                            }
                            if matches!(&step.effect,TextureEffect::Sharpen {..}|TextureEffect::Emboss {..}) {
                                label {if matches!(&step.effect,TextureEffect::Sharpen {..}) {"Sharpen amount (0 to 4)"} else {"Emboss strength (0 to 1)"}
                                    input {disabled:!ready,value:"{paint_fields().secondaries[i]}",maxlength:32,aria_label:"Texture effect {i} secondary amount or strength",oninput:move |e|{
                                        let text=e.value();paint_fields.write().secondaries[i]=text.clone();
                                        if let Some(DetailRecipe::Paint(p))=draft.write().as_mut(){let kind=texture_effect_kind(&p.effects[i].effect);if let Err(e)=texture_effect_update_values(p,i,kind,&paint_fields().intensities[i],&text){message.set(e);}}
                                    }}
                                }
                            }
                            button {disabled:!ready||i==0,onclick:move |_|if let Some(DetailRecipe::Paint(p))=draft.write().as_mut(){if let Err(e)=texture_effect_move(p,&mut paint_fields.write(),i,i-1){message.set(e);}},"Move effect up"}
                            button {disabled:!ready||i+1==p.effects.len(),onclick:move |_|if let Some(DetailRecipe::Paint(p))=draft.write().as_mut(){if let Err(e)=texture_effect_move(p,&mut paint_fields.write(),i,i+1){message.set(e);}},"Move effect down"}
                            button {disabled:!ready,onclick:move |_|if let Some(DetailRecipe::Paint(p))=draft.write().as_mut(){if let Err(e)=texture_effect_remove(p,&mut paint_fields.write(),i){message.set(e);}},"Remove effect"}
                        }
                    }
                }
            }
            if let Some(DetailRecipe::Procedural(_))=draft() {
                for (i,label) in PROCEDURAL_FIELDS.iter().enumerate() {label {"{label}" input {disabled:!ready,value:"{procedural_values()[i]}",maxlength:32,aria_label:"Procedural {label}",oninput:move |e|procedural_values.write()[i]=e.value()}}}
                button {disabled:!ready,onclick:move |_|match procedural(&procedural_values()){Ok(p)=>draft.set(Some(p)),Err(e)=>message.set(e)},"Update procedural draft"}
            }
            button {disabled:!ready||draft().is_none(),onclick:move |_|{
                if !ready {return;}
                if let Some(p)=draft(){
                    if let DetailRecipe::Paint(p)=&p {if let Err(e)=paint_origin_fields(p,&base_digest(),&side()){message.set(e);return;}}
                    let stroke_values=match &p{DetailRecipe::Paint(p) if p.layers.get(layer_index()).is_some()=>Some(fields()),_=>None};
                    match recipe_for_apply(&p,&paint_fields(),stroke_values.as_deref(),&procedural_values()){
                        Ok(p)=>onapply.call(Some(p)),Err(e)=>message.set(e)
                    }
                }
            },"Apply shared texture/detail recipe"}
            button {disabled:!ready,onclick:move |_|onapply.call(None),"Remove detail recipe; restore authored map bindings"}
            button {disabled:!fresh,onclick:move |_|oninspect.call(()),"Inspect accepted texture pixels"}
            if fresh {if let Some(uri)=preview_uri {img {src:uri,alt:"Current accepted derived base-colour texture pixels",style:"display:block;max-width:100%;width:320px;image-rendering:pixelated;"}}}
            p {role:"status","{message}"}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn texture_effect_paint() -> PaintRecipe {
        PaintRecipe {
            version: 1,
            side: 64,
            base_sha256: "1".repeat(64),
            effects: vec![],
            layers: vec![PaintLayer {
                name: "Existing paint".into(),
                visible: false,
                opacity: 0.4,
                strokes: vec![stroke(&default_stroke_fields(64)).unwrap()],
                ..Default::default()
            }],
        }
    }
    #[test]
    fn image_completed_controls_four_filters_roundtrip_fields_and_preserve_paint() {
        let original = texture_effect_paint();
        let cases = [
            (
                "gaussian_blur",
                "1.5",
                "1",
                TextureEffect::GaussianBlur { sigma: 1.5 },
            ),
            (
                "sharpen",
                "2",
                "3.5",
                TextureEffect::Sharpen {
                    sigma: 2.0,
                    amount: 3.5,
                },
            ),
            (
                "hue_rotate",
                "-90",
                "1",
                TextureEffect::HueRotate { degrees: -90.0 },
            ),
            (
                "emboss",
                "135",
                "0.25",
                TextureEffect::Emboss {
                    azimuth_degrees: 135.0,
                    strength: 0.25,
                },
            ),
        ];
        let mut p = original.clone();
        let mut fields = PaintFieldDraft::from_recipe(&p);
        for (kind, primary, secondary, expected) in cases {
            texture_effect_add(&mut p, &mut fields, kind).unwrap();
            let i = p.effects.len() - 1;
            fields.intensities[i] = primary.into();
            fields.secondaries[i] = secondary.into();
            texture_effect_update_values(&mut p, i, kind, primary, secondary).unwrap();
            assert_eq!(p.effects[i].effect, expected);
            assert_eq!(texture_effect_kind(&p.effects[i].effect), kind);
        }
        p.effects[1].enabled = false;
        let DetailRecipe::Paint(applied) =
            recipe_for_apply(&DetailRecipe::Paint(p.clone()), &fields, None, &[]).unwrap()
        else {
            panic!("paint")
        };
        assert_eq!(applied, p);
        assert_eq!(applied.version, 3);
        assert_eq!(applied.layers, original.layers);
        assert_eq!(applied.base_sha256, original.base_sha256);
        assert_eq!(applied.side, original.side);
        assert_eq!(PaintFieldDraft::from_recipe(&applied), fields);
        let source = DetailRecipe::Paint(applied).source_expression().unwrap();
        for kind in ["gaussian_blur", "sharpen", "hue_rotate", "emboss"] {
            assert!(source.contains(kind));
        }
    }
    #[test]
    fn image_completed_controls_filter_ranges_and_disabled_invalid_text_refuse_old_values() {
        for (kind, primary, secondary) in [
            ("gaussian_blur", "16.01", "1"),
            ("gaussian_blur", "-0.01", "1"),
            ("sharpen", "1", "4.01"),
            ("sharpen", "1", "-"),
            ("hue_rotate", "360.01", "1"),
            ("hue_rotate", "-360.01", "1"),
            ("emboss", "NaN", "1"),
            ("emboss", "135", "1.01"),
            ("emboss", "135", "-0.01"),
            ("sharpen", "inf", "0"),
        ] {
            let mut p = texture_effect_paint();
            let mut fields = PaintFieldDraft::from_recipe(&p);
            texture_effect_add(&mut p, &mut fields, kind).unwrap();
            p.effects[0].enabled = false;
            fields.intensities[0] = primary.into();
            fields.secondaries[0] = secondary.into();
            let before = p.clone();
            assert!(texture_effect_update_values(&mut p, 0, kind, primary, secondary).is_err());
            assert_eq!(p, before);
            assert!(recipe_for_apply(&DetailRecipe::Paint(p), &fields, None, &[]).is_err());
            assert_eq!(fields.intensities[0], primary);
            assert_eq!(fields.secondaries[0], secondary);
        }
        for (kind, primary, secondary) in [
            ("gaussian_blur", "0", "1"),
            ("gaussian_blur", "16", "1"),
            ("sharpen", "16", "4"),
            ("sharpen", "0", "0"),
            ("hue_rotate", "-360", "1"),
            ("hue_rotate", "360", "1"),
            ("emboss", "-360", "0"),
            ("emboss", "360", "1"),
        ] {
            assert!(texture_effect_values(kind, primary, secondary).is_ok());
        }
    }
    #[test]
    fn image_completed_controls_all_blends_offsets_and_rectangle_masks_keep_source_layers() {
        let original = texture_effect_paint();
        for name in BLEND_MODES {
            let p = original.clone();
            let mut fields = PaintFieldDraft::from_recipe(&p);
            let raw = &mut fields.layers[0];
            raw.blend_mode = name.into();
            raw.offset_x = "-7".into();
            raw.offset_y = "9".into();
            raw.mask_kind = "rectangle".into();
            raw.rectangle = ["2".into(), "3".into(), "4".into(), "5".into(), "128".into()];
            let DetailRecipe::Paint(applied) =
                recipe_for_apply(&DetailRecipe::Paint(p), &fields, None, &[]).unwrap()
            else {
                panic!("paint")
            };
            assert_eq!(applied.version, 3);
            assert_eq!(blend_mode_name(applied.layers[0].blend_mode), name);
            assert_eq!(applied.layers[0].offset, [-7, 9]);
            assert_eq!(
                applied.layers[0].mask,
                Some(LayerMask::Rectangle {
                    x: 2,
                    y: 3,
                    width: 4,
                    height: 5,
                    coverage: 128
                })
            );
            assert_eq!(applied.layers[0].strokes, original.layers[0].strokes);
            assert_eq!(applied.layers[0].name, original.layers[0].name);
            assert!(!applied.layers[0].visible);
            assert_eq!(applied.layers[0].opacity, original.layers[0].opacity);
            assert_eq!(applied.base_sha256, original.base_sha256);
            assert_eq!(
                PaintFieldDraft::from_recipe(&applied).layers[0],
                fields.layers[0]
            );
        }
    }
    #[test]
    fn image_completed_controls_mask_channels_raw_refusal_and_hidden_bounds() {
        let mut p = texture_effect_paint();
        p.version = 3;
        for channel in [MaskChannel::Red, MaskChannel::Alpha] {
            p.layers[0].mask = Some(LayerMask::Resource {
                sha256: "a".repeat(64),
                channel,
            });
            let fields = PaintFieldDraft::from_recipe(&p);
            assert_eq!(
                recipe_for_apply(&DetailRecipe::Paint(p.clone()), &fields, None, &[]).unwrap(),
                DetailRecipe::Paint(p.clone())
            );
        }
        let valid = PaintFieldDraft::from_recipe(&p);
        for (field, invalid) in [
            ("offset", "-"),
            ("offset", "2147483648"),
            ("blend", "unknown"),
            ("sha", "bad"),
            ("channel", "green"),
            ("mask", "unknown"),
        ] {
            let mut fields = valid.clone();
            let raw = &mut fields.layers[0];
            match field {
                "offset" => raw.offset_x = invalid.into(),
                "blend" => raw.blend_mode = invalid.into(),
                "sha" => raw.resource_sha256 = invalid.into(),
                "channel" => raw.resource_channel = invalid.into(),
                _ => raw.mask_kind = invalid.into(),
            }
            let retained = fields.clone();
            assert!(recipe_for_apply(&DetailRecipe::Paint(p.clone()), &fields, None, &[]).is_err());
            assert_eq!(fields, retained);
        }
        for rectangle in [
            ["0", "0", "0", "1", "255"],
            ["63", "0", "2", "1", "255"],
            ["0", "-1", "1", "1", "255"],
            ["0", "0", "1", "1", "256"],
        ] {
            let mut fields = valid.clone();
            fields.layers[0].mask_kind = "rectangle".into();
            fields.layers[0].rectangle = rectangle.map(str::to_owned);
            assert!(recipe_for_apply(&DetailRecipe::Paint(p.clone()), &fields, None, &[]).is_err());
        }
        let mut fields = valid;
        fields.layers[0].offset_x = i32::MIN.to_string();
        fields.layers[0].offset_y = i32::MAX.to_string();
        assert_eq!(fields.layers[0].parse().unwrap().1, [i32::MIN, i32::MAX]);
        fields.layers[0].mask_kind = "none".into();
        assert_eq!(fields.layers[0].parse().unwrap().2, None);
    }
    #[test]
    fn image_completed_controls_secondary_order_retention_and_no_version_downgrade() {
        let mut p = texture_effect_paint();
        p.version = 3;
        let mut fields = PaintFieldDraft::from_recipe(&p);
        texture_effect_add(&mut p, &mut fields, "invert").unwrap();
        assert_eq!(p.version, 3);
        texture_effect_add(&mut p, &mut fields, "emboss").unwrap();
        fields.intensities[1] = "-".into();
        fields.secondaries[1] = "invalid strength".into();
        texture_effect_select_values(
            &mut p,
            1,
            "invert",
            &fields.intensities[1],
            &fields.secondaries[1],
        )
        .unwrap();
        texture_effect_select_values(
            &mut p,
            1,
            "emboss",
            &fields.intensities[1],
            &fields.secondaries[1],
        )
        .unwrap();
        texture_effect_move(&mut p, &mut fields, 1, 0).unwrap();
        assert_eq!(fields.intensities[0], "-");
        assert_eq!(fields.secondaries[0], "invalid strength");
        assert!(recipe_for_apply(&DetailRecipe::Paint(p.clone()), &fields, None, &[]).is_err());
        texture_effect_remove(&mut p, &mut fields, 0).unwrap();
        texture_effect_remove(&mut p, &mut fields, 0).unwrap();
        assert_eq!(p.version, 3);
        assert!(fields.intensities.is_empty() && fields.secondaries.is_empty());
        assert!(recipe_for_apply(&DetailRecipe::Paint(p), &fields, None, &[]).is_ok());
    }
    #[test]
    fn texture_effect_controls_add_update_toggle_order_remove_preserve_legacy() {
        let original = texture_effect_paint();
        let legacy_expression = DetailRecipe::Paint(original.clone())
            .source_expression()
            .unwrap();
        let mut p = original.clone();
        let mut fields = PaintFieldDraft::from_recipe(&p);
        let unchanged =
            recipe_for_apply(&DetailRecipe::Paint(p.clone()), &fields, None, &[]).unwrap();
        assert_eq!(unchanged.source_expression().unwrap(), legacy_expression);
        assert_eq!(p.version, 1);
        texture_effect_add(&mut p, &mut fields, "invert").unwrap();
        texture_effect_add(&mut p, &mut fields, "sepia").unwrap();
        fields.intensities[1] = "0.25".into();
        texture_effect_update(&mut p, 1, "sepia", &fields.intensities[1]).unwrap();
        p.effects[1].enabled = false;
        texture_effect_move(&mut p, &mut fields, 1, 0).unwrap();
        assert_eq!(p.version, 2);
        assert!(!p.effects[0].enabled);
        assert_eq!(
            p.effects[0].effect,
            TextureEffect::Sepia { intensity: 0.25 }
        );
        assert_eq!(p.effects[1].effect, TextureEffect::Invert);
        assert_eq!(fields.intensities, ["0.25", "1"]);
        texture_effect_remove(&mut p, &mut fields, 1).unwrap();
        assert_eq!(p.layers, original.layers);
        assert_eq!(p.base_sha256, original.base_sha256);
        assert_eq!(p.side, original.side);
        let applied =
            recipe_for_apply(&DetailRecipe::Paint(p.clone()), &fields, None, &[]).unwrap();
        assert_eq!(applied, DetailRecipe::Paint(p.clone()));
        texture_effect_remove(&mut p, &mut fields, 0).unwrap();
        assert_eq!(p.version, 2);
        assert!(p.effects.is_empty());
        assert_eq!(p.layers, original.layers);
    }
    #[test]
    fn texture_effect_controls_invalid_text_retains_form_and_refuses_apply() {
        let mut p = texture_effect_paint();
        let mut fields = PaintFieldDraft::from_recipe(&p);
        texture_effect_add(&mut p, &mut fields, "sepia").unwrap();
        p.effects[0].enabled = false;
        for invalid in ["", "-", "NaN", "inf", "-0.1", "1.1", "not a number"] {
            fields.intensities[0] = invalid.into();
            let before = p.clone();
            assert!(texture_effect_update(&mut p, 0, "sepia", invalid).is_err());
            assert_eq!(p, before);
            assert!(recipe_for_apply(&DetailRecipe::Paint(p.clone()), &fields, None, &[]).is_err());
            assert_eq!(fields.intensities[0], invalid);
        }
        fields.intensities[0] = "0.75".into();
        let applied =
            recipe_for_apply(&DetailRecipe::Paint(p.clone()), &fields, None, &[]).unwrap();
        let DetailRecipe::Paint(applied) = applied else {
            panic!("expected paint")
        };
        assert_eq!(
            applied.effects[0].effect,
            TextureEffect::Sepia { intensity: 0.75 }
        );
        fields.opacities[0] = "invalid opacity".into();
        assert!(recipe_for_apply(&DetailRecipe::Paint(p), &fields, None, &[]).is_err());
        assert_eq!(fields.opacities[0], "invalid opacity");
    }
    #[test]
    fn texture_effect_controls_kind_switch_reveals_retained_invalid_sepia() {
        let mut p = texture_effect_paint();
        let mut fields = PaintFieldDraft::from_recipe(&p);
        texture_effect_add(&mut p, &mut fields, "sepia").unwrap();
        fields.intensities[0] = "invalid".into();
        texture_effect_select(&mut p, 0, "invert", &fields.intensities[0]).unwrap();
        texture_effect_select(&mut p, 0, "sepia", &fields.intensities[0]).unwrap();
        assert_eq!(texture_effect_kind(&p.effects[0].effect), "sepia");
        assert_eq!(fields.intensities[0], "invalid");
        assert!(recipe_for_apply(&DetailRecipe::Paint(p), &fields, None, &[]).is_err());
    }
    #[test]
    fn texture_effect_controls_bound_and_unknown_edits_are_atomic() {
        let mut p = texture_effect_paint();
        let mut fields = PaintFieldDraft::from_recipe(&p);
        let before = p.clone();
        assert!(texture_effect_add(&mut p, &mut fields, "blur").is_err());
        assert_eq!(p, before);
        for _ in 0..8 {
            texture_effect_add(&mut p, &mut fields, "grayscale").unwrap();
        }
        let before = p.clone();
        let before_fields = fields.clone();
        assert!(texture_effect_add(&mut p, &mut fields, "invert").is_err());
        assert!(texture_effect_move(&mut p, &mut fields, 0, 8).is_err());
        assert!(texture_effect_remove(&mut p, &mut fields, 8).is_err());
        assert!(texture_effect_update(&mut p, 0, "unknown", "1").is_err());
        assert_eq!(p, before);
        assert_eq!(fields, before_fields);
    }
    #[test]
    fn texture_effect_controls_changed_source_selection_or_context_requires_reload() {
        let accepted = Some(DetailRecipe::Paint(texture_effect_paint()));
        let mut local = accepted.clone();
        let mut fields = match local.as_ref().unwrap() {
            DetailRecipe::Paint(p) => PaintFieldDraft::from_recipe(p),
            _ => unreachable!(),
        };
        if let Some(DetailRecipe::Paint(p)) = &mut local {
            texture_effect_add(p, &mut fields, "sepia").unwrap();
        }
        fields.intensities[0] = "-".into();
        let retained = (local.clone(), fields.clone());
        assert!(detail_draft_ready(
            true,
            true,
            &accepted,
            &accepted,
            "session-1:part-A",
            "session-1:part-A"
        ));
        assert!(!detail_draft_ready(
            false,
            true,
            &accepted,
            &accepted,
            "session-1:part-A",
            "session-1:part-A"
        ));
        assert!(!detail_draft_ready(
            true,
            true,
            &accepted,
            &accepted,
            "session-1:part-A",
            "session-1:part-B"
        ));
        assert!(!detail_draft_ready(
            true,
            true,
            &accepted,
            &accepted,
            "session-1:part-A",
            "session-2:part-A"
        ));
        assert!(!detail_draft_ready(
            true,
            true,
            &accepted,
            &local,
            "session-1:part-A",
            "session-1:part-A"
        ));
        assert_eq!((local, fields), retained);
    }
    #[test]
    fn texture_effect_controls_invalid_stroke_or_procedural_form_refuses_old_recipe() {
        let p = texture_effect_paint();
        let fields = PaintFieldDraft::from_recipe(&p);
        let mut stroke_values = default_stroke_fields(64);
        stroke_values[0] = "invalid size".into();
        assert!(
            recipe_for_apply(&DetailRecipe::Paint(p), &fields, Some(&stroke_values), &[]).is_err()
        );
        let mut values = [
            "64", "7", "1", "1", "1", "0.4", "0.01", "8", "1", "0.7", "0.8", "0",
        ]
        .into_iter()
        .map(str::to_owned)
        .collect::<Vec<_>>();
        let p = procedural(&values).unwrap();
        values[5] = "NaN".into();
        assert!(recipe_for_apply(&p, &PaintFieldDraft::default(), None, &values).is_err());
    }
    #[test]
    fn texture_effect_controls_invalid_or_changed_origin_fields_refuse_old_recipe() {
        let p = texture_effect_paint();
        assert!(paint_origin_fields(&p, &p.base_sha256, "64").is_ok());
        assert!(paint_origin_fields(&p, &p.base_sha256, "NaN").is_err());
        assert!(paint_origin_fields(&p, &p.base_sha256, "32").is_err());
        assert!(paint_origin_fields(&p, &"2".repeat(64), "64").is_err());
    }
    #[test]
    fn explicit_dab_fields_preserve_order_and_reject_invalid_coordinates() {
        let fields = vec![
            "4".into(),
            "255,0,10".into(),
            "0.5".into(),
            "1".into(),
            "2,3;4,5".into(),
        ];
        let s = stroke(&fields).unwrap();
        assert_eq!(s.points, vec![[2.0, 3.0], [4.0, 5.0]]);
        assert_eq!(stroke(&stroke_fields(&s)).unwrap(), s);
        let mut invalid = fields;
        invalid[4] = "NaN,2".into();
        assert!(stroke(&invalid).is_err());
    }
}

#[cfg(test)]
#[path = "texture_effect_component_tests.rs"]
mod component_tests;
