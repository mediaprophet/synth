//! Native controls issue the same checked source operations as Vibe edits.
use super::{sound_field::SoundFieldEdit, AuthoringEdit, AuthoringView};
use crate::graphical::preview::SourceRef;
use dioxus::prelude::*;

fn position(text: &str) -> Result<[f64; 3], String> {
    if text.len() > 256 {
        return Err("Position text exceeds 256 bytes".into());
    }
    let mut components = text.split(',');
    let mut values = [0.0; 3];
    for value in &mut values {
        *value = components
            .next()
            .ok_or("Enter exactly X,Y,Z metres")?
            .trim()
            .parse()
            .map_err(|_| "Enter finite X,Y,Z metres")?;
        if !value.is_finite() {
            return Err("Enter finite X,Y,Z metres".into());
        }
    }
    if components.next().is_some() {
        return Err("Enter exactly X,Y,Z metres".into());
    }
    Ok(values)
}
fn vector(v: [f64; 3]) -> String {
    format!("{},{},{}", v[0], v[1], v[2])
}

#[component]
pub(super) fn SoundFieldControls(
    inspected: Signal<Option<AuthoringView>>,
    fresh: bool,
    onedit: EventHandler<AuthoringEdit>,
) -> Element {
    let mut loaded = use_signal(|| None::<SourceRef>);
    let mut clip = use_signal(String::new);
    let mut entity = use_signal(String::new);
    let mut xyz = use_signal(|| "0,0,0".to_owned());
    let mut radius = use_signal(|| "25".to_owned());
    let mut gain = use_signal(|| "1".to_owned());
    let mut priority = use_signal(|| "0".to_owned());
    let mut listener = use_signal(|| "0,0,0".to_owned());
    let mut yaw = use_signal(|| "0".to_owned());
    let mut message = use_signal(String::new);
    let view = inspected();
    let ready = fresh
        && view
            .as_ref()
            .is_some_and(|v| v.timeline.is_some() && loaded().as_ref() == Some(&v.source));
    let bindings = view
        .as_ref()
        .and_then(|v| v.sound_field.as_ref())
        .map(|f| f.bindings.clone())
        .unwrap_or_default();
    rsx! {
        fieldset { style:"margin-top:10px;border:1px solid #405264;", legend { "Spatial sound field" }
            p { "Assignments persist in this source and its HMC checkpoint. Timeline clips retain exact start/end and source trim; the selected package must separately contain every associated clip. Static positions are local metres; entity identifiers are authored declarations. No implicit resampling or moving-emitter simulation." }
            button { disabled:!fresh, onclick:move |_| {
                let current = inspected();
                let Some(view) = current.as_ref() else { return; };
                if view.source.check_current().is_err() { message.set("Source changed; inspect again.".into()); return; }
                if let Some(field) = &view.sound_field { listener.set(vector(field.listener_position_m)); yaw.set(field.listener_yaw_rad.to_string()); }
                loaded.set(Some(view.source.clone())); message.set(String::new());
            }, "Load current field controls" }
            for b in bindings { p { "Clip {b.clip_id} → {b.entity_identifier} · gain {b.gain} · priority {b.priority} · position {b.position_m:?} m" } }
            label { "Clip ID" input { disabled:!ready, value:"{clip}", maxlength:128, oninput:move |e| clip.set(e.value()) } }
            button { disabled:!ready, onclick:move |_| {
                if let Some(b) = inspected().as_ref().and_then(|v| v.sound_field.as_ref()).and_then(|f| f.bindings.iter().find(|b| b.clip_id == clip())) {
                    entity.set(b.entity_identifier.clone()); xyz.set(vector(b.position_m)); radius.set(b.radius_m.to_string()); gain.set(b.gain.to_string()); priority.set(b.priority.to_string());
                } else { message.set("No existing field assignment for this clip.".into()); }
            }, "Reload selected assignment" }
            label { "Entity identifier" input { disabled:!ready, value:"{entity}", maxlength:2048, oninput:move |e| entity.set(e.value()) } }
            label { "Emitter X,Y,Z metres" input { disabled:!ready, value:"{xyz}", maxlength:256, oninput:move |e| xyz.set(e.value()) } }
            label { "Coherence radius metres" input { disabled:!ready, value:"{radius}", maxlength:64, oninput:move |e| radius.set(e.value()) } }
            label { "Gain 0–1" input { disabled:!ready, value:"{gain}", maxlength:64, oninput:move |e| gain.set(e.value()) } }
            label { "Priority (exact integer)" input { disabled:!ready, value:"{priority}", maxlength:16, oninput:move |e| priority.set(e.value()) } }
            button { disabled:!ready, onclick:move |_| {
                let edit = (|| Ok::<_, String>(SoundFieldEdit::SetClip { clip_id:clip(), entity_identifier:entity(), position_m:position(&xyz())?,
                    radius_m:radius().parse().map_err(|_| "Invalid radius")?, gain:gain().parse().map_err(|_| "Invalid gain")?, priority:priority().parse().map_err(|_| "Invalid priority")? }))();
                match edit { Ok(edit) => onedit.call(AuthoringEdit::SoundField { edit }), Err(error) => message.set(error) }
            }, "Set clip emitter assignment" }
            button { disabled:!ready, onclick:move |_| onedit.call(AuthoringEdit::SoundField { edit:SoundFieldEdit::RemoveClip { clip_id:clip() } }), "Remove clip emitter assignment" }
            label { "Listener X,Y,Z metres" input { disabled:!ready, value:"{listener}", maxlength:256, oninput:move |e| listener.set(e.value()) } }
            label { "Listener yaw radians" input { disabled:!ready, value:"{yaw}", maxlength:64, oninput:move |e| yaw.set(e.value()) } }
            button { disabled:!ready, onclick:move |_| {
                match position(&listener()).and_then(|position_m| yaw().parse::<f64>().map(|yaw_rad| SoundFieldEdit::SetListener { position_m, yaw_rad }).map_err(|_| "Invalid listener yaw".into())) {
                    Ok(edit) => onedit.call(AuthoringEdit::SoundField { edit }), Err(error) => message.set(error)
                }
            }, "Set listener" }
            p { "{message}" }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::position;
    #[test]
    fn coordinate_entry_is_bounded_before_parsing_or_allocation() {
        assert_eq!(position("1, 2, -3").unwrap(), [1., 2., -3.]);
        for invalid in ["1,2", "1,2,3,4", "NaN,0,0"] {
            assert!(position(invalid).is_err());
        }
        assert!(position(&"0,".repeat(1024)).is_err());
    }
}
