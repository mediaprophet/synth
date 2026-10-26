//! Native diagnostic integration: actual Vibe, geometry/material admission,
//! checkpoint decorator, fresh-process load and explicit read-only rebuild.
//! Direct diagnostic host construction is not authenticated-human evidence.
#[path = "restart_resources.rs"]
mod resource_fixture;
use super::{
    CheckpointEditorHost, CheckpointLimits, CheckpointService, ExpectedFile, TrustedBinding,
};
use crate::graphical::{
    controller::{CanonicalEditorHost, EditorHost, PreviewProvider},
    geometry_host::GeometryHost,
    materials::{self, MaterialDraft},
    preview::{PreviewFrame, PreviewLimits, SourceRef},
};
use qualia_core_db::{
    container_10d::MaterialRecord,
    poet_host::invoke::studio_store::{self, checkpoint, StudioArtifact},
    render::{
        assets::Mesh,
        compile_10d::{compile_mesh_to_10d, decode_10d_mesh},
    },
};
use sha2::{Digest, Sha256};
use std::{
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};
use vibe::{session::SessionLimits, Value};

const SOURCE_ID: &str = "urn:poet:diagnostic:native-checkpoint-restart";
const CHILD_TEST: &str = "graphical::checkpoints::restart_tests::native_restart_child";
const FILE: &str = "native-restart.hmc";

#[derive(Clone)]
struct Observed {
    mesh: Mesh,
    material: MaterialRecord,
}
#[derive(Clone, Default)]
struct HeadlessAdmission(Arc<Mutex<Vec<Observed>>>);
impl PreviewProvider for HeadlessAdmission {
    fn validate(&self, value: &Value) -> Result<(), String> {
        let (mesh, material) = materials::authored_scene(
            value,
            MaterialRecord::legacy_default(),
            PreviewLimits::default(),
        )?;
        self.0
            .lock()
            .map_err(|_| "diagnostic observation poisoned")?
            .push(Observed { mesh, material });
        Ok(())
    }
    fn preview(&mut self, _: SourceRef, _: &Value) -> Result<PreviewFrame, String> {
        Err(
            "headless diagnostic deliberately does not render a frame; GPU acceptance is separate"
                .into(),
        )
    }
}
impl HeadlessAdmission {
    fn last(&self) -> Observed {
        self.0.lock().unwrap().last().unwrap().clone()
    }
    fn count(&self) -> usize {
        self.0.lock().unwrap().len()
    }
}

fn binding() -> TrustedBinding {
    TrustedBinding {
        project_id: "urn:project:diagnostic-restart".into(),
        document_id: "urn:document:diagnostic-restart".into(),
        source_id: SOURCE_ID.into(),
        domain: "Geometry".into(),
    }
}
fn service(root: &Path) -> CheckpointService {
    CheckpointService::new(root, binding(), CheckpointLimits::default()).unwrap()
}
fn host(root: &Path, admission: HeadlessAdmission) -> CheckpointEditorHost {
    host_with_service(service(root), admission)
}
fn host_with_service(
    service: CheckpointService,
    admission: HeadlessAdmission,
) -> CheckpointEditorHost {
    let grants = vec![
        "capability.invoke".into(),
        "ComputationalGeometry.extrude_profile".into(),
    ];
    let canonical = CanonicalEditorHost::open(
        SOURCE_ID.into(),
        SessionLimits::default(),
        GeometryHost::new(grants, 200_000),
        Box::new(admission),
    )
    .unwrap();
    CheckpointEditorHost::new(Box::new(canonical), service).unwrap()
}
fn first_edit() -> MaterialDraft {
    MaterialDraft {
        base_color: [0.05, 0.4, 0.8, 1.0],
        roughness: 0.13,
        metallic: 0.75,
        ..Default::default()
    }
}
fn second_edit() -> MaterialDraft {
    MaterialDraft {
        base_color: [0.7, 0.1, 0.04, 1.0],
        roughness: 0.82,
        metallic: 0.04,
        ..Default::default()
    }
}
fn assert_material(observed: &Observed, draft: &MaterialDraft) {
    assert_eq!(
        observed.material,
        MaterialRecord {
            id: materials::diagnostic_material_id(),
            base_color: draft.base_color,
            emissive: draft.emissive,
            roughness: draft.roughness,
            metallic: draft.metallic,
            specular: draft.specular,
            alpha_cutoff: draft.alpha_cutoff,
            opacity_mode: draft.opacity_mode,
            double_sided: draft.double_sided,
            ..MaterialRecord::legacy_default()
        }
    );
    assert!(!observed.mesh.positions.is_empty());
    assert!(!observed.mesh.triangles.is_empty());
}

/// Bit-exact raw evaluated geometry witness. Unlike a QuantizedMesh derivative,
/// this hashes every authored f32 bit (including signed zero) and exact index.
/// The 32-byte test witness is not a source store or a distribution format.
fn raw_geometry_digest(mesh: &Mesh) -> [u8; 32] {
    let mut digest = Sha256::new();
    digest.update(b"poet-native-restart-raw-geometry/1");
    digest.update((mesh.positions.len() as u64).to_le_bytes());
    digest.update((mesh.triangles.len() as u64).to_le_bytes());
    for position in &mesh.positions {
        for component in position {
            digest.update(component.to_bits().to_le_bytes());
        }
    }
    for triangle in &mesh.triangles {
        for index in triangle {
            digest.update(index.to_le_bytes());
        }
    }
    digest.finalize().into()
}

/// The native QuantizedMesh contract is u16 per AABB axis, with documented
/// error <= extent/65535 (mesh_section.rs module contract and quantize/dequantize).
/// No generic epsilon or master-source tolerance is introduced here.
fn assert_quantized_derivative(master: &Mesh, derivative: &Mesh) {
    assert_eq!(master.positions.len(), derivative.positions.len());
    assert_eq!(
        master.triangles, derivative.triangles,
        "native derivative indices remain exact"
    );
    let mut minimum = [f32::INFINITY; 3];
    let mut maximum = [f32::NEG_INFINITY; 3];
    let mut max_error = [0.0f64; 3];
    for (raw, decoded) in master.positions.iter().zip(&derivative.positions) {
        for axis in 0..3 {
            minimum[axis] = minimum[axis].min(raw[axis]);
            maximum[axis] = maximum[axis].max(raw[axis]);
            max_error[axis] =
                max_error[axis].max((f64::from(raw[axis]) - f64::from(decoded[axis])).abs());
        }
    }
    let bound: [f64; 3] =
        std::array::from_fn(|axis| f64::from(maximum[axis] - minimum[axis]) / f64::from(u16::MAX));
    assert_eq!(
        minimum, derivative.min,
        "native codec retains exact AABB frame"
    );
    assert_eq!(
        maximum, derivative.max,
        "native codec retains exact AABB frame"
    );
    println!("native QuantizedMesh per-axis max_error={max_error:?}; documented_extent_div_65535_bound={bound:?}; indices_exact=true; raw_master_tolerance=none");
    for axis in 0..3 {
        assert!(
            max_error[axis].is_finite()
                && bound[axis].is_finite()
                && max_error[axis] <= bound[axis],
            "axis {axis} native derivative error {} exceeds documented u16 AABB bound {}",
            max_error[axis],
            bound[axis]
        );
    }
}

fn write_phase(root: &Path) {
    assert!(studio_store::get_artifact_stamped(SOURCE_ID).is_none());
    let mut artifact = StudioArtifact::new(
        SOURCE_ID,
        "Geometry",
        "Diagnostic native material source restart",
    );
    artifact.properties.insert(
        "contribution.status".into(),
        "diagnostic; unreviewed; no human attribution asserted".into(),
    );
    studio_store::create_artifact_if_absent(artifact).unwrap();
    let admission = HeadlessAdmission::default();
    let mut editor = host(root, admission.clone());
    assert!(editor.checkpoint_supported());
    assert!(editor.snapshot().unwrap().checkpoint_supported);
    let initial = materials::diagnostic_source();
    let initial_source = editor.snapshot().unwrap().source;
    let outcome = editor.submit(&initial, initial_source).unwrap();
    assert!(outcome.frame.is_none());
    let before = admission.last();
    assert_material(&before, &MaterialDraft::default());
    let edit = first_edit()
        .edit_source(
            outcome.snapshot.session_revision + 1,
            outcome
                .snapshot
                .material_edit_id
                .expect("actual accepted mutable material binding"),
        )
        .unwrap();
    let outcome = editor
        .material_edit(&edit, outcome.snapshot.source)
        .unwrap();
    assert!(outcome.frame.is_none());
    let after = admission.last();
    assert_material(&after, &first_edit());
    assert_eq!(before.mesh.positions, after.mesh.positions);
    assert_eq!(before.mesh.triangles, after.mesh.triangles);
    assert_eq!(
        editor
            .source_cells(outcome.snapshot.source.clone())
            .unwrap(),
        vec![initial, edit]
    );
    let receipt = editor
        .save_checkpoint(FILE, ExpectedFile::Absent, outcome.snapshot.source)
        .unwrap();
    let receipt = resource_fixture::add(&mut editor, receipt, root);
    let saved = checkpoint::load(&receipt.path, Default::default()).unwrap();
    assert_eq!(
        saved.artifact(),
        &studio_store::get_artifact_stamped(SOURCE_ID).unwrap().0
    );
    assert_eq!(admission.count(), 2);
    // Native derived geometry is test evidence only, never an alternate source
    // store or a complete embodiment. Raw replay is checked separately by a
    // bit-exact digest; this .10d proof is an explicitly quantized derivative.
    std::fs::write(
        root.join("raw-geometry.sha256"),
        raw_geometry_digest(&after.mesh),
    )
    .unwrap();
    let proof = compile_mesh_to_10d(&after.mesh).unwrap();
    assert!(proof.len() < 1024 * 1024);
    assert_quantized_derivative(&after.mesh, &decode_10d_mesh(&proof).unwrap());
    std::fs::write(root.join("observed-geometry.10d"), proof).unwrap();
}

fn read_phase(root: &Path) {
    assert!(studio_store::get_artifact_stamped(SOURCE_ID).is_none());
    let service = service(root);
    let prepared = service.prepare_reopen(FILE).unwrap();
    let previous_digest = prepared.file_digest();
    let exact_saved_source = prepared.artifact().clone();
    let (_, fresh_stamp) = service.activate(prepared).unwrap();
    let admission = HeadlessAdmission::default();
    let baseline =
        checkpoint::load(&root.join("resource-baseline.hmc"), Default::default()).unwrap();
    assert_eq!(baseline.resources().len(), 7);
    let mut editor = host_with_service(service, admission.clone());
    let held = editor.snapshot().unwrap();
    assert_eq!(held.source.stamp, fresh_stamp);
    assert!(held.reconciliation_required);
    assert!(held.material_edit_id.is_none());
    assert_eq!(
        admission.count(),
        0,
        "load/open never evaluates retained source"
    );
    let rebuilt = editor.rebuild_execution(held.source).unwrap();
    assert!(rebuilt.frame.is_none());
    assert!(!rebuilt.snapshot.reconciliation_required);
    assert!(!held.execution_epoch.is_current());
    assert_eq!(rebuilt.snapshot.session_revision, 2);
    assert_eq!(
        studio_store::get_artifact_stamped(SOURCE_ID).unwrap().0,
        exact_saved_source
    );
    let rebuilt_observation = admission.last();
    assert_material(&rebuilt_observation, &first_edit());
    let raw_witness = std::fs::read(root.join("raw-geometry.sha256")).unwrap();
    assert_eq!(raw_witness.len(), 32);
    assert_eq!(raw_geometry_digest(&rebuilt_observation.mesh).as_slice(), raw_witness,
        "actual source replay must reproduce every raw f32/index bit; no normalization is permitted");
    let proof_bytes = std::fs::read(root.join("observed-geometry.10d")).unwrap();
    assert!(proof_bytes.len() < 1024 * 1024);
    let proof = decode_10d_mesh(&proof_bytes).unwrap();
    assert_quantized_derivative(&rebuilt_observation.mesh, &proof);
    let edit = second_edit()
        .edit_source(
            rebuilt.snapshot.session_revision + 1,
            rebuilt
                .snapshot
                .material_edit_id
                .expect("rebuild restores actual mutable material binding"),
        )
        .unwrap();
    let changed = editor
        .material_edit(&edit, rebuilt.snapshot.source)
        .unwrap();
    assert!(changed.frame.is_none());
    let edited = admission.last();
    assert_material(&edited, &second_edit());
    assert_eq!(edited.mesh.positions, rebuilt_observation.mesh.positions);
    assert_eq!(edited.mesh.triangles, rebuilt_observation.mesh.triangles);
    assert_eq!(
        raw_geometry_digest(&edited.mesh).as_slice(),
        raw_witness,
        "material editing must retain every raw geometry bit"
    );
    let receipt = editor
        .save_checkpoint(
            FILE,
            ExpectedFile::Sha256(previous_digest),
            changed.snapshot.source.clone(),
        )
        .unwrap();
    let saved = checkpoint::load(&receipt.path, Default::default()).unwrap();
    assert_eq!(saved.resources(), baseline.resources());
    assert_eq!(saved.parent_digest(), Some(previous_digest));
    assert_eq!(
        saved.artifact(),
        &studio_store::get_artifact_stamped(SOURCE_ID).unwrap().0
    );
    assert_eq!(
        editor
            .source_cells(changed.snapshot.source.clone())
            .unwrap(),
        vec![
            materials::diagnostic_source(),
            first_edit()
                .edit_source(2, materials::diagnostic_material_id())
                .unwrap(),
            edit
        ]
    );
    // Save As must use the activated service's retained carrier, not rebuild a
    // source-only package. This also verifies resource edits survive restart.
    let copy = editor
        .save_checkpoint(
            "restart-working-copy.hmc",
            ExpectedFile::Absent,
            changed.snapshot.source,
        )
        .unwrap();
    let copied = checkpoint::load(&copy.path, Default::default()).unwrap();
    assert_eq!(copied.resources(), baseline.resources());
    assert_eq!(copied.parent_digest(), Some(receipt.digest));
}

struct RunDirectory {
    root: PathBuf,
    path: PathBuf,
}
impl RunDirectory {
    fn new() -> Self {
        let root = std::env::var_os("POET_NATIVE_CHECKPOINT_TEST_ROOT")
            .map(PathBuf::from)
            .unwrap_or_else(std::env::temp_dir)
            .canonicalize()
            .expect("existing host-selected test root");
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = root.join(format!(
            "poet-native-restart-{}-{nonce}",
            std::process::id()
        ));
        std::fs::create_dir(&path).unwrap();
        std::fs::write(
            path.join(".owned-native-restart"),
            b"Poet diagnostic RAII run",
        )
        .unwrap();
        Self { root, path }
    }
}
impl Drop for RunDirectory {
    fn drop(&mut self) {
        if self.path.parent() == Some(self.root.as_path())
            && self.path.join(".owned-native-restart").is_file()
        {
            let _ = std::fs::remove_dir_all(&self.path);
        }
    }
}

#[test]
fn source_material_checkpoint_survives_fresh_native_process() {
    let run = RunDirectory::new();
    for phase in ["write", "read"] {
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", CHILD_TEST, "--nocapture", "--test-threads=1"])
            .env("POET_NATIVE_RESTART_PHASE", phase)
            .env("POET_NATIVE_RESTART_DIRECTORY", &run.path)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "native child {phase}: {} {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        println!(
            "native checkpoint {phase} evidence:\n{}",
            String::from_utf8_lossy(&output.stdout)
        );
    }
    let final_source = checkpoint::load(&run.path.join(FILE), Default::default()).unwrap();
    assert_eq!(final_source.artifact().nodes.len(), 3);
    assert!(final_source.parent_digest().is_some());
}

#[test]
fn native_restart_child() {
    let Some(phase) = std::env::var_os("POET_NATIVE_RESTART_PHASE") else {
        return;
    };
    let root = PathBuf::from(
        std::env::var_os("POET_NATIVE_RESTART_DIRECTORY").expect("parent-owned run directory"),
    );
    match phase.to_str().unwrap() {
        "write" => write_phase(&root),
        "read" => read_phase(&root),
        _ => panic!("unknown native restart diagnostic phase"),
    }
}
