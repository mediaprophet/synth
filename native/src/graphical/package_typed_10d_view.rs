//! Snapshot inspection and guarded visual derivatives; authored masters use source edits.
use crate::graphical::{
    checkpoints::{
        typed_10d::{Typed10dEditRequest, Typed10dInspection},
        ResourceInspectionRequest,
    },
    controller::EditorSnapshot,
    jobs::{self, Action, Context, HostHandle},
};
use dioxus::prelude::*;
#[derive(Clone)]
struct Bound {
    context: Context,
    value: Typed10dInspection,
}
#[allow(clippy::too_many_arguments)]
fn queue(
    host: HostHandle,
    current: EditorSnapshot,
    action: Action,
    mut panel: Signal<Option<Bound>>,
    mut busy: Signal<bool>,
    mut status: Signal<String>,
    on_saved: EventHandler<()>,
    edit: bool,
) {
    if busy() || !current.publication_is_current() {
        return;
    }
    busy.set(true);
    panel.set(None);
    let origin = Context::from_snapshot(&current);
    let guard = host.clone();
    spawn(async move {
        match jobs::run(host,action,origin.clone()).await {
            Ok(reply) if jobs::completion_is_current(&guard,&current,&reply)=>{
                if edit && reply.operation_succeeded {on_saved.call(());}
                panel.set(reply.typed_10d.map(|value|Bound{context:origin,value}));
                status.set(reply.message);
            }
            Ok(_)=>status.set("Source/session changed; .10d result withheld. Refresh source and package inventory.".into()),
            Err(error)=>status.set(error),
        }
        busy.set(false);
    });
}
#[component]
pub(super) fn Typed10dControls(
    host: HostHandle,
    current: EditorSnapshot,
    request: ResourceInspectionRequest,
    busy: Signal<bool>,
    mut status: Signal<String>,
    on_saved: EventHandler<()>,
) -> Element {
    let panel = use_signal(|| None::<Bound>);
    let mut selected = use_signal(|| 0u64);
    let mut factors = use_signal(|| vec![String::new(); 9]);
    let value = panel()
        .filter(|p| {
            p.context == Context::from_snapshot(&current)
                && current.publication_is_current()
                && p.value.request == request
        })
        .map(|p| p.value);
    let inspect_host = host.clone();
    let inspect_current = current.clone();
    let inspect_request = request.clone();
    let save_host = host.clone();
    let save_current = current.clone();
    rsx! {
        details{
            summary{"Inspect .10d materials, frames and scientific fields"}
            button{disabled:busy(),onclick:move |_|queue(inspect_host.clone(),inspect_current.clone(),
                Action::InspectTyped10d{request:inspect_request.clone(),expected:inspect_current.source.clone()},panel,busy,status,on_saved,false),
                "Inspect selected .10d snapshot"}
            if let Some(value)=value{
                p{"Version {value.version}; {value.vertex_count} vertices; {value.index_count} indices; {value.materials.len()} materials; {value.ranges.len()} ranges; {value.fields.len()} field declarations."}
                p{"Frames: {value.frames:?}"}
                p{"This is the selected saved digest snapshot. Reinspect after any package change. Source association does not prove regeneration of a directly edited visual derivative."}
                if let Some(blocker)=value.edit_blocker.clone(){
                    p{"Editing held: {blocker}"}
                    p{"For an authored assembly, edit its accepted source through assembly/material controls and compile it again. This updates the compiled resource and its graph together."}
                }else{
                    select{value:"{selected}",disabled:busy(),aria_label:"Selected persisted .10d material",onchange:move|event|{
                        let id=event.value().parse::<u64>().unwrap_or(0);selected.set(id);
                        if let Some(m)=panel().as_ref().and_then(|v|v.value.materials.iter().find(|m|m.id==id).copied()){
                            let mut f=m.base_color.iter().map(|v|v.to_string()).collect::<Vec<_>>();
                            f.extend(m.emissive.iter().map(|v|v.to_string()));f.push(m.metallic.to_string());f.push(m.roughness.to_string());factors.set(f);
                        }
                    },option{value:"0","Choose a material"}for m in value.materials{option{value:"{m.id}","Material {m.id}"}}}
                    for (index,label) in ["Linear red","Linear green","Linear blue","Alpha","Emissive red","Emissive green","Emissive blue","Metallic","Roughness"].iter().enumerate(){
                        label{"{label}" input{disabled:busy()||selected()==0,value:"{factors()[index]}",aria_label:"Persisted material {label}",oninput:move|event|factors.write()[index]=event.value()}}
                    }
                    button{disabled:busy()||selected()==0,onclick:move |_|{
                        let Some(view)=panel().filter(|p|p.context==Context::from_snapshot(&save_current)&&p.value.request==request)else{return;};
                        let numbers=factors().iter().map(|v|v.parse::<f32>().map_err(|_|"Enter finite material factors")).collect::<Result<Vec<_>,_>>();
                        match numbers{
                            Ok(n)=>{
                                let edit=Typed10dEditRequest{resource:request.clone(),source_digest:view.value.source_digest,material_id:selected(),
                                    base_color:[n[0],n[1],n[2],n[3]],emissive:[n[4],n[5],n[6]],metallic:n[7],roughness:n[8]};
                                if let Err(error)=edit.validate_bounds(){status.set(error);return;}
                                queue(save_host.clone(),save_current.clone(),Action::EditTyped10d{request:edit,expected:save_current.source.clone()},panel,busy,status,on_saved,true);
                            }
                            Err(error)=>status.set(error.into()),
                        }
                    },"Save guarded visual material derivative"}
                }
            }
        }
    }
}
