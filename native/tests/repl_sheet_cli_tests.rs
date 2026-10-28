//! Real CLI subprocess checks; no GPU or workbook claim.
use std::{fs, process::Command};
fn run(source: &str, grants: &[&str]) -> std::process::Output {
    let dir=tempfile::tempdir().unwrap(); let path=dir.path().join("sheet.vibe");
    fs::write(&path,source).unwrap();
    let mut cmd=Command::new(env!("CARGO_BIN_EXE_poet")); cmd.arg("eval").arg(path);
    for grant in grants { cmd.arg("--grant").arg(grant); }
    cmd.output().unwrap()
}
#[test]
fn legacy_eval_without_grants_still_evaluates_pure_cell() {
    assert!(run("= 1 + 2", &[]).status.success());
}
#[test]
fn eval_ungranted_sheet_never_returns_echo_success() {
    let source=r#"requires [capability("capability.invoke")]; let target="Sheet.stats"; capability.invoke(target,{grid:[[1]]});"#;
    assert!(!run(source,&["capability.invoke"]).status.success());
}
#[cfg(feature="native-graphical")]
#[test]
fn eval_granted_sheet_executes_through_owner() {
    let source=r#"requires [capability("Sheet.stats")]; Sheet.stats([[1,2],[3,4]]);"#;
    let output=run(source,&["Sheet.stats"]); assert!(output.status.success(),"{}",String::from_utf8_lossy(&output.stderr));
    let text=String::from_utf8_lossy(&output.stdout); assert!(text.contains("sum") && text.contains("10"));
}
#[cfg(not(feature="native-graphical"))]
#[test]
fn eval_sheet_refuses_without_native_owner() {
    let source=r#"requires [capability("Sheet.stats")]; Sheet.stats([[1,2]]);"#;
    assert!(!run(source,&["Sheet.stats"]).status.success());
}
