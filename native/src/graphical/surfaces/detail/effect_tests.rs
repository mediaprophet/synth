use super::*;
use crate::graphical::surfaces::TextureRole;
use qualia_core_db::render::{
    asset_package::HmcTextureResource,
    texture_decode::{decode_hmc_texture_rgba8_into, inspect_hmc_texture_requirements,
        TextureColorSpace, TextureDecodeLimits},
};
use sha2::{Digest, Sha256};

fn recipe(effects: Vec<TextureEffectStep>) -> DetailRecipe {
    DetailRecipe::Paint(PaintRecipe {
        version: 2, side: 8, base_sha256: "11".repeat(32), layers: vec![], effects,
    })
}
fn step(effect: TextureEffect) -> TextureEffectStep {
    TextureEffectStep { enabled: true, effect }
}
fn base(rgba: &[u8]) -> DetailBase<'_> {
    DetailBase { digest: [0x11; 32], width: 8, height: 8, rgba,
        colour_space: TextureColorSpace::Srgb }
}
fn decode(output: &DetailOutput) -> Vec<u8> {
    assert_eq!(output.dependencies.len(), 1);
    let dep = &output.dependencies[0];
    assert_eq!(dep.mime_type, "image/png");
    assert_eq!(dep.digest, <[u8; 32]>::from(Sha256::digest(&dep.bytes)));
    assert_eq!(output.overrides, vec![(TextureRole::BaseColor, dep.digest)]);
    let resource = HmcTextureResource {
        digest: dep.digest, mime_type: &dep.mime_type, bytes: &dep.bytes,
    };
    let (info, scratch) = inspect_hmc_texture_requirements(
        &resource, TextureDecodeLimits::default()).unwrap();
    let mut rgba = vec![0; info.rgba8_bytes];
    let mut scratch = vec![0; scratch];
    decode_hmc_texture_rgba8_into(&resource, TextureDecodeLimits::default(),
        &mut rgba, &mut scratch).unwrap();
    rgba
}
fn render(recipe: &DetailRecipe, rgba: &[u8]) -> DetailOutput {
    render_detail(recipe, Some(base(rgba)), DetailLimits::default()).unwrap()
}

#[test]
fn texture_effect_real_filters_match_independent_expected_rgba_vectors() {
    let original = [100, 150, 200, 255].repeat(64);
    for (effect, expected) in [
        (TextureEffect::Grayscale, [140, 140, 140, 255]),
        (TextureEffect::Invert, [155, 105, 55, 255]),
        (TextureEffect::Sepia { intensity: 1.0 }, [192, 171, 133, 255]),
        (TextureEffect::Sepia { intensity: 0.5 }, [146, 160, 166, 255]),
        (TextureEffect::Sepia { intensity: 0.0 }, [100, 150, 200, 255]),
    ] {
        let recipe = recipe(vec![step(effect)]);
        let output = render(&recipe, &original);
        assert_eq!(decode(&output), expected.repeat(64));
        assert_eq!(output.base_digest, Some([0x11; 32]));
        assert_eq!(output.recipe_sha256, recipe.digest().unwrap());
        assert_eq!(output.dependencies, render(&recipe, &original).dependencies);
    }
    assert_eq!(original, [100, 150, 200, 255].repeat(64));
}

#[test]
fn texture_effect_order_disable_remove_and_source_immutability() {
    let original = [100, 150, 200, 255].repeat(64);
    let mut recipe = recipe(vec![step(TextureEffect::Invert),
        step(TextureEffect::Sepia { intensity: 1.0 })]);
    let retained = recipe.clone();
    assert_eq!(decode(&render(&recipe, &original)), [152, 135, 105, 255].repeat(64));
    assert_eq!(recipe, retained);
    let DetailRecipe::Paint(p) = &mut recipe else { unreachable!() };
    p.effects.swap(0, 1);
    assert_eq!(decode(&render(&recipe, &original)), [63, 84, 122, 255].repeat(64));
    let DetailRecipe::Paint(p) = &mut recipe else { unreachable!() };
    p.effects[0].enabled = false;
    let disabled = p.effects[0].clone();
    assert_eq!(decode(&render(&recipe, &original)), [155, 105, 55, 255].repeat(64));
    let DetailRecipe::Paint(p) = &mut recipe else { unreachable!() };
    assert_eq!(p.effects[0], disabled);
    p.effects.remove(0);
    assert_eq!(decode(&render(&recipe, &original)), [155, 105, 55, 255].repeat(64));
    let DetailRecipe::Paint(p) = &mut recipe else { unreachable!() };
    p.effects.clear();
    assert_eq!(decode(&render(&recipe, &original)), original);
}

#[test]
fn texture_effect_runs_after_paint_composite_and_retains_layers() {
    let original = [0, 0, 0, 255].repeat(64);
    let mut recipe = recipe(vec![step(TextureEffect::Invert)]);
    let DetailRecipe::Paint(p) = &mut recipe else { unreachable!() };
    p.layers.push(PaintLayer { name: "Detail".into(), visible: true, opacity: 1.0,
        strokes: vec![PaintStroke { size: 4.0, colour: [255, 0, 0, 255],
            opacity: 0.5, hardness: 1.0, points: vec![[4.0, 4.0]] }],
        ..crate::graphical::surfaces::detail::PaintLayer::default()
    });
    let retained = recipe.clone();
    let result = decode(&render(&recipe, &original));
    assert_eq!(&result[144..148], &[127, 255, 255, 255]);
    assert_eq!(&result[..4], &[255, 255, 255, 255]);
    assert_eq!(recipe, retained);
    assert_eq!(original, [0, 0, 0, 255].repeat(64));
}

#[test]
fn texture_effect_legacy_expression_and_version_admission() {
    let legacy_json = serde_json::json!({"kind":"paint", "version":1, "side":8,
        "base_sha256":"11".repeat(32), "layers":[]});
    let legacy: DetailRecipe = serde_json::from_value(legacy_json.clone()).unwrap();
    let expected = format!("{{base_sha256:\"{}\",kind:\"paint\",layers:[],side:8,version:1}}", "11".repeat(32));
    assert_eq!(legacy.source_expression().unwrap(), expected);
    assert_eq!(serde_json::to_value(&legacy).unwrap(), legacy_json);
    let source = format!("({});", legacy.source_expression().unwrap());
    let value = vibe::session::SourceSession::default().submit(&source,
        &mut vibe::LocalHost::default(), vibe::Budget::default()).unwrap().value;
    assert_eq!(DetailRecipe::from_value(&value).unwrap(), legacy);
    let mut with_effect = recipe(vec![step(TextureEffect::Invert)]);
    for version in [0, 1, 4, u32::MAX] {
        let DetailRecipe::Paint(p) = &mut with_effect else { unreachable!() };
        p.version = version;
        assert!(with_effect.validate(DetailLimits::default()).is_err());
    }
    assert!(recipe(vec![]).validate(DetailLimits::default()).is_ok());
    let authored = recipe(vec![step(TextureEffect::Grayscale), step(TextureEffect::Invert),
        TextureEffectStep { enabled: false, effect: TextureEffect::Sepia { intensity: 0.75 } }]);
    let source = format!("({});", authored.source_expression().unwrap());
    let value = vibe::session::SourceSession::default().submit(&source,
        &mut vibe::LocalHost::default(), vibe::Budget::default()).unwrap().value;
    assert_eq!(DetailRecipe::from_value(&value).unwrap(), authored);
    let mut empty_explicit = legacy_json;
    empty_explicit["effects"] = serde_json::json!([]);
    let empty_explicit: DetailRecipe = serde_json::from_value(empty_explicit).unwrap();
    assert_eq!(empty_explicit.source_expression().unwrap(), expected);
}

#[test]
fn texture_effect_strict_fields_kinds_required_fields_and_parameters() {
    for invalid in [
        serde_json::json!({"enabled":true,"effect":{"kind":"blur"}}),
        serde_json::json!({"enabled":true,"effect":{"kind":"invert","intensity":0.5}}),
        serde_json::json!({"enabled":true,"effect":{"kind":"grayscale","extra":0}}),
        serde_json::json!({"enabled":true,"effect":{"kind":"sepia","intensity":0.5,"extra":0}}),
        serde_json::json!({"enabled":true,"effect":{"kind":"sepia"}}),
        serde_json::json!({"effect":{"kind":"invert"}}),
        serde_json::json!({"enabled":true,"effect":{"kind":"invert"},"extra":0}),
    ] {
        assert!(serde_json::from_value::<TextureEffectStep>(invalid).is_err());
    }
    for intensity in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY, -0.01, 1.01] {
        for enabled in [true, false] {
            let effects = vec![TextureEffectStep { enabled,
                effect: TextureEffect::Sepia { intensity } }];
            assert!(validate_effects(&effects).is_err());
            assert!(recipe(effects).validate(DetailLimits::default()).is_err());
        }
    }
    assert!(validate_effects(&vec![step(TextureEffect::Invert); 8]).is_ok());
    assert!(validate_effects(&vec![step(TextureEffect::Invert); 9]).is_err());
    let mut too_large = recipe(vec![step(TextureEffect::Invert); 9]);
    assert!(too_large.validate(DetailLimits::default()).is_err());
    let DetailRecipe::Paint(p) = &mut too_large else { unreachable!() };
    p.effects.clear();
    p.side = 1025;
    assert!(too_large.validate(DetailLimits::default()).is_err());
}

#[test]
fn texture_effect_budgets_charge_disabled_work_and_temporary_before_base_allocation() {
    let baseline = recipe(vec![]).budget_plan(DetailLimits::default()).unwrap();
    let effects = vec![TextureEffectStep { enabled: false,
        effect: TextureEffect::Sepia { intensity: 0.75 } }; 8];
    let recipe = recipe(effects);
    let cost = recipe.budget_plan(DetailLimits::default()).unwrap();
    assert_eq!(cost.pixel_visits - baseline.pixel_visits, 64 * (8 + 2));
    assert_eq!(cost.working_bytes - baseline.working_bytes, 64 * 4);
    assert_eq!(cost.output_bytes, baseline.output_bytes);
    for limits in [
        DetailLimits { pixel_visits: cost.pixel_visits - 1, ..DetailLimits::default() },
        DetailLimits { working_bytes: cost.working_bytes - 1, ..DetailLimits::default() },
        DetailLimits { output_bytes: cost.output_bytes - 1, ..DetailLimits::default() },
    ] {
        // No base: a budget error proves admission happens before base/resource/render work.
        assert!(render_detail(&recipe, None, limits).unwrap_err().contains("budget"));
    }
    let exact = DetailLimits { pixel_visits: cost.pixel_visits,
        working_bytes: cost.working_bytes, output_bytes: cost.output_bytes,
        ..DetailLimits::default() };
    assert!(recipe.validate(exact).is_ok());
    let original = [100, 150, 200, 255].repeat(64);
    assert_eq!(decode(&render_detail(&recipe, Some(base(&original)), exact).unwrap()), original);
}
