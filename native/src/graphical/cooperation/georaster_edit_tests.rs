//! Synthetic grids only. Software-agent fixture authority is not human acceptance.
use super::*;
use crate::graphical::{
    checkpoints::{
        self, ui_georaster_edit_limits, CheckpointEditorHost, CheckpointLimits, CheckpointService,
        ExpectedFile, GeoRasterCreateRequest, GeoRasterEditRequest, ResourceData, ResourceEdit,
        ResourceInspectionRequest, SourceAnnotation, SourceDeclaration, SourceQualification,
        SourceStatus, TrustedBinding,
    },
    package::{ResourceImportHost, ResourceLibrary},
    shared_host::SharedEditorHost,
};
use qualia_core_db::poet_host::invoke::studio_store::checkpoint as core;
use qualia_core_db::q42::m64_matter::{
    M64PayloadInterval, M64RasterEdges, M64RasterRows, M64Reader,
};
use qualia_geospatial::{
    raster_m64::{
        compile_raster_to_m64_v3, decode_raster_from_m64_v3, RasterM64Limits, RasterM64Policy,
    },
    ElevationRaster, GeodeticFrame,
};
use std::{path::Path, sync::atomic::AtomicBool};

#[path = "georaster_edit_support.rs"]
mod support;
use support::*;
#[path = "georaster_metadata_tests.rs"]
mod metadata_tests;

fn await_bounded<F: std::future::Future>(future: F) -> F::Output {
    struct Wake;
    impl std::task::Wake for Wake {
        fn wake(self: Arc<Self>) {}
    }
    let waker = std::task::Waker::from(Arc::new(Wake));
    let mut context = std::task::Context::from_waker(&waker);
    let mut future = std::pin::pin!(future);
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    loop {
        if let std::task::Poll::Ready(value) = future.as_mut().poll(&mut context) {
            return value;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "native raster job exceeded bounded fixture deadline"
        );
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
}

#[test]
fn georaster_edit_authorized_save_and_fresh_process_reopen() {
    const CHILD_ROOT: &str = "POET_GEORASTER_REOPEN_ROOT";
    if let Some(root) = std::env::var_os(CHILD_ROOT) {
        let root = std::path::PathBuf::from(root);
        let scope = Scope {
            project_id: "fixture-project".into(),
            document_id: std::env::var("POET_GEORASTER_DOCUMENT").unwrap(),
            source_id: std::env::var("POET_GEORASTER_SOURCE_ID").unwrap(),
            domain: "Geometry".into(),
        };
        assert!(studio_store::get_artifact_stamped(&scope.source_id).is_none());
        let service = service(&root, &scope);
        let prepared = service.prepare_reopen(FILE).unwrap();
        let source = prepared.artifact();
        let owner = Arc::new(FixtureOwner {
            scope: scope.clone(),
            state: Mutex::new(FixtureState {
                role: ProjectRole::Contributor,
                document: DocumentRevision {
                    project_id: scope.project_id.clone(),
                    document_id: scope.document_id.clone(),
                    source_id: scope.source_id.clone(),
                    revision: source.lamport_clock.into(),
                    source_digest: source_digest(source, GatewayLimits::default()).unwrap(),
                    author_subject: Some("fixture-subject".into()),
                },
                generation: 1,
                revoked: false,
                calls: 1,
            }),
        });
        checkpoints::authorised_reopen::reopen(
            owner.as_ref(),
            &scope,
            &service,
            prepared,
            GatewayLimits::default(),
        )
        .unwrap();
        let host = host(&root, &scope, owner, service);
        assert_eq!(host.source_cells(reference(&scope)).unwrap(), [SOURCE]);
        assert_saved(&root, &scope);
        assert_created(&root);
        let request = selected(&host, &scope);
        let mut scratch = [M64PayloadInterval::default(); 1];
        assert!(host
            .inspect_package_resource(
                request.resource,
                reference(&scope),
                checkpoints::ui_inspection_limits(),
                &mut scratch
            )
            .unwrap()
            .payload
            .georaster
            .is_some());
        return;
    }
    let root = tempfile::tempdir().unwrap();
    let scope = scope();
    let owner = Arc::new(FixtureOwner::new(&scope, ProjectRole::Contributor));
    let mut host = host(
        root.path(),
        &scope,
        owner.clone(),
        service(root.path(), &scope),
    );
    seed(root.path(), &scope, &mut host, owner.as_ref());
    let before = core::load(&root.path().join(FILE), CheckpointLimits::default()).unwrap();
    let request = selected(&host, &scope);
    let receipt = host
        .edit_georaster(
            request.clone(),
            reference(&scope),
            ui_georaster_edit_limits(),
            &AtomicBool::new(false),
        )
        .unwrap();
    let after = core::load(&root.path().join(FILE), CheckpointLimits::default()).unwrap();
    assert_ne!(receipt.digest, request.resource.package_digest);
    assert_eq!(after.parent_digest(), Some(request.resource.package_digest));
    assert_eq!(before.artifact(), after.artifact());
    let a: Vec<_> = before.resources().iter().filter(|r| r.key != KEY).collect();
    let b: Vec<_> = after.resources().iter().filter(|r| r.key != KEY).collect();
    assert_eq!(
        a, b,
        "every unrelated resource and opaque metadata must be exact"
    );
    assert_eq!(host.source_cells(reference(&scope)).unwrap(), [SOURCE]);
    assert_saved(root.path(), &scope);
    let original = grid();
    let mut scratch = [M64PayloadInterval::default(); 1];
    let reader =
        M64Reader::parse(&original, checkpoints::ui_inspection_limits(), &mut scratch).unwrap();
    let mut metadata = *reader.georaster().unwrap();
    metadata.cell_size_mm = 6_007;
    host.create_georaster(
        GeoRasterCreateRequest {
            filename: FILE.into(),
            package_digest: receipt.digest,
            key: "fixture/reopened-source.m64".into(),
            metadata,
            lut_bank_epoch: 7,
            samples_mm: vec![2_003, -8_007, i32::MIN, 9_001],
            source: synthetic_source(),
        },
        reference(&scope),
        ui_georaster_edit_limits(),
        &AtomicBool::new(false),
    )
    .unwrap();
    let inventory = host.package_inventory(FILE, reference(&scope)).unwrap();
    let created = inventory
        .resources
        .iter()
        .find(|r| r.key == "fixture/reopened-source.m64")
        .unwrap();
    let carrier = core::load(&root.path().join(FILE), CheckpointLimits::default()).unwrap();
    let created_source = carrier
        .resources()
        .iter()
        .find(|r| r.key == created.key)
        .unwrap();
    let annotation: SourceAnnotation =
        ciborium::de::from_reader(created_source.meta.as_deref().unwrap()).unwrap();
    assert_eq!(annotation.origin_payload_digest, created.digest);
    host.edit_georaster(
        GeoRasterEditRequest {
            resource: ResourceInspectionRequest {
                filename: FILE.into(),
                package_digest: inventory.digest,
                key: created.key.clone(),
                resource_digest: created.digest,
            },
            origin_elevation_mm: 350_001,
            cell_size_mm: 6_007,
            sample_x: 1,
            sample_y: 1,
            sample_mm: 9_007,
        },
        reference(&scope),
        ui_georaster_edit_limits(),
        &AtomicBool::new(false),
    )
    .unwrap();
    let edited_inventory = host.package_inventory(FILE, reference(&scope)).unwrap();
    assert_eq!(
        edited_inventory
            .resources
            .iter()
            .find(|r| r.key == created.key)
            .unwrap()
            .metadata_digest,
        created.metadata_digest,
        "ReplaceBytes must preserve exact source annotation CBOR"
    );
    assert_created(root.path());
    drop(host);
    let mut child = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "graphical::cooperation::tests::georaster_edit_tests::georaster_edit_authorized_save_and_fresh_process_reopen", "--nocapture", "--test-threads=1"])
        .env(CHILD_ROOT, root.path()).env("POET_GEORASTER_DOCUMENT", &scope.document_id)
        .env("POET_GEORASTER_SOURCE_ID", &scope.source_id)
        .stdout(std::process::Stdio::piped()).stderr(std::process::Stdio::piped()).spawn().unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
    loop {
        if child.try_wait().unwrap().is_some() {
            break;
        }
        if std::time::Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            panic!("fresh reopen child exceeded bounded 30-second fixture deadline");
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    let output = child.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("1 passed"),
        "child selected no test"
    );
}

#[test]
fn georaster_edit_authority_stale_digests_source_and_cancel_leave_hmc_exact() {
    let root = tempfile::tempdir().unwrap();
    let scope = scope();
    let owner = Arc::new(FixtureOwner::new(&scope, ProjectRole::Contributor));
    let mut host = host(
        root.path(),
        &scope,
        owner.clone(),
        service(root.path(), &scope),
    );
    seed(root.path(), &scope, &mut host, owner.as_ref());
    let request = selected(&host, &scope);
    let bytes = std::fs::read(root.path().join(FILE)).unwrap();
    for which in 0..5 {
        let mut bad = request.clone();
        let cancelled = AtomicBool::new(which == 4);
        match which {
            0 => bad.resource.package_digest[0] ^= 1,
            1 => bad.resource.resource_digest[0] ^= 1,
            2 => bad.sample_x = 2,
            3 => owner.state.lock().unwrap().revoked = true,
            _ => {}
        }
        assert!(host
            .edit_georaster(
                bad,
                reference(&scope),
                ui_georaster_edit_limits(),
                &cancelled
            )
            .is_err());
        owner.state.lock().unwrap().revoked = false;
        assert_eq!(std::fs::read(root.path().join(FILE)).unwrap(), bytes);
    }
    owner.state.lock().unwrap().role = ProjectRole::Observer;
    assert!(host
        .edit_georaster(
            request.clone(),
            reference(&scope),
            ui_georaster_edit_limits(),
            &AtomicBool::new(false)
        )
        .is_err());
    owner.state.lock().unwrap().role = ProjectRole::Contributor;
    let old = reference(&scope);
    let outcome = host
        .submit("let another_grid_source: I64 = 1;", old.clone())
        .unwrap();
    owner.refresh_document();
    assert_ne!(outcome.snapshot.source, old);
    assert!(host
        .edit_georaster(
            request,
            old,
            ui_georaster_edit_limits(),
            &AtomicBool::new(false)
        )
        .is_err());
    assert_eq!(std::fs::read(root.path().join(FILE)).unwrap(), bytes);
}

#[test]
fn georaster_create_explicit_source_guarded_add_and_refusals_preserve_hmc() {
    let root = tempfile::tempdir().unwrap();
    let scope = scope();
    let owner = Arc::new(FixtureOwner::new(&scope, ProjectRole::Contributor));
    let mut host = host(
        root.path(),
        &scope,
        owner.clone(),
        service(root.path(), &scope),
    );
    seed(root.path(), &scope, &mut host, owner.as_ref());
    let before = core::load(&root.path().join(FILE), CheckpointLimits::default()).unwrap();
    let mut scratch = [M64PayloadInterval::default(); 1];
    let original = grid();
    let reader =
        M64Reader::parse(&original, checkpoints::ui_inspection_limits(), &mut scratch).unwrap();
    let mut metadata = *reader.georaster().unwrap();
    metadata.origin_elevation_mm = 101_103;
    metadata.cell_size_mm = 7_009;
    let request = GeoRasterCreateRequest {
        filename: FILE.into(),
        package_digest: before.file_digest().unwrap(),
        key: "fixture/new-source.m64".into(),
        metadata,
        lut_bank_epoch: 9,
        samples_mm: vec![4_001, i32::MIN, -7_003, 8_009],
        source: provided_source(),
    };
    let original_bytes = std::fs::read(root.path().join(FILE)).unwrap();
    for which in 0..7 {
        let mut bad = request.clone();
        let cancelled = AtomicBool::new(which == 3);
        let mut limits = ui_georaster_edit_limits();
        match which {
            0 => bad.package_digest[0] ^= 1,
            1 => bad.key = KEY.into(),
            2 => {
                bad.samples_mm.pop();
            }
            4 => limits.max_working_bytes = 1,
            5 => owner.state.lock().unwrap().revoked = true,
            6 => bad.source.description.clear(),
            _ => {}
        }
        assert!(host
            .create_georaster(bad, reference(&scope), limits, &cancelled)
            .is_err());
        owner.state.lock().unwrap().revoked = false;
        assert_eq!(
            std::fs::read(root.path().join(FILE)).unwrap(),
            original_bytes
        );
    }
    let receipt = host
        .create_georaster(
            request.clone(),
            reference(&scope),
            ui_georaster_edit_limits(),
            &AtomicBool::new(false),
        )
        .unwrap();
    let after = core::load(&root.path().join(FILE), CheckpointLimits::default()).unwrap();
    assert_eq!(after.file_digest(), Some(receipt.digest));
    assert_eq!(after.artifact(), before.artifact());
    for old in before.resources() {
        assert_eq!(
            after.resources().iter().find(|r| r.key == old.key),
            Some(old)
        );
    }
    let added = after
        .resources()
        .iter()
        .find(|r| r.key == request.key)
        .unwrap();
    let annotation: SourceAnnotation =
        ciborium::de::from_reader(added.meta.as_deref().unwrap()).unwrap();
    assert_eq!(annotation.source_declaration, request.source);
    assert_eq!(annotation.qualification, SourceQualification::Unverified);
    assert_eq!(annotation.origin_payload_digest, added.summary().digest);
    let reader = M64Reader::parse(
        &added.bytes,
        checkpoints::ui_inspection_limits(),
        &mut scratch,
    )
    .unwrap();
    assert_eq!(reader.georaster(), Some(&request.metadata));
    let mut samples = [0; 4];
    decode_raster_from_m64_v3(&reader, &AtomicBool::new(false), &mut samples).unwrap();
    assert_eq!(samples.as_slice(), request.samples_mm);
    assert!(host
        .create_georaster(
            request,
            reference(&scope),
            ui_georaster_edit_limits(),
            &AtomicBool::new(false)
        )
        .is_err());
}

#[test]
fn georaster_jobs_dispatch_real_authorized_create_edit_and_withhold_revoked_results() {
    use crate::graphical::jobs::{self, Action, Context, Worker};
    let root = tempfile::tempdir().unwrap();
    let scope = scope();
    let owner = Arc::new(FixtureOwner::new(&scope, ProjectRole::Contributor));
    let mut host = host(
        root.path(),
        &scope,
        owner.clone(),
        service(root.path(), &scope),
    );
    seed(root.path(), &scope, &mut host, owner.as_ref());
    let request = selected(&host, &scope);
    let snapshot = host.snapshot().unwrap();
    let (worker, handle) = Worker::start(Box::new(host)).unwrap();
    let reply = await_bounded(jobs::run(
        handle.clone(),
        Action::EditGeoRaster {
            request,
            expected: snapshot.source.clone(),
        },
        Context::from_snapshot(&snapshot),
    ))
    .unwrap();
    assert!(reply.operation_succeeded, "{}", reply.message);
    assert!(jobs::completion_is_current(&handle, &snapshot, &reply));
    assert_saved(root.path(), &scope);
    let package = reply.inventory.unwrap();
    let snapshot = reply.snapshot.unwrap();
    let source = grid();
    let mut scratch = [M64PayloadInterval::default(); 1];
    let reader =
        M64Reader::parse(&source, checkpoints::ui_inspection_limits(), &mut scratch).unwrap();
    let request = GeoRasterCreateRequest {
        filename: FILE.into(),
        package_digest: package.digest,
        key: "fixture/from-native-job.m64".into(),
        metadata: *reader.georaster().unwrap(),
        lut_bank_epoch: 7,
        samples_mm: vec![1, 2, i32::MIN, 4],
        source: provided_source(),
    };
    let reply = await_bounded(jobs::run(
        handle.clone(),
        Action::CreateGeoRaster {
            request: request.clone(),
            expected: snapshot.source.clone(),
        },
        Context::from_snapshot(&snapshot),
    ))
    .unwrap();
    assert!(reply.operation_succeeded, "{}", reply.message);
    assert!(jobs::completion_is_current(&handle, &snapshot, &reply));
    let saved = std::fs::read(root.path().join(FILE)).unwrap();
    owner.state.lock().unwrap().revoked = true;
    let reply = await_bounded(jobs::run(
        handle.clone(),
        Action::CreateGeoRaster {
            request,
            expected: snapshot.source.clone(),
        },
        Context::from_snapshot(&snapshot),
    ))
    .unwrap();
    assert!(!reply.operation_succeeded);
    assert!(!jobs::completion_is_current(&handle, &snapshot, &reply));
    assert!(reply.inventory.is_none());
    assert_eq!(std::fs::read(root.path().join(FILE)).unwrap(), saved);
    worker.finish().unwrap();
}
