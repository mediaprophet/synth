//! Invertebrate Exoskeleton Articulation, Gait Stepper & Cuticle Optics (`arthropod_articulation.rs`).
//!
//! Provides zero-heap locomotion, exoskeleton joint limit enforcement, and structural
//! cuticle iridescence modeling for insects, arachnids, myriapods, and crustaceans:
//! - [`ArthropodBodyPlan`]: hexapod, arachnid, myriapod, decapod body configurations.
//! - [`ExoskeletonSegment`]: morphological exoskeleton segment classes.
//! - [`ExoskeletonJointLimit`]: hard hinge and ball-socket kinematic constraints.
//! - [`evaluate_arthropod_gait_zero_heap`]: zero-heap tripod, tetrapod, and metachronal gait stepper.
//! - [`CuticleOpticalProfile`]: multilayer thin-film interference, melanin absorption, and micro-setae optics.

use serde::{Deserialize, Serialize};

/// Arthropod body plan taxonomy defining appendage count and gait coupling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ArthropodBodyPlan {
    Hexapod6Leg,
    Arachnid8Leg,
    MyriapodSegmented,
    CrustaceanDecapod,
}

impl ArthropodBodyPlan {
    /// Number of articulating walking appendages for this body plan.
    #[inline]
    pub const fn leg_count(&self) -> usize {
        match self {
            Self::Hexapod6Leg => 6,
            Self::Arachnid8Leg => 8,
            Self::MyriapodSegmented => 16,
            Self::CrustaceanDecapod => 10,
        }
    }
}

/// Exoskeleton morphological anatomical segment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ExoskeletonSegment {
    Coxa,
    Trochanter,
    Femur,
    Tibia,
    Tarsus,
    Chela,
    ThoraxCarapace,
    AbdomenTergite,
    WingFore,
    WingHind,
}

/// Articulating joint classification for exoskeleton sclerite connections.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ExoskeletonJointKind {
    Hinge,
    BallSocket,
    Saddle,
}

/// Hard mechanical constraint on an exoskeleton joint degree of freedom.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ExoskeletonJointLimit {
    pub kind: ExoskeletonJointKind,
    pub axis: [f32; 3],
    pub min_angle_rad: f32,
    pub max_angle_rad: f32,
}

impl ExoskeletonJointLimit {
    /// Enforces joint rotation limits, clamping or failing closed on NaN / invalid range.
    pub fn enforce_limit(&self, angle_rad: f32) -> Result<f32, ArthropodGaitError> {
        if !angle_rad.is_finite() || self.min_angle_rad > self.max_angle_rad {
            return Err(ArthropodGaitError::InvalidJointAngle { angle_rad });
        }
        if angle_rad < self.min_angle_rad || angle_rad > self.max_angle_rad {
            return Err(ArthropodGaitError::JointLimitExceeded {
                angle_rad,
                min: self.min_angle_rad,
                max: self.max_angle_rad,
            });
        }
        Ok(angle_rad)
    }

    /// Clamps an angle within the permitted joint limits.
    #[inline]
    pub fn clamp_limit(&self, angle_rad: f32) -> f32 {
        if !angle_rad.is_finite() {
            return (self.min_angle_rad + self.max_angle_rad) * 0.5;
        }
        angle_rad.clamp(self.min_angle_rad, self.max_angle_rad)
    }
}

/// Fail-closed error taxonomy for arthropod locomotion and articulation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ArthropodGaitError {
    BufferTooSmall { required: usize, provided: usize },
    InvalidGaitCycle { cycle: f32 },
    InvalidStrideParameters,
    InvalidJointAngle { angle_rad: f32 },
    JointLimitExceeded { angle_rad: f32, min: f32, max: f32 },
}

/// Evaluates multi-leg phase-coupled gait stepping with zero heap allocations.
pub fn evaluate_arthropod_gait_zero_heap(
    body_plan: ArthropodBodyPlan,
    gait_cycle: f32,
    stride_length: f32,
    step_height: f32,
    leg_phase_offsets: &mut [f32],
    leg_tip_positions: &mut [[f32; 3]],
    leg_is_stance: &mut [bool],
) -> Result<usize, ArthropodGaitError> {
    let leg_count = body_plan.leg_count();
    if leg_phase_offsets.len() < leg_count
        || leg_tip_positions.len() < leg_count
        || leg_is_stance.len() < leg_count
    {
        return Err(ArthropodGaitError::BufferTooSmall {
            required: leg_count,
            provided: leg_tip_positions.len(),
        });
    }
    if !gait_cycle.is_finite() || gait_cycle < 0.0 {
        return Err(ArthropodGaitError::InvalidGaitCycle { cycle: gait_cycle });
    }
    if !stride_length.is_finite() || stride_length <= 0.0 || !step_height.is_finite() || step_height < 0.0 {
        return Err(ArthropodGaitError::InvalidStrideParameters);
    }

    let norm_cycle = gait_cycle.fract();

    // Duty factor: fraction of gait cycle spent in ground contact (stance)
    let (duty_factor, phase_step) = match body_plan {
        ArthropodBodyPlan::Hexapod6Leg => (0.50f32, 0.50f32),
        ArthropodBodyPlan::Arachnid8Leg => (0.60f32, 0.25f32),
        ArthropodBodyPlan::MyriapodSegmented => (0.70f32, 0.0625f32),
        ArthropodBodyPlan::CrustaceanDecapod => (0.65f32, 0.20f32),
    };

    for i in 0..leg_count {
        let is_left = (i % 2) == 0;
        let segment_idx = i / 2;

        // Compute phase offset per body plan:
        let offset = match body_plan {
            ArthropodBodyPlan::Hexapod6Leg => {
                // Tripod gait: L1, R2, L3 (group 0) vs R1, L2, R3 (group 1)
                let group = if is_left { segment_idx % 2 } else { (segment_idx + 1) % 2 };
                group as f32 * phase_step
            }
            ArthropodBodyPlan::Arachnid8Leg => {
                // Alternating tetrapod gait
                let tetrapod = if is_left { segment_idx % 2 } else { (segment_idx + 1) % 2 };
                tetrapod as f32 * 0.5 + (segment_idx as f32 * 0.125)
            }
            ArthropodBodyPlan::MyriapodSegmented => {
                // Metachronal travelling wave
                (segment_idx as f32 * phase_step) % 1.0
            }
            ArthropodBodyPlan::CrustaceanDecapod => {
                (segment_idx as f32 * phase_step + if is_left { 0.0 } else { 0.5 }) % 1.0
            }
        };

        leg_phase_offsets[i] = offset;
        let local_phase = (norm_cycle + offset).fract();

        // Anatomical attachment offsets relative to body center:
        let side_sign = if is_left { -1.0f32 } else { 1.0f32 };
        let base_x = side_sign * (0.35 + (segment_idx as f32) * 0.05);
        let base_z = 0.5 - (segment_idx as f32) * (1.0 / (leg_count / 2).max(1) as f32);

        if local_phase < duty_factor {
            // STANCE PHASE: foot planted on ground, moves backward relative to thorax
            leg_is_stance[i] = true;
            let stance_prog = local_phase / duty_factor;
            let z_disp = stride_length * (0.5 - stance_prog);
            leg_tip_positions[i] = [base_x, 0.0, base_z + z_disp];
        } else {
            // SWING PHASE: foot lifted off ground, moves forward
            leg_is_stance[i] = false;
            let swing_prog = (local_phase - duty_factor) / (1.0 - duty_factor);
            let z_disp = stride_length * (-0.5 + swing_prog);
            let y_lift = step_height * (core::f32::consts::PI * swing_prog).sin();
            leg_tip_positions[i] = [base_x, y_lift, base_z + z_disp];
        }
    }

    Ok(leg_count)
}

/// Structural cuticle optical profile governing iridescence and micro-setae light scattering.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CuticleOpticalProfile {
    /// Chitin epicuticle thin-film thickness in nanometers (typically 120 - 750 nm).
    pub thin_film_thickness_nm: f32,
    /// Refractive index of chitin matrix (typically 1.50 - 1.62).
    pub chitin_ior: f32,
    /// Melanin absorption coefficient in underlying exocuticle [0.0, 1.0].
    pub melanin_absorption: f32,
    /// Micro-setae / structural roughness scattering [0.0, 1.0].
    pub micro_setae_roughness: f32,
}

/// Error taxonomy for cuticle optical parameters.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CuticleOpticalError {
    InvalidThickness,
    InvalidIor,
    InvalidAbsorption,
    InvalidRoughness,
}

impl CuticleOpticalProfile {
    /// Validates physical plausibility of optical cuticle properties.
    pub fn validate(&self) -> Result<(), CuticleOpticalError> {
        if !self.thin_film_thickness_nm.is_finite() || self.thin_film_thickness_nm <= 0.0 {
            return Err(CuticleOpticalError::InvalidThickness);
        }
        if !self.chitin_ior.is_finite() || self.chitin_ior < 1.0 || self.chitin_ior > 3.0 {
            return Err(CuticleOpticalError::InvalidIor);
        }
        if !self.melanin_absorption.is_finite() || self.melanin_absorption < 0.0 || self.melanin_absorption > 1.0 {
            return Err(CuticleOpticalError::InvalidAbsorption);
        }
        if !self.micro_setae_roughness.is_finite() || self.micro_setae_roughness < 0.0 || self.micro_setae_roughness > 1.0 {
            return Err(CuticleOpticalError::InvalidRoughness);
        }
        Ok(())
    }

    /// Evaluates thin-film interference reflectance $R \in [0.0, 1.0]$ at a specific wavelength.
    pub fn evaluate_thin_film_reflectance(&self, wavelength_nm: f32, cos_theta: f32) -> f32 {
        let n = self.chitin_ior;
        let d = self.thin_film_thickness_nm;
        let cos_t = cos_theta.clamp(0.0, 1.0);

        // Refracted angle in film using Snell's law: n1 sin(theta1) = n2 sin(theta2)
        let sin2_t = 1.0 - cos_t * cos_t;
        let cos2_refr = (1.0 - sin2_t / (n * n)).max(0.001);
        let cos_refr = cos2_refr.sqrt();

        // Optical Path Difference: OPD = 2 * n * d * cos(theta_refr)
        let opd = 2.0 * n * d * cos_refr;
        let phase_delta = 2.0 * core::f32::consts::PI * opd / wavelength_nm.max(100.0);

        // Normal-incidence Fresnel amplitude reflection at air-chitin boundary:
        let r0 = (n - 1.0) / (n + 1.0);
        let f_coeff = (4.0 * r0 * r0) / ((1.0 - r0 * r0) * (1.0 - r0 * r0));

        let sin_half_delta = (phase_delta * 0.5).sin();
        let sin2_half = sin_half_delta * sin_half_delta;
        let base_refl = (f_coeff * sin2_half) / (1.0 + f_coeff * sin2_half);

        // Melanin in exocuticle acts as a dark absorber backing, saturating iridescence:
        let dark_backing_contrast = 1.0 - 0.5 * (1.0 - self.melanin_absorption);
        let effective_refl = base_refl * dark_backing_contrast + 0.04 * (1.0 - self.melanin_absorption);

        // Micro-setae scatter the coherent phase, reducing peak specular sharpness:
        let roughness_damping = 1.0 - 0.45 * self.micro_setae_roughness;
        (effective_refl * roughness_damping).clamp(0.0, 1.0)
    }

    /// Computes spectral RGB iridescence values at standard wavelengths: Red (650nm), Green (532nm), Blue (450nm).
    pub fn spectral_iridescence_rgb(&self, cos_theta: f32) -> [f32; 3] {
        [
            self.evaluate_thin_film_reflectance(650.0, cos_theta),
            self.evaluate_thin_film_reflectance(532.0, cos_theta),
            self.evaluate_thin_film_reflectance(450.0, cos_theta),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hexapod_tripod_gait_evaluation() {
        let mut offsets = [0.0; 6];
        let mut tips = [[0.0; 3]; 6];
        let mut stance = [false; 6];

        // At cycle 0.0: Group 0 (L1, R2, L3) stance, Group 1 (R1, L2, R3) swing
        let count = evaluate_arthropod_gait_zero_heap(
            ArthropodBodyPlan::Hexapod6Leg,
            0.0,
            0.20,
            0.05,
            &mut offsets,
            &mut tips,
            &mut stance,
        )
        .unwrap();

        assert_eq!(count, 6);
        // L1 (idx 0), R2 (idx 3), L3 (idx 4) should be in stance:
        assert!(stance[0], "L1 should be in stance at cycle 0.0");
        assert!(stance[3], "R2 should be in stance at cycle 0.0");
        assert!(stance[4], "L3 should be in stance at cycle 0.0");

        // R1 (idx 1), L2 (idx 2), R3 (idx 5) should be in swing (lifted):
        assert!(!stance[1], "R1 should be in swing at cycle 0.0");
        assert!(!stance[2], "L2 should be in swing at cycle 0.0");
        assert!(!stance[5], "R3 should be in swing at cycle 0.0");
        assert!(tips[1][1] >= 0.0, "Swing leg should have non-negative height");

        // At cycle 0.5: Roles reverse
        evaluate_arthropod_gait_zero_heap(
            ArthropodBodyPlan::Hexapod6Leg,
            0.5,
            0.20,
            0.05,
            &mut offsets,
            &mut tips,
            &mut stance,
        )
        .unwrap();

        assert!(!stance[0], "L1 should be in swing at cycle 0.5");
        assert!(stance[1], "R1 should be in stance at cycle 0.5");
    }

    #[test]
    fn test_arachnid_8leg_gait_evaluation() {
        let mut offsets = [0.0; 8];
        let mut tips = [[0.0; 3]; 8];
        let mut stance = [false; 8];

        let count = evaluate_arthropod_gait_zero_heap(
            ArthropodBodyPlan::Arachnid8Leg,
            0.25,
            0.15,
            0.04,
            &mut offsets,
            &mut tips,
            &mut stance,
        )
        .unwrap();

        assert_eq!(count, 8);
        for i in 0..8 {
            assert!(tips[i].iter().all(|v| v.is_finite()));
        }
    }

    #[test]
    fn test_cuticle_optical_profile_validation_and_rgb() {
        let profile = CuticleOpticalProfile {
            thin_film_thickness_nm: 360.0,
            chitin_ior: 1.56,
            melanin_absorption: 0.85,
            micro_setae_roughness: 0.15,
        };

        profile.validate().unwrap();

        let rgb_normal = profile.spectral_iridescence_rgb(1.0);
        let rgb_glancing = profile.spectral_iridescence_rgb(0.5);

        for c in 0..3 {
            assert!(rgb_normal[c] >= 0.0 && rgb_normal[c] <= 1.0);
            assert!(rgb_glancing[c] >= 0.0 && rgb_glancing[c] <= 1.0);
        }
        // Thin film interference shifts with angle:
        assert!(rgb_normal != rgb_glancing);
    }

    #[test]
    fn test_gait_buffer_too_small_fails_closed() {
        let mut offsets = [0.0; 4];
        let mut tips = [[0.0; 3]; 4];
        let mut stance = [false; 4];

        let err = evaluate_arthropod_gait_zero_heap(
            ArthropodBodyPlan::Hexapod6Leg,
            0.0,
            0.1,
            0.02,
            &mut offsets,
            &mut tips,
            &mut stance,
        )
        .unwrap_err();

        assert_eq!(err, ArthropodGaitError::BufferTooSmall { required: 6, provided: 4 });
    }

    #[test]
    fn test_exoskeleton_joint_limit_enforce() {
        let limit = ExoskeletonJointLimit {
            kind: ExoskeletonJointKind::Hinge,
            axis: [1.0, 0.0, 0.0],
            min_angle_rad: -0.2,
            max_angle_rad: 1.4,
        };

        assert_eq!(limit.enforce_limit(0.5).unwrap(), 0.5);
        assert!(limit.enforce_limit(1.8).is_err());
        assert_eq!(limit.clamp_limit(1.8), 1.4);
        assert_eq!(limit.clamp_limit(-0.5), -0.2);
    }

    #[test]
    fn test_cuticle_invalid_parameters_fail_closed() {
        let bad_thick = CuticleOpticalProfile {
            thin_film_thickness_nm: -50.0,
            chitin_ior: 1.5,
            melanin_absorption: 0.5,
            micro_setae_roughness: 0.1,
        };
        assert_eq!(bad_thick.validate(), Err(CuticleOpticalError::InvalidThickness));

        let bad_ior = CuticleOpticalProfile {
            thin_film_thickness_nm: 200.0,
            chitin_ior: 0.8,
            melanin_absorption: 0.5,
            micro_setae_roughness: 0.1,
        };
        assert_eq!(bad_ior.validate(), Err(CuticleOpticalError::InvalidIor));
    }
}
