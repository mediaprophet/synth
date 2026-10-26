//! Trusted immutable HMC snapshot -> canonical decode -> actual PortalGpu residency.
//! The host supplies an authorised, SHA-guarded package selection, never a source path.
use super::binding::TextureRole;
#[path = "source_carrier.rs"]
mod source_carrier;
use qualia_core_db::{
    bundle::BundleReader,
    container_10d::{MaterialRecord, OpacityMode},
    render::{
        asset_package::resolve_hmc_texture_resource,
        assets::TextureDependency,
        gpu::{PortalGpu, TextureColorSpace, TextureMipSemantic},
        texture_decode::{
            decode_hmc_texture_rgba8_into, inspect_hmc_texture_requirements,
            TextureColorSpace as EncodedSpace, TextureDecodeLimits,
        },
    },
};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

#[derive(Clone, Copy, Debug)]
pub struct SurfaceTextureLimits {
    pub package_bytes: usize,
    pub encoded_bytes: usize,
    pub decoded_bytes: usize,
    /// All retained decoded pixels + GPU mip levels + peak codec scratch.
    pub working_bytes: usize,
    pub bindings: usize,
    pub width: u32,
    pub height: u32,
}
impl Default for SurfaceTextureLimits {
    fn default() -> Self {
        Self {
            package_bytes: 64 * 1024 * 1024,
            encoded_bytes: 16 * 1024 * 1024,
            decoded_bytes: 64 * 1024 * 1024,
            working_bytes: 192 * 1024 * 1024,
            bindings: 64,
            width: 4096,
            height: 4096,
        }
    }
}
#[derive(Clone)]
pub struct TextureResources {
    bytes: Arc<[u8]>,
    limits: SurfaceTextureLimits,
}
struct Pixels {
    digest: [u8; 32],
    space: TextureColorSpace,
    semantic: TextureMipSemantic,
    width: u32,
    height: u32,
    rgba: Vec<u8>,
}
struct Prepared {
    pixels: Vec<Pixels>,
    dependencies: Vec<TextureDependency>,
}
fn charge(total: &mut usize, n: usize, max: usize) -> Result<(), String> {
    *total = total
        .checked_add(n)
        .filter(|n| *n <= max)
        .ok_or("Surface texture aggregate budget exceeded")?;
    Ok(())
}
fn mip_bytes(mut w: u32, mut h: u32) -> Result<usize, String> {
    let mut total = 0;
    loop {
        let bytes = (w as usize)
            .checked_mul(h as usize)
            .and_then(|n| n.checked_mul(4))
            .ok_or("Texture mip size overflow")?;
        charge(&mut total, bytes, usize::MAX)?;
        if w == 1 && h == 1 {
            return Ok(total);
        }
        w = (w / 2).max(1);
        h = (h / 2).max(1);
    }
}
impl TextureResources {
    pub(crate) fn snapshot_bytes(&self) -> usize {
        self.bytes.len()
    }
    pub(crate) fn source_byte_len(&self, digest: [u8; 32]) -> Result<usize, String> {
        super::texture_inputs::length(&self.bytes, self.limits, digest)
    }
    pub(crate) fn retained_inputs(
        &self,
    ) -> Result<
        Vec<qualia_core_db::poet_host::invoke::studio_store::checkpoint::ResourceData>,
        String,
    > {
        super::texture_inputs::retained(&self.bytes, self.limits)
    }
    pub(crate) fn preview_input(
        &self,
        digest: [u8; 32],
        available_working_bytes: usize,
    ) -> Result<super::texture_inputs::DetailInput, String> {
        let limits = SurfaceTextureLimits {
            working_bytes: self.limits.working_bytes.min(available_working_bytes),
            ..self.limits
        };
        super::texture_inputs::decode_any(&self.bytes, limits, digest, None)
    }

    pub(crate) fn detail_input(
        &self,
        digest: [u8; 32],
        side: u32,
    ) -> Result<super::texture_inputs::DetailInput, String> {
        super::texture_inputs::decode(&self.bytes, self.limits, digest, side)
    }
    pub(crate) fn mask_input(
        &self,
        digest: [u8; 32],
        side: u32,
    ) -> Result<super::texture_inputs::DetailInput, String> {
        super::texture_inputs::decode_mask(&self.bytes, self.limits, digest, side)
    }
    pub(crate) fn with_dependencies(&self, deps: &[TextureDependency]) -> Result<Self, String> {
        let bytes = super::texture_inputs::merge(&self.bytes, self.limits, deps)?;
        Self::from_hmc_bytes(&bytes, self.limits)
    }
    pub(crate) fn source_dependency(&self, digest: [u8; 32]) -> Result<TextureDependency, String> {
        super::texture_inputs::dependency(&self.bytes, self.limits, digest)
    }

    pub fn from_hmc_bytes(bytes: &[u8], limits: SurfaceTextureLimits) -> Result<Self, String> {
        if bytes.len() > limits.package_bytes
            || limits.bindings == 0
            || limits.width == 0
            || limits.height == 0
        {
            return Err("Surface HMC snapshot/limits exceed host bound".into());
        }
        BundleReader::parse(bytes).map_err(|e| format!("Surface HMC snapshot: {e}"))?;
        Ok(Self {
            bytes: Arc::from(bytes),
            limits,
        })
    }
    fn prepare(&self, materials: &[MaterialRecord]) -> Result<Prepared, String> {
        let bundle =
            BundleReader::parse(&self.bytes).map_err(|e| format!("Surface HMC snapshot: {e}"))?;
        let l = self.limits;
        let limits = TextureDecodeLimits {
            max_encoded_bytes: l.encoded_bytes,
            max_width: l.width,
            max_height: l.height,
            max_decoded_bytes: l.decoded_bytes,
        };
        let mut bindings = 0;
        let mut unique = BTreeSet::new();
        let mut encoded = 0;
        let mut decoded = 0;
        let mut working = 0;
        let mut plans = Vec::new();
        let mut dependencies = BTreeMap::new();
        // Preflight every dependency and aggregate allocation before decoding/upload.
        for material in materials {
            super::draft::validate(material)?;
            for role in TextureRole::ALL {
                let digest = role.digest(material);
                if digest == [0; 32] {
                    continue;
                }
                charge(&mut bindings, 1, l.bindings)?;
                let space = if role.colour_space() == "srgb" {
                    TextureColorSpace::Srgb
                } else {
                    TextureColorSpace::Linear
                };
                let semantic = match role {
                    TextureRole::Normal => TextureMipSemantic::Normal,
                    TextureRole::BaseColor if material.opacity_mode == OpacityMode::Mask => {
                        TextureMipSemantic::alpha_mask(material.alpha_cutoff)
                            .ok_or("Invalid texture alpha cutoff")?
                    }
                    _ if space == TextureColorSpace::Srgb => TextureMipSemantic::Color,
                    _ => TextureMipSemantic::LinearData,
                };
                if !unique.insert((digest, space, semantic)) {
                    continue;
                }
                let resource = resolve_hmc_texture_resource(&bundle, &digest)
                    .map_err(|e| format!("Surface dependency: {e}"))?;
                let (info, scratch) = inspect_hmc_texture_requirements(&resource, limits)
                    .map_err(|e| format!("Surface texture admission: {e}"))?;
                if matches!(
                    (info.color_space, space),
                    (EncodedSpace::Srgb, TextureColorSpace::Linear)
                        | (EncodedSpace::Linear, TextureColorSpace::Srgb)
                ) {
                    return Err(
                        "Texture transfer function conflicts with declared material role".into(),
                    );
                }
                if !dependencies.contains_key(&digest) {
                    charge(&mut encoded, resource.bytes.len(), l.encoded_bytes)?;
                    dependencies.insert(digest, (resource.mime_type, resource.bytes));
                }
                charge(&mut decoded, info.rgba8_bytes, l.decoded_bytes)?;
                charge(&mut working, info.rgba8_bytes, l.working_bytes)?;
                charge(
                    &mut working,
                    mip_bytes(info.width, info.height)?,
                    l.working_bytes,
                )?;
                if working
                    .checked_add(scratch)
                    .filter(|n| *n <= l.working_bytes)
                    .is_none()
                {
                    return Err("Surface codec scratch exceeds aggregate budget".into());
                }
                plans.push((digest, space, semantic, resource, info, scratch));
            }
        }
        // Recheck peak scratch against the final retained set, not only each prefix.
        if plans.iter().any(|p| {
            working
                .checked_add(p.5)
                .filter(|n| *n <= l.working_bytes)
                .is_none()
        }) {
            return Err("Surface aggregate codec scratch exceeds budget".into());
        }
        let mut pixels = Vec::with_capacity(plans.len());
        for (digest, space, semantic, resource, info, scratch_len) in plans {
            let mut rgba = vec![0; info.rgba8_bytes];
            let mut scratch = vec![0; scratch_len];
            let actual = decode_hmc_texture_rgba8_into(&resource, limits, &mut rgba, &mut scratch)
                .map_err(|e| format!("Surface texture decode: {e}"))?;
            if actual != info {
                return Err("Texture decode metadata changed during admission".into());
            }
            pixels.push(Pixels {
                digest,
                space,
                semantic,
                width: info.width,
                height: info.height,
                rgba,
            });
        }
        Ok(Prepared {
            pixels,
            dependencies: dependencies
                .into_iter()
                .map(|(digest, (mime, bytes))| TextureDependency {
                    digest,
                    mime_type: mime.to_owned(),
                    bytes: bytes.to_vec(),
                })
                .collect(),
        })
    }
    pub fn validate(&self, materials: &[MaterialRecord]) -> Result<(), String> {
        self.prepare(materials).map(|_| ())
    }
    pub fn dependencies(
        &self,
        materials: &[MaterialRecord],
    ) -> Result<Vec<TextureDependency>, String> {
        self.prepare(materials).map(|p| p.dependencies)
    }
    pub fn upload(&self, materials: &[MaterialRecord], gpu: &mut PortalGpu) -> Result<(), String> {
        let prepared = self.prepare(materials)?;
        for p in prepared.pixels {
            gpu.upload_resident_texture_rgba8_with_mips(
                p.digest, p.space, p.semantic, p.width, p.height, &p.rgba,
            )
            .map_err(|e| format!("Native surface texture upload: {e:?}"))?;
        }
        Ok(())
    }
}
pub fn validate_material_resources(
    materials: &[MaterialRecord],
    resources: Option<&TextureResources>,
) -> Result<(), String> {
    for m in materials {
        super::draft::validate(m)?;
    }
    if materials
        .iter()
        .any(|m| TextureRole::ALL.iter().any(|r| r.digest(m) != [0; 32]))
    {
        resources
            .ok_or("Textured material needs an authorised native HMC resource snapshot")?
            .validate(materials)?;
    }
    Ok(())
}
