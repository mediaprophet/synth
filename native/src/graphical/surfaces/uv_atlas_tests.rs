use super::*;
#[test]
fn seam_derived_mesh_preserves_original_geometry_and_exact_normal_provenance() {
    let mesh = Mesh {
        positions: vec![[0., 0., 0.], [1., 0., 0.], [1., 1., 0.], [0., 1., 0.]],
        triangles: vec![[0, 1, 2], [0, 2, 3]],
        min: [0.; 3],
        max: [1., 1., 0.],
    };
    let original = mesh.clone();
    let normals = [[0., 0., 1.], [0., 1., 1.], [1., 0., 1.], [1., 1., 1.]];
    let draft = AtlasDraft {
        seams: vec![[0, 2]],
        ..AtlasDraft::default()
    };
    let derived = apply(
        &draft,
        &mesh,
        Some(&normals),
        AtlasLimits::default(),
        &|| false,
    )
    .unwrap();
    assert_eq!(mesh, original);
    assert_eq!(derived.charts.len(), 2);
    assert_eq!(derived.mesh.positions.len(), 6);
    for (i, &source) in derived.source_vertices.iter().enumerate() {
        assert_eq!(derived.mesh.positions[i], mesh.positions[source as usize]);
        assert_eq!(
            derived.normals.as_ref().unwrap()[i],
            normals[source as usize]
        );
    }
    assert_eq!(
        derived
            .mesh
            .triangles
            .iter()
            .map(|t| t.map(|v| derived.source_vertices[v as usize]))
            .collect::<Vec<_>>(),
        mesh.triangles
    );
    assert!(!source(&draft).unwrap().contains("vertices"));
}
#[test]
fn atlas_recipe_is_strict_and_unsupported_versions_refuse_without_input_mutation() {
    let draft = AtlasDraft::default();
    let mut json = serde_json::to_value(&draft).unwrap();
    json["extra"] = true.into();
    assert!(serde_json::from_value::<AtlasDraft>(json).is_err());
    let mut bad = draft.clone();
    bad.version = 2;
    assert!(source(&bad).is_err());
    let mut record = std::collections::BTreeMap::new();
    record.insert("unknown".into(), Value::I64(1));
    assert!(from_value(&Value::Record(record)).is_err());
    assert!(preflight(
        &draft,
        2000,
        2000,
        AtlasLimits {
            scratch_bytes: 1,
            ..AtlasLimits::default()
        }
    )
    .is_err());
}
#[test]
fn exact_recipe_survives_actual_persistent_vibe_source_evaluation() {
    let draft = AtlasDraft {
        seams: vec![[0, 2], [1, 3]],
        ..AtlasDraft::default()
    };
    let mut session = vibe::session::SourceSession::default();
    let init = format!(
        "let mut poet_uv_atlas = {}; (poet_uv_atlas);",
        source(&draft).unwrap()
    );
    let evaluated = session
        .submit(
            &init,
            &mut vibe::LocalHost::default(),
            vibe::Budget::default(),
        )
        .unwrap()
        .value;
    assert_eq!(from_value(&evaluated).unwrap(), draft);
    let next = AtlasDraft {
        gutter_pixels: 8,
        ..draft
    };
    let evaluated = session
        .submit(
            &format!(
                "poet_uv_atlas = {}; (poet_uv_atlas);",
                source(&next).unwrap()
            ),
            &mut vibe::LocalHost::default(),
            vibe::Budget::default(),
        )
        .unwrap()
        .value;
    assert_eq!(from_value(&evaluated).unwrap(), next);
}
