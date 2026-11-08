use super::*;
use qualia_core_db::bundle::{BundleReader, BundleWriter};
const SVG: &str = r##"import "vibe:0.3/render" as render;
let radius = render.spatial.skeletal_lerp_joint([8.0, 0.0, 0.0], [12.0, 0.0, 0.0], 0.5)[0];
let mut asset = { files: [{ key: "icons/dot.svg", kind: "svg", text: "<svg xmlns=\"http://www.w3.org/2000/svg\"><circle r=\"10\"/></svg>" }] };"##;

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
