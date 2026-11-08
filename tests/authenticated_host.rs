//! Signed test identities exercise the real native owner; they are not customer credentials.
use qualia_cooperative_core::{
    authoring::*,
    projects::{Project, ProjectMembership, ProjectRole},
};
use qualia_core_db::{crypto::network::ed25519, poet_host::native_authority::*};
use std::{
    path::PathBuf,
    sync::{atomic::AtomicBool, Arc},
    time::{SystemTime, UNIX_EPOCH},
};
use synth::{production::*, NativeOutputRoot, SynthHost};

fn root() -> PathBuf {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target/host-tests")
        .join(format!(
            "{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
    std::fs::create_dir_all(&path).unwrap();
    path
}
fn authority(path: &std::path::Path, binding: [u8; 32]) -> (Arc<NativeAuthority>, SessionHandle) {
    std::fs::create_dir_all(path).unwrap();
    const ISSUER: [u8; 32] = [17; 32];
    const ACTOR: [u8; 32] = [23; 32];
    let until = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        + 3600;
    let snapshot = AdmissionSnapshot {
        version: 1,
        sequence: 1,
        issuer_identifier: "test-authority".into(),
        project: Project {
            id: "test-project".into(),
            name: "Synth public API test".into(),
            description: String::new(),
            licensing_ontologies: vec![],
            created_at_unix: 1,
            created_at_instant: None,
        },
        valid_from_unix: 1,
        valid_until_unix: until,
        revoked: false,
        package_reviews: vec![],
        subjects: vec![SubjectAdmission {
            subject: "test-author".into(),
            kind: PrincipalKind::SoftwareAgent,
            controller_public_key: ed25519::public_from_seed(&ACTOR),
            membership: ProjectMembership {
                id: "test-membership".into(),
                project_id: "test-project".into(),
                member_did: "did:test:author".into(),
                role: ProjectRole::Contributor,
                agreed_at_unix: 1,
                agreed_at_instant: None,
            },
            identifiers: vec![IdentifierAssociationEvidence {
                value: "did:test:author".into(),
                scheme: "did:test".into(),
                issuer: "test-authority".into(),
                project_id: "test-project".into(),
                evidence_ref: "test-association".into(),
                valid_from_unix: 1,
                valid_until_unix: until,
            }],
            reviews: vec![],
            admission_ref: "test-adoption".into(),
            valid_from_unix: 1,
            valid_until_unix: until,
            revoked: false,
        }],
    };
    let mut signature = [0; 64];
    ed25519::sign_with_context(
        &ISSUER,
        &admission_digest(&snapshot, 1024 * 1024).unwrap(),
        ADMISSION_CONTEXT,
        &mut signature,
    )
    .unwrap();
    let owner = Arc::new(
        NativeAuthority::open(
            path,
            TrustedProject {
                project_id: "test-project".into(),
                issuer_identifier: "test-authority".into(),
                public_key: ed25519::public_from_seed(&ISSUER),
            },
            SignedAdmission {
                snapshot,
                signature: signature.to_vec(),
            },
            DocumentRevision {
                project_id: "test-project".into(),
                document_id: "asset-document".into(),
                source_id: "asset-source".into(),
                revision: 1,
                source_digest: binding,
                author_subject: Some("test-author".into()),
            },
            1024 * 1024,
            256,
        )
        .unwrap(),
    );
    let challenge = owner.begin_session("test-author", 300).unwrap();
    ed25519::sign_with_context(
        &ACTOR,
        challenge.message_digest(),
        SESSION_CONTEXT,
        &mut signature,
    )
    .unwrap();
    let session = owner.complete_session(challenge, &signature).unwrap();
    (owner, session)
}

#[test]
fn general_host_enforces_admission_edit_context_audit_reopen_and_revocation() {
    let registry = ProducerRegistry::with_builtins();
    let cancel = AtomicBool::new(false);
    let mut production = registry
        .session("files", vec![], ProductionLimits::default())
        .unwrap();
    production
        .submit(include_str!("../examples/production.vibe"), &cancel)
        .unwrap();
    let directory = root();
    let (owner, session) = authority(
        &directory.join("authority"),
        production.document_digest().unwrap(),
    );
    let mut wrong = registry
        .session("files", vec![], ProductionLimits::default())
        .unwrap();
    wrong.submit("let asset = { files: [{ key: \"other.txt\", kind: \"txt\", text: \"wrong source\" }] };", &cancel).unwrap();
    assert!(SynthHost::admit(owner.clone(), session.clone(), wrong).is_err());
    let mut host = SynthHost::admit(owner.clone(), session.clone(), production).unwrap();
    host.configure_output(NativeOutputRoot::admit(&directory).unwrap())
        .unwrap();
    let initial = host.inspect().unwrap();
    assert_eq!(initial.producer, "files");
    assert_eq!(initial.resources.len(), 9);
    assert!(host
        .read_source(&initial.context, 0, 0, 64)
        .unwrap()
        .starts_with("import"));
    assert!(host
        .submit(&initial.context, "asset = { files: [] };", &cancel)
        .is_err());
    assert_eq!(host.inspect().unwrap().context, initial.context);
    host.submit(
        &initial.context,
        r#"requires [capability("capability.invoke")];
        let recolored = capability.invoke("synth.profile.svg", {profile:panel,fill:[240,120,60,255]});
        asset = {files:[{key:"material.svg",resource:recolored}]};"#,
        &cancel,
    )
    .unwrap();
    assert!(host.read_source(&initial.context, 0, 0, 64).is_err());
    let current = host.inspect().unwrap();
    assert_eq!(current.resources[0].0, "material.svg");
    let saved = host
        .save_candidate(&current.context, "material.hmc")
        .unwrap();
    assert!(saved.audit_recorded);
    assert!(host
        .save_candidate(&current.context, "material.hmc")
        .is_err());
    host.reopen_candidate(&current.context, "material.hmc", saved.digest, &cancel)
        .unwrap();
    assert!(host.read_source(&current.context, 0, 0, 64).is_err());
    let shared = Arc::new(std::sync::Mutex::new(host));
    let mut agent = synth::agent::SynthAgentBackend::bound(shared.clone()).unwrap();
    use qualia_mcp::authoring::AuthoringBackend;
    assert_eq!(agent.tools().len(), 7);
    let categories = agent
        .call("synth.ontology.categories", serde_json::json!({}))
        .unwrap();
    assert_eq!(
        categories["schema"],
        "hypermedia.categorical-ontology/v1"
    );
    let radiance_morphisms = agent
        .call(
            "synth.ontology.morphisms",
            serde_json::json!({"category": "radiance"}),
        )
        .unwrap();
    let expected_radiance = synth::production::assets::ontology::MORPHISMS
        .iter()
        .filter(|m| m.category == "radiance")
        .count();
    assert_eq!(radiance_morphisms["count"], expected_radiance);
    let inspection = agent.call("synth.inspect", serde_json::json!({})).unwrap();
    assert_eq!(inspection["producer"], "files");
    assert_eq!(
        inspection["asset_operations"]["operations"]
            .as_array()
            .unwrap()
            .len(),
        synth::production::assets::OPERATIONS.len()
    );
    assert_eq!(inspection["resources"].as_array().unwrap().len(), 1);
    let context = inspection["context"].clone();
    agent
        .call(
            "synth.source.read",
            serde_json::json!({"context": context, "cell": 0, "offset": 0, "bytes": 64}),
        )
        .unwrap();
    agent.call("synth.source.submit", serde_json::json!({"context": context, "source": "asset = { files: [{ key: \"agent.png\", resource: processedPng }] };"})).unwrap();
    assert!(agent
        .call(
            "synth.source.read",
            serde_json::json!({"context": context, "cell": 0, "offset": 0, "bytes": 64})
        )
        .is_err());
    assert_eq!(
        shared.lock().unwrap().inspect().unwrap().resources[0].0,
        "agent.png"
    );
    owner.disconnect(&session).unwrap();
    assert!(shared.lock().unwrap().inspect().is_err());
}
