// Candidate: include in native_project_biosphere_round1_tests.rs, where the
// signed botanical fixture helpers are in scope. Explicit GPU case only.
// Synthetic actors and geometry; not human or species acceptance.
const BOTANICAL_GPU_CHILD: &str = "POET_BOTANICAL_GPU_CHILD_ROOT";

fn botanical_gpu_provider() -> Box<dyn PreviewProvider> {
    use crate::graphical::{controller::MeshPreviewProvider, preview::PreviewCamera,
        viewport_provider::InteractivePreviewProvider};
    Box::new(InteractivePreviewProvider::new(MeshPreviewProvider {
        material: MaterialRecord::legacy_default(),
        camera: PreviewCamera { yaw: 0.0, pitch: 0.0, distance: 4.0, target: [0.0, 0.0, 0.8] },
        limits: PreviewLimits { side: 64, vertices: 8192, indices: 49152 },
    }).expect("native GPU provider required"))
}

fn botanical_gpu_frame(host: &mut dyn EditorHost) -> PreviewFrame {
    use crate::graphical::viewport::ViewportCommand;
    use std::sync::atomic::AtomicBool;
    let snapshot = host.snapshot().unwrap();
    assert!(!snapshot.reconciliation_required && snapshot.publication_is_current());
    let out = host.viewport_action(ViewportCommand::Redraw, snapshot.source.clone(), &AtomicBool::new(false)).unwrap();
    assert_eq!(out.frame.source, snapshot.source);
    assert_eq!(out.state.source, snapshot.source);
    assert_eq!(out.frame.session_revision, Some(snapshot.session_revision));
    assert!(out.frame.execution_epoch.as_ref().is_some_and(|e| e == &snapshot.execution_epoch && e.is_current()));
    assert_eq!(out.frame.side, 64);
    assert_eq!(out.frame.rgba8.len(), 64 * 64 * 4);
    // Compare with the clear corner, as in the existing retained viewport fixture.
    // The scene also contains an unrelated part; this is not a botanical mask.
    let background = &out.frame.rgba8[0..3];
    assert!(out.frame.rgba8.chunks_exact(4).filter(|p| &p[0..3] != background).count() >= 4,
        "GPU frame has no discernible foreground against its clear corner");
    out.frame
}

#[test]
#[ignore = "requires explicit retained native GPU qualification"]
fn signed_botanical_gpu_edit_checkpoint_fresh_reopen() {
    if let Some(root) = std::env::var_os(BOTANICAL_GPU_CHILD) {
        let root = Path::new(&root);
        let scope = Scope { project_id: std::env::var("POET_BOTANICAL_GPU_PROJECT").unwrap(),
            document_id: std::env::var("POET_BOTANICAL_GPU_DOCUMENT").unwrap(),
            source_id: std::env::var("POET_BOTANICAL_GPU_SOURCE").unwrap(), domain: "Geometry".into() };
        assert!(studio_store::get_artifact_stamped(&scope.source_id).is_none());
        let authority = Arc::new(NativeAuthority::open_retained_current(&root.join("authority"), TrustedProject {
            project_id: scope.project_id.clone(), issuer_identifier: "fixture-issuer".into(),
            public_key: ed25519::public_from_seed(&AUTHORITY),
        }, 1024 * 1024, 256).unwrap());
        let author = login(&authority, "fixture-author", &AUTHOR);
        let project = NativeProject::open(botanical_config(scope.clone(), &root.join("hmc"), &root.join("library")),
            authority, author.clone(), Some("saved.hmc"), botanical_gpu_provider()).unwrap();
        let mut host = project.connect(author).unwrap();
        let held = host.snapshot().unwrap();
        assert!(held.reconciliation_required);
        host.rebuild_execution(held.source).unwrap();
        let saved = checkpoint::load(&root.join("hmc/saved.hmc"), CheckpointLimits::default()).unwrap();
        assert_eq!(saved.artifact(), &studio_store::get_artifact_stamped(&scope.source_id).unwrap().0);
        assert_botanical(&host, 0.12, 0.7);
        let frame = botanical_gpu_frame(&mut host);
        assert_eq!(frame.rgba8, std::fs::read(root.join("gpu-parent.rgba")).unwrap(), "fresh-process preview differs from saved source");
        std::fs::write(root.join("gpu-child-ok"), b"reopened source and preview").unwrap();
        return;
    }
    let temp = tempfile::tempdir().unwrap();
    let scope = {
        for name in ["authority", "hmc", "library"] { std::fs::create_dir(temp.path().join(name)).unwrap(); }
        let (scope, _, authority) = fixture(&temp.path().join("authority"));
        let author = login(&authority, "fixture-author", &AUTHOR);
        let project = NativeProject::open(botanical_config(scope.clone(), &temp.path().join("hmc"), &temp.path().join("library")),
            authority.clone(), author.clone(), None, botanical_gpu_provider()).unwrap();
        let mut host = project.connect(author).unwrap();
        let initial = host.snapshot().unwrap();
        host.submit(botanical_source(), initial.source).unwrap();
        let source = host.snapshot().unwrap().source;
        host.authoring_edit(AuthoringEdit::Botanical { edit: BotanicalEdit::SetRecipe { recipe: botanical_recipe() } }, source).unwrap();
        let before = botanical_gpu_frame(&mut host);
        let source = host.snapshot().unwrap().source;
        host.authoring_edit(AuthoringEdit::Botanical { edit: BotanicalEdit::SetTipRadius { part_id: "branch".into(), tip_radius_m: 0.12 } }, source).unwrap();
        let source = host.snapshot().unwrap().source;
        host.authoring_edit(AuthoringEdit::Botanical { edit: BotanicalEdit::SetOrganPosition { part_id: "branch".into(), organ_key: "lateral".into(), position_m: [0.7, 0.0, 1.5] } }, source).unwrap();
        assert_botanical(&host, 0.12, 0.7);
        let after = botanical_gpu_frame(&mut host);
        let changed = before.rgba8.chunks_exact(4).zip(after.rgba8.chunks_exact(4)).filter(|(a,b)| a != b).count();
        assert!(changed >= 4, "botanical edit produced no discernible GPU change: {changed}");
        let current = host.snapshot().unwrap();
        let saved = host.save_checkpoint("saved.hmc", ExpectedFile::Absent, current.source).unwrap();
        let loaded = checkpoint::load(&saved.path, CheckpointLimits::default()).unwrap();
        assert_eq!(loaded.artifact(), &studio_store::get_artifact_stamped(&scope.source_id).unwrap().0);
        std::fs::write(temp.path().join("gpu-parent.rgba"), &after.rgba8).unwrap();
        scope
    };
    let test_name = std::thread::current().name().unwrap().to_owned();
    let output = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", test_name.as_str(), "--ignored", "--nocapture", "--test-threads=1"])
        .env(BOTANICAL_GPU_CHILD, temp.path())
        .env("POET_BOTANICAL_GPU_PROJECT", &scope.project_id)
        .env("POET_BOTANICAL_GPU_DOCUMENT", &scope.document_id)
        .env("POET_BOTANICAL_GPU_SOURCE", &scope.source_id).output().unwrap();
    assert!(output.status.success(), "fresh botanical GPU child failed: {}\nstdout:\n{}\nstderr:\n{}", output.status, String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr));
    assert!(temp.path().join("gpu-child-ok").is_file(), "zero-test child is not evidence");
}
