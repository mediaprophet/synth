//! Signed software-actor integration: real source/authority/HMC, no GUI pixels
//! or human/domain acceptance. The first test re-enters in a fresh child process.
use super::*;
use crate::graphical::{
    authoring::{
        material_motion::{
            MaterialMotionDraft, MaterialMotionEdit, MaterialMotionKey, MotionInterpolation,
        },
        AuthoringEdit,
    },
    jobs, native_dispatch,
    session_host::SessionEditorHost,
};
use std::sync::Mutex;

const CHILD_CASE: &str = "POET_MATERIAL_MOTION_CASE_ROOT";

struct MotionNoPixels;
impl PreviewProvider for MotionNoPixels {
    // This fixture does not retain a viewport or rendered frame.
    fn retire_retained_viewport(&mut self) -> Result<(), String> { Ok(()) }
    fn validate(&self, value: &Value) -> Result<(), String> {
        crate::graphical::assembly::admit_assembly(value, PreviewLimits::default()).map(|_| ())
    }
    fn preview(&mut self, _: SourceRef, _: &Value) -> Result<PreviewFrame, String> {
        Err("Software motion fixture deliberately provides no GUI/GPU pixels".into())
    }
}

fn assembly_source() -> &'static str {
    r#"
let m={id:1,colour_space:"linear",base_color:[0.3,0.4,0.5,1.0],emissive:[0.0,0.0,0.0],metallic:0.1,roughness:0.5,specular:1.0,alpha_cutoff:0.5,opacity_mode:"Opaque",double_sided:true};
let tri={vertices:[[0.0,0.0,0.0],[1.0,0.0,0.0],[0.0,1.0,0.0]],triangles:[[0,1,2]]};
let mut poet_parts=[{id:"root",semantic_id:101,geometry:tri,material:m,translation:[0.0,0.0,0.0],rotation:[0.0,0.0,0.0],scale:[1.0,1.0,1.0]}];
({parts:poet_parts});
"#
}

fn draft() -> MaterialMotionDraft {
    MaterialMotionDraft {
        version: 1,
        material_id: 1,
        keys: vec![
            MaterialMotionKey {
                time_ms: 0,
                base_color: [0.0, 0.0, 0.0, 1.0],
                roughness: 0.25,
                metallic: 0.0,
                interpolation: MotionInterpolation::Linear,
            },
            MaterialMotionKey {
                time_ms: 1000,
                base_color: [1.0, 0.5, 0.25, 1.0],
                roughness: 0.75,
                metallic: 1.0,
                interpolation: MotionInterpolation::Linear,
            },
        ],
    }
}

fn block_on<F: std::future::Future>(future: F) -> F::Output {
    struct Wake(std::thread::Thread);
    impl std::task::Wake for Wake {
        fn wake(self: Arc<Self>) {
            self.0.unpark();
        }
        fn wake_by_ref(self: &Arc<Self>) {
            self.0.unpark();
        }
    }
    let waker = std::task::Waker::from(Arc::new(Wake(std::thread::current())));
    let mut cx = std::task::Context::from_waker(&waker);
    let mut future = std::pin::pin!(future);
    loop {
        match future.as_mut().poll(&mut cx) {
            std::task::Poll::Ready(value) => return value,
            std::task::Poll::Pending => std::thread::park(),
        }
    }
}

fn check_view(host: &dyn EditorHost, sample_ms: u32) {
    let current = host.snapshot().unwrap();
    let view = host.authoring_view(current.source).unwrap();
    assert_eq!(view.material_motion.len(), 1);
    assert_eq!(
        serde_json::to_value(&view.material_motion[0].draft).unwrap(),
        serde_json::to_value(draft()).unwrap()
    );
    assert_eq!(view.material_motion[0].sample_ms, sample_ms);
    let part = view.parts.iter().find(|part| part.id == "root").unwrap();
    assert_eq!((part.vertices, part.triangles, part.material_id), (3, 1, 1));
    let (colour, roughness, metallic) = match sample_ms {
        0 => ([0.0, 0.0, 0.0, 1.0], 0.25, 0.0),
        500 => ([0.5, 0.25, 0.125, 1.0], 0.5, 0.5),
        _ => panic!("fixture only checks endpoints and midpoint"),
    };
    assert_eq!(part.base_color, colour);
    assert_eq!(part.roughness, roughness);
    assert_eq!(part.metallic, metallic);
}

fn history_round_trip(host: &mut dyn EditorHost) {
    let before = host.snapshot().unwrap();
    let cells = host.source_cells(before.source.clone()).unwrap();
    let undone = host.undo(before.source).unwrap();
    assert!(undone.reconciliation_required);
    assert!(host.authoring_view(undone.source.clone()).is_err());
    host.rebuild_execution(undone.source).unwrap();
    check_view(host, 0);
    let current = host.snapshot().unwrap();
    let redone = host.redo(current.source).unwrap();
    assert!(redone.reconciliation_required);
    let rebuilt = host.rebuild_execution(redone.source).unwrap();
    assert!(rebuilt.frame.is_none());
    assert_eq!(host.source_cells(rebuilt.snapshot.source).unwrap(), cells);
    check_view(host, 500);
}

fn child_reopen(root: &Path) {
    let witness: serde_json::Value =
        serde_json::from_slice(&std::fs::read(root.join("witness.json")).unwrap()).unwrap();
    let scope = Scope {
        project_id: witness["project_id"].as_str().unwrap().into(),
        document_id: witness["document_id"].as_str().unwrap().into(),
        source_id: witness["source_id"].as_str().unwrap().into(),
        domain: "Geometry".into(),
    };
    assert!(
        studio_store::get_artifact_stamped(&scope.source_id).is_none(),
        "the child must begin without the parent's process-local source"
    );
    let authority = Arc::new(
        NativeAuthority::open_retained_current(
            &root.join("authority"),
            TrustedProject {
                project_id: scope.project_id.clone(),
                issuer_identifier: "fixture-issuer".into(),
                public_key: ed25519::public_from_seed(&AUTHORITY),
            },
            1024 * 1024,
            256,
        )
        .unwrap(),
    );
    let author = login(&authority, "fixture-author", &AUTHOR);
    let project = NativeProject::open(
        config(scope.clone(), &root.join("hmc"), &root.join("library")),
        authority,
        author.clone(),
        Some("motion.hmc"),
        Box::new(MotionNoPixels),
    )
    .unwrap();
    let mut ui = project.connect(author).unwrap();
    let restored = ui.snapshot().unwrap();
    assert!(restored.reconciliation_required);
    assert_eq!(
        json!(ui.source_cells(restored.source.clone()).unwrap()),
        witness["source_cells"]
    );
    let loaded =
        checkpoint::load(&root.join("hmc/motion.hmc"), CheckpointLimits::default()).unwrap();
    assert_eq!(
        loaded.artifact(),
        &studio_store::get_artifact_stamped(&scope.source_id)
            .unwrap()
            .0,
        "HMC must restore the exact source artifact and its retained history"
    );
    ui.rebuild_execution(restored.source).unwrap();
    check_view(&ui, 500);
    history_round_trip(&mut ui);
    std::fs::write(
        root.join("child-recovered.json"),
        serde_json::to_vec(&json!({
            "source_id":scope.source_id,"sample_ms":500,"source_and_history_restored":true,
        }))
        .unwrap(),
    )
    .unwrap();
}

#[test]
fn signed_material_motion_ui_mcp_history_and_hmc_reopen() {
    if let Some(root) = std::env::var_os(CHILD_CASE) {
        child_reopen(Path::new(&root));
        return;
    }
    let temp = tempfile::tempdir().unwrap();
    for name in ["authority", "hmc", "library"] {
        std::fs::create_dir(temp.path().join(name)).unwrap();
    }
    {
        let (scope, _, authority) = fixture(&temp.path().join("authority"));
        let author = login(&authority, "fixture-author", &AUTHOR);
        let project = Arc::new(
            NativeProject::open(
                config(
                    scope.clone(),
                    &temp.path().join("hmc"),
                    &temp.path().join("library"),
                ),
                authority,
                author.clone(),
                None,
                Box::new(MotionNoPixels),
            )
            .unwrap(),
        );
        let sessions = Arc::new(Mutex::new(Some(author.clone())));
        let mut operator = SessionEditorHost::new(project.clone(), sessions);
        let initial = operator.snapshot().unwrap();
        let submitted = operator.submit(assembly_source(), initial.source).unwrap();
        assert!(submitted.frame.is_none());
        let current = submitted.snapshot;
        let (worker, handle) = jobs::Worker::start(Box::new(operator)).unwrap();
        let reply = block_on(native_dispatch::run(
            handle.clone(),
            jobs::Action::AuthoringEdit {
                edit: AuthoringEdit::MaterialMotion {
                    edit: MaterialMotionEdit::Set {
                        draft: draft(),
                        sample_ms: 0,
                    },
                },
                expected: current.source.clone(),
            },
            jobs::Context::from_snapshot(&current),
        ))
        .unwrap();
        assert!(
            reply.operation_succeeded,
            "native motion intent: {}",
            reply.message
        );
        let mut ui = project.connect(author.clone()).unwrap();
        check_view(&ui, 0);
        let mut mcp = project.mcp_backend(author).unwrap();
        let read = mcp.call("poet_source_read", json!({})).unwrap();
        assert!(read["source_cells"]
            .as_array()
            .unwrap()
            .iter()
            .any(|cell| cell.as_str().unwrap().contains("poet_material_motion")));
        let inspected = mcp
            .call(
                "poet_material_motion_inspect",
                json!({"context":read["context"]}),
            )
            .unwrap();
        assert_eq!(
            inspected["material_motion"][0]["draft"],
            serde_json::to_value(draft()).unwrap()
        );
        assert_eq!(inspected["source_revision"], read["source_revision"]);
        let sampled = mcp.call("poet_material_motion_edit", json!({
            "context":read["context"],
            "edit":serde_json::to_value(MaterialMotionEdit::Sample {material_id:1,sample_ms:500}).unwrap(),
        })).unwrap();
        assert_eq!(sampled["read_required"], true);
        assert_eq!(sampled["preview_rendered"], false);
        assert!(
            mcp.call(
                "poet_material_motion_inspect",
                json!({"context":read["context"]})
            )
            .is_err(),
            "a successful mutation consumes its connection context"
        );
        check_view(&ui, 500);
        history_round_trip(&mut ui);
        let snapshot = ui.snapshot().unwrap();
        let cells = ui.source_cells(snapshot.source.clone()).unwrap();
        let saved = ui
            .save_checkpoint("motion.hmc", ExpectedFile::Absent, snapshot.source)
            .unwrap();
        let bytes = std::fs::read(&saved.path).unwrap();
        let digest: [u8; 32] = Sha256::digest(&bytes).into();
        assert_eq!(digest, saved.digest);
        let loaded = checkpoint::load(&saved.path, CheckpointLimits::default()).unwrap();
        assert_eq!(
            loaded.artifact(),
            &studio_store::get_artifact_stamped(&scope.source_id)
                .unwrap()
                .0
        );
        assert!(!loaded.artifact().history.is_empty());
        std::fs::write(
            temp.path().join("witness.json"),
            serde_json::to_vec(&json!({
                "project_id":scope.project_id,"document_id":scope.document_id,
                "source_id":scope.source_id,"source_cells":cells,
            }))
            .unwrap(),
        )
        .unwrap();
        drop(mcp);
        drop(ui);
        worker.finish().unwrap();
        drop(handle);
        drop(project);
    }
    // Re-enter only this test, with an empty Studio global store and retained
    // authority/HMC files. Fixture signing seeds stay in test code, never files.
    let current = std::thread::current();
    let test_name = current
        .name()
        .expect("Rust test thread supplies exact test name");
    let output = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", test_name, "--nocapture", "--test-threads=1"])
        .env(CHILD_CASE, temp.path())
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "fresh-process HMC restore failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        temp.path().join("child-recovered.json").is_file(),
        "a successful zero-test child exit must never count as recovery evidence"
    );
}

#[test]
fn signed_material_motion_rejects_stale_observer_revoked_and_malformed_edits() {
    let temp = tempfile::tempdir().unwrap();
    for name in ["authority", "hmc", "library"] {
        std::fs::create_dir(temp.path().join(name)).unwrap();
    }
    let (scope, mut admission, authority) = fixture(&temp.path().join("authority"));
    admission.sequence = 2;
    admission.subjects[1].membership.role = ProjectRole::Observer;
    authority
        .replace_admission(signed(admission.clone()))
        .unwrap();
    let author = login(&authority, "fixture-author", &AUTHOR);
    let project = NativeProject::open(
        config(
            scope.clone(),
            &temp.path().join("hmc"),
            &temp.path().join("library"),
        ),
        authority.clone(),
        author.clone(),
        None,
        Box::new(MotionNoPixels),
    )
    .unwrap();
    let mut ui = project.connect(author.clone()).unwrap();
    let initial = ui.snapshot().unwrap();
    ui.submit(assembly_source(), initial.source).unwrap();
    let mut mcp = project.mcp_backend(author).unwrap();
    let stale = mcp.call("poet_source_read", json!({})).unwrap();
    let current = ui.snapshot().unwrap();
    ui.authoring_edit(
        AuthoringEdit::MaterialMotion {
            edit: MaterialMotionEdit::Set {
                draft: draft(),
                sample_ms: 0,
            },
        },
        current.source,
    )
    .unwrap();
    let retained = studio_store::get_artifact_stamped(&scope.source_id).unwrap();
    let sample = serde_json::to_value(MaterialMotionEdit::Sample {
        material_id: 1,
        sample_ms: 500,
    })
    .unwrap();
    assert!(mcp
        .call(
            "poet_material_motion_edit",
            json!({"context":stale["context"],"edit":sample})
        )
        .is_err());
    assert_eq!(
        studio_store::get_artifact_stamped(&scope.source_id).unwrap(),
        retained
    );

    for nested_extra in [false, true] {
        let read = mcp.call("poet_source_read", json!({})).unwrap();
        let mut args = json!({"context":read["context"],"edit":sample});
        if nested_extra {
            args["edit"]
                .as_object_mut()
                .unwrap()
                .insert("unknown_motion_field".into(), json!(true));
        } else {
            args["untrusted_principal"] = json!("fixture-author");
        }
        assert!(mcp.call("poet_material_motion_edit", args).is_err());
        assert_eq!(
            studio_store::get_artifact_stamped(&scope.source_id).unwrap(),
            retained
        );
    }
    let read = mcp.call("poet_source_read", json!({})).unwrap();
    let mut invalid = draft();
    invalid.keys[0].roughness = 1.5;
    let invalid_error = mcp.call("poet_material_motion_edit", json!({
        "context":read["context"],
        "edit":serde_json::to_value(MaterialMotionEdit::Set {draft:invalid,sample_ms:0}).unwrap(),
    })).unwrap_err();
    assert_eq!(invalid_error.code, "invalid_arguments");
    assert!(
        mcp.call(
            "poet_material_motion_inspect",
            json!({"context":read["context"]})
        )
        .is_ok(),
        "preflight parameter rejection does not attempt a native mutation"
    );
    let missing = serde_json::to_value(MaterialMotionEdit::Sample {
        material_id: 99,
        sample_ms: 500,
    })
    .unwrap();
    assert!(
        mcp.call(
            "poet_material_motion_edit",
            json!({"context":read["context"],"edit":missing})
        )
        .is_err(),
        "a valid-shaped request reaches the host, which refuses the missing material/track"
    );
    assert!(
        mcp.call(
            "poet_material_motion_edit",
            json!({"context":read["context"],"edit":sample})
        )
        .is_err(),
        "a refused attempted native mutation consumes context and requires a new read"
    );
    assert_eq!(
        studio_store::get_artifact_stamped(&scope.source_id).unwrap(),
        retained
    );

    // An independent reviewer proof does not revoke the existing author.
    let observer = login(&authority, "fixture-reviewer", &REVIEWER);
    assert!(
        ui.snapshot().is_ok(),
        "reviewer login preserves independent author handles"
    );
    assert!(mcp.call("poet_source_read", json!({})).is_ok());
    let mut observer_mcp = project.mcp_backend(observer.clone()).unwrap();
    let read = observer_mcp.call("poet_source_read", json!({})).unwrap();
    assert!(observer_mcp
        .call(
            "poet_material_motion_inspect",
            json!({"context":read["context"]})
        )
        .is_ok());
    assert!(observer_mcp
        .call(
            "poet_material_motion_edit",
            json!({"context":read["context"],"edit":sample})
        )
        .is_err());
    let mut observer_ui = project.connect(observer).unwrap();
    let read = observer_ui.snapshot().unwrap();
    assert!(observer_ui
        .authoring_edit(
            AuthoringEdit::MaterialMotion {
                edit: MaterialMotionEdit::Remove { material_id: 1 },
            },
            read.source
        )
        .is_err());
    assert_eq!(
        studio_store::get_artifact_stamped(&scope.source_id).unwrap(),
        retained
    );

    // Reauthenticate the author before the separate admission-revocation case;
    // its failure must be caused by revocation rather than the earlier handoff.
    drop(observer_ui);
    drop(observer_mcp);
    drop(ui);
    drop(mcp);
    let author = login(&authority, "fixture-author", &AUTHOR);
    let ui = project.connect(author.clone()).unwrap();
    let mut mcp = project.mcp_backend(author).unwrap();
    let read = mcp.call("poet_source_read", json!({})).unwrap();
    admission.sequence = 3;
    admission.subjects[0].revoked = true;
    authority.replace_admission(signed(admission)).unwrap();
    assert!(ui.snapshot().is_err());
    assert!(mcp
        .call(
            "poet_material_motion_inspect",
            json!({"context":read["context"]})
        )
        .is_err());
    assert!(mcp
        .call(
            "poet_material_motion_edit",
            json!({"context":read["context"],"edit":sample})
        )
        .is_err());
    assert_eq!(
        studio_store::get_artifact_stamped(&scope.source_id).unwrap(),
        retained,
        "all refused requests preserve source, revision and retained history"
    );
}
