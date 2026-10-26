//! Actual canonical mixed-resource fixtures; no master-quality/completeness claim.
use super::*;
use qualia_core_db::bundle::{BundleReader, BundleWriter};
use qualia_core_db::render::compile_10d::CompiledAsset;

fn binding(id: &str) -> TrustedBinding {
    TrustedBinding {
        project_id: "fixture-project".into(),
        document_id: id.into(),
        source_id: id.into(),
        domain: "Geometry".into(),
    }
}
fn actual_asset() -> CompiledAsset {
    qualia_core_db::render::compile_10d::compile_asset(
        b"v 0 0 0\nv 1 0 0\nv 0 1 0\nf 1 2 3\n",
        Some("obj"),
        "urn:fixture:retained-derived",
        "obj",
    )
    .unwrap()
}
fn mixed(root: &Path, id: &str) -> (CheckpointService, StoreStamp, SaveReceipt, Vec<u8>) {
    let (_, stamp) = studio_store::create_artifact_if_absent(StudioArtifact::new(
        id,
        "Geometry",
        "Resource preservation fixture",
    ))
    .unwrap();
    let checkpoint = core::capture(id, &stamp, Default::default()).unwrap();
    let encoded = core::encode(
        &checkpoint,
        vec![("derived/geometry.10d".into(), actual_asset())],
        Default::default(),
    )
    .unwrap();
    let reader = BundleReader::parse(&encoded).unwrap();
    let mut writer = BundleWriter::new();
    for entry in reader.entries() {
        writer
            .add_file(
                &entry.key,
                &entry.kind,
                reader.get(&entry.key).unwrap().to_vec(),
                entry.meta.clone(),
            )
            .unwrap();
    }
    for (key, kind, bytes) in [
        (
            "masters/editable.obj",
            "text/plain",
            b"# exact editable source fixture\nv 1 2 3\n".as_slice(),
        ),
        (
            "rigs/editable.json",
            "application/json",
            b"{\"joints\":[],\"fixture\":true}".as_slice(),
        ),
        ("textures/raw.bytes", "future/texture", &[0, 42, 255]),
        (
            "unknown/evidence.bin",
            "future/evidence+v9",
            b"unreviewed source evidence".as_slice(),
        ),
    ] {
        writer
            .add_file(key, kind, bytes.to_vec(), Some(vec![0xa1, 0x61, b'x', 1]))
            .unwrap();
    }
    let bytes = writer.build().unwrap();
    let receipt = core::save(
        &root.join("mixed.hmc"),
        &bytes,
        ExpectedFile::Absent,
        Default::default(),
    )
    .unwrap();
    let service = CheckpointService::new(root, binding(id), Default::default()).unwrap();
    (service, stamp, receipt, bytes)
}

#[test]
fn source_material_save_retains_geometry_q42_lexicon_masters_rigs_unknown_bytes_and_metadata() {
    let root = tempfile::tempdir().unwrap();
    let id = "urn:poet:resource-preservation-save";
    let (service, stamp, saved, before) = mixed(root.path(), id);
    let old = service.inventory("mixed.hmc").unwrap();
    let (_, stamp) = studio_store::mutate_artifact_stamped(id, &stamp, |artifact| {
        artifact
            .nodes
            .entry("poet.native.submission/00000000000000000001".into())
            .or_default()
            .insert(
                "text".into(),
                "let poet_material = {roughness:0.65,metallic:0.1};\n".into(),
            );
        Ok(())
    })
    .unwrap();
    let saved = service
        .save(
            &stamp,
            "mixed.hmc",
            ExpectedFile::Sha256(saved.digest),
            Some(saved.digest),
            vec![],
        )
        .unwrap();
    let after = std::fs::read(&saved.receipt.path).unwrap();
    let new = service.inventory("mixed.hmc").unwrap();
    assert_eq!(old.resources, new.resources);
    assert_ne!(old.digest, new.digest);
    let original = BundleReader::parse(&before).unwrap();
    let retained = BundleReader::parse(&after).unwrap();
    for entry in original
        .entries()
        .iter()
        .filter(|entry| entry.key != "authoring/native-source.json")
    {
        let current = retained.entry(&entry.key).unwrap();
        assert_eq!(entry.sha256, current.sha256);
        assert_eq!(entry.meta, current.meta);
        assert_eq!(entry.kind, current.kind);
        assert_eq!(original.get(&entry.key), retained.get(&entry.key));
    }
    assert!(service
        .save(
            &stamp,
            "mixed.hmc",
            ExpectedFile::Sha256(new.digest),
            Some(new.digest),
            vec![("derived/geometry.10d".into(), actual_asset())]
        )
        .is_err());
    assert_eq!(std::fs::read(&saved.receipt.path).unwrap(), after);
}

#[test]
fn guarded_inventory_add_replace_remove_and_failed_edits_preserve_work() {
    let root = tempfile::tempdir().unwrap();
    let (service, stamp, saved, before) = mixed(root.path(), "urn:poet:resource-edits");
    let inventory = service.inventory("mixed.hmc").unwrap();
    let selected = inventory
        .resources
        .iter()
        .find(|entry| entry.key == "masters/editable.obj")
        .unwrap();
    for edits in [
        vec![ResourceEdit::Remove {
            key: selected.key.clone(),
            expected: [0; 32],
        }],
        vec![ResourceEdit::Remove {
            key: "authoring/native-source.json".into(),
            expected: [0; 32],
        }],
        vec![ResourceEdit::Add(ResourceData {
            key: "invalid.q42".into(),
            kind: "q42".into(),
            bytes: vec![0; 20],
            meta: None,
        })],
    ] {
        assert!(service
            .save_with_resource_edits(
                &stamp,
                "mixed.hmc",
                ExpectedFile::Sha256(saved.digest),
                Some(saved.digest),
                vec![],
                edits
            )
            .is_err());
        assert_eq!(std::fs::read(&saved.path).unwrap(), before);
    }
    assert!(service
        .save_with_resource_edits(
            &stamp,
            "new.hmc",
            ExpectedFile::Absent,
            None,
            vec![],
            vec![ResourceEdit::Remove {
                key: selected.key.clone(),
                expected: selected.digest
            }]
        )
        .is_err());
    assert!(service
        .save(
            &stamp,
            "mixed.hmc",
            ExpectedFile::Sha256([0; 32]),
            None,
            vec![]
        )
        .is_err());
    let remove = inventory
        .resources
        .iter()
        .find(|entry| entry.key == "rigs/editable.json")
        .unwrap();
    let edited = service
        .save_with_resource_edits(
            &stamp,
            "mixed.hmc",
            ExpectedFile::Sha256(saved.digest),
            Some(saved.digest),
            vec![],
            vec![
                ResourceEdit::ReplaceBytes {
                    key: selected.key.clone(),
                    expected: selected.digest,
                    bytes: b"# explicitly edited source\n".to_vec(),
                },
                ResourceEdit::Remove {
                    key: remove.key.clone(),
                    expected: remove.digest,
                },
                ResourceEdit::Add(ResourceData {
                    key: "evidence/added.txt".into(),
                    kind: "text/plain".into(),
                    bytes: b"explicit unreviewed addition".to_vec(),
                    meta: None,
                }),
            ],
        )
        .unwrap();
    let current = service.inventory("mixed.hmc").unwrap();
    assert_eq!(current.digest, edited.receipt.digest);
    let replaced = current
        .resources
        .iter()
        .find(|entry| entry.key == selected.key)
        .unwrap();
    assert_eq!(replaced.kind, selected.kind);
    assert_eq!(replaced.metadata_digest, selected.metadata_digest);
    assert_ne!(replaced.digest, selected.digest);
    assert!(!current
        .resources
        .iter()
        .any(|entry| entry.key == remove.key));
    for entry in inventory
        .resources
        .iter()
        .filter(|entry| entry.key != selected.key && entry.key != remove.key)
    {
        assert!(current.resources.contains(entry));
    }
    assert!(service
        .save_with_resource_edits(
            &stamp,
            "mixed.hmc",
            ExpectedFile::Sha256(current.digest),
            Some(current.digest),
            vec![],
            vec![ResourceEdit::Remove {
                key: selected.key.clone(),
                expected: selected.digest
            }]
        )
        .is_err());
    assert_eq!(service.inventory("mixed.hmc").unwrap(), current);
}

#[test]
fn immutable_host_release_derives_complete_working_copy_with_exact_origin_lineage() {
    let root = tempfile::tempdir().unwrap();
    let id = "urn:poet:immutable-resource-origin";
    let (_, stamp, saved, original) = mixed(root.path(), id);
    let service = CheckpointService::new_with_policy(
        root.path(),
        binding(id),
        Default::default(),
        PackageWritePolicy::ImmutableRelease {
            filename: "mixed.hmc".into(),
            digest: saved.digest,
        },
    )
    .unwrap();
    assert!(service
        .save(
            &stamp,
            "mixed.hmc",
            ExpectedFile::Sha256(saved.digest),
            Some(saved.digest),
            vec![]
        )
        .is_err());
    assert!(service
        .save(&stamp, "MIXED.hmc", ExpectedFile::Absent, None, vec![])
        .is_err());
    let copy = service
        .save(
            &stamp,
            "working-copy.hmc",
            ExpectedFile::Absent,
            Some(saved.digest),
            vec![],
        )
        .unwrap();
    assert_eq!(std::fs::read(&saved.path).unwrap(), original);
    assert_eq!(
        service.inventory("mixed.hmc").unwrap().resources,
        service.inventory("working-copy.hmc").unwrap().resources
    );
    let derived = core::load(&copy.receipt.path, Default::default()).unwrap();
    assert_eq!(derived.parent_digest(), Some(saved.digest));
    service
        .save(
            &stamp,
            "working-copy.hmc",
            ExpectedFile::Sha256(copy.receipt.digest),
            Some(copy.receipt.digest),
            vec![],
        )
        .unwrap();
    assert_eq!(std::fs::read(saved.path).unwrap(), original);
}

#[test]
fn stale_source_and_budget_fail_before_any_resource_or_file_change() {
    let root = tempfile::tempdir().unwrap();
    let id = "urn:poet:resource-stale-budget";
    let (service, stamp, saved, before) = mixed(root.path(), id);
    let (_, current) = studio_store::mutate_artifact_stamped(id, &stamp, |artifact| {
        artifact.title = "new accepted source".into();
        Ok(())
    })
    .unwrap();
    assert!(service
        .save(
            &stamp,
            "mixed.hmc",
            ExpectedFile::Sha256(saved.digest),
            Some(saved.digest),
            vec![]
        )
        .is_err());
    let limits = CheckpointLimits {
        file_bytes: before.len() - 1,
        ..Default::default()
    };
    let limited = CheckpointService::new(root.path(), binding(id), limits).unwrap();
    assert!(limited
        .save(
            &current,
            "mixed.hmc",
            ExpectedFile::Sha256(saved.digest),
            Some(saved.digest),
            vec![]
        )
        .is_err());
    assert_eq!(std::fs::read(saved.path).unwrap(), before);
}
