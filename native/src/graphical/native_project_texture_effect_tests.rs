//! Signed software-actor source/HMC tests. Synthetic images and canonical CPU
//! decoding are engineering witnesses, never GUI/GPU or human acceptance.
use super::*;
use crate::graphical::{
    authoring::AuthoringEdit,
    jobs, native_dispatch,
    session_host::SessionEditorHost,
    surface_host::SurfacePackageRequest,
    surfaces::{
        binding::digest_hex,
        detail::{
            DetailRecipe, PaintLayer, PaintRecipe, PaintStroke, TextureEffect, TextureEffectStep,
        },
    },
};
use qualia_core_db::render::{
    asset_package::{texture_hmc_key, HmcTextureResource},
    surface_authoring::encode_surface_png_into,
    texture_decode::{
        decode_hmc_texture_rgba8_into, inspect_hmc_texture_requirements, TextureDecodeLimits,
    },
};
use std::sync::Mutex;

const TEXTURE_EFFECT_CHILD: &str = "POET_TEXTURE_EFFECT_CASE_ROOT";
const TEXTURE_EFFECT_META: [u8; 4] = [0xa1, 0x61, b'p', 7];

struct TextureEffectNoGpu;
impl PreviewProvider for TextureEffectNoGpu {
    // This fixture does not retain a viewport or rendered frame.
    fn retire_retained_viewport(&mut self) -> Result<(), String> { Ok(()) }
    fn validate(&self, value: &Value) -> Result<(), String> {
        crate::graphical::assembly::admit_assembly(value, PreviewLimits::default()).map(|_| ())
    }
    fn preview(&mut self, _: SourceRef, _: &Value) -> Result<PreviewFrame, String> {
        Err("Software texture effect fixture provides no GUI/GPU preview".into())
    }
}

fn texture_effect_source() -> &'static str {
    r#"
let m={id:1,colour_space:"linear",base_color:[0.3,0.4,0.5,1.0],emissive:[0.0,0.0,0.0],metallic:0.1,roughness:0.5,specular:1.0,alpha_cutoff:0.5,opacity_mode:"Opaque",double_sided:true};
let tri={vertices:[[0.0,0.0,0.0],[1.0,0.0,0.0],[0.0,1.0,0.0]],triangles:[[0,1,2]],uv0:[[0.0,0.0],[1.0,0.0],[0.0,1.0]]};
let mut poet_parts=[{id:"root",semantic_id:101,geometry:tri,material:m,translation:[0.0,0.0,0.0],rotation:[0.0,0.0,0.0],scale:[1.0,1.0,1.0]}];
({parts:poet_parts});
"#
}

fn texture_effect_png() -> Vec<u8> {
    let mut rgba = Vec::new();
    for y in 0..8u8 {
        for x in 0..8u8 {
            rgba.extend_from_slice(&[20 + x * 15, 40 + y * 10, 170 - x * 10, 255]);
        }
    }
    let mut png = vec![0; rgba.len() + 8 + 4096];
    let n = encode_surface_png_into(8, &rgba, &mut png).unwrap();
    png.truncate(n);
    png
}

fn texture_effect_recipe(base: [u8; 32], effects: Vec<TextureEffectStep>) -> DetailRecipe {
    DetailRecipe::Paint(PaintRecipe {
        version: if effects.is_empty() { 1 } else { 2 },
        side: 8,
        base_sha256: digest_hex(&base),
        layers: vec![PaintLayer {
            name: "Retained editable marks".into(),
            visible: true,
            opacity: 0.7,
            strokes: vec![PaintStroke {
                size: 3.0,
                colour: [220, 30, 20, 255],
                opacity: 0.8,
                hardness: 0.5,
                points: vec![[3.0, 3.0], [4.0, 4.0]],
            }],

            ..crate::graphical::surfaces::detail::PaintLayer::default()
        }],
        effects,
    })
}

fn texture_effect_step(effect: TextureEffect) -> TextureEffectStep {
    TextureEffectStep {
        enabled: true,
        effect,
    }
}

fn texture_effect_block_on<F: std::future::Future>(future: F) -> F::Output {
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

/// Real HMC import and authorized binding; the digest never grants authority.
fn texture_effect_seed(host: &mut dyn EditorHost, base: [u8; 32], png: Vec<u8>) {
    let initial = host.snapshot().unwrap();
    let source = host
        .submit(texture_effect_source(), initial.source)
        .unwrap()
        .snapshot
        .source;
    let saved = host
        .save_checkpoint("input.hmc", ExpectedFile::Absent, source.clone())
        .unwrap();
    let imported = host
        .edit_package_resources(
            "input.hmc",
            ExpectedFile::Sha256(saved.digest),
            vec![ResourceEdit::Add(ResourceData {
                key: texture_hmc_key(&base),
                kind: "image/png".into(),
                bytes: png,
                meta: Some(TEXTURE_EFFECT_META.to_vec()),
            })],
            source.clone(),
        )
        .unwrap();
    host.bind_surface_package(
        SurfacePackageRequest {
            filename: "input.hmc".into(),
            digest: imported.digest,
        },
        source,
    )
    .unwrap();
    let current = host.snapshot().unwrap();
    let mut surface =
        crate::graphical::surfaces::SurfaceDraft::from_material(&MaterialRecord::legacy_default());
    surface.material.base_color_texture = base;
    let source = host
        .authoring_edit(
            AuthoringEdit::Surface {
                material_id: 1,
                surface,
            },
            current.source,
        )
        .unwrap()
        .snapshot
        .source;
    host.authoring_edit(
        AuthoringEdit::TextureDetail {
            material_id: 1,
            detail: Some(texture_effect_recipe(base, vec![])),
        },
        source,
    )
    .unwrap();
}

fn texture_effect_assert_recipe(host: &dyn EditorHost, expected: &DetailRecipe) {
    let source = host.snapshot().unwrap().source;
    let view = host.authoring_view(source).unwrap();
    let part = view.parts.iter().find(|part| part.id == "root").unwrap();
    assert_eq!(part.detail.as_ref(), Some(expected));
    assert_eq!((part.material_id, part.vertices, part.triangles), (1, 3, 1));
}

fn texture_effect_digest(host: &dyn EditorHost) -> [u8; 32] {
    let source = host.snapshot().unwrap().source;
    let inspected = host.inspect_surface("root", source).unwrap();
    assert!(inspected
        .texture_image
        .unwrap()
        .starts_with("data:image/png;base64,"));
    inspected
        .texture_sha256
        .expect("actual resolved colour digest")
}

fn texture_effect_pixels(bytes: &[u8], digest: [u8; 32]) -> Vec<u8> {
    let reader = qualia_core_db::bundle::BundleReader::parse(bytes).unwrap();
    let png = reader.get(&texture_hmc_key(&digest)).unwrap();
    assert_eq!(<[u8; 32]>::from(Sha256::digest(png)), digest);
    let resource = HmcTextureResource {
        digest,
        mime_type: "image/png",
        bytes: png,
    };
    let (info, scratch) =
        inspect_hmc_texture_requirements(&resource, TextureDecodeLimits::default()).unwrap();
    assert_eq!(info.rgba8_bytes, 8 * 8 * 4);
    let mut rgba = vec![0; info.rgba8_bytes];
    let mut scratch = vec![0; scratch];
    decode_hmc_texture_rgba8_into(
        &resource,
        TextureDecodeLimits::default(),
        &mut rgba,
        &mut scratch,
    )
    .unwrap();
    rgba
}

/// Opt-in retained diagnostics contain the actual encoded HMC resources. The
/// host running this test creates/selects this directory; product APIs do not.
fn texture_effect_evidence_png(package: &[u8], digest: [u8; 32], filename: &str) {
    if let Some(dir) = std::env::var_os("POET_TEXTURE_EFFECT_EVIDENCE_DIR") {
        let dir = std::path::PathBuf::from(dir);
        assert!(
            dir.is_dir(),
            "operator-selected evidence directory must already exist"
        );
        let reader = qualia_core_db::bundle::BundleReader::parse(package).unwrap();
        let png = reader.get(&texture_hmc_key(&digest)).unwrap();
        assert_eq!(<[u8; 32]>::from(Sha256::digest(png)), digest);
        std::fs::write(dir.join(filename), png).unwrap();
    }
}

fn texture_effect_history(
    host: &mut dyn EditorHost,
    prior: &DetailRecipe,
    prior_sha: [u8; 32],
    final_recipe: &DetailRecipe,
    final_sha: [u8; 32],
) {
    let before = host.snapshot().unwrap();
    let cells = host.source_cells(before.source.clone()).unwrap();
    let undone = host.undo(before.source).unwrap();
    assert!(undone.reconciliation_required);
    assert!(host.authoring_view(undone.source.clone()).is_err());
    host.rebuild_execution(undone.source).unwrap();
    texture_effect_assert_recipe(host, prior);
    assert_eq!(texture_effect_digest(host), prior_sha);
    let source = host.snapshot().unwrap().source;
    let redone = host.redo(source).unwrap();
    assert!(redone.reconciliation_required);
    let rebuilt = host.rebuild_execution(redone.source).unwrap();
    assert!(
        rebuilt.frame.is_none(),
        "CPU diagnostics are not a GPU frame"
    );
    assert_eq!(host.source_cells(rebuilt.snapshot.source).unwrap(), cells);
    texture_effect_assert_recipe(host, final_recipe);
    assert_eq!(texture_effect_digest(host), final_sha);
}

fn texture_effect_child_reopen(root: &Path) {
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
        "fresh process has no retained source"
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
        Some("saved.hmc"),
        Box::new(TextureEffectNoGpu),
    )
    .unwrap();
    let mut ui = project.connect(author.clone()).unwrap();
    let held = ui.snapshot().unwrap();
    assert!(held.reconciliation_required);
    assert_eq!(
        json!(ui.source_cells(held.source.clone()).unwrap()),
        witness["source_cells"]
    );
    let saved = std::fs::read(root.join("hmc/saved.hmc")).unwrap();
    let loaded =
        checkpoint::load(&root.join("hmc/saved.hmc"), CheckpointLimits::default()).unwrap();
    assert_eq!(
        loaded.artifact(),
        &studio_store::get_artifact_stamped(&scope.source_id)
            .unwrap()
            .0
    );
    assert!(!loaded.artifact().history.is_empty());
    ui.bind_surface_package(
        SurfacePackageRequest {
            filename: "saved.hmc".into(),
            digest: Sha256::digest(&saved).into(),
        },
        held.source.clone(),
    )
    .unwrap();
    ui.rebuild_execution(held.source).unwrap();
    let base: [u8; 32] = serde_json::from_value(witness["base"].clone()).unwrap();
    let prior_sha: [u8; 32] = serde_json::from_value(witness["prior_sha"].clone()).unwrap();
    let final_sha: [u8; 32] = serde_json::from_value(witness["final_sha"].clone()).unwrap();
    let prior: DetailRecipe = serde_json::from_value(witness["prior"].clone()).unwrap();
    let final_recipe: DetailRecipe =
        serde_json::from_value(witness["final_recipe"].clone()).unwrap();
    texture_effect_assert_recipe(&ui, &final_recipe);
    assert_eq!(texture_effect_digest(&ui), final_sha);
    texture_effect_history(&mut ui, &prior, prior_sha, &final_recipe, final_sha);
    let reader = qualia_core_db::bundle::BundleReader::parse(&saved).unwrap();
    assert_eq!(
        reader.get(&texture_hmc_key(&base)),
        Some(texture_effect_png().as_slice())
    );
    assert_eq!(
        reader.entry(&texture_hmc_key(&base)).unwrap().meta,
        Some(TEXTURE_EFFECT_META.to_vec())
    );
    let final_pixels = texture_effect_pixels(&saved, final_sha);
    assert_eq!(json!(final_pixels), witness["final_pixels"]);
    let mut mcp = project.mcp_backend(author).unwrap();
    let read = mcp.call("poet_source_read", json!({})).unwrap();
    let inspected = mcp
        .call(
            "poet_texture_effects_inspect",
            json!({"context":read["context"],"part_id":"root"}),
        )
        .unwrap();
    assert_eq!(
        inspected["detail"],
        serde_json::to_value(final_recipe).unwrap()
    );
    assert_eq!(inspected["output_sha256"], digest_hex(&final_sha));
    let source = ui.snapshot().unwrap().source;
    let resaved = ui
        .save_checkpoint("child.hmc", ExpectedFile::Absent, source)
        .unwrap();
    let reloaded = checkpoint::load(&resaved.path, CheckpointLimits::default()).unwrap();
    for digest in [base, final_sha] {
        let key = texture_hmc_key(&digest);
        assert_eq!(
            loaded.resources().iter().find(|r| r.key == key),
            reloaded.resources().iter().find(|r| r.key == key)
        );
    }
    texture_effect_evidence_png(&saved, final_sha, "reopened.png");
    std::fs::write(
        root.join("child-recovered.json"),
        serde_json::to_vec(&json!({
            "source_id":scope.source_id,"source_history_resources_restored":true,
        }))
        .unwrap(),
    )
    .unwrap();
}

#[test]
fn signed_texture_effect_native_mcp_pixels_history_hmc_and_fresh_process() {
    if let Some(root) = std::env::var_os(TEXTURE_EFFECT_CHILD) {
        texture_effect_child_reopen(Path::new(&root));
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
                Box::new(TextureEffectNoGpu),
            )
            .unwrap(),
        );
        let mut operator =
            SessionEditorHost::new(project.clone(), Arc::new(Mutex::new(Some(author.clone()))));
        let png = texture_effect_png();
        let base: [u8; 32] = Sha256::digest(&png).into();
        texture_effect_seed(&mut operator, base, png.clone());
        let baseline_sha = texture_effect_digest(&operator);
        let current = operator.snapshot().unwrap();
        let baseline = operator
            .compile_checkpoint(
                "baseline.hmc",
                "texture-effect-baseline",
                ExpectedFile::Absent,
                current.source.clone(),
            )
            .unwrap();
        let baseline_pixels =
            texture_effect_pixels(&std::fs::read(&baseline.path).unwrap(), baseline_sha);
        let current = operator.snapshot().unwrap();
        let prior = texture_effect_recipe(
            base,
            vec![texture_effect_step(TextureEffect::Sepia { intensity: 0.6 })],
        );
        let (worker, handle) = jobs::Worker::start(Box::new(operator)).unwrap();
        let reply = texture_effect_block_on(native_dispatch::run(
            handle.clone(),
            jobs::Action::AuthoringEdit {
                edit: AuthoringEdit::TextureDetail {
                    material_id: 1,
                    detail: Some(prior.clone()),
                },
                expected: current.source.clone(),
            },
            jobs::Context::from_snapshot(&current),
        ))
        .unwrap();
        assert!(
            reply.operation_succeeded,
            "native effect intent: {}",
            reply.message
        );
        let mut ui = project.connect(author.clone()).unwrap();
        texture_effect_assert_recipe(&ui, &prior);
        let prior_sha = texture_effect_digest(&ui);
        assert_ne!(prior_sha, baseline_sha);
        let mut mcp = project.mcp_backend(author).unwrap();
        let read = mcp.call("poet_source_read", json!({})).unwrap();
        let inspected = mcp
            .call(
                "poet_texture_effects_inspect",
                json!({"context":read["context"],"part_id":"root"}),
            )
            .unwrap();
        assert_eq!(inspected["detail"], serde_json::to_value(&prior).unwrap());
        assert_eq!(inspected["base_sha256"], digest_hex(&base));
        assert_eq!(inspected["output_sha256"], digest_hex(&prior_sha));
        assert_eq!(inspected["source_revision"], read["source_revision"]);
        assert_eq!(inspected["session_revision"], read["session_revision"]);
        assert_eq!(inspected["artifact_id"], read["artifact_id"]);
        assert_eq!(inspected["part_id"], "root");
        assert_eq!(inspected["material_id"], 1);
        let effects = vec![
            texture_effect_step(TextureEffect::Invert),
            TextureEffectStep {
                enabled: false,
                effect: TextureEffect::Sepia { intensity: 0.25 },
            },
        ];
        let final_recipe = texture_effect_recipe(base, effects.clone());
        let edited = mcp
            .call(
                "poet_texture_effects_edit",
                json!({"context":read["context"],"material_id":1,"effects":effects}),
            )
            .unwrap();
        assert_eq!(edited["read_required"], true);
        assert_eq!(edited["preview_rendered"], false);
        assert!(mcp
            .call(
                "poet_texture_effects_inspect",
                json!({"context":read["context"],"part_id":"root"})
            )
            .is_err());
        texture_effect_assert_recipe(&ui, &final_recipe);
        let final_sha = texture_effect_digest(&ui);
        assert_ne!(final_sha, baseline_sha);
        let read = mcp.call("poet_source_read", json!({})).unwrap();
        let inspected = mcp
            .call(
                "poet_texture_effects_inspect",
                json!({"context":read["context"],"part_id":"root"}),
            )
            .unwrap();
        assert_eq!(
            inspected["detail"],
            serde_json::to_value(&final_recipe).unwrap()
        );
        assert_eq!(inspected["base_sha256"], digest_hex(&base));
        assert_eq!(inspected["output_sha256"], digest_hex(&final_sha));
        texture_effect_history(&mut ui, &prior, prior_sha, &final_recipe, final_sha);
        let source = ui.snapshot().unwrap().source;
        let compiled = ui
            .compile_checkpoint(
                "effects.hmc",
                "texture-effect",
                ExpectedFile::Absent,
                source.clone(),
            )
            .unwrap();
        let compiled_bytes = std::fs::read(&compiled.path).unwrap();
        let final_pixels = texture_effect_pixels(&compiled_bytes, final_sha);
        assert_ne!(
            final_pixels, baseline_pixels,
            "encoded output must change actual canonical pixels"
        );
        for (before, after) in baseline_pixels
            .chunks_exact(4)
            .zip(final_pixels.chunks_exact(4))
        {
            assert_eq!(
                after,
                &[255 - before[0], 255 - before[1], 255 - before[2], before[3]]
            );
        }
        ui.bind_surface_package(
            SurfacePackageRequest {
                filename: "effects.hmc".into(),
                digest: compiled.digest,
            },
            source.clone(),
        )
        .unwrap();
        ui.rebuild_execution(source).unwrap();
        let source = ui.snapshot().unwrap().source;
        let cells = ui.source_cells(source.clone()).unwrap();
        let saved = ui
            .save_checkpoint("saved.hmc", ExpectedFile::Absent, source)
            .unwrap();
        let loaded = checkpoint::load(&saved.path, CheckpointLimits::default()).unwrap();
        let compiled_loaded =
            checkpoint::load(&compiled.path, CheckpointLimits::default()).unwrap();
        assert_eq!(
            loaded.artifact(),
            &studio_store::get_artifact_stamped(&scope.source_id)
                .unwrap()
                .0
        );
        assert!(!loaded.artifact().history.is_empty());
        for digest in [base, final_sha] {
            let key = texture_hmc_key(&digest);
            let retained = loaded.resources().iter().find(|r| r.key == key).unwrap();
            assert_eq!(
                Some(retained),
                compiled_loaded.resources().iter().find(|r| r.key == key)
            );
        }
        let reader_bytes = std::fs::read(&saved.path).unwrap();
        let reader = qualia_core_db::bundle::BundleReader::parse(&reader_bytes).unwrap();
        assert_eq!(reader.get(&texture_hmc_key(&base)), Some(png.as_slice()));
        assert_eq!(
            reader.entry(&texture_hmc_key(&base)).unwrap().meta,
            Some(TEXTURE_EFFECT_META.to_vec())
        );
        std::fs::write(temp.path().join("witness.json"), serde_json::to_vec(&json!({
            "project_id":scope.project_id,"document_id":scope.document_id,"source_id":scope.source_id,
            "source_cells":cells,"base":base,"prior":prior,"prior_sha":prior_sha,
            "final_recipe":final_recipe,"final_sha":final_sha,"final_pixels":final_pixels,
        })).unwrap()).unwrap();
        drop(mcp);
        drop(ui);
        worker.finish().unwrap();
        drop(handle);
        drop(project);
    }
    let current = std::thread::current();
    let test_name = current
        .name()
        .expect("test thread supplies exact test name");
    let output = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", test_name, "--nocapture", "--test-threads=1"])
        .env(TEXTURE_EFFECT_CHILD, temp.path())
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "fresh-process texture effect restore failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        temp.path().join("child-recovered.json").is_file(),
        "zero-test child exit is not recovery evidence"
    );
    let witness: serde_json::Value =
        serde_json::from_slice(&std::fs::read(temp.path().join("witness.json")).unwrap()).unwrap();
    let base: [u8; 32] = serde_json::from_value(witness["base"].clone()).unwrap();
    let final_sha: [u8; 32] = serde_json::from_value(witness["final_sha"].clone()).unwrap();
    let package = std::fs::read(temp.path().join("hmc/saved.hmc")).unwrap();
    texture_effect_evidence_png(&package, base, "original.png");
    texture_effect_evidence_png(&package, final_sha, "effects.png");
    if let Some(dir) = std::env::var_os("POET_TEXTURE_EFFECT_EVIDENCE_DIR") {
        std::fs::write(std::path::PathBuf::from(dir).join("witness.json"), serde_json::to_vec_pretty(&json!({
            "qualification":"8x8 synthetic calibration; CPU canonical encoded resources; no biological/production asset, human or GPU acceptance",
            "project_id":witness["project_id"],"document_id":witness["document_id"],"source_id":witness["source_id"],
            "base_sha256":digest_hex(&base),"output_sha256":digest_hex(&final_sha),
            "package_sha256":digest_hex(&Sha256::digest(&package).into()),
        })).unwrap()).unwrap();
    }
}

#[test]
fn signed_texture_effect_refusals_preserve_source_history_and_context_rules() {
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
        Box::new(TextureEffectNoGpu),
    )
    .unwrap();
    let mut ui = project.connect(author.clone()).unwrap();
    let png = texture_effect_png();
    let base: [u8; 32] = Sha256::digest(&png).into();
    texture_effect_seed(&mut ui, base, png);
    let mut mcp = project.mcp_backend(author).unwrap();
    let stale = mcp.call("poet_source_read", json!({})).unwrap();
    let source = ui.snapshot().unwrap().source;
    let recipe = texture_effect_recipe(base, vec![texture_effect_step(TextureEffect::Invert)]);
    ui.authoring_edit(
        AuthoringEdit::TextureDetail {
            material_id: 1,
            detail: Some(recipe),
        },
        source,
    )
    .unwrap();
    let retained = studio_store::get_artifact_stamped(&scope.source_id).unwrap();
    let mut invalid_native =
        texture_effect_recipe(base, vec![texture_effect_step(TextureEffect::Invert)]);
    let DetailRecipe::Paint(paint) = &mut invalid_native else {
        unreachable!()
    };
    paint.version = 4;
    let current = ui.snapshot().unwrap();
    assert!(ui
        .authoring_edit(
            AuthoringEdit::TextureDetail {
                material_id: 1,
                detail: Some(invalid_native),
            },
            current.source
        )
        .is_err());
    assert_eq!(
        studio_store::get_artifact_stamped(&scope.source_id).unwrap(),
        retained
    );
    let valid = json!([{"enabled":true,"effect":{"kind":"grayscale"}}]);
    assert!(mcp
        .call(
            "poet_texture_effects_edit",
            json!({"context":stale["context"],"material_id":1,"effects":valid})
        )
        .is_err());
    assert_eq!(
        studio_store::get_artifact_stamped(&scope.source_id).unwrap(),
        retained
    );
    // A well-formed request with a foreign connection token must consume this
    // connection's pending context; parameters alone are not a reusable permit.
    let read = mcp.call("poet_source_read", json!({})).unwrap();
    assert!(mcp
        .call(
            "poet_texture_effects_edit",
            json!({"context":"foreign-context","material_id":1,"effects":valid})
        )
        .is_err());
    assert!(mcp
        .call(
            "poet_texture_effects_inspect",
            json!({"context":read["context"],"part_id":"root"})
        )
        .is_err());
    assert_eq!(
        studio_store::get_artifact_stamped(&scope.source_id).unwrap(),
        retained
    );
    for invalid in [
        json!([{"enabled":true,"effect":{"kind":"unknown"}}]),
        json!([{"enabled":false,"effect":{"kind":"sepia","intensity":1.1}}]),
        json!([{"enabled":true,"effect":{"kind":"invert","strength":0.5}}]),
        json!([{"enabled":true,"effect":{"kind":"invert"},"unknown":true}]),
        json!(vec![json!({"enabled":true,"effect":{"kind":"invert"}}); 9]),
    ] {
        let read = mcp.call("poet_source_read", json!({})).unwrap();
        assert!(mcp
            .call(
                "poet_texture_effects_edit",
                json!({"context":read["context"],"material_id":1,"effects":invalid})
            )
            .is_err());
        assert!(
            mcp.call(
                "poet_texture_effects_inspect",
                json!({"context":read["context"],"part_id":"root"})
            )
            .is_ok(),
            "shape/parameter preflight must leave read context reusable"
        );
        assert_eq!(
            studio_store::get_artifact_stamped(&scope.source_id).unwrap(),
            retained
        );
    }
    let read = mcp.call("poet_source_read", json!({})).unwrap();
    assert!(mcp.call("poet_texture_effects_edit", json!({"context":read["context"],"material_id":1,"effects":valid,"untrusted_principal":"fixture-author"})).is_err());
    assert!(mcp
        .call(
            "poet_texture_effects_inspect",
            json!({"context":read["context"],"part_id":"root"})
        )
        .is_ok());
    assert!(mcp
        .call(
            "poet_texture_effects_edit",
            json!({"context":read["context"],"material_id":99,"effects":valid})
        )
        .is_err());
    assert!(
        mcp.call(
            "poet_texture_effects_inspect",
            json!({"context":read["context"],"part_id":"root"})
        )
        .is_err(),
        "valid-shaped missing-target mutation consumes context"
    );
    assert_eq!(
        studio_store::get_artifact_stamped(&scope.source_id).unwrap(),
        retained
    );
    let observer = login(&authority, "fixture-reviewer", &REVIEWER);
    assert!(
        ui.snapshot().is_ok(),
        "reviewer login preserves the author session"
    );
    assert!(mcp.call("poet_source_read", json!({})).is_ok());
    let mut observer_mcp = project.mcp_backend(observer.clone()).unwrap();
    let read = observer_mcp.call("poet_source_read", json!({})).unwrap();
    assert!(observer_mcp
        .call(
            "poet_texture_effects_inspect",
            json!({"context":read["context"],"part_id":"root"})
        )
        .is_ok());
    assert!(observer_mcp
        .call(
            "poet_texture_effects_edit",
            json!({"context":read["context"],"material_id":1,"effects":valid})
        )
        .is_err());
    let read = observer_mcp.call("poet_source_read", json!({})).unwrap();
    assert!(observer_mcp.call("poet_texture_mask_morphology", json!({
        "context":read["context"],"material_id":1,"layer_index":0,
        "operation":"expand","radius":1.0
    })).is_err());
    let mut observer_ui = project.connect(observer).unwrap();
    let source = observer_ui.snapshot().unwrap().source;
    assert!(observer_ui
        .authoring_edit(
            AuthoringEdit::TextureDetail {
                material_id: 1,
                detail: None
            },
            source
        )
        .is_err());
    assert_eq!(
        studio_store::get_artifact_stamped(&scope.source_id).unwrap(),
        retained
    );
    drop(observer_ui);
    drop(observer_mcp);
    drop(ui);
    drop(mcp);
    let author = login(&authority, "fixture-author", &AUTHOR);
    let ui = project.connect(author.clone()).unwrap();
    let mut mcp = project.mcp_backend(author.clone()).unwrap();
    let mut revoked_morphology = project.mcp_backend(author).unwrap();
    let read = mcp.call("poet_source_read", json!({})).unwrap();
    let morphology_read = revoked_morphology.call("poet_source_read", json!({})).unwrap();
    admission.sequence = 3;
    admission.subjects[0].revoked = true;
    authority.replace_admission(signed(admission)).unwrap();
    assert!(ui.snapshot().is_err());
    assert!(mcp
        .call(
            "poet_texture_effects_inspect",
            json!({"context":read["context"],"part_id":"root"})
        )
        .is_err());
    assert!(mcp
        .call(
            "poet_texture_effects_edit",
            json!({"context":read["context"],"material_id":1,"effects":valid})
        )
        .is_err());
    assert!(revoked_morphology.call("poet_texture_mask_morphology", json!({
        "context":morphology_read["context"],"material_id":1,"layer_index":0,
        "operation":"expand","radius":1.0
    })).is_err());
    assert_eq!(
        studio_store::get_artifact_stamped(&scope.source_id).unwrap(),
        retained,
        "refusals preserve exact source artifact, stamp and history"
    );
}

const MASK_MORPHOLOGY_CHILD: &str = "POET_MASK_MORPHOLOGY_CASE_ROOT";

fn mask_digest(recipe: &DetailRecipe) -> [u8; 32] {
    use crate::graphical::surfaces::detail::LayerMask;
    let DetailRecipe::Paint(paint) = recipe else { panic!("expected paint") };
    let Some(LayerMask::Resource { sha256, .. }) = &paint.layers[0].mask else {
        panic!("expected generated mask resource")
    };
    crate::graphical::surfaces::binding::parse_digest(sha256).unwrap()
}

fn mask_recipe(host: &dyn EditorHost) -> DetailRecipe {
    let source = host.snapshot().unwrap().source;
    host.authoring_view(source).unwrap().parts.into_iter()
        .find(|part| part.id == "root").unwrap().detail.unwrap()
}

fn morphology_reopen(root: &Path) {
    let witness: serde_json::Value = serde_json::from_slice(&std::fs::read(root.join("mask-witness.json")).unwrap()).unwrap();
    let scope = Scope {
        project_id: witness["project_id"].as_str().unwrap().into(),
        document_id: witness["document_id"].as_str().unwrap().into(),
        source_id: witness["source_id"].as_str().unwrap().into(),
        domain: "Geometry".into(),
    };
    assert!(studio_store::get_artifact_stamped(&scope.source_id).is_none());
    let authority = Arc::new(NativeAuthority::open_retained_current(
        &root.join("authority"), TrustedProject {
            project_id: scope.project_id.clone(), issuer_identifier: "fixture-issuer".into(),
            public_key: ed25519::public_from_seed(&AUTHORITY),
        }, 1024 * 1024, 256).unwrap());
    let author = login(&authority, "fixture-author", &AUTHOR);
    let project = NativeProject::open(config(scope, &root.join("hmc"), &root.join("library")),
        authority, author.clone(), Some("mask-saved.hmc"), Box::new(TextureEffectNoGpu)).unwrap();
    let mut ui = project.connect(author).unwrap();
    let held = ui.snapshot().unwrap();
    let bytes = std::fs::read(root.join("hmc/mask-saved.hmc")).unwrap();
    let final_recipe: DetailRecipe = serde_json::from_value(witness["final_recipe"].clone()).unwrap();
    let prior_recipe: DetailRecipe = serde_json::from_value(witness["prior_recipe"].clone()).unwrap();
    let generated = mask_digest(&final_recipe);
    let reader = qualia_core_db::bundle::BundleReader::parse(&bytes).unwrap();
    let png = reader.get(&texture_hmc_key(&generated)).unwrap();
    assert_eq!(<[u8; 32]>::from(Sha256::digest(png)), generated);
    let prior_digest = mask_digest(&prior_recipe);
    assert_ne!(prior_digest, generated);
    // A package containing the current mask but omitting the earlier one can
    // resolve today's source, but cannot safely serve the retained undo state.
    let mut reduced = qualia_core_db::bundle::BundleWriter::new();
    for entry in reader.entries() {
        if entry.key != texture_hmc_key(&prior_digest) {
            reduced.add_file(&entry.key, &entry.kind, reader.get(&entry.key).unwrap().to_vec(), entry.meta.clone()).unwrap();
        }
    }
    let reduced = reduced.build().unwrap();
    std::fs::write(root.join("hmc/mask-missing-history.hmc"), &reduced).unwrap();
    // The carrier protects the first bind before an editor snapshot exists.
    assert!(ui.bind_surface_package(SurfacePackageRequest {
        filename:"mask-missing-history.hmc".into(), digest:Sha256::digest(&reduced).into(),
    }, held.source.clone()).is_err());
    ui.bind_surface_package(SurfacePackageRequest {
        filename: "mask-saved.hmc".into(), digest: Sha256::digest(&bytes).into(),
    }, held.source.clone()).unwrap();
    ui.rebuild_execution(held.source).unwrap();
    assert_eq!(mask_recipe(&ui), final_recipe);
    let current = ui.snapshot().unwrap().source;
    assert!(ui.bind_surface_package(SurfacePackageRequest {
        filename:"mask-missing-history.hmc".into(), digest:Sha256::digest(&reduced).into(),
    }, current).is_err());
    let source = ui.snapshot().unwrap().source;
    let undone = ui.undo(source).unwrap();
    ui.rebuild_execution(undone.source).unwrap();
    assert_eq!(mask_recipe(&ui), prior_recipe);
    let source = ui.snapshot().unwrap().source;
    let redone = ui.redo(source).unwrap();
    ui.rebuild_execution(redone.source).unwrap();
    assert_eq!(mask_recipe(&ui), final_recipe);
    texture_effect_digest(&ui);
}

#[test]
fn signed_mask_morphology_history_rebind_checkpoint_and_fresh_reopen() {
    use crate::graphical::surfaces::detail::{LayerMask, MaskMorphologyOperation};
    if let Some(root) = std::env::var_os(MASK_MORPHOLOGY_CHILD) {
        morphology_reopen(Path::new(&root));
        return;
    }
    let temp = tempfile::tempdir().unwrap();
    for name in ["authority", "hmc", "library"] { std::fs::create_dir(temp.path().join(name)).unwrap(); }
    {
        let (scope, _, authority) = fixture(&temp.path().join("authority"));
        let author = login(&authority, "fixture-author", &AUTHOR);
        let project = NativeProject::open(config(scope.clone(), &temp.path().join("hmc"), &temp.path().join("library")),
            authority, author.clone(), None, Box::new(TextureEffectNoGpu)).unwrap();
        let mut ui = project.connect(author.clone()).unwrap();
        let png = texture_effect_png();
        let base: [u8; 32] = Sha256::digest(&png).into();
        texture_effect_seed(&mut ui, base, png);
        let mut prior = texture_effect_recipe(base, vec![]);
        let DetailRecipe::Paint(paint) = &mut prior else { unreachable!() };
        paint.version = 3;
        paint.layers[0].mask = Some(LayerMask::Rectangle { x: 2, y: 2, width: 3, height: 3, coverage: 255 });
        let source = ui.snapshot().unwrap().source;
        ui.authoring_edit(AuthoringEdit::TextureDetail { material_id: 1, detail: Some(prior.clone()) }, source).unwrap();
        let source = ui.snapshot().unwrap().source;
        let old = ui.save_checkpoint("before-mask.hmc", ExpectedFile::Absent, source).unwrap();
        let mut mcp = project.mcp_backend(author).unwrap();
        let stale = mcp.call("poet_source_read", json!({})).unwrap();
        let before = ui.snapshot().unwrap().source;
        ui.authoring_edit(AuthoringEdit::TextureMaskMorphology {
            material_id: 1, layer_index: 0, operation: MaskMorphologyOperation::Expand, radius: 1.0,
        }, before).unwrap();
        let final_recipe = mask_recipe(&ui);
        let generated = mask_digest(&final_recipe);
        let retained = studio_store::get_artifact_stamped(&scope.source_id).unwrap();
        assert!(mcp.call("poet_texture_mask_morphology", json!({"context":stale["context"],
            "material_id":1,"layer_index":0,"operation":"expand","radius":1.0})).is_err());
        assert_eq!(studio_store::get_artifact_stamped(&scope.source_id).unwrap(), retained);

        // Undo does not permit a package switch to drop bytes required by redo.
        let source = ui.snapshot().unwrap().source;
        let undone = ui.undo(source).unwrap();
        ui.rebuild_execution(undone.source.clone()).unwrap();
        assert_eq!(mask_recipe(&ui), prior);
        assert!(ui.bind_surface_package(SurfacePackageRequest { filename: "before-mask.hmc".into(), digest: old.digest }, undone.source).is_err());
        let source = ui.snapshot().unwrap().source;
        let redone = ui.redo(source).unwrap();
        ui.rebuild_execution(redone.source).unwrap();
        assert_eq!(mask_recipe(&ui), final_recipe);

        // The MCP route makes the same source-owned operation available to
        // authorized agents and consumes its own connection-local context.
        let read = mcp.call("poet_source_read", json!({})).unwrap();
        mcp.call("poet_texture_mask_morphology", json!({"context":read["context"],
            "material_id":1,"layer_index":0,"operation":"contract","radius":1.0})).unwrap();
        let prior = final_recipe;
        let final_recipe = mask_recipe(&ui);
        let final_generated = mask_digest(&final_recipe);

        let source = ui.snapshot().unwrap().source;
        let saved = ui.save_checkpoint("mask-saved.hmc", ExpectedFile::Absent, source.clone()).unwrap();
        let loaded = checkpoint::load(&saved.path, CheckpointLimits::default()).unwrap();
        assert!(loaded.resources().iter().any(|r| r.key == texture_hmc_key(&generated)));
        assert!(loaded.resources().iter().any(|r| r.key == texture_hmc_key(&final_generated)));
        ui.bind_surface_package(SurfacePackageRequest { filename: "mask-saved.hmc".into(), digest: saved.digest }, source).unwrap();
        std::fs::write(temp.path().join("mask-witness.json"), serde_json::to_vec(&json!({
            "project_id":scope.project_id,"document_id":scope.document_id,"source_id":scope.source_id,
            "prior_recipe":prior,"final_recipe":final_recipe,
        })).unwrap()).unwrap();
    }
    let current = std::thread::current();
    let output = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", current.name().unwrap(), "--nocapture", "--test-threads=1"])
        .env(MASK_MORPHOLOGY_CHILD, temp.path()).output().unwrap();
    assert!(output.status.success(), "fresh-process mask reopen failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status, String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr));
}

#[path = "native_project_texture_effect_gpu_tests.rs"]
mod gpu_tests;
