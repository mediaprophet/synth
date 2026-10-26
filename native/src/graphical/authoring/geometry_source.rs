//! Typed, bounded recipes. The canonical native host retains authority and resource limits.
use super::{GeometryRecipe, source::literal};

pub(super) fn dimension(n: f64) -> Result<String, String> {
    if n <= 0.0 || n < 1.0e-6 || n > 1.0e6 {
        return Err("Geometry dimensions must be in [0.000001,1000000] metres".into());
    }
    literal(n)
}
fn segments(n: u32) -> Result<u32, String> {
    if !(3..=256).contains(&n) { return Err("Geometry segments must be in [3,256]".into()); }
    Ok(n)
}
pub(super) fn expression(recipe: &GeometryRecipe) -> Result<(String, String), String> {
    let (operation, args) = match recipe {
        GeometryRecipe::Sphere { radius, lat_segments, lon_segments } => (
            "ComputationalGeometry.uv_sphere",
            format!("{{radius:{},lat_segments:{},lon_segments:{}}}", dimension(*radius)?, segments(*lat_segments)?, segments(*lon_segments)?),
        ),
        GeometryRecipe::Torus { major_radius, minor_radius, major_segments, minor_segments } => {
            if minor_radius >= major_radius { return Err("Torus minor radius must be below its major radius".into()); }
            ("ComputationalGeometry.torus", format!("{{major_radius:{},minor_radius:{},major_segments:{},minor_segments:{}}}", dimension(*major_radius)?, dimension(*minor_radius)?, segments(*major_segments)?, segments(*minor_segments)?))
        }
        GeometryRecipe::CircularSweep { radius, length, segments: count, caps } => (
            "ComputationalGeometry.profile_sweep",
            format!("{{profile:{{kind:\"circle\",radius:{},segments:{}}},length:{},caps:{caps}}}", dimension(*radius)?, segments(*count)?, dimension(*length)?),
        ),
        GeometryRecipe::ProfileSweep { profile, length, caps } => (
            "ComputationalGeometry.profile_sweep",
            format!("{{profile:{},length:{},caps:{caps}}}", profile.expression()?, dimension(*length)?),
        ),
        GeometryRecipe::RevolveProfile { points, segments } => (
            "ComputationalGeometry.revolve_profile",
            super::cad_recipe::expression(points,*segments)?,
        ),
    };
    Ok((
        format!("requires [capability(\"capability.invoke\"), capability(\"{operation}\")];\n"),
        format!("capability.invoke(\"{operation}\",{args})"),
    ))
}
