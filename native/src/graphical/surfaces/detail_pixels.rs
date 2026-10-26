//! Actual native assembly/atlas/detail resource path; unavailable GPU fails.
use super::*;
use crate::graphical::{
    assembly,
    preview::{PreviewCamera, PreviewLimits, SourceRef},
};
use qualia_core_db::{
    container_10d::MaterialRecord,
    poet_host::invoke::studio_store::{self, StudioArtifact},
};
use vibe::Value;
fn value(material: &MaterialRecord, recipe: Option<&detail::DetailRecipe>) -> Value {
    let mut fields = SurfaceDraft::from_material(material).updates().unwrap();
    fields.insert("id", "1".into());
    let material = format!(
        "{{{}}}",
        fields
            .iter()
            .map(|(k, v)| format!("{k}:{v}"))
            .collect::<Vec<_>>()
            .join(",")
    );
    let atlas = uv_atlas::source(&uv_atlas::AtlasDraft {
        atlas_size: 64,
        gutter_pixels: 2,
        texels_per_metre: 12.,
        ..Default::default()
    })
    .unwrap();
    let detail = recipe
        .map(|d| d.source_expression().unwrap())
        .unwrap_or_else(|| "null".into());
    let source=format!("({{parts:[{{id:\"p\",semantic_id:1,geometry:{{vertices:[[-1.0,-1.0,0.0],[1.0,-1.0,0.0],[1.0,1.0,0.0],[-1.0,1.0,0.0]],triangles:[[0,1,2],[0,2,3]],normals:[[0.0,0.0,1.0],[0.0,0.0,1.0],[0.0,0.0,1.0],[0.0,0.0,1.0]]}},material:{material},translation:[0.0,0.0,0.0],rotation:[0.0,0.0,0.0],scale:[1.0,1.0,1.0],uv_atlas:{atlas},detail:{detail}}}]}});");
    vibe::session::SourceSession::default()
        .submit(
            &source,
            &mut vibe::LocalHost::default(),
            vibe::Budget::default(),
        )
        .unwrap()
        .value
}
fn render(
    source: SourceRef,
    value: &Value,
    resources: &TextureResources,
) -> (Vec<u8>, MaterialRecord) {
    let resolved = resolution::resolve(value, Some(resources)).unwrap();
    let limits = PreviewLimits {
        side: 64,
        vertices: 256,
        indices: 768,
    };
    let scene =
        assembly::admit_assembly_with_materials(value, limits, &resolved.materials).unwrap();
    let mut settings = assembly::AssemblyRenderSettings::for_camera(PreviewCamera {
        yaw: 0.,
        pitch: 0.,
        distance: 3.2,
        target: [0.; 3],
    });
    settings.shadows = false;
    settings.screen_space_ao = false;
    let frame = assembly::render_assembly_with_resources(
        source,
        &scene,
        settings,
        limits,
        None,
        resolved.resources.as_ref(),
    )
    .expect("actual native atlas/detail GPU path");
    (frame.rgba8, scene.materials[0])
}
#[test]
#[ignore = "explicit serial native atlas/detail GPU readback"]
fn authored_atlas_paint_and_procedural_detail_change_actual_native_pixels() {
    let (bytes, d) = super::integration_tests::input();
    let resources = TextureResources::from_hmc_bytes(&bytes, Default::default()).unwrap();
    let id = format!("urn:poet:round7-gpu:{}", std::process::id());
    let (_, stamp) = studio_store::create_artifact_if_absent(StudioArtifact::new(
        &id,
        "Geometry",
        "Synthetic atlas/detail calibration",
    ))
    .unwrap();
    let source = SourceRef {
        artifact_id: id,
        stamp,
    };
    let mut material = MaterialRecord::legacy_default();
    material.id = 1;
    material.double_sided = true;
    material.base_color = [1.; 4];
    material.base_color_texture = d.digest;
    let paint = detail::DetailRecipe::Paint(detail::PaintRecipe {
        effects: vec![],
        version: 1,
        side: 8,
        base_sha256: binding::digest_hex(&d.digest),
        layers: vec![detail::PaintLayer {
            name: "Synthetic paint".into(),
            visible: true,
            opacity: 1.,
            strokes: vec![detail::PaintStroke {
                size: 8.,
                colour: [240, 20, 20, 255],
                opacity: 1.,
                hardness: 0.8,
                points: vec![[3.5, 3.5]],
            }],
        
        ..crate::graphical::surfaces::detail::PaintLayer::default()
    }],
    });
    let procedural = detail::DetailRecipe::Procedural(detail::ProceduralRecipe {
        version: 1,
        side: 16,
        seed: 17,
        tint: [0.1, 0.8, 0.3],
        contrast: 0.5,
        relief: 0.2,
        bands: [2, 3],
        band_strength: 0.3,
        roughness: 0.5,
        metallic: 0.2,
    });
    let (before, m0) = render(source.clone(), &value(&material, None), &resources);
    let (painted, m1) = render(source.clone(), &value(&material, Some(&paint)), &resources);
    let (generated, m2) = render(source, &value(&material, Some(&procedural)), &resources);
    let change = |a: &[u8], b: &[u8]| {
        a.chunks_exact(4)
            .zip(b.chunks_exact(4))
            .filter(|(a, b)| a != b)
            .count()
    };
    let paint_changes = change(&before, &painted);
    let procedural_changes = change(&before, &generated);
    eprintln!("native_atlas_paint changed_pixels={paint_changes}");
    eprintln!("native_atlas_procedural changed_pixels={procedural_changes}");
    assert!(paint_changes >= 4);
    assert!(procedural_changes >= 4);
    if let Some(mut output) = super::pixels::Artifacts::selected() {
        output.retain("atlas-source-base", m0, &before);
        output.retain("atlas-painted", m1, &painted);
        output.retain("atlas-procedural", m2, &generated);
    }
}
