//! Native owner adapter: canonical evaluated Vibe values -> retained Core viewport.
//! Never owns editable source, host effects, package rights or a competing scene store.
use super::{
    assembly::{self, AssemblyRenderSettings},
    controller::{MeshPreviewProvider, PreviewProvider, SessionEpoch},
    preview::{PreviewFrame, SourceRef},
    surfaces::TextureResources,
    viewport::{
        apply_gesture, frame_bounds, RetainedViewport, ViewLighting, ViewportBudget,
        ViewportCommand, ViewportCounters, ViewportGesture,
    },
    viewport_contract::{ViewportOutput, ViewportState},
};
use std::sync::atomic::{AtomicBool, Ordering};
use vibe::Value;

pub struct InteractivePreviewProvider {
    admission: MeshPreviewProvider,
    viewport: RetainedViewport,
    scene_source: Option<SourceRef>,
    source_has_parts: bool,
    epoch: Option<SessionEpoch>,
}
impl InteractivePreviewProvider {
    pub fn new(admission: MeshPreviewProvider) -> Result<Self, String> {
        Self::with_budget(admission, ViewportBudget::default())
    }
    pub fn with_budget(
        admission: MeshPreviewProvider,
        budget: ViewportBudget,
    ) -> Result<Self, String> {
        let viewport = RetainedViewport::new(admission.limits, budget)?;
        Ok(Self {
            admission,
            viewport,
            scene_source: None,
            source_has_parts: false,
            epoch: None,
        })
    }
    pub fn counters(&self) -> ViewportCounters {
        self.viewport.counters()
    }

    fn check(&self, source: &SourceRef, cancelled: &AtomicBool) -> Result<(), String> {
        if cancelled.load(Ordering::Acquire) {
            return Err("native viewport cancelled".into());
        }
        if self.epoch.as_ref().is_some_and(|epoch| !epoch.is_current()) {
            return Err("viewport execution context retired".into());
        }
        source.check_current()
    }
    fn invalidate(&mut self) {
        self.scene_source = None;
        self.source_has_parts = false;
        self.viewport.invalidate();
    }

    /// A cache hit never clones/evaluates/adopts the supplied canonical value.
    /// Source and execution-context identity determine whether derived geometry is reusable.
    fn ensure_scene(
        &mut self,
        source: &SourceRef,
        value: &Value,
        resources: Option<&TextureResources>,
        cancelled: &AtomicBool,
    ) -> Result<Option<PreviewFrame>, String> {
        self.check(source, cancelled)?;
        if self.scene_source.as_ref() == Some(source) && self.viewport.bounds().is_some() {
            return Ok(None);
        }
        // Preserve the actual clamped user view across accepted source changes.
        // invalidate retires scene/selection/frame bindings, not the user's view intent.
        let camera = self.viewport.camera().unwrap_or(self.admission.camera);
        self.invalidate();
        let is_parts = assembly::is_assembly(value) || matches!(value, Value::List(_));
        let result = if is_parts {
            // Only source changes pay for wrapping the raw SourceSession parts list.
            let envelope;
            let authored = if matches!(value, Value::List(_)) {
                envelope = Value::Record(std::collections::BTreeMap::from([(
                    "parts".into(),
                    value.clone(),
                )]));
                &envelope
            } else {
                value
            };
            let resolved = super::surfaces::resolution::resolve(authored, resources)?;
            let scene = assembly::admit_assembly_with_materials(
                authored,
                self.admission.limits,
                &resolved.materials,
            )?;
            self.check(source, cancelled)?;
            self.viewport.replace_scene(
                source.clone(),
                scene,
                AssemblyRenderSettings::for_camera(camera),
                resolved.resources.as_ref(),
                None,
                cancelled,
            )
        } else {
            // Uses the old factor-only source admission and exact Core generated-normal upload.
            let (mesh, material) = super::materials::authored_scene(
                value,
                self.admission.material,
                self.admission.limits,
            )?;
            self.check(source, cancelled)?;
            self.viewport
                .replace_mesh(source.clone(), mesh, material, camera, cancelled)
        };
        let frame = match result {
            Ok(frame) => frame,
            Err(error) => {
                self.invalidate();
                return Err(error);
            }
        };
        self.check(source, cancelled)?;
        self.admission.camera = self.viewport.camera().unwrap_or(camera);
        self.scene_source = Some(source.clone());
        self.source_has_parts = is_parts;
        Ok(Some(frame))
    }

    fn finish(
        &self,
        source: SourceRef,
        mut frame: PreviewFrame,
        cancelled: &AtomicBool,
    ) -> Result<ViewportOutput, String> {
        self.check(&source, cancelled)?;
        frame.execution_epoch = self.epoch.clone();
        let state = ViewportState {
            source,
            execution_epoch: self.epoch.clone(),
            frame_id: self
                .viewport
                .frame_id()
                .ok_or("viewport frame retired before publication")?,
            selected_part: self
                .viewport
                .selection()
                .map(|selection| selection.part_key.clone()),
            camera: self
                .viewport
                .camera()
                .ok_or("viewport camera unavailable")?,
        };
        Ok(ViewportOutput { frame, state })
    }
}

impl PreviewProvider for InteractivePreviewProvider {
    fn viewport_cached(&self, source: &SourceRef) -> bool {
        self.epoch.as_ref().is_some_and(SessionEpoch::is_current)
            && source.check_current().is_ok()
            && self.scene_source.as_ref() == Some(source)
            && self.viewport.bounds().is_some()
    }
    /// Context reset is conservative even when the epoch compares equal: host resource binding
    /// may change authorized immutable inputs without changing editable source text.
    fn set_execution_context(&mut self, epoch: &SessionEpoch) {
        self.invalidate();
        self.epoch = Some(epoch.clone());
    }
    fn validate(&self, value: &Value) -> Result<(), String> {
        self.admission.validate(value)
    }
    fn validate_with_resources(
        &self,
        value: &Value,
        resources: Option<&TextureResources>,
    ) -> Result<(), String> {
        self.admission.validate_with_resources(value, resources)
    }
    fn preview(&mut self, source: SourceRef, value: &Value) -> Result<PreviewFrame, String> {
        self.preview_with_resources(source, value, None)
    }
    fn preview_with_resources(
        &mut self,
        source: SourceRef,
        value: &Value,
        resources: Option<&TextureResources>,
    ) -> Result<PreviewFrame, String> {
        let cancelled = AtomicBool::new(false);
        let prepared = self.ensure_scene(&source, value, resources, &cancelled)?;
        let frame = if let Some(frame) = prepared {
            frame
        } else {
            self.viewport
                .render(source.clone(), self.admission.camera, &cancelled)?
        };
        Ok(self.finish(source, frame, &cancelled)?.frame)
    }
    fn viewport(
        &mut self,
        request: ViewportCommand,
        source: SourceRef,
        value: &Value,
        resources: Option<&TextureResources>,
        cancelled: &AtomicBool,
    ) -> Result<ViewportOutput, String> {
        let lighting = ViewLighting::from_command(&request);
        if let Some(lighting) = lighting {
            lighting.validate()?;
        }
        let prepared = self.ensure_scene(&source, value, resources, cancelled)?;
        let mut camera = self.viewport.camera().unwrap_or(self.admission.camera);
        let side = self.admission.limits.side;
        let mut prepared = prepared;
        match request {
            ViewportCommand::Redraw => {}
            ViewportCommand::FrameAll => {
                let (min, max) = self
                    .viewport
                    .bounds()
                    .ok_or("viewport source bounds unavailable")?;
                camera = frame_bounds(camera, min, max, side, side)?;
                prepared = None;
            }
            ViewportCommand::FramePart { key } => {
                if !self.source_has_parts {
                    return Err("single-mesh source has no declared assembly part keys".into());
                }
                let (min, max) = self.viewport.part_bounds(&key)?;
                camera = frame_bounds(camera, min, max, side, side)?;
                prepared = None;
            }
            ViewportCommand::Orbit { dx, dy } => {
                camera = apply_gesture(camera, ViewportGesture::Orbit { dx, dy }, side, side)?;
                prepared = None;
            }
            ViewportCommand::Pan { dx, dy } => {
                camera = apply_gesture(camera, ViewportGesture::Pan { dx, dy }, side, side)?;
                prepared = None;
            }
            ViewportCommand::Zoom { steps } => {
                camera = apply_gesture(camera, ViewportGesture::Zoom { steps }, side, side)?;
                prepared = None;
            }
            ViewportCommand::Pick { frame_id, x, y } => {
                if !self.source_has_parts {
                    return Err("single-mesh source has no declared assembly part keys".into());
                }
                self.viewport
                    .pick(source.clone(), frame_id, x, y, cancelled)?;
                prepared = None;
            }
            ViewportCommand::ClearSelection => {
                self.viewport.clear_selection();
                prepared = None;
            }
            ViewportCommand::Lighting { .. } => {
                let mut settings = AssemblyRenderSettings::for_camera(camera);
                lighting
                    .ok_or("native viewport lighting command unavailable")?
                    .apply_to(&mut settings);
                prepared = Some(self.viewport.update_lighting(
                    source.clone(),
                    settings,
                    cancelled,
                )?);
            }
        }
        self.check(&source, cancelled)?;
        let frame = if let Some(frame) = prepared {
            frame
        } else {
            self.viewport.render(source.clone(), camera, cancelled)?
        };
        self.check(&source, cancelled)?;
        self.admission.camera = camera;
        self.finish(source, frame, cancelled)
    }
}

#[cfg(test)]
#[path = "viewport_provider_tests.rs"]
mod tests;
