//! Synthetic temporary authoring checks; no master, rendered output or human acceptance.
use super::*;
use crate::graphical::{controller::{CanonicalEditorHost, EditorHost, PreviewProvider}, geometry_host::{GeometryHost, GeometryHostLimits}, preview::{PreviewFrame, SourceRef}};
use vibe::{session::{SourceSession, SessionLimits}, Budget, Value};

fn grants() -> Vec<String> { vec!["capability.invoke".into(),"ComputationalGeometry.profile_sweep".into()] }
fn host(actual: Vec<String>, work: usize, points: usize) -> GeometryHost {
    GeometryHost::with_limits(actual,GeometryHostLimits {max_vertices:10000,submission_work:work,dispatches:1,scratch_bytes:16*1024*1024,profile_points:points}).unwrap()
}
fn session() -> SourceSession {
    let mut session = SourceSession::new(grants(),SessionLimits::default());
    let fixture=super::tests::source().replace("geometry:mesh,material:","profile_note:{preserved:[1,2,3]},geometry:mesh,material:");
    session.submit(&fixture,&mut vibe::LocalHost::default(),Budget::default()).unwrap();
    session
}
fn field<'a>(value: &'a Value,key:&str) -> &'a Value {
    let Value::Record(r)=value else {panic!("record")}; r.get(key).unwrap()
}
fn parts(session:&SourceSession) -> Vec<Value> {
    let Value::List(rows)=session.value("poet_parts").unwrap() else {panic!("parts")}; rows.clone()
}
fn edit(session:&mut SourceSession, host:&mut GeometryHost, geometry:GeometryRecipe) -> Result<(),String> {
    let command=AuthoringEdit::ReplaceGeometry {id:"lever".into(),geometry};
    let generated=source::edit_source(session.value("poet_parts").unwrap(),None,&command,session.revision()+1)?;
    assert_eq!(generated.matches("capability.invoke(").count(),1);
    assert!(!generated.contains(".map("),"no capability capture in a pinned Vibe 0.1 lambda");
    vibe::parse_program(&generated).map_err(|e|format!("{e:?}"))?;
    session.submit(&generated,host,Budget::default()).map(|_|()).map_err(|e|format!("{e:?}"))
}
fn rounded() -> ProfileRecipe {ProfileRecipe::RoundedRectangle {width:0.4,height:0.2,radius:0.02,segments:4}}
fn channel() -> ProfileRecipe {ProfileRecipe::OpenChannel {width:0.4,height:0.2,wall:0.02,corner_radius:0.01,segments:4}}
fn polygon() -> ProfileRecipe {ProfileRecipe::Polyline {points:vec![[0.0,0.0],[0.4,0.0],[0.4,0.2],[0.2,0.1],[0.0,0.2]]}}
fn sweep(profile:ProfileRecipe,caps:bool) -> GeometryRecipe {GeometryRecipe::ProfileSweep {profile,length:0.75,caps}}
fn rows<'a>(value:&'a Value,key:&str) -> &'a [Value] {let Value::List(v)=field(value,key) else {panic!("list")};v}

#[test]
fn metric_profiles_execute_once_with_caps_length_and_preserved_records() {
    for profile in [rounded(),channel(),polygon()] {
        let mut meshes=Vec::new();
        for caps in [false,true] {
            let mut session=session();
            let before=parts(&session);
            let mut actual=host(grants(),10000,1024);
            edit(&mut session,&mut actual,sweep(profile.clone(),caps)).unwrap();
            let after=parts(&session);
            assert_eq!(after[0],before[0]);
            for key in ["parent","translation","rotation","scale","material","id","semantic_id","profile_note"] {assert_eq!(field(&after[1],key),field(&before[1],key));}
            let geometry=field(&after[1],"geometry").clone();
            let vertices=rows(&geometry,"vertices");
            let z:Vec<_>=vertices.iter().map(|v|{let Value::List(v)=v else {panic!("point")};source::number(&v[2]).unwrap()}).collect();
            assert!((z.iter().copied().fold(f64::INFINITY,f64::min)-0.0).abs()<1e-6);
            assert!((z.iter().copied().fold(f64::NEG_INFINITY,f64::max)-0.75).abs()<1e-6);
            let revision=session.revision();
            assert!(edit(&mut session,&mut actual,sweep(profile.clone(),caps)).unwrap_err().contains("E400"));
            assert_eq!(session.revision(),revision,"one dispatch was consumed by the successful edit");
            assert_eq!(parts(&session),after);
            meshes.push(geometry);
        }
        let count=rows(&meshes[0],"vertices").len()/2;
        assert_eq!(rows(&meshes[1],"vertices"),rows(&meshes[0],"vertices"));
        assert_eq!(rows(&meshes[0],"triangles").len(),count*2);
        assert_eq!(rows(&meshes[1],"triangles").len(),count*2+(count-2)*2);
    }
}
#[test]
fn metric_profile_controls_validate_fields_units_bounds_and_simple_sections() {
    for kind in ["rounded_rectangle","open_channel","polyline"] {
        let geometry=super::profile_controls::recipe(kind,["0.4","0.2","0.01","0.02","4","0.75"],"[[0,0],[0.4,0],[0.4,0.2],[0,0.2]]",true).unwrap();
        assert!(geometry.validate().is_ok());
    }
    let mut session=session();let before=parts(&session);let revision=session.revision();
    let mut actual=host(grants(),10000,1024);
    for profile in [
        ProfileRecipe::RoundedRectangle {width:0.4,height:0.2,radius:0.11,segments:4},
        ProfileRecipe::OpenChannel {width:0.4,height:0.2,wall:0.2,corner_radius:0.01,segments:4},
        ProfileRecipe::Polyline {points:vec![[0.0,0.0],[3.0,0.0],[0.0,2.0],[2.0,2.0]]},
        ProfileRecipe::Polyline {points:vec![[0.0,0.0],[0.0,1.0],[1.0,0.0]]},
        ProfileRecipe::Polyline {points:vec![[0.0,0.0],[1.0,0.0],[2.0,0.0]]},
        ProfileRecipe::Polyline {points:vec![[f64::NAN,0.0];3]},
        ProfileRecipe::Polyline {points:vec![[0.0,0.0];257]},
        ProfileRecipe::RoundedRectangle {width:0.4,height:0.2,radius:0.01,segments:65},
    ] {assert!(edit(&mut session,&mut actual,sweep(profile,true)).is_err());}
    for length in [-1.0,0.0,1.0e-7,1.0e7,f64::INFINITY,f64::NAN] {assert!(edit(&mut session,&mut actual,GeometryRecipe::ProfileSweep {profile:rounded(),length,caps:true}).is_err());}
    assert_eq!(session.revision(),revision);assert_eq!(parts(&session),before);
    edit(&mut session,&mut actual,sweep(rounded(),true)).expect("invalid source preflight consumed no host dispatch");
    for (key,value) in [("units","\"mm\""),("segments","1"),("code","\"capability.invoke\"")] {
        let json=format!("{{\"kind\":\"polyline\",\"points\":[[0,0],[1,0],[0,1]],\"{key}\":{value}}}");
        assert!(serde_json::from_str::<ProfileRecipe>(&json).is_err(),"unknown source recipe field: {key}");
    }
    assert!(serde_json::from_str::<GeometryRecipe>(r#"{"kind":"ProfileSweep","profile":{"kind":"polyline","points":[[0,0],[1,0],[0,1]]},"length":1,"caps":"true"}"#).is_err());
    assert!(super::profile_controls::recipe("rounded_rectangle",["10 mm","0.2","0","0.02","4","1"],"",true).is_err());
}
#[test]
fn metric_profile_grants_work_and_point_budgets_refuse_without_source_promotion() {
    for (actual,work,points,code) in [(vec!["capability.invoke".into()],10000,1024,"E300"),(grants(),1,1024,"E400"),(grants(),10000,3,"E400")] {
        let mut session=session(); let before=parts(&session);let revision=session.revision();
        let error=edit(&mut session,&mut host(actual,work,points),sweep(channel(),true)).unwrap_err();
        assert!(error.contains(code),"{error}");assert_eq!(session.revision(),revision);assert_eq!(parts(&session),before);
    }
}
#[test]
fn metric_profile_replacement_refuses_loss_of_topology_bound_geometry_fields() {
    for key in ["uvs","normals","skin","morphs","future_extension"] {
        let session=session();
        let mut value=session.value("poet_parts").unwrap().clone();
        let Value::List(parts)=&mut value else {unreachable!()};
        let Value::Record(part)=&mut parts[1] else {unreachable!()};
        let Value::Record(geometry)=part.get_mut("geometry").unwrap() else {unreachable!()};
        geometry.insert(key.into(),Value::List(vec![Value::F64(0.5)]));
        let command=AuthoringEdit::ReplaceGeometry {id:"lever".into(),geometry:sweep(rounded(),true)};
        assert!(source::edit_source(&value,None,&command,session.revision()+1).is_err(),"refuse before discarding {key}");
    }
}

struct NoPixels;
impl PreviewProvider for NoPixels {
    fn validate(&self,value:&Value)->Result<(),String> {crate::graphical::assembly::admit_assembly(value,crate::graphical::preview::PreviewLimits::default()).map(|_|())}
    fn preview(&mut self,_:SourceRef,_:&Value)->Result<PreviewFrame,String> {Err("No GPU requested by profile source test".into())}
}
#[test]
fn metric_profile_add_stale_undo_and_source_to_native_hmc() {
    use qualia_core_db::poet_host::invoke::studio_store::{self, StudioArtifact, checkpoint as core};
    static NEXT:std::sync::atomic::AtomicU64=std::sync::atomic::AtomicU64::new(1);
    let id=format!("urn:poet:profile-test:{}:{}",std::process::id(),NEXT.fetch_add(1,std::sync::atomic::Ordering::Relaxed));
    studio_store::create_artifact_if_absent(StudioArtifact::new(&id,"Geometry","Temporary profile source")).unwrap();
    let mut actual=CanonicalEditorHost::open(id,SessionLimits::default(),GeometryHost::new(grants(),10000),Box::new(NoPixels)).unwrap();
    let first=actual.snapshot().unwrap();actual.submit(&super::tests::source(),first.source).unwrap();
    let before=actual.snapshot().unwrap();
    let add=AuthoringEdit::AddGeometry {id:"new-profile".into(),semantic_id:103,material_from:"lever".into(),geometry:sweep(channel(),true)};
    let added=actual.authoring_edit(add,before.source.clone()).unwrap();
    assert!(actual.authoring_edit(AuthoringEdit::ReplaceGeometry {id:"new-profile".into(),geometry:sweep(polygon(),false)},before.source).is_err());
    assert_eq!(actual.authoring_view(added.snapshot.source.clone()).unwrap().parts.len(),3);
    let cells=actual.source_cells(added.snapshot.source.clone()).unwrap();
    assert_eq!(cells.last().unwrap().matches("capability.invoke(").count(),1);
    let assets=actual.compiled_assets("profile-synthetic",added.snapshot.source.clone()).unwrap();
    let limits=core::CheckpointLimits::default();
    let candidate=core::capture(&added.snapshot.source.artifact_id,&added.snapshot.source.stamp,limits).unwrap();
    let bytes=core::encode(&candidate,assets,limits).unwrap();
    let reopened=core::decode(&bytes,limits).unwrap();
    assert!(reopened.inventory().iter().any(|r|r.kind=="10d"));
    assert!(reopened.inventory().iter().any(|r|r.kind=="q42"));
    let undone=actual.undo(added.snapshot.source).unwrap();assert!(undone.reconciliation_required);
    let rebuilt=actual.rebuild_execution(undone.source).unwrap();
    assert_eq!(actual.authoring_view(rebuilt.snapshot.source).unwrap().parts.len(),2);
}
