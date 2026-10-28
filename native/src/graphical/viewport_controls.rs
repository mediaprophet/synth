//! Native retained viewport controls. Gestures are view-only derived operations.
use super::{
    controller::EditorSnapshot,
    jobs::{self, Action, Context, HostHandle},
    native_dispatch,
    viewport::ViewportCommand,
    viewport_contract::ViewportState,
};
use crate::graphical::viewport_input::{client_to_attachment, delta_to_attachment, ClientRect};
use dioxus::{
    html::{
        events::{MountedData, MouseData, WheelData},
        geometry::WheelDelta,
        input_data::MouseButton,
        point_interaction::{InteractionLocation, PointerInteraction},
    },
    prelude::*,
};
use std::rc::Rc;

const ATTACHMENT_SIDE: u32 = 512;

fn rig_resource_request(filename: &str, package_digest: &str, key: &str, resource_digest: &str)
    -> Result<super::checkpoints::ResourceInspectionRequest, String> {
    let request = super::checkpoints::ResourceInspectionRequest {
        filename: filename.to_string(), package_digest: super::package::parse_digest(package_digest.trim())?,
        key: key.to_string(), resource_digest: super::package::parse_digest(resource_digest.trim())?,
    };
    request.validate_bounds()?;
    Ok(request)
}

fn rig_preview_request(filename: &str, package_digest: &str, key: &str, resource_digest: &str,
    ready: Option<&(Context, String)>, current: &Context, selected: &str,
) -> Result<super::checkpoints::ResourceInspectionRequest, String> {
    if !ready.is_some_and(|(origin, part)| origin == current && part == selected && !part.is_empty())
        || current.reconciliation_required || !current.execution_epoch.is_current() {
        return Err("Confirm the rig resource for the current source and selected part before preview.".into());
    }
    rig_resource_request(filename, package_digest, key, resource_digest)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DragMode {
    Orbit,
    Pan,
}

#[derive(Clone)]
struct DragState {
    mode: DragMode,
    last_x: f64,
    last_y: f64,
    start_x: f64,
    start_y: f64,
    moved: bool,
    /// Frame visible when the pointer went down. Clicks cannot be retargeted to
    /// a newer pose or a restored scene while awaiting asynchronous layout.
    start_frame_id: Option<u64>,
    context: Context,
}

fn click_frame_is_current(captured: Option<u64>, state: &ViewportState, context: &Context) -> bool {
    captured == Some(state.frame_id)
        && state.source == context.source
        && state.execution_epoch.as_ref() == Some(&context.execution_epoch)
}

#[derive(Clone)]
struct PendingCommand {
    command: ViewportCommand,
    context: Context,
}

fn add_commands(previous: ViewportCommand, next: ViewportCommand) -> ViewportCommand {
    match (previous, next) {
        (ViewportCommand::Orbit { dx: ax, dy: ay }, ViewportCommand::Orbit { dx: bx, dy: by }) => {
            ViewportCommand::Orbit {
                dx: (ax + bx).clamp(-8192.0, 8192.0),
                dy: (ay + by).clamp(-8192.0, 8192.0),
            }
        }
        (ViewportCommand::Pan { dx: ax, dy: ay }, ViewportCommand::Pan { dx: bx, dy: by }) => {
            ViewportCommand::Pan {
                dx: (ax + bx).clamp(-8192.0, 8192.0),
                dy: (ay + by).clamp(-8192.0, 8192.0),
            }
        }
        (ViewportCommand::Zoom { steps: a }, ViewportCommand::Zoom { steps: b }) => {
            ViewportCommand::Zoom {
                steps: (a + b).clamp(-64.0, 64.0),
            }
        }
        (_, next) => next,
    }
}

fn clear_frame(
    mut image: Signal<Option<String>>,
    mut selected: Signal<String>,
    mut viewport_state: Signal<Option<ViewportState>>,
) {
    image.set(None);
    selected.set(String::new());
    viewport_state.set(None);
}

fn enqueue(
    command: ViewportCommand,
    context: Context,
    host: HostHandle,
    view: Signal<EditorSnapshot>,
    image: Signal<Option<String>>,
    busy: Signal<bool>,
    mut status: Signal<String>,
    selected: Signal<String>,
    viewport_state: Signal<Option<ViewportState>>,
    mut pending: Signal<Option<PendingCommand>>,
) {
    let current = view();
    if Context::from_snapshot(&current) != context || !current.publication_is_current() {
        status.set("Viewport gesture dropped because source, session, or actor authority changed. Refresh the viewport for the current source.".into());
        return;
    }
    if busy() {
        {
            let mut slot = pending.write();
            *slot = Some(match slot.take() {
                Some(previous) if previous.context == context => PendingCommand {
                    command: add_commands(previous.command, command),
                    context,
                },
                _ => PendingCommand { command, context },
            });
        }
        return;
    }
    run_command(
        command,
        context,
        host,
        current,
        view,
        image,
        busy,
        status,
        selected,
        viewport_state,
    );
}

fn run_command(
    command: ViewportCommand,
    context: Context,
    host: HostHandle,
    current: EditorSnapshot,
    view: Signal<EditorSnapshot>,
    mut image: Signal<Option<String>>,
    mut busy: Signal<bool>,
    mut status: Signal<String>,
    mut selected: Signal<String>,
    mut viewport_state: Signal<Option<ViewportState>>,
) {
    if busy() {
        status.set("Viewport command is waiting for the current native operation.".into());
        return;
    }
    if Context::from_snapshot(&current) != context || !current.publication_is_current() {
        status.set(
            "Viewport command dropped because its native source context is no longer current."
                .into(),
        );
        return;
    }
    busy.set(true);
    let guard = host.clone();
    spawn(async move {
        let expected = current.source.clone();
        match native_dispatch::run(
            host,
            Action::Viewport {
                request: command,
                expected,
            },
            context.clone(),
        )
        .await
        {
            Ok(reply) if jobs::completion_is_current(&guard, &current, &reply) => {
                let current_view = view();
                if Context::from_snapshot(&current_view) != context {
                    clear_frame(image, selected, viewport_state);
                    status.set("Viewport result withheld because the source/session/actor context changed.".into());
                } else if !reply.operation_succeeded {
                    clear_frame(image, selected, viewport_state);
                    status.set(reply.message);
                } else if let (Some(snapshot), Some(state), Some(uri)) =
                    (reply.snapshot.as_ref(), reply.viewport, reply.image_uri)
                {
                    if state.source == snapshot.source
                        && state.execution_epoch.as_ref() == Some(&snapshot.execution_epoch)
                        && snapshot.source == current.source
                    {
                        selected.set(state.selected_part.clone().unwrap_or_default());
                        viewport_state.set(Some(state));
                        image.set(Some(uri));
                        status.set(reply.message);
                    } else {
                        clear_frame(image, selected, viewport_state);
                        status.set("Viewport frame metadata does not match the current source and execution epoch; frame withheld.".into());
                    }
                } else {
                    clear_frame(image, selected, viewport_state);
                    status.set("Native viewport returned no current frame/state; frame and pick metadata cleared.".into());
                }
            }
            Ok(_) => {
                clear_frame(image, selected, viewport_state);
                status.set("Viewport completion is stale; frame and pick metadata cleared. Refresh the current viewport.".into())
            }
            Err(error) => {
                clear_frame(image, selected, viewport_state);
                status.set(error)
            }
        }
        busy.set(false);
    });
}

async fn client_rect(element: Rc<MountedData>) -> Option<ClientRect> {
    let rect = element.get_client_rect().await.ok()?;
    Some(ClientRect {
        x: rect.origin.x,
        y: rect.origin.y,
        width: rect.size.width,
        height: rect.size.height,
    })
}

fn wheel_steps(delta: WheelDelta) -> f32 {
    let amount = match delta {
        WheelDelta::Pixels(value) => -value.y / 100.0,
        WheelDelta::Lines(value) => -value.y * 0.5,
        WheelDelta::Pages(value) => -value.y * 3.0,
    };
    if amount.is_finite() {
        amount.clamp(-10.0, 10.0) as f32
    } else {
        0.0
    }
}

#[component]
pub(super) fn ViewportControls(
    host: HostHandle,
    view: Signal<EditorSnapshot>,
    mut image: Signal<Option<String>>,
    busy: Signal<bool>,
    mut status: Signal<String>,
    mut selected: Signal<String>,
    mut rig_filename: Signal<String>,
    mut rig_package_digest: Signal<String>,
    mut rig_resource_key: Signal<String>,
    mut rig_resource_digest: Signal<String>,
    mut rig_ready_source: Signal<Option<(Context, String)>>,
) -> Element {
    let mut mounted = use_signal(|| None::<Rc<MountedData>>);
    let mut drag = use_signal(|| None::<DragState>);
    let mut pending = use_signal(|| None::<PendingCommand>);
    let viewport_state = use_signal(|| None::<ViewportState>);
    let mut context = use_signal(|| Context::from_snapshot(&view()));

    use_effect(move || {
        let snapshot = view();
        let latest = Context::from_snapshot(&snapshot);
        if context() != latest {
            context.set(latest.clone());
            rig_ready_source.set(None);
            drag.set(None);
            clear_frame(image, selected, viewport_state);
            if !snapshot.reconciliation_required {
                pending.set(Some(PendingCommand {
                    command: ViewportCommand::Redraw,
                    context: latest,
                }));
                status.set("Source/session/actor context changed. Old frame and pick cleared; refreshing the current cached native preview.".into());
            } else {
                pending.set(None);
                status.set("Source/session/actor context changed. Old frame and pick cleared; explicitly refresh the held source after reconciliation.".into());
            }
        }
    });

    {
        let host = host.clone();
        use_effect(move || {
            let is_busy = busy();
            let next = pending();
            if !is_busy {
                if let Some(next) = next {
                    pending.set(None);
                    if Context::from_snapshot(&view()) == next.context {
                        run_command(
                            next.command,
                            next.context,
                            host.clone(),
                            view(),
                            view,
                            image,
                            busy,
                            status,
                            selected,
                            viewport_state,
                        );
                    } else {
                        status.set("Coalesced viewport motion dropped after source/session/actor context changed.".into());
                    }
                }
            }
        });
    }

    let refresh = {
        let host = host.clone();
        move |_| {
            enqueue(
                ViewportCommand::Redraw,
                Context::from_snapshot(&view()),
                host.clone(),
                view,
                image,
                busy,
                status,
                selected,
                viewport_state,
                pending,
            )
        }
    };
    let frame_all = {
        let host = host.clone();
        move |_| {
            enqueue(
                ViewportCommand::FrameAll,
                Context::from_snapshot(&view()),
                host.clone(),
                view,
                image,
                busy,
                status,
                selected,
                viewport_state,
                pending,
            )
        }
    };
    let frame_selected = {
        let host = host.clone();
        move |_| {
            let key = selected();
            if !key.is_empty() {
                enqueue(
                    ViewportCommand::FramePart { key },
                    Context::from_snapshot(&view()),
                    host.clone(),
                    view,
                    image,
                    busy,
                    status,
                    selected,
                    viewport_state,
                    pending,
                );
            }
        }
    };
    let clear_selection = {
        let host = host.clone();
        move |_| {
            enqueue(
                ViewportCommand::ClearSelection,
                Context::from_snapshot(&view()),
                host.clone(),
                view,
                image,
                busy,
                status,
                selected,
                viewport_state,
                pending,
            )
        }
    };
    let lighting_host = host.clone();
    let apply_lighting = move |command: ViewportCommand| {
        enqueue(
            command,
            Context::from_snapshot(&view()),
            lighting_host.clone(),
            view,
            image,
            busy,
            status,
            selected,
            viewport_state,
            pending,
        );
    };

    let rig_host = host.clone();
    let mut preview_rig = move |delta_seconds: f64| {
        if !view().publication_is_current() {
            rig_ready_source.set(None);
            status.set("The source or authority is no longer current; refresh before preview.".into());
            return;
        }
        let result = rig_preview_request(
            &rig_filename(), &rig_package_digest(), &rig_resource_key(), &rig_resource_digest(),
            rig_ready_source().as_ref(), &Context::from_snapshot(&view()), &selected(),
        );
        match result {
            Ok(resource) => enqueue(ViewportCommand::RigPreview { resource, delta_seconds }, Context::from_snapshot(&view()), rig_host.clone(), view, image, busy, status, selected, viewport_state, pending),
            Err(error) => status.set(error),
        }
    };
    let mut preview_rig_step = preview_rig.clone();
    let restore_host = host.clone();
    let restore_rig = move |_| enqueue(ViewportCommand::RestoreSourceGeometry, Context::from_snapshot(&view()), restore_host.clone(), view, image, busy, status, selected, viewport_state, pending);

    let start_drag = move |event: Event<MouseData>| {
        let data = event.data();
        let mode = match data.trigger_button() {
            Some(MouseButton::Primary) => DragMode::Orbit,
            Some(MouseButton::Auxiliary | MouseButton::Secondary) => DragMode::Pan,
            _ => return,
        };
        let point = data.client_coordinates();
        let current = view();
        if !current.publication_is_current() {
            return;
        }
        drag.set(Some(DragState {
            mode,
            last_x: point.x,
            last_y: point.y,
            start_x: point.x,
            start_y: point.y,
            moved: false,
            start_frame_id: viewport_state().map(|state| state.frame_id),
            context: Context::from_snapshot(&current),
        }));
    };
    let move_drag = {
        let host = host.clone();
        move |event: Event<MouseData>| {
            let Some(mut gesture) = drag() else {
                return;
            };
            let point = event.data().client_coordinates();
            let dx = point.x - gesture.last_x;
            let dy = point.y - gesture.last_y;
            if !dx.is_finite() || !dy.is_finite() || (dx == 0.0 && dy == 0.0) {
                return;
            }
            gesture.last_x = point.x;
            gesture.last_y = point.y;
            gesture.moved |=
                (point.x - gesture.start_x).abs() + (point.y - gesture.start_y).abs() > 2.0;
            drag.set(Some(gesture.clone()));
            if Context::from_snapshot(&view()) != gesture.context {
                drag.set(None);
                return;
            }
            let Some(element) = mounted() else {
                return;
            };
            let host = host.clone();
            spawn(async move {
                let Some(rect) = client_rect(element).await else {
                    return;
                };
                let Some([dx, dy]) = delta_to_attachment(dx, dy, rect, ATTACHMENT_SIDE) else {
                    return;
                };
                let command = match gesture.mode {
                    DragMode::Orbit => ViewportCommand::Orbit { dx, dy },
                    DragMode::Pan => ViewportCommand::Pan { dx, dy },
                };
                enqueue(
                    command,
                    gesture.context,
                    host,
                    view,
                    image,
                    busy,
                    status,
                    selected,
                    viewport_state,
                    pending,
                );
            });
        }
    };
    let end_drag = {
        let host = host.clone();
        move |event: Event<MouseData>| {
            let Some(gesture) = drag() else {
                return;
            };
            drag.set(None);
            if gesture.mode != DragMode::Orbit || gesture.moved {
                return;
            }
            let point = event.data().client_coordinates();
            let Some(state) = viewport_state() else {
                status.set("Refresh the viewport once before picking; no current frame identifier is available.".into());
                return;
            };
            let current = view();
            if Context::from_snapshot(&current) != gesture.context
                || !click_frame_is_current(gesture.start_frame_id, &state, &gesture.context)
            {
                status.set(
                    "Pick metadata is stale; refresh the current viewport before selecting a part."
                        .into(),
                );
                return;
            }
            let Some(element) = mounted() else {
                return;
            };
            let host = host.clone();
            spawn(async move {
                let Some(rect) = client_rect(element).await else {
                    return;
                };
                let Some([x, y]) = client_to_attachment(point.x, point.y, rect, ATTACHMENT_SIDE)
                else {
                    return;
                };
                let latest = view();
                let Some(latest_state) = viewport_state() else { return; };
                if Context::from_snapshot(&latest) != gesture.context
                    || !click_frame_is_current(gesture.start_frame_id, &latest_state, &gesture.context)
                    || latest_state.frame_id != state.frame_id {
                    return;
                }
                enqueue(
                    ViewportCommand::Pick {
                        frame_id: state.frame_id,
                        x,
                        y,
                    },
                    gesture.context,
                    host,
                    view,
                    image,
                    busy,
                    status,
                    selected,
                    viewport_state,
                    pending,
                );
            });
        }
    };
    let wheel = {
        let host = host.clone();
        move |event: Event<WheelData>| {
            let data = event.data();
            let point = data.client_coordinates();
            let steps = wheel_steps(data.delta());
            if steps == 0.0 {
                return;
            }
            let gesture_context = Context::from_snapshot(&view());
            let Some(element) = mounted() else {
                return;
            };
            let host = host.clone();
            spawn(async move {
                let Some(rect) = client_rect(element).await else {
                    return;
                };
                if client_to_attachment(point.x, point.y, rect, ATTACHMENT_SIDE).is_none() {
                    return;
                }
                enqueue(
                    ViewportCommand::Zoom { steps },
                    gesture_context,
                    host,
                    view,
                    image,
                    busy,
                    status,
                    selected,
                    viewport_state,
                    pending,
                );
            });
        }
    };

    rsx! {
        section { style:"padding:14px;border:1px solid #465463;margin:10px 0;display:flex;flex-direction:column;gap:8px;",
            h2 { "Native source viewport" }
            div { style:"display:flex;gap:8px;flex-wrap:wrap;",
                button { disabled:busy(), onclick:refresh, "Refresh viewport for current source" }
                button { disabled:busy(), onclick:frame_all, "Frame all" }
                button { disabled:busy() || selected().is_empty(), onclick:frame_selected, "Frame selected part" }
                button { disabled:busy() || selected().is_empty(), onclick:clear_selection, "Clear part selection" }
            }
            details {
                summary { "Preview authored rig" }
                p { "Select the bound part, then identify the saved HMC resource. Preview changes only the viewport; restore source geometry before authoring another rig." }
                input { aria_label:"Rig HMC filename", placeholder:"HMC filename", value:"{rig_filename}", oninput:move |event| {rig_filename.set(event.value());rig_ready_source.set(None);} }
                input { aria_label:"Rig package SHA256", placeholder:"Package SHA256", value:"{rig_package_digest}", oninput:move |event| {rig_package_digest.set(event.value());rig_ready_source.set(None);} }
                input { aria_label:"Rig resource key", placeholder:"Rig resource key", value:"{rig_resource_key}", oninput:move |event| {rig_resource_key.set(event.value());rig_ready_source.set(None);} }
                input { aria_label:"Rig resource SHA256", placeholder:"Resource SHA256", value:"{rig_resource_digest}", oninput:move |event| {rig_resource_digest.set(event.value());rig_ready_source.set(None);} }
                button { disabled:busy() || selected().is_empty(), onclick:move |_| {
                    if !view().publication_is_current() { status.set("The source or authority is no longer current; refresh before preview.".into()); return; }
                    match rig_resource_request(&rig_filename(), &rig_package_digest(), &rig_resource_key(), &rig_resource_digest()) {
                        Ok(_) => { rig_ready_source.set(Some((Context::from_snapshot(&view()), selected()))); status.set("Rig resource selected for this source; the host will verify the exact package and resource on preview.".into()); }
                        Err(error) => { rig_ready_source.set(None); status.set(error); }
                    }
                }, "Use rig resource for current source" }
                button { disabled:busy() || selected().is_empty(), onclick:move |_| preview_rig(0.0), "Preview current pose" }
                button { disabled:busy() || selected().is_empty(), onclick:move |_| preview_rig_step(1.0 / 30.0), "Advance 1/30 second" }
                button { disabled:busy(), onclick:restore_rig, "Restore source geometry" }
            }
            super::lookdev_controls::PreviewLightingControls { busy:busy(), onapply:apply_lighting }
            p { "Drag left to orbit; drag middle or right to pan; use the wheel to zoom; click a rendered part to select it. Gestures change only derived view state." }
            div {
                style:"width:min(100%,640px);aspect-ratio:1/1;position:relative;overflow:hidden;background:#101820;outline:1px solid #536273;touch-action:none;user-select:none;",
                onmounted:move |event|mounted.set(Some(event.data())),
                onmousedown:start_drag,
                onmousemove:move_drag,
                onmouseup:end_drag,
                onmouseleave:move |_|drag.set(None),
                onwheel:wheel,
                if let Some(uri)=image() {
                    img { src:"{uri}", alt:"Current native authored source viewport", draggable:"false", style:"width:100%;height:100%;object-fit:contain;display:block;pointer-events:none;" }
                } else {
                    p { style:"padding:24px;color:#ccd7df;", "No current frame. Refresh the viewport explicitly to render this source." }
                }
            }
            if !selected().is_empty() { p { "Selected assembly part key: {selected()}" } }
            if !status().is_empty() { p { "Viewport status: {status()}" } }
        }
    }
}

#[cfg(test)]
mod rig_resource_selection_tests {
    use super::*;
    use qualia_core_db::poet_host::invoke::studio_store::{self, StudioArtifact};
    #[test]
    fn click_keeps_mouse_down_frame_across_source_rig_source_transitions() {
        let id = format!("urn:poet:captured-pick:{}", std::process::id());
        let (_, stamp) = studio_store::create_artifact_if_absent(StudioArtifact::new(&id, "Geometry", "captured pick fixture")).unwrap();
        let source = super::super::preview::SourceRef { artifact_id:id, stamp };
        let context = Context { source:source.clone(), session_revision:1,
            execution_epoch:super::super::controller::SessionEpoch::new(), reconciliation_required:false, authorisation_stamp:None };
        let mut state = ViewportState { source, execution_epoch:Some(context.execution_epoch.clone()), frame_id:7,
            selected_part:None, camera:super::super::preview::PreviewCamera { yaw:0.0,pitch:0.0,distance:1.0,target:[0.0;3] } };
        assert!(click_frame_is_current(Some(7), &state, &context));
        assert!(!click_frame_is_current(None, &state, &context));
        state.frame_id = 8; // A rig pose generated a new frame under the same source.
        assert!(!click_frame_is_current(Some(7), &state, &context));
        state.frame_id = 9; // Source restored: same source, new frame (ABA).
        assert!(!click_frame_is_current(Some(7), &state, &context));
        state.execution_epoch = Some(super::super::controller::SessionEpoch::new());
        assert!(!click_frame_is_current(Some(9), &state, &context));
        state.execution_epoch = Some(context.execution_epoch.clone());
        state.source.artifact_id.push_str("-other");
        assert!(!click_frame_is_current(Some(9), &state, &context));
    }

    #[test]
    fn selected_rig_resource_requires_explicit_current_source_and_exact_digests() {
        let id = format!("urn:poet:rig-resource-selection:{}", std::process::id());
        let (_, stamp) = studio_store::create_artifact_if_absent(StudioArtifact::new(&id, "Geometry", "rig input fixture")).unwrap();
        let source = super::super::preview::SourceRef { artifact_id:id, stamp };
        let current = Context { source, session_revision: 1,
            execution_epoch: super::super::controller::SessionEpoch::new(),
            reconciliation_required:false, authorisation_stamp:None };
        let mut other=current.clone(); other.source.artifact_id.push_str("-other");
        let mut changed_session=current.clone(); changed_session.session_revision+=1;
        let mut changed_epoch=current.clone(); changed_epoch.execution_epoch=super::super::controller::SessionEpoch::new();
        let hash="11".repeat(32);
        let chosen = (current.clone(), "body".to_string());
        assert!(rig_preview_request("rig.hmc", &hash, "rig/key", &hash, None, &current, "body").is_err());
        assert!(rig_preview_request("rig.hmc", &hash, "rig/key", &hash, Some(&(other,"body".into())), &current, "body").is_err());
        assert!(rig_preview_request("rig.hmc", &hash, "rig/key", &hash, Some(&(changed_session,"body".into())), &current, "body").is_err());
        assert!(rig_preview_request("rig.hmc", &hash, "rig/key", &hash, Some(&(changed_epoch,"body".into())), &current, "body").is_err());
        assert!(rig_preview_request("rig.hmc", &hash, "rig/key", &hash, Some(&chosen), &current, "other-part").is_err());
        assert!(rig_preview_request("rig.hmc", &hash, "rig/key", &hash, Some(&chosen), &current, "").is_err());
        let result=rig_preview_request("rig.hmc", &hash, "rig/key", &hash, Some(&chosen), &current, "body").unwrap();
        assert_eq!(result.package_digest, [0x11;32]); assert_eq!(result.resource_digest, [0x11;32]);
        assert!(rig_preview_request("rig.hmc", "bad", "rig/key", &hash, Some(&chosen), &current, "body").is_err());
        assert!(rig_preview_request("rig.hmc", &hash, "", &hash, Some(&chosen), &current, "body").is_err());
    }
}
