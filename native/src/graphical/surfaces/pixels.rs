//! Explicit serial native GPU readback; unavailable GPU is a failure, never a skip.
use super::*;
use qualia_core_db::{
    container_10d::{MaterialRecord, ShadingModel, SubmeshRange},
    render::{gpu::PortalGpu, telemetry::SystemTelemetry},
};
use sha2::{Digest, Sha256};
use std::{fs::OpenOptions, io::Write, path::PathBuf};

/// Optional explicit retention of existing test readbacks. No render or source
/// evaluation occurs here. This is calibration evidence, not a model asset.
pub(super) struct Artifacts {
    directory: PathBuf,
    files: usize,
    bytes: usize,
}
impl Artifacts {
    pub(super) fn selected() -> Option<Self> {
        let selected = std::env::var_os("POET_SURFACE_TEST_ARTIFACTS")?;
        let path = PathBuf::from(selected);
        assert!(
            path.is_absolute(),
            "Calibration artifact directory must be absolute"
        );
        let metadata = std::fs::symlink_metadata(&path)
            .expect("explicit owned artifact directory must already exist");
        assert!(
            metadata.is_dir() && !metadata.file_type().is_symlink(),
            "Calibration artifact directory must be a real directory"
        );
        let directory = path
            .canonicalize()
            .expect("resolve selected calibration directory");
        Some(Self {
            directory,
            files: 0,
            bytes: 0,
        })
    }
    pub(super) fn retain(&mut self, name: &str, material: MaterialRecord, rgba8: &[u8]) {
        use crate::graphical::preview::{native_image_uri, PreviewFrame, SourceRef};
        use base64::Engine;
        use qualia_core_db::poet_host::invoke::studio_store::{self, StudioArtifact};
        assert!(
            self.files < 14 && rgba8.len() == 64 * 64 * 4,
            "Calibration retention bound"
        );
        assert!(
            !name.is_empty()
                && name.len() <= 80
                && name
                    .bytes()
                    .all(|b| b.is_ascii_lowercase() || b == b'_' || b == b'-'),
            "Calibration filename outside fixed profile"
        );
        assert_eq!(
            self.directory.canonicalize().unwrap(),
            self.directory,
            "Calibration directory changed"
        );
        let rgba_digest = binding::digest_hex(&Sha256::digest(rgba8).into());
        let id = format!(
            "urn:poet:surface-calibration:{}:{}:{name}",
            std::process::id(),
            self.files
        );
        let mut artifact =
            StudioArtifact::new(&id, "Geometry", "Synthetic native surface GPU calibration");
        artifact.properties.insert("calibration_scope".into(), "Direct native GPU readback of synthetic calibration geometry/maps; dimensions and consumer path declared by the owning test; no human review or model acceptance claimed".into());
        artifact.properties.insert(
            "calibration_material_json".into(),
            serde_json::to_string(&SurfaceDraft::from_material(&material)).unwrap(),
        );
        artifact
            .properties
            .insert("calibration_rgba8_sha256".into(), rgba_digest.clone());
        let (_, stamp) = studio_store::create_artifact_if_absent(artifact).unwrap();
        let frame = PreviewFrame {
            source: SourceRef {
                artifact_id: id,
                stamp,
            },
            session_revision: None,
            execution_epoch: None,
            side: 64,
            rgba8: rgba8.to_vec(),
        };
        let uri = native_image_uri(&frame)
            .expect("encode exact current calibration frame through native preview");
        let encoded = uri
            .strip_prefix("data:image/png;base64,")
            .expect("native preview PNG URI");
        assert!(encoded.len() <= 44 * 1024, "Calibration encoded PNG bound");
        let png = base64::engine::general_purpose::STANDARD
            .decode(encoded)
            .unwrap();
        assert!(
            png.len() <= 32 * 1024 && self.bytes + png.len() <= 512 * 1024,
            "Calibration aggregate PNG bound"
        );
        let destination = self.directory.join(format!("{name}.png"));
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&destination)
            .expect("calibration outputs must be new files in the selected owned directory");
        file.write_all(&png).unwrap();
        file.sync_all().unwrap();
        self.files += 1;
        self.bytes += png.len();
        eprintln!(
            "native_surface_calibration_png path={} rgba8_sha256={rgba_digest} png_sha256={}",
            destination.display(),
            binding::digest_hex(&Sha256::digest(&png).into())
        );
    }
}
fn render(m: MaterialRecord, resources: Option<&TextureResources>) -> Vec<u8> {
    resources::validate_material_resources(&[m], resources).unwrap();
    let mut gpu = PortalGpu::new_offscreen(64, 64, 1).expect("actual native GPU required");
    if let Some(r) = resources {
        r.upload(&[m], &mut gpu)
            .expect("decode and real map upload");
    }
    gpu.set_ambient_enabled(false);
    gpu.set_tensor_projection_enabled(false);
    gpu.set_camera_target(0.0, 0.0, 3.2, 0.0, 0.0, 0.0);
    gpu.set_lighting(0.3, 0.7, 1.0, 1.4, 0.2);
    gpu.set_shadows_enabled(false);
    gpu.set_screen_space_ao_enabled(false);
    gpu.upload_mesh_colored_with_frames_and_materials_and_uv0(
        &[
            [-1.0, -1.0, 0.0],
            [1.0, -1.0, 0.0],
            [1.0, 1.0, 0.0],
            [-1.0, 1.0, 0.0],
        ],
        &[[1.0; 4]; 4],
        Some(&[[0.0, 0.0, 1.0]; 4]),
        None,
        Some(&[[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]]),
        &[0, 1, 2, 0, 2, 3],
        &[m],
        &[SubmeshRange {
            first_index: 0,
            index_count: 6,
            material_id: m.id,
            semantic_id: 1,
        }],
    )
    .expect("actual MAT4 draw and UV-aligned tangent upload");
    gpu.render(0.0, &SystemTelemetry::default()).unwrap();
    let mut rgba = vec![0; gpu.required_rgba8_bytes()];
    gpu.read_rgba8_into(&mut rgba).unwrap();
    rgba
}
fn changed(a: &[u8], b: &[u8]) -> usize {
    a.chunks_exact(4)
        .zip(b.chunks_exact(4))
        .filter(|(a, b)| a != b)
        .count()
}
#[test]
#[ignore = "explicit serial native GPU surface readback; run separately with --ignored --exact"]
fn canonical_maps_and_uv_transform_change_actual_native_pixels() {
    let mut artifacts = Artifacts::selected();
    for role in TextureRole::ALL {
        let mut material = MaterialRecord::legacy_default();
        material.double_sided = true;
        material.base_color = [0.65, 0.45, 0.25, 1.0];
        material.roughness = 0.8;
        let pixel = match role {
            TextureRole::BaseColor => [20, 150, 240, 255],
            TextureRole::Normal => [255, 128, 128, 255],
            TextureRole::MetallicRoughness => {
                material.metallic = 1.0;
                [255, 15, 255, 255]
            }
            TextureRole::Occlusion => [0, 0, 0, 255],
            TextureRole::Emissive => {
                material.emissive = [0.8; 3];
                [0, 0, 0, 255]
            }
            TextureRole::StylizedRamp => {
                material.shading_model = ShadingModel::Stylized;
                [5, 20, 240, 255]
            }
        };
        let before = render(material, None);
        if let Some(sink) = artifacts.as_mut() {
            sink.retain(&format!("{}-before", role.name()), material, &before);
        }
        let payload = super::tests::texture(
            role.colour_space() == "srgb",
            std::array::from_fn(|i| pixel[i % 4]),
        );
        role.set_digest(&mut material, Sha256::digest(&payload).into());
        let bytes = super::tests::package(&[(payload, "image/ktx2")]);
        let resources =
            TextureResources::from_hmc_bytes(&bytes, SurfaceTextureLimits::default()).unwrap();
        let after = render(material, Some(&resources));
        if let Some(sink) = artifacts.as_mut() {
            sink.retain(&format!("{}-after", role.name()), material, &after);
        }
        let count = changed(&before, &after);
        assert!(
            count >= 4,
            "{} map changed only {count} pixels",
            role.name()
        );
        eprintln!("native_surface_map={} changed_pixels={count}", role.name());
    }
    let payload = super::tests::texture(
        true,
        [
            255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 255, 255,
        ],
    );
    let mut m = MaterialRecord::legacy_default();
    m.double_sided = true;
    m.base_color_texture = Sha256::digest(&payload).into();
    let bytes = super::tests::package(&[(payload, "image/ktx2")]);
    let resources =
        TextureResources::from_hmc_bytes(&bytes, SurfaceTextureLimits::default()).unwrap();
    let before = render(m, Some(&resources));
    if let Some(sink) = artifacts.as_mut() {
        sink.retain("uv-before", m, &before);
    }
    m.texture_transforms[0].offset = [0.5, 0.0];
    let after = render(m, Some(&resources));
    if let Some(sink) = artifacts.as_mut() {
        sink.retain("uv-offset", m, &after);
    }
    let count = changed(&before, &after);
    assert!(count >= 4, "map UV transform changed only {count} pixels");
    eprintln!("native_surface_uv_transform changed_pixels={count}");
}
