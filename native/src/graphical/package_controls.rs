//! Native controls over the same current, authorised editor worker.
use super::{
    checkpoints::PackageInventory,
    controller::EditorSnapshot,
    jobs::{self, Action, Context, HostHandle},
    package::{hex_digest, ImportRequest},
};
use dioxus::prelude::*;
#[path = "package_inspection_view.rs"]
mod inspection_view;
use inspection_view::M64InspectionView;
#[path = "package_georaster_edit_view.rs"]
mod georaster_edit_view;
use georaster_edit_view::GeoRasterEditView;
#[path = "package_georaster_create_view.rs"]
mod georaster_create_view;
use georaster_create_view::GeoRasterCreateView;
#[path = "package_typed_10d_view.rs"]
mod typed_10d_view;
use typed_10d_view::Typed10dControls;
#[derive(Clone)]
struct InspectedResource {
    context: Context,
    result: super::checkpoints::ResourceInspection,
}
#[derive(Clone)]
struct InspectedPackage {
    context: Context,
    package: PackageInventory,
}

fn command(
    action: Action,
    host: HostHandle,
    current: EditorSnapshot,
    mut inventory: Signal<Option<InspectedPackage>>,
    mut status: Signal<String>,
    mut busy: Signal<bool>,
) {
    if busy() {
        return;
    }
    busy.set(true);
    inventory.set(None);
    let origin = Context::from_snapshot(&current);
    let guard = host.clone();
    spawn(async move {
        match jobs::run(host,action,origin).await {
            Ok(reply) if jobs::completion_is_current(&guard,&current,&reply)=>{
                inventory.set(reply.inventory.map(|package|InspectedPackage{context:Context::from_snapshot(&current),package}));
                status.set(reply.message);
            }
            Ok(_)=>status.set("Source/session/authority changed; resource result withheld. Refresh source and inventory before another edit.".into()),
            Err(error)=>status.set(error),
        }
        busy.set(false);
    });
}

#[component]
pub(super) fn PackageControls(
    host: HostHandle,
    current: EditorSnapshot,
    busy: Signal<bool>,
    status: Signal<String>,
) -> Element {
    let mut name = use_signal(|| "poet-checkpoint.hmc".to_string());
    let mut filename = use_signal(String::new);
    let mut key = use_signal(String::new);
    let mut kind = use_signal(|| "asset".to_string());
    let mut inventory = use_signal(|| None::<InspectedPackage>);
    let mut inspection = use_signal(|| None::<InspectedResource>);
    let entries = inventory()
        .filter(|view| {
            view.context == Context::from_snapshot(&current) && current.publication_is_current()
        })
        .map(|view| view.package);
    let inventory_available = entries.is_some();
    let scan = {
        let host = host.clone();
        let current = current.clone();
        move |_| {
            command(
                Action::Inventory {
                    name: name(),
                    expected: current.source.clone(),
                },
                host.clone(),
                current.clone(),
                inventory,
                status,
                busy,
            );
        }
    };
    let import = {
        let host = host.clone();
        let current = current.clone();
        move |_| {
            if let Some(package) = inventory()
                .filter(|view| {
                    view.context == Context::from_snapshot(&current)
                        && current.publication_is_current()
                })
                .map(|view| view.package)
            {
                let expected_resource = package
                    .resources
                    .iter()
                    .find(|r| r.key == key())
                    .map(|r| r.digest);
                command(
                    Action::ImportResource {
                        name: name(),
                        file_digest: package.digest,
                        request: ImportRequest {
                            filename: filename(),
                            key: key(),
                            kind: kind(),
                            expected_resource,
                        },
                        expected: current.source.clone(),
                    },
                    host.clone(),
                    current.clone(),
                    inventory,
                    status,
                    busy,
                );
            }
        }
    };
    rsx! {
        section{style:"padding:12px;border:1px solid #465463;display:flex;flex-direction:column;gap:8px;",
            h2{style:"margin:0;font-size:19px;","HMC resources"}
            p{"Inspect a saved package before editing. Unedited resources and metadata are retained. Published releases require a new working copy."}
            div{style:"display:flex;gap:8px;flex-wrap:wrap;",
                input{value:"{name}",aria_label:"HMC resource package filename",disabled:busy(),
                    oninput:move|e|{name.set(e.value());inventory.set(None);}}
                button{disabled:busy(),onclick:scan,"Inspect resources"}
            }
            if let Some(package)=entries {
                crate::graphical::package_review_controls::PackageReviewControls{
                    host:host.clone(),current:current.clone(),filename:name(),digest:package.digest,busy,status
                }

                p{style:"overflow-wrap:anywhere;","Inspected package snapshot SHA256 {hex_digest(&package.digest)}"}
                if let Some(view)=inspection().filter(|view| {
                    view.context==Context::from_snapshot(&current) && current.publication_is_current()
                        && view.result.matches_inventory(&name(), &package)
                }) {
                    M64InspectionView{view:view.result.clone()}
                    {let key=format!("{}:{}:{}:{}",view.result.request.filename,view.result.request.key,
                        hex_digest(&view.result.request.package_digest),hex_digest(&view.result.request.resource_digest));
                        rsx!{GeoRasterEditView{key:"{key}",view:view.result,host:host.clone(),current:current.clone(),busy,status,
                            on_saved:move |_|{inventory.set(None);inspection.set(None);}}}}


                }
                {let host=host.clone();let current=current.clone();let digest=package.digest;
                    rsx!{button{disabled:busy(),onclick:move |_|{
                        command(Action::SaveInspected{name:name(),file_digest:digest,expected:current.source.clone()},host.clone(),current.clone(),inventory,status,busy);
                    },"Save accepted source into this HMC"}}
                }
                {let key=format!("{}:{}",name(),hex_digest(&package.digest));
                    rsx!{GeoRasterCreateView{key:"{key}",host:host.clone(),current:current.clone(),filename:name(),package_digest:package.digest,
                        busy,status,on_saved:move |_|{inventory.set(None);inspection.set(None);}}}}

                for resource in package.resources {
                    {let host=host.clone();let current=current.clone();let remove_key=resource.key.clone();let digest=resource.digest;let file_digest=package.digest;
                    let inspect_host=host.clone();let inspect_current=current.clone();let inspect_key=resource.key.clone();
                    rsx!{div{style:"padding:8px;background:#0c141c;overflow-wrap:anywhere;",
                        strong{"{resource.key}"}
                        p{"{resource.kind} | {resource.bytes} bytes | {resource.metadata_bytes} metadata bytes"}
                        p{"SHA256 {hex_digest(&digest)}"}
                        {let key=format!("{}:{}:{}",name(),resource.key,hex_digest(&package.digest));
                            rsx!{Typed10dControls{key:"{key}",host:host.clone(),current:current.clone(),
                                request:super::checkpoints::ResourceInspectionRequest{filename:name(),package_digest:file_digest,key:resource.key.clone(),resource_digest:digest},
                                busy,status,on_saved:move |_|{inventory.set(None);inspection.set(None);}}}}

                        button{disabled:busy(),onclick:move |_|{
                            if busy(){return;}
                            busy.set(true);inspection.set(None);
                            let host=inspect_host.clone();let guard=host.clone();let current=inspect_current.clone();
                            let request=super::checkpoints::ResourceInspectionRequest{filename:name(),package_digest:file_digest,key:inspect_key.clone(),resource_digest:digest};
                            let origin=Context::from_snapshot(&current);
                            spawn(async move {
                                match jobs::run(host,Action::InspectResource{request,expected:current.source.clone()},origin.clone()).await {
                                    Ok(reply) if jobs::completion_is_current(&guard,&current,&reply)=>{
                                        inspection.set(reply.inspection.map(|result|InspectedResource{context:origin,result}));status.set(reply.message);
                                    }
                                    Ok(_)=>status.set("Source/session/authority changed; selected resource details withheld. Refresh inventory.".into()),
                                    Err(error)=>status.set(error),
                                }
                                busy.set(false);
                            });
                        },"Inspect M64 metadata"}
                        button{disabled:busy(),onclick:move |_|{
                            command(Action::RemoveResource{name:name(),file_digest,key:remove_key.clone(),resource_digest:digest,expected:current.source.clone()},
                                host.clone(),current.clone(),inventory,status,busy);
                        },"Remove this resource"}
                    }}}
                }
            }
            div{style:"display:flex;gap:8px;flex-wrap:wrap;",
                input{value:"{filename}",placeholder:"Filename in native import library",aria_label:"Native resource library filename",disabled:busy(),oninput:move|e|filename.set(e.value())}
                input{value:"{key}",placeholder:"Resource key",aria_label:"Resource key",disabled:busy(),oninput:move|e|key.set(e.value())}
                input{value:"{kind}",placeholder:"Kind for new resource",aria_label:"Resource kind for additions",disabled:busy(),oninput:move|e|kind.set(e.value())}
                button{disabled:busy() || !inventory_available,onclick:import,"Import or replace contents"}
            }
            p{"Imports use the native host's selected library. Replacement keeps the existing resource kind and metadata. Rendering and model acceptance require separate checks."}
        }
    }
}
