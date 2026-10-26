use super::super::geometry_host::GeometryHost;
use super::*;
use vibe::session::{SessionLimits, SourceSession};

fn grants() -> Vec<String> {
    vec![
        "capability.invoke".into(),
        "ComputationalGeometry.extrude_profile".into(),
    ]
}
fn scene(
    session: &mut SourceSession,
    host: &mut GeometryHost,
    source: &str,
) -> (Mesh, MaterialRecord) {
    vibe::parse_program(source)
        .unwrap_or_else(|error| panic!("canonical parse: {error:?}\n{source}"));
    let result = session
        .submit(source, host, vibe::Budget::default())
        .unwrap();
    authored_scene(
        &result.value,
        MaterialRecord::legacy_default(),
        PreviewLimits::default(),
    )
    .unwrap()
}

#[test]
fn canonical_material_declaration_and_live_edit_preserve_actual_geometry() {
    let mut session = SourceSession::new(grants(), SessionLimits::default());
    let mut host = GeometryHost::new(grants(), 200_000);
    let (before, material) = scene(&mut session, &mut host, &diagnostic_source());
    assert_eq!(material.base_color, MaterialDraft::default().base_color);
    assert_eq!(material.id, diagnostic_material_id());
    assert!(session.is_mutable("poet_geometry"));
    assert!(session.is_mutable("poet_material"));
    let edit = MaterialDraft {
        base_color: [0.05, 0.4, 0.8, 1.0],
        roughness: 0.13,
        metallic: 0.75,
        ..MaterialDraft::default()
    };
    let (after, material) = scene(
        &mut session,
        &mut host,
        &edit.edit_source(2, material.id).unwrap(),
    );
    assert_eq!(before.positions, after.positions);
    assert_eq!(before.triangles, after.triangles);
    assert_eq!(material.base_color, edit.base_color);
    assert_eq!(material.roughness, 0.13);
    assert_eq!(material.metallic, 0.75);
    assert_eq!(session.revision(), 2);
}

#[test]
fn material_source_replays_under_unchanged_geometry_grants() {
    let initial = diagnostic_source();
    let edit = MaterialDraft {
        emissive: [2.0, 0.0, 0.0],
        ..MaterialDraft::default()
    }
    .edit_source(2, diagnostic_material_id())
    .unwrap();
    let mut first = SourceSession::new(grants(), SessionLimits::default());
    let mut host = GeometryHost::new(grants(), 200_000);
    scene(&mut first, &mut host, &initial);
    let (mesh, material) = scene(&mut first, &mut host, &edit);
    let mut rebuilt = SourceSession::new(grants(), SessionLimits::default());
    let mut rebuilt_host = GeometryHost::new(grants(), 200_000);
    let (old_mesh, old_material) = scene(&mut rebuilt, &mut rebuilt_host, &initial);
    assert_ne!(old_material, material);
    let (new_mesh, new_material) = scene(&mut rebuilt, &mut rebuilt_host, &edit);
    assert_eq!(mesh.positions, old_mesh.positions);
    assert_eq!(mesh.positions, new_mesh.positions);
    assert_eq!(material, new_material);
}

#[test]
fn material_mapping_refuses_wrong_colour_semantics_ranges_and_textures() {
    let mut session = SourceSession::new(grants(), SessionLimits::default());
    let mut host = GeometryHost::new(grants(), 200_000);
    let result = session
        .submit(&diagnostic_source(), &mut host, vibe::Budget::default())
        .unwrap();
    let Value::Record(envelope) = result.value else {
        panic!("scene record")
    };
    let Value::Record(valid) = envelope.get("material").unwrap() else {
        panic!("material record")
    };
    for (field, bad) in [
        ("colour_space", Value::String("srgb".into())),
        ("roughness", Value::F64(-0.1)),
        ("metallic", Value::F64(1.01)),
        ("specular", Value::F64(f64::NAN)),
        ("id", Value::U64(0)),
        ("base_color", Value::List(vec![Value::F64(0.2); 3])),
        ("base_color_texture", Value::String("unverified".into())),
        ("opacity_mode", Value::String("unknown".into())),
    ] {
        let mut record = valid.clone();
        record.insert(field.into(), bad);
        assert!(
            material_from_value(&Value::Record(record)).is_err(),
            "admitted {field}"
        );
    }
    let mut record = valid.clone();
    record.remove("roughness");
    assert!(material_from_value(&Value::Record(record)).is_err());
    let mut envelope = envelope.clone();
    envelope.remove("material");
    assert!(authored_scene(
        &Value::Record(envelope),
        MaterialRecord::legacy_default(),
        PreviewLimits::default()
    )
    .is_err());
}

#[test]
fn generated_draft_refuses_invalid_factor_before_source_generation() {
    let draft = MaterialDraft {
        roughness: 1.01,
        ..MaterialDraft::default()
    };
    assert!(draft.edit_source(2, diagnostic_material_id()).is_err());
    let mut material = MaterialRecord::legacy_default();
    material.base_color_texture = [1; 32];
    assert!(validate_factor_material(&material).is_err());
}

#[test]
fn bare_mesh_remains_compatible_and_authored_material_never_uses_fallback() {
    let mut session = SourceSession::new(grants(), SessionLimits::default());
    let mut host = GeometryHost::new(grants(), 200_000);
    let result = session
        .submit(
            super::super::geometry_host::ORGANIC_SURFACE_SOURCE,
            &mut host,
            vibe::Budget::default(),
        )
        .unwrap();
    let mut caller = MaterialRecord::legacy_default();
    caller.base_color = [0.9, 0.8, 0.1, 1.0];
    let (_, material) = authored_scene(&result.value, caller, PreviewLimits::default()).unwrap();
    assert_eq!(caller, material);
    let denied = SourceSession::default();
    assert!(denied.prepare(&diagnostic_source()).is_err());
}
