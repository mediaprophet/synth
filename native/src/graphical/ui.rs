//! Native Aether projection of actual Poet source/container commands.
use super::controller::EditorSnapshot;
use super::jobs::{self, Action, Context, HostHandle};
use dioxus::prelude::*;
fn dispatch(
    action: Action,
    host: HostHandle,
    mut view: Signal<EditorSnapshot>,
    mut draft: Signal<String>,
    mut status: Signal<String>,
    mut busy: Signal<bool>,
    mut image: Signal<Option<String>>,
    draft_buffer: Option<super::draft_buffer::DraftBuffer>,
    replace_draft: bool,
) {
    if busy() {
        return;
    }
    let preserve_draft = replace_draft && draft() != view().latest_source;
    let capture = if replace_draft && !preserve_draft {
        match super::draft_buffer::DraftCapture::capture(draft(), draft_buffer.as_ref()) {
            Ok(capture) => Some(capture),
            Err(error) => {
                status.set(error);
                return;
            }
        }
    } else {
        None
    };
    busy.set(true);
    status.set("Native command queued; source and session are checked before execution.".into());
    let origin = Context::from_snapshot(&view());
    let guard = host.clone();
    spawn(async move {
        match super::native_dispatch::run(host, action, origin).await {
            Ok(reply) if jobs::completion_is_current(&guard, &view(), &reply) => {
                let mut retained_local_draft = preserve_draft;
                let mut context_changed = false;
                if let Some(snapshot) = reply.snapshot {
                    context_changed = snapshot.source != view().source
                        || snapshot.execution_epoch != view().execution_epoch
                        || snapshot.reconciliation_required;
                    if let Some(capture) = &capture {
                        if let Some(text) = capture.accept(
                            reply.operation_succeeded,
                            &draft(),
                            &snapshot.latest_source,
                            draft_buffer.as_ref(),
                        ) {
                            draft.set(text);
                        } else if reply.operation_succeeded {
                            retained_local_draft = true;
                        }
                    }
                    view.set(snapshot);
                }
                if reply.image_uri.is_some() || context_changed {
                    image.set(reply.image_uri);
                }
                status.set(if retained_local_draft {
                    format!("{}; unsubmitted draft preserved", reply.message)
                } else {
                    reply.message
                });
            }
            Ok(_) => {
                image.set(None);
                status.set("Source/session could not be revalidated. Refresh source; no late frame installed.".into());
            }
            Err(error) => {
                // A lost worker reply may follow committed source changes.
                image.set(None);
                status.set(error);
            }
        }
        busy.set(false);
    });
}

impl PartialEq for EditorSnapshot {
    fn eq(&self, other: &Self) -> bool {
        self.source == other.source
            && self.session_revision == other.session_revision
            && self.execution_epoch == other.execution_epoch
            && self.rebuild_supported == other.rebuild_supported
            && self.reconciliation_required == other.reconciliation_required
            && self.title == other.title
            && self.latest_source == other.latest_source
            && self.material_edit_id == other.material_edit_id
            && self.checkpoint_supported == other.checkpoint_supported
            && self.authorisation_stamp == other.authorisation_stamp
    }
}

#[component]
pub(super) fn NativeStudio(
    host: HostHandle,
    initial: EditorSnapshot,
    initial_draft: String,
    #[props(default)] draft_buffer: Option<super::draft_buffer::DraftBuffer>,
    #[props(default)] shared_busy: Option<Signal<bool>>,
) -> Element {
    let view = use_signal(|| initial);
    let mut draft = use_signal(|| initial_draft);
    let mut status =
        use_signal(|| "Source ready. Check or submit an explicit Vibe command.".to_string());
    let own_busy = use_signal(|| false);
    let busy = shared_busy.unwrap_or(own_busy);
    let image = use_signal(|| None::<String>);
    let selected = use_signal(String::new);
    // Shared user-entered rig resource selection; these values grant no access.
    let rig_filename = use_signal(|| "working.hmc".to_string());
    let rig_package_digest = use_signal(String::new);
    let rig_resource_key = use_signal(String::new);
    let rig_resource_digest = use_signal(String::new);
    let rig_ready_source = use_signal(|| None::<(jobs::Context, String)>);
    let current = view();
    let workspace =
        super::poet_workspace::WorkspaceProjection::for_source(&current.artifact_id).ok();
    let mut restore_context = use_signal(|| None::<jobs::Context>);
    let mut checkpoint_name = use_signal(|| "poet-checkpoint.hmc".to_string());
    let save = {
        let host = host.clone();
        let draft_buffer = draft_buffer.clone();
        move |_| {
            dispatch(
                Action::Save {
                    name: checkpoint_name(),
                    expected: view().source,
                },
                host.clone(),
                view,
                draft,
                status,
                busy,
                image,
                draft_buffer.clone(),
                false,
            )
        }
    };
    let check = {
        let host = host.clone();
        let draft_buffer = draft_buffer.clone();
        move |_| {
            dispatch(
                Action::Check(draft()),
                host.clone(),
                view,
                draft,
                status,
                busy,
                image,
                draft_buffer.clone(),
                false,
            )
        }
    };
    let submit = {
        let host = host.clone();
        let draft_buffer = draft_buffer.clone();
        move |_| {
            dispatch(
                Action::Submit {
                    source: draft(),
                    expected: view().source,
                },
                host.clone(),
                view,
                draft,
                status,
                busy,
                image,
                draft_buffer.clone(),
                false,
            )
        }
    };
    let undo = {
        let host = host.clone();
        let draft_buffer = draft_buffer.clone();
        move |_| {
            dispatch(
                Action::Undo(view().source),
                host.clone(),
                view,
                draft,
                status,
                busy,
                image,
                draft_buffer.clone(),
                true,
            )
        }
    };
    let redo = {
        let host = host.clone();
        let draft_buffer = draft_buffer.clone();
        move |_| {
            dispatch(
                Action::Redo(view().source),
                host.clone(),
                view,
                draft,
                status,
                busy,
                image,
                draft_buffer.clone(),
                true,
            )
        }
    };
    let refresh = {
        let host = host.clone();
        let draft_buffer = draft_buffer.clone();
        move |_| {
            dispatch(
                Action::Refresh,
                host.clone(),
                view,
                draft,
                status,
                busy,
                image,
                draft_buffer.clone(),
                true,
            )
        }
    };
    let rebuild = {
        let host = host.clone();
        let draft_buffer = draft_buffer.clone();
        move |_| {
            dispatch(
                Action::Rebuild(view().source),
                host.clone(),
                view,
                draft,
                status,
                busy,
                image,
                draft_buffer.clone(),
                false,
            )
        }
    };
    let input_buffer = draft_buffer.clone();
    let material_buffer = draft_buffer.clone();
    let restore_buffer = draft_buffer.clone();
    rsx! {
        div { style: "height:100vh;box-sizing:border-box;overflow:auto;background:#111820;color:#e8eef4;display:flex;font-family:sans-serif;",
            aside { style: "width:160px;flex-shrink:0;box-sizing:border-box;padding:18px;background:#18232e;",
                h2 { "Poet" }
                p { "Tool-Chest" }
                if let Some(workspace)=&workspace {
                    strong { "{workspace.manifold.label}" }
                    p { "Construct · {workspace.construct.honesty}" }
                    for container in &workspace.manifold.containers {
                        p { style:"font-size:12px;", "{container.title} · {container.honesty}" }
                    }
                }
                p { "Selected native 3D projection of Poet. Other tools retain their existing definitions and port status." }
            }
            main { style: "flex:1;min-width:0;min-height:0;box-sizing:border-box;padding:16px;display:flex;flex-direction:column;gap:10px;",
                header {
                    h1 { style: "margin:0 0 8px;font-size:24px;", "{current.title}" }
                    p { style: "margin:0;", "{current.artifact_id} · source revision {current.source_revision} · Vibe session {current.session_revision}" }
                }
                div { style: "display:flex;gap:12px;flex:1;min-width:0;min-height:0;align-items:flex-start;",
                    section { style: "flex:1 1 0;min-width:0;display:flex;flex-direction:column;gap:8px;",
                        h2 { style: "margin:0;font-size:20px;", "Vibe source draft" }
                        textarea { value: "{draft}", disabled: busy(), aria_label: "Vibe source draft",
                            style: "height:calc(100vh - 360px);min-height:220px;max-height:560px;width:100%;min-width:0;box-sizing:border-box;background:#081018;color:#e8eef4;padding:12px;font-family:monospace;font-size:15px;",
                            oninput: move |event| {let text=event.value();if let Some(buffer)=&input_buffer{buffer.set(text.clone());}draft.set(text);},
                        }
                        details {
                            summary { "Load current accepted source into the local draft" }
                            textarea { value:"{current.latest_source}",readonly:true,aria_label:"Accepted source preview for local draft replacement",style:"width:100%;height:140px;box-sizing:border-box;" }
                            label {
                                input { r#type:"checkbox",disabled:busy(),checked:restore_context().as_ref()==Some(&jobs::Context::from_snapshot(&current)),
                                    onchange:move|event|restore_context.set(if event.checked(){Some(jobs::Context::from_snapshot(&view()))}else{None}) }
                                "Replace my local draft with this accepted source; unsubmitted local text will be replaced."
                            }
                            button { disabled:busy()||restore_context().as_ref()!=Some(&jobs::Context::from_snapshot(&current)),onclick:move |_|{
                                let snapshot=view();let confirmed=restore_context().as_ref()==Some(&jobs::Context::from_snapshot(&snapshot));
                                match super::draft_buffer::accepted_source_for_draft(&snapshot,confirmed){
                                    Ok(text)=>{if let Some(buffer)=&restore_buffer{buffer.set(text.clone());}draft.set(text);status.set("Local draft replaced with current accepted source. No accepted source or review decision changed.".into());restore_context.set(None);}
                                    Err(error)=>status.set(error),
                                }
                            },"Replace local draft with accepted source" }
                        }
                        div { style: "display:flex;gap:10px;flex-wrap:wrap;",
                            button { disabled: busy() || current.reconciliation_required, onclick: move |event| check(event), "Check" }
                            button { disabled: busy() || current.reconciliation_required, onclick: move |event| submit(event), "Submit source" }
                            button { disabled: busy(), onclick: move |event| undo(event), "Undo source" }
                            button { disabled: busy(), onclick: move |event| redo(event), "Redo source" }
                            button { disabled: busy(), onclick: move |event| refresh(event), "Refresh source" }
                            if current.rebuild_supported {
                                button { disabled: busy() || !current.reconciliation_required,
                                    onclick: move |event| rebuild(event), "Rebuild execution session" }
                            }
                        }
                        if current.reconciliation_required {
                            if current.rebuild_supported {
                                p { "Source is preserved. Rebuild the admitted geometry execution session before another submission. No source is resubmitted automatically." }
                            } else {
                                p { "Source is preserved. The native host must reconcile the execution session before another submission." }
                            }
                        }
                        if current.checkpoint_supported {
                            div { style: "display:flex;gap:8px;flex-wrap:wrap;",
                                input { value: "{checkpoint_name}", aria_label: "HMC checkpoint filename",
                                    disabled: busy(), oninput: move |event| checkpoint_name.set(event.value()) }
                                button { disabled: busy(), onclick: move |event| save(event), "Save accepted source as new HMC" }
                            }
                        }
                        p { "{status}" }
                        super::numeric_sheet_controls::NumericSheetControls { host:host.clone(),view,draft,status,image,busy,buffer:draft_buffer.clone() }
                        super::material_controls::MaterialControls { current: current.clone(), busy: busy(), draft, status, draft_buffer:material_buffer }
                        super::cooperative_controls::CooperativeControls { host:host.clone(),current:current.clone(),busy,status,draft }
                        if current.checkpoint_supported {
                            super::package_controls::PackageControls { host: host.clone(), current: current.clone(), busy, status }
                            super::package_batch_controls::PackageBatchControls { host: host.clone(), current: current.clone(), busy, status }
                        }
                    }
                    section { style: "width:512px;max-width:45%;min-width:0;flex-shrink:0;display:flex;flex-direction:column;gap:8px;",
                        h2 { style: "margin:0;font-size:20px;", "Manifold preview" }
                        super::viewport_controls::ViewportControls{host:host.clone(),view,image,busy,status,selected,rig_filename,rig_package_digest,rig_resource_key,rig_resource_digest,rig_ready_source}
                        p { "Accepted source revision {current.source_revision} · Vibe session {current.session_revision}" }
                        p { "Native assembly/material working preview. Master quality, source rights and human review remain separate." }
                        super::authoring::controls::AuthoringControls {host:host.clone(),view,draft,status,image,busy,selected,rig_filename,rig_package_digest,rig_resource_key,rig_resource_digest,rig_ready_source,buffer:draft_buffer.clone()}
                    }
                }
            }
        }
    }
}
