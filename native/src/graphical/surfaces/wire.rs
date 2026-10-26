//! Strict owned intent DTOs; these are not source expressions or a material codec.
use super::{SurfaceDraft, TextureBindingDraft, TextureRole};
use qualia_core_db::container_10d::{MaterialRecord, OpacityMode, ShadingModel, TextureTransform};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use vibe::Value;

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UvTransformWire {
    pub offset: [f32; 2],
    pub scale: [f32; 2],
    pub rotation: f32,
    pub tex_coord: u32,
}
impl From<TextureTransform> for UvTransformWire {
    fn from(t: TextureTransform) -> Self {
        Self {
            offset: t.offset,
            scale: t.scale,
            rotation: t.rotation,
            tex_coord: t.tex_coord,
        }
    }
}
impl TryFrom<UvTransformWire> for TextureTransform {
    type Error = String;
    fn try_from(t: UvTransformWire) -> Result<Self, String> {
        let t = Self {
            offset: t.offset,
            scale: t.scale,
            rotation: t.rotation,
            tex_coord: t.tex_coord,
        };
        super::uv::validate(&t)?;
        Ok(t)
    }
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct BindingWire {
    role: String,
    sha256: String,
    colour_space: String,
    channels: String,
    transform: UvTransformWire,
    wrap_s: String,
    wrap_t: String,
    mag_filter: String,
    min_filter: String,
}
impl From<TextureBindingDraft> for BindingWire {
    fn from(b: TextureBindingDraft) -> Self {
        Self {
            role: b.role.name().into(),
            sha256: super::binding::digest_hex(&b.digest),
            colour_space: b.role.colour_space().into(),
            channels: b.role.channels().into(),
            transform: b.transform.into(),
            wrap_s: format!("{:?}", b.sampler.wrap_s),
            wrap_t: format!("{:?}", b.sampler.wrap_t),
            mag_filter: format!("{:?}", b.sampler.mag_filter),
            min_filter: format!("{:?}", b.sampler.min_filter),
        }
    }
}
impl BindingWire {
    fn native(self, role: TextureRole) -> Result<TextureBindingDraft, String> {
        // Reuse source-value admission without parsing or evaluating source text.
        let strings = [
            ("role", self.role),
            ("sha256", self.sha256),
            ("colour_space", self.colour_space),
            ("channels", self.channels),
            ("wrap_s", self.wrap_s),
            ("wrap_t", self.wrap_t),
            ("mag_filter", self.mag_filter),
            ("min_filter", self.min_filter),
        ];
        let mut record = strings
            .into_iter()
            .map(|(k, v)| (k.to_owned(), Value::String(v)))
            .collect::<std::collections::BTreeMap<_, _>>();
        let t: TextureTransform = self.transform.try_into()?;
        record.insert(
            "transform".into(),
            Value::Record(std::collections::BTreeMap::from([
                (
                    "offset".into(),
                    Value::List(t.offset.map(|v| Value::F64(v as f64)).to_vec()),
                ),
                (
                    "scale".into(),
                    Value::List(t.scale.map(|v| Value::F64(v as f64)).to_vec()),
                ),
                ("rotation".into(), Value::F64(t.rotation as f64)),
                ("tex_coord".into(), Value::U64(t.tex_coord as u64)),
            ])),
        );
        TextureBindingDraft::from_value(role, &Value::Record(record))
    }
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SurfaceWire {
    id: u64,
    shading_model: String,
    opacity_mode: String,
    double_sided: bool,
    casts_shadow: bool,
    receives_shadow: bool,
    pickable: bool,
    multiply_vertex_color: bool,
    wind_animated: bool,
    base_color: [f32; 4],
    emissive: [f32; 3],
    metallic: f32,
    roughness: f32,
    specular: f32,
    normal_scale: f32,
    occlusion_strength: f32,
    alpha_cutoff: f32,
    /// Slot order is fixed MAT4 order; each non-null record also declares its role.
    texture_bindings: [Option<BindingWire>; 6],
}
impl From<&SurfaceDraft> for SurfaceWire {
    fn from(d: &SurfaceDraft) -> Self {
        let m = d.material;
        Self {
            id: m.id,
            shading_model: format!("{:?}", m.shading_model),
            opacity_mode: format!("{:?}", m.opacity_mode),
            double_sided: m.double_sided,
            casts_shadow: m.casts_shadow,
            receives_shadow: m.receives_shadow,
            pickable: m.pickable,
            multiply_vertex_color: m.multiply_vertex_color,
            wind_animated: m.wind_animated,
            base_color: m.base_color,
            emissive: m.emissive,
            metallic: m.metallic,
            roughness: m.roughness,
            specular: m.specular,
            normal_scale: m.normal_scale,
            occlusion_strength: m.occlusion_strength,
            alpha_cutoff: m.alpha_cutoff,
            texture_bindings: std::array::from_fn(|i| {
                TextureBindingDraft::from_material(TextureRole::ALL[i], &m).map(Into::into)
            }),
        }
    }
}
impl TryFrom<SurfaceWire> for SurfaceDraft {
    type Error = String;
    fn try_from(w: SurfaceWire) -> Result<Self, String> {
        let mut m = MaterialRecord {
            id: w.id,
            shading_model: match w.shading_model.as_str() {
                "MetallicRoughness" => ShadingModel::MetallicRoughness,
                "Stylized" => ShadingModel::Stylized,
                _ => return Err("Unknown intent shading model".into()),
            },
            opacity_mode: match w.opacity_mode.as_str() {
                "Opaque" => OpacityMode::Opaque,
                "Mask" => OpacityMode::Mask,
                "Blend" => OpacityMode::Blend,
                _ => return Err("Unknown intent opacity mode".into()),
            },
            double_sided: w.double_sided,
            casts_shadow: w.casts_shadow,
            receives_shadow: w.receives_shadow,
            pickable: w.pickable,
            multiply_vertex_color: w.multiply_vertex_color,
            wind_animated: w.wind_animated,
            base_color: w.base_color,
            emissive: w.emissive,
            metallic: w.metallic,
            roughness: w.roughness,
            specular: w.specular,
            normal_scale: w.normal_scale,
            occlusion_strength: w.occlusion_strength,
            alpha_cutoff: w.alpha_cutoff,
            ..MaterialRecord::legacy_default()
        };
        for (i, b) in w.texture_bindings.into_iter().enumerate() {
            if let Some(b) = b {
                b.native(TextureRole::ALL[i])?.apply(&mut m)?;
            }
        }
        super::draft::validate(&m)?;
        Ok(Self::from_material(&m))
    }
}
impl Serialize for SurfaceDraft {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        super::draft::validate(&self.material).map_err(serde::ser::Error::custom)?;
        SurfaceWire::from(self).serialize(serializer)
    }
}
impl<'de> Deserialize<'de> for SurfaceDraft {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        SurfaceWire::deserialize(deserializer)?
            .try_into()
            .map_err(serde::de::Error::custom)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn strict_native_intent_roundtrip_refuses_unknown_nested_and_role_mismatch() {
        let mut m = MaterialRecord::legacy_default();
        m.normal_texture = [3; 32];
        let source = SurfaceDraft::from_material(&m);
        let mut json = serde_json::to_value(&source).unwrap();
        let decoded: SurfaceDraft = serde_json::from_value(json.clone()).unwrap();
        assert_eq!(decoded.material, m);
        json["texture_bindings"][1]["path"] = serde_json::json!("C:/private.png");
        assert!(serde_json::from_value::<SurfaceDraft>(json).is_err());
        let mut json = serde_json::to_value(&source).unwrap();
        json["texture_bindings"][1]["role"] = serde_json::json!("base_color");
        assert!(serde_json::from_value::<SurfaceDraft>(json).is_err());
        let mut json = serde_json::to_value(&source).unwrap();
        json["extra"] = serde_json::json!(true);
        assert!(serde_json::from_value::<SurfaceDraft>(json).is_err());
    }
}
