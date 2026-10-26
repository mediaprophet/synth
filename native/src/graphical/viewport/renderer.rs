//! One PortalGpu per native preview owner. Camera-only frames never upload scene geometry.
use super::{
    camera::{camera_state, preview_camera},
    highlight,
    picking::{check, pick_part},
    PartSelection,
};
use crate::graphical::{
    assembly::{validate_compiled, AssemblyRenderSettings, CompiledAssembly},
    preview::{PreviewCamera, PreviewFrame, PreviewLimits, SourceRef},
    surfaces::TextureResources,
};
use qualia_core_db::render::{
    gpu::PortalGpu, lighting::environment_lighting::EnvironmentProbeSet, telemetry::SystemTelemetry,
};
use std::sync::atomic::AtomicBool;

#[derive(Clone, Copy)]
pub struct ViewportBudget {
    /// CPU-owned scene + flatten/upload scratch + one RGBA output; GPU uses Core's own admission.
    pub working_bytes: usize,
    pub pick_triangles: usize,
    pub overlay_steps: usize,
}
impl Default for ViewportBudget {
    fn default() -> Self {
        Self {
            working_bytes: 128 * 1024 * 1024,
            pick_triangles: 400_000,
            overlay_steps: 1_000_000,
        }
    }
}

/// Actual operation counts, not timing or GPU allocation measurements.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ViewportCounters {
    pub device_creations: u64,
    pub scene_uploads: u64,
    pub frames: u64,
}

pub struct RetainedViewport {
    renderer: Option<PortalGpu>,
    scene: Option<CompiledAssembly>,
    bare_mesh: Option<qualia_core_db::render::assets::Mesh>,
    source: Option<SourceRef>,
    camera: Option<PreviewCamera>,
    selected: Option<PartSelection>,
    view_lighting: Option<super::ViewLighting>,
    limits: PreviewLimits,
    budget: ViewportBudget,
    counters: ViewportCounters,
    frame_id: u64,
    frame_valid: bool,
}
impl RetainedViewport {
    /// GPU creation is lazy and remains on the native worker that owns this provider.
    pub fn new(limits: PreviewLimits, budget: ViewportBudget) -> Result<Self, String> {
        super::camera::extent(limits.side, limits.side)?;
        if limits.vertices == 0
            || limits.indices == 0
            || budget.working_bytes == 0
            || budget.pick_triangles == 0
            || budget.overlay_steps == 0
        {
            return Err("empty native viewport budget".into());
        }
        Ok(Self {
            renderer: None,
            scene: None,
            bare_mesh: None,
            source: None,
            camera: None,
            selected: None,
            view_lighting: None,
            limits,
            budget,
            counters: ViewportCounters::default(),
            frame_id: 0,
            frame_valid: false,
        })
    }
    pub fn counters(&self) -> ViewportCounters {
        self.counters
    }
    pub fn frame_id(&self) -> Option<u64> {
        (self.frame_valid
            && self
                .source
                .as_ref()
                .is_some_and(|s| s.check_current().is_ok()))
        .then_some(self.frame_id)
    }
    pub fn selection(&self) -> Option<&PartSelection> {
        self.frame_id().and(self.selected.as_ref())
    }
    pub fn camera(&self) -> Option<PreviewCamera> {
        self.camera
    }
    pub fn bounds(&self) -> Option<([f32; 3], [f32; 3])> {
        self.scene
            .as_ref()
            .map(|s| (s.mesh.min, s.mesh.max))
            .or_else(|| self.bare_mesh.as_ref().map(|m| (m.min, m.max)))
    }

    /// Call on session/authority retirement even when the Studio stamp is unchanged.
    /// Device survives; derived scene/frame/selection cannot publish afterward.
    pub fn invalidate(&mut self) {
        self.scene = None;
        self.bare_mesh = None;
        self.source = None;
        self.camera = None;
        self.selected = None;
        self.frame_valid = false;
    }
    pub fn clear(&mut self) {
        self.invalidate();
    }
    pub fn clear_selection(&mut self) {
        self.selected = None;
    }

    pub fn part_bounds(&self, key: &str) -> Result<([f32; 3], [f32; 3]), String> {
        if key.is_empty() || key.len() > 4096 {
            return Err("invalid viewport part key".into());
        }
        let scene = self.scene.as_ref().ok_or("native viewport not uploaded")?;
        let part = scene
            .parts
            .iter()
            .find(|p| p.key == key)
            .ok_or("viewport source part absent")?;
        let mut min = [f32::INFINITY; 3];
        let mut max = [f32::NEG_INFINITY; 3];
        for triangle in &scene.mesh.triangles
            [part.first_index as usize / 3..(part.first_index + part.index_count) as usize / 3]
        {
            for vertex in triangle {
                for axis in 0..3 {
                    min[axis] = min[axis].min(scene.mesh.positions[*vertex as usize][axis]);
                    max[axis] = max[axis].max(scene.mesh.positions[*vertex as usize][axis]);
                }
            }
        }
        Ok((min, max))
    }

    fn admit_bytes(&self, scene: &CompiledAssembly) -> Result<(), String> {
        let add = |sum: usize, count: usize, size: usize| {
            count
                .checked_mul(size)
                .and_then(|n| sum.checked_add(n))
                .ok_or("viewport working byte overflow")
        };
        // Positions + normals + UV + upload colours + conservative Core normal/tangent scratch.
        let mut n = add(0, scene.mesh.positions.len(), 160)?;
        n = add(n, scene.mesh.triangles.len(), 24)?;
        n = add(
            n,
            scene.materials.len(),
            std::mem::size_of::<qualia_core_db::container_10d::MaterialRecord>(),
        )?;
        n = add(n, scene.parts.len(), 256)?;
        n = add(n, self.limits.side as usize * self.limits.side as usize, 4)?;
        for part in &scene.parts {
            n = add(
                n,
                part.key.len() + part.parent.as_ref().map_or(0, String::len),
                1,
            )?;
        }
        if n > self.budget.working_bytes {
            return Err("viewport scene working byte budget exceeded".into());
        }
        Ok(())
    }

    /// Preserve the existing single-mesh factor-only path, including Core's own
    /// generated normal frames. Such source has no assembly part identifiers.
    pub fn replace_mesh(
        &mut self,
        source: SourceRef,
        mesh: qualia_core_db::render::assets::Mesh,
        material: qualia_core_db::container_10d::MaterialRecord,
        camera: PreviewCamera,
        cancelled: &AtomicBool,
    ) -> Result<PreviewFrame, String> {
        use qualia_core_db::container_10d::SubmeshRange;
        check(&source, cancelled)?;
        camera_state(camera)?;
        crate::graphical::materials::validate_factor_material(&material)?;
        let indices = mesh
            .triangles
            .len()
            .checked_mul(3)
            .ok_or("viewport mesh size overflow")?;
        let bytes = mesh
            .positions
            .len()
            .checked_mul(160)
            .and_then(|n| indices.checked_mul(8).and_then(|i| n.checked_add(i)))
            .and_then(|n| n.checked_add(self.limits.side as usize * self.limits.side as usize * 4))
            .ok_or("viewport mesh byte overflow")?;
        if mesh.positions.is_empty()
            || mesh.positions.len() > self.limits.vertices
            || indices == 0
            || indices > self.limits.indices
            || indices > u32::MAX as usize
            || bytes > self.budget.working_bytes
            || mesh.positions.iter().flatten().any(|v| !v.is_finite())
            || mesh
                .triangles
                .iter()
                .flatten()
                .any(|i| *i as usize >= mesh.positions.len())
        {
            return Err("invalid or oversized native viewport mesh".into());
        }
        self.invalidate();
        if self.renderer.is_none() {
            self.renderer = Some(PortalGpu::new_offscreen(
                self.limits.side,
                self.limits.side,
                1,
            )?);
            self.counters.device_creations += 1;
        }
        let renderer = self.renderer.as_mut().unwrap();
        let mut settings = AssemblyRenderSettings::for_camera(camera);
        if let Some(lighting) = self.view_lighting {
            lighting.apply_to(&mut settings);
        }
        renderer.set_ambient_enabled(false);
        renderer.set_tensor_projection_enabled(false);
        renderer.set_clear_color(0.025, 0.035, 0.05, 1.0);
        renderer.set_lighting(
            settings.sun_direction[0],
            settings.sun_direction[1],
            settings.sun_direction[2],
            settings.sun_intensity,
            settings.ambient_intensity,
        );
        renderer.set_shadows_enabled(settings.shadows);
        renderer.set_screen_space_ao_enabled(settings.screen_space_ao);
        renderer.set_environment_probes(None, [0.0; 3]);
        if !renderer.set_exposure_compensation(settings.exposure_ev) {
            return Err("native viewport exposure unavailable".into());
        }
        let indices: Vec<u32> = mesh.triangles.iter().flatten().copied().collect();
        let colours = vec![[1.0; 4]; mesh.positions.len()];
        let ranges = [SubmeshRange {
            first_index: 0,
            index_count: indices.len() as u32,
            material_id: material.id,
            semantic_id: 0,
        }];
        check(&source, cancelled)?;
        renderer.upload_mesh_colored_with_frames_and_materials(
            &mesh.positions,
            &colours,
            None,
            None,
            &indices,
            &[material],
            &ranges,
        )?;
        self.counters.scene_uploads += 1;
        check(&source, cancelled)?;
        self.bare_mesh = Some(mesh);
        self.source = Some(source.clone());
        self.render(source, camera, cancelled)
    }

    pub fn replace_scene(
        &mut self,
        source: SourceRef,
        scene: CompiledAssembly,
        mut settings: AssemblyRenderSettings,
        resources: Option<&TextureResources>,
        probes: Option<&EnvironmentProbeSet>,
        cancelled: &AtomicBool,
    ) -> Result<PreviewFrame, String> {
        check(&source, cancelled)?;
        validate_compiled(&scene, self.limits)?;
        self.admit_bytes(&scene)?;
        crate::graphical::surfaces::resources::validate_material_resources(
            &scene.materials,
            resources,
        )?;
        crate::graphical::assembly::validate_render_settings(&settings)?;
        if let Some(lighting) = self.view_lighting {
            lighting.apply_to(&mut settings);
        }
        let state = camera_state(settings.camera)?;
        // Once upload starts no prior scene is allowed to publish after a partial failure.
        self.invalidate();
        if self.renderer.is_none() {
            self.renderer = Some(PortalGpu::new_offscreen(
                self.limits.side,
                self.limits.side,
                1,
            )?);
            self.counters.device_creations += 1;
        }
        let renderer = self.renderer.as_mut().unwrap();
        if let Some(resources) = resources {
            resources.upload(&scene.materials, renderer)?;
        }
        check(&source, cancelled)?;
        renderer.set_ambient_enabled(false);
        renderer.set_tensor_projection_enabled(false);
        renderer.set_clear_color(
            settings.clear_color[0],
            settings.clear_color[1],
            settings.clear_color[2],
            settings.clear_color[3],
        );
        renderer.set_lighting(
            settings.sun_direction[0],
            settings.sun_direction[1],
            settings.sun_direction[2],
            settings.sun_intensity,
            settings.ambient_intensity,
        );
        renderer.set_shadows_enabled(settings.shadows);
        renderer.set_screen_space_ao_enabled(settings.screen_space_ao);
        let indices: Vec<u32> = scene.mesh.triangles.iter().flatten().copied().collect();
        let colours = vec![[1.0; 4]; scene.mesh.positions.len()];
        renderer.upload_mesh_colored_with_frames_and_materials_and_uv0(
            &scene.mesh.positions,
            &colours,
            Some(&scene.normals),
            None,
            Some(&scene.uv0),
            &indices,
            &scene.materials,
            &scene.ranges,
        )?;
        self.counters.scene_uploads += 1;
        renderer.set_environment_probes(probes, [0.0; 3]);
        if !renderer.set_exposure_compensation(settings.exposure_ev) {
            return Err("native viewport exposure unavailable".into());
        }
        check(&source, cancelled)?;
        self.scene = Some(scene);
        self.source = Some(source.clone());
        self.render(source, preview_camera(state), cancelled)
    }

    /// Update only view illumination/exposure using the retained device and uploaded scene.
    /// The selected view preset persists through source changes; no authored field is changed.
    pub fn update_lighting(
        &mut self,
        source: SourceRef,
        settings: AssemblyRenderSettings,
        cancelled: &AtomicBool,
    ) -> Result<PreviewFrame, String> {
        check(&source, cancelled)?;
        crate::graphical::assembly::validate_render_settings(&settings)?;
        super::ViewLighting::from_settings(&settings).validate()?;
        if self.source.as_ref() != Some(&source)
            || (self.scene.is_none() && self.bare_mesh.is_none())
        {
            return Err("native viewport lighting source has retired".into());
        }
        let camera = self.camera.ok_or("native viewport camera unavailable")?;
        let renderer = self
            .renderer
            .as_mut()
            .ok_or("native viewport not uploaded")?;
        if !renderer.set_exposure_compensation(settings.exposure_ev) {
            return Err("native viewport exposure unavailable".into());
        }
        renderer.set_lighting(
            settings.sun_direction[0],
            settings.sun_direction[1],
            settings.sun_direction[2],
            settings.sun_intensity,
            settings.ambient_intensity,
        );
        renderer.set_shadows_enabled(settings.shadows);
        renderer.set_screen_space_ao_enabled(settings.screen_space_ao);
        self.view_lighting = Some(super::ViewLighting::from_settings(&settings));
        self.render(source, camera, cancelled)
    }

    /// Native GPU submission/readback is synchronous; cancellation is checked before and after.
    /// Core does not expose interruption of an already submitted GPU readback here.
    pub fn render(
        &mut self,
        source: SourceRef,
        camera: PreviewCamera,
        cancelled: &AtomicBool,
    ) -> Result<PreviewFrame, String> {
        self.frame_valid = false;
        check(&source, cancelled)?;
        if self.source.as_ref() != Some(&source)
            || (self.scene.is_none() && self.bare_mesh.is_none())
        {
            return Err("native viewport scene source has retired".into());
        }
        let state = camera_state(camera)?;
        let next = self
            .frame_id
            .checked_add(1)
            .ok_or("viewport frame sequence exhausted")?;
        let renderer = self
            .renderer
            .as_mut()
            .ok_or("native viewport not uploaded")?;
        renderer.set_camera_target(
            state.yaw,
            state.pitch,
            state.zoom,
            state.target[0],
            state.target[1],
            state.target[2],
        );
        renderer.render(0.0, &SystemTelemetry::default())?;
        self.counters.frames += 1;
        let mut rgba8 = vec![0; renderer.required_rgba8_bytes()];
        renderer.read_rgba8_into(&mut rgba8)?;
        check(&source, cancelled)?;
        if let Some(selected) = self.selected.as_ref() {
            highlight::overlay(
                &source,
                self.scene.as_ref().unwrap(),
                selected,
                preview_camera(state),
                self.limits.side,
                &mut rgba8,
                self.budget.overlay_steps,
                cancelled,
            )?;
        }
        check(&source, cancelled)?;
        self.frame_id = next;
        self.frame_valid = true;
        self.camera = Some(preview_camera(state));
        if let Some(selected) = self.selected.as_mut() {
            selected.frame_id = next;
        }
        Ok(PreviewFrame {
            source,
            session_revision: None,
            execution_epoch: None,
            side: self.limits.side,
            rgba8,
        })
    }

    /// Picking consumes the exact frame sequence currently displayed by the caller.
    pub fn pick(
        &mut self,
        source: SourceRef,
        frame_id: u64,
        x: f32,
        y: f32,
        cancelled: &AtomicBool,
    ) -> Result<Option<PartSelection>, String> {
        check(&source, cancelled)?;
        if !self.frame_valid || frame_id != self.frame_id || self.source.as_ref() != Some(&source) {
            return Err("pick source/frame has retired".into());
        }
        let selected = pick_part(
            &source,
            frame_id,
            self.scene.as_ref().ok_or("viewport scene unavailable")?,
            self.camera.ok_or("viewport camera unavailable")?,
            self.limits.side,
            self.limits.side,
            x,
            y,
            self.budget.pick_triangles,
            cancelled,
        )?;
        self.selected = selected.clone();
        Ok(selected)
    }
}
