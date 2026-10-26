use super::*;
use crate::graphical::{
    controller::{CanonicalEditorHost, EditorHost, PreviewProvider},
    geometry_host::GeometryHost,
    preview::{PreviewFrame, PreviewLimits, SourceRef},
};
use qualia_core_db::poet_host::invoke::studio_store::{self, StudioArtifact};
use vibe::{session::SessionLimits, Value};
struct NoPixels;
impl PreviewProvider for NoPixels {
    fn validate(&self, value: &Value) -> Result<(), String> {
        if crate::graphical::assembly::is_assembly(value) {
            crate::graphical::assembly::admit_assembly(value, PreviewLimits::default()).map(|_| ())
        } else {
            Ok(())
        }
    }
    fn preview(&mut self, _: SourceRef, _: &Value) -> Result<PreviewFrame, String> {
        Err("No GPU requested by this source/persistence test".into())
    }
}
pub(super) const MATERIAL: &str = r#"{id:1,colour_space:"linear",base_color:[0.3,0.4,0.5,1.0],emissive:[0.0,0.0,0.0],metallic:0.1,roughness:0.5,specular:1.0,alpha_cutoff:0.5,opacity_mode:"Opaque",double_sided:true}"#;
pub(super) fn source() -> String {
    format!(
        r#"
pure fn poet_rebuild_parts(p:Record)->List<Record>{{
 let mesh={{vertices:[[0.0,0.0,0.0],[1.0,0.0,0.0],[0.0,1.0,0.0]],triangles:[[0,1,2]]}};
 return [{{id:"body",semantic_id:101,geometry:mesh,material:{MATERIAL},translation:[p.offset,0.0,0.0],rotation:[0.0,0.0,0.0],scale:[1.0,1.0,1.0]}},
 {{id:"lever",semantic_id:102,parent:"body",geometry:mesh,material:{MATERIAL},translation:[0.0,1.0,0.0],rotation:[0.0,0.0,0.0],scale:[1.0,1.0,1.0]}}];
}}
let mut poet_parameters={{offset:0.0,channels:6}};
let mut poet_parameter_parts=poet_rebuild_parts(poet_parameters);
let mut poet_parts=poet_parameter_parts;
({{parts:poet_parts}});
"#
    )
}
pub(super) fn host() -> CanonicalEditorHost {
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
    let id = format!(
        "urn:poet:authoring-test:{}:{}",
        std::process::id(),
        NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    );
    studio_store::create_artifact_if_absent(StudioArtifact::new(
        &id,
        "Geometry",
        "Authoring source test",
    ))
    .unwrap();
    CanonicalEditorHost::open(
        id,
        SessionLimits::default(),
        GeometryHost::default(),
        Box::new(NoPixels),
    )
    .unwrap()
}
#[test]
fn every_generated_edit_is_a_canonical_program_with_a_record_result() {
    let fixture = source();
    vibe::parse_program(&fixture).expect("authoring fixture must be canonical Vibe 0.1");
    let mut session = vibe::session::SourceSession::default();
    let mut local = vibe::LocalHost::default();
    session.submit(&fixture, &mut local, vibe::Budget::default()).unwrap();
    let Value::List(initial_parts) = session.value("poet_parts").unwrap() else {
        panic!("fixture parts");
    };
    let initial_geometry = initial_parts.iter().map(|part| {
        let Value::Record(part) = part else { panic!("fixture part record") };
        part.get("geometry").unwrap().clone()
    }).collect::<Vec<_>>();
    let edits = [
        AuthoringEdit::Parameter { key: "offset".into(), value: 1.25 },
        AuthoringEdit::Transform {
            id: "body".into(), translation: [2.0, 0.0, 0.0],
            rotation: [0.0; 3], scale: [1.0; 3],
        },
        AuthoringEdit::Material {
            material_id: 1, base_color: [0.2, 0.3, 0.4, 1.0],
            roughness: 0.8, metallic: 0.0,
        },
    ];
    for (index, edit) in edits.iter().enumerate() {
        let generated = super::source::edit_source_with_baseline(
            session.value("poet_parts").unwrap(), session.value("poet_parameters"), session.value("poet_parameter_parts"),
            edit, index as u64 + 2,
        ).unwrap();
        let program = vibe::parse_program(&generated)
            .unwrap_or_else(|error| panic!("{edit:?}: {error:?}\n{generated}"));
        assert!(matches!(program.items.last(),
            Some(vibe::Item::Statement(vibe::Stmt::Expr { expr, .. }))
                if matches!(&expr.kind, vibe::ExprKind::Record(_))),
            "generated edit must end with a record expression: {edit:?}");
        let candidate = session.prepare(&generated)
            .unwrap_or_else(|error| panic!("generated edit preparation: {edit:?}: {error:?}"));
        let result = session.execute(candidate, &mut local, vibe::Budget::default())
            .unwrap_or_else(|error| panic!("generated edit execution: {edit:?}: {error:?}"));
        assert_eq!(session.revision(), index as u64 + 2);
        let Value::Record(output) = result.value else { panic!("edit record output") };
        assert_eq!(output.get("parts"), session.value("poet_parts"));
        let Value::Record(parameters) = session.value("poet_parameters").unwrap() else {
            panic!("retained parameters");
        };
        assert_eq!(parameters.get("offset"), Some(&Value::F64(1.25)));
        assert_eq!(parameters.get("channels"), Some(&Value::I64(6)));
        let Value::List(parts) = session.value("poet_parts").unwrap() else { panic!("parts") };
        assert_eq!(parts.len(), 2);
        for (part_index, part) in parts.iter().enumerate() {
            let Value::Record(part) = part else { panic!("part record") };
            assert_eq!(part.get("geometry"), Some(&initial_geometry[part_index]));
            assert_eq!(part.get("id"), Some(&Value::String(if part_index == 0 { "body" } else { "lever" }.into())));
            if part_index == 0 {
                assert!(matches!(part.get("parent"), None | Some(Value::Null)));
                let x = if index == 0 { 1.25 } else { 2.0 };
                assert_eq!(part.get("translation"), Some(&Value::List(vec![Value::F64(x), Value::F64(0.0), Value::F64(0.0)])));
            } else {
                assert_eq!(part.get("parent"), Some(&Value::String("body".into())));
                assert_eq!(part.get("translation"), Some(&Value::List(vec![Value::F64(0.0), Value::F64(1.0), Value::F64(0.0)])));
            }
            let Value::Record(material) = part.get("material").unwrap() else { panic!("material") };
            assert_eq!(material.get("id"), Some(&Value::I64(1)));
            assert_eq!(material.get("roughness"), Some(&Value::F64(if index == 2 { 0.8 } else { 0.5 })));
            assert_eq!(material.get("metallic"), Some(&Value::F64(if index == 2 { 0.0 } else { 0.1 })));
            if index == 2 {
                assert_eq!(material.get("base_color"), Some(&Value::List(vec![Value::F64(0.2), Value::F64(0.3), Value::F64(0.4), Value::F64(1.0)])));
            }
        }
    }
}
#[test]
fn part_material_parameter_edits_are_real_source_revisions_with_stale_safety() {
    let mut h = host();
    let first = h.snapshot().unwrap();
    h.submit(&source(), first.source).unwrap();
    let initial = h.snapshot().unwrap();
    let before = h.authoring_view(initial.source.clone()).unwrap();
    assert_eq!(before.parts.len(), 2);
    let out = h
        .authoring_edit(
            AuthoringEdit::Transform {
                id: "body".into(),
                translation: [2.0, 0.0, 0.0],
                rotation: [0.0; 3],
                scale: [1.0; 3],
            },
            initial.source.clone(),
        )
        .unwrap();
    let moved = h.authoring_view(out.snapshot.source.clone()).unwrap();
    assert_eq!(moved.parts[0].translation, [2.0, 0.0, 0.0]);
    assert_eq!(moved.parts[0].parent, None);
    assert_eq!(moved.parts[1].parent.as_deref(), Some("body"));
    let revision = out.snapshot.source_revision;
    assert!(h
        .authoring_edit(
            AuthoringEdit::Parameter {
                key: "offset".into(),
                value: 5.0
            },
            initial.source
        )
        .is_err());
    assert_eq!(h.snapshot().unwrap().source_revision, revision);
    let out = h
        .authoring_edit(
            AuthoringEdit::Material {
                material_id: 1,
                base_color: [0.2, 0.3, 0.4, 1.0],
                roughness: 0.8,
                metallic: 0.0,
            },
            out.snapshot.source,
        )
        .unwrap();
    let material = h.authoring_view(out.snapshot.source.clone()).unwrap();
    assert!(material.parts.iter().all(|p| p.roughness == 0.8));
    assert_eq!(material.parts[0].parent, None);
    assert_eq!(material.parts[1].parent.as_deref(), Some("body"));
    let stable = h.snapshot().unwrap();
    let cells = h.source_cells(stable.source.clone()).unwrap();
    let accepted = h.authoring_edit(
        AuthoringEdit::Parameter { key:"offset".into(),value:3.5 },stable.source.clone()
    ).expect("source layers preserve downstream material and TRS edits");
    let view = h.authoring_view(accepted.snapshot.source.clone()).unwrap();
    assert_eq!(view.parts[0].translation[0],2.0);
    assert!(view.parts.iter().all(|p|p.roughness==0.8));
    assert_eq!(h.source_cells(accepted.snapshot.source.clone()).unwrap().len(),cells.len()+1);
    let held = h.undo(accepted.snapshot.source).unwrap();
    assert!(held.reconciliation_required);
    let re = h.rebuild_execution(held.source).unwrap();
    let state = h.authoring_view(re.snapshot.source).unwrap();
    assert_eq!(state.parts[0].translation[0], 2.0);
    assert!(state.parts.iter().all(|p| p.roughness == 0.8));
}
#[test]
fn compiled_hmc_replacement_preserves_opaque_resources_and_native_companion_graph() {
    use crate::graphical::checkpoints::*;
    let temp = tempfile::tempdir().unwrap();
    let mut raw = host();
    let initial = raw.snapshot().unwrap();
    raw.submit(&source(), initial.source).unwrap();
    let binding = TrustedBinding {
        project_id: "urn:poet:test-project".into(),
        document_id: "urn:poet:test-document".into(),
        source_id: raw.snapshot().unwrap().source.artifact_id,
        domain: "Geometry".into(),
    };
    let service =
        CheckpointService::new(temp.path(), binding, CheckpointLimits::default()).unwrap();
    let mut h = CheckpointEditorHost::new(Box::new(raw), service).unwrap();
    let current = h.snapshot().unwrap();
    let first = h
        .compile_checkpoint(
            "derived.hmc",
            "test-assembly",
            ExpectedFile::Absent,
            current.source.clone(),
        )
        .unwrap();
    let opaque = ResourceData {
        key: "source/unknown.bin".into(),
        kind: "future/schema".into(),
        bytes: vec![0, 42, 255],
        meta: Some(vec![0xa1, 0x61, b'x', 1]),
    };
    let saved = h
        .edit_package_resources(
            "derived.hmc",
            ExpectedFile::Sha256(first.digest),
            vec![ResourceEdit::Add(opaque.clone())],
            current.source.clone(),
        )
        .unwrap();
    let moved = h
        .authoring_edit(
            AuthoringEdit::Transform {
                id: "body".into(),
                translation: [4.0, 0.0, 0.0],
                rotation: [0.0; 3],
                scale: [1.0; 3],
            },
            current.source,
        )
        .unwrap();
    let before_refusal = std::fs::read(temp.path().join("derived.hmc")).unwrap();
    assert!(h
        .compile_checkpoint(
            "derived.hmc",
            "test-assembly",
            ExpectedFile::Sha256([0; 32]),
            moved.snapshot.source.clone()
        )
        .is_err());
    assert_eq!(
        std::fs::read(temp.path().join("derived.hmc")).unwrap(),
        before_refusal
    );
    assert!(h.compile_checkpoint("derived.hmc","test-assembly",ExpectedFile::Sha256(saved.digest),moved.snapshot.source.clone()).unwrap_err().contains("new asset key"));
    assert_eq!(std::fs::read(temp.path().join("derived.hmc")).unwrap(),before_refusal,"same-key rebuild must never erase old sections/graph facts");
    let old=qualia_core_db::poet_host::invoke::studio_store::checkpoint::load(&temp.path().join("derived.hmc"),CheckpointLimits::default()).unwrap();
    let old_resources=old.resources().to_vec();drop(old);
    let second = h
        .compile_checkpoint(
            "derived.hmc",
            "test-assembly-v2",
            ExpectedFile::Sha256(saved.digest),
            moved.snapshot.source,
        )
        .unwrap();
    assert_ne!(saved.digest, second.digest);
    let loaded = qualia_core_db::poet_host::invoke::studio_store::checkpoint::load(
        &temp.path().join("derived.hmc"),
        CheckpointLimits::default(),
    )
    .unwrap();
    assert!(loaded.resources().iter().any(|r| r == &opaque));
    for old in old_resources {assert!(loaded.resources().iter().any(|r|r==&old),"every prior resource/metadata must remain exact under new-key compilation");}
    assert!(loaded.resources().iter().any(|r| r.kind == "q42"));
    assert!(loaded.resources().iter().any(|r| r.kind == "10d"));
}
