//! SourceSession-to-GeometryHost-to-QualiaDB formula tests; no GPU/UI claim.
use super::*;
use vibe::session::{SessionLimits, SourceSession};
fn grants() -> Vec<String> { vec!["capability.invoke".into(), "Sheet.eval_formula".into()] }
const SOURCE: &str = r#"requires [capability("capability.invoke"), capability("Sheet.eval_formula")]; capability.invoke("Sheet.eval_formula", {formula:"=SUM(A1:B2)*1.15",grid:[[1,2],[3,4]]});"#;
#[test]
fn granted_formula_source_calls_existing_owner_and_allows_resubmission() {
    let grants = grants(); let mut session = SourceSession::new(grants.clone(), SessionLimits::default());
    let mut host = GeometryHost::new(grants, 200_000);
    let result = session.submit(SOURCE, &mut host, vibe::Budget::default()).unwrap();
    let Value::Record(result) = result.value else { panic!("formula result record") };
    assert_eq!(result.get("value"), Some(&Value::F64(11.5)));
    assert_eq!(result.get("cached"), Some(&Value::Bool(false)));
    assert_eq!(session.revision(), 1);
    host.begin_submission();
    let again = session.submit(SOURCE, &mut host, vibe::Budget::default()).unwrap();
    assert!(matches!(again.value, Value::Record(ref result) if result.get("value") == Some(&Value::F64(11.5))));
}
#[test]
fn formula_target_grant_dispatch_work_and_ast_limits_are_enforced() {
    let dynamic = r#"requires [capability("capability.invoke")]; let op="Sheet.eval_formula"; capability.invoke(op,{formula:"=1"});"#;
    let mut denied = GeometryHost::new(vec!["capability.invoke".into()], 200_000);
    let mut session = SourceSession::new(vec!["capability.invoke".into()], SessionLimits::default());
    assert!(session.submit(dynamic, &mut denied, vibe::Budget::default()).is_err());
    let g = grants();
    let mut host = GeometryHost::with_limits(g.clone(), GeometryHostLimits {max_vertices: 10_000, submission_work: 131_072, dispatches: 1, scratch_bytes: 1024*1024, profile_points: 32}).unwrap();
    let mut session = SourceSession::new(g, SessionLimits::default());
    assert!(session.submit(SOURCE, &mut host, vibe::Budget::default()).is_ok());
    assert!(session.submit(SOURCE, &mut host, vibe::Budget::default()).is_err());
    host.begin_submission();
    let forbidden = r#"requires [capability("capability.invoke"), capability("Sheet.eval_formula")]; capability.invoke("Sheet.eval_formula",{formula:"=NOW()"});"#;
    assert!(session.submit(forbidden, &mut host, vibe::Budget::default()).is_err());
    host.begin_submission();
    let outside = r#"requires [capability("capability.invoke"), capability("Sheet.eval_formula")]; capability.invoke("Sheet.eval_formula",{formula:"=A33",grid:[[1]]});"#;
    assert!(session.submit(outside, &mut host, vibe::Budget::default()).is_err());
    let g = grants(); let mut host = GeometryHost::with_limits(g.clone(), GeometryHostLimits {max_vertices: 10_000, submission_work: 131_071, dispatches: 2, scratch_bytes: 1024*1024, profile_points: 32}).unwrap();
    let mut session = SourceSession::new(g, SessionLimits::default());
    assert!(session.submit(SOURCE, &mut host, vibe::Budget::default()).is_err());
}

#[test]
fn canonical_source_rebuild_replays_formula_binding() {
    use crate::graphical::{controller::{CanonicalEditorHost, EditorHost, PreviewProvider}, preview::{SourceRef, PreviewFrame}};
    use qualia_core_db::poet_host::invoke::studio_store::{self, StudioArtifact};
    struct NoPixels;
    impl PreviewProvider for NoPixels {
        fn preview(&mut self, _: SourceRef, _: &Value) -> Result<PreviewFrame, String> { Err("headless formula test".into()) }
    }
    let id = format!("urn:poet:formula-rebuild-test:{}", std::process::id());
    studio_store::create_artifact_if_absent(StudioArtifact::new(&id,"Geometry","formula rebuild")).unwrap();
    let host = GeometryHost::new(grants(), 200_000);
    let mut editor = CanonicalEditorHost::open(id, SessionLimits::default(), host, Box::new(NoPixels)).unwrap();
    let initial = editor.snapshot().unwrap().source;
    let source = r#"requires [capability("capability.invoke"), capability("Sheet.eval_formula")]; let answer=capability.invoke("Sheet.eval_formula",{formula:"=SUM(A1:B2)*1.15",grid:[[1,2],[3,4]]}); let mut poet_sheet=[[0.0]]; poet_sheet;"#;
    let accepted = editor.submit(source, initial).unwrap().snapshot.source;
    let rebuilt = editor.rebuild_execution(accepted).unwrap().snapshot.source;
    assert_eq!(editor.source_cells(rebuilt.clone()).unwrap(), vec![source.to_string()]);
    let next = editor.submit("poet_sheet=[[answer.value]]; poet_sheet;", rebuilt).unwrap().snapshot.source;
    let sheet = editor.numeric_sheet_view(next).unwrap();
    assert_eq!(sheet.rows[0][0], Value::F64(11.5));
}
