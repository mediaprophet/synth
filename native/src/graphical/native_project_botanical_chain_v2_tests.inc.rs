// Synthetic signed integration: explicit v1-to-v2 upgrade, HMC fresh reopen.
// Requires inclusion in native_project_biosphere_round1_tests.rs.
const CHAIN_V2_CHILD: &str = "POET_BOTANICAL_CHAIN_V2_CHILD";
#[test]
fn signed_botanical_chain_v2_upgrade_and_fresh_hmc_reopen() {
    if let Some(root) = std::env::var_os(CHAIN_V2_CHILD) {
        let root = Path::new(&root);
        let scope = Scope {
            project_id: std::env::var("POET_CHAIN_PROJECT").unwrap(),
            document_id: std::env::var("POET_CHAIN_DOCUMENT").unwrap(),
            source_id: std::env::var("POET_CHAIN_SOURCE").unwrap(),
            domain: "Geometry".into(),
        };
        assert!(studio_store::get_artifact_stamped(&scope.source_id).is_none());
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
            Some("chain.hmc"),
            Box::new(BotanicalNoPixels),
        )
        .unwrap();
        let mut host = project.connect(author).unwrap();
        let held = host.snapshot().unwrap();
        assert!(held.reconciliation_required);
        let source = host.rebuild_execution(held.source).unwrap().snapshot.source;
        let view = host.authoring_view(source).unwrap();
        let recipe = view.botanical.unwrap();
        assert_eq!(recipe.version, 2);
        assert_eq!(recipe.organs[3].parent.as_deref(), Some("lateral"));
        assert_eq!(
            view.parts
                .iter()
                .find(|p| p.id == "branch")
                .unwrap()
                .vertices,
            4 * recipe.ring_slices + 2
        );
        let saved =
            checkpoint::load(&root.join("hmc/chain.hmc"), CheckpointLimits::default()).unwrap();
        assert_eq!(
            saved.artifact(),
            &studio_store::get_artifact_stamped(&scope.source_id)
                .unwrap()
                .0
        );
        std::fs::write(root.join("chain-child-ok"), b"verified").unwrap();
        return;
    }
    let temp = tempfile::tempdir().unwrap();
    let scope = {
        for name in ["authority", "hmc", "library"] {
            std::fs::create_dir(temp.path().join(name)).unwrap();
        }
        let (scope, _, authority) = fixture(&temp.path().join("authority"));
        let author = login(&authority, "fixture-author", &AUTHOR);
        let project = NativeProject::open(
            botanical_config(
                scope.clone(),
                &temp.path().join("hmc"),
                &temp.path().join("library"),
            ),
            authority,
            author.clone(),
            None,
            Box::new(BotanicalNoPixels),
        )
        .unwrap();
        let mut host = project.connect(author).unwrap();
        let initial = host.snapshot().unwrap().source;
        host.submit(botanical_source(), initial).unwrap();
        let mut recipe = botanical_recipe();
        recipe.organs[3].parent = Some("lateral".into());
        recipe.organs[3].position_m = [1.0, 0.0, 2.0];
        let source = host.snapshot().unwrap().source;
        host.authoring_edit(
            AuthoringEdit::Botanical {
                edit: BotanicalEdit::SetRecipe {
                    recipe: recipe.clone(),
                },
            },
            source,
        )
        .unwrap();
        let source = host.snapshot().unwrap().source;
        let before = host.authoring_view(source.clone()).unwrap();
        assert_eq!(before.botanical.unwrap().version, 1);
        let legacy_vertices = before
            .parts
            .iter()
            .find(|p| p.id == "branch")
            .unwrap()
            .vertices;
        recipe.version = 2;
        host.authoring_edit(
            AuthoringEdit::Botanical {
                edit: BotanicalEdit::SetRecipe {
                    recipe: recipe.clone(),
                },
            },
            source,
        )
        .unwrap();
        let current = host.snapshot().unwrap();
        let view = host.authoring_view(current.source.clone()).unwrap();
        assert_eq!(view.botanical.unwrap().version, 2);
        assert_eq!(
            view.parts
                .iter()
                .find(|p| p.id == "branch")
                .unwrap()
                .vertices,
            4 * recipe.ring_slices + 2
        );
        assert!(legacy_vertices > 4 * recipe.ring_slices + 2);
        host.save_checkpoint("chain.hmc", ExpectedFile::Absent, current.source)
            .unwrap();
        scope
    };
    let name = std::thread::current().name().unwrap().to_owned();
    let mut child = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", name.as_str(), "--nocapture", "--test-threads=1"])
        .env(CHAIN_V2_CHILD, temp.path())
        .env("POET_CHAIN_PROJECT", &scope.project_id)
        .env("POET_CHAIN_DOCUMENT", &scope.document_id)
        .env("POET_CHAIN_SOURCE", &scope.source_id)
        .spawn()
        .unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
    loop {
        if let Some(status) = child.try_wait().unwrap() {
            assert!(status.success(), "fresh chain child failed: {status}");
            break;
        }
        if std::time::Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            panic!("fresh chain child exceeded 30 second deadline");
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    assert!(temp.path().join("chain-child-ok").is_file());
}
