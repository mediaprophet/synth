use super::*;
use crate::graphical::surfaces::TextureRole;
use qualia_core_db::render::{
    asset_package::HmcTextureResource,
    texture_decode::{
        decode_hmc_texture_rgba8_into, inspect_hmc_texture_requirements, TextureColorSpace,
        TextureDecodeLimits,
    },
};

fn paint() -> DetailRecipe {
    DetailRecipe::Paint(PaintRecipe {
        effects: vec![],
        version: 1,
        side: 8,
        base_sha256: "11".repeat(32),
        layers: vec![PaintLayer {
            name: "Red detail".into(),
            visible: true,
            opacity: 1.0,
            strokes: vec![PaintStroke {
                size: 4.0,
                colour: [255, 0, 0, 255],
                opacity: 0.5,
                hardness: 1.0,
                points: vec![[4.0, 4.0]],
            }],
        
        ..crate::graphical::surfaces::detail::PaintLayer::default()
    }],
    })
}
fn base(rgba: &[u8]) -> DetailBase<'_> {
    DetailBase {
        digest: [0x11; 32],
        width: 8,
        height: 8,
        rgba,
        colour_space: TextureColorSpace::Srgb,
    }
}
fn pixels(output: &DetailOutput, role: TextureRole) -> Vec<u8> {
    let digest = output.overrides.iter().find(|(r, _)| *r == role).unwrap().1;
    let dep = output
        .dependencies
        .iter()
        .find(|d| d.digest == digest)
        .unwrap();
    let resource = HmcTextureResource {
        digest,
        mime_type: &dep.mime_type,
        bytes: &dep.bytes,
    };
    let (info, scratch) =
        inspect_hmc_texture_requirements(&resource, TextureDecodeLimits::default()).unwrap();
    let mut rgba = vec![0; info.rgba8_bytes];
    let mut scratch = vec![0; scratch];
    decode_hmc_texture_rgba8_into(
        &resource,
        TextureDecodeLimits::default(),
        &mut rgba,
        &mut scratch,
    )
    .unwrap();
    rgba
}
#[test]
fn source_recipe_round_trips_pinned_vibe_and_strict_unknown_fields_refuse() {
    let expected = paint();
    let text = format!("({});", expected.source_expression().unwrap());
    let mut session = vibe::session::SourceSession::default();
    let value = session
        .submit(
            &text,
            &mut vibe::LocalHost::default(),
            vibe::Budget::default(),
        )
        .unwrap()
        .value;
    assert_eq!(DetailRecipe::from_value(&value).unwrap(), expected);
    let mut json = serde_json::to_value(&expected).unwrap();
    json.as_object_mut()
        .unwrap()
        .insert("blend_mode".into(), "screen".into());
    assert!(serde_json::from_value::<DetailRecipe>(json).is_err());
}
#[test]
fn opaque_import_layer_edit_visibility_and_replay_change_real_encoded_pixels() {
    let original: Vec<u8> = (0..64).flat_map(|_| [0, 0, 0, 255]).collect();
    let recipe = paint();
    let output = render_detail(&recipe, Some(base(&original)), DetailLimits::default()).unwrap();
    assert_eq!(
        &pixels(&output, TextureRole::BaseColor)[144..148],
        &[128, 0, 0, 255]
    );
    assert_eq!(output.base_digest, Some([0x11; 32]));
    assert_eq!(output.recipe_sha256, recipe.digest().unwrap());
    assert_eq!(original[144..148], [0, 0, 0, 255]);
    let repeated = render_detail(&recipe, Some(base(&original)), DetailLimits::default()).unwrap();
    assert_eq!(output.dependencies, repeated.dependencies);
    let mut hidden = recipe.clone();
    let DetailRecipe::Paint(p) = &mut hidden else {
        unreachable!()
    };
    p.layers[0].visible = false;
    let hidden = render_detail(&hidden, Some(base(&original)), DetailLimits::default()).unwrap();
    assert_eq!(pixels(&hidden, TextureRole::BaseColor), original);
    assert_ne!(output.overrides, hidden.overrides);
}
#[test]
fn input_digest_extent_transfer_alpha_and_resource_budgets_refuse() {
    let original: Vec<u8> = (0..64).flat_map(|_| [0, 0, 0, 255]).collect();
    let p = paint();
    for which in 0..4 {
        let mut b = base(&original);
        match which {
            0 => b.digest = [2; 32],
            1 => b.width = 9,
            2 => b.colour_space = TextureColorSpace::Linear,
            _ => b.colour_space = TextureColorSpace::Unknown,
        }
        assert!(render_detail(&p, Some(b), DetailLimits::default()).is_err());
    }
    let mut alpha = original.clone();
    alpha[3] = 127;
    assert!(render_detail(&p, Some(base(&alpha)), DetailLimits::default()).is_err());
    let mut limits = DetailLimits::default();
    limits.working_bytes = 1;
    assert!(render_detail(&p, Some(base(&original)), limits)
        .unwrap_err()
        .contains("budget"));
    let mut limits = DetailLimits::default();
    limits.pixel_visits = 1;
    assert!(p.validate(limits).is_err());
    let mut limits = DetailLimits::default();
    limits.points = 0;
    assert!(p.validate(limits).is_err());
    let mut invalid = p.clone();
    let DetailRecipe::Paint(p) = &mut invalid else {
        unreachable!()
    };
    p.layers[0].strokes[0].points[0][0] = f32::NAN;
    assert!(invalid.validate(DetailLimits::default()).is_err());
}
#[test]
fn imported_base_only_recipe_still_charges_pixel_work_before_render() {
    let mut recipe = paint();
    let DetailRecipe::Paint(p) = &mut recipe else {
        unreachable!()
    };
    p.layers.clear();
    let plan = recipe.budget_plan(DetailLimits::default()).unwrap();
    assert!(plan.pixel_visits >= 8 * 8 * 4);
    let mut limits = DetailLimits::default();
    limits.pixel_visits = 0;
    assert!(recipe.validate(limits).is_err());
    let original: Vec<u8> = (0..64).flat_map(|_| [0, 0, 0, 255]).collect();
    assert!(render_detail(&recipe, Some(base(&original)), limits).is_err());
}
#[test]
fn paint_layer_and_stroke_order_are_editable_source_not_cached_pixels() {
    let original: Vec<u8> = (0..64).flat_map(|_| [0, 0, 0, 255]).collect();
    let mut recipe = paint();
    let DetailRecipe::Paint(p) = &mut recipe else {
        unreachable!()
    };
    let mut blue = p.layers[0].clone();
    blue.name = "Blue detail".into();
    blue.strokes[0].colour = [0, 0, 255, 255];
    p.layers.push(blue);
    let first = render_detail(&recipe, Some(base(&original)), DetailLimits::default()).unwrap();
    let DetailRecipe::Paint(p) = &mut recipe else {
        unreachable!()
    };
    p.layers.swap(0, 1);
    let second = render_detail(&recipe, Some(base(&original)), DetailLimits::default()).unwrap();
    assert_ne!(
        pixels(&first, TextureRole::BaseColor),
        pixels(&second, TextureRole::BaseColor)
    );
    let DetailRecipe::Paint(p) = &mut recipe else {
        unreachable!()
    };
    let red = p.layers.remove(1);
    p.layers[0].strokes.push(red.strokes[0].clone());
    let combined = render_detail(&recipe, Some(base(&original)), DetailLimits::default()).unwrap();
    let DetailRecipe::Paint(p) = &mut recipe else {
        unreachable!()
    };
    p.layers[0].strokes.swap(0, 1);
    let reverse = render_detail(&recipe, Some(base(&original)), DetailLimits::default()).unwrap();
    assert_ne!(combined.dependencies, reverse.dependencies);
}
#[test]
fn procedural_canonical_detail_generates_colour_normal_orm_and_distinct_relief() {
    let mut p = ProceduralRecipe {
        version: 1,
        side: 8,
        seed: 7,
        tint: [1.0; 3],
        contrast: 0.4,
        relief: 0.0,
        bands: [4, 1],
        band_strength: 0.7,
        roughness: 0.8,
        metallic: 0.1,
    };
    let flat = render_detail(
        &DetailRecipe::Procedural(p.clone()),
        None,
        DetailLimits::default(),
    )
    .unwrap();
    p.relief = 0.1;
    let relief =
        render_detail(&DetailRecipe::Procedural(p), None, DetailLimits::default()).unwrap();
    assert_eq!(relief.dependencies.len(), 3);
    assert_eq!(relief.overrides.len(), 4);
    assert_ne!(
        pixels(&flat, TextureRole::Normal),
        pixels(&relief, TextureRole::Normal)
    );
    assert!(pixels(&relief, TextureRole::MetallicRoughness)
        .chunks_exact(4)
        .all(|p| p[0] == 255 && p[2] == 26 && p[3] == 255));
    assert_eq!(relief.base_digest, None);
}
