//! Caller-budgeted adapters over existing Qualia geometry kernels, never a renderer.
use crate::{AuthoringError, Profile, ProfileLimits, finite_positive};
use qualia_core_db::{
    render::assets::Mesh,
    specialized_libs::computational_geometry::{
        self as cg, Point2, Point3, authoring, math_geometry::AffineFrame3,
    },
};
#[derive(Clone, Copy, Debug)]
pub struct GeometryLimits {
    pub vertices: usize,
    pub triangles: usize,
    pub scratch_bytes: usize,
    pub profile_points: usize,
}
impl Default for GeometryLimits {
    fn default() -> Self {
        Self {
            vertices: 2_000_000,
            triangles: 4_000_000,
            scratch_bytes: 256 * 1024 * 1024,
            profile_points: 4096,
        }
    }
}
impl GeometryLimits {
    pub fn admit(
        &self,
        vertices: usize,
        triangles: usize,
        extra: usize,
    ) -> Result<(), AuthoringError> {
        let bytes = vertices
            .checked_mul(64)
            .and_then(|b| triangles.checked_mul(64).and_then(|t| b.checked_add(t)))
            .and_then(|b| b.checked_add(extra));
        if vertices == 0
            || triangles == 0
            || vertices > self.vertices
            || triangles > self.triangles
            || vertices > u32::MAX as usize
            || bytes.is_none_or(|b| b > self.scratch_bytes)
        {
            Err(AuthoringError::Budget(
                "geometry exceeds caller vertex/triangle/scratch budget",
            ))
        } else {
            Ok(())
        }
    }
}
fn mesh(positions: Vec<[f32; 3]>, triangles: Vec<[u32; 3]>) -> Result<Mesh, AuthoringError> {
    if positions.is_empty()
        || triangles.is_empty()
        || positions.iter().flatten().any(|v| !v.is_finite())
        || triangles
            .iter()
            .flatten()
            .any(|&i| i as usize >= positions.len())
    {
        return Err(AuthoringError::Invalid(
            "kernel mesh is nonfinite, empty or out of range",
        ));
    }
    let (mut min, mut max) = ([f32::INFINITY; 3], [f32::NEG_INFINITY; 3]);
    for p in &positions {
        for a in 0..3 {
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
/// A metric profile swept along a finite 3-D axis. Concave end caps reuse the
/// Core polygon triangulator; centroid fans are deliberately not used for U sections.
pub fn profile_sweep(
    profile: &Profile,
    axis: &[[f64; 3]],
    caps: bool,
    limits: GeometryLimits,
) -> Result<Mesh, AuthoringError> {
    if axis.len() < 2
        || axis.len() > 4096
        || axis
            .iter()
            .flatten()
            .any(|n| !n.is_finite() || !(*n as f32).is_finite())
    {
        return Err(AuthoringError::Invalid(
            "sweep axis needs 2..4096 finite stations",
        ));
    }
    let points = profile.points(ProfileLimits {
        points: limits.profile_points,
        scratch_bytes: limits.scratch_bytes,
    })?;
    let n = points.len();
    let nv = axis
        .len()
        .checked_mul(n)
        .ok_or(AuthoringError::Budget("sweep vertex overflow"))?;
    let side_triangles = (axis.len() - 1)
        .checked_mul(n)
        .and_then(|n| n.checked_mul(2))
        .ok_or(AuthoringError::Budget("sweep triangle overflow"))?;
    let nt = side_triangles
        .checked_add(if caps { (n - 2) * 2 } else { 0 })
        .ok_or(AuthoringError::Budget("sweep cap overflow"))?;
    limits.admit(
        nv,
        nt,
        n.checked_mul(256)
            .and_then(|b| axis.len().checked_mul(128).and_then(|a| b.checked_add(a)))
            .ok_or(AuthoringError::Budget("sweep scratch overflow"))?,
    )?;
    let section: Vec<_> = points.iter().map(|p| Point2::new(p[0], p[1])).collect();
    if !cg::polygon_validation::validate_simple_polygon(&section)
        .issues
        .is_empty()
    {
        return Err(AuthoringError::Invalid(
            "profile is not a valid simple polygon",
        ));
    }
    let axis: Vec<_> = axis.iter().map(|p| Point3::new(p[0], p[1], p[2])).collect();
    let zero = Point3::default();
    let mut frames = vec![
        AffineFrame3 {
            origin: zero,
            e0: zero,
            e1: zero,
            e2: zero
        };
        axis.len()
    ];
    let mut positions = vec![zero; nv];
    let mut triangles = vec![[0; 3]; nt];
    let (v, t) = cg::extrude_profile_along_path(
        &axis,
        &section,
        None,
        true,
        false,
        &mut frames,
        &mut positions,
        &mut triangles,
    )
    .map_err(|e| AuthoringError::Kernel(format!("Core profile sweep: {e:?}")))?;
    if v != nv || t != side_triangles {
        return Err(AuthoringError::Invalid(
            "Core sweep counts differ from preflight",
        ));
    }
    if caps {
        let cap = cg::triangulation_2::triangulate_polygon(&section);
        if cap.len() != n - 2 {
            return Err(AuthoringError::Invalid(
                "profile end-cap triangulation incomplete",
            ));
        }
        let polygon_area = poet_geometry::shoelace(
            &points
                .iter()
                .map(|p| poet_geometry::Vec2::new(p[0], p[1]))
                .collect::<Vec<_>>(),
        );
        let area: f64 = cap.iter().map(|t| t.signed_area()).sum();
        if (area - polygon_area).abs() > polygon_area.abs().max(1.0) * 1e-8 {
            return Err(AuthoringError::Invalid(
                "profile cap area differs from section",
            ));
        }
        let mut cursor = t;
        for triangle in cap {
            let mut indices = [0u32; 3];
            for (i, p) in [triangle.a, triangle.b, triangle.c].iter().enumerate() {
                indices[i] = section
                    .iter()
                    .position(|s| s.x == p.x && s.y == p.y)
                    .ok_or(AuthoringError::Invalid("cap changed section vertex"))?
                    as u32;
            }
            triangles[cursor] = [indices[0], indices[2], indices[1]];
            cursor += 1;
            let top = ((axis.len() - 1) * n) as u32;
            triangles[cursor] = [top + indices[0], top + indices[1], top + indices[2]];
            cursor += 1;
        }
    }
    mesh(
        positions
            .into_iter()
            .map(|p| [p.x as f32, p.y as f32, p.z as f32])
            .collect(),
        triangles,
    )
}
pub fn sphere(
    radius: f64,
    lat_segments: usize,
    lon_segments: usize,
    limits: GeometryLimits,
) -> Result<Mesh, AuthoringError> {
    if !finite_positive(radius)
        || lat_segments < 2
        || lon_segments < 3
        || lat_segments > 4096
        || lon_segments > 4096
    {
        return Err(AuthoringError::Invalid("sphere radius/segments invalid"));
    }
    let nv = (lat_segments + 1)
        .checked_mul(lon_segments + 1)
        .ok_or(AuthoringError::Budget("sphere vertex overflow"))?;
    limits.admit(
        nv,
        lat_segments
            .checked_mul(lon_segments)
            .and_then(|n| n.checked_mul(2))
            .ok_or(AuthoringError::Budget("sphere triangle overflow"))?,
        0,
    )?;
    authoring::uv_sphere(radius as f32, lat_segments as u32, lon_segments as u32)
        .map_err(|e| AuthoringError::Kernel(format!("Core sphere: {e:?}")))
}
pub fn torus(
    major_radius: f64,
    minor_radius: f64,
    major_segments: usize,
    minor_segments: usize,
    limits: GeometryLimits,
) -> Result<Mesh, AuthoringError> {
    if !finite_positive(major_radius)
        || !finite_positive(minor_radius)
        || minor_radius >= major_radius
        || major_segments < 3
        || minor_segments < 3
        || major_segments > 4096
        || minor_segments > 4096
    {
        return Err(AuthoringError::Invalid("torus radii/segments invalid"));
    }
    let count = (major_segments + 1)
        .checked_mul(minor_segments + 1)
        .ok_or(AuthoringError::Budget("torus vertex overflow"))?;
    limits.admit(
        count,
        major_segments
            .checked_mul(minor_segments)
            .and_then(|n| n.checked_mul(2))
            .ok_or(AuthoringError::Budget("torus triangle overflow"))?,
        0,
    )?;
    authoring::torus(
        major_radius as f32,
        minor_radius as f32,
        major_segments as u32,
        minor_segments as u32,
    )
    .map_err(|e| AuthoringError::Kernel(format!("Core torus: {e:?}")))
}
