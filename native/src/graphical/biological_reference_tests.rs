//! Explicit native GPU diagnostics for branching fixture command streams.
//! Fixture labels describe command complexity only, not biological stages.
use super::{
    controller::{CanonicalEditorHost, EditorHost, MeshPreviewProvider, PreviewProvider},
    geometry_host::GeometryHost,
    preview::{mesh_from_invoke, PreviewCamera, PreviewLimits},
    viewport::{ViewportCommand, ViewLighting},
    viewport_provider::InteractivePreviewProvider,
};
use qualia_core_db::{
    container_10d::MaterialRecord,
    poet_host::invoke::studio_store::{self, StudioArtifact},
    render::surface_authoring::encode_surface_png_into,
};
use sha2::{Digest, Sha256};
use std::{collections::BTreeSet, io::Write, sync::atomic::AtomicBool};
use vibe::{session::{SessionLimits, SourceSession}, Budget};

const FIXTURES: [(&str, &str, &str); 3] = [
    (
        "establishment",
        "establishment_branching_mesh",
        "C:/github/ecopunk/tools/poet-foundation/biological-reference/fixtures/establishment-branching.vibe",
    ),
    (
        "young",
        "young_branching_mesh",
        "C:/github/ecopunk/tools/poet-foundation/biological-reference/fixtures/young-branching.vibe",
    ),
    (
        "mature",
        "mature_branching_mesh",
        "C:/github/ecopunk/tools/poet-foundation/biological-reference/fixtures/mature-branching.vibe",
    ),
];

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn provider() -> Result<InteractivePreviewProvider, String> {
    let mut material = MaterialRecord::legacy_default();
    // Neutral diagnostic brown only; not a bark or species material claim.
    material.base_color = [0.38, 0.24, 0.14, 1.0];
    material.emissive = [0.0; 3];
    material.metallic = 0.0;
    material.roughness = 0.95;
    let limits = PreviewLimits {
        side: 256,
        vertices: 20_000,
        indices: 80_000,
    };
    InteractivePreviewProvider::new(MeshPreviewProvider {
        material,
        camera: PreviewCamera {
            yaw: std::f32::consts::PI,
            pitch: 0.12,
            distance: 6.0,
            target: [0.0, 1.0, 0.0],
        },
        limits,
    })
}

fn make_host(label: &str) -> Result<CanonicalEditorHost, String> {
    let id = format!(
        "urn:poet:biological-reference:{}:{}:{}",
        label,
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| e.to_string())?
            .as_nanos()
    );
    studio_store::create_artifact_if_absent(StudioArtifact::new(
        &id,
        "Geometry",
        &format!("Branching diagnostic fixture: {label}"),
    ))
    .map_err(|e| format!("create diagnostic source: {e:?}"))?;
    let grants = vec![
        "capability.invoke".to_owned(),
        "ComputationalGeometry.turtle_3d".to_owned(),
    ];
    let preview: Box<dyn PreviewProvider> = Box::new(provider()?);
    CanonicalEditorHost::open(
        id,
        SessionLimits::default(),
        GeometryHost::new(grants, 20_000),
        preview,
    )
}

fn write_evidence(label: &str, source: &[u8], frame: &[u8], side: u32, record: serde_json::Value) {
    let Some(configured) = std::env::var_os("POET_BIOLOGICAL_DIAGNOSTICS") else {
        return;
    };
    let root = std::fs::canonicalize(configured)
        .expect("configured diagnostic output directory must already exist");
    let qa = std::fs::canonicalize("C:/github/ecopunk/docs/qa")
        .expect("Ecopunk QA root must exist");
    assert!(root.is_dir() && root.starts_with(qa), "diagnostics must stay under Ecopunk docs/qa");

    let mut png = vec![0; frame.len() + side as usize + 4096];
    let written = encode_surface_png_into(side, frame, &mut png).expect("encode native frame");
    let mut image = std::fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(root.join(format!("branching-{label}.png")))
        .expect("create diagnostic PNG without overwriting earlier evidence");
    image.write_all(&png[..written]).expect("write diagnostic PNG");
    let mut json = record;
    json["source_sha256"] = serde_json::Value::String(sha256(source));
    json["png_sha256"] = serde_json::Value::String(sha256(&png[..written]));
    json["fixture_label_is_biological_stage"] = serde_json::Value::Bool(false);
    json["biological_validation"] = serde_json::Value::String("not assessed".into());
    let mut metadata = std::fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(root.join(format!("branching-{label}.json")))
        .expect("create diagnostic JSON without overwriting earlier evidence");
    metadata
        .write_all(&serde_json::to_vec_pretty(&json).expect("serialize diagnostic metadata"))
        .expect("write diagnostic JSON");
}

/// Actual native invoke + canonical editor/GPU viewport diagnostics; never a production claim.
#[test]
#[ignore = "explicit native GPU branching diagnostics; not model or human acceptance"]
fn native_branching_diagnostics_render_each_source_cell() {
    let grants = vec![
        "capability.invoke".to_owned(),
        "ComputationalGeometry.turtle_3d".to_owned(),
    ];
    let mut source_session = SourceSession::new(grants.clone(), SessionLimits::default());
    let mut geometry_host = GeometryHost::new(grants, 20_000);
    let mut signatures = BTreeSet::new();
    let mut source_hashes = BTreeSet::new();

    for (label, binding, fixture_path) in FIXTURES {
        let source = std::fs::read(fixture_path).expect("read checked-in Vibe source fixture");
        let source_text = std::str::from_utf8(&source).expect("fixture source is UTF-8");
        let source_hash = sha256(&source);
        assert!(source_hashes.insert(source_hash.clone()), "fixture source streams must differ");
        source_session
            .submit(source_text, &mut geometry_host, Budget::default())
            .expect("actual Vibe source evaluation through granted native geometry host");
        let value = source_session
            .value(binding)
            .expect("fixture's named turtle_3d result binding");
        let mesh = mesh_from_invoke(
            value,
            PreviewLimits { side: 256, vertices: 20_000, indices: 80_000 },
        )
        .expect("actual turtle_3d output must be an admissible mesh");
        assert!(!mesh.positions.is_empty() && !mesh.triangles.is_empty());
        assert!(mesh.positions.iter().flatten().all(|v| v.is_finite()));
        assert!(mesh.triangles.iter().flatten().all(|i| (*i as usize) < mesh.positions.len()));
        assert!((0..3).all(|axis| mesh.max[axis].is_finite() && mesh.min[axis].is_finite() && mesh.max[axis] > mesh.min[axis]));
        let signature = format!("{}:{}:{:?}:{:?}", mesh.positions.len(), mesh.triangles.len(), mesh.min, mesh.max);
        assert!(signatures.insert(signature), "fixture command streams must produce distinct geometry signatures");

        let mut editor = make_host(label).expect("construct canonical editor host");
        let initial = editor.snapshot().expect("initial identified source snapshot");
        let submitted = editor
            .submit(source_text, initial.source)
            .expect("canonical editor host evaluates source and admits actual preview provider");
        assert!(submitted.frame.is_some(), "canonical submit must render a native preview");
        let current = editor.snapshot().expect("committed source snapshot");
        assert_eq!(current.latest_source, source_text);
        let expected_source = current.source.clone();
        let cancelled = AtomicBool::new(false);
        let _lighting = editor
            .viewport_action(ViewLighting::neutral_studio().command(), current.source.clone(), &cancelled)
            .expect("apply neutral diagnostic viewport lighting");
        let output = editor
            .viewport_action(ViewportCommand::FrameAll, current.source, &cancelled)
            .expect("fit actual admitted branching mesh into 256-pixel viewport");
        let frame = output.frame;
        assert_eq!(frame.source, expected_source);
        assert_eq!(frame.side, 256);
        let background = &frame.rgba8[..4];
        let nonuniform_pixels = frame.rgba8.chunks_exact(4).filter(|pixel| *pixel != background).count();
        assert!(nonuniform_pixels > 0, "native GPU image must be nonuniform; this alone is not geometry coverage");

        let metrics = serde_json::json!({
            "diagnostic": "branching fixture render",
            "fixture": label,
            "source_sha256": source_hash,
            "artifact_id": frame.source.artifact_id,
            "store_stamp_debug": format!("{:?}", frame.source.stamp),
            "session_revision": current.session_revision,
            "execution_epoch_debug": format!("{:?}", frame.execution_epoch),
            "rgba8_sha256": sha256(&frame.rgba8),
            "vertices": mesh.positions.len(),
            "triangles": mesh.triangles.len(),
            "bounds_min": mesh.min,
            "bounds_max": mesh.max,
            "nonuniform_pixels": nonuniform_pixels,
            "frame_side": frame.side,
            "production_acceptance": "unassessed",
            "human_review": "not performed"
        });
        write_evidence(label, &source, &frame.rgba8, frame.side, metrics);
    }
    assert_eq!(source_hashes.len(), 3);
    assert_eq!(signatures.len(), 3);
}

