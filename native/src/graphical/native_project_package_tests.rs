//! Software actors and ephemeral grids; no human, GUI or production-asset claim.
use super::*;
use crate::graphical::checkpoints::{
    ui_georaster_edit_limits, GeoRasterCreateRequest, GeoRasterEditRequest,
    ResourceInspectionRequest, SourceDeclaration, SourceStatus,
};
use crate::graphical::{
    package_workflow::{PackageSelection, PackageWorkflowAction},
    session_host::SessionEditorHost,
};
use qualia_core_db::{
    poet_host::native_authority::package_review::PackageReviewGrant,
    q42::m64_matter::{M64GeoRasterMetadata, M64RasterEdges, M64RasterRows},
};
use std::sync::{atomic::AtomicBool, Mutex};

#[test]
fn package_workflow_real_producer_separate_reviewer_reopen_and_same_source_successor() {
    let temp = tempfile::tempdir().unwrap();
    let authority_dir = temp.path().join("authority");
    let hmc = temp.path().join("hmc");
    let library = temp.path().join("library");
    for p in [&authority_dir, &hmc, &library] {
        std::fs::create_dir(p).unwrap();
    }
    let (scope, source, selection, saved, author) = {
        let (scope, mut admission, authority) = fixture(&authority_dir);
        let author = login(&authority, "fixture-author", &AUTHOR);
        let project = Arc::new(
            NativeProject::open(
                config(scope.clone(), &hmc, &library),
                authority.clone(),
                author.clone(),
                None,
                Box::new(NoPixels),
            )
            .unwrap(),
        );
        let sessions = Arc::new(Mutex::new(Some(author.clone())));
        let mut operator = SessionEditorHost::new(project.clone(), sessions.clone());
        let initial = operator.snapshot().unwrap();
        let source = operator
            .submit(&materials::diagnostic_source(), initial.source)
            .unwrap()
            .snapshot
            .source;
        let saved = operator
            .save_checkpoint("review.hmc", ExpectedFile::Absent, source.clone())
            .unwrap();
        let saved = operator
            .edit_package_resources(
                "review.hmc",
                ExpectedFile::Sha256(saved.digest),
                vec![ResourceEdit::Add(ResourceData {
                    key: "unknown/retained".into(),
                    kind: "future/raw".into(),
                    bytes: vec![5, 6, 7],
                    meta: Some(vec![0xa1, 0x61, b'x', 1]),
                })],
                source.clone(),
            )
            .unwrap();
        let metadata = M64GeoRasterMetadata {
            width: 2,
            height: 1,
            origin_lat_deg: -35.78123456789,
            origin_lon_deg: 148.29123456789,
            origin_elevation_mm: 350_123,
            cell_size_mm: 250,
            epoch_year: 2020.0,
            srid: 7844,
            vertical_datum: 5711,
            rows: M64RasterRows::South,
            edges: M64RasterEdges::RepeatNearest,
            nodata_sentinel_mm: None,
        };
        let saved = operator
            .create_georaster(
                GeoRasterCreateRequest {
                    filename: "review.hmc".into(),
                    package_digest: saved.digest,
                    key: "data/synthetic.m64".into(),
                    metadata,
                    lut_bank_epoch: 3,
                    samples_mm: vec![500_123, 500_123],
                    source: SourceDeclaration {
                        status: SourceStatus::Synthetic,
                        description: "Ephemeral software test grid".into(),
                        evidence_ref: Some("urn:fixture:synthetic-grid".into()),
                        observation_claim: false,
                    },
                },
                source.clone(),
                ui_georaster_edit_limits(),
                &AtomicBool::new(false),
            )
            .unwrap();
        let selection = PackageSelection {
            filename: "review.hmc".into(),
            package_id: format!("urn:fixture:package:{}", scope.project_id),
            digest: saved.digest,
        };
        let service = project.package_workflow_service(sessions.clone());
        let view = service
            .run(
                PackageWorkflowAction::Inspect(selection.clone()),
                source.clone(),
            )
            .unwrap();
        assert!(view.evidence.can_contribute);
        assert!(!view.evidence.can_review);
        assert_eq!(view.evidence.target.producer_subject, "fixture-author");
        let target = view.evidence.target.clone();
        let view = service
            .run(
                PackageWorkflowAction::Contribution {
                    target: target.clone(),
                    generation: view.evidence.authority_generation,
                    id: "package-contribution".into(),
                    description: "Explicit synthetic grid plus retained opaque resource".into(),
                    minutes: 1,
                    status: WorkStatus::Unpaid {
                        evidence_ref: "urn:fixture:unpaid".into(),
                    },
                },
                source.clone(),
            )
            .unwrap();
        let view = service
            .run(
                PackageWorkflowAction::Request {
                    target: target.clone(),
                    generation: view.evidence.authority_generation,
                    contribution_id: "package-contribution".into(),
                    id: "package-request".into(),
                    evidence: "urn:fixture:request".into(),
                },
                source.clone(),
            )
            .unwrap();
        assert_eq!(view.evidence.requests.len(), 1);
        let reviewer = login(&authority, "fixture-reviewer", &REVIEWER);
        *sessions.lock().unwrap() = Some(reviewer.clone());
        let view = service
            .run(
                PackageWorkflowAction::Inspect(selection.clone()),
                source.clone(),
            )
            .unwrap();
        assert!(
            !view.evidence.can_review,
            "existing source-review grant must not imply package review"
        );
        assert!(service
            .run(
                PackageWorkflowAction::Decide {
                    target: target.clone(),
                    generation: view.evidence.authority_generation,
                    request_id: "package-request".into(),
                    id: "denied-no-grant".into(),
                    decision: ReviewDecision::Accepted,
                    rationale: "Must not be recorded".into(),
                    evidence: "urn:fixture:denied".into()
                },
                source.clone()
            )
            .is_err());

        // Explicit software-fixture signed successor; native steward/signer admin
        // has separate public Core tests. No user key or human grant is fabricated.
        admission.sequence += 1;
        admission.package_reviews.push(PackageReviewGrant {
            id: "exact-package-grant".into(),
            target: target.clone(),
            reviewer_subject: "fixture-reviewer".into(),
            admission_ref: "urn:fixture:explicit-package-grant".into(),
            valid_from_unix: 1,
            valid_until_unix: admission.valid_until_unix,
            revoked: false,
        });
        authority.replace_admission(signed(admission)).unwrap();
        assert!(!view.publication_is_current());
        let reviewer = login(&authority, "fixture-reviewer", &REVIEWER);
        *sessions.lock().unwrap() = Some(reviewer);
        let view = service
            .run(
                PackageWorkflowAction::Inspect(selection.clone()),
                source.clone(),
            )
            .unwrap();
        assert!(view.evidence.can_review);
        let reviewed =
            service
                .run(
                    PackageWorkflowAction::Decide {
                        target: target.clone(),
                        generation: view.evidence.authority_generation,
                        request_id: "package-request".into(),
                        id: "package-decision".into(),
                        decision: ReviewDecision::Accepted,
                        rationale:
                            "Software fixture verified exact bytes; no human or model acceptance"
                                .into(),
                        evidence: "urn:fixture:decision".into(),
                    },
                    source.clone(),
                )
                .unwrap();
        assert_eq!(reviewed.evidence.reviews.len(), 1);
        assert!(reviewed.publication_is_current());
        assert_eq!(
            reviewed.evidence.reviews[0].actor.principal_kind,
            PrincipalKind::SoftwareAgent
        );
        drop(reviewed);
        drop(view);
        drop(service);
        drop(operator);
        drop(project);
        drop(authority);
        (scope, source, selection, saved, author)
    };

    let reopened = Arc::new(
        NativeAuthority::open_retained_current(
            &authority_dir,
            TrustedProject {
                project_id: scope.project_id.clone(),
                issuer_identifier: "fixture-issuer".into(),
                public_key: ed25519::public_from_seed(&AUTHORITY),
            },
            1024 * 1024,
            256,
        )
        .unwrap(),
    );
    assert!(
        reopened.workflow_evidence(&author).is_err(),
        "saved evidence cannot restore an old session"
    );
    let reviewer = login(&reopened, "fixture-reviewer", &REVIEWER);
    let project = Arc::new(
        NativeProject::open(
            config(scope.clone(), &hmc, &library),
            reopened.clone(),
            reviewer.clone(),
            None,
            Box::new(NoPixels),
        )
        .unwrap(),
    );
    let sessions = Arc::new(Mutex::new(Some(reviewer.clone())));
    let service = project.package_workflow_service(sessions);
    let view = service
        .run(
            PackageWorkflowAction::Inspect(selection.clone()),
            source.clone(),
        )
        .unwrap();
    assert_eq!(view.evidence.reviews[0].id, "package-decision");
    assert!(view.publication_is_current());
    let carrier = checkpoint::load(&hmc.join("review.hmc"), CheckpointLimits::default()).unwrap();
    let raster = carrier
        .resources()
        .iter()
        .find(|r| r.key == "data/synthetic.m64")
        .unwrap();
    let raster_digest = raster.summary().digest;
    assert!(raster.meta.is_some());
    let unknown = carrier
        .resources()
        .iter()
        .find(|r| r.key == "unknown/retained")
        .unwrap();
    assert_eq!(unknown.bytes, vec![5, 6, 7]);
    assert_eq!(unknown.meta, Some(vec![0xa1, 0x61, b'x', 1]));
    drop(carrier);
    // A separate authorized frontend models the same shared-owner route used
    // by MCP. Same source/session, new package bytes: old acceptance must hide.
    let mut other_frontend = project.connect(reviewer.clone()).unwrap();
    let successor = other_frontend
        .edit_georaster(
            GeoRasterEditRequest {
                resource: ResourceInspectionRequest {
                    filename: "review.hmc".into(),
                    package_digest: saved.digest,
                    key: "data/synthetic.m64".into(),
                    resource_digest: raster_digest,
                },
                origin_elevation_mm: 350_123,
                cell_size_mm: 250,
                sample_x: 1,
                sample_y: 0,
                sample_mm: 500_456,
            },
            source.clone(),
            ui_georaster_edit_limits(),
            &AtomicBool::new(false),
        )
        .unwrap();
    assert_ne!(successor.digest, saved.digest);
    assert_eq!(other_frontend.snapshot().unwrap().source, source);
    assert!(
        !view.publication_is_current(),
        "same-source resource save must retire accepted package display"
    );
    let changed = PackageSelection {
        digest: successor.digest,
        ..selection
    };
    let current = service
        .run(
            PackageWorkflowAction::Inspect(changed.clone()),
            source.clone(),
        )
        .unwrap();
    assert!(
        current.evidence.reviews.is_empty(),
        "old accepted decision cannot appear under successor bytes"
    );
    assert!(
        !current.evidence.can_review,
        "new package writer cannot review its own successor"
    );
    let mut corrupted = std::fs::read(hmc.join("review.hmc")).unwrap();
    corrupted.push(0x55);
    std::fs::write(hmc.join("review.hmc"), corrupted).unwrap();
    assert!(
        service
            .run(PackageWorkflowAction::Inspect(changed), source)
            .is_err(),
        "actual file changes must refuse on refresh/consume"
    );
}
