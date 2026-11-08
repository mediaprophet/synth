//! External-consumer test: uses only Synth's public general-purpose API.
use std::sync::atomic::AtomicBool;
use synth::production::{
    AssetProducer, AssetValue, ProducerRegistry, ProductionContext, ProductionLimits, Resource,
};

struct RasterProducer;
impl AssetProducer for RasterProducer {
    fn id(&self) -> &str {
        "test-raster"
    }
    fn version(&self) -> &str {
        "1"
    }
    fn produce(
        &self,
        asset: &AssetValue,
        context: &ProductionContext<'_>,
    ) -> Result<Vec<Resource>, String> {
        context.check_cancelled()?;
        let AssetValue::Record(fields) = asset else {
            return Err("Expected a raster record".into());
        };
        let integer = |key: &str| -> Result<usize, String> {
            match fields.get(key) {
                Some(AssetValue::I64(v)) => {
                    usize::try_from(*v).map_err(|_| "Negative raster value".into())
                }
                Some(AssetValue::U64(v)) => {
                    usize::try_from(*v).map_err(|_| "Raster value overflow".into())
                }
                _ => Err(format!("Missing integer {key}")),
            }
        };
        let (width, height, red) = (integer("width")?, integer("height")?, integer("red")?);
        if width == 0 || height == 0 || width > 256 || height > 256 || red > 255 {
            return Err("Raster dimensions/channel outside test producer limits".into());
        }
        let mut bytes = format!("P6\n{width} {height}\n255\n").into_bytes();
        if bytes.len() + width * height * 3 > context.limits.resource_bytes {
            return Err("Raster exceeds output budget".into());
        }
        for _ in 0..width * height {
            bytes.extend_from_slice(&[red as u8, 100, 200]);
        }
        Ok(vec![Resource {
            key: "texture.ppm".into(),
            kind: "ppm".into(),
            bytes,
        }])
    }
}

#[test]
fn unrelated_producer_generates_edits_packages_and_reopens_through_public_api() {
    let mut registry = ProducerRegistry::with_builtins();
    registry.register(RasterProducer).unwrap();
    let cancel = AtomicBool::new(false);
    let mut session = registry
        .session("test-raster", vec![], ProductionLimits::default())
        .unwrap();
    session.submit(r#"import "vibe:0.3/nquin" as nquin;
        let mut asset = { width: 8, height: 4, red: nquin.wire.compute_wire_parity([1, 2, 4, 8, 16]) };"#, &cancel).unwrap();
    let before = session.document_digest().unwrap();
    assert!(session
        .prepare_edit("asset = { width: -1, height: 4, red: 31 };", &cancel)
        .is_err());
    let edited = session
        .prepare_edit("asset = { width: 16, height: 8, red: 63 };", &cancel)
        .unwrap();
    assert_eq!(session.document_digest().unwrap(), before);
    assert_ne!(edited.document_digest().unwrap(), before);
    let candidate = edited.candidate().unwrap();
    assert_eq!(
        candidate.outputs()[0].bytes.len(),
        b"P6\n16 8\n255\n".len() + 16 * 8 * 3
    );
    assert!(candidate.outputs()[0].bytes.ends_with(&[63, 100, 200]));
    let reopened = registry
        .reopen(candidate.bundle(), ProductionLimits::default(), &cancel)
        .unwrap();
    assert_eq!(
        reopened.document_digest().unwrap(),
        edited.document_digest().unwrap()
    );
    assert_eq!(reopened.candidate().unwrap().bundle(), candidate.bundle());
}

#[test]
fn unrelated_file_formats_share_the_same_pipeline() {
    let registry = ProducerRegistry::with_builtins();
    let cancel = AtomicBool::new(false);
    let mut session = registry
        .session("files", vec![], ProductionLimits::default())
        .unwrap();
    session
        .submit(include_str!("../examples/assets.vibe"), &cancel)
        .unwrap();
    let outputs = session.candidate().unwrap().outputs();
    assert_eq!(outputs.len(), 3);
    assert!(outputs[0].bytes.starts_with(b"<svg"));
    let palette: serde_json::Value = serde_json::from_slice(&outputs[1].bytes).unwrap();
    assert_eq!(palette["foreground"], "#8ce6ba");
    assert_eq!(outputs[2].bytes, [0, 31, 127, 255]);
    registry
        .reopen(
            session.candidate().unwrap().bundle(),
            ProductionLimits::default(),
            &cancel,
        )
        .unwrap();
}
