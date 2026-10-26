//! Software source-state cases. No GUI, production material or human acceptance.
use super::material_motion::{
    self as motion, MaterialMotionDraft, MaterialMotionEdit, MaterialMotionKey,
    MaterialMotionTrack, MotionInterpolation, MAX_KEYS, MAX_TIME_MS, MAX_TRACKS,
};
use std::collections::BTreeMap;
use vibe::{session::SourceSession, Budget, Value};

fn key(time_ms: u32, value: f64, interpolation: MotionInterpolation) -> MaterialMotionKey {
    MaterialMotionKey {
        time_ms,
        base_color: [value, 0.25, 0.5, 1.0],
        roughness: value,
        metallic: 1.0 - value,
        interpolation,
    }
}
fn draft(mode: MotionInterpolation) -> MaterialMotionDraft {
    MaterialMotionDraft {
        version: 1,
        material_id: 1,
        keys: vec![
            key(100, 0.0, mode),
            key(1100, 1.0, MotionInterpolation::Linear),
        ],
    }
}
fn session() -> SourceSession {
    let mut session = SourceSession::default();
    session
        .submit(
            &super::tests::source(),
            &mut vibe::LocalHost::default(),
            Budget::default(),
        )
        .unwrap();
    session
}
fn source(session: &SourceSession, edit: &MaterialMotionEdit) -> Result<String, String> {
    motion::edit_source(
        session.value("poet_parts").unwrap(),
        session.value("poet_parameters"),
        session.value("poet_parameter_parts"),
        session.value("poet_source_layer"),
        session.value("poet_material_motion"),
        edit,
        session.revision() + 1,
    )
}
fn apply(session: &mut SourceSession, edit: MaterialMotionEdit) -> String {
    let text = source(session, &edit).unwrap();
    let result = session
        .submit(&text, &mut vibe::LocalHost::default(), Budget::default())
        .unwrap_or_else(|e| panic!("generated material motion source failed: {e:?}\n{text}"));
    let Value::Record(result) = result.value else {
        panic!("record result")
    };
    assert_eq!(result.get("parts"), session.value("poet_parts"));
    text
}
fn material(session: &SourceSession) -> &BTreeMap<String, Value> {
    let Value::List(parts) = session.value("poet_parts").unwrap() else {
        panic!()
    };
    let Value::Record(part) = &parts[0] else {
        panic!()
    };
    let Value::Record(material) = &part["material"] else {
        panic!()
    };
    material
}
fn tracks(session: &SourceSession) -> Vec<MaterialMotionTrack> {
    motion::read_tracks(session.value("poet_material_motion")).unwrap()
}

#[test]
fn material_motion_uses_real_vibe_easing_with_explicit_outgoing_modes() {
    let linear = draft(MotionInterpolation::Linear);
    assert_eq!(linear.sample_factors(0).unwrap().1, 0.0);
    assert_eq!(linear.sample_factors(1100).unwrap().1, 1.0);
    assert_eq!(linear.sample_factors(MAX_TIME_MS).unwrap().1, 1.0);
    assert_eq!(linear.sample_factors(350).unwrap().1, 0.25);
    let cubic = draft(MotionInterpolation::CubicInOut);
    assert_eq!(cubic.sample_factors(350).unwrap().1, 0.0625);
    assert_eq!(cubic.sample_factors(600).unwrap().1, 0.5);
    let hold = draft(MotionInterpolation::Hold);
    assert_eq!(hold.sample_factors(1099).unwrap().1, 0.0);
    assert_eq!(hold.sample_factors(1100).unwrap().1, 1.0);
    assert!(linear.sample_factors(MAX_TIME_MS + 1).is_err());
}

#[test]
fn material_motion_source_edits_keys_and_samples_while_preserving_other_fields() {
    let mut session = session();
    let before = session.value("poet_parts").unwrap().clone();
    let first = apply(
        &mut session,
        MaterialMotionEdit::Set {
            draft: draft(MotionInterpolation::CubicInOut),
            sample_ms: 350,
        },
    );
    assert!(first.contains("let mut poet_material_motion"));
    assert_eq!(material(&session)["roughness"], Value::F64(0.0625));
    assert_eq!(material(&session)["metallic"], Value::F64(0.9375));
    assert_eq!(
        tracks(&session)[0].draft,
        draft(MotionInterpolation::CubicInOut)
    );
    let Value::List(old) = before else { panic!() };
    let Value::List(new) = session.value("poet_parts").unwrap() else {
        panic!()
    };
    for (old, new) in old.iter().zip(new) {
        let (Value::Record(old), Value::Record(new)) = (old, new) else {
            panic!()
        };
        for (field, value) in old {
            if field != "material" {
                assert_eq!(new.get(field), Some(value), "preserve {field}");
            }
        }
        let (Value::Record(old), Value::Record(new)) = (&old["material"], &new["material"]) else {
            panic!()
        };
        for (field, value) in old {
            if !["base_color", "roughness", "metallic"].contains(&field.as_str()) {
                assert_eq!(new.get(field), Some(value), "preserve material {field}");
            }
        }
    }
    let next = apply(
        &mut session,
        MaterialMotionEdit::Sample {
            material_id: 1,
            sample_ms: 600,
        },
    );
    assert!(!next.contains("let mut poet_material_motion"));
    assert_eq!(material(&session)["roughness"], Value::F64(0.5));
    assert_eq!(tracks(&session)[0].sample_ms, 600);
    assert_eq!(tracks(&session)[0].draft.keys.len(), 2);
    apply(
        &mut session,
        MaterialMotionEdit::Set {
            draft: draft(MotionInterpolation::Hold),
            sample_ms: 600,
        },
    );
    assert_eq!(material(&session)["roughness"], Value::F64(0.0));
    assert_eq!(
        tracks(&session).len(),
        1,
        "replacing a target is not a duplicate track"
    );
}

#[test]
fn material_motion_remove_keeps_sampled_factors_and_recipe_state_is_replayable() {
    let mut session = session();
    apply(
        &mut session,
        MaterialMotionEdit::Set {
            draft: draft(MotionInterpolation::Linear),
            sample_ms: 350,
        },
    );
    let accepted_keys = tracks(&session);
    let before_remove = session.sources().to_vec();
    let factors = session.value("poet_parts").unwrap().clone();
    apply(&mut session, MaterialMotionEdit::Remove { material_id: 1 });
    assert!(tracks(&session).is_empty());
    assert_eq!(session.value("poet_parts"), Some(&factors));
    // Replay the exact retained cell boundaries; concatenation is not assumed equivalent.
    let mut replay = SourceSession::default();
    for cell in before_remove {
        replay
            .submit(&cell, &mut vibe::LocalHost::default(), Budget::default())
            .unwrap();
    }
    assert_eq!(tracks(&replay), accepted_keys);
    assert_eq!(replay.value("poet_parts"), Some(&factors));
}

#[test]
fn material_motion_rejects_bad_drafts_and_missing_targets_without_source_changes() {
    let session = session();
    let cells = session.sources().to_vec();
    let parts = session.value("poet_parts").unwrap().clone();
    let mut cases = Vec::new();
    let mut bad = draft(MotionInterpolation::Linear);
    bad.version = 2;
    cases.push(bad);
    let mut bad = draft(MotionInterpolation::Linear);
    bad.material_id = 0;
    cases.push(bad);
    let mut bad = draft(MotionInterpolation::Linear);
    bad.material_id = 99;
    cases.push(bad);
    let mut bad = draft(MotionInterpolation::Linear);
    bad.keys.clear();
    cases.push(bad);
    let mut bad = draft(MotionInterpolation::Linear);
    bad.keys = vec![key(0, 0.5, MotionInterpolation::Linear); MAX_KEYS + 1];
    cases.push(bad);
    let mut bad = draft(MotionInterpolation::Linear);
    bad.keys[1].time_ms = 100;
    cases.push(bad);
    let mut bad = draft(MotionInterpolation::Linear);
    bad.keys.reverse();
    cases.push(bad);
    let mut bad = draft(MotionInterpolation::Linear);
    bad.keys[0].base_color[0] = f64::NAN;
    cases.push(bad);
    let mut bad = draft(MotionInterpolation::Linear);
    bad.keys[0].roughness = f64::INFINITY;
    cases.push(bad);
    let mut bad = draft(MotionInterpolation::Linear);
    bad.keys[0].metallic = -0.1;
    cases.push(bad);
    let mut bad = draft(MotionInterpolation::Linear);
    bad.keys[1].time_ms = MAX_TIME_MS + 1;
    cases.push(bad);
    for draft in cases {
        assert!(source(
            &session,
            &MaterialMotionEdit::Set {
                draft,
                sample_ms: 0
            }
        )
        .is_err());
    }
    for edit in [
        MaterialMotionEdit::Sample {
            material_id: 99,
            sample_ms: 0,
        },
        MaterialMotionEdit::Remove { material_id: 99 },
        MaterialMotionEdit::Sample {
            material_id: 1,
            sample_ms: 0,
        },
    ] {
        assert!(source(&session, &edit).is_err());
    }
    assert_eq!(session.sources(), cells.as_slice());
    assert_eq!(session.value("poet_parts"), Some(&parts));
}

#[test]
fn material_motion_strict_source_schema_never_silently_repairs_unknown_state() {
    assert!(motion::read_tracks(None).unwrap().is_empty());
    assert!(motion::read_tracks(Some(&Value::Null)).is_err());
    let mut session = session();
    apply(
        &mut session,
        MaterialMotionEdit::Set {
            draft: draft(MotionInterpolation::Linear),
            sample_ms: 350,
        },
    );
    let accepted = session.value("poet_material_motion").unwrap().clone();
    let Value::Record(mut changed) = accepted.clone() else {
        panic!()
    };
    changed.insert("schema".into(), Value::String("foreign/v2".into()));
    assert!(motion::read_tracks(Some(&Value::Record(changed))).is_err());
    let Value::Record(mut changed) = accepted.clone() else {
        panic!()
    };
    changed.insert("unknown".into(), Value::Bool(true));
    assert!(motion::read_tracks(Some(&Value::Record(changed))).is_err());
    let Value::Record(mut changed) = accepted else {
        panic!()
    };
    let Value::List(rows) = changed.get_mut("tracks").unwrap() else {
        panic!()
    };
    rows.push(rows[0].clone());
    assert!(
        motion::read_tracks(Some(&Value::Record(changed))).is_err(),
        "duplicate target"
    );
}

#[test]
fn material_motion_typed_requests_reject_unknown_fields_modes_and_fractional_time() {
    assert!(MaterialMotionEdit::Remove { material_id: 0 }
        .validate()
        .is_err());
    let valid = serde_json::json!({"operation":"set","draft":draft(MotionInterpolation::Linear),"sample_ms":350});
    assert!(serde_json::from_value::<MaterialMotionEdit>(valid.clone()).is_ok());
    let mut unknown = valid.clone();
    unknown["unexpected"] = serde_json::json!(true);
    assert!(serde_json::from_value::<MaterialMotionEdit>(unknown).is_err());
    let mut unknown = valid.clone();
    unknown["draft"]["keys"][0]["extra"] = serde_json::json!(1);
    assert!(serde_json::from_value::<MaterialMotionEdit>(unknown).is_err());
    let mut unknown = valid.clone();
    unknown["draft"]["keys"][0]["interpolation"] = serde_json::json!("expression");
    assert!(serde_json::from_value::<MaterialMotionEdit>(unknown).is_err());
    let mut fraction = valid;
    fraction["sample_ms"] = serde_json::json!(0.5);
    assert!(serde_json::from_value::<MaterialMotionEdit>(fraction).is_err());
}

#[test]
fn material_motion_refuses_non_linear_or_orphaned_targets_before_source_generation() {
    let mut session = session();
    apply(
        &mut session,
        MaterialMotionEdit::Set {
            draft: draft(MotionInterpolation::Linear),
            sample_ms: 350,
        },
    );
    let edit = MaterialMotionEdit::Sample {
        material_id: 1,
        sample_ms: 500,
    };
    let Value::List(mut parts) = session.value("poet_parts").unwrap().clone() else {
        panic!()
    };
    let Value::Record(part) = &mut parts[0] else {
        panic!()
    };
    let Value::Record(material) = part.get_mut("material").unwrap() else {
        panic!()
    };
    material.insert("colour_space".into(), Value::String("srgb".into()));
    assert!(motion::edit_source(
        &Value::List(parts),
        None,
        None,
        None,
        session.value("poet_material_motion"),
        &edit,
        4
    )
    .unwrap_err()
    .contains("linear"));
    let Value::Record(mut binding) = session.value("poet_material_motion").unwrap().clone() else {
        panic!()
    };
    let Value::List(rows) = binding.get_mut("tracks").unwrap() else {
        panic!()
    };
    let Value::Record(row) = &mut rows[0] else {
        panic!()
    };
    let Value::Record(draft) = row.get_mut("draft").unwrap() else {
        panic!()
    };
    draft.insert("material_id".into(), Value::U64(99));
    assert!(motion::edit_source(
        session.value("poet_parts").unwrap(),
        None,
        None,
        None,
        Some(&Value::Record(binding)),
        &edit,
        4
    )
    .unwrap_err()
    .contains("orphaned"));
}

#[test]
fn material_motion_track_limit_preserves_distinct_targets_and_rejects_extra() {
    let mut session = session();
    let parts = (1..=(MAX_TRACKS + 1)).map(|id| {
        let material = super::tests::MATERIAL.replacen("{id:1,", &format!("{{id:{id},"), 1);
        format!("{{id:\"part{id}\",semantic_id:{id},geometry:poet_parts[0].geometry,material:{material},translation:[0.0,0.0,0.0],rotation:[0.0,0.0,0.0],scale:[1.0,1.0,1.0]}}")
    }).collect::<Vec<_>>().join(",");
    let fixture = format!("poet_parts = [{parts}]; ({{parts:poet_parts}});");
    session
        .submit(&fixture, &mut vibe::LocalHost::default(), Budget::default())
        .unwrap();
    for id in 1..=MAX_TRACKS {
        let mut draft = draft(MotionInterpolation::Linear);
        draft.material_id = id as u64;
        apply(
            &mut session,
            MaterialMotionEdit::Set {
                draft,
                sample_ms: 0,
            },
        );
    }
    assert_eq!(tracks(&session).len(), MAX_TRACKS);
    assert_eq!(
        tracks(&session)
            .iter()
            .map(|t| t.draft.material_id)
            .collect::<Vec<_>>(),
        (1..=MAX_TRACKS as u64).collect::<Vec<_>>()
    );
    let mut extra = draft(MotionInterpolation::Linear);
    extra.material_id = (MAX_TRACKS + 1) as u64;
    assert!(source(
        &session,
        &MaterialMotionEdit::Set {
            draft: extra,
            sample_ms: 0
        }
    )
    .is_err());
}

#[test]
fn material_motion_total_generated_source_is_bounded_without_touching_accepted_work() {
    let mut session = session();
    apply(
        &mut session,
        MaterialMotionEdit::Set {
            draft: draft(MotionInterpolation::Linear),
            sample_ms: 350,
        },
    );
    let before = session.sources().to_vec();
    let Value::List(old_parts) = session.value("poet_parts").unwrap() else {
        panic!()
    };
    let mut parts = Vec::new();
    let Value::Record(mut binding) = session.value("poet_material_motion").unwrap().clone() else {
        panic!()
    };
    let Value::List(first) = &binding["tracks"] else {
        panic!()
    };
    let first = first[0].clone();
    let mut rows = Vec::new();
    for id in 1..=MAX_TRACKS {
        let Value::Record(mut part) = old_parts[0].clone() else {
            panic!()
        };
        part.insert("id".into(), Value::String(format!("limit{id}")));
        part.insert("semantic_id".into(), Value::U64(id as u64));
        let Value::Record(material) = part.get_mut("material").unwrap() else {
            panic!()
        };
        material.insert("id".into(), Value::U64(id as u64));
        parts.push(Value::Record(part));
        let Value::Record(mut row) = first.clone() else {
            panic!()
        };
        let Value::Record(draft) = row.get_mut("draft").unwrap() else {
            panic!()
        };
        draft.insert("material_id".into(), Value::U64(id as u64));
        let keys = (0..MAX_KEYS)
            .map(|i| {
                Value::Record(BTreeMap::from([
                    ("time_ms".into(), Value::U64(i as u64)),
                    (
                        "base_color".into(),
                        Value::List(vec![Value::F64(1.2345678901234568e-300); 4]),
                    ),
                    ("roughness".into(), Value::F64(1.2345678901234568e-300)),
                    ("metallic".into(), Value::F64(1.2345678901234568e-300)),
                    ("interpolation".into(), Value::String("linear".into())),
                ]))
            })
            .collect();
        draft.insert("keys".into(), Value::List(keys));
        rows.push(Value::Record(row));
    }
    binding.insert("tracks".into(), Value::List(rows));
    let binding = Value::Record(binding);
    assert_eq!(
        motion::read_tracks(Some(&binding)).unwrap().len(),
        MAX_TRACKS
    );
    let error = motion::edit_source(
        &Value::List(parts),
        None,
        None,
        None,
        Some(&binding),
        &MaterialMotionEdit::Sample {
            material_id: 1,
            sample_ms: 0,
        },
        5,
    )
    .unwrap_err();
    assert!(error.contains("60 KiB"), "{error}");
    assert_eq!(session.sources(), before.as_slice());
}
