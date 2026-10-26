//! Fresh native process HMC source/lineage recovery; no implicit rebuild.
use super::parameter_safety_tests::{parameter, unchanged_after_refusal};
use super::*;
use crate::graphical::{
    checkpoints::{
        CheckpointEditorHost, CheckpointLimits, CheckpointService, ExpectedFile, TrustedBinding,
    },
    controller::{CanonicalEditorHost, EditorHost, PreviewProvider},
    geometry_host::GeometryHost,
    preview::{PreviewFrame, PreviewLimits, SourceRef},
};
use qualia_core_db::poet_host::invoke::studio_store::{self, StudioArtifact};
use vibe::{session::SessionLimits, Value};
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
        Err("No GPU requested by parameter source recovery case".into())
    }
}
const ID: &str = "urn:poet:parameter-safety-restart";
const CHILD: &str =
    "graphical::authoring::parameter_safety_restart_tests::parameter_safety_restart_child";
fn restart_host(root: &std::path::Path) -> CheckpointEditorHost {
    let canonical = CanonicalEditorHost::open(
        ID.into(),
        SessionLimits::default(),
        GeometryHost::default(),
        Box::new(Admission),
    )
    .unwrap();
    let service = CheckpointService::new(
        root,
        TrustedBinding {
            project_id: "urn:poet:parameter-safety-project".into(),
            document_id: "urn:poet:parameter-safety-document".into(),
            source_id: ID.into(),
            domain: "Geometry".into(),
        },
        CheckpointLimits::default(),
    )
    .unwrap();
    CheckpointEditorHost::new(Box::new(canonical), service).unwrap()
}
#[test]
fn parameter_safety_and_recipe_history_survive_fresh_process_hmc_reopen() {
    let root = tempfile::tempdir().unwrap();
    for phase in ["write", "read"] {
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", CHILD, "--nocapture", "--test-threads=1"])
            .env("POET_PARAMETER_SAFETY_PHASE", phase)
            .env("POET_PARAMETER_SAFETY_ROOT", root.path())
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
fn parameter_safety_restart_child() {
    let Some(phase) = std::env::var_os("POET_PARAMETER_SAFETY_PHASE") else {
        return;
    };
    let root = std::path::PathBuf::from(std::env::var_os("POET_PARAMETER_SAFETY_ROOT").unwrap());
    assert!(studio_store::get_artifact_stamped(ID).is_none());
    if phase == "write" {
        studio_store::create_artifact_if_absent(StudioArtifact::new(
            ID,
            "Geometry",
            "Synthetic parameter recovery",
        ))
        .unwrap();
        let mut h = restart_host(&root);
        let source = h.snapshot().unwrap().source;
        let source = h
            .submit(&super::tests::source(), source)
            .unwrap()
            .snapshot
            .source;
        let source = h
            .authoring_edit(parameter(1.25), source)
            .unwrap()
            .snapshot
            .source;
        let source = h
            .authoring_edit(
                AuthoringEdit::Material {
                    material_id: 1,
                    base_color: [0.2, 0.3, 0.4, 1.0],
                    roughness: 0.8,
                    metallic: 0.0,
                },
                source,
            )
            .unwrap()
            .snapshot
            .source;
        let source = h
            .authoring_edit(AuthoringEdit::RemoveLeaf { id: "lever".into() }, source)
            .unwrap()
            .snapshot
            .source;
        unchanged_after_refusal(&mut h);
        h.save_checkpoint("parameter.hmc", ExpectedFile::Absent, source)
            .unwrap();
    } else {
        assert_eq!(phase, "read");
        let service = CheckpointService::new(
            &root,
            TrustedBinding {
                project_id: "urn:poet:parameter-safety-project".into(),
                document_id: "urn:poet:parameter-safety-document".into(),
                source_id: ID.into(),
                domain: "Geometry".into(),
            },
            CheckpointLimits::default(),
        )
        .unwrap();
        let prepared = service.prepare_reopen("parameter.hmc").unwrap();
        let exact = prepared.artifact().clone();
        service.activate(prepared).unwrap();
        let mut h = restart_host(&root);
        let held = h.snapshot().unwrap();
        assert!(
            held.reconciliation_required,
            "open cannot replay implicitly"
        );
        let source = h.rebuild_execution(held.source).unwrap().snapshot.source;
        assert_eq!(studio_store::get_artifact_stamped(ID).unwrap().0, exact);
        let cells = h.source_cells(source.clone()).unwrap();
        assert_eq!(cells.len(), 4);
        assert!(cells[0].contains("let mut poet_parameter_parts="));
        assert!(cells[1].contains("poet_parameter_parts = poet_rebuild_parts"));
        let view = h.authoring_view(source.clone()).unwrap();
        assert_eq!(view.parts[0].translation[0], 1.25);
        assert!(view.parts.iter().all(|p| p.roughness == 0.8));
        unchanged_after_refusal(&mut h);
        let held = h.undo(source).unwrap();
        let source = h.rebuild_execution(held.source).unwrap().snapshot.source;
        let source = h
            .authoring_edit(parameter(2.5), source)
            .unwrap()
            .snapshot
            .source;
        assert_eq!(
            h.authoring_view(source).unwrap().parts[0].translation[0],
            2.5
        );
    }
}
