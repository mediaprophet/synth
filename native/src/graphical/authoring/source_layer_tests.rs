//! Synthetic equipment-design cases; no master asset or acceptance claim.
use super::*;
use crate::graphical::controller::EditorHost;
use vibe::{session::SourceSession, Budget, Value};

pub(super) fn fixture() -> String {
    super::tests::source()
        .replace(
            "channels:6",
            "channels:6,width:1.0,opaque:{big:9007199254740993u64,keep:true}",
        )
        .replace("[1.0,0.0,0.0]", "[p.width,0.0,0.0]")
        .replace(
            "geometry:mesh,material:",
            "opaque:{big:9007199254740993u64,negative_zero:-0.0},geometry:mesh,material:",
        )
}
/// Native render admission intentionally has a closed part-record vocabulary.
/// Keep opaque extensions in the raw canonical-session preservation cases;
/// native host cases exercise the renderer's actual supported part contract.
pub(super) fn native_fixture() -> String {
    fixture().replace(
        "opaque:{big:9007199254740993u64,negative_zero:-0.0},geometry:mesh,material:",
        "geometry:mesh,material:",
    )
}
pub(super) fn width(value: f64) -> AuthoringEdit {
    AuthoringEdit::Parameter {
        key: "width".into(),
        value,
    }
}
pub(super) fn material() -> AuthoringEdit {
    AuthoringEdit::Material {
        material_id: 1,
        base_color: [0.2, 0.3, 0.4, 1.0],
        roughness: 0.8,
        metallic: 0.0,
    }
}
pub(super) fn transform() -> AuthoringEdit {
    AuthoringEdit::Transform {
        id: "body".into(),
        translation: [3.0, 0.0, 0.0],
        rotation: [0.0; 3],
        scale: [1.0; 3],
    }
}
fn generated(session: &SourceSession, edit: &AuthoringEdit) -> Result<String, String> {
    source::edit_source_with_layers(
        session.value("poet_parts").unwrap(),
        session.value("poet_parameters"),
        session.value("poet_parameter_parts"),
        session.value("poet_source_layer"),
        edit,
        session.revision() + 1,
    )
}
fn submit(session: &mut SourceSession, source: &str, budget: Budget) -> Result<(), String> {
    let (parts, base) = source_layers::validation_snapshot(
        session.value("poet_parts"),
        session.value("poet_parameter_parts"),
    )?;
    session
        .submit_validated(source, &mut vibe::LocalHost::default(), budget, |result| {
            source_layers::validate_result(result, parts.as_ref(), base.as_ref()).map_err(
                |message| {
                    vibe::Diagnostic::new(vibe::DiagCode::E701, vibe::Span::point(0), message)
                },
            )
        })
        .map(|_| ())
        .map_err(|e| e.diagnostic.message)
}
fn value_parts(session: &SourceSession) -> &Vec<Value> {
    let Value::List(parts) = session.value("poet_parts").unwrap() else {
        panic!()
    };
    parts
}
#[test]
fn raw_opaque_preservation_fixture_and_native_render_fixture_have_explicit_scopes() {
    for (fixture, renderable) in [(fixture(), false), (native_fixture(), true)] {
        let mut session = SourceSession::default();
        let result = session
            .submit(&fixture, &mut vibe::LocalHost::default(), Budget::default())
            .unwrap();
        let admitted = crate::graphical::assembly::admit_assembly(
            &result.value,
            crate::graphical::preview::PreviewLimits::default(),
        );
        assert_eq!(admitted.is_ok(), renderable);
        if !renderable {
            assert!(admitted.err().unwrap().contains("opaque"));
        }
        let parameters = source::record(session.value("poet_parameters").unwrap()).unwrap();
        assert_eq!(
            source::record(&parameters["opaque"]).unwrap()["big"],
            Value::U64(9007199254740993)
        );
    }
}
#[test]
fn known_layer_render_metadata_requires_exact_schema_and_shape_and_keeps_other_fields_strict() {
    use std::collections::BTreeMap;
    let mut session = SourceSession::default();
    let result = session
        .submit(
            &native_fixture(),
            &mut vibe::LocalHost::default(),
            Budget::default(),
        )
        .unwrap();
    let Value::Record(mut root) = result.value else {
        panic!()
    };
    let guard = BTreeMap::from([
        (
            "schema".into(),
            Value::String("poet.source-layer/v1".into()),
        ),
        ("base".into(), root["parts"].clone()),
        ("overrides".into(), Value::List(Vec::new())),
    ]);
    let admit = |root| {
        crate::graphical::assembly::admit_assembly(
            &Value::Record(root),
            crate::graphical::preview::PreviewLimits::default(),
        )
    };
    root.insert("_poet_source_layer".into(), Value::Record(guard.clone()));
    assert!(
        admit(root.clone()).is_ok(),
        "recognized metadata is inert; this does not prove its layer relation"
    );
    let mut migration = guard.clone();
    migration.insert(
        "schema".into(),
        Value::String("poet.source-layer/migrate-v1".into()),
    );
    root.insert("_poet_source_layer".into(), Value::Record(migration));
    assert!(admit(root.clone()).is_ok());
    for (key, value) in [
        ("schema", Value::String("unrecognized-layer".into())),
        ("base", Value::Null),
        ("base", Value::List(vec![Value::Record(BTreeMap::new())])),
        ("overrides", Value::Null),
        ("overrides", Value::List(vec![Value::Null])),
        ("unknown", Value::Bool(true)),
    ] {
        let mut malformed = guard.clone();
        malformed.insert(key.into(), value);
        root.insert("_poet_source_layer".into(), Value::Record(malformed));
        assert!(admit(root.clone())
            .err()
            .unwrap()
            .contains("source-layer render envelope"));
    }
    root.insert("_poet_source_layer".into(), Value::Record(guard));
    root.insert("unknown_root_metadata".into(), Value::Bool(true));
    assert!(admit(root).err().unwrap().contains("unknown_root_metadata"));
}
#[test]
fn exact_overrides_preserve_full_records_and_precision_while_geometry_changes() {
    let mut session = SourceSession::default();
    submit(&mut session, &fixture(), Budget::default()).unwrap();
    for edit in [width(1.5), material(), transform()] {
        let cell = generated(&session, &edit).unwrap();
        submit(&mut session, &cell, Budget::default()).unwrap();
    }
    let extension = super::assembly_source::Inventory::new(session.value("poet_parts").unwrap())
        .unwrap()
        .replace(
            "body",
            std::collections::BTreeMap::from([
                (
                    "opaque",
                    "{big:9007199254740993u64,negative_zero:-0.0,edited:true}".into(),
                ),
                (
                    "future_part_field",
                    "{payload:[9007199254740995u64],author:\"synthetic\"}".into(),
                ),
            ]),
        )
        .unwrap();
    submit(&mut session, &extension, Budget::default()).unwrap();
    let old = value_parts(&session).clone();
    for value in [2.0, 3.0] {
        let cell = generated(&session, &width(value)).unwrap();
        assert_eq!(
            cell.matches("poet_rebuild_parts(").count(),
            1,
            "one canonical evaluation"
        );
        submit(&mut session, &cell, Budget::default()).unwrap();
        for (old, new) in old.iter().zip(value_parts(&session)) {
            let old = source::record(old).unwrap();
            let new = source::record(new).unwrap();
            for key in ["material", "translation", "rotation", "scale", "opaque"] {
                assert!(
                    parameter_safety::equal(&old[key], &new[key]).unwrap(),
                    "{key}"
                );
            }
            if let Some(value) = old.get("future_part_field") {
                assert!(parameter_safety::equal(value, &new["future_part_field"]).unwrap());
            }
            let geometry = source::record(&new["geometry"]).unwrap();
            let Value::List(vertices) = &geometry["vertices"] else {
                panic!()
            };
            let Value::List(vertex) = &vertices[1] else {
                panic!()
            };
            assert_eq!(vertex[0], Value::F64(value));
            let opaque = source::record(&new["opaque"]).unwrap();
            assert_eq!(opaque["big"], Value::U64(9007199254740993));
            assert!(
                matches!(opaque["negative_zero"],Value::F64(n) if n.to_bits()==(-0.0f64).to_bits())
            );
        }
    }
    let retained = session.value("poet_parts").unwrap().clone();
    let mut fresh = SourceSession::default();
    for cell in session.sources() {
        submit(&mut fresh, cell, Budget::default()).unwrap();
    }
    assert!(parameter_safety::equal(&retained, fresh.value("poet_parts").unwrap()).unwrap());
}
#[test]
fn native_host_accepts_parameter_after_material_trs_and_restores_undo_rebuild() {
    let mut h = super::tests::host();
    let mut source = h.snapshot().unwrap().source;
    source = h.submit(&native_fixture(), source).unwrap().snapshot.source;
    for edit in [width(1.5), material(), transform()] {
        source = h.authoring_edit(edit, source).unwrap().snapshot.source;
    }
    let stale = source.clone();
    source = h
        .authoring_edit(width(2.5), source)
        .unwrap()
        .snapshot
        .source;
    let view = h.authoring_view(source.clone()).unwrap();
    assert_eq!(
        view.parameters
            .iter()
            .find(|(key, _)| key == "width")
            .unwrap()
            .1,
        2.5
    );
    assert_eq!(view.parts[0].translation, [3.0, 0.0, 0.0]);
    assert!(view.parts.iter().all(|p| p.roughness == 0.8));
    assert!(h.authoring_edit(width(4.0), stale).is_err());
    let held = h.undo(source).unwrap();
    let source = h.rebuild_execution(held.source).unwrap().snapshot.source;
    let view = h.authoring_view(source.clone()).unwrap();
    assert_eq!(
        view.parameters
            .iter()
            .find(|(key, _)| key == "width")
            .unwrap()
            .1,
        1.5
    );
    assert_eq!(view.parts[0].translation, [3.0, 0.0, 0.0]);
    let source = h
        .authoring_edit(width(3.5), source)
        .unwrap()
        .snapshot
        .source;
    assert!(h
        .source_cells(source)
        .unwrap()
        .last()
        .unwrap()
        .contains("poet_source_layer.overrides"));
}
#[test]
fn native_explicit_legacy_adoption_preserves_existing_overrides() {
    let legacy = native_fixture()
        .replace(
            "let mut poet_parameter_parts=poet_rebuild_parts(poet_parameters);",
            "",
        )
        .replace(
            "let mut poet_parts=poet_parameter_parts;",
            "let mut poet_parts=poet_rebuild_parts(poet_parameters);",
        );
    let mut h = super::tests::host();
    let source = h.snapshot().unwrap().source;
    let source = h.submit(&legacy, source).unwrap().snapshot.source;
    let source = h
        .authoring_edit(material(), source)
        .unwrap()
        .snapshot
        .source;
    let source = h
        .authoring_edit(transform(), source)
        .unwrap()
        .snapshot
        .source;
    let source = h
        .authoring_edit(AuthoringEdit::EnableSourceLayers, source)
        .unwrap()
        .snapshot
        .source;
    let source = h
        .authoring_edit(width(2.5), source)
        .unwrap()
        .snapshot
        .source;
    let view = h.authoring_view(source).unwrap();
    assert_eq!(view.parts[0].translation, [3.0, 0.0, 0.0]);
    assert!(view.parts.iter().all(|p| p.roughness == 0.8));
}
#[test]
fn guard_failure_budget_and_ambiguous_recipe_preserve_every_accepted_binding() {
    let mut session = SourceSession::default();
    submit(&mut session, &fixture(), Budget::default()).unwrap();
    let cell = generated(&session, &material()).unwrap();
    submit(&mut session, &cell, Budget::default()).unwrap();
    let parts = session.value("poet_parts").unwrap().clone();
    let base = session.value("poet_parameter_parts").unwrap().clone();
    let params = session.value("poet_parameters").unwrap().clone();
    let history = session.sources().to_vec();
    let revision = session.revision();
    let cell = generated(&session, &width(2.0)).unwrap();
    for invalid in [
        cell.replace("schema:\"poet.source-layer/v1\"", "schema:\"unknown\""),
        cell.replace(
            "poet_parameter_parts = poet_rebuild_parts(poet_parameters);",
            "poet_parameter_parts = [poet_parts[0],poet_parts[0]];",
        ),
    ] {
        assert!(submit(&mut session, &invalid, Budget::default()).is_err());
    }
    assert!(submit(&mut session, &cell, Budget::from_steps(1)).is_err());
    assert_eq!(session.revision(), revision);
    assert_eq!(session.sources(), history);
    assert_eq!(session.value("poet_parts"), Some(&parts));
    assert_eq!(session.value("poet_parameter_parts"), Some(&base));
    assert_eq!(session.value("poet_parameters"), Some(&params));
    assert!(session.value("poet_source_layer").is_none());
}
#[test]
fn topology_bound_coordinates_refuse_geometry_changes_without_remapping() {
    let fixture = fixture().replace(
        "triangles:[[0,1,2]]",
        "triangles:[[0,1,2]],normals:[[0.0,0.0,1.0],[0.0,0.0,1.0],[0.0,0.0,1.0]]",
    );
    let mut session = SourceSession::default();
    submit(&mut session, &fixture, Budget::default()).unwrap();
    let history = session.sources().to_vec();
    let parts = session.value("poet_parts").unwrap().clone();
    let cell = generated(&session, &width(2.0)).unwrap();
    assert!(submit(&mut session, &cell, Budget::default())
        .unwrap_err()
        .contains("topology correspondence"));
    assert_eq!(session.sources(), history);
    assert_eq!(session.value("poet_parts"), Some(&parts));
    let cell = generated(&session, &super::parameter_safety_tests::parameter(2.0)).unwrap();
    submit(&mut session, &cell, Budget::default()).unwrap(); // exact geometry correspondence is qualified
}

#[test]
fn generated_source_budget_is_charged_before_large_override_construction() {
    let mut session = SourceSession::default();
    submit(&mut session, &fixture(), Budget::default()).unwrap();
    let template = source::record(&value_parts(&session)[0]).unwrap();
    let mut base = Vec::new();
    let mut edited = Vec::new();
    for index in 0..128 {
        let mut part = template.clone();
        part.insert("id".into(), Value::String(format!("part-{index}")));
        part.insert("semantic_id".into(), Value::I64(1000 + index));
        for key in 0..128 {
            part.insert(format!("opaque_extension_{key}"), Value::U64(0));
        }
        base.push(Value::Record(part.clone()));
        for key in 0..128 {
            part.insert(format!("opaque_extension_{key}"), Value::U64(u64::MAX));
        }
        edited.push(Value::Record(part));
    }
    let error = source::edit_source_with_layers(
        &Value::List(edited),
        session.value("poet_parameters"),
        Some(&Value::List(base)),
        None,
        &width(2.0),
        2,
    )
    .unwrap_err();
    assert!(error.contains("generated-source budget"), "{error}");
    assert_eq!(session.revision(), 1);
}
