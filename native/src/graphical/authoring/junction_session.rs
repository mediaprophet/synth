//! Tentative-session admission for the separate junction profile.
//! This checks the exact source/geometry projection before binding promotion.
use crate::graphical::geometry_host::GeometryHost;
use vibe::session::SessionBindings;

/// Minimum startup replay profile for this bounded junction slice. This does
/// not imply unlimited replay history or guarantee a particular source fits.
pub(crate) fn require_replay_limits(limits: &crate::graphical::controller::RebuildLimits) -> Result<(), String> {
    if limits.total_geometry_work < 64_000_000 || limits.total_kernel_dispatches < 256 {
        return Err("Junction editing requires the botanical startup replay profile (POET_NATIVE_BOTANICAL_PROFILE=1)".into());
    }
    Ok(())
}
pub(crate) fn is_junction_binding(value: &vibe::Value) -> bool {
    matches!(value, vibe::Value::Record(r)
        if matches!(r.get("schema"), Some(vibe::Value::String(s)) if s == super::botanical::junction_source::SCHEMA
            || s == super::botanical::junction_source::SMOOTH_SCHEMA))
}
pub(crate) fn validate_tentative(bindings: SessionBindings<'_>, host: &mut GeometryHost) -> Result<(), String> {
    let Some(binding) = bindings.value("poet_botanical_junction") else { return Ok(()); };
    if !is_junction_binding(binding) { return Ok(()); }
    if bindings.value("poet_botanical").is_some() || bindings.value("poet_source_layer").is_some()
        || bindings.value("poet_parameter_parts").is_some() {
        return Err("Junction cannot share another procedural projection without explicit migration".into());
    }
    let parts = bindings.value("poet_parts").ok_or("Junction binding requires poet_parts")?;
    let limits = host.junction_limits()?;
    super::botanical::junction_edit::validate_binding_projection(parts, binding, limits, host)?;
    Ok(())
}
