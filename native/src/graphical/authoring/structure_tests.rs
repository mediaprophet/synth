//! Temporary synthetic source only; no production asset or human acceptance claim.
use super::*;
use crate::graphical::{controller::EditorHost, geometry_host::{GeometryHost, GeometryHostLimits}};
use vibe::{session::{SourceSession, SessionLimits}, Budget, Value};

fn session(grants: Vec<String>) -> SourceSession {
    let mut session = SourceSession::new(grants, SessionLimits::default());
    session.submit(&super::tests::source(), &mut vibe::LocalHost::default(), Budget::default()).unwrap();
    session
}
fn parts(session: &SourceSession) -> Vec<Value> {
    let Value::List(parts) = session.value("poet_parts").unwrap() else { panic!("parts") };
    parts.clone()
}
fn field<'a>(value: &'a Value, key: &str) -> &'a Value {
    let Value::Record(record) = value else { panic!("record") };
    record.get(key).unwrap()
}
fn edit<H: vibe::Host>(session: &mut SourceSession, host: &mut H, edit: AuthoringEdit) -> Result<(), String> {
    let source = super::source::edit_source_with_baseline(session.value("poet_parts").unwrap(), session.value("poet_parameters"), session.value("poet_parameter_parts"), &edit, session.revision()+1)?;
    vibe::parse_program(&source).map_err(|e|format!("{e:?}"))?;
    session.submit(&source,host,Budget::default()).map(|_|()).map_err(|e|format!("{e:?}"))
}
fn sphere() -> GeometryRecipe { GeometryRecipe::Sphere { radius:0.25,lat_segments:8,lon_segments:12 } }
fn grants() -> Vec<String> { vec!["capability.invoke".into(),"ComputationalGeometry.uv_sphere".into(),"ComputationalGeometry.torus".into(),"ComputationalGeometry.profile_sweep".into()] }
fn geometry_host(grants: Vec<String>, dispatches: usize, work: usize) -> GeometryHost {
    GeometryHost::with_limits(grants,GeometryHostLimits {max_vertices:10000,submission_work:work,dispatches,scratch_bytes:16*1024*1024,profile_points:1024}).unwrap()
}

#[test]
fn reparent_detach_and_remove_leaf_preserve_local_trs_and_unrelated_values() {
    let mut session = session(Vec::new());
    let mut host = vibe::LocalHost::default();
    let before = parts(&session);
    edit(&mut session,&mut host,AuthoringEdit::Reparent {id:"lever".into(),parent:None}).unwrap();
    let detached = parts(&session);
    assert_eq!(field(&detached[1],"parent"),&Value::Null);
    for key in ["translation","rotation","scale","geometry","material","semantic_id"] { assert_eq!(field(&detached[1],key),field(&before[1],key)); }
    assert_eq!(detached[0],before[0]);
    edit(&mut session,&mut host,AuthoringEdit::Reparent {id:"body".into(),parent:Some("lever".into())}).unwrap();
    let attached = parts(&session);
    assert_eq!(field(&attached[0],"parent"),&Value::String("lever".into()));
    assert_eq!(field(&attached[0],"translation"),field(&before[0],"translation"));
    edit(&mut session,&mut host,AuthoringEdit::RemoveLeaf {id:"body".into()}).unwrap();
    assert_eq!(parts(&session),vec![detached[1].clone()]);
    assert!(edit(&mut session,&mut host,AuthoringEdit::RemoveLeaf {id:"lever".into()}).unwrap_err().contains("final"));
}
#[test]
fn assembly_mutations_refuse_cycles_missing_parents_and_nonleaf_removal_without_execution() {
    let mut session = session(Vec::new());
    let held = parts(&session);
    let revision = session.revision();
    for command in [
        AuthoringEdit::Reparent {id:"body".into(),parent:Some("lever".into())},
        AuthoringEdit::Reparent {id:"body".into(),parent:Some("body".into())},
        AuthoringEdit::Reparent {id:"lever".into(),parent:Some("missing".into())},
        AuthoringEdit::Reparent {id:"missing".into(),parent:None},
        AuthoringEdit::RemoveLeaf {id:"body".into()},
    ] { assert!(edit(&mut session,&mut vibe::LocalHost::default(),command).is_err()); }
    assert_eq!(session.revision(),revision);
    assert_eq!(parts(&session),held);
}
#[test]
fn unique_material_preserves_complete_material_and_only_selected_assignment() {
    let mut session = session(Vec::new());
    let mut host = vibe::LocalHost::default();
    let before = parts(&session);
    edit(&mut session,&mut host,AuthoringEdit::UniqueMaterial {id:"lever".into(),material_id:99}).unwrap();
    let unique = parts(&session);
    assert_eq!(unique[0],before[0]);
    let Value::Record(mut expected) = field(&before[1],"material").clone() else {panic!("material")};
    expected.insert("id".into(),Value::I64(99));
    assert_eq!(field(&unique[1],"material"),&Value::Record(expected));
    edit(&mut session,&mut host,AuthoringEdit::Material {material_id:99,base_color:[0.1,0.2,0.3,1.0],roughness:0.9,metallic:0.0}).unwrap();
    assert_eq!(parts(&session)[0],before[0]);
    assert_eq!(field(field(&parts(&session)[1],"material"),"roughness"),&Value::F64(0.9));
    assert!(edit(&mut session,&mut host,AuthoringEdit::UniqueMaterial {id:"lever".into(),material_id:1}).is_err());
}
#[test]
fn field_preserving_edits_keep_distinct_extension_schemas_and_refuse_unsafe_names() {
    let mut session = session(Vec::new());
    let mut value = Value::List(parts(&session));
    let Value::List(ref mut records) = value else {unreachable!()};
    for (i, part) in records.iter_mut().enumerate() {
        let Value::Record(p) = part else {panic!("part")};
        p.insert(format!("extension_{i}"),Value::String(format!("part {i}")));
        let Value::Record(m) = p.get_mut("material").unwrap() else {panic!("material")};
        m.insert(format!("texture_extension_{i}"),Value::List(vec![Value::String("retain".into())]));
    }
    // Inject extension data through canonical source so the generated expressions reference real values.
    let fixture=super::tests::source().replace("geometry:mesh,material:","extension_note:{raw:[1,2,3]},geometry:mesh,material:")
        .replace("double_sided:true}","double_sided:true,future_texture:{uri:\"urn:test:texture\",channels:[1,2,3]}}");
    session = SourceSession::default();
    session.submit(&fixture,&mut vibe::LocalHost::default(),Budget::default()).unwrap();
    let before = parts(&session);
    edit(&mut session,&mut vibe::LocalHost::default(),AuthoringEdit::Transform {id:"body".into(),translation:[0.123456789012345,0.0,0.0],rotation:[0.0;3],scale:[1.0;3]}).unwrap();
    edit(&mut session,&mut vibe::LocalHost::default(),AuthoringEdit::Material {material_id:1,base_color:[0.2,0.2,0.2,1.0],roughness:0.8,metallic:0.0}).unwrap();
    for (i, part) in parts(&session).iter().enumerate() {
        assert_eq!(field(part,"extension_note"),field(&before[i],"extension_note"));
        assert_eq!(field(field(part,"material"),"future_texture"),field(field(&before[i],"material"),"future_texture"));
    }
    assert_eq!(field(&parts(&session)[0],"translation"),&Value::List(vec![Value::F64(0.123456789012345),Value::F64(0.0),Value::F64(0.0)]));
    let generated=super::source::edit_source(&value,None,&AuthoringEdit::Material {material_id:1,base_color:[0.2;4],roughness:0.5,metallic:0.0},2).unwrap();
    assert!(generated.contains("texture_extension_0:p.material.texture_extension_0"));
    assert!(generated.contains("texture_extension_1:p.material.texture_extension_1"));
    let Value::List(records)=&mut value else {unreachable!()};
    let Value::Record(first)=&mut records[0] else {unreachable!()};
    first.insert("unsafe-field".into(),Value::Null);
    assert!(super::source::edit_source(&value,None,&AuthoringEdit::Reparent {id:"body".into(),parent:None},2).unwrap_err().contains("preserve"));
}
#[test]
fn bounded_geometry_recipes_add_and_replace_through_actual_grants_and_one_dispatch() {
    let mut session = session(grants());
    for (i, geometry) in [sphere(),GeometryRecipe::Torus {major_radius:0.5,minor_radius:0.1,major_segments:12,minor_segments:8},GeometryRecipe::CircularSweep {radius:0.1,length:1.0,segments:12,caps:true}].into_iter().enumerate() {
        let before = parts(&session);
        let mut host=geometry_host(grants(),1,10000);
        let command=AuthoringEdit::ReplaceGeometry {id:"lever".into(),geometry};
        let generated=super::source::edit_source(session.value("poet_parts").unwrap(),None,&command,session.revision()+1).unwrap();
        assert_eq!(generated.matches("capability.invoke(").count(),1,"exactly one kernel invocation must be authored");
        assert!(!generated.contains(".map("),"capability root must not be captured by a pinned 0.1 lambda");
        edit(&mut session,&mut host,command).unwrap();
        let after=parts(&session);
        assert_eq!(after[0],before[0]);
        for key in ["parent","translation","rotation","scale","material","id","semantic_id"] { assert_eq!(field(&after[1],key),field(&before[1],key)); }
        assert_ne!(field(&after[1],"geometry"),field(&before[1],"geometry"),"operator {i}");
        let Value::List(vertices)=field(field(&after[1],"geometry"),"vertices") else {panic!("vertices")};
        assert!(vertices.len()>8);
        // The successful edit consumed exactly the only admitted dispatch.
        assert!(edit(&mut session,&mut host,AuthoringEdit::ReplaceGeometry {id:"lever".into(),geometry:sphere()}).is_err());
    }
    let mut host=geometry_host(grants(),1,10000);
    let before=parts(&session);
    edit(&mut session,&mut host,AuthoringEdit::AddGeometry {id:"new-part".into(),semantic_id:103,material_from:"lever".into(),geometry:sphere()}).unwrap();
    let after=parts(&session);
    assert_eq!(&after[..2],before.as_slice());
    assert_eq!(field(&after[2],"material"),field(&before[1],"material"));
    assert!(matches!(after[2],Value::Record(ref p) if !p.contains_key("parent")));
    assert_eq!(field(&after[2],"scale"),&Value::List(vec![Value::F64(1.0);3]));
}
#[test]
fn geometry_edits_refuse_denied_effect_and_exhausted_work_without_advancing_source() {
    for (actual_grants,work,expected_code) in [(vec!["capability.invoke".to_string()],10000,"E300"),(grants(),1,"E400")] {
        let mut session=session(grants());
        let held=parts(&session);
        let revision=session.revision();
        let mut host=geometry_host(actual_grants,1,work);
        let error=edit(&mut session,&mut host,AuthoringEdit::ReplaceGeometry {id:"lever".into(),geometry:sphere()}).unwrap_err();
        assert!(error.contains(expected_code),"must reach the actual grant/work refusal: {error}");
        assert_eq!(session.revision(),revision);
        assert_eq!(parts(&session),held);
    }
}
#[test]
fn authoring_inventory_and_recipes_refuse_duplicates_injection_and_nonfinite_arguments() {
    let session=session(Vec::new());
    let mut invalid=Value::List(parts(&session));
    let Value::List(ref mut list)=invalid else {unreachable!()};
    list.push(list[0].clone());
    assert!(super::source::edit_source(&invalid,None,&AuthoringEdit::RemoveLeaf {id:"lever".into()},2).unwrap_err().contains("Duplicate"));
    for (id,semantic_id) in [("body",103),("bad\";injection",103),("new",101),("new",u64::MAX)] {
        assert!(super::source::edit_source(session.value("poet_parts").unwrap(),None,&AuthoringEdit::AddGeometry {id:id.into(),semantic_id,material_from:"body".into(),geometry:sphere()},2).is_err());
    }
    for geometry in [GeometryRecipe::Sphere {radius:f64::NAN,lat_segments:8,lon_segments:12},GeometryRecipe::Sphere {radius:0.0,lat_segments:8,lon_segments:12},GeometryRecipe::Sphere {radius:1.0,lat_segments:2,lon_segments:12},GeometryRecipe::Torus {major_radius:0.1,minor_radius:0.2,major_segments:8,minor_segments:8},GeometryRecipe::CircularSweep {radius:0.1,length:f64::INFINITY,segments:12,caps:true}] {
        assert!(geometry.validate().is_err());
    }
    assert!(serde_json::from_str::<GeometryRecipe>(r#"{"kind":"Sphere","radius":1.0,"lat_segments":8,"lon_segments":12,"code":"arbitrary"}"#).is_err());
}
#[test]
fn structural_source_revisions_keep_stale_guard_undo_and_rebuild_lineage() {
    let mut host=super::tests::host();
    let first=host.snapshot().unwrap();
    host.submit(&super::tests::source(),first.source).unwrap();
    let accepted=host.snapshot().unwrap();
    let detached=host.authoring_edit(AuthoringEdit::Reparent {id:"lever".into(),parent:None},accepted.source.clone()).unwrap();
    assert!(host.authoring_edit(AuthoringEdit::RemoveLeaf {id:"lever".into()},accepted.source).is_err());
    let unique=host.authoring_edit(AuthoringEdit::UniqueMaterial {id:"lever".into(),material_id:99},detached.snapshot.source).unwrap();
    let removed=host.authoring_edit(AuthoringEdit::RemoveLeaf {id:"lever".into()},unique.snapshot.source).unwrap();
    assert_eq!(host.authoring_view(removed.snapshot.source.clone()).unwrap().parts.len(),1);
    let undone=host.undo(removed.snapshot.source).unwrap();
    assert!(undone.reconciliation_required);
    let rebuilt=host.rebuild_execution(undone.source).unwrap();
    let restored=host.authoring_view(rebuilt.snapshot.source).unwrap();
    assert_eq!(restored.parts.len(),2);
    assert_eq!(restored.parts[1].material_id,99);
    assert_eq!(restored.parts[1].parent,None);
}
#[test]
fn parameter_rebuild_retains_unsigned_precision_and_unknown_untouched_values() {
    let fixture=super::tests::source().replace("channels:6", "channels:9007199254740993u64,notes:{keep:true}");
    let mut session=SourceSession::default();
    let mut host=vibe::LocalHost::default();
    session.submit(&fixture,&mut host,Budget::default()).unwrap();
    edit(&mut session,&mut host,AuthoringEdit::Parameter {key:"offset".into(),value:1.0e-12}).unwrap();
    let parameters=session.value("poet_parameters").unwrap();
    assert_eq!(field(parameters,"channels"),&Value::U64(9007199254740993));
    assert_eq!(field(parameters,"offset"),&Value::F64(1.0e-12));
    assert_eq!(field(field(parameters,"notes"),"keep"),&Value::Bool(true));
}
#[test]
fn shared_material_edit_groups_256_parts_without_source_or_expression_depth_growth() {
    let rows=(0..256).map(|i|format!(r#"{{id:"part_{i}",semantic_id:{},geometry:mesh,material:mat,translation:[{}.0,0.0,0.0],rotation:[0.0,0.0,0.0],scale:[1.0,1.0,1.0],notes:{{retain:{i}}}}}"#,i+1,i)).collect::<Vec<_>>().join(",");
    let fixture=format!("let mesh={{vertices:[[0.0,0.0,0.0],[1.0,0.0,0.0],[0.0,1.0,0.0]],triangles:[[0,1,2]]}};let mat={};let mut poet_parts=[{rows}];({{parts:poet_parts}});",super::tests::MATERIAL);
    vibe::parse_program(&fixture).expect("256-part fixture must be a canonical program, without expression-mode fallback");
    let mut session=SourceSession::default();
    let mut host=vibe::LocalHost::default();
    session.submit(&fixture,&mut host,Budget::default()).unwrap();
    let before=parts(&session);
    let command=AuthoringEdit::Material {material_id:1,base_color:[0.2,0.3,0.4,1.0],roughness:0.8,metallic:0.0};
    let generated=super::source::edit_source(session.value("poet_parts").unwrap(),None,&command,2).unwrap();
    assert!(generated.len()<2048,"shared schema must not scale source with part count");
    assert_eq!(generated.matches("if ").count(),1);
    edit(&mut session,&mut host,command).unwrap();
    let after=parts(&session);
    assert_eq!(after.len(),256);
    for (old,new) in before.iter().zip(&after) {
        for key in ["id","semantic_id","geometry","translation","rotation","scale","notes"] {assert_eq!(field(old,key),field(new,key));}
        assert_eq!(field(field(new,"material"),"roughness"),&Value::F64(0.8));
        assert_eq!(field(field(new,"material"),"double_sided"),field(field(old,"material"),"double_sided"));
    }
}
#[test]
fn shared_material_delta_matches_unsigned_and_mixed_identifiers_without_losing_extensions() {
    for wide in [false,true] {
        let fixture=super::tests::source().replace("double_sided:true}","double_sided:true,extension:{retained:[1,2,3]}}");
        let fixture=if wide {fixture.replace("material:{id:1,","material:{id:18446744073709551615u64,")} else {fixture.replacen("material:{id:1,","material:{id:1u64,",1)};
        let mut session=SourceSession::default();
        let mut host=vibe::LocalHost::default();
        session.submit(&fixture,&mut host,Budget::default()).unwrap();
        let before=parts(&session);
        let material_id=if wide {u64::MAX} else {1};
        edit(&mut session,&mut host,AuthoringEdit::Material {material_id,base_color:[0.1,0.2,0.3,1.0],roughness:0.9,metallic:0.7}).unwrap();
        for (old,new) in before.iter().zip(parts(&session).iter()) {
            let old_material=field(old,"material");
            let material=field(new,"material");
            assert_eq!(field(material,"id"),field(old_material,"id"));
            assert_eq!(field(material,"extension"),field(old_material,"extension"));
            assert_eq!(field(material,"roughness"),&Value::F64(0.9));
            assert_eq!(field(material,"metallic"),&Value::F64(0.7));
            for key in ["id","semantic_id","geometry","translation","rotation","scale"] {assert_eq!(field(new,key),field(old,key));}
        }
        if !wide {
            assert_eq!(field(field(&parts(&session)[0],"material"),"id"),&Value::U64(1));
            assert_eq!(field(field(&parts(&session)[1],"material"),"id"),&Value::I64(1));
        }
    }
}
