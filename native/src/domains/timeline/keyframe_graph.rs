// Clean-room adaptation derived from EffectCraft architectural concepts
// Copyright (c) 2024 Storytold / EffectCraft contributors
// Licensed under the MIT License.
// Adapted for native Poet / VibeScript timeline interpolation without external JS/Boa dependencies.

use std::fmt;

/// Supported interpolation algorithms for keyframe segments.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum KeyframeInterpolation {
    /// Step hold: retains previous value until next keyframe is reached.
    HoldStep,
    /// Linear lerp between adjacent keyframes.
    Linear,
    /// Cubic Bezier transition using normalized control tangents in [0.0, 1.0] for time x.
    CubicBezier {
        out_tangent: [f32; 2],
        in_tangent: [f32; 2],
    },
}

/// A discrete keyframe along a timeline property track.
#[derive(Debug, Clone, PartialEq)]
pub struct KeyframePoint {
    pub frame_index: u32,
    pub value: f64,
    pub interp: KeyframeInterpolation,
}

/// A property track bound to an entity DID and a property identifier hash.
#[derive(Debug, Clone, PartialEq)]
pub struct PropertyTrack {
    pub property_id_hash: u64,
    pub target_entity_did: u64,
    pub keyframes: Vec<KeyframePoint>,
    pub vibe_expression: Option<String>,
}

/// Error kinds for timeline keyframe graph validation and evaluation.
#[derive(Debug, Clone, PartialEq)]
pub enum KeyframeGraphError {
    NonMonotonicKeyframes { prev_frame: u32, next_frame: u32 },
    DuplicateKeyframe { frame_index: u32 },
    ControlPointOutOfRange { axis: &'static str, val: f32 },
    NonFiniteValue,
    TrackNotFound { property_id_hash: u64, target_entity_did: u64 },
    EmptyTrack,
}

impl fmt::Display for KeyframeGraphError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NonMonotonicKeyframes { prev_frame, next_frame } => {
                write!(f, "Non-monotonic keyframes: frame {} > {}", prev_frame, next_frame)
            }
            Self::DuplicateKeyframe { frame_index } => {
                write!(f, "Duplicate keyframe at frame index {}", frame_index)
            }
            Self::ControlPointOutOfRange { axis, val } => {
                write!(f, "Control point {}={} out of normalized range [0.0, 1.0]", axis, val)
            }
            Self::NonFiniteValue => write!(f, "Encountered non-finite keyframe or control value"),
            Self::TrackNotFound { property_id_hash, target_entity_did } => {
                write!(f, "Track not found for prop 0x{:016x}, did 0x{:016x}", property_id_hash, target_entity_did)
            }
            Self::EmptyTrack => write!(f, "Track has zero keyframes"),
        }
    }
}

impl std::error::Error for KeyframeGraphError {}

/// Solves cubic Bezier parameter t in [0.0, 1.0] for target time x in [0.0, 1.0].
/// Uses bounded Newton-Raphson with bisection fallback for strict determinism.
pub fn solve_cubic_bezier_time(x_target: f64, x1: f32, x2: f32) -> f64 {
    if x_target <= 0.0 {
        return 0.0;
    }
    if x_target >= 1.0 {
        return 1.0;
    }

    let x1 = x1 as f64;
    let x2 = x2 as f64;

    // Bezier polynomial coefficients for X(t) with P0=(0,0) and P3=(1,1):
    // X(t) = 3(1-t)^2*t*x1 + 3(1-t)*t^2*x2 + t^3
    let cx = 3.0 * x1;
    let bx = 3.0 * (x2 - x1) - cx;
    let ax = 1.0 - cx - bx;

    let sample_x = |t: f64| ((ax * t + bx) * t + cx) * t;
    let sample_dx = |t: f64| (3.0 * ax * t + 2.0 * bx) * t + cx;

    let mut t = x_target;
    let mut t_min = 0.0f64;
    let mut t_max = 1.0f64;

    // Bounded Newton-Raphson (up to 8 iterations)
    for _ in 0..8 {
        let current_x = sample_x(t);
        let error = current_x - x_target;
        if error.abs() < 1e-7 {
            return t;
        }
        if current_x > x_target {
            t_max = t;
        } else {
            t_min = t;
        }
        let dx = sample_dx(t);
        if dx.abs() < 1e-6 {
            break;
        }
        let t_next = t - error / dx;
        if t_next <= t_min || t_next >= t_max {
            break;
        }
        t = t_next;
    }

    // Bisection fallback (up to 16 iterations)
    for _ in 0..16 {
        t = 0.5 * (t_min + t_max);
        let current_x = sample_x(t);
        if (current_x - x_target).abs() < 1e-7 {
            return t;
        }
        if current_x > x_target {
            t_max = t;
        } else {
            t_min = t;
        }
    }

    0.5 * (t_min + t_max)
}

/// Evaluates cubic Bezier Y coordinate given parameter t in [0.0, 1.0].
pub fn sample_cubic_bezier_y(t: f64, y1: f32, y2: f32) -> f64 {
    let y1 = y1 as f64;
    let y2 = y2 as f64;
    let cy = 3.0 * y1;
    let by = 3.0 * (y2 - y1) - cy;
    let ay = 1.0 - cy - by;
    ((ay * t + by) * t + cy) * t
}

/// A timeline graph holding multi-entity property tracks.
#[derive(Debug, Clone, Default)]
pub struct TimelineKeyframeGraph {
    tracks: Vec<PropertyTrack>,
}

impl TimelineKeyframeGraph {
    pub fn new() -> Self {
        Self { tracks: Vec::new() }
    }

    /// Validates and adds a property track to the graph.
    pub fn add_track(&mut self, track: PropertyTrack) -> Result<(), KeyframeGraphError> {
        Self::validate_track(&track)?;
        // Replace existing track or append
        if let Some(pos) = self.tracks.iter().position(|t| {
            t.property_id_hash == track.property_id_hash
                && t.target_entity_did == track.target_entity_did
        }) {
            self.tracks[pos] = track;
        } else {
            self.tracks.push(track);
        }
        Ok(())
    }

    pub fn get_track(&self, property_id_hash: u64, target_entity_did: u64) -> Option<&PropertyTrack> {
        self.tracks.iter().find(|t| {
            t.property_id_hash == property_id_hash && t.target_entity_did == target_entity_did
        })
    }

    /// Validates keyframe monotonicity, bounds, and finiteness.
    pub fn validate_track(track: &PropertyTrack) -> Result<(), KeyframeGraphError> {
        if track.keyframes.is_empty() {
            return Err(KeyframeGraphError::EmptyTrack);
        }

        for (i, kf) in track.keyframes.iter().enumerate() {
            if !kf.value.is_finite() {
                return Err(KeyframeGraphError::NonFiniteValue);
            }
            if let KeyframeInterpolation::CubicBezier { out_tangent, in_tangent } = kf.interp {
                if !out_tangent[0].is_finite() || !out_tangent[1].is_finite()
                    || !in_tangent[0].is_finite() || !in_tangent[1].is_finite()
                {
                    return Err(KeyframeGraphError::NonFiniteValue);
                }
                if !(0.0..=1.0).contains(&out_tangent[0]) {
                    return Err(KeyframeGraphError::ControlPointOutOfRange {
                        axis: "out_tangent.x",
                        val: out_tangent[0],
                    });
                }
                if !(0.0..=1.0).contains(&in_tangent[0]) {
                    return Err(KeyframeGraphError::ControlPointOutOfRange {
                        axis: "in_tangent.x",
                        val: in_tangent[0],
                    });
                }
            }
            if i > 0 {
                let prev = track.keyframes[i - 1].frame_index;
                let curr = kf.frame_index;
                if curr == prev {
                    return Err(KeyframeGraphError::DuplicateKeyframe { frame_index: curr });
                }
                if curr < prev {
                    return Err(KeyframeGraphError::NonMonotonicKeyframes {
                        prev_frame: prev,
                        next_frame: curr,
                    });
                }
            }
        }
        Ok(())
    }

    /// Evaluates the property value at a fractional frame index.
    pub fn evaluate_track(
        &self,
        property_id_hash: u64,
        target_entity_did: u64,
        frame: f64,
    ) -> Result<f64, KeyframeGraphError> {
        if !frame.is_finite() {
            return Err(KeyframeGraphError::NonFiniteValue);
        }
        let track = self.get_track(property_id_hash, target_entity_did)
            .ok_or(KeyframeGraphError::TrackNotFound { property_id_hash, target_entity_did })?;

        Self::evaluate_keyframes(&track.keyframes, frame)
    }

    /// Evaluates a verified slice of keyframes at the target frame.
    pub fn evaluate_keyframes(keyframes: &[KeyframePoint], frame: f64) -> Result<f64, KeyframeGraphError> {
        if keyframes.is_empty() {
            return Err(KeyframeGraphError::EmptyTrack);
        }
        if keyframes.len() == 1 {
            return Ok(keyframes[0].value);
        }

        let first = &keyframes[0];
        let last = &keyframes[keyframes.len() - 1];

        if frame <= first.frame_index as f64 {
            return Ok(first.value);
        }
        if frame >= last.frame_index as f64 {
            return Ok(last.value);
        }

        // Find enclosing keyframe span [k0, k1]
        for window in keyframes.windows(2) {
            let k0 = &window[0];
            let k1 = &window[1];
            let f0 = k0.frame_index as f64;
            let f1 = k1.frame_index as f64;

            if frame >= f0 && frame <= f1 {
                let span = f1 - f0;
                let norm_x = if span > 0.0 { (frame - f0) / span } else { 0.0 };

                return match k0.interp {
                    KeyframeInterpolation::HoldStep => Ok(k0.value),
                    KeyframeInterpolation::Linear => {
                        Ok(k0.value + (k1.value - k0.value) * norm_x)
                    }
                    KeyframeInterpolation::CubicBezier { out_tangent, in_tangent } => {
                        let t = solve_cubic_bezier_time(norm_x, out_tangent[0], in_tangent[0]);
                        let y = sample_cubic_bezier_y(t, out_tangent[1], in_tangent[1]);
                        Ok(k0.value + (k1.value - k0.value) * y)
                    }
                };
            }
        }

        Ok(last.value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hold_step_interpolation() {
        let keyframes = vec![
            KeyframePoint { frame_index: 0, value: 10.0, interp: KeyframeInterpolation::HoldStep },
            KeyframePoint { frame_index: 10, value: 20.0, interp: KeyframeInterpolation::HoldStep },
        ];
        assert_eq!(TimelineKeyframeGraph::evaluate_keyframes(&keyframes, 0.0).unwrap(), 10.0);
        assert_eq!(TimelineKeyframeGraph::evaluate_keyframes(&keyframes, 5.0).unwrap(), 10.0);
        assert_eq!(TimelineKeyframeGraph::evaluate_keyframes(&keyframes, 9.99).unwrap(), 10.0);
        assert_eq!(TimelineKeyframeGraph::evaluate_keyframes(&keyframes, 10.0).unwrap(), 20.0);
    }

    #[test]
    fn test_linear_interpolation() {
        let keyframes = vec![
            KeyframePoint { frame_index: 0, value: 0.0, interp: KeyframeInterpolation::Linear },
            KeyframePoint { frame_index: 100, value: 50.0, interp: KeyframeInterpolation::Linear },
        ];
        let val_50 = TimelineKeyframeGraph::evaluate_keyframes(&keyframes, 50.0).unwrap();
        assert!((val_50 - 25.0).abs() < 1e-6);
        let val_20 = TimelineKeyframeGraph::evaluate_keyframes(&keyframes, 20.0).unwrap();
        assert!((val_20 - 10.0).abs() < 1e-6);
    }

    #[test]
    fn test_cubic_bezier_ease_in_out() {
        let keyframes = vec![
            KeyframePoint {
                frame_index: 0,
                value: 0.0,
                interp: KeyframeInterpolation::CubicBezier {
                    out_tangent: [0.42, 0.0],
                    in_tangent: [0.58, 1.0],
                },
            },
            KeyframePoint { frame_index: 100, value: 100.0, interp: KeyframeInterpolation::Linear },
        ];
        let mid = TimelineKeyframeGraph::evaluate_keyframes(&keyframes, 50.0).unwrap();
        assert!((mid - 50.0).abs() < 1e-2);
        let quarter = TimelineKeyframeGraph::evaluate_keyframes(&keyframes, 25.0).unwrap();
        assert!(quarter < 25.0); // Ease-in starts slower than linear
    }

    #[test]
    fn test_clamping_before_and_after_range() {
        let keyframes = vec![
            KeyframePoint { frame_index: 10, value: 5.0, interp: KeyframeInterpolation::Linear },
            KeyframePoint { frame_index: 20, value: 15.0, interp: KeyframeInterpolation::Linear },
        ];
        assert_eq!(TimelineKeyframeGraph::evaluate_keyframes(&keyframes, 0.0).unwrap(), 5.0);
        assert_eq!(TimelineKeyframeGraph::evaluate_keyframes(&keyframes, 30.0).unwrap(), 15.0);
    }

    #[test]
    fn test_negative_non_monotonic_rejected() {
        let track = PropertyTrack {
            property_id_hash: 1,
            target_entity_did: 42,
            keyframes: vec![
                KeyframePoint { frame_index: 20, value: 1.0, interp: KeyframeInterpolation::Linear },
                KeyframePoint { frame_index: 10, value: 2.0, interp: KeyframeInterpolation::Linear },
            ],
            vibe_expression: None,
        };
        let err = TimelineKeyframeGraph::validate_track(&track).unwrap_err();
        assert_eq!(err, KeyframeGraphError::NonMonotonicKeyframes { prev_frame: 20, next_frame: 10 });
    }

    #[test]
    fn test_negative_duplicate_keyframe_rejected() {
        let track = PropertyTrack {
            property_id_hash: 1,
            target_entity_did: 42,
            keyframes: vec![
                KeyframePoint { frame_index: 10, value: 1.0, interp: KeyframeInterpolation::Linear },
                KeyframePoint { frame_index: 10, value: 2.0, interp: KeyframeInterpolation::Linear },
            ],
            vibe_expression: None,
        };
        let err = TimelineKeyframeGraph::validate_track(&track).unwrap_err();
        assert_eq!(err, KeyframeGraphError::DuplicateKeyframe { frame_index: 10 });
    }

    #[test]
    fn test_negative_control_point_out_of_range_rejected() {
        let track = PropertyTrack {
            property_id_hash: 1,
            target_entity_did: 42,
            keyframes: vec![
                KeyframePoint {
                    frame_index: 0,
                    value: 0.0,
                    interp: KeyframeInterpolation::CubicBezier {
                        out_tangent: [1.5, 0.0],
                        in_tangent: [0.5, 1.0],
                    },
                },
                KeyframePoint { frame_index: 10, value: 1.0, interp: KeyframeInterpolation::Linear },
            ],
            vibe_expression: None,
        };
        let err = TimelineKeyframeGraph::validate_track(&track).unwrap_err();
        assert_eq!(
            err,
            KeyframeGraphError::ControlPointOutOfRange { axis: "out_tangent.x", val: 1.5 }
        );
    }

    #[test]
    fn test_negative_non_finite_rejected() {
        let track = PropertyTrack {
            property_id_hash: 1,
            target_entity_did: 42,
            keyframes: vec![
                KeyframePoint { frame_index: 0, value: f64::NAN, interp: KeyframeInterpolation::Linear },
            ],
            vibe_expression: None,
        };
        let err = TimelineKeyframeGraph::validate_track(&track).unwrap_err();
        assert_eq!(err, KeyframeGraphError::NonFiniteValue);
    }
}
