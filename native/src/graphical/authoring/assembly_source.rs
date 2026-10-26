//! Safe assembly mutations retain local TRS and all unrelated record fields.
use super::{
    source::{field, identifier, integer, record, text},
    AuthoringEdit,
};
use std::collections::{BTreeMap, BTreeSet};
use vibe::Value;
type Record = BTreeMap<String, Value>;

fn quoted(s: &str) -> Result<String, String> {
    serde_json::to_string(s).map_err(|e| e.to_string())
}
fn part_key(id: &str) -> Result<(), String> {
    if id.is_empty()
        || id.len() > 128
        || !id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-' | b'.' | b':' | b'/'))
    {
        return Err("Part key is outside the editable identifier profile".into());
    }
    Ok(())
}
/// Only copy fields representable by canonical member access. Never silently discard extensions.
fn copied(r: &Record, prefix: &str, updates: BTreeMap<&str, String>) -> Result<String, String> {
    let mut fields = Vec::new();
    for key in r.keys() {
        if !identifier(key) {
            return Err(format!(
                "Unsupported source field {key}; edit refused to preserve authored data"
            ));
        }
        fields.push(format!(
            "{key}:{}",
            updates
                .get(key.as_str())
                .cloned()
                .unwrap_or_else(|| format!("{prefix}.{key}"))
        ));
    }
    for (key, value) in updates {
        if !r.contains_key(key) {
            fields.push(format!("{key}:{value}"));
        }
    }
    Ok(format!("{{{}}}", fields.join(",")))
}
pub(super) struct Inventory<'a> {
    parts: Vec<&'a Record>,
    ids: BTreeMap<String, usize>,
}
impl<'a> Inventory<'a> {
    pub(super) fn new(value: &'a Value) -> Result<Self, String> {
        let Value::List(values) = value else {
            return Err("Missing mutable poet_parts".into());
        };
        if values.is_empty() || values.len() > 256 {
            return Err("Authoring part count outside native assembly bounds".into());
        }
        let mut parts = Vec::new();
        let mut ids = BTreeMap::new();
        let mut semantics = BTreeSet::new();
        for (i, value) in values.iter().enumerate() {
            let part = record(value)?;
            let id = text(field(part, "id")?)?;
            part_key(id)?;
            if ids.insert(id.into(), i).is_some() {
                return Err("Duplicate assembly part identifier".into());
            }
            if !semantics.insert(integer(field(part, "semantic_id")?)?) {
                return Err("Duplicate assembly semantic identifier".into());
            }
            parts.push(part);
        }
        let result = Self { parts, ids };
        for part in &result.parts {
            let id = text(field(part, "id")?)?;
            let mut parent = result.parent(id)?;
            let mut seen = BTreeSet::from([id.to_owned()]);
            while let Some(next) = parent {
                if !seen.insert(next.to_owned()) {
                    return Err("Assembly containment cycle refused".into());
                }
                parent = result.parent(next)?;
            }
        }
        Ok(result)
    }
    fn selected(&self, id: &str) -> Result<&'a Record, String> {
        self.ids
            .get(id)
            .map(|i| self.parts[*i])
            .ok_or_else(|| "Unknown selected part or parent".into())
    }
    fn parent(&self, id: &str) -> Result<Option<&'a str>, String> {
        match self.selected(id)?.get("parent") {
            None | Some(Value::Null) => Ok(None),
            Some(v) => {
                let id = text(v)?;
                self.selected(id)?;
                Ok(Some(id))
            }
        }
    }
    fn mapped(&self, replacements: Vec<(&str, String)>) -> Result<String, String> {
        // Share identical schema expressions instead of nesting once per part.
        let mut groups: BTreeMap<String, Vec<&str>> = BTreeMap::new();
        for (id, replacement) in replacements {
            groups.entry(replacement).or_default().push(id);
        }
        let mut expression = "p".to_string();
        for (replacement, ids) in groups.into_iter().rev() {
            let predicate = if ids.len() == 1 {
                format!("p.id == {}", quoted(ids[0])?)
            } else {
                format!(
                    "[{}].contains(p.id)",
                    ids.into_iter()
                        .map(quoted)
                        .collect::<Result<Vec<_>, _>>()?
                        .join(",")
                )
            };
            expression = format!("if {predicate} {{ {replacement} }} else {{ {expression} }}");
        }
        Ok(format!(
            "poet_parts = poet_parts.map(|p| {expression});\n({{parts:poet_parts}});"
        ))
    }
    pub(super) fn replace(
        &self,
        id: &str,
        updates: BTreeMap<&str, String>,
    ) -> Result<String, String> {
        self.mapped(vec![(id, copied(self.selected(id)?, "p", updates)?)])
    }
    pub(super) fn material(
        &self,
        id: u64,
        updates: BTreeMap<&str, String>,
    ) -> Result<String, String> {
        let mut replacements = Vec::new();
        for part in &self.parts {
            let material = record(field(part, "material")?)?;
            if integer(field(material, "id")?)? == id {
                let next = copied(material, "p.material", updates.clone())?;
                replacements.push((
                    text(field(part, "id")?)?,
                    copied(part, "p", BTreeMap::from([("material", next)]))?,
                ));
            }
        }
        if replacements.is_empty() {
            return Err("Unknown selected material".into());
        }
        if replacements
            .iter()
            .all(|(_, next)| next == &replacements[0].1)
        {
            // Canonical equality preserves numeric variants; the same admitted
            // identifier may be authored as signed or unsigned source values.
            let predicate = if id <= i64::MAX as u64 {
                format!("(p.material.id == {id} || p.material.id == {id}u64)")
            } else {
                format!("p.material.id == {id}u64")
            };
            return Ok(format!("poet_parts = poet_parts.map(|p| if {predicate} {{ {} }} else {{ p }});\n({{parts:poet_parts}});", replacements[0].1));
        }
        self.mapped(replacements)
    }
    pub(super) fn shared_part_fields(
        &self,
        id: u64,
        updates: BTreeMap<&str, String>,
    ) -> Result<String, String> {
        let mut replacements = Vec::new();
        for part in &self.parts {
            if integer(field(record(field(part, "material")?)?, "id")?)? == id {
                replacements.push((
                    text(field(part, "id")?)?,
                    copied(part, "p", updates.clone())?,
                ));
            }
        }
        if replacements.is_empty() {
            return Err("Unknown selected material".into());
        }
        self.mapped(replacements)
    }
    /// Reject incompatible UV proposals before evaluating a candidate source cell.
    /// Output validation remains the independent final admission guard.
    pub(super) fn preflight_uv_edit(&self, edit: &AuthoringEdit) -> Result<(), String> {
        use qualia_core_db::container_10d::TextureTransform;
        let selected = match edit {
            AuthoringEdit::UvAtlas { id, .. }
            | AuthoringEdit::UvProjection { id, .. }
            | AuthoringEdit::UvTransform { id, .. } => self.selected(id)?,
            _ => return Ok(()),
        };
        let atlas = selected
            .get("uv_atlas")
            .is_some_and(|v| !matches!(v, Value::Null));
        match edit {
            AuthoringEdit::UvProjection {
                projection: Some(_),
                ..
            } if atlas => {
                return Err(
                    "Remove the atlas explicitly before applying planar UV projection".into(),
                )
            }
            AuthoringEdit::UvTransform { transform, .. }
                if atlas && *transform != TextureTransform::IDENTITY =>
            {
                return Err(
                    "Remove the atlas explicitly before applying a nonidentity part UV transform"
                        .into(),
                )
            }
            AuthoringEdit::UvAtlas { atlas: Some(_), .. } => {
                if selected
                    .get("uv_projection")
                    .is_some_and(|v| !matches!(v, Value::Null))
                {
                    return Err(
                        "Remove planar projection explicitly before applying an atlas".into(),
                    );
                }
                if selected
                    .get("uv_transform")
                    .map(crate::graphical::surfaces::uv::from_value)
                    .transpose()?
                    .is_some_and(|t| t != TextureTransform::IDENTITY)
                {
                    return Err(
                        "Reset the part UV affine transform explicitly before applying an atlas"
                            .into(),
                    );
                }
            }
            _ => {}
        }
        Ok(())
    }
    pub(super) fn structural(&self, edit: &AuthoringEdit) -> Result<String, String> {
        match edit {
            AuthoringEdit::Reparent { id, parent } => {
                self.selected(id)?;
                let mut ancestor = parent.as_deref();
                while let Some(next) = ancestor {
                    if next == id {
                        return Err("Assembly containment cycle refused".into());
                    }
                    ancestor = self.parent(next)?;
                }
                let value = parent
                    .as_deref()
                    .map(quoted)
                    .transpose()?
                    .unwrap_or_else(|| "null".into());
                self.replace(id, BTreeMap::from([("parent", value)]))
            }
            AuthoringEdit::RemoveLeaf { id } => {
                self.selected(id)?;
                if self.parts.len() == 1 {
                    return Err("Removing the final assembly part is refused".into());
                }
                for key in self.ids.keys() {
                    if self.parent(key)? == Some(id.as_str()) {
                        return Err(
                            "Remove children or detach them before removing this part".into()
                        );
                    }
                }
                Ok(format!(
                    "poet_parts = poet_parts.filter(|p| p.id != {});\n({{parts:poet_parts}});",
                    quoted(id)?
                ))
            }
            AuthoringEdit::ReplaceGeometry { id, geometry } => {
                if self
                    .selected(id)?
                    .get("uv_atlas")
                    .is_some_and(|v| !matches!(v, Value::Null))
                {
                    return Err(
                        "Remove the editable UV atlas explicitly before replacing geometry".into(),
                    );
                }
                let selected = self.selected(id)?;
                let g = record(field(selected, "geometry")?)?;
                if g.keys()
                    .any(|key| !matches!(key.as_str(), "vertices" | "triangles"))
                    || selected
                        .get("uv_projection")
                        .is_some_and(|v| !matches!(v, Value::Null))
                    || selected
                        .get("uv_transform")
                        .map(crate::graphical::surfaces::uv::from_value)
                        .transpose()?
                        .is_some_and(|t| {
                            t != qualia_core_db::container_10d::TextureTransform::IDENTITY
                        })
                {
                    return Err("Geometry replacement would discard authored UV/normal/projection or richer data; retain the source and use a new part or an explicitly qualified regeneration operation".into());
                }
                let index = self.ids[id];
                let (requirements, expression) = super::geometry_source::expression(geometry)?;
                // Pinned 0.1 lambdas cannot capture the capability root. Build
                // the replacement outside a lambda: one admitted invocation,
                // no temporary binding, and exact references for other parts.
                let next = copied(
                    selected,
                    &format!("poet_parts[{index}]"),
                    BTreeMap::from([("geometry", expression)]),
                )?;
                let values = (0..self.parts.len())
                    .map(|i| {
                        if i == index {
                            next.clone()
                        } else {
                            format!("poet_parts[{i}]")
                        }
                    })
                    .collect::<Vec<_>>();
                Ok(format!(
                    "{requirements}poet_parts = [{}];\n({{parts:poet_parts}});",
                    values.join(",")
                ))
            }
            AuthoringEdit::AddGeometry {
                id,
                semantic_id,
                material_from,
                geometry,
            } => {
                part_key(id)?;
                if self.parts.len() >= 256 || self.ids.contains_key(id) {
                    return Err("New part identifier is duplicate or assembly is full".into());
                }
                // This bounded control profile uses positive signed-range identifiers.
                if *semantic_id == 0
                    || *semantic_id > i64::MAX as u64
                    || self.parts.iter().any(|p| {
                        integer(field(p, "semantic_id").expect("inventory field")).ok()
                            == Some(*semantic_id)
                    })
                {
                    return Err("New semantic identifier must be distinct and fit a positive Vibe 0.1 integer".into());
                }
                self.selected(material_from)?;
                let template = self.ids[material_from];
                let (requirements, expression) = super::geometry_source::expression(geometry)?;
                Ok(format!("{requirements}poet_parts = poet_parts.push({{id:{},semantic_id:{semantic_id},geometry:{expression},material:poet_parts[{template}].material,translation:[0.0,0.0,0.0],rotation:[0.0,0.0,0.0],scale:[1.0,1.0,1.0]}});\n({{parts:poet_parts}});",quoted(id)?))
            }
            AuthoringEdit::UniqueMaterial { id, material_id } => {
                let selected = self.selected(id)?;
                if *material_id == 0 || *material_id > i64::MAX as u64 {
                    return Err(
                        "New material identifier must fit a positive Vibe 0.1 integer".into(),
                    );
                }
                for part in &self.parts {
                    if integer(field(record(field(part, "material")?)?, "id")?)? == *material_id {
                        return Err("Material identifier is already used".into());
                    }
                }
                let material = copied(
                    record(field(selected, "material")?)?,
                    "p.material",
                    BTreeMap::from([("id", material_id.to_string())]),
                )?;
                self.replace(id, BTreeMap::from([("material", material)]))
            }
            _ => Err("Unsupported structural authoring edit".into()),
        }
    }
}
