//! Native part inspector and explicit source commands sharing the existing worker.
use super::{AuthoringEdit, AuthoringView};
use crate::graphical::{
    controller::EditorSnapshot,
    jobs::{self, Action, Context, HostHandle},
    preview::SourceRef,
};
use dioxus::prelude::*;

fn fields(values: &[f64]) -> Vec<String> {
    values.iter().map(|v| v.to_string()).collect()
}
fn numbers<const N: usize>(values: &[String]) -> Result<[f64; N], String> {
    if values.len() != N {
        return Err("Incomplete authoring fields".into());
    }
    let mut out = [0.0; N];
    for (i, s) in values.iter().enumerate() {
        out[i] = s
            .parse::<f64>()
            .map_err(|_| "Enter finite numeric authoring values")?;
        if !out[i].is_finite() {
            return Err("Authoring values must be finite".into());
        }
    }
    Ok(out)
}
#[allow(clippy::too_many_arguments)]
fn load_part_form(
    data: &AuthoringView,
    key: &str,
    source: SourceRef,
    mut transform: Signal<Vec<String>>,
    mut material: Signal<Vec<String>>,
    mut material_id: Signal<u64>,
    mut loaded: Signal<Option<(String, SourceRef)>>,
) -> bool {
    if data.source != source {
        return false;
    }
    let Some(part) = data.parts.iter().find(|part| part.id == key) else {
        return false;
    };
    let mut trs = fields(&part.translation);
    trs.extend(fields(&part.rotation));
    trs.extend(fields(&part.scale));
    transform.set(trs);
    let mut mat = fields(&part.base_color);
    mat.push(part.roughness.to_string());
    mat.push(part.metallic.to_string());
    material.set(mat);
    material_id.set(part.material_id);
    loaded.set(Some((key.to_string(), source)));
    true
}
#[allow(clippy::too_many_arguments)]
fn send(
    action: Action,
    host: HostHandle,
    mut view: Signal<EditorSnapshot>,
    mut draft: Signal<String>,
    mut status: Signal<String>,
    mut image: Signal<Option<String>>,
    mut busy: Signal<bool>,
    mut inspected: Signal<Option<AuthoringView>>,
    buffer: Option<crate::graphical::draft_buffer::DraftBuffer>,
    edit: bool,
) {
    if busy() {
        return;
    }
    if edit && draft() != view().latest_source {
        status.set("Unsubmitted draft preserved. Submit it or explicitly load accepted source before using part controls.".into());
        return;
    }
    let capture = if edit {
        match crate::graphical::draft_buffer::DraftCapture::capture(draft(), buffer.as_ref()) {
            Ok(capture) => Some(capture),
            Err(error) => {
                status.set(error);
                return;
            }
        }
    } else {
        None
    };
    let origin = Context::from_snapshot(&view());
    let guard = host.clone();
    busy.set(true);
    status.set("Native authoring command queued against the displayed source.".into());
    spawn(async move {
        match crate::graphical::native_dispatch::run(host, action, origin).await {
            Ok(reply) if jobs::completion_is_current(&guard, &view(), &reply) => {
                let epoch_changed = reply
                    .snapshot
                    .as_ref()
                    .is_some_and(|s| s.execution_epoch != view().execution_epoch);
                let mut retained_local_draft = false;
                let mut source_changed = false;
                if let Some(snapshot) = reply.snapshot {
                    source_changed =
                        snapshot.source != view().source || snapshot.reconciliation_required;
                    if let Some(capture) = &capture {
                        if let Some(text) = capture.accept(
                            reply.operation_succeeded,
                            &draft(),
                            &snapshot.latest_source,
                            buffer.as_ref(),
                        ) {
                            draft.set(text);
                        } else if reply.operation_succeeded {
                            retained_local_draft = true;
                        }
                    }
                    view.set(snapshot);
                }
                if source_changed || epoch_changed {
                    image.set(reply.image_uri);
                } else if reply.image_uri.is_some() {
                    image.set(reply.image_uri);
                }
                if let Some(authoring) = reply.authoring {
                    inspected.set(Some(authoring));
                }
                status.set(if retained_local_draft {
                    format!("{}; newer unsubmitted draft preserved", reply.message)
                } else {
                    reply.message
                });
            }
            Ok(_) => {
                image.set(None);
                status.set("Source or contributor changed; late authoring result withheld.".into())
            }
            Err(error) => {
                // A lost worker reply may follow committed source changes.
                image.set(None);
                status.set(error);
            }
        }
        busy.set(false);
    });
}

#[component]
pub(crate) fn AuthoringControls(
    host: HostHandle,
    view: Signal<EditorSnapshot>,
    draft: Signal<String>,
    mut status: Signal<String>,
    image: Signal<Option<String>>,
    busy: Signal<bool>,
    mut selected: Signal<String>,
    mut rig_filename: Signal<String>,
    mut rig_package_digest: Signal<String>,
    mut rig_resource_key: Signal<String>,
    mut rig_resource_digest: Signal<String>,
    mut rig_ready_source: Signal<Option<(Context, String)>>,
    #[props(default)] buffer: Option<crate::graphical::draft_buffer::DraftBuffer>,
) -> Element {
    let inspected = use_signal(|| None::<AuthoringView>);
    let mut loaded_part = use_signal(|| None::<(String, SourceRef)>);
    let mut transform = use_signal(|| fields(&[0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0]));
    let mut material = use_signal(|| fields(&[0.5, 0.5, 0.5, 1.0, 0.5, 0.0]));
    let material_id = use_signal(|| 0u64);
    let mut parameter = use_signal(String::new);
    let mut parameter_value = use_signal(String::new);
    let mut parameter_context = use_signal(|| None::<(String, SourceRef)>);
    let mut filename = use_signal(|| "growing-station-derived.hmc".to_string());
    let mut asset_key = use_signal(|| "growing-station-game-v1".to_string());
    let mut expected_digest = use_signal(String::new);
    let mut rig_bind_id = use_signal(String::new);
    let current = view();
    let fresh = inspected()
        .as_ref()
        .is_some_and(|v| v.source == current.source)
        && !current.reconciliation_required;
    use_effect(move || {
        let key = selected();
        if key.is_empty() {
            loaded_part.set(None);
            return;
        }
        // A changed source with the same part key is stale, not a new selection:
        // retain the local form until the user explicitly reloads it.
        if loaded_part()
            .as_ref()
            .is_some_and(|(loaded_key, _)| loaded_key == &key)
        {
            return;
        }
        let current = view();
        if current.reconciliation_required {
            return;
        }
        if let Some(data) = inspected()
            .as_ref()
            .filter(|data| data.source == current.source)
        {
            let _ = load_part_form(
                data,
                &key,
                current.source,
                transform,
                material,
                material_id,
                loaded_part,
            );
        }
    });
    let loaded = loaded_part();
    let form_fresh = fresh
        && !selected().is_empty()
        && loaded
            .as_ref()
            .is_some_and(|(key, source)| key == &selected() && source == &current.source);
    let parameter_fresh = fresh
        && !parameter().is_empty()
        && parameter_context()
            .as_ref()
            .is_some_and(|(key, source)| key == &parameter() && source == &current.source);
    let inspect_host = host.clone();
    let inspect_buffer = buffer.clone();
    let transform_host = host.clone();
    let transform_buffer = buffer.clone();
    let material_host = host.clone();
    let material_buffer = buffer.clone();
    let layers_host = host.clone();
    let layers_buffer = buffer.clone();
    let layers_source = current.source.clone();
    let param_host = host.clone();
    let param_buffer = buffer.clone();
    let structure_host = host.clone();
    let structure_buffer = buffer.clone();
    let compile_host = host.clone();
    let compile_buffer = buffer.clone();
    let profile_host = host.clone();
    let profile_buffer = buffer.clone();
    let profile_source = current.source.clone();
    let cad_host = host.clone();
    let cad_buffer = buffer.clone();
    let cad_source = current.source.clone();
    let surface_host = host.clone();
    let surface_buffer = buffer.clone();
    let surface_source = current.source.clone();
    let motion_host = host.clone();
    let motion_buffer = buffer.clone();
    let motion_source = current.source.clone();
    let rig_source = current.source.clone();
    let rig_host = host.clone();
    let rig_buffer = buffer.clone();
    let botanical_source = current.source.clone();
    let botanical_host = host.clone();
    let botanical_buffer = buffer.clone();
    let binding_host = host.clone();
    let binding_buffer = buffer.clone();
    let binding_source = current.source.clone();
    let structure_source = current.source.clone();
    let transform_source = current.source.clone();
    let material_source = current.source.clone();
    let param_source = current.source.clone();
    let compile_source = current.source.clone();
    let data = inspected();
    let tools_host = host.clone();
    let tools_source = current.source.clone();
    let tools_buffer = buffer.clone();
    rsx! {
        fieldset{disabled:busy(),style:"margin-top:12px;padding:10px;border:1px solid #405264;min-width:0;",
            legend{"Assembly · parts, profiles and materials"}
            button{disabled:busy()||current.reconciliation_required,onclick:move |_|send(Action::AuthoringInspect(view().source),inspect_host.clone(),view,draft,status,image,busy,inspected,inspect_buffer.clone(),false),"Inspect accepted assembly"}
            p{style:"font-size:12px;","Parent-relative transforms use metres/radians. Edits are checked Vibe source revisions; shared material edits affect every part using that material."}
            super::surface_binding_controls::SurfaceBindingControls{busy,onbind:move|request|send(Action::BindSurfacePackage{request,expected:binding_source.clone()},binding_host.clone(),view,draft,status,image,busy,inspected,binding_buffer.clone(),false)}
            if let Some(data)=data{
                p{"{data.parts.len()} parts · source {data.source.artifact_id}"}
                select{disabled:!fresh||busy(),value:"{selected}",aria_label:"Selected assembly part",onchange:move|event|{
                    let key=event.value();
                    if key.is_empty(){
                        loaded_part.set(None);
                    }else{
                        let current=view();
                        if !current.reconciliation_required{
                            if let Some(data)=inspected().as_ref(){let _=load_part_form(data,&key,current.source,transform,material,material_id,loaded_part);}
                        }
                    }
                    selected.set(key);
                },option{value:"","Choose a part"}for part in data.parts{option{value:"{part.id}","{part.id} · material {part.material_id}"}}}
                button{disabled:!fresh||selected().is_empty()||busy(),onclick:move |_|{
                    let key=selected();let current=view();
                    if !current.reconciliation_required{
                      if let Some(data)=inspected().as_ref(){
                        if !load_part_form(data,&key,current.source,transform,material,material_id,loaded_part){status.set("Could not reload part values from the current inspection.".into());}
                      }
                    }
                },"Reload inspected part values (replace local form)"}
                if !selected().is_empty()&&!form_fresh{
                    p{style:"color:#f0c674;","Local transform/material values are not bound to the current inspected source. Your form is preserved; reload to replace it with current inspected values before applying."}
                }
                div{style:"display:flex;flex-wrap:wrap;gap:6px;margin-top:8px;",
                    for (index,label) in ["Translate X","Translate Y","Translate Z","Rotate X","Rotate Y","Rotate Z","Scale X","Scale Y","Scale Z"].iter().enumerate(){
                        label{style:"width:100px;font-size:12px;","{label}" input{disabled:!fresh||selected().is_empty(),value:"{transform()[index]}",aria_label:"{label}",style:"width:94px;",oninput:move|e|transform.write()[index]=e.value()}}
                    }
                }
                button{disabled:!form_fresh||selected().is_empty(),onclick:move |_|{
                    match numbers::<9>(&transform()){
                        Ok(n)=>send(Action::AuthoringEdit{edit:AuthoringEdit::Transform{id:selected(),translation:[n[0],n[1],n[2]],rotation:[n[3],n[4],n[5]],scale:[n[6],n[7],n[8]]},expected:transform_source.clone()},transform_host.clone(),view,draft,status,image,busy,inspected,transform_buffer.clone(),true),
                        Err(e)=>status.set(e),
                    }
                },"Apply part transform"}
                super::profile_controls::ProfileControls{selected,inspected,fresh,onedit:move|edit|send(Action::AuthoringEdit{edit,expected:profile_source.clone()},profile_host.clone(),view,draft,status,image,busy,inspected,profile_buffer.clone(),true)}
                super::cad_controls::CadControls{selected,inspected,fresh,onedit:move|edit|send(Action::AuthoringEdit{edit,expected:cad_source.clone()},cad_host.clone(),view,draft,status,image,busy,inspected,cad_buffer.clone(),true)}
                super::surface_tools::SurfaceTools{host:tools_host.clone(),view,selected,inspected,fresh: fresh && !busy(),busy,status,
                    onedit:move|edit|send(Action::AuthoringEdit{edit,expected:tools_source.clone()},tools_host.clone(),view,draft,status,image,busy,inspected,tools_buffer.clone(),true)}
                super::surface_controls::SurfaceControls{selected,inspected,fresh,onedit:move|edit|send(Action::AuthoringEdit{edit,expected:surface_source.clone()},surface_host.clone(),view,draft,status,image,busy,inspected,surface_buffer.clone(),true)}
                if let Some(descriptor) = data.rig.clone() {
                    label { "Rig HMC filename" input { value:"{rig_filename}", oninput:move|e|{rig_filename.set(e.value());rig_ready_source.set(None);} } }
                    label { "Inspected HMC SHA256" input { value:"{rig_package_digest}", oninput:move|e|{rig_package_digest.set(e.value());rig_ready_source.set(None);} } }
                    p { "Select this exact rig part in the viewport; inspect and save the matching current source and HMC before applying. The viewport supports manual pose preview and stepping." }
                    {let transfer_key=descriptor.resource_key.clone(); let transfer_digest=descriptor.resource_sha256.clone(); let transfer_part=descriptor.part_id.clone(); let transfer_source=data.source.clone(); rsx! {
                        button { disabled:!fresh || busy() || selected()!=transfer_part, onclick:move |_| {
                            if view().source != transfer_source || selected()!=transfer_part || !view().publication_is_current() {
                                status.set("Inspect the current source and select its bound part before transferring the rig resource.".into());
                            } else {
                                rig_resource_key.set(transfer_key.clone()); rig_resource_digest.set(transfer_digest.clone()); rig_ready_source.set(None);
                                status.set("Rig resource copied to the viewport controls. Confirm the package and resource for this source before preview.".into());
                            }
                        }, "Use inspected rig in viewport" }
                    }}
                    {let rig_key = descriptor.resource_key.clone(); let rig_digest = descriptor.resource_sha256.clone(); let enabled = fresh && !busy() && selected() == descriptor.part_id; rsx! {
                        crate::graphical::rig_binding::RigControls { descriptor, busy: !enabled, on_control: move |control| {
                            let request = (|| -> Result<crate::graphical::checkpoints::ResourceInspectionRequest,String> {
                                Ok(crate::graphical::checkpoints::ResourceInspectionRequest { filename: rig_filename(), package_digest: parse_digest(rig_package_digest().trim())?, key: rig_key.clone(), resource_digest: parse_digest(&rig_digest)? })
                            })();
                            match request { Ok(resource) => send(Action::AuthoringEdit { edit: AuthoringEdit::Rig { resource, control }, expected: rig_source.clone() }, rig_host.clone(), view,draft,status,image,busy,inspected,rig_buffer.clone(),true), Err(error)=>status.set(error) }
                        } }
                    }}
                } else {
                    label { "Rig HMC filename" input { value:"{rig_filename}", oninput:move|e|{rig_filename.set(e.value());rig_ready_source.set(None);} } }
                    label { "Inspected HMC SHA256" input { value:"{rig_package_digest}", oninput:move|e|{rig_package_digest.set(e.value());rig_ready_source.set(None);} } }
                    label { "Selected .10d resource key" input { value:"{rig_resource_key}", oninput:move|e|{rig_resource_key.set(e.value());rig_ready_source.set(None);} } }
                    label { "Inspected resource SHA256" input { value:"{rig_resource_digest}", oninput:move|e|{rig_resource_digest.set(e.value());rig_ready_source.set(None);} } }
                    label { "Local rig binding identifier" input { value:"{rig_bind_id}", oninput:move|e|rig_bind_id.set(e.value()) } }
                    p { "Select the same part in the viewport and assembly list. Adoption requires an exact bind mesh and uses the first available clip; it preserves an existing binding and refuses unsupported geometry." }
                    button { disabled: !fresh || busy() || selected().is_empty(), onclick: move |_| {
                        let request = (|| -> Result<crate::graphical::checkpoints::ResourceInspectionRequest,String> {
                            let resource = crate::graphical::checkpoints::ResourceInspectionRequest { filename:rig_filename(), package_digest:parse_digest(rig_package_digest().trim())?, key:rig_resource_key(), resource_digest:parse_digest(rig_resource_digest().trim())? };
                            resource.validate_bounds()?;
                            Ok(resource)
                        })();
                        match request {
                            Ok(resource) => send(Action::AuthoringEdit { edit:AuthoringEdit::Rig { resource, control:crate::graphical::rig_binding::RigControl::Adopt { part_id:selected(), bind_id:rig_bind_id() } }, expected:rig_source.clone() },rig_host.clone(),view,draft,status,image,busy,inspected,rig_buffer.clone(),true),
                            Err(error) => status.set(error),
                        }
                    }, "Adopt selected rig binding" }
                }
                super::botanical::BotanicalControls {
                    accepted: data.botanical.clone(), selected_part: selected(), source: botanical_source.clone(),
                    fresh: fresh && !busy(), live_view: view, live_selected: selected,
                    onedit: move |edit| send(Action::AuthoringEdit {edit:AuthoringEdit::Botanical {edit}, expected:botanical_source.clone()}, botanical_host.clone(),view,draft,status,image,busy,inspected,botanical_buffer.clone(),true),
                }
                super::material_motion_controls::MaterialMotionControls{selected,inspected,fresh,onedit:move|edit|send(Action::AuthoringEdit{edit,expected:motion_source.clone()},motion_host.clone(),view,draft,status,image,busy,inspected,motion_buffer.clone(),true)}
                super::structure_controls::StructureControls {
                    selected, inspected, fresh,
                    onedit: move |edit| send(Action::AuthoringEdit { edit, expected: structure_source.clone() }, structure_host.clone(), view, draft, status, image, busy, inspected, structure_buffer.clone(), true),
                }
                div{style:"display:flex;flex-wrap:wrap;gap:6px;margin-top:8px;",
                    for (index,label) in ["Linear red","Linear green","Linear blue","Alpha","Roughness","Metallic"].iter().enumerate(){
                        label{style:"width:100px;font-size:12px;","{label}" input{disabled:!fresh||selected().is_empty()||material_id()==0,value:"{material()[index]}",aria_label:"Assembly material {label}",style:"width:94px;",oninput:move|e|material.write()[index]=e.value()}}
                    }
                }
                button{disabled:!form_fresh||selected().is_empty()||material_id()==0,onclick:move |_|{
                    match numbers::<6>(&material()){
                        Ok(n)=>send(Action::AuthoringEdit{edit:AuthoringEdit::Material{material_id:material_id(),base_color:[n[0],n[1],n[2],n[3]],roughness:n[4],metallic:n[5]},expected:material_source.clone()},material_host.clone(),view,draft,status,image,busy,inspected,material_buffer.clone(),true),
                        Err(e)=>status.set(e),
                    }
                },"Apply shared material"}
                if !data.parameters.is_empty(){
                    h4{"Profile and geometry parameters"}
                    button{disabled:!fresh,onclick:move |_|send(Action::AuthoringEdit{edit:AuthoringEdit::EnableSourceLayers,expected:layers_source.clone()},layers_host.clone(),view,draft,status,image,busy,inspected,layers_buffer.clone(),true),"Enable non-destructive source layers"}
                    select{disabled:!fresh,value:"{parameter}",aria_label:"Authored geometry parameter",onchange:move|event|{
                        let key=event.value();parameter_context.set(None);let current=view();
                        if !key.is_empty()&&!current.reconciliation_required{
                            if let Some((_,value))=inspected().as_ref().filter(|data|data.source==current.source).and_then(|v|v.parameters.iter().find(|(k,_)|k==&key).cloned()){
                                parameter_value.set(value.to_string());parameter_context.set(Some((key.clone(),current.source.clone())));
                            }
                        }
                        parameter.set(key);
                    },option{value:"","Choose a parameter"}for (key,value) in data.parameters{option{value:"{key}","{key} = {value}"}}}
                    input{disabled:!fresh,value:"{parameter_value}",aria_label:"Geometry parameter value",oninput:move|event|{
                        parameter_value.set(event.value());let current=view();
                        if !parameter().is_empty()&&!current.reconciliation_required&&inspected().as_ref().is_some_and(|data|data.source==current.source){parameter_context.set(Some((parameter(),current.source.clone())));}else{parameter_context.set(None);}
                    }}
                    if !parameter().is_empty()&&!parameter_fresh{
                        p{style:"color:#f0c674;","Parameter value belongs to an older source context. Reselect the parameter or edit its value after inspecting the current source before rebuilding."}
                    }
                    button{disabled:!parameter_fresh,onclick:move |_|match parameter_value().parse::<f64>(){
                        Ok(value) if value.is_finite()=>send(Action::AuthoringEdit{edit:AuthoringEdit::Parameter{key:parameter(),value},expected:param_source.clone()},param_host.clone(),view,draft,status,image,busy,inspected,param_buffer.clone(),true),
                        _=>status.set("Enter a finite geometry parameter value.".into()),
                    },"Rebuild parts from parameter"}
                    p{style:"font-size:12px;","Enable source layers for an existing procedural recipe, then change dimensions while retaining material and transform overrides. Changes without qualified geometry correspondence preserve the current source by refusing regeneration; UV/detail topology remapping remains unqualified."}
                }
            }
            details{summary{"Compile accepted assembly into HMC"}
                label{"Package filename" input{value:"{filename}",aria_label:"Compiled HMC filename",oninput:move|e|filename.set(e.value())}}
                label{"Asset key" input{value:"{asset_key}",aria_label:"Compiled assembly asset key",oninput:move|e|asset_key.set(e.value())}}
                label{"Inspected package SHA256 (empty creates a new file)" input{value:"{expected_digest}",aria_label:"Compile expected package digest",oninput:move|e|expected_digest.set(e.value())}}
                button{disabled:current.reconciliation_required,onclick:move |_|{
                    let text=expected_digest();let file=if text.trim().is_empty(){Ok(qualia_core_db::poet_host::invoke::studio_store::checkpoint::ExpectedFile::Absent)}else{parse_digest(text.trim()).map(qualia_core_db::poet_host::invoke::studio_store::checkpoint::ExpectedFile::Sha256)};
                    match file{Ok(file)=>send(Action::CompileAssembly{name:filename(),key:asset_key(),file,expected:compile_source.clone()},compile_host.clone(),view,draft,status,image,busy,inspected,compile_buffer.clone(),false),Err(e)=>status.set(e)}
                },"Compile source, materials and parts into HMC"}
                p{style:"font-size:12px;","Compiles the accepted assembly without evaluating source again. Retained resources survive; existing compiled entries require the exact inspected package digest. Source review does not approve the complete package. Changed compiled entries require a new asset key, retaining all prior sections and graph facts."}
            }
        }
    }
}
fn parse_digest(text: &str) -> Result<[u8; 32], String> {
    if text.len() != 64 || !text.is_ascii() {
        return Err("Expected a 64-character hexadecimal SHA256".into());
    }
    let mut result = [0; 32];
    for (i, slot) in result.iter_mut().enumerate() {
        *slot = u8::from_str_radix(&text[i * 2..i * 2 + 2], 16)
            .map_err(|_| "Invalid package SHA256")?;
    }
    Ok(result)
}
