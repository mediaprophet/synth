//! Versioned explicit-seam source recipe; base geometry and authored UVs remain
//! in the original part record. Canonical Core owns chart topology and packing.
use super::uv::{exact, field, record};
use qualia_core_db::{
    render::assets::Mesh,
    specialized_libs::computational_geometry::{
        uv_atlas::{self, AtlasChart, AtlasLimits, AtlasOptions, AtlasOutput},
        Point3,
    },
};
use vibe::Value;
#[cfg(test)]
#[path = "uv_atlas_tests.rs"]
mod tests;
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AtlasDraft {
    pub version: u32,
    pub seams: Vec<[u32; 2]>,
    pub atlas_size: u32,
    pub gutter_pixels: u32,
    /// Chart-area density in part-local source metres, before assembly TRS.
    pub texels_per_metre: f64,
    pub iterations: usize,
}
impl Default for AtlasDraft {
    fn default() -> Self {
        Self {
            version: 1,
            seams: Vec::new(),
            atlas_size: 1024,
            gutter_pixels: 4,
            texels_per_metre: 64.0,
            iterations: 64,
        }
    }
}
impl AtlasDraft {
    pub fn options(&self) -> Result<AtlasOptions, String> {
        if !matches!(self.version, 1 | 2) {
            return Err("Unsupported UV atlas recipe version".into());
        }
        if self.seams.len() > 65_536
            || self.seams.iter().any(|s| s[0] >= s[1])
            || self.seams.windows(2).any(|s| s[0] >= s[1])
        {
            return Err(
                "UV seams must be unique sorted [smaller,larger] source-index pairs".into(),
            );
        }
        let o = AtlasOptions {
            atlas_size: self.atlas_size,
            gutter_pixels: self.gutter_pixels,
            texels_per_metre: self.texels_per_metre,
            iterations: self.iterations,
        };
        uv_atlas::preflight_counts(3, 1, 0, o, AtlasLimits::default())
            .map_err(|e| format!("UV atlas settings: {e:?}"))?;
        Ok(o)
    }
}
fn uint(v: &Value) -> Result<u32, String> {
    match v {
        Value::I64(v) if *v >= 0 => u32::try_from(*v).map_err(|_| "UV integer exceeds u32".into()),
        Value::U64(v) => u32::try_from(*v).map_err(|_| "UV integer exceeds u32".into()),
        _ => Err("UV atlas integer required".into()),
    }
}
pub fn from_value(v: &Value) -> Result<AtlasDraft, String> {
    let r = record(v)?;
    exact(
        r,
        &[
            "version",
            "seams",
            "atlas_size",
            "gutter_pixels",
            "texels_per_metre",
            "iterations",
        ],
    )?;
    let Value::List(seams) = field(r, "seams")? else {
        return Err("UV seams must be a list".into());
    };
    if seams.len() > 65_536 {
        return Err("UV seam count exceeds recipe bound".into());
    }
    let seams = seams
        .iter()
        .map(|s| {
            let Value::List(s) = s else {
                return Err("UV seam pair required".into());
            };
            if s.len() != 2 {
                return Err("UV seam requires two vertex indices".into());
            }
            Ok([uint(&s[0])?, uint(&s[1])?])
        })
        .collect::<Result<Vec<_>, String>>()?;
    let density = match field(r, "texels_per_metre")? {
        Value::F64(n) => *n,
        Value::I64(n) => *n as f64,
        Value::U64(n) => *n as f64,
        _ => return Err("UV density number required".into()),
    };
    let draft = AtlasDraft {
        version: uint(field(r, "version")?)?,
        seams,
        atlas_size: uint(field(r, "atlas_size")?)?,
        gutter_pixels: uint(field(r, "gutter_pixels")?)?,
        texels_per_metre: density,
        iterations: uint(field(r, "iterations")?)? as usize,
    };
    draft.options()?;
    Ok(draft)
}
pub fn source(draft: &AtlasDraft) -> Result<String, String> {
    draft.options()?;
    Ok(format!("{{version:{},seams:{:?},atlas_size:{},gutter_pixels:{},texels_per_metre:{:?},iterations:{}}}",
        draft.version,draft.seams,draft.atlas_size,draft.gutter_pixels,draft.texels_per_metre,draft.iterations))
}
fn kernel_limits(
    vertices: usize,
    triangles: usize,
    limits: AtlasLimits,
    smooth_frames: bool,
) -> Result<AtlasLimits, String> {
    // Includes caller outputs, converted Point3 input and remapped Mesh/normals.
    // Core independently charges its own complete construction workspace.
    let wrapper = vertices
        .checked_mul(24)
        .and_then(|n| triangles.checked_mul(512).and_then(|t| n.checked_add(t)))
        .ok_or("UV wrapper budget overflow")?;
    let frame_bytes = if smooth_frames {
        vertices
            .checked_mul(36)
            .and_then(|v| triangles.checked_mul(204)?.checked_add(v))
            .ok_or("UV frame budget overflow")?
    } else {
        0
    };
    let frame_work = if smooth_frames {
        u64::try_from(vertices)
            .ok()
            .and_then(|v| v.checked_mul(8))
            .and_then(|v| {
                u64::try_from(triangles)
                    .ok()
                    .and_then(|t| t.checked_mul(432)?.checked_add(v))
            })
            .ok_or("UV frame work overflow")?
    } else {
        0
    };
    Ok(AtlasLimits {
        work_units: limits
            .work_units
            .checked_sub(frame_work)
            .ok_or("UV frames exceed work budget")?,
        scratch_bytes: limits
            .scratch_bytes
            .checked_sub(
                wrapper
                    .checked_add(frame_bytes)
                    .ok_or("UV frame budget overflow")?,
            )
            .ok_or("UV wrapper exceeds scratch budget")?,
        ..limits
    })
}

fn frame_vec<T: Clone>(count: usize, value: T) -> Result<Vec<T>, String> {
    let mut result = Vec::new();
    result
        .try_reserve_exact(count)
        .map_err(|_| "UV frame allocation refused")?;
    result.resize(count, value);
    Ok(result)
}

/// Smooth on original source topology before UV cuts. Existing authored
/// normals take precedence, including normals on intentional hard-edge splits.
fn smooth_source_normals(points: &[Point3], triangles: &[[u32; 3]]) -> Result<Vec<[f32; 3]>, String> {
    use qualia_core_db::specialized_libs::computational_geometry::ddg_operators::{
        face_vector_areas, vertex_normals,
    };
    let zero = Point3::new(0.0, 0.0, 0.0);
    let mut faces = frame_vec(triangles.len(), zero)?;
    face_vector_areas(points, triangles, &mut faces)
        .map_err(|e| format!("Smooth atlas face refused: {e:?}"))?;
    if faces.iter().any(|n| {
        !n.x.is_finite()
            || !n.y.is_finite()
            || !n.z.is_finite()
            || !((n.x * n.x + n.y * n.y + n.z * n.z) > 0.0)
    }) {
        return Err("Smooth atlas has nonfinite or degenerate source face".into());
    }
    drop(faces);
    let mut normals = frame_vec(points.len(), zero)?;
    vertex_normals(points, triangles, &mut normals)
        .map_err(|e| format!("Smooth atlas normals refused: {e:?}"))?;
    let mut output = frame_vec(points.len(), [0.0f32; 3])?;
    for (dst, n) in output.iter_mut().zip(normals) {
        *dst = [n.x as f32, n.y as f32, n.z as f32];
        let norm = dst.iter().map(|n| f64::from(*n).powi(2)).sum::<f64>();
        if dst.iter().any(|v| !v.is_finite()) || !norm.is_finite() || norm <= f64::EPSILON {
            return Err("Smooth atlas source vertex has no normal".into());
        }
    }
    Ok(output)
}
/// Allocation-free count preflight for aggregate admission. The returned value
/// is a conservative corner-split vertex ceiling, never a successful unwrap.
pub fn preflight(
    draft: &AtlasDraft,
    vertices: usize,
    triangles: usize,
    limits: AtlasLimits,
) -> Result<usize, String> {
    uv_atlas::preflight_counts(
        vertices,
        triangles,
        draft.seams.len(),
        draft.options()?,
        kernel_limits(vertices, triangles, limits, draft.version == 2)?,
    )
    .map_err(|e| format!("UV atlas preflight: {e:?}"))
}
#[derive(Clone, Debug)]
pub struct AtlasApplied {
    pub mesh: Mesh,
    pub normals: Option<Vec<[f32; 3]>>,
    pub uv0: Vec<[f32; 2]>,
    /// v2-only strictly admitted derived frames, never authored master data.
    pub tangents: Option<Vec<[f32; 4]>>,
    /// Each derived vertex identifies its retained raw source vertex; triangles
    /// keep source order. Never substitute this derivative for the raw master.
    pub source_vertices: Vec<u32>,
    pub triangle_charts: Vec<u32>,
    pub charts: Vec<AtlasChart>,
}
/// Only the admitted positions/triangles/normals/UV0 geometry schema is supported.
/// The caller MUST reject tangent, skin, morph, extra UV or extension streams
/// before projecting a richer record onto this narrow Mesh API.
pub fn apply(
    draft: &AtlasDraft,
    mesh: &Mesh,
    normals: Option<&[[f32; 3]]>,
    limits: AtlasLimits,
    cancelled: &dyn Fn() -> bool,
) -> Result<AtlasApplied, String> {
    if cancelled() {
        return Err("Native UV atlas cancelled before allocation".into());
    }
    let maximum = preflight(draft, mesh.positions.len(), mesh.triangles.len(), limits)?;
    if normals.is_some_and(|n| {
        n.len() != mesh.positions.len()
            || n.iter().any(|v| {
                v.iter().any(|v| !v.is_finite())
                    || v.iter().map(|v| (*v as f64).powi(2)).sum::<f64>() <= f64::EPSILON
            })
    }) {
        return Err(
            "UV atlas normal stream must be finite nonzero and source-vertex aligned".into(),
        );
    }
    let points: Vec<_> = mesh
        .positions
        .iter()
        .map(|p| Point3::new(p[0] as f64, p[1] as f64, p[2] as f64))
        .collect();
    let generated_normals = if draft.version == 2 && normals.is_none() {
        Some(smooth_source_normals(&points, &mesh.triangles)?)
    } else {
        None
    };
    let source_normals = normals.or(generated_normals.as_deref());
    let mut sources = vec![0u32; maximum];
    let mut triangles = vec![[0u32; 3]; mesh.triangles.len()];
    let mut ids = vec![0u32; mesh.triangles.len()];
    let mut uv0 = vec![[0.0f32; 2]; maximum];
    let mut charts = vec![AtlasChart::default(); mesh.triangles.len().min(limits.charts)];
    let counts = uv_atlas::unwrap_atlas_into(
        &points,
        &mesh.triangles,
        &draft.seams,
        draft.options()?,
        kernel_limits(
            mesh.positions.len(),
            mesh.triangles.len(),
            limits,
            draft.version == 2,
        )?,
        cancelled,
        AtlasOutput {
            source_vertices: &mut sources,
            triangles: &mut triangles,
            triangle_charts: &mut ids,
            uv0: &mut uv0,
            charts: &mut charts,
        },
    )
    .map_err(|e| format!("Native seam/disk UV atlas: {e:?}"))?;
    sources.truncate(counts.vertices);
    uv0.truncate(counts.vertices);
    charts.truncate(counts.charts);
    let positions: Vec<_> = sources
        .iter()
        .map(|&v| mesh.positions[v as usize])
        .collect();
    let normals: Option<Vec<[f32; 3]>> =
        source_normals.map(|n| sources.iter().map(|&v| n[v as usize]).collect());
    let tangents = if draft.version == 2 {
        let aligned = normals
            .as_deref()
            .ok_or("Smooth atlas lost normal correspondence")?;
        let mut output = frame_vec(sources.len(), [0.0f32; 4])?;
        let mut temporary = frame_vec(sources.len(), [0.0f32; 4])?;
        let mut scratch = frame_vec(sources.len(), [0.0f32; 6])?;
        let index_count = triangles
            .len()
            .checked_mul(3)
            .ok_or("Atlas tangent index overflow")?;
        let mut indices = Vec::new();
        indices
            .try_reserve_exact(index_count)
            .map_err(|_| "Atlas tangent index allocation refused")?;
        for tri in &triangles {
            indices.extend_from_slice(tri);
        }
        let work = u64::try_from(sources.len())
            .ok()
            .and_then(|v| v.checked_mul(100))
            .and_then(|v| {
                u64::try_from(triangles.len())
                    .ok()
                    .and_then(|t| t.checked_mul(100)?.checked_add(v))
            })
            .ok_or("Atlas tangent work overflow")?;
        qualia_core_db::render::mesh_tangent_frames::tangents_from_uvs_strict_into(
            &positions,
            &uv0,
            aligned,
            &indices,
            work,
            &mut output,
            &mut temporary,
            &mut scratch,
        )
        .map_err(|e| format!("Atlas UV tangent correspondence refused: {e:?}"))?;
        Some(output)
    } else {
        None
    };
    let mut min = [f32::INFINITY; 3];
    let mut max = [f32::NEG_INFINITY; 3];
    for p in &positions {
        for i in 0..3 {
            min[i] = min[i].min(p[i]);
            max[i] = max[i].max(p[i]);
        }
    }
    if cancelled() {
        return Err("Native UV atlas cancelled before derived mesh delivery".into());
    }
    Ok(AtlasApplied {
        mesh: Mesh {
            positions,
            triangles,
            min,
            max,
        },
        normals,
        uv0,
        tangents,
        source_vertices: sources,
        triangle_charts: ids,
        charts,
    })
}
