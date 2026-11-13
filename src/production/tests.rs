use super::*;
use qualia_core_db::bundle::{BundleReader, BundleWriter};
const SVG: &str = r##"import "vibe:0.3/render" as render;
let radius = render.spatial.skeletal_lerp_joint([8.0, 0.0, 0.0], [12.0, 0.0, 0.0], 0.5)[0];
let mut asset = { files: [{ key: "icons/dot.svg", kind: "svg", text: "<svg xmlns=\"http://www.w3.org/2000/svg\"><circle r=\"10\"/></svg>" }] };"##;

#[test]
fn original_files_v1_packages_reopen_with_original_semantics() {
    let limits = ProductionLimits::default();
    let cancel = AtomicBool::new(false);
    let mut original = ProductionSession::new(Arc::new(super::files::LegacyFilesProducer), vec![], limits.clone());
    original.submit(r#"let mut asset = {files:[{key:"legacy.txt",kind:"txt",text:"original"}]};"#, &cancel).unwrap();
    let bytes = original.candidate().unwrap().bundle();
    let reopened = ProducerRegistry::with_builtins().reopen(bytes, limits, &cancel).unwrap();
    assert_eq!(reopened.profile(), ("files", "1"));
    assert_eq!(reopened.candidate().unwrap().bundle(), bytes);
    assert_eq!(FilesProducer.version(), "2");
}

#[test]
fn original_files_v1_package_writer_keeps_reserved_looking_images_under_outputs() {
    let resource=Resource{key:format!("textures/sha256/{}","0".repeat(64)),kind:"image/png".into(),bytes:vec![1,2,3]};
    let bytes=super::package::build(&super::files::LegacyFilesProducer,&["fixture".into()],&[],&[resource.clone()],&ProductionLimits::default()).unwrap();
    let reader=BundleReader::parse(&bytes).unwrap();
    assert!(reader.get(&format!("outputs/{}",resource.key)).is_some());
    assert!(reader.get(&resource.key).is_none());
}

#[test]
fn heterogeneous_files_and_retained_inputs_roundtrip_with_edits() {
    let registry = ProducerRegistry::with_builtins();
    let cancel = AtomicBool::new(false);
    let inputs = vec![Resource {
        key: "sample.bin".into(),
        kind: "bin".into(),
        bytes: vec![2, 4, 8],
    }];
    let mut session = registry
        .session("files", inputs, ProductionLimits::default())
        .unwrap();
    session.submit(SVG, &cancel).unwrap();
    session
        .submit(
            r#"asset = { files: [
        { key: "palette.json", kind: "json", text: "{\"color\":\"blue\"}" },
        { key: "pixels.bin", kind: "bin", bytes: [0, 127, 255] },
        { key: "sample-copy.bin", kind: "bin", input: "sample.bin" }
    ] };"#,
            &cancel,
        )
        .unwrap();
    let candidate = session.candidate().unwrap();
    assert_eq!(candidate.revision(), 2);
    assert_eq!(candidate.outputs()[1].bytes, [0, 127, 255]);
    let reopened = registry
        .reopen(candidate.bundle(), ProductionLimits::default(), &cancel)
        .unwrap();
    assert_eq!(reopened.sources(), session.sources());
    assert_eq!(reopened.candidate().unwrap().bundle(), candidate.bundle());
    assert_eq!(reopened.candidate().unwrap().outputs(), candidate.outputs());
}

// External consumer-style producer: no species, rig, playback or JSON schema dependency.
struct NumberProducer {
    version: &'static str,
    bias: i64,
}
impl AssetProducer for NumberProducer {
    fn id(&self) -> &str {
        "number"
    }
    fn version(&self) -> &str {
        self.version
    }
    fn produce(
        &self,
        asset: &AssetValue,
        context: &ProductionContext<'_>,
    ) -> Result<Vec<Resource>, String> {
        context.check_cancelled()?;
        let n = match asset {
            AssetValue::I64(n) => *n,
            AssetValue::U64(n) => {
                i64::try_from(*n).map_err(|_| "Integer exceeds producer range")?
            }
            _ => return Err("Expected integer asset".into()),
        };
        if n < 0 {
            return Err("Expected nonnegative integer".into());
        }
        Ok(vec![Resource {
            key: "value.txt".into(),
            kind: "txt".into(),
            bytes: (n + self.bias).to_string().into_bytes(),
        }])
    }
}
#[test]
fn custom_producer_sdk_expression_and_replay_require_matching_semantics() {
    let mut registry = ProducerRegistry::default();
    registry
        .register(NumberProducer {
            version: "1",
            bias: 0,
        })
        .unwrap();
    assert!(registry
        .register(NumberProducer {
            version: "1",
            bias: 0
        })
        .is_err());
    let cancel = AtomicBool::new(false);
    let mut session = registry
        .session("number", vec![], ProductionLimits::default())
        .unwrap();
    session
        .submit(
            r#"import "vibe:0.3/nquin" as nquin;
        let mut asset = nquin.wire.compute_wire_parity([1, 2, 4, 8, 16]);"#,
            &cancel,
        )
        .unwrap();
    assert_eq!(session.candidate().unwrap().outputs()[0].bytes, b"31");
    let bytes = session.candidate().unwrap().bundle();
    registry
        .reopen(bytes, ProductionLimits::default(), &cancel)
        .unwrap();
    for (version, bias) in [("2", 0), ("1", 1)] {
        let mut changed = ProducerRegistry::default();
        changed.register(NumberProducer { version, bias }).unwrap();
        assert!(changed
            .reopen(bytes, ProductionLimits::default(), &cancel)
            .is_err());
    }
    assert!(ProducerRegistry::with_builtins()
        .reopen(bytes, ProductionLimits::default(), &cancel)
        .is_err());
}

#[test]
fn rejection_and_cancellation_preserve_source_bindings_and_package() {
    let registry = ProducerRegistry::with_builtins();
    let cancel = AtomicBool::new(false);
    let mut session = registry
        .session("files", vec![], ProductionLimits::default())
        .unwrap();
    session.submit(SVG, &cancel).unwrap();
    let hash = session.candidate().unwrap().sha256();
    for source in [
        r#"asset = { files: [{ key: "../escape", kind: "txt", text: "no" }] };"#,
        r#"asset = { files: [{ key: "a", kind: "bin", bytes: [256] }] };"#,
        r#"asset = { files: [{ key: "a", kind: "txt", text: "a" }, { key: "a", kind: "txt", text: "b" }] };"#,
        r#"asset = { files: [{ key: "a", kind: "bin", input: "absent" }] };"#,
    ] {
        assert!(session.submit(source, &cancel).is_err());
    }
    cancel.store(true, Ordering::Release);
    assert!(session.submit("= asset", &cancel).is_err());
    assert_eq!(session.revision(), 1);
    assert_eq!(session.candidate().unwrap().sha256(), hash);
    cancel.store(false, Ordering::Release);
    session.submit("= asset", &cancel).unwrap();
    assert_eq!(
        session.candidate().unwrap().outputs()[0].key,
        "icons/dot.svg"
    );
}

#[test]
fn package_tampering_rejected_even_with_recomputed_container_hashes() {
    let registry = ProducerRegistry::with_builtins();
    let cancel = AtomicBool::new(false);
    let mut session = registry
        .session("files", vec![], ProductionLimits::default())
        .unwrap();
    session.submit(SVG, &cancel).unwrap();
    let bytes = session.candidate().unwrap().bundle();
    assert!(registry
        .reopen(&bytes[..128], ProductionLimits::default(), &cancel)
        .is_err());
    let reader = BundleReader::parse(bytes).unwrap();
    let mut writer = BundleWriter::new();
    for entry in reader.entries() {
        let payload = if entry.key.starts_with("outputs/") {
            b"forged".to_vec()
        } else {
            reader.get(&entry.key).unwrap().to_vec()
        };
        writer
            .add_file(&entry.key, &entry.kind, payload, None)
            .unwrap();
    }
    assert!(registry
        .reopen(
            &writer.build().unwrap(),
            ProductionLimits::default(),
            &cancel
        )
        .is_err());
}

#[test]
fn source_resource_and_package_budgets_are_enforced_before_promotion() {
    let registry = ProducerRegistry::with_builtins();
    let cancel = AtomicBool::new(false);
    for limits in [
        ProductionLimits {
            resource_bytes: 1,
            ..ProductionLimits::default()
        },
        ProductionLimits {
            resources: 0,
            ..ProductionLimits::default()
        },
        ProductionLimits {
            package_bytes: 512,
            ..ProductionLimits::default()
        },
        ProductionLimits {
            source: vibe::session::SessionLimits {
                submission_bytes: 10,
                ..Default::default()
            },
            ..ProductionLimits::default()
        },
        ProductionLimits {
            evaluation: vibe::Budget::from_steps(0),
            ..ProductionLimits::default()
        },
    ] {
        let mut session = registry.session("files", vec![], limits).unwrap();
        assert!(session.submit(SVG, &cancel).is_err());
        assert_eq!(session.revision(), 0);
        assert!(session.candidate().is_none());
    }
    let mut session = registry
        .session("files", vec![], ProductionLimits::default())
        .unwrap();
    assert!(session
        .submit("let helper = 31;", &cancel)
        .unwrap()
        .is_none());
    session
        .submit(
            r#"let asset = { files: [{ key: "value.bin", kind: "bin", bytes: [helper] }] };"#,
            &cancel,
        )
        .unwrap();
    assert_eq!(session.candidate().unwrap().outputs()[0].bytes, [31]);
    registry
        .reopen(
            session.candidate().unwrap().bundle(),
            ProductionLimits::default(),
            &cancel,
        )
        .unwrap();
    assert!(registry
        .session("unknown", vec![], ProductionLimits::default())
        .is_err());
}

#[test]
fn adaptive_source_budget_accepts_structured_large_source_and_replays_exactly() {
    let registry = ProducerRegistry::with_builtins();
    let cancel = AtomicBool::new(false);
    let source = format!("let data = [{}]; let asset = {{files:[{{key:\"value.bin\",kind:\"bin\",bytes:data}}]}};", vec!["0"; 34_000].join(","));
    assert!(source.len() > 64 * 1024);
    let mut session = registry.session("files", vec![], ProductionLimits::default()).unwrap();
    session.submit(&source, &cancel).unwrap();
    assert_eq!(session.candidate().unwrap().outputs()[0].bytes, vec![0; 34_000]);
    let bundle = session.candidate().unwrap().bundle().to_vec();
    let reopened = registry.reopen(&bundle, ProductionLimits::default(), &cancel).unwrap();
    assert_eq!(reopened.sources(), &[source]);
    assert_eq!(reopened.candidate().unwrap().bundle(), bundle);
}

#[test]
fn adaptive_source_budget_accounts_for_inputs_history_and_overflow() {
    let mut limits = ProductionLimits::default();
    limits.total_resource_bytes = 1024 * 1024;
    let input = Resource { key:"input.bin".into(), kind:"bin".into(), bytes:vec![0; 256 * 1024] };
    let empty = limits.for_retained_inputs(&[]).unwrap();
    let with_input = limits.for_retained_inputs(&[input.clone()]).unwrap();
    assert_eq!(empty.source.submission_bytes, 8192);
    assert_eq!(with_input.source.submission_bytes, 6144);
    let registry = ProducerRegistry::with_builtins();
    let mut session = registry.session("files", vec![input], limits).unwrap();
    let cancel = AtomicBool::new(false);
    session.submit("let helper = 1;", &cancel).unwrap();
    let source = "x".repeat(with_input.source.retained_source_bytes);
    assert!(matches!(session.submit(&source, &cancel), Err(e) if e.contains("retained history")));
    let mut overflow = ProductionLimits::default();
    overflow.total_resource_bytes = usize::MAX;
    assert!(overflow.for_retained_inputs(&[]).is_err());
}

#[test]
fn small_source_keeps_more_than_22_mib_of_default_resource_headroom() {
    let inputs = (0..2).map(|i| Resource { key:format!("input{i}.bin"), kind:"bin".into(), bytes:vec![7; 12*1024*1024] }).collect();
    let registry = ProducerRegistry::with_builtins();
    let cancel = AtomicBool::new(false);
    let mut session = registry.session("files", inputs, ProductionLimits::default()).unwrap();
    session.submit(SVG, &cancel).unwrap();
    assert!(session.candidate().is_some());
}

#[test]
fn explicit_source_cap_and_rejected_edit_preserve_candidate() {
    let registry = ProducerRegistry::with_builtins();
    let cancel = AtomicBool::new(false);
    let mut limits = ProductionLimits::default();
    limits.source.submission_bytes = 64 * 1024;
    let mut session = registry.session("files", vec![], limits).unwrap();
    assert_eq!(session.limits().source.submission_bytes, 64 * 1024);
    session.submit("let helper = 31;", &cancel).unwrap();
    session.submit(r#"let asset = { files: [{ key: "value.bin", kind: "bin", bytes: [helper] }] };"#, &cancel).unwrap();
    let old = session.candidate().unwrap().sha256();
    let old_sources = session.sources().to_vec();
    assert!(session.prepare_edit(&"x".repeat(64 * 1024 + 1), &cancel).is_err());
    assert_eq!(session.candidate().unwrap().sha256(), old);
    assert_eq!(session.sources(), old_sources);
}

#[test]
fn retained_image_workspace_above_22_mib_fits_and_smaller_budget_refuses() {
    let mut source = String::from("requires [capability(\"capability.invoke\")];\n");
    for i in 0..6 {
        source.push_str(&format!("let image{i}=capability.invoke(\"hypermedia.radiance.create_rgba\",{{width:1024,height:1024,color:[1,2,3,255]}});\n"));
    }
    source.push_str("let pixel=capability.invoke(\"hypermedia.radiance.create_rgba\",{width:1,height:1,color:[1,2,3,255]}); let png=capability.invoke(\"hypermedia.radiance.encode_rgba_png\",{image:pixel}); let asset={files:[{key:\"pixel.png\",resource:png}]};");
    let registry = ProducerRegistry::with_builtins();
    let cancel = AtomicBool::new(false);
    let mut enough = registry.session("files", vec![], ProductionLimits::default()).unwrap();
    enough.submit(&source, &cancel).unwrap();
    assert!(enough.candidate().is_some());
    let mut small = ProductionLimits::default();
    small.total_resource_bytes = 20 * 1024 * 1024;
    let mut refused = registry.session("files", vec![], small).unwrap();
    assert!(refused.submit(&source, &cancel).is_err());
    assert!(refused.candidate().is_none());
}

#[test]
fn explicitly_fixed_evaluator_steps_are_not_expanded() {
    let mut limits = ProductionLimits::default();
    limits.evaluation = vibe::Budget::from_steps(100_000);
    assert_eq!(limits.for_source_usage(100_000).unwrap().evaluation.steps_left, 100_000);
    assert_eq!(ProductionLimits::default().for_source_usage(100_000).unwrap().evaluation.steps_left, 400_000);
}

#[test]
fn rejected_cumulative_edit_never_replays_producer() {
    use std::sync::atomic::AtomicUsize;
    struct CountingProducer(Arc<AtomicUsize>);
    impl AssetProducer for CountingProducer {
        fn id(&self) -> &str { "counting" }
        fn version(&self) -> &str { "1" }
        fn produce(&self, asset: &AssetValue, context: &ProductionContext<'_>) -> Result<Vec<Resource>, String> {
            self.0.fetch_add(1, Ordering::SeqCst);
            NumberProducer { version: "1", bias: 0 }.produce(asset, context)
        }
    }
    let calls = Arc::new(AtomicUsize::new(0));
    let mut registry = ProducerRegistry::default();
    registry.register(CountingProducer(calls.clone())).unwrap();
    let mut limits = ProductionLimits::default();
    limits.source.submission_bytes = 128;
    limits.source.retained_source_bytes = 96;
    let cancel = AtomicBool::new(false);
    let input = Resource { key: "input.bin".into(), kind: "bin".into(), bytes: vec![1, 2, 3] };
    let mut session = registry.session("counting", vec![input], limits).unwrap();
    session.submit("let helper = 31;", &cancel).unwrap();
    session.submit("let mut asset = helper;", &cancel).unwrap();
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    let old_hash = session.candidate().unwrap().sha256();
    let old_sources = session.sources().to_vec();
    let old_revision = session.revision();
    let history: usize = old_sources.iter().map(String::len).sum();
    let too_much = " ".repeat(96 - history + 1);
    assert!(too_much.len() < session.limits().source.submission_bytes);
    assert!(matches!(session.prepare_edit(&too_much, &cancel), Err(e) if e.contains("retained history")));
    assert_eq!(calls.load(Ordering::SeqCst), 1, "refused edit replayed the producer");
    assert_eq!(session.candidate().unwrap().sha256(), old_hash);
    assert_eq!(session.sources(), old_sources);
    assert_eq!(session.revision(), old_revision);
    cancel.store(true, Ordering::Release);
    assert!(matches!(session.prepare_edit("asset = 32;", &cancel), Err(e) if e.contains("cancelled")));
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    cancel.store(false, Ordering::Release);
    let edited = session.prepare_edit("asset = 32;", &cancel).unwrap();
    assert_eq!(calls.load(Ordering::SeqCst), 3, "accepted edit must replay and produce");
    assert_eq!(edited.candidate().unwrap().outputs()[0].bytes, b"32");
    assert_eq!(session.candidate().unwrap().sha256(), old_hash);
}
