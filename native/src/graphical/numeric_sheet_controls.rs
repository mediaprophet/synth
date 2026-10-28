//! Typed controls for an existing source-owned numeric grid.
use super::{controller::EditorSnapshot, jobs::{self, Action, Context, HostHandle}, numeric_sheet::{SheetEdit, SheetView}};
use dioxus::prelude::*;
use vibe::Value;

fn display(value: &Value) -> String {
    match value { Value::I64(n) => n.to_string(), Value::U64(n) => n.to_string(), Value::F64(n) => format!("{n:?}"), _ => "unsupported".into() }
}
#[allow(clippy::too_many_arguments)]
fn send(action: Action, edit: bool, host: HostHandle, mut view: Signal<EditorSnapshot>, mut draft: Signal<String>, mut status: Signal<String>, mut image: Signal<Option<String>>, mut busy: Signal<bool>, mut sheet: Signal<Option<(Context, SheetView)>>, buffer: Option<super::draft_buffer::DraftBuffer>) {
    if busy() { return; }
    if edit && draft() != view().latest_source { status.set("Unsubmitted source draft preserved; submit or explicitly restore it before editing the grid.".into()); return; }
    let capture = if edit { match super::draft_buffer::DraftCapture::capture(draft(), buffer.as_ref()) { Ok(c) => Some(c), Err(e) => { status.set(e); return; } } } else { None };
    let origin = Context::from_snapshot(&view());
    let guard = host.clone();
    busy.set(true);
    spawn(async move {
        let result = if origin.authorisation_stamp.is_some() {
            let intent = match action {
                Action::NumericSheetInspect(_) => super::poet_workspace::NumericSheetIntent::Inspect,
                Action::NumericSheetEdit { edit, .. } => super::poet_workspace::NumericSheetIntent::Edit(edit),
                _ => unreachable!("numeric sheet control only dispatches numeric intents"),
            };
            match super::poet_workspace::NativeIntentBus::new(host, origin.clone()) {
                Ok(bus) => match bus.dispatch_numeric_sheet(intent).await {
                    Ok(dispatch) => dispatch.reply.ok_or_else(|| format!("Poet intent rejected: {:?}", dispatch.receipt.status)),
                    Err(error) => Err(error),
                },
                Err(error) => Err(error),
            }
        } else { jobs::run(host, action, origin.clone()).await };
        match result {
            Ok(reply) if jobs::completion_is_current(&guard, &view(), &reply) => {
                let mut retained = false;
                if let Some(snapshot) = reply.snapshot {
                    if let Some(capture) = &capture {
                        if let Some(text) = capture.accept(reply.operation_succeeded, &draft(), &snapshot.latest_source, buffer.as_ref()) { draft.set(text); }
                        else if reply.operation_succeeded { retained = true; }
                    }
                    if edit && reply.operation_succeeded { sheet.set(None); image.set(reply.image_uri); }
                    view.set(snapshot);
                }
                if let Some(result) = reply.numeric_sheet { sheet.set(Some((origin, result))); }
                status.set(if retained { format!("{}; newer unsubmitted draft preserved", reply.message) } else { reply.message });
            }
            Ok(_) => { sheet.set(None); image.set(None); status.set("Source or contributor changed; late grid result withheld.".into()); }
            Err(error) => { sheet.set(None); image.set(None); status.set(error); }
        }
        busy.set(false);
    });
}
#[component]
pub(super) fn NumericSheetControls(host: HostHandle, view: Signal<EditorSnapshot>, draft: Signal<String>, mut status: Signal<String>, image: Signal<Option<String>>, busy: Signal<bool>, #[props(default)] buffer: Option<super::draft_buffer::DraftBuffer>) -> Element {
    let sheet = use_signal(|| None::<(Context, SheetView)>);
    let mut row = use_signal(|| "0".to_string());
    let mut col = use_signal(|| "0".to_string());
    let mut value = use_signal(String::new);
    let current = view();
    let current_context = Context::from_snapshot(&current);
    let fresh = sheet().as_ref().is_some_and(|(context, data)| *context == current_context && data.source == current.source) && !current.reconciliation_required;
    let inspect_host = host.clone(); let inspect_buffer = buffer.clone();
    let edit_host = host.clone(); let edit_buffer = buffer.clone();
    rsx! {
        fieldset { disabled: busy() || current.reconciliation_required, style: "margin:0;padding:10px;border:1px solid #405264;min-width:0;",
            legend { "Numeric source grid" }
            p { "Inspect an existing mutable poet_sheet numeric grid. Coordinates start at zero; values are finite numbers. This is not a complete workbook." }
            p { "To admit a new grid, enter and submit `let mut poet_sheet=[[1,2],[3,4]]; poet_sheet;` in the source editor. This creates a binding only after normal source authorization. Editing depends on the host profile and contributor permissions. Existing source is not replaced automatically." }
            button { onclick: move |_| { let expected=view().source; send(Action::NumericSheetInspect(expected), false, inspect_host.clone(), view, draft, status, image, busy, sheet, inspect_buffer.clone()); }, "Inspect grid" }
            if fresh {
                if let Some((_, data)) = sheet() {
                    div { style:"overflow:auto;max-height:240px;", table { for cells in data.rows.iter() { tr { for cell in cells.iter() { td { style:"padding:3px;border:1px solid #405264;", "{display(cell)}" } } } } } }
                }
            }
            div { style:"display:flex;gap:6px;flex-wrap:wrap;",
                label { "Row" input { value:"{row}", aria_label:"Grid row (zero based)", oninput: move |e| row.set(e.value()) } }
                label { "Column" input { value:"{col}", aria_label:"Grid column (zero based)", oninput: move |e| col.set(e.value()) } }
                label { "Number" input { value:"{value}", aria_label:"New finite cell value", oninput: move |e| value.set(e.value()) } }
                button { disabled: !fresh, onclick: move |_| {
                    if !sheet().as_ref().is_some_and(|(ctx, _)| *ctx == Context::from_snapshot(&view())) { status.set("Inspect the current grid before editing.".into()); return; }
                    let parsed = row().parse::<usize>().ok().zip(col().parse::<usize>().ok()).zip(value().parse::<f64>().ok()).filter(|(_,v)| v.is_finite());
                    let Some(((row, col), value)) = parsed else { status.set("Enter nonnegative integer coordinates and a finite number.".into()); return; };
                    let expected = view().source;
                    let Some((_, inspected)) = sheet() else { status.set("Inspect the current grid before editing.".into()); return; };
                    send(Action::NumericSheetEdit { edit: SheetEdit { row, col, value, execution_epoch: inspected.execution_epoch }, expected }, true, edit_host.clone(), view, draft, status, image, busy, sheet, edit_buffer.clone());
                }, "Set cell" }
            }
        }
    }
}
