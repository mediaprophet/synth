use super::*;
use crate::graphical::surfaces::{
    detail::{DetailRecipe, ProceduralRecipe},
    uv_atlas::AtlasDraft,
};
use crate::graphical::{
    controller::{EditorHost, MeshPreviewProvider, PreviewProvider},
    jobs::Context,
    preview::{PreviewCamera, PreviewLimits},
};
use vibe::Value;
fn procedural() -> DetailRecipe {
    DetailRecipe::Procedural(ProceduralRecipe {
        version: 1,
        side: 8,
        seed: 9,
        tint: [0.3, 0.5, 0.7],
        contrast: 0.5,
        relief: 0.2,
        bands: [2, 3],
        band_strength: 0.4,
        roughness: 0.6,
        metallic: 0.2,
    })
}
#[test]
fn opaque_context_cursor_retires_equal_source_part_drafts_on_new_epoch_or_session() {
    let h = super::tests::host();
    let snapshot = h.snapshot().unwrap();
    let context = Context::from_snapshot(&snapshot);
    let mut c = super::surface_tools::DraftCursor::default();
    let first = c.token(context.clone(), "body").unwrap();
    assert_eq!(Some(first.clone()), c.token(context.clone(), "body"));
    let mut next = context.clone();
    next.execution_epoch = crate::graphical::controller::SessionEpoch::new();
    let epoch = c.token(next.clone(), "body").unwrap();
    assert_ne!(first, epoch);
    next.session_revision += 1;
    let session = c.token(next.clone(), "body").unwrap();
    assert_ne!(epoch, session);
    assert_ne!(Some(session), c.token(next, "other"));
}
#[test]
fn atlas_source_edits_restore_original_geometry_and_refuse_conflicting_projection_atomically() {
    let mut h = super::tests::host();
    let s = h.snapshot().unwrap().source;
    let s = h
        .submit(&super::tests::source(), s)
        .unwrap()
        .snapshot
        .source;
    let original = h
        .compiled_assets("before", s.clone())
        .unwrap()
        .remove(0)
        .1
        .mesh;
    let a = AtlasDraft {
        atlas_size: 64,
        gutter_pixels: 2,
        texels_per_metre: 8.,
        ..Default::default()
    };
    let applied = h
        .authoring_edit(
            AuthoringEdit::UvAtlas {
                id: "body".into(),
                atlas: Some(a.clone()),
            },
            s,
        )
        .unwrap()
        .snapshot
        .source;
    assert_eq!(
        h.authoring_view(applied.clone()).unwrap().parts[0].uv_atlas,
        Some(a)
    );
    let before = h.snapshot().unwrap();
    assert!(h
        .authoring_edit(
            AuthoringEdit::UvProjection {
                id: "body".into(),
                projection: Some(crate::graphical::surfaces::uv::PlanarUvDraft {
                    axes: [0, 1],
                    metres_per_repeat: 1.
                })
            },
            applied.clone()
        )
        .is_err());
    assert_eq!(h.snapshot().unwrap().source, before.source);
    let restored = h
        .authoring_edit(
            AuthoringEdit::UvAtlas {
                id: "body".into(),
                atlas: None,
            },
            applied,
        )
        .unwrap()
        .snapshot
        .source;
    let after = h
        .compiled_assets("after", restored)
        .unwrap()
        .remove(0)
        .1
        .mesh;
    assert_eq!(after.positions, original.positions);
    assert_eq!(after.triangles, original.triangles);
}
#[test]
fn generated_texture_without_uv_refuses_actual_mesh_provider_before_promotion() {
    let mut session = vibe::session::SourceSession::default();
    let mut local = vibe::LocalHost::default();
    let value = session
        .submit(&super::tests::source(), &mut local, vibe::Budget::default())
        .unwrap()
        .value;
    let Value::Record(mut root) = value else {
        panic!()
    };
    let Some(Value::List(parts)) = root.get_mut("parts") else {
        panic!()
    };
    let encoded = procedural().source_expression().unwrap();
    let recipe = session
        .submit(
            &format!("({encoded});"),
            &mut local,
            vibe::Budget::default(),
        )
        .unwrap()
        .value;
    for p in parts {
        let Value::Record(p) = p else { panic!() };
        p.insert("detail".into(), recipe.clone());
    }
    let provider = MeshPreviewProvider {
        material: qualia_core_db::container_10d::MaterialRecord::legacy_default(),
        camera: PreviewCamera {
            yaw: 0.4,
            pitch: 0.3,
            distance: 3.,
            target: [0.; 3],
        },
        limits: PreviewLimits::default(),
    };
    let error = provider
        .validate_with_resources(&Value::Record(root), None)
        .unwrap_err();
    assert!(error.contains("UV"), "{error}");
}
#[test]
fn shared_material_detail_edit_and_removal_keep_all_original_source_fields() {
    let mut session = vibe::session::SourceSession::default();
    let mut local = vibe::LocalHost::default();
    session
        .submit(&super::tests::source(), &mut local, vibe::Budget::default())
        .unwrap();
    let original = session.value("poet_parts").unwrap().clone();
    let edit = source::edit_source(
        &original,
        session.value("poet_parameters"),
        &AuthoringEdit::TextureDetail {
            material_id: 1,
            detail: Some(procedural()),
        },
        1,
    )
    .unwrap();
    session
        .submit(&edit, &mut local, vibe::Budget::default())
        .unwrap();
    let Value::List(parts) = session.value("poet_parts").unwrap() else {
        panic!()
    };
    let Value::List(old) = &original else {
        panic!()
    };
    for (p, old) in parts.iter().zip(old) {
        let (Value::Record(p), Value::Record(old)) = (p, old) else {
            panic!()
        };
        assert!(p.contains_key("detail"));
        for (k, v) in old {
            assert_eq!(p.get(k), Some(v));
        }
    }
    let edit = source::edit_source(
        session.value("poet_parts").unwrap(),
        session.value("poet_parameters"),
        &AuthoringEdit::TextureDetail {
            material_id: 1,
            detail: None,
        },
        2,
    )
    .unwrap();
    session
        .submit(&edit, &mut local, vibe::Budget::default())
        .unwrap();
    let Value::List(parts) = session.value("poet_parts").unwrap() else {
        panic!()
    };
    assert!(parts
        .iter()
        .all(|p| matches!(p,Value::Record(p) if matches!(p.get("detail"),Some(Value::Null)))));
}

#[test]
fn scalar_returning_mutations_cannot_bypass_selected_geometry_or_atlas_inspection_admission() {
    let mut h = super::tests::host();
    let source = h.snapshot().unwrap().source;
    let source = h
        .submit(&super::tests::source(), source)
        .unwrap()
        .snapshot
        .source;
    let invalid = r#"poet_parts=poet_parts.map(|p| ({id:p.id,semantic_id:p.semantic_id,geometry:{vertices:p.geometry.vertices,triangles:p.geometry.triangles,skin:[1]},material:p.material,translation:p.translation,rotation:p.rotation,scale:p.scale}));1;"#;
    let source = h.submit(invalid, source).unwrap().snapshot.source;
    assert!(h
        .inspect_surface("body", source)
        .unwrap_err()
        .contains("unsupported"));
    let mut h = super::tests::host();
    let source = h.snapshot().unwrap().source;
    let source = h
        .submit(&super::tests::source(), source)
        .unwrap()
        .snapshot
        .source;
    let atlas = AtlasDraft {
        atlas_size: 64,
        gutter_pixels: 2,
        texels_per_metre: 8.,
        ..Default::default()
    };
    let source = h
        .authoring_edit(
            AuthoringEdit::UvAtlas {
                id: "body".into(),
                atlas: Some(atlas),
            },
            source,
        )
        .unwrap()
        .snapshot
        .source;
    let transformed = r#"poet_parts=poet_parts.map(|p| if p.id=="body" {{id:p.id,semantic_id:p.semantic_id,geometry:p.geometry,material:p.material,translation:p.translation,rotation:p.rotation,scale:p.scale,uv_atlas:p.uv_atlas,uv_transform:{offset:[0.0,0.0],scale:[2.0,1.0],rotation:0.0,tex_coord:0}}}else{p});1;"#;
    let source = h.submit(transformed, source).unwrap().snapshot.source;
    assert!(h
        .inspect_surface("body", source)
        .unwrap_err()
        .contains("affine"));
}
