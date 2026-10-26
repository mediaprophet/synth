use super::{
    planning::{parts, topology_bound, Plan},
    SCHEMA,
};
use crate::graphical::authoring::{
    parameter_safety::equal,
    source::{field, record, text},
};
use vibe::Value;

/// Admission inside SourceSession::submit_validated, including each rebuild cell.
/// Previous values must come from the real session before evaluating this cell.
/// No guard means ordinary authored source, with its existing explicit authority.
pub(crate) fn validate_result(
    value: &Value,
    previous_parts: Option<&Value>,
    previous_base: Option<&Value>,
) -> Result<(), String> {
    let Value::Record(output) = value else {
        return Ok(());
    };
    let Some(guard) = output.get("_poet_source_layer") else {
        return Ok(());
    };
    let guard = record(guard)?;
    if text(field(guard, "schema")?)? == "poet.source-layer/migrate-v1" {
        if guard.len() != 3
            || previous_base.is_some()
            || !matches!(field(guard,"overrides")?,Value::List(values) if values.is_empty())
        {
            return Err("Source layer migration cannot replace existing lineage".into());
        }
        let current =
            previous_parts.ok_or("Source layer migration needs existing authored parts")?;
        Plan::new(current, field(guard, "base")?)?;
        if !equal(field(output, "parts")?, current)? {
            return Err("Source layer migration changed accepted parts".into());
        }
        return Ok(());
    }
    if guard.len() != 3 || text(field(guard, "schema")?)? != SCHEMA {
        return Err("Unknown native source layer validation schema".into());
    }
    let current = previous_parts.ok_or("Source layer lacks previous authored parts")?;
    let base = previous_base.ok_or("Source layer lacks previous recipe lineage")?;
    let plan = Plan::new(current, base)?;
    let next = field(guard, "base")?;
    equal(next, next)?;
    let next = parts(next)?;
    let merged_value = field(output, "parts")?;
    equal(merged_value, merged_value)?;
    let merged = parts(merged_value)?;
    let Value::List(overrides) = field(guard, "overrides")? else {
        return Err("Source layer overrides must be a list".into());
    };
    if next.len() != plan.entries.len()
        || merged.keys().ne(next.keys())
        || overrides.len()
            != plan
                .entries
                .iter()
                .filter(|e| !e.changed.is_empty())
                .count()
    {
        return Err("Source layer recipe membership changed; accepted source preserved".into());
    }
    let mut override_values = overrides.iter();
    for entry in &plan.entries {
        let generated = next
            .get(entry.id)
            .ok_or("Source layer local part key disappeared")?;
        if generated.keys().ne(entry.base.keys())
            || !equal(
                field(generated, "semantic_id")?,
                field(entry.base, "semantic_id")?,
            )?
        {
            return Err("Source layer generated schema or qualified local part identity changed; explicitly migrate source".into());
        }
        if (topology_bound(entry.current)?
            || topology_bound(entry.base)?
            || topology_bound(generated)?)
            && !equal(
                field(entry.base, "geometry")?,
                field(generated, "geometry")?,
            )?
        {
            return Err("Source layer topology correspondence unavailable for UV/atlas/detail/normals; accepted source preserved".into());
        }
        if !entry.changed.is_empty() {
            let override_record = record(
                override_values
                    .next()
                    .ok_or("Missing source layer override")?,
            )?;
            if override_record.len() != 2 || text(field(override_record, "id")?)? != entry.id {
                return Err("Invalid source layer part override".into());
            }
            let fields = record(field(override_record, "fields")?)?;
            if fields.len() != entry.changed.len()
                || entry.changed.iter().any(|key| !fields.contains_key(*key))
            {
                return Err("Source layer override fields differ from actual accepted work".into());
            }
            for key in &entry.changed {
                if !equal(&fields[*key], &entry.current[*key])? {
                    return Err(
                        "Source layer override value differs from actual accepted work".into(),
                    );
                }
            }
        }
        let actual = merged[entry.id];
        if actual.len() != entry.current.len() {
            return Err("Source layer lost or invented part fields".into());
        }
        for (key, old) in entry.current {
            let expected = if entry.changed.contains(&key.as_str()) {
                old
            } else {
                &generated[key]
            };
            if !equal(field(actual, key)?, expected)? {
                return Err(
                    "Source layer merged output differs from declared source overrides".into(),
                );
            }
        }
    }
    Ok(())
}
