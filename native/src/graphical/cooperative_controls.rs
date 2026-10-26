//! Human-operated native contributor/reviewer controls over the one host worker.
use super::{
    controller::EditorSnapshot,
    cooperative_workflow::{self, WorkflowAction, WorkflowView},
    jobs::{self, Action, Context, HostHandle},
    package::hex_digest,
};
use dioxus::prelude::*;
use qualia_cooperative_core::authoring::{ReviewDecision, WorkStatus};

#[derive(Clone)]
struct Inspected {
    context: Context,
    view: WorkflowView,
}

fn command(
    action: WorkflowAction,
    host: HostHandle,
    current: EditorSnapshot,
    mut inspected: Signal<Option<Inspected>>,
    mut busy: Signal<bool>,
    mut status: Signal<String>,
) {
    if busy() {
        return;
    }
    if let Err(error) = action.validate_bounds() {
        status.set(error);
        return;
    }
    busy.set(true);
    inspected.set(None);
    let origin = Context::from_snapshot(&current);
    let guard = host.clone();
    spawn(async move {
        match jobs::run(host, Action::Workflow { action, expected: current.source.clone() }, origin).await {
            Ok(reply) if jobs::completion_is_current(&guard, &current, &reply) => {
                if let Some(view) = reply.workflow {
                    inspected.set(Some(Inspected { context: Context::from_snapshot(&current), view }));
                }
                status.set(reply.message);
            }
            Ok(_) => status.set("Source/session/authority changed; workflow evidence withheld. Draft preserved; refresh source and inspect again.".into()),
            Err(error) => status.set(error),
        }
        busy.set(false);
    });
}

fn current_view(
    inspected: Signal<Option<Inspected>>,
    current: &EditorSnapshot,
) -> Option<WorkflowView> {
    inspected()
        .filter(|inspection| {
            inspection.context == Context::from_snapshot(current)
                && current.publication_is_current()
                && inspection.view.source == current.source
        })
        .map(|inspection| inspection.view)
}

fn paid_status(value: &str, evidence: String) -> WorkStatus {
    match value {
        "unpaid" => WorkStatus::Unpaid {
            evidence_ref: evidence,
        },
        "self_funded" => WorkStatus::SelfFunded {
            evidence_ref: evidence,
        },
        "partial" => WorkStatus::PartiallyCompensated {
            evidence_ref: evidence,
        },
        "compensated" => WorkStatus::Compensated {
            evidence_ref: evidence,
        },
        "disputed" => WorkStatus::Disputed {
            evidence_ref: evidence,
        },
        _ => WorkStatus::Unknown,
    }
}

#[component]
pub(super) fn CooperativeControls(
    host: HostHandle,
    current: EditorSnapshot,
    busy: Signal<bool>,
    mut status: Signal<String>,
    draft: Signal<String>,
) -> Element {
    let inspected = use_signal(|| None::<Inspected>);
    let mut contribution_id = use_signal(String::new);
    let mut description = use_signal(String::new);
    let mut minutes = use_signal(|| "0".to_string());
    let mut work_status = use_signal(|| "unknown".to_string());
    let mut work_evidence = use_signal(String::new);
    let mut request_id = use_signal(String::new);
    let mut request_evidence = use_signal(String::new);
    let mut selected_request = use_signal(String::new);
    let mut review_id = use_signal(String::new);
    let mut rationale = use_signal(String::new);
    let mut review_evidence = use_signal(String::new);
    let dirty = draft() != current.latest_source;
    let view = current_view(inspected, &current);
    let can_contribute =
        !busy() && !dirty && view.as_ref().is_some_and(|v| v.evidence.can_contribute);
    let can_review = !busy()
        && !dirty
        && view.as_ref().is_some_and(|v| {
            v.evidence.can_review
                && v.evidence.requests.iter().any(|request| {
                    request.id == selected_request() && request.target == v.evidence.target
                })
        });
    let inspect = {
        let host = host.clone();
        let current = current.clone();
        move |_| {
            command(
                WorkflowAction::Inspect,
                host.clone(),
                current.clone(),
                inspected,
                busy,
                status,
            )
        }
    };
    let contribution = {
        let host = host.clone();
        let current = current.clone();
        move |_| {
            if draft() != current.latest_source {
                status.set("Unsubmitted draft preserved. Submit or retain it separately before recording this exact source.".into());
                return;
            }
            let Some(view) = current_view(inspected, &current) else {
                status.set("Inspect current cooperative source first".into());
                return;
            };
            let Ok(effort) = minutes().parse::<u32>() else {
                status.set(
                    "Effort must be whole nonnegative minutes; no monetary valuation is inferred"
                        .into(),
                );
                return;
            };
            command(
                WorkflowAction::Contribution {
                    target: view.evidence.target,
                    id: contribution_id(),
                    description: description(),
                    minutes: effort,
                    work_status: paid_status(&work_status(), work_evidence()),
                },
                host.clone(),
                current.clone(),
                inspected,
                busy,
                status,
            )
        }
    };
    let request = {
        let host = host.clone();
        let current = current.clone();
        move |_| {
            if draft() != current.latest_source {
                status.set("Unsubmitted draft preserved. Source review requests never include unsubmitted draft text.".into());
                return;
            }
            if let Some(view) = current_view(inspected, &current) {
                command(
                    WorkflowAction::RequestReview {
                        target: view.evidence.target,
                        id: request_id(),
                        evidence: request_evidence(),
                    },
                    host.clone(),
                    current.clone(),
                    inspected,
                    busy,
                    status,
                )
            }
        }
    };
    let decide = {
        let host = host.clone();
        let current = current.clone();
        move |decision| {
            if draft() != current.latest_source {
                status.set("Review denied while an unsubmitted draft differs from the displayed source. Draft preserved.".into());
                return;
            }
            if let Some(view) = current_view(inspected, &current) {
                command(
                    WorkflowAction::Review {
                        target: view.evidence.target,
                        request_id: selected_request(),
                        id: review_id(),
                        decision,
                        rationale: rationale(),
                        evidence: review_evidence(),
                    },
                    host.clone(),
                    current.clone(),
                    inspected,
                    busy,
                    status,
                )
            }
        }
    };
    let mut approve = decide.clone();
    let mut changes = decide.clone();
    let mut reject = decide;
    rsx! {
        section { style:"border:1px solid #486173;padding:12px;display:flex;flex-direction:column;gap:8px;",
            h2 { "Cooperative source work" }
            button { disabled:busy(), onclick:inspect, "Inspect current source, history and review evidence" }
            if dirty { p { "Unsubmitted draft preserved. Workflow records bind the admitted source only; submit before requesting or deciding review." } }
            if let Some(view) = view {
                super::cooperative_workbench::CooperativeWorkbench { projection:super::cooperative_workbench::project(&view.evidence), status:status() }
                details { summary { "Actual source, predecessor comparison and retained history" }
                    p { "Difference from the existing undo predecessor. Its historical revision number is not retained. Package resources are outside this source comparison." }
                    pre { style:"max-height:240px;overflow:auto;white-space:pre-wrap;", "{cooperative_workflow::source_difference(view.previous_source_json.as_deref(), &view.current_source_json)}" }
                    h3 { "Complete current editable source projection" }
                    pre { style:"max-height:280px;overflow:auto;white-space:pre-wrap;", "{view.current_source_json}" }
                    h3 { "Current admitted Vibe cells" }
                    for (index, cell) in view.source_cells.iter().enumerate() {
                        p { "Cell {index + 1}" }
                        pre { style:"max-height:180px;overflow:auto;white-space:pre-wrap;", "{cell}" }
                    }
                    h3 { "Canonical source history" }
                    for entry in &view.source_history { p { "{entry}" } }
                }
                h3 { "Record contribution to this exact source" }
                input { value:"{contribution_id}", aria_label:"Contribution identifier", placeholder:"Contribution identifier", disabled:busy(), oninput:move |e|contribution_id.set(e.value()) }
                textarea { value:"{description}", aria_label:"Contribution description", placeholder:"Actual contribution", disabled:busy(), oninput:move |e|description.set(e.value()) }
                input { value:"{minutes}", aria_label:"Effort in minutes", disabled:busy(), oninput:move |e|minutes.set(e.value()) }
                select { value:"{work_status}", aria_label:"Actual paid or unpaid status", disabled:busy(), onchange:move |e|work_status.set(e.value()),
                    option { value:"unknown", "Paid status unknown" }
                    option { value:"unpaid", "Unpaid" }
                    option { value:"self_funded", "Self funded" }
                    option { value:"partial", "Partially compensated" }
                    option { value:"compensated", "Compensated" }
                    option { value:"disputed", "Compensation disputed" }
                }
                input { value:"{work_evidence}", aria_label:"Work status evidence reference", placeholder:"Status evidence reference (required except unknown)", disabled:busy(), oninput:move |e|work_evidence.set(e.value()) }
                button { disabled:!can_contribute, onclick:contribution, "Record actual effort and status" }
                p { "This records effort and asserted status evidence; it does not move funds, value effort, waive obligations or verify payment." }
                h3 { "Request source review" }
                input { value:"{request_id}", aria_label:"Source review request identifier", placeholder:"Request identifier", disabled:busy(), oninput:move |e|request_id.set(e.value()) }
                input { value:"{request_evidence}", aria_label:"Source review request evidence", placeholder:"Request evidence reference", disabled:busy(), oninput:move |e|request_evidence.set(e.value()) }
                button { disabled:!can_contribute, onclick:request, "Request review of displayed source revision" }
                h3 { "Distinct reviewer decision" }
                p { "Inspect the exact source and evidence first. Current review authority and a different reviewer subject are checked at commit. Software-agent decisions remain software review." }
                select { value:"{selected_request}", aria_label:"Exact source review request", disabled:busy(), onchange:move |e|selected_request.set(e.value()),
                    option { value:"", "Choose an exact current source request" }
                    for request in view.evidence.requests.iter().filter(|r|r.target == view.evidence.target) {
                        option { value:"{request.id}", "{request.id} · {request.requester_subject} · revision {request.target.revision}" }
                    }
                }
                input { value:"{review_id}", aria_label:"Source review decision identifier", placeholder:"Decision identifier", disabled:busy(), oninput:move |e|review_id.set(e.value()) }
                textarea { value:"{rationale}", aria_label:"Source review rationale", placeholder:"Observed findings and rationale", disabled:busy(), oninput:move |e|rationale.set(e.value()) }
                input { value:"{review_evidence}", aria_label:"Source review decision evidence", placeholder:"Actual inspection evidence reference", disabled:busy(), oninput:move |e|review_evidence.set(e.value()) }
                div { style:"display:flex;gap:8px;flex-wrap:wrap;",
                    button { disabled:!can_review, onclick:move |_|approve(ReviewDecision::Accepted), "Approve source only" }
                    button { disabled:!can_review, onclick:move |_|changes(ReviewDecision::ChangesRequested), "Request source changes" }
                    button { disabled:!can_review, onclick:move |_|reject(ReviewDecision::Rejected), "Reject source" }
                }
                details { summary { "Durable contributions and package write evidence" }
                    for record in &view.evidence.contributions {
                        p { "Contribution {record.contribution.id} · {record.contributor_subject} · {record.principal_kind:?} · revision {record.target.revision} · {record.work_status:?}" }
                    }
                    for record in &view.evidence.package_writes {
                        p { "Package write {record.filename} · SHA256 {hex_digest(&record.result_digest)} · {record.actor_subject} · source revision {record.source.revision}. Write audit only." }
                    }
                }
                p { "Whole HMC/master acceptance is unavailable here. It requires a separate qualified scope and exact package SHA256; no source decision approves resources, rights, a complete model or a high-grade master." }
            } else { p { "Inspect with the current admitted controller to enable explicit workflow actions." } }
        }
    }
}
