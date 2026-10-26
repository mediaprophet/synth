use super::{source_fields, Record};
use crate::graphical::authoring::{
    assembly_source::Inventory,
    parameter_safety::equal,
    source::{field, record, text},
};
use std::collections::BTreeMap;
use vibe::Value;

pub(super) struct Entry<'a> {
    pub index: usize,
    pub id: &'a str,
    pub current: &'a Record,
    pub base: &'a Record,
    pub changed: Vec<&'a str>,
}
pub(super) struct Plan<'a> {
    pub entries: Vec<Entry<'a>>,
}
pub(super) fn parts(value: &Value) -> Result<BTreeMap<&str, &Record>, String> {
    Inventory::new(value)?;
    let Value::List(values) = value else {
        unreachable!()
    };
    values
        .iter()
        .map(|value| {
            let value = record(value)?;
            source_fields(value)?;
            Ok((text(field(value, "id")?)?, value))
        })
        .collect()
}
impl<'a> Plan<'a> {
    pub fn new(current: &'a Value, base: &'a Value) -> Result<Self, String> {
        // Bounded, type-exact traversal also rejects unsupported opaque domains.
        equal(current, current)?;
        equal(base, base)?;
        let baseline = parts(base)?;
        let now = parts(current)?;
        if baseline.keys().ne(now.keys()) {
            return Err("Parameter regeneration refused to preserve authored work: part membership changed; explicitly migrate the source recipe".into());
        }
        let Value::List(values) = current else {
            unreachable!()
        };
        let mut entries = Vec::new();
        for (index, value) in values.iter().enumerate() {
            let current = record(value)?;
            let id = text(field(current, "id")?)?;
            let base = baseline[id];
            if base.keys().any(|key| !current.contains_key(key)) {
                return Err(
                    "Source layer cannot remove generated fields; migrate explicitly".into(),
                );
            }
            for key in ["semantic_id", "geometry"] {
                if !equal(field(current, key)?, field(base, key)?)? {
                    return Err(format!("Parameter regeneration refused to preserve authored work: {key} override requires an explicit source recipe migration"));
                }
            }
            let mut changed = Vec::new();
            for (key, value) in current {
                if base
                    .get(key)
                    .map(|old| equal(value, old))
                    .transpose()?
                    .unwrap_or(false)
                {
                    continue;
                }
                changed.push(key.as_str());
            }
            entries.push(Entry {
                index,
                id,
                current,
                base,
                changed,
            });
        }
        Ok(Self { entries })
    }
}

pub(super) fn topology_bound(part: &Record) -> Result<bool, String> {
    if ["uv_atlas", "detail", "uv_projection"]
        .iter()
        .any(|key| part.get(*key).is_some_and(|v| !matches!(v, Value::Null)))
    {
        return Ok(true);
    }
    if part
        .get("uv_transform")
        .map(crate::graphical::surfaces::uv::from_value)
        .transpose()?
        .is_some_and(|t| t != qualia_core_db::container_10d::TextureTransform::IDENTITY)
    {
        return Ok(true);
    }
    Ok(record(field(part, "geometry")?)?
        .keys()
        .any(|key| !matches!(key.as_str(), "vertices" | "triangles")))
}
