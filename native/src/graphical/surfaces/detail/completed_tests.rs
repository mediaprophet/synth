use super::*;
use qualia_core_db::render::{
    asset_package::HmcTextureResource,
    texture_decode::{
        decode_hmc_texture_rgba8_into, inspect_hmc_texture_requirements, TextureColorSpace,
        TextureDecodeLimits,
    },
};

fn paint() -> PaintRecipe {
    PaintRecipe {
        version: 3,
        side: 8,
        base_sha256: "11".repeat(32),
        layers: Vec::new(),
        effects: Vec::new(),
    }
}
fn stroke() -> PaintStroke {
    PaintStroke {
        size: 2.0,
        colour: [255, 0, 0, 255],
        opacity: 1.0,
        hardness: 1.0,
        points: vec![[3.0, 3.0]],
    }
}
fn layer() -> PaintLayer {
    PaintLayer {
        name: "Ink".into(),
        strokes: vec![stroke()],
        ..PaintLayer::default()
    }
}
fn base<'a>(bytes: &'a [u8]) -> DetailBase<'a> {
    DetailBase {
        digest: [0x11; 32],
        width: 8,
        height: 8,
        rgba: bytes,
        colour_space: TextureColorSpace::Srgb,
    }
}
fn pixels(output: &DetailOutput) -> Vec<u8> {
    let dep = &output.dependencies[0];
    let resource = HmcTextureResource {
        digest: dep.digest,
        mime_type: &dep.mime_type,
        bytes: &dep.bytes,
    };
    let limits = TextureDecodeLimits::default();
    let (info, scratch_len) = inspect_hmc_texture_requirements(&resource, limits).unwrap();
    let mut rgba = vec![0; info.rgba8_bytes];
    let mut scratch = vec![0; scratch_len];
    decode_hmc_texture_rgba8_into(&resource, limits, &mut rgba, &mut scratch).unwrap();
    rgba
}
fn pixel(bytes: &[u8], x: usize, y: usize) -> [u8; 4] {
    bytes[(y * 8 + x) * 4..(y * 8 + x + 1) * 4]
        .try_into()
        .unwrap()
}

#[test]
fn image_completed_domain_v3_rectangle_offset_source_and_pixels() {
    let mut p = paint();
    let mut l = layer();
    l.offset = [1, 0];
    l.mask = Some(LayerMask::Rectangle {
        x: 3,
        y: 3,
        width: 1,
        height: 1,
        coverage: 255,
    });
    p.layers.push(l);
    assert_eq!(required_paint_version(&p), 3);
    let recipe = DetailRecipe::Paint(p.clone());
    let expression = recipe.source_expression().unwrap();
    assert!(expression.contains("mask:"));
    let source = format!("({expression});");
    let value = vibe::session::SourceSession::default()
        .submit(
            &source,
            &mut vibe::LocalHost::default(),
            vibe::Budget::default(),
        )
        .unwrap()
        .value;
    assert_eq!(DetailRecipe::from_value(&value).unwrap(), recipe);
    let original = [20, 30, 40, 255].repeat(64);
    let output = render_detail(&recipe, Some(base(&original)), DetailLimits::default()).unwrap();
    let rendered = pixels(&output);
    assert_eq!(pixel(&rendered, 4, 3), [255, 0, 0, 255]);
    assert_eq!(pixel(&rendered, 3, 3), [20, 30, 40, 255]);
    assert_eq!(original, [20, 30, 40, 255].repeat(64));
}

#[test]
fn image_completed_domain_resource_mask_red_alpha_and_missing_refusal() {
    let original = [20, 30, 40, 255].repeat(64);
    let mut mask_bytes = [0u8; 8 * 8 * 4];
    mask_bytes[(3 * 8 + 3) * 4] = 255;
    mask_bytes[(3 * 8 + 3) * 4 + 3] = 0;
    let mask = DetailMaskBase {
        digest: [0x22; 32],
        width: 8,
        height: 8,
        rgba: &mask_bytes,
    };
    for (channel, expected) in [
        (MaskChannel::Red, [255, 0, 0, 255]),
        (MaskChannel::Alpha, [20, 30, 40, 255]),
    ] {
        let mut p = paint();
        let mut l = layer();
        l.mask = Some(LayerMask::Resource {
            sha256: "22".repeat(32),
            channel,
        });
        p.layers.push(l);
        let recipe = DetailRecipe::Paint(p);
        assert!(render_detail(&recipe, Some(base(&original)), DetailLimits::default()).is_err());
        let output = render_detail_with_masks(
            &recipe,
            Some(base(&original)),
            &[DetailMaskBase {
                digest: mask.digest,
                width: mask.width,
                height: mask.height,
                rgba: mask.rgba,
            }],
            DetailLimits::default(),
        )
        .unwrap();
        assert_eq!(pixel(&pixels(&output), 3, 3), expected);
    }
}

#[test]
fn image_completed_domain_v3_filter_order_and_strict_version() {
    let original = [100, 150, 200, 255].repeat(64);
    let mut p = paint();
    p.effects = vec![
        TextureEffectStep {
            enabled: true,
            effect: TextureEffect::HueRotate { degrees: 180.0 },
        },
        TextureEffectStep {
            enabled: true,
            effect: TextureEffect::GaussianBlur { sigma: 0.0 },
        },
        TextureEffectStep {
            enabled: true,
            effect: TextureEffect::Sharpen {
                sigma: 1.0,
                amount: 0.0,
            },
        },
        TextureEffectStep {
            enabled: true,
            effect: TextureEffect::Emboss {
                azimuth_degrees: 90.0,
                strength: 0.0,
            },
        },
    ];
    let first = DetailRecipe::Paint(p.clone());
    let changed =
        pixels(&render_detail(&first, Some(base(&original)), DetailLimits::default()).unwrap());
    assert_ne!(changed, original);
    let mut plain = paint();
    plain.effects = p.effects[1..].to_vec();
    let identity = pixels(
        &render_detail(
            &DetailRecipe::Paint(plain),
            Some(base(&original)),
            DetailLimits::default(),
        )
        .unwrap(),
    );
    assert_eq!(identity, original);
    p.version = 2;
    assert!(DetailRecipe::Paint(p)
        .validate(DetailLimits::default())
        .is_err());
    for invalid in [
        serde_json::json!({"kind":"gaussian_blur","sigma":1.0,"unknown":1}),
        serde_json::json!({"kind":"sharpen","sigma":1.0}),
        serde_json::json!({"kind":"hue_rotate","degrees":400.0,"unknown":1}),
    ] {
        assert!(serde_json::from_value::<TextureEffect>(invalid).is_err());
    }
}

#[test]
fn image_completed_domain_blend_and_extreme_offset_use_core() {
    let original = [100, 150, 200, 255].repeat(64);
    let mut p = paint();
    let mut l = layer();
    l.blend_mode = LayerBlendMode::Multiply;
    p.layers.push(l);
    let recipe = DetailRecipe::Paint(p.clone());
    assert_eq!(
        pixel(
            &pixels(
                &render_detail(&recipe, Some(base(&original)), DetailLimits::default()).unwrap()
            ),
            3,
            3
        ),
        [100, 0, 0, 255]
    );
    p.layers[0].offset = [i32::MIN, i32::MAX];
    let clipped = pixels(
        &render_detail(
            &DetailRecipe::Paint(p),
            Some(base(&original)),
            DetailLimits::default(),
        )
        .unwrap(),
    );
    assert_eq!(clipped, original);
    assert!(serde_json::from_str::<LayerBlendMode>("\"unrecognised\"").is_err());
}
