//! Input adaptation over the same canonical camera used by PortalGpu.
use crate::graphical::preview::PreviewCamera;
use qualia_core_db::render::{
    camera::{CameraState, CAMERA_MAX_ZOOM, CAMERA_TAN_HALF_FOV},
    lighting::normalize3,
};

#[derive(Clone, Copy, Debug)]
pub enum ViewportGesture {
    /// Physical viewport pixel deltas, after native UI coordinate conversion.
    Orbit {
        dx: f32,
        dy: f32,
    },
    Pan {
        dx: f32,
        dy: f32,
    },
    /// Positive wheel steps move closer to the target.
    Zoom {
        steps: f32,
    },
}

pub fn camera_state(camera: PreviewCamera) -> Result<CameraState, String> {
    crate::graphical::preview::validate_frame(8, camera)?;
    Ok(CameraState::new(camera.yaw, camera.pitch, camera.distance)
        .with_target(camera.target)
        .clamped())
}

pub(crate) fn preview_camera(state: CameraState) -> PreviewCamera {
    PreviewCamera {
        yaw: state.yaw,
        pitch: state.pitch,
        distance: state.zoom,
        target: state.target,
    }
}

pub(crate) fn extent(width: u32, height: u32) -> Result<f32, String> {
    if !(8..=1024).contains(&width) || !(8..=1024).contains(&height) {
        return Err("viewport extent exceeds native budget".into());
    }
    // The public canonical projection clamps very narrow aspect ratios to 0.05.
    Ok((width as f32 / height as f32).max(0.05))
}

/// Screen axes are recovered from the canonical projection, not a second orbit implementation.
pub(crate) fn screen_axes(camera: CameraState, aspect: f32) -> ([f32; 3], [f32; 3]) {
    let matrix = camera.view_projection(aspect);
    (
        normalize3([matrix[0][0], matrix[1][0], matrix[2][0]]),
        normalize3([matrix[0][1], matrix[1][1], matrix[2][1]]),
    )
}

pub fn apply_gesture(
    camera: PreviewCamera,
    gesture: ViewportGesture,
    width: u32,
    height: u32,
) -> Result<PreviewCamera, String> {
    let aspect = extent(width, height)?;
    let mut state = camera_state(camera)?;
    match gesture {
        ViewportGesture::Orbit { dx, dy } | ViewportGesture::Pan { dx, dy }
            if !dx.is_finite() || !dy.is_finite() || dx.abs() > 8192.0 || dy.abs() > 8192.0 =>
        {
            return Err("invalid viewport gesture delta".into())
        }
        ViewportGesture::Orbit { dx, dy } => {
            state.yaw -= dx / width as f32 * std::f32::consts::TAU;
            state.pitch += dy / height as f32 * std::f32::consts::PI;
        }
        ViewportGesture::Pan { dx, dy } => {
            let (right, up) = screen_axes(state, aspect);
            let span = 2.0 * state.zoom * CAMERA_TAN_HALF_FOV;
            for axis in 0..3 {
                state.target[axis] += -dx * right[axis] * span * aspect / width as f32
                    + dy * up[axis] * span / height as f32;
            }
        }
        ViewportGesture::Zoom { steps } => {
            if !steps.is_finite() || steps.abs() > 100.0 {
                return Err("invalid viewport zoom".into());
            }
            state.zoom *= (-steps * 0.15).exp();
        }
    }
    let state = state.clamped();
    if state.target.iter().any(|v| !v.is_finite()) {
        return Err("viewport pan overflow".into());
    }
    Ok(preview_camera(state))
}

/// Fit a bounding sphere enclosing the authored AABB, accounting for vertical AND horizontal FOV.
/// Core navigation::camera_frame_node frames an origin-relative node, not arbitrary part bounds.
pub fn frame_bounds(
    camera: PreviewCamera,
    min: [f32; 3],
    max: [f32; 3],
    width: u32,
    height: u32,
) -> Result<PreviewCamera, String> {
    let aspect = extent(width, height)?;
    let mut state = camera_state(camera)?;
    if (0..3).any(|i| !min[i].is_finite() || !max[i].is_finite() || min[i] > max[i]) {
        return Err("invalid viewport framing bounds".into());
    }
    let mut radius2 = 0.0_f64;
    for axis in 0..3 {
        state.target[axis] = (min[axis] as f64 * 0.5 + max[axis] as f64 * 0.5) as f32;
        radius2 += ((max[axis] as f64 - min[axis] as f64) * 0.5).powi(2);
    }
    let tangent = f64::from(CAMERA_TAN_HALF_FOV * aspect.min(1.0));
    let distance = radius2.sqrt() * (1.0 + tangent * tangent).sqrt() / tangent * 1.1;
    if !distance.is_finite() || distance > f64::from(CAMERA_MAX_ZOOM) {
        return Err("scene bounds exceed canonical camera framing range".into());
    }
    state.zoom = distance.max(0.1) as f32;
    Ok(preview_camera(state.clamped()))
}
