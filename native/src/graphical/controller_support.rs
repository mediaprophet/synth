//! Existing projections extracted to keep the native controller focused.
use super::*;
pub(super) fn material_edit_id(session: &SourceSession, held: bool) -> Option<u64> {
    if held || !session.is_mutable("poet_geometry") || !session.is_mutable("poet_material") {
        return None;
    }
    let material = crate::graphical::materials::material_from_value(session.value("poet_material")?).ok()?;
    if !matches!(session.value("poet_geometry")?, Value::Record(_)) {
        return None;
    }
    Some(material.id)
}

pub(super) fn source_cells(artifact: &StudioArtifact) -> Result<Vec<&str>, String> {
    artifact
        .nodes
        .iter()
        .filter(|(key, _)| key.starts_with(SOURCE_PREFIX))
        .enumerate()
        .map(|(index, (key, node))| {
            if key != &format!("{SOURCE_PREFIX}{:020}", index + 1) {
                return Err(
                    "stored native source cells are not a contiguous identified sequence".into(),
                );
            }
            node.get("text")
                .map(String::as_str)
                .ok_or_else(|| "stored source cell lacks text".into())
        })
        .collect()
}

