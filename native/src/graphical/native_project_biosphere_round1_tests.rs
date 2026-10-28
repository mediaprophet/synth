//! Signed headless botanical source-owner integration. This diagnostic graph is
//! neither a species model nor human/native-window or GPU acceptance evidence.
use super::*;
use crate::graphical::{
    authoring::{
        botanical::{BotanicalEdit, BotanicalRecipe, Organ},
        AuthoringEdit,
    },
    jobs, native_dispatch,
    session_host::SessionEditorHost,
};
use std::sync::Mutex;

const BOTANICAL_CHILD: &str = "POET_BOTANICAL_ROUND1_CHILD_ROOT";

struct BotanicalNoPixels;
impl PreviewProvider for BotanicalNoPixels {
    // This fixture does not retain a viewport or rendered frame.
    fn retire_retained_viewport(&mut self) -> Result<(), String> { Ok(()) }
    fn validate(&self, value: &Value) -> Result<(), String> {
        crate::graphical::assembly::admit_assembly(value, PreviewLimits::default()).map(|_| ())
    }
    fn preview(&mut self, _: SourceRef, _: &Value) -> Result<PreviewFrame, String> {
        Err("headless botanical diagnostic has no GPU frame".into())
    }
}

fn botanical_config(scope: Scope, root: &Path, library: &Path) -> ProjectConfig {
    let mut cfg = config(scope, root, library);
    cfg.geometry_grants.extend([
        "ComputationalGeometry.space_colonize_3d".into(),
        "ComputationalGeometry.pipe_model_radii".into(),
        // The bounded adapter calls Core's public sweep kernel directly; the
        // grant is checked by the existing native host before typed edits.
        "ComputationalGeometry.extrude_profile".into(),
    ]);
    cfg
}

fn botanical_source() -> &'static str {
    r#"
let m={id:1,colour_space:"linear",base_color:[0.3,0.4,0.5,1.0],emissive:[0.0,0.0,0.0],metallic:0.1,roughness:0.5,specular:1.0,alpha_cutoff:0.5,opacity_mode:"Opaque",double_sided:true};
let tri={vertices:[[0.0,0.0,0.0],[1.0,0.0,0.0],[0.0,1.0,0.0]],triangles:[[0,1,2]]};
let mut poet_parts=[
  {id:"branch",semantic_id:101,geometry:tri,material:m,translation:[0.0,0.0,0.0],rotation:[0.0,0.0,0.0],scale:[1.0,1.0,1.0]},
  {id:"unrelated",semantic_id:102,geometry:tri,material:m,translation:[2.0,0.0,0.0],rotation:[0.0,0.0,0.0],scale:[1.0,1.0,1.0]}
];
({parts:poet_parts});
"#
}

fn botanical_recipe() -> BotanicalRecipe {
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

fn botanical_block_on<F: std::future::Future>(future: F) -> F::Output {
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

fn assert_botanical(host: &dyn EditorHost, radius: f64, lateral_x: f64) {
    let source = host.snapshot().unwrap().source;
    let view = host.authoring_view(source).unwrap();
    let recipe = view.botanical.expect("accepted botanical source binding");
    assert_eq!(recipe.part_id, "branch");
    assert_eq!(
        recipe
            .organs
            .iter()
            .map(|o| o.key.as_str())
            .collect::<Vec<_>>(),
        ["root", "axis", "lateral", "tip"]
    );
    assert_eq!(recipe.organs[2].parent.as_deref(), Some("axis"));
    assert_eq!(recipe.tip_radius_m, radius);
    assert_eq!(recipe.organs[2].position_m[0], lateral_x);
    assert_eq!(
        view.parts
            .iter()
            .find(|p| p.id == "unrelated")
            .unwrap()
            .vertices,
        3
    );
    assert!(
        view.parts
            .iter()
            .find(|p| p.id == "branch")
            .unwrap()
            .vertices
            > 3
    );
}

fn botanical_child_reopen(root: &Path) {
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
        "fresh child must recover from retained HMC/journal"
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
        botanical_config(scope.clone(), &root.join("hmc"), &root.join("library")),
        authority,
        author.clone(),
        Some("saved.hmc"),
        Box::new(BotanicalNoPixels),
    )
    .unwrap();
    let mut ui = project.connect(author.clone()).unwrap();
    let held = ui.snapshot().unwrap();
    assert!(held.reconciliation_required);
    assert_eq!(
        json!(ui.source_cells(held.source.clone()).unwrap()),
        witness["source_cells"]
    );
    ui.rebuild_execution(held.source).unwrap();
    assert_botanical(&ui, 0.04, 0.7);
    let mut mcp = project.mcp_backend(author).unwrap();
    let read = mcp.call("poet_source_read", json!({})).unwrap();
    let inspected = mcp
        .call("poet_botanical_inspect", json!({"context":read["context"]}))
        .unwrap();
    assert_eq!(inspected["botanical"]["tip_radius_m"], 0.04);
    let saved = checkpoint::load(&root.join("hmc/saved.hmc"), CheckpointLimits::default()).unwrap();
    assert_eq!(
        saved.artifact(),
        &studio_store::get_artifact_stamped(&scope.source_id)
            .unwrap()
            .0
    );
    std::fs::write(
        root.join("child-recovered.json"),
        b"source, recipe and HMC reopened",
    )
    .unwrap();
}

#[test]
fn signed_botanical_native_intent_mcp_refusals_history_and_hmc_reopen() {
    if let Some(root) = std::env::var_os(BOTANICAL_CHILD) {
        botanical_child_reopen(Path::new(&root));
        return;
    }
    let temp = tempfile::tempdir().unwrap();
    { // Release every source/authority publication before the fresh child opens the journal.
    for name in ["authority", "hmc", "library"] {
        std::fs::create_dir(temp.path().join(name)).unwrap();
    }
    let (scope, _, authority) = fixture(&temp.path().join("authority"));
    let author = login(&authority, "fixture-author", &AUTHOR);
    let reviewer = login(&authority, "fixture-reviewer", &REVIEWER);
    let project = Arc::new(
        NativeProject::open(
            botanical_config(
                scope.clone(),
                &temp.path().join("hmc"),
                &temp.path().join("library"),
            ),
            authority.clone(),
            author.clone(),
            None,
            Box::new(BotanicalNoPixels),
        )
        .unwrap(),
    );
    let mut ui = project.connect(author.clone()).unwrap();
    let reviewer_ui = project.connect(reviewer.clone()).unwrap();
    assert!(
        ui.snapshot().is_ok() && reviewer_ui.snapshot().is_ok(),
        "independently signed author/reviewer sessions coexist"
    );
    let initial = ui.snapshot().unwrap();
    ui.submit(botanical_source(), initial.source).unwrap();
    let sessions = Arc::new(Mutex::new(Some(author.clone())));
    let operator = SessionEditorHost::new(project.clone(), sessions);
    let current = operator.snapshot().unwrap();
    let (worker, handle) = jobs::Worker::start(Box::new(operator)).unwrap();
    let reply = botanical_block_on(native_dispatch::run(
        handle.clone(),
        jobs::Action::AuthoringEdit {
            edit: AuthoringEdit::Botanical {
                edit: BotanicalEdit::SetRecipe {
                    recipe: botanical_recipe(),
                },
            },
            expected: current.source.clone(),
        },
        jobs::Context::from_snapshot(&current),
    ))
    .unwrap();
    assert!(
        reply.operation_succeeded,
        "native botanical adoption: {}",
        reply.message
    );
    assert!(reply.image_uri.is_none());
    assert_botanical(&ui, 0.02, 0.5);
    assert!(
        reviewer_ui.snapshot().is_ok(),
        "reviewer session survives author edit"
    );
    let mut mcp = project.mcp_backend(author.clone()).unwrap();
    let mut reviewer_mcp = project.mcp_backend(reviewer).unwrap();
    let stale = reviewer_mcp.call("poet_source_read", json!({})).unwrap();
    let read = mcp.call("poet_source_read", json!({})).unwrap();
    let radius = BotanicalEdit::SetTipRadius {
        part_id: "branch".into(),
        tip_radius_m: 0.04,
    };
    let changed = mcp
        .call(
            "poet_botanical_edit",
            json!({"context":read["context"],"edit":radius}),
        )
        .unwrap();
    assert_eq!(changed["read_required"], true);
    assert_botanical(&ui, 0.04, 0.5);
    assert!(reviewer_mcp
        .call(
            "poet_botanical_edit",
            json!({"context":stale["context"],
        "edit":{"operation":"set_tip_radius","part_id":"branch","tip_radius_m":0.06}})
        )
        .is_err());
    let read = mcp.call("poet_source_read", json!({})).unwrap();
    let move_organ = BotanicalEdit::SetOrganPosition {
        part_id: "branch".into(),
        organ_key: "lateral".into(),
        position_m: [0.7, 0.0, 1.5],
    };
    mcp.call(
        "poet_botanical_edit",
        json!({"context":read["context"],"edit":move_organ}),
    )
    .unwrap();
    assert_botanical(&ui, 0.04, 0.7);
    let accepted = studio_store::get_artifact_stamped(&scope.source_id).unwrap();
    let read = mcp.call("poet_source_read", json!({})).unwrap();
    assert!(mcp.call("poet_botanical_edit",json!({"context":read["context"],
        "edit":{"operation":"set_tip_radius","part_id":"branch","tip_radius_m":0.08,"unknown":true}})).is_err());
    assert!(mcp.call("poet_botanical_edit",json!({"context":read["context"],
        "edit":{"operation":"set_organ_position","part_id":"branch","organ_key":"unknown","position_m":[1.0,0.0,1.0]}})).is_err());
    assert_eq!(
        studio_store::get_artifact_stamped(&scope.source_id).unwrap(),
        accepted
    );
    let before = ui.snapshot().unwrap();
    let undone = ui.undo(before.source).unwrap();
    assert!(undone.reconciliation_required);
    assert!(ui.authoring_view(undone.source.clone()).is_err());
    ui.rebuild_execution(undone.source).unwrap();
    assert_botanical(&ui, 0.04, 0.5);
    let redone = ui.redo(ui.snapshot().unwrap().source).unwrap();
    assert!(redone.reconciliation_required);
    ui.rebuild_execution(redone.source).unwrap();
    assert_botanical(&ui, 0.04, 0.7);
    let source = ui.snapshot().unwrap().source;
    let retained_before_drift = studio_store::get_artifact_stamped(&scope.source_id).unwrap();
    let drift = r#"poet_parts = poet_parts.map(|p| if p.id == "branch" {{id:p.id,semantic_id:p.semantic_id,geometry:{vertices:[[0.0,0.0,0.0],[1.0,0.0,0.0],[0.0,1.0,0.0]],triangles:[[0,1,2]]},material:p.material,translation:p.translation,rotation:p.rotation,scale:p.scale}} else {p}); ({parts:poet_parts});"#;
    assert!(
        ui.submit(drift, source.clone()).is_err(),
        "unlinked geometry mutation must be held"
    );
    assert_eq!(
        studio_store::get_artifact_stamped(&scope.source_id).unwrap(),
        retained_before_drift
    );
    ui.rebuild_execution(source).unwrap();
    assert_botanical(&ui, 0.04, 0.7);
    let current = ui.snapshot().unwrap();
    let cells = ui.source_cells(current.source.clone()).unwrap();
    let saved = ui
        .save_checkpoint("saved.hmc", ExpectedFile::Absent, current.source)
        .unwrap();
    let loaded = checkpoint::load(&saved.path, CheckpointLimits::default()).unwrap();
    assert_eq!(
        loaded.artifact(),
        &studio_store::get_artifact_stamped(&scope.source_id)
            .unwrap()
            .0
    );
    std::fs::write(temp.path().join("witness.json"),serde_json::to_vec(&json!({
        "project_id":scope.project_id,"document_id":scope.document_id,"source_id":scope.source_id,
        "source_cells":cells,
    })).unwrap()).unwrap();
    drop(mcp);
    drop(reviewer_mcp);
    drop(reviewer_ui);
    drop(ui);
    worker.finish().unwrap();
    drop(handle);
    drop(project);
    drop(authority);
    }
    let test_name = std::thread::current()
        .name()
        .expect("test thread has exact name")
        .to_owned();
    let output = std::process::Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            test_name.as_str(),
            "--nocapture",
            "--test-threads=1",
        ])
        .env(BOTANICAL_CHILD, temp.path())
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "fresh botanical reopen failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        temp.path().join("child-recovered.json").is_file(),
        "zero-test child exit is not evidence"
    );
}

#[test]
fn botanical_native_edit_requires_actual_core_grants() {
    let temp = tempfile::tempdir().unwrap();
    for name in ["authority", "hmc", "library"] {
        std::fs::create_dir(temp.path().join(name)).unwrap();
    }
    let (scope, _, authority) = fixture(&temp.path().join("authority"));
    let author = login(&authority, "fixture-author", &AUTHOR);
    let mut denied = botanical_config(
        scope.clone(),
        &temp.path().join("hmc"),
        &temp.path().join("library"),
    );
    denied
        .geometry_grants
        .retain(|g| g != "ComputationalGeometry.pipe_model_radii");
    let project = NativeProject::open(
        denied,
        authority,
        author.clone(),
        None,
        Box::new(BotanicalNoPixels),
    )
    .unwrap();
    let mut ui = project.connect(author).unwrap();
    let first = ui.snapshot().unwrap();
    ui.submit(botanical_source(), first.source).unwrap();
    let accepted = studio_store::get_artifact_stamped(&scope.source_id).unwrap();
    let source = ui.snapshot().unwrap().source;
    assert!(ui
        .authoring_edit(
            AuthoringEdit::Botanical {
                edit: BotanicalEdit::SetRecipe {
                    recipe: botanical_recipe()
                }
            },
            source
        )
        .is_err());
    assert_eq!(
        studio_store::get_artifact_stamped(&scope.source_id).unwrap(),
        accepted
    );
}

include!("native_project_botanical_gpu_tests.inc.rs");

include!("native_project_botanical_review_tests.inc.rs");

include!("native_project_botanical_chain_v2_tests.inc.rs");
include!("native_project_botanical_uv_tests.inc.rs");
