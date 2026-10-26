//! Explicit steward actions using the existing native signer/provider boundary.
use super::{files, runtime::Runtime, types::PackageGrantIntent};
use qualia_core_db::poet_host::native_authority::package_review::PackageReviewGrant;

impl Runtime {
    pub(in crate::graphical::native_entry) fn grant_package(
        &mut self,
        intent: PackageGrantIntent,
    ) -> Result<(), String> {
        intent.validate()?;
        let owner = self.owner()?;
        let session = self.session()?;
        let permission = owner.enrollment_permission(&session)?;
        let expected = crate::graphical::package::parse_digest(&intent.package_digest)?;
        let target = owner.saved_package_target(&session, &intent.filename, &intent.package_id)?;
        if target.package_digest != expected {
            return Err("Package differs from steward's inspected digest; refresh".into());
        }
        self.project
            .as_ref()
            .ok_or("Open the native project before granting package review")?
            .verify_package_target(&target)?;
        let admission = owner.enrollment_snapshot()?;
        let subject = admission
            .snapshot
            .subjects
            .iter()
            .find(|s| s.subject == intent.subject && !s.revoked)
            .ok_or("Package reviewer must already be admitted and unrevoked")?;
        let now = super::runtime::actions::now()?;
        let until = now
            .checked_add(u64::from(intent.seconds))
            .ok_or("Review grant expiry overflow")?
            .min(subject.valid_until_unix)
            .min(admission.snapshot.valid_until_unix);
        if until <= now {
            return Err("No valid reviewer grant window remains".into());
        }
        let signer = self.authority_signer()?;
        owner.grant_package_review(
            &permission,
            PackageReviewGrant {
                id: intent.grant_id,
                target,
                reviewer_subject: intent.subject,
                admission_ref: intent.evidence,
                valid_from_unix: now,
                valid_until_unix: until,
                revoked: false,
            },
            &signer,
        )?;
        self.detach()
    }
    pub(in crate::graphical::native_entry) fn revoke_package(
        &mut self,
        id: &str,
        evidence: &str,
    ) -> Result<(), String> {
        files::bounded_text(id, 4096)?;
        files::bounded_text(evidence, 4096)?;
        let owner = self.owner()?;
        let permission = owner.enrollment_permission(&self.session()?)?;
        let signer = self.authority_signer()?;
        owner.revoke_package_review(&permission, id, evidence, &signer)?;
        self.detach()
    }
}
