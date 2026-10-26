use qualia_cooperative_core::authoring::{PrincipalKind, ReviewDecision};
use qualia_core_db::poet_host::native_authority::WorkflowEvidence;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReviewRequestRow {
    pub id: String,
    pub requester_subject: String,
    pub evidence_ref: String,
    pub pending: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DecisionRow {
    pub id: String,
    pub reviewer_subject: String,
    pub reviewer_kind: PrincipalKind,
    pub revision: u64,
    pub decision: ReviewDecision,
    pub rationale: String,
    pub evidence_ref: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkbenchProjection {
    pub project_id: String,
    pub document_id: String,
    pub source_id: String,
    pub revision: u64,
    pub source_digest: [u8; 32],
    pub actor_subject: String,
    pub actor_kind: PrincipalKind,
    pub can_contribute: bool,
    pub can_review: bool,
    pub pending_reviews: Vec<ReviewRequestRow>,
    pub current_decisions: Vec<DecisionRow>,
    pub historical_decisions: Vec<DecisionRow>,
}

/// Projects persisted service evidence without interpreting an old decision as
/// current approval. Requests are pending only when bound to the exact revision
/// and unanswered; all older-source decisions remain visible as history.
pub fn project(evidence: &WorkflowEvidence) -> WorkbenchProjection {
    let target = &evidence.target;
    let pending_reviews = evidence
        .requests
        .iter()
        .filter(|request| request.target == *target && request.decision_ids.is_empty())
        .map(|request| ReviewRequestRow {
            id: request.id.clone(),
            requester_subject: request.requester_subject.clone(),
            evidence_ref: request.evidence_ref.clone(),
            pending: true,
        })
        .collect();
    let decisions = evidence.reviews.iter().map(|record| DecisionRow {
        id: record.id.clone(),
        reviewer_subject: record.reviewer_subject.clone(),
        reviewer_kind: record.principal_kind,
        revision: record.target.revision,
        decision: record.decision,
        rationale: record.rationale.clone(),
        evidence_ref: record.decision_evidence_ref.clone(),
    });
    let mut current_decisions = Vec::new();
    let mut historical_decisions = Vec::new();
    for (record, row) in evidence.reviews.iter().zip(decisions) {
        if record.target == *target {
            current_decisions.push(row);
        } else {
            historical_decisions.push(row);
        }
    }
    WorkbenchProjection {
        project_id: target.project_id.clone(),
        document_id: target.document_id.clone(),
        source_id: target.source_id.clone(),
        revision: target.revision,
        source_digest: target.source_digest,
        actor_subject: evidence.principal_subject.clone(),
        actor_kind: evidence.principal_kind,
        can_contribute: evidence.can_contribute,
        can_review: evidence.can_review,
        pending_reviews,
        current_decisions,
        historical_decisions,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use qualia_cooperative_core::authoring::DocumentRevision;
    use qualia_core_db::poet_host::native_authority::{
        SourceReviewRequest, WorkflowEvidence,
    };
    use qualia_cooperative_core::authoring::IdentifierAssociationEvidence;
    use qualia_cooperative_core::authoring::ReviewRecord;

    fn revision(number: u64) -> DocumentRevision {
        DocumentRevision {
            project_id: "project:one".into(),
            document_id: "document:one".into(),
            source_id: "source:one".into(),
            revision: number,
            source_digest: [number as u8; 32],
            author_subject: Some("actor:author".into()),
        }
    }

    fn evidence() -> WorkflowEvidence {
        let target = revision(4);
        let stale = revision(3);
        let req = |id: &str, target: DocumentRevision, decisions: Vec<String>| SourceReviewRequest {
            id: id.into(), target, requester_subject: "actor:requester".into(),
            principal_kind: PrincipalKind::Human, authentication_ref: "auth".into(),
            authority_generation: 7, occurred_at_unix: 10, evidence_ref: "review-evidence".into(),
            decision_ids: decisions,
        };
        let decision = |id: &str, target: DocumentRevision, kind: PrincipalKind, value| ReviewRecord {
            id: id.into(), target, reviewer_subject: "actor:reviewer".into(),
            reviewer_identifier: "reviewer-id".into(), principal_kind: kind,
            authentication_ref: "auth".into(),
            identifier_association: IdentifierAssociationEvidence {
                value: "reviewer-id".into(), scheme: "test".into(), issuer: "issuer".into(),
                project_id: "project:one".into(), evidence_ref: "association".into(),
                valid_from_unix: 1, valid_until_unix: 100,
            },
            membership_id: "membership".into(), admission_ref: "admission".into(),
            review_grant_id: "grant".into(), review_admission_ref: "review-admission".into(),
            authority_generation: 7, occurred_at_unix: 20, decision: value,
            rationale: "reviewed this source".into(), decision_evidence_ref: "review-note".into(),
        };
        WorkflowEvidence {
                target: target.clone(), principal_subject: "actor:reviewer".into(),
                principal_kind: PrincipalKind::Human, authority_generation: 7,
                can_contribute: false, can_review: true, contributions: vec![],
                requests: vec![req("pending-current", target.clone(), vec![]),
                    req("answered-current", target.clone(), vec!["decision-current".into()]),
                    req("pending-old", stale.clone(), vec![])],
                reviews: vec![
                    decision("decision-current", target, PrincipalKind::Human, ReviewDecision::Accepted),
                    decision("decision-old", stale, PrincipalKind::SoftwareAgent, ReviewDecision::Rejected),
                ], package_writes: vec![],
        }
    }

    #[test]
    fn only_unanswered_requests_for_the_exact_revision_are_pending() {
        let projection = project(&evidence());
        assert_eq!(projection.revision, 4);
        assert_eq!(projection.pending_reviews.iter().map(|row| row.id.as_str()).collect::<Vec<_>>(),
            vec!["pending-current"]);
        assert_eq!(projection.current_decisions.iter().map(|row| row.id.as_str()).collect::<Vec<_>>(),
            vec!["decision-current"]);
        assert_eq!(projection.historical_decisions.iter().map(|row| row.id.as_str()).collect::<Vec<_>>(),
            vec!["decision-old"]);
        assert!(projection.can_review);
        assert!(!projection.can_contribute);
        assert_eq!(projection.current_decisions[0].reviewer_kind, PrincipalKind::Human);
        assert_eq!(projection.historical_decisions[0].reviewer_kind, PrincipalKind::SoftwareAgent);
    }
}
