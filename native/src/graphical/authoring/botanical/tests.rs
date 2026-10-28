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
        (
            "schema",
            text(if r.version == 1 {
                SCHEMA
            } else {
                super::recipe::CHAIN_SCHEMA
            }),
        ),
        ("version", Value::I64(r.version as i64)),
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
fn botanical_edit_refuses_selected_atlas_without_invalidating_existing_source() {
    let r = recipe();
    let limits = BotanicalLimits::default();
    let before = evaluate(&r, limits).unwrap();
    let mut scene = parts(geometry(&before.vertices, &before.triangles));
    let Value::List(rows) = &mut scene else {
        unreachable!()
    };
    let Value::Record(selected) = &mut rows[0] else {
        unreachable!()
    };
    selected.insert(
        "uv_atlas".into(),
        rec([
            ("version", Value::I64(1)),
            (
                "seams",
                Value::List(vec![Value::List(vec![Value::I64(0), Value::I64(1)])]),
            ),
            ("atlas_size", Value::I64(1024)),
            ("gutter_pixels", Value::I64(4)),
            ("texels_per_metre", Value::F64(64.0)),
            ("iterations", Value::I64(64)),
        ]),
    );
    let preserved = scene.clone();
    let current = as_value(&r);
    // Existing accepted source remains inspectable; only regeneration is refused.
    validate_projection(&scene, Some(&r), &r, limits).unwrap();
    let error = edit_source(
        &scene,
        Some(&current),
        &BotanicalEdit::SetTipRadius {
            part_id: "branch".into(),
            tip_radius_m: 0.04,
        },
        limits,
    )
    .unwrap_err();
    assert!(error.contains("UV atlas"));
    assert_eq!(scene, preserved);
    let Value::List(rows) = &mut scene else {
        unreachable!()
    };
    let Value::Record(selected) = &mut rows[0] else {
        unreachable!()
    };
    selected.insert("uv_atlas".into(), Value::Null);
    assert!(edit_source(
        &scene,
        Some(&current),
        &BotanicalEdit::SetTipRadius {
            part_id: "branch".into(),
            tip_radius_m: 0.04,
        },
        limits
    )
    .is_ok());
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

#[test]
fn explicit_v2_chain_sweep_removes_degree_two_caps_and_preserves_keys() {
    use std::collections::BTreeMap;
    let limits = BotanicalLimits::default();
    let mut legacy = recipe();
    legacy.organs[3].parent = Some("lateral".into());
    legacy.organs[3].position_m = [1.0, 0.0, 2.0];
    let old = evaluate(&legacy, limits).unwrap();
    assert_eq!(old, evaluate(&legacy, limits).unwrap());
    let mut chain = legacy.clone();
    chain.version = 2;
    let result = evaluate(&chain, limits).unwrap();
    assert_eq!(result, evaluate(&chain, limits).unwrap());
    assert_eq!(result.adopted, old.adopted);
    assert_eq!(old.vertices.len(), 3 * (2 * chain.ring_slices + 2));
    assert_eq!(result.vertices.len(), 4 * chain.ring_slices + 2);
    assert_eq!(result.triangles.len(), 8 * chain.ring_slices);
    // Every edge of this single capped chain has two incident triangles.
    let mut incidence = BTreeMap::<(u32, u32), usize>::new();
    for triangle in &result.triangles {
        for (a, b) in [
            (triangle[0], triangle[1]),
            (triangle[1], triangle[2]),
            (triangle[2], triangle[0]),
        ] {
            *incidence.entry((a.min(b), a.max(b))).or_default() += 1;
        }
    }
    assert!(incidence.values().all(|&n| n == 2));
    assert!(result.vertices.iter().flatten().all(|v| v.is_finite()));
    let mut insufficient = limits;
    insufficient.max_vertices = 1;
    assert!(evaluate(&chain, insufficient).is_err());
    let mut reversal = chain.clone();
    reversal.organs[2].position_m = reversal.organs[0].position_m;
    assert!(evaluate(&reversal, limits).is_err());
}

#[test]
fn v2_is_explicitly_versioned_and_legacy_source_is_preserved() {
    let limits = BotanicalLimits::default();
    let legacy = recipe();
    let old = evaluate(&legacy, limits).unwrap();
    let scene = parts(geometry(&old.vertices, &old.triangles));
    assert_eq!(
        read_recipe(Some(&as_value(&legacy)), limits).unwrap(),
        Some(legacy.clone())
    );
    let mut chain = legacy.clone();
    chain.version = 2;
    assert_eq!(
        read_recipe(Some(&as_value(&chain)), limits).unwrap(),
        Some(chain.clone())
    );
    let mut mismatch = as_value(&chain);
    let Value::Record(ref mut r) = mismatch else {
        unreachable!()
    };
    r.insert("schema".into(), text(SCHEMA));
    assert!(read_recipe(Some(&mismatch), limits).is_err());
    let cell = edit_source(
        &scene,
        Some(&as_value(&legacy)),
        &BotanicalEdit::SetRecipe { recipe: chain },
        limits,
    )
    .unwrap();
    assert!(cell.contains("poet.botanical/v2"));
    assert!(cell.contains("version:2"));
    let legacy_cell = edit_source(
        &scene,
        Some(&as_value(&legacy)),
        &BotanicalEdit::SetRecipe { recipe: legacy },
        limits,
    )
    .unwrap();
    assert!(legacy_cell.contains("poet.botanical/v1"));
    assert!(legacy_cell.contains("version:1"));
}

#[test]
fn v2_fork_is_still_separate_capped_chains() {
    let mut r = recipe();
    r.version = 2;
    let result = evaluate(&r, BotanicalLimits::default()).unwrap();
    // A root-to-fork edge plus each outgoing edge is a separate capped sweep.
    assert_eq!(result.vertices.len(), 3 * (2 * r.ring_slices + 2));
    assert_eq!(result.triangles.len(), 3 * 4 * r.ring_slices);
}

#[test]
fn v2_explicit_upgrade_replays_source_and_preserves_other_parts() {
    use vibe::{session::SourceSession, Budget, LocalHost};
    let limits = BotanicalLimits::default();
    let mut session = SourceSession::default();
    let mut host = LocalHost::default();
    session.submit(r#"let mut poet_parts=[{id:"branch",geometry:{vertices:[],triangles:[]},material:"keep"},{id:"unrelated",geometry:{vertices:[],triangles:[]},material:"other"}];"#, &mut host, Budget::default()).unwrap();
    let initial = session.value("poet_parts").unwrap().clone();
    let mut old = recipe();
    old.organs[3].parent = Some("lateral".into());
    old.organs[3].position_m = [1.0, 0.0, 2.0];
    let legacy_cell = edit_source(
        &initial,
        None,
        &BotanicalEdit::SetRecipe {
            recipe: old.clone(),
        },
        limits,
    )
    .unwrap();
    session
        .submit(&legacy_cell, &mut host, Budget::default())
        .unwrap();
    let before = session.value("poet_parts").unwrap().clone();
    let prior = session.value("poet_botanical").unwrap().clone();
    let mut new = old.clone();
    new.version = 2;
    let new_cell = edit_source(
        &before,
        Some(&prior),
        &BotanicalEdit::SetRecipe {
            recipe: new.clone(),
        },
        limits,
    )
    .unwrap();
    session
        .submit(&new_cell, &mut host, Budget::default())
        .unwrap();
    assert_eq!(
        read_recipe(session.value("poet_botanical"), limits).unwrap(),
        Some(new.clone())
    );
    let parts = session.value("poet_parts").unwrap();
    validate_projection(parts, Some(&new), &new, limits).unwrap();
    let (Value::List(old_parts), Value::List(new_parts)) = (&before, parts) else {
        panic!("parts")
    };
    assert_eq!(old_parts[1], new_parts[1]);
    // Replaying accepted source reproduces the same new recipe and projection.
    let mut replay = SourceSession::default();
    let mut replay_host = LocalHost::default();
    replay.submit(r#"let mut poet_parts=[{id:"branch",geometry:{vertices:[],triangles:[]},material:"keep"},{id:"unrelated",geometry:{vertices:[],triangles:[]},material:"other"}];"#, &mut replay_host, Budget::default()).unwrap();
    replay
        .submit(&legacy_cell, &mut replay_host, Budget::default())
        .unwrap();
    replay
        .submit(&new_cell, &mut replay_host, Budget::default())
        .unwrap();
    assert_eq!(session.value("poet_parts"), replay.value("poet_parts"));
    assert_eq!(
        session.value("poet_botanical"),
        replay.value("poet_botanical")
    );
}

#[test]
fn pinned_legacy_host_matches_all_v1_outputs() {
    use crate::graphical::botanical_graph_host::legacy_v1_snapshot;
    let branching = recipe();
    let mut chain = recipe();
    chain.organs[3].parent = Some("lateral".into());
    chain.organs[3].position_m = [1.0, 0.0, 2.0];
    for r in [branching, chain] {
        let baseline = legacy_v1_snapshot::evaluate(&r, BotanicalLimits::default()).unwrap();
        let current = evaluate(&r, BotanicalLimits::default()).unwrap();
        assert_eq!(baseline.vertices, current.vertices);
        assert_eq!(baseline.triangles, current.triangles);
        assert_eq!(baseline.proposed_positions_m, current.proposed_positions_m);
        assert_eq!(baseline.proposed_parents, current.proposed_parents);
        assert_eq!(baseline.iterations_run, current.iterations_run);
        assert_eq!(baseline.remaining_attractors, current.remaining_attractors);
        assert_eq!(baseline.adopted.len(), current.adopted.len());
        for (a, b) in baseline.adopted.iter().zip(&current.adopted) {
            assert_eq!(
                (&a.key, &a.parent_key, a.position_m, a.radius_m),
                (&b.key, &b.parent_key, b.position_m, b.radius_m)
            );
        }
    }
}

#[test]
fn v2_chain_work_is_reserved_and_invalid_upgrade_does_not_change_source() {
    let limits = BotanicalLimits::default();
    let mut old = recipe();
    old.organs[3].parent = Some("lateral".into());
    old.organs[3].position_m = [1.0, 0.0, 2.0];
    let baseline = evaluate(&old, limits).unwrap();
    let scene = parts(geometry(&baseline.vertices, &baseline.triangles));
    let untouched = scene.clone();
    let prior = as_value(&old);
    let mut tight = limits;
    tight.max_vertices = (old.organs.len() - 1) * (old.ring_slices * 2 + 2);
    assert!(evaluate(&old, tight).is_ok());
    let mut v2 = old.clone();
    v2.version = 2;
    assert!(evaluate(&v2, tight).is_err());
    tight.max_vertices += old.organs.len() * 2;
    assert!(evaluate(&v2, tight).is_ok());
    v2.organs[2].position_m = v2.organs[0].position_m;
    assert!(edit_source(
        &scene,
        Some(&prior),
        &BotanicalEdit::SetRecipe { recipe: v2 },
        limits
    )
    .is_err());
    assert_eq!(scene, untouched);
}

#[test]
fn explicit_v2_migration_refuses_authored_or_atlas_bound_mesh() {
    let limits = BotanicalLimits::default();
    let old = recipe();
    let generated = evaluate(&old, limits).unwrap();
    let scene = parts(geometry(&generated.vertices, &generated.triangles));
    let mut authored = scene.clone();
    let Value::List(ref mut rows) = authored else {
        unreachable!()
    };
    let Value::Record(ref mut selected) = rows[0] else {
        unreachable!()
    };
    let Value::Record(ref mut mesh) = selected.get_mut("geometry").unwrap() else {
        unreachable!()
    };
    // Use a valid, aligned stream so refusal proves topology migration is
    // guarded rather than merely rejecting malformed UV input.
    mesh.insert(
        "uv0".into(),
        Value::List(
            (0..generated.vertices.len())
                .map(|i| Value::List(vec![Value::F64(i as f64 / 100.0), Value::F64(0.5)]))
                .collect(),
        ),
    );
    let authored_before = authored.clone();
    assert!(edit_source(
        &authored,
        Some(&as_value(&old)),
        &BotanicalEdit::SetTipRadius {
            part_id: "branch".into(),
            tip_radius_m: old.tip_radius_m * 1.1,
        },
        limits,
    )
    .is_ok());
    let mut v2 = old.clone();
    v2.version = 2;
    assert!(edit_source(
        &authored,
        Some(&as_value(&old)),
        &BotanicalEdit::SetRecipe { recipe: v2.clone() },
        limits
    )
    .is_err());
    assert_eq!(authored, authored_before);
    let mut atlas = scene.clone();
    let Value::List(ref mut rows) = atlas else {
        unreachable!()
    };
    let Value::Record(ref mut selected) = rows[0] else {
        unreachable!()
    };
    selected.insert("uv_atlas".into(), rec([("version", Value::I64(1))]));
    assert!(edit_source(
        &atlas,
        Some(&as_value(&old)),
        &BotanicalEdit::SetRecipe { recipe: v2 },
        limits
    )
    .is_err());
}

#[test]
fn legacy_v1_straight_segment_fixed_topology_golden() {
    let mut r = recipe();
    r.organs.truncate(2);
    r.max_iterations = 0;
    let output = evaluate(&r, BotanicalLimits::default()).unwrap();
    assert_eq!(output.vertices.len(), 18);
    assert_eq!(output.triangles.len(), 32);
    assert_eq!(output.triangles[0], [0, 1, 9]);
    assert_eq!(output.triangles[1], [0, 9, 8]);
    assert_eq!(output.triangles[16], [16, 1, 0]);
    assert_eq!(output.triangles[24], [17, 8, 9]);
    assert_eq!(output.vertices[0][2], 0.0);
    assert_eq!(output.vertices[8][2], 1.0);
    let first = output.vertices[0];
    assert!((first[0].hypot(first[1]) - r.tip_radius_m).abs() < 1e-12);
}

#[test]
fn botanical_radius_edit_preserves_valid_authored_uv0_exactly() {
    use vibe::{session::SourceSession, Budget, LocalHost};
    let limits = BotanicalLimits::default();
    let r = recipe();
    let mut session = SourceSession::default();
    let mut host = LocalHost::default();
    session.submit(r#"let mut poet_parts=[{id:"branch",geometry:{vertices:[],triangles:[]},material:"keep",detail:"keep"},{id:"unrelated",geometry:{vertices:[],triangles:[]}}];"#, &mut host, Budget::default()).unwrap();
    let first = edit_source(session.value("poet_parts").unwrap(), None, &BotanicalEdit::SetRecipe { recipe: r.clone() }, limits).unwrap();
    session.submit(&first, &mut host, Budget::default()).unwrap();
    let before = evaluate(&r, limits).unwrap();
    // Distinct per-vertex coordinates prove correspondence rather than merely
    // preserving a constant stream; the existing native owner stores them.
    let uv_source = (0..before.vertices.len()).map(|i| format!("[{i}.0,{}.0]", i + 1)).collect::<Vec<_>>().join(",");
    let attach = format!("poet_parts = poet_parts.map(|p| if p.id == \"branch\" {{ {{id:p.id,geometry:{{vertices:p.geometry.vertices,triangles:p.geometry.triangles,uv0:[{uv_source}]}},material:p.material,detail:p.detail}} }} else {{p}}); ({{parts:poet_parts}});");
    session.submit(&attach, &mut host, Budget::default()).unwrap();
    let original_parts = session.value("poet_parts").unwrap().clone();
    let Value::List(rows) = &original_parts else { panic!("parts") };
    let Value::Record(part) = &rows[0] else { panic!("part") };
    let Value::Record(geometry) = &part["geometry"] else { panic!("geometry") };
    let original_uv = geometry["uv0"].clone();
    let current = session.value("poet_botanical").unwrap().clone();
    let source = edit_source(&original_parts, Some(&current), &BotanicalEdit::SetTipRadius { part_id:"branch".into(), tip_radius_m:0.04 }, limits).unwrap();
    let mut revised = r.clone(); revised.tip_radius_m = 0.04;
    let after_mesh = evaluate(&revised, limits).unwrap();
    assert_eq!(before.vertices.len(), after_mesh.vertices.len());
    assert_eq!(before.triangles, after_mesh.triangles);
    session.submit(&source, &mut host, Budget::default()).unwrap();
    let Value::List(rows) = session.value("poet_parts").unwrap() else { panic!("parts") };
    let Value::Record(part) = &rows[0] else { panic!("part") };
    let Value::Record(geometry) = &part["geometry"] else { panic!("geometry") };
    assert_eq!(geometry["uv0"], original_uv);
    let Value::List(original_rows) = &original_parts else { unreachable!() };
    assert_eq!(rows[1], original_rows[1]);
    assert_ne!(geometry["vertices"], match &original_parts { Value::List(rows) => match &rows[0] { Value::Record(p) => match &p["geometry"] { Value::Record(g) => g["vertices"].clone(), _ => unreachable!() }, _ => unreachable!() }, _ => unreachable!() });
}

#[test]
fn botanical_uv0_refuses_pose_topology_malformed_and_richer_streams_without_mutation() {
    let limits = BotanicalLimits::default();
    let r = recipe();
    let before = evaluate(&r, limits).unwrap();
    let mut scene = parts(geometry(&before.vertices, &before.triangles));
    let Value::List(rows) = &mut scene else { unreachable!() };
    let Value::Record(part) = &mut rows[0] else { unreachable!() };
    let Value::Record(geo) = part.get_mut("geometry").unwrap() else { unreachable!() };
    let uv = Value::List((0..before.vertices.len()).map(|i| Value::List(vec![Value::F64(i as f64), Value::F64(0.0)])).collect());
    geo.insert("uv0".into(), uv.clone());
    let untouched = scene.clone();
    let current = as_value(&r);
    let moved = BotanicalEdit::SetOrganPosition { part_id:"branch".into(), organ_key:"lateral".into(), position_m:[0.7,0.0,1.5] };
    assert!(edit_source(&scene, Some(&current), &moved, limits).is_err());
    let mut changed = r.clone(); changed.ring_slices += 1;
    assert!(edit_source(&scene, Some(&current), &BotanicalEdit::SetRecipe { recipe:changed }, limits).is_err());
    let mut changed = r.clone(); changed.version = 2;
    assert!(edit_source(&scene, Some(&current), &BotanicalEdit::SetRecipe { recipe:changed }, limits).is_err());
    assert!(edit_source(&scene, None, &BotanicalEdit::SetRecipe { recipe:r.clone() }, limits).is_err());
    assert_eq!(scene, untouched);
    let mut drifted = scene.clone();
    let Value::List(rows) = &mut drifted else { unreachable!() };
    let Value::Record(part) = &mut rows[0] else { unreachable!() };
    let Value::Record(geo) = part.get_mut("geometry").unwrap() else { unreachable!() };
    let Value::List(vertices) = geo.get_mut("vertices").unwrap() else { unreachable!() };
    let Value::List(first) = &mut vertices[0] else { unreachable!() };
    first[0] = Value::F64(123.0);
    let drift_snapshot = drifted.clone();
    assert!(edit_source(&drifted, Some(&current), &BotanicalEdit::SetTipRadius { part_id:"branch".into(), tip_radius_m:0.04 }, limits).is_err());
    assert_eq!(drifted, drift_snapshot);
    let Value::List(rows) = &mut scene else { unreachable!() };
    let Value::Record(part) = &mut rows[0] else { unreachable!() };
    let Value::Record(geo) = part.get_mut("geometry").unwrap() else { unreachable!() };
    geo.insert("uv0".into(), Value::List(vec![Value::F64(f64::NAN)]));
    assert!(edit_source(&scene, Some(&current), &BotanicalEdit::SetTipRadius { part_id:"branch".into(), tip_radius_m:0.04 }, limits).is_err());
    for invalid in [
        Value::List(vec![Value::List(vec![Value::F64(0.0)]); before.vertices.len()]),
        Value::List(vec![Value::F64(0.0); before.vertices.len() * 2 - 1]),
        { let mut v = vec![Value::List(vec![Value::F64(0.0), Value::F64(0.0)]); before.vertices.len()]; v[1] = Value::F64(0.0); Value::List(v) },
    ] {
        let Value::List(rows) = &mut scene else { unreachable!() };
        let Value::Record(part) = &mut rows[0] else { unreachable!() };
        let Value::Record(geo) = part.get_mut("geometry").unwrap() else { unreachable!() };
        geo.insert("uv0".into(), invalid);
        assert!(edit_source(&scene, Some(&current), &BotanicalEdit::SetTipRadius { part_id:"branch".into(), tip_radius_m:0.04 }, limits).is_err());
    }
    let Value::List(rows) = &mut scene else { unreachable!() };
    let Value::Record(part) = &mut rows[0] else { unreachable!() };
    let Value::Record(geo) = part.get_mut("geometry").unwrap() else { unreachable!() };
    geo.insert("uv0".into(), uv);
    geo.insert("normals".into(), Value::List(vec![]));
    assert!(edit_source(&scene, Some(&current), &BotanicalEdit::SetTipRadius { part_id:"branch".into(), tip_radius_m:0.04 }, limits).is_err());
    let Value::List(rows) = &mut scene else { unreachable!() };
    let Value::Record(part) = &mut rows[0] else { unreachable!() };
    let Value::Record(geo) = part.get_mut("geometry").unwrap() else { unreachable!() };
    geo.remove("normals");
    geo.insert("skin".into(), Value::List(vec![]));
    let with_skin = scene.clone();
    assert!(edit_source(&scene, Some(&current), &BotanicalEdit::SetTipRadius { part_id:"branch".into(), tip_radius_m:0.04 }, limits).is_err());
    assert_eq!(scene, with_skin);
}
