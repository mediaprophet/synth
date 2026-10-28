//! Native selectors project exact canonical evidence; they never grant authority.
use qualia_cooperative_core::authoring::{DocumentRevision, ReviewRecord};
use qualia_core_db::poet_host::native_authority::{
    package_review::{PackageContributionRecord, PackageReviewRecord, PackageReviewRequest, PackageRevision},
    SourceReviewRequest,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct RequestChoice {
    pub id: String,
    pub actor_subject: String,
    pub contribution_id: Option<String>,
    pub evidence_ref: String,
    pub decision_ids: Vec<String>,
}

/// A choice remains usable only while it is in the current inspected evidence.
/// Backend session, authority, scope and independence checks remain mandatory.
pub(super) fn current_selection<'a>(
    choices: &'a [RequestChoice],
    id: &str,
) -> Option<&'a RequestChoice> {
    choices.iter().find(|row| row.id == id)
}

pub(super) fn source_requests(
    target: &DocumentRevision,
    requests: &[SourceReviewRequest],
) -> Vec<RequestChoice> {
    requests
        .iter()
        .filter(|row| &row.target == target)
        .map(|row| RequestChoice {
            id: row.id.clone(),
            actor_subject: row.requester_subject.clone(),
            contribution_id: None,
            evidence_ref: row.evidence_ref.clone(),
            decision_ids: row.decision_ids.clone(),
        })
        .collect()
}

/// Resolve only decisions linked to this request and the exact inspected source.
/// Missing references remain visible rather than being silently treated as absent.
pub(super) fn source_request_decisions<'a>(
    target: &DocumentRevision,
    request: &'a RequestChoice,
    reviews: &'a [ReviewRecord],
) -> Vec<(&'a str, Option<&'a ReviewRecord>)> {
    request.decision_ids.iter().map(|id| {
        (id.as_str(), reviews.iter().find(|record| record.id == *id && &record.target == target))
    }).collect()
}

pub(super) fn package_requests(
    target: &PackageRevision,
    requests: &[PackageReviewRequest],
) -> Vec<RequestChoice> {
    requests
        .iter()
        .filter(|row| &row.target == target)
        .map(|row| RequestChoice {
            id: row.id.clone(),
            actor_subject: row.actor.subject.clone(),
            contribution_id: Some(row.contribution_id.clone()),
            evidence_ref: row.evidence_ref.clone(),
            decision_ids: row.decision_ids.clone(),
        })
        .collect()
}

/// Resolve references only for this exact request and package. Missing or
/// inconsistent references remain visible to the reviewer. This is a projection,
/// not an authorization or independent verification of review records.
pub(super) fn package_request_decisions<'a>(
    target: &PackageRevision,
    request: &'a RequestChoice,
    reviews: &'a [PackageReviewRecord],
) -> Vec<(&'a str, Option<&'a PackageReviewRecord>)> {
    request.decision_ids.iter().map(|id| {
        (id.as_str(), reviews.iter().find(|record| {
            record.id == *id && record.request_id == request.id && &record.target == target
        }))
    }).collect()
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ContributionChoice {
    pub id: String,
    pub actor_subject: String,
}

/// Request creation uses this actor's recorded contribution to these exact bytes.
pub(super) fn package_contributions(
    target: &PackageRevision,
    actor: &str,
    contributions: &[PackageContributionRecord],
) -> Vec<ContributionChoice> {
    contributions
        .iter()
        .filter(|row| &row.target == target && row.actor.subject == actor)
        .map(|row| ContributionChoice {
            id: row.contribution.id.clone(),
            actor_subject: row.actor.subject.clone(),
        })
        .collect()
}

#[cfg(test)]
#[path = "cooperation_selection_tests.rs"]
mod tests;
