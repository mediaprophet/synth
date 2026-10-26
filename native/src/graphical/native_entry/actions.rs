//! Explicit operator actions; no transport constructs a principal or admission.
use super::*;
use qualia_cooperative_core::{
    authoring::{IdentifierAssociationEvidence, PrincipalKind},
    projects::{ProjectMembership, ProjectRole},
};
use qualia_core_db::poet_host::native_authority::{
    admission_digest,
    enrollment::{EnrollmentApproval, EnrollmentRequest},
    ReviewAdmission, SubjectAdmission,
};
use std::time::{SystemTime, UNIX_EPOCH};
pub(crate) fn now() -> Result<u64, String> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .map_err(|e| e.to_string())
}
impl Runtime {
    pub fn perform(&mut self, action: LaunchAction) -> Result<(), String> {
        if let Err(error) = self.guard() {
            let _ = self.detach();
            self.adopted = false;
            self.bump();
            return Err(error);
        }
        let result = match action {
            LaunchAction::GrantPackageReview(intent) => self.grant_package(intent),
            LaunchAction::RevokePackageReview { grant_id, evidence } => {
                self.revoke_package(&grant_id, &evidence)
            }
            LaunchAction::Adopt { fingerprint } => self.adopt(&fingerprint),
            LaunchAction::CreateProject {
                name,
                profile,
                kind,
                evidence,
            } => super::super::create::create_project(self, &name, &profile, kind, &evidence),
            LaunchAction::CreateProfile { profile, kind } => {
                self.create_profile(&profile, kind).map(|_| ())
            }
            LaunchAction::SubmitEnrollment { profile, evidence } => {
                self.submit_enrollment(&profile, &evidence)
            }
            LaunchAction::Login { profile, filename } => self.login(&profile, &filename),
            LaunchAction::Logout => self.detach(),
            LaunchAction::Approve {
                id,
                evidence,
                human_evidence,
            } => self.approve(&id, evidence, human_evidence),
            LaunchAction::Decline { id, evidence } => self.decline(&id, &evidence),
            LaunchAction::GrantReview { subject, evidence } => {
                self.grant_review(&subject, &evidence)
            }
            LaunchAction::Revoke { subject, evidence } => self.revoke(&subject, &evidence),
        };
        self.bump();
        result
    }
    pub(in crate::graphical::native_entry) fn create_profile(
        &self,
        label: &str,
        kind: PrincipalKind,
    ) -> Result<Profile, String> {
        files::profile_name(label)?;
        let path = self.files.profile_path(label, true)?;
        if path.join("profile.json").exists() {
            return Err(
                "Native profile already exists; unlock it rather than replacing its key".into(),
            );
        }
        let vault = KeyVault::open_native_secure(
            path.to_str().ok_or("Native profile path is not Unicode")?,
            true,
        )?;
        let signer = NativeSigner::from_vault(&vault, files::CONTROLLER_LABEL)?;
        let profile = Profile {
            version: 1,
            label: label.into(),
            requested_kind: kind,
            controller_public_key: signer.public_key(),
        };
        self.files.save_profile(&profile)?;
        Ok(profile)
    }
    fn submit_enrollment(&self, label: &str, evidence: &str) -> Result<(), String> {
        files::bounded_text(evidence, 4096)?;
        let owner = self.owner()?;
        let (profile, vault) = self.unlock(label)?;
        let signer = NativeSigner::from_vault(&vault, files::CONTROLLER_LABEL)?;
        let admission = owner.enrollment_snapshot()?;
        let now = now()?;
        let until = admission.snapshot.valid_until_unix.min(
            now.checked_add(365 * 86400)
                .ok_or("Enrollment window overflow")?,
        );
        let candidate = candidate(
            &profile,
            &admission.snapshot.project.id,
            &admission.snapshot.issuer_identifier,
            ProjectRole::Contributor,
            evidence,
            now,
            until,
        )?;
        let request = EnrollmentRequest::signed(
            admission.snapshot.project.id.clone(),
            admission.snapshot.sequence,
            admission_digest(&admission.snapshot, 4 * 1024 * 1024)?,
            now,
            until.min(now + 86400),
            candidate,
            &signer,
        )?;
        owner.submit_enrollment(request)?;
        Ok(())
    }
    pub(in crate::graphical::native_entry) fn authority_signer(
        &self,
    ) -> Result<NativeSigner, String> {
        let label = self
            .current_profile
            .as_deref()
            .ok_or("Authenticate the project authority's steward profile first")?;
        let (_, vault) = self.unlock(label)?;
        NativeSigner::from_vault(&vault, files::AUTHORITY_LABEL)
    }
    fn approve(
        &mut self,
        id: &str,
        evidence: String,
        human_evidence: Option<String>,
    ) -> Result<(), String> {
        let owner = self.owner()?;
        let permission = owner.enrollment_permission(&self.session()?)?;
        let signer = self.authority_signer()?;
        owner.approve_enrollment(
            &permission,
            id,
            EnrollmentApproval {
                decision_evidence_ref: evidence,
                human_verification_ref: human_evidence,
            },
            &signer,
        )?;
        self.detach()
    }
    fn decline(&self, id: &str, evidence: &str) -> Result<(), String> {
        let owner = self.owner()?;
        let permission = owner.enrollment_permission(&self.session()?)?;
        owner.decline_enrollment(&permission, id, evidence)
    }
    fn grant_review(&mut self, subject: &str, evidence: &str) -> Result<(), String> {
        files::bounded_text(evidence, 4096)?;
        let owner = self.owner()?;
        let permission = owner.enrollment_permission(&self.session()?)?;
        let signer = self.authority_signer()?;
        let admission = owner.enrollment_snapshot()?;
        let admitted = admission
            .snapshot
            .subjects
            .iter()
            .find(|s| s.subject == subject && !s.revoked)
            .ok_or("Review subject is not currently admitted")?;
        let now = now()?;
        let config = self.config.as_ref().ok_or("Native project missing")?;
        let id = format!(
            "urn:qualia:review-grant:{}:{}",
            admission.snapshot.sequence,
            files::hex_key(&signer.public_key())
        );
        owner.grant_enrollment_review(
            &permission,
            subject,
            ReviewAdmission {
                id,
                document_id: config.document_id.clone(),
                admission_ref: evidence.into(),
                valid_from_unix: now,
                valid_until_unix: admitted
                    .valid_until_unix
                    .min(admission.snapshot.valid_until_unix),
                revoked: false,
            },
            &signer,
        )?;
        self.detach()
    }
    fn revoke(&mut self, subject: &str, evidence: &str) -> Result<(), String> {
        let owner = self.owner()?;
        let permission = owner.enrollment_permission(&self.session()?)?;
        if owner
            .enrollment_snapshot()?
            .snapshot
            .subjects
            .iter()
            .any(|s| s.subject == subject && s.membership.role == ProjectRole::Steward)
        {
            return Err("Steward revocation requires a separate explicit authority rotation and recovery procedure".into());
        }
        let signer = self.authority_signer()?;
        owner.revoke_enrollment(&permission, subject, evidence, &signer)?;
        self.detach()
    }
}
pub(crate) fn candidate(
    profile: &Profile,
    project: &str,
    issuer: &str,
    role: ProjectRole,
    evidence: &str,
    now: u64,
    until: u64,
) -> Result<SubjectAdmission, String> {
    let member = files::subject(&profile.controller_public_key);
    Ok(SubjectAdmission {
        subject: member.clone(),
        kind: profile.requested_kind,
        controller_public_key: profile.controller_public_key,
        membership: ProjectMembership {
            id: format!(
                "urn:qualia:membership:{project}:{}",
                files::hex_key(&profile.controller_public_key)
            ),
            project_id: project.into(),
            member_did: member.clone(),
            role,
            agreed_at_unix: now
                .try_into()
                .map_err(|_| "Membership time exceeds canonical u32 range")?,
            agreed_at_instant: None,
        },
        identifiers: vec![IdentifierAssociationEvidence {
            value: member,
            scheme: "urn:qualia:native-controller".into(),
            issuer: issuer.into(),
            project_id: project.into(),
            evidence_ref: evidence.into(),
            valid_from_unix: now,
            valid_until_unix: until,
        }],
        reviews: Vec::new(),
        admission_ref: evidence.into(),
        valid_from_unix: now,
        valid_until_unix: until,
        revoked: false,
    })
}
