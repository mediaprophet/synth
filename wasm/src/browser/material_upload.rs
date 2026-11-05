//! Typed MAT4 material texture-role validation and upload binding.
//!
//! Fixes the browser `(idx as u8) % 4` material color/texture-role upload defect
//! (AR-01, AR-06) by enforcing exact 6-role PBR semantics across Native and WASM
//! without modulo-wrapping or silent slot distortion.

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// Canonical 6-slot MAT4 PBR material texture roles.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum MaterialTextureRole {
    AlbedoBaseColor = 0,
    NormalTangent = 1,
    MetallicRoughness = 2,
    EmissiveOcclusionHeight = 3,
    SubsurfaceTransmission = 4,
    AnisotropyMicroflake = 5,
}

impl MaterialTextureRole {
    pub const ALL: [Self; 6] = [
        Self::AlbedoBaseColor,
        Self::NormalTangent,
        Self::MetallicRoughness,
        Self::EmissiveOcclusionHeight,
        Self::SubsurfaceTransmission,
        Self::AnisotropyMicroflake,
    ];

    pub const fn from_u8(val: u8) -> Result<Self, MaterialUploadError> {
        match val {
            0 => Ok(Self::AlbedoBaseColor),
            1 => Ok(Self::NormalTangent),
            2 => Ok(Self::MetallicRoughness),
            3 => Ok(Self::EmissiveOcclusionHeight),
            4 => Ok(Self::SubsurfaceTransmission),
            5 => Ok(Self::AnisotropyMicroflake),
            other => Err(MaterialUploadError::InvalidRoleId { role_id: other }),
        }
    }

    pub const fn as_u8(self) -> u8 {
        self as u8
    }

    pub const fn name(&self) -> &'static str {
        match self {
            Self::AlbedoBaseColor => "albedo_base_color",
            Self::NormalTangent => "normal_tangent",
            Self::MetallicRoughness => "metallic_roughness",
            Self::EmissiveOcclusionHeight => "emissive_occlusion_height",
            Self::SubsurfaceTransmission => "subsurface_transmission",
            Self::AnisotropyMicroflake => "anisotropy_microflake",
        }
    }

    pub const fn channels(&self) -> &'static str {
        match self {
            Self::AlbedoBaseColor => "rgba",
            Self::NormalTangent => "rgb_tangent",
            Self::MetallicRoughness => "g_roughness_b_metallic",
            Self::EmissiveOcclusionHeight => "rgb_emissive_a_occlusion",
            Self::SubsurfaceTransmission => "rgb_scattering_a_transmission",
            Self::AnisotropyMicroflake => "rg_direction_b_spread",
        }
    }

    pub const fn colour_space(&self) -> &'static str {
        match self {
            Self::AlbedoBaseColor => "srgb",
            _ => "linear",
        }
    }
}

/// Unvalidated input binding for MAT4 texture role upload.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Mat4RoleBinding {
    pub material_id: u64,
    pub role_id: u8,
    pub sha256_digest: [u8; 32],
    pub width: u32,
    pub height: u32,
    pub channel_count: u8,
}

impl Mat4RoleBinding {
    pub fn new(
        material_id: u64,
        role: MaterialTextureRole,
        sha256_digest: [u8; 32],
        width: u32,
        height: u32,
        channel_count: u8,
    ) -> Self {
        Self {
            material_id,
            role_id: role.as_u8(),
            sha256_digest,
            width,
            height,
            channel_count,
        }
    }

    pub fn from_raw(
        material_id: u64,
        role_id: u8,
        sha256_digest: [u8; 32],
        width: u32,
        height: u32,
        channel_count: u8,
    ) -> Self {
        Self { material_id, role_id, sha256_digest, width, height, channel_count }
    }

    pub fn role(&self) -> Result<MaterialTextureRole, MaterialUploadError> {
        MaterialTextureRole::from_u8(self.role_id)
    }
}

/// A validated, bounds-checked MAT4 role binding ready for GPU upload.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidatedRoleBinding {
    pub material_id: u64,
    pub role: MaterialTextureRole,
    pub sha256_digest: [u8; 32],
    pub width: u32,
    pub height: u32,
    pub channel_count: u8,
    pub byte_size: usize,
}

/// An atomically validated batch of MAT4 texture role uploads.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidatedMat4UploadBatch {
    pub bindings: Vec<ValidatedRoleBinding>,
    pub distinct_materials: usize,
    pub total_byte_count: usize,
}

impl ValidatedMat4UploadBatch {
    pub fn find_binding(&self, material_id: u64, role: MaterialTextureRole) -> Option<&ValidatedRoleBinding> {
        self.bindings.iter().find(|b| b.material_id == material_id && b.role == role)
    }
}

/// Typed error returned when material texture upload validation fails.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MaterialUploadError {
    InvalidRoleId { role_id: u8 },
    ConflictingRoleBinding { material_id: u64, role_id: u8 },
    EmptyOrUnverifiedDigest { material_id: u64, role_id: u8 },
    MissingRequiredAlbedo { material_id: u64 },
    InvalidDimensions { material_id: u64, role_id: u8, width: u32, height: u32 },
    InvalidChannelCount { material_id: u64, role_id: u8, channels: u8 },
    EmptyBatch,
    ByteSizeOverflow { material_id: u64, role_id: u8 },
}

impl std::fmt::Display for MaterialUploadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidRoleId { role_id } => write!(f, "Invalid material texture role_id {role_id} (must be 0..=5; never modulo-wrap)"),
            Self::ConflictingRoleBinding { material_id, role_id } => write!(f, "Conflicting duplicate texture binding for material {material_id} at role {role_id}"),
            Self::EmptyOrUnverifiedDigest { material_id, role_id } => write!(f, "Empty or all-zero SHA256 digest for material {material_id} at role {role_id}"),
            Self::MissingRequiredAlbedo { material_id } => write!(f, "Material {material_id} missing mandatory AlbedoBaseColor (role 0) binding"),
            Self::InvalidDimensions { material_id, role_id, width, height } => write!(f, "Invalid texture dimensions {width}x{height} for material {material_id} at role {role_id}"),
            Self::InvalidChannelCount { material_id, role_id, channels } => write!(f, "Invalid channel count {channels} for material {material_id} at role {role_id}"),
            Self::EmptyBatch => write!(f, "Cannot bind empty material upload batch"),
            Self::ByteSizeOverflow { material_id, role_id } => write!(f, "Texture byte size calculation overflow for material {material_id} at role {role_id}"),
        }
    }
}
impl std::error::Error for MaterialUploadError {}

/// Validate and bind MAT4 PBR textures, preventing `(idx % 4)` modulo wrapping.
pub fn validate_and_bind_mat4_textures(
    bindings: &[Mat4RoleBinding],
) -> Result<ValidatedMat4UploadBatch, MaterialUploadError> {
    if bindings.is_empty() {
        return Err(MaterialUploadError::EmptyBatch);
    }
    let mut validated = Vec::with_capacity(bindings.len());
    let mut seen_slots = BTreeSet::new();
    let mut materials = BTreeSet::new();
    let mut materials_with_albedo = BTreeSet::new();
    let mut total_byte_count: usize = 0;

    for binding in bindings {
        let role = MaterialTextureRole::from_u8(binding.role_id)?;
        if binding.sha256_digest == [0; 32] {
            return Err(MaterialUploadError::EmptyOrUnverifiedDigest {
                material_id: binding.material_id,
                role_id: binding.role_id,
            });
        }
        if binding.width == 0 || binding.height == 0 || binding.width > 8192 || binding.height > 8192 {
            return Err(MaterialUploadError::InvalidDimensions {
                material_id: binding.material_id,
                role_id: binding.role_id,
                width: binding.width,
                height: binding.height,
            });
        }
        if binding.channel_count == 0 || binding.channel_count > 4 {
            return Err(MaterialUploadError::InvalidChannelCount {
                material_id: binding.material_id,
                role_id: binding.role_id,
                channels: binding.channel_count,
            });
        }
        if !seen_slots.insert((binding.material_id, binding.role_id)) {
            return Err(MaterialUploadError::ConflictingRoleBinding {
                material_id: binding.material_id,
                role_id: binding.role_id,
            });
        }

        materials.insert(binding.material_id);
        if role == MaterialTextureRole::AlbedoBaseColor {
            materials_with_albedo.insert(binding.material_id);
        }

        let byte_size = (binding.width as usize)
            .checked_mul(binding.height as usize)
            .and_then(|px| px.checked_mul(binding.channel_count as usize))
            .ok_or(MaterialUploadError::ByteSizeOverflow {
                material_id: binding.material_id,
                role_id: binding.role_id,
            })?;

        total_byte_count = total_byte_count
            .checked_add(byte_size)
            .ok_or(MaterialUploadError::ByteSizeOverflow {
                material_id: binding.material_id,
                role_id: binding.role_id,
            })?;

        validated.push(ValidatedRoleBinding {
            material_id: binding.material_id,
            role,
            sha256_digest: binding.sha256_digest,
            width: binding.width,
            height: binding.height,
            channel_count: binding.channel_count,
            byte_size,
        });
    }

    for &mat_id in &materials {
        if !materials_with_albedo.contains(&mat_id) {
            return Err(MaterialUploadError::MissingRequiredAlbedo { material_id: mat_id });
        }
    }

    Ok(ValidatedMat4UploadBatch {
        bindings: validated,
        distinct_materials: materials.len(),
        total_byte_count,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const DUMMY_DIGEST: [u8; 32] = [0xab; 32];

    #[test]
    fn test_roles_4_and_5_never_modulo_wrap() {
        let role_4 = MaterialTextureRole::from_u8(4).expect("role 4 valid");
        assert_eq!(role_4, MaterialTextureRole::SubsurfaceTransmission);
        assert_eq!(role_4.as_u8(), 4);
        assert_ne!(role_4.as_u8() % 4, role_4.as_u8()); // 4 % 4 == 0 (buggy modulo wrap)

        let role_5 = MaterialTextureRole::from_u8(5).expect("role 5 valid");
        assert_eq!(role_5, MaterialTextureRole::AnisotropyMicroflake);
        assert_eq!(role_5.as_u8(), 5);
        assert_ne!(role_5.as_u8() % 4, role_5.as_u8()); // 5 % 4 == 1 (buggy modulo wrap)

        let role_6_err = MaterialTextureRole::from_u8(6).unwrap_err();
        assert_eq!(role_6_err, MaterialUploadError::InvalidRoleId { role_id: 6 });
    }

    #[test]
    fn test_valid_full_6_role_batch() {
        let mat_id = 42;
        let bindings = vec![
            Mat4RoleBinding::new(mat_id, MaterialTextureRole::AlbedoBaseColor, DUMMY_DIGEST, 1024, 1024, 4),
            Mat4RoleBinding::new(mat_id, MaterialTextureRole::NormalTangent, DUMMY_DIGEST, 1024, 1024, 3),
            Mat4RoleBinding::new(mat_id, MaterialTextureRole::MetallicRoughness, DUMMY_DIGEST, 1024, 1024, 2),
            Mat4RoleBinding::new(mat_id, MaterialTextureRole::EmissiveOcclusionHeight, DUMMY_DIGEST, 1024, 1024, 4),
            Mat4RoleBinding::new(mat_id, MaterialTextureRole::SubsurfaceTransmission, DUMMY_DIGEST, 512, 512, 4),
            Mat4RoleBinding::new(mat_id, MaterialTextureRole::AnisotropyMicroflake, DUMMY_DIGEST, 512, 512, 3),
        ];

        let batch = validate_and_bind_mat4_textures(&bindings).expect("full batch valid");
        assert_eq!(batch.distinct_materials, 1);
        assert_eq!(batch.bindings.len(), 6);

        let sub_b = batch.find_binding(mat_id, MaterialTextureRole::SubsurfaceTransmission).unwrap();
        assert_eq!(sub_b.role, MaterialTextureRole::SubsurfaceTransmission);
        assert_eq!(sub_b.width, 512);

        let aniso_b = batch.find_binding(mat_id, MaterialTextureRole::AnisotropyMicroflake).unwrap();
        assert_eq!(aniso_b.role, MaterialTextureRole::AnisotropyMicroflake);
        assert_eq!(aniso_b.width, 512);
    }

    #[test]
    fn test_negative_conflicting_duplicate_role() {
        let mat_id = 99;
        let bindings = vec![
            Mat4RoleBinding::new(mat_id, MaterialTextureRole::AlbedoBaseColor, DUMMY_DIGEST, 512, 512, 4),
            Mat4RoleBinding::new(mat_id, MaterialTextureRole::AlbedoBaseColor, [0xcd; 32], 512, 512, 4),
        ];
        let err = validate_and_bind_mat4_textures(&bindings).unwrap_err();
        assert_eq!(err, MaterialUploadError::ConflictingRoleBinding { material_id: mat_id, role_id: 0 });
    }

    #[test]
    fn test_negative_missing_required_albedo() {
        let mat_id = 77;
        let bindings = vec![
            Mat4RoleBinding::new(mat_id, MaterialTextureRole::NormalTangent, DUMMY_DIGEST, 512, 512, 3),
        ];
        let err = validate_and_bind_mat4_textures(&bindings).unwrap_err();
        assert_eq!(err, MaterialUploadError::MissingRequiredAlbedo { material_id: mat_id });
    }

    #[test]
    fn test_negative_empty_digest_rejection() {
        let mat_id = 88;
        let bindings = vec![
            Mat4RoleBinding::new(mat_id, MaterialTextureRole::AlbedoBaseColor, [0; 32], 512, 512, 4),
        ];
        let err = validate_and_bind_mat4_textures(&bindings).unwrap_err();
        assert_eq!(err, MaterialUploadError::EmptyOrUnverifiedDigest { material_id: mat_id, role_id: 0 });
    }

    #[test]
    fn test_negative_invalid_role_id_greater_than_5() {
        let bindings = vec![
            Mat4RoleBinding::from_raw(1, 0, DUMMY_DIGEST, 256, 256, 4),
            Mat4RoleBinding::from_raw(1, 7, DUMMY_DIGEST, 256, 256, 4),
        ];
        let err = validate_and_bind_mat4_textures(&bindings).unwrap_err();
        assert_eq!(err, MaterialUploadError::InvalidRoleId { role_id: 7 });
    }

    #[test]
    fn test_negative_zero_dimensions_and_channels() {
        let bad_w = vec![Mat4RoleBinding::from_raw(1, 0, DUMMY_DIGEST, 0, 256, 4)];
        assert!(matches!(validate_and_bind_mat4_textures(&bad_w), Err(MaterialUploadError::InvalidDimensions { .. })));

        let bad_ch = vec![Mat4RoleBinding::from_raw(1, 0, DUMMY_DIGEST, 256, 256, 0)];
        assert!(matches!(validate_and_bind_mat4_textures(&bad_ch), Err(MaterialUploadError::InvalidChannelCount { .. })));

        let bad_ch5 = vec![Mat4RoleBinding::from_raw(1, 0, DUMMY_DIGEST, 256, 256, 5)];
        assert!(matches!(validate_and_bind_mat4_textures(&bad_ch5), Err(MaterialUploadError::InvalidChannelCount { .. })));
    }

    #[test]
    fn test_negative_empty_batch() {
        assert_eq!(validate_and_bind_mat4_textures(&[]), Err(MaterialUploadError::EmptyBatch));
    }
}
