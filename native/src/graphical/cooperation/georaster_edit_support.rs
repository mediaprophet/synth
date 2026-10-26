//! Ephemeral synthetic grids built with public canonical codecs.
use super::*;
use sha2::{Digest, Sha256};

pub(super) fn source_annotation(bytes: &[u8], source: SourceDeclaration) -> Vec<u8> {
    let annotation = SourceAnnotation {
        origin_payload_digest: Sha256::digest(bytes).into(),
        source_declaration: source,
        qualification: SourceQualification::Unverified,
    };
    let mut out = Vec::new();
    ciborium::ser::into_writer(&annotation, &mut out).unwrap();
    out
}

pub(super) const FILE: &str = "synthetic.hmc";
pub(super) const KEY: &str = "fixture/grid.m64";
pub(super) const SOURCE: &str = "let retained_grid_source: I64 = 42;";
pub(super) fn synthetic_source() -> SourceDeclaration {
    SourceDeclaration {
        status: SourceStatus::Synthetic,
        description: "synthetic ephemeral test grid; unqualified".into(),
        evidence_ref: Some("urn:fixture:synthetic-grid-source".into()),
        observation_claim: false,
    }
}
pub(super) fn provided_source() -> SourceDeclaration {
    SourceDeclaration {
        status: SourceStatus::Provided,
        description: "provided integer array with an unverified observation claim".into(),
        evidence_ref: Some("urn:fixture:declared-source-evidence-only".into()),
        observation_claim: true,
    }
}
pub(super) fn binding(scope: &Scope) -> TrustedBinding {
    TrustedBinding {
        project_id: scope.project_id.clone(),
        document_id: scope.document_id.clone(),
        source_id: scope.source_id.clone(),
        domain: scope.domain.clone(),
    }
}
pub(super) fn service(root: &Path, scope: &Scope) -> CheckpointService {
    CheckpointService::new(root, binding(scope), CheckpointLimits::default()).unwrap()
}
pub(super) fn host(
    root: &Path,
    scope: &Scope,
    owner: Arc<dyn AuthorityOwner>,
    service: CheckpointService,
) -> AuthorizedEditorHost {
    let inner = CanonicalEditorHost::open(
        scope.source_id.clone(),
        SessionLimits::default(),
        GeometryHost::default(),
        Box::new(NoPreview),
    )
    .unwrap();
    let checkpoint = CheckpointEditorHost::new(Box::new(inner), service).unwrap();
    let imports = ResourceImportHost::new(
        Box::new(checkpoint),
        ResourceLibrary::new(root, 1024 * 1024).unwrap(),
    );
    AuthorizedEditorHost::new(
        Box::new(SharedEditorHost::new(Box::new(imports))),
        scope.clone(),
        owner,
        GatewayLimits::default(),
    )
    .unwrap()
}
pub(super) fn grid() -> Vec<u8> {
    let raster = ElevationRaster {
        width: 2,
        height: 2,
        cell_size_mm: 30_001,
        elevations_mm: &[10_001, i32::MIN, -1_203, 40_005],
    };
    let mut out = vec![0; 192 + 64 + 4096 * 64];
    let frame = GeodeticFrame {
        origin_lat_deg: -35.12345678912345,
        origin_lon_deg: 148.98765432198765,
        origin_elevation_mm: 350_001,
        epoch_year: 2020.25,
        srid: 7844,
        vertical_datum: 5711,
    };
    let n = compile_raster_to_m64_v3(
        &raster,
        &frame,
        7,
        RasterM64Policy {
            rows: M64RasterRows::South,
            edges: M64RasterEdges::RepeatNearest,
            nodata_sentinel_mm: Some(i32::MIN),
        },
        RasterM64Limits {
            max_output_bytes: out.len(),
            max_chunks: 1,
            max_records: 4096,
        },
        &AtomicBool::new(false),
        &mut out,
    )
    .unwrap();
    out.truncate(n);
    out
}
pub(super) fn selected(host: &dyn EditorHost, scope: &Scope) -> GeoRasterEditRequest {
    let inventory = host.package_inventory(FILE, reference(scope)).unwrap();
    GeoRasterEditRequest {
        resource: ResourceInspectionRequest {
            filename: FILE.into(),
            package_digest: inventory.digest,
            key: KEY.into(),
            resource_digest: inventory
                .resources
                .iter()
                .find(|r| r.key == KEY)
                .unwrap()
                .digest,
        },
        cell_size_mm: 12_345,
        origin_elevation_mm: -20_007,
        sample_x: 1,
        sample_y: 0,
        sample_mm: -50_009,
    }
}
pub(super) fn seed(root: &Path, scope: &Scope, host: &mut dyn EditorHost, owner: &FixtureOwner) {
    let outcome = host.submit(SOURCE, reference(scope)).unwrap();
    owner.refresh_document();
    let limits = CheckpointLimits::default();
    let grid = grid();
    let annotation = source_annotation(&grid, synthetic_source());
    let checkpoint = core::capture(&scope.source_id, &outcome.snapshot.source.stamp, limits)
        .unwrap()
        .apply_resource_edits(
            vec![
                ResourceEdit::Add(ResourceData {
                    key: KEY.into(),
                    kind: "m64".into(),
                    bytes: grid,
                    meta: Some(annotation),
                }),
                ResourceEdit::Add(ResourceData {
                    key: "unknown/resource".into(),
                    kind: "future/inert".into(),
                    bytes: b"preserve exact opaque bytes".to_vec(),
                    meta: Some(vec![0xa1, 0x61, b'z', 2]),
                }),
                ResourceEdit::Add(ResourceData {
                    key: "editable/source".into(),
                    kind: "source/raw".into(),
                    bytes: b"synthetic grid editable master".to_vec(),
                    meta: None,
                }),
            ],
            limits,
        )
        .unwrap();
    core::save(
        &root.join(FILE),
        &core::encode(&checkpoint, vec![], limits).unwrap(),
        ExpectedFile::Absent,
        limits,
    )
    .unwrap();
}
pub(super) fn assert_saved(root: &Path, scope: &Scope) {
    let checkpoint = core::load(&root.join(FILE), CheckpointLimits::default()).unwrap();
    assert_eq!(checkpoint.artifact().id, scope.source_id);
    let resource = checkpoint
        .resources()
        .iter()
        .find(|r| r.key == KEY)
        .unwrap();
    assert_eq!(resource.kind, "m64");
    let annotation: SourceAnnotation =
        ciborium::de::from_reader(resource.meta.as_deref().unwrap()).unwrap();
    assert_eq!(annotation.source_declaration, synthetic_source());
    assert_eq!(
        annotation.origin_payload_digest,
        <[u8; 32]>::from(Sha256::digest(&grid()))
    );
    assert_ne!(annotation.origin_payload_digest, resource.summary().digest);
    let mut scratch = [M64PayloadInterval::default(); 1];
    let reader = M64Reader::parse(
        &resource.bytes,
        checkpoints::ui_inspection_limits(),
        &mut scratch,
    )
    .unwrap();
    let meta = reader.georaster().unwrap();
    assert_eq!(
        (meta.cell_size_mm, meta.origin_elevation_mm),
        (12_345, -20_007)
    );
    assert_eq!(
        meta.origin_lat_deg.to_bits(),
        (-35.12345678912345f64).to_bits()
    );
    assert_eq!(
        meta.origin_lon_deg.to_bits(),
        148.98765432198765f64.to_bits()
    );
    assert_eq!(
        (meta.srid, meta.vertical_datum, meta.nodata_sentinel_mm),
        (7844, 5711, Some(i32::MIN))
    );
    let mut samples = [0; 4];
    decode_raster_from_m64_v3(&reader, &AtomicBool::new(false), &mut samples).unwrap();
    assert_eq!(samples, [10_001, -50_009, -1_203, 40_005]);
}
pub(super) fn assert_created(root: &Path) {
    let checkpoint = core::load(&root.join(FILE), CheckpointLimits::default()).unwrap();
    let created = checkpoint
        .resources()
        .iter()
        .find(|r| r.key == "fixture/reopened-source.m64")
        .unwrap();
    let annotation: SourceAnnotation =
        ciborium::de::from_reader(created.meta.as_deref().unwrap()).unwrap();
    assert_eq!(annotation.source_declaration, synthetic_source());
    assert_eq!(annotation.qualification, SourceQualification::Unverified);
    assert_ne!(annotation.origin_payload_digest, [0; 32]);
    assert_ne!(
        annotation.origin_payload_digest,
        created.summary().digest,
        "retained source annotation binds original source, not edited samples"
    );
    let unknown = checkpoint
        .resources()
        .iter()
        .find(|r| r.key == "unknown/resource")
        .unwrap();
    assert_eq!(unknown.bytes.as_slice(), b"preserve exact opaque bytes");
    assert_eq!(unknown.meta, Some(vec![0xa1, 0x61, b'z', 2]));
    let mut scratch = [M64PayloadInterval::default(); 1];
    let reader = M64Reader::parse(
        &created.bytes,
        checkpoints::ui_inspection_limits(),
        &mut scratch,
    )
    .unwrap();
    assert_eq!(reader.georaster().unwrap().cell_size_mm, 6_007);
    let mut samples = [0; 4];
    decode_raster_from_m64_v3(&reader, &AtomicBool::new(false), &mut samples).unwrap();
    assert_eq!(samples, [2_003, -8_007, i32::MIN, 9_007]);
}
