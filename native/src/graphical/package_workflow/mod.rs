//! Exact saved-package workflow; authority and durable records remain Core-owned.
mod service;
use crate::graphical::preview::SourceRef;
use qualia_cooperative_core::authoring::{ReviewDecision, WorkStatus};
use qualia_core_db::poet_host::native_authority::package_review::{
    PackageRevision, PackageWorkflowEvidence,
};
pub use service::{verify_package_guard, PackageWorkflowService};

#[derive(Clone, Debug)]
pub struct PackageSelection {
    pub filename: String,
    pub package_id: String,
    pub digest: [u8; 32],
}
impl PackageSelection {
    pub fn validate(&self) -> Result<(), String> {
        if self.filename.is_empty()
            || self.filename.len() > 128
            || self.package_id.trim().is_empty()
            || self.package_id.len() > 4096
            || self.filename.contains('\0')
            || self.package_id.contains('\0')
        {
            return Err(
                "Explicit bounded package filename and project-scoped release identifier required"
                    .into(),
            );
        }
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub enum PackageWorkflowAction {
    Inspect(PackageSelection),
    Contribution {
        target: PackageRevision,
        generation: u64,
        id: String,
        description: String,
        minutes: u32,
        status: WorkStatus,
    },
    Request {
        target: PackageRevision,
        generation: u64,
        contribution_id: String,
        id: String,
        evidence: String,
    },
    Decide {
        target: PackageRevision,
        generation: u64,
        request_id: String,
        id: String,
        decision: ReviewDecision,
        rationale: String,
        evidence: String,
    },
}
impl PackageWorkflowAction {
    pub fn validate(&self) -> Result<(), String> {
        let fields: Vec<&str> = match self {
            Self::Inspect(selection) => return selection.validate(),
            Self::Contribution {
                id,
                description,
                status,
                ..
            } => {
                let mut fields = vec![id.as_str(), description.as_str()];
                match status {
                    WorkStatus::Unknown => (),
                    WorkStatus::Unpaid { evidence_ref }
                    | WorkStatus::SelfFunded { evidence_ref }
                    | WorkStatus::PartiallyCompensated { evidence_ref }
                    | WorkStatus::Compensated { evidence_ref }
                    | WorkStatus::Disputed { evidence_ref } => fields.push(evidence_ref),
                }
                fields
            }
            Self::Request {
                contribution_id,
                id,
                evidence,
                ..
            } => vec![contribution_id, id, evidence],
            Self::Decide {
                request_id,
                id,
                rationale,
                evidence,
                ..
            } => vec![request_id, id, rationale, evidence],
        };
        if fields
            .iter()
            .any(|v| v.trim().is_empty() || v.len() > 4096 || v.contains('\0'))
        {
            return Err("Package workflow fields require one to 4096 bytes".into());
        }
        let target = match self {
            Self::Inspect(_) => unreachable!(),
            Self::Contribution { target, .. }
            | Self::Request { target, .. }
            | Self::Decide { target, .. } => target,
        };
        PackageSelection {
            filename: target.filename.clone(),
            package_id: target.package_id.clone(),
            digest: target.package_digest,
        }
        .validate()
    }
}

#[derive(Clone, Debug)]
pub struct PackageWorkflowView {
    pub source: SourceRef,
    pub evidence: PackageWorkflowEvidence,
    pub(super) publication: PackagePublicationStamp,
}
impl PackageWorkflowView {
    pub fn publication_is_current(&self) -> bool {
        self.evidence.target == self.publication.target
            && self.evidence.authority_generation == self.publication.generation
            && self.source.artifact_id == self.publication.target.source.source_id
            && self.source.check_current().is_ok()
            && self.publication.is_current()
    }
}
/// Ephemeral native proof only: never serialized or reconstructed from a client.
#[derive(Clone)]
pub(super) struct PackagePublicationStamp {
    selection: crate::graphical::native_project::SessionSlot,
    authority: std::sync::Arc<qualia_core_db::poet_host::native_authority::NativeAuthority>,
    session: qualia_core_db::poet_host::native_authority::SessionHandle,
    generation: u64,
    target: PackageRevision,
}
impl std::fmt::Debug for PackagePublicationStamp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("NativePackagePublicationStamp")
    }
}
impl PackagePublicationStamp {
    fn is_current(&self) -> bool {
        // Lock order matches selected editor operations: selection -> authority.
        // The publication path never waits for a busy selection or authority.
        let Ok(selection) = self.selection.try_lock() else { return false; };
        if selection.as_ref() != Some(&self.session) { return false; }
        self.authority
            .package_publication_is_current(&self.session, self.generation, &self.target)
    }
}
