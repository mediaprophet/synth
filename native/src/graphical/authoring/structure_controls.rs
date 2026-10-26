//! Explicit typed authoring intents; all authority, validation and execution remain host-owned.
use super::{AuthoringEdit, AuthoringView, GeometryRecipe};
use dioxus::prelude::*;

fn recipe(kind: &str, radius: &str, extra: &str, first: &str, second: &str, caps: bool) -> Result<GeometryRecipe, String> {
    let dimension = |s: &str| s.parse::<f64>().map_err(|_| "Enter numeric geometry dimensions".to_string());
    let count = |s: &str| s.parse::<u32>().map_err(|_| "Enter whole segment counts".to_string());
    match kind {
        "sphere" => Ok(GeometryRecipe::Sphere { radius: dimension(radius)?, lat_segments: count(first)?, lon_segments: count(second)? }),
        "torus" => Ok(GeometryRecipe::Torus { major_radius: dimension(radius)?, minor_radius: dimension(extra)?, major_segments: count(first)?, minor_segments: count(second)? }),
        "sweep" => Ok(GeometryRecipe::CircularSweep { radius: dimension(radius)?, length: dimension(extra)?, segments: count(first)?, caps }),
        _ => Err("Choose a supported geometry operator".into()),
    }
}

#[component]
pub(super) fn StructureControls(
    mut selected: Signal<String>,
    inspected: Signal<Option<AuthoringView>>,
    fresh: bool,
    onedit: EventHandler<AuthoringEdit>,
) -> Element {
    let mut parent = use_signal(String::new);
    let mut unique = use_signal(String::new);
    let mut kind = use_signal(|| "sphere".to_string());
    let mut radius = use_signal(|| "0.25".to_string());
    let mut extra = use_signal(|| "0.1".to_string());
    let mut first = use_signal(|| "16".to_string());
    let mut second = use_signal(|| "24".to_string());
    let mut caps = use_signal(|| true);
    let mut new_id = use_signal(String::new);
    let mut semantic = use_signal(String::new);
    let mut error = use_signal(String::new);
    let data = inspected();
    let parts = data.map(|v| v.parts).unwrap_or_default();
    let valid = fresh && parts.iter().any(|p| p.id == selected());
    let current_parent = parts.iter().find(|p| p.id == selected()).and_then(|p| p.parent.as_deref()).unwrap_or("unattached").to_string();
    rsx! {
        details { style:"margin-top:8px;", summary { "Containment and selected material" }
            p { "Current parent: {current_parent}. Reparent and detach retain local metres/radians/TRS exactly; world placement may change. Containment must stay acyclic." }
            select { disabled:!valid, value:"{parent}", aria_label:"New assembly parent", onchange:move |e| parent.set(e.value()),
                option { value:"", "Choose parent" }
                for part in parts.iter().filter(|p| p.id != selected()) { option { value:"{part.id}", "{part.id}" } }
            }
            button { disabled:!valid||parent().is_empty(), onclick:move |_| onedit.call(AuthoringEdit::Reparent { id:selected(), parent:Some(parent()) }), "Reparent, keep local transform" }
            button { disabled:!valid, onclick:move |_| onedit.call(AuthoringEdit::Reparent { id:selected(), parent:None }), "Detach, keep local transform" }
            button { disabled:!valid, onclick:move |_| { onedit.call(AuthoringEdit::RemoveLeaf { id:selected() }); selected.set(String::new()); }, "Remove selected leaf" }
            p { "Removal refuses children and the final part. Undo retains the previous source revision." }
            input { disabled:!valid, value:"{unique}", aria_label:"Unused material identifier", placeholder:"Unused material identifier", oninput:move |e|unique.set(e.value()) }
            button { disabled:!valid, onclick:move |_|match unique().parse::<u64>() {
                Ok(material_id)=>{ onedit.call(AuthoringEdit::UniqueMaterial { id:selected(),material_id }); selected.set(String::new()); error.set("Reselect the part to inspect its material identifier after the command completes.".into()); },
                Err(_)=>error.set("Enter a positive unused material identifier.".into()),
            }, "Make selected material unique" }
            p { "Copies every admitted material field to an unused identifier on this part. Reselect before editing its factors. Texture and physical-material admission remain outside the current factor-material profile." }
        }
        details { style:"margin-top:8px;", summary { "Add or replace parametric geometry" }
            p { "Dimensions are metres in [0.000001,1000000]; segment counts are whole numbers in [3,256]. The native host applies its actual grants and remaining resource budgets. Replacement changes the geometry record; other part fields and its material remain." }
            select { value:"{kind}", aria_label:"Parametric geometry operator", onchange:move |e|kind.set(e.value()),
                option { value:"sphere", "UV sphere" } option { value:"torus", "Torus" } option { value:"sweep", "Straight circular profile sweep along local Z" }
            }
            label { "Radius / torus major radius" input { value:"{radius}", aria_label:"Geometry radius",oninput:move |e|radius.set(e.value()) } }
            if kind() != "sphere" { label { "Torus minor radius / sweep length" input { value:"{extra}",aria_label:"Geometry minor radius or length",oninput:move |e|extra.set(e.value()) } } }
            label { "Latitude / major / profile segments" input { value:"{first}", aria_label:"Geometry first segment count",oninput:move |e|first.set(e.value()) } }
            if kind() != "sweep" { label { "Longitude / minor segments" input { value:"{second}", aria_label:"Geometry second segment count",oninput:move |e|second.set(e.value()) } } }
            if kind() == "sweep" { label { input { r#type:"checkbox",checked:caps(),aria_label:"Cap circular sweep ends",onchange:move |e|caps.set(e.checked()) } "Cap sweep ends" } }
            button { disabled:!valid, onclick:move |_|match recipe(&kind(),&radius(),&extra(),&first(),&second(),caps()) {
                Ok(geometry)=>{error.set(String::new());onedit.call(AuthoringEdit::ReplaceGeometry {id:selected(),geometry});},Err(e)=>error.set(e),
            }, "Replace selected geometry" }
            label { "New part identifier" input {value:"{new_id}",aria_label:"New geometry part identifier",oninput:move |e|new_id.set(e.value())} }
            label { "New semantic identifier" input {value:"{semantic}",aria_label:"New geometry semantic identifier",oninput:move |e|semantic.set(e.value())} }
            button { disabled:!valid, onclick:move |_| {
                let next=recipe(&kind(),&radius(),&extra(),&first(),&second(),caps()).and_then(|geometry|semantic().parse::<u64>().map(|id|(geometry,id)).map_err(|_|"Enter a positive unused semantic identifier".to_string()));
                match next { Ok((geometry,semantic_id))=>{error.set(String::new());onedit.call(AuthoringEdit::AddGeometry {id:new_id(),semantic_id,material_from:selected(),geometry});},Err(e)=>error.set(e) }
            }, "Add unattached part with selected material" }
            p { "New parts use identity local TRS and copy the selected part's complete material. Choose the new part after completion to transform or reparent it. Parameter rebuild regenerates the original recipe and replaces these overrides." }
        }
        if !error().is_empty() { p { role:"status", "{error}" } }
    }
}
