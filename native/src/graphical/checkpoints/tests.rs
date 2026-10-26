use super::*;

fn binding(id: &str) -> TrustedBinding {
    TrustedBinding {
        project_id: "urn:project:host-selected".into(),
        document_id: "urn:document:host-selected".into(),
        source_id: id.into(),
        domain: "Geometry".into(),
    }
}

fn source(id: &str) -> (StudioArtifact, StoreStamp) {
    let mut artifact = StudioArtifact::new(id, "Geometry", "Native retained material source");
    artifact.nodes.entry("poet.native.submission/00000000000000000001".into()).or_default()
        .insert("text".into(), "let poet_material = { base_color: [0.17,0.26,0.08,1.0], metallic: 0.0, roughness: 0.72 };".into());
    studio_store::create_artifact_if_absent(artifact).unwrap()
}

#[test]
fn save_prepare_exact_source_and_existing_conflict_are_explicit() {
    let root = tempfile::tempdir().unwrap();
    let (artifact, stamp) = source("checkpoint-service-source");
    let service =
        CheckpointService::new(root.path(), binding(&artifact.id), Default::default()).unwrap();
    let saved = service
        .save(
            &stamp,
            "native-source.hmc",
            ExpectedFile::Absent,
            None,
            vec![],
        )
        .unwrap();
    assert_eq!(saved.source_revision, artifact.lamport_clock);
    let prepared = service.prepare_reopen("native-source.hmc").unwrap();
    assert_eq!(prepared.artifact(), &artifact);
    assert_eq!(prepared.file_digest(), saved.receipt.digest);
    assert!(service.activate(prepared).is_err());
    assert_eq!(
        studio_store::get_artifact_stamped(&artifact.id).unwrap(),
        (artifact, stamp)
    );
}

#[test]
fn filename_and_document_association_are_checked_before_activation() {
    let root = tempfile::tempdir().unwrap();
    let (artifact, stamp) = source("checkpoint-service-paths");
    let service =
        CheckpointService::new(root.path(), binding(&artifact.id), Default::default()).unwrap();
    for filename in [
        "../outside.hmc",
        "C:/outside.hmc",
        "nested/file.hmc",
        "nested\\file.hmc",
        ".hmc",
        "source.json",
        "CON.hmc",
        "LPT1.hmc",
    ] {
        assert!(service
            .save(&stamp, filename, ExpectedFile::Absent, None, vec![])
            .is_err());
    }
    std::fs::create_dir(root.path().join("directory.hmc")).unwrap();
    assert!(service.prepare_reopen("directory.hmc").is_err());
    service
        .save(&stamp, "saved.hmc", ExpectedFile::Absent, None, vec![])
        .unwrap();
    let other =
        CheckpointService::new(root.path(), binding("different-source"), Default::default())
            .unwrap();
    assert!(other.prepare_reopen("saved.hmc").is_err());
    assert!(studio_store::get_artifact_stamped("different-source").is_none());
    let (_, other_stamp) = source("different-source");
    let prepared = service.prepare_reopen("saved.hmc").unwrap();
    assert!(other
        .save(
            &other_stamp,
            "saved.hmc",
            ExpectedFile::Sha256(prepared.file_digest()),
            None,
            vec![]
        )
        .is_err());
}

#[test]
fn stale_prepared_disk_and_changed_binding_cannot_promote() {
    let root = tempfile::tempdir().unwrap();
    let (artifact, stamp) = source("checkpoint-service-stale");
    let service =
        CheckpointService::new(root.path(), binding(&artifact.id), Default::default()).unwrap();
    service
        .save(&stamp, "saved.hmc", ExpectedFile::Absent, None, vec![])
        .unwrap();
    let prepared = service.prepare_reopen("saved.hmc").unwrap();
    let mut altered = binding(&artifact.id);
    altered.project_id = "different-trusted-project".into();
    let other = CheckpointService::new(root.path(), altered, Default::default()).unwrap();
    assert!(other.activate(prepared).is_err());
    let prepared = service.prepare_reopen("saved.hmc").unwrap();
    std::fs::write(root.path().join("saved.hmc"), b"truncated").unwrap();
    assert!(service.activate(prepared).is_err());
    assert_eq!(
        studio_store::get_artifact_stamped(&artifact.id).unwrap(),
        (artifact, stamp)
    );
}
