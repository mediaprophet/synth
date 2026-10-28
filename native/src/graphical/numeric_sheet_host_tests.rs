use super::*;
use crate::graphical::numeric_sheet::SheetEdit;
use std::sync::atomic::{AtomicUsize, Ordering};
struct NoPixels;
impl PreviewProvider for NoPixels {
    fn preview(&mut self, _: SourceRef, _: &Value) -> Result<PreviewFrame, String> { Err("no GPU in sheet test".into()) }
}
fn fresh() -> CanonicalEditorHost {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let id = format!("urn:poet:numeric-sheet-test:{}:{}", std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed));
    studio_store::create_artifact_if_absent(StudioArtifact::new(&id, "Document", "sheet")).unwrap();
    let host = GeometryHost::new(vec!["capability.invoke".into(), "Sheet.set_cell".into()], 200_000);
    CanonicalEditorHost::open(id, SessionLimits::default(), host, Box::new(NoPixels)).unwrap()
}
#[test]
fn sheet_edit_preserves_unrelated_values_and_replays_accepted_source() {
    let mut host = fresh();
    let first = host.snapshot().unwrap().source;
    let first = host.submit("let mut poet_sheet=[[1,2],[3,4]]; let unrelated=99; poet_sheet;", first).unwrap().snapshot.source;
    let old = host.numeric_sheet_view(first.clone()).unwrap();
    assert_eq!(old.rows.len(), 2);
    assert!(host.numeric_sheet_edit(SheetEdit { row: 2, col: 0, value: 7.0, execution_epoch: old.execution_epoch.clone() }, first.clone()).is_err());
    let next = host.numeric_sheet_edit(SheetEdit { row: 0, col: 1, value: 7.0, execution_epoch: old.execution_epoch.clone() }, first.clone()).unwrap().snapshot.source;
    assert!(host.numeric_sheet_view(first).is_err());
    let new = host.numeric_sheet_view(next.clone()).unwrap();
    assert_eq!(new.rows[0][0], old.rows[0][0]);
    assert_eq!(new.rows[1], old.rows[1]);
    assert_eq!(new.rows[0][1], Value::F64(7.0));
    assert_eq!(host.session.value("unrelated"), Some(&Value::I64(99)));
    let rebuilt = host.rebuild_execution(next).unwrap().snapshot.source;
    assert!(host.numeric_sheet_edit(SheetEdit { row: 0, col: 0, value: 5.0, execution_epoch: new.execution_epoch.clone() }, rebuilt.clone()).is_err());
    assert_eq!(host.numeric_sheet_view(rebuilt).unwrap().rows, new.rows);
    assert_eq!(host.session.value("unrelated"), Some(&Value::I64(99)));
}
#[test]
fn refuses_ragged_nonfinite_and_immutable_bindings() {
    let mut host = fresh(); let start = host.snapshot().unwrap().source;
    let ragged = host.submit("let mut poet_sheet=[[1],[2,3]]; poet_sheet;", start).unwrap().snapshot.source;
    assert!(host.numeric_sheet_view(ragged).is_err());
    let mut host = fresh(); let start = host.snapshot().unwrap().source;
    let immutable = host.submit("let poet_sheet=[[1]]; poet_sheet;", start).unwrap().snapshot.source;
    assert!(host.numeric_sheet_edit(SheetEdit { row: 0, col: 0, value: 2.0, execution_epoch: host.execution_epoch.clone() }, immutable.clone()).is_err());
    assert!(host.numeric_sheet_edit(SheetEdit { row: 0, col: 0, value: f64::NAN, execution_epoch: host.execution_epoch.clone() }, immutable).is_err());
}

#[test]
fn sheet_float_source_replays_negative_zero_and_exponent() {
    let mut host = fresh(); let start = host.snapshot().unwrap().source;
    let current = host.submit("let mut poet_sheet=[[0.0,0.0]]; poet_sheet;", start).unwrap().snapshot.source;
    let current = host.numeric_sheet_edit(SheetEdit {row: 0, col: 0, value: -0.0, execution_epoch: host.execution_epoch.clone()}, current).unwrap().snapshot.source;
    let current = host.numeric_sheet_edit(SheetEdit {row: 0, col: 1, value: 1e100, execution_epoch: host.execution_epoch.clone()}, current).unwrap().snapshot.source;
    let result = host.numeric_sheet_view(current.clone()).unwrap();
    assert!(matches!(result.rows[0][0], Value::F64(v) if v.to_bits() == (-0.0f64).to_bits()));
    assert!(matches!(result.rows[0][1], Value::F64(v) if v == 1e100));
    let rebuilt = host.rebuild_execution(current).unwrap().snapshot.source;
    let replay = host.numeric_sheet_view(rebuilt).unwrap();
    assert!(matches!(replay.rows[0][0], Value::F64(v) if v.to_bits() == (-0.0f64).to_bits()));
    assert!(matches!(replay.rows[0][1], Value::F64(v) if v == 1e100));
}
