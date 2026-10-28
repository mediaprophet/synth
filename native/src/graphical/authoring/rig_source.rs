//! Existing-source-only rig binding projection and candidate source construction.
//! Read/parse is not HMC admission or authorization. The host must stage a
//! candidate against its authorized resource and current selected viewport.
use crate::graphical::{
    preview::SourceRef,
    rig_binding::{parse_descriptor_source_literal, RigSourceDescriptor},
    rig_binding::rig_candidate::StagedRigControl,
};
use vibe::Value;

/// Inspect an existing source binding. It remains unadmitted until a host
/// verifies the resource, geometry, span and authority against this source.
pub(crate) fn read_binding(
    current: Option<&Value>, expected: &SourceRef,
) -> Result<Option<RigSourceDescriptor>, String> {
    expected.check_current()?;
    let Some(value) = current else { return Ok(None) };
    let Value::String(text) = value else {
        return Err("Unsupported rig source binding; preserve source".into());
    };
    let descriptor = parse_descriptor_source_literal(text)?;
    if descriptor.schema != "poet.bound-rig/v1"
        || descriptor.source_artifact_id != expected.artifact_id
        || descriptor.part_id.is_empty() || descriptor.bind_id.is_empty()
        || descriptor.resource_key.is_empty()
    {
        return Err("Unsupported or foreign rig source binding; preserve source".into());
    }
    expected.check_current()?;
    Ok(Some(descriptor))
}

/// Create only an absent binding from a candidate admitted by the actual
/// selected resource and viewport. Never replace an existing binding.
pub(crate) fn adopt_source(
    current: Option<&Value>, expected: &SourceRef, candidate: &StagedRigControl,
) -> Result<String, String> {
    if current.is_some() || candidate.expected() != expected {
        return Err("Rig binding already exists or source context changed; preserve source".into());
    }
    let quoted = serde_json::to_string(candidate.literal()).map_err(|e| format!("rig source encoding: {e}"))?;
    let cell = format!("let mut poet_rig = {quoted};\n({{parts:poet_parts}});");
    if cell.len() > 60 * 1024 { return Err("Rig adoption exceeds source cell budget; preserve source".into()); }
    expected.check_current()?;
    Ok(cell)
}

/// Build a Vibe source cell for one already admitted candidate. Existing
/// binding and mutability are mandatory; source adoption is not implemented.
/// The canonical editor must enforce authority and expected source again while
/// submitting this cell, and retain its existing history/checkpoint path.
pub(crate) fn edit_source(
    current: Option<&Value>, mutable: bool, expected: &SourceRef,
    candidate: &StagedRigControl,
) -> Result<String, String> {
    if !mutable || candidate.expected() != expected {
        return Err("Rig binding is immutable or source context changed; preserve source".into());
    }
    let old = read_binding(current, expected)?.ok_or("Rig source binding is absent; adoption is not supported")?;
    if &old != candidate.before() {
        return Err("Rig binding changed after admission; preserve source".into());
    }
    let quoted = serde_json::to_string(candidate.literal()).map_err(|e| format!("rig source encoding: {e}"))?;
    let cell = format!("poet_rig = {quoted};\n({{parts:poet_parts}});");
    if cell.len() > 60 * 1024 {
        return Err("Rig edit exceeds source cell budget; preserve source".into());
    }
    expected.check_current()?;
    Ok(cell)
}
