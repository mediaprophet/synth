//! Native inputs generate an inspectable Vibe draft; submit owns admission.
use super::controller::EditorSnapshot;
use super::materials::MaterialDraft;
use dioxus::prelude::*;
use qualia_core_db::container_10d::OpacityMode;

const LABELS: [&str; 11] = [
    "Linear red",
    "Linear green",
    "Linear blue",
    "Alpha",
    "Roughness",
    "Metallic",
    "Specular",
    "Linear emissive red",
    "Linear emissive green",
    "Linear emissive blue",
    "Alpha cutoff",
];

fn parse_fields(
    fields: &[String],
    mode: &str,
    double_sided: bool,
) -> Result<MaterialDraft, String> {
    if fields.len() != LABELS.len() {
        return Err("material fields are incomplete".into());
    }
    let mut values = [0.0; 11];
    for (index, field) in fields.iter().enumerate() {
        values[index] = field
            .parse::<f32>()
            .map_err(|_| format!("{} must be a finite number", LABELS[index]))?;
    }
    Ok(MaterialDraft {
        base_color: [values[0], values[1], values[2], values[3]],
        roughness: values[4],
        metallic: values[5],
        specular: values[6],
        emissive: [values[7], values[8], values[9]],
        alpha_cutoff: values[10],
        double_sided,
        opacity_mode: match mode {
            "Opaque" => OpacityMode::Opaque,
            "Mask" => OpacityMode::Mask,
            "Blend" => OpacityMode::Blend,
            _ => return Err("unsupported opacity mode".into()),
        },
    })
}

#[component]
pub(super) fn MaterialControls(
    current: EditorSnapshot,
    busy: bool,
    mut draft: Signal<String>,
    mut status: Signal<String>,
    #[props(default)] draft_buffer: Option<super::draft_buffer::DraftBuffer>,
) -> Element {
    let mut fields = use_signal(|| {
        [
            "0.36", "0.20", "0.085", "1.0", "0.65", "0.0", "1.0", "0.0", "0.0", "0.0", "0.5",
        ]
        .into_iter()
        .map(String::from)
        .collect::<Vec<_>>()
    });
    let mut opacity = use_signal(|| "Opaque".to_string());
    let mut double_sided = use_signal(|| false);
    let enabled = !busy && !current.reconciliation_required && current.material_edit_id.is_some();
    rsx! {
        fieldset { disabled: !enabled, style: "margin:0;padding:10px;border:1px solid #405264;min-width:0;",
            legend { "Material draft · linear factors" }
            p { style: "margin:0 0 8px;font-size:13px;", "Fields are a new draft, not the current material inspector. Colour/alpha, roughness, metallic, specular and cutoff use 0–1; emissive uses native linear factor bounds. No texture admission." }
            div { style: "display:flex;flex-wrap:wrap;gap:8px;",
                for (index, label) in LABELS.iter().enumerate() {
                    label { style: "display:flex;flex-direction:column;width:112px;font-size:13px;",
                        "{label}"
                        input { r#type: "text", disabled: !enabled, value: "{fields()[index]}", aria_label: "{label}",
                            style: "width:100%;box-sizing:border-box;background:#081018;color:#e8eef4;",
                            oninput: move |event| { fields.write()[index] = event.value(); }
                        }
                    }
                }
                label { "Opacity"
                    select { disabled: !enabled, value: "{opacity}", aria_label: "Material opacity mode", onchange: move |event| opacity.set(event.value()),
                        option { value: "Opaque", "Opaque" }
                        option { value: "Mask", "Mask" }
                        option { value: "Blend", "Blend" }
                    }
                }
                label { "Double sided"
                    input { r#type: "checkbox", disabled: !enabled, checked: double_sided(), aria_label: "Material double sided",
                        onchange: move |event| double_sided.set(event.checked()) }
                }
            }
            button { disabled: !enabled, style: "margin-top:8px;", onclick: move |_| {
                if !enabled {
                    status.set("Material drafting requires a current admitted execution session with editable material and geometry bindings.".into());
                    return;
                }
                if draft() != current.latest_source {
                    status.set("Unsubmitted source draft preserved. Submit it or explicitly restore the accepted source before staging a material edit.".into());
                    return;
                }
                let result = current.material_edit_id.ok_or_else(|| "Current source has no admitted editable material binding".to_string())
                    .and_then(|id| parse_fields(&fields(), &opacity(), double_sided())
                        .and_then(|values| values.edit_source(current.session_revision.saturating_add(1), id)));
                match result {
                    Ok(source) => { if let Some(buffer)=&draft_buffer{buffer.set(source.clone());}draft.set(source); status.set("Material source draft staged. Inspect, Check, then Submit source to accept and render it.".into()); }
                    Err(error) => status.set(error),
                }
            }, "Stage material source draft" }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn malformed_control_input_and_infinite_factors_cannot_generate_source() {
        let mut fields = vec!["0.5".to_string(); 11];
        fields[2] = "hello".into();
        assert!(parse_fields(&fields, "Opaque", false).is_err());
        fields[2] = "NaN".into();
        assert!(parse_fields(&fields, "Opaque", false)
            .unwrap()
            .edit_source(2, 1)
            .is_err());
        fields[2] = "0.5".into();
        assert!(parse_fields(&fields, "unknown", false).is_err());
    }
}
