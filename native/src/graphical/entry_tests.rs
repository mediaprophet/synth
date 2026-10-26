//! Headless diagnostic factory tests; no source evaluation or window/GPU launch.
use super::*;
fn args(values: &[&str]) -> Vec<String> {
    values.iter().map(|v| (*v).into()).collect()
}

#[test]
fn configured_factory_exposes_save_with_empty_unevaluated_source() {
    let directory = tempfile::tempdir().unwrap();
    let prepared = prepare(&args(&[
        "--diagnostic-source",
        "--checkpoint-root",
        directory.path().to_str().unwrap(),
    ]))
    .unwrap();
    let snapshot = prepared.host.snapshot().unwrap();
    assert!(prepared.host.checkpoint_supported());
    assert!(snapshot.checkpoint_supported);
    assert!(snapshot.title.contains("LOCAL DIAGNOSTIC"));
    assert!(snapshot.authorisation_stamp.is_none());
    assert_eq!(snapshot.session_revision, 0);
    assert!(!snapshot.reconciliation_required);
    assert!(prepared
        .host
        .source_cells(snapshot.source)
        .unwrap()
        .is_empty());
    assert_eq!(prepared.initial_draft, diagnostic_source());
}

#[test]
fn old_diagnostic_new_source_remains_new_only_and_requires_no_root() {
    let first = prepare(&args(&["--diagnostic-new-source"])).unwrap();
    let second = prepare(&args(&["--diagnostic-new-source"])).unwrap();
    assert_ne!(
        first.host.snapshot().unwrap().artifact_id,
        second.host.snapshot().unwrap().artifact_id
    );
    assert!(!first.host.checkpoint_supported());
    assert!(prepare(&args(&["--diagnostic-new-source", "--reopen", "saved.hmc"])).is_err());
}

#[test]
fn traversal_missing_configuration_and_claimed_authority_are_denied() {
    for values in [
        vec!["--diagnostic-source", "--reopen", "saved.hmc"],
        vec!["--checkpoint-root", "."],
        vec!["--diagnostic-source", "--role", "steward"],
        vec!["--diagnostic-source", "--principal", "pretend-human"],
        vec!["--diagnostic-source", "--grant", "File.write"],
        vec!["--diagnostic-source", "--checkpoint-root"],
        vec![
            "--diagnostic-source",
            "--reopen",
            "../saved.hmc",
            "--checkpoint-root",
            ".",
        ],
        vec![
            "--diagnostic-source",
            "--reopen",
            "CON.hmc",
            "--checkpoint-root",
            ".",
        ],
        vec![
            "--diagnostic-source",
            "--check-source",
            "--reopen",
            "saved.hmc",
            "--checkpoint-root",
            ".",
        ],
    ] {
        assert!(prepare(&args(&values)).is_err());
    }
    for name in [
        "../outside.hmc",
        "C:/outside.hmc",
        "nested/file.hmc",
        "nested\\file.hmc",
        ".hmc",
        "file.json",
        "LPT1.hmc",
    ] {
        assert!(filename(name).is_err());
    }
}

#[test]
fn reopen_of_an_existing_store_id_never_replaces_that_record() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().to_str().unwrap();
    let mut prepared = prepare(&args(&["--diagnostic-source", "--checkpoint-root", root])).unwrap();
    let before = prepared.host.snapshot().unwrap();
    prepared
        .host
        .save_checkpoint(
            "empty.hmc",
            core::ExpectedFile::Absent,
            before.source.clone(),
        )
        .unwrap();
    assert!(prepare(&args(&[
        "--diagnostic-source",
        "--checkpoint-root",
        root,
        "--reopen",
        "empty.hmc"
    ]))
    .is_err());
    let after = prepared.host.snapshot().unwrap();
    assert_eq!(before.source, after.source);
    assert_eq!(after.session_revision, 0);
    assert!(prepared.host.source_cells(after.source).unwrap().is_empty());
}

#[test]
fn diagnostic_reopen_hold_also_blocks_an_empty_session_until_explicit_rebuild() {
    let prepared = prepare(&args(&["--diagnostic-source"])).unwrap();
    let mut held = DiagnosticEditor {
        inner: prepared.host,
        pending_reopen: true,
    };
    let before = held.snapshot().unwrap();
    assert!(before.reconciliation_required);
    assert!(held.check("1;").is_err());
    assert!(held.submit("1;", before.source.clone()).is_err());
    let rebuilt = held.rebuild_execution(before.source).unwrap();
    assert!(!rebuilt.snapshot.reconciliation_required);
    assert_eq!(rebuilt.snapshot.session_revision, 0);
    assert!(rebuilt.frame.is_none());
    assert!(held.check("1;").is_ok());
}
