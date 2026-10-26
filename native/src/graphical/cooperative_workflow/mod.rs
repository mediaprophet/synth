//! Native cooperative operator projection. Authority and evidence remain core-owned.
mod service;
mod source;
use super::preview::SourceRef;
use qualia_cooperative_core::authoring::{DocumentRevision, ReviewDecision, WorkStatus};
use qualia_core_db::poet_host::native_authority::WorkflowEvidence;
pub use service::WorkflowService;
pub use source::source_difference;

#[derive(Clone, Debug)]
pub enum WorkflowAction {
    Inspect,
    Contribution {
        target: DocumentRevision,
        id: String,
        description: String,
        minutes: u32,
        work_status: WorkStatus,
    },
    RequestReview {
        target: DocumentRevision,
        id: String,
        evidence: String,
    },
    Review {
        target: DocumentRevision,
        request_id: String,
        id: String,
        decision: ReviewDecision,
        rationale: String,
        evidence: String,
    },
}

#[derive(Clone, Debug)]
pub struct WorkflowView {
    pub evidence: WorkflowEvidence,
    pub source: SourceRef,
    pub source_cells: Vec<String>,
    pub source_history: Vec<String>,
    /// Actual retained undo source contents. Its original revision is unknown;
    /// never label this an exact historical package or full HMC snapshot.
    pub previous_source_json: Option<String>,
    pub current_source_json: String,
}

impl WorkflowAction {
    pub fn validate_bounds(&self) -> Result<(), String> {
        let texts: Vec<&str> = match self {
            Self::Inspect => Vec::new(),
            Self::Contribution {
                id,
                description,
                work_status,
                ..
            } => {
                let evidence = match work_status {
                    WorkStatus::Unknown => None,
                    WorkStatus::Unpaid { evidence_ref }
                    | WorkStatus::SelfFunded { evidence_ref }
                    | WorkStatus::PartiallyCompensated { evidence_ref }
                    | WorkStatus::Compensated { evidence_ref }
                    | WorkStatus::Disputed { evidence_ref } => Some(evidence_ref.as_str()),
                };
                [Some(id.as_str()), Some(description.as_str()), evidence]
                    .into_iter()
                    .flatten()
                    .collect()
            }
            Self::RequestReview { id, evidence, .. } => vec![id, evidence],
            Self::Review {
                request_id,
                id,
                rationale,
                evidence,
                ..
            } => vec![request_id, id, rationale, evidence],
        };
        if texts
            .iter()
            .any(|text| text.is_empty() || text.len() > 4096)
        {
            return Err("Explicit workflow fields require one to 4096 bytes".into());
        }
        Ok(())
    }
}
