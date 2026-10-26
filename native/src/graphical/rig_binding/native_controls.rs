//! Narrow human controls for the selected source-bound rig.
//! The caller stages actions through the existing authorized native edit route.
use super::{RigControl, RigSourceDescriptor};
use dioxus::prelude::*;
use qualia_core_db::render::skeletal::Motor;

#[component]
pub fn RigControls(
    descriptor: RigSourceDescriptor,
    busy: bool,
    on_control: EventHandler<RigControl>,
) -> Element {
    let mut clip = use_signal(|| descriptor.selected_clip_id.to_string());
    let mut seek = use_signal(|| descriptor.seek_seconds.to_string());
    let mut joint = use_signal(|| descriptor.joint_ids.first().copied().unwrap_or(0).to_string());
    let mut delta = use_signal(|| [String::from("0"), String::from("0"), String::from("0")]);
    let mut status = use_signal(String::new);
    let enabled = !busy && !descriptor.clip_ids.is_empty() && !descriptor.joint_ids.is_empty();
    rsx! {
        fieldset { disabled: !enabled, style: "padding:10px;border:1px solid #405264;",
            legend { "Rig · selected source" }
            p { "Part {descriptor.part_id} · bind {descriptor.bind_id} · {descriptor.resource_key}" }
            label { "Clip ID"
                select { value: "{clip}", onchange: move |e| clip.set(e.value()),
                    for id in descriptor.clip_ids.iter() {
                        option { value: "{id}", "{id}" }
                    }
                }
            }
            button { onclick: move |_| {
                match clip().parse::<u64>() {
                    Ok(clip_id) => on_control.call(RigControl::SelectClip { clip_id, fade_seconds: 0.0 }),
                    Err(_) => status.set("Select a known clip identifier".into()),
                }
            }, "Select clip" }
            label { "Seek seconds"
                input { r#type: "number", step: "0.01", value: "{seek}",
                    oninput: move |e| seek.set(e.value()) }
            }
            button { onclick: move |_| {
                match seek().parse::<f64>() {
                    Ok(seconds) if seconds.is_finite() && seconds.abs() <= 1e12 =>
                        on_control.call(RigControl::Seek { seconds }),
                    _ => status.set("Enter a finite seek time".into()),
                }
            }, "Seek pose" }
            label { "Joint ID"
                select { value: "{joint}", onchange: move |e| joint.set(e.value()),
                    for id in descriptor.joint_ids.iter() {
                        option { value: "{id}", "{id}" }
                    }
                }
            }
            for axis in 0..3 {
                label { "Translation {axis}"
                    input { r#type: "number", step: "0.01", value: "{delta()[axis]}",
                        oninput: move |e| delta.write()[axis] = e.value() }
                }
            }
            button { onclick: move |_| {
                let parsed = delta().iter().map(|s| s.parse::<f64>()).collect::<Result<Vec<_>, _>>();
                match (joint().parse::<u64>(), parsed) {
                    (Ok(joint_id), Ok(v)) if v.len() == 3 && v.iter().all(|x| x.is_finite()) => {
                        let m = Motor::from_translation(v[0], v[1], v[2]);
                        on_control.call(RigControl::SetJointDelta { joint_id,
                            motor: [m.r_w, m.r_x, m.r_y, m.r_z, m.d_w, m.d_x, m.d_y, m.d_z] });
                    }
                    _ => status.set("Enter a known joint and finite translation".into()),
                }
            }, "Stage joint translation" }
            p { role: "status", "{status}" }
        }
    }
}
