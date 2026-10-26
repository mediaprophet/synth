use super::*;
use qualia_core_db::{
    bundle::{BundleReader, BundleWriter},
    container_10d::{self, MaterialRecord, ShadingModel, TextureSampler, TextureTransform},
    render::{asset_package::texture_hmc_key, compile_10d::CompiledAsset},
};
use sha2::{Digest, Sha256};
use vibe::Value;

// Synthetic 2x2 memory-only KTX2 fixture following the canonical decoder vectors.
pub(super) fn texture(srgb: bool, pixels: [u8; 16]) -> Vec<u8> {
    let mut b = vec![0; 152];
    b[..12].copy_from_slice(&qualia_core_db::render::texture_ktx2::KTX2_IDENTIFIER);
    for (at, n) in [
        (12, if srgb { 43u32 } else { 37 }),
        (16, 1),
        (20, 2),
        (24, 2),
        (36, 1),
        (40, 1),
        (48, 104),
        (52, 28),
        (104, 28),
        (112, (24 << 16) | 2),
    ] {
        b[at..at + 4].copy_from_slice(&n.to_le_bytes());
    }
    for (at, n) in [(80, 136u64), (88, 16), (96, 16)] {
        b[at..at + 8].copy_from_slice(&n.to_le_bytes());
    }
    b[136..].copy_from_slice(&pixels);
    b
}
pub(super) fn package(textures: &[(Vec<u8>, &str)]) -> Vec<u8> {
    let mut w = BundleWriter::new();
    for (bytes, mime) in textures {
        let digest: [u8; 32] = Sha256::digest(bytes).into();
        w.add_file(texture_hmc_key(&digest), *mime, bytes.clone(), None)
            .unwrap();
    }
    w.build().unwrap()
}
fn parsed(source: &str) -> Value {
    let mut s = vibe::session::SourceSession::default();
    s.submit(
        source,
        &mut vibe::LocalHost::default(),
        vibe::Budget::default(),
    )
    .unwrap()
    .value
}
fn source(m: &MaterialRecord) -> String {
    let mut fields = SurfaceDraft::from_material(m).updates().unwrap();
    fields.insert("id", format!("{}u64", m.id));
    format!(
        "({{{}}});",
        fields
            .into_iter()
            .map(|(k, v)| format!("{k}:{v}"))
            .collect::<Vec<_>>()
            .join(",")
    )
}
#[test]
fn all_visual_mat4_fields_and_six_role_records_round_trip_pinned_vibe() {
    let mut m = MaterialRecord::legacy_default();
    m.shading_model = ShadingModel::Stylized;
    m.normal_scale = 2.0;
    m.occlusion_strength = 0.3;
    m.emissive = [1.2, 0.1, 4.0];
    m.specular = 0.4;
    m.double_sided = true;
    m.casts_shadow = false;
    m.receives_shadow = false;
    m.pickable = false;
    m.multiply_vertex_color = false;
    m.wind_animated = true;
    for (i, role) in TextureRole::ALL.into_iter().enumerate() {
        TextureBindingDraft {
            role,
            digest: [(i + 1) as u8; 32],
            transform: TextureTransform {
                offset: [0.1, 0.2],
                scale: [2.0, 3.0],
                rotation: if role == TextureRole::Normal {
                    0.0
                } else {
                    0.5
                },
                tex_coord: 0,
            },
            sampler: TextureSampler::GLTF_DEFAULT,
        }
        .apply(&mut m)
        .unwrap();
    }
    let v = parsed(&source(&m));
    assert_eq!(
        crate::graphical::materials::material_from_value(&v).unwrap(),
        m
    );
    let Value::Record(mut r) = v else {
        panic!("record")
    };
    let Value::Record(binding) = r.get_mut("normal_texture").unwrap() else {
        panic!("binding")
    };
    binding.insert("colour_space".into(), Value::String("srgb".into()));
    assert!(crate::graphical::materials::material_from_value(&Value::Record(r)).is_err());
}
#[test]
fn normal_slot_rotation_reflection_and_legacy_textured_envelopes_refuse() {
    let mut m = MaterialRecord::legacy_default();
    m.normal_texture = [3; 32];
    let value = parsed(&source(&m));
    for (rotation, scale) in [(0.1, [1.0, 1.0]), (0.0, [-1.0, 1.0]), (0.0, [1.0, -1.0])] {
        let mut invalid = m;
        invalid.texture_transforms[1].rotation = rotation;
        invalid.texture_transforms[1].scale = scale;
        assert!(draft::validate(&invalid).is_err());
        assert!(serde_json::to_value(SurfaceDraft::from_material(&invalid)).is_err());
        let Value::Record(mut record) = value.clone() else {
            panic!("material")
        };
        let Value::Record(binding) = record.get_mut("normal_texture").unwrap() else {
            panic!("binding")
        };
        let Value::Record(transform) = binding.get_mut("transform").unwrap() else {
            panic!("transform")
        };
        transform.insert("rotation".into(), Value::F64(rotation as f64));
        transform.insert(
            "scale".into(),
            Value::List(scale.into_iter().map(|n| Value::F64(n as f64)).collect()),
        );
        assert!(crate::graphical::materials::material_from_value(&Value::Record(record)).is_err());
    }
    let geometry =
        parsed("({vertices:[[0.0,0.0,0.0],[1.0,0.0,0.0],[0.0,1.0,0.0]],triangles:[[0,1,2]]});");
    let envelope = Value::Record(std::collections::BTreeMap::from([
        ("geometry".into(), geometry),
        ("material".into(), value),
    ]));
    assert!(crate::graphical::materials::authored_scene(
        &envelope,
        MaterialRecord::legacy_default(),
        crate::graphical::preview::PreviewLimits::default()
    )
    .is_err());
}
#[test]
fn uv_projection_uses_canonical_metres_and_transform_is_atomic() {
    let p = uv::PlanarUvDraft {
        axes: [0, 2],
        metres_per_repeat: 2.0,
    };
    let mut uv = uv::project(&p, &[[2.0, 9.0, 4.0], [4.0, 1.0, 6.0]]).unwrap();
    assert_eq!(uv, [[1.0, 2.0], [2.0, 3.0]]);
    let t = TextureTransform {
        offset: [0.5, -0.25],
        scale: [2.0, 0.5],
        rotation: 0.0,
        tex_coord: 0,
    };
    uv::apply(&t, &mut uv).unwrap();
    assert_eq!(uv[0], [2.5, 0.75]);
    assert_eq!(
        uv::from_value(&parsed(&format!("({});", uv::source(&t).unwrap()))).unwrap(),
        t
    );
    let old = uv.clone();
    let mut bad = t;
    bad.scale = [1_000_000.0; 2];
    assert!(uv::apply(&bad, &mut uv).is_err());
    assert_eq!(uv, old);
    bad.tex_coord = 1;
    assert!(uv::source(&bad).is_err());
}
#[test]
fn verified_dependency_closure_reopens_and_refuses_missing_corrupt_and_role_mismatch() {
    let payload = texture(true, [255; 16]);
    let digest = Sha256::digest(&payload).into();
    let bundle = package(&[(payload.clone(), "image/ktx2")]);
    let resources =
        TextureResources::from_hmc_bytes(&bundle, SurfaceTextureLimits::default()).unwrap();
    let mut m = MaterialRecord::legacy_default();
    m.base_color_texture = digest;
    let deps = resources.dependencies(&[m]).unwrap();
    assert_eq!(deps.len(), 1);
    assert_eq!(deps[0].bytes, payload);
    let authored = format!(
        r#"({{parts:[{{id:"synthetic",semantic_id:1,geometry:{{vertices:[[0.0,0.0,0.0],[1.0,0.0,0.0],[0.0,1.0,0.0]],triangles:[[0,1,2]],uv0:[[0.0,0.0],[1.0,0.0],[0.0,1.0]]}},material:{},translation:[0.0,0.0,0.0],rotation:[0.0,0.0,0.0],scale:[1.0,1.0,1.0]}}]}});"#,
        source(&m).trim_end_matches(';')
    );
    let a = crate::graphical::assembly::admit_assembly(
        &parsed(&authored),
        crate::graphical::preview::PreviewLimits::default(),
    )
    .unwrap();
    let bytes = crate::graphical::assembly::compile_assembly(&a).unwrap();
    let compiled_digest = qualia_core_db::render::compile_10d::compiled_digest(&bytes);
    let source_digest = container_10d::crc32c(authored.as_bytes());
    let (quins, lexicon) = qualia_core_db::render::assets::mesh_to_nquins_with_digests(
        &a.mesh,
        "urn:poet:synthetic-surface-test",
        "vibe",
        source_digest,
        compiled_digest,
    );
    let compiled = CompiledAsset {
        mesh: a.mesh,
        container_10d: bytes,
        texture_dependencies: deps,
        compiled_digest,
        source_digest,
        quins,
        lexicon,
    };
    let reopened = compiled.build_hmc_bundle("synthetic.10d").unwrap();
    let reader = BundleReader::parse(&reopened).unwrap();
    let mut materials = [MaterialRecord::legacy_default()];
    let mut ranges = [container_10d::SubmeshRange {
        first_index: 0,
        index_count: 0,
        material_id: 0,
        semantic_id: 0,
    }];
    container_10d::read_material_section_into(
        reader.get("synthetic.10d").unwrap(),
        3,
        &mut materials,
        &mut ranges,
    )
    .unwrap();
    assert_eq!(materials[0], m);
    assert_eq!(
        qualia_core_db::render::asset_package::resolve_hmc_texture_resource(&reader, &digest)
            .unwrap()
            .bytes,
        payload
    );
    TextureResources::from_hmc_bytes(&reopened, SurfaceTextureLimits::default())
        .unwrap()
        .validate(&[m])
        .unwrap();
    m.normal_texture = digest;
    assert!(resources.validate(&[m]).is_err());
    m.normal_texture = [0; 32];
    m.base_color_texture = [1; 32];
    assert!(resources.validate(&[m]).is_err());
    m.base_color_texture = digest;
    assert!(resources::validate_material_resources(&[m], None).is_err());
    let wrong_payload = texture(true, [0; 16]);
    let mut wrong_key = BundleWriter::new();
    wrong_key
        .add_file(texture_hmc_key(&digest), "image/ktx2", wrong_payload, None)
        .unwrap();
    let wrong_key = wrong_key.build().unwrap();
    let wrong_key =
        TextureResources::from_hmc_bytes(&wrong_key, SurfaceTextureLimits::default()).unwrap();
    assert!(wrong_key.validate(&[m]).is_err());
    let corrupt = package(&[(vec![0; 152], "image/ktx2")]);
    let bad = TextureResources::from_hmc_bytes(&corrupt, SurfaceTextureLimits::default()).unwrap();
    m.base_color_texture = Sha256::digest(vec![0; 152]).into();
    assert!(bad.validate(&[m]).is_err());
}

#[test]
fn shared_linear_orm_dependency_is_retained_once_with_both_map_slots() {
    let payload = texture(
        false,
        [
            255, 128, 200, 255, 255, 128, 200, 255, 255, 128, 200, 255, 255, 128, 200, 255,
        ],
    );
    let digest = Sha256::digest(&payload).into();
    let bytes = package(&[(payload.clone(), "image/ktx2")]);
    let r = TextureResources::from_hmc_bytes(&bytes, SurfaceTextureLimits::default()).unwrap();
    let mut m = MaterialRecord::legacy_default();
    m.metallic_roughness_texture = digest;
    m.occlusion_texture = digest;
    let dependencies = r.dependencies(&[m]).unwrap();
    assert_eq!(dependencies.len(), 1);
    assert_eq!(dependencies[0].bytes, payload);
    assert_eq!(
        crate::graphical::materials::material_from_value(&parsed(&source(&m))).unwrap(),
        m
    );
}
#[test]
fn resource_and_binding_budgets_refuse_without_silent_refinement() {
    let payload = texture(true, [255; 16]);
    let mut m = MaterialRecord::legacy_default();
    m.base_color_texture = Sha256::digest(&payload).into();
    let bytes = package(&[(payload, "image/ktx2")]);
    for l in [
        SurfaceTextureLimits {
            encoded_bytes: 1,
            ..Default::default()
        },
        SurfaceTextureLimits {
            decoded_bytes: 15,
            ..Default::default()
        },
        SurfaceTextureLimits {
            working_bytes: 35,
            ..Default::default()
        },
        SurfaceTextureLimits {
            width: 1,
            ..Default::default()
        },
    ] {
        assert!(TextureResources::from_hmc_bytes(&bytes, l)
            .unwrap()
            .validate(&[m])
            .is_err());
    }
    let r = TextureResources::from_hmc_bytes(
        &bytes,
        SurfaceTextureLimits {
            bindings: 1,
            ..Default::default()
        },
    )
    .unwrap();
    assert!(r.validate(&[m, m]).is_err());
    assert!(TextureResources::from_hmc_bytes(
        &bytes,
        SurfaceTextureLimits {
            package_bytes: 1,
            ..Default::default()
        }
    )
    .is_err());
}
