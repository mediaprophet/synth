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
    context: Context,
}

#[derive(Clone)]
struct PendingCommand {
    command: ViewportCommand,
    context: Context,
}

fn add_commands(previous: ViewportCommand, next: ViewportCommand) -> ViewportCommand {
    match (previous, next) {
        (ViewportCommand::Orbit { dx: ax, dy: ay }, ViewportCommand::Orbit { dx: bx, dy: by }) =>
            ViewportCommand::Orbit { dx: (ax + bx).clamp(-8192.0, 8192.0), dy: (ay + by).clamp(-8192.0, 8192.0) },
        (ViewportCommand::Pan { dx: ax, dy: ay }, ViewportCommand::Pan { dx: bx, dy: by }) =>
            ViewportCommand::Pan { dx: (ax + bx).clamp(-8192.0, 8192.0), dy: (ay + by).clamp(-8192.0, 8192.0) },
        (ViewportCommand::Zoom { steps: a }, ViewportCommand::Zoom { steps: b }) =>
            ViewportCommand::Zoom { steps: (a + b).clamp(-64.0, 64.0) },
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
    mut selected: Signal<String>,
    mut viewport_state: Signal<Option<ViewportState>>,
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
    run_command(command, context, host, current, view, image, busy, status, selected, viewport_state);
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
        status.set("Viewport command dropped because its native source context is no longer current.".into());
        return;
    }
    busy.set(true);
    let guard = host.clone();
    spawn(async move {
        let expected = current.source.clone();
        match native_dispatch::run(
            host,
            Action::Viewport { request: command, expected },
            context.clone(),
        ).await {
            Ok(reply) if jobs::completion_is_current(&guard, &current, &reply) => {
                let current_view = view();
                if Context::from_snapshot(&current_view) != context {
                    clear_frame(image, selected, viewport_state);
                    status.set("Viewport result withheld because the source/session/actor context changed.".into());
                } else if !reply.operation_succeeded {
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
    if amount.is_finite() { amount.clamp(-10.0, 10.0) as f32 } else { 0.0 }
}

#[component]
pub(super) fn ViewportControls(
    host: HostHandle,
    view: Signal<EditorSnapshot>,
    mut image: Signal<Option<String>>,
    busy: Signal<bool>,
    mut status: Signal<String>,
    mut selected: Signal<String>,
) -> Element {
    let mut mounted = use_signal(|| None::<Rc<MountedData>>);
    let mut drag = use_signal(|| None::<DragState>);
    let mut pending = use_signal(|| None::<PendingCommand>);
    let mut viewport_state = use_signal(|| None::<ViewportState>);
    let mut context = use_signal(|| Context::from_snapshot(&view()));

    use_effect(move || {
        let snapshot = view();
        let latest = Context::from_snapshot(&snapshot);
        if context() != latest {
            context.set(latest.clone());
            drag.set(None);
            clear_frame(image, selected, viewport_state);
            if !snapshot.reconciliation_required {
                pending.set(Some(PendingCommand { command:ViewportCommand::Redraw, context:latest }));
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
                            next.command, next.context, host.clone(), view(), view, image, busy,
                            status, selected, viewport_state,
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
        move |_| enqueue(ViewportCommand::Redraw, Context::from_snapshot(&view()), host.clone(), view, image, busy, status, selected, viewport_state, pending)
    };
    let frame_all = {
        let host = host.clone();
        move |_| enqueue(ViewportCommand::FrameAll, Context::from_snapshot(&view()), host.clone(), view, image, busy, status, selected, viewport_state, pending)
    };
    let frame_selected = {
        let host = host.clone();
        move |_| {
            let key = selected();
            if !key.is_empty() {
                enqueue(ViewportCommand::FramePart { key }, Context::from_snapshot(&view()), host.clone(), view, image, busy, status, selected, viewport_state, pending);
            }
        }
    };
    let clear_selection = {
        let host = host.clone();
        move |_| enqueue(ViewportCommand::ClearSelection, Context::from_snapshot(&view()), host.clone(), view, image, busy, status, selected, viewport_state, pending)
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

    let start_drag = move |event: Event<MouseData>| {
        let data = event.data();
        let mode = match data.trigger_button() {
            Some(MouseButton::Primary) => DragMode::Orbit,
            Some(MouseButton::Auxiliary | MouseButton::Secondary) => DragMode::Pan,
            _ => return,
        };
        let point = data.client_coordinates();
        let current = view();
        if !current.publication_is_current() { return; }
        drag.set(Some(DragState {
            mode, last_x:point.x, last_y:point.y, start_x:point.x, start_y:point.y,
            moved:false, context:Context::from_snapshot(&current),
        }));
    };
    let move_drag = {
        let host = host.clone();
        move |event: Event<MouseData>| {
            let Some(mut gesture) = drag() else { return; };
            let point = event.data().client_coordinates();
            let dx = point.x - gesture.last_x;
            let dy = point.y - gesture.last_y;
            if !dx.is_finite() || !dy.is_finite() || (dx == 0.0 && dy == 0.0) { return; }
            gesture.last_x = point.x;
            gesture.last_y = point.y;
            gesture.moved |= (point.x - gesture.start_x).abs() + (point.y - gesture.start_y).abs() > 2.0;
            drag.set(Some(gesture.clone()));
            if Context::from_snapshot(&view()) != gesture.context { drag.set(None); return; }
            let Some(element) = mounted() else { return; };
            let host = host.clone();
            spawn(async move {
                let Some(rect) = client_rect(element).await else { return; };
                let Some([dx, dy]) = delta_to_attachment(dx, dy, rect, ATTACHMENT_SIDE) else { return; };
                let command = match gesture.mode {
                    DragMode::Orbit => ViewportCommand::Orbit { dx, dy },
                    DragMode::Pan => ViewportCommand::Pan { dx, dy },
                };
                enqueue(command, gesture.context, host, view, image, busy, status, selected, viewport_state, pending);
            });
        }
    };
    let end_drag = {
        let host = host.clone();
        move |event: Event<MouseData>| {
            let Some(gesture) = drag() else { return; };
            drag.set(None);
            if gesture.mode != DragMode::Orbit || gesture.moved { return; }
            let point = event.data().client_coordinates();
            let Some(state) = viewport_state() else {
                status.set("Refresh the viewport once before picking; no current frame identifier is available.".into());
                return;
            };
            let current = view();
            if state.source != current.source || state.execution_epoch.as_ref() != Some(&current.execution_epoch) {
                status.set("Pick metadata is stale; refresh the current viewport before selecting a part.".into());
                return;
            }
            let Some(element) = mounted() else { return; };
            let host = host.clone();
            spawn(async move {
                let Some(rect) = client_rect(element).await else { return; };
                let Some([x, y]) = client_to_attachment(point.x, point.y, rect, ATTACHMENT_SIDE) else { return; };
                enqueue(ViewportCommand::Pick { frame_id:state.frame_id, x, y }, gesture.context, host, view, image, busy, status, selected, viewport_state, pending);
            });
        }
    };
    let wheel = {
        let host = host.clone();
        move |event: Event<WheelData>| {
            let data = event.data();
            let point = data.client_coordinates();
            let steps = wheel_steps(data.delta());
            if steps == 0.0 { return; }
            let gesture_context = Context::from_snapshot(&view());
            let Some(element) = mounted() else { return; };
            let host = host.clone();
            spawn(async move {
                let Some(rect) = client_rect(element).await else { return; };
                if client_to_attachment(point.x, point.y, rect, ATTACHMENT_SIDE).is_none() { return; }
                enqueue(ViewportCommand::Zoom { steps }, gesture_context, host, view, image, busy, status, selected, viewport_state, pending);
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

