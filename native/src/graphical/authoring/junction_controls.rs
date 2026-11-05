//! Typed single-junction controls; all source parameters are entered by a person.
use super::{AuthoringEdit, botanical::{BotanicalEdit, BotanicalLimits, BotanicalRecipe, JunctionSource}};
use crate::graphical::controller::EditorSnapshot;
use crate::graphical::preview::SourceRef;
use dioxus::prelude::*;

#[component]
pub(super) fn JunctionControls(
    accepted: Option<JunctionSource>, selected_part: String, source: SourceRef, fresh: bool,
    live_view: Signal<EditorSnapshot>, live_selected: Signal<String>, onedit: EventHandler<AuthoringEdit>,
) -> Element {
    let mut graph = use_signal(|| None::<(SourceRef, String, BotanicalRecipe)>);
    let mut values = use_signal(|| std::array::from_fn::<_, 12, _>(|_| String::new()));
    let mut settings = use_signal(|| std::array::from_fn::<_, 13, _>(|_| String::new()));
    let mut settings_context = use_signal(|| None::<(SourceRef, String)>);
    let mut organ = use_signal(String::new);
    let mut position = use_signal(|| std::array::from_fn::<_, 3, _>(|_| String::new()));
    let mut status = use_signal(String::new);
    let mut smooth_normals = use_signal(|| false);
    let current = graph();
    let source_for_settings = source.clone();
    let source_for_load = source.clone();
    let selected_for_settings = selected_part.clone();
    let selected_for_load = selected_part.clone();
    let accepted_for_load = accepted.clone();
    let source_for_edit = source.clone();
    let source_for_graph = source.clone();
    let selected_for_edit = selected_part.clone();
    let selected_for_graph = selected_part.clone();
    let accepted_graph = accepted.as_ref().and_then(|s| s.graph(BotanicalLimits::default()).ok());
    let matches_selection = accepted_graph.as_ref().is_some_and(|g| g.part_id == selected_part);
    let has_junction = accepted.is_some();
    let labels = ["Grid origin X (m)","Grid origin Y (m)","Grid origin Z (m)","Grid spacing (m)",
        "Grid X samples","Grid Y samples","Grid Z samples","Blend (m)","Minimum axis length (m)",
        "Projection tolerance (m)","Maximum same-direction cosine","Minimum squared face cross product (m⁴)"];
    rsx! {
        fieldset { style:"margin-top:10px;border:1px solid #405264;",
            legend { "Single branch junction" }
            p { "Enter your adopted graph and sampling parameters. Adoption replaces the selected part geometry in source history. Existing vertex UVs, normals, skin and indexed atlas seams need explicit migration. Planar UV settings are retained." }
            p { "Reopening a saved junction currently requires starting Poet with POET_NATIVE_BOTANICAL_PROFILE=1; this enables the bounded replay allowance." }
            if !has_junction {
                label { input { r#type:"checkbox", checked:smooth_normals(), disabled:!fresh,
                    onchange:move |e| smooth_normals.set(e.checked()) }
                    "Regenerate smooth area-weighted normals after each junction edit" }
                super::botanical::typed_controls::TypedBotanicalControls {
                    selected_part: selected_part.clone(), source: source.clone(), fresh, has_graph: false,
                    live_view, live_selected,
                    onedit: move |edit| if let BotanicalEdit::SetRecipe { recipe } = edit {
                        if recipe.version != 1 { status.set("Select the v1 graph profile for junction input.".into()); return; }
                        graph.set(Some((source_for_graph.clone(), selected_for_graph.clone(), recipe)));
                        status.set("Graph draft captured. Enter all junction parameters.".into());
                    },
                }
                if current.is_some() { p { "Graph draft captured for this source and part." } }
                for (i,label) in labels.iter().enumerate() {
                    label { "{label}" input { value:"{values()[i]}", disabled: !fresh || current.is_none(),
                        aria_label:"{label}", oninput: move |e| { let v=e.value(); if v.len()<=64 { values.write()[i]=v; } } } }
                }
                button { disabled: !fresh || selected_part.is_empty() || current.is_none(),
                    onclick: move |_| {
                        let live = live_view();
                        let Some((draft_source, draft_part, recipe)) = graph() else { status.set("Enter a graph first.".into()); return; };
                        if !fresh || live.reconciliation_required || live.source != draft_source || live_selected() != draft_part {
                            status.set("Source or selected part changed; retain the draft and restart on the current source.".into()); return;
                        }
                        let v=values();
                        let parsed = (|| -> Result<JunctionSource,String> {
                            let num = |i:usize| -> Result<f64,String> { let n:f64=v[i].parse().map_err(|_|"Enter all finite junction numbers")?; if !n.is_finite() { return Err("Junction values must be finite".into()); } Ok(n) };
                            let dim = |i:usize| -> Result<usize,String> { v[i].parse().map_err(|_|"Grid sample counts must be whole numbers".into()) };
                            JunctionSource::from_recipe(&recipe,[num(0)?,num(1)?,num(2)?],num(3)?,[dim(4)?,dim(5)?,dim(6)?],
                                num(7)?,num(8)?,num(9)?,num(10)?,num(11)?,BotanicalLimits::default(),4096)
                        })();
                        match parsed.map(|s| if smooth_normals() { s.with_regenerated_normals() } else { s })
                            .and_then(|s| s.to_value(BotanicalLimits::default(),4096)) {
                            Ok(source) => onedit.call(AuthoringEdit::BotanicalJunctionAdopt {source}), Err(e) => status.set(e),
                        }
                    }, "Adopt junction on selected part" }
            } else if matches_selection {
                if accepted.as_ref().is_some_and(|s| s.regenerates_normals()) {
                    p { "Normals: regenerate smooth area-weighted shading from each accepted junction shape. Planar UV mapping can follow topology changes; indexed UVs and atlas seams require exact correspondence." }
                } else {
                    p { "Normals: legacy source policy. Authored normals block shape changes until explicitly migrated." }
                }
                if let Some(g) = accepted_graph {
                    p { "Accepted junction: {g.part_id}" }
                    for item in g.organs.iter() { p { "Organ {item.key}: {item.position_m:?}" } }
                }
                label { "Organ identifier" input { value:"{organ}", disabled:!fresh, oninput:move |e| {let v=e.value();if v.len()<=128 {organ.set(v);} } } }
                for (i,label) in ["X (m)","Y (m)","Z (m)"].iter().enumerate() {
                    label { "{label}" input { value:"{position()[i]}", disabled:!fresh,
                        oninput: move |e| { let v=e.value();if v.len()<=64 {position.write()[i]=v;} } } }
                }
                button { disabled: !fresh || organ().is_empty(), onclick: move |_| {
                    let live=live_view();
                    if !fresh || live.reconciliation_required || live.source != source_for_edit || live_selected() != selected_for_edit {
                        status.set("Source or selected part changed; refresh before editing.".into()); return;
                    }
                    let input=position(); let mut point=[0.0f64;3];
                    for i in 0..3 { match input[i].parse::<f64>() { Ok(v) if v.is_finite() => point[i]=v,
                        _ => {status.set("Enter finite coordinates in metres.".into()); return;} } }
                    onedit.call(AuthoringEdit::BotanicalJunctionPosition { target:selected_for_edit.clone(), organ:organ(), position_m:point });
                }, "Move junction organ" }
                button { disabled: !fresh, onclick: move |_| {
                    let live=live_view();
                    if live.reconciliation_required || live.source != source_for_load || live_selected() != selected_for_load {
                        status.set("Source or selected part changed; refresh before loading settings.".into()); return;
                    }
                    let Some(current)=accepted_for_load.clone() else {status.set("No accepted junction.".into());return;};
                    let Ok(g)=current.graph(BotanicalLimits::default()) else {status.set("Accepted graph cannot be read.".into());return;};
                    if g.part_id != selected_for_load {status.set("Select the bound part.".into());return;}
                    settings.set([format!("{:?}",current.origin_m[0]), format!("{:?}",current.origin_m[1]),
                        format!("{:?}",current.origin_m[2]),format!("{:?}",current.spacing_m),
                        current.dimensions[0].to_string(),current.dimensions[1].to_string(),current.dimensions[2].to_string(),
                        format!("{:?}",current.blend_m),format!("{:?}",current.min_axis_m),
                        format!("{:?}",current.projection_tolerance_m),format!("{:?}",current.max_same_direction_cosine),
                        format!("{:?}",current.min_area_sq_m4),format!("{:?}",g.tip_radius_m)]);
                    settings_context.set(Some((source_for_load.clone(),selected_for_load.clone())));
                    status.set("Current settings loaded for editing.".into());
                }, "Load current junction settings" }
                if settings_context().is_some() {
                    for (i,label) in labels.iter().enumerate() {
                        label { "{label}" input { value:"{settings()[i]}", disabled:!fresh,
                            oninput:move |e| {let v=e.value();if v.len()<=64 {settings.write()[i]=v;}} } }
                    }
                    label { "Tip radius (m)" input { value:"{settings()[12]}", disabled:!fresh,
                        oninput:move |e| {let v=e.value();if v.len()<=64 {settings.write()[12]=v;}} } }
                    button { disabled:!fresh, onclick:move |_| {
                        let live=live_view();
                        if live.reconciliation_required || live.source != source_for_settings || live_selected() != selected_for_settings
                            || settings_context() != Some((source_for_settings.clone(),selected_for_settings.clone())) {
                            status.set("Source or selected part changed; reload settings before applying.".into());return;
                        }
                        let v=settings();
                        let parsed=(|| -> Result<super::botanical::JunctionSettings,String> {
                            let n=|i:usize| -> Result<f64,String> {let x:f64=v[i].parse().map_err(|_|"Enter all finite settings")?;if !x.is_finite(){return Err("Settings must be finite".into());}Ok(x)};
                            let d=|i:usize| -> Result<usize,String> {v[i].parse().map_err(|_|"Grid sample counts must be whole numbers".into())};
                            Ok(super::botanical::JunctionSettings { origin_m:[n(0)?,n(1)?,n(2)?],spacing_m:n(3)?,dimensions:[d(4)?,d(5)?,d(6)?],
                                blend_m:n(7)?,min_axis_m:n(8)?,projection_tolerance_m:n(9)?,max_same_direction_cosine:n(10)?,min_area_sq_m4:n(11)?,tip_radius_m:n(12)? })
                        })();
                        match parsed {Ok(settings)=>onedit.call(AuthoringEdit::BotanicalJunctionSettings {target:selected_for_settings.clone(),settings}),Err(error)=>status.set(error)}
                    }, "Apply junction settings" }
                }            } else { p { "Select the part bound to the accepted junction to edit it." } }
            p { role:"status", "{status}" }
        }
    }
}
