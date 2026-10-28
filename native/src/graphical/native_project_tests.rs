//! Real native cryptography + Vibe + Studio + HMC + shared MCP. Fixture keys and
//! software-agent decisions never establish human enrollment or acceptance.
use super::super::{
    checkpoints::{CheckpointLimits, ExpectedFile, PackageWritePolicy, ResourceData, ResourceEdit},
    controller::{EditorHost, PreviewProvider},
    cooperation::{source_digest, GatewayLimits, Scope},
    materials::{self, MaterialDraft},
    package::{ImportRequest, ResourceLibrary},
    preview::{PreviewFrame, PreviewLimits, SourceRef},
};
use super::{NativeProject, ProjectConfig};
use qualia_cooperative_core::{
    authoring::*,
    projects::{Project, ProjectMembership, ProjectRole},
};
use qualia_core_db::{
    container_10d::MaterialRecord,
    crypto::network::ed25519,
    poet_host::{
        invoke::studio_store::{self, checkpoint, StudioArtifact},
        native_authority::*,
    },
};
use qualia_mcp::authoring::AuthoringBackend;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{
    path::Path,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    time::{SystemTime, UNIX_EPOCH},
};
use vibe::{session::SessionLimits, Value};

const AUTHORITY: [u8; 32] = [37; 32];
const AUTHOR: [u8; 32] = [41; 32];
const REVIEWER: [u8; 32] = [43; 32];
static NEXT: AtomicU64 = AtomicU64::new(1);
struct NoPixels;
impl PreviewProvider for NoPixels {
    // This fixture does not retain a viewport or rendered frame.
    fn retire_retained_viewport(&mut self) -> Result<(), String> { Ok(()) }
    fn validate(&self, value: &Value) -> Result<(), String> {
        let (mesh, _) = materials::authored_scene(
            value,
            MaterialRecord::legacy_default(),
            PreviewLimits::default(),
        )?;
        if mesh.positions.is_empty() || mesh.triangles.is_empty() {
            return Err("expected actual native geometry".into());
        }
        Ok(())
    }
    fn preview(&mut self, _: SourceRef, _: &Value) -> Result<PreviewFrame, String> {
        Err("headless native integration deliberately does not claim rendered pixels".into())
    }
}
fn signed(snapshot: AdmissionSnapshot) -> SignedAdmission {
    let digest = admission_digest(&snapshot, 1024 * 1024).unwrap();
    let mut signature = [0; 64];
    ed25519::sign_with_context(&AUTHORITY, &digest, ADMISSION_CONTEXT, &mut signature).unwrap();
    SignedAdmission {
        snapshot,
        signature: signature.to_vec(),
    }
}
fn login(authority: &NativeAuthority, subject: &str, key: &[u8; 32]) -> SessionHandle {
    let challenge = authority.begin_session(subject, 300).unwrap();
    let mut signature = [0; 64];
    ed25519::sign_with_context(
        key,
        challenge.message_digest(),
        SESSION_CONTEXT,
        &mut signature,
    )
    .unwrap();
    authority.complete_session(challenge, &signature).unwrap()
}
fn fixture(root: &Path) -> (Scope, AdmissionSnapshot, Arc<NativeAuthority>) {
    let serial = NEXT.fetch_add(1, Ordering::Relaxed);
    let suffix = format!("{}-{serial}", std::process::id());
    let scope = Scope {
        project_id: format!("urn:fixture:project:{suffix}"),
        document_id: format!("urn:fixture:document:{suffix}"),
        source_id: format!("urn:fixture:source:{suffix}"),
        domain: "Geometry".into(),
    };
    studio_store::create_artifact_if_absent(StudioArtifact::new(
        &scope.source_id,
        "Geometry",
        "signed native integration fixture",
    ))
    .unwrap();
    let until = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        + 3600;
    let subject = |name: &str, key: &[u8; 32]| SubjectAdmission {
        subject: format!("fixture-{name}"),
        kind: PrincipalKind::SoftwareAgent,
        controller_public_key: ed25519::public_from_seed(key),
        membership: ProjectMembership {
            id: format!("membership/{name}"),
            project_id: scope.project_id.clone(),
            member_did: format!("did:test:{name}"),
            role: ProjectRole::Contributor,
            agreed_at_unix: 1,
            agreed_at_instant: None,
        },
        identifiers: vec![IdentifierAssociationEvidence {
            value: format!("did:test:{name}"),
            scheme: "did:test".into(),
            issuer: "fixture-issuer".into(),
            project_id: scope.project_id.clone(),
            evidence_ref: format!("association/{name}"),
            valid_from_unix: 1,
            valid_until_unix: until,
        }],
        reviews: vec![ReviewAdmission {
            id: format!("review-grant/{name}"),
            document_id: scope.document_id.clone(),
            admission_ref: format!("review-adoption/{name}"),
            valid_from_unix: 1,
            valid_until_unix: until,
            revoked: false,
        }],
        admission_ref: format!("fixture-adoption/{name}"),
        valid_from_unix: 1,
        valid_until_unix: until,
        revoked: false,
    };
    let snapshot = AdmissionSnapshot {
        package_reviews: Vec::new(),
        version: 1,
        sequence: 1,
        issuer_identifier: "fixture-issuer".into(),
        project: Project {
            id: scope.project_id.clone(),
            name: "native integration fixture".into(),
            description: String::new(),
            licensing_ontologies: vec![],
            created_at_unix: 1,
            created_at_instant: None,
        },
        valid_from_unix: 1,
        valid_until_unix: until,
        revoked: false,
        subjects: vec![subject("author", &AUTHOR), subject("reviewer", &REVIEWER)],
    };
    let artifact = studio_store::get_artifact_stamped(&scope.source_id)
        .unwrap()
        .0;
    let document = DocumentRevision {
        project_id: scope.project_id.clone(),
        document_id: scope.document_id.clone(),
        source_id: scope.source_id.clone(),
        revision: artifact.lamport_clock.into(),
        source_digest: source_digest(&artifact, GatewayLimits::default()).unwrap(),
        author_subject: None,
    };
    let authority = NativeAuthority::open(
        root,
        TrustedProject {
            project_id: scope.project_id.clone(),
            issuer_identifier: "fixture-issuer".into(),
            public_key: ed25519::public_from_seed(&AUTHORITY),
        },
        signed(snapshot.clone()),
        document,
        1024 * 1024,
        256,
    )
    .unwrap();
    (scope, snapshot, Arc::new(authority))
}
fn config(scope: Scope, root: &Path, library: &Path) -> ProjectConfig {
    ProjectConfig {
        scope,
        checkpoint_root: root.into(),
        checkpoint_limits: CheckpointLimits::default(),
        write_policy: PackageWritePolicy::WorkingCopy,
        import_library: Some(ResourceLibrary::new(library, 1024 * 1024).unwrap()),
        session_limits: SessionLimits::default(),
        gateway_limits: GatewayLimits::default(),
        geometry_grants: vec![
            "capability.invoke".into(),
            "ComputationalGeometry.extrude_profile".into(),
        ],
        max_vertices: 200_000,
    }
}

#[test]
fn signed_native_project_shares_actual_edits_hmc_resources_and_revokes_ui_mcp() {
    let temp = tempfile::tempdir().unwrap();
    let authority_dir = temp.path().join("authority");
    let hmc_dir = temp.path().join("hmc");
    let library = temp.path().join("library");
    for root in [&authority_dir, &hmc_dir, &library] {
        std::fs::create_dir(root).unwrap()
    }
    std::fs::write(library.join("texture.bin"), b"new editable resource").unwrap();
    let (scope, mut admission, authority) = fixture(&authority_dir);
    let session = login(&authority, "fixture-author", &AUTHOR);
    let project = NativeProject::open(
        config(scope.clone(), &hmc_dir, &library),
        authority.clone(),
        session.clone(),
        None,
        Box::new(NoPixels),
    )
    .unwrap();
    let mut ui = project.connect(session.clone()).unwrap();
    let mut mcp = project.mcp_backend(session.clone()).unwrap();
    assert!(mcp
        .tools()
        .iter()
        .any(|t| t.name == "poet_hmc_import_resource"));
    let initial_source = materials::diagnostic_source();
    let before = ui.snapshot().unwrap();
    let outcome = ui.submit(&initial_source, before.source).unwrap();
    assert!(outcome.frame.is_none());
    assert!(outcome.snapshot.publication_is_current());
    let read = mcp.call("poet_source_read", json!({})).unwrap();
    assert_eq!(read["source_cells"], json!([initial_source]));
    let draft = MaterialDraft {
        roughness: 0.2,
        metallic: 0.4,
        ..Default::default()
    };
    let edit = draft
        .edit_source(2, outcome.snapshot.material_edit_id.unwrap())
        .unwrap();
    let edited = mcp
        .call(
            "poet_material_apply",
            json!({"context":read["context"],"source":edit}),
        )
        .unwrap();
    assert_eq!(edited["preview_rendered"], false);
    let current = ui.snapshot().unwrap();
    assert_eq!(current.session_revision, 2);
    assert_eq!(ui.source_cells(current.source.clone()).unwrap().len(), 2);
    let receipt = ui
        .save_checkpoint("working.hmc", ExpectedFile::Absent, current.source.clone())
        .unwrap();
    let master = ResourceData {
        key: "masters/editable.obj".into(),
        kind: "text/plain".into(),
        bytes: b"# retained exact editable master\nv 1 2 3\n".to_vec(),
        meta: Some(vec![0xa1, 0x61, b'x', 1]),
    };
    let opaque = ResourceData {
        key: "future/unassessed.bin".into(),
        kind: "future/schema".into(),
        bytes: vec![0, 42, 255],
        meta: Some(vec![0xa1, 0x61, b'z', 2]),
    };
    let receipt = ui
        .edit_package_resources(
            "working.hmc",
            ExpectedFile::Sha256(receipt.digest),
            vec![
                ResourceEdit::Add(master.clone()),
                ResourceEdit::Add(opaque.clone()),
            ],
            current.source.clone(),
        )
        .unwrap();
    let inventory = ui
        .package_inventory("working.hmc", current.source.clone())
        .unwrap();
    assert_eq!(inventory.resources.len(), 2);
    let imported = ui
        .import_package_resource(
            "working.hmc",
            ExpectedFile::Sha256(receipt.digest),
            ImportRequest {
                filename: "texture.bin".into(),
                key: "textures/editable.bin".into(),
                kind: "future/texture".into(),
                expected_resource: None,
            },
            current.source.clone(),
        )
        .unwrap();
    let digest: [u8; 32] = Sha256::digest(b"new editable resource").into();
    let edited_package = ui
        .edit_package_resources(
            "working.hmc",
            ExpectedFile::Sha256(imported.digest),
            vec![ResourceEdit::ReplaceBytes {
                key: "textures/editable.bin".into(),
                expected: digest,
                bytes: b"revised resource".to_vec(),
            }],
            current.source.clone(),
        )
        .unwrap();
    let saved = checkpoint::load(&edited_package.path, CheckpointLimits::default()).unwrap();
    assert_eq!(
        saved.resources().iter().find(|r| r.key == master.key),
        Some(&master)
    );
    assert_eq!(
        saved.resources().iter().find(|r| r.key == opaque.key),
        Some(&opaque)
    );
    assert_eq!(
        saved
            .resources()
            .iter()
            .find(|r| r.key == "textures/editable.bin")
            .unwrap()
            .bytes,
        b"revised resource"
    );
    let read = mcp.call("poet_source_read", json!({})).unwrap();
    let inventory = mcp
        .call(
            "poet_hmc_inventory",
            json!({"context":read["context"],"name":"working.hmc"}),
        )
        .unwrap();
    assert_eq!(inventory["resources"].as_array().unwrap().len(), 3);
    let package_records = authority.package_write_evidence(&session).unwrap();
    assert_eq!(package_records.len(), 4);
    let package_record = package_records.last().unwrap();
    assert_eq!(package_record.actor_subject, "fixture-author");
    assert_eq!(package_record.expected_digest, Some(imported.digest));
    assert_eq!(package_record.result_digest, edited_package.digest);
    assert_eq!(
        package_record.source.revision,
        u64::from(current.source_revision)
    );
    let retained_bytes = std::fs::read(&edited_package.path).unwrap();
    let contribution = authority
        .record_contribution(
            &session,
            "fixture-contribution",
            "Actual native Vibe and package fixture",
            5,
            WorkStatus::Unpaid {
                evidence_ref: "fixture-work".into(),
            },
        )
        .unwrap();
    assert_eq!(
        contribution.target.revision,
        u64::from(current.source_revision)
    );
    let stamp = current.authorisation_stamp.clone().unwrap();
    admission.sequence = 2;
    admission.subjects[0].revoked = true;
    authority.replace_admission(signed(admission)).unwrap();
    assert!(!stamp.is_current());
    assert!(ui.snapshot().is_err());
    assert!(mcp.call("poet_source_read", json!({})).is_err());
    assert!(mcp
        .call(
            "poet_material_apply",
            json!({"context":read["context"],"source":"1;"})
        )
        .is_err());
    let reviewer = login(&authority, "fixture-reviewer", &REVIEWER);
    let record = authority
        .record_review(
            &reviewer,
            "fixture-review",
            ReviewDecision::ChangesRequested,
            "Native integration fixture only; visual and human acceptance remain open",
            "fixture-decision",
        )
        .unwrap();
    assert_eq!(record.principal_kind, PrincipalKind::SoftwareAgent);
    assert_eq!(record.target, contribution.target);
    assert_ne!(record.reviewer_subject, contribution.contributor_subject);
    assert_eq!(std::fs::read(&edited_package.path).unwrap(), retained_bytes);
}

#[path = "native_project_workflow_tests.rs"]
mod workflow_tests;

#[path = "native_project_package_tests.rs"]
mod package_workflow_tests;

#[path = "native_project_spatial_tests.rs"]
mod spatial_workflow_tests;

#[path = "native_project_surface_tests.rs"]
mod surface_workflow_tests;

#[path = "native_project_uv_detail_tests.rs"]
mod uv_detail_workflow_tests;

#[path = "native_project_motion_tests.rs"]
mod motion_workflow_tests;

#[path = "native_project_texture_effect_tests.rs"]
mod texture_effect_workflow_tests;

#[path = "native_project_biosphere_round1_tests.rs"]
mod biosphere_round1_tests;
#[path = "native_selected_session_tests.rs"]
mod native_selected_session_tests;

#[path = "native_project_rig_tests.rs"]
mod rig_composed_tests;

#[path = "native_project_numeric_sheet_tests.rs"]
mod numeric_sheet_tests;
