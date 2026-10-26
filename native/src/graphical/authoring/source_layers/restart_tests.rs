//! Fresh-process HMC proof of editable source lineage; no GPU/master changes.
use crate::graphical::{
    authoring::source_layer_tests::{material, native_fixture as fixture, transform, width},
    checkpoints::{
        CheckpointEditorHost, CheckpointLimits, CheckpointService, ExpectedFile, TrustedBinding,
    },
    controller::{CanonicalEditorHost, EditorHost, PreviewProvider},
    geometry_host::GeometryHost,
    preview::{PreviewFrame, PreviewLimits, SourceRef},
};
use qualia_core_db::poet_host::invoke::studio_store::{self, StudioArtifact};
use vibe::{session::SessionLimits, Value};
const ID: &str = "urn:poet:source-layer-positive-restart";
const CHILD: &str =
    "graphical::authoring::source_layers::restart_tests::source_layer_restart_child";
struct Admission;
impl PreviewProvider for Admission {
    fn validate(&self, value: &Value) -> Result<(), String> {
        if crate::graphical::assembly::is_assembly(value) {
            crate::graphical::assembly::admit_assembly(value, PreviewLimits::default()).map(|_| ())
        } else {
            Ok(())
        }
    }
    fn preview(&mut self, _: SourceRef, _: &Value) -> Result<PreviewFrame, String> {
        Err("No GPU requested by source-layer restart case".into())
    }
}
fn binding() -> TrustedBinding {
    TrustedBinding {
        project_id: "urn:poet:source-layer-project".into(),
        document_id: "urn:poet:source-layer-document".into(),
        source_id: ID.into(),
        domain: "Geometry".into(),
    }
}
fn service(root: &std::path::Path) -> CheckpointService {
    CheckpointService::new(root, binding(), CheckpointLimits::default()).unwrap()
}
fn host(root: &std::path::Path) -> CheckpointEditorHost {
    let host = CanonicalEditorHost::open(
        ID.into(),
        SessionLimits::default(),
        GeometryHost::default(),
        Box::new(Admission),
    )
    .unwrap();
    CheckpointEditorHost::new(Box::new(host), service(root)).unwrap()
}
#[test]
fn parameter_layers_survive_fresh_process_hmc_reopen_and_further_edits() {
    let root = tempfile::tempdir().unwrap();
    for phase in ["write", "read"] {
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", CHILD, "--nocapture", "--test-threads=1"])
            .env("POET_SOURCE_LAYER_PHASE", phase)
            .env("POET_SOURCE_LAYER_ROOT", root.path())
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{phase}: {} {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
}
#[test]
fn source_layer_restart_child() {
    let Some(phase) = std::env::var_os("POET_SOURCE_LAYER_PHASE") else {
        return;
    };
    let root = std::path::PathBuf::from(std::env::var_os("POET_SOURCE_LAYER_ROOT").unwrap());
    assert!(studio_store::get_artifact_stamped(ID).is_none());
    if phase == "write" {
        studio_store::create_artifact_if_absent(StudioArtifact::new(
            ID,
            "Geometry",
            "Synthetic source-layer recovery",
        ))
        .unwrap();
        let mut host = host(&root);
        let source = host.snapshot().unwrap().source;
        let mut source = host.submit(&fixture(), source).unwrap().snapshot.source;
        for edit in [width(1.5), material(), transform(), width(2.5)] {
            source = host.authoring_edit(edit, source).unwrap().snapshot.source;
        }
        host.save_checkpoint("source-layer.hmc", ExpectedFile::Absent, source)
            .unwrap();
    } else {
        assert_eq!(phase, "read");
        let service = service(&root);
        let prepared = service.prepare_reopen("source-layer.hmc").unwrap();
        let exact = prepared.artifact().clone();
        service.activate(prepared).unwrap();
        let mut host = host(&root);
        let held = host.snapshot().unwrap();
        assert!(held.reconciliation_required);
        let source = host.rebuild_execution(held.source).unwrap().snapshot.source;
        assert_eq!(studio_store::get_artifact_stamped(ID).unwrap().0, exact);
        let view = host.authoring_view(source.clone()).unwrap();
        assert_eq!(
            view.parameters
                .iter()
                .find(|(key, _)| key == "width")
                .unwrap()
                .1,
            2.5
        );
        assert_eq!(view.parts[0].translation, [3.0, 0.0, 0.0]);
        assert!(view.parts.iter().all(|p| p.roughness == 0.8));
        let cells = host.source_cells(source.clone()).unwrap();
        assert_eq!(cells.len(), 5);
        assert!(cells[4].contains("poet_source_layer.overrides"));
        let source = host
            .authoring_edit(width(4.0), source)
            .unwrap()
            .snapshot
            .source;
        let view = host.authoring_view(source).unwrap();
        assert_eq!(
            view.parameters
                .iter()
                .find(|(key, _)| key == "width")
                .unwrap()
                .1,
            4.0
        );
        assert_eq!(view.parts[0].translation, [3.0, 0.0, 0.0]);
        assert!(view.parts.iter().all(|p| p.roughness == 0.8));
    }
}
