//! Synthetic signed project/checkpoint rig test; not a GPU or human trial.
use super::*;
use crate::graphical::{
    checkpoints::ResourceInspectionRequest,
    rig_binding::{self, BindMesh, RigControl, RigSourceDescriptor, ViewportSpan},
};
use qualia_core_db::{
    container_10d::{
        skeletal_container::attach_skeletal_section_into,
        skeletal_section::{encode_skeletal_section_into, ClipRecord, SkeletalData},
    },
    render::{assets::Mesh, compile_10d::compile_mesh_to_10d_with_nodes,
        skeletal::{Joint, Key, Motor, SkinWeight, Track}},
};
use std::sync::atomic::AtomicBool;

const CHILD: &str = "POET_SIGNED_RIG_REOPEN_TEST";
const KEY: &str = "resources/synthetic-rig.10d";

struct RigFixturePreview;
impl PreviewProvider for RigFixturePreview {
    // This fixture does not retain a viewport or rendered frame.
    fn retire_retained_viewport(&mut self) -> Result<(), String> { Ok(()) }
    fn validate(&self, value: &Value) -> Result<(), String> {
        crate::graphical::assembly::admit_assembly(value, PreviewLimits::default()).map(|_| ())
    }
    fn preview(&mut self, _: SourceRef, _: &Value) -> Result<PreviewFrame, String> {
        Err("synthetic fixture deliberately renders no GPU frame".into())
    }
    fn stage_rig_control(&mut self, source: SourceRef, descriptor: RigSourceDescriptor,
        bytes: &[u8], action: RigControl, cancelled: &AtomicBool)
        -> Result<rig_binding::rig_candidate::StagedRigControl, String> {
        if cancelled.load(Ordering::Acquire) { return Err("cancelled".into()); }
        let positions = [[0.0,0.0,0.0],[1.0,0.0,0.0],[0.0,1.0,0.0]];
        let normals = [[0.0,0.0,1.0]; 3];
        let uv = [[0.0,0.0],[1.0,0.0],[0.0,1.0]];
        let indices = [0,1,2];
        let key = descriptor.resource_key.clone();
        rig_binding::rig_candidate::stage_rig_control(source, descriptor, &key, bytes,
            BindMesh { positions:&positions, normals:&normals, uv0:&uv, indices:&indices },
            ViewportSpan { first_vertex:0, vertex_count:3 }, 42 * 1024 * 1024, action)
    }
}

fn fixture(source_id: &str) -> (Vec<u8>, RigSourceDescriptor) {
    let positions = vec![[0.0,0.0,0.0],[1.0,0.0,0.0],[0.0,1.0,0.0]];
    let normals = vec![[0.0,0.0,1.0]; 3];
    let uv = vec![[0.0,0.0],[1.0,0.0],[0.0,1.0]];
    let indices = vec![0,1,2];
    let mesh = Mesh { positions:positions.clone(), triangles:vec![[0,1,2]], min:[0.0;3], max:[1.0,1.0,0.0] };
    let base = compile_mesh_to_10d_with_nodes(&mesh, &[]).unwrap();
    let joint_id=qualia_core_db::q_hash("urn:test:composed-rig:root");
    let clip_id=qualia_core_db::q_hash("urn:test:composed-rig:clip");
    let joints=[Joint { parent:None, rest_local:Motor::identity(), inverse_bind:Motor::identity() }];
    let skins=[SkinWeight { joints:[0;4], weights:[1.0,0.0,0.0,0.0] };3];
    let clips=[ClipRecord { id:clip_id, duration:1.0, first_track:0, track_count:1, looping:false }];
    let tracks=[Track { joint:0, first_key:0, key_count:2 }];
    let keys=[Key { time:0.0, local:Motor::identity() }, Key { time:1.0, local:Motor::from_translation(1.0,0.0,0.0) }];
    let data=SkeletalData { joints:&joints, joint_ids:&[joint_id], skin:&skins, clips:&clips, tracks:&tracks, keys:&keys };
    let mut payload=vec![0;432]; let n=encode_skeletal_section_into(&data,&mut payload).unwrap(); payload.truncate(n);
    let mut bytes=vec![0;8192]; let n=attach_skeletal_section_into(&base,&payload,&mut bytes).unwrap(); bytes.truncate(n);
    let desc=RigSourceDescriptor { schema:"poet.bound-rig/v1".into(), source_artifact_id:source_id.into(),
        part_id:"synthetic-part".into(), bind_id:"rest-v1".into(), resource_key:KEY.into(),
        resource_sha256:rig_binding::resource_sha256(&bytes),
        bind_mesh_sha256:rig_binding::bind_mesh_sha256(&positions,&indices),
        normal_uv_sha256:rig_binding::normal_uv_sha256(&normals,&uv),
        joint_ids:vec![joint_id], clip_ids:vec![clip_id], selected_clip_id:clip_id, seek_seconds:0.0,
        joint_deltas:vec![], vertex_lineage:vec![0,1,2] };
    (bytes,desc)
}

fn reopen(root: &Path) {
    let witness: serde_json::Value=serde_json::from_slice(&std::fs::read(root.join("witness.json")).unwrap()).unwrap();
    let scope=Scope { project_id:witness["project_id"].as_str().unwrap().into(),
        document_id:witness["document_id"].as_str().unwrap().into(),
        source_id:witness["source_id"].as_str().unwrap().into(), domain:"Geometry".into() };
    assert!(studio_store::get_artifact_stamped(&scope.source_id).is_none());
    let owner=Arc::new(NativeAuthority::open_retained_current(&root.join("authority"),
        TrustedProject { project_id:scope.project_id.clone(), issuer_identifier:"fixture-issuer".into(),
            public_key:ed25519::public_from_seed(&AUTHORITY) },1024*1024,256).unwrap());
    let session=login(&owner,"fixture-author",&AUTHOR);
    let project=NativeProject::open(config(scope.clone(),&root.join("hmc"),&root.join("library")),
        owner,session.clone(),Some("saved.hmc"),Box::new(RigFixturePreview)).unwrap();
    let mut host=project.connect(session).unwrap();
    let held=host.snapshot().unwrap();
    if held.reconciliation_required { host.rebuild_execution(held.source).unwrap(); }
    let current=host.snapshot().unwrap();
    assert_eq!(host.authoring_view(current.source).unwrap().rig.unwrap().seek_seconds,1.0);
    let loaded=checkpoint::load(&root.join("hmc/saved.hmc"),CheckpointLimits::default()).unwrap();
    assert!(loaded.resources().iter().any(|r| r.key==KEY && r.kind=="10d"));
    std::fs::write(root.join("rig-child-ok"),b"reopened").unwrap();
}

#[test]
fn signed_native_rig_selection_edit_refusals_and_fresh_checkpoint_reopen() {
    if let Some(root)=std::env::var_os(CHILD) { reopen(Path::new(&root)); return; }
    let temp=tempfile::tempdir().unwrap();
    {
        for name in ["authority","hmc","library"] { std::fs::create_dir(temp.path().join(name)).unwrap(); }
        let (scope,_,authority)=super::fixture(&temp.path().join("authority"));
        let session=login(&authority,"fixture-author",&AUTHOR);
        let project=NativeProject::open(config(scope.clone(),&temp.path().join("hmc"),&temp.path().join("library")),
            authority.clone(),session.clone(),None,Box::new(RigFixturePreview)).unwrap();
        let mut host=project.connect(session.clone()).unwrap();
        let (bytes,desc)=fixture(&scope.source_id);
        let literal=rig_binding::descriptor_source_literal(&desc).unwrap();
        let quoted=serde_json::to_string(&literal).unwrap();
        let source=format!(r#"let m={{id:1,colour_space:"linear",base_color:[0.3,0.4,0.5,1.0],emissive:[0.0,0.0,0.0],metallic:0.1,roughness:0.5,specular:1.0,alpha_cutoff:0.5,opacity_mode:"Opaque",double_sided:true}}; let tri={{vertices:[[0.0,0.0,0.0],[1.0,0.0,0.0],[0.0,1.0,0.0]],triangles:[[0,1,2]]}}; let mut poet_parts=[{{id:"synthetic-part",semantic_id:101,geometry:tri,material:m,translation:[0.0,0.0,0.0],rotation:[0.0,0.0,0.0],scale:[1.0,1.0,1.0]}}]; let mut poet_rig={quoted}; ({{parts:poet_parts}});"#);
        let initial=host.snapshot().unwrap();
        let accepted=host.submit(&source,initial.source).unwrap().snapshot;
        let saved=host.save_checkpoint("saved.hmc",ExpectedFile::Absent,accepted.source.clone()).unwrap();
        let resource=ResourceData { key:KEY.into(),kind:"10d".into(),bytes:bytes.clone(),meta:None };
        let saved=host.edit_package_resources("saved.hmc",ExpectedFile::Sha256(saved.digest),
            vec![ResourceEdit::Add(resource)],accepted.source.clone()).unwrap();
        let request=ResourceInspectionRequest { filename:"saved.hmc".into(),package_digest:saved.digest,
            key:KEY.into(),resource_digest:Sha256::digest(&bytes).into() };
        let cancelled=AtomicBool::new(true);
        let before=host.snapshot().unwrap().source;
        assert!(host.rig_authoring_edit(request.clone(),RigControl::Seek {seconds:1.0},before.clone(),&cancelled).is_err());
        assert_eq!(host.snapshot().unwrap().source,before);
        cancelled.store(false,Ordering::Release);
        let mut bad=request.clone(); bad.resource_digest[0]^=1;
        assert!(host.rig_authoring_edit(bad,RigControl::Seek {seconds:1.0},before.clone(),&cancelled).is_err());
        assert_eq!(host.snapshot().unwrap().source,before);
        let after=host.rig_authoring_edit(request.clone(),RigControl::Seek {seconds:1.0},before.clone(),&cancelled).unwrap().snapshot;
        assert_eq!(host.authoring_view(after.source.clone()).unwrap().rig.unwrap().seek_seconds,1.0);
        assert!(host.rig_authoring_edit(request,RigControl::Seek {seconds:0.0},before,&cancelled).is_err());
        let final_source=after.source;
        let saved=host.save_checkpoint("saved.hmc",ExpectedFile::Sha256(saved.digest),final_source.clone()).unwrap();
        let loaded=checkpoint::load(&saved.path,CheckpointLimits::default()).unwrap();
        assert!(loaded.resources().iter().any(|r| r.key==KEY && r.bytes==bytes));
        let saved_request=ResourceInspectionRequest { filename:"saved.hmc".into(),package_digest:saved.digest,
            key:KEY.into(),resource_digest:Sha256::digest(&bytes).into() };
        authority.disconnect(&session).unwrap();
        assert!(host.rig_authoring_edit(saved_request,RigControl::Seek {seconds:0.0},final_source.clone(),&cancelled).is_err());
        assert_eq!(studio_store::get_artifact_stamped(&scope.source_id).unwrap().1,final_source.stamp);
        std::fs::write(temp.path().join("witness.json"),serde_json::to_vec(&json!({"project_id":scope.project_id,
            "document_id":scope.document_id,"source_id":scope.source_id})).unwrap()).unwrap();
        drop(host); drop(project); drop(authority);
    }
    let name=std::thread::current().name().unwrap().to_owned();
    let output=std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact",name.as_str(),"--nocapture","--test-threads=1"]).env(CHILD,temp.path()).output().unwrap();
    assert!(output.status.success(),"rig reopen child failed: {}\n{}\n{}",output.status,
        String::from_utf8_lossy(&output.stdout),String::from_utf8_lossy(&output.stderr));
    assert!(temp.path().join("rig-child-ok").is_file(),"zero-test child is not evidence");
}

/// Signed native-project integration using a real retained GPU and HMC resource.
/// This is a software-agent fixture, not human or asset acceptance evidence.
#[test]
#[ignore = "requires explicit native GPU qualification"]
fn signed_mounted_rig_preview_reads_hmc_steps_restores_and_rejects_stale_context() {
    use crate::graphical::{
        controller::MeshPreviewProvider,
        preview::PreviewCamera,
        viewport::ViewportCommand,
        viewport_provider::InteractivePreviewProvider,
    };
    let temp = tempfile::tempdir().unwrap();
    for name in ["authority", "hmc", "library"] {
        std::fs::create_dir(temp.path().join(name)).unwrap();
    }
    let (scope, _, authority) = super::fixture(&temp.path().join("authority"));
    let session = login(&authority, "fixture-author", &AUTHOR);
    let provider = InteractivePreviewProvider::new(MeshPreviewProvider {
        material: MaterialRecord::legacy_default(),
        camera: PreviewCamera { yaw: 0.0, pitch: 0.0, distance: 4.0, target: [0.4, 0.4, 0.0] },
        limits: PreviewLimits { side: 64, vertices: 64, indices: 128 },
    }).unwrap();
    let project = NativeProject::open(config(scope.clone(), &temp.path().join("hmc"), &temp.path().join("library")),
        authority.clone(), session.clone(), None, Box::new(provider)).unwrap();
    let mut host = project.connect(session.clone()).unwrap();
    let (bytes, desc) = fixture(&scope.source_id);
    let quoted = serde_json::to_string(&rig_binding::descriptor_source_literal(&desc).unwrap()).unwrap();
    let source_text = format!(r#"let m={{id:1,colour_space:"linear",base_color:[0.8,0.15,0.08,1.0],emissive:[0.0,0.0,0.0],metallic:0.1,roughness:0.5,specular:1.0,alpha_cutoff:0.5,opacity_mode:"Opaque",double_sided:true}}; let tri={{vertices:[[0.0,0.0,0.0],[1.0,0.0,0.0],[0.0,1.0,0.0]],triangles:[[0,1,2]],normals:[[0.0,0.0,1.0],[0.0,0.0,1.0],[0.0,0.0,1.0]],uv0:[[0.0,0.0],[1.0,0.0],[0.0,1.0]]}}; let mut poet_parts=[{{id:"synthetic-part",semantic_id:101,geometry:tri,material:m,translation:[0.0,0.0,0.0],rotation:[0.0,0.0,0.0],scale:[1.0,1.0,1.0]}}]; let mut poet_rig={quoted}; ({{parts:poet_parts}});"#);
    let initial = host.snapshot().unwrap();
    let current = host.submit(&source_text, initial.source).unwrap().snapshot.source;
    let saved = host.save_checkpoint("saved.hmc", ExpectedFile::Absent, current.clone()).unwrap();
    let saved = host.edit_package_resources("saved.hmc", ExpectedFile::Sha256(saved.digest),
        vec![ResourceEdit::Add(ResourceData { key: KEY.into(), kind: "10d".into(), bytes: bytes.clone(), meta: None })], current.clone()).unwrap();
    let request = ResourceInspectionRequest { filename: "saved.hmc".into(), package_digest: saved.digest,
        key: KEY.into(), resource_digest: Sha256::digest(&bytes).into() };
    let cancel = AtomicBool::new(false);
    let rest = host.viewport_action(ViewportCommand::Redraw, current.clone(), &cancel).unwrap();
    assert_eq!(rest.frame.rgba8.len(), 64 * 64 * 4);
    assert!(rest.frame.rgba8.chunks_exact(4).any(|p| p[0] != p[1] || p[1] != p[2]), "GPU readback lacks rendered colour");
    let selected = host.viewport_action(ViewportCommand::Pick { frame_id: rest.state.frame_id, x: 32.0, y: 32.0 }, current.clone(), &cancel).unwrap();
    assert_eq!(selected.state.selected_part.as_deref(), Some("synthetic-part"));
    let stale = host.viewport_action(ViewportCommand::Pick { frame_id: rest.state.frame_id, x: 32.0, y: 32.0 }, current.clone(), &cancel).err().unwrap();
    assert!(stale.contains("viewport reset"), "stale pick refusal must explain recovery: {stale}");
    let after_stale = host.viewport_action(ViewportCommand::Redraw, current.clone(), &cancel).unwrap();
    assert!(after_stale.state.selected_part.is_none());
    let after_stale_pick = host.viewport_action(ViewportCommand::Pick { frame_id: after_stale.state.frame_id, x: 32.0, y: 32.0 }, current.clone(), &cancel).unwrap();
    assert_eq!(after_stale_pick.state.selected_part.as_deref(), Some("synthetic-part"));
    let mut bad = request.clone(); bad.resource_digest[0] ^= 1;
    let bad_error = host.viewport_action(ViewportCommand::RigPreview { resource: bad, delta_seconds: 0.0 }, current.clone(), &cancel).err().unwrap();
    assert!(bad_error.contains("viewport reset"), "bad-resource refusal must explain recovery: {bad_error}");
    let after_bad = host.viewport_action(ViewportCommand::Redraw, current.clone(), &cancel).unwrap();
    assert!(after_bad.state.selected_part.is_none());
    let after_bad_pick = host.viewport_action(ViewportCommand::Pick { frame_id: after_bad.state.frame_id, x: 32.0, y: 32.0 }, current.clone(), &cancel).unwrap();
    assert_eq!(after_bad_pick.state.selected_part.as_deref(), Some("synthetic-part"));
    cancel.store(true, Ordering::Release);
    assert!(host.viewport_action(ViewportCommand::RigPreview { resource: request.clone(), delta_seconds: 0.0 }, current.clone(), &cancel).is_err(), "cancelled preview must be refused");
    cancel.store(false, Ordering::Release);
    let pose = host.viewport_action(ViewportCommand::RigPreview { resource: request.clone(), delta_seconds: 0.0 }, current.clone(), &cancel).unwrap();
    assert_eq!(pose.state.selected_part.as_deref(), Some("synthetic-part"));
    let unhighlighted_rest = host.viewport_action(ViewportCommand::ClearSelection, current.clone(), &cancel).unwrap();
    assert!(unhighlighted_rest.state.selected_part.is_none());
    let reselected = host.viewport_action(ViewportCommand::Pick { frame_id: unhighlighted_rest.state.frame_id, x: 32.0, y: 32.0 }, current.clone(), &cancel).unwrap();
    assert_eq!(reselected.state.selected_part.as_deref(), Some("synthetic-part"));
    let stepped = host.viewport_action(ViewportCommand::RigPreview { resource: request.clone(), delta_seconds: 1.0 }, current.clone(), &cancel).unwrap();
    assert!(stepped.state.frame_id > reselected.state.frame_id);
    assert_eq!(stepped.state.selected_part.as_deref(), Some("synthetic-part"));
    let unhighlighted_step = host.viewport_action(ViewportCommand::ClearSelection, current.clone(), &cancel).unwrap();
    assert!(unhighlighted_step.state.selected_part.is_none());
    let changed = unhighlighted_rest.frame.rgba8.chunks_exact(4).zip(unhighlighted_step.frame.rgba8.chunks_exact(4)).filter(|(a,b)| a != b).count();
    assert!(changed > 10, "GPU readback must show skeletal motion, changed pixels={changed}");
    assert_eq!(host.snapshot().unwrap().source, current, "preview must not mutate source");
    let restored = host.viewport_action(ViewportCommand::RestoreSourceGeometry, current.clone(), &cancel).unwrap();
    assert!(restored.state.selected_part.is_none());
    assert_eq!(restored.frame.rgba8, rest.frame.rgba8, "restore must reload the source geometry and read it back");
    assert!(host.viewport_action(ViewportCommand::RigPreview { resource: request.clone(), delta_seconds: 0.0 }, current.clone(), &cancel).is_err(), "preview requires the current selection");
    let successor = host.submit("let rig_preview_test_successor=1; ({parts:poet_parts});", current.clone()).unwrap().snapshot.source;
    assert_ne!(successor, current);
    assert!(host.viewport_action(ViewportCommand::RigPreview { resource: request.clone(), delta_seconds: 0.0 }, current, &cancel).is_err(), "retired source must be refused");
    authority.disconnect(&session).unwrap();
    assert!(host.viewport_action(ViewportCommand::RigPreview { resource: request, delta_seconds: 0.0 }, successor, &cancel).is_err(), "disconnected authority must be refused");
}
