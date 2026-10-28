// Synthetic source/UV continuity diagnostic; this is not a biological asset.
const BOTANICAL_UV_CHILD: &str = "POET_BOTANICAL_UV_CHILD_ROOT";

fn botanical_compiled_uv(host: &dyn EditorHost) -> Vec<u8> {
    let snapshot = host.snapshot().unwrap();
    let assets = host.compiled_assets("uv-continuity-diagnostic", snapshot.source).unwrap();
    assert!(!assets.is_empty());
    let asset = &assets[0].1;
    let mut uv = vec![[0.0f32; 2]; asset.mesh.positions.len()];
    qualia_core_db::container_10d::read_texture_coordinate_section_into(
        &asset.container_10d, uv.len(), &mut uv,
    ).unwrap();
    let mut bytes = Vec::with_capacity(uv.len() * 8);
    for p in uv { bytes.extend_from_slice(&p[0].to_le_bytes()); bytes.extend_from_slice(&p[1].to_le_bytes()); }
    bytes
}

#[test]
fn signed_botanical_uv_radius_edit_hmc_fresh_reopen() {
    if let Some(root) = std::env::var_os(BOTANICAL_UV_CHILD) {
        let root = Path::new(&root);
        let scope = Scope { project_id:std::env::var("POET_BOTANICAL_UV_PROJECT").unwrap(), document_id:std::env::var("POET_BOTANICAL_UV_DOCUMENT").unwrap(), source_id:std::env::var("POET_BOTANICAL_UV_SOURCE").unwrap(), domain:"Geometry".into() };
        assert!(studio_store::get_artifact_stamped(&scope.source_id).is_none());
        let authority = Arc::new(NativeAuthority::open_retained_current(&root.join("authority"), TrustedProject { project_id:scope.project_id.clone(), issuer_identifier:"fixture-issuer".into(), public_key:ed25519::public_from_seed(&AUTHORITY) }, 1024*1024, 256).unwrap());
        let author = login(&authority,"fixture-author",&AUTHOR);
        let project = NativeProject::open(botanical_config(scope.clone(),&root.join("hmc"),&root.join("library")),authority,author.clone(),Some("saved.hmc"),Box::new(BotanicalNoPixels)).unwrap();
        let mut host=project.connect(author).unwrap();
        let held=host.snapshot().unwrap(); assert!(held.reconciliation_required);
        host.rebuild_execution(held.source).unwrap();
        assert_botanical(&host,0.04,0.5);
        let artifact=checkpoint::load(&root.join("hmc/saved.hmc"),CheckpointLimits::default()).unwrap();
        assert_eq!(artifact.artifact(),&studio_store::get_artifact_stamped(&scope.source_id).unwrap().0);
        assert_eq!(botanical_compiled_uv(&host),std::fs::read(root.join("before.uv")).unwrap());
        std::fs::write(root.join("uv-child-ok"),b"fresh HMC source and compiled UV").unwrap();
        return;
    }
    let temp=tempfile::tempdir().unwrap();
    let scope={
        for name in ["authority","hmc","library"] { std::fs::create_dir(temp.path().join(name)).unwrap(); }
        let (scope,_,authority)=fixture(&temp.path().join("authority"));
        let author=login(&authority,"fixture-author",&AUTHOR);
        let project=NativeProject::open(botanical_config(scope.clone(),&temp.path().join("hmc"),&temp.path().join("library")),authority,author.clone(),None,Box::new(BotanicalNoPixels)).unwrap();
        let mut host=project.connect(author).unwrap();
        let initial=host.snapshot().unwrap().source;
        host.submit(botanical_source(),initial).unwrap();
        let source=host.snapshot().unwrap().source;
        host.authoring_edit(AuthoringEdit::Botanical { edit:BotanicalEdit::SetRecipe { recipe:botanical_recipe() } },source).unwrap();
        let recipe=botanical_recipe();
        let mesh=crate::graphical::botanical_graph_host::evaluate(&recipe,crate::graphical::authoring::botanical::BotanicalLimits::default()).unwrap();
        let uv=(0..mesh.vertices.len()).map(|i| format!("[{i}.0,{}.0]",i+1)).collect::<Vec<_>>().join(",");
        let attach=format!("poet_parts = poet_parts.map(|p| if p.id == \"branch\" {{ {{id:p.id,semantic_id:p.semantic_id,geometry:{{vertices:p.geometry.vertices,triangles:p.geometry.triangles,uv0:[{uv}]}},material:p.material,translation:p.translation,rotation:p.rotation,scale:p.scale}} }} else {{p}}); ({{parts:poet_parts}});");
        let source=host.snapshot().unwrap().source;
        host.submit(&attach,source).unwrap();
        let expected=botanical_compiled_uv(&host);
        assert!(expected.chunks_exact(8).any(|p|p != [0;8]));
        let source=host.snapshot().unwrap().source;
        host.authoring_edit(AuthoringEdit::Botanical { edit:BotanicalEdit::SetTipRadius { part_id:"branch".into(),tip_radius_m:0.04 } },source).unwrap();
        assert_eq!(botanical_compiled_uv(&host),expected);
        let source=host.snapshot().unwrap().source;
        assert!(host.authoring_edit(AuthoringEdit::Botanical { edit:BotanicalEdit::SetOrganPosition { part_id:"branch".into(),organ_key:"lateral".into(),position_m:[0.7,0.0,1.5] } },source.clone()).is_err());
        assert_eq!(host.snapshot().unwrap().source,source);
        host.save_checkpoint("saved.hmc",ExpectedFile::Absent,source).unwrap();
        std::fs::write(temp.path().join("before.uv"),expected).unwrap();
        scope
    };
    let name=std::thread::current().name().unwrap().to_owned();
    let mut child=std::process::Command::new(std::env::current_exe().unwrap()).args(["--exact",name.as_str(),"--nocapture","--test-threads=1"]).env(BOTANICAL_UV_CHILD,temp.path()).env("POET_BOTANICAL_UV_PROJECT",&scope.project_id).env("POET_BOTANICAL_UV_DOCUMENT",&scope.document_id).env("POET_BOTANICAL_UV_SOURCE",&scope.source_id).spawn().unwrap();
    let deadline=std::time::Instant::now()+std::time::Duration::from_secs(30);
    loop {
        if let Some(status)=child.try_wait().unwrap() { assert!(status.success(),"fresh HMC UV child failed: {status}"); break; }
        if std::time::Instant::now()>=deadline { let _=child.kill(); let _=child.wait(); panic!("fresh HMC UV child exceeded deadline"); }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    assert!(temp.path().join("uv-child-ok").is_file());
}
