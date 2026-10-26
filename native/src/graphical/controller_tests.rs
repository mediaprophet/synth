use super::super::geometry_host::ORGANIC_SURFACE_SOURCE;
use super::*;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

struct NoPixels(Arc<AtomicUsize>);
impl PreviewProvider for NoPixels {
    fn preview(&mut self, _: SourceRef, _: &Value) -> Result<PreviewFrame, String> {
        self.0.fetch_add(1, Ordering::Relaxed);
        Err("CPU-only test; GPU preview deliberately not requested".into())
    }
}
fn fresh(label: &str) -> (CanonicalEditorHost, Arc<AtomicUsize>) {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let id = format!(
        "urn:poet:recovery-test:{label}:{}:{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    );
    studio_store::create_artifact_if_absent(StudioArtifact::new(&id, "Geometry", label)).unwrap();
    let calls = Arc::new(AtomicUsize::new(0));
    let host = GeometryHost::new(
        vec![
            "capability.invoke".into(),
            "ComputationalGeometry.extrude_profile".into(),
        ],
        200_000,
    );
    (
        CanonicalEditorHost::open(
            id,
            SessionLimits::default(),
            host,
            Box::new(NoPixels(calls.clone())),
        )
        .unwrap(),
        calls,
    )
}
fn accept(host: &mut CanonicalEditorHost) -> EditorSnapshot {
    let source = host.snapshot().unwrap().source;
    host.submit(ORGANIC_SURFACE_SOURCE, source)
        .unwrap()
        .snapshot
}

#[test]
fn undo_then_explicit_rebuild_restores_editability_without_preview_replay() {
    let (mut host, calls) = fresh("undo");
    accept(&mut host);
    let twice = accept(&mut host);
    let old_epoch = twice.execution_epoch;
    let undone = host.undo(twice.source).unwrap();
    assert!(undone.reconciliation_required);
    let output = host.rebuild_execution(undone.source).unwrap();
    assert!(output.frame.is_none());
    assert_eq!(output.snapshot.session_revision, 1);
    assert!(!output.snapshot.reconciliation_required);
    assert!(!old_epoch.is_current());
    assert!(output.snapshot.execution_epoch.is_current());
    assert_eq!(calls.load(Ordering::Relaxed), 2);
    assert_eq!(accept(&mut host).session_revision, 2);
}

#[test]
fn runtime_error_rebuild_keeps_source_and_retires_numeric_revision_epoch() {
    let (mut host, _) = fresh("error");
    let accepted = accept(&mut host);
    let broken = r#"requires [capability("capability.invoke"), capability("ComputationalGeometry.extrude_profile")];
        capability.invoke("ComputationalGeometry.extrude_profile", {axis:[],section:[]});"#;
    assert!(host.submit(&broken, accepted.source.clone()).is_err());
    let held = host.snapshot().unwrap();
    assert!(held.reconciliation_required);
    assert_eq!(held.source, accepted.source);
    let rebuilt = host.rebuild_execution(held.source).unwrap().snapshot;
    assert_eq!(rebuilt.session_revision, accepted.session_revision);
    assert_ne!(rebuilt.execution_epoch, accepted.execution_epoch);
    assert!(!accepted.execution_epoch.is_current());
    assert_eq!(accept(&mut host).session_revision, 2);
}

#[test]
fn replay_failure_preserves_prior_session_epoch_and_source() {
    let (mut host, _) = fresh("failure");
    let accepted = accept(&mut host);
    let (_, stamp) = studio_store::mutate_artifact_stamped(
        &host.artifact_id,
        &accepted.source.stamp,
        |artifact| {
            artifact
                .nodes
                .get_mut(&format!("{SOURCE_PREFIX}{:020}", 1))
                .unwrap()
                .insert("text".into(), "let = ;".into());
            Ok(())
        },
    )
    .unwrap();
    let source = SourceRef {
        artifact_id: host.artifact_id.clone(),
        stamp,
    };
    assert!(host.rebuild_execution(source.clone()).is_err());
    let held = host.snapshot().unwrap();
    assert_eq!(held.source, source);
    assert_eq!(held.session_revision, accepted.session_revision);
    assert_eq!(held.execution_epoch, accepted.execution_epoch);
    assert!(held.execution_epoch.is_current());
    assert!(held.reconciliation_required);
}

#[test]
fn identical_replacement_between_replay_and_promotion_is_rejected() {
    let (mut host, _) = fresh("stale");
    let accepted = accept(&mut host);
    let candidate = execution::prepare(
        accepted.source.clone(),
        &host.host,
        &host.factory_limits,
        &host.rebuild_limits,
        host.preview.as_ref(),
        host.surface_resources.as_ref(),
    )
    .unwrap();
    let artifact = studio_store::get_artifact_stamped(&host.artifact_id)
        .unwrap()
        .0;
    studio_store::upsert_artifact(artifact);
    assert!(host.activate_rebuild(candidate).is_err());
    assert_eq!(host.session.revision(), accepted.session_revision);
    assert_eq!(host.execution_epoch, accepted.execution_epoch);
    assert!(host.execution_epoch.is_current());
    assert_ne!(host.snapshot().unwrap().source, accepted.source);
}

#[test]
fn recovery_aggregate_dispatch_and_instruction_limits_do_not_promote() {
    let (mut host, _) = fresh("bounds");
    accept(&mut host);
    let accepted = accept(&mut host);
    host.rebuild_limits.total_kernel_dispatches = 1;
    assert!(host.rebuild_execution(accepted.source.clone()).is_err());
    assert_eq!(host.execution_epoch, accepted.execution_epoch);
    host.rebuild_limits.total_kernel_dispatches = 256;
    host.rebuild_limits.total_steps = 1;
    assert!(host.rebuild_execution(accepted.source).is_err());
    assert_eq!(host.session.revision(), 2);
    assert!(host.execution_epoch.is_current());
}

#[test]
fn gapped_or_noncanonical_accepted_cell_keys_cannot_open_or_rebuild() {
    let (mut host, _) = fresh("gaps");
    let accepted = accept(&mut host);
    let (_, stamp) = studio_store::mutate_artifact_stamped(
        &host.artifact_id,
        &accepted.source.stamp,
        |artifact| {
            let cell = artifact
                .nodes
                .remove(&format!("{SOURCE_PREFIX}{:020}", 1))
                .unwrap();
            artifact
                .nodes
                .insert(format!("{SOURCE_PREFIX}{:020}", 3), cell);
            Ok(())
        },
    )
    .unwrap();
    let source = SourceRef {
        artifact_id: host.artifact_id.clone(),
        stamp,
    };
    assert!(host.rebuild_execution(source).is_err());
    assert_eq!(host.session.revision(), 1);
    assert_eq!(host.execution_epoch, accepted.execution_epoch);
    assert!(host.execution_epoch.is_current());
    let artifact = studio_store::get_artifact_stamped(&host.artifact_id)
        .unwrap()
        .0;
    assert!(source_cells(&artifact).is_err());
    assert!(CanonicalEditorHost::open(
        host.artifact_id.clone(),
        SessionLimits::default(),
        GeometryHost::default(),
        Box::new(NoPixels(Arc::new(AtomicUsize::new(0))))
    )
    .is_err());
}
