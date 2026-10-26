//! Vibe 0.1 material values projected onto canonical native material records.
//! Source cells remain authoritative; this is neither a codec nor a host effect.
use super::preview::{mesh_from_invoke, PreviewLimits};
use qualia_core_db::{
    container_10d::{validate_material_bindings, MaterialRecord, OpacityMode, SubmeshRange},
    render::assets::Mesh,
};
use vibe::Value;

fn number(value: &Value) -> Result<f32, String> {
    let n = match value {
        Value::F64(n) => *n,
        Value::I64(n) => *n as f64,
        Value::U64(n) => *n as f64,
        _ => return Err("material factor must be numeric".into()),
    };
    if !n.is_finite() || !(n as f32).is_finite() {
        return Err("material factor must be finite f32".into());
    }
    Ok(n as f32)
}

fn components<const N: usize>(value: &Value) -> Result<[f32; N], String> {
    let Value::List(values) = value else {
        return Err("material colour must be a list".into());
    };
    if values.len() != N {
        return Err(format!("material colour requires {N} components"));
    }
    let mut result = [0.0; N];
    for (slot, value) in result.iter_mut().zip(values) {
        *slot = number(value)?;
    }
    Ok(result)
}

/// Factors are dimensionless. Base colour is linear RGBA in [0,1]; emissive
/// is linear RGB, with the canonical MAT4 HDR-factor bound (preview stays SDR).
/// An explicit colour-space acknowledgement prevents accidental sRGB admission.
/// Texture records require explicit canonical role/colour-space/channel semantics;
/// their bytes are admitted separately from a host-authorised HMC snapshot.
pub fn material_from_value(value: &Value) -> Result<MaterialRecord, String> {
    let Value::Record(record) = value else {
        return Err("material must be a Vibe Material record".into());
    };
    const FIELDS: &[&str] = &[
        "name",
        "id",
        "colour_space",
        "base_color",
        "emissive",
        "metallic",
        "roughness",
        "specular",
        "alpha_cutoff",
        "opacity_mode",
        "double_sided",
    ];
    for key in record.keys() {
        if !FIELDS.contains(&key.as_str())
            && !super::surfaces::draft::EXTRA_FIELDS.contains(&key.as_str())
        {
            return Err(format!(
                "unsupported material property '{key}'; physical material admission is separate"
            ));
        }
    }
    let get = |key: &str| {
        record
            .get(key)
            .ok_or_else(|| format!("material lacks explicit '{key}'"))
    };
    if !matches!(get("colour_space")?, Value::String(s) if s == "linear") {
        return Err("material colour_space must explicitly be 'linear'".into());
    }
    let id = match get("id")? {
        Value::U64(n) if *n != 0 => *n,
        Value::I64(n) if *n > 0 => *n as u64,
        _ => return Err("material id must be a nonzero unsigned integer".into()),
    };
    let opacity_mode = match get("opacity_mode")? {
        Value::String(s) if s == "Opaque" => OpacityMode::Opaque,
        Value::String(s) if s == "Mask" => OpacityMode::Mask,
        Value::String(s) if s == "Blend" => OpacityMode::Blend,
        _ => return Err("material opacity_mode must be Opaque, Mask or Blend".into()),
    };
    let double_sided = match get("double_sided")? {
        Value::Bool(v) => *v,
        _ => return Err("material double_sided must be boolean".into()),
    };
    let mut material = MaterialRecord {
        id,
        opacity_mode,
        double_sided,
        base_color: components(get("base_color")?)?,
        emissive: components(get("emissive")?)?,
        metallic: number(get("metallic")?)?,
        roughness: number(get("roughness")?)?,
        specular: number(get("specular")?)?,
        alpha_cutoff: number(get("alpha_cutoff")?)?,
        ..MaterialRecord::legacy_default()
    };
    super::surfaces::draft::project(record, &mut material)?;
    Ok(material)
}

/// Use the same native material validation as MAT4 encoding and GPU admission.
pub fn validate_factor_material(material: &MaterialRecord) -> Result<(), String> {
    if [
        material.base_color_texture,
        material.normal_texture,
        material.metallic_roughness_texture,
        material.occlusion_texture,
        material.emissive_texture,
        material.stylized_ramp_texture,
    ]
    .iter()
    .any(|digest| *digest != [0; 32])
    {
        return Err(
            "material has texture bindings; verified colour/role admission is required".into(),
        );
    }
    validate_material_bindings(
        &[*material],
        &[SubmeshRange {
            first_index: 0,
            index_count: 3,
            material_id: material.id,
            semantic_id: 0,
        }],
        3,
    )
    .map_err(|error| format!("native material factors: {error}"))
}

/// The source result is a Vibe record projection, not a second document format.
/// Bare geometry keeps the existing caller-selected material compatibility path.
/// Presence of either envelope field requires both: never silently use fallback.
pub fn authored_scene(
    value: &Value,
    caller_material: MaterialRecord,
    limits: PreviewLimits,
) -> Result<(Mesh, MaterialRecord), String> {
    if super::assembly::is_assembly(value) {
        return Err(
            "multi-part assembly requires the shared native Poet assembly projection".into(),
        );
    }
    if let Value::Record(record) = value {
        if record.contains_key("geometry") || record.contains_key("material") {
            if record.len() != 2 {
                return Err("authored scene result requires exactly geometry and material".into());
            }
            let material = material_from_value(
                record
                    .get("material")
                    .ok_or("authored scene lacks material")?,
            )?;
            // This legacy single-mesh API has no trusted-resource parameter.
            // Textures are currently admitted only through native assemblies.
            validate_factor_material(&material)?;
            let mesh = mesh_from_invoke(
                record
                    .get("geometry")
                    .ok_or("authored scene lacks geometry")?,
                limits,
            )?;
            return Ok((mesh, material));
        }
    }
    validate_factor_material(&caller_material)?;
    Ok((mesh_from_invoke(value, limits)?, caller_material))
}

/// UI values are unaccepted drafts until submitted through the normal session.
#[derive(Clone)]
pub struct MaterialDraft {
    pub base_color: [f32; 4],
    pub emissive: [f32; 3],
    pub metallic: f32,
    pub roughness: f32,
    pub specular: f32,
    pub alpha_cutoff: f32,
    pub opacity_mode: OpacityMode,
    pub double_sided: bool,
}
impl Default for MaterialDraft {
    fn default() -> Self {
        Self {
            base_color: [0.36, 0.20, 0.085, 1.0],
            emissive: [0.0; 3],
            metallic: 0.0,
            roughness: 0.65,
            specular: 1.0,
            alpha_cutoff: 0.5,
            opacity_mode: OpacityMode::Opaque,
            double_sided: false,
        }
    }
}
impl MaterialDraft {
    pub fn declaration(&self, revision: u64, id: u64) -> Result<String, String> {
        let material = MaterialRecord {
            id,
            base_color: self.base_color,
            emissive: self.emissive,
            metallic: self.metallic,
            roughness: self.roughness,
            specular: self.specular,
            alpha_cutoff: self.alpha_cutoff,
            opacity_mode: self.opacity_mode,
            double_sided: self.double_sided,
            ..MaterialRecord::legacy_default()
        };
        validate_factor_material(&material)?;
        let opacity = match self.opacity_mode {
            OpacityMode::Opaque => "Opaque",
            OpacityMode::Mask => "Mask",
            OpacityMode::Blend => "Blend",
        };
        Ok(format!("material poet_surface_{revision}: Material\n  id: {id}u64, colour_space: \"linear\",\n  base_color: [{:?}, {:?}, {:?}, {:?}],\n  emissive: [{:?}, {:?}, {:?}],\n  metallic: {:?}, roughness: {:?}, specular: {:?},\n  alpha_cutoff: {:?}, opacity_mode: \"{opacity}\", double_sided: {};\n",
            self.base_color[0], self.base_color[1], self.base_color[2], self.base_color[3],
            self.emissive[0], self.emissive[1], self.emissive[2], self.metallic, self.roughness,
            self.specular, self.alpha_cutoff, self.double_sided))
    }
    pub fn edit_source(&self, revision: u64, id: u64) -> Result<String, String> {
        Ok(format!("{}poet_material = poet_surface_{revision};\n({{geometry: poet_geometry, material: poet_material}});", self.declaration(revision, id)?))
    }
}

pub fn diagnostic_source() -> String {
    // Preserve the actual operator and requirements from the existing recipe.
    let geometry = super::geometry_host::ORGANIC_SURFACE_SOURCE;
    let (requirements, expression) = geometry.split_once('\n').expect("static diagnostic source");
    format!("{requirements}\n{}let mut poet_material = poet_surface_1;\nlet mut poet_geometry = {expression}\n({{geometry: poet_geometry, material: poet_material}});",
        MaterialDraft::default().declaration(1, diagnostic_material_id()).expect("static valid material"))
}

pub fn diagnostic_material_id() -> u64 {
    qualia_core_db::q_hash("urn:ecopunk:diagnostic:preview-material")
}

#[cfg(test)]
#[path = "materials_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "material_pixels.rs"]
mod pixels;
