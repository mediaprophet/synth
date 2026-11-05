//! Native terrain operator form over an inspected HMC resource and current intent context.
use crate::graphical::{checkpoints::ResourceInspection,controller::EditorSnapshot,jobs::{self,Action,Context,HostHandle},terrain_brush::{TerrainBrushDraft,TerrainStrokeDraft,TerrainOperation,TerrainFalloff}};
use dioxus::prelude::*;

#[component]
pub(super) fn TerrainBrushView(host:HostHandle,current:EditorSnapshot,view:ResourceInspection,busy:Signal<bool>,status:Signal<String>,on_saved:EventHandler<()>) -> Element {
    let meta=view.payload.georaster;
    let ready=current.publication_is_current();
    let mut column=use_signal(||"0".to_string());let mut row=use_signal(||"0".to_string());
    let mut side=use_signal(||"1".to_string());
    let mut center_x=use_signal(||"0".to_string());let mut center_y=use_signal(||"0".to_string());
    let mut radius=use_signal(move||meta.map(|m|(m.cell_size_mm as f64/1000.0).to_string()).unwrap_or_else(||"1".into()));
    let mut strength=use_signal(||"1".to_string());let mut target=use_signal(String::new);let mut step=use_signal(String::new);
    let mut operation=use_signal(||"raise".to_string());let mut falloff=use_signal(||"smooth_step".to_string());
    let apply=move |_| {
        if busy() || !current.publication_is_current() {return;}
        let draft=(||->Result<TerrainBrushDraft,String>{
            let op=TerrainOperation::parse(&operation())?;
            let number=|text:String|text.parse::<f64>().map_err(|_|"Enter finite metre values".to_string());
            let draft=TerrainBrushDraft {resource:view.request.clone(),window_x:column().parse().map_err(|_|"Column requires a nonnegative integer")?,window_y:row().parse().map_err(|_|"Row requires a nonnegative integer")?,window_side:side().parse().map_err(|_|"Window side requires an integer from 1 to 64")?,strokes:vec![TerrainStrokeDraft {
                operation:op,falloff:TerrainFalloff::parse(&falloff())?,center_m:[number(center_x())?,number(center_y())?],radius_m:number(radius())?,strength:number(strength())?,
                target_height_m:if op==TerrainOperation::Flatten{Some(number(target())?)}else{None},step_m:if op==TerrainOperation::Terrace{Some(number(step())?)}else{None},
            }]};draft.validate()?;Ok(draft)
        })();
        let draft=match draft {Ok(d)=>d,Err(e)=>{status.set(e);return;}};
        busy.set(true);let host=host.clone();let guard=host.clone();let current=current.clone();
        let origin=Context::from_snapshot(&current);
        spawn(async move {
            match crate::graphical::native_dispatch::run(host,Action::BrushGeoRaster{draft,expected:current.source.clone()},origin).await {
                Ok(reply) if jobs::completion_is_current(&guard,&current,&reply)=>{if reply.operation_succeeded {on_saved.call(());}status.set(reply.message);},
                Ok(_)=>status.set("Terrain context changed; refresh source and inventory before another stroke. A started write may need reconciliation.".into()),
                Err(e)=>status.set(e),
            }busy.set(false);
        });
    };
    rsx! {if let Some(meta)=meta {fieldset {disabled:busy() || !ready,
        legend {"Terrain brush"}
        p {"Select a finite square within this {meta.width} × {meta.height} raster. Spacing is {meta.cell_size_mm} mm. Local X/Y follow its persisted row order; heights use the retained datum."}
        label {"Start column " input {value:"{column}",aria_label:"Terrain window start column",oninput:move|e|column.set(e.value())}}
        label {"Start row " input {value:"{row}",aria_label:"Terrain window start row",oninput:move|e|row.set(e.value())}}
        label {"Window side (1–64 cells) " input {value:"{side}",aria_label:"Terrain square window side",oninput:move|e|side.set(e.value())}}
        label {"Operation " select {value:"{operation}",aria_label:"Terrain brush operation",onchange:move|e|operation.set(e.value()),
            option {value:"raise","Raise"} option {value:"lower","Lower"} option {value:"flatten","Flatten"} option {value:"smooth","Smooth"} option {value:"terrace","Terrace"}}}
        label {"Falloff " select {value:"{falloff}",aria_label:"Terrain brush falloff",onchange:move|e|falloff.set(e.value()),
            option {value:"smooth_step","Smooth step"} option {value:"gaussian","Gaussian"} option {value:"linear","Linear"} option {value:"plateau","Plateau"}}}
        label {"Local centre X (m) " input {value:"{center_x}",aria_label:"Terrain local centre X metres",oninput:move|e|center_x.set(e.value())}}
        label {"Local centre Y (m) " input {value:"{center_y}",aria_label:"Terrain local centre Y metres",oninput:move|e|center_y.set(e.value())}}
        label {"Radius (m) " input {value:"{radius}",aria_label:"Terrain brush radius metres",oninput:move|e|radius.set(e.value())}}
        label {"Strength (metres for raise/lower; 0–1 otherwise) " input {value:"{strength}",aria_label:"Terrain brush strength",oninput:move|e|strength.set(e.value())}}
        if operation()=="flatten" {label {"Absolute target height (m) " input {value:"{target}",aria_label:"Terrain absolute flatten height metres",oninput:move|e|target.set(e.value())}}}
        if operation()=="terrace" {label {"Terrace step (m) " input {value:"{step}",aria_label:"Terrain positive terrace step metres",oninput:move|e|step.set(e.value())}}}
        p {"Nodata within the selected window refuses the stroke. Smooth clamps at window edges. Saving retains the datum, origin, spacing, source annotation and other resources; refresh the inventory afterward."}
        button {onclick:apply,"Apply stroke and save working copy"}
    }}}
}
