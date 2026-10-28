use super::*;
use qualia_cooperative_core::authoring::{IdentifierAssociationEvidence, PrincipalKind};
use qualia_core_db::poet_host::native_authority::package_review::PackageActorEvidence;

fn source() -> DocumentRevision {
    DocumentRevision {
        project_id: "project:a".into(),
        document_id: "document:a".into(),
        source_id: "source:a".into(),
        revision: 3,
        source_digest: [3; 32],
        author_subject: Some("actor:author".into()),
    }
}
fn package() -> PackageRevision {
    PackageRevision {
        package_id: "package:a".into(),
        filename: "working.hmc".into(),
        package_digest: [4; 32],
        predecessor_digest: None,
        package_bytes: 512,
        source: source(),
        producer_subject: "actor:author".into(),
    }
}
fn actor() -> PackageActorEvidence {
    PackageActorEvidence {
        subject: "actor:author".into(),
        principal_kind: PrincipalKind::SoftwareAgent,
        authentication_ref: "synthetic-auth".into(),
        identifier_association: IdentifierAssociationEvidence {
            value: "actor-id".into(),
            scheme: "test".into(),
            issuer: "test".into(),
            project_id: "project:a".into(),
            evidence_ref: "synthetic-association".into(),
            valid_from_unix: 1,
            valid_until_unix: 100,
        },
        membership_id: "membership".into(),
        admission_ref: "admission".into(),
        authority_generation: 2,
        occurred_at_unix: 10,
    }
}

#[test]
fn source_choices_require_complete_target_match_for_each_selection() {
    let target = source();
    let request = |id: &str, target, decisions| SourceReviewRequest {
        id: id.into(),
        target,
        requester_subject: "actor:author".into(),
        principal_kind: PrincipalKind::SoftwareAgent,
        authentication_ref: "test".into(),
        authority_generation: 2,
        occurred_at_unix: 10,
        evidence_ref: "inspection".into(),
        decision_ids: decisions,
    };
    let mut different_digest = target.clone();
    different_digest.source_digest = [9; 32];
    let mut different_author = target.clone();
    different_author.author_subject = Some("other".into());
    let rows = source_requests(
        &target,
        &[
            request("pending", target.clone(), vec![]),
            request("answered", target.clone(), vec!["decision".into()]),
            request("other-digest", different_digest, vec![]),
            request("other-author", different_author, vec![]),
        ],
    );
    assert_eq!(rows.len(), 2);
    assert_eq!(
        current_selection(&rows, "pending").unwrap().evidence_ref,
        "inspection"
    );
    assert!(current_selection(&rows, "answered").is_some());
    assert!(current_selection(&rows, "other-digest").is_none());
    assert!(current_selection(&rows, "invented-id").is_none());
    assert!(current_selection(&[], "pending").is_none());
}

#[test]
fn package_choices_preserve_exact_bytes_scope_requester_and_answer_history() {
    let target = package();
    let request = |id: &str, target, decisions| PackageReviewRequest {
        id: id.into(),
        target,
        contribution_id: "contribution:a".into(),
        actor: actor(),
        evidence_ref: "package-inspection".into(),
        decision_ids: decisions,
        contribution_rights_scope: "unassessed".into(),
        known_destination_lineage_complete: false,
    };
    let mut bytes_changed = target.clone();
    bytes_changed.package_digest = [9; 32];
    let mut other_project = target.clone();
    other_project.source.project_id = "project:b".into();
    let mut overwritten = target.clone();
    overwritten.predecessor_digest = Some([8; 32]);
    let rows = package_requests(
        &target,
        &[
            request("pending", target.clone(), vec![]),
            request("answered", target.clone(), vec!["review:a".into()]),
            request("changed", bytes_changed, vec![]),
            request("foreign", other_project, vec![]),
            request("overwritten", overwritten, vec![]),
        ],
    );
    assert_eq!(rows.len(), 2);
    let selected = current_selection(&rows, "pending").unwrap();
    assert_eq!(selected.actor_subject, "actor:author");
    assert_eq!(selected.contribution_id.as_deref(), Some("contribution:a"));
    assert_eq!(rows[1].decision_ids, vec!["review:a"]);
    assert!(current_selection(&rows, "answered").is_some());
    assert!(current_selection(&rows, "changed").is_none());
}

#[test]
fn package_request_contributions_are_this_actors_exact_saved_work() {
    use qualia_cooperative_core::{
        authoring::WorkStatus,
        projects::{Contribution, ContributionPrivacy},
    };
    let target = package();
    let record = |id: &str, target, actor| PackageContributionRecord {
        contribution: Contribution {
            id: id.into(),
            project_id: "project:a".into(),
            contributor_did: "identifier:author".into(),
            description: "synthetic work".into(),
            effort_minutes: 4,
            capital_cents: 0,
            roi_multiplier: 1.0,
            privacy_level: ContributionPrivacy::Private,
            occurred_at_unix: 10,
            occurred_at_instant: None,
            predecessor_id: None,
        },
        target,
        actor,
        work_status: WorkStatus::Unknown,
    };
    let mut other_actor = actor();
    other_actor.subject = "actor:other".into();
    let mut other_bytes = target.clone();
    other_bytes.package_digest = [7; 32];
    let choices = package_contributions(
        &target,
        "actor:author",
        &[
            record("mine", target.clone(), actor()),
            record("theirs", target.clone(), other_actor),
            record("old", other_bytes, actor()),
        ],
    );
    assert_eq!(choices.len(), 1);
    assert_eq!(choices[0].id, "mine");
}

#[test]
fn selected_source_request_decisions_are_linked_and_bound_to_exact_source() {
    use qualia_cooperative_core::authoring::{ReviewDecision, ReviewRecord};
    let target = source();
    let review = |id: &str, target: DocumentRevision| ReviewRecord {
        id: id.into(), target, reviewer_subject: "actor:reviewer".into(),
        reviewer_identifier: "reviewer-id".into(), principal_kind: PrincipalKind::SoftwareAgent,
        authentication_ref: "auth".into(), identifier_association: actor().identifier_association,
        membership_id: "membership".into(), admission_ref: "admission".into(),
        review_grant_id: "grant".into(), review_admission_ref: "review-admission".into(),
        authority_generation: 2, occurred_at_unix: 10, decision: ReviewDecision::ChangesRequested,
        rationale: "inspect this case".into(), decision_evidence_ref: "evidence".into(),
    };
    let choice = RequestChoice { id: "request".into(), actor_subject: "author".into(),
        contribution_id: None, evidence_ref: "request-evidence".into(),
        decision_ids: vec!["review/one".into(), "review/foreign".into(), "review/missing".into()] };
    let mut foreign = target.clone();
    foreign.source_digest = [9; 32];
    let reviews = vec![review("review/one", target.clone()), review("review/foreign", foreign),
        review("review/unlinked", target.clone())];
    let decisions = source_request_decisions(&target, &choice, &reviews);
    assert_eq!(decisions.len(), 3);
    assert_eq!(decisions[0].0, "review/one");
    assert_eq!(decisions[0].1.unwrap().rationale, "inspect this case");
    assert_eq!(decisions[1].0, "review/foreign");
    assert!(decisions[1].1.is_none());
    assert_eq!(decisions[2].0, "review/missing");
    assert!(decisions[2].1.is_none());
}

#[test]
fn selected_package_request_decisions_require_request_and_exact_package() {
    use qualia_cooperative_core::authoring::{ReviewDecision, package_review::PackageReviewRecord};
    let target = package();
    let review = |id: &str, request: &str, target: PackageRevision| PackageReviewRecord {
        id: id.into(), target, request_id: request.into(), actor: actor(),
        review_grant_id: "grant".into(), review_admission_ref: "admission".into(),
        decision: ReviewDecision::ChangesRequested, rationale: "inspect this case".into(),
        decision_evidence_ref: "evidence".into(), known_destination_lineage_complete: false,
        contribution_rights_scope: "unassessed".into(),
    };
    let choice = RequestChoice { id: "request".into(), actor_subject: "author".into(),
        contribution_id: Some("contribution".into()), evidence_ref: "request-evidence".into(),
        decision_ids: vec!["linked".into(), "wrong-request".into(), "wrong-package".into(), "missing".into()] };
    let mut foreign = target.clone();
    foreign.package_digest = [9; 32];
    let reviews = vec![review("linked", "request", target.clone()),
        review("wrong-request", "other", target.clone()), review("wrong-package", "request", foreign),
        review("unlinked", "request", target.clone())];
    let decisions = package_request_decisions(&target, &choice, &reviews);
    assert_eq!(decisions.len(), 4);
    assert_eq!(decisions[0].1.unwrap().rationale, "inspect this case");
    assert!(decisions[1..].iter().all(|(_, decision)| decision.is_none()));
}
