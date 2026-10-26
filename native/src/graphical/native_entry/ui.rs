//! Native operator gestures. No serialized request is an adoption or approval.
use super::{
    types::*,
    worker::{self, LaunchHandle},
};
use dioxus::prelude::*;
use qualia_cooperative_core::authoring::PrincipalKind;
use qualia_cooperative_core::projects::ProjectRole;
use qualia_core_db::poet_host::native_authority::enrollment::EnrollmentStatus;

pub(super) fn dispatch(
    handle: LaunchHandle,
    action: LaunchAction,
    mut view: Signal<LaunchView>,
    mut status: Signal<String>,
    mut busy: Signal<bool>,
) {
    if busy() {
        return;
    }
    busy.set(true);
    status.set("Native project operation queued; current authority will be checked.".into());
    spawn(async move {
        match worker::run(handle, action).await {
            Ok(reply) => {
                if let Some(next) = reply.view {
                    view.set(next)
                }
                status.set(reply.message)
            }
            Err(error) => status.set(error),
        }
        busy.set(false);
    });
}
#[component]
pub(super) fn NativeProjectShell(handle: LaunchHandle, initial: LaunchView) -> Element {
    let view = use_signal(|| initial);
    let busy = use_signal(|| false);
    let status = use_signal(|| {
        "Choose or create a native project. No key or project authority has been adopted by opening this window.".to_string()
    });
    let mut name = use_signal(String::new);
    let mut profile = use_signal(String::new);
    let mut evidence = use_signal(String::new);
    let mut human_evidence = use_signal(String::new);
    let mut filename = use_signal(|| "working.hmc".to_string());
    let mut person = use_signal(|| false);
    let mut consent = use_signal(|| false);
    let current = view();
    let project_name = current
        .project_name
        .clone()
        .unwrap_or_else(|| "unavailable".to_string());
    let active_role_summary = current.current_subject.as_deref().and_then(|subject| {
        current.members.iter().find(|member| member.subject == subject).map(|member| {
            format!(
                "Project role: {:?} · principal kind: {:?} · reviewer grant active: {} · membership revoked: {}",
                member.role, member.kind, member.has_review, member.revoked
            )
        })
    });
    let capacity = if person() {
        PrincipalKind::Human
    } else {
        PrincipalKind::SoftwareAgent
    };
    rsx! {
        div{style:"height:100vh;overflow:auto;background:#111820;color:#e8eef4;font-family:sans-serif;",
            header{style:"padding:18px;display:flex;flex-direction:column;gap:8px;border-bottom:1px solid #465463;",
                h1{style:"margin:0;","Poet · Cooperative Project"}
                p{style:"margin:0;color:#bacbd9;",{crate::graphical::native_authoring_profile::selected().label()}}
                if let Some(project)=&current.project_name{
                    h2{"{project}"}
                    if let Some(fingerprint)=&current.authority_fingerprint{
                        p{style:"overflow-wrap:anywhere;","Project authority public-key fingerprint: {fingerprint}"}
                        if !current.adopted{
                            label{input{r#type:"checkbox",checked:consent(),disabled:busy(),onchange:move|e|consent.set(e.checked())}
                                "I have checked this authority fingerprint and choose to adopt it for this project."}
                            {let h=handle.clone();let fingerprint=fingerprint.clone();
                            rsx!{button{disabled:busy()||!consent(),onclick:move |_|dispatch(h.clone(),LaunchAction::Adopt{fingerprint:fingerprint.clone()},view,status,busy),"Adopt this project authority"}}}
                        }
                    }
                }else{
                    input{value:"{name}",aria_label:"New project name",placeholder:"New project name",disabled:busy(),oninput:move|e|name.set(e.value())}
                    label{input{r#type:"checkbox",checked:consent(),disabled:busy(),onchange:move|e|consent.set(e.checked())}
                        "Create a new local project authority and explicitly admit this controller as its steward. This grants no authority in other projects."}
                }
                div{style:"display:flex;gap:8px;flex-wrap:wrap;",
                    input{value:"{profile}",aria_label:"Local secure controller profile",placeholder:"Local controller profile label",disabled:busy(),oninput:move|e|profile.set(e.value())}
                    label{input{r#type:"checkbox",checked:person(),disabled:busy(),onchange:move|e|person.set(e.checked())}
                        "Person capacity requested (self-declared; project verification is separate)"}
                    input{value:"{evidence}",aria_label:"Decision or enrollment evidence reference",placeholder:"Consent / enrollment / decision evidence reference",disabled:busy(),oninput:move|e|evidence.set(e.value())}
                }
                if current.project_name.is_none(){
                    {let h=handle.clone();rsx!{button{disabled:busy()||!consent()||name().trim().is_empty()||profile().trim().is_empty()||evidence().trim().is_empty(),
                        onclick:move |_|dispatch(h.clone(),LaunchAction::CreateProject{name:name(),profile:profile(),kind:capacity,evidence:evidence()},view,status,busy),"Create project and secure steward profile"}}}
                }else if current.adopted{
                    div{style:"display:flex;gap:8px;flex-wrap:wrap;",
                        {let h=handle.clone();rsx!{button{disabled:busy()||profile().trim().is_empty(),onclick:move |_|dispatch(h.clone(),LaunchAction::CreateProfile{profile:profile(),kind:capacity},view,status,busy),"Create this local secure controller"}}}
                        {let h=handle.clone();rsx!{button{disabled:busy()||profile().trim().is_empty()||evidence().trim().is_empty(),onclick:move |_|dispatch(h.clone(),LaunchAction::SubmitEnrollment{profile:profile(),evidence:evidence()},view,status,busy),"Request contributor enrollment"}}}
                        input{value:"{filename}",aria_label:"Working HMC filename",disabled:busy(),oninput:move|e|filename.set(e.value())}
                        {let h=handle.clone();rsx!{button{disabled:busy()||profile().trim().is_empty(),onclick:move |_|dispatch(h.clone(),LaunchAction::Login{profile:profile(),filename:filename()},view,status,busy),"Unlock and sign in"}}}
                        if current.current_subject.is_some(){
                            {let h=handle.clone();rsx!{button{disabled:busy(),onclick:move |_|dispatch(h.clone(),LaunchAction::Logout,view,status,busy),"Lock and switch contributor"}}}
                        }
                    }
                }
                if let Some(subject)=&current.current_subject{p{style:"overflow-wrap:anywhere;","Current admitted subject: {subject}"}}
                p{"{status}"}
                p{"Controller keys stay in the native protected store. Enrollment does not imply reviewer authority, payment, or acceptance of a model."}
            }
            if !current.pending.is_empty() || !current.members.is_empty(){
                section{style:"padding:18px;border-bottom:1px solid #465463;",
                    h2{"Steward enrollment decisions"}
                    input{value:"{human_evidence}",aria_label:"Human verification evidence reference",placeholder:"Human verification evidence reference, only for a person",disabled:busy(),oninput:move|e|human_evidence.set(e.value())}
                    p{"Approval and reviewer grants change signed admission and lock the current session. Sign in again afterward."}
                    for pending in current.pending{
                        {let id=pending.request.id();let subject=pending.request.candidate.subject.clone();let human=pending.request.candidate.kind==PrincipalKind::Human;let pending_now=pending.status==EnrollmentStatus::Pending;
                            let approve_handle=handle.clone();let decline_handle=handle.clone();let decline_id=id.clone();
                            rsx!{div{style:"padding:10px;border:1px solid #465463;overflow-wrap:anywhere;",
                                p{"Request {id} · {subject} · {pending.status:?}"}
                                p{"Requested capacity {pending.request.candidate.kind:?}; role {pending.request.candidate.membership.role:?}"}
                                button{disabled:busy()||!pending_now||evidence().trim().is_empty()||(human&&human_evidence().trim().is_empty()),onclick:move |_|dispatch(approve_handle.clone(),LaunchAction::Approve{id:id.clone(),evidence:evidence(),human_evidence:if human{Some(human_evidence())}else{None}},view,status,busy),"Approve this enrollment"}
                                button{disabled:busy()||!pending_now||evidence().trim().is_empty(),onclick:move |_|dispatch(decline_handle.clone(),LaunchAction::Decline{id:decline_id.clone(),evidence:evidence()},view,status,busy),"Decline this request"}
                            }}
                        }
                    }
                    for member in current.members{
                        {let review_handle=handle.clone();let revoke_handle=handle.clone();let review_subject=member.subject.clone();let revoke_subject=member.subject.clone();
                            rsx!{div{style:"padding:10px;overflow-wrap:anywhere;",
                                p{"{member.subject} · {member.kind:?} · {member.role:?} · reviewer {member.has_review} · revoked {member.revoked}"}
                                button{disabled:busy()||member.revoked||member.has_review||evidence().trim().is_empty(),onclick:move |_|dispatch(review_handle.clone(),LaunchAction::GrantReview{subject:review_subject.clone(),evidence:evidence()},view,status,busy),"Grant explicit source-review authority"}
                                button{disabled:busy()||member.revoked||member.role==ProjectRole::Steward||evidence().trim().is_empty(),onclick:move |_|dispatch(revoke_handle.clone(),LaunchAction::Revoke{subject:revoke_subject.clone(),evidence:evidence()},view,status,busy),"Revoke this enrollment"}
                            }}
                        }
                    }
                    p{"Steward key rotation and recovery are separate operations; this panel does not revoke a project steward."}
                }
            }
            if current.current_subject.is_some(){
                super::package_grant_controls::PackageGrantControls{handle:handle.clone(),view,status,busy}
            }
            if let Some(lease)=current.studio{
                section{style:"padding:14px;border-bottom:1px solid #465463;",
                    h2{"Active authoring context"}
                    p{"Project: {project_name}"}
                    if let Some(subject)=&current.current_subject{
                        p{style:"overflow-wrap:anywhere;","Signed-in actor: {subject}"}
                        if let Some(role_summary)=&active_role_summary{
                            p{"{role_summary}"}
                        }else{
                            p{"Project role: no active membership record is available for this subject."}
                        }
                    }else{
                        p{"No signed-in actor is available; inspect native session status before cooperative work."}
                    }
                }
                if let Ok(text)=lease.draft_buffer.read(){
                    crate::graphical::ui::NativeStudio{key:"{lease.epoch}",host:lease.host,initial:lease.initial,initial_draft:text,draft_buffer:Some(lease.draft_buffer),shared_busy:Some(busy)}
                }else{
                    p{"Your private local draft could not be reopened. Keep this project open for reconciliation; no draft was discarded."}
                }
            }
        }
    }
}
