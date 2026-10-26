use super::*;
use crate::graphical::materials::{self, MaterialDraft};

struct Headless(MeshPreviewProvider);
impl PreviewProvider for Headless {
    fn validate(&self,value:&Value)->Result<(),String> { self.0.validate(value) }
    fn preview(&mut self,_source:SourceRef,_value:&Value)->Result<PreviewFrame,String> {
        Err("headless source admission test; no frame requested".into())
    }
}
fn host(label:&str)->CanonicalEditorHost {
    let id=format!("urn:poet:material-admission:{label}:{}",std::process::id());
    studio_store::create_artifact_if_absent(StudioArtifact::new(&id,"Geometry",label)).unwrap();
    CanonicalEditorHost::open(id,SessionLimits::default(),GeometryHost::new(vec![
        "capability.invoke".into(),"ComputationalGeometry.extrude_profile".into()],200_000),
        Box::new(Headless(MeshPreviewProvider {
            material:qualia_core_db::container_10d::MaterialRecord::legacy_default(),
            camera:preview::PreviewCamera { yaw:0.45,pitch:0.2,distance:3.2,target:[0.0;3] },
            limits:preview::PreviewLimits::default(),
        }))).unwrap()
}
#[test]
fn invalid_material_output_cannot_promote_session_or_source_and_rebuild_recovers() {
    let mut host=host("invalid-output");
    let accepted=host.submit(&materials::diagnostic_source(),host.snapshot().unwrap().source).unwrap().snapshot;
    let invalid=MaterialDraft::default().edit_source(2,accepted.material_edit_id.unwrap()).unwrap()
        .replace("roughness: 0.65","roughness: 2.0");
    assert!(host.submit(&invalid,accepted.source.clone()).is_err());
    let held=host.snapshot().unwrap();
    assert_eq!(held.source,accepted.source);
    assert_eq!(held.session_revision,accepted.session_revision);
    assert!(held.reconciliation_required);
    assert!(held.material_edit_id.is_none());
    let recovered=host.rebuild_execution(held.source).unwrap().snapshot;
    assert!(!recovered.reconciliation_required);
    assert_eq!(recovered.material_edit_id,accepted.material_edit_id);
    assert_eq!(host.source_cells(recovered.source).unwrap().len(),1);
}
#[test]
fn pure_helper_cells_remain_accepted_and_material_only_route_rejects_geometry_changes() {
    let mut host=host("helper-cells");
    let initial=host.submit(&materials::diagnostic_source(),host.snapshot().unwrap().source).unwrap().snapshot;
    let helper=host.submit("let supporting_value: i64 = 17;",initial.source).unwrap().snapshot;
    assert_eq!(helper.session_revision,2);
    let injected="poet_geometry = poet_geometry;\n({geometry: poet_geometry, material: poet_material});";
    assert!(host.material_edit(injected,helper.source.clone()).is_err());
    assert_eq!(host.snapshot().unwrap().source,helper.source);
    let edit=MaterialDraft { metallic:0.8,..MaterialDraft::default() }
        .edit_source(3,helper.material_edit_id.unwrap()).unwrap();
    let accepted=host.material_edit(&edit,helper.source).unwrap().snapshot;
    assert_eq!(accepted.session_revision,3);
    assert_eq!(host.source_cells(accepted.source).unwrap().len(),3);
}
