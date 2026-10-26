//! Headless software-actor workflow, never human/GUI/HMC acceptance evidence.
use super::*;
use crate::graphical::cooperative_workflow::{source_difference, WorkflowAction};
use crate::graphical::{draft_buffer::DraftBuffer, session_host::SessionEditorHost};
use std::sync::Mutex;

#[test]
fn session_wrapper_worker_publishes_exact_successor_and_withholds_after_reviewer_handoff() {
    use crate::graphical::jobs::{self, Action, Context, Worker};
    use std::{
        future::Future,
        task::{Context as TaskContext, Poll, Waker},
        time::Duration,
    };
    let temp = tempfile::tempdir().unwrap();
    let authority_dir = temp.path().join("authority");
    let hmc_dir = temp.path().join("hmc");
    let library = temp.path().join("library");
    for path in [&authority_dir, &hmc_dir, &library] {
        std::fs::create_dir(path).unwrap();
    }
    let (scope, _, authority) = fixture(&authority_dir);
    let author = login(&authority, "fixture-author", &AUTHOR);
    let project = Arc::new(
        NativeProject::open(
            config(scope, &hmc_dir, &library),
            authority.clone(),
            author.clone(),
            None,
            Box::new(NoPixels),
        )
        .unwrap(),
    );
    let sessions = Arc::new(Mutex::new(Some(author)));
    let wrapper = SessionEditorHost::new(project, sessions.clone());
    let initial = wrapper.snapshot().unwrap();
    // Repeated snapshots must preserve the same derived owner incarnation.
    // Native authorization itself is still rechecked on each read.
    assert!(
        Context::from_snapshot(&initial) == Context::from_snapshot(&wrapper.snapshot().unwrap())
    );
    let (worker, handle) = Worker::start(Box::new(wrapper)).unwrap();
    let action = Action::Submit {
        source: materials::diagnostic_source(),
        expected: initial.source.clone(),
    };
    let result = {
        let mut future = std::pin::pin!(jobs::run(
            handle.clone(),
            action,
            Context::from_snapshot(&initial)
        ));
        let mut context = TaskContext::from_waker(Waker::noop());
        let mut completed = None;
        for _ in 0..200 {
            match future.as_mut().poll(&mut context) {
                Poll::Ready(result) => {
                    completed = Some(result);
                    break;
                }
                Poll::Pending => std::thread::sleep(Duration::from_millis(5)),
            }
        }
        completed
    };
    let Some(result) = result else {
        worker.finish().unwrap();
        panic!("bounded native worker did not complete source submission");
    };
    let reply = result.unwrap();
    assert!(
        initial.source.check_current().is_err(),
        "successful edit retires its origin source stamp"
    );
    assert!(
        jobs::completion_is_current(&handle, &initial, &reply),
        "the real UI may publish this exact admitted successor"
    );
    let snapshot = reply.snapshot.as_ref().unwrap();
    assert!(snapshot.publication_is_current());
    assert_ne!(snapshot.source, initial.source);
    let reviewer = login(&authority, "fixture-reviewer", &REVIEWER);
    *sessions.lock().unwrap() = Some(reviewer);
    assert!(!initial.authorisation_stamp.as_ref().unwrap().is_current());
    assert!(!snapshot.publication_is_current());
    assert!(
        !jobs::completion_is_current(&handle, &initial, &reply),
        "late author output must not cross actor handoff"
    );
    worker.finish().unwrap();
}

#[test]
fn cooperative_operator_service_binds_edit_save_request_review_revoke_and_reopen() {
    let temp = tempfile::tempdir().unwrap();
    let authority_dir = temp.path().join("authority");
    let hmc_dir = temp.path().join("hmc");
    let library = temp.path().join("library");
    for path in [&authority_dir, &hmc_dir, &library] {
        std::fs::create_dir(path).unwrap();
    }
    let (scope, admission, authority) = fixture(&authority_dir);
    let author = login(&authority, "fixture-author", &AUTHOR);
    let project = Arc::new(
        NativeProject::open(
            config(scope.clone(), &hmc_dir, &library),
            authority.clone(),
            author.clone(),
            None,
            Box::new(NoPixels),
        )
        .unwrap(),
    );
    let sessions = Arc::new(Mutex::new(Some(author.clone())));
    let service = project.workflow_service(sessions.clone());
    let mut operator = SessionEditorHost::new(project.clone(), sessions.clone());
    let mut editor = project.connect(author.clone()).unwrap();
    let initial = operator.snapshot().unwrap();
    assert!(operator.check(&materials::diagnostic_source()).is_ok());
    let output = operator
        .submit(&materials::diagnostic_source(), initial.source.clone())
        .unwrap();
    let source = output.snapshot.source.clone();
    assert_eq!(
        operator.source_cells(source.clone()).unwrap(),
        vec![materials::diagnostic_source()]
    );
    // Opaque publication stamps retain the authority owner. Release those
    // snapshots before the later journal-owner reopen.
    drop(initial);
    drop(output);
    let saved = operator
        .save_checkpoint("cooperative.hmc", ExpectedFile::Absent, source.clone())
        .unwrap();
    let master = ResourceData {
        key: "masters/editable-source.obj".into(),
        kind: "text/plain".into(),
        bytes: b"# retained editable fixture master\nv 0 1 2\n".to_vec(),
        meta: None,
    };
    let opaque = ResourceData {
        key: "retained/unknown.bin".into(),
        kind: "future/schema".into(),
        bytes: vec![0, 42, 255],
        meta: Some(vec![0xa1, 0x61, b'x', 1]),
    };
    let package = editor
        .edit_package_resources(
            "cooperative.hmc",
            ExpectedFile::Sha256(saved.digest),
            vec![
                ResourceEdit::Add(master.clone()),
                ResourceEdit::Add(opaque.clone()),
            ],
            source,
        )
        .unwrap();
    let before = service.run(WorkflowAction::Inspect).unwrap();
    assert_eq!(
        operator
            .workflow(WorkflowAction::Inspect)
            .unwrap()
            .evidence
            .principal_subject,
        "fixture-author"
    );
    assert_eq!(before.source_cells, vec![materials::diagnostic_source()]);
    assert!(!before.source_history.is_empty());
    assert!(before.previous_source_json.is_some());
    assert!(source_difference(
        before.previous_source_json.as_deref(),
        &before.current_source_json
    )
    .contains("+ "));
    let target = before.evidence.target.clone();
    service
        .run(WorkflowAction::Contribution {
            target: target.clone(),
            id: "operator/effort".into(),
            description: "Actual native fixture edit and resource-preserving save".into(),
            minutes: 5,
            work_status: WorkStatus::Unpaid {
                evidence_ref: "fixture/unpaid".into(),
            },
        })
        .unwrap();
    service
        .run(WorkflowAction::RequestReview {
            target: target.clone(),
            id: "operator/request".into(),
            evidence: "fixture/source-request".into(),
        })
        .unwrap();
    assert!(service
        .run(WorkflowAction::Review {
            target: target.clone(),
            request_id: "operator/request".into(),
            id: "operator/self".into(),
            decision: ReviewDecision::Accepted,
            rationale: "self review denied".into(),
            evidence: "fixture/self".into()
        })
        .is_err());
    let author_publication = operator.snapshot().unwrap();
    let author_draft = DraftBuffer::new("private unsubmitted author source".into());
    let author_draft_clone = author_draft.clone();
    let reviewer_draft = DraftBuffer::new(author_publication.latest_source.clone());
    reviewer_draft.set("reviewer private inspection note".into());
    assert_ne!(author_draft.read().unwrap(), reviewer_draft.read().unwrap());
    assert_eq!(
        author_draft_clone.read().unwrap(),
        "private unsubmitted author source"
    );
    let reviewer = login(&authority, "fixture-reviewer", &REVIEWER);
    *sessions.lock().unwrap() = Some(reviewer.clone());
    assert!(
        editor.snapshot().is_ok(),
        "reviewer login preserves the author session"
    );
    assert!(
        !author_publication.publication_is_current(),
        "selected-window publication cannot cross actor handoff"
    );
    drop(author_publication);
    let review_view = operator.workflow(WorkflowAction::Inspect).unwrap();
    assert_eq!(review_view.evidence.principal_subject, "fixture-reviewer");
    assert_eq!(
        author_draft_clone.read().unwrap(),
        "private unsubmitted author source"
    );
    assert!(!review_view
        .source_cells
        .iter()
        .any(|cell| cell.contains("private unsubmitted author source")));
    assert_eq!(review_view.evidence.target, target);
    assert!(review_view.evidence.can_review);
    assert!(!review_view.evidence.can_contribute);
    assert_eq!(
        review_view
            .evidence
            .package_writes
            .last()
            .unwrap()
            .result_digest,
        package.digest
    );
    let mut stale = target.clone();
    stale.source_digest = [9; 32];
    assert!(service
        .run(WorkflowAction::Review {
            target: stale,
            request_id: "operator/request".into(),
            id: "operator/stale".into(),
            decision: ReviewDecision::Accepted,
            rationale: "stale".into(),
            evidence: "fixture/stale".into()
        })
        .is_err());
    let decided = service
        .run(WorkflowAction::Review {
            target: target.clone(),
            request_id: "operator/request".into(),
            id: "operator/decision".into(),
            decision: ReviewDecision::ChangesRequested,
            rationale: "Source inspected; master/HMC acceptance remains separate".into(),
            evidence: "fixture/source-inspection".into(),
        })
        .unwrap();
    assert_eq!(
        decided.evidence.reviews[0].principal_kind,
        PrincipalKind::SoftwareAgent
    );
    assert_eq!(
        decided.evidence.requests[0].decision_ids,
        vec!["operator/decision".to_string()]
    );
    let loaded = checkpoint::load(&package.path, CheckpointLimits::default()).unwrap();
    assert_eq!(
        loaded
            .resources()
            .iter()
            .find(|resource| resource.key == master.key),
        Some(&master)
    );
    assert_eq!(
        loaded
            .resources()
            .iter()
            .find(|resource| resource.key == opaque.key),
        Some(&opaque)
    );
    authority.disconnect(&reviewer).unwrap();
    assert!(service.run(WorkflowAction::Inspect).is_err());
    assert!(operator.workflow(WorkflowAction::Inspect).is_err());
    drop(editor);
    drop(operator);
    drop(service);
    drop(project);
    drop(authority);
    // Reopen the actual native journal. Existing process-local Studio source is
    // retained here; this is not a fresh-process/source-reopen qualification.
    let authority = NativeAuthority::open(
        &authority_dir,
        TrustedProject {
            project_id: scope.project_id.clone(),
            issuer_identifier: "fixture-issuer".into(),
            public_key: ed25519::public_from_seed(&AUTHORITY),
        },
        signed(admission),
        target.clone(),
        1024 * 1024,
        256,
    )
    .unwrap();
    assert!(authority.workflow_evidence(&reviewer).is_err());
    let reviewer = login(&authority, "fixture-reviewer", &REVIEWER);
    let retained = authority.workflow_evidence(&reviewer).unwrap();
    assert_eq!(retained.target, target);
    assert_eq!(retained.contributions.len(), 1);
    assert_eq!(retained.requests.len(), 1);
    assert_eq!(retained.reviews.len(), 1);
    assert_eq!(
        retained.package_writes.last().unwrap().result_digest,
        package.digest
    );
}
