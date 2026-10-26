//! Mounted native component events for the version-three image controls.
//! Included below texture_effect_component_tests.rs to reuse its actual VDOM harness.
use super::*;

const ADD_V3: &str = "Add effect (paint version 3)";

#[test]
fn image_completed_component_four_filter_fields_apply_preserves_source() {
    let original = texture_effect_component_accepted();
    let mut ui = ComponentHarness::new();
    ui.click(LOAD, 0);
    for (kind, primary_label, primary, secondary) in [
        ("gaussian_blur", "Texture effect 0 sigma", "1.5", None),
        ("sharpen", "Texture effect 1 sigma", "2", Some("3.5")),
        ("hue_rotate", "Texture effect 2 degrees", "-90", None),
        (
            "emboss",
            "Texture effect 3 azimuth degrees",
            "135",
            Some("0.25"),
        ),
    ] {
        ui.change("Texture effect to add", kind);
        ui.click(ADD_V3, 0);
        ui.input(primary_label, primary);
        if let Some(value) = secondary {
            let label = primary_label
                .replace(" sigma", " secondary amount or strength")
                .replace(" azimuth degrees", " secondary amount or strength");
            ui.input(&label, value);
        }
        assert!(ui.emitted.borrow().is_empty());
        assert_eq!(ui.rendered.value(primary_label), primary);
    }
    ui.click(APPLY, 0);
    let applied = ui.emitted_paint();
    assert_eq!(applied.version, 3);
    assert_eq!(applied.side, original.side);
    assert_eq!(applied.base_sha256, original.base_sha256);
    assert_eq!(applied.layers, original.layers);
    assert_eq!(
        applied
            .effects
            .iter()
            .map(|s| s.effect.clone())
            .collect::<Vec<_>>(),
        vec![
            TextureEffect::GaussianBlur { sigma: 1.5 },
            TextureEffect::Sharpen {
                sigma: 2.0,
                amount: 3.5
            },
            TextureEffect::HueRotate { degrees: -90.0 },
            TextureEffect::Emboss {
                azimuth_degrees: 135.0,
                strength: 0.25
            },
        ]
    );
}

#[test]
fn image_completed_component_all_blends_signed_offsets_rectangle_mask_apply() {
    for mode in [
        "normal",
        "multiply",
        "screen",
        "overlay",
        "soft_light",
        "hard_light",
        "color_dodge",
        "color_burn",
        "difference",
        "exclusion",
    ] {
        let original = texture_effect_component_accepted();
        let mut ui = ComponentHarness::new();
        ui.click(LOAD, 0);
        ui.change("Layer 0 blend mode", mode);
        ui.input("Layer 0 offset X", "-7");
        ui.input("Layer 0 offset Y", "9");
        ui.change("Layer 0 mask type", "rectangle");
        for (label, value) in [
            ("Layer 0 Mask X in pixels", "2"),
            ("Layer 0 Mask Y in pixels", "3"),
            ("Layer 0 Mask width in pixels", "4"),
            ("Layer 0 Mask height in pixels", "5"),
            ("Layer 0 Mask coverage byte (0 to 255)", "128"),
        ] {
            ui.input(label, value);
        }
        ui.click(APPLY, 0);
        let applied = ui.emitted_paint();
        assert_eq!(applied.version, 3, "{mode}");
        assert_eq!(applied.base_sha256, original.base_sha256);
        assert_eq!(applied.layers[0].strokes, original.layers[0].strokes);
        assert_eq!(applied.layers[0].offset, [-7, 9]);
        assert_eq!(
            applied.layers[0].mask,
            Some(LayerMask::Rectangle {
                x: 2,
                y: 3,
                width: 4,
                height: 5,
                coverage: 128
            })
        );
        assert_eq!(
            format!("{:?}", applied.layers[0].blend_mode)
                .to_lowercase()
                .replace('_', ""),
            mode.replace('_', "")
        );
    }
}

#[test]
fn image_completed_component_resource_channels_and_invalid_raw_refuse_old_values() {
    for (channel, expected) in [("red", MaskChannel::Red), ("alpha", MaskChannel::Alpha)] {
        let mut ui = ComponentHarness::new();
        ui.click(LOAD, 0);
        ui.change("Layer 0 mask type", "resource");
        ui.input("Layer 0 mask resource SHA-256", &"b".repeat(64));
        ui.change("Layer 0 mask resource channel", channel);
        ui.click(APPLY, 0);
        assert_eq!(
            ui.emitted_paint().layers[0].mask,
            Some(LayerMask::Resource {
                sha256: "b".repeat(64),
                channel: expected
            })
        );
    }
    for (field, invalid) in [
        ("Layer 0 offset X", "-"),
        ("Layer 0 offset Y", "2147483648"),
        ("Layer 0 mask resource SHA-256", "bad"),
    ] {
        let mut ui = ComponentHarness::new();
        ui.click(LOAD, 0);
        ui.change("Layer 0 mask type", "resource");
        ui.input("Layer 0 mask resource SHA-256", &"b".repeat(64));
        ui.input(field, invalid);
        assert_eq!(ui.rendered.value(field), invalid);
        ui.click(APPLY, 0);
        assert!(
            ui.emitted.borrow().is_empty(),
            "invalid {field} emitted old typed value"
        );
    }
    let mut ui = ComponentHarness::new();
    ui.click(LOAD, 0);
    ui.change("Layer 0 mask type", "rectangle");
    ui.input("Layer 0 Mask width in pixels", "65");
    ui.click(APPLY, 0);
    assert!(
        ui.emitted.borrow().is_empty(),
        "hidden layer still validates mask extent"
    );
}

#[test]
fn image_completed_component_new_filter_invalid_primary_refuses_stale_typed_value() {
    for (kind, label, invalid) in [
        ("gaussian_blur", "Texture effect 0 sigma", "16.1"),
        ("sharpen", "Texture effect 0 sigma", "-"),
        ("hue_rotate", "Texture effect 0 degrees", "361"),
        ("emboss", "Texture effect 0 azimuth degrees", "NaN"),
    ] {
        let mut ui = ComponentHarness::new();
        ui.click(LOAD, 0);
        ui.change("Texture effect to add", kind);
        ui.click(ADD_V3, 0);
        ui.input(label, invalid);
        assert_eq!(ui.rendered.value(label), invalid, "{kind}");
        ui.click(APPLY, 0);
        assert!(
            ui.emitted.borrow().is_empty(),
            "{kind} emitted an old typed value"
        );
    }
}

#[test]
fn image_completed_component_disabled_filter_invalid_secondary_and_v3_legacy_add() {
    let mut source = texture_effect_component_accepted();
    source.version = 3;
    let mut ui = ComponentHarness::new();
    ui.set_accepted(DetailRecipe::Paint(source.clone()), "session-1:part-A");
    ui.click(LOAD, 0);
    ui.change("Texture effect to add", "sharpen");
    ui.click(ADD_V3, 0);
    ui.input("Texture effect 0 sigma", "2");
    ui.input("Texture effect 0 secondary amount or strength", "3");
    ui.change("Texture effect 0 enabled", "false");
    ui.input("Texture effect 0 secondary amount or strength", "NaN");
    assert_eq!(
        ui.rendered
            .value("Texture effect 0 secondary amount or strength"),
        "NaN"
    );
    ui.click(APPLY, 0);
    assert!(
        ui.emitted.borrow().is_empty(),
        "disabled filter cannot emit stale amount"
    );
    ui.input("Texture effect 0 secondary amount or strength", "0.5");
    ui.change("Texture effect to add", "invert");
    ui.click(ADD, 0);
    ui.click(APPLY, 0);
    let applied = ui.emitted_paint();
    assert_eq!(
        applied.version, 3,
        "legacy Add may not downgrade accepted v3"
    );
    assert_eq!(applied.layers, source.layers);
    assert_eq!(
        applied.effects[0],
        TextureEffectStep {
            enabled: false,
            effect: TextureEffect::Sharpen {
                sigma: 2.0,
                amount: 0.5
            }
        }
    );
    assert_eq!(applied.effects[1].effect, TextureEffect::Invert);
}

#[test]
fn image_completed_component_source_context_change_retains_v3_draft_until_reload() {
    let mut ui = ComponentHarness::new();
    ui.click(LOAD, 0);
    ui.change("Layer 0 mask type", "rectangle");
    ui.input("Layer 0 Mask width in pixels", "7");
    ui.change("Texture effect to add", "gaussian_blur");
    ui.click(ADD_V3, 0);
    ui.input("Texture effect 0 sigma", "2.5");
    let mut next = texture_effect_component_accepted();
    next.layers[0].name = "New accepted source layer".into();
    ui.set_accepted(DetailRecipe::Paint(next.clone()), "session-2:part-B");
    assert_eq!(ui.rendered.value("Texture effect 0 sigma"), "2.5");
    assert_eq!(ui.rendered.value("Layer 0 Mask width in pixels"), "7");
    assert!(ui.rendered.disabled(ui.rendered.button(APPLY, 0)));
    ui.force_click(APPLY, 0);
    assert!(ui.emitted.borrow().is_empty());
    ui.click(LOAD, 0);
    assert_eq!(ui.rendered.value("Layer 0 name"), next.layers[0].name);
    assert!(ui.rendered.matching(|n, _| matches!(n.attrs.get("aria-label"), Some(AttributeValue::Text(v)) if v == "Texture effect 0 sigma")).is_empty());
    ui.click(APPLY, 0);
    assert_eq!(ui.emitted_paint(), next);
}

#[test]
fn image_completed_component_layer_navigation_keeps_each_stroke_form_and_replace_target() {
    let mut accepted = texture_effect_component_accepted();
    let first = accepted.layers[0].strokes[0].clone();
    let mut second = first.clone();
    second.size = 9.0;
    second.colour = [91, 82, 73, 255];
    accepted.layers.push(PaintLayer {
        name: "Second paint layer".into(),
        strokes: vec![second.clone()],
        ..PaintLayer::default()
    });
    let mut ui = ComponentHarness::new();
    ui.set_accepted(DetailRecipe::Paint(accepted.clone()), "session-1:part-A");
    ui.click(LOAD, 0);
    assert_eq!(ui.rendered.value("Diameter in pixels"), "8");
    ui.input("Diameter in pixels", "-");
    ui.click("Select layer 1", 0);
    assert_eq!(ui.rendered.value("Diameter in pixels"), "9");
    ui.click("Replace loaded stroke", 0);
    ui.click("Select layer 0", 0);
    assert_eq!(
        ui.rendered.value("Diameter in pixels"),
        "-",
        "invalid raw form survives layer navigation"
    );
    ui.click("Select layer 1", 0);
    ui.click("Move layer up", 1);
    assert_eq!(ui.rendered.value("Diameter in pixels"), "9");
    ui.click("Move layer down", 0);
    assert_eq!(ui.rendered.value("Diameter in pixels"), "9");
    ui.click("Select layer 0", 0);
    assert_eq!(ui.rendered.value("Diameter in pixels"), "-");
    ui.click(APPLY, 0);
    assert!(
        ui.emitted.borrow().is_empty(),
        "pending invalid first-layer stroke must refuse Apply"
    );
    ui.input("Diameter in pixels", "8");
    ui.click(APPLY, 0);
    let applied = ui.emitted_paint();
    assert_eq!(applied.layers[0].strokes[0], first);
    assert_eq!(
        applied.layers[1].strokes[0], second,
        "Replace on second layer must never copy the first layer's form"
    );
}

#[test]
fn image_completed_component_stroke_reorder_remove_refreshes_loaded_form() {
    let mut accepted = texture_effect_component_accepted();
    let first = accepted.layers[0].strokes[0].clone();
    let mut second = first.clone();
    second.size = 13.0;
    second.points = vec![[17.0, 19.0]];
    accepted.layers[0].strokes.push(second);
    let mut ui = ComponentHarness::new();
    ui.set_accepted(DetailRecipe::Paint(accepted), "session-1:part-A");
    ui.click(LOAD, 0);
    ui.click("Load stroke 1 (1 dabs)", 0);
    assert_eq!(ui.rendered.value("Diameter in pixels"), "13");
    ui.click("Move stroke up", 1);
    assert_eq!(ui.rendered.value("Diameter in pixels"), "13");
    ui.click("Move stroke down", 0);
    assert_eq!(ui.rendered.value("Diameter in pixels"), "13");
    ui.click("Remove stroke", 1);
    assert_eq!(ui.rendered.value("Diameter in pixels"), "8");
    ui.click("Replace loaded stroke", 0);
    ui.click(APPLY, 0);
    assert_eq!(
        ui.emitted_paint().layers[0].strokes,
        vec![first],
        "a removed stroke's stale form must not overwrite the remaining stroke"
    );
}

#[test]
fn image_completed_component_layer_removal_loads_remaining_stroke_form() {
    let mut accepted = texture_effect_component_accepted();
    let mut second = accepted.layers[0].strokes[0].clone();
    second.size = 12.0;
    second.colour = [8, 190, 77, 255];
    accepted.layers.push(PaintLayer {
        name: "Remaining layer".into(),
        strokes: vec![second.clone()],
        ..PaintLayer::default()
    });
    let mut ui = ComponentHarness::new();
    ui.set_accepted(DetailRecipe::Paint(accepted), "session-1:part-A");
    ui.click(LOAD, 0);
    ui.click("Remove layer", 0);
    assert_eq!(ui.rendered.value("Diameter in pixels"), "12");
    ui.click("Replace loaded stroke", 0);
    ui.click(APPLY, 0);
    assert_eq!(ui.emitted_paint().layers[0].strokes, vec![second]);
}
