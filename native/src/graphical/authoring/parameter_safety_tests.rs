//! Synthetic source/journal tests. No production assets or human acceptance.
use super::*;
use crate::graphical::controller::EditorHost;
use vibe::{session::SourceSession, Budget, Value};

pub(super) fn parameter(value: f64) -> AuthoringEdit {
    AuthoringEdit::Parameter {
        key: "offset".into(),
        value,
    }
}
fn generate(session: &SourceSession, value: f64) -> Result<String, String> {
    source::edit_source_with_baseline(
        session.value("poet_parts").unwrap(),
        session.value("poet_parameters"),
        session.value("poet_parameter_parts"),
        &parameter(value),
        session.revision() + 1,
    )
}
#[test]
fn explicit_baseline_allows_repeated_parameters_and_preserves_unknown_parameter_values() {
    let mut session = SourceSession::default();
    let mut local = vibe::LocalHost::default();
    let fixture = super::tests::source().replace(
        "channels:6",
        "channels:9007199254740993u64,notes:{keep:true}",
    );
    session
        .submit(&fixture, &mut local, Budget::default())
        .unwrap();
    for (index, value) in [1.25, 2.5, 1.0e-12].into_iter().enumerate() {
        let generated = generate(&session, value).unwrap();
        assert_eq!(
            generated.matches("poet_rebuild_parts(").count(),
            1,
            "no hidden replay"
        );
        session
            .submit(&generated, &mut local, Budget::default())
            .unwrap();
        assert_eq!(session.revision(), index as u64 + 2);
        assert_eq!(
            session.value("poet_parts"),
            session.value("poet_parameter_parts")
        );
        let Value::Record(parameters) = session.value("poet_parameters").unwrap() else {
            panic!()
        };
        assert_eq!(parameters["offset"], Value::F64(value));
        assert_eq!(parameters["channels"], Value::U64(9007199254740993));
        assert!(
            matches!(&parameters["notes"], Value::Record(notes) if notes["keep"] == Value::Bool(true))
        );
    }
}
#[test]
fn all_downstream_fields_refuse_without_changing_session_parameters_or_history() {
    let mut session = SourceSession::default();
    session
        .submit(
            &super::tests::source(),
            &mut vibe::LocalHost::default(),
            Budget::default(),
        )
        .unwrap();
    let baseline = session.value("poet_parts").unwrap().clone();
    let history = session.sources().to_vec();
    let parameters = session.value("poet_parameters").unwrap().clone();
    for key in [
        "uv_atlas",
        "detail",
        "material",
        "parent",
        "geometry",
        "translation",
        "extension",
    ] {
        let Value::List(mut parts) = baseline.clone() else {
            panic!()
        };
        let Value::Record(part) = &mut parts[0] else {
            panic!()
        };
        part.insert(key.into(), Value::String("retained authored edit".into()));
        assert!(
            parameter_safety::preflight(&Value::List(parts), Some(&baseline))
                .unwrap_err()
                .contains("preserve authored work"),
            "{key}"
        );
    }
    let Value::List(mut parts) = baseline.clone() else {
        panic!()
    };
    parts.pop();
    assert!(parameter_safety::preflight(&Value::List(parts), Some(&baseline)).is_err());
    assert_eq!(session.revision(), 1);
    assert_eq!(session.sources(), history);
    assert_eq!(session.value("poet_parameters"), Some(&parameters));
    assert_eq!(session.value("poet_parts"), Some(&baseline));
}
#[test]
fn canonical_detail_material_and_structure_edits_remain_exact_after_refused_regeneration() {
    use crate::graphical::surfaces::detail::{DetailRecipe, ProceduralRecipe};
    for edit in [
        AuthoringEdit::TextureDetail {
            material_id: 1,
            detail: Some(DetailRecipe::Procedural(ProceduralRecipe {
                version: 1,
                side: 8,
                seed: 9,
                tint: [0.3, 0.5, 0.7],
                contrast: 0.5,
                relief: 0.2,
                bands: [2, 3],
                band_strength: 0.4,
                roughness: 0.6,
                metallic: 0.2,
            })),
        },
        AuthoringEdit::Material {
            material_id: 1,
            base_color: [0.2, 0.3, 0.4, 1.0],
            roughness: 0.8,
            metallic: 0.0,
        },
        AuthoringEdit::Reparent {
            id: "lever".into(),
            parent: None,
        },
        AuthoringEdit::RemoveLeaf { id: "lever".into() },
        AuthoringEdit::Transform {
            id: "body".into(),
            translation: [2.0, 0.0, 0.0],
            rotation: [0.0; 3],
            scale: [1.0; 3],
        },
    ] {
        let mut session = SourceSession::default();
        let mut local = vibe::LocalHost::default();
        session
            .submit(&super::tests::source(), &mut local, Budget::default())
            .unwrap();
        let source = source::edit_source(
            session.value("poet_parts").unwrap(),
            session.value("poet_parameters"),
            &edit,
            2,
        )
        .unwrap();
        session
            .submit(&source, &mut local, Budget::default())
            .unwrap();
        let parts = session.value("poet_parts").unwrap().clone();
        let baseline = session.value("poet_parameter_parts").unwrap().clone();
        let parameters = session.value("poet_parameters").unwrap().clone();
        let history = session.sources().to_vec();
        assert!(
            generate(&session, 4.0)
                .unwrap_err()
                .contains("preserve authored work"),
            "{edit:?}"
        );
        assert_eq!(session.revision(), 2);
        assert_eq!(session.sources(), history);
        assert_eq!(session.value("poet_parts"), Some(&parts));
        assert_eq!(session.value("poet_parameter_parts"), Some(&baseline));
        assert_eq!(session.value("poet_parameters"), Some(&parameters));
    }
}
#[test]
fn source_owned_uv_detail_and_richer_geometry_cannot_be_rebased_to_bypass_safety() {
    let mut session = SourceSession::default();
    session
        .submit(
            &super::tests::source(),
            &mut vibe::LocalHost::default(),
            Budget::default(),
        )
        .unwrap();
    let original = session.value("poet_parts").unwrap();
    for key in ["uv_atlas", "detail", "uv_projection"] {
        let Value::List(mut parts) = original.clone() else {
            panic!()
        };
        let Value::Record(part) = &mut parts[0] else {
            panic!()
        };
        part.insert(key.into(), Value::Record(Default::default()));
        let parts = Value::List(parts);
        assert!(parameter_safety::preflight(&parts, Some(&parts))
            .unwrap_err()
            .contains("cannot remap"));
    }
    for key in ["uv0", "normals", "future_payload"] {
        let Value::List(mut parts) = original.clone() else {
            panic!()
        };
        let Value::Record(part) = &mut parts[0] else {
            panic!()
        };
        let Value::Record(geometry) = part.get_mut("geometry").unwrap() else {
            panic!()
        };
        geometry.insert(key.into(), Value::List(vec![]));
        let parts = Value::List(parts);
        assert!(parameter_safety::preflight(&parts, Some(&parts))
            .unwrap_err()
            .contains("richer geometry"));
    }
}
#[test]
fn comparison_refuses_deep_or_unsupported_data_and_distinguishes_numeric_bits() {
    let mut value = Value::Null;
    for _ in 0..70 {
        value = Value::List(vec![value]);
    }
    assert!(parameter_safety::preflight(&value, Some(&value))
        .unwrap_err()
        .contains("nesting bound"));
    assert!(
        parameter_safety::preflight(&Value::F64(0.0), Some(&Value::F64(-0.0)))
            .unwrap_err()
            .contains("preserve authored work")
    );
    let text = Value::String("x".repeat(1024 * 1024 + 1));
    assert!(parameter_safety::preflight(&text, Some(&text))
        .unwrap_err()
        .contains("text bound"));
}
pub(super) fn unchanged_after_refusal(h: &mut dyn EditorHost) {
    let before = h.snapshot().unwrap();
    let source = h.source_cells(before.source.clone()).unwrap();
    let parts = h.authoring_view(before.source.clone()).unwrap();
    let error = h
        .authoring_edit(parameter(4.0), before.source.clone())
        .err()
        .expect("parameter regeneration must refuse downstream edits");
    assert!(error.contains("preserve authored work"), "{error}");
    let after = h.snapshot().unwrap();
    assert_eq!(after.source, before.source);
    assert_eq!(after.source_revision, before.source_revision);
    assert_eq!(after.session_revision, before.session_revision);
    assert!(!after.reconciliation_required);
    assert_eq!(h.source_cells(after.source.clone()).unwrap(), source);
    let retained = h.authoring_view(after.source).unwrap();
    assert_eq!(format!("{retained:?}"), format!("{parts:?}"));
}
#[test]
fn native_material_overrides_survive_and_membership_changes_refuse() {
    let mut h = super::tests::host();
    let source = h.snapshot().unwrap().source;
    let source = h
        .submit(&super::tests::source(), source)
        .unwrap()
        .snapshot
        .source;
    let source = h
        .authoring_edit(parameter(1.25), source)
        .unwrap()
        .snapshot
        .source;
    let source = h
        .authoring_edit(super::source_layer_tests::material(), source)
        .unwrap()
        .snapshot
        .source;
    let source = h
        .authoring_edit(parameter(2.5), source)
        .unwrap()
        .snapshot
        .source;
    assert!(h
        .authoring_view(source.clone())
        .unwrap()
        .parts
        .iter()
        .all(|p| p.roughness == 0.8));
    let source = h
        .authoring_edit(AuthoringEdit::RemoveLeaf { id: "lever".into() }, source)
        .unwrap()
        .snapshot
        .source;
    unchanged_after_refusal(&mut h);
    let held = h.undo(source).unwrap();
    let source = h.rebuild_execution(held.source).unwrap().snapshot.source;
    assert!(h.authoring_edit(parameter(3.5), source).is_ok());
}
#[test]
fn legacy_and_immutable_baselines_refuse_before_execution_with_migration_guidance() {
    for (fixture, message) in [
        (
            super::tests::source().replace(
                "let mut poet_parameter_parts=poet_rebuild_parts(poet_parameters);",
                "let poet_parameter_parts=poet_rebuild_parts(poet_parameters);",
            ),
            "must be mutable",
        ),
        (
            super::tests::source()
                .replace(
                    "let mut poet_parameter_parts=poet_rebuild_parts(poet_parameters);",
                    "",
                )
                .replace(
                    "let mut poet_parts=poet_parameter_parts;",
                    "let mut poet_parts=poet_rebuild_parts(poet_parameters);",
                ),
            "explicit source lineage",
        ),
    ] {
        let mut h = super::tests::host();
        let source = h.snapshot().unwrap().source;
        let before = h.submit(&fixture, source).unwrap().snapshot;
        let cells = h.source_cells(before.source.clone()).unwrap();
        assert!(h
            .authoring_edit(parameter(1.0), before.source.clone())
            .err()
            .expect("legacy or immutable parameter lineage must refuse")
            .contains(message));
        let after = h.snapshot().unwrap();
        assert_eq!(after.source, before.source);
        assert_eq!(after.session_revision, before.session_revision);
        assert_eq!(h.source_cells(after.source).unwrap(), cells);
    }
}
