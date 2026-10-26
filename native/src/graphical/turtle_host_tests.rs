use super::*;
use vibe::session::{SessionLimits, SourceSession};

const ID: &str = "ComputationalGeometry.turtle_3d";
fn grants() -> Vec<String> {
    vec!["capability.invoke".into(), ID.into()]
}
fn limits() -> GeometryHostLimits {
    GeometryHostLimits {
        max_vertices: 10_000,
        submission_work: 20_000,
        dispatches: 8,
        scratch_bytes: 16 * 1024 * 1024,
        profile_points: 1024,
    }
}
fn source(args: &str) -> String {
    format!("requires [capability(\"capability.invoke\"), capability(\"{ID}\")]; capability.invoke(\"{ID}\", {args});")
}
fn basic() -> &'static str {
    "{commands:[[\"F\",1.0]], ring_slices:8, capacity:32}"
}
fn run_error(args: &str, config: GeometryHostLimits) -> Diagnostic {
    let mut host = GeometryHost::with_limits(grants(), config).unwrap();
    let mut session = SourceSession::new(grants(), SessionLimits::default());
    let error = session
        .submit(&source(args), &mut host, vibe::Budget::default())
        .unwrap_err();
    assert_eq!(
        session.revision(),
        0,
        "refused input must not promote source"
    );
    error.diagnostic
}

#[test]
fn turtle_source_session_reaches_actual_core_branch_leaf_and_preview() {
    let mut host = GeometryHost::with_limits(grants(), limits()).unwrap();
    let mut session = SourceSession::new(grants(), SessionLimits::default());
    let args = r#"{commands:[
        ["!",0.2],["F",1.0,0.1],["["],["+",45.0],["F",0.5,0.05],["]"],
        ["T",1.0,0.0,0.0,0.1],["G"],["F",0.4],["F",0.2],["g"],
        ["{"],["."],["f",0.1],["."],["+",90.0],["f",0.1],["."],["}"]
        ],ring_slices:8,capacity:128}"#;
    let result = session
        .submit(&source(args), &mut host, vibe::Budget::default())
        .unwrap();
    let mesh = crate::graphical::preview::mesh_from_invoke(
        &result.value,
        crate::graphical::preview::PreviewLimits::default(),
    )
    .unwrap();
    // A welded pair saves one ring; the leaf adds three vertices and one face.
    assert_eq!(mesh.positions.len(), 59);
    assert_eq!(mesh.triangles.len(), 65);
    assert!(mesh.positions.iter().any(|p| p[2] > 1.5));
    assert!(mesh.positions.iter().flatten().all(|n| n.is_finite()));
    assert!(mesh
        .triangles
        .iter()
        .flatten()
        .all(|&i| (i as usize) < mesh.positions.len()));
    assert_eq!(result.revision, 1);
}

#[test]
fn turtle_dynamic_target_requires_actual_runtime_grant() {
    let allow = vec!["capability.invoke".into()];
    let mut host = GeometryHost::with_limits(allow.clone(), limits()).unwrap();
    let mut session = SourceSession::new(allow, SessionLimits::default());
    let source = format!(
        "requires [capability(\"capability.invoke\")]; let op=\"{ID}\"; capability.invoke(op, {});",
        basic()
    );
    let error = session
        .submit(&source, &mut host, vibe::Budget::default())
        .unwrap_err();
    assert_eq!(error.diagnostic.code, DiagCode::E300);
    assert_eq!(session.revision(), 0);
}

#[test]
fn turtle_capacity_tessellation_and_scratch_fail_before_dispatch() {
    for args in [
        "{commands:[[\"F\",1.0]],capacity:10001}",
        "{commands:[[\"F\",1.0]],capacity:0}",
        "{commands:[[\"F\",1.0]],capacity:8,ring_slices:8}",
        "{commands:[[\"F\",1.0]],capacity:32,ring_slices:33}",
        "{commands:[[\"F\",1.0]],capacity:32,ring_slices:8.0}",
        "{commands:[[\"F\",1.0]],capacity:32.0}",
    ] {
        assert_eq!(run_error(args, limits()).code, DiagCode::E400, "{args}");
    }
    let mut config = limits();
    config.scratch_bytes = 1;
    assert_eq!(run_error(basic(), config).code, DiagCode::E400);
}

#[test]
fn turtle_malformed_dimensions_and_unbalanced_scopes_do_not_promote() {
    for commands in [
        "[]",
        "[[]]",
        "[[\"unknown\"]]",
        "[[\"F\"]]",
        "[[\"F\",1.0,0.2,0.3]]",
        "[[\"F\",\"1\"]]",
        "[[\"F\",0.0]]",
        "[[\"f\",-1.0]]",
        "[[\"!\",-1.0]]",
        "[[\"F\",1.0,0.0]]",
        "[[\"+\",1000001.0]]",
        "[[\"T\",0.0,0.0,0.0,1.0]]",
        "[[\"]\"],[\"F\",1.0]]",
        "[[\"[\"],[\"F\",1.0]]",
        "[[\".\"],[\"F\",1.0]]",
        "[[\"{\"],[\"F\",1.0]]",
        "[[\"}\"],[\"F\",1.0]]",
        "[[\"{\"],[\"{\"],[\"F\",1.0]]",
        "[[\"G\"],[\"G\"],[\"F\",1.0],[\"g\"],[\"g\"]]",
        "[[\"g\"],[\"F\",1.0]]",
        "[[\"G\"],[\"F\",1.0]]",
    ] {
        assert_eq!(
            run_error(&format!("{{commands:{commands},capacity:128}}"), limits()).code,
            DiagCode::E400,
            "{commands}"
        );
    }
    assert_eq!(
        run_error("{commands:[[\"F\",1.0]],extra:true}", limits()).code,
        DiagCode::E400
    );
}

#[test]
fn turtle_stack_and_polygon_limits_are_checked_before_core_allocation() {
    let pushes = std::iter::repeat_n("[\"[\"]", 65)
        .collect::<Vec<_>>()
        .join(",");
    assert_eq!(
        run_error(
            &format!("{{commands:[{pushes},[\"F\",1.0]],capacity:128}}"),
            limits()
        )
        .code,
        DiagCode::E400
    );
    let dots = std::iter::repeat_n("[\".\"]", 33)
        .collect::<Vec<_>>()
        .join(",");
    assert_eq!(
        run_error(
            &format!("{{commands:[[\"{{\"],{dots},[\"}}\"]],capacity:128}}"),
            limits()
        )
        .code,
        DiagCode::E400
    );
}

#[test]
fn turtle_nonfinite_numbers_and_wrong_root_are_refused() {
    for n in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let args = Value::Record(std::collections::BTreeMap::from([
            (
                "commands".into(),
                Value::List(vec![Value::List(vec![
                    Value::String("F".into()),
                    Value::F64(n),
                ])]),
            ),
            ("capacity".into(), Value::U64(32)),
        ]));
        let mut host = GeometryHost::with_limits(grants(), limits()).unwrap();
        assert_eq!(
            host.capability_invoke(ID, &args, Span::point(0))
                .unwrap_err()
                .code,
            DiagCode::E400
        );
        assert_eq!(host.remaining_dispatches, 8);
    }
    let mut host = GeometryHost::with_limits(grants(), limits()).unwrap();
    assert_eq!(
        host.capability_invoke(ID, &Value::Null, Span::point(0))
            .unwrap_err()
            .code,
        DiagCode::E400
    );
}

#[test]
fn turtle_work_and_dispatches_are_cumulative_until_submission_reset() {
    for dispatch_limit in [false, true] {
        let mut config = limits();
        if dispatch_limit {
            config.dispatches = 1;
        } else {
            config.submission_work = 200;
        }
        let mut host = GeometryHost::with_limits(grants(), config).unwrap();
        let mut session = SourceSession::new(grants(), SessionLimits::default());
        let first = session
            .submit(&source(basic()), &mut host, vibe::Budget::default())
            .unwrap();
        assert_eq!(first.revision, 1);
        let error = session
            .submit(&source(basic()), &mut host, vibe::Budget::default())
            .unwrap_err();
        assert_eq!(error.diagnostic.code, DiagCode::E400);
        assert_eq!(session.revision(), 1);
        host.begin_submission();
        assert!(session
            .submit(&source(basic()), &mut host, vibe::Budget::default())
            .is_ok());
    }
    let mut config = limits();
    config.submission_work = 100;
    assert_eq!(run_error(basic(), config).code, DiagCode::E400);
}
