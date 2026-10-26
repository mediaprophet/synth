//! Explicit retained native GPU calibration for the signed texture-effect workflow.
//! Synthetic inputs and software actors do not establish human or product acceptance.
use super::*;
use crate::graphical::{
    controller::MeshPreviewProvider,
    preview::{PreviewCamera, PreviewFrame, PreviewLimits},
    viewport::ViewportCommand,
    viewport_provider::InteractivePreviewProvider,
};
use std::{io::Write, sync::atomic::AtomicBool};

const GPU_CHILD: &str = "POET_TEXTURE_EFFECT_GPU_CASE_ROOT";

fn gpu_provider() -> Box<dyn PreviewProvider> {
    Box::new(
        InteractivePreviewProvider::new(MeshPreviewProvider {
            material: MaterialRecord::legacy_default(),
            camera: PreviewCamera {
                yaw: 0.0,
                pitch: 0.0,
                distance: 3.2,
                target: [0.0; 3],
            },
            limits: PreviewLimits {
                side: 64,
                vertices: 64,
                indices: 192,
            },
        })
        .expect("retained native GPU provider admission"),
    )
}

fn gpu_frame(host: &mut dyn EditorHost) -> PreviewFrame {
    let snapshot = host.snapshot().unwrap();
    assert!(!snapshot.reconciliation_required);
    assert!(snapshot.publication_is_current());
    let output = host
        .viewport_action(
            ViewportCommand::Redraw,
            snapshot.source.clone(),
            &AtomicBool::new(false),
        )
        .expect("actual retained native GPU frame; missing GPU fails the qualification");
    assert_eq!(output.frame.source, snapshot.source);
    assert_eq!(output.state.source, snapshot.source);
    assert_eq!(
        output.frame.session_revision,
        Some(snapshot.session_revision)
    );
    assert!(output
        .frame
        .execution_epoch
        .as_ref()
        .is_some_and(|e| e == &snapshot.execution_epoch && e.is_current()));
    assert!(output
        .state
        .execution_epoch
        .as_ref()
        .is_some_and(|e| e == &snapshot.execution_epoch && e.is_current()));
    assert_eq!(output.frame.side, 64);
    assert_eq!(output.frame.rgba8.len(), 64 * 64 * 4);
    assert!(output
        .frame
        .rgba8
        .chunks_exact(4)
        .any(|p| p[0..3] != [0, 0, 0]));
    output.frame
}

fn changed(a: &PreviewFrame, b: &PreviewFrame) -> usize {
    a.rgba8
        .chunks_exact(4)
        .zip(b.rgba8.chunks_exact(4))
        .filter(|(a, b)| a != b)
        .count()
}

fn assert_change(a: &PreviewFrame, b: &PreviewFrame, label: &str) {
    let pixels = changed(a, b);
    eprintln!("texture_effect_gpu {label} changed_pixels={pixels}");
    assert!(
        pixels >= 4,
        "{label}: actual native GPU pixels did not change at the material surface"
    );
}

fn gpu_apply(host: &mut dyn EditorHost, recipe: &DetailRecipe) -> ([u8; 32], PreviewFrame) {
    let source = host.snapshot().unwrap().source;
    host.authoring_edit(
        AuthoringEdit::TextureDetail {
            material_id: 1,
            detail: Some(recipe.clone()),
        },
        source,
    )
    .unwrap();
    texture_effect_assert_recipe(host, recipe);
    let digest = texture_effect_digest(host);
    (digest, gpu_frame(host))
}

fn gpu_evidence(frame: &PreviewFrame, name: &str) -> Option<String> {
    let dir = std::env::var_os("POET_TEXTURE_EFFECT_GPU_EVIDENCE_DIR")?;
    let dir = std::path::PathBuf::from(dir);
    assert!(
        dir.is_dir(),
        "caller-selected GPU evidence directory must exist"
    );
    let mut png = vec![0; frame.rgba8.len() + frame.side as usize + 4096];
    let n = encode_surface_png_into(frame.side, &frame.rgba8, &mut png).unwrap();
    let digest: [u8; 32] = Sha256::digest(&png[..n]).into();
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(dir.join(name))
        .expect("create fresh GPU evidence PNG");
    file.write_all(&png[..n]).unwrap();
    Some(digest_hex(&digest))
}

fn gpu_child_reopen(root: &Path) {
    let witness: serde_json::Value =
        serde_json::from_slice(&std::fs::read(root.join("gpu-witness.json")).unwrap()).unwrap();
    let scope = Scope {
        project_id: witness["project_id"].as_str().unwrap().into(),
        document_id: witness["document_id"].as_str().unwrap().into(),
        source_id: witness["source_id"].as_str().unwrap().into(),
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
        config(scope.clone(), &root.join("hmc"), &root.join("library")),
        authority,
        author.clone(),
        Some("saved-gpu.hmc"),
        gpu_provider(),
    )
    .unwrap();
    let mut ui = project.connect(author).unwrap();
    let held = ui.snapshot().unwrap();
    assert!(held.reconciliation_required);
    assert_eq!(
        json!(ui.source_cells(held.source.clone()).unwrap()),
        witness["source_cells"]
    );
    let package = std::fs::read(root.join("hmc/saved-gpu.hmc")).unwrap();
    let loaded =
        checkpoint::load(&root.join("hmc/saved-gpu.hmc"), CheckpointLimits::default()).unwrap();
    assert_eq!(
        loaded.artifact(),
        &studio_store::get_artifact_stamped(&scope.source_id)
            .unwrap()
            .0
    );
    assert!(!loaded.artifact().history.is_empty());
    let base: [u8; 32] = serde_json::from_value(witness["base"].clone()).unwrap();
    let final_sha: [u8; 32] = serde_json::from_value(witness["final_sha"].clone()).unwrap();
    let recipe: DetailRecipe = serde_json::from_value(witness["final_recipe"].clone()).unwrap();
    for digest in [base, final_sha] {
        assert!(loaded
            .resources()
            .iter()
            .any(|r| r.key == texture_hmc_key(&digest)));
    }
    let reader = qualia_core_db::bundle::BundleReader::parse(&package).unwrap();
    assert_eq!(
        reader.get(&texture_hmc_key(&base)),
        Some(texture_effect_png().as_slice())
    );
    assert_eq!(
        reader.entry(&texture_hmc_key(&base)).unwrap().meta,
        Some(TEXTURE_EFFECT_META.to_vec())
    );
    ui.bind_surface_package(
        SurfacePackageRequest {
            filename: "saved-gpu.hmc".into(),
            digest: Sha256::digest(&package).into(),
        },
        held.source.clone(),
    )
    .unwrap();
    ui.rebuild_execution(held.source).unwrap();
    texture_effect_assert_recipe(&ui, &recipe);
    assert_eq!(texture_effect_digest(&ui), final_sha);
    let frame = gpu_frame(&mut ui);
    assert_eq!(
        json!(frame.rgba8),
        witness["final_gpu_rgba8"],
        "fresh-process retained GPU pixels changed"
    );
    let png_sha = gpu_evidence(&frame, "child-reopened-gpu.png");
    let source = ui.snapshot().unwrap().source;
    let saved = ui
        .save_checkpoint("child-resaved-gpu.hmc", ExpectedFile::Absent, source)
        .unwrap();
    let reloaded = checkpoint::load(&saved.path, CheckpointLimits::default()).unwrap();
    for digest in [base, final_sha] {
        let key = texture_hmc_key(&digest);
        assert_eq!(
            loaded.resources().iter().find(|r| r.key == key),
            reloaded.resources().iter().find(|r| r.key == key)
        );
    }
    std::fs::write(
        root.join("gpu-child-recovered.json"),
        serde_json::to_vec(&json!({
            "source_id":scope.source_id,"source_history_resources_restored":true,
            "gpu_frame_sha256":digest_hex(&Sha256::digest(&frame.rgba8).into()),
            "gpu_png_sha256":png_sha,
        }))
        .unwrap(),
    )
    .unwrap();
    eprintln!("texture_effect_gpu_child_recovered");
}

#[test]
#[ignore = "explicit serial signed native retained GPU texture-effect readback"]
fn texture_effect_gpu_native_dispatch_mcp_history_hmc_fresh_process() {
    if let Some(root) = std::env::var_os(GPU_CHILD) {
        gpu_child_reopen(Path::new(&root));
        return;
    }
    let temp = tempfile::tempdir().unwrap();
    for name in ["authority", "hmc", "library"] {
        std::fs::create_dir(temp.path().join(name)).unwrap();
    }
    // Keep every admission/session/authority guard, snapshot and worker inside
    // this scope. The fresh child must acquire the journal only after all of
    // those owners have dropped; the returned JSON contains plain witnesses.
    let parent_evidence = {
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
                gpu_provider(),
            )
            .unwrap(),
        );
        let mut operator =
            SessionEditorHost::new(project.clone(), Arc::new(Mutex::new(Some(author.clone()))));
        let png = texture_effect_png();
        let base: [u8; 32] = Sha256::digest(&png).into();
        texture_effect_seed(&mut operator, base, png);
        let mut ui = project.connect(author.clone()).unwrap();
        let baseline_sha = texture_effect_digest(&ui);
        let baseline = gpu_frame(&mut ui);
        let baseline_png_sha = gpu_evidence(&baseline, "parent-baseline-gpu.png");

        let gray = texture_effect_recipe(base, vec![texture_effect_step(TextureEffect::Grayscale)]);
        let current = operator.snapshot().unwrap();
        let (worker, handle) = jobs::Worker::start(Box::new(operator)).unwrap();
        let reply = texture_effect_block_on(native_dispatch::run(
            handle.clone(),
            jobs::Action::AuthoringEdit {
                edit: AuthoringEdit::TextureDetail {
                    material_id: 1,
                    detail: Some(gray.clone()),
                },
                expected: current.source.clone(),
            },
            jobs::Context::from_snapshot(&current),
        ))
        .unwrap();
        assert!(
            reply.operation_succeeded,
            "native TextureDetail intent: {}",
            reply.message
        );
        texture_effect_assert_recipe(&ui, &gray);
        let gray_sha = texture_effect_digest(&ui);
        assert_ne!(gray_sha, baseline_sha);
        let gray_frame = gpu_frame(&mut ui);
        assert_change(&baseline, &gray_frame, "grayscale");
        let gray_png_sha = gpu_evidence(&gray_frame, "parent-grayscale-gpu.png");

        let mut mcp = project.mcp_backend(author.clone()).unwrap();
        let read = mcp.call("poet_source_read", json!({})).unwrap();
        let inspect = mcp
            .call(
                "poet_texture_effects_inspect",
                json!({"context":read["context"],"part_id":"root"}),
            )
            .unwrap();
        assert_eq!(inspect["detail"], serde_json::to_value(&gray).unwrap());
        assert_eq!(inspect["output_sha256"], digest_hex(&gray_sha));
        let invert = vec![texture_effect_step(TextureEffect::Invert)];
        let edited = mcp
            .call(
                "poet_texture_effects_edit",
                json!({"context":read["context"],"material_id":1,"effects":invert}),
            )
            .unwrap();
        assert_eq!(edited["read_required"], true);
        let invert_recipe = texture_effect_recipe(base, invert);
        texture_effect_assert_recipe(&ui, &invert_recipe);
        let invert_sha = texture_effect_digest(&ui);
        let invert_frame = gpu_frame(&mut ui);
        assert_change(&baseline, &invert_frame, "invert");
        let invert_png_sha = gpu_evidence(&invert_frame, "parent-invert-gpu.png");

        let sepia = texture_effect_recipe(
            base,
            vec![texture_effect_step(TextureEffect::Sepia { intensity: 1.0 })],
        );
        let (sepia_sha, sepia_frame) = gpu_apply(&mut ui, &sepia);
        assert_change(&baseline, &sepia_frame, "sepia");
        let sepia_png_sha = gpu_evidence(&sepia_frame, "parent-sepia-gpu.png");
        let order_a = texture_effect_recipe(
            base,
            vec![
                texture_effect_step(TextureEffect::Sepia { intensity: 1.0 }),
                texture_effect_step(TextureEffect::Invert),
            ],
        );
        let (order_a_sha, order_a_frame) = gpu_apply(&mut ui, &order_a);
        assert_change(&sepia_frame, &order_a_frame, "ordered second step");
        let order_b = texture_effect_recipe(
            base,
            vec![
                texture_effect_step(TextureEffect::Invert),
                texture_effect_step(TextureEffect::Sepia { intensity: 1.0 }),
            ],
        );
        let (order_b_sha, order_b_frame) = gpu_apply(&mut ui, &order_b);
        assert_ne!(
            order_a_sha, order_b_sha,
            "effect order must change encoded output"
        );
        assert_change(&order_a_frame, &order_b_frame, "order");
        let disabled = texture_effect_recipe(
            base,
            vec![
                texture_effect_step(TextureEffect::Sepia { intensity: 1.0 }),
                TextureEffectStep {
                    enabled: false,
                    effect: TextureEffect::Invert,
                },
            ],
        );
        let (disabled_sha, disabled_frame) = gpu_apply(&mut ui, &disabled);
        assert_eq!(disabled_sha, sepia_sha);
        assert_eq!(disabled_frame.rgba8, sepia_frame.rgba8);
        let mut removed = texture_effect_recipe(base, vec![]);
        let DetailRecipe::Paint(paint) = &mut removed else {
            unreachable!()
        };
        paint.version = 2;
        let (removed_sha, removed_frame) = gpu_apply(&mut ui, &removed);
        assert_eq!(removed_sha, baseline_sha);
        assert_eq!(removed_frame.rgba8, baseline.rgba8);
        let source = ui.snapshot().unwrap().source;
        let undone = ui.undo(source).unwrap();
        assert!(undone.reconciliation_required);
        assert!(ui
            .viewport_action(
                ViewportCommand::Redraw,
                undone.source.clone(),
                &AtomicBool::new(false)
            )
            .is_err());
        ui.rebuild_execution(undone.source).unwrap();
        texture_effect_assert_recipe(&ui, &disabled);
        assert_eq!(texture_effect_digest(&ui), disabled_sha);
        assert_eq!(gpu_frame(&mut ui).rgba8, disabled_frame.rgba8);
        let source = ui.snapshot().unwrap().source;
        let redone = ui.redo(source).unwrap();
        assert!(redone.reconciliation_required);
        ui.rebuild_execution(redone.source).unwrap();
        texture_effect_assert_recipe(&ui, &removed);
        assert_eq!(texture_effect_digest(&ui), removed_sha);
        assert_eq!(gpu_frame(&mut ui).rgba8, removed_frame.rgba8);
        let (final_sha, final_frame) = gpu_apply(&mut ui, &order_a);
        assert_eq!(final_sha, order_a_sha);
        assert_eq!(final_frame.rgba8, order_a_frame.rgba8);
        let final_png_sha = gpu_evidence(&final_frame, "parent-final-gpu.png");

        let source = ui.snapshot().unwrap().source;
        let compiled = ui
            .compile_checkpoint(
                "effects-gpu.hmc",
                "texture-effect-gpu",
                ExpectedFile::Absent,
                source.clone(),
            )
            .unwrap();
        let compiled_loaded =
            checkpoint::load(&compiled.path, CheckpointLimits::default()).unwrap();
        for digest in [base, final_sha] {
            assert!(compiled_loaded
                .resources()
                .iter()
                .any(|r| r.key == texture_hmc_key(&digest)));
        }
        ui.bind_surface_package(
            SurfacePackageRequest {
                filename: "effects-gpu.hmc".into(),
                digest: compiled.digest,
            },
            source.clone(),
        )
        .unwrap();
        ui.rebuild_execution(source).unwrap();
        assert_eq!(gpu_frame(&mut ui).rgba8, final_frame.rgba8);
        let source = ui.snapshot().unwrap().source;
        let cells = ui.source_cells(source.clone()).unwrap();
        let saved = ui
            .save_checkpoint("saved-gpu.hmc", ExpectedFile::Absent, source)
            .unwrap();
        let loaded = checkpoint::load(&saved.path, CheckpointLimits::default()).unwrap();
        assert_eq!(
            loaded.artifact(),
            &studio_store::get_artifact_stamped(&scope.source_id)
                .unwrap()
                .0
        );
        assert!(!loaded.artifact().history.is_empty());
        for digest in [base, final_sha] {
            let key = texture_hmc_key(&digest);
            assert_eq!(
                loaded.resources().iter().find(|r| r.key == key),
                compiled_loaded.resources().iter().find(|r| r.key == key)
            );
        }
        let package = std::fs::read(&saved.path).unwrap();
        std::fs::write(temp.path().join("gpu-witness.json"), serde_json::to_vec(&json!({
        "project_id":scope.project_id,"document_id":scope.document_id,"source_id":scope.source_id,
        "source_cells":cells,"base":base,"final_sha":final_sha,"final_recipe":order_a,
        "final_gpu_rgba8":final_frame.rgba8,
    })).unwrap()).unwrap();
        worker.finish().unwrap();
        drop(handle);
        json!({
            "qualification":"64px synthetic retained native GPU readback; no current-window, human, domain or production acceptance",
            "source_id":scope.source_id,"base_sha256":digest_hex(&base),
            "grayscale_output_sha256":digest_hex(&gray_sha),"invert_output_sha256":digest_hex(&invert_sha),
            "sepia_output_sha256":digest_hex(&sepia_sha),"ordered_output_sha256":digest_hex(&final_sha),
            "package_sha256":digest_hex(&Sha256::digest(&package).into()),
            "gpu_rgba8_sha256":digest_hex(&Sha256::digest(&final_frame.rgba8).into()),
            "gpu_png_sha256":{"baseline":baseline_png_sha,"grayscale":gray_png_sha,
                "invert":invert_png_sha,"sepia":sepia_png_sha,"final":final_png_sha},
            "gpu_backend":"unavailable through public Poet provider API",
        })
    };

    let test_name = std::thread::current()
        .name()
        .expect("exact native GPU test name")
        .to_string();
    let output = std::process::Command::new(std::env::current_exe().unwrap())
        .args([
            "--ignored",
            "--exact",
            &test_name,
            "--nocapture",
            "--test-threads=1",
        ])
        .env(GPU_CHILD, temp.path())
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "fresh-process retained GPU restore failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        temp.path().join("gpu-child-recovered.json").is_file(),
        "zero-test child exit is not GPU recovery evidence"
    );
    let recovered: serde_json::Value = serde_json::from_slice(
        &std::fs::read(temp.path().join("gpu-child-recovered.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(recovered["source_history_resources_restored"], true);
    if let Some(dir) = std::env::var_os("POET_TEXTURE_EFFECT_GPU_EVIDENCE_DIR") {
        let path = std::path::PathBuf::from(dir).join("gpu-witness.json");
        let mut record = parent_evidence;
        record["gpu_png_sha256"]["child"] = recovered["gpu_png_sha256"].clone();
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)
            .unwrap();
        file.write_all(&serde_json::to_vec_pretty(&record).unwrap())
            .unwrap();
    }
    eprintln!("texture_effect_gpu_parent_and_child_passed");
}

#[path = "native_project_image_completed_tests.rs"]
mod image_completed_tests;
