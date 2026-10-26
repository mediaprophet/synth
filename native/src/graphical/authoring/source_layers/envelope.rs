//! Borrowed render transport for one recognized source-owned metadata property.
//! Rendering consumes only root.parts. This does not establish layer correctness:
//! canonical submit/rebuild must first validate_result against preceding source.
use super::super::source::{identifier, record, text};
use vibe::Value;

pub(crate) fn validate_render_envelope(value: &Value) -> Result<(), String> {
    let Value::Record(root) = value else {
        return Ok(());
    };
    let Some(guard) = root.get("_poet_source_layer") else {
        return Ok(());
    };
    let bad = || "Unsupported source-layer render envelope schema or shape".to_string();
    let guard = record(guard).map_err(|_| bad())?;
    let schema = guard
        .get("schema")
        .and_then(|v| text(v).ok())
        .ok_or_else(bad)?;
    if guard.len() != 3 || !matches!(schema, super::SCHEMA | "poet.source-layer/migrate-v1") {
        return Err(bad());
    }
    let Some(Value::List(base)) = guard.get("base") else {
        return Err(bad());
    };
    let Some(Value::List(overrides)) = guard.get("overrides") else {
        return Err(bad());
    };
    if base.is_empty()
        || base.len() > 256
        || overrides.len() > 256
        || (schema == "poet.source-layer/migrate-v1" && !overrides.is_empty())
    {
        return Err(bad());
    }
    for part in base {
        let part = record(part).map_err(|_| bad())?;
        if part.len() > 256
            || part.keys().any(|key| !identifier(key))
            || !matches!(part.get("id"),Some(Value::String(id)) if !id.is_empty() && id.len()<=128)
            || !part
                .get("semantic_id")
                .is_some_and(|v| super::super::source::integer(v).is_ok())
            || !["geometry", "material"]
                .iter()
                .all(|key| matches!(part.get(*key), Some(Value::Record(_))))
            || !["translation", "rotation", "scale"]
                .iter()
                .all(|key| matches!(part.get(*key), Some(Value::List(_))))
        {
            return Err(bad());
        }
    }
    for value in overrides {
        let value = record(value).map_err(|_| bad())?;
        let Some(Value::String(id)) = value.get("id") else {
            return Err(bad());
        };
        let Some(Value::Record(fields)) = value.get("fields") else {
            return Err(bad());
        };
        if value.len() != 2
            || id.is_empty()
            || id.len() > 128
            || fields.len() > 256
            || fields.keys().any(|key| {
                !identifier(key) || matches!(key.as_str(), "id" | "semantic_id" | "geometry")
            })
        {
            return Err(bad());
        }
    }
    Ok(())
}
