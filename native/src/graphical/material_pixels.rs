//! Explicit opt-in real-GPU proof; no skip-as-pass when the renderer is absent.
use super::super::{
    controller::{CanonicalEditorHost, EditorHost, MeshPreviewProvider},
    geometry_host::GeometryHost,
    preview::{PreviewCamera, PreviewFrame, SourceRef},
};
use super::*;
use qualia_core_db::poet_host::invoke::studio_store::{self, StudioArtifact};
use vibe::session::SessionLimits;

struct AuthoredPreview(MeshPreviewProvider);
impl super::super::controller::PreviewProvider for AuthoredPreview {
    fn preview(&mut self, source: SourceRef, value: &Value) -> Result<PreviewFrame, String> {
        let (mesh, material) = authored_scene(value, self.0.material, self.0.limits)?;
        super::super::preview::render_mesh(source, &mesh, material, self.0.camera, self.0.limits)
    }
}

#[test]
#[ignore = "explicit native GPU readback qualification; run separately with --ignored --exact"]
fn accepted_material_edits_change_native_gpu_pixels() {
    let id = format!("urn:poet:material-pixel-test:{}", std::process::id());
    studio_store::create_artifact_if_absent(StudioArtifact::new(
        &id,
        "Geometry",
        "material pixel evidence",
    ))
    .unwrap();
    let provider = MeshPreviewProvider {
        material: MaterialRecord::legacy_default(),
        camera: PreviewCamera {
            yaw: 0.45,
            pitch: 0.2,
            distance: 3.2,
            target: [0.0; 3],
        },
        limits: PreviewLimits {
            side: 64,
            ..PreviewLimits::default()
        },
    };
    let grants = vec![
        "capability.invoke".into(),
        "ComputationalGeometry.extrude_profile".into(),
    ];
    let mut editor = CanonicalEditorHost::open(
        id,
        SessionLimits::default(),
        GeometryHost::new(grants, 200_000),
        Box::new(AuthoredPreview(provider)),
    )
    .unwrap();
    let initial = editor
        .submit(&diagnostic_source(), editor.snapshot().unwrap().source)
        .unwrap();
    let mut before = initial
        .frame
        .expect("real initial GPU frame; no silent skip");
    let mut draft = MaterialDraft::default();
    for (index, label) in ["base_color", "roughness", "metallic", "emissive"]
        .iter()
        .enumerate()
    {
        match *label {
            "base_color" => draft.base_color = [0.04, 0.25, 0.82, 1.0],
            "roughness" => draft.roughness = 0.04,
            "metallic" => draft.metallic = 0.95,
            "emissive" => draft.emissive = [0.4, 0.0, 0.0],
            _ => unreachable!(),
        }
        let next = editor
            .submit(
                &draft
                    .edit_source(index as u64 + 2, diagnostic_material_id())
                    .unwrap(),
                editor.snapshot().unwrap().source,
            )
            .unwrap();
        let after = next.frame.expect("real edited GPU frame; no silent skip");
        let changed = before
            .rgba8
            .chunks_exact(4)
            .zip(after.rgba8.chunks_exact(4))
            .filter(|(a, b)| a != b)
            .count();
        assert!(
            changed >= 4,
            "{label} produced only {changed} changed pixels"
        );
        assert_ne!(before.source.stamp, after.source.stamp);
        assert_eq!(after.session_revision, Some(index as u64 + 2));
        eprintln!("actual_native_material_pixels field={label} side=64 changed_pixels={changed} source_revision={}", next.snapshot.source_revision);
        before = after;
    }
}
