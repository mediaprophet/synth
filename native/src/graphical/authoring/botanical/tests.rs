use super::*;
use crate::graphical::botanical_graph_host::evaluate;
use vibe::Value;

fn rec(items: impl IntoIterator<Item = (&'static str, Value)>) -> Value {
    Value::Record(items.into_iter().map(|(k, v)| (k.to_owned(), v)).collect())
}
fn text(s: &str) -> Value {
    Value::String(s.into())
}
fn point(p: [f64; 3]) -> Value {
    Value::List(p.into_iter().map(Value::F64).collect())
}
fn recipe() -> BotanicalRecipe {
    BotanicalRecipe {
        version: 1,
        part_id: "branch".into(),
        organs: vec![
            Organ {
                key: "root".into(),
                parent: None,
                position_m: [0.0, 0.0, 0.0],
            },
            Organ {
                key: "axis".into(),
                parent: Some("root".into()),
                position_m: [0.0, 0.0, 1.0],
            },
            Organ {
                key: "lateral".into(),
                parent: Some("axis".into()),
                position_m: [0.5, 0.0, 1.5],
            },
            Organ {
                key: "tip".into(),
                parent: Some("axis".into()),
                position_m: [-0.5, 0.0, 1.5],
            },
        ],
        attractors_m: vec![[0.6, 0.0, 1.9], [-0.6, 0.0, 1.9]],
        node_length_m: 0.2,
        kill_radius_m: 0.05,
        perception_radius_m: 0.8,
        cone_angle_rad: std::f64::consts::TAU,
        max_iterations: 2,
        tip_radius_m: 0.02,
        pipe_exponent: 2.0,
        ring_slices: 8,
    }
}
fn as_value(r: &BotanicalRecipe) -> Value {
    rec([
        ("schema", text(SCHEMA)),
        ("version", Value::I64(1)),
        ("part_id", text(&r.part_id)),
        (
            "organs",
            Value::List(
                r.organs
                    .iter()
                    .map(|o| {
                        rec([
                            ("key", text(&o.key)),
                            (
                                "parent",
                                o.parent.as_ref().map(|s| text(s)).unwrap_or(Value::Null),
                            ),
                            ("position_m", point(o.position_m)),
                        ])
                    })
                    .collect(),
            ),
        ),
        (
            "attractors_m",
            Value::List(r.attractors_m.iter().copied().map(point).collect()),
        ),
        ("node_length_m", Value::F64(r.node_length_m)),
        ("kill_radius_m", Value::F64(r.kill_radius_m)),
        ("perception_radius_m", Value::F64(r.perception_radius_m)),
        ("cone_angle_rad", Value::F64(r.cone_angle_rad)),
        ("max_iterations", Value::I64(r.max_iterations as i64)),
        ("tip_radius_m", Value::F64(r.tip_radius_m)),
        ("pipe_exponent", Value::F64(r.pipe_exponent)),
        ("ring_slices", Value::I64(r.ring_slices as i64)),
    ])
}
fn geometry(vertices: &[[f64; 3]], triangles: &[[u32; 3]]) -> Value {
    rec([
        (
            "vertices",
            Value::List(vertices.iter().copied().map(point).collect()),
        ),
        (
            "triangles",
            Value::List(
                triangles
                    .iter()
                    .map(|t| Value::List(t.iter().map(|n| Value::I64(*n as i64)).collect()))
                    .collect(),
            ),
        ),
    ])
}
fn parts(geo: Value) -> Value {
    Value::List(vec![
        rec([
            ("id", text("branch")),
            ("geometry", geo),
            ("material", text("retained material")),
            ("uv_transform", text("retained UV transform")),
            ("detail", text("retained detail")),
        ]),
        rec([
            ("id", text("unrelated")),
            (
                "geometry",
                rec([
                    ("vertices", Value::List(vec![])),
                    ("triangles", Value::List(vec![])),
                ]),
            ),
            ("material", text("untouched")),
        ]),
    ])
}

#[test]
fn adopted_graph_replay_and_radius_edit_use_core_geometry() {
    let r = recipe();
    let limits = BotanicalLimits::default();
    let before = evaluate(&r, limits).unwrap();
    assert_eq!(before, evaluate(&r, limits).unwrap());
    assert_eq!(
        before
            .adopted
            .iter()
            .map(|o| o.key.as_str())
            .collect::<Vec<_>>(),
        ["root", "axis", "lateral", "tip"]
    );
    assert_eq!(before.adopted[1].parent_key.as_deref(), Some("root"));
    assert!(!before.vertices.is_empty() && !before.triangles.is_empty());
    let current = as_value(&r);
    assert_eq!(
        read_recipe(Some(&current), limits).unwrap(),
        Some(r.clone())
    );
    let scene = parts(geometry(&before.vertices, &before.triangles));
    let source = edit_source(
        &scene,
        Some(&current),
        &BotanicalEdit::SetTipRadius {
            part_id: "branch".into(),
            tip_radius_m: 0.04,
        },
        limits,
    )
    .unwrap();
    assert!(source.contains("poet_botanical = ") && source.contains("poet_parts.map"));
    assert!(
        source.contains("p.material")
            && source.contains("p.uv_transform")
            && source.contains("p.detail")
    );
    assert!(source.contains("else {p}"));
    let mut changed = r.clone();
    changed.tip_radius_m = 0.04;
    let after = evaluate(&changed, limits).unwrap();
    assert!(after.adopted[2].radius_m > before.adopted[2].radius_m);
    assert_ne!(after.vertices, before.vertices);
    assert_eq!(
        after.adopted.iter().map(|o| &o.key).collect::<Vec<_>>(),
        before.adopted.iter().map(|o| &o.key).collect::<Vec<_>>()
    );
}

#[test]
fn explicit_first_adoption_rebuilds_selected_geometry_and_keeps_unrelated_part() {
    let r = recipe();
    let limits = BotanicalLimits::default();
    let initial = parts(rec([
        ("vertices", Value::List(vec![])),
        ("triangles", Value::List(vec![])),
    ]));
    let source = edit_source(
        &initial,
        None,
        &BotanicalEdit::SetRecipe { recipe: r },
        limits,
    )
    .unwrap();
    assert!(source.contains("let mut poet_botanical = "));
    assert!(source.contains("if p.id == \"branch\""));
    assert!(source.contains("p.material") && source.contains("p.detail"));
    assert!(source.contains("else {p}"));
    assert!(source.contains("triangles:[["));
}

#[test]
fn position_edit_preserves_other_keys_and_changes_sweep() {
    let r = recipe();
    let limits = BotanicalLimits::default();
    let before = evaluate(&r, limits).unwrap();
    let scene = parts(geometry(&before.vertices, &before.triangles));
    let cell = edit_source(
        &scene,
        Some(&as_value(&r)),
        &BotanicalEdit::SetOrganPosition {
            part_id: "branch".into(),
            organ_key: "lateral".into(),
            position_m: [0.7, 0.0, 1.5],
        },
        limits,
    )
    .unwrap();
    assert!(cell.contains("0.7"));
    let mut changed = r.clone();
    changed.organs[2].position_m[0] = 0.7;
    let after = evaluate(&changed, limits).unwrap();
    assert_ne!(after.vertices, before.vertices);
    assert_eq!(after.adopted[3], before.adopted[3]);
}

#[test]
fn cycles_budget_foreign_target_and_drift_are_refused() {
    let r = recipe();
    let limits = BotanicalLimits::default();
    let before = evaluate(&r, limits).unwrap();
    let scene = parts(geometry(&before.vertices, &before.triangles));
    let mut cycle = r.clone();
    cycle.organs[1].parent = Some("tip".into());
    assert!(cycle.validate(limits).unwrap_err().contains("cycle"));
    let mut small = limits;
    small.max_vertices = 10;
    assert!(evaluate(&r, small).is_err());
    assert!(edit_source(
        &scene,
        Some(&as_value(&r)),
        &BotanicalEdit::SetTipRadius {
            part_id: "foreign".into(),
            tip_radius_m: 0.03
        },
        limits
    )
    .is_err());
    let drift = parts(rec([
        ("vertices", Value::List(vec![])),
        ("triangles", Value::List(vec![])),
    ]));
    assert!(validate_projection(&drift, Some(&r), &r, limits)
        .unwrap_err()
        .contains("differs"));
    let with_uv = parts(rec([
        ("vertices", Value::List(vec![])),
        ("triangles", Value::List(vec![])),
        ("uv0", Value::List(vec![])),
    ]));
    assert!(validate_projection(&with_uv, None, &r, limits)
        .unwrap_err()
        .contains("remap"));
}

#[test]
fn emitted_botanical_cell_evaluates_and_preserves_unrelated_source() {
    use vibe::{session::SourceSession, Budget, LocalHost};
    let mut session = SourceSession::default();
    let mut host = LocalHost::default();
    session.submit(r#"let mut poet_parts=[{id:"branch",geometry:{vertices:[],triangles:[]},material:"keep",uv_transform:"keep",detail:"keep"},{id:"unrelated",geometry:{vertices:[],triangles:[]}}];"#,
        &mut host,Budget::default()).unwrap();
    let before = session.value("poet_parts").unwrap().clone();
    let r = recipe();
    let source = edit_source(
        &before,
        None,
        &BotanicalEdit::SetRecipe { recipe: r.clone() },
        BotanicalLimits::default(),
    )
    .unwrap();
    session
        .submit(&source, &mut host, Budget::default())
        .unwrap();
    assert_eq!(
        read_recipe(session.value("poet_botanical"), BotanicalLimits::default()).unwrap(),
        Some(r.clone())
    );
    let after = session.value("poet_parts").unwrap();
    let (Value::List(before), Value::List(after_rows)) = (&before, after) else {
        panic!("parts")
    };
    assert_eq!(before[1], after_rows[1]);
    validate_projection(after, Some(&r), &r, BotanicalLimits::default()).unwrap();
}
