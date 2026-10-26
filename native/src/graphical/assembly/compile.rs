//! Dense derived geometry only. Raw editable source and Q42 part graph stay separate.
use super::{validate_compiled, CompiledAssembly};
use crate::graphical::preview::PreviewLimits;
use qualia_core_db::{
    container_10d::{attach_material_section, attach_texture_coordinate_section},
    render::compile_10d::compile_mesh_to_10d_with_frames,
};

/// Reuses native .10d normal/UV/MAT sections; never creates a private scene codec.
/// Position quantization makes this a derivative, not a replacement for the master.
pub fn compile_assembly(assembly: &CompiledAssembly) -> Result<Vec<u8>, String> {
    validate_compiled(
        assembly,
        PreviewLimits {
            side: 512,
            vertices: u32::MAX as usize,
            indices: u32::MAX as usize,
        },
    )?;
    let geometry = compile_mesh_to_10d_with_frames(&assembly.mesh, Some(&assembly.normals), None)
        .map_err(|e| format!("native assembly geometry compile: {e}"))?;
    let geometry = attach_texture_coordinate_section(&geometry, &assembly.uv0)
        .map_err(|e| format!("native assembly UV compile: {e:?}"))?;
    attach_material_section(
        &geometry,
        &assembly.materials,
        &assembly.ranges,
        u32::try_from(
            assembly
                .mesh
                .triangles
                .len()
                .checked_mul(3)
                .ok_or("assembly index count overflow")?,
        )
        .map_err(|_| "assembly index count exceeds native format")?,
    )
    .map_err(|e| format!("native assembly material compile: {e:?}"))
}
