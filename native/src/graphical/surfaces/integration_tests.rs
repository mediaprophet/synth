use super::*;
use qualia_core_db::{
    bundle::{BundleReader, BundleWriter},
    render::{asset_package::texture_hmc_key, assets::TextureDependency},
};
use sha2::{Digest, Sha256};
use vibe::Value;
pub(super) fn input() -> (Vec<u8>, TextureDependency) {
    let pixels = vec![80u8, 120, 160, 255].repeat(64);
    let mut png = vec![0; 8192];
    let n =
        qualia_core_db::render::surface_authoring::encode_surface_png_into(8, &pixels, &mut png)
            .unwrap();
    png.truncate(n);
    let d = TextureDependency {
        digest: Sha256::digest(&png).into(),
        mime_type: "image/png".into(),
        bytes: png,
    };
    let mut w = BundleWriter::new();
    w.add_file(
        texture_hmc_key(&d.digest),
        &d.mime_type,
        d.bytes.clone(),
        Some(vec![0xa1, 0x61, b'p', 1]),
    )
    .unwrap();
    w.add_file(
        "unknown/retained",
        "future/raw",
        vec![1, 2, 3],
        Some(vec![0xa1, 0x61, b'q', 9]),
    )
    .unwrap();
    (w.build().unwrap(), d)
}
fn crc(bytes: &mut [u8]) {
    bytes[36..40].fill(0);
    let crc = qualia_core_db::container_10d::crc32c(bytes);
    bytes[36..40].copy_from_slice(&crc.to_le_bytes());
}
#[test]
fn merge_preserves_exact_unknown_metadata_and_rejects_rehashed_corruption_or_flags() {
    let (bytes, dep) = input();
    let out =
        texture_inputs::merge(&bytes, Default::default(), std::slice::from_ref(&dep)).unwrap();
    let old = BundleReader::parse(&bytes).unwrap();
    let new = BundleReader::parse(&out).unwrap();
    for e in old.entries() {
        assert_eq!(new.get(&e.key), old.get(&e.key));
        assert_eq!(new.entry(&e.key).unwrap().meta, e.meta);
    }
    let at = old.entry("unknown/retained").unwrap().offset as usize;
    drop(old);
    let mut bad = bytes.clone();
    bad[at] ^= 1;
    crc(&mut bad);
    assert!(BundleReader::parse(&bad).is_ok());
    assert!(texture_inputs::merge(&bad, Default::default(), &[]).is_err());
    let mut flags = bytes.clone();
    flags[6] = 1;
    crc(&mut flags);
    assert!(texture_inputs::merge(&flags, Default::default(), &[]).is_err());
    let small = SurfaceTextureLimits {
        working_bytes: bytes.len() * 2,
        ..Default::default()
    };
    assert!(texture_inputs::merge(&bytes, small, &[]).is_err());
}
#[test]
fn original_authored_map_remains_a_dependency_after_procedural_override_and_can_be_restored() {
    let (bytes, d) = input();
    let r = TextureResources::from_hmc_bytes(&bytes, Default::default()).unwrap();
    let mut m = qualia_core_db::container_10d::MaterialRecord::legacy_default();
    m.id = 1;
    m.base_color_texture = d.digest;
    let mut f = SurfaceDraft::from_material(&m).updates().unwrap();
    f.insert("id", "1".into());
    let material = format!(
        "{{{}}}",
        f.iter()
            .map(|(k, v)| format!("{k}:{v}"))
            .collect::<Vec<_>>()
            .join(",")
    );
    let source=format!("({{parts:[{{id:\"p\",semantic_id:1,geometry:{{vertices:[[0.0,0.0,0.0],[1.0,0.0,0.0],[0.0,1.0,0.0]],triangles:[[0,1,2]],uv0:[[0.0,0.0],[1.0,0.0],[0.0,1.0]]}},material:{material},translation:[0.0,0.0,0.0],rotation:[0.0,0.0,0.0],scale:[1.0,1.0,1.0],detail:{{kind:\"procedural\",version:1,side:8,seed:7,tint:[0.3,0.4,0.5],contrast:0.4,relief:0.1,bands:[2,3],band_strength:0.3,roughness:0.5,metallic:0.2}}}}]}});");
    let mut session = vibe::session::SourceSession::default();
    let mut host = vibe::LocalHost::default();
    let mut value = session
        .submit(&source, &mut host, vibe::Budget::default())
        .unwrap()
        .value;
    let resolved = resolution::resolve(&value, Some(&r)).unwrap();
    let deps = resolved
        .dependencies(&resolved.materials.values().copied().collect::<Vec<_>>())
        .unwrap();
    assert!(deps
        .iter()
        .any(|v| v.digest == d.digest && v.bytes == d.bytes));
    let Value::Record(root) = &mut value else {
        panic!()
    };
    let Some(Value::List(parts)) = root.get_mut("parts") else {
        panic!()
    };
    let Value::Record(p) = &mut parts[0] else {
        panic!()
    };
    p.insert("detail".into(), Value::Null);
    let restored = resolution::resolve(&value, resolved.resources.as_ref()).unwrap();
    assert_eq!(restored.materials[&1].base_color_texture, d.digest);
    assert_eq!(
        restored
            .dependencies(&restored.materials.values().copied().collect::<Vec<_>>())
            .unwrap()[0]
            .bytes,
        d.bytes
    );
}
#[test]
fn source_header_metadata_refuses_snapshot_only_compilation_instead_of_disappearing() {
    let mut w = BundleWriter::new();
    w.add_file(
        qualia_core_db::poet_host::invoke::studio_store::checkpoint::SOURCE_RESOURCE_KEY,
        "future/source",
        vec![1],
        Some(vec![0xa0]),
    )
    .unwrap();
    let bytes = w.build().unwrap();
    assert!(texture_inputs::retained(&bytes, Default::default())
        .unwrap_err()
        .contains("source-entry metadata"));
}

#[test]
fn selected_decode_obeys_remaining_combined_inspection_budget_before_output_allocation() {
    let (bytes, d) = input();
    let r = TextureResources::from_hmc_bytes(&bytes, Default::default()).unwrap();
    assert!(r.preview_input(d.digest, bytes.len() + 64).is_err());
    assert!(r.preview_input(d.digest, 16 * 1024 * 1024).is_ok());
}
