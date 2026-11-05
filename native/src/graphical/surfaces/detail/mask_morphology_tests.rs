use super::*;
use super::mask_morphology::prepare;
use crate::graphical::surfaces::{SurfaceTextureLimits, TextureResources};
use sha2::{Digest, Sha256};
use qualia_core_db::{
    bundle::BundleWriter,
    render::{
        asset_package::texture_hmc_key,
        surface_authoring::encode_surface_png_into,
    },
};

fn resource_snapshot() -> (TextureResources, [u8; 32], Vec<u8>) {
    let mut rgba = vec![255; 8 * 8 * 4];
    for y in 0..8 {
        for x in 4..8 {
            let offset = (y * 8 + x) * 4;
            rgba[offset] = 0;
        }
    }
    let mut png = vec![0; rgba.len() + 8 + 4096];
    let used = encode_surface_png_into(8, &rgba, &mut png).unwrap();
    png.truncate(used);
    let digest: [u8; 32] = Sha256::digest(&png).into();
    let mut writer = BundleWriter::new();
    writer
        .add_file(texture_hmc_key(&digest), "image/png", png.clone(), None)
        .unwrap();
    let hmc = writer.build().unwrap();
    (
        TextureResources::from_hmc_bytes(&hmc, SurfaceTextureLimits::default()).unwrap(),
        digest,
        png,
    )
}

fn recipe(mask_digest: [u8; 32]) -> PaintRecipe {
    PaintRecipe {
        version: 3,
        side: 8,
        base_sha256: "11".repeat(32),
        layers: vec![PaintLayer {
            mask: Some(LayerMask::Resource {
                sha256: crate::graphical::surfaces::binding::digest_hex(&mask_digest),
                channel: MaskChannel::Red,
            }),
            ..PaintLayer::default()
        }],
        effects: vec![],
    }
}

#[test]
fn source_mask_morphology_derives_resource_and_retains_original_hmc_bytes() {
    let (resources, original_digest, original_png) = resource_snapshot();
    let source = recipe(original_digest);
    let (next, candidate, derived_digest) = prepare(
        &source,
        0,
        MaskMorphologyOperation::Expand,
        1.0,
        &resources,
        DetailLimits::default(),
    )
    .unwrap();
    assert_ne!(derived_digest, original_digest);
    assert_ne!(next.layers[0].mask, source.layers[0].mask);
    assert_eq!(
        resources.source_dependency(original_digest).unwrap().bytes,
        original_png
    );
    assert_eq!(
        candidate.source_dependency(original_digest).unwrap().bytes,
        original_png,
        "existing immutable mask input remains available"
    );
    assert_eq!(
        candidate.source_dependency(derived_digest).unwrap().mime_type,
        "image/png"
    );
    let decoded = candidate
        .mask_input_with_budget(derived_digest, 8, DetailLimits::default().working_bytes)
        .unwrap();
    assert_eq!((decoded.width, decoded.height), (8, 8));
    assert_eq!(decoded.rgba[4 * 4], 255, "red-channel coverage expanded at the edge");
}

#[test]
fn morphology_budget_refusal_preserves_recipe_and_original_resource() {
    let (resources, original_digest, original_png) = resource_snapshot();
    let source = recipe(original_digest);
    let before = source.clone();
    let refusal = prepare(
        &source,
        0,
        MaskMorphologyOperation::Border,
        1.0,
        &resources,
        DetailLimits {
            working_bytes: 1,
            ..DetailLimits::default()
        },
    );
    assert!(refusal.is_err());
    assert_eq!(source.layers[0].mask, before.layers[0].mask);
    assert_eq!(
        resources.source_dependency(original_digest).unwrap().bytes,
        original_png
    );
}

