//! Software actors and ephemeral grids; no human, GUI or production-asset claim.
use super::*;
use crate::graphical::checkpoints::{
    typed_10d::{Typed10dEditRequest, Typed10dLimits},
    ResourceInspectionRequest,
};
use crate::graphical::{
    authoring::{AuthoringEdit, GeometryRecipe},
    jobs, native_dispatch,
};
use crate::graphical::{
    package_workflow::{PackageSelection, PackageWorkflowAction},
    session_host::SessionEditorHost,
};
use qualia_core_db::poet_host::native_authority::package_review::PackageReviewGrant;
use std::sync::{atomic::AtomicBool, Mutex};

struct AssemblyNoPixels;
impl PreviewProvider for AssemblyNoPixels {
    // This fixture does not retain a viewport or rendered frame.
    fn retire_retained_viewport(&mut self) -> Result<(), String> { Ok(()) }
    fn validate(&self, value: &Value) -> Result<(), String> {
        crate::graphical::assembly::admit_assembly(value, PreviewLimits::default()).map(|_| ())
    }
    fn preview(&mut self, _: SourceRef, _: &Value) -> Result<PreviewFrame, String> {
        Err("No GUI/GPU claimed by spatial workflow test".into())
    }
}
fn spatial_config(scope: Scope, root: &Path, library: &Path) -> ProjectConfig {
    let mut cfg = config(scope, root, library);
    cfg.geometry_grants.extend([
        "ComputationalGeometry.uv_sphere".into(),
        "ComputationalGeometry.torus".into(),
        "ComputationalGeometry.profile_sweep".into(),
    ]);
    cfg
}
fn block_on<F: std::future::Future>(future: F) -> F::Output {
    struct Wake(std::thread::Thread);
    impl std::task::Wake for Wake {
        fn wake(self: Arc<Self>) {
            self.0.unpark();
        }
        fn wake_by_ref(self: &Arc<Self>) {
            self.0.unpark();
        }
    }
    let waker = std::task::Waker::from(Arc::new(Wake(std::thread::current())));
    let mut cx = std::task::Context::from_waker(&waker);
    let mut future = std::pin::pin!(future);
    loop {
        match future.as_mut().poll(&mut cx) {
            std::task::Poll::Ready(v) => return v,
            std::task::Poll::Pending => std::thread::park(),
        }
    }
}
fn assembly_source() -> &'static str {
    r#"
let m={id:1,colour_space:"linear",base_color:[0.3,0.4,0.5,1.0],emissive:[0.0,0.0,0.0],metallic:0.1,roughness:0.5,specular:1.0,alpha_cutoff:0.5,opacity_mode:"Opaque",double_sided:true};
let tri={vertices:[[0.0,0.0,0.0],[1.0,0.0,0.0],[0.0,1.0,0.0]],triangles:[[0,1,2]]};
let mut poet_parts=[{id:"root",semantic_id:101,geometry:tri,material:m,translation:[0.0,0.0,0.0],rotation:[0.0,0.0,0.0],scale:[1.0,1.0,1.0]}];
({parts:poet_parts});
"#
}
#[test]
fn spatial_authoring_native_intents_compile_typed_inspect_and_separate_package_review_reopen() {
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
                spatial_config(scope.clone(), &hmc, &library),
                authority.clone(),
                author.clone(),
                None,
                Box::new(AssemblyNoPixels),
            )
            .unwrap(),
        );
        let sessions = Arc::new(Mutex::new(Some(author.clone())));
        let mut operator = SessionEditorHost::new(project.clone(), sessions.clone());
        let initial = operator.snapshot().unwrap();
        let source = operator
            .submit(assembly_source(), initial.source)
            .unwrap()
            .snapshot
            .source;
        let mut current = operator.snapshot().unwrap();
        let (worker, handle) = jobs::Worker::start(Box::new(operator)).unwrap();
        for edit in [
            AuthoringEdit::AddGeometry {
                id: "ring".into(),
                semantic_id: 102,
                material_from: "root".into(),
                geometry: GeometryRecipe::Sphere {
                    radius: 0.2,
                    lat_segments: 6,
                    lon_segments: 8,
                },
            },
            AuthoringEdit::ReplaceGeometry {
                id: "ring".into(),
                geometry: GeometryRecipe::Torus {
                    major_radius: 0.3,
                    minor_radius: 0.05,
                    major_segments: 12,
                    minor_segments: 6,
                },
            },
            AuthoringEdit::Reparent {
                id: "ring".into(),
                parent: Some("root".into()),
            },
            AuthoringEdit::UniqueMaterial {
                id: "ring".into(),
                material_id: 9,
            },
            AuthoringEdit::Material {
                material_id: 9,
                base_color: [0.2, 0.6, 0.3, 1.0],
                roughness: 0.25,
                metallic: 0.8,
            },
        ] {
            let reply = block_on(native_dispatch::run(
                handle.clone(),
                jobs::Action::AuthoringEdit {
                    edit,
                    expected: current.source.clone(),
                },
                jobs::Context::from_snapshot(&current),
            ))
            .unwrap();
            assert!(
                reply.operation_succeeded,
                "native Poet intent failed: {}",
                reply.message
            );
            current = reply.snapshot.unwrap();
        }
        assert_ne!(current.source, source, "real source revisions required");
        let stale = block_on(native_dispatch::run(
            handle.clone(),
            jobs::Action::AuthoringEdit {
                edit: AuthoringEdit::Reparent {
                    id: "ring".into(),
                    parent: None,
                },
                expected: source.clone(),
            },
            jobs::Context::from_snapshot(&current),
        ));
        assert!(
            stale.is_err(),
            "native intent adapter must not retarget stale expected source to a fresh context"
        );
        worker.finish().unwrap();
        let mut operator = SessionEditorHost::new(project.clone(), sessions.clone());
        let source = current.source;
        let parts = operator.authoring_view(source.clone()).unwrap();
        let ring = parts.parts.iter().find(|p| p.id == "ring").unwrap();
        assert_eq!(ring.parent.as_deref(), Some("root"));
        assert_eq!(ring.material_id, 9);
        assert_eq!(ring.roughness, 0.25);
        let saved = operator
            .compile_checkpoint(
                "review.hmc",
                "spatial",
                ExpectedFile::Absent,
                source.clone(),
            )
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
        let inventory = operator
            .package_inventory("review.hmc", source.clone())
            .unwrap();
        let entry = inventory
            .resources
            .iter()
            .find(|r| r.kind == "10d")
            .unwrap();
        let request = ResourceInspectionRequest {
            filename: "review.hmc".into(),
            package_digest: saved.digest,
            key: entry.key.clone(),
            resource_digest: entry.digest,
        };
        let typed = operator
            .inspect_typed_10d(request.clone(), source.clone(), Typed10dLimits::default())
            .unwrap();
        assert_eq!(
            typed
                .materials
                .iter()
                .find(|m| m.id == 9)
                .unwrap()
                .roughness,
            0.25
        );
        assert!(typed.frames.is_some());
        assert!(
            typed.edit_blocker.is_some(),
            "compiled Q42 pair must require source-route editing"
        );
        let before = std::fs::read(hmc.join("review.hmc")).unwrap();
        assert!(operator
            .edit_typed_10d(
                Typed10dEditRequest {
                    resource: request,
                    source_digest: typed.source_digest,
                    material_id: 9,
                    base_color: [0.1, 0.2, 0.3, 1.0],
                    emissive: [0.0; 3],
                    metallic: 0.0,
                    roughness: 0.9
                },
                source.clone(),
                Typed10dLimits::default(),
                &AtomicBool::new(false)
            )
            .is_err());
        assert_eq!(
            before,
            std::fs::read(hmc.join("review.hmc")).unwrap(),
            "refused companion edit must be atomic"
        );
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
                    description:
                        "Native geometry/material/assembly source and retained opaque resource"
                            .into(),
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
            spatial_config(scope.clone(), &hmc, &library),
            reopened.clone(),
            reviewer.clone(),
            None,
            Box::new(AssemblyNoPixels),
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
    let geometry = carrier
        .resources()
        .iter()
        .find(|r| r.kind == "10d")
        .unwrap();
    assert!(!geometry.bytes.is_empty());
    let graph = carrier
        .resources()
        .iter()
        .find(|r| r.kind == "q42")
        .unwrap();
    let quins = qualia_core_db::q42_volume::WasmSliceQ42Reader::open(&graph.bytes)
        .unwrap()
        .collect_all_quins()
        .unwrap();
    let cells = project
        .connect(reviewer.clone())
        .unwrap()
        .source_cells(source.clone())
        .unwrap();
    let raw = cells.join("\n\n").into_bytes();
    let source_crc = qualia_core_db::container_10d::crc32c(&raw);
    let compiled_crc = qualia_core_db::render::compile_10d::compiled_digest(&geometry.bytes);
    for (name, digest) in [
        ("urn:qualia:geometry:sourceDigest", source_crc),
        ("urn:qualia:geometry:compiledDigest", compiled_crc),
    ] {
        let predicate = qualia_core_db::q_hash(name);
        assert!(
            quins
                .iter()
                .any(|q| q.predicate == predicate && q.object == u64::from(digest)),
            "reopened Q42 must bind actual source/output CRC witness: {name}"
        );
    }
    let mut changed = raw.clone();
    changed[52] ^= 1;
    assert_ne!(
        source_crc,
        qualia_core_db::container_10d::crc32c(&changed),
        "raw source witness must include source bytes52..56"
    );

    assert!(carrier.resources().iter().any(|r| r.kind == "q42"));
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
        .edit_package_resources(
            "review.hmc",
            ExpectedFile::Sha256(saved.digest),
            vec![ResourceEdit::Add(ResourceData {
                key: "notes/changed".into(),
                kind: "test/opaque".into(),
                bytes: vec![1, 2],
                meta: None,
            })],
            source.clone(),
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
