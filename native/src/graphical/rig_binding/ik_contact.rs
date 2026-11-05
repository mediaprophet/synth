//! Zero-Heap Analytical Two-Bone Terrain Foot-Contact IK Solver (`ik_contact.rs`).
//!
//! Provides real-time terrain conforming foot placement, slope normal alignment,
//! vertical pelvis sink compensation, and substrate penetration prevention for
//! vertebrate bipeds, quadrupeds, and invertebrate limbs:
//! - [`FootContactTarget`]: terrain contact target position, normal, and stance phase locking.
//! - [`LimbChainSpec`]: two-bone limb dimensions, anatomical hinge limits, and pole direction.
//! - [`IkContactResult`]: solved joint positions, foot orientation, pelvis sink, and contact status.
//! - [`solve_terrain_foot_contact_zero_heap`]: zero-heap per-frame solver.

/// Specification of a two-bone articulating limb chain (e.g. thigh+shank, humerus+radius).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LimbChainSpec {
    pub upper_length: f32,
    pub lower_length: f32,
    pub foot_length: f32,
    /// Minimum hinge angle in radians (prevents hyperextension popping, e.g. 0.08 rad).
    pub min_hinge_rad: f32,
    /// Maximum hinge angle in radians (prevents self-intersection hyperflexion, e.g. 2.75 rad).
    pub max_hinge_rad: f32,
    /// Primary anatomical bend vector (e.g. [0.0, 0.0, 1.0] for knees, [0.0, 0.0, -1.0] for hocks).
    pub pole_vector: [f32; 3],
}

/// Authoring / animation target for foot ground contact.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FootContactTarget {
    pub target_position: [f32; 3],
    pub terrain_normal: [f32; 3],
    pub stance_phase_locked: bool,
    pub foot_planted_threshold_m: f32,
}

/// Output solution containing solved joint positions and contact diagnostics.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct IkContactResult {
    pub solved_hip_position: [f32; 3],
    pub solved_knee_position: [f32; 3],
    pub solved_ankle_position: [f32; 3],
    pub solved_toe_position: [f32; 3],
    /// Foot orientation in Euler angles [pitch, yaw, roll] aligned to terrain slope.
    pub foot_orientation_pyr: [f32; 3],
    pub pelvis_sink_offset_y: f32,
    pub contact_locked: bool,
    pub penetration_prevented: bool,
    pub hinge_angle_rad: f32,
}

impl Default for IkContactResult {
    fn default() -> Self {
        Self {
            solved_hip_position: [0.0; 3],
            solved_knee_position: [0.0; 3],
            solved_ankle_position: [0.0; 3],
            solved_toe_position: [0.0; 3],
            foot_orientation_pyr: [0.0; 3],
            pelvis_sink_offset_y: 0.0,
            contact_locked: false,
            penetration_prevented: false,
            hinge_angle_rad: 0.0,
        }
    }
}

/// Fail-closed error taxonomy for foot contact IK evaluation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IkContactError {
    InvalidLimbLength,
    InvalidTerrainNormal,
    InvalidTargetPosition,
    InvalidHingeLimits,
}

#[inline]
fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[inline]
fn len3(v: [f32; 3]) -> f32 {
    dot3(v, v).sqrt()
}

#[inline]
fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

#[inline]
fn add3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

#[inline]
fn scale3(v: [f32; 3], s: f32) -> [f32; 3] {
    [v[0] * s, v[1] * s, v[2] * s]
}

#[inline]
fn cross3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

#[inline]
fn normalize3_or(v: [f32; 3], fallback: [f32; 3]) -> [f32; 3] {
    let l = len3(v);
    if l > 1e-6 {
        [v[0] / l, v[1] / l, v[2] / l]
    } else {
        fallback
    }
}

/// Solves zero-heap terrain-conforming foot-contact inverse kinematics.
pub fn solve_terrain_foot_contact_zero_heap(
    hip_origin: [f32; 3],
    spec: &LimbChainSpec,
    target: &FootContactTarget,
    substrate_height: f32,
    result: &mut IkContactResult,
) -> Result<(), IkContactError> {
    if spec.upper_length <= 0.0 || spec.lower_length <= 0.0 || spec.foot_length < 0.0 {
        return Err(IkContactError::InvalidLimbLength);
    }
    if spec.min_hinge_rad < 0.0 || spec.min_hinge_rad > spec.max_hinge_rad {
        return Err(IkContactError::InvalidHingeLimits);
    }
    let norm_len = len3(target.terrain_normal);
    if norm_len < 1e-4 || !target.terrain_normal.iter().all(|v| v.is_finite()) {
        return Err(IkContactError::InvalidTerrainNormal);
    }
    if !target.target_position.iter().all(|v| v.is_finite()) {
        return Err(IkContactError::InvalidTargetPosition);
    }

    let ground_normal = normalize3_or(target.terrain_normal, [0.0, 1.0, 0.0]);

    // 1. Pelvis sink compensation:
    // If the target foot position is farther than reach * 0.98, sink pelvis to reach ground.
    let l1 = spec.upper_length;
    let l2 = spec.lower_length;
    let max_reach = l1 + l2;
    let mut initial_diff = sub3(target.target_position, hip_origin);
    let initial_dist = len3(initial_diff);

    let mut pelvis_sink = 0.0f32;
    if initial_dist > max_reach * 0.98 {
        pelvis_sink = (initial_dist - max_reach * 0.98).min(l1 * 0.85);
    }
    let hip = [hip_origin[0], hip_origin[1] - pelvis_sink, hip_origin[2]];

    // 2. Ankle placement & Stance locking:
    let mut ankle = target.target_position;
    let mut penetration_prevented = false;

    // Prevent substrate penetration:
    if ankle[1] < substrate_height {
        ankle[1] = substrate_height;
        penetration_prevented = true;
    }

    let diff = sub3(ankle, hip);
    let mut dist = len3(diff);

    // Clamp distance within reachable two-bone envelope:
    let min_reach = (l1 - l2).abs() + 1e-4;
    let max_reach_clamped = max_reach * 0.999;
    let clamped_dist = dist.clamp(min_reach, max_reach_clamped);

    // 3. Two-bone Law of Cosines with anatomical hinge clamping:
    // Interior hinge angle beta opposite to target distance:
    let cos_beta_raw = (l1 * l1 + l2 * l2 - clamped_dist * clamped_dist) / (2.0 * l1 * l2);
    let beta_raw = cos_beta_raw.clamp(-1.0, 1.0).acos();

    // Clamp to anatomical limits:
    let beta = beta_raw.clamp(spec.min_hinge_rad, spec.max_hinge_rad);

    // Recompute actual limb reach based on clamped hinge angle:
    let effective_dist_sq = l1 * l1 + l2 * l2 - 2.0 * l1 * l2 * beta.cos();
    let effective_dist = effective_dist_sq.max(1e-6).sqrt();

    // Angle alpha at hip:
    let cos_alpha = (l1 * l1 + effective_dist * effective_dist - l2 * l2) / (2.0 * l1 * effective_dist);
    let alpha = cos_alpha.clamp(-1.0, 1.0).acos();

    let u = normalize3_or(diff, [0.0, -1.0, 0.0]);
    let pole = normalize3_or(spec.pole_vector, [0.0, 0.0, 1.0]);

    // Construct perpendicular in the bend plane:
    let mut bend_normal = cross3(u, pole);
    if len3(bend_normal) < 1e-5 {
        bend_normal = if u[0].abs() < 0.9 { cross3(u, [1.0, 0.0, 0.0]) } else { cross3(u, [0.0, 0.0, 1.0]) };
    }
    let bend_normal = normalize3_or(bend_normal, [1.0, 0.0, 0.0]);
    let bend_perp = normalize3_or(cross3(bend_normal, u), [0.0, 0.0, 1.0]);

    // Knee position:
    let knee = add3(
        add3(hip, scale3(u, l1 * alpha.cos())),
        scale3(bend_perp, l1 * alpha.sin()),
    );

    // Solved ankle from knee:
    let to_ankle = normalize3_or(sub3(ankle, knee), u);
    let solved_ankle = add3(knee, scale3(to_ankle, l2));

    // 4. Slope normal alignment & Toe position:
    // Compute Euler angles [pitch, yaw, roll] aligning foot sole to terrain normal:
    let pitch = (-ground_normal[2]).atan2(ground_normal[1]);
    let roll = (ground_normal[0]).atan2(ground_normal[1]);
    let yaw = 0.0f32;

    // Foot forward vector projected onto terrain plane:
    let base_forward = [0.0, 0.0, 1.0];
    let normal_comp = dot3(base_forward, ground_normal);
    let foot_dir = normalize3_or(
        sub3(base_forward, scale3(ground_normal, normal_comp)),
        [0.0, 0.0, 1.0],
    );
    let mut solved_toe = add3(solved_ankle, scale3(foot_dir, spec.foot_length));

    // Check toe substrate penetration:
    if solved_toe[1] < substrate_height {
        solved_toe[1] = substrate_height;
        penetration_prevented = true;
    }

    // 5. Populate output result:
    result.solved_hip_position = hip;
    result.solved_knee_position = knee;
    result.solved_ankle_position = solved_ankle;
    result.solved_toe_position = solved_toe;
    result.foot_orientation_pyr = [pitch, yaw, roll];
    result.pelvis_sink_offset_y = pelvis_sink;
    result.contact_locked = target.stance_phase_locked;
    result.penetration_prevented = penetration_prevented;
    result.hinge_angle_rad = beta;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_spec() -> LimbChainSpec {
        LimbChainSpec {
            upper_length: 0.5,
            lower_length: 0.5,
            foot_length: 0.2,
            min_hinge_rad: 0.15, // ~8.6 degrees
            max_hinge_rad: 2.70, // ~154 degrees
            pole_vector: [0.0, 0.0, 1.0],
        }
    }

    #[test]
    fn test_solve_terrain_foot_contact_flat() {
        let spec = sample_spec();
        let target = FootContactTarget {
            target_position: [0.0, 0.0, 0.1],
            terrain_normal: [0.0, 1.0, 0.0],
            stance_phase_locked: true,
            foot_planted_threshold_m: 0.02,
        };
        let mut result = IkContactResult::default();
        let hip_origin = [0.0, 0.9, 0.0];

        solve_terrain_foot_contact_zero_heap(hip_origin, &spec, &target, 0.0, &mut result).unwrap();

        assert!(result.contact_locked);
        assert!(!result.penetration_prevented);
        assert!(result.hinge_angle_rad >= spec.min_hinge_rad);
        assert!(result.hinge_angle_rad <= spec.max_hinge_rad);
        assert_eq!(result.foot_orientation_pyr, [0.0, 0.0, 0.0]);
        assert!((result.solved_ankle_position[1] - 0.0).abs() < 0.05);
    }

    #[test]
    fn test_solve_terrain_foot_contact_sloped() {
        let spec = sample_spec();
        // 30 degree slope: normal has y = cos(30 deg) ~ 0.866, z = sin(30 deg) = 0.5
        let target = FootContactTarget {
            target_position: [0.0, 0.1, 0.2],
            terrain_normal: [0.0, 0.866025, 0.5],
            stance_phase_locked: false,
            foot_planted_threshold_m: 0.02,
        };
        let mut result = IkContactResult::default();
        let hip_origin = [0.0, 0.9, 0.0];

        solve_terrain_foot_contact_zero_heap(hip_origin, &spec, &target, 0.0, &mut result).unwrap();

        assert!(!result.contact_locked);
        // Pitch should be non-zero and negative reflecting upward slope in z:
        assert!(result.foot_orientation_pyr[0].abs() > 0.4);
    }

    #[test]
    fn test_pelvis_sink_on_far_target() {
        let spec = sample_spec();
        // Target is at distance 1.2, but max reach is 1.0:
        let target = FootContactTarget {
            target_position: [0.0, -0.3, 0.0],
            terrain_normal: [0.0, 1.0, 0.0],
            stance_phase_locked: true,
            foot_planted_threshold_m: 0.02,
        };
        let mut result = IkContactResult::default();
        let hip_origin = [0.0, 0.9, 0.0];

        solve_terrain_foot_contact_zero_heap(hip_origin, &spec, &target, -0.5, &mut result).unwrap();

        assert!(result.pelvis_sink_offset_y > 0.0);
        assert_eq!(result.solved_hip_position[1], hip_origin[1] - result.pelvis_sink_offset_y);
    }

    #[test]
    fn test_hinge_limits_prevent_hyperextension() {
        let spec = sample_spec();
        let target = FootContactTarget {
            target_position: [0.0, 0.0, 0.0],
            terrain_normal: [0.0, 1.0, 0.0],
            stance_phase_locked: true,
            foot_planted_threshold_m: 0.02,
        };
        let mut result = IkContactResult::default();
        let hip_origin = [0.0, 0.99, 0.0];

        solve_terrain_foot_contact_zero_heap(hip_origin, &spec, &target, 0.0, &mut result).unwrap();

        // Hinge angle must be clamped to at least min_hinge_rad:
        assert!(result.hinge_angle_rad >= spec.min_hinge_rad);
    }

    #[test]
    fn test_substrate_penetration_prevention() {
        let spec = sample_spec();
        let target = FootContactTarget {
            target_position: [0.0, -0.2, 0.0], // penetrates substrate at 0.0
            terrain_normal: [0.0, 1.0, 0.0],
            stance_phase_locked: true,
            foot_planted_threshold_m: 0.02,
        };
        let mut result = IkContactResult::default();
        let hip_origin = [0.0, 0.8, 0.0];

        solve_terrain_foot_contact_zero_heap(hip_origin, &spec, &target, 0.0, &mut result).unwrap();

        assert!(result.penetration_prevented);
        assert!(result.solved_ankle_position[1] >= 0.0);
        assert!(result.solved_toe_position[1] >= 0.0);
    }

    #[test]
    fn test_invalid_inputs_fail_closed() {
        let mut spec = sample_spec();
        spec.upper_length = -1.0;
        let target = FootContactTarget {
            target_position: [0.0, 0.0, 0.0],
            terrain_normal: [0.0, 1.0, 0.0],
            stance_phase_locked: false,
            foot_planted_threshold_m: 0.02,
        };
        let mut result = IkContactResult::default();
        assert_eq!(
            solve_terrain_foot_contact_zero_heap([0.0, 1.0, 0.0], &spec, &target, 0.0, &mut result),
            Err(IkContactError::InvalidLimbLength)
        );

        let mut spec_ok = sample_spec();
        let mut target_bad_norm = target;
        target_bad_norm.terrain_normal = [0.0, 0.0, 0.0];
        assert_eq!(
            solve_terrain_foot_contact_zero_heap([0.0, 1.0, 0.0], &spec_ok, &target_bad_norm, 0.0, &mut result),
            Err(IkContactError::InvalidTerrainNormal)
        );
    }
}
