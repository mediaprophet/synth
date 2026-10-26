//! Typed source projection; validation and mesh construction remain native-owned.
use super::source::literal;
use crate::graphical::geometry_host::revolve_host::validate_profile;
pub(super) fn expression(points: &[[f64; 2]], segments: u32) -> Result<String, String> {
    validate_profile(points, segments as usize).map_err(str::to_string)?;
    let rows = points
        .iter()
        .map(|p| Ok(format!("[{},{}]", literal(p[0])?, literal(p[1])?)))
        .collect::<Result<Vec<_>, String>>()?;
    Ok(format!(
        "{{points:[{}],segments:{segments}}}",
        rows.join(",")
    ))
}
