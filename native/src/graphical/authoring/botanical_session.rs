//! Validate derived organ/mesh correspondence before publishing or compiling.
use super::botanical::{read_recipe, validate_projection, BotanicalRecipe};
use crate::graphical::geometry_host::GeometryHost;
use vibe::session::SourceSession;
use vibe::Value;

/// Conservative early classification; it never traverses UV contents.
/// A malformed/oversized parts inventory takes the stricter budget and is
/// subsequently rejected by the ordinary source-owner validation.
pub(crate) fn requires_uv_budget(parts: &Value, target: &str) -> bool {
    let Value::List(parts) = parts else { return true; };
    if parts.len() > 256 || target.is_empty() || target.len() > 128 { return true; }
    let mut selected = None;
    for part in parts {
        let Value::Record(part) = part else { return true; };
        let Some(Value::String(id)) = part.get("id") else { return true; };
        if id == target {
            if selected.is_some() { return true; }
            let Some(Value::Record(geometry)) = part.get("geometry") else { return true; };
            selected = Some(geometry.contains_key("uv0"));
        }
    }
    selected.unwrap_or(true)
}

pub(crate) fn validate(
    session: &SourceSession,
    host: &GeometryHost,
) -> Result<Option<BotanicalRecipe>, String> {
    let Some(binding) = session.value("poet_botanical") else {
        return Ok(None);
    };
    let parts = session
        .value("poet_parts")
        .ok_or("Botanical source has no parts")?;
    let target = match binding { Value::Record(r) => match r.get("part_id") { Some(Value::String(id)) => Some(id.as_str()), _ => None }, _ => None };
    let limits = host.botanical_limits(target.map_or(true, |id| requires_uv_budget(parts, id)))?;
    let recipe = read_recipe(Some(binding), limits)?.ok_or("Botanical recipe missing")?;
    validate_projection(parts, Some(&recipe), &recipe, limits)?;
    Ok(Some(recipe))
}

#[cfg(test)]
mod uv_budget_tests {
    use super::*;
    use std::collections::BTreeMap;
    fn part(id: &str, uv: bool) -> Value {
        let mut geometry = BTreeMap::new();
        if uv { geometry.insert("uv0".into(), Value::Null); }
        Value::Record(BTreeMap::from([("id".into(),Value::String(id.into())),("geometry".into(), Value::Record(geometry))]))
    }
    #[test]
    fn classifies_bounded_parts_before_touching_uv_content() {
        assert!(!requires_uv_budget(&Value::List(vec![part("branch",false),part("unrelated",true)]),"branch"));
        assert!(requires_uv_budget(&Value::List(vec![part("branch",true),part("unrelated",false)]),"branch"));
        assert!(requires_uv_budget(&Value::List(vec![part("branch",false),part("branch",false)]),"branch"));
        assert!(requires_uv_budget(&Value::List(vec![part("other",false)]),"branch"));
        assert!(requires_uv_budget(&Value::List(vec![Value::Null]),"branch"));
        assert!(requires_uv_budget(&Value::List(vec![part("branch",false);257]),"branch"));
        assert!(requires_uv_budget(&Value::Null,"branch"));
    }
}
