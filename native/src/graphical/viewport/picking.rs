//! Exact mesh-part mapping over Core's public ray/triangle kernel.
//! Qualified for opaque assembly geometry; textured alpha/Blend selection fails closed.
use super::camera::{camera_state, extent, screen_axes};
use crate::graphical::{
    assembly::CompiledAssembly,
    preview::{PreviewCamera, SourceRef},
};
use qualia_core_db::{
    container_10d::OpacityMode,
    render::{
        camera::{orbit_eye_position_target, orbit_forward, CAMERA_TAN_HALF_FOV},
        temporal_producers::Mat4ColumnMajor,
    },
    specialized_libs::computational_geometry::{
        ray_triangle_intersect_3d, Point3, RayTriangleResult,
    },
};
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PartSelection {
    pub source: SourceRef,
    pub frame_id: u64,
    /// Local source assembly key, explicitly distinct from a graph node.
    pub part_key: String,
    pub semantic_id: u64,
    pub triangle_index: usize,
}

pub(crate) fn check(source: &SourceRef, cancelled: &AtomicBool) -> Result<(), String> {
    if cancelled.load(Ordering::Acquire) {
        return Err("native viewport cancelled".into());
    }
    source.check_current()
}

pub(crate) fn project(matrix: &Mat4ColumnMajor, point: [f32; 3]) -> Option<[f32; 3]> {
    let clip = matrix.transform_point3(point);
    if clip.iter().any(|v| !v.is_finite()) || clip[3] <= 0.0 || clip[2] < 0.0 || clip[2] > clip[3] {
        return None;
    }
    Some([clip[0] / clip[3], clip[1] / clip[3], clip[2] / clip[3]])
}

pub(crate) fn pick_part(
    source: &SourceRef,
    frame_id: u64,
    assembly: &CompiledAssembly,
    camera: PreviewCamera,
    width: u32,
    height: u32,
    x: f32,
    y: f32,
    max_triangles: usize,
    cancelled: &AtomicBool,
) -> Result<Option<PartSelection>, String> {
    check(source, cancelled)?;
    let aspect = extent(width, height)?;
    if !x.is_finite()
        || !y.is_finite()
        || x < 0.0
        || y < 0.0
        || x >= width as f32
        || y >= height as f32
    {
        return Err("pick outside native viewport".into());
    }
    if assembly.mesh.triangles.len() > max_triangles {
        return Err("viewport pick triangle budget exceeded".into());
    }
    if assembly
        .materials
        .iter()
        .any(|m| m.opacity_mode != OpacityMode::Opaque || m.wind_animated)
    {
        return Err(
            "mesh-part picking for alpha-masked, blended or wind-deformed materials is unqualified"
                .into(),
        );
    }
    let state = camera_state(camera)?;
    let matrix = Mat4ColumnMajor::from_cols(state.view_projection(aspect));
    let origin = orbit_eye_position_target(state.yaw, state.pitch, state.zoom, state.target);
    let forward = orbit_forward(state.yaw, state.pitch);
    let (right, up) = screen_axes(state, aspect);
    // Same top-left texel-center convention as Core's native GPU picking pass.
    let nx = ((x.floor() + 0.5) / width as f32 * 2.0 - 1.0) * aspect * CAMERA_TAN_HALF_FOV;
    let ny = (1.0 - (y.floor() + 0.5) / height as f32 * 2.0) * CAMERA_TAN_HALF_FOV;
    let direction = std::array::from_fn(|i| forward[i] + right[i] * nx + up[i] * ny);
    let p = |v: [f32; 3]| Point3::new(v[0] as f64, v[1] as f64, v[2] as f64);
    let mut best: Option<(usize, usize, f32)> = None;
    for (part_index, (part, range)) in assembly.parts.iter().zip(&assembly.ranges).enumerate() {
        let _material = assembly
            .materials
            .iter()
            .find(|m| m.id == range.material_id)
            .ok_or("selected part lacks admitted material")?;
        let start = part.first_index as usize / 3;
        let end = start + part.index_count as usize / 3;
        for index in start..end {
            if index % 256 == 0 {
                check(source, cancelled)?;
            }
            let tri = assembly.mesh.triangles[index];
            let [a, b, c] = tri.map(|i| assembly.mesh.positions[i as usize]);
            // Core mesh forward/pick pipelines use cull_mode None; double_sided changes
            // normal treatment, not triangle coverage. Preserve those exact semantics.
            let RayTriangleResult::Hit(hit) =
                ray_triangle_intersect_3d(p(origin), p(direction), p(a), p(b), p(c))
            else {
                continue;
            };
            if hit.t < 0.0 {
                continue;
            }
            let world = std::array::from_fn(|i| origin[i] + direction[i] * hit.t as f32);
            let Some(clip) = project(&matrix, world) else {
                continue;
            };
            if best.is_none_or(|(_, _, depth)| clip[2] < depth) {
                best = Some((part_index, index, clip[2]));
            }
        }
    }
    check(source, cancelled)?;
    Ok(best.map(|(part, triangle_index, _)| PartSelection {
        source: source.clone(),
        frame_id,
        part_key: assembly.parts[part].key.clone(),
        semantic_id: assembly.parts[part].semantic_id,
        triangle_index,
    }))
}
