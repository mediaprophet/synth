//! Small native editor for transitions in an already-admitted timeline.
use super::{AuthoringEdit, AuthoringView};
use super::timeline::TimelineEdit;
use crate::graphical::preview::SourceRef;
use dioxus::prelude::*;

#[component]
pub(super) fn TimelineControls(inspected: Signal<Option<AuthoringView>>, fresh: bool, timeline_resource_selection:Signal<Option<crate::graphical::timeline_resource_selection::TimelineResourceSelection>>,editor_view:Signal<crate::graphical::controller::EditorSnapshot>,onedit: EventHandler<AuthoringEdit>) -> Element {
    let mut selected=use_signal(String::new);
    let mut start=use_signal(String::new);
    let mut duration=use_signal(String::new);
    let mut start_frame=use_signal(String::new);
    let mut duration_frames=use_signal(String::new);
    let mut message=use_signal(String::new);
    let mut loaded=use_signal(|| None::<SourceRef>);
    let view=inspected();
    let timeline=view.as_ref().and_then(|v|v.timeline.as_ref());
    let transitions=timeline.map(|t|t.transitions.clone()).unwrap_or_default();
    let ready=fresh && view.as_ref().is_some_and(|v| loaded().as_ref() == Some(&v.source));
    let ticks_per_second=timeline_editing::TICKS_PER_SECOND;
    rsx! {
        fieldset { style:"margin-top:10px;border:1px solid #405264;",
            legend { "Timeline transitions" }
            super::timeline_create_controls::TimelineCreateControls{inspected,fresh,timeline_resource_selection,editor_view,onedit}
            super::sound_field_controls::SoundFieldControls{inspected,fresh,onedit}
            p { "Edit transition spans in exact editorial ticks ({ticks_per_second} ticks per second). Media source offsets and timeline positions remain separate. These controls do not provide playback." }
            if timeline.is_none() { p { "No admitted timeline in the current source." } }
            if timeline.is_some() {
                select { disabled:!fresh, value:"{selected}", aria_label:"Timeline transition", onchange:move |e| {
                    let id=e.value();
                    if let Some(t)=inspected().as_ref().and_then(|v|v.timeline.as_ref()).and_then(|v|v.transitions.iter().find(|t|t.id==id)) {
                        start.set(t.start.to_string()); duration.set(t.duration.to_string());
                        if let Some(timeline)=inspected().as_ref().and_then(|v|v.timeline.as_ref()) {
                            start_frame.set(super::timeline_frame_entry::ticks_to_frames_exact(t.start,timeline.frame_ticks).map(|n|n.to_string()).unwrap_or_default());
                            duration_frames.set(super::timeline_frame_entry::ticks_to_frames_exact(t.duration,timeline.frame_ticks).map(|n|n.to_string()).unwrap_or_default());
                        }
                        loaded.set(inspected().as_ref().map(|v| v.source.clone()));
                    } else {
                        loaded.set(None);
                    }
                    selected.set(id);
                },
                    option { value:"", "Choose a transition" }
                    for t in transitions { option { value:"{t.id}", "{t.id} · {t.kind}" } }
                }
                button { disabled:!fresh||selected().is_empty(), onclick:move |_| {
                    let current=inspected();
                    let Some(source)=current.as_ref() else {message.set("Inspect the source first.".into());return;};
                    if source.source.check_current().is_err() {message.set("Source changed; inspect it again.".into());return;}
                    let Some(timeline)=source.timeline.as_ref() else {message.set("No accepted timeline.".into());return;};
                    let Some(t)=timeline.transitions.iter().find(|t|t.id==selected()) else {loaded.set(None);message.set("Transition not found in the current source.".into());return;};
                    start.set(t.start.to_string());duration.set(t.duration.to_string());
                    start_frame.set(super::timeline_frame_entry::ticks_to_frames_exact(t.start,timeline.frame_ticks).map(|n|n.to_string()).unwrap_or_default());
                    duration_frames.set(super::timeline_frame_entry::ticks_to_frames_exact(t.duration,timeline.frame_ticks).map(|n|n.to_string()).unwrap_or_default());
                    loaded.set(Some(source.source.clone()));message.set(String::new());
                }, "Reload selected transition (replace local fields)" }
                label { "Start tick" input { disabled:!fresh, value:"{start}", oninput:move |e|start.set(e.value()) } }
                label { "Duration ticks" input { disabled:!fresh, value:"{duration}", oninput:move |e|duration.set(e.value()) } }
                label { "Start frame" input { disabled:!ready, value:"{start_frame}", oninput:move |e|start_frame.set(e.value()) } }
                label { "Duration in frames" input { disabled:!ready, value:"{duration_frames}", oninput:move |e|duration_frames.set(e.value()) } }
                button { disabled:!ready, onclick:move |_| {
                    let current=inspected(); let result=(|| -> Result<(u64,u64),String> {
                        let source=current.as_ref().ok_or("Inspect the source first")?;
                        if !fresh || loaded().as_ref()!=Some(&source.source) {return Err("Reload the current transition first".into());}
                        source.source.check_current()?;
                        let ticks=source.timeline.as_ref().ok_or("No accepted timeline")?.frame_ticks;
                        let a=start_frame().parse::<u64>().map_err(|_|"Enter a whole nonnegative start frame")?;
                        let b=duration_frames().parse::<u64>().map_err(|_|"Enter a whole nonnegative duration in frames")?;
                        Ok((super::timeline_frame_entry::frames_to_ticks(a,ticks)?,super::timeline_frame_entry::frames_to_ticks(b,ticks)?))
                    })();
                    match result {Ok((a,b))=>{start.set(a.to_string());duration.set(b.to_string());message.set(String::new());},Err(error)=>message.set(error)}
                }, "Use transition frames" }
                if !selected().is_empty() && !ready { p { "Source changed. Select the transition again before applying local values." } }
                button { disabled:!ready||selected().is_empty(), onclick:move |_| {
                    match (start().parse::<u64>(), duration().parse::<u64>()) {
                        (Ok(start),Ok(duration)) => { onedit.call(AuthoringEdit::Timeline { edit:TimelineEdit::SetTransition{id:selected(),start,duration} }); message.set(String::new()); },
                        _ => message.set("Enter whole nonnegative tick values.".into()),
                    }
                }, "Apply transition span" }
            }
            if !message().is_empty() { p { role:"status", "{message}" } }
        }
    }
}
