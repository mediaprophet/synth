//! Native surface-of-revolution controls over the shared authoring command path.
use super::{AuthoringEdit, AuthoringView, GeometryRecipe};
use dioxus::prelude::*;
pub(super) fn recipe(points: &str, segments: &str) -> Result<GeometryRecipe, String> {
    if points.len() > 16384 {
        return Err("Radius-height input exceeds the 16384-byte operator bound".into());
    }
    let geometry = GeometryRecipe::RevolveProfile {
        points: serde_json::from_str(points).map_err(|_| {
            "Enter metre pairs [[radius,height],...] from top to bottom".to_string()
        })?,
        segments: segments
            .parse()
            .map_err(|_| "Enter whole radial segments in [3,256]".to_string())?,
    };
    geometry.validate()?;
    Ok(geometry)
}
#[component]
pub(super) fn CadControls(
    selected: Signal<String>,
    inspected: Signal<Option<AuthoringView>>,
    fresh: bool,
    onedit: EventHandler<AuthoringEdit>,
) -> Element {
    let mut points = use_signal(|| "[[0.2,1.0],[0.35,0.5],[0.25,0.0]]".to_string());
    let mut segments = use_signal(|| "24".to_string());
    let mut new_id = use_signal(String::new);
    let mut semantic = use_signal(String::new);
    let mut error = use_signal(String::new);
    let valid = fresh && inspected().is_some_and(|v| v.parts.iter().any(|p| p.id == selected()));
    rsx! {
        details {style:"margin-top:8px;",summary {"CAD surface of revolution · open ends"}
            p {"Rotate a radius-height polyline through one full turn around local +Y. List 2–256 stations from top to bottom with strictly decreasing height and positive radius. Radius and height are metres; heights must remain distinct at native output precision."}
            label {"Radius-height pairs (m)" textarea {value:"{points}",aria_label:"Revolution radius height pairs metres",oninput:move|e|points.set(e.value())}}
            label {"Radial segments" input {value:"{segments}",aria_label:"Revolution radial segments",oninput:move|e|segments.set(e.value())}}
            p {"The angular seam closes. Both profile ends remain open: this operation does not add end caps, poles or thickness. Only a full turn and strictly decreasing heights are supported."}
            button {disabled:!valid,onclick:move |_|match recipe(&points(),&segments()) {
                Ok(geometry)=>{error.set(String::new());onedit.call(AuthoringEdit::ReplaceGeometry {id:selected(),geometry});},Err(e)=>error.set(e),
            },"Replace selected with open revolution surface"}
            label {"New part identifier" input {value:"{new_id}",aria_label:"New revolution part identifier",oninput:move|e|new_id.set(e.value())}}
            label {"New semantic identifier" input {value:"{semantic}",aria_label:"New revolution semantic identifier",oninput:move|e|semantic.set(e.value())}}
            button {disabled:!valid,onclick:move |_| {
                let next=recipe(&points(),&segments()).and_then(|g|semantic().parse::<u64>().map(|id|(g,id)).map_err(|_|"Enter a positive unused semantic identifier".to_string()));
                match next {Ok((geometry,semantic_id))=>{error.set(String::new());onedit.call(AuthoringEdit::AddGeometry {id:new_id(),semantic_id,material_from:selected(),geometry});},Err(e)=>error.set(e)}
            },"Add revolution part with selected material"}
            p {"Current grants and working limits govern execution. New parts start at identity TRS; replacement retains part/material fields and refuses unsupported topology data. Cancellation cannot interrupt an individual revolution operation. These operator bounds do not set master quality."}
            if !error().is_empty() {p {role:"status","{error}"}}
        }
    }
}
