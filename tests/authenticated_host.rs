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
    assert_eq!(agent.tools().len(), 8);
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
    // Verify Antigravity Python SDK / FastMCP identifier-safe mode, HTML work presentation,
    // and the unified QualiaDB 0.0.41 comprehensive MCP toolset.
    let mut agy_agent = synth::agent::SynthAgentBackend::bound(shared.clone())
        .unwrap()
        .with_identifier_safe_tools()
        .with_qualia_engine_tools();
    assert_eq!(agy_agent.tools().len(), 8 + qualia_mcp::stable_mcp_tools().len());
    for tool in agy_agent.tools() {
        assert!(!tool.name.contains('.'), "Tool name must be a valid Python identifier: {}", tool.name);
    }
    let sdk_res = agy_agent
        .call("etalon_vibe_sdk", serde_json::json!({"op": "list_families"}))
        .unwrap();
    assert_eq!(sdk_res["status"], "ok");
    let anatomy_res = agy_agent
        .call("living_world_anatomy", serde_json::json!({"op": "catalog"}))
        .unwrap();
    assert_eq!(anatomy_res["status"], "ok");
    let cosmic_res = agy_agent
        .call("cosmic_coordinates", serde_json::json!({"op": "celestial_profile", "body": "earth"}))
        .unwrap();
    assert_eq!(cosmic_res["status"], "ok");
    let agy_inspect = agy_agent.call("synth_inspect", serde_json::json!({})).unwrap();
    let agy_ctx = agy_inspect["context"].clone();
    let presented = agy_agent
        .call(
            "synth_package_present",
            serde_json::json!({"context": agy_ctx, "filename": "showcase.html"}),
        )
        .unwrap();
    assert_eq!(presented["schema"], "synth.presentation-html/v1");
    assert!(presented["antigravity_embed_hint"]
        .as_str()
        .unwrap()
        .contains("<agent-embed"));
    let html_bytes = std::fs::read_to_string(directory.join("showcase.html")).unwrap();
    assert!(html_bytes.contains("--background: #0a1014"));
    assert!(html_bytes.contains("CsiFrame"));
    assert!(html_bytes.contains("UV01"));
    assert!(html_bytes.contains("data:image/png;base64,"));
    assert!(agy_agent
        .call(
            "synth_package_present",
            serde_json::json!({"context": agy_ctx, "filename": "showcase.html"}),
        )
        .is_err());
    // Verify Synth self-hosted Tray & Studio environment bootstrap (`examples/tray_environment.vibe`)
    let env = synth::tray_env::bootstrap_environment().unwrap();
    assert_eq!(env.schema, "synth.tray-environment/v1");
    assert_eq!(env.resources.len(), 71);
    assert!(env
        .resources
        .iter()
        .any(|r| r.key == "world_of_god/human_welfare_female_trimester_3.10d"));
    assert!(env
        .resources
        .iter()
        .any(|r| r.key == "world_of_god/eucalyptus_botanical_organ.10d"));
    assert!(env.emblem_svg_data_uri.starts_with("data:image/svg+xml;base64,"));
    assert!(env.banner_png_data_uri.starts_with("data:image/png;base64,"));
    assert!(env.active_tray_icon_data_uri.starts_with("data:image/png;base64,"));
    assert!(env.ready_wav_data_uri.starts_with("data:audio/wav;base64,"));
    assert!(env.ecosystem.vibe_sdk_families > 0);
    assert!(env.ecosystem.living_world_entries > 0);
    // Verify the 5-tab full-capabilities HTML showcase includes all substantive capabilities
    assert!(env.showcase_html.contains("id=\"tab-assets\""));
    assert!(env.showcase_html.contains("id=\"tab-ontology\""));
    assert!(env.showcase_html.contains("id=\"tab-mcp\""));
    assert!(env.showcase_html.contains("id=\"tab-sdk-living\""));
    assert!(env.showcase_html.contains("id=\"tab-llm-vault\""));
    assert!(env.showcase_html.contains("runLiveVibeProduction"));
    assert!(env.showcase_html.contains("invokeMcpToolLive"));
    // Verify non-conflicting port policy and local HTTP Studio server (`src/web_server.rs`)
    assert!(synth::web_server::is_reserved_webizen_port(8080));
    assert!(synth::web_server::is_reserved_webizen_port(4242));
    assert!(synth::web_server::is_reserved_webizen_port(9001));
    assert!(!synth::web_server::is_reserved_webizen_port(8787));
    assert!(synth::web_server::spawn_background_server(Some(8080)).is_err());
    let server_url = synth::web_server::spawn_background_server(Some(0)).unwrap();
    let server_port: u16 = server_url
        .rsplit_once(':')
        .unwrap()
        .1
        .parse()
        .unwrap();
    assert!(!synth::web_server::is_reserved_webizen_port(server_port));
    {
        use std::io::{Read, Write};
        let mut stream = std::net::TcpStream::connect(("127.0.0.1", server_port)).unwrap();
        stream
            .write_all(b"GET /api/status HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n")
            .unwrap();
        let mut resp = String::new();
        stream.read_to_string(&mut resp).unwrap();
        assert!(resp.starts_with("HTTP/1.1 200 OK"));
        assert!(resp.contains("\"schema\": \"synth.tray-environment/v1\""));
    }
    let mut standalone_mcp = synth::mcp_full::StandaloneSynthMcp::default();
    let env_mcp = standalone_mcp
        .call("synth_environment_bootstrap", serde_json::json!({}))
        .unwrap();
    assert_eq!(env_mcp["schema"], "synth.tray-environment/v1");
    assert_eq!(env_mcp["resources"].as_array().unwrap().len(), 71);
    owner.disconnect(&session).unwrap();
    assert!(shared.lock().unwrap().inspect().is_err());
}
