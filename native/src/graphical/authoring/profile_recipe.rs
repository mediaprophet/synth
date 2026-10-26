//! Metric source recipes; canonical libraries own tessellation and polygon admission.
use super::{geometry_source::dimension, source::literal};
use poet_spatial_authoring::{Profile, ProfileLimits};
use qualia_core_db::specialized_libs::computational_geometry::{Point2, polygon_validation};

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ProfileRecipe {
    RoundedRectangle { width: f64, height: f64, radius: f64, segments: u32 },
    OpenChannel { width: f64, height: f64, wall: f64, corner_radius: f64, segments: u32 },
    /// Implicitly closed, counterclockwise simple polygon in local XY metres.
    Polyline { points: Vec<[f64; 2]> },
}
fn count(n: u32) -> Result<usize, String> {
    if !(1..=64).contains(&n) { return Err("Profile arc segments must be in [1,64]".into()); }
    Ok(n as usize)
}
fn radius(n: f64) -> Result<String, String> {
    if !n.is_finite() || !(0.0..=1.0e6).contains(&n) { return Err("Profile radius must be in [0,1000000] metres".into()); }
    literal(n)
}
impl ProfileRecipe {
    pub(super) fn expression(&self) -> Result<String, String> {
        let (native, expression) = match self {
            Self::RoundedRectangle { width, height, radius: corner, segments } => {
                let (w,h,r) = (dimension(*width)?,dimension(*height)?,radius(*corner)?);
                let n = count(*segments)?;
                (Profile::RoundedRectangle { width:*width,height:*height,radius:*corner,segments:n },
                    format!("{{kind:\"rounded_rectangle\",width:{w},height:{h},radius:{r},segments:{n}}}"))
            }
            Self::OpenChannel { width, height, wall, corner_radius, segments } => {
                let (w,h,t,r) = (dimension(*width)?,dimension(*height)?,dimension(*wall)?,radius(*corner_radius)?);
                let n = count(*segments)?;
                (Profile::OpenChannel { width:*width,height:*height,wall:*wall,corner_radius:*corner_radius,segments:n },
                    format!("{{kind:\"open_channel\",width:{w},height:{h},wall:{t},corner_radius:{r},segments:{n}}}"))
            }
            Self::Polyline { points } => {
                if !(3..=256).contains(&points.len()) || points.iter().flatten().any(|n| !n.is_finite() || n.abs() > 1.0e6) {
                    return Err("Profile needs 3..256 finite XY points within +/-1000000 metres".into());
                }
                let rows = points.iter().map(|p| Ok(format!("[{},{}]",literal(p[0])?,literal(p[1])?))).collect::<Result<Vec<_>,String>>()?;
                (Profile::Polyline { points:points.clone() },format!("{{kind:\"polyline\",points:[{}],segments:1}}",rows.join(",")))
            }
        };
        // Bounded cold preflight, with no host effect or second mesh generator.
        // Actual invocation repeats native admission against current host budgets.
        let points = native.points(ProfileLimits { points:4096,scratch_bytes:1024*1024 }).map_err(|e|format!("Invalid metric profile: {e:?}"))?;
        let section: Vec<_> = points.iter().map(|p| Point2::new(p[0],p[1])).collect();
        if !polygon_validation::validate_simple_polygon(&section).issues.is_empty() {
            return Err("Profile must be a simple counterclockwise polygon without crossing or degenerate edges".into());
        }
        Ok(expression)
    }
}
