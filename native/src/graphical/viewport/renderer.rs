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
    rig_preview: Option<crate::graphical::animated_rig_preview::AnimatedRigPreview>,
}
impl RetainedViewport {
    /// Seed retained CPU state for source/selection validation only. This test
    /// helper does not create or exercise a GPU, rendering or human operation.
    #[cfg(test)]
    pub(crate) fn rig_test_seed(source: SourceRef, scene: CompiledAssembly, selected: PartSelection) -> Self {
        let mut viewport = Self::new(PreviewLimits::default(), ViewportBudget::default()).unwrap();
        viewport.frame_id = selected.frame_id;
        viewport.source = Some(source);
        viewport.scene = Some(scene);
        viewport.selected = Some(selected);
        viewport.frame_valid = true;
        viewport
    }
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
            rig_preview: None,
        })
    }

    /// Admit only the currently displayed selection and its exact compiled bind
    /// geometry. The caller must resolve resource bytes under existing project
    /// read authority; this method is not a resource or permission provider.
    pub fn admit_selected_rig(
        &self,
        source: SourceRef,
        descriptor: crate::graphical::rig_binding::RigSourceDescriptor,
        selected_resource_key: &str,
        resource_bytes: &[u8],
        bind: crate::graphical::rig_binding::BindMesh<'_>,
        resident_budget: usize,
        cancelled: &AtomicBool,
    ) -> Result<crate::graphical::rig_binding::BoundRig, String> {
        check(&source, cancelled)?;
        if self.source.as_ref() != Some(&source) || !self.frame_valid {
            return Err("rig viewport source/frame retired".into());
        }
        let selection = self.selected.as_ref().ok_or("select a rig part first")?;
        if selection.source != source || selection.frame_id != self.frame_id
            || selection.part_key != descriptor.part_id
        { return Err("selected rig part/source/frame changed".into()); }
        let scene = self.scene.as_ref().ok_or("rig assembly viewport unavailable")?;
        let span = selected_rig_span(scene, selection, &bind)?;
        let rig = crate::graphical::rig_binding::admit_bound_rig(
            source.clone(), descriptor, selected_resource_key, resource_bytes, bind, span, resident_budget,
        )?;
        check(&source, cancelled)?;
        Ok(rig)
    }

    /// Stage against the currently selected retained geometry. No GPU mutation occurs.
    fn prepare_selected_rig(&self, source: SourceRef, mut descriptor: crate::graphical::rig_binding::RigSourceDescriptor, resource_bytes: &[u8], action: crate::graphical::rig_binding::RigControl, cancelled: &AtomicBool) -> Result<(crate::graphical::rig_binding::BoundRig, crate::graphical::rig_binding::RigControl),String> {
        use qualia_core_db::container_10d::{self, SectionType, Container10dHeader};
        check(&source, cancelled)?;
        let header = Container10dHeader::parse(resource_bytes).map_err(|e| format!("rig header refused: {e:?}"))?;
        let table = container_10d::parse_section_table(resource_bytes, &header).map_err(|e| format!("rig table refused: {e:?}"))?;
        let mesh = table.iter().find(|s| s.typ() == Some(SectionType::QuantizedMesh)).ok_or("rig lacks mesh")?;
        let start = mesh.byte_offset as usize;
        let end = start.checked_add(mesh.byte_length as usize).ok_or("rig mesh range overflow")?;
        let mesh_bytes = resource_bytes.get(start..end).ok_or("rig mesh outside resource")?;
        let (mesh_header, needed) = container_10d::parse_mesh_header(mesh_bytes).map_err(|e| format!("rig mesh header refused: {e:?}"))?;
        if needed != mesh_bytes.len() || mesh_header.vertex_count as usize > self.limits.vertices || (mesh_header.triangle_count as usize).checked_mul(3).is_none_or(|n| n > self.limits.indices) { return Err("Selected rig mesh exceeds viewport limits".into()); }
        let decoded = container_10d::decode_mesh_section(mesh_bytes).map_err(|e| format!("rig mesh refused: {e:?}"))?;
        let indices: Vec<u32> = decoded.triangles.iter().flatten().copied().collect();
        let scene = self.scene.as_ref().ok_or("rig viewport scene unavailable")?;
        let selected = self.selected.as_ref().ok_or("select the rig part in the viewport")?;
        let part = scene.parts.iter().find(|p| p.key == selected.part_key).ok_or("selected rig part absent")?;
        let tri = scene.mesh.triangles.get(part.first_index as usize / 3).ok_or("selected rig has no triangles")?;
        let first = tri[0].checked_sub(*indices.first().ok_or("rig has no indices")?).ok_or("rig offset is negative")? as usize;
        let end = first.checked_add(decoded.positions.len()).ok_or("rig vertex overflow")?;
        let normals = scene.normals.get(first..end).ok_or("selected rig normals unavailable")?;
        let uv0 = scene.uv0.get(first..end).ok_or("selected rig UVs unavailable")?;
        if let crate::graphical::rig_binding::RigControl::Adopt { part_id, bind_id } = &action {
            if part_id != &selected.part_key || bind_id.is_empty() || bind_id.len() > 128 || bind_id.contains('\0') {
                return Err("Rig adoption requires the selected viewport part and a bounded bind identifier".into());
            }
            let section = container_10d::skeletal_container::read_skeletal_section(resource_bytes)
                .map_err(|e| format!("sealed SKL1 resource refused: {e:?}"))?;
            let counts = section.counts();
            if counts.vertices != decoded.positions.len() || counts.joints == 0 || counts.clips == 0 {
                return Err("Rig adoption requires compatible vertices, joints and a clip".into());
            }
            let joint_ids = (0..counts.joints).map(|i| section.joint_id(i).ok_or("Missing rig joint identifier"))
                .collect::<Result<Vec<_>, _>>()?;
            let clip_ids = (0..counts.clips).map(|i| section.clip(i).map(|clip| clip.id).ok_or("Missing rig clip identifier"))
                .collect::<Result<Vec<_>, _>>()?;
            let selected_clip_id = *clip_ids.first().ok_or("Rig has no clips")?;
            let vertex_lineage = (0..counts.vertices).map(|i| u32::try_from(i).map_err(|_| "Rig lineage overflow"))
                .collect::<Result<Vec<_>, _>>()?;
            descriptor = crate::graphical::rig_binding::RigSourceDescriptor {
                schema: "poet.bound-rig/v1".into(), source_artifact_id: source.artifact_id.clone(),
                part_id: part_id.clone(), bind_id: bind_id.clone(), resource_key: descriptor.resource_key.clone(),
                resource_sha256: crate::graphical::rig_binding::resource_sha256(resource_bytes),
                bind_mesh_sha256: crate::graphical::rig_binding::bind_mesh_sha256(&decoded.positions, &indices),
                normal_uv_sha256: crate::graphical::rig_binding::normal_uv_sha256(normals, uv0),
                joint_ids, clip_ids, selected_clip_id, seek_seconds: 0.0,
                joint_deltas: Vec::new(), vertex_lineage,
            };
        }
        let bind = crate::graphical::rig_binding::BindMesh { positions: &decoded.positions, normals, uv0, indices: &indices };
        let key = descriptor.resource_key.clone();
        let rig = self.admit_selected_rig(source.clone(), descriptor, &key, resource_bytes, bind, 42 * 1024 * 1024, cancelled)?;
        check(&source, cancelled)?;
        let action = match action {
            crate::graphical::rig_binding::RigControl::Adopt { .. } => crate::graphical::rig_binding::RigControl::Seek { seconds: 0.0 },
            other => other,
        };
        Ok((rig, action))
    }

    pub(crate) fn stage_selected_rig(&self, source: SourceRef, descriptor: crate::graphical::rig_binding::RigSourceDescriptor, resource_bytes: &[u8], action: crate::graphical::rig_binding::RigControl, cancelled: &AtomicBool) -> Result<crate::graphical::rig_binding::rig_candidate::StagedRigControl,String> {
        let (rig, action) = self.prepare_selected_rig(source.clone(), descriptor, resource_bytes, action, cancelled)?;
        let candidate = crate::graphical::rig_binding::rig_candidate::stage_admitted(rig, action)?;
        check(&source, cancelled)?;
        Ok(candidate)
    }

    /// Render an authorized, selected rig pose without changing authored source.
    /// Runtime time is a disposable viewport state; reset reloads source geometry.
    pub(crate) fn preview_selected_rig(
        &mut self, source: SourceRef, descriptor: crate::graphical::rig_binding::RigSourceDescriptor,
        resource_bytes: &[u8], delta_seconds: f64, cancelled: &AtomicBool,
    ) -> Result<PreviewFrame, String> {
        use crate::graphical::{animated_rig_preview::AnimatedRigPreview, rig_binding::{RigControl, resource_sha256}};
        check(&source, cancelled)?;
        if !delta_seconds.is_finite() || !(0.0..=1.0).contains(&delta_seconds) { return Err("Rig preview step must be finite and between 0 and 1 second".into()); }
        let selected = self.selection().ok_or("Select the current rig part in the viewport")?;
        if selected.source != source || selected.part_key != descriptor.part_id || self.source.as_ref() != Some(&source) {
            return Err("Rig preview selection/source retired".into());
        }
        if resource_sha256(resource_bytes) != descriptor.resource_sha256 { return Err("Rig preview resource changed".into()); }
        let mut preview = if let Some(existing) = self.rig_preview.take() {
            if existing.rig().source() != &source || existing.rig().descriptor() != &descriptor {
                self.rig_preview = Some(existing);
                return Err("Restore source geometry before previewing a different rig".into());
            }
            existing
        } else {
            let seek = descriptor.seek_seconds;
            let (rig, _) = self.prepare_selected_rig(source.clone(), descriptor, resource_bytes, RigControl::Seek { seconds: seek }, cancelled)?;
            AnimatedRigPreview::new(rig)
        };
        // Once a player has advanced, a failure may have changed CPU or GPU
        // buffers; retire the complete viewport rather than publishing stale state.
        let result = (|| {
            let frame = preview.advance(delta_seconds, false)?;
            let scene = self.scene.as_mut().ok_or("rig viewport scene unavailable")?;
            validate_rig_pose(scene, frame.span, frame.positions, frame.normals, frame.tangents)?;
            check(&source, cancelled)?;
            let renderer = self.renderer.as_mut().ok_or("native viewport not uploaded")?;
            if !renderer.write_mesh_animated_frames(frame.span.first_vertex, frame.positions, frame.normals, frame.tangents) {
                return Err("native rig frame upload refused".into());
            }
            apply_rig_pose(scene, frame.span, frame.positions, frame.normals);
            check(&source, cancelled)?;
            let camera = self.camera.ok_or("native viewport camera unavailable")?;
            self.render(source.clone(), camera, cancelled)
        })();
        match result {
            Ok(frame) => { self.rig_preview = Some(preview); Ok(frame) }
            Err(error) => { self.invalidate(); Err(error) }
        }
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
        self.rig_preview = None;
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

/// Verify the selected part has the exact contiguous bind layout expected by a
/// rig, and that no other rendered triangle shares the animated vertices.
/// Transformed, expanded, reordered or shared geometry is refused until a
/// qualified correspondence/remap is supplied by the geometry owner.
pub(super) fn selected_rig_span(
    scene: &CompiledAssembly,
    selected: &PartSelection,
    bind: &crate::graphical::rig_binding::BindMesh<'_>,
) -> Result<crate::graphical::rig_binding::ViewportSpan, String> {
    use crate::graphical::rig_binding::ViewportSpan;
    let part = scene.parts.iter().find(|p| p.key == selected.part_key && p.semantic_id == selected.semantic_id)
        .ok_or("selected rig part is absent from compiled scene")?;
    if part.first_index % 3 != 0 || part.index_count == 0 || part.index_count % 3 != 0
        || bind.indices.len() != part.index_count as usize || bind.positions.is_empty()
        || bind.normals.len() != bind.positions.len() || bind.uv0.len() != bind.positions.len()
    { return Err("selected rig topology or stream count differs from compiled part".into()); }
    let start = (part.first_index / 3) as usize;
    let end = start.checked_add((part.index_count / 3) as usize).ok_or("selected rig range overflow")?;
    let triangles = scene.mesh.triangles.get(start..end).ok_or("selected rig part range outside scene")?;
    if !(start..end).contains(&selected.triangle_index) {
        return Err("selected rig triangle no longer belongs to part".into());
    }
    let first = triangles.first().ok_or("selected rig has no triangles")?[0]
        .checked_sub(*bind.indices.first().ok_or("selected rig has no indices")?)
        .ok_or("selected rig bind offset is negative")?;
    let first_usize = first as usize;
    let limit = first_usize.checked_add(bind.positions.len()).ok_or("selected rig vertex range overflow")?;
    if limit > scene.mesh.positions.len() || limit > scene.normals.len() || limit > scene.uv0.len()
        || bind.indices.iter().any(|v| *v as usize >= bind.positions.len())
        || triangles.iter().flatten().zip(bind.indices.iter()).any(|(actual, local)| {
            local.checked_add(first) != Some(*actual)
        })
    { return Err("selected rig requires exact contiguous vertex correspondence".into()); }
    let bits_equal = |a: &[f32], b: &[f32]| a.iter().zip(b).all(|(x, y)| x.to_bits() == y.to_bits());
    for i in 0..bind.positions.len() {
        if !bits_equal(&scene.mesh.positions[first_usize + i], &bind.positions[i])
            || !bits_equal(&scene.normals[first_usize + i], &bind.normals[i])
            || !bits_equal(&scene.uv0[first_usize + i], &bind.uv0[i])
        { return Err("selected rig bind position, normal or UV differs from viewport".into()); }
    }
    for (i, tri) in scene.mesh.triangles.iter().enumerate() {
        if !(start..end).contains(&i) && tri.iter().any(|v| (*v as usize) >= first_usize && (*v as usize) < limit) {
            return Err("selected rig vertices are shared by other rendered triangles".into());
        }
    }
    u32::try_from(limit).map_err(|_| "selected rig vertex range exceeds GPU address space")?;
    Ok(ViewportSpan { first_vertex: first, vertex_count: bind.positions.len() })
}

/// Validate the CPU mirror before the GPU changes; no allocation or mutation.
pub(super) fn validate_rig_pose(scene: &CompiledAssembly, span: crate::graphical::rig_binding::ViewportSpan, positions: &[[f32;3]], normals: &[[f32;3]], tangents: &[[f32;4]]) -> Result<(), String> {
    let first = span.first_vertex as usize;
    let end = first.checked_add(span.vertex_count).ok_or("rig pose span overflow")?;
    if span.vertex_count == 0 || positions.len() != span.vertex_count || normals.len() != span.vertex_count || tangents.len() != span.vertex_count || end > scene.mesh.positions.len() || end > scene.normals.len() || positions.iter().flatten().chain(normals.iter().flatten()).chain(tangents.iter().flatten()).any(|x| !x.is_finite()) {
        return Err("invalid native rig pose or viewport span".into());
    }
    Ok(())
}

/// Called only after validation and GPU admission succeed.
pub(super) fn apply_rig_pose(scene: &mut CompiledAssembly, span: crate::graphical::rig_binding::ViewportSpan, positions: &[[f32;3]], normals: &[[f32;3]]) {
    let first = span.first_vertex as usize;
    let end = first + span.vertex_count;
    scene.mesh.positions[first..end].copy_from_slice(positions);
    scene.normals[first..end].copy_from_slice(normals);
    let mut min = [f32::INFINITY;3];
    let mut max = [f32::NEG_INFINITY;3];
    for position in &scene.mesh.positions { for axis in 0..3 { min[axis] = min[axis].min(position[axis]); max[axis] = max[axis].max(position[axis]); } }
    scene.mesh.min = min; scene.mesh.max = max;
}
