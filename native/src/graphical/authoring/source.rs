//! Bounded source construction; the canonical parser/checker/evaluator executes edits once.
use super::{AuthoringEdit, AuthoringView, PartView};
use crate::graphical::preview::SourceRef;
use std::collections::BTreeMap;
use vibe::Value;

pub(crate) fn number(v: &Value) -> Result<f64, String> {
    let n = match v {
        Value::F64(n) => *n,
        Value::I64(n) if n.unsigned_abs() <= (1u64 << 53) => *n as f64,
        Value::U64(n) if *n <= (1u64 << 53) => *n as f64,
        Value::I64(_) | Value::U64(_) => {
            return Err("Integer is outside the exact f64 control profile".into())
        }
        _ => return Err("Expected numeric authored value".into()),
    };
    if !n.is_finite() {
        return Err("Authored value must be finite".into());
    }
    Ok(n)
}
pub(super) fn record(v: &Value) -> Result<&BTreeMap<String, Value>, String> {
    match v {
        Value::Record(r) => Ok(r),
        _ => Err("Expected authored record".into()),
    }
}
pub(super) fn field<'a>(r: &'a BTreeMap<String, Value>, k: &str) -> Result<&'a Value, String> {
    r.get(k).ok_or_else(|| format!("Authored record lacks {k}"))
}
pub(super) fn integer(v: &Value) -> Result<u64, String> {
    match v {
        Value::U64(n) if *n > 0 => Ok(*n),
        Value::I64(n) if *n > 0 => Ok(*n as u64),
        _ => Err("Expected positive authored integer".into()),
    }
}
pub(super) fn text(v: &Value) -> Result<&str, String> {
    match v {
        Value::String(s) => Ok(s),
        _ => Err("Expected authored string".into()),
    }
}
fn vector<const N: usize>(v: &Value) -> Result<[f64; N], String> {
    let Value::List(xs) = v else {
        return Err("Expected authored vector".into());
    };
    if xs.len() != N {
        return Err("Wrong authored vector dimension".into());
    }
    let mut result = [0.0; N];
    for (slot, value) in result.iter_mut().zip(xs) {
        *slot = number(value)?;
    }
    Ok(result)
}
pub(crate) fn view(
    source: SourceRef,
    value: &Value,
    parameters: Option<&Value>,
) -> Result<AuthoringView, String> {
    let Value::List(parts) = value else {
        return Err("Authoring controls require mutable poet_parts".into());
    };
    if parts.is_empty() || parts.len() > 2048 {
        return Err("Authoring part count outside host bounds".into());
    }
    let mut result = Vec::with_capacity(parts.len());
    for part in parts {
        let p = record(part)?;
        let m = record(field(p, "material")?)?;
        let g = record(field(p, "geometry")?)?;
        let count = |k: &str| -> usize {
            match g.get(k) {
                Some(Value::List(xs)) if matches!(xs.first(), Some(Value::List(_))) => xs.len(),
                Some(Value::List(xs)) => xs.len() / 3,
                _ => 0,
            }
        };
        result.push(PartView {
            uv_atlas: p
                .get("uv_atlas")
                .filter(|v| !matches!(v, Value::Null))
                .map(crate::graphical::surfaces::uv_atlas::from_value)
                .transpose()?,
            detail: p
                .get("detail")
                .filter(|v| !matches!(v, Value::Null))
                .map(crate::graphical::surfaces::detail::DetailRecipe::from_value)
                .transpose()?,
            surface: crate::graphical::surfaces::SurfaceDraft::from_material(
                &crate::graphical::materials::material_from_value(field(p, "material")?)?,
            ),
            uv_transform: p
                .get("uv_transform")
                .map(crate::graphical::surfaces::uv::from_value)
                .transpose()?
                .unwrap_or(qualia_core_db::container_10d::TextureTransform::IDENTITY),
            uv_projection: match p.get("uv_projection") {
                None | Some(Value::Null) => None,
                Some(v) => Some(crate::graphical::surfaces::uv::projection_from_value(v)?),
            },
            id: text(field(p, "id")?)?.into(),
            parent: match p.get("parent") {
                None | Some(Value::Null) => None,
                Some(v) => Some(text(v)?.into()),
            },
            semantic_id: integer(field(p, "semantic_id")?)?,
            translation: vector(field(p, "translation")?)?,
            rotation: vector(field(p, "rotation")?)?,
            scale: vector(field(p, "scale")?)?,
            material_id: integer(field(m, "id")?)?,
            base_color: vector(field(m, "base_color")?)?,
            roughness: number(field(m, "roughness")?)?,
            metallic: number(field(m, "metallic")?)?,
            vertices: count("vertices"),
            triangles: count("triangles"),
        });
    }
    let mut params = Vec::new();
    if let Some(value) = parameters {
        let p = record(value)?;
        if p.len() > 64 {
            return Err("Authoring parameter count outside host bounds".into());
        }
        for (key, value) in p {
            if identifier(key) {
                if let Ok(n) = number(value) {
                    params.push((key.clone(), n));
                }
            }
        }
    }
    Ok(AuthoringView {
        sound_field: None,
        timeline: None,
        timeline_media: Vec::new(),
        material_motion: Vec::new(),
        botanical: None,
        junction: None,
        rig: None,
        source,
        parts: result,
        parameters: params,
    })
}
pub(super) fn identifier(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 64
        && s.bytes()
            .enumerate()
            .all(|(i, b)| b.is_ascii_alphabetic() || b == b'_' || (i > 0 && b.is_ascii_digit()))
}
pub(super) fn literal(n: f64) -> Result<String, String> {
    if !n.is_finite() || n.abs() > 1.0e9 {
        return Err("Edit exceeds finite authored numeric range".into());
    }
    Ok(format!("{n:?}"))
}
pub(super) fn array(xs: &[f64]) -> Result<String, String> {
    Ok(format!(
        "[{}]",
        xs.iter()
            .map(|n| literal(*n))
            .collect::<Result<Vec<_>, _>>()?
            .join(",")
    ))
}
/// Each generated cell is evaluated once by the existing canonical source session.
/// Shape-safe part edits preserve every existing field; unsupported field names fail closed.
#[cfg(test)]
pub(crate) fn edit_source(
    parts: &Value,
    parameters: Option<&Value>,
    edit: &AuthoringEdit,
    revision: u64,
) -> Result<String, String> {
    edit_source_with_baseline(parts, parameters, None, edit, revision)
}
#[cfg(test)]
pub(crate) fn edit_source_with_baseline(
    parts: &Value,
    parameters: Option<&Value>,
    parameter_baseline: Option<&Value>,
    edit: &AuthoringEdit,
    revision: u64,
) -> Result<String, String> {
    edit_source_impl(
        parts,
        parameters,
        parameter_baseline,
        None,
        false,
        edit,
        revision,
    )
}
pub(crate) fn edit_source_with_layers(
    parts: &Value,
    parameters: Option<&Value>,
    parameter_baseline: Option<&Value>,
    source_layer: Option<&Value>,
    edit: &AuthoringEdit,
    revision: u64,
) -> Result<String, String> {
    edit_source_impl(
        parts,
        parameters,
        parameter_baseline,
        source_layer,
        true,
        edit,
        revision,
    )
}
fn edit_source_impl(
    parts: &Value,
    parameters: Option<&Value>,
    parameter_baseline: Option<&Value>,
    source_layer: Option<&Value>,
    layer_profile: bool,
    edit: &AuthoringEdit,
    revision: u64,
) -> Result<String, String> {
    let inventory = super::assembly_source::Inventory::new(parts)?;
    inventory.preflight_uv_edit(edit)?;
    if matches!(edit, AuthoringEdit::EnableSourceLayers) {
        if parameter_baseline.is_some() || source_layer.is_some() {
            return Err(
                "Source lineage already exists; preserve it and use the native parameter controls"
                    .into(),
            );
        }
        record(parameters.ok_or("This source has no poet_parameters binding")?)?;
        return bounded("let mut poet_parameter_parts = poet_rebuild_parts(poet_parameters);\nlet mut poet_source_layer = {schema:\"poet.source-layer/v1\",overrides:[]};\n({parts:poet_parts,_poet_source_layer:{schema:\"poet.source-layer/migrate-v1\",base:poet_parameter_parts,overrides:[]}});".into());
    }
    if let AuthoringEdit::Parameter { key, value } = edit {
        if !layer_profile {
            // Retain the R8 helper contract for callers that explicitly selected it.
            super::parameter_safety::preflight(parts, parameter_baseline)?;
        }
        if !identifier(key) {
            return Err("Invalid authoring parameter name".into());
        }
        let params = record(parameters.ok_or("This source has no poet_parameters binding")?)?;
        if !params.contains_key(key) {
            return Err("Unknown authored parameter".into());
        }
        let mut fields = Vec::new();
        for (name, current) in params {
            if !identifier(name) {
                return Err("Unsupported parameter identifier".into());
            }
            let encoded = if name == key {
                match current {
                    Value::I64(_) | Value::U64(_) => {
                        if value.fract() != 0.0 {
                            return Err("Integer parameter requires a whole number".into());
                        }
                        literal(*value)?;
                        if matches!(current, Value::U64(_)) {
                            if *value < 0.0 {
                                return Err(
                                    "Unsigned parameter requires a nonnegative value".into()
                                );
                            }
                            format!("{value:.0}u64")
                        } else {
                            format!("{value:.0}")
                        }
                    }
                    Value::F64(_) => literal(*value)?,
                    _ => return Err("Unsupported parameter value".into()),
                }
            } else {
                // Preserve integer type/precision; retain unsupported untouched values by reference.
                match current {
                    Value::I64(n) => n.to_string(),
                    Value::U64(n) => format!("{n}u64"),
                    Value::F64(n) => literal(*n)?,
                    _ => format!("poet_parameters.{name}"),
                }
            };
            fields.push(format!("{name}:{encoded}"));
        }
        let assignment = format!("poet_parameters = {{{}}};", fields.join(","));
        return bounded(if layer_profile {
            super::source_layers::source(
                parts,
                parameter_baseline,
                source_layer,
                &assignment,
                revision,
            )?
        } else {
            format!("{assignment}\npoet_parameter_parts = poet_rebuild_parts(poet_parameters);\npoet_parts = poet_parameter_parts;\n({{parts:poet_parts}});")
        });
    }
    let body = match edit {
        AuthoringEdit::UvAtlas { id, atlas } => inventory.replace(
            id,
            BTreeMap::from([(
                "uv_atlas",
                atlas
                    .as_ref()
                    .map(crate::graphical::surfaces::uv_atlas::source)
                    .transpose()?
                    .unwrap_or_else(|| "null".into()),
            )]),
        )?,
        AuthoringEdit::TextureDetail {
            material_id,
            detail,
        } => inventory.shared_part_fields(
            *material_id,
            BTreeMap::from([(
                "detail",
                detail
                    .as_ref()
                    .map(|d| d.source_expression())
                    .transpose()?
                    .unwrap_or_else(|| "null".into()),
            )]),
        )?,
        AuthoringEdit::TextureMaskMorphology { .. } => {
            return Err("Mask morphology is a host-owned resource/source transaction".into());
        }
        AuthoringEdit::Surface {
            material_id,
            surface,
        } => inventory.material(*material_id, surface.updates()?)?,
        AuthoringEdit::UvTransform { id, transform } => inventory.replace(
            id,
            BTreeMap::from([(
                "uv_transform",
                crate::graphical::surfaces::uv::source(transform)?,
            )]),
        )?,
        AuthoringEdit::UvProjection { id, projection } => inventory.replace(
            id,
            BTreeMap::from([(
                "uv_projection",
                projection
                    .as_ref()
                    .map(crate::graphical::surfaces::uv::projection_source)
                    .transpose()?
                    .unwrap_or_else(|| "null".into()),
            )]),
        )?,
        AuthoringEdit::Transform {
            id,
            translation,
            rotation,
            scale,
        } => {
            if scale.iter().any(|v| *v <= 0.0) {
                return Err("Scale must be positive".into());
            }
            inventory.replace(
                id,
                BTreeMap::from([
                    ("translation", array(translation)?),
                    ("rotation", array(rotation)?),
                    ("scale", array(scale)?),
                ]),
            )?
        }
        AuthoringEdit::Material {
            material_id,
            base_color,
            roughness,
            metallic,
        } => {
            if *material_id == 0
                || base_color
                    .iter()
                    .chain([roughness, metallic])
                    .any(|v| !v.is_finite() || !(0.0..=1.0).contains(v))
            {
                return Err("Material factors must be in [0,1]".into());
            }
            inventory.material(
                *material_id,
                BTreeMap::from([
                    ("base_color", array(base_color)?),
                    ("roughness", literal(*roughness)?),
                    ("metallic", literal(*metallic)?),
                ]),
            )?
        }
        _ => inventory.structural(edit)?,
    };
    bounded(body)
}
fn bounded(source: String) -> Result<String, String> {
    if source.len() > 60 * 1024 {
        return Err("Authoring edit source exceeds host bound".into());
    }
    Ok(source)
}
