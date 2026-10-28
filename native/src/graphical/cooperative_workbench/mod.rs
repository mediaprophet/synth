mod projection;
pub(super) use projection::{project, WorkbenchProjection};

use super::package::hex_digest;
use dioxus::prelude::*;
use projection::DecisionRow;

fn decision_row(row: &DecisionRow, historical: bool) -> Element {
    let scope = if historical {
        "Historical source revision"
    } else {
        "Decision on displayed revision"
    };
    let acceptance = "Recorded actor capacity is an admission label. This source decision does not establish adopted human authority, independent human review, or actual human/domain acceptance of an HMC package or master.";
    rsx! {
        article { style:"padding:10px;border:1px solid #465463;margin:6px 0;overflow-wrap:anywhere;",
            p { "{scope} · revision {row.revision} · {row.decision:?}" }
            p { "Decision {row.id} · acting subject {row.reviewer_subject} · admitted capacity {row.reviewer_kind:?}" }
            p { "{row.rationale} · evidence {row.evidence_ref}" }
            small { "{acceptance}" }
        }
    }
}

fn panel(projection: &WorkbenchProjection) -> Element {
    let next_step = if !projection.pending_reviews.is_empty() && projection.can_review {
        "Next step: select this exact pending request below and record a source-only decision after inspection."
    } else if !projection.pending_reviews.is_empty() {
        "Next step: an admitted reviewer must inspect and decide this exact pending request."
    } else if projection.can_contribute {
        "Next step: record actual contribution evidence or request review for this exact source revision."
    } else if projection.can_review {
        "Next step: inspect the source; no unanswered review request is recorded for this revision."
    } else {
        "Next step: refresh native admission or ask an admitted author/reviewer to perform the available source action."
    };
    rsx! {
        section { style:"padding:16px;border-bottom:1px solid #465463;",
            h2 { "Cooperative source workbench" }
            p { "Project {projection.project_id} · document {projection.document_id}" }
            p { "Source {projection.source_id} · exact revision {projection.revision} · SHA256 {hex_digest(&projection.source_digest)}" }
            p { "Signed-in actor {projection.actor_subject} · {projection.actor_kind:?}" }
            p { "Current capabilities: contribution {projection.can_contribute:?}; source review {projection.can_review:?}. These are separate from authorship, reviewer identifiers, and compensation." }
            p { "{next_step}" }
            h3 { "Pending reviews for this exact source revision" }
            if projection.pending_reviews.is_empty() {
                p { "No unanswered review requests are recorded for this revision." }
            }
            for request in &projection.pending_reviews {
                article { style:"padding:10px;border:1px solid #465463;margin:6px 0;overflow-wrap:anywhere;",
                    p { "Pending request {request.id} · requested by {request.requester_subject}" }
                    p { "Evidence {request.evidence_ref} · pending {request.pending}" }
                    if !projection.can_review { p { "Next step: an admitted reviewer can inspect this exact revision and record a decision." } }
                }
            }
            h3 { "Decisions for this revision" }
            if projection.current_decisions.is_empty() { p { "No source decisions recorded for this revision." } }
            for row in &projection.current_decisions { { decision_row(row, false) } }
            h3 { "Historical decisions on earlier source revisions" }
            if projection.historical_decisions.is_empty() { p { "No earlier source decisions are retained." } }
            for row in &projection.historical_decisions { { decision_row(row, true) } }
            p { "Source review decisions do not accept package resources, permissions, species/thing completeness, or a high-grade master." }
        }
    }
}

#[component]
pub(super) fn CooperativeWorkbench(projection: WorkbenchProjection, status: String) -> Element {
    rsx! {
        {panel(&projection)}
        if !status.is_empty() {
            p { "Workflow status / next step: {status}" }
        }
    }
}
