//! Child of native_project_tests: signed software fixtures, not human acceptance.
use super::*;
use crate::graphical::{
    package_workflow::{PackageSelection, PackageWorkflowAction},
    session_host::SessionEditorHost,
};
use std::sync::Mutex;

#[test]
fn selected_ui_handoff_withholds_old_stamps_while_direct_author_stays_admitted() {
    let temp = tempfile::tempdir().unwrap();
    for name in ["authority", "hmc", "library"] {
        std::fs::create_dir(temp.path().join(name)).unwrap();
    }
    let (scope, _, authority) = fixture(&temp.path().join("authority"));
    let author = login(&authority, "fixture-author", &AUTHOR);
    let project = Arc::new(
        NativeProject::open(
            config(
                scope,
                &temp.path().join("hmc"),
                &temp.path().join("library"),
            ),
            authority.clone(),
            author.clone(),
            None,
            Box::new(NoPixels),
        )
        .unwrap(),
    );
    let selection = Arc::new(Mutex::new(Some(author.clone())));
    let selected = SessionEditorHost::new(project.clone(), selection.clone());
    let author_selected = selected.snapshot().unwrap();
    let direct = project.connect(author.clone()).unwrap();
    let direct_snapshot = direct.snapshot().unwrap();
    let mut direct_mcp = project.mcp_backend(author.clone()).unwrap();
    let reviewer = login(&authority, "fixture-reviewer", &REVIEWER);
    // Authentication alone changes no UI selection and revokes nobody.
    assert!(author_selected.publication_is_current());
    assert!(direct_snapshot.publication_is_current());
    *selection.lock().unwrap() = Some(reviewer);
    assert!(!author_selected
        .authorisation_stamp
        .as_ref()
        .unwrap()
        .is_current());
    assert!(!author_selected.publication_is_current());
    assert!(direct_snapshot.publication_is_current());
    assert!(direct.snapshot().is_ok());
    assert!(direct_mcp.call("poet_source_read", json!({})).is_ok());
    let reviewer_selected = selected.snapshot().unwrap();
    assert!(reviewer_selected.publication_is_current());
    assert_ne!(
        reviewer_selected.authorisation_stamp,
        author_selected.authorisation_stamp
    );
    authority.disconnect(&author).unwrap();
    assert!(!direct_snapshot.publication_is_current());
    assert!(direct.snapshot().is_err());
    assert!(reviewer_selected.publication_is_current());
}

struct SelectionProbe {
    selection: crate::graphical::native_project::SessionSlot,
    observed_locked: Arc<std::sync::atomic::AtomicBool>,
}
impl PreviewProvider for SelectionProbe {
    // This fixture does not retain a viewport or rendered frame.
    fn retire_retained_viewport(&mut self) -> Result<(), String> { Ok(()) }
    fn validate(&self, value: &Value) -> Result<(), String> {
        self.observed_locked.store(
            self.selection.try_lock().is_err(),
            std::sync::atomic::Ordering::SeqCst,
        );
        NoPixels.validate(value)
    }
    fn preview(&mut self, source: SourceRef, value: &Value) -> Result<PreviewFrame, String> {
        NoPixels.preview(source, value)
    }
}

#[test]
fn selected_connection_refuses_busy_selection_and_package_audit_still_uses_native_owner() {
    let temp = tempfile::tempdir().unwrap();
    for name in ["authority", "hmc", "library"] {
        std::fs::create_dir(temp.path().join(name)).unwrap();
    }
    let (scope, _, authority) = fixture(&temp.path().join("authority"));
    let author = login(&authority, "fixture-author", &AUTHOR);
    let selection = Arc::new(Mutex::new(Some(author.clone())));
    let observed_locked = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let project = Arc::new(
        NativeProject::open(
            config(
                scope,
                &temp.path().join("hmc"),
                &temp.path().join("library"),
            ),
            authority.clone(),
            author.clone(),
            None,
            Box::new(SelectionProbe {
                selection: selection.clone(),
                observed_locked: observed_locked.clone(),
            }),
        )
        .unwrap(),
    );
    let mut selected = SessionEditorHost::new(project.clone(), selection.clone());
    let initial = selected.snapshot().unwrap();
    let source = selected
        .submit(&materials::diagnostic_source(), initial.source)
        .unwrap()
        .snapshot
        .source;
    assert!(
        observed_locked.load(std::sync::atomic::Ordering::SeqCst),
        "selection guard is retained inside the admitted native validation callback"
    );
    let current = selected.snapshot().unwrap();
    let direct = project.connect(author.clone()).unwrap();
    let direct_snapshot = direct.snapshot().unwrap();
    {
        let _busy_selection = selection.lock().unwrap();
        assert!(!current.publication_is_current());
        assert!(selected.snapshot().is_err());
        assert!(direct_snapshot.publication_is_current());
    }
    assert!(current.publication_is_current());
    let saved = selected
        .save_checkpoint("selected.hmc", ExpectedFile::Absent, source.clone())
        .unwrap();
    let audit = authority.workflow_evidence(&author).unwrap();
    assert_eq!(audit.package_writes.len(), 1);
    assert_eq!(audit.package_writes[0].actor_subject, "fixture-author");
    assert_eq!(audit.package_writes[0].result_digest, saved.digest);
    let service = project.package_workflow_service(selection.clone());
    let package_view = service
        .run(
            PackageWorkflowAction::Inspect(PackageSelection {
                filename: "selected.hmc".into(),
                package_id: "urn:fixture:selected-package".into(),
                digest: saved.digest,
            }),
            source,
        )
        .unwrap();
    assert!(package_view.publication_is_current());
    {
        let _busy_selection = selection.lock().unwrap();
        assert!(!package_view.publication_is_current());
    }
    let reviewer = login(&authority, "fixture-reviewer", &REVIEWER);
    assert!(package_view.publication_is_current());
    *selection.lock().unwrap() = Some(reviewer);
    assert!(!package_view.publication_is_current());
    assert!(direct.snapshot().is_ok());
}

#[test]
fn declined_native_proofs_do_not_exhaust_challenges_or_retire_another_session() {
    let temp = tempfile::tempdir().unwrap();
    let (_, _, authority) = fixture(temp.path());
    let reviewer = login(&authority, "fixture-reviewer", &REVIEWER);
    for _ in 0..40 {
        let error = crate::graphical::native_entry::session_proof::authenticate(
            &authority,
            "fixture-author",
            300,
            |_| Err("fixture credential provider declined".into()),
        )
        .unwrap_err();
        assert_eq!(error, "fixture credential provider declined");
    }
    let author = crate::graphical::native_entry::session_proof::authenticate(
        &authority,
        "fixture-author",
        300,
        |challenge| {
            let mut proof = [0; 64];
            ed25519::sign_with_context(
                &AUTHOR,
                challenge.message_digest(),
                SESSION_CONTEXT,
                &mut proof,
            )
            .map_err(|e| format!("{e:?}"))?;
            Ok(proof)
        },
    )
    .unwrap();
    for session in [&author, &reviewer] {
        authority
            .transact(session, |_| DocumentTransaction {
                result: Ok(()),
                successor: None,
                reconciliation_required: false,
            })
            .unwrap();
    }
}
