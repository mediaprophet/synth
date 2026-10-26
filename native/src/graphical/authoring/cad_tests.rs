//! Canonical revolution output checks on ephemeral authoring source only.
use super::*;
use crate::graphical::{
    controller::{CanonicalEditorHost, EditorHost, PreviewProvider},
    geometry_host::{GeometryHost, GeometryHostLimits},
    preview::{PreviewFrame, SourceRef},
};
use vibe::{
    Budget, Value,
    session::{SessionLimits, SourceSession},
};
fn grants() -> Vec<String> {
    vec![
        "capability.invoke".into(),
        "ComputationalGeometry.revolve_profile".into(),
    ]
}
fn points() -> Vec<[f64; 2]> {
    vec![[0.2, 1.0], [0.35, 0.5], [0.25, 0.0]]
}
fn recipe() -> GeometryRecipe {
    GeometryRecipe::RevolveProfile {
        points: points(),
        segments: 8,
    }
}
fn host(actual: Vec<String>, work: usize, scratch: usize, points: usize) -> GeometryHost {
    GeometryHost::with_limits(
        actual,
        GeometryHostLimits {
            max_vertices: 10000,
            submission_work: work,
            dispatches: 1,
            scratch_bytes: scratch,
            profile_points: points,
        },
    )
    .unwrap()
}
fn session() -> SourceSession {
    let mut s = SourceSession::new(grants(), SessionLimits::default());
    s.submit(
        &super::tests::source(),
        &mut vibe::LocalHost::default(),
        Budget::default(),
    )
    .unwrap();
    s
}
fn field<'a>(value: &'a Value, key: &str) -> &'a Value {
    let Value::Record(r) = value else {
        panic!("record")
    };
    r.get(key).unwrap()
}
fn list(v: &Value) -> &[Value] {
    let Value::List(v) = v else { panic!("list") };
    v
}
fn replace(s: &mut SourceSession, h: &mut GeometryHost) -> Result<(), String> {
    let edit = AuthoringEdit::ReplaceGeometry {
        id: "lever".into(),
        geometry: recipe(),
    };
    let generated = source::edit_source(
        s.value("poet_parts").unwrap(),
        None,
        &edit,
        s.revision() + 1,
    )?;
    assert_eq!(generated.matches("capability.invoke(").count(), 1);
    assert!(!generated.contains(".map("));
    vibe::parse_program(&generated).map_err(|e| format!("{e:?}"))?;
    s.submit(&generated, h, Budget::default())
        .map(|_| ())
        .map_err(|e| format!("{e:?}"))
}
#[test]
fn cad_revolution_native_counts_outward_winding_open_ends_and_one_dispatch() {
    let mut s = session();
    let before = s.value("poet_parts").unwrap().clone();
    let mut h = host(grants(), 10000, 16 * 1024 * 1024, 1024);
    replace(&mut s, &mut h).unwrap();
    let after = s.value("poet_parts").unwrap().clone();
    assert_eq!(list(&after)[0], list(&before)[0]);
    for key in [
        "id",
        "semantic_id",
        "parent",
        "translation",
        "rotation",
        "scale",
        "material",
    ] {
        assert_eq!(field(&list(&after)[1], key), field(&list(&before)[1], key));
    }
    let g = field(&list(&after)[1], "geometry");
    let vertices: Vec<[f64; 3]> = list(field(g, "vertices"))
        .iter()
        .map(|v| std::array::from_fn(|i| source::number(&list(v)[i]).unwrap()))
        .collect();
    let triangles: Vec<[usize; 3]> = list(field(g, "triangles"))
        .iter()
        .map(|v| std::array::from_fn(|i| source::number(&list(v)[i]).unwrap() as usize))
        .collect();
    assert_eq!(vertices.len(), 24);
    assert_eq!(triangles.len(), 32);
    let mut edges = std::collections::BTreeMap::new();
    for t in &triangles {
        assert!(t.iter().all(|i| *i < vertices.len()));
        let (a, b, c) = (vertices[t[0]], vertices[t[1]], vertices[t[2]]);
        let u = std::array::from_fn::<_, 3, _>(|i| b[i] - a[i]);
        let v = std::array::from_fn::<_, 3, _>(|i| c[i] - a[i]);
        let normal = [
            u[1] * v[2] - u[2] * v[1],
            u[2] * v[0] - u[0] * v[2],
            u[0] * v[1] - u[1] * v[0],
        ];
        let radial = [(a[0] + b[0] + c[0]) / 3.0, 0.0, (a[2] + b[2] + c[2]) / 3.0];
        assert!(
            normal[0] * radial[0] + normal[2] * radial[2] > 0.0,
            "Core winding must point radially outward for descending-height input"
        );
        for (a, b) in [(t[0], t[1]), (t[1], t[2]), (t[2], t[0])] {
            *edges.entry((a.min(b), a.max(b))).or_insert(0usize) += 1;
        }
    }
    assert_eq!(
        edges.values().filter(|count| **count == 1).count(),
        16,
        "two open ring ends, eight edges each; seam closed"
    );
    assert!(edges.values().all(|count| *count == 1 || *count == 2));
    let revision = s.revision();
    assert!(replace(&mut s, &mut h).unwrap_err().contains("E400"));
    assert_eq!(s.revision(), revision);
    assert_eq!(s.value("poet_parts"), Some(&after));
}
#[test]
fn cad_revolution_actual_grant_work_scratch_and_point_budgets_are_enforced() {
    for (actual, work, scratch, code) in [
        (vec!["capability.invoke".into()], 10000, 1024 * 1024, "E300"),
        (grants(), 1, 1024 * 1024, "E400"),
        (grants(), 10000, 1, "E400"),
    ] {
        let mut s = session();
        let held = s.value("poet_parts").unwrap().clone();
        let revision = s.revision();
        assert!(
            replace(&mut s, &mut host(actual, work, scratch, 1024))
                .unwrap_err()
                .contains(code)
        );
        assert_eq!(s.revision(), revision);
        assert_eq!(s.value("poet_parts"), Some(&held));
    }
    let raw = "requires [capability(\"capability.invoke\"),capability(\"ComputationalGeometry.revolve_profile\")];capability.invoke(\"ComputationalGeometry.revolve_profile\",{points:[[0.2,3.0],[0.2,2.0],[0.2,1.0],[0.2,0.0]],segments:8});";
    let mut s = session();
    let revision = s.revision();
    let held = s.value("poet_parts").unwrap().clone();
    let e = s
        .submit(
            raw,
            &mut host(grants(), 10000, 1024 * 1024, 3),
            Budget::default(),
        )
        .unwrap_err();
    assert!(format!("{e:?}").contains("E400"));
    assert_eq!(s.revision(), revision);
    assert_eq!(s.value("poet_parts"), Some(&held));
}
#[test]
fn cad_revolution_host_rejects_nonmonotonic_degenerate_units_caps_and_unknown_fields() {
    for args in [
        "{points:[[0.2,0.0],[0.2,1.0]],segments:8}",
        "{points:[[0.2,1.0],[0.2,1.0]],segments:8}",
        "{points:[[0.2,1000000.0],[0.2,999999.9999]],segments:8}",
        "{points:[[0.0,1.0],[0.2,0.0]],segments:8}",
        "{points:[[0.2,1.0],[0.2,0.0]],segments:2}",
        "{points:[[0.2,1.0],[0.2,0.0]],segments:257}",
        "{points:[[0.2,1.0],[0.2,0.0]],segments:8,caps:true}",
        "{points:[[0.2,1.0],[0.2,0.0]],segments:8,units:\"mm\"}",
        "{points:[[0.2,1.0,0.0],[0.2,0.0,0.0]],segments:8}",
    ] {
        let mut s = session();
        let revision = s.revision();
        let held = s.value("poet_parts").unwrap().clone();
        let request = format!(
            "requires [capability(\"capability.invoke\"),capability(\"ComputationalGeometry.revolve_profile\")];capability.invoke(\"ComputationalGeometry.revolve_profile\",{args});"
        );
        let mut actual = host(grants(), 10000, 1024 * 1024, 1024);
        let e = s
            .submit(&request, &mut actual, Budget::default())
            .unwrap_err();
        assert!(
            format!("{e:?}").contains("E400"),
            "actual host refusal required: {e:?}"
        );
        assert_eq!(s.revision(), revision);
        assert_eq!(s.value("poet_parts"), Some(&held));
        replace(&mut s, &mut actual).expect("rejected host input must not consume a dispatch");
    }
    assert!(super::cad_controls::recipe("[[0.2,1],[0.2,0]]", "8").is_ok());
    assert!(super::cad_controls::recipe("[[\"20 mm\",1],[0.2,0]]", "8").is_err());
    assert!(
        serde_json::from_str::<GeometryRecipe>(
            r#"{"kind":"RevolveProfile","points":[[0.2,1],[0.2,0]],"segments":8,"caps":true}"#
        )
        .is_err()
    );
}
struct NoPixels;
impl PreviewProvider for NoPixels {
    fn validate(&self, v: &Value) -> Result<(), String> {
        crate::graphical::assembly::admit_assembly(
            v,
            crate::graphical::preview::PreviewLimits::default(),
        )
        .map(|_| ())
    }
    fn preview(&mut self, _: SourceRef, _: &Value) -> Result<PreviewFrame, String> {
        Err("No GPU requested by revolution source test".into())
    }
}
#[test]
fn cad_revolution_source_to_hmc_preserves_source_and_stale_undo_guards() {
    use qualia_core_db::poet_host::invoke::studio_store::{
        self, StudioArtifact, checkpoint as core,
    };
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
    let id = format!(
        "urn:poet:revolve-test:{}:{}",
        std::process::id(),
        NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    );
    studio_store::create_artifact_if_absent(StudioArtifact::new(
        &id,
        "Geometry",
        "Synthetic revolution test",
    ))
    .unwrap();
    let mut h = CanonicalEditorHost::open(
        id,
        SessionLimits::default(),
        GeometryHost::new(grants(), 10000),
        Box::new(NoPixels),
    )
    .unwrap();
    let first = h.snapshot().unwrap();
    h.submit(&super::tests::source(), first.source).unwrap();
    let before = h.snapshot().unwrap();
    let edit = AuthoringEdit::AddGeometry {
        id: "lathe-surface".into(),
        semantic_id: 103,
        material_from: "lever".into(),
        geometry: recipe(),
    };
    let result = h
        .authoring_edit(edit.clone(), before.source.clone())
        .unwrap();
    assert!(h.authoring_edit(edit, before.source).is_err());
    let cells = h.source_cells(result.snapshot.source.clone()).unwrap();
    assert_eq!(
        cells.last().unwrap().matches("capability.invoke(").count(),
        1
    );
    let assets = h
        .compiled_assets("synthetic-revolve", result.snapshot.source.clone())
        .unwrap();
    let limits = core::CheckpointLimits::default();
    let candidate = core::capture(
        &result.snapshot.source.artifact_id,
        &result.snapshot.source.stamp,
        limits,
    )
    .unwrap();
    let bytes = core::encode(&candidate, assets, limits).unwrap();
    let reopened = core::decode(&bytes, limits).unwrap();
    for kind in ["10d", "q42"] {
        assert!(reopened.inventory().iter().any(|r| r.kind == kind));
    }
    let undone = h.undo(result.snapshot.source).unwrap();
    assert!(undone.reconciliation_required);
    let rebuilt = h.rebuild_execution(undone.source).unwrap();
    assert_eq!(
        h.authoring_view(rebuilt.snapshot.source)
            .unwrap()
            .parts
            .len(),
        2
    );
}
