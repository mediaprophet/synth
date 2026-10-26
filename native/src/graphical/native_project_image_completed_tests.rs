//! Explicit synthetic, signed native GPU qualification of completed image operations.
//! Source pixels and a fresh retained HMC closure are engineering evidence only.
use super::*;
use crate::graphical::surfaces::detail::{LayerBlendMode, LayerMask, MaskChannel};

const IMAGE_CHILD: &str = "POET_IMAGE_COMPLETION_CASE_ROOT";
const MASK_META: [u8; 4] = [0xa1, 0x61, b'm', 8];

fn image_gpu_evidence(frame: &PreviewFrame, name: &str) -> Option<String> {
    let dir = std::path::PathBuf::from(std::env::var_os("POET_IMAGE_COMPLETION_EVIDENCE_DIR")?);
    assert!(
        dir.is_dir(),
        "caller-selected image evidence directory must exist"
    );
    // The retained GPU writer is the common path when the caller points both
    // evidence variables at the same fresh directory. Keep this case usable
    // when only the image-specific variable is set as well.
    if std::env::var_os("POET_TEXTURE_EFFECT_GPU_EVIDENCE_DIR").as_deref() == Some(dir.as_os_str())
    {
        return gpu_evidence(frame, name);
    }
    let mut png = vec![0; frame.rgba8.len() + frame.side as usize + 4096];
    let n = encode_surface_png_into(frame.side, &frame.rgba8, &mut png).unwrap();
    let digest: [u8; 32] = Sha256::digest(&png[..n]).into();
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(dir.join(name))
        .expect("create fresh image GPU evidence PNG");
    file.write_all(&png[..n]).unwrap();
    Some(digest_hex(&digest))
}

fn image_copy_evidence(source: &Path, dir: &Path, name: &str) -> String {
    let bytes = std::fs::read(source).unwrap();
    let digest: [u8; 32] = Sha256::digest(&bytes).into();
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(dir.join(name))
        .expect("create fresh retained image evidence");
    file.write_all(&bytes).unwrap();
    digest_hex(&digest)
}

fn image_copy_sha(source: &Path) -> String {
    digest_hex(&Sha256::digest(std::fs::read(source).unwrap()).into())
}

fn image_recipe(base: [u8; 32]) -> DetailRecipe {
    let mut recipe = texture_effect_recipe(base, vec![]);
    let DetailRecipe::Paint(p) = &mut recipe else {
        unreachable!()
    };
    p.version = 3;
    p.layers = vec![PaintLayer {
        name: "Full synthetic ink".into(),
        visible: true,
        opacity: 0.7,
        strokes: vec![PaintStroke {
            size: 16.0,
            colour: [232, 52, 93, 255],
            opacity: 1.0,
            hardness: 1.0,
            points: vec![[3.5, 3.5]],
        }],
        ..PaintLayer::default()
    }];
    recipe
}

fn paint(recipe: &mut DetailRecipe) -> &mut PaintRecipe {
    let DetailRecipe::Paint(p) = recipe else {
        unreachable!()
    };
    p
}

fn image_mask_png() -> Vec<u8> {
    let mut rgba = vec![0; 8 * 8 * 4];
    for y in 0..8 {
        for x in 0..8 {
            let i = (y * 8 + x) * 4;
            rgba[i] = if x < 4 { 255 } else { 0 };
            rgba[i + 1] = 17;
            rgba[i + 2] = 31;
            rgba[i + 3] = if y < 4 { 255 } else { 0 };
        }
    }
    let mut png = vec![0; rgba.len() + 8 + 4096];
    let n = encode_surface_png_into(8, &rgba, &mut png).unwrap();
    png.truncate(n);
    png
}

fn image_child_reopen(root: &Path) {
    // The ancestor performs signed authority reopening, exact source replay,
    // rebind/rebuild, and a real GPU byte-for-byte comparison in this process.
    gpu_child_reopen(root);
    let witness: serde_json::Value =
        serde_json::from_slice(&std::fs::read(root.join("gpu-witness.json")).unwrap()).unwrap();
    let loaded =
        checkpoint::load(&root.join("hmc/saved-gpu.hmc"), CheckpointLimits::default()).unwrap();
    let bytes = std::fs::read(root.join("hmc/saved-gpu.hmc")).unwrap();
    let reader = qualia_core_db::bundle::BundleReader::parse(&bytes).unwrap();
    let mask: [u8; 32] = serde_json::from_value(witness["mask"].clone()).unwrap();
    let key = texture_hmc_key(&mask);
    assert!(loaded.resources().iter().any(|r| r.key == key));
    assert_eq!(reader.get(&key), Some(image_mask_png().as_slice()));
    assert_eq!(reader.entry(&key).unwrap().meta, Some(MASK_META.to_vec()));
    let recovered = checkpoint::load(
        &root.join("hmc/child-resaved-gpu.hmc"),
        CheckpointLimits::default(),
    )
    .unwrap();
    assert_eq!(
        loaded.resources().iter().find(|r| r.key == key),
        recovered.resources().iter().find(|r| r.key == key)
    );
    let gpu_child: serde_json::Value =
        serde_json::from_slice(&std::fs::read(root.join("gpu-child-recovered.json")).unwrap())
            .unwrap();
    let mut marker = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(root.join("image-child-recovered.json"))
        .unwrap();
    marker
        .write_all(
            &serde_json::to_vec(&json!({
                "mask_resource_restored": true,
                "mask_sha256": digest_hex(&mask),
                "mask_metadata": MASK_META,
                "gpu_child": gpu_child,
            }))
            .unwrap(),
        )
        .unwrap();
}

#[test]
#[ignore = "explicit signed retained native GPU image completion; serial GPU required"]
fn image_completed_workflow_filters_blends_masks_hmc_and_fresh_process() {
    if let Some(root) = std::env::var_os(IMAGE_CHILD) {
        image_child_reopen(Path::new(&root));
        return;
    }
    let temp = tempfile::tempdir().unwrap();
    for name in ["authority", "hmc", "library"] {
        std::fs::create_dir(temp.path().join(name)).unwrap();
    }
    let mut parent_evidence = {
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
        let mut ui = project.connect(author.clone()).unwrap();
        let base_png = texture_effect_png();
        let base: [u8; 32] = Sha256::digest(&base_png).into();
        texture_effect_seed(&mut ui, base, base_png);
        let mut recipe = image_recipe(base);
        let (normal_sha, normal_frame) = gpu_apply(&mut ui, &recipe);
        let normal_gpu_png = image_gpu_evidence(&normal_frame, "image-normal-gpu.png");
        let mut mcp = project.mcp_backend(author).unwrap();
        let mut witnessed = serde_json::Map::new();

        for (name, effect) in [
            ("gaussian_blur", TextureEffect::GaussianBlur { sigma: 1.2 }),
            (
                "sharpen",
                TextureEffect::Sharpen {
                    sigma: 1.1,
                    amount: 2.0,
                },
            ),
            ("hue_rotate", TextureEffect::HueRotate { degrees: 120.0 }),
            (
                "emboss",
                TextureEffect::Emboss {
                    azimuth_degrees: 60.0,
                    strength: 1.0,
                },
            ),
        ] {
            paint(&mut recipe).effects = vec![texture_effect_step(effect)];
            let (sha, frame) = gpu_apply(&mut ui, &recipe);
            assert_ne!(sha, normal_sha, "{name}: source image must change");
            assert_change(&normal_frame, &frame, name);
            let read = mcp.call("poet_source_read", json!({})).unwrap();
            let edited = mcp.call("poet_texture_effects_edit", json!({
                "context":read["context"],"material_id":1,"effects":&paint(&mut recipe).effects,
            })).unwrap();
            assert_eq!(edited["read_required"], true);
            texture_effect_assert_recipe(&ui, &recipe);
            assert_eq!(texture_effect_digest(&ui), sha);
            assert_eq!(gpu_frame(&mut ui).rgba8, frame.rgba8);
            let png_sha = image_gpu_evidence(&frame, &format!("image-{name}-gpu.png"));
            witnessed.insert(
                name.into(),
                json!({"output_sha256":digest_hex(&sha),
                "gpu_sha256":digest_hex(&Sha256::digest(&frame.rgba8).into()),
                "gpu_png_sha256":png_sha,
                "gpu_png_file":format!("image-{name}-gpu.png")}),
            );
        }
        paint(&mut recipe).effects.clear();
        let (_, normal_again) = gpu_apply(&mut ui, &recipe);
        assert_eq!(normal_again.rgba8, normal_frame.rgba8);

        for (name, mode) in [
            ("multiply", LayerBlendMode::Multiply),
            ("screen", LayerBlendMode::Screen),
            ("overlay", LayerBlendMode::Overlay),
            ("soft_light", LayerBlendMode::SoftLight),
            ("hard_light", LayerBlendMode::HardLight),
            ("color_dodge", LayerBlendMode::ColorDodge),
            ("color_burn", LayerBlendMode::ColorBurn),
            ("difference", LayerBlendMode::Difference),
            ("exclusion", LayerBlendMode::Exclusion),
        ] {
            paint(&mut recipe).layers[0].blend_mode = mode;
            let (sha, frame) = gpu_apply(&mut ui, &recipe);
            assert_ne!(sha, normal_sha, "{name}: encoded image must change");
            assert_change(&normal_frame, &frame, name);
            let read = mcp.call("poet_source_read", json!({})).unwrap();
            let edited = mcp.call("poet_texture_layers_edit", json!({
                "context":read["context"],"material_id":1,"layers":&paint(&mut recipe).layers,
            })).unwrap();
            assert_eq!(edited["read_required"], true);
            texture_effect_assert_recipe(&ui, &recipe);
            assert_eq!(texture_effect_digest(&ui), sha);
            assert_eq!(gpu_frame(&mut ui).rgba8, frame.rgba8);
            let png_sha = image_gpu_evidence(&frame, &format!("image-{name}-gpu.png"));
            witnessed.insert(
                name.into(),
                json!({"output_sha256":digest_hex(&sha),
                "gpu_sha256":digest_hex(&Sha256::digest(&frame.rgba8).into()),
                "gpu_png_sha256":png_sha,
                "gpu_png_file":format!("image-{name}-gpu.png")}),
            );
        }

        paint(&mut recipe).layers[0].blend_mode = LayerBlendMode::Normal;
        paint(&mut recipe).layers[0].offset = [1, -1];
        paint(&mut recipe).layers[0].mask = Some(LayerMask::Rectangle {
            x: 2,
            y: 2,
            width: 3,
            height: 3,
            coverage: 255,
        });
        let (rectangle_sha, rectangle_frame) = gpu_apply(&mut ui, &recipe);
        assert_change(&normal_frame, &rectangle_frame, "rectangle signed offset");
        let rectangle_gpu_png =
            image_gpu_evidence(&rectangle_frame, "image-rectangle-offset-gpu.png");
        let read = mcp.call("poet_source_read", json!({})).unwrap();
        mcp.call(
            "poet_texture_layers_edit",
            json!({
                "context":read["context"],"material_id":1,"layers":&paint(&mut recipe).layers,
            }),
        )
        .unwrap();
        assert_eq!(gpu_frame(&mut ui).rgba8, rectangle_frame.rgba8);

        let mask_png = image_mask_png();
        let mask: [u8; 32] = Sha256::digest(&mask_png).into();
        let source = ui.snapshot().unwrap().source;
        let bound = ui
            .save_checkpoint("mask-source.hmc", ExpectedFile::Absent, source.clone())
            .unwrap();
        let imported = ui
            .edit_package_resources(
                "mask-source.hmc",
                ExpectedFile::Sha256(bound.digest),
                vec![ResourceEdit::Add(ResourceData {
                    key: texture_hmc_key(&mask),
                    kind: "image/png".into(),
                    bytes: mask_png,
                    meta: Some(MASK_META.to_vec()),
                })],
                source.clone(),
            )
            .unwrap();
        ui.bind_surface_package(
            SurfacePackageRequest {
                filename: "mask-source.hmc".into(),
                digest: imported.digest,
            },
            source.clone(),
        )
        .unwrap();
        ui.rebuild_execution(source).unwrap();
        paint(&mut recipe).layers[0].offset = [0, 0];
        paint(&mut recipe).layers[0].mask = Some(LayerMask::Resource {
            sha256: digest_hex(&mask),
            channel: MaskChannel::Red,
        });
        let (red_sha, red_frame) = gpu_apply(&mut ui, &recipe);
        assert_change(&normal_frame, &red_frame, "red resource mask");
        let red_gpu_png = image_gpu_evidence(&red_frame, "image-mask-red-gpu.png");
        let read = mcp.call("poet_source_read", json!({})).unwrap();
        mcp.call(
            "poet_texture_layers_edit",
            json!({
                "context":read["context"],"material_id":1,"layers":&paint(&mut recipe).layers,
            }),
        )
        .unwrap();
        assert_eq!(gpu_frame(&mut ui).rgba8, red_frame.rgba8);
        paint(&mut recipe).layers[0].mask = Some(LayerMask::Resource {
            sha256: digest_hex(&mask),
            channel: MaskChannel::Alpha,
        });
        let (alpha_sha, alpha_frame) = gpu_apply(&mut ui, &recipe);
        assert_ne!(
            red_sha, alpha_sha,
            "raw red/alpha channels must remain distinct"
        );
        assert_change(&red_frame, &alpha_frame, "alpha resource mask");
        let alpha_gpu_png = image_gpu_evidence(&alpha_frame, "image-mask-alpha-gpu.png");
        let read = mcp.call("poet_source_read", json!({})).unwrap();
        mcp.call(
            "poet_texture_layers_edit",
            json!({
                "context":read["context"],"material_id":1,"layers":&paint(&mut recipe).layers,
            }),
        )
        .unwrap();
        assert_eq!(gpu_frame(&mut ui).rgba8, alpha_frame.rgba8);
        // Keep the final two distinct source commits adjacent for the exact
        // undo/redo assertion after both native and MCP paths have been shown.
        paint(&mut recipe).layers[0].mask = Some(LayerMask::Resource {
            sha256: digest_hex(&mask),
            channel: MaskChannel::Red,
        });
        let (red_again_sha, red_again_frame) = gpu_apply(&mut ui, &recipe);
        assert_eq!(red_again_sha, red_sha);
        assert_eq!(red_again_frame.rgba8, red_frame.rgba8);
        paint(&mut recipe).layers[0].mask = Some(LayerMask::Resource {
            sha256: digest_hex(&mask),
            channel: MaskChannel::Alpha,
        });
        let (alpha_again_sha, alpha_again_frame) = gpu_apply(&mut ui, &recipe);
        assert_eq!(alpha_again_sha, alpha_sha);
        assert_eq!(alpha_again_frame.rgba8, alpha_frame.rgba8);
        let read = mcp.call("poet_source_read", json!({})).unwrap();
        let inspected = mcp
            .call(
                "poet_texture_effects_inspect",
                json!({"context":read["context"],"part_id":"root"}),
            )
            .unwrap();
        assert_eq!(inspected["detail"], serde_json::to_value(&recipe).unwrap());
        assert_eq!(inspected["base_sha256"], digest_hex(&base));
        assert_eq!(inspected["output_sha256"], digest_hex(&alpha_sha));
        assert_eq!(paint(&mut recipe).version, 3);

        let source = ui.snapshot().unwrap().source;
        let undone = ui.undo(source).unwrap();
        assert!(undone.reconciliation_required);
        ui.rebuild_execution(undone.source).unwrap();
        paint(&mut recipe).layers[0].mask = Some(LayerMask::Resource {
            sha256: digest_hex(&mask),
            channel: MaskChannel::Red,
        });
        texture_effect_assert_recipe(&ui, &recipe);
        assert_eq!(texture_effect_digest(&ui), red_sha);
        assert_eq!(gpu_frame(&mut ui).rgba8, red_frame.rgba8);
        let source = ui.snapshot().unwrap().source;
        let redone = ui.redo(source).unwrap();
        assert!(redone.reconciliation_required);
        ui.rebuild_execution(redone.source).unwrap();
        paint(&mut recipe).layers[0].mask = Some(LayerMask::Resource {
            sha256: digest_hex(&mask),
            channel: MaskChannel::Alpha,
        });
        texture_effect_assert_recipe(&ui, &recipe);
        assert_eq!(texture_effect_digest(&ui), alpha_sha);
        assert_eq!(gpu_frame(&mut ui).rgba8, alpha_frame.rgba8);
        let read = mcp.call("poet_source_read", json!({})).unwrap();
        mcp.call(
            "poet_texture_effects_edit",
            json!({"context":read["context"],"material_id":1,"effects":[]}),
        )
        .unwrap();
        texture_effect_assert_recipe(&ui, &recipe);
        assert_eq!(texture_effect_digest(&ui), alpha_sha);
        assert_eq!(gpu_frame(&mut ui).rgba8, alpha_frame.rgba8);

        let source = ui.snapshot().unwrap().source;
        let compiled = ui
            .compile_checkpoint(
                "image-completed.hmc",
                "image-completed",
                ExpectedFile::Absent,
                source.clone(),
            )
            .unwrap();
        let compiled_bytes = std::fs::read(&compiled.path).unwrap();
        let reader = qualia_core_db::bundle::BundleReader::parse(&compiled_bytes).unwrap();
        for digest in [base, mask, alpha_sha] {
            assert!(reader.get(&texture_hmc_key(&digest)).is_some());
        }
        assert_eq!(
            reader.entry(&texture_hmc_key(&mask)).unwrap().meta,
            Some(MASK_META.to_vec())
        );
        ui.bind_surface_package(
            SurfacePackageRequest {
                filename: "image-completed.hmc".into(),
                digest: compiled.digest,
            },
            source.clone(),
        )
        .unwrap();
        ui.rebuild_execution(source).unwrap();
        assert_eq!(gpu_frame(&mut ui).rgba8, alpha_frame.rgba8);
        let source = ui.snapshot().unwrap().source;
        let cells = ui.source_cells(source.clone()).unwrap();
        let saved = ui
            .save_checkpoint("saved-gpu.hmc", ExpectedFile::Absent, source)
            .unwrap();
        let saved_bytes = std::fs::read(&saved.path).unwrap();
        let reader = qualia_core_db::bundle::BundleReader::parse(&saved_bytes).unwrap();
        for digest in [base, mask, alpha_sha] {
            assert!(reader.get(&texture_hmc_key(&digest)).is_some());
        }
        assert_eq!(
            reader.entry(&texture_hmc_key(&mask)).unwrap().meta,
            Some(MASK_META.to_vec())
        );
        std::fs::write(temp.path().join("gpu-witness.json"), serde_json::to_vec(&json!({
            "project_id":scope.project_id,"document_id":scope.document_id,"source_id":scope.source_id,
            "source_cells":cells,"base":base,"mask":mask,"final_sha":alpha_sha,
            "final_recipe":recipe,"final_gpu_rgba8":alpha_frame.rgba8,
        })).unwrap()).unwrap();
        json!({"qualification":"signed native GPU + MCP + retained HMC synthetic 8x8 image case",
            "base_sha256":digest_hex(&base),"mask_sha256":digest_hex(&mask),
            "normal_output_sha256":digest_hex(&normal_sha),"rectangle_output_sha256":digest_hex(&rectangle_sha),
            "red_output_sha256":digest_hex(&red_sha),"alpha_output_sha256":digest_hex(&alpha_sha),
            "gpu_png_sha256":{
                "normal":normal_gpu_png,
                "rectangle_offset":rectangle_gpu_png,
                "mask_red":red_gpu_png,
                "mask_alpha":alpha_gpu_png,
            },"gpu_png_files":{
                "normal":"image-normal-gpu.png",
                "rectangle_offset":"image-rectangle-offset-gpu.png",
                "mask_red":"image-mask-red-gpu.png",
                "mask_alpha":"image-mask-alpha-gpu.png",
            },"cases":witnessed})
    }; // All session, project, authority and source snapshots release before child opens its journal.

    let test_name = std::thread::current().name().unwrap().to_string();
    let child = std::process::Command::new(std::env::current_exe().unwrap())
        .args([
            "--ignored",
            "--exact",
            &test_name,
            "--nocapture",
            "--test-threads=1",
        ])
        .env(IMAGE_CHILD, temp.path())
        .output()
        .unwrap();
    assert!(
        child.status.success(),
        "image child failed: {}\nstdout:\n{}\nstderr:\n{}",
        child.status,
        String::from_utf8_lossy(&child.stdout),
        String::from_utf8_lossy(&child.stderr)
    );
    assert!(
        temp.path().join("image-child-recovered.json").is_file(),
        "zero-test child is not recovery evidence"
    );
    if let Some(dir) = std::env::var_os("POET_IMAGE_COMPLETION_EVIDENCE_DIR") {
        let dir = std::path::PathBuf::from(dir);
        assert!(dir.is_dir());
        let mut files = serde_json::Map::new();
        for (source, name) in [
            ("hmc/image-completed.hmc", "image-completed.hmc"),
            ("hmc/saved-gpu.hmc", "image-saved-gpu.hmc"),
            ("hmc/child-resaved-gpu.hmc", "image-child-resaved-gpu.hmc"),
            ("gpu-witness.json", "image-source-witness.json"),
            ("gpu-child-recovered.json", "image-gpu-child-recovered.json"),
            ("image-child-recovered.json", "image-child-recovered.json"),
        ] {
            let sha = image_copy_evidence(&temp.path().join(source), &dir, name);
            files.insert(name.into(), json!(sha));
        }
        for (case, name) in [
            ("normal", "image-normal-gpu.png"),
            ("rectangle_offset", "image-rectangle-offset-gpu.png"),
            ("mask_red", "image-mask-red-gpu.png"),
            ("mask_alpha", "image-mask-alpha-gpu.png"),
        ] {
            let actual = image_copy_sha(&dir.join(name));
            assert_eq!(parent_evidence["gpu_png_sha256"][case], actual);
            files.insert(name.into(), json!(actual));
        }
        for case in [
            "gaussian_blur",
            "sharpen",
            "hue_rotate",
            "emboss",
            "multiply",
            "screen",
            "overlay",
            "soft_light",
            "hard_light",
            "color_dodge",
            "color_burn",
            "difference",
            "exclusion",
        ] {
            let name = format!("image-{case}-gpu.png");
            let actual = image_copy_sha(&dir.join(&name));
            assert_eq!(parent_evidence["cases"][case]["gpu_png_sha256"], actual);
            files.insert(name, json!(actual));
        }
        let reopened_gpu = dir.join("child-reopened-gpu.png");
        if reopened_gpu.is_file() {
            files.insert(
                "child-reopened-gpu.png".into(),
                json!(image_copy_sha(&reopened_gpu)),
            );
        }
        parent_evidence["evidence_files_sha256"] = serde_json::Value::Object(files);
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(dir.join("image-completed-witness.json"))
            .unwrap();
        file.write_all(&serde_json::to_vec_pretty(&parent_evidence).unwrap())
            .unwrap();
    }
    eprintln!("image_completed_workflow_parent_child_passed");
}

#[test]
#[ignore = "explicit signed native GPU image completion refusal qualification"]
fn image_completed_workflow_refusals_preserve_source_history() {
    let temp = tempfile::tempdir().unwrap();
    for name in ["authority", "hmc", "library"] {
        std::fs::create_dir(temp.path().join(name)).unwrap();
    }
    let (scope, mut admission, authority) = fixture(&temp.path().join("authority"));
    admission.sequence = 2;
    admission.subjects[1].membership.role = ProjectRole::Observer;
    authority.replace_admission(signed(admission)).unwrap();
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
        gpu_provider(),
    )
    .unwrap();
    let mut ui = project.connect(author.clone()).unwrap();
    let png = texture_effect_png();
    let base: [u8; 32] = Sha256::digest(&png).into();
    texture_effect_seed(&mut ui, base, png);
    let recipe = image_recipe(base);
    let (_, accepted_frame) = gpu_apply(&mut ui, &recipe);
    let accepted = studio_store::get_artifact_stamped(&scope.source_id).unwrap();
    let mut mcp = project.mcp_backend(author).unwrap();

    let mut unknown = recipe.clone();
    paint(&mut unknown).layers[0].mask = Some(LayerMask::Resource {
        sha256: "ab".repeat(32),
        channel: MaskChannel::Red,
    });
    let source = ui.snapshot().unwrap().source;
    let accepted_source = source.clone();
    assert!(
        ui.authoring_edit(
            AuthoringEdit::TextureDetail {
                material_id: 1,
                detail: Some(unknown.clone()),
            },
            source
        )
        .is_err(),
        "unknown digest has no authority to invent a mask resource"
    );
    assert_eq!(
        studio_store::get_artifact_stamped(&scope.source_id).unwrap(),
        accepted
    );
    let held = ui.snapshot().unwrap();
    assert!(
        held.reconciliation_required,
        "failed host resource evaluation holds execution"
    );
    assert_eq!(held.source, accepted_source);
    let read = mcp.call("poet_source_read", json!({})).unwrap();
    assert_eq!(read["reconciliation_required"], true);
    let rebuilt = mcp
        .call("poet_execution_rebuild", json!({"context":read["context"]}))
        .unwrap();
    assert_eq!(rebuilt["read_required"], true);
    assert_eq!(rebuilt["reconciliation_required"], false);
    assert_eq!(ui.snapshot().unwrap().source, accepted_source);
    assert_eq!(gpu_frame(&mut ui).rgba8, accepted_frame.rgba8);
    assert_eq!(
        studio_store::get_artifact_stamped(&scope.source_id).unwrap(),
        accepted
    );

    // The MCP attempt starts from rebuilt accepted execution, so its refusal
    // actually reaches mask-resource admission rather than an existing hold.
    let read = mcp.call("poet_source_read", json!({})).unwrap();
    assert_eq!(read["reconciliation_required"], false);
    let mask_refusal = mcp
        .call(
            "poet_texture_layers_edit",
            json!({
                "context":read["context"],"material_id":1,"layers":&paint(&mut unknown).layers,
            }),
        )
        .unwrap_err();
    assert_eq!(mask_refusal.code, "native_operation_failed");
    assert_eq!(
        studio_store::get_artifact_stamped(&scope.source_id).unwrap(),
        accepted
    );
    let held = ui.snapshot().unwrap();
    assert!(
        held.reconciliation_required,
        "MCP missing-mask submission holds execution"
    );
    assert_eq!(held.source, accepted_source);
    let read = mcp.call("poet_source_read", json!({})).unwrap();
    assert_eq!(read["reconciliation_required"], true);
    let rebuilt = mcp
        .call("poet_execution_rebuild", json!({"context":read["context"]}))
        .unwrap();
    assert_eq!(rebuilt["read_required"], true);
    assert_eq!(rebuilt["reconciliation_required"], false);
    assert_eq!(ui.snapshot().unwrap().source, accepted_source);
    assert_eq!(gpu_frame(&mut ui).rgba8, accepted_frame.rgba8);
    assert_eq!(
        studio_store::get_artifact_stamped(&scope.source_id).unwrap(),
        accepted
    );

    let mut invalid = recipe.clone();
    paint(&mut invalid).effects = vec![texture_effect_step(TextureEffect::GaussianBlur {
        sigma: 17.0,
    })];
    let source = ui.snapshot().unwrap().source;
    assert!(ui
        .authoring_edit(
            AuthoringEdit::TextureDetail {
                material_id: 1,
                detail: Some(invalid),
            },
            source
        )
        .is_err());
    let read = mcp.call("poet_source_read", json!({})).unwrap();
    assert!(mcp
        .call(
            "poet_texture_effects_edit",
            json!({
                "context":read["context"],"material_id":1,
                "effects":[{"enabled":true,"effect":{"kind":"gaussian_blur","sigma":17.0}}],
            })
        )
        .is_err());
    assert!(
        !ui.snapshot().unwrap().reconciliation_required,
        "parameter preflight must not hold accepted execution"
    );
    assert!(
        mcp.call(
            "poet_texture_effects_inspect",
            json!({
                "context":read["context"],"part_id":"root",
            })
        )
        .is_ok(),
        "shape/parameter refusal preserves current read context"
    );
    let mut invalid_version = recipe.clone();
    paint(&mut invalid_version).version = 2;
    paint(&mut invalid_version).layers[0].blend_mode = LayerBlendMode::Multiply;
    let source = ui.snapshot().unwrap().source;
    assert!(ui
        .authoring_edit(
            AuthoringEdit::TextureDetail {
                material_id: 1,
                detail: Some(invalid_version),
            },
            source
        )
        .is_err());
    assert_eq!(
        studio_store::get_artifact_stamped(&scope.source_id).unwrap(),
        accepted
    );
    assert_eq!(gpu_frame(&mut ui).rgba8, accepted_frame.rgba8);

    let stale = mcp.call("poet_source_read", json!({})).unwrap();
    let mut changed = recipe.clone();
    paint(&mut changed).effects = vec![texture_effect_step(TextureEffect::HueRotate {
        degrees: 90.0,
    })];
    let _ = gpu_apply(&mut ui, &changed);
    let current = studio_store::get_artifact_stamped(&scope.source_id).unwrap();
    assert!(mcp
        .call(
            "poet_texture_effects_edit",
            json!({
                "context":stale["context"],"material_id":1,"effects":[],
            })
        )
        .is_err());
    assert_eq!(
        studio_store::get_artifact_stamped(&scope.source_id).unwrap(),
        current
    );

    let observer = login(&authority, "fixture-reviewer", &REVIEWER);
    let mut observer_mcp = project.mcp_backend(observer.clone()).unwrap();
    let read = observer_mcp.call("poet_source_read", json!({})).unwrap();
    let mut layers = changed.clone();
    paint(&mut layers).layers[0].blend_mode = LayerBlendMode::Screen;
    assert!(observer_mcp
        .call(
            "poet_texture_layers_edit",
            json!({
                "context":read["context"],"material_id":1,"layers":&paint(&mut layers).layers,
            })
        )
        .is_err());
    let mut observer_ui = project.connect(observer).unwrap();
    let source = observer_ui.snapshot().unwrap().source;
    assert!(observer_ui
        .authoring_edit(
            AuthoringEdit::TextureDetail {
                material_id: 1,
                detail: Some(layers),
            },
            source
        )
        .is_err());
    assert_eq!(
        studio_store::get_artifact_stamped(&scope.source_id).unwrap(),
        current
    );
}


#[test]
#[ignore = "explicit serial native GPU signed offsets isolated from masks"]
fn image_completed_workflow_offsets_without_masks_change_native_gpu() {
    let temp = tempfile::tempdir().unwrap();
    for name in ["authority", "hmc", "library"] {
        std::fs::create_dir(temp.path().join(name)).unwrap();
    }
    let (scope, _, authority) = fixture(&temp.path().join("authority"));
    let author = login(&authority, "fixture-author", &AUTHOR);
    let project = NativeProject::open(
        config(scope, &temp.path().join("hmc"), &temp.path().join("library")),
        authority, author.clone(), None, gpu_provider(),
    ).unwrap();
    let mut ui = project.connect(author.clone()).unwrap();
    let png = texture_effect_png();
    let base: [u8; 32] = Sha256::digest(&png).into();
    texture_effect_seed(&mut ui, base, png);
    let mut recipe = image_recipe(base);
    let (baseline_sha, baseline_frame) = gpu_apply(&mut ui, &recipe);
    image_gpu_evidence(&baseline_frame, "image-offset-baseline-gpu.png");
    let mut mcp = project.mcp_backend(author).unwrap();
    for offset in [[1, -1], [-1, 1]] {
        paint(&mut recipe).layers[0].offset = offset;
        assert!(paint(&mut recipe).layers[0].mask.is_none());
        let (sha, frame) = gpu_apply(&mut ui, &recipe);
        assert_ne!(sha, baseline_sha, "offset alone changes encoded pixels");
        assert_change(&baseline_frame, &frame, "unmasked signed offset");
        image_gpu_evidence(&frame, if offset[0] > 0 { "image-offset-positive-gpu.png" } else { "image-offset-negative-gpu.png" });
        let read = mcp.call("poet_source_read", json!({})).unwrap();
        mcp.call("poet_texture_layers_edit", json!({
            "context":read["context"],"material_id":1,"layers":&paint(&mut recipe).layers,
        })).unwrap();
        texture_effect_assert_recipe(&ui, &recipe);
        assert_eq!(texture_effect_digest(&ui), sha);
        assert_eq!(gpu_frame(&mut ui).rgba8, frame.rgba8);
    }
    let mut original_only = recipe.clone();
    paint(&mut original_only).layers.clear();
    let (original_sha, original_frame) = gpu_apply(&mut ui, &original_only);
    image_gpu_evidence(&original_frame, "image-offset-original-only-gpu.png");
    for offset in [[i32::MIN, i32::MAX], [i32::MAX, i32::MIN]] {
        paint(&mut recipe).layers[0].offset = offset;
        let (sha, frame) = gpu_apply(&mut ui, &recipe);
        assert_eq!(sha, original_sha, "offscreen offset clips the entire layer");
        assert_eq!(frame.rgba8, original_frame.rgba8);
    }
    paint(&mut recipe).layers[0].offset = [0, 0];
    let read = mcp.call("poet_source_read", json!({})).unwrap();
    mcp.call("poet_texture_layers_edit", json!({
        "context":read["context"],"material_id":1,"layers":&paint(&mut recipe).layers,
    })).unwrap();
    texture_effect_assert_recipe(&ui, &recipe);
    assert_eq!(texture_effect_digest(&ui), baseline_sha);
    assert_eq!(gpu_frame(&mut ui).rgba8, baseline_frame.rgba8);
}
