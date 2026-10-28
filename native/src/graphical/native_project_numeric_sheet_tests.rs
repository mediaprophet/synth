//! Signed software-agent fixture only; no human, native-window or GPU claim.
use super::*;
use crate::graphical::numeric_sheet::SheetEdit;
const CHILD: &str = "POET_NUMERIC_SHEET_TEST_CHILD";
struct SheetNoPixels;
impl PreviewProvider for SheetNoPixels {
    // Stateless fixture: no retained viewport exists to invalidate.
    fn retire_retained_viewport(&mut self) -> Result<(), String> { Ok(()) }
    fn preview(&mut self, _: SourceRef, _: &Value) -> Result<PreviewFrame, String> { Err("no pixels in numeric sheet fixture".into()) }
}
fn sheet_config(scope: Scope, root: &Path, sheet: bool) -> ProjectConfig {
    let mut c = config(scope, &root.join("hmc"), &root.join("library"));
    if sheet { c.geometry_grants.push("Sheet.set_cell".into()); } c
}
fn child(root: &Path) {
    let scope = Scope { project_id: std::env::var("POET_NUMERIC_TEST_PROJECT").unwrap(), document_id: std::env::var("POET_NUMERIC_TEST_DOCUMENT").unwrap(), source_id: std::env::var("POET_NUMERIC_TEST_SOURCE").unwrap(), domain: "Geometry".into() };
    assert!(studio_store::get_artifact_stamped(&scope.source_id).is_none());
    let authority = Arc::new(NativeAuthority::open_retained_current(&root.join("authority"), TrustedProject { project_id: scope.project_id.clone(), issuer_identifier: "fixture-issuer".into(), public_key: ed25519::public_from_seed(&AUTHORITY) }, 1024 * 1024, 256).unwrap());
    let session = login(&authority, "fixture-author", &AUTHOR);
    let project = NativeProject::open(sheet_config(scope.clone(), root, true), authority, session.clone(), Some("sheet.hmc"), Box::new(SheetNoPixels)).unwrap();
    let mut ui = project.connect(session).unwrap();
    let held = ui.snapshot().unwrap(); assert!(held.reconciliation_required);
    assert!(ui.numeric_sheet_view(held.source.clone()).is_err());
    let source = ui.rebuild_execution(held.source).unwrap().snapshot.source;
    let view = ui.numeric_sheet_view(source.clone()).unwrap();
    assert_eq!(view.rows[0][0], Value::I64(1));
    assert_eq!(view.rows[0][1], Value::F64(7.0));
    assert_eq!(view.rows[1], vec![Value::I64(3), Value::I64(4)]);
    let cells = ui.source_cells(source).unwrap();
    assert_eq!(cells.len(), 2); assert!(cells[0].contains("let unrelated=99"));
}
#[test]
fn signed_numeric_sheet_edit_hmc_reopen_and_revocation() {
    if let Some(root) = std::env::var_os(CHILD) { child(Path::new(&root)); return; }
    let temp = tempfile::tempdir().unwrap();
    for name in ["authority", "hmc", "library"] { std::fs::create_dir(temp.path().join(name)).unwrap(); }
    // Drop snapshots as well as the project: their publication stamps retain
    // an authority owner and therefore the journal lock.
    let (scope, mut admission) = {
        let (scope, admission, authority) = fixture(&temp.path().join("authority"));
        let session = login(&authority, "fixture-author", &AUTHOR);
        let project = NativeProject::open(sheet_config(scope.clone(), temp.path(), true), authority.clone(), session.clone(), None, Box::new(SheetNoPixels)).unwrap();
        let mut ui = project.connect(session).unwrap();
        let initial = ui.snapshot().unwrap();
        let source = ui.submit("let mut poet_sheet=[[1,2],[3,4]]; let unrelated=99; poet_sheet;", initial.source.clone()).unwrap().snapshot.source;
        let view = ui.numeric_sheet_view(source.clone()).unwrap();
        let edit = SheetEdit { row: 0, col: 1, value: 7.0, execution_epoch: view.execution_epoch.clone() };
        let changed = ui.numeric_sheet_edit(edit.clone(), source.clone()).unwrap().snapshot.source;
        assert!(ui.numeric_sheet_edit(edit, source).is_err());
        ui.save_checkpoint("sheet.hmc", ExpectedFile::Absent, changed).unwrap();
        (scope, admission)
    };
    // A separate process has no live Studio store and must reopen accepted HMC.
    let mut process = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "graphical::native_project::tests::numeric_sheet_tests::signed_numeric_sheet_edit_hmc_reopen_and_revocation", "--nocapture", "--test-threads=1"])
        .env(CHILD, temp.path()).env("POET_NUMERIC_TEST_PROJECT", &scope.project_id).env("POET_NUMERIC_TEST_DOCUMENT", &scope.document_id).env("POET_NUMERIC_TEST_SOURCE", &scope.source_id)
        .spawn().unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
    loop { if let Some(status) = process.try_wait().unwrap() { assert!(status.success()); break; } if std::time::Instant::now() >= deadline { let _ = process.kill(); let _ = process.wait(); panic!("numeric sheet fresh-process reopen exceeded deadline"); } std::thread::sleep(std::time::Duration::from_millis(10)); }
    // The original process still has the live source; reopen authority from retained signed journal.
    let authority = Arc::new(NativeAuthority::open_retained_current(&temp.path().join("authority"), TrustedProject { project_id: scope.project_id.clone(), issuer_identifier: "fixture-issuer".into(), public_key: ed25519::public_from_seed(&AUTHORITY) }, 1024 * 1024, 256).unwrap());
    let session = login(&authority, "fixture-author", &AUTHOR);
    let project = NativeProject::open(sheet_config(scope.clone(), temp.path(), true), authority.clone(), session.clone(), None, Box::new(SheetNoPixels)).unwrap();
    let mut ui = project.connect(session).unwrap();
    let held = ui.snapshot().unwrap(); let current = ui.rebuild_execution(held.source).unwrap().snapshot.source;
    let old = ui.numeric_sheet_view(current.clone()).unwrap();
    admission.sequence = 2; admission.subjects[0].revoked = true;
    authority.replace_admission(signed(admission)).unwrap();
    assert!(ui.numeric_sheet_view(current.clone()).is_err());
    assert!(ui.numeric_sheet_edit(SheetEdit { row: 0, col: 0, value: 4.0, execution_epoch: old.execution_epoch }, current).is_err());
}
#[test]
fn signed_sheet_edit_is_denied_without_the_target_capability() {
    let temp = tempfile::tempdir().unwrap();
    for name in ["authority", "hmc", "library"] { std::fs::create_dir(temp.path().join(name)).unwrap(); }
    let (scope, _, authority) = fixture(&temp.path().join("authority"));
    let session = login(&authority, "fixture-author", &AUTHOR);
    let project = NativeProject::open(sheet_config(scope, temp.path(), false), authority, session.clone(), None, Box::new(SheetNoPixels)).unwrap();
    let mut ui = project.connect(session).unwrap();
    let start = ui.snapshot().unwrap().source;
    let source = ui.submit("let mut poet_sheet=[[1]]; poet_sheet;", start).unwrap().snapshot.source;
    let view = ui.numeric_sheet_view(source.clone()).unwrap();
    assert!(ui.numeric_sheet_edit(SheetEdit { row: 0, col: 0, value: 2.0, execution_epoch: view.execution_epoch }, source.clone()).is_err());
    assert_eq!(ui.snapshot().unwrap().source, source);
}
