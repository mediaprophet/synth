//! Native multi-material assembly preview, through existing PortalGpu only.
use super::{validate_compiled, CompiledAssembly};
use crate::graphical::preview::{self, PreviewCamera, PreviewFrame, PreviewLimits, SourceRef};
use qualia_core_db::render::{
    gpu::PortalGpu, lighting::environment_lighting::EnvironmentProbeSet, telemetry::SystemTelemetry,
};

#[derive(Clone, Copy)]
pub struct AssemblyRenderSettings {
    pub camera: PreviewCamera,
    pub clear_color: [f64; 4],
    pub sun_direction: [f32; 3],
    pub sun_intensity: f32,
    pub ambient_intensity: f32,
    pub shadows: bool,
    pub screen_space_ao: bool,
    pub exposure_ev: f32,
}
impl AssemblyRenderSettings {
    pub fn for_camera(camera: PreviewCamera) -> Self {
        Self {
            camera,
            clear_color: [0.025, 0.035, 0.05, 1.0],
            sun_direction: [0.45, 0.8, 0.35],
            sun_intensity: 1.4,
            ambient_intensity: 0.15,
            shadows: true,
            screen_space_ao: true,
            exposure_ev: qualia_core_db::render::output::DEFAULT_HDR_EXPOSURE_EV,
        }
    }
}
pub fn render_assembly(
    source: SourceRef,
    assembly: &CompiledAssembly,
    camera: PreviewCamera,
    limits: PreviewLimits,
) -> Result<PreviewFrame, String> {
    render_assembly_with_settings(
        source,
        assembly,
        AssemblyRenderSettings::for_camera(camera),
        limits,
        None,
    )
}
pub fn render_assembly_with_settings(
    source: SourceRef,
    assembly: &CompiledAssembly,
    settings: AssemblyRenderSettings,
    limits: PreviewLimits,
    probes: Option<&EnvironmentProbeSet>,
) -> Result<PreviewFrame, String> {
    render_assembly_with_resources(source, assembly, settings, limits, probes, None)
}

/// Textures resolve only from the host's immutable authorised package snapshot.
pub fn render_assembly_with_resources(
    source: SourceRef,
    assembly: &CompiledAssembly,
    settings: AssemblyRenderSettings,
    limits: PreviewLimits,
    probes: Option<&EnvironmentProbeSet>,
    resources: Option<&crate::graphical::surfaces::TextureResources>,
) -> Result<PreviewFrame, String> {
    source.check_current()?;
    preview::validate_frame(limits.side, settings.camera)?;
    validate_compiled(assembly, limits)?;
    crate::graphical::surfaces::resources::validate_material_resources(
        &assembly.materials,
        resources,
    )?;
    validate_render_settings(&settings)?;
    let s = settings;
    let indices: Vec<u32> = assembly.mesh.triangles.iter().flatten().copied().collect();
    let colors = vec![[1.0; 4]; assembly.mesh.positions.len()];
    let mut renderer = PortalGpu::new_offscreen(limits.side, limits.side, 1)?;
    if let Some(resources) = resources {
        resources.upload(&assembly.materials, &mut renderer)?;
    }
    renderer.set_ambient_enabled(false);
    renderer.set_tensor_projection_enabled(false);
    renderer.set_clear_color(
        s.clear_color[0],
        s.clear_color[1],
        s.clear_color[2],
        s.clear_color[3],
    );
    renderer.set_camera_target(
        s.camera.yaw,
        s.camera.pitch,
        s.camera.distance,
        s.camera.target[0],
        s.camera.target[1],
        s.camera.target[2],
    );
    renderer.set_lighting(
        s.sun_direction[0],
        s.sun_direction[1],
        s.sun_direction[2],
        s.sun_intensity,
        s.ambient_intensity,
    );
    renderer.set_shadows_enabled(s.shadows);
    renderer.set_screen_space_ao_enabled(s.screen_space_ao);
    renderer.upload_mesh_colored_with_frames_and_materials_and_uv0(
        &assembly.mesh.positions,
        &colors,
        Some(&assembly.normals),
        None,
        Some(&assembly.uv0),
        &indices,
        &assembly.materials,
        &assembly.ranges,
    )?;
    renderer.set_environment_probes(probes, [0.0; 3]);
    if !renderer.set_exposure_compensation(s.exposure_ev) {
        return Err("native output exposure capability unavailable; image withheld".into());
    }
    renderer.render(0.0, &SystemTelemetry::default())?;
    let mut rgba8 = vec![0; renderer.required_rgba8_bytes()];
    renderer.read_rgba8_into(&mut rgba8)?;
    source.check_current()?;
    Ok(PreviewFrame {
        source,
        session_revision: None,
        execution_epoch: None,
        side: limits.side,
        rgba8,
    })
}

/// Shared admission for one-shot and retained native Portal rendering.
pub(crate) fn validate_render_settings(settings: &AssemblyRenderSettings) -> Result<(), String> {
    let s = settings;
    if s.clear_color
        .iter()
        .any(|v| !v.is_finite() || !(0.0..=1.0).contains(v))
        || s.sun_direction.iter().any(|v| !v.is_finite())
        || {
            let length2 = s.sun_direction.iter().map(|v| v * v).sum::<f32>();
            !length2.is_finite() || length2 <= f32::EPSILON
        }
        || [s.sun_intensity, s.ambient_intensity]
            .iter()
            .any(|v| !v.is_finite() || *v < 0.0 || *v > 100_000.0)
        || qualia_core_db::render::output::exposure_scale_from_ev(s.exposure_ev).is_none()
    {
        return Err("invalid native assembly render settings".into());
    }
    Ok(())
}
