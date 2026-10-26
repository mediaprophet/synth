//! Conservative source-owned parameter lineage. No cached document or modifier replay.
use super::source::{field, record};
use vibe::Value;

const MAX_NODES: usize = 1_000_000;
const MAX_DEPTH: usize = 64;
const MAX_TEXT_BYTES: usize = 1024 * 1024;

const REMEDY: &str = "Parameter regeneration refused to preserve authored work. Move retained edits into poet_rebuild_parts in an explicit source revision, or retain this assembly and create a separate derivation. Do not reset poet_parameter_parts onto edited parts as a substitute for merging.";

/// The binding is an explicit authored opt-in, not a proof of arbitrary source provenance.
/// Native regeneration updates it exactly once with the canonical generated parts.
pub(super) fn preflight(parts: &Value, baseline: Option<&Value>) -> Result<(), String> {
    let baseline = baseline.ok_or("Parameter regeneration requires explicit source lineage: after moving all retained edits into poet_rebuild_parts, initialize `let mut poet_parameter_parts=poet_rebuild_parts(poet_parameters); let mut poet_parts=poet_parameter_parts;`. Existing work is preserved; no automatic migration was attempted.")?;
    if !equal(parts, baseline)? {
        return Err(REMEDY.into());
    }
    let Value::List(parts) = baseline else {
        return Err("Parameter baseline must be an authored part list".into());
    };
    for part in parts {
        let part = record(part)?;
        if ["uv_atlas", "detail", "uv_projection"]
            .iter()
            .any(|key| part.get(*key).is_some_and(|v| !matches!(v, Value::Null)))
        {
            return Err(format!("{REMEDY} UV seams, projection and detail/brush coordinates require qualified regeneration; this control cannot remap them."));
        }
        if part
            .get("uv_transform")
            .map(crate::graphical::surfaces::uv::from_value)
            .transpose()?
            .is_some_and(|t| t != qualia_core_db::container_10d::TextureTransform::IDENTITY)
            || record(field(part, "geometry")?)?
                .keys()
                .any(|key| !matches!(key.as_str(), "vertices" | "triangles"))
        {
            return Err(format!("{REMEDY} Authored UVs, normals, transforms or richer geometry require a qualified regeneration operation."));
        }
    }
    Ok(())
}

/// Iterative exact comparison of the bounded editable data profile. Unknown
/// domain values refuse rather than using recursive equality or losing fields.
pub(super) fn equal(a: &Value, b: &Value) -> Result<bool, String> {
    enum Frame<'a> {
        Pair(&'a Value, &'a Value),
        List(std::slice::Iter<'a, Value>, std::slice::Iter<'a, Value>),
        Record(
            std::collections::btree_map::Iter<'a, String, Value>,
            std::collections::btree_map::Iter<'a, String, Value>,
        ),
    }
    let mut frames = vec![Frame::Pair(a, b)];
    let mut nodes = MAX_NODES;
    let mut text = MAX_TEXT_BYTES;
    while let Some(frame) = frames.pop() {
        match frame {
            Frame::Pair(a, b) => {
                nodes = nodes.checked_sub(1).ok_or("Parameter comparison exceeds node bound; preserve source and use a qualified derivation")?;
                match (a, b) {
                    (Value::Null, Value::Null) => {}
                    (Value::Bool(a), Value::Bool(b)) if a == b => {}
                    (Value::I64(a), Value::I64(b)) if a == b => {}
                    (Value::U64(a), Value::U64(b)) if a == b => {}
                    (Value::F64(a), Value::F64(b)) if a.to_bits() == b.to_bits() => {}
                    (Value::String(a), Value::String(b)) => {
                        text = text.checked_sub(a.len()).ok_or("Parameter comparison exceeds text bound")?;
                        if a != b { return Ok(false); }
                    }
                    (Value::List(a), Value::List(b)) if a.len() == b.len() =>
                        frames.push(Frame::List(a.iter(), b.iter())),
                    (Value::Record(a), Value::Record(b)) if a.len() == b.len() =>
                        frames.push(Frame::Record(a.iter(), b.iter())),
                    (Value::List(_), Value::List(_)) | (Value::Record(_), Value::Record(_)) => return Ok(false),
                    (Value::Null | Value::Bool(_) | Value::I64(_) | Value::U64(_) | Value::F64(_) | Value::String(_) | Value::List(_) | Value::Record(_), _) => return Ok(false),
                    _ => return Err("Parameter comparison encountered unsupported domain data; preserve source and use a qualified derivation".into()),
                }
            }
            Frame::List(mut a, mut b) => {
                if let (Some(a_value), Some(b_value)) = (a.next(), b.next()) {
                    frames.push(Frame::List(a, b));
                    frames.push(Frame::Pair(a_value, b_value));
                }
            }
            Frame::Record(mut a, mut b) => {
                if let (Some((a_key, a_value)), Some((b_key, b_value))) = (a.next(), b.next()) {
                    text = text
                        .checked_sub(a_key.len())
                        .ok_or("Parameter comparison exceeds text bound")?;
                    if a_key != b_key {
                        return Ok(false);
                    }
                    frames.push(Frame::Record(a, b));
                    frames.push(Frame::Pair(a_value, b_value));
                }
            }
        }
        if frames.len() > MAX_DEPTH {
            return Err("Parameter comparison exceeds nesting bound; preserve source and use a qualified derivation".into());
        }
    }
    Ok(true)
}
