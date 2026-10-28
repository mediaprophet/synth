//! Development source contract `poet.source-layer/v1` on pinned Vibe 0.1.
//!
//! `poet_parameter_parts` owns the last generated base. Native parameter edits
//! derive explicit field overrides from current `poet_parts` by stable local
//! `id` (qualified by unchanged semantic_id), then evaluate the recipe once.
//! `poet_source_layer = {schema, overrides:[{id,fields:{...}}]}` is ordinary
//! editable source data retained by SourceSession/Studio/HMC, never hidden state.
//! Whole material records, local TRS, parent and opaque part fields are retained
//! exactly; geometry belongs to the recipe. Membership, identity or base schema
//! changes require an explicit source migration. Legacy sources remain readable;
//! EnableSourceLayers evaluates an existing recipe into a new baseline once,
//! validates it against actual retained work and leaves edited parts unchanged.
//! UV/atlas/detail/normals require exact geometry correspondence in this profile.
//! No arbitrary topology remapping or general modifier stack is claimed.
//! Direct source authors retain their existing explicit replacement authority.
mod envelope;
mod planning;
#[cfg(test)]
mod restart_tests;
mod snapshot;
mod validation;
use super::source::{identifier, record};
pub(crate) use envelope::validate_render_envelope;
#[cfg(test)]
pub(crate) use snapshot::validation_snapshot;
pub(crate) use snapshot::validation_snapshot_with_budget;
use std::collections::BTreeMap;
pub(crate) use validation::validate_result;
use vibe::Value;
pub(super) const SCHEMA: &str = "poet.source-layer/v1";
pub(super) type Record = BTreeMap<String, Value>;

struct SourceBytes(usize);
impl SourceBytes {
    fn charge(&mut self, bytes: usize) -> Result<(), String> {
        self.0 = self.0.checked_sub(bytes).ok_or(
            "Source layer edit exceeds 60 KiB generated-source budget; accepted source preserved",
        )?;
        Ok(())
    }
}

pub(super) fn source(
    parts: &Value,
    baseline: Option<&Value>,
    layer: Option<&Value>,
    parameter_assignment: &str,
    revision: u64,
) -> Result<String, String> {
    let baseline = baseline.ok_or("Parameter regeneration requires explicit source lineage in poet_parameter_parts; use EnableSourceLayers to qualify the existing recipe without resetting edited parts")?;
    let plan = planning::Plan::new(parts, baseline)?;
    if let Some(layer) = layer {
        super::parameter_safety::equal(layer, layer)?;
        let layer = record(layer)?;
        if layer.len() != 2
            || layer.get("schema") != Some(&Value::String(SCHEMA.into()))
            || !matches!(layer.get("overrides"), Some(Value::List(_)))
        {
            return Err(
                "Unknown or modified source layer schema; preserve source and explicitly migrate"
                    .into(),
            );
        }
        let Value::List(overrides) = &layer["overrides"] else {
            unreachable!()
        };
        if overrides.len() > 256 {
            return Err("Source layer override count exceeds native bounds".into());
        }
        let mut ids = std::collections::BTreeSet::new();
        for value in overrides {
            let value = record(value)?;
            let id = super::source::text(super::source::field(value, "id")?)?;
            let fields = record(super::source::field(value, "fields")?)?;
            source_fields(fields)?;
            if value.len() != 2
                || !ids.insert(id)
                || fields
                    .keys()
                    .any(|key| matches!(key.as_str(), "id" | "semantic_id" | "geometry"))
            {
                return Err(
                    "Malformed source layer overrides; preserve source and migrate explicitly"
                        .into(),
                );
            }
        }
    }
    let mut overrides = Vec::new();
    let mut merge_functions: BTreeMap<String, String> = BTreeMap::new();
    let mut merged_parts = Vec::new();
    let mut signatures = std::collections::BTreeSet::new();
    let mut bytes = SourceBytes(60 * 1024);
    // Fixed assignment/guard syntax plus the already bounded parameter record.
    bytes.charge(
        1024usize
            .checked_add(parameter_assignment.len())
            .ok_or("Source byte budget overflow")?,
    )?;
    for entry in &plan.entries {
        // Inventory's ASCII local-key profile needs no JSON escaping.
        // Charge final source fragments before allocating format/join buffers.
        bytes.charge(entry.id.len() + 52)?;
        let id = serde_json::to_string(entry.id).map_err(|e| e.to_string())?;
        // Pinned 0.1 list lambdas are non-capturing. All member references
        // inside this lambda are local; explicit merge parameters carry overrides.
        let generated = format!("poet_parameter_parts.find(|p| p.id == {id})");
        if entry.changed.is_empty() {
            merged_parts.push(generated);
            continue;
        }
        bytes.charge(entry.id.len() + 24 + 96)?; // override wrapper and merge call
        for key in &entry.changed {
            bytes.charge(2 * key.len() + 40)?;
        }
        let fields = entry
            .changed
            .iter()
            .map(|key| format!("{key}:poet_parts[{}].{key}", entry.index))
            .collect::<Vec<_>>()
            .join(",");
        let slot = overrides.len();
        overrides.push(format!("{{id:{id},fields:{{{fields}}}}}"));
        let mut keys = entry.base.keys().collect::<Vec<_>>();
        keys.extend(
            entry
                .current
                .keys()
                .filter(|key| !entry.base.contains_key(*key)),
        );
        let signature = keys
            .iter()
            .map(|key| (key.as_str(), entry.changed.contains(&key.as_str())))
            .collect::<Vec<_>>();
        if signatures.insert(signature) {
            bytes.charge(160)?; // named pure function declaration
            for key in &keys {
                bytes.charge(2 * key.len() + 12)?;
            }
        }
        let fields = keys
            .into_iter()
            .map(|key| {
                let value = if entry.changed.contains(&key.as_str()) {
                    format!("o.{key}")
                } else {
                    format!("p.{key}")
                };
                format!("{key}:{value}")
            })
            .collect::<Vec<_>>()
            .join(",");
        let group = merge_functions.len();
        let name = merge_functions
            .entry(format!("{{{fields}}}"))
            .or_insert_with(|| format!("poet_layer_merge_{revision}_{group}"));
        merged_parts.push(format!(
            "{name}({generated},poet_source_layer.overrides[{slot}].fields)"
        ));
    }
    let declarations = merge_functions
        .into_iter()
        .map(|(body, name)| {
            format!("pure fn {name}(p:Record,o:Record)->Record{{return {body};}}\n")
        })
        .collect::<String>();
    let declaration = if layer.is_some() { "" } else { "let mut " };
    Ok(format!("{declarations}{declaration}poet_source_layer = {{schema:\"{SCHEMA}\",overrides:[{}]}};\n{parameter_assignment}\npoet_parameter_parts = poet_rebuild_parts(poet_parameters);\npoet_parts = [{}];\n({{parts:poet_parts,_poet_source_layer:{{schema:\"{SCHEMA}\",base:poet_parameter_parts,overrides:poet_source_layer.overrides}}}});",overrides.join(","),merged_parts.join(",")))
}

fn source_fields(record: &Record) -> Result<(), String> {
    if record.len() > 256 || record.keys().any(|key| !identifier(key)) {
        return Err(
            "Unsupported source layer field; preserve all fields and migrate explicitly".into(),
        );
    }
    Ok(())
}
