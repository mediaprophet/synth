//! Explicit editorial material keys over the existing admitted source session.
//! EffectCraft's property/key inspector is an interaction reference only; no
//! EffectCraft implementation, clock, expression host or interpolation is copied.
use super::material_motion::{
    MaterialMotionDraft, MaterialMotionEdit, MaterialMotionKey, MotionInterpolation, MAX_KEYS,
    MAX_TIME_MS, MAX_TRACKS,
};
use super::{AuthoringEdit, AuthoringView, PartView};
use crate::graphical::preview::SourceRef;
use dioxus::prelude::*;

const FACTORS: [&str; 6] = [
    "Linear red",
    "Linear green",
    "Linear blue",
    "Alpha",
    "Roughness",
    "Metallic",
];

#[derive(Clone)]
struct MotionForm {
    source: SourceRef,
    part_id: String,
    draft: MaterialMotionDraft,
    accepted: bool,
    playhead: String,
    selected_key: Option<u32>,
    key_time: String,
    values: Vec<String>,
    interpolation: String,
    track_dirty: bool,
    fields_dirty: bool,
}

fn factor_fields(part: &PartView) -> Vec<String> {
    part.base_color
        .into_iter()
        .chain([part.roughness, part.metallic])
        .map(|n| n.to_string())
        .collect()
}

fn time_ms(value: &str) -> Result<u32, String> {
    value
        .trim()
        .parse::<u32>()
        .ok()
        .filter(|n| *n <= MAX_TIME_MS)
        .ok_or_else(|| "Time must be whole milliseconds from 0 to 86400000".into())
}

impl MotionForm {
    fn load(view: &AuthoringView, part_id: &str) -> Option<Self> {
        let part = view
            .parts
            .iter()
            .find(|p| p.id == part_id && p.material_id != 0)?;
        let track = view
            .material_motion
            .iter()
            .find(|t| t.draft.material_id == part.material_id);
        let sample = track.map(|t| t.sample_ms).unwrap_or(0).to_string();
        Some(Self {
            source: view.source.clone(),
            part_id: part.id.clone(),
            draft: track
                .map(|t| t.draft.clone())
                .unwrap_or(MaterialMotionDraft {
                    version: 1,
                    material_id: part.material_id,
                    keys: vec![],
                }),
            accepted: track.is_some(),
            playhead: sample.clone(),
            selected_key: None,
            key_time: sample,
            values: factor_fields(part),
            interpolation: "linear".into(),
            track_dirty: false,
            fields_dirty: false,
        })
    }

    fn current(&self, view: &AuthoringView, part_id: &str) -> bool {
        self.source == view.source
            && self.part_id == part_id
            && view
                .parts
                .iter()
                .any(|p| p.id == part_id && p.material_id == self.draft.material_id)
    }

    fn dirty(&self) -> bool {
        self.track_dirty || self.fields_dirty
    }

    fn key(&self) -> Result<MaterialMotionKey, String> {
        if self.values.len() != FACTORS.len() {
            return Err("Incomplete key fields".into());
        }
        let mut n = [0.0; 6];
        for (slot, text) in n.iter_mut().zip(&self.values) {
            *slot = text
                .trim()
                .parse::<f64>()
                .map_err(|_| "Enter numeric key factors".to_owned())?;
            if !slot.is_finite() || !(0.0..=1.0).contains(slot) {
                return Err("All key factors must be finite values from 0 to 1".into());
            }
        }
        Ok(MaterialMotionKey {
            time_ms: time_ms(&self.key_time)?,
            base_color: [n[0], n[1], n[2], n[3]],
            roughness: n[4],
            metallic: n[5],
            interpolation: match self.interpolation.as_str() {
                "hold" => MotionInterpolation::Hold,
                "linear" => MotionInterpolation::Linear,
                "cubic_in_out" => MotionInterpolation::CubicInOut,
                _ => return Err("Choose a supported outgoing interpolation".into()),
            },
        })
    }

    // Build and validate a candidate before modifying the retained local draft.
    fn write_key(&mut self, replace: bool) -> Result<(), String> {
        let key = self.key()?;
        let old = if replace {
            Some(self.selected_key.ok_or("Select a key to update")?)
        } else {
            None
        };
        let mut candidate = self.draft.clone();
        if let Some(old) = old {
            let index = candidate
                .keys
                .iter()
                .position(|k| k.time_ms == old)
                .ok_or("The selected key is no longer in the local draft")?;
            candidate.keys.remove(index);
        }
        if candidate.keys.iter().any(|k| k.time_ms == key.time_ms) {
            return Err("A key already exists at this time; select it to update".into());
        }
        let time = key.time_ms;
        candidate.keys.push(key);
        candidate.keys.sort_by_key(|k| k.time_ms);
        candidate.validate()?;
        self.draft = candidate;
        self.selected_key = Some(time);
        self.fields_dirty = false;
        self.track_dirty = true;
        Ok(())
    }

    fn load_key(&mut self) -> Result<(), String> {
        let key = self
            .draft
            .keys
            .iter()
            .find(|k| Some(k.time_ms) == self.selected_key)
            .ok_or("Select an existing local key")?;
        self.key_time = key.time_ms.to_string();
        self.values = key
            .base_color
            .into_iter()
            .chain([key.roughness, key.metallic])
            .map(|n| n.to_string())
            .collect();
        self.interpolation = match key.interpolation {
            MotionInterpolation::Hold => "hold",
            MotionInterpolation::Linear => "linear",
            MotionInterpolation::CubicInOut => "cubic_in_out",
        }
        .into();
        self.fields_dirty = false;
        Ok(())
    }
}

#[component]
pub(super) fn MaterialMotionControls(
    selected: Signal<String>,
    inspected: Signal<Option<AuthoringView>>,
    fresh: bool,
    onedit: EventHandler<AuthoringEdit>,
) -> Element {
    let mut form = use_signal(|| None::<MotionForm>);
    let mut message = use_signal(String::new);
    let current = inspected();
    let local = form();
    let selected_part = current
        .as_ref()
        .and_then(|v| v.parts.iter().find(|p| p.id == selected()));
    let can_load = fresh && selected_part.is_some_and(|p| p.material_id != 0);
    let ready = fresh
        && local
            .as_ref()
            .is_some_and(|f| current.as_ref().is_some_and(|v| f.current(v, &selected())));
    let dirty = local.as_ref().is_some_and(MotionForm::dirty);
    let track_limit = local.as_ref().is_some_and(|f| !f.accepted)
        && current
            .as_ref()
            .is_some_and(|v| v.material_motion.len() >= MAX_TRACKS);
    rsx! {
        fieldset { style:"margin-top:10px;border:1px solid #405264;",
            legend { "Material motion · explicit editorial samples" }
            p { "Keys store linear RGBA, roughness and metallic factors for a shared material. Time is integer milliseconds. Applying a sample edits the source and preview; this is not continuous playback or organism simulation." }
            button {
                disabled:!can_load,
                onclick:move |_| {
                    if !can_load { return; }
                    if let Some(next) = inspected().as_ref().and_then(|v| MotionForm::load(v, &selected())) {
                        form.set(Some(next));
                        message.set("Accepted track and current material factors loaded. Reload explicitly replaces local fields and keys.".into());
                    }
                },
                "Load / reload accepted track (replace local draft)"
            }
            if local.is_some() && !ready {
                p { role:"status", "Source, part or material changed. Local keys and fields are retained. Reload the current selected material before editing or applying." }
            }
            if dirty {
                p { "Unsaved local motion changes. Reload replaces them; source refresh and selection changes preserve them." }
            }
            if track_limit { p { "The source already has 8 material tracks. Remove an accepted track before saving another." } }
            if let Some(f) = local {
                p { "Material {f.draft.material_id} · {f.draft.keys.len()} / 64 keys · loaded part {f.part_id}" }
                label { "Playhead in milliseconds"
                    input { disabled:!ready, value:"{f.playhead}", maxlength:8, aria_label:"Material motion playhead milliseconds",
                        oninput:move |e| if let Some(f)=form.write().as_mut() { f.playhead=e.value(); }
                    }
                }
                p { "Keys are ordered by time. Interpolation belongs to the outgoing segment; the final key has no outgoing segment." }
                ol {
                    for key in &f.draft.keys {
                        li { "{key.time_ms} ms · {key.interpolation:?} · RGBA {key.base_color:?} · roughness {key.roughness} · metallic {key.metallic}" }
                    }
                }
                label { "Selected local key"
                    select { disabled:!ready, value:f.selected_key.map(|n|n.to_string()).unwrap_or_default(), aria_label:"Selected material motion key",
                        onchange:move |e| if let Some(f)=form.write().as_mut() { f.selected_key=e.value().parse().ok(); },
                        option { value:"", "Choose a key" }
                        for key in &f.draft.keys { option { value:"{key.time_ms}", "{key.time_ms} milliseconds" } }
                    }
                }
                button { disabled:!ready||f.selected_key.is_none(), onclick:move |_| {
                    if !ready { return; }
                    if let Some(f)=form.write().as_mut() { match f.load_key() {
                        Ok(())=>message.set("Selected key loaded into fields; prior key fields explicitly replaced.".into()),
                        Err(e)=>message.set(e),
                    }}
                }, "Load selected key (replace key fields)" }
                button { disabled:!ready, onclick:move |_| {
                    if !ready { return; }
                    let values=inspected().and_then(|v|v.parts.iter().find(|p|p.id==selected()).map(factor_fields));
                    if let (Some(values),Some(f))=(values,form.write().as_mut()) {
                        f.values=values; f.fields_dirty=true;
                        message.set("Current accepted material factors copied to key fields. Add or update a key to retain them in the track.".into());
                    }
                }, "Use current accepted material factors (replace factors)" }
                label { "Key time in milliseconds"
                    input { disabled:!ready, value:"{f.key_time}", maxlength:8, aria_label:"Material motion key milliseconds",
                        oninput:move |e| if let Some(f)=form.write().as_mut() { f.key_time=e.value(); f.fields_dirty=true; }
                    }
                }
                div { style:"display:flex;flex-wrap:wrap;gap:6px;",
                    for (i,label) in FACTORS.iter().enumerate() {
                        label { "{label} (0–1)"
                            input { disabled:!ready, value:"{f.values[i]}", maxlength:32, aria_label:"Motion key {label}",
                                oninput:move |e| if let Some(f)=form.write().as_mut() { f.values[i]=e.value(); f.fields_dirty=true; }
                            }
                        }
                    }
                }
                label { "Outgoing interpolation"
                    select { disabled:!ready, value:"{f.interpolation}", aria_label:"Motion key outgoing interpolation",
                        onchange:move |e| if let Some(f)=form.write().as_mut() { f.interpolation=e.value(); f.fields_dirty=true; },
                        option { value:"hold", "Hold" }
                        option { value:"linear", "Linear" }
                        option { value:"cubic_in_out", "Cubic in/out" }
                    }
                }
                if f.fields_dirty { p { "Key fields have changes. Add a key or update the selected key before saving or sampling the track." } }
                button { disabled:!ready||f.draft.keys.len()>=MAX_KEYS, onclick:move |_| {
                    if !ready { return; }
                    if let Some(f)=form.write().as_mut() { match f.write_key(false) {
                        Ok(())=>message.set("Key added to the local draft. Apply / save track to submit it.".into()), Err(e)=>message.set(e),
                    }}
                }, "Add key from fields" }
                button { disabled:!ready||f.selected_key.is_none(), onclick:move |_| {
                    if !ready { return; }
                    if let Some(f)=form.write().as_mut() { match f.write_key(true) {
                        Ok(())=>message.set("Selected key updated and ordered in the local draft.".into()), Err(e)=>message.set(e),
                    }}
                }, "Update selected key from fields" }
                button { disabled:!ready||f.selected_key.is_none(), onclick:move |_| {
                    if !ready { return; }
                    if let Some(f)=form.write().as_mut() {
                        let selected=f.selected_key;
                        f.draft.keys.retain(|k|Some(k.time_ms)!=selected);
                        f.selected_key=None; f.track_dirty=true;
                        message.set("Selected key removed locally. A saved track needs at least one key.".into());
                    }
                }, "Remove selected key from local draft" }
                button { disabled:!ready||f.fields_dirty||track_limit||f.draft.keys.is_empty(), onclick:move |_| {
                    if !ready { return; }
                    if let Some(f)=form() {
                        let result=f.draft.validate().and_then(|_|time_ms(&f.playhead));
                        match result {
                            Ok(sample_ms) if !f.fields_dirty && !track_limit => {
                                onedit.call(AuthoringEdit::MaterialMotion {edit:MaterialMotionEdit::Set {draft:f.draft,sample_ms}});
                                message.set("Track save and explicit sample requested. Inspect accepted source for the result; local draft is retained until reload.".into());
                            },
                            Ok(_)=>message.set("Retain edited key fields and check the track limit before saving".into()),
                            Err(e)=>message.set(e),
                        }
                    }
                }, "Apply / save track and sample at playhead" }
                button { disabled:!ready||!f.accepted||dirty, onclick:move |_| {
                    if !ready { return; }
                    if let Some(f)=form().filter(|f|f.accepted&&!f.dirty()) { match time_ms(&f.playhead) {
                        Ok(sample_ms)=>{
                            onedit.call(AuthoringEdit::MaterialMotion {edit:MaterialMotionEdit::Sample {material_id:f.draft.material_id,sample_ms}});
                            message.set("Explicit accepted-track sample requested. Inspect accepted source for the result.".into());
                        }, Err(e)=>message.set(e),
                    }}
                }, "Sample / apply accepted track at playhead" }
                button { disabled:!ready||!f.accepted||dirty, onclick:move |_| {
                    if !ready { return; }
                    if let Some(f)=form().filter(|f|f.accepted&&!f.dirty()) {
                        onedit.call(AuthoringEdit::MaterialMotion {edit:MaterialMotionEdit::Remove {material_id:f.draft.material_id}});
                        message.set("Track removal requested; the shared material is kept. Inspect accepted source for the result.".into());
                    }
                }, "Remove accepted track (keep material)" }
                if dirty && f.accepted { p { "Save local key edits, or explicitly reload to discard them, before sampling or removing the accepted track." } }
            }
            p { role:"status", "{message}" }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graphical::controller::EditorHost;

    fn form() -> MotionForm {
        MotionForm {
            source: crate::graphical::authoring::tests::host().snapshot().unwrap().source,
            part_id: "body".into(),
            draft: MaterialMotionDraft {
                version: 1,
                material_id: 1,
                keys: vec![],
            },
            accepted: false,
            playhead: "0".into(),
            selected_key: None,
            key_time: "1000".into(),
            values: ["0.2", "0.3", "0.4", "1", "0.5", "0.1"]
                .into_iter()
                .map(str::to_owned)
                .collect(),
            interpolation: "linear".into(),
            track_dirty: false,
            fields_dirty: true,
        }
    }

    #[test]
    fn key_moves_are_ordered_and_collision_refusal_retains_local_source() {
        let mut f = form();
        f.write_key(false).unwrap();
        f.key_time = "0".into();
        f.write_key(false).unwrap();
        assert_eq!(
            f.draft.keys.iter().map(|k| k.time_ms).collect::<Vec<_>>(),
            [0, 1000]
        );
        assert_eq!(f.selected_key, Some(0));
        f.key_time = "1000".into();
        let before = f.draft.clone();
        assert!(f.write_key(true).is_err());
        assert_eq!(f.draft, before);
        assert_eq!(f.selected_key, Some(0));
        f.key_time = "2000".into();
        f.write_key(true).unwrap();
        assert_eq!(
            f.draft.keys.iter().map(|k| k.time_ms).collect::<Vec<_>>(),
            [1000, 2000]
        );
        assert!(f.track_dirty);
        assert!(!f.fields_dirty);
    }

    #[test]
    fn invalid_field_updates_preserve_keys_and_uncommitted_text() {
        let mut f = form();
        f.write_key(false).unwrap();
        let before = f.draft.clone();
        for bad in ["NaN", "inf", "-0.1", "1.1", "not-a-number"] {
            f.values[0] = bad.into();
            f.fields_dirty = true;
            assert!(f.write_key(true).is_err());
            assert_eq!(f.draft, before);
            assert_eq!(f.values[0], bad);
            assert!(f.fields_dirty);
        }
        assert_eq!(time_ms("86400000").unwrap(), 86_400_000);
        for bad in ["86400001", "1.5", "-1", "4294967296"] {
            assert!(time_ms(bad).is_err());
        }
        f.load_key().unwrap();
        assert_eq!(f.values[0], "0.2");
        assert!(!f.fields_dirty);
    }
}
