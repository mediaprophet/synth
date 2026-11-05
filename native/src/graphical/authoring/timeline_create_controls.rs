//! Explicit, source-bound timeline creation. Media references are not resolved.
use super::{AuthoringEdit,AuthoringView};
use super::timeline::{Clip,Timeline,TimelineEdit,Transition};
use crate::graphical::preview::SourceRef;
use dioxus::prelude::*;

#[component]
pub(super) fn TimelineCreateControls(inspected:Signal<Option<AuthoringView>>,fresh:bool,timeline_resource_selection:Signal<Option<crate::graphical::timeline_resource_selection::TimelineResourceSelection>>,editor_view:Signal<crate::graphical::controller::EditorSnapshot>,onedit:EventHandler<AuthoringEdit>)->Element {
    let mut loaded=use_signal(||None::<SourceRef>); let mut message=use_signal(String::new);
    let mut loaded_clip=use_signal(||None::<(SourceRef,String,String)>);
    let mut projection=use_signal(||None::<(SourceRef,String,String,String)>);
    let mut frame=use_signal(||(timeline_editing::TICKS_PER_SECOND/30).to_string());
    let mut fps_num=use_signal(||"30".to_string()); let mut fps_den=use_signal(||"1".to_string());
    let mut clip_start_frames=use_signal(String::new); let mut clip_duration_frames=use_signal(String::new);
    let mut track=use_signal(String::new); let mut clip=use_signal(String::new); let mut source=use_signal(String::new);
    let mut source_start=use_signal(||"0".to_string()); let mut start=use_signal(||"0".to_string()); let mut duration=use_signal(String::new);
    let mut transition=use_signal(String::new); let mut kind=use_signal(||"cross_dissolve".to_string());
    let mut from=use_signal(String::new); let mut to=use_signal(String::new); let mut transition_start=use_signal(String::new); let mut transition_duration=use_signal(String::new);
    let mut transition_start_frames=use_signal(String::new); let mut transition_duration_frames=use_signal(String::new);
    let mut map_clip=use_signal(String::new); let mut map_tick=use_signal(String::new);
    let view=inspected(); let ready=fresh && view.as_ref().is_some_and(|v| loaded().as_ref()==Some(&v.source));
    let exists=view.as_ref().is_some_and(|v|v.timeline.is_some());
    let clip_ready=ready && view.as_ref().is_some_and(|v|loaded_clip().as_ref().is_some_and(|(s,t,c)|s==&v.source && t==&track() && c==&clip()));
    let tracks=view.as_ref().and_then(|v|v.timeline.as_ref()).map(|t|t.tracks.clone()).unwrap_or_default();
    rsx! {
      fieldset { style:"margin-top:10px;border:1px solid #405264;", legend { "Build timeline" }
        p { "Inspect the current source, then load it here before adding a timeline, track, clip or transition. Times are exact editorial ticks. Media identifiers are recorded as supplied; this tool does not resolve media or provide playback." }
        button { disabled:!fresh, onclick:move |_| { loaded.set(inspected().as_ref().map(|v|v.source.clone())); message.set(String::new()); }, "Load current source for timeline edits" }
        if !ready { p { "Load or reload the current source before applying these fields." } }
        if !exists {
          p { "Set a rational frame rate or enter a frame duration directly. Only rates with an exact whole-tick duration are admitted." }
          label { "Frames per second numerator" input { disabled:!ready, value:"{fps_num}", oninput:move|e|fps_num.set(e.value()) } }
          label { "Frames per second denominator" input { disabled:!ready, value:"{fps_den}", oninput:move|e|fps_den.set(e.value()) } }
          button { disabled:!ready, onclick:move |_| {
            let result=fps_num().parse::<u64>().map_err(|_|"Enter a whole numerator".to_string()).and_then(|n|fps_den().parse::<u64>().map_err(|_|"Enter a whole denominator".to_string()).and_then(|d|super::timeline_frame_entry::frame_ticks(n,d)));
            match result {Ok(ticks)=>{frame.set(ticks.to_string());message.set(String::new());},Err(error)=>message.set(error)}
          }, "Use exact frame rate" }
          label { "Frame duration (ticks)" input { disabled:!ready, value:"{frame}", oninput:move|e|frame.set(e.value()) } }
          button { disabled:!ready, onclick:move |_| match frame().parse::<u64>() {
            Ok(frame_ticks)=>onedit.call(AuthoringEdit::Timeline{edit:TimelineEdit::Adopt{timeline:Timeline{version:1,frame_ticks,tracks:vec![],transitions:vec![]}}}),
            Err(_)=>message.set("Enter a whole frame duration in ticks.".into()),
          }, "Create empty timeline" }
        } else {
          for t in tracks {
            p { "Track {t.id}" if t.locked { " · locked" } }
            ul { for c in t.clips { li { "{c.id} · {c.source} · start {c.start} · duration {c.duration} ticks" } } }
          }
          h5 { "Add track" }
          label { "Track ID (also the target for clips and transitions)" input { disabled:!ready, value:"{track}", oninput:move|e|track.set(e.value()) } }
          button { disabled:!ready, onclick:move |_| onedit.call(AuthoringEdit::Timeline{edit:TimelineEdit::AddTrack{id:track()}}), "Add track" }
          button { disabled:!ready, onclick:move |_| onedit.call(AuthoringEdit::Timeline{edit:TimelineEdit::SetTrackLocked{id:track(),locked:true}}), "Lock track" }
          button { disabled:!ready, onclick:move |_| onedit.call(AuthoringEdit::Timeline{edit:TimelineEdit::SetTrackLocked{id:track(),locked:false}}), "Unlock track" }
          button { disabled:!ready, onclick:move |_| onedit.call(AuthoringEdit::Timeline{edit:TimelineEdit::RemoveTrack{id:track()}}), "Remove empty track" }
          h5 { "Add clip to track" }
          label { "Clip ID" input { disabled:!ready, value:"{clip}", oninput:move|e|clip.set(e.value()) } }
          label { "Media identifier" input { disabled:!ready, value:"{source}", oninput:move|e|source.set(e.value()) } }
          label { "Source offset (ticks)" input { disabled:!ready, value:"{source_start}", oninput:move|e|source_start.set(e.value()) } }
          label { "Timeline position (ticks)" input { disabled:!ready, value:"{start}", oninput:move|e|start.set(e.value()) } }
          label { "Duration (ticks)" input { disabled:!ready, value:"{duration}", oninput:move|e|duration.set(e.value()) } }
          p { "For the timeline position and duration, whole frames can be converted exactly. Source offset uses the media's own clock and remains in ticks." }
          label { "Timeline start frame" input { disabled:!ready, value:"{clip_start_frames}", oninput:move|e|clip_start_frames.set(e.value()) } }
          label { "Duration in frames" input { disabled:!ready, value:"{clip_duration_frames}", oninput:move|e|clip_duration_frames.set(e.value()) } }
          button { disabled:!ready, onclick:move |_| {
            let current=inspected();
            let result=(|| -> Result<(u64,u64),String> {
                let source=current.as_ref().ok_or("Inspect the source first")?;
                if !fresh || loaded().as_ref()!=Some(&source.source) {return Err("Reload the current source first".into());}
                source.source.check_current()?;
                let ticks=source.timeline.as_ref().ok_or("No accepted timeline")?.frame_ticks;
                let a=clip_start_frames().parse::<u64>().map_err(|_|"Enter a whole nonnegative start frame")?;
                let b=clip_duration_frames().parse::<u64>().map_err(|_|"Enter a whole nonnegative frame duration")?;
                Ok((super::timeline_frame_entry::frames_to_ticks(a,ticks)?,super::timeline_frame_entry::frames_to_ticks(b,ticks)?))
            })();
            match result {Ok((a,b))=>{start.set(a.to_string());duration.set(b.to_string());message.set(String::new());},Err(error)=>message.set(error)}
          }, "Use clip frames" }
          button { disabled:!ready, onclick:move |_| {
              let current=inspected();
              if let Some(c)=current.as_ref().and_then(|v|v.timeline.as_ref()).and_then(|t|t.tracks.iter().find(|t|t.id==track())).and_then(|t|t.clips.iter().find(|c|c.id==clip())) {
                  source.set(c.source.clone()); source_start.set(c.source_start.to_string()); start.set(c.start.to_string()); duration.set(c.duration.to_string()); message.set(String::new());
                  if let Some(t)=current.as_ref().and_then(|v|v.timeline.as_ref()) {
                      clip_start_frames.set(super::timeline_frame_entry::ticks_to_frames_exact(c.start,t.frame_ticks).map(|n|n.to_string()).unwrap_or_default());
                      clip_duration_frames.set(super::timeline_frame_entry::ticks_to_frames_exact(c.duration,t.frame_ticks).map(|n|n.to_string()).unwrap_or_default());
                  }
                  loaded_clip.set(current.as_ref().map(|v|(v.source.clone(),track(),clip())));
              } else { loaded_clip.set(None); message.set("Clip not found in the selected track.".into()); }
          }, "Load clip fields" }
          if !clip().is_empty() && !clip_ready { p { "Load the selected clip before updating its fields." } }
          button { disabled:!ready, onclick:move |_| match (source_start().parse::<u64>(),start().parse::<u64>(),duration().parse::<u64>()) {
            (Ok(source_start),Ok(start),Ok(duration))=>onedit.call(AuthoringEdit::Timeline{edit:TimelineEdit::AddClip{track:track(),clip:Clip{id:clip(),source:source(),source_start,start,duration}}}),
            _=>message.set("Enter whole nonnegative clip times in ticks.".into()),
          }, "Add clip" }
          button { disabled:!clip_ready, onclick:move |_| { let current=inspected();
            if !fresh || !current.as_ref().is_some_and(|v|v.source.check_current().is_ok() && loaded_clip().as_ref().is_some_and(|(s,t,c)|s==&v.source && t==&track() && c==&clip())) { message.set("Reload the selected clip from the current source.".into()); return; }
            match (source_start().parse::<u64>(),start().parse::<u64>(),duration().parse::<u64>()) {
            (Ok(source_start),Ok(start),Ok(duration))=>onedit.call(AuthoringEdit::Timeline{edit:TimelineEdit::UpdateClip{track:track(),clip:Clip{id:clip(),source:source(),source_start,start,duration}}}),
            _=>message.set("Enter whole nonnegative clip times in ticks.".into()),
          } }, "Update clip" }
          button { disabled:!ready, onclick:move |_| onedit.call(AuthoringEdit::Timeline{edit:TimelineEdit::RemoveClip{track:track(),id:clip()}}), "Remove clip" }
          h5 { "Associate selected HMC resource" }
          p { "Inspect an HMC package and select a resource above. The association records its key and digest for this clip; each later media read rechecks the selected package. This does not establish media rights or playback." }
          if let Some(chosen)=timeline_resource_selection() {
              p { "Package {chosen.request.filename} · key {chosen.request.key}" }
              p { "Package SHA256 {crate::graphical::package::hex_digest(&chosen.request.package_digest)}" }
              p { "Resource SHA256 {crate::graphical::package::hex_digest(&chosen.request.resource_digest)}" }
          } else { p { "No HMC resource selected." } }
          if let Some(data)=view.as_ref() { for binding in data.timeline_media.iter() { p { "Clip {binding.clip_id} → resource {binding.resource_key} · SHA256 {crate::graphical::package::hex_digest(&binding.resource_digest)}" } } }
          button { disabled:!ready||clip().is_empty()||timeline_resource_selection().is_none(), onclick:move |_| {
              let current=editor_view(); let selected=timeline_resource_selection();
              let result=selected.as_ref().ok_or("Select a package resource first".to_string()).and_then(|s|s.checked_request(&current));
              match result { Ok(request)=>onedit.call(AuthoringEdit::Timeline{edit:TimelineEdit::AssociateResource{clip_id:clip(),resource_key:request.key,resource_digest:request.resource_digest}}), Err(e)=>message.set(e) }
          }, "Associate selected resource with clip" }
          button { disabled:!ready||clip().is_empty(), onclick:move |_| onedit.call(AuthoringEdit::Timeline{edit:TimelineEdit::DissociateResource{clip_id:clip()}}), "Remove clip association" }
          h5 { "Add transition to track" }
          label { "Transition ID" input { disabled:!ready, value:"{transition}", oninput:move|e|transition.set(e.value()) } }
          label { "Kind" input { disabled:!ready, value:"{kind}", oninput:move|e|kind.set(e.value()) } }
          label { "Outgoing clip ID" input { disabled:!ready, value:"{from}", oninput:move|e|from.set(e.value()) } }
          label { "Incoming clip ID" input { disabled:!ready, value:"{to}", oninput:move|e|to.set(e.value()) } }
          label { "Start (ticks)" input { disabled:!ready, value:"{transition_start}", oninput:move|e|transition_start.set(e.value()) } }
          label { "Duration (ticks)" input { disabled:!ready, value:"{transition_duration}", oninput:move|e|transition_duration.set(e.value()) } }
          label { "Transition start frame" input { disabled:!ready, value:"{transition_start_frames}", oninput:move|e|transition_start_frames.set(e.value()) } }
          label { "Transition duration in frames" input { disabled:!ready, value:"{transition_duration_frames}", oninput:move|e|transition_duration_frames.set(e.value()) } }
          button { disabled:!ready, onclick:move |_| {
            let current=inspected(); let result=(|| -> Result<(u64,u64),String> {
                let source=current.as_ref().ok_or("Inspect the source first")?;
                if !fresh || loaded().as_ref()!=Some(&source.source) {return Err("Reload the current source first".into());}
                source.source.check_current()?;
                let ticks=source.timeline.as_ref().ok_or("No accepted timeline")?.frame_ticks;
                let start=transition_start_frames().parse::<u64>().map_err(|_|"Enter a whole nonnegative transition start frame")?;
                let duration=transition_duration_frames().parse::<u64>().map_err(|_|"Enter a whole nonnegative transition duration in frames")?;
                Ok((super::timeline_frame_entry::frames_to_ticks(start,ticks)?,super::timeline_frame_entry::frames_to_ticks(duration,ticks)?))
            })();
            match result {Ok((start,duration))=>{transition_start.set(start.to_string());transition_duration.set(duration.to_string());message.set(String::new());},Err(error)=>message.set(error)}
          }, "Use transition frames for new transition" }
          button { disabled:!ready, onclick:move |_| match (transition_start().parse::<u64>(),transition_duration().parse::<u64>()) {
            (Ok(start),Ok(duration))=>onedit.call(AuthoringEdit::Timeline{edit:TimelineEdit::AddTransition{transition:Transition{id:transition(),kind:kind(),track:track(),from:from(),to:to(),start,duration}}}),
            _=>message.set("Enter whole nonnegative transition times in ticks.".into()),
          }, "Add transition" }
          button { disabled:!ready, onclick:move |_| onedit.call(AuthoringEdit::Timeline{edit:TimelineEdit::RemoveTransition{id:transition()}}), "Remove transition" }
          h5 { "Inspect clip source time" }
          p { "Project a timeline tick onto the selected clip's source clock; no media is opened." }
          label { "Clip ID" input { disabled:!ready, value:"{map_clip}", oninput:move|e|map_clip.set(e.value()) } }
          label { "Timeline tick" input { disabled:!ready, value:"{map_tick}", oninput:move|e|map_tick.set(e.value()) } }
          button { disabled:!ready, onclick:move |_| {
              let current=inspected();
              if !fresh || loaded().as_ref() != current.as_ref().map(|v|&v.source) { message.set("Reload the current source before projecting.".into()); return; }
              let result=map_tick().parse::<u64>().map_err(|_|"Enter a whole timeline tick".to_string()).and_then(|tick| {
                  let source=current.as_ref().ok_or("No inspected source")?;
                  source.source.check_current()?;
                  let timeline=source.timeline.as_ref().ok_or("No accepted timeline")?;
                  let id=map_clip(); let value=super::timeline_media::map_clip_time(timeline,&id,tick)?;
                  source.source.check_current()?;
                  Ok(format!("Source {} at media tick {} (identifier not resolved)",value.source_identifier,value.media_tick))
              });
              match result { Ok(s) => { if let Some(v)=current { projection.set(Some((v.source,map_clip(),map_tick(),s))); message.set(String::new()); } }, Err(e) => { projection.set(None); message.set(e); } }
          }, "Show source time" }
          if let Some((source, clip, tick, text))=projection() {
              if fresh && view.as_ref().is_some_and(|v|v.source==source) && source.check_current().is_ok() && clip==map_clip() && tick==map_tick() { p { role:"status", "{text}" } }
              else { p { role:"status", "Source or input changed; the previous source-time projection is stale." } }
          }
        }
        if !message().is_empty() { p { role:"status", "{message}" } }
      }
    }
}
