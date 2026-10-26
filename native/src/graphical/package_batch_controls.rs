//! Reviewed command drafts submit one atomic native resource plan.
use super::{
    checkpoints::PackageInventory,
    controller::EditorSnapshot,
    jobs::{self, Action, Context, HostHandle},
    package::{hex_digest, parse_resource_plan_json},
};
use dioxus::prelude::*;

#[derive(Clone)]
struct Inspected {
    name: String,
    context: Context,
    inventory: PackageInventory,
}

#[component]
pub(super) fn PackageBatchControls(
    host: HostHandle,
    current: EditorSnapshot,
    busy: Signal<bool>,
    status: Signal<String>,
) -> Element {
    let mut name = use_signal(|| "poet-checkpoint.hmc".to_string());
    let mut draft = use_signal(|| "[]".to_string());
    let mut inspected = use_signal(|| None::<Inspected>);
    let usable = inspected().filter(|view| {
        view.name == name()
            && view.context == Context::from_snapshot(&current)
            && current.publication_is_current()
    });
    let scan = {
        let host = host.clone();
        let current = current.clone();
        move |_| {
            if busy() {
                return;
            }
            busy.set(true);
            inspected.set(None);
            let filename = name();
            let origin = Context::from_snapshot(&current);
            let host = host.clone();
            let current = current.clone();
            spawn(async move {
                match jobs::run(
                    host.clone(),
                    Action::Inventory {
                        name: filename.clone(),
                        expected: current.source.clone(),
                    },
                    origin,
                )
                .await
                {
                    Ok(reply) if jobs::completion_is_current(&host, &current, &reply) => {
                        inspected.set(reply.inventory.map(|inventory| Inspected {
                            name: filename,
                            context: Context::from_snapshot(&current),
                            inventory,
                        }));
                        status.set(reply.message);
                    }
                    Ok(_) => status.set(
                        "Source or authority changed; inspect the package again before editing."
                            .into(),
                    ),
                    Err(error) => status.set(error),
                }
                busy.set(false);
            });
        }
    };
    let apply = {
        let host = host.clone();
        let current = current.clone();
        move |_| {
            if busy() {
                return;
            }
            let Some(view) = inspected().filter(|view| {
                view.name == name()
                    && view.context == Context::from_snapshot(&current)
                    && current.publication_is_current()
            }) else {
                status.set("Inspect this package under the current source and authority before applying a plan.".into());
                return;
            };
            let plan = match parse_resource_plan_json(&draft()) {
                Ok(plan) => plan,
                Err(error) => {
                    status.set(error);
                    return;
                }
            };
            busy.set(true);
            inspected.set(None);
            let host = host.clone();
            let current = current.clone();
            spawn(async move {
                match jobs::run(host.clone(),Action::ResourcePlan{name:view.name.clone(),file_digest:view.inventory.digest,plan,expected:current.source.clone()},Context::from_snapshot(&current)).await {
                    Ok(reply) if jobs::completion_is_current(&host,&current,&reply)=>{
                        inspected.set(reply.inventory.map(|inventory|Inspected{name:view.name,context:Context::from_snapshot(&current),inventory}));
                        status.set(reply.message);
                    },
                    Ok(_)=>status.set("Source or authority changed; result withheld. Inspect the current package before another plan.".into()),
                    Err(error)=>status.set(error),
                }
                busy.set(false);
            });
        }
    };
    rsx! {
        section {style:"padding:12px;border:1px solid #465463;display:flex;flex-direction:column;gap:8px;",
            h2 {style:"margin:0;font-size:19px;", "Atomic HMC resource plan"}
            p {"Edit related geometry, graph and texture resources together. Review the entire JSON command draft, then apply it once. Each replacement or removal requires the inspected resource SHA256."}
            div {style:"display:flex;gap:8px;",
                input {value:"{name}",aria_label:"Atomic resource plan HMC filename",disabled:busy(),oninput:move|event|{name.set(event.value());inspected.set(None);}}
                button {disabled:busy(),onclick:scan,"Inspect plan package"}
            }
            if let Some(view)=usable {
                p {style:"overflow-wrap:anywhere;", "Expected package SHA256 {hex_digest(&view.inventory.digest)}"}
                for resource in view.inventory.resources {
                    p {style:"overflow-wrap:anywhere;", "{resource.key} · {resource.kind} · {resource.bytes} bytes · SHA256 {hex_digest(&resource.digest)}"}
                }
            }
            p {"Use op add with filename, key and kind; op replace with filename, key and expected_resource_sha256; op remove with key and expected_resource_sha256. Use a JSON array of 1–64 objects. Filenames select the native import library; they cannot select filesystem paths."}
            textarea {value:"{draft}",aria_label:"Reviewed atomic resource plan JSON command draft",rows:10,disabled:busy(),oninput:move|event|draft.set(event.value())}
            button {disabled:busy() || inspected().is_none(),onclick:apply,"Apply entire reviewed resource plan"}
            p {"All resource guards are checked before import reads. One configured byte budget covers all imports. A failed dependency check installs no resource changes. Completion and human review remain separate."}
        }
    }
}
