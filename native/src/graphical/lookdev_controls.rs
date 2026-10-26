//! Native view-only lighting controls for source look development.
use super::viewport::{ViewLighting, ViewportCommand};
use dioxus::prelude::*;

fn bounded(text: &str, label: &str, min: f32, max: f32) -> Result<f32, String> {
    let value = text
        .parse::<f32>()
        .map_err(|_| format!("{label} must be a number in [{min}, {max}]"))?;
    if !value.is_finite() || value < min || value > max {
        return Err(format!("{label} must be finite and in [{min}, {max}]"));
    }
    Ok(value)
}

fn command(
    exposure_ev: &str,
    sun_x: &str,
    sun_y: &str,
    sun_z: &str,
    sun_intensity: &str,
    ambient_intensity: &str,
    shadows: bool,
    screen_space_ao: bool,
) -> Result<ViewportCommand, String> {
    let direction = [
        bounded(sun_x, "Sun direction X", -1.0, 1.0)?,
        bounded(sun_y, "Sun direction Y", -1.0, 1.0)?,
        bounded(sun_z, "Sun direction Z", -1.0, 1.0)?,
    ];
    if direction.iter().map(|value| value * value).sum::<f32>() <= f32::EPSILON {
        return Err("Sun direction must have a nonzero vector".into());
    }
    Ok(ViewportCommand::Lighting {
        exposure_ev: bounded(exposure_ev, "Exposure EV", -8.0, 8.0)?,
        sun_direction: direction,
        sun_intensity: bounded(sun_intensity, "Sun intensity", 0.0, 100.0)?,
        ambient_intensity: bounded(ambient_intensity, "Ambient intensity", 0.0, 100.0)?,
        shadows,
        screen_space_ao,
    })
}

fn preset(
    lighting: ViewLighting,
    onapply: EventHandler<ViewportCommand>,
    mut exposure_ev: Signal<String>,
    mut sun_x: Signal<String>,
    mut sun_y: Signal<String>,
    mut sun_z: Signal<String>,
    mut sun_intensity: Signal<String>,
    mut ambient_intensity: Signal<String>,
    mut shadows: Signal<bool>,
    mut screen_space_ao: Signal<bool>,
    mut error: Signal<String>,
) {
    let command = lighting.command();
    if let ViewportCommand::Lighting {
        exposure_ev: exposure,
        sun_direction,
        sun_intensity: sun,
        ambient_intensity: ambient,
        shadows: shadow_state,
        screen_space_ao: ao_state,
    } = &command
    {
        exposure_ev.set(exposure.to_string());
        sun_x.set(sun_direction[0].to_string());
        sun_y.set(sun_direction[1].to_string());
        sun_z.set(sun_direction[2].to_string());
        sun_intensity.set(sun.to_string());
        ambient_intensity.set(ambient.to_string());
        shadows.set(*shadow_state);
        screen_space_ao.set(*ao_state);
        error.set(String::new());
        onapply.call(command);
    } else {
        error.set("Lighting preset did not produce a preview-lighting command.".into());
    }
}

#[component]
pub(super) fn PreviewLightingControls(
    busy: bool,
    onapply: EventHandler<ViewportCommand>,
) -> Element {
    let mut exposure_ev = use_signal(|| "0.07038933".to_string());
    let mut sun_x = use_signal(|| "0.45".to_string());
    let mut sun_y = use_signal(|| "0.8".to_string());
    let mut sun_z = use_signal(|| "0.35".to_string());
    let mut sun_intensity = use_signal(|| "1.4".to_string());
    let mut ambient_intensity = use_signal(|| "0.15".to_string());
    let mut shadows = use_signal(|| true);
    let mut screen_space_ao = use_signal(|| true);
    let mut error = use_signal(String::new);

    let existing = move |_| preset(
        ViewLighting::existing_scene(), onapply, exposure_ev, sun_x, sun_y, sun_z,
        sun_intensity, ambient_intensity, shadows, screen_space_ao, error,
    );
    let studio = move |_| preset(
        ViewLighting::neutral_studio(), onapply, exposure_ev, sun_x, sun_y, sun_z,
        sun_intensity, ambient_intensity, shadows, screen_space_ao, error,
    );
    let apply = move |_| match command(
        &exposure_ev(), &sun_x(), &sun_y(), &sun_z(), &sun_intensity(),
        &ambient_intensity(), shadows(), screen_space_ao(),
    ) {
        Ok(command) => { error.set(String::new()); onapply.call(command); }
        Err(problem) => error.set(problem),
    };

    rsx! {
        details { style:"margin:8px 0;", open:true,
            summary { "Preview lighting · view only" }
            p { "Adjust exposure, sun direction and intensity, ambient light, shadows, and screen-space AO for this rendered preview. These controls do not edit source, materials, or world data." }
            div { style:"display:flex;gap:8px;flex-wrap:wrap;",
                button { disabled:busy, onclick:existing, "Existing scene lighting" }
                button { disabled:busy, onclick:studio, "Neutral studio lighting" }
            }
            div { style:"display:grid;grid-template-columns:repeat(auto-fit,minmax(170px,1fr));gap:8px;",
                label { "Exposure EV (−8 to 8)"
                    input { r#type:"number", min:"-8", max:"8", step:"0.1", value:"{exposure_ev}", disabled:busy, oninput:move |event|exposure_ev.set(event.value()) }
                }
                label { "Sun direction X (−1 to 1)"
                    input { r#type:"number", min:"-1", max:"1", step:"0.05", value:"{sun_x}", disabled:busy, oninput:move |event|sun_x.set(event.value()) }
                }
                label { "Sun direction Y (−1 to 1)"
                    input { r#type:"number", min:"-1", max:"1", step:"0.05", value:"{sun_y}", disabled:busy, oninput:move |event|sun_y.set(event.value()) }
                }
                label { "Sun direction Z (−1 to 1)"
                    input { r#type:"number", min:"-1", max:"1", step:"0.05", value:"{sun_z}", disabled:busy, oninput:move |event|sun_z.set(event.value()) }
                }
                label { "Sun intensity (0 to 100)"
                    input { r#type:"number", min:"0", max:"100", step:"0.1", value:"{sun_intensity}", disabled:busy, oninput:move |event|sun_intensity.set(event.value()) }
                }
                label { "Ambient intensity (0 to 100)"
                    input { r#type:"number", min:"0", max:"100", step:"0.1", value:"{ambient_intensity}", disabled:busy, oninput:move |event|ambient_intensity.set(event.value()) }
                }
                label { input { r#type:"checkbox", checked:shadows(), disabled:busy, onchange:move |event|shadows.set(event.checked()) } " Shadows" }
                label { input { r#type:"checkbox", checked:screen_space_ao(), disabled:busy, onchange:move |event|screen_space_ao.set(event.checked()) } " Screen-space AO" }
            }
            button { disabled:busy, onclick:apply, "Apply preview lighting" }
            if !error().is_empty() { p { "Lighting input: {error}" } }
        }
    }
}
