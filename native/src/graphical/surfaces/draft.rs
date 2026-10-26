//! A full visual MAT4 draft; physical material properties are a separate contract.
use super::{
    binding::{self, TextureBindingDraft, TextureRole},
    uv,
};
use qualia_core_db::container_10d::{
    validate_material_bindings, MaterialRecord, ShadingModel, SubmeshRange,
};
use std::collections::BTreeMap;
use vibe::Value;

pub const EXTRA_FIELDS: &[&str] = &[
    "shading_model",
    "normal_scale",
    "occlusion_strength",
    "casts_shadow",
    "receives_shadow",
    "pickable",
    "multiply_vertex_color",
    "wind_animated",
    "base_color_texture",
    "normal_texture",
    "metallic_roughness_texture",
    "occlusion_texture",
    "emissive_texture",
    "stylized_ramp_texture",
];
/// Existing source without extension fields keeps the exact legacy defaults.
pub fn project(r: &BTreeMap<String, Value>, m: &mut MaterialRecord) -> Result<(), String> {
    if let Some(v) = r.get("shading_model") {
        m.shading_model = match uv::text(v)? {
            "MetallicRoughness" => ShadingModel::MetallicRoughness,
            "Stylized" => ShadingModel::Stylized,
            _ => return Err("Unknown native visual shading model".into()),
        };
    }
    if let Some(v) = r.get("normal_scale") {
        m.normal_scale = uv::number(v)?;
    }
    if let Some(v) = r.get("occlusion_strength") {
        m.occlusion_strength = uv::number(v)?;
    }
    for (key, slot) in [
        ("casts_shadow", &mut m.casts_shadow),
        ("receives_shadow", &mut m.receives_shadow),
        ("pickable", &mut m.pickable),
        ("multiply_vertex_color", &mut m.multiply_vertex_color),
        ("wind_animated", &mut m.wind_animated),
    ] {
        if let Some(v) = r.get(key) {
            *slot = match v {
                Value::Bool(b) => *b,
                _ => return Err(format!("Visual flag {key} must be boolean")),
            };
        }
    }
    binding::project(r, m)?;
    validate(m)
}
pub fn validate(m: &MaterialRecord) -> Result<(), String> {
    validate_material_bindings(
        &[*m],
        &[SubmeshRange {
            first_index: 0,
            index_count: 3,
            material_id: m.id,
            semantic_id: 0,
        }],
        3,
    )
    .map_err(|e| format!("Native visual material: {e}"))?;
    for role in TextureRole::ALL {
        if let Some(binding) = TextureBindingDraft::from_material(role, m) {
            uv::validate(&binding.transform)?;
        } else if m.texture_transforms[role.index()]
            != qualia_core_db::container_10d::TextureTransform::IDENTITY
            || m.texture_samplers[role.index()]
                != qualia_core_db::container_10d::TextureSampler::GLTF_DEFAULT
        {
            return Err("Unbound texture slot has dormant transform/sampler state; explicit binding required to retain it".into());
        }
    }
    let normal_transform = m.texture_transforms[TextureRole::Normal.index()];
    if m.normal_texture != [0; 32]
        && (normal_transform.rotation != 0.0 || normal_transform.scale.iter().any(|s| *s <= 0.0))
    {
        return Err("Normal-map rotation/reflection needs transform-aware tangent frames; use part UV0 transformation or an unrotated positive-scale normal slot".into());
    }
    // A dormant ramp is ambiguous authoring; do not silently store an unused map.
    if m.stylized_ramp_texture != [0; 32] && m.shading_model != ShadingModel::Stylized {
        return Err("Stylized ramp requires the Stylized shading model".into());
    }
    Ok(())
}
#[derive(Clone, Debug)]
pub struct SurfaceDraft {
    pub material: MaterialRecord,
}
impl SurfaceDraft {
    pub fn from_material(m: &MaterialRecord) -> Self {
        Self { material: *m }
    }
    /// Full replacement of visual factors/maps only. Caller preserves id/name and all
    /// part/geometry fields via the canonical assembly source's member projection.
    pub fn updates(&self) -> Result<BTreeMap<&'static str, String>, String> {
        let m = &self.material;
        validate(m)?;
        let mut fields = BTreeMap::from([
            ("colour_space", "\"linear\"".into()),
            ("base_color", format!("{:?}", m.base_color)),
            ("emissive", format!("{:?}", m.emissive)),
            ("metallic", format!("{:?}", m.metallic)),
            ("roughness", format!("{:?}", m.roughness)),
            ("specular", format!("{:?}", m.specular)),
            ("normal_scale", format!("{:?}", m.normal_scale)),
            ("occlusion_strength", format!("{:?}", m.occlusion_strength)),
            ("alpha_cutoff", format!("{:?}", m.alpha_cutoff)),
            ("opacity_mode", format!("\"{:?}\"", m.opacity_mode)),
            ("shading_model", format!("\"{:?}\"", m.shading_model)),
            ("double_sided", m.double_sided.to_string()),
            ("casts_shadow", m.casts_shadow.to_string()),
            ("receives_shadow", m.receives_shadow.to_string()),
            ("pickable", m.pickable.to_string()),
            ("multiply_vertex_color", m.multiply_vertex_color.to_string()),
            ("wind_animated", m.wind_animated.to_string()),
        ]);
        for role in TextureRole::ALL {
            fields.insert(
                role.field(),
                TextureBindingDraft::from_material(role, m)
                    .map(|b| b.source())
                    .transpose()?
                    .unwrap_or_else(|| "null".into()),
            );
        }
        Ok(fields)
    }
}
