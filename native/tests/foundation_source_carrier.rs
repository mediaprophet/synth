//! Public native consumer regressions. Synthetic data, no GPU or human acceptance.
#[path = "foundation_source_carrier/restart.rs"]
mod source_carrier_restart;
use poet_cli::graphical::{
    authoring::AuthoringEdit,
    checkpoints::{
        CheckpointEditorHost, CheckpointLimits, CheckpointService, ExpectedFile, TrustedBinding,
    },
    controller::{CanonicalEditorHost, EditorHost, PreviewProvider},
    geometry_host::GeometryHost,
    preview::{PreviewFrame, PreviewLimits, SourceRef},
    surface_host::SurfacePackageRequest,
};
use qualia_core_db::{
    bundle::{BundleReader, BundleWriter},
    poet_host::invoke::studio_store::{self, checkpoint as core, StudioArtifact},
};
use sha2::{Digest, Sha256};
use std::{
    path::Path,
    sync::atomic::{AtomicU64, Ordering},
};
use vibe::{session::SessionLimits, Value};

const SOURCE: &str = r#"
let mesh={vertices:[[0.0,0.0,0.0],[1.0,0.0,0.0],[0.0,1.0,0.0]],triangles:[[0,1,2]]};
let mat={id:1,colour_space:"linear",base_color:[0.3,0.4,0.5,1.0],emissive:[0.0,0.0,0.0],metallic:0.1,roughness:0.5,specular:1.0,alpha_cutoff:0.5,opacity_mode:"Opaque",double_sided:true};
let mut poet_parts=[{id:"body",semantic_id:101,geometry:mesh,material:mat,translation:[0.0,0.0,0.0],rotation:[0.0,0.0,0.0],scale:[1.0,1.0,1.0]}];
({parts:poet_parts});
"#;
struct Admission;
impl PreviewProvider for Admission {
    fn validate(&self, value: &Value) -> Result<(), String> {
        if poet_cli::graphical::assembly::is_assembly(value) {
            poet_cli::graphical::assembly::admit_assembly(value, PreviewLimits::default())
                .map(|_| ())
        } else {
            Ok(())
        }
    }
    fn preview(&mut self, _: SourceRef, _: &Value) -> Result<PreviewFrame, String> {
        Err("Public source-carrier case deliberately requests no GPU".into())
    }
}
fn id() -> String {
    static NEXT: AtomicU64 = AtomicU64::new(1);
    format!(
        "urn:poet:public-carrier:{}:{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    )
}
fn binding(id: &str) -> TrustedBinding {
    TrustedBinding {
        project_id: "urn:synthetic:carrier-project".into(),
        document_id: "urn:synthetic:carrier-document".into(),
        source_id: id.into(),
        domain: "Geometry".into(),
    }
}
fn host(id: &str, root: &Path) -> CheckpointEditorHost {
    let canonical = CanonicalEditorHost::open(
        id.into(),
        SessionLimits::default(),
        GeometryHost::default(),
        Box::new(Admission),
    )
    .unwrap();
    CheckpointEditorHost::new(
        Box::new(canonical),
        CheckpointService::new(root, binding(id), CheckpointLimits::default()).unwrap(),
    )
    .unwrap()
}
fn create(root: &Path) -> (String, CheckpointEditorHost) {
    let id = id();
    studio_store::create_artifact_if_absent(StudioArtifact::new(
        &id,
        "Geometry",
        "Synthetic public carrier",
    ))
    .unwrap();
    let mut h = host(&id, root);
    let source = h.snapshot().unwrap().source;
    h.submit(SOURCE, source).unwrap();
    (id, h)
}
fn input_resource() -> core::ResourceData {
    core::ResourceData {
        key: "original/input.bin".into(),
        kind: "future/opaque+v9".into(),
        bytes: vec![0, 42, 255],
        meta: Some(vec![0xa1, 0x61, b'x', 1]),
    }
}
fn package(id: &str, stamp: &studio_store::StoreStamp, meta: Option<Vec<u8>>) -> Vec<u8> {
    let current = core::capture(id, stamp, Default::default())
        .unwrap()
        .apply_resource_edits(
            vec![core::ResourceEdit::Add(input_resource())],
            Default::default(),
        )
        .unwrap();
    let bytes = core::encode(&current, vec![], Default::default()).unwrap();
    let reader = BundleReader::parse(&bytes).unwrap();
    let mut writer = BundleWriter::new();
    for entry in reader.entries() {
        let mut payload = reader.get(&entry.key).unwrap().to_vec();
        if entry.key == core::SOURCE_RESOURCE_KEY {
            // Exact noncanonical JSON whitespace is intentional signature context.
            payload.insert(0, b' ');
            payload.push(b'\n');
        }
        writer
            .add_file(
                &entry.key,
                &entry.kind,
                payload,
                if entry.key == core::SOURCE_RESOURCE_KEY {
                    meta.clone()
                } else {
                    entry.meta.clone()
                },
            )
            .unwrap();
    }
    writer.build().unwrap()
}
fn assert_origin(bytes: &[u8], original: &[u8], expected: &core::Checkpoint) {
    let reader = BundleReader::parse(bytes).unwrap();
    let carrier = core::decode_source_carrier(original, Default::default()).unwrap();
    let archive = reader
        .entry(&carrier.origin_key())
        .expect("exact original archive");
    let digest: [u8; 32] = Sha256::digest(original).into();
    assert_eq!(carrier.origin_digest(), digest);
    assert_eq!(archive.kind, "hmc");
    assert_eq!(archive.meta, None);
    assert_eq!(reader.get(&carrier.origin_key()), Some(original));
    assert_eq!(reader.entry(core::SOURCE_RESOURCE_KEY).unwrap().meta, None);
    let current = core::encode(expected, vec![], Default::default()).unwrap();
    assert_eq!(
        reader.get(core::SOURCE_RESOURCE_KEY),
        BundleReader::parse(&current)
            .unwrap()
            .get(core::SOURCE_RESOURCE_KEY),
        "current source, full journal and parent must remain exact"
    );
    let old = BundleReader::parse(original).unwrap();
    assert_eq!(
        reader.get("original/input.bin"),
        old.get("original/input.bin")
    );
    assert_eq!(
        reader.entry("original/input.bin").unwrap().meta,
        input_resource().meta
    );
}
fn expected(id: &str, source: &SourceRef, parent: [u8; 32]) -> core::Checkpoint {
    core::capture(id, &source.stamp, Default::default())
        .unwrap()
        .with_parent_digest(parent)
}

#[test]
fn native_bound_metadata_survives_edit_rebuild_compile_replacement_and_save_as() {
    let root = tempfile::tempdir().unwrap();
    let (id, mut h) = create(root.path());
    let initial = h.snapshot().unwrap();
    let original = package(&id, &initial.source.stamp, Some(vec![0xa1, 0x61, b's', 1]));
    let input = core::save(
        &root.path().join("bound.hmc"),
        &original,
        ExpectedFile::Absent,
        Default::default(),
    )
    .unwrap();
    h.bind_surface_package(
        SurfacePackageRequest {
            filename: "bound.hmc".into(),
            digest: input.digest,
        },
        initial.source,
    )
    .unwrap();
    let source = h.snapshot().unwrap().source;
    let moved = h
        .authoring_edit(
            AuthoringEdit::Transform {
                id: "body".into(),
                translation: [2.0, 0.0, 0.0],
                rotation: [0.0; 3],
                scale: [1.0; 3],
            },
            source,
        )
        .unwrap()
        .snapshot;
    let source = h.rebuild_execution(moved.source).unwrap().snapshot.source;
    assert_eq!(
        h.authoring_view(source.clone()).unwrap().parts[0].translation[0],
        2.0
    );
    let saved = h
        .compile_checkpoint(
            "derived.hmc",
            "model-v1",
            ExpectedFile::Absent,
            source.clone(),
        )
        .unwrap();
    let first = std::fs::read(&saved.path).unwrap();
    assert_origin(&first, &original, &expected(&id, &source, input.digest));
    assert!(BundleReader::parse(&first)
        .unwrap()
        .entries()
        .iter()
        .any(|e| e.kind == "10d"));
    let changed = h
        .authoring_edit(
            AuthoringEdit::Material {
                material_id: 1,
                base_color: [0.2, 0.3, 0.4, 1.0],
                roughness: 0.8,
                metallic: 0.0,
            },
            source,
        )
        .unwrap()
        .snapshot;
    let source = changed.source;
    assert!(h
        .compile_checkpoint(
            "derived.hmc",
            "model-v2",
            ExpectedFile::Sha256([0; 32]),
            source.clone()
        )
        .is_err());
    assert_eq!(
        std::fs::read(&saved.path).unwrap(),
        first,
        "stale file preimage cannot write"
    );
    let replaced = h
        .compile_checkpoint(
            "derived.hmc",
            "model-v2",
            ExpectedFile::Sha256(saved.digest),
            source.clone(),
        )
        .unwrap();
    let second = std::fs::read(&replaced.path).unwrap();
    assert_origin(&second, &original, &expected(&id, &source, saved.digest));
    let before = BundleReader::parse(&first).unwrap();
    let after = BundleReader::parse(&second).unwrap();
    for entry in before
        .entries()
        .iter()
        .filter(|e| e.key != core::SOURCE_RESOURCE_KEY)
    {
        assert_eq!(after.get(&entry.key), before.get(&entry.key));
        assert_eq!(after.entry(&entry.key).unwrap().meta, entry.meta);
    }
    let copy = h
        .save_checkpoint("copy.hmc", ExpectedFile::Absent, source.clone())
        .unwrap();
    let copied = std::fs::read(&copy.path).unwrap();
    assert_origin(&copied, &original, &expected(&id, &source, replaced.digest));
    assert_eq!(
        BundleReader::parse(&copied)
            .unwrap()
            .entries()
            .iter()
            .filter(|e| e.key.starts_with("authoring/origins/"))
            .count(),
        1,
        "no recursive source archive wrapping"
    );
    drop(h);
    // Fresh service and interpreter from retained accepted source, held until explicit rebuild.
    let mut reopened = host(&id, root.path());
    let held = reopened.snapshot().unwrap();
    assert!(held.reconciliation_required);
    reopened
        .bind_surface_package(
            SurfacePackageRequest {
                filename: "copy.hmc".into(),
                digest: copy.digest,
            },
            held.source.clone(),
        )
        .unwrap();
    let source = reopened
        .rebuild_execution(held.source)
        .unwrap()
        .snapshot
        .source;
    assert_eq!(
        reopened.authoring_view(source.clone()).unwrap().parts[0].roughness,
        0.8
    );
    let fresh = reopened
        .compile_checkpoint(
            "fresh-reopen.hmc",
            "model-v3",
            ExpectedFile::Absent,
            source.clone(),
        )
        .unwrap();
    assert_origin(
        &std::fs::read(fresh.path).unwrap(),
        &original,
        &expected(&id, &source, copy.digest),
    );
    assert_eq!(
        std::fs::read(input.path).unwrap(),
        original,
        "original source package untouched"
    );
}

#[test]
fn public_writer_legacy_helper_foreign_scope_and_budgets_fail_closed() {
    let root = tempfile::tempdir().unwrap();
    let (id, h) = create(root.path());
    let source = h.snapshot().unwrap().source;
    let original = package(&id, &source.stamp, Some(vec![0xa0]));
    let old = core::decode(&original, Default::default()).unwrap();
    let carrier = core::decode_source_carrier(&original, Default::default()).unwrap();
    let current = core::capture(&id, &source.stamp, Default::default()).unwrap();
    assert!(core::encode(&old, vec![], Default::default()).is_err());
    assert!(core::encode(
        &old.clone().with_parent_digest([7; 32]),
        vec![],
        Default::default()
    )
    .is_err());
    assert!(current
        .clone()
        .with_resources_from(&old, Default::default())
        .is_err());
    assert!(old
        .clone()
        .with_resources_from(&current, Default::default())
        .is_err());
    let (foreign_id, foreign) = create(root.path());
    let foreign_source = foreign.snapshot().unwrap().source;
    let bytes = package(&foreign_id, &foreign_source.stamp, None);
    let foreign = core::decode_source_carrier(&bytes, Default::default()).unwrap();
    assert!(current
        .clone()
        .with_source_carrier(&foreign, Default::default())
        .is_err());
    for limits in [
        CheckpointLimits {
            entries: 2,
            ..Default::default()
        },
        CheckpointLimits {
            source_bytes: 1,
            ..Default::default()
        },
        CheckpointLimits {
            file_bytes: original.len(),
            ..Default::default()
        },
    ] {
        assert!(current
            .clone()
            .with_source_carrier(&carrier, limits)
            .is_err());
    }
    assert!(core::decode_source_carrier(
        &original,
        CheckpointLimits {
            file_bytes: original.len() - 1,
            ..Default::default()
        }
    )
    .is_err());
    let mut flagged = original.clone();
    // Canonical format.rs documents flags at 6 and CRC at 36. Repair checksum so
    // this vector proves the flags policy, rather than stopping at a bad CRC.
    flagged[6..8].copy_from_slice(&1u16.to_le_bytes());
    flagged[36..40].fill(0);
    let crc = qualia_core_db::container_10d::crc32c(&flagged);
    flagged[36..40].copy_from_slice(&crc.to_le_bytes());
    assert_eq!(BundleReader::parse(&flagged).unwrap().flags(), 1);
    assert!(core::decode_source_carrier(&flagged, Default::default())
        .err()
        .expect("unsupported flags")
        .contains("flags"));
    assert_eq!(
        core::encode(&current, vec![], Default::default()).unwrap(),
        core::encode(
            &core::capture(&id, &source.stamp, Default::default()).unwrap(),
            vec![],
            Default::default()
        )
        .unwrap()
    );
}

#[test]
fn public_resource_conflicts_and_protected_origin_edits_preserve_original() {
    let root = tempfile::tempdir().unwrap();
    let (id, h) = create(root.path());
    let source = h.snapshot().unwrap().source;
    let original = package(&id, &source.stamp, Some(vec![0xa0]));
    let carrier = core::decode_source_carrier(&original, Default::default()).unwrap();
    let current = core::capture(&id, &source.stamp, Default::default()).unwrap();
    for variant in ["payload", "kind", "metadata", "absence"] {
        let mut resource = input_resource();
        match variant {
            "payload" => resource.bytes.push(1),
            "kind" => resource.kind = "future/changed".into(),
            "metadata" => resource.meta = Some(vec![0x80]),
            _ => resource.meta = None,
        }
        let conflict = current
            .clone()
            .apply_resource_edits(vec![core::ResourceEdit::Add(resource)], Default::default())
            .unwrap();
        let before = core::encode(&conflict, vec![], Default::default()).unwrap();
        assert!(conflict
            .clone()
            .with_source_carrier(&carrier, Default::default())
            .is_err());
        assert_eq!(
            core::encode(&conflict, vec![], Default::default()).unwrap(),
            before
        );
    }
    let merged = current
        .with_source_carrier(&carrier, Default::default())
        .unwrap();
    let before = core::encode(&merged, vec![], Default::default()).unwrap();
    for edit in [
        core::ResourceEdit::Remove {
            key: carrier.origin_key(),
            expected: carrier.origin_digest(),
        },
        core::ResourceEdit::ReplaceBytes {
            key: carrier.origin_key(),
            expected: carrier.origin_digest(),
            bytes: original.clone(),
        },
        core::ResourceEdit::Add(core::ResourceData {
            key: carrier.origin_key(),
            kind: "hmc".into(),
            bytes: original.clone(),
            meta: None,
        }),
    ] {
        assert!(merged
            .apply_resource_edits(vec![edit], Default::default())
            .is_err());
    }
    assert_eq!(
        core::encode(&merged, vec![], Default::default()).unwrap(),
        before
    );
    let reader = BundleReader::parse(&before).unwrap();
    for variant in ["payload", "kind", "metadata", "key"] {
        let mut writer = BundleWriter::new();
        for entry in reader.entries() {
            let mut payload = reader.get(&entry.key).unwrap().to_vec();
            let mut key = entry.key.clone();
            let mut kind = entry.kind.clone();
            let mut meta = entry.meta.clone();
            if key == carrier.origin_key() {
                match variant {
                    "payload" => payload.push(1),
                    "kind" => kind = "other".into(),
                    "metadata" => meta = Some(vec![0xa0]),
                    _ => key = key.to_uppercase(),
                }
            }
            writer.add_file(key, kind, payload, meta).unwrap();
        }
        assert!(
            core::decode_source_carrier(&writer.build().unwrap(), Default::default()).is_err(),
            "{variant}"
        );
    }
}
