//! Native projection over the canonical package workflow, never a second ledger.
use crate::graphical::{
    controller::EditorSnapshot,
    jobs::{self, Action, Context, HostHandle},
    package::hex_digest,
    package_workflow::{PackageSelection, PackageWorkflowAction, PackageWorkflowView},
};
use dioxus::prelude::*;
use qualia_cooperative_core::authoring::{ReviewDecision, WorkStatus};

#[derive(Clone)]
struct BoundView {
    context: Context,
    view: PackageWorkflowView,
}
fn submit(
    host: HostHandle,
    current: EditorSnapshot,
    action: PackageWorkflowAction,
    mut panel: Signal<Option<BoundView>>,
    mut busy: Signal<bool>,
    mut status: Signal<String>,
) {
    if busy() {
        return;
    }
    if let Err(error) = action.validate() {
        status.set(error);
        return;
    }
    busy.set(true);
    panel.set(None);
    let origin = Context::from_snapshot(&current);
    let guard = host.clone();
    spawn(async move {
        match jobs::run(
            host,
            Action::PackageWorkflow {
                action,
                expected: current.source.clone(),
            },
            origin.clone(),
        )
        .await
        {
            Ok(reply) if jobs::completion_is_current(&guard, &current, &reply) => {
                panel.set(reply.package_workflow.map(|view| BoundView {
                    context: origin,
                    view,
                }));
                status.set(reply.message);
            }
            Ok(_) => status
                .set("Source/session/authority changed; inspect the saved package again.".into()),
            Err(error) => status.set(error),
        }
        busy.set(false);
    });
}

#[component]
pub(crate) fn PackageReviewControls(
    host: HostHandle,
    current: EditorSnapshot,
    filename: String,
    digest: [u8; 32],
    busy: Signal<bool>,
    status: Signal<String>,
) -> Element {
    let mut package_id = use_signal(String::new);
    let mut record_id = use_signal(String::new);
    let mut contribution_id = use_signal(String::new);
    let mut request_id = use_signal(String::new);
    let mut description = use_signal(String::new);
    let mut evidence = use_signal(String::new);
    let mut minutes = use_signal(|| "0".to_string());
    let mut work = use_signal(|| "unknown".to_string());
    let mut panel = use_signal(|| None::<BoundView>);
    let view = panel()
        .filter(|p| {
            p.context == Context::from_snapshot(&current)
                && current.publication_is_current()
                && p.view.publication_is_current()
                && p.view.source == current.source
                && p.view.evidence.target.filename == filename
                && p.view.evidence.target.package_digest == digest
                && p.view.evidence.target.package_id == package_id()
        })
        .map(|p| p.view);
    let scan = {
        let h = host.clone();
        let c = current.clone();
        let name = filename.clone();
        move |_| {
            submit(
                h.clone(),
                c.clone(),
                PackageWorkflowAction::Inspect(PackageSelection {
                    filename: name.clone(),
                    package_id: package_id(),
                    digest,
                }),
                panel,
                busy,
                status,
            );
        }
    };
    rsx! {
        section{style:"padding:12px;border:1px solid #667d8b;display:flex;flex-direction:column;gap:8px;",
            h3{"Saved package contribution and review"}
            p{"This workflow concerns the exact saved HMC. Source review and model acceptance remain separate."}
            p{style:"overflow-wrap:anywhere;","{filename} | SHA256 {hex_digest(&digest)}"}
            input{value:"{package_id}",aria_label:"Project-scoped package release identifier",placeholder:"Project-scoped package release identifier",disabled:busy(),oninput:move|e|{package_id.set(e.value());panel.set(None);}}
            button{disabled:busy()||package_id().trim().is_empty(),onclick:scan,"Inspect saved package workflow"}
            if let Some(view)=view {
                p{"Current actor {view.evidence.principal_subject} | {view.evidence.principal_kind:?}"}
                p{"Package producer {view.evidence.target.producer_subject}; source author {view.evidence.target.source.author_subject:?}."}
                p{"Known destination overwrite lineage complete: {view.evidence.known_destination_lineage_complete}."}
                p{"{view.evidence.contribution_rights_scope}"}
                p{"An explicit exact-package grant is required for review. Known contributing actors cannot review their own work."}
                input{value:"{record_id}",aria_label:"Package workflow record identifier",placeholder:"New contribution, request or decision record identifier",disabled:busy(),oninput:move|e|record_id.set(e.value())}
                textarea{value:"{description}",aria_label:"Contribution description or package review rationale",placeholder:"Contribution description / review rationale",disabled:busy(),oninput:move|e|description.set(e.value())}
                input{value:"{evidence}",aria_label:"Package workflow evidence reference",placeholder:"Evidence reference",disabled:busy(),oninput:move|e|evidence.set(e.value())}
                input{value:"{minutes}",aria_label:"Contribution effort in minutes",disabled:busy(),oninput:move|e|minutes.set(e.value())}
                select{value:"{work}",aria_label:"Actual contribution compensation status",disabled:busy(),onchange:move|e|work.set(e.value()),
                    option{value:"unknown","Compensation status unknown"}
                    option{value:"unpaid","Unpaid"} option{value:"self_funded","Self-funded"}
                    option{value:"partial","Partially compensated"} option{value:"compensated","Compensated"}
                    option{value:"disputed","Disputed"}
                }
                {let h=host.clone();let c=current.clone();let t=view.evidence.target.clone();let generation=view.evidence.authority_generation;
                    rsx!{button{disabled:busy()||!view.evidence.can_contribute,onclick:move |_|{
                        let count=match minutes().parse::<u32>(){Ok(n)=>n,Err(_)=>{{let mut msg=status;msg.set("Effort requires whole nonnegative minutes".into());}return;}};
                        let status_value=match work().as_str(){
                            "unpaid"=>WorkStatus::Unpaid{evidence_ref:evidence()},
                            "self_funded"=>WorkStatus::SelfFunded{evidence_ref:evidence()},
                            "partial"=>WorkStatus::PartiallyCompensated{evidence_ref:evidence()},
                            "compensated"=>WorkStatus::Compensated{evidence_ref:evidence()},
                            "disputed"=>WorkStatus::Disputed{evidence_ref:evidence()},_=>WorkStatus::Unknown};
                        submit(h.clone(),c.clone(),PackageWorkflowAction::Contribution{target:t.clone(),generation,id:record_id(),description:description(),minutes:count,status:status_value},panel,busy,status);
                    },"Record package contribution"}}
                }
                input{value:"{contribution_id}",aria_label:"Exact package contribution identifier",placeholder:"Existing exact-package contribution identifier",disabled:busy(),oninput:move|e|contribution_id.set(e.value())}
                {let h=host.clone();let c=current.clone();let t=view.evidence.target.clone();let generation=view.evidence.authority_generation;
                    rsx!{button{disabled:busy()||!view.evidence.can_contribute,onclick:move |_|submit(h.clone(),c.clone(),PackageWorkflowAction::Request{target:t.clone(),generation,contribution_id:contribution_id(),id:record_id(),evidence:evidence()},panel,busy,status),"Request review of this package"}}
                }
                input{value:"{request_id}",aria_label:"Exact package review request identifier",placeholder:"Existing package review request identifier",disabled:busy(),oninput:move|e|request_id.set(e.value())}
                for (label,decision) in [("Accept scoped package review",ReviewDecision::Accepted),("Request package changes",ReviewDecision::ChangesRequested),("Reject package review",ReviewDecision::Rejected)] {
                    {let h=host.clone();let c=current.clone();let t=view.evidence.target.clone();let generation=view.evidence.authority_generation;
                        rsx!{button{disabled:busy()||!view.evidence.can_review,onclick:move |_|submit(h.clone(),c.clone(),PackageWorkflowAction::Decide{target:t.clone(),generation,request_id:request_id(),id:record_id(),decision,rationale:description(),evidence:evidence()},panel,busy,status),"{label}"}}
                    }
                }
                for contribution in view.evidence.contributions.iter().filter(|r|r.target==view.evidence.target) {
                    p{style:"overflow-wrap:anywhere;","Contribution {contribution.contribution.id} | {contribution.actor.subject} | {contribution.work_status:?}"}
                }
                for request in view.evidence.requests.iter().filter(|r|r.target==view.evidence.target) {
                    p{style:"overflow-wrap:anywhere;","Request {request.id} | contribution {request.contribution_id} | decisions {request.decision_ids:?}"}
                }
                if !view.evidence.prior_contributions.is_empty() {
                    h4 { "Other recorded package contributions" }
                    p { "Historical qualified records; these are not decisions accepting the selected package, and full derivation/rights ancestry remains unassessed." }
                    for prior in view.evidence.prior_contributions.iter() {
                        p { style:"overflow-wrap:anywhere;", "{prior.contribution.id} · {prior.actor.subject} · {prior.target.filename} · source revision {prior.target.source.revision} · package SHA256 {hex_digest(&prior.target.package_digest)} · {prior.work_status:?}" }
                    }
                }
                for review in view.evidence.reviews.iter().filter(|r|r.target==view.evidence.target) {
                    p{style:"overflow-wrap:anywhere;","Decision {review.id} | {review.actor.subject} | {review.actor.principal_kind:?} | {review.decision:?}: {review.rationale}"}
                }
            }
        }
    }
}
