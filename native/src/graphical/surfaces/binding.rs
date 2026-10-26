//! Fixed MAT4 map roles, transfer functions, channel semantics and sampler state.
use super::uv;
use qualia_core_db::container_10d::{
    MaterialRecord, TextureAddressMode as Address, TextureFilterMode as Filter,
    TextureMinFilter as Min, TextureSampler, TextureTransform,
};
use vibe::Value;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextureRole {
    BaseColor,
    Normal,
    MetallicRoughness,
    Occlusion,
    Emissive,
    StylizedRamp,
}
impl TextureRole {
    pub const ALL: [Self; 6] = [
        Self::BaseColor,
        Self::Normal,
        Self::MetallicRoughness,
        Self::Occlusion,
        Self::Emissive,
        Self::StylizedRamp,
    ];
    pub fn index(self) -> usize {
        self as usize
    }
    pub fn field(self) -> &'static str {
        [
            "base_color_texture",
            "normal_texture",
            "metallic_roughness_texture",
            "occlusion_texture",
            "emissive_texture",
            "stylized_ramp_texture",
        ][self.index()]
    }
    pub fn name(self) -> &'static str {
        [
            "base_color",
            "normal",
            "metallic_roughness",
            "occlusion",
            "emissive",
            "stylized_ramp",
        ][self.index()]
    }
    pub fn colour_space(self) -> &'static str {
        match self {
            Self::Normal | Self::MetallicRoughness | Self::Occlusion => "linear",
            _ => "srgb",
        }
    }
    pub fn channels(self) -> &'static str {
        match self {
            Self::BaseColor => "rgba",
            Self::Normal => "rgb_tangent",
            Self::MetallicRoughness => "g_roughness_b_metallic",
            Self::Occlusion => "r_occlusion",
            Self::Emissive | Self::StylizedRamp => "rgb",
        }
    }
    pub fn digest(self, m: &MaterialRecord) -> [u8; 32] {
        [
            m.base_color_texture,
            m.normal_texture,
            m.metallic_roughness_texture,
            m.occlusion_texture,
            m.emissive_texture,
            m.stylized_ramp_texture,
        ][self.index()]
    }
    pub fn set_digest(self, m: &mut MaterialRecord, d: [u8; 32]) {
        match self {
            Self::BaseColor => m.base_color_texture = d,
            Self::Normal => m.normal_texture = d,
            Self::MetallicRoughness => m.metallic_roughness_texture = d,
            Self::Occlusion => m.occlusion_texture = d,
            Self::Emissive => m.emissive_texture = d,
            Self::StylizedRamp => m.stylized_ramp_texture = d,
        }
    }
}
pub fn digest_hex(d: &[u8; 32]) -> String {
    d.iter().map(|b| format!("{b:02x}")).collect()
}
pub fn parse_digest(s: &str) -> Result<[u8; 32], String> {
    if s.len() != 64
        || !s
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return Err("Texture SHA-256 must be 64 lowercase hexadecimal digits".into());
    }
    let mut d = [0; 32];
    for (i, out) in d.iter_mut().enumerate() {
        *out =
            u8::from_str_radix(&s[2 * i..2 * i + 2], 16).map_err(|_| "Invalid texture digest")?;
    }
    if d == [0; 32] {
        return Err("All-zero texture digest is reserved for no binding".into());
    }
    Ok(d)
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TextureBindingDraft {
    pub role: TextureRole,
    pub digest: [u8; 32],
    pub transform: TextureTransform,
    pub sampler: TextureSampler,
}
fn address(s: &str) -> Result<Address, String> {
    match s {
        "Repeat" => Ok(Address::Repeat),
        "ClampToEdge" => Ok(Address::ClampToEdge),
        "MirroredRepeat" => Ok(Address::MirroredRepeat),
        _ => Err("Unknown texture address mode".into()),
    }
}
fn filter(s: &str) -> Result<Filter, String> {
    match s {
        "Nearest" => Ok(Filter::Nearest),
        "Linear" => Ok(Filter::Linear),
        _ => Err("Unknown texture magnification filter".into()),
    }
}
fn min_filter(s: &str) -> Result<Min, String> {
    match s {
        "Nearest" => Ok(Min::Nearest),
        "Linear" => Ok(Min::Linear),
        "NearestMipmapNearest" => Ok(Min::NearestMipmapNearest),
        "LinearMipmapNearest" => Ok(Min::LinearMipmapNearest),
        "NearestMipmapLinear" => Ok(Min::NearestMipmapLinear),
        "LinearMipmapLinear" => Ok(Min::LinearMipmapLinear),
        _ => Err("Unknown texture minification filter".into()),
    }
}
impl TextureBindingDraft {
    pub fn from_value(role: TextureRole, v: &Value) -> Result<Self, String> {
        let r = uv::record(v)?;
        uv::exact(
            r,
            &[
                "role",
                "sha256",
                "colour_space",
                "channels",
                "transform",
                "wrap_s",
                "wrap_t",
                "mag_filter",
                "min_filter",
            ],
        )?;
        let text = |k| uv::text(uv::field(r, k)?);
        if text("role")? != role.name()
            || text("colour_space")? != role.colour_space()
            || text("channels")? != role.channels()
        {
            return Err("Texture role/colour-space/channel semantics mismatch".into());
        }
        Ok(Self {
            role,
            digest: parse_digest(text("sha256")?)?,
            transform: uv::from_value(uv::field(r, "transform")?)?,
            sampler: TextureSampler {
                wrap_s: address(text("wrap_s")?)?,
                wrap_t: address(text("wrap_t")?)?,
                mag_filter: filter(text("mag_filter")?)?,
                min_filter: min_filter(text("min_filter")?)?,
            },
        })
    }
    pub fn from_material(role: TextureRole, m: &MaterialRecord) -> Option<Self> {
        let digest = role.digest(m);
        (digest != [0; 32]).then_some(Self {
            role,
            digest,
            transform: m.texture_transforms[role.index()],
            sampler: m.texture_samplers[role.index()],
        })
    }
    pub fn apply(&self, m: &mut MaterialRecord) -> Result<(), String> {
        parse_digest(&digest_hex(&self.digest))?;
        uv::validate(&self.transform)?;
        self.role.set_digest(m, self.digest);
        m.texture_transforms[self.role.index()] = self.transform;
        m.texture_samplers[self.role.index()] = self.sampler;
        Ok(())
    }
    pub fn source(&self) -> Result<String, String> {
        parse_digest(&digest_hex(&self.digest))?;
        Ok(format!("{{role:\"{}\",sha256:\"{}\",colour_space:\"{}\",channels:\"{}\",transform:{},wrap_s:\"{:?}\",wrap_t:\"{:?}\",mag_filter:\"{:?}\",min_filter:\"{:?}\"}}",self.role.name(),digest_hex(&self.digest),self.role.colour_space(),self.role.channels(),uv::source(&self.transform)?,self.sampler.wrap_s,self.sampler.wrap_t,self.sampler.mag_filter,self.sampler.min_filter))
    }
}
/// Parse only declared bindings. Absent/Null retains the legacy unbound slot.
pub fn project(
    r: &std::collections::BTreeMap<String, Value>,
    m: &mut MaterialRecord,
) -> Result<(), String> {
    for role in TextureRole::ALL {
        if let Some(v) = r.get(role.field()).filter(|v| !matches!(v, Value::Null)) {
            TextureBindingDraft::from_value(role, v)?.apply(m)?;
        }
    }
    Ok(())
}
