//! PUBLIC TEST SEEDS copied from native_project_tests: not production credentials.
//! They establish only a controlled SoftwareAgent fixture trust scope.
use crate::{files, RunInfo};
use poet_cli::graphical::cooperation::{source_digest, GatewayLimits, Scope};
use qualia_cooperative_core::{
    authoring::*,
    projects::{Project, ProjectMembership, ProjectRole},
};
use qualia_core_db::{
    crypto::network::ed25519,
    poet_host::{
        invoke::studio_store::{self, StudioArtifact},
        native_authority::*,
    },
};
use std::{
    path::Path,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

const AUTHORITY: [u8; 32] = [37; 32];
const AUTHOR: [u8; 32] = [41; 32];
const REVIEWER: [u8; 32] = [43; 32];
pub const AUTHOR_SUBJECT: &str = "software-fixture-author";
pub const REVIEWER_SUBJECT: &str = "software-fixture-reviewer";
const ISSUER: &str = "urn:poet:software-fixture:public-test-issuer";

fn anchor(project: &str) -> TrustedProject {
    TrustedProject {
        project_id: project.into(),
        issuer_identifier: ISSUER.into(),
        public_key: ed25519::public_from_seed(&AUTHORITY),
    }
}
pub fn create(root: &Path) -> Result<(Scope, Arc<NativeAuthority>), String> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_secs();
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_nanos();
    let id = format!(
        "urn:poet:software-fixture:cg-gs:{}-{nonce}",
        std::process::id()
    );
    let scope = Scope {
        project_id: id.clone(),
        document_id: format!("{id}:document"),
        source_id: format!("{id}:source"),
        domain: "Geometry".into(),
    };
    studio_store::create_artifact_if_absent(StudioArtifact::new(
        &scope.source_id,
        "Geometry",
        "Community Grounds growing station — controlled software fixture",
    ))?;
    let until = now
        .checked_add(7 * 86400)
        .ok_or("admission time overflow")?;
    let subject = |name: &str, key: &[u8; 32], reviewer: bool| SubjectAdmission {
        subject: name.into(),
        kind: PrincipalKind::SoftwareAgent,
        controller_public_key: ed25519::public_from_seed(key),
        membership: ProjectMembership {
            id: format!("membership/{name}"),
            project_id: id.clone(),
            member_did: format!("urn:poet:software-fixture:actor:{name}"),
            role: ProjectRole::Contributor,
            agreed_at_unix: now as u32,
            agreed_at_instant: None,
        },
        identifiers: vec![IdentifierAssociationEvidence {
            value: format!("urn:poet:software-fixture:actor:{name}"),
            scheme: "urn".into(),
            issuer: ISSUER.into(),
            project_id: id.clone(),
            evidence_ref: "public-test-seed/software-only".into(),
            valid_from_unix: now,
            valid_until_unix: until,
        }],
        reviews: if reviewer {
            vec![ReviewAdmission {
                id: "software-technical-source-review".into(),
                document_id: scope.document_id.clone(),
                admission_ref: "fixture-explicit-review-grant".into(),
                valid_from_unix: now,
                valid_until_unix: until,
                revoked: false,
            }]
        } else {
            vec![]
        },
        admission_ref: "controlled-fixture-only/no-human-or-host-adoption".into(),
        valid_from_unix: now,
        valid_until_unix: until,
        revoked: false,
    };
    let snapshot = AdmissionSnapshot {
        package_reviews: Vec::new(),
        version: 1,
        sequence: 1,
        issuer_identifier: ISSUER.into(),
        project: Project {
            id: id.clone(),
            name: "Growing station controlled software fixture".into(),
            description: "Original synthetic concept; no real-host adoption or human acceptance"
                .into(),
            licensing_ontologies: vec![],
            created_at_unix: now as u32,
            created_at_instant: None,
        },
        valid_from_unix: now,
        valid_until_unix: until,
        revoked: false,
        subjects: vec![
            subject(AUTHOR_SUBJECT, &AUTHOR, false),
            subject(REVIEWER_SUBJECT, &REVIEWER, true),
        ],
    };
    let mut signature = [0; 64];
    ed25519::sign_with_context(
        &AUTHORITY,
        &admission_digest(&snapshot, 1024 * 1024)?,
        ADMISSION_CONTEXT,
        &mut signature,
    )
    .map_err(|e| format!("fixture admission signing: {e:?}"))?;
    let artifact = studio_store::get_artifact_stamped(&scope.source_id)
        .ok_or("new fixture source is absent")?
        .0;
    let document = DocumentRevision {
        project_id: id.clone(),
        document_id: scope.document_id.clone(),
        source_id: scope.source_id.clone(),
        revision: artifact.lamport_clock.into(),
        source_digest: source_digest(&artifact, GatewayLimits::default())?,
        author_subject: None,
    };
    let authority = NativeAuthority::open(
        root,
        anchor(&id),
        SignedAdmission {
            snapshot,
            signature: signature.to_vec(),
        },
        document,
        1024 * 1024,
        4096,
    )?;
    Ok((scope, Arc::new(authority)))
}
pub fn retained(root: &Path, info: &RunInfo) -> Result<Arc<NativeAuthority>, String> {
    info.validate()?;
    files::existing_directory(root, false)?;
    Ok(Arc::new(NativeAuthority::open_retained_current(
        root,
        anchor(&info.project_id),
        1024 * 1024,
        4096,
    )?))
}
pub fn login(authority: &NativeAuthority, reviewer: bool) -> Result<SessionHandle, String> {
    let (subject, key) = if reviewer {
        (REVIEWER_SUBJECT, &REVIEWER)
    } else {
        (AUTHOR_SUBJECT, &AUTHOR)
    };
    let challenge = authority.begin_session(subject, 3600)?;
    let mut signature = [0; 64];
    ed25519::sign_with_context(
        key,
        challenge.message_digest(),
        SESSION_CONTEXT,
        &mut signature,
    )
    .map_err(|e| format!("fixture session signing: {e:?}"))?;
    authority.complete_session(challenge, &signature)
}
