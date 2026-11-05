//! Actual native authority/software-agent fixture; no OS credential or GPU access.
use super::{
    core::{ActionType, IntentBus, IntentStatus, Provenance, TargetIdentifier, VibeScriptPayload},
    *,
};
use crate::graphical::{
    checkpoints::{CheckpointLimits, PackageWritePolicy},
    controller::{EditorHost, PreviewProvider},
    cooperation::{source_digest, GatewayLimits, Scope},
    jobs,
    native_project::{NativeProject, ProjectConfig},
    preview::{PreviewFrame, SourceRef},
    session_host::SessionEditorHost,
};
use qualia_cooperative_core::{
    authoring::{DocumentRevision, IdentifierAssociationEvidence, PrincipalKind},
    projects::{Project, ProjectMembership, ProjectRole},
};
use qualia_core_db::{
    crypto::network::ed25519,
    poet_host::{
        invoke::studio_store::{self, StudioArtifact},
        native_authority::*,
    },
};
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc, Mutex,
};
use vibe::Value;
static NEXT: AtomicU64 = AtomicU64::new(1);
fn block_on<F: std::future::Future>(future: F) -> F::Output {
    struct WakeThread(std::thread::Thread);
    impl std::task::Wake for WakeThread {
        fn wake(self: Arc<Self>) {
            self.0.unpark();
        }
        fn wake_by_ref(self: &Arc<Self>) {
            self.0.unpark();
        }
    }
    let waker = std::task::Waker::from(Arc::new(WakeThread(std::thread::current())));
    let mut context = std::task::Context::from_waker(&waker);
    let mut future = std::pin::pin!(future);
    loop {
        match future.as_mut().poll(&mut context) {
            std::task::Poll::Ready(value) => return value,
            std::task::Poll::Pending => std::thread::park(),
        }
    }
}
struct NoPixels;
impl PreviewProvider for NoPixels {
    // This headless fixture does not retain a viewport or frame.
    fn retire_retained_viewport(&mut self) -> Result<(), String> { Ok(()) }
    fn preview(&mut self, _: SourceRef, _: &Value) -> Result<PreviewFrame, String> {
        Err("No GUI/GPU claimed by native intent tests".into())
    }
}
struct Fixture {
    worker: Option<jobs::Worker>,
    bus: NativeIntentBus,
    owner: Arc<NativeAuthority>,
    session: SessionHandle,
    source: String,
    _temp: tempfile::TempDir,
}
impl Drop for Fixture {
    fn drop(&mut self) {
        if let Some(worker) = self.worker.take() {
            worker.finish().unwrap();
        }
        let _ = self.owner.disconnect(&self.session);
    }
}
fn fixture() -> Fixture { fixture_with_grid(false) }
fn fixture_with_grid(with_grid: bool) -> Fixture { fixture_with_role(with_grid, ProjectRole::Contributor) }
fn fixture_with_role(with_grid: bool, role: ProjectRole) -> Fixture {
    let temp = tempfile::tempdir().unwrap();
    let journal = temp.path().join("journal");
    let packages = temp.path().join("packages");
    std::fs::create_dir(&journal).unwrap();
    std::fs::create_dir(&packages).unwrap();
    let suffix = format!(
        "{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    );
    let scope = Scope {
        project_id: format!("intent-project-{suffix}"),
        document_id: format!("intent-document-{suffix}"),
        source_id: format!("intent-source-{suffix}"),
        domain: "Geometry".into(),
    };
    let artifact = StudioArtifact::new(&scope.source_id, "Geometry", "Native Poet intent fixture");
    let document = DocumentRevision {
        project_id: scope.project_id.clone(),
        document_id: scope.document_id.clone(),
        source_id: scope.source_id.clone(),
        revision: artifact.lamport_clock.into(),
        source_digest: source_digest(&artifact, GatewayLimits::default()).unwrap(),
        author_subject: None,
    };
    studio_store::create_artifact_if_absent(artifact).unwrap();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let until = now + 3600;
    let snapshot = AdmissionSnapshot {
        package_reviews: Vec::new(),
        version: 1,
        sequence: 1,
        issuer_identifier: "fixture-issuer".into(),
        project: Project {
            id: scope.project_id.clone(),
            name: "Software-agent intent fixture".into(),
            description: String::new(),
            licensing_ontologies: Vec::new(),
            created_at_unix: 1,
            created_at_instant: None,
        },
        valid_from_unix: now - 1,
        valid_until_unix: until,
        revoked: false,
        subjects: vec![SubjectAdmission {
            subject: "fixture-software-agent".into(),
            kind: PrincipalKind::SoftwareAgent,
            controller_public_key: ed25519::public_from_seed(&[53; 32]),
            membership: ProjectMembership {
                id: "fixture-membership".into(),
                project_id: scope.project_id.clone(),
                member_did: "urn:fixture:agent".into(),
                role,
                agreed_at_unix: 1,
                agreed_at_instant: None,
            },
            identifiers: vec![IdentifierAssociationEvidence {
                value: "urn:fixture:agent".into(),
                scheme: "urn:fixture".into(),
                issuer: "fixture-issuer".into(),
                project_id: scope.project_id.clone(),
                evidence_ref: "fixture-association".into(),
                valid_from_unix: now - 1,
                valid_until_unix: until,
            }],
            reviews: Vec::new(),
            admission_ref: "fixture-admission".into(),
            valid_from_unix: now - 1,
            valid_until_unix: until,
            revoked: false,
        }],
    };
    let mut proof = [0; 64];
    ed25519::sign_with_context(
        &[51; 32],
        &admission_digest(&snapshot, 1024 * 1024).unwrap(),
        ADMISSION_CONTEXT,
        &mut proof,
    )
    .unwrap();
    let owner = Arc::new(
        NativeAuthority::open(
            &journal,
            TrustedProject {
                project_id: scope.project_id.clone(),
                issuer_identifier: "fixture-issuer".into(),
                public_key: ed25519::public_from_seed(&[51; 32]),
            },
            SignedAdmission {
                snapshot,
                signature: proof.to_vec(),
            },
            document,
            1024 * 1024,
            256,
        )
        .unwrap(),
    );
    let challenge = owner.begin_session("fixture-software-agent", 300).unwrap();
    ed25519::sign_with_context(
        &[53; 32],
        challenge.message_digest(),
        SESSION_CONTEXT,
        &mut proof,
    )
    .unwrap();
    let session = owner.complete_session(challenge, &proof).unwrap();
    let project = Arc::new(
        NativeProject::open(
            ProjectConfig {
                scope: scope.clone(),
                checkpoint_root: packages,
                checkpoint_limits: CheckpointLimits::default(),
                write_policy: PackageWritePolicy::WorkingCopy,
                import_library: None,
                session_limits: Default::default(),
                gateway_limits: GatewayLimits::default(),
                geometry_grants: if with_grid { vec!["capability.invoke".into(), "Sheet.set_cell".into()] } else { Vec::new() },
                max_vertices: 1000,
            },
            owner.clone(),
            session.clone(),
            None,
            Box::new(NoPixels),
        )
        .unwrap(),
    );
    let mut host = SessionEditorHost::new(project, Arc::new(Mutex::new(Some(session.clone()))));
    if with_grid {
        let expected = host.snapshot().unwrap().source;
        host.submit("let mut poet_sheet=[[1,2],[3,4]]; poet_sheet;", expected).unwrap();
    }
    let origin = jobs::Context::from_snapshot(&host.snapshot().unwrap());
    let (worker, handle) = jobs::Worker::start(Box::new(host)).unwrap();
    let bus = NativeIntentBus::new(handle, origin).unwrap();
    Fixture {
        worker: Some(worker),
        bus,
        owner,
        session,
        source: scope.source_id,
        _temp: temp,
    }
}
fn payload(
    source: &str,
    action: ActionType,
    parameters: serde_json::Value,
) -> VibeScriptPayload<serde_json::Value> {
    VibeScriptPayload::new(action, TargetIdentifier::component_id(source), parameters)
}
#[test]
fn native_projection_reuses_original_studio_containers_and_holds_unqualified_services() {
    let projection = WorkspaceProjection::for_source("source/fixture").unwrap();
    assert_eq!(projection.construct.honesty, "partial");
    assert_eq!(projection.construct.id, "poet");
    assert!(projection.construct.contains_manifold("studio-bay"));
    assert!(projection.construct.contains_manifold("research"));
    assert!(projection.construct.contains_manifold("rights"));
    assert_eq!(projection.manifold.id, "studio");
    for kind in ["dual_studio", "scene_view", "audio_session", "inspector"] {
        assert!(projection
            .manifold
            .containers
            .iter()
            .any(|c| c.container_type == kind));
    }
    assert!(projection
        .manifold
        .containers
        .iter()
        .all(|c| c.honesty != "live"));
}
#[test]
fn same_native_actor_provenance_is_real_and_backend_failure_stays_rejected() {
    let f = fixture();
    let request = payload(
        &f.source,
        ActionType::Query,
        serde_json::json!({"command":"source.check","source":"1 + 2;"}),
    );
    let result = block_on(f.bus.dispatch_with_reply(request)).unwrap();
    assert_eq!(result.receipt.status, IntentStatus::Accepted);
    assert!(result.reply.unwrap().operation_succeeded);
    assert_eq!(
        result.receipt.provenance.unwrap().emitter_did,
        "fixture-software-agent"
    );
    let invalid = payload(
        &f.source,
        ActionType::Query,
        serde_json::json!({"command":"source.check","source":"let ="}),
    );
    let rejected = block_on(f.bus.dispatch_with_reply(invalid)).unwrap();
    assert!(matches!(rejected.receipt.status, IntentStatus::Rejected(_)));
    assert!(!rejected.reply.unwrap().operation_succeeded);
}
#[test]
fn foreign_target_context_client_provenance_unknown_and_oversize_are_rejected() {
    let f = fixture();
    let base = || {
        payload(
            &f.source,
            ActionType::Query,
            serde_json::json!({"command":"source.check","source":"1;"}),
        )
    };
    let mut foreign = base();
    foreign.target_identifier = TargetIdentifier::component_id("another-source");
    let mut context = base();
    context.context = "https://untrusted.invalid/context".into();
    let mut forged = base();
    forged.provenance = Some(Provenance {
        emitter_did: "untrusted".into(),
        component_label: "fake".into(),
        intent_counter: 1,
        capability_scope: None,
    });
    let unknown = payload(
        &f.source,
        ActionType::Invoke,
        serde_json::json!({"command":"authority.self_admit"}),
    );
    let huge = payload(
        &f.source,
        ActionType::Query,
        serde_json::json!({"command":"source.check","source":"x".repeat(100*1024)}),
    );
    for request in [foreign, context, forged, unknown, huge] {
        let result = block_on(f.bus.dispatch_with_reply(request)).unwrap();
        assert!(matches!(result.receipt.status, IntentStatus::Rejected(_)));
        assert!(result.reply.is_none());
    }
}
#[test]
fn revocation_refuses_dispatch_and_cancel_never_claims_cancelled() {
    let f = fixture();
    f.owner.disconnect(&f.session).unwrap();
    let request = payload(
        &f.source,
        ActionType::Query,
        serde_json::json!({"command":"source.check","source":"1;"}),
    );
    let result = block_on(f.bus.dispatch_with_reply(request)).unwrap();
    assert!(matches!(result.receipt.status, IntentStatus::Rejected(_)));
    assert!(result.reply.is_none());
    let cancelled = block_on(f.bus.cancel(42)).unwrap();
    assert!(matches!(cancelled.status, IntentStatus::Rejected(_)));
}
#[test]
fn read_only_refresh_recovers_same_actor_source_successor_without_rebinding_actor() {
    let f = fixture();
    let submit = payload(
        &f.source,
        ActionType::Invoke,
        serde_json::json!({"command":"source.submit","source":"let mut fixture_value = 1;"}),
    );
    let result = block_on(f.bus.dispatch_with_reply(submit)).unwrap();
    assert_eq!(result.receipt.status, IntentStatus::Accepted);
    let stale = payload(
        &f.source,
        ActionType::Query,
        serde_json::json!({"command":"source.check","source":"1;"}),
    );
    assert!(matches!(
        block_on(f.bus.dispatch_with_reply(stale))
            .unwrap()
            .receipt
            .status,
        IntentStatus::Rejected(_)
    ));
    let refresh = payload(
        &f.source,
        ActionType::Query,
        serde_json::json!({"command":"source.refresh"}),
    );
    let result = block_on(f.bus.dispatch_with_reply(refresh)).unwrap();
    assert_eq!(result.receipt.status, IntentStatus::Accepted);
    assert_eq!(
        result.receipt.provenance.unwrap().emitter_did,
        "fixture-software-agent"
    );
}

#[test]
fn typed_numeric_sheet_intent_uses_actor_receipt_and_refuses_stale_or_revoked_session() {
    let f = fixture_with_grid(true);
    let inspect = block_on(f.bus.dispatch_numeric_sheet(NumericSheetIntent::Inspect)).unwrap();
    assert_eq!(inspect.receipt.status, IntentStatus::Accepted);
    assert_eq!(inspect.receipt.provenance.unwrap().emitter_did, "fixture-software-agent");
    let view = inspect.reply.unwrap().numeric_sheet.unwrap();
    let edit = crate::graphical::numeric_sheet::SheetEdit { row: 0, col: 1, value: 7.0, execution_epoch: view.execution_epoch };
    let edited = block_on(f.bus.dispatch_numeric_sheet(NumericSheetIntent::Edit(edit))).unwrap();
    assert_eq!(edited.receipt.status, IntentStatus::Accepted);
    assert_eq!(edited.receipt.provenance.unwrap().emitter_did, "fixture-software-agent");
    assert!(edited.reply.unwrap().operation_succeeded);
    let stale = block_on(f.bus.dispatch_numeric_sheet(NumericSheetIntent::Inspect)).unwrap();
    assert!(matches!(stale.receipt.status, IntentStatus::Rejected(_)));
    let f = fixture_with_grid(true);
    f.owner.disconnect(&f.session).unwrap();
    let revoked = block_on(f.bus.dispatch_numeric_sheet(NumericSheetIntent::Inspect)).unwrap();
    assert!(matches!(revoked.receipt.status, IntentStatus::Rejected(_)));
    assert!(revoked.reply.is_none());
}

#[test]
fn typed_numeric_sheet_observer_read_is_admitted_but_execute_is_denied() {
    // No binding is installed by this observer. A missing-binding diagnostic
    // demonstrates the read reached the inner owner after ReadSource admission.
    let f = fixture_with_role(false, ProjectRole::Observer);
    let inspect = block_on(f.bus.dispatch_numeric_sheet(NumericSheetIntent::Inspect)).unwrap();
    assert!(matches!(inspect.receipt.status, IntentStatus::Rejected(_)));
    let reply = inspect.reply.expect("read reached inner owner");
    assert!(reply.message.contains("no poet_sheet binding"));
    let epoch = reply.snapshot.unwrap().execution_epoch;
    let edit = crate::graphical::numeric_sheet::SheetEdit { row: 0, col: 0, value: 7.0, execution_epoch: epoch };
    let denied = block_on(f.bus.dispatch_numeric_sheet(NumericSheetIntent::Edit(edit))).unwrap();
    assert!(matches!(denied.receipt.status, IntentStatus::Rejected(_)));
    assert!(denied.reply.unwrap().message.contains("RoleDenied"));
}

#[test]
fn terrain_vibe_payload_uses_existing_actor_and_cannot_bypass_observer_write_authority() {
    let f=fixture_with_role(false,ProjectRole::Observer);
    let parameters=serde_json::json!({"command":"authoring.terrain_brush","draft":{
        "resource":{"filename":"terrain.hmc","package_digest":vec![1u8;32],"key":"height.m64","resource_digest":vec![2u8;32]},
        "window_x":0,"window_y":0,"window_side":1,
        "strokes":[{"operation":"raise","falloff":"linear","center_m":[0.0,0.0],"radius_m":1.0,"strength":1.0}]
    }});
    let denied=block_on(f.bus.dispatch_with_reply(payload(&f.source,ActionType::Invoke,parameters))).unwrap();
    assert!(matches!(denied.receipt.status,IntentStatus::Rejected(_)));
    assert_eq!(denied.receipt.provenance.unwrap().capability_scope.as_deref(),Some("Studio.checkpoint"));
    assert!(denied.reply.unwrap().message.contains("RoleDenied"));
    // A revoked native session is rejected before any domain operation.
    f.owner.disconnect(&f.session).unwrap();
    let revoked=block_on(f.bus.dispatch_with_reply(payload(&f.source,ActionType::Invoke,serde_json::json!({"command":"authoring.terrain_brush","draft":{
        "resource":{"filename":"terrain.hmc","package_digest":vec![1u8;32],"key":"height.m64","resource_digest":vec![2u8;32]},
        "window_x":0,"window_y":0,"window_side":1,
        "strokes":[{"operation":"raise","falloff":"linear","center_m":[0.0,0.0],"radius_m":1.0,"strength":1.0}]
    }})))).unwrap();
    assert!(matches!(revoked.receipt.status,IntentStatus::Rejected(_)));
    assert!(revoked.reply.is_none());
}
