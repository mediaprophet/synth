//! Explicit fixture authority only. These are not production credentials.
use super::*;
use crate::graphical::{
    controller::{CanonicalEditorHost, PreviewProvider},
    geometry_host::GeometryHost,
    preview::PreviewFrame,
};
use qualia_cooperative_core::{
    authoring::*,
    projects::{Project, ProjectMembership, ProjectRole},
};
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Mutex,
};
use vibe::session::SessionLimits;

#[path = "georaster_edit_tests.rs"]
mod georaster_edit_tests;
#[path = "mcp_tests.rs"]
mod mcp_tests;

#[test]
fn selected_resource_inspection_requires_current_read_authority_and_source() {
    use crate::graphical::checkpoints::{ui_inspection_limits, ResourceInspectionRequest};
    use qualia_core_db::q42::m64_matter::M64PayloadInterval;
    let scope = scope();
    let expected = reference(&scope);
    let request = ResourceInspectionRequest {
        filename: "fixture.hmc".into(),
        package_digest: [0; 32],
        key: "raster.m64".into(),
        resource_digest: [0; 32],
    };
    let mut scratch = [M64PayloadInterval::default(); 1];
    let unbound = editor(scope.clone(), Arc::new(UnboundAuthorityOwner));
    let error = unbound
        .inspect_package_resource(
            request.clone(),
            expected.clone(),
            ui_inspection_limits(),
            &mut scratch,
        )
        .unwrap_err();
    assert!(
        !error.contains("unavailable for this host"),
        "authority denies before inner dispatch"
    );
    let owner = Arc::new(FixtureOwner::new(&scope, ProjectRole::Contributor));
    let current = editor(scope.clone(), owner.clone());
    // The raw canonical fixture intentionally has no resource service: its
    // unavailable result proves valid read admission reached the inner host.
    assert!(current
        .inspect_package_resource(
            request.clone(),
            expected.clone(),
            ui_inspection_limits(),
            &mut scratch
        )
        .unwrap_err()
        .contains("unavailable for this host"));
    let published = current.snapshot().unwrap();
    owner.state.lock().unwrap().revoked = true;
    assert!(!published.publication_is_current());
    let error = current
        .inspect_package_resource(
            request.clone(),
            expected.clone(),
            ui_inspection_limits(),
            &mut scratch,
        )
        .unwrap_err();
    assert!(!error.contains("unavailable for this host"));
    owner.state.lock().unwrap().revoked = false;
    studio_store::upsert_artifact(studio_store::get_artifact(&scope.source_id).unwrap());
    let error = current
        .inspect_package_resource(request, expected, ui_inspection_limits(), &mut scratch)
        .unwrap_err();
    assert!(error.contains("stamp differs"));
}

static NEXT: AtomicU64 = AtomicU64::new(1);
fn scope() -> Scope {
    let serial = NEXT.fetch_add(1, Ordering::Relaxed);
    let id = format!(
        "urn:poet:cooperation-fixture:{}:{serial}",
        std::process::id()
    );
    studio_store::create_artifact_if_absent(StudioArtifact::new(&id, "Geometry", "fixture source"))
        .unwrap();
    Scope {
        project_id: "fixture-project".into(),
        document_id: format!("fixture-document-{serial}"),
        source_id: id,
        domain: "Geometry".into(),
    }
}
fn document(scope: &Scope) -> DocumentRevision {
    let (source, _) = studio_store::get_artifact_stamped(&scope.source_id).unwrap();
    DocumentRevision {
        project_id: scope.project_id.clone(),
        document_id: scope.document_id.clone(),
        source_id: scope.source_id.clone(),
        revision: source.lamport_clock.into(),
        source_digest: source_digest(&source, GatewayLimits::default()).unwrap(),
        author_subject: Some("fixture-subject".into()),
    }
}
struct FixtureState {
    role: ProjectRole,
    document: DocumentRevision,
    generation: u64,
    revoked: bool,
    calls: usize,
}
struct FixtureOwner {
    scope: Scope,
    state: Mutex<FixtureState>,
}
impl FixtureOwner {
    fn new(scope: &Scope, role: ProjectRole) -> Self {
        Self {
            scope: scope.clone(),
            state: Mutex::new(FixtureState {
                role,
                document: document(scope),
                generation: 1,
                revoked: false,
                calls: 1,
            }),
        }
    }
    fn refresh_document(&self) {
        self.state.lock().unwrap().document = document(&self.scope);
    }
}
impl AuthorityOwner for FixtureOwner {
    fn publication_is_current(&self, generation: u64) -> bool {
        self.state
            .try_lock()
            .is_ok_and(|state| state.generation == generation && !state.revoked)
    }
    fn with_current(
        &self,
        operation: &mut dyn FnMut(&HostAuthorityView<'_>) -> Result<(), String>,
    ) -> Result<(), String> {
        let state = self.state.lock().map_err(|_| "fixture poisoned")?;
        let project = Project {
            id: self.scope.project_id.clone(),
            name: "fixture".into(),
            description: String::new(),
            licensing_ontologies: vec![],
            created_at_unix: 1,
            created_at_instant: None,
        };
        let member = ProjectMembership {
            id: "fixture-member".into(),
            project_id: self.scope.project_id.clone(),
            member_did: "did:test:contributor".into(),
            role: state.role.clone(),
            agreed_at_unix: 1,
            agreed_at_instant: None,
        };
        let ids = [QualifiedIdentifier {
            value: "did:test:contributor",
            scheme: "did:test",
            issuer: "fixture-issuer",
            project_id: &self.scope.project_id,
            evidence_ref: "fixture-association",
            valid_from_unix: 1,
            valid_until_unix: 1000,
            revoked: false,
        }];
        let principal = AuthenticatedPrincipal {
            subject: "fixture-subject",
            kind: PrincipalKind::SoftwareAgent,
            authentication_ref: "fixture-only",
            authenticated_until_unix: 1000,
            identifiers: &ids,
        };
        let members = [AdmittedMembership {
            membership: &member,
            member_subject: "fixture-subject",
            admission_ref: "fixture-admission",
            valid_from_unix: 1,
            valid_until_unix: 1000,
            revoked: state.revoked,
        }];
        let view = HostAuthorityView {
            principal: Some(&principal),
            project: &project,
            memberships: &members,
            review_grants: &[],
            current_document: &state.document,
            generation: state.generation,
            now_unix: 100,
            limits: AuthoringLimits::default(),
        };
        for _ in 0..state.calls {
            operation(&view)?;
        }
        Ok(())
    }
}
struct NoPreview;
impl PreviewProvider for NoPreview {
    // This headless fixture does not retain a viewport or frame.
    fn retire_retained_viewport(&mut self) -> Result<(), String> { Ok(()) }
    fn preview(&mut self, _: SourceRef, _: &vibe::Value) -> Result<PreviewFrame, String> {
        Err("fixture does not create fake rendered frames".into())
    }
}
fn editor(scope: Scope, owner: Arc<dyn AuthorityOwner>) -> AuthorizedEditorHost {
    let inner = CanonicalEditorHost::open(
        scope.source_id.clone(),
        SessionLimits::default(),
        GeometryHost::default(),
        Box::new(NoPreview),
    )
    .unwrap();
    AuthorizedEditorHost::new(Box::new(inner), scope, owner, GatewayLimits::default()).unwrap()
}
fn reference(scope: &Scope) -> SourceRef {
    SourceRef {
        artifact_id: scope.source_id.clone(),
        stamp: studio_store::get_artifact_stamped(&scope.source_id)
            .unwrap()
            .1,
    }
}

#[test]
fn unbound_native_owner_denies_reads_checks_and_writes_without_source_mutation() {
    let scope = scope();
    let original = reference(&scope);
    let mut editor = editor(scope.clone(), Arc::new(UnboundAuthorityOwner));
    assert!(editor.snapshot().is_err());
    assert!(editor.check("1;").is_err());
    assert!(editor.submit("let x = 1;", original.clone()).is_err());
    assert_eq!(reference(&scope), original);
}

#[test]
fn observer_reads_actual_source_but_every_edit_route_denies() {
    let scope = scope();
    let owner = Arc::new(FixtureOwner::new(&scope, ProjectRole::Observer));
    let mut editor = editor(scope.clone(), owner);
    let snapshot = editor.snapshot().unwrap();
    assert_eq!(snapshot.source, reference(&scope));
    assert!(editor.check("1;").is_ok());
    assert!(editor
        .submit("let x = 1;", snapshot.source.clone())
        .is_err());
    assert!(editor
        .material_edit("let x = 1;", snapshot.source.clone())
        .is_err());
    assert!(editor.undo(snapshot.source.clone()).is_err());
    assert!(editor.redo(snapshot.source.clone()).is_err());
    assert!(editor.rebuild_execution(snapshot.source.clone()).is_err());
    assert!(editor
        .save_checkpoint(
            "never-written.hmc",
            studio_store::checkpoint::ExpectedFile::Absent,
            snapshot.source.clone()
        )
        .is_err());
    assert!(studio_store::get_artifact_stamped(&scope.source_id)
        .unwrap()
        .0
        .nodes
        .is_empty());
}

#[test]
fn real_canonical_write_requires_native_owner_document_update_before_next_action() {
    let scope = scope();
    let owner = Arc::new(FixtureOwner::new(&scope, ProjectRole::Contributor));
    let mut editor = editor(scope.clone(), owner.clone());
    let before = editor.snapshot().unwrap();
    let accepted = editor
        .submit("let mut fixture_value: I64 = 1;", before.source)
        .unwrap();
    assert_eq!(accepted.snapshot.session_revision, 1);
    assert!(accepted.frame.is_none());
    let accepted_stamp = accepted
        .snapshot
        .authorisation_stamp
        .clone()
        .expect("admitted outcome publication stamp");
    assert!(accepted_stamp.is_current());
    assert!(
        editor.snapshot().is_err(),
        "stale native document must not follow a source write implicitly"
    );
    owner.refresh_document();
    let current = editor.snapshot().unwrap();
    assert_eq!(current.source, accepted.snapshot.source);
    assert_eq!(current.authorisation_stamp.as_ref(), Some(&accepted_stamp));
    let sources = editor.source_cells(accepted.snapshot.source).unwrap();
    assert_eq!(sources, vec!["let mut fixture_value: I64 = 1;"]);
    let undone = editor.undo(current.source).unwrap();
    assert_eq!(undone.authorisation_stamp.as_ref(), Some(&accepted_stamp));
    owner.refresh_document();
    let rebuilt = editor.rebuild_execution(undone.source).unwrap();
    assert_eq!(rebuilt.snapshot.session_revision, 0);
    assert_eq!(
        rebuilt.snapshot.authorisation_stamp.as_ref(),
        Some(&accepted_stamp)
    );
    let current = editor.snapshot().unwrap();
    assert_eq!(
        current.authorisation_stamp,
        rebuilt.snapshot.authorisation_stamp
    );
    let redone = editor.redo(current.source).unwrap();
    assert_eq!(redone.authorisation_stamp.as_ref(), Some(&accepted_stamp));
    owner.refresh_document();
    let rebuilt = editor.rebuild_execution(redone.source).unwrap();
    assert_eq!(rebuilt.snapshot.session_revision, 1);
    assert_eq!(
        rebuilt.snapshot.authorisation_stamp,
        editor.snapshot().unwrap().authorisation_stamp
    );
}

#[test]
fn foreign_scope_stamp_source_digest_and_current_generation_fail_closed() {
    let scope = scope();
    let foreign = super::tests::scope();
    let owner = Arc::new(FixtureOwner::new(&scope, ProjectRole::Contributor));
    let mut editor = editor(scope.clone(), owner.clone());
    assert!(editor.submit("let x = 1;", reference(&foreign)).is_err());
    owner.state.lock().unwrap().document.source_digest = [9; 32];
    assert!(editor.snapshot().is_err());
    owner.refresh_document();
    owner.state.lock().unwrap().document.document_id = "different-document".into();
    assert!(editor.snapshot().is_err());
    owner.refresh_document();
    owner.state.lock().unwrap().generation = 0;
    assert!(editor.snapshot().is_err());
    owner.state.lock().unwrap().generation = 2;
    owner.state.lock().unwrap().revoked = true;
    assert!(editor.snapshot().is_err());
}

#[test]
fn provider_never_calls_or_calls_twice_never_repeats_underlying_operation() {
    let scope = scope();
    let owner = FixtureOwner::new(&scope, ProjectRole::Contributor);
    let mut actual_calls = 0;
    owner.state.lock().unwrap().calls = 0;
    assert!(with_authorized(
        &owner,
        &scope,
        None,
        AuthoringAction::ReadSource,
        GatewayLimits::default(),
        |_, _| {
            actual_calls += 1;
            Ok(())
        }
    )
    .is_err());
    assert_eq!(actual_calls, 0);
    owner.state.lock().unwrap().calls = 2;
    assert!(with_authorized(
        &owner,
        &scope,
        None,
        AuthoringAction::ReadSource,
        GatewayLimits::default(),
        |_, _| {
            actual_calls += 1;
            Ok(())
        }
    )
    .is_err());
    assert_eq!(actual_calls, 1);
}

#[test]
fn self_review_and_unbounded_source_digest_are_refused() {
    let scope = scope();
    let owner = FixtureOwner::new(&scope, ProjectRole::Contributor);
    let error = with_authorized(
        &owner,
        &scope,
        None,
        AuthoringAction::ReviewDecision,
        GatewayLimits::default(),
        |_, _| Ok(()),
    )
    .unwrap_err();
    assert!(error.contains("SelfReview"));
    let (artifact, _) = studio_store::get_artifact_stamped(&scope.source_id).unwrap();
    assert!(source_digest(
        &artifact,
        GatewayLimits {
            max_digest_bytes: 8,
            ..GatewayLimits::default()
        }
    )
    .is_err());
    let mut bad = artifact;
    bad.nodes.insert(
        "poet.native.submission/00000000000000000002".into(),
        std::collections::BTreeMap::from([("text".into(), "1;".into())]),
    );
    assert!(source_digest(&bad, GatewayLimits::default()).is_err());
}

#[test]
fn publication_stamp_rejects_revocation_new_generation_and_busy_owner_without_blocking() {
    let scope = scope();
    let owner = Arc::new(FixtureOwner::new(&scope, ProjectRole::Contributor));
    let editor = editor(scope, owner.clone());
    let completed = editor.snapshot().unwrap();
    let stamp = completed.authorisation_stamp.unwrap();
    assert!(stamp.is_current());
    assert_eq!(stamp, stamp.clone());
    {
        let mut state = owner.state.lock().unwrap();
        assert!(
            !stamp.is_current(),
            "contended fixture owner must return false without blocking"
        );
        state.revoked = true;
        state.generation += 1;
    }
    assert!(
        !stamp.is_current(),
        "completion from before revocation cannot publish"
    );
    {
        let mut state = owner.state.lock().unwrap();
        state.revoked = false;
        state.generation += 1;
    }
    let next = editor.snapshot().unwrap().authorisation_stamp.unwrap();
    assert!(next.is_current());
    assert_ne!(next, stamp);
    assert!(
        !stamp.is_current(),
        "new authorisation cannot revive an old completion"
    );
}

struct NoFastGuard(FixtureOwner);
impl AuthorityOwner for NoFastGuard {
    fn with_current(
        &self,
        operation: &mut dyn FnMut(&HostAuthorityView<'_>) -> Result<(), String>,
    ) -> Result<(), String> {
        self.0.with_current(operation)
    }
}
#[test]
fn owner_without_publication_guard_keeps_native_operations_but_cannot_publish_completion() {
    let scope = scope();
    let owner = Arc::new(NoFastGuard(FixtureOwner::new(
        &scope,
        ProjectRole::Contributor,
    )));
    let mut editor = editor(scope.clone(), owner);
    let snapshot = editor.snapshot().unwrap();
    assert!(!snapshot.authorisation_stamp.as_ref().unwrap().is_current());
    let accepted = editor
        .submit("let fixture_work: I64 = 1;", snapshot.source)
        .unwrap();
    assert_eq!(accepted.snapshot.session_revision, 1);
}

// Scripted authority fixtures exercise shared ownership only; they do not prove
// GPU behavior, a native human workflow, or every possible publication race.
mod viewport_outer_transaction_tests {
    use super::*;
    use crate::graphical::{controller::{EditorHost, EditorOutcome, EditorSnapshot}, preview::PreviewCamera,
        shared_host::SharedEditorHost, viewport::ViewportCommand, viewport_contract::{ViewportOutput, ViewportState}};
    use std::sync::{atomic::AtomicBool, mpsc};

    struct FakeViewport { dirty: Arc<AtomicBool>, can_retire: bool }
    impl EditorHost for FakeViewport {
        fn viewport_action(&mut self, _: ViewportCommand, source: SourceRef, _: &AtomicBool) -> Result<ViewportOutput, String> {
            self.dirty.store(true, Ordering::Release);
            Ok(ViewportOutput { frame: PreviewFrame { source: source.clone(), session_revision: None, execution_epoch: None, side: 1, rgba8: vec![0;4] },
                state: ViewportState { source, execution_epoch: None, frame_id: 1, selected_part: None,
                    camera: PreviewCamera { yaw: 0.0, pitch: 0.0, distance: 1.0, target: [0.0;3] } } })
        }
        fn retire_retained_viewport(&mut self) -> Result<(), String> {
            if !self.can_retire { return Err("retirement unsupported".into()); }
            self.dirty.store(false, Ordering::Release); Ok(())
        }
        fn snapshot(&self) -> Result<EditorSnapshot,String> { Err("fixture".into()) }
        fn check(&self, _: &str) -> Result<String,String> { Err("fixture".into()) }
        fn submit(&mut self, _: &str, _: SourceRef) -> Result<EditorOutcome,String> { Err("fixture".into()) }
        fn undo(&mut self, _: SourceRef) -> Result<EditorSnapshot,String> { Err("fixture".into()) }
        fn redo(&mut self, _: SourceRef) -> Result<EditorSnapshot,String> { Err("fixture".into()) }
    }
    struct RejectAfter { inner: FixtureOwner }
    impl AuthorityOwner for RejectAfter {
        fn publication_is_current(&self, generation: u64) -> bool { self.inner.publication_is_current(generation) }
        fn with_current(&self, op: &mut dyn FnMut(&HostAuthorityView<'_>) -> Result<(),String>) -> Result<(),String> {
            self.inner.with_current(&mut |view| { op(view)?; Err("scripted post-callback rejection".into()) })
        }
    }
    struct PauseReject { inner: FixtureOwner, entered: mpsc::Sender<()>, resume: Mutex<mpsc::Receiver<()>> }
    impl AuthorityOwner for PauseReject {
        fn publication_is_current(&self, generation: u64) -> bool { self.inner.publication_is_current(generation) }
        fn with_current(&self, op: &mut dyn FnMut(&HostAuthorityView<'_>) -> Result<(),String>) -> Result<(),String> {
            self.inner.with_current(&mut |view| { op(view)?; self.entered.send(()).unwrap();
                self.resume.lock().unwrap().recv().unwrap(); Err("scripted post-callback rejection".into()) })
        }
    }
    fn wrap(scope: &Scope, shared: &SharedEditorHost, owner: Arc<dyn AuthorityOwner>) -> AuthorizedEditorHost {
        AuthorizedEditorHost::new(Box::new(shared.clone()), scope.clone(), owner, GatewayLimits::default()).unwrap()
    }
    #[test]
    fn outer_viewport_transaction_excludes_frontend_until_rejected_frame_retired() {
        let scope = scope(); let source = reference(&scope); let dirty = Arc::new(AtomicBool::new(false));
        let shared = SharedEditorHost::new(Box::new(FakeViewport { dirty: dirty.clone(), can_retire: true }));
        let (entered_tx, entered_rx) = mpsc::channel(); let (resume_tx, resume_rx) = mpsc::channel();
        let owner = Arc::new(PauseReject { inner: FixtureOwner::new(&scope, ProjectRole::Contributor), entered: entered_tx, resume: Mutex::new(resume_rx) });
        let mut first = wrap(&scope, &shared, owner); let first_source = source.clone();
        let worker = std::thread::spawn(move || first.viewport_action(ViewportCommand::Redraw, first_source, &AtomicBool::new(false)));
        entered_rx.recv().unwrap(); assert!(dirty.load(Ordering::Acquire));
        let mut second = wrap(&scope, &shared, Arc::new(FixtureOwner::new(&scope, ProjectRole::Contributor)));
        assert!(second.viewport_action(ViewportCommand::Redraw, source.clone(), &AtomicBool::new(false)).err().unwrap().contains("busy"));
        resume_tx.send(()).unwrap(); assert!(worker.join().unwrap().is_err());
        assert!(!dirty.load(Ordering::Acquire), "refused candidate must retire before release");
        second.viewport_action(ViewportCommand::Redraw, source, &AtomicBool::new(false)).unwrap();
        assert!(dirty.load(Ordering::Acquire), "later accepted frame must remain available");
    }
    #[test]
    fn outer_viewport_transaction_quarantines_if_retirement_unavailable() {
        let scope = scope(); let source = reference(&scope); let dirty = Arc::new(AtomicBool::new(false));
        let shared = SharedEditorHost::new(Box::new(FakeViewport { dirty, can_retire: false }));
        let mut first = wrap(&scope, &shared, Arc::new(RejectAfter { inner: FixtureOwner::new(&scope, ProjectRole::Contributor) }));
        assert!(first.viewport_action(ViewportCommand::Redraw, source.clone(), &AtomicBool::new(false)).is_err());
        let mut second = wrap(&scope, &shared, Arc::new(FixtureOwner::new(&scope, ProjectRole::Contributor)));
        assert!(second.viewport_action(ViewportCommand::Redraw, source, &AtomicBool::new(false)).err().unwrap().contains("quarantined"));
    }
    struct CancelAfter { inner: FixtureOwner, cancel: Arc<AtomicBool> }
    impl AuthorityOwner for CancelAfter {
        fn publication_is_current(&self, generation: u64) -> bool { self.inner.publication_is_current(generation) }
        fn with_current(&self, op: &mut dyn FnMut(&HostAuthorityView<'_>) -> Result<(),String>) -> Result<(),String> {
            self.inner.with_current(&mut |view| { op(view)?; self.cancel.store(true, Ordering::Release); Ok(()) })
        }
    }
    #[test]
    fn outer_viewport_transaction_retires_on_post_callback_cancellation() {
        let scope = scope(); let source = reference(&scope); let dirty = Arc::new(AtomicBool::new(false));
        let shared = SharedEditorHost::new(Box::new(FakeViewport { dirty: dirty.clone(), can_retire: true }));
        let cancel = Arc::new(AtomicBool::new(false));
        let owner = Arc::new(CancelAfter { inner: FixtureOwner::new(&scope, ProjectRole::Contributor), cancel: cancel.clone() });
        let mut editor = wrap(&scope, &shared, owner);
        assert!(editor.viewport_action(ViewportCommand::Redraw, source, &cancel).is_err());
        assert!(!dirty.load(Ordering::Acquire));
    }
    #[test]
    fn outer_viewport_transaction_retires_on_duplicate_callback() {
        let scope = scope(); let source = reference(&scope); let dirty = Arc::new(AtomicBool::new(false));
        let shared = SharedEditorHost::new(Box::new(FakeViewport { dirty: dirty.clone(), can_retire: true }));
        let owner = Arc::new(FixtureOwner::new(&scope, ProjectRole::Contributor)); owner.state.lock().unwrap().calls = 2;
        let mut editor = wrap(&scope, &shared, owner);
        assert!(editor.viewport_action(ViewportCommand::Redraw, source, &AtomicBool::new(false)).is_err());
        assert!(!dirty.load(Ordering::Acquire));
    }
}


// Scripted owner and CPU provider: source commits are real, rendered pixels and
// native-authority/gpu behavior require separate integration checks.
mod source_submit_transaction_tests {
    use super::*;
    use crate::graphical::shared_host::SharedEditorHost;
    use std::sync::{atomic::AtomicBool, mpsc};
    struct TrackedPreview { dirty: Arc<AtomicBool>, failed: bool, retire: bool }
    impl PreviewProvider for TrackedPreview {
        fn preview(&mut self, source: SourceRef, _: &vibe::Value) -> Result<PreviewFrame, String> {
            self.dirty.store(true, Ordering::Release);
            if self.failed { return Err("fixture preview failed after mutation".into()); }
            Ok(PreviewFrame { source, session_revision: None, execution_epoch: None, side: 1, rgba8: vec![0; 4] })
        }
        fn retire_retained_viewport(&mut self) -> Result<(),String> {
            if !self.retire { return Err("cannot retire".into()); }
            self.dirty.store(false, Ordering::Release); Ok(())
        }
    }
    struct PauseReject { inner: FixtureOwner, entered: mpsc::Sender<()>, resume: Mutex<mpsc::Receiver<()>> }
    impl AuthorityOwner for PauseReject {
        fn with_current(&self, op: &mut dyn FnMut(&HostAuthorityView<'_>) -> Result<(), String>) -> Result<(), String> {
            self.inner.with_current(&mut |view| { op(view)?; self.entered.send(()).unwrap(); self.resume.lock().unwrap().recv().unwrap(); Err("fixture post-submit refusal".into()) })
        }
    }
    fn shared(scope: &Scope, dirty: Arc<AtomicBool>, failed: bool, retire: bool) -> SharedEditorHost {
        let host=CanonicalEditorHost::open(scope.source_id.clone(), SessionLimits::default(), GeometryHost::default(), Box::new(TrackedPreview { dirty, failed, retire })).unwrap();
        SharedEditorHost::new(Box::new(host))
    }
    fn wrap(scope: &Scope, shared: &SharedEditorHost, owner: Arc<dyn AuthorityOwner>) -> AuthorizedEditorHost {
        AuthorizedEditorHost::new(Box::new(shared.clone()), scope.clone(), owner, GatewayLimits::default()).unwrap()
    }
    #[test]
    fn source_submit_transaction_holds_shared_owner_through_outer_refusal() {
        let scope=scope(); let before=reference(&scope); let dirty=Arc::new(AtomicBool::new(false));
        let shared=shared(&scope, dirty.clone(), false, true); let other=shared.clone();
        let (entered_tx,entered_rx)=mpsc::channel(); let (resume_tx,resume_rx)=mpsc::channel();
        let owner=Arc::new(PauseReject { inner:FixtureOwner::new(&scope,ProjectRole::Contributor), entered:entered_tx, resume:Mutex::new(resume_rx) });
        let mut writer=wrap(&scope,&shared,owner); let first=before.clone();
        let task=std::thread::spawn(move || writer.submit("let mut source_tx_value: I64 = 1;",first));
        entered_rx.recv().unwrap(); assert!(dirty.load(Ordering::Acquire));
        assert!(other.snapshot().err().unwrap().contains("busy"));
        let committed=reference(&scope); assert_ne!(before, committed, "source commit is real and not rolled back");
        resume_tx.send(()).unwrap(); let error=task.join().unwrap().err().unwrap();
        assert!(error.contains("committed") && error.contains("reconcil"));
        assert!(!dirty.load(Ordering::Acquire)); assert_eq!(reference(&scope),committed);
        let snapshot=other.snapshot().unwrap(); assert_eq!(snapshot.source,committed);
    }
    #[test]
    fn source_submit_transaction_retires_failed_preview_without_losing_commit() {
        let scope=scope(); let before=reference(&scope); let dirty=Arc::new(AtomicBool::new(false));
        let shared=shared(&scope,dirty.clone(),true,true); let owner=Arc::new(FixtureOwner::new(&scope,ProjectRole::Contributor));
        let mut writer=wrap(&scope,&shared,owner); let result=writer.submit("let mut failed_preview_value: I64 = 1;",before.clone()).unwrap();
        assert!(result.frame.is_none()); assert!(!dirty.load(Ordering::Acquire)); assert_ne!(before,reference(&scope));
        assert!(result.message.contains("preview unavailable"));
    }
    #[test]
    fn source_submit_transaction_quarantines_unretirable_failed_preview() {
        let scope=scope(); let before=reference(&scope); let dirty=Arc::new(AtomicBool::new(false));
        let shared=shared(&scope,dirty,true,false); let owner=Arc::new(FixtureOwner::new(&scope,ProjectRole::Contributor));
        let mut writer=wrap(&scope,&shared,owner); let error=writer.submit("let mut quarantine_value: I64 = 1;",before.clone()).err().unwrap();
        assert!(error.contains("quarantined") && error.contains("committed")); assert_ne!(before,reference(&scope));
        assert!(shared.snapshot().err().unwrap().contains("quarantined"));
    }
    #[test]
    fn source_submit_transaction_duplicate_callback_retires_without_repeating_commit() {
        let scope=scope(); let before=reference(&scope); let dirty=Arc::new(AtomicBool::new(false));
        let shared=shared(&scope,dirty.clone(),false,true); let owner=Arc::new(FixtureOwner::new(&scope,ProjectRole::Contributor));
        owner.state.lock().unwrap().calls=2; let mut writer=wrap(&scope,&shared,owner);
        let error=writer.submit("let mut duplicate_value: I64 = 1;",before.clone()).err().unwrap();
        assert!(error.contains("not repeated") && error.contains("reconcil"));
        assert!(!dirty.load(Ordering::Acquire)); assert_ne!(before,reference(&scope));
        assert_eq!(shared.source_cells(reference(&scope)).unwrap().len(),1);
    }
    struct MisboundFrame(CanonicalEditorHost);
    impl EditorHost for MisboundFrame {
        fn submit(&mut self, code: &str, source: SourceRef) -> Result<EditorOutcome,String> {
            let mut result=self.0.submit(code,source)?;
            result.frame.as_mut().unwrap().session_revision=None;
            Ok(result)
        }
        fn retire_retained_viewport(&mut self) -> Result<(),String> { self.0.retire_retained_viewport() }
        fn snapshot(&self)->Result<EditorSnapshot,String> { self.0.snapshot() }
        fn check(&self, code:&str)->Result<String,String> { self.0.check(code) }
        fn undo(&mut self, source:SourceRef)->Result<EditorSnapshot,String> { self.0.undo(source) }
        fn redo(&mut self, source:SourceRef)->Result<EditorSnapshot,String> { self.0.redo(source) }
    }
    #[test]
    fn source_submit_transaction_withholds_unbound_session_frame() {
        let scope=scope(); let before=reference(&scope); let dirty=Arc::new(AtomicBool::new(false));
        let inner=CanonicalEditorHost::open(scope.source_id.clone(), SessionLimits::default(), GeometryHost::default(), Box::new(TrackedPreview { dirty:dirty.clone(),failed:false,retire:true })).unwrap();
        let shared=SharedEditorHost::new(Box::new(MisboundFrame(inner))); let owner=Arc::new(FixtureOwner::new(&scope,ProjectRole::Contributor));
        let mut writer=wrap(&scope,&shared,owner); let outcome=writer.submit("let mut unbound_preview_value: I64 = 1;",before.clone()).unwrap();
        assert!(outcome.frame.is_none()); assert!(outcome.message.contains("mismatched preview withheld"));
        assert!(!dirty.load(Ordering::Acquire)); assert_ne!(before,reference(&scope));
    }
    #[test]
    fn source_submit_transaction_retains_accepted_exact_frame() {
        let scope=scope(); let before=reference(&scope); let dirty=Arc::new(AtomicBool::new(false));
        let shared=shared(&scope,dirty.clone(),false,true); let owner=Arc::new(FixtureOwner::new(&scope,ProjectRole::Contributor));
        let mut writer=wrap(&scope,&shared,owner); let result=writer.submit("let mut accepted_preview_value: I64 = 1;",before).unwrap();
        assert_eq!(result.frame.unwrap().source,result.snapshot.source); assert!(dirty.load(Ordering::Acquire));
    }
}


// Scripted owners verify lock/publication behavior; no GPU or human acceptance.
mod surface_binding_transaction_tests {
    use super::*;
    use crate::graphical::{controller::{EditorHost, EditorOutcome, EditorSnapshot}, shared_host::SharedEditorHost,
        surface_host::SurfacePackageRequest};
    use std::sync::{atomic::{AtomicBool, AtomicUsize}, mpsc};
    use std::time::Duration;

    struct BindingHost { changed: Arc<AtomicBool>, calls: Arc<AtomicUsize>, fail_after_change: bool }
    impl EditorHost for BindingHost {
        fn bind_surface_package(&mut self, _: SurfacePackageRequest, _: SourceRef) -> Result<(), String> {
            self.calls.fetch_add(1, Ordering::AcqRel);
            self.changed.store(true, Ordering::Release);
            if self.fail_after_change { Err("fixture failed after changing binding".into()) } else { Ok(()) }
        }
        fn bind_surface_package_transaction(&mut self, request: SurfacePackageRequest, source: SourceRef)
            -> Result<(), crate::graphical::surface_host::SurfaceBindingFailure> {
            use crate::graphical::surface_host::SurfaceBindingFailure;
            if request.filename == "missing.hmc" { return Err(SurfaceBindingFailure::BeforeMutation("fixture preflight".into())); }
            self.bind_surface_package(request, source).map_err(SurfaceBindingFailure::MayHaveMutated)
        }
        fn snapshot(&self) -> Result<EditorSnapshot, String> { Err("fixture snapshot".into()) }
        fn check(&self, _: &str) -> Result<String, String> { Ok("available".into()) }
        fn submit(&mut self, _: &str, _: SourceRef) -> Result<EditorOutcome, String> { Err("fixture".into()) }
        fn undo(&mut self, _: SourceRef) -> Result<EditorSnapshot, String> { Err("fixture".into()) }
        fn redo(&mut self, _: SourceRef) -> Result<EditorSnapshot, String> { Err("fixture".into()) }
    }
    struct PauseReject { inner: FixtureOwner, entered: mpsc::Sender<()>, resume: Mutex<mpsc::Receiver<()>> }
    impl AuthorityOwner for PauseReject {
        fn with_current(&self, op: &mut dyn FnMut(&HostAuthorityView<'_>) -> Result<(), String>) -> Result<(), String> {
            self.inner.with_current(&mut |view| {
                op(view)?;
                self.entered.send(()).map_err(|_| "fixture receiver closed")?;
                self.resume.lock().map_err(|_| "fixture poisoned")?.recv_timeout(Duration::from_secs(15)).map_err(|_| "fixture resume timed out")?;
                Err("fixture post-binding refusal".into())
            })
        }
    }
    fn request() -> SurfacePackageRequest { SurfacePackageRequest { filename:"fixture.hmc".into(), digest:[7;32] } }
    fn wrap(scope: &Scope, shared: &SharedEditorHost, owner: Arc<dyn AuthorityOwner>) -> AuthorizedEditorHost {
        AuthorizedEditorHost::new(Box::new(shared.clone()), scope.clone(), owner, GatewayLimits::default()).unwrap()
    }
    fn host(changed: Arc<AtomicBool>, calls: Arc<AtomicUsize>, fail_after_change: bool) -> SharedEditorHost {
        SharedEditorHost::new(Box::new(BindingHost { changed, calls, fail_after_change }))
    }
    #[test]
    fn surface_binding_transaction_excludes_other_frontend_and_quarantines_outer_refusal() {
        let scope=scope(); let before=reference(&scope); let changed=Arc::new(AtomicBool::new(false));
        let calls=Arc::new(AtomicUsize::new(0)); let shared=host(changed.clone(),calls.clone(),false);
        let (entered_tx,entered_rx)=mpsc::channel(); let (resume_tx,resume_rx)=mpsc::channel();
        let owner=Arc::new(PauseReject { inner:FixtureOwner::new(&scope,ProjectRole::Contributor), entered:entered_tx, resume:Mutex::new(resume_rx) });
        let mut first=wrap(&scope,&shared,owner); let expected=before.clone();
        let worker=std::thread::spawn(move || first.bind_surface_package(request(),expected));
        entered_rx.recv_timeout(Duration::from_secs(15)).expect("binding callback must reach barrier");
        assert!(changed.load(Ordering::Acquire)); assert!(shared.check("x").unwrap_err().contains("busy"));
        resume_tx.send(()).unwrap(); let error=worker.join().unwrap().unwrap_err();
        assert!(error.contains("quarantined") && error.contains("reconcile"));
        assert!(shared.check("x").unwrap_err().contains("quarantined"));
        assert_eq!(calls.load(Ordering::Acquire),1); assert_eq!(reference(&scope),before);
    }
    #[test]
    fn surface_binding_transaction_accepts_binding_after_owner_checks() {
        let scope=scope(); let before=reference(&scope); let changed=Arc::new(AtomicBool::new(false)); let calls=Arc::new(AtomicUsize::new(0));
        let shared=host(changed.clone(),calls.clone(),false);
        let mut editor=wrap(&scope,&shared,Arc::new(FixtureOwner::new(&scope,ProjectRole::Contributor)));
        editor.bind_surface_package(request(),before.clone()).unwrap();
        assert!(changed.load(Ordering::Acquire)); assert_eq!(calls.load(Ordering::Acquire),1);
        assert_eq!(shared.check("x").unwrap(),"available"); assert_eq!(reference(&scope),before);
    }
    #[test]
    fn surface_binding_transaction_preserves_host_when_authority_refuses_before_dispatch() {
        let scope=scope(); let before=reference(&scope); let changed=Arc::new(AtomicBool::new(false)); let calls=Arc::new(AtomicUsize::new(0));
        let shared=host(changed.clone(),calls.clone(),false);
        let mut editor=wrap(&scope,&shared,Arc::new(UnboundAuthorityOwner));
        assert!(editor.bind_surface_package(request(),before.clone()).is_err());
        assert!(!changed.load(Ordering::Acquire)); assert_eq!(calls.load(Ordering::Acquire),0);
        assert_eq!(shared.check("x").unwrap(),"available"); assert_eq!(reference(&scope),before);
    }
    #[test]
    fn surface_binding_transaction_quarantines_inner_error_after_dispatch() {
        let scope=scope(); let before=reference(&scope); let changed=Arc::new(AtomicBool::new(false)); let calls=Arc::new(AtomicUsize::new(0));
        let shared=host(changed.clone(),calls.clone(),true);
        let mut editor=wrap(&scope,&shared,Arc::new(FixtureOwner::new(&scope,ProjectRole::Contributor)));
        assert!(editor.bind_surface_package(request(),before.clone()).unwrap_err().contains("quarantined"));
        assert!(changed.load(Ordering::Acquire)); assert_eq!(calls.load(Ordering::Acquire),1);
        assert!(shared.check("x").unwrap_err().contains("quarantined")); assert_eq!(reference(&scope),before);
    }
    #[test]
    fn surface_binding_transaction_duplicate_callback_is_not_repeated_and_quarantines() {
        let scope=scope(); let before=reference(&scope); let changed=Arc::new(AtomicBool::new(false)); let calls=Arc::new(AtomicUsize::new(0));
        let shared=host(changed.clone(),calls.clone(),false); let owner=Arc::new(FixtureOwner::new(&scope,ProjectRole::Contributor));
        owner.state.lock().unwrap().calls=2; let mut editor=wrap(&scope,&shared,owner);
        assert!(editor.bind_surface_package(request(),before.clone()).unwrap_err().contains("quarantined"));
        assert_eq!(calls.load(Ordering::Acquire),1); assert!(shared.check("x").unwrap_err().contains("quarantined"));
        assert_eq!(reference(&scope),before);
    }
    #[test]
    fn surface_binding_transaction_preflight_refusal_preserves_host_for_retry() {
        let scope=scope(); let before=reference(&scope); let changed=Arc::new(AtomicBool::new(false)); let calls=Arc::new(AtomicUsize::new(0));
        let shared=host(changed.clone(),calls.clone(),false);
        let mut editor=wrap(&scope,&shared,Arc::new(FixtureOwner::new(&scope,ProjectRole::Contributor)));
        let mut bad=request(); bad.filename="missing.hmc".into();
        assert!(editor.bind_surface_package(bad,before.clone()).unwrap_err().contains("preflight"));
        assert!(!changed.load(Ordering::Acquire)); assert_eq!(shared.check("x").unwrap(),"available");
        editor.bind_surface_package(request(),before.clone()).unwrap();
        assert!(changed.load(Ordering::Acquire)); assert_eq!(calls.load(Ordering::Acquire),1); assert_eq!(reference(&scope),before);
    }
    #[test]
    fn surface_binding_transaction_real_checkpoint_preflight_then_owner_rejection() {
        use crate::graphical::checkpoints::{CheckpointEditorHost, CheckpointLimits, CheckpointService, ExpectedFile, TrustedBinding};
        let scope=scope(); let before=reference(&scope); let temp=tempfile::tempdir().unwrap();
        let binding=TrustedBinding { project_id:scope.project_id.clone(), document_id:scope.document_id.clone(), source_id:scope.source_id.clone(), domain:scope.domain.clone() };
        let service=CheckpointService::new(temp.path(),binding,CheckpointLimits::default()).unwrap();
        let saved=service.save(&before.stamp,"fixture.hmc",ExpectedFile::Absent,None,vec![]).unwrap();
        let canonical=CanonicalEditorHost::open(scope.source_id.clone(),SessionLimits::default(),GeometryHost::default(),Box::new(NoPreview)).unwrap();
        let checkpoint=CheckpointEditorHost::new(Box::new(canonical),service).unwrap();
        let shared=SharedEditorHost::new(Box::new(checkpoint));
        let original_epoch=shared.snapshot().unwrap().execution_epoch;
        let mut admitted=wrap(&scope,&shared,Arc::new(FixtureOwner::new(&scope,ProjectRole::Contributor)));
        let mut bad=request(); bad.digest=[9;32];
        assert!(admitted.bind_surface_package(bad,before.clone()).is_err());
        assert!(original_epoch.is_current()); assert!(shared.snapshot().is_ok());
        let (entered_tx,entered_rx)=mpsc::channel(); let (resume_tx,resume_rx)=mpsc::channel();
        let owner=Arc::new(PauseReject { inner:FixtureOwner::new(&scope,ProjectRole::Contributor), entered:entered_tx, resume:Mutex::new(resume_rx) });
        let mut writer=wrap(&scope,&shared,owner); let source=before.clone(); let digest=saved.receipt.digest;
        let worker=std::thread::spawn(move || writer.bind_surface_package(SurfacePackageRequest { filename:"fixture.hmc".into(),digest },source));
        entered_rx.recv_timeout(Duration::from_secs(15)).expect("real checkpoint binding must reach barrier");
        assert!(!original_epoch.is_current()); assert!(shared.snapshot().err().unwrap().contains("busy"));
        resume_tx.send(()).unwrap(); assert!(worker.join().unwrap().unwrap_err().contains("quarantined"));
        assert!(shared.snapshot().err().unwrap().contains("quarantined")); assert_eq!(reference(&scope),before);
    }
    #[test]
    fn surface_binding_transaction_rejects_nonshared_host_before_dispatch() {
        let scope=scope(); let before=reference(&scope); let changed=Arc::new(AtomicBool::new(false)); let calls=Arc::new(AtomicUsize::new(0));
        let inner=BindingHost { changed:changed.clone(), calls:calls.clone(), fail_after_change:false };
        let mut editor=AuthorizedEditorHost::new(Box::new(inner),scope.clone(),Arc::new(FixtureOwner::new(&scope,ProjectRole::Contributor)),GatewayLimits::default()).unwrap();
        assert!(editor.bind_surface_package(request(),before.clone()).unwrap_err().contains("shared transaction"));
        assert_eq!(calls.load(Ordering::Acquire),0); assert!(!changed.load(Ordering::Acquire)); assert_eq!(reference(&scope),before);
    }
}
