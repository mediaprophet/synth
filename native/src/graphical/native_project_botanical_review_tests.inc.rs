// Candidate to include in native_project_biosphere_round1_tests.rs. Synthetic
// graph and signed software actors; source review is not HMC/package acceptance.
#[test]
fn signed_botanical_checkpoint_exact_source_review_retains_journal() {
    use crate::graphical::cooperative_workflow::WorkflowAction;
    let temp = tempfile::tempdir().unwrap();
    let authority_dir = temp.path().join("authority");
    let hmc_dir = temp.path().join("hmc");
    let library = temp.path().join("library");
    for path in [&authority_dir, &hmc_dir, &library] { std::fs::create_dir(path).unwrap(); }
    let (scope, target, package_digest) = {
        let (scope, _, authority) = fixture(&authority_dir);
        let author = login(&authority, "fixture-author", &AUTHOR);
        let project = Arc::new(NativeProject::open(
            botanical_config(scope.clone(), &hmc_dir, &library), authority.clone(), author.clone(),
            None, Box::new(BotanicalNoPixels)).unwrap());
        let sessions = Arc::new(Mutex::new(Some(author.clone())));
        let service = project.workflow_service(sessions.clone());
        let mut host = SessionEditorHost::new(project.clone(), sessions.clone());
        let initial = host.snapshot().unwrap();
        host.submit(botanical_source(), initial.source).unwrap();
        let current = host.snapshot().unwrap().source;
        host.authoring_edit(AuthoringEdit::Botanical { edit: BotanicalEdit::SetRecipe {
            recipe: botanical_recipe() } }, current).unwrap();
        let current = host.snapshot().unwrap().source;
        host.authoring_edit(AuthoringEdit::Botanical { edit: BotanicalEdit::SetTipRadius {
            part_id: "branch".into(), tip_radius_m: 0.04 } }, current).unwrap();
        assert_botanical(&host, 0.04, 0.5);
        let current = host.snapshot().unwrap().source;
        let cells = host.source_cells(current.clone()).unwrap();
        let saved = host.save_checkpoint("botanical-review.hmc", ExpectedFile::Absent, current).unwrap();
        let bytes = std::fs::read(&saved.path).unwrap();
        let actual_digest: [u8; 32] = Sha256::digest(&bytes).into();
        assert_eq!(actual_digest, saved.digest);
        let loaded = checkpoint::load(&saved.path, CheckpointLimits::default()).unwrap();
        assert_eq!(loaded.artifact(), &studio_store::get_artifact_stamped(&scope.source_id).unwrap().0);
        let inspected = service.run(WorkflowAction::Inspect).unwrap();
        assert_eq!(inspected.source_cells, cells);
        assert_eq!(inspected.evidence.principal_subject, "fixture-author");
        let target = inspected.evidence.target.clone();
        assert_eq!(target.project_id, scope.project_id);
        assert_eq!(target.document_id, scope.document_id);
        assert_eq!(target.source_id, scope.source_id);
        assert_eq!(target.author_subject.as_deref(), Some("fixture-author"));
        service.run(WorkflowAction::Contribution { target: target.clone(), id: "botanical/contribution".into(),
            description: "Synthetic botanical source edit and checkpoint".into(), minutes: 1,
            work_status: WorkStatus::Unpaid { evidence_ref: "urn:fixture:unpaid".into() } }).unwrap();
        let requested = service.run(WorkflowAction::RequestReview { target: target.clone(),
            id: "botanical/request".into(), evidence: "urn:fixture:request".into() }).unwrap();
        assert_eq!(requested.evidence.requests.last().unwrap().target, target);
        assert!(service.run(WorkflowAction::Review { target: target.clone(), request_id: "botanical/request".into(),
            id: "botanical/self".into(), decision: ReviewDecision::Accepted,
            rationale: "self review must fail".into(), evidence: "urn:fixture:self".into() }).is_err());
        let after_self = service.run(WorkflowAction::Inspect).unwrap();
        assert_eq!(after_self.evidence.target, target);
        assert_eq!(after_self.source_cells, cells);
        assert!(after_self.evidence.requests.last().unwrap().decision_ids.is_empty());
        assert!(!after_self.evidence.reviews.iter().any(|r| r.id == "botanical/self"));
        let reviewer = login(&authority, "fixture-reviewer", &REVIEWER);
        *sessions.lock().unwrap() = Some(reviewer);
        let inspected = service.run(WorkflowAction::Inspect).unwrap();
        assert_eq!(inspected.evidence.principal_subject, "fixture-reviewer");
        assert!(inspected.evidence.can_review);
        assert_eq!(inspected.evidence.target, target);
        assert_eq!(inspected.source_cells, cells);
        let mut stale = target.clone();
        stale.source_digest[0] ^= 1;
        assert!(service.run(WorkflowAction::Review { target: stale, request_id: "botanical/request".into(),
            id: "botanical/stale".into(), decision: ReviewDecision::Accepted,
            rationale: "stale review must fail".into(), evidence: "urn:fixture:stale".into() }).is_err());
        let after_stale = service.run(WorkflowAction::Inspect).unwrap();
        assert_eq!(after_stale.evidence.target, target);
        assert_eq!(after_stale.source_cells, cells);
        assert_eq!(after_stale.evidence.requests.last().unwrap().target, target);
        assert!(after_stale.evidence.requests.last().unwrap().decision_ids.is_empty());
        assert!(!after_stale.evidence.reviews.iter().any(|r| r.id == "botanical/stale"));
        let reviewed = service.run(WorkflowAction::Review { target: target.clone(),
            request_id: "botanical/request".into(), id: "botanical/decision".into(),
            decision: ReviewDecision::ChangesRequested, rationale: "Synthetic source inspection only".into(),
            evidence: "urn:fixture:decision".into() }).unwrap();
        assert_eq!(reviewed.evidence.target, target);
        assert_eq!(reviewed.evidence.requests.last().unwrap().target, target);
        assert_eq!(reviewed.evidence.requests.last().unwrap().decision_ids, vec!["botanical/decision".to_string()]);
        assert_eq!(reviewed.evidence.reviews.last().unwrap().target, target);
        assert_eq!(reviewed.evidence.reviews.last().unwrap().reviewer_subject, "fixture-reviewer");
        assert_eq!(reviewed.evidence.reviews.last().unwrap().principal_kind, PrincipalKind::SoftwareAgent);
        (scope, target, saved.digest)
    };
    // Reopen the durable authority journal after dropping the previous owner.
    // Studio state is still process-local; this does not claim a fresh process.
    let authority = NativeAuthority::open_retained_current(&authority_dir, TrustedProject {
        project_id: scope.project_id.clone(), issuer_identifier: "fixture-issuer".into(),
        public_key: ed25519::public_from_seed(&AUTHORITY),
    }, 1024 * 1024, 256).unwrap();
    let reviewer = login(&authority, "fixture-reviewer", &REVIEWER);
    let retained = authority.workflow_evidence(&reviewer).unwrap();
    assert_eq!(retained.target, target);
    assert_eq!(retained.requests.last().unwrap().target, target);
    assert_eq!(retained.requests.last().unwrap().decision_ids, vec!["botanical/decision".to_string()]);
    assert_eq!(retained.reviews.last().unwrap().target, target);
    assert_eq!(retained.reviews.last().unwrap().reviewer_subject, "fixture-reviewer");
    assert_eq!(retained.reviews.last().unwrap().principal_kind, PrincipalKind::SoftwareAgent);
    let package_write = retained.package_writes.last().unwrap();
    assert_eq!(package_write.result_digest, package_digest);
    assert_eq!(package_write.source, target);
    assert_eq!(package_write.filename, "botanical-review.hmc");
    assert_eq!(package_write.actor_subject, "fixture-author");
    let bytes = std::fs::read(hmc_dir.join("botanical-review.hmc")).unwrap();
    let actual_digest: [u8; 32] = Sha256::digest(&bytes).into();
    assert_eq!(actual_digest, package_digest);
    assert_eq!(package_write.result_bytes, bytes.len());
    let loaded = checkpoint::load(&hmc_dir.join("botanical-review.hmc"), CheckpointLimits::default()).unwrap();
    assert_eq!(loaded.artifact(), &studio_store::get_artifact_stamped(&scope.source_id).unwrap().0);
}
