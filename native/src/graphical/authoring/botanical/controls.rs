//! Selected-organ inspector for the admitted native Poet authoring surface.
//! Dispatch is typed; the parent controller still owns actor, revision and source.
use super::{BotanicalEdit, BotanicalLimits, BotanicalRecipe};
use crate::graphical::preview::SourceRef;
use dioxus::prelude::*;

const MAX_ADOPTION_JSON_BYTES: usize = 64 * 1024;

#[derive(Clone)]
struct AdoptionDraft {
    source: SourceRef,
    part_id: String,
    json: String,
}
impl AdoptionDraft {
    fn edit(&self) -> Result<BotanicalEdit, String> {
        if self.json.is_empty() || self.json.len() > MAX_ADOPTION_JSON_BYTES {
            return Err("Enter a bounded botanical recipe before adoption".into());
        }
        let recipe: BotanicalRecipe = serde_json::from_str(&self.json)
            .map_err(|e| format!("Botanical recipe JSON is invalid: {e}"))?;
        recipe.validate(BotanicalLimits::default())?;
        if recipe.part_id != self.part_id {
            return Err("Recipe part_id must match the selected part".into());
        }
        Ok(BotanicalEdit::SetRecipe { recipe })
    }
}

#[derive(Clone)]
struct Form {
    source: SourceRef,
    recipe: BotanicalRecipe,
    organ_key: String,
    tip_radius: String,
    position: [String; 3],
}
impl Form {
    fn load(source: SourceRef, recipe: BotanicalRecipe) -> Self {
        let organ_key = recipe.organs[0].key.clone();
        let position = recipe.organs[0].position_m.map(|n| n.to_string());
        let tip_radius = recipe.tip_radius_m.to_string();
        Self {
            source,
            recipe,
            organ_key,
            tip_radius,
            position,
        }
    }
    fn select(&mut self, key: &str) {
        if let Some(o) = self.recipe.organs.iter().find(|o| o.key == key) {
            self.organ_key = key.to_owned();
            self.position = o.position_m.map(|n| n.to_string());
        }
    }
    fn position_edit(&self) -> Result<BotanicalEdit, String> {
        let mut p = [0.0; 3];
        for (i, text) in self.position.iter().enumerate() {
            p[i] = text
                .parse::<f64>()
                .map_err(|_| "Enter finite metre coordinates")?;
            if !p[i].is_finite() || p[i].abs() > 1.0e6 {
                return Err("Organ position outside finite metre bound".into());
            }
        }
        Ok(BotanicalEdit::SetOrganPosition {
            part_id: self.recipe.part_id.clone(),
            organ_key: self.organ_key.clone(),
            position_m: p,
        })
    }
    fn radius_edit(&self) -> Result<BotanicalEdit, String> {
        let radius = self
            .tip_radius
            .parse::<f64>()
            .map_err(|_| "Enter a finite tip radius")?;
        if !radius.is_finite() || radius <= 0.0 || radius > 1.0e6 {
            return Err("Tip radius outside positive metre bound".into());
        }
        Ok(BotanicalEdit::SetTipRadius {
            part_id: self.recipe.part_id.clone(),
            tip_radius_m: radius,
        })
    }
}

#[component]
pub fn BotanicalControls(
    accepted: Option<BotanicalRecipe>,
    selected_part: String,
    source: SourceRef,
    fresh: bool,
    onedit: EventHandler<BotanicalEdit>,
) -> Element {
    let mut form = use_signal(|| None::<Form>);
    let mut adoption = use_signal(|| None::<AdoptionDraft>);
    let mut message = use_signal(String::new);
    let current = form();
    let adoption_current = adoption();
    let adoption_ready = fresh
        && adoption_current
            .as_ref()
            .is_some_and(|d| d.source == source && d.part_id == selected_part);
    let adoption_accepted = accepted.clone();
    let adoption_part = selected_part.clone();
    let adoption_source = source.clone();
    let rebind_part = selected_part.clone();
    let rebind_source = source.clone();
    let inspector_part = selected_part.clone();
    let can_load = fresh
        && accepted
            .as_ref()
            .is_some_and(|r| r.part_id == selected_part);
    let ready = fresh
        && current
            .as_ref()
            .is_some_and(|f| f.source == source && f.recipe.part_id == selected_part);
    rsx! {
        fieldset { style:"margin-top:10px;border:1px solid #405264;",
            legend { "Botanical adopted graph" }
            p { "Advanced graph entry. Adopt an explicit source recipe for the selected part, then select its organ keys below. No organ is created until you apply a recipe." }
            button { disabled:!fresh || selected_part.is_empty(), onclick:move |_| {
                let json = adoption_accepted.as_ref().filter(|r| r.part_id == adoption_part)
                    .and_then(|r| serde_json::to_string_pretty(r).ok()).unwrap_or_default();
                adoption.set(Some(AdoptionDraft { source:adoption_source.clone(),
                    part_id:adoption_part.clone(), json }));
                message.set("Recipe entry bound to current source. Loading replaces local recipe text.".into());
            }, "Load accepted recipe / start blank entry" }
            if adoption_current.is_some() && !adoption_ready {
                p { role:"status", "Source or selected part changed. Recipe text is retained but cannot be applied until explicitly rebound." }
                button { disabled:!fresh || selected_part.is_empty(), onclick:move |_| {
                    if let Some(draft) = adoption.write().as_mut() {
                        draft.source = rebind_source.clone();
                        draft.part_id = rebind_part.clone();
                        message.set("Local recipe text rebound to the current selected source; check its part_id before applying.".into());
                    }
                }, "Rebind retained recipe text to current source" }
            }
            if let Some(draft) = adoption_current {
                label { "Versioned botanical recipe JSON"
                    textarea { disabled:!adoption_ready, value:"{draft.json}", maxlength:65536,
                        rows:12, cols:72, aria_label:"Botanical adopted graph recipe JSON",
                        oninput:move |e| {
                            let next=e.value();
                            if next.len() <= MAX_ADOPTION_JSON_BYTES {
                                if let Some(draft)=adoption.write().as_mut() { draft.json=next; }
                            } else { message.set("Recipe text exceeds 64 KiB".into()); }
                        }
                    }
                }
                button { disabled:!adoption_ready, onclick:move |_| {
                    if let Some(draft)=adoption().as_ref() {
                        match draft.edit() { Ok(edit)=>onedit.call(edit), Err(error)=>message.set(error) }
                    }
                }, "Adopt graph for selected part" }
            }
            p { "The organ inspector below edits an accepted graph. Colonization output remains an unadopted proposal until a keyed recipe is explicitly submitted." }
            button { disabled:!can_load, onclick:move |_| {
                if let Some(recipe) = accepted.clone().filter(|r| r.part_id == inspector_part) {
                    form.set(Some(Form::load(source.clone(), recipe)));
                    message.set("Accepted botanical graph loaded. Reload replaces local fields.".into());
                }
            }, "Load / reload accepted graph" }
            if current.is_some() && !ready {
                p { role:"status", "Source or selected part changed. Local fields are retained; reload before submitting." }
            }
            if let Some(f) = current {
                p { "{f.recipe.organs.len()} adopted organs · selected part {f.recipe.part_id}" }
                label { "Organ key"
                    select { disabled:!ready, value:"{f.organ_key}",
                        onchange:move |e| if let Some(f) = form.write().as_mut() { f.select(&e.value()); },
                        for organ in &f.recipe.organs {
                            option { value:"{organ.key}", "{organ.key}" }
                        }
                    }
                }
                for i in 0..3 {
                    label { "Position {i} (m)"
                        input { disabled:!ready, value:"{f.position[i]}",
                            oninput:move |e| if let Some(f) = form.write().as_mut() { f.position[i] = e.value(); }
                        }
                    }
                }
                button { disabled:!ready, onclick:move |_| {
                    if let Some(f) = form().as_ref() {
                        match f.position_edit() { Ok(edit) => onedit.call(edit), Err(e) => message.set(e) }
                    }
                }, "Apply organ position" }
                label { "Tip radius (m)"
                    input { disabled:!ready, value:"{f.tip_radius}",
                        oninput:move |e| if let Some(f) = form.write().as_mut() { f.tip_radius = e.value(); }
                    }
                }
                button { disabled:!ready, onclick:move |_| {
                    if let Some(f) = form().as_ref() {
                        match f.radius_edit() { Ok(edit) => onedit.call(edit), Err(e) => message.set(e) }
                    }
                }, "Apply pipe radius" }
            }
            p { role:"status", "{message}" }
        }
    }
}
