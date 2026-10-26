//! Editable UV0 affine projection; original geometry UVs remain in authored source.
use qualia_core_db::container_10d::TextureTransform;
use std::collections::BTreeMap;
use vibe::Value;

pub type UvTransformDraft = TextureTransform;

#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlanarUvDraft {
    pub axes: [usize; 2],
    pub metres_per_repeat: f32,
}
pub fn projection_from_value(v: &Value) -> Result<PlanarUvDraft, String> {
    let r = record(v)?;
    exact(r, &["axes", "metres_per_repeat"])?;
    let axes = match text(field(r, "axes")?)? {
        "XY" => [0, 1],
        "XZ" => [0, 2],
        "YZ" => [1, 2],
        _ => return Err("UV projection axes must be XY, XZ or YZ".into()),
    };
    let p = PlanarUvDraft {
        axes,
        metres_per_repeat: number(field(r, "metres_per_repeat")?)?,
    };
    projection_source(&p)?;
    Ok(p)
}
pub fn projection_source(p: &PlanarUvDraft) -> Result<String, String> {
    let axes = match p.axes {
        [0, 1] => "XY",
        [0, 2] => "XZ",
        [1, 2] => "YZ",
        _ => return Err("UV planar axes outside editable profile".into()),
    };
    if !p.metres_per_repeat.is_finite() || !(0.000001..=1_000_000.0).contains(&p.metres_per_repeat)
    {
        return Err("UV metres per repeat must be finite in [0.000001,1000000]".into());
    }
    Ok(format!(
        "{{axes:\"{axes}\",metres_per_repeat:{:?}}}",
        p.metres_per_repeat
    ))
}
pub fn project(p: &PlanarUvDraft, positions: &[[f32; 3]]) -> Result<Vec<[f32; 2]>, String> {
    projection_source(p)?;
    let mut uv = vec![[0.0; 2]; positions.len()];
    qualia_core_db::render::surface_authoring::planar_uvs_into(
        positions,
        p.axes,
        p.metres_per_repeat,
        &mut uv,
    )
    .map_err(|e| format!("Native planar UV projection: {e:?}"))?;
    apply(&TextureTransform::IDENTITY, &mut uv)?;
    Ok(uv)
}

pub(crate) fn number(v: &Value) -> Result<f32, String> {
    let n = match v {
        Value::F64(n) => *n,
        Value::I64(n) => *n as f64,
        Value::U64(n) => *n as f64,
        _ => return Err("Surface value must be numeric".into()),
    };
    if !n.is_finite() || !(n as f32).is_finite() {
        return Err("Surface number must be finite f32".into());
    }
    Ok(n as f32)
}
pub(crate) fn vector<const N: usize>(v: &Value) -> Result<[f32; N], String> {
    let Value::List(xs) = v else {
        return Err("Surface vector must be a list".into());
    };
    if xs.len() != N {
        return Err("Surface vector has wrong dimension".into());
    }
    let mut out = [0.0; N];
    for (o, v) in out.iter_mut().zip(xs) {
        *o = number(v)?;
    }
    Ok(out)
}
pub(crate) fn record(v: &Value) -> Result<&BTreeMap<String, Value>, String> {
    match v {
        Value::Record(r) => Ok(r),
        _ => Err("Surface record required".into()),
    }
}
pub(crate) fn field<'a>(r: &'a BTreeMap<String, Value>, k: &str) -> Result<&'a Value, String> {
    r.get(k)
        .ok_or_else(|| format!("Surface lacks explicit {k}"))
}
pub(crate) fn text(v: &Value) -> Result<&str, String> {
    match v {
        Value::String(s) => Ok(s),
        _ => Err("Surface string required".into()),
    }
}
pub(crate) fn exact(r: &BTreeMap<String, Value>, names: &[&str]) -> Result<(), String> {
    if r.len() != names.len() || r.keys().any(|k| !names.contains(&k.as_str())) {
        return Err("Surface record has missing/unknown fields".into());
    }
    Ok(())
}
pub fn validate(t: &TextureTransform) -> Result<(), String> {
    if t.tex_coord != 0
        || t.offset
            .iter()
            .chain(&t.scale)
            .chain([&t.rotation])
            .any(|v| !v.is_finite() || v.abs() > 1_000_000.0)
        || t.scale.contains(&0.0)
    {
        return Err(
            "UV transform needs finite bounded offset/scale/radians, nonzero scale and UV0".into(),
        );
    }
    Ok(())
}
pub fn from_value(v: &Value) -> Result<TextureTransform, String> {
    let r = record(v)?;
    exact(r, &["offset", "scale", "rotation", "tex_coord"])?;
    if !matches!(field(r, "tex_coord")?, Value::I64(0) | Value::U64(0)) {
        return Err("Only vertex UV0 is supported".into());
    }
    let t = TextureTransform {
        offset: vector(field(r, "offset")?)?,
        scale: vector(field(r, "scale")?)?,
        rotation: number(field(r, "rotation")?)?,
        tex_coord: 0,
    };
    validate(&t)?;
    Ok(t)
}
pub fn source(t: &TextureTransform) -> Result<String, String> {
    validate(t)?;
    Ok(format!(
        "{{offset:{:?},scale:{:?},rotation:{:?},tex_coord:0}}",
        t.offset, t.scale, t.rotation
    ))
}
pub fn apply(t: &TextureTransform, uv: &mut [[f32; 2]]) -> Result<(), String> {
    validate(t)?;
    let (s, c) = t.rotation.sin_cos();
    // Preflight the complete stream before mutation, including f32 overflow.
    let mapped = |p: [f32; 2]| {
        [
            t.offset[0] + c * t.scale[0] * p[0] - s * t.scale[1] * p[1],
            t.offset[1] + s * t.scale[0] * p[0] + c * t.scale[1] * p[1],
        ]
    };
    if uv.iter().any(|p| {
        p.iter().any(|v| !v.is_finite())
            || mapped(*p)
                .iter()
                .any(|v| !v.is_finite() || v.abs() > 1_000_000.0)
    }) {
        return Err("Transformed UV exceeds native bound".into());
    }
    for p in uv {
        *p = mapped(*p);
    }
    Ok(())
}
