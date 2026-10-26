//! Actual canonical source/geometry fixtures; no GUI, GPU or production authentication.
use super::*;
use crate::graphical::{
    controller::{CanonicalEditorHost, PreviewProvider},
    geometry_host::GeometryHost,
    materials::{self, MaterialDraft},
    preview::{PreviewFrame, PreviewLimits},
};
use qualia_core_db::{
    container_10d::MaterialRecord,
    poet_host::invoke::studio_store::{self, StudioArtifact},
};
use std::sync::atomic::{AtomicU64, Ordering};
use vibe::{session::SessionLimits, Value};

static NEXT: AtomicU64 = AtomicU64::new(1);
struct Headless;
impl PreviewProvider for Headless {
    fn validate(&self, value: &Value) -> Result<(), String> {
        // Real native mesh/material projection, without rendering a fake frame.
        materials::authored_scene(
            value,
            MaterialRecord::legacy_default(),
            PreviewLimits::default(),
        )
        .map(|_| ())
    }
    fn preview(&mut self, _: SourceRef, _: &Value) -> Result<PreviewFrame, String> {
        Err("headless fixture; no GPU frame produced".into())
    }
}
fn fixture() -> SharedEditorHost {
    let id = format!(
        "urn:poet:shared-fixture:{}:{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    );
    studio_store::create_artifact_if_absent(StudioArtifact::new(&id, "Geometry", "shared fixture"))
        .unwrap();
    let host = CanonicalEditorHost::open(
        id,
        SessionLimits::default(),
        GeometryHost::new(
            vec![
                "capability.invoke".into(),
                "ComputationalGeometry.extrude_profile".into(),
            ],
            200_000,
        ),
        Box::new(Headless),
    )
    .unwrap();
    SharedEditorHost::new(Box::new(host))
}

#[test]
fn clones_observe_the_same_actual_geometry_material_session_and_rebuild() {
    let mut ui = fixture();
    let mut mcp = ui.clone();
    let original = ui.snapshot().unwrap();
    assert!(original.authorisation_stamp.is_none());
    let source = materials::diagnostic_source();
    let accepted = ui.submit(&source, original.source.clone()).unwrap();
    assert!(accepted.frame.is_none());
    let observed = mcp.snapshot().unwrap();
    assert_eq!(observed.session_revision, 1);
    assert_eq!(observed.source, accepted.snapshot.source);
    assert_eq!(
        mcp.source_cells(observed.source.clone()).unwrap(),
        vec![source.clone()]
    );
    let material_id = observed.material_edit_id.unwrap();
    let mut draft = MaterialDraft::default();
    draft.roughness = 0.25;
    let edited = mcp
        .material_edit(&draft.edit_source(2, material_id).unwrap(), observed.source)
        .unwrap();
    assert_eq!(ui.snapshot().unwrap().session_revision, 2);
    let cells = ui.source_cells(edited.snapshot.source.clone()).unwrap();
    assert_eq!(cells.len(), 2);
    assert_eq!(cells[0], source);
    assert!(ui.submit(&source, original.source).is_err());
    assert_eq!(ui.snapshot().unwrap().session_revision, 2);
    let undone = ui.undo(edited.snapshot.source).unwrap();
    assert!(mcp.snapshot().unwrap().reconciliation_required);
    let rebuilt = mcp.rebuild_execution(undone.source).unwrap();
    assert!(!ui.snapshot().unwrap().reconciliation_required);
    assert_eq!(ui.snapshot().unwrap().session_revision, 1);
    assert!(!observed.execution_epoch.is_current());
    let redone = mcp.redo(rebuilt.snapshot.source).unwrap();
    assert!(ui.snapshot().unwrap().reconciliation_required);
    ui.rebuild_execution(redone.source).unwrap();
    assert_eq!(mcp.snapshot().unwrap().session_revision, 2);
    assert!(mcp.snapshot().unwrap().authorisation_stamp.is_none());
}

#[test]
fn busy_shared_lock_denies_without_dispatch_or_retry() {
    let shared = fixture();
    let mut other = shared.clone();
    let snapshot = other.snapshot().unwrap();
    let guard = shared.inner.lock().unwrap();
    assert!(other.snapshot().err().unwrap().contains("busy"));
    assert!(other.check("1;").unwrap_err().contains("busy"));
    assert!(other
        .submit(&materials::diagnostic_source(), snapshot.source.clone())
        .err()
        .unwrap()
        .contains("busy"));
    assert!(other
        .material_edit("invalid", snapshot.source.clone())
        .err()
        .unwrap()
        .contains("busy"));
    assert!(other
        .source_cells(snapshot.source.clone())
        .unwrap_err()
        .contains("busy"));
    assert!(other
        .save_checkpoint("fixture.hmc", ExpectedFile::Absent, snapshot.source)
        .err()
        .unwrap()
        .contains("busy"));
    // Availability is stable and never inferred from this busy lock.
    assert!(!other.checkpoint_supported());
    drop(guard);
    assert_eq!(other.snapshot().unwrap().session_revision, 0);
    assert!(other
        .source_cells(other.snapshot().unwrap().source)
        .unwrap()
        .is_empty());
}

#[test]
fn poison_is_explicit_and_cannot_silently_recover_the_session() {
    let shared = fixture();
    let poisoned = shared.clone();
    let _ = std::thread::spawn(move || {
        let _guard = poisoned.inner.lock().unwrap();
        panic!("explicit shared-host poison fixture");
    })
    .join();
    let message = shared.snapshot().err().unwrap();
    assert!(message.contains("poisoned") && message.contains("reconciliation"));
    assert!(shared.check("1;").is_err());
}
