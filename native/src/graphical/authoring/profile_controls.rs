//! Native metric profile controls submit the same revision-bound geometry edits.
use super::{AuthoringEdit, AuthoringView, GeometryRecipe, ProfileRecipe};
use dioxus::prelude::*;

pub(super) fn recipe(kind: &str, dimensions: [&str; 6], points: &str, caps: bool) -> Result<GeometryRecipe, String> {
    let [width,height,radius,wall,segments,length] = dimensions;
    let number = |s:&str| s.parse::<f64>().map_err(|_| "Enter numeric dimensions in metres".to_string());
    let count = || segments.parse::<u32>().map_err(|_| "Enter whole arc segments in [1,64]".to_string());
    let profile = match kind {
        "rounded_rectangle" => ProfileRecipe::RoundedRectangle { width:number(width)?,height:number(height)?,radius:number(radius)?,segments:count()? },
        "open_channel" => ProfileRecipe::OpenChannel { width:number(width)?,height:number(height)?,wall:number(wall)?,corner_radius:number(radius)?,segments:count()? },
        "polyline" => {
            if points.len() > 16384 { return Err("XY point input exceeds this operator's 16384-byte bound".into()); }
            ProfileRecipe::Polyline { points:serde_json::from_str(points).map_err(|_| "Enter XY points as [[x,y],[x,y],...] in metres".to_string())? }
        }
        _ => return Err("Choose a metric profile".into()),
    };
    let geometry = GeometryRecipe::ProfileSweep { profile,length:number(length)?,caps };
    geometry.validate()?;
    Ok(geometry)
}

#[component]
pub(super) fn ProfileControls(
    selected: Signal<String>, inspected: Signal<Option<AuthoringView>>, fresh: bool,
    onedit: EventHandler<AuthoringEdit>,
) -> Element {
    let mut kind = use_signal(|| "rounded_rectangle".to_string());
    let mut width = use_signal(|| "0.4".to_string());
    let mut height = use_signal(|| "0.2".to_string());
    let mut radius = use_signal(|| "0.02".to_string());
    let mut wall = use_signal(|| "0.02".to_string());
    let mut segments = use_signal(|| "8".to_string());
    let mut length = use_signal(|| "1.0".to_string());
    let mut points = use_signal(|| "[[0,0],[0.4,0],[0.4,0.2],[0.2,0.1],[0,0.2]]".to_string());
    let mut caps = use_signal(|| true);
    let mut new_id = use_signal(String::new);
    let mut semantic = use_signal(String::new);
    let mut error = use_signal(String::new);
    let valid = fresh && inspected().is_some_and(|v| v.parts.iter().any(|p| p.id == selected()));
    rsx! {
        details { style:"margin-top:8px;", summary { "Metric profile / CAD straight sweep" }
            p { "Author a cross-section in local XY metres and sweep along local +Z. The open channel is a U-shaped solid section. This operator profile permits 1–64 arc segments, 3–256 polygon points and positive dimensions in [0.000001,1000000] metres; these working limits do not define master quality." }
            select { value:"{kind}",aria_label:"Metric profile kind",onchange:move |e|kind.set(e.value()),
                option {value:"rounded_rectangle","Rounded rectangle"}
                option {value:"open_channel","Open channel (U section)"}
                option {value:"polyline","Explicit closed polyline polygon"}
            }
            if kind() == "polyline" {
                label { "Counterclockwise XY points, implicit closing edge" textarea { value:"{points}",aria_label:"Profile XY points in metres",oninput:move |e|points.set(e.value()) } }
                p { "List each corner once. Crossings, clockwise or degenerate sections refuse. Example: [[0,0],[0.4,0],[0.4,0.2],[0,0.2]]." }
            } else {
                label { "Width (m)" input {value:"{width}",aria_label:"Profile width metres",oninput:move |e|width.set(e.value())} }
                label { "Height (m)" input {value:"{height}",aria_label:"Profile height metres",oninput:move |e|height.set(e.value())} }
                label { "Corner radius (m; zero is square)" input {value:"{radius}",aria_label:"Profile corner radius metres",oninput:move |e|radius.set(e.value())} }
                if kind() == "open_channel" { label { "Wall thickness (m)" input {value:"{wall}",aria_label:"Profile wall metres",oninput:move |e|wall.set(e.value())} } }
                label { "Arc segments" input {value:"{segments}",aria_label:"Profile arc segments",oninput:move |e|segments.set(e.value())} }
            }
            label { "Sweep length (m)" input {value:"{length}",aria_label:"Profile sweep length metres",oninput:move |e|length.set(e.value())} }
            label { input {r#type:"checkbox",checked:caps(),aria_label:"Cap profile sweep ends",onchange:move |e|caps.set(e.checked())} "Cap ends (concave polygon triangulation)" }
            button {disabled:!valid,onclick:move |_|match recipe(&kind(),[&width(),&height(),&radius(),&wall(),&segments(),&length()],&points(),caps()) {
                Ok(geometry)=>{error.set(String::new());onedit.call(AuthoringEdit::ReplaceGeometry {id:selected(),geometry});},Err(e)=>error.set(e),
            },"Replace selected with profile sweep"}
            label {"New part identifier" input {value:"{new_id}",aria_label:"New profile part identifier",oninput:move |e|new_id.set(e.value())} }
            label {"New semantic identifier" input {value:"{semantic}",aria_label:"New profile semantic identifier",oninput:move |e|semantic.set(e.value())} }
            button {disabled:!valid,onclick:move |_| {
                let next=recipe(&kind(),[&width(),&height(),&radius(),&wall(),&segments(),&length()],&points(),caps()).and_then(|g|semantic().parse::<u64>().map(|id|(g,id)).map_err(|_|"Enter a positive unused semantic identifier".to_string()));
                match next {Ok((geometry,semantic_id))=>{error.set(String::new());onedit.call(AuthoringEdit::AddGeometry {id:new_id(),semantic_id,material_from:selected(),geometry});},Err(e)=>error.set(e)}
            },"Add profile part with selected material"}
            p { "Actual host grants and budgets decide execution. New parts use identity TRS. Replacement retains other part fields/material and refuses unsupported topology-bound data. Parameter rebuild restores the retained original recipe; undo retains this source edit." }
            if !error().is_empty() {p {role:"status","{error}"}}
        }
    }
}
