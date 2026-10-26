//! Bounded adapter over Core's caller-buffered full-turn revolution kernel.
//! No caps or axis poles: positive-radius stations run from top to bottom.
use poet_spatial_authoring::{AuthoringError, GeometryLimits};
use qualia_core_db::{
    render::assets::Mesh,
    specialized_libs::computational_geometry::{self as cg, Point3},
};

pub(crate) fn validate_profile(points: &[[f64; 2]], segments: usize) -> Result<(), &'static str> {
    if !(2..=256).contains(&points.len()) || !(3..=256).contains(&segments) {
        return Err("Revolution requires 2..256 radius-height stations and 3..256 radial segments");
    }
    for (i, p) in points.iter().enumerate() {
        if !p[0].is_finite()
            || !(1.0e-6..=1.0e6).contains(&p[0])
            || !p[1].is_finite()
            || p[1].abs() > 1.0e6
        {
            return Err(
                "Revolution radii must be [0.000001,1000000] metres; heights finite within +/-1000000 metres",
            );
        }
        // Core's triangle order has outward winding for decreasing heights.
        // Distinct f32 heights prevent collapsed rings in the output resource.
        if i > 0 && (p[1] >= points[i - 1][1] || p[1] as f32 >= points[i - 1][1] as f32) {
            return Err(
                "Revolution heights must decrease strictly top-to-bottom and remain distinct at native output precision",
            );
        }
    }
    Ok(())
}
pub(super) fn mesh(
    points: &[[f64; 2]],
    segments: usize,
    limits: GeometryLimits,
) -> Result<Mesh, AuthoringError> {
    validate_profile(points, segments).map_err(AuthoringError::Invalid)?;
    if points.len() > limits.profile_points {
        return Err(AuthoringError::Budget(
            "Revolution exceeds host profile-point budget",
        ));
    }
    let nv = points
        .len()
        .checked_mul(segments)
        .ok_or(AuthoringError::Budget("Revolution vertex overflow"))?;
    let nt = (points.len() - 1)
        .checked_mul(segments)
        .and_then(|n| n.checked_mul(2))
        .ok_or(AuthoringError::Budget("Revolution triangle overflow"))?;
    limits.admit(
        nv,
        nt,
        points
            .len()
            .checked_mul(24)
            .ok_or(AuthoringError::Budget("Revolution scratch overflow"))?,
    )?;
    let profile: Vec<_> = points
        .iter()
        .map(|p| Point3::new(p[0], p[1], 0.0))
        .collect();
    let mut vertices = vec![Point3::default(); nv];
    let mut triangles = vec![[0; 3]; nt];
    let (v, t) = cg::revolve_profile(&profile, segments, &mut vertices, &mut triangles)
        .map_err(|e| AuthoringError::Kernel(format!("Core revolution: {e:?}")))?;
    if v != nv || t != nt || triangles.iter().flatten().any(|i| *i as usize >= nv) {
        return Err(AuthoringError::Invalid(
            "Core revolution returned inconsistent output counts or indices",
        ));
    }
    let positions: Vec<_> = vertices
        .into_iter()
        .map(|p| [p.x as f32, p.y as f32, p.z as f32])
        .collect();
    let (mut min, mut max) = ([f32::INFINITY; 3], [f32::NEG_INFINITY; 3]);
    for p in &positions {
        for a in 0..3 {
            if !p[a].is_finite() {
                return Err(AuthoringError::Invalid(
                    "Core revolution returned nonfinite output",
                ));
            }
            min[a] = min[a].min(p[a]);
            max[a] = max[a].max(p[a]);
        }
    }
    Ok(Mesh {
        positions,
        triangles,
        min,
        max,
    })
}
