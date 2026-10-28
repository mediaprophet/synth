//! Retained native Portal viewport. Derived state only; SourceSession owns edits.
mod camera;
mod highlight;
mod picking;
mod renderer;
pub use camera::{apply_gesture, camera_state, frame_bounds, ViewportGesture};
pub use picking::PartSelection;
pub use renderer::{RetainedViewport, ViewportBudget, ViewportCounters};

/// Coordinates refer to the square native RGBA attachment, after UI letterbox mapping.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum ViewportCommand {
    Redraw,
    FrameAll,
    FramePart {
        key: String,
    },
    Orbit {
        dx: f32,
        dy: f32,
    },
    Pan {
        dx: f32,
        dy: f32,
    },
    Zoom {
        steps: f32,
    },
    Pick {
        frame_id: u64,
        x: f32,
        y: f32,
    },
    ClearSelection,
    /// View-only skeletal step against an exact selected HMC resource.
    RigPreview { resource: crate::graphical::checkpoints::ResourceInspectionRequest, delta_seconds: f64 },
    /// Reload source geometry and discard disposable rig playback.
    RestoreSourceGeometry,
    /// View-only illumination/exposure; never writes material, source or world physics.
    Lighting {
        exposure_ev: f32,
        sun_direction: [f32; 3],
        sun_intensity: f32,
        ambient_intensity: f32,
        shadows: bool,
        screen_space_ao: bool,
    },
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ViewLighting {
    pub exposure_ev: f32,
    pub sun_direction: [f32; 3],
    pub sun_intensity: f32,
    pub ambient_intensity: f32,
    pub shadows: bool,
    pub screen_space_ao: bool,
}
impl ViewLighting {
    pub fn existing_scene() -> Self {
        let settings = crate::graphical::assembly::AssemblyRenderSettings::for_camera(
            crate::graphical::preview::PreviewCamera {
                yaw: 0.0,
                pitch: 0.0,
                distance: 1.0,
                target: [0.0; 3],
            },
        );
        Self::from_settings(&settings)
    }
    /// Candidate studio viewing preset. Visual acceptance must be reviewed per content/profile.
    pub fn neutral_studio() -> Self {
        Self {
            exposure_ev: 1.0,
            sun_direction: [0.45, 0.8, -0.6],
            sun_intensity: 1.8,
            ambient_intensity: 0.5,
            shadows: true,
            screen_space_ao: true,
        }
    }
    pub fn command(self) -> ViewportCommand {
        preview_lighting(self)
    }
    pub fn validate(self) -> Result<(), String> {
        use qualia_core_db::render::output::{MAX_HDR_EXPOSURE_EV, MIN_HDR_EXPOSURE_EV};
        if !self.exposure_ev.is_finite()
            || !(MIN_HDR_EXPOSURE_EV..=MAX_HDR_EXPOSURE_EV).contains(&self.exposure_ev)
        {
            return Err("view lighting exposure exceeds canonical EV bounds".into());
        }
        let mut settings = crate::graphical::assembly::AssemblyRenderSettings::for_camera(
            crate::graphical::preview::PreviewCamera {
                yaw: 0.0,
                pitch: 0.0,
                distance: 1.0,
                target: [0.0; 3],
            },
        );
        self.apply_to(&mut settings);
        crate::graphical::assembly::validate_render_settings(&settings)
    }
    pub(crate) fn from_settings(s: &crate::graphical::assembly::AssemblyRenderSettings) -> Self {
        Self {
            exposure_ev: s.exposure_ev,
            sun_direction: s.sun_direction,
            sun_intensity: s.sun_intensity,
            ambient_intensity: s.ambient_intensity,
            shadows: s.shadows,
            screen_space_ao: s.screen_space_ao,
        }
    }
    pub(crate) fn apply_to(self, s: &mut crate::graphical::assembly::AssemblyRenderSettings) {
        s.exposure_ev = self.exposure_ev;
        s.sun_direction = self.sun_direction;
        s.sun_intensity = self.sun_intensity;
        s.ambient_intensity = self.ambient_intensity;
        s.shadows = self.shadows;
        s.screen_space_ao = self.screen_space_ao;
    }
    pub(crate) fn from_command(command: &ViewportCommand) -> Option<Self> {
        if let ViewportCommand::Lighting {
            exposure_ev,
            sun_direction,
            sun_intensity,
            ambient_intensity,
            shadows,
            screen_space_ao,
        } = command
        {
            Some(Self {
                exposure_ev: *exposure_ev,
                sun_direction: *sun_direction,
                sun_intensity: *sun_intensity,
                ambient_intensity: *ambient_intensity,
                shadows: *shadows,
                screen_space_ao: *screen_space_ao,
            })
        } else {
            None
        }
    }
}
pub fn preview_lighting(s: ViewLighting) -> ViewportCommand {
    ViewportCommand::Lighting {
        exposure_ev: s.exposure_ev,
        sun_direction: s.sun_direction,
        sun_intensity: s.sun_intensity,
        ambient_intensity: s.ambient_intensity,
        shadows: s.shadows,
        screen_space_ao: s.screen_space_ao,
    }
}

#[cfg(test)]
pub(crate) mod tests;
