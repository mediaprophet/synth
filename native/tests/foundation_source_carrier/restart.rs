//! Fresh-process public native HMC activation and explicit reconstruction.
use super::*;

const CHILD: &str = "source_carrier_restart::source_carrier_restart_child";

#[test]
fn metadata_carrier_native_output_survives_fresh_process_activation_and_save_as() {
    let root = tempfile::tempdir().unwrap();
    let (id, mut h) = create(root.path());
    let initial = h.snapshot().unwrap().source;
    let original = package(&id, &initial.stamp, Some(vec![0xa1, 0x61, b's', 2]));
    let saved = core::save(
        &root.path().join("restart-original.hmc"),
        &original,
        ExpectedFile::Absent,
        Default::default(),
    )
    .unwrap();
    h.bind_surface_package(
        SurfacePackageRequest {
            filename: "restart-original.hmc".into(),
            digest: saved.digest,
        },
        initial,
    )
    .unwrap();
    let source = h.snapshot().unwrap().source;
    let source = h
        .authoring_edit(
            AuthoringEdit::Transform {
                id: "body".into(),
                translation: [3.0, 0.0, 0.0],
                rotation: [0.0; 3],
                scale: [1.0; 3],
            },
            source,
        )
        .unwrap()
        .snapshot
        .source;
    let source = h
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
        .snapshot
        .source;
    let output = h
        .compile_checkpoint(
            "restart.hmc",
            "restart-model",
            ExpectedFile::Absent,
            source.clone(),
        )
        .unwrap();
    assert_origin(
        &std::fs::read(&output.path).unwrap(),
        &original,
        &expected(&id, &source, saved.digest),
    );
    drop(h);
    let child = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", CHILD, "--nocapture", "--test-threads=1"])
        .env("POET_CARRIER_RESTART_ROOT", root.path())
        .env("POET_CARRIER_RESTART_SOURCE_ID", &id)
        .output()
        .unwrap();
    assert!(
        child.status.success(),
        "fresh native child: {} {}",
        String::from_utf8_lossy(&child.stdout),
        String::from_utf8_lossy(&child.stderr)
    );
    let copy = core::load(&root.path().join("restart-child.hmc"), Default::default()).unwrap();
    assert_eq!(copy.parent_digest(), Some(output.digest));
    assert_eq!(std::fs::read(saved.path).unwrap(), original);
}

#[test]
fn source_carrier_restart_child() {
    let Some(root) = std::env::var_os("POET_CARRIER_RESTART_ROOT") else {
        return;
    };
    let root = std::path::PathBuf::from(root);
    let id = std::env::var("POET_CARRIER_RESTART_SOURCE_ID").unwrap();
    assert!(
        studio_store::get_artifact_stamped(&id).is_none(),
        "child must have a fresh source store"
    );
    let service = CheckpointService::new(&root, binding(&id), CheckpointLimits::default()).unwrap();
    let prepared = service.prepare_reopen("restart.hmc").unwrap();
    let digest = prepared.file_digest();
    let exact_artifact = prepared.artifact().clone();
    let (activated, stamp) = service.activate(prepared).unwrap();
    assert_eq!(activated, exact_artifact);
    let canonical = CanonicalEditorHost::open(
        id.clone(),
        SessionLimits::default(),
        GeometryHost::default(),
        Box::new(Admission),
    )
    .unwrap();
    let mut h = CheckpointEditorHost::new(Box::new(canonical), service).unwrap();
    let held = h.snapshot().unwrap();
    assert!(
        held.reconciliation_required,
        "activation must not replay source implicitly"
    );
    assert_eq!(held.source.stamp, stamp);
    assert!(h.authoring_view(held.source.clone()).is_err());
    let source = h.rebuild_execution(held.source).unwrap().snapshot.source;
    assert!(!h.snapshot().unwrap().reconciliation_required);
    let parts = h.authoring_view(source.clone()).unwrap().parts;
    assert_eq!(parts[0].translation[0], 3.0);
    assert_eq!(parts[0].roughness, 0.8);
    assert_eq!(
        studio_store::get_artifact_stamped(&id).unwrap().0,
        exact_artifact,
        "explicit rebuild does not rewrite source or its journal"
    );
    let cells = h.source_cells(source.clone()).unwrap();
    assert_eq!(cells.len(), 3);
    assert_eq!(cells[0], SOURCE);
    let original = std::fs::read(root.join("restart-original.hmc")).unwrap();
    let copy = h
        .save_checkpoint("restart-child.hmc", ExpectedFile::Absent, source.clone())
        .unwrap();
    let bytes = std::fs::read(&copy.path).unwrap();
    assert_origin(&bytes, &original, &expected(&id, &source, digest));
    let reader = BundleReader::parse(&bytes).unwrap();
    assert_eq!(
        reader
            .entries()
            .iter()
            .filter(|e| e.key.starts_with("authoring/origins/"))
            .count(),
        1
    );
    assert_eq!(
        core::decode(&bytes, Default::default()).unwrap().artifact(),
        &exact_artifact
    );
}
