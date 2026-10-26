//! Explicit x-ray selection wireframe, derived from the selected source part range.
//! This overlay does not change material factors/textures or claim occlusion-correct outlines.
use super::{
    camera::camera_state,
    picking::{check, project},
    PartSelection,
};
use crate::graphical::{
    assembly::CompiledAssembly,
    preview::{PreviewCamera, SourceRef},
};
use qualia_core_db::render::temporal_producers::Mat4ColumnMajor;
use std::sync::atomic::AtomicBool;

pub(crate) fn overlay(
    source: &SourceRef,
    scene: &CompiledAssembly,
    selected: &PartSelection,
    camera: PreviewCamera,
    side: u32,
    rgba: &mut [u8],
    max_steps: usize,
    cancelled: &AtomicBool,
) -> Result<(), String> {
    check(source, cancelled)?;
    if &selected.source != source {
        return Err("selection source has retired".into());
    }
    let part = scene
        .parts
        .iter()
        .find(|p| p.key == selected.part_key && p.semantic_id == selected.semantic_id)
        .ok_or("selected source part is unavailable")?;
    let matrix = Mat4ColumnMajor::from_cols(camera_state(camera)?.view_projection(1.0));
    let mut steps = 0usize;
    for index in part.first_index as usize / 3..(part.first_index + part.index_count) as usize / 3 {
        if index % 256 == 0 {
            check(source, cancelled)?;
        }
        let points =
            scene.mesh.triangles[index].map(|i| project(&matrix, scene.mesh.positions[i as usize]));
        let [Some(a), Some(b), Some(c)] = points else {
            continue;
        };
        // Bounded overlay scope: edges of fully on-screen triangles only.
        if [a, b, c]
            .iter()
            .any(|p| p[0].abs() > 1.0 || p[1].abs() > 1.0)
        {
            continue;
        }
        let pixel = |p: [f32; 3]| {
            [
                (p[0] * 0.5 + 0.5) * side as f32,
                (0.5 - p[1] * 0.5) * side as f32,
            ]
        };
        for [a, b] in [
            [pixel(a), pixel(b)],
            [pixel(b), pixel(c)],
            [pixel(c), pixel(a)],
        ] {
            let count = (a[0] - b[0]).abs().max((a[1] - b[1]).abs()).ceil() as usize + 1;
            steps = steps
                .checked_add(count)
                .filter(|n| *n <= max_steps)
                .ok_or("selection overlay step budget exceeded")?;
            for step in 0..count {
                let t = step as f32 / count.saturating_sub(1).max(1) as f32;
                let x = (a[0] + (b[0] - a[0]) * t)
                    .floor()
                    .clamp(0.0, side as f32 - 1.0) as usize;
                let y = (a[1] + (b[1] - a[1]) * t)
                    .floor()
                    .clamp(0.0, side as f32 - 1.0) as usize;
                let offset = (y * side as usize + x) * 4;
                rgba[offset..offset + 4].copy_from_slice(&[255, 194, 45, 255]);
            }
        }
    }
    check(source, cancelled)
}
