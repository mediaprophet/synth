//! Ignored engineering qualification of the unchanged, pinned reference workload.
//! Native offscreen execution only: no GUI, contributor/human review, new master,
//! derivative release or production acceptance. Film/game/cartoon remain HOLD.
use super::{
    authoring::AuthoringEdit,
    checkpoints::{
        CheckpointEditorHost, CheckpointLimits, CheckpointService, ExpectedFile, TrustedBinding,
    },
    controller::{CanonicalEditorHost, EditorHost, MeshPreviewProvider},
    geometry_host::GeometryHost,
    preview::{PreviewCamera, PreviewFrame},
    viewport::ViewportCommand,
    viewport_provider::InteractivePreviewProvider,
};
use qualia_core_db::{
    container_10d::MaterialRecord,
    poet_host::invoke::studio_store::{self, StudioArtifact},
    render::surface_authoring::encode_surface_png_into,
};
use sha2::{Digest, Sha256};
use std::{io::Write, path::Path, sync::atomic::AtomicBool, time::Instant};

const MASTER: &str =
    "C:/github/ecopunk/assets/reference/community-grounds-growing-station/master.vibe";
const PIN: &str = "8aac87e9f98dc305ea85d45f2ebae19aba5fbd9323c165f160c83bb46ce89a63";
const ID: &str = "urn:poet:round9:read-only-growing-reference-qualification";
const CHILD: &str = "graphical::reference_authoring_tests::r9_reference_authoring_child";
const HMC: &str = "diagnostic-equipment.hmc";
fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn binding() -> TrustedBinding {
    TrustedBinding {
        project_id: "urn:poet:r9:diagnostic-project".into(),
        document_id: "urn:poet:r9:diagnostic-document".into(),
        source_id: ID.into(),
        domain: "Geometry".into(),
    }
}
fn service(root: &Path) -> CheckpointService {
    CheckpointService::new(root, binding(), CheckpointLimits::default()).unwrap()
}
fn host(root: &Path) -> CheckpointEditorHost {
    let profile=super::native_authoring_profile::NativeAuthoringProfile::Equipment;
    let provider=InteractivePreviewProvider::with_budget(MeshPreviewProvider {
        material:MaterialRecord::legacy_default(),
        camera:PreviewCamera{yaw:std::f32::consts::PI,pitch:0.15,distance:5.0,target:[0.0,0.8,0.0]},
        limits:profile.preview_limits(),
    },profile.viewport_budget()).unwrap();
    let grants=["capability.invoke","ComputationalGeometry.profile_sweep","ComputationalGeometry.uv_sphere","ComputationalGeometry.torus"].into_iter().map(str::to_owned).collect();
    let host=CanonicalEditorHost::open_with_rebuild_limits(ID.into(),profile.session_limits(),
        GeometryHost::new(grants,profile.max_vertices()),Box::new(provider),profile.rebuild_limits()).unwrap();
    CheckpointEditorHost::new(Box::new(host),service(root)).unwrap()
}
fn diagnostic(name: &str, frame: &PreviewFrame) {
    let Some(root) = std::env::var_os("POET_REFERENCE_QUALIFICATION_ARTIFACTS") else {
        return;
    };
    let root = std::fs::canonicalize(root).unwrap();
    let qa = std::fs::canonicalize("C:/github/ecopunk/docs/qa").unwrap();
    assert!(
        root.is_dir() && root.starts_with(qa),
        "diagnostics must target the configured Ecopunk QA directory"
    );
    let mut png = vec![0; frame.rgba8.len() + frame.side as usize + 4096];
    let n = encode_surface_png_into(frame.side, &frame.rgba8, &mut png).unwrap();
    let mut file = std::fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(root.join(name))
        .unwrap();
    file.write_all(&png[..n]).unwrap();
}
fn check_view(host: &dyn EditorHost) {
    let source = host.snapshot().unwrap().source;
    let view = host.authoring_view(source).unwrap();
    assert_eq!(
        view.parts.len(),
        199,
        "actual unchanged reference assembly workload"
    );
    assert_eq!(
        view.parameters
            .iter()
            .find(|(key, _)| key == "width")
            .unwrap()
            .1,
        2.8
    );
    assert_eq!(
        view.parts
            .iter()
            .find(|part| part.id == "station")
            .unwrap()
            .translation,
        [0.05, 0.0, 0.0]
    );
    let cabinet = view
        .parts
        .iter()
        .filter(|part| part.material_id == 9005)
        .collect::<Vec<_>>();
    assert!(!cabinet.is_empty());
    assert!(cabinet.iter().all(|part| part.roughness == 0.72
        && part.metallic == 0.12
        && part.base_color == [0.08, 0.22, 0.13, 1.0]));
}

#[test]
#[ignore = "explicit native GPU/reference engineering qualification; never production acceptance"]
fn r9_reference_master_source_layers_viewport_and_fresh_hmc() {
    let root = tempfile::tempdir().unwrap();
    for phase in ["write", "read"] {
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--ignored",
                "--exact",
                CHILD,
                "--nocapture",
                "--test-threads=1",
            ])
            .env("POET_REFERENCE_AUTHORING_PHASE", phase)
            .env("POET_REFERENCE_AUTHORING_ROOT", root.path())
            .output()
            .unwrap();
        println!("{}", String::from_utf8_lossy(&output.stdout));
        assert!(
            output.status.success(),
            "reference {phase} failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(root.path().join(HMC).is_file(), "child must execute the real workload");
        assert!(root.path().join(if phase=="write" {"observed.json"} else {"reopened.json"}).is_file(), "each child phase must produce its evidence");
    }
    if let Some(out) = std::env::var_os("POET_REFERENCE_QUALIFICATION_ARTIFACTS") {
        let out=std::fs::canonicalize(out).unwrap();
        let qa=std::fs::canonicalize("C:/github/ecopunk/docs/qa").unwrap();
        assert!(out.starts_with(qa));
        for name in ["observed.json","reopened.json",HMC,"accepted-source.vibe"] {
            let bytes=std::fs::read(root.path().join(name)).unwrap();
            let mut file=std::fs::OpenOptions::new().create_new(true).write(true).open(out.join(name)).unwrap();
            file.write_all(&bytes).unwrap();
        }
    }
}
#[test]
#[ignore = "child of explicit reference qualification; requires native GPU"]
fn r9_reference_authoring_child() {
    let Some(phase) = std::env::var_os("POET_REFERENCE_AUTHORING_PHASE") else {
        return;
    };
    let timer = Instant::now();
    let root = std::path::PathBuf::from(std::env::var_os("POET_REFERENCE_AUTHORING_ROOT").unwrap());
    let master = std::fs::read(MASTER).unwrap();
    assert_eq!(master.len(), 15_690);
    assert_eq!(hash(&master), PIN);
    let master = std::str::from_utf8(&master).unwrap();
    let cancel = AtomicBool::new(false);
    assert!(studio_store::get_artifact_stamped(ID).is_none());
    if phase == "write" {
        studio_store::create_artifact_if_absent(StudioArtifact::new(
            ID,
            "Geometry",
            "Read-only source / temporary diagnostic qualification",
        ))
        .unwrap();
        let mut h = host(&root);
        let source = h.snapshot().unwrap().source;
        let source = h
            .submit(master, source)
            .expect("unchanged full master must execute, with no reduced preset")
            .snapshot
            .source;
        let before = h.authoring_view(source.clone()).unwrap();
        assert_eq!(before.parts.len(), 199);
        let mesh_before = h
            .compiled_assets("before", source.clone())
            .unwrap()
            .remove(0)
            .1
            .mesh;
        h.viewport_action(super::viewport::preview_lighting(super::viewport::ViewLighting::neutral_studio()),source.clone(),&cancel).unwrap();
        let framed = h
            .viewport_action(ViewportCommand::FrameAll, source.clone(), &cancel)
            .unwrap();
        diagnostic("r9-reference-frame-all.png", &framed.frame);
        let source = h
            .authoring_edit(AuthoringEdit::EnableSourceLayers, source)
            .unwrap()
            .snapshot
            .source;
        let source = h
            .authoring_edit(
                AuthoringEdit::Material {
                    material_id: 9005,
                    base_color: [0.08, 0.22, 0.13, 1.0],
                    roughness: 0.72,
                    metallic: 0.12,
                },
                source,
            )
            .unwrap()
            .snapshot
            .source;
        let source = h
            .authoring_edit(
                AuthoringEdit::Transform {
                    id: "station".into(),
                    translation: [0.05, 0.0, 0.0],
                    rotation: [0.0; 3],
                    scale: [1.0; 3],
                },
                source,
            )
            .unwrap()
            .snapshot
            .source;
        let stale = source.clone();
        let source = h
            .authoring_edit(
                AuthoringEdit::Parameter {
                    key: "width".into(),
                    value: 2.8,
                },
                source,
            )
            .expect("real dimensions must regenerate after material/TRS overrides")
            .snapshot
            .source;
        check_view(&h);
        assert!(h
            .authoring_edit(
                AuthoringEdit::Parameter {
                    key: "width".into(),
                    value: 3.0
                },
                stale
            )
            .is_err());
        let compiled = h
            .compiled_assets("after", source.clone())
            .unwrap()
            .remove(0)
            .1;
        assert!(
            compiled.mesh.max[0] - compiled.mesh.min[0] > mesh_before.max[0] - mesh_before.min[0],
            "actual native geometry must reflect changed width"
        );
        h.viewport_action(
            ViewportCommand::Orbit { dx: 8.0, dy: 4.0 },
            source.clone(),
            &cancel,
        )
        .unwrap();
        let hatch = h
            .viewport_action(
                ViewportCommand::FramePart {
                    key: "cabinet/front-hatch".into(),
                },
                source.clone(),
                &cancel,
            )
            .unwrap();
        let picked = h
            .viewport_action(
                ViewportCommand::Pick {
                    frame_id: hatch.state.frame_id,
                    x: 256.0,
                    y: 256.0,
                },
                source.clone(),
                &cancel,
            )
            .unwrap();
        assert_eq!(
            picked.state.selected_part.as_deref(),
            Some("cabinet/front-hatch")
        );
        diagnostic("r9-reference-picked-hatch.png", &picked.frame);
        let cells = h.source_cells(source.clone()).unwrap();
        assert_eq!(cells.len(), 5);
        assert_eq!(cells[0], master);
        std::fs::write(root.join("accepted-source.vibe"),cells.join("

")).unwrap();
        h.compile_checkpoint(HMC, "diagnostic-equipment", ExpectedFile::Absent, source)
            .unwrap();
        let observed = serde_json::json!({"fixture_only":true,"production_acceptance":"unassessed-HOLD","master_sha256":PIN,"source_cells":cells.len(),"parts":before.parts.len(),"vertices":compiled.mesh.positions.len(),"triangles":compiled.mesh.triangles.len(),"compiled_sha256":hash(&compiled.container_10d),"history_sha256":hash(cells.join("\n\n").as_bytes()),"hmc_bytes":std::fs::metadata(root.join(HMC)).unwrap().len(),"elapsed_ms":timer.elapsed().as_millis()});
        std::fs::write(
            root.join("observed.json"),
            serde_json::to_vec(&observed).unwrap(),
        )
        .unwrap();
        println!("{observed}");
    } else {
        assert_eq!(phase, "read");
        let expected: serde_json::Value =
            serde_json::from_slice(&std::fs::read(root.join("observed.json")).unwrap()).unwrap();
        let service = service(&root);
        let prepared = service.prepare_reopen(HMC).unwrap();
        let resource_count = prepared.resources().len();
        assert!(resource_count > 1);
        let exact = prepared.artifact().clone();
        service.activate(prepared).unwrap();
        let mut h = host(&root);
        let held = h.snapshot().unwrap();
        assert!(held.reconciliation_required);
        let source = h.rebuild_execution(held.source).unwrap().snapshot.source;
        assert_eq!(studio_store::get_artifact_stamped(ID).unwrap().0, exact);
        check_view(&h);
        let cells = h.source_cells(source.clone()).unwrap();
        assert_eq!(cells[0], master);
        assert_eq!(
            serde_json::Value::String(hash(cells.join("\n\n").as_bytes())),
            expected["history_sha256"]
        );
        let compiled = h
            .compiled_assets("reopened", source.clone())
            .unwrap()
            .remove(0)
            .1;
        assert_eq!(
            serde_json::Value::String(hash(&compiled.container_10d)),
            expected["compiled_sha256"]
        );
        h.viewport_action(super::viewport::preview_lighting(super::viewport::ViewLighting::neutral_studio()),source.clone(),&cancel).unwrap();
        let frame = h
            .viewport_action(ViewportCommand::FrameAll, source, &cancel)
            .unwrap();
        diagnostic("r9-reference-reopened.png", &frame.frame);
        let reopened=serde_json::json!({"fixture_only":true,"production_acceptance":"unassessed-HOLD","phase":"fresh-reopen","source_cells":cells.len(),"parts":h.authoring_view(frame.state.source).unwrap().parts.len(),"resources":resource_count,"elapsed_ms":timer.elapsed().as_millis()});
        std::fs::write(root.join("reopened.json"), serde_json::to_vec(&reopened).unwrap()).unwrap();
        println!("{reopened}");
    }
    assert_eq!(
        hash(&std::fs::read(MASTER).unwrap()),
        PIN,
        "master remains read-only"
    );
}
