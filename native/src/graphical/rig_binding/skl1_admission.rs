//! SKL1 Skeletal Rig Admission, Hierarchy Validation, Joint Limits & Morph Targets (`skl1_admission.rs`).
//!
//! Provides fail-closed cold admission and validation for vertebrate and invertebrate
//! SKL1 skeletal rigs:
//! - [`Skl1HierarchyValidator`]: cyclic parent reference and out-of-range bone index rejection.
//! - [`JointLimitSpec`]: anatomical flexion, extension, abduction, and twist range enforcement.
//! - [`SkinWeight4`]: 4-joint normalized skin weight verification and checked normalization.
//! - [`MorphTargetDescriptor`]: categorical morph target displacement mapping.
//! - [`AdmittedSkl1Rig`]: sealed, validated skeletal rig representation.

use serde::{Deserialize, Serialize};

/// Maximum joints supported in a single SKL1 skeletal admission.
pub const MAX_SKL1_JOINTS: usize = 256;

/// Categorical anatomical morph target classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MorphCategory {
    MuscleBulge,
    ThoracicBreathing,
    VocalSacInflation,
    GillOperculum,
    JawGape,
    FeatherCrestErection,
}

/// Morph target displacement descriptor for facial, respiratory, and muscle deformation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MorphTargetDescriptor {
    pub name_hash: u64,
    pub category: MorphCategory,
    pub max_displacement_mm: f32,
    pub vertex_deltas: Vec<(u32, [f32; 3])>,
}

/// Enforcement policy for joint limits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LimitEnforcementMode {
    Refuse,
    Clamp,
}

/// Anatomical joint rotation limits in radians per rotational degree of freedom.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct JointLimitSpec {
    pub flexion: [f32; 2],
    pub extension: [f32; 2],
    pub abduction: [f32; 2],
    pub twist: [f32; 2],
}

impl JointLimitSpec {
    #[inline]
    fn enforce_single_axis(
        val: f32,
        limits: [f32; 2],
        axis_name: &'static str,
        joint_index: usize,
        mode: LimitEnforcementMode,
    ) -> Result<f32, Skl1AdmissionError> {
        if !val.is_finite() || limits[0] > limits[1] {
            return Err(Skl1AdmissionError::JointLimitExceeded {
                joint_index,
                axis: axis_name,
                value: val,
                min: limits[0],
                max: limits[1],
            });
        }
        if val < limits[0] || val > limits[1] {
            match mode {
                LimitEnforcementMode::Refuse => Err(Skl1AdmissionError::JointLimitExceeded {
                    joint_index,
                    axis: axis_name,
                    value: val,
                    min: limits[0],
                    max: limits[1],
                }),
                LimitEnforcementMode::Clamp => Ok(val.clamp(limits[0], limits[1])),
            }
        } else {
            Ok(val)
        }
    }

    /// Enforces joint rotation limits across flexion/extension, abduction, and twist axes.
    pub fn enforce_rotation(
        &self,
        joint_index: usize,
        angles_rad: [f32; 3], // [pitch (flex/ext), yaw (abduction), roll (twist)]
        mode: LimitEnforcementMode,
    ) -> Result<[f32; 3], Skl1AdmissionError> {
        let flex_ext_limits = if angles_rad[0] >= 0.0 {
            self.flexion
        } else {
            self.extension
        };
        let p = Self::enforce_single_axis(angles_rad[0], flex_ext_limits, "flexion/extension", joint_index, mode)?;
        let y = Self::enforce_single_axis(angles_rad[1], self.abduction, "abduction", joint_index, mode)?;
        let r = Self::enforce_single_axis(angles_rad[2], self.twist, "twist", joint_index, mode)?;
        Ok([p, y, r])
    }
}

/// 4-joint skin binding weight record for a single vertex.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SkinWeight4 {
    pub joint_indices: [u16; 4],
    pub weights: [f32; 4],
}

/// Fail-closed error taxonomy for SKL1 rig admission and validation.
#[derive(Debug, Clone, PartialEq)]
pub enum Skl1AdmissionError {
    CyclicBoneHierarchy { joint_index: u16 },
    OutOfRangeBoneIndex { joint_index: u16, joint_count: usize },
    UnnormalizedSkinWeights { vertex_index: usize, sum: f32 },
    InvalidWeightComponent { vertex_index: usize, component: usize },
    ZeroWeightError,
    JointLimitExceeded {
        joint_index: usize,
        axis: &'static str,
        value: f32,
        min: f32,
        max: f32,
    },
    EmptyHierarchy,
    TooManyJoints { count: usize, max: usize },
    InvalidMorphDelta { target_name_hash: u64, vertex_index: u32 },
}

/// Validator for SKL1 bone hierarchies.
pub struct Skl1HierarchyValidator;

impl Skl1HierarchyValidator {
    /// Validates bone parent pointers, rejecting cycles, self-references, and out-of-range indices.
    pub fn validate_hierarchy(parents: &[Option<u16>]) -> Result<(), Skl1AdmissionError> {
        let count = parents.len();
        if count == 0 {
            return Err(Skl1AdmissionError::EmptyHierarchy);
        }
        if count > MAX_SKL1_JOINTS {
            return Err(Skl1AdmissionError::TooManyJoints { count, max: MAX_SKL1_JOINTS });
        }

        for (idx, &parent_opt) in parents.iter().enumerate() {
            let mut curr = idx as u16;
            let mut visited = [false; MAX_SKL1_JOINTS];
            visited[curr as usize] = true;

            let mut next = parent_opt;
            while let Some(parent) = next {
                if (parent as usize) >= count {
                    return Err(Skl1AdmissionError::OutOfRangeBoneIndex {
                        joint_index: parent,
                        joint_count: count,
                    });
                }
                if visited[parent as usize] {
                    return Err(Skl1AdmissionError::CyclicBoneHierarchy {
                        joint_index: parent,
                    });
                }
                visited[parent as usize] = true;
                curr = parent;
                next = parents[curr as usize];
            }
        }
        Ok(())
    }
}

/// Validates that all skin weights are non-negative, finite, indexed within bounds,
/// and sum to 1.0 within epsilon [1.0 - 1e-3, 1.0 + 1e-3].
pub fn validate_skin_weights(weights: &[SkinWeight4], joint_count: usize) -> Result<(), Skl1AdmissionError> {
    for (v_idx, sw) in weights.iter().enumerate() {
        let mut sum = 0.0f32;
        for c in 0..4 {
            let w = sw.weights[c];
            if !w.is_finite() || w < 0.0 {
                return Err(Skl1AdmissionError::InvalidWeightComponent {
                    vertex_index: v_idx,
                    component: c,
                });
            }
            if (sw.joint_indices[c] as usize) >= joint_count {
                return Err(Skl1AdmissionError::OutOfRangeBoneIndex {
                    joint_index: sw.joint_indices[c],
                    joint_count,
                });
            }
            sum += w;
        }
        if (sum - 1.0).abs() > 1e-3 {
            return Err(Skl1AdmissionError::UnnormalizedSkinWeights {
                vertex_index: v_idx,
                sum,
            });
        }
    }
    Ok(())
}

/// Normalizes 4-joint skin weights, failing closed on zero weights, NaNs, or out-of-range indices.
pub fn normalize_skin_weights_checked(
    joint_indices: [u16; 4],
    raw_weights: [f32; 4],
    joint_count: usize,
) -> Result<SkinWeight4, Skl1AdmissionError> {
    let mut sum = 0.0f32;
    for c in 0..4 {
        let w = raw_weights[c];
        if !w.is_finite() || w < 0.0 {
            return Err(Skl1AdmissionError::InvalidWeightComponent {
                vertex_index: 0,
                component: c,
            });
        }
        if (joint_indices[c] as usize) >= joint_count {
            return Err(Skl1AdmissionError::OutOfRangeBoneIndex {
                joint_index: joint_indices[c],
                joint_count,
            });
        }
        sum += w;
    }
    if sum < 1e-6 {
        return Err(Skl1AdmissionError::ZeroWeightError);
    }
    let inv_sum = 1.0 / sum;
    Ok(SkinWeight4 {
        joint_indices,
        weights: [
            raw_weights[0] * inv_sum,
            raw_weights[1] * inv_sum,
            raw_weights[2] * inv_sum,
            raw_weights[3] * inv_sum,
        ],
    })
}

/// Fully admitted, verified SKL1 skeletal rig with skinning and morph capabilities.
#[derive(Debug, Clone, PartialEq)]
pub struct AdmittedSkl1Rig {
    pub joint_count: usize,
    pub parents: Vec<Option<u16>>,
    pub limits: Vec<Option<JointLimitSpec>>,
    pub skin_weights: Vec<SkinWeight4>,
    pub morph_targets: Vec<MorphTargetDescriptor>,
}

impl AdmittedSkl1Rig {
    /// Validates and admits an SKL1 rig.
    pub fn admit(
        parents: Vec<Option<u16>>,
        limits: Vec<Option<JointLimitSpec>>,
        skin_weights: Vec<SkinWeight4>,
        morph_targets: Vec<MorphTargetDescriptor>,
        vertex_count: usize,
    ) -> Result<Self, Skl1AdmissionError> {
        let joint_count = parents.len();
        Skl1HierarchyValidator::validate_hierarchy(&parents)?;
        validate_skin_weights(&skin_weights, joint_count)?;

        for target in &morph_targets {
            for &(v_idx, delta) in &target.vertex_deltas {
                if (v_idx as usize) >= vertex_count || delta.iter().any(|d| !d.is_finite()) {
                    return Err(Skl1AdmissionError::InvalidMorphDelta {
                        target_name_hash: target.name_hash,
                        vertex_index: v_idx,
                    });
                }
            }
        }

        Ok(Self {
            joint_count,
            parents,
            limits,
            skin_weights,
            morph_targets,
        })
    }

    /// Applies morph target vertex displacements scaled by weight to `positions`.
    pub fn apply_morph_target(
        &self,
        target_index: usize,
        weight: f32,
        positions: &mut [[f32; 3]],
    ) -> Result<(), Skl1AdmissionError> {
        let target = self
            .morph_targets
            .get(target_index)
            .ok_or(Skl1AdmissionError::InvalidMorphDelta {
                target_name_hash: 0,
                vertex_index: 0,
            })?;
        let clamped_weight = weight.clamp(0.0, 1.0);
        for &(v_idx, delta) in &target.vertex_deltas {
            if let Some(pos) = positions.get_mut(v_idx as usize) {
                pos[0] += delta[0] * clamped_weight;
                pos[1] += delta[1] * clamped_weight;
                pos[2] += delta[2] * clamped_weight;
            } else {
                return Err(Skl1AdmissionError::InvalidMorphDelta {
                    target_name_hash: target.name_hash,
                    vertex_index: v_idx,
                });
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_skl1_admission() {
        let parents = vec![None, Some(0), Some(1)];
        let limits = vec![
            None,
            Some(JointLimitSpec {
                flexion: [0.0, 1.57],
                extension: [-0.5, 0.0],
                abduction: [-0.3, 0.3],
                twist: [-0.2, 0.2],
            }),
            None,
        ];
        let skin_weights = vec![
            SkinWeight4 { joint_indices: [0, 1, 0, 0], weights: [0.7, 0.3, 0.0, 0.0] },
            SkinWeight4 { joint_indices: [1, 2, 0, 0], weights: [0.5, 0.5, 0.0, 0.0] },
        ];
        let morph_targets = vec![MorphTargetDescriptor {
            name_hash: 0x1234,
            category: MorphCategory::MuscleBulge,
            max_displacement_mm: 15.0,
            vertex_deltas: vec![(0, [0.0, 2.0, 0.0])],
        }];

        let admitted = AdmittedSkl1Rig::admit(parents, limits, skin_weights, morph_targets, 2).unwrap();
        assert_eq!(admitted.joint_count, 3);

        let mut positions = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]];
        admitted.apply_morph_target(0, 0.5, &mut positions).unwrap();
        assert_eq!(positions[0], [0.0, 1.0, 0.0]);
    }

    #[test]
    fn test_cyclic_bone_hierarchy_rejected() {
        // Direct cycle: 0 -> 1 -> 0
        let parents = vec![Some(1), Some(0)];
        let err = Skl1HierarchyValidator::validate_hierarchy(&parents).unwrap_err();
        assert!(matches!(err, Skl1AdmissionError::CyclicBoneHierarchy { .. }));

        // Self-cycle: 0 -> 0
        let parents_self = vec![Some(0)];
        let err_self = Skl1HierarchyValidator::validate_hierarchy(&parents_self).unwrap_err();
        assert!(matches!(err_self, Skl1AdmissionError::CyclicBoneHierarchy { joint_index: 0 }));
    }

    #[test]
    fn test_out_of_range_bone_index_rejected() {
        let parents = vec![None, Some(5)];
        let err = Skl1HierarchyValidator::validate_hierarchy(&parents).unwrap_err();
        assert_eq!(err, Skl1AdmissionError::OutOfRangeBoneIndex { joint_index: 5, joint_count: 2 });
    }

    #[test]
    fn test_unnormalized_skin_weights_rejected() {
        let sw_under = [SkinWeight4 { joint_indices: [0, 0, 0, 0], weights: [0.4, 0.0, 0.0, 0.0] }];
        assert!(matches!(
            validate_skin_weights(&sw_under, 1).unwrap_err(),
            Skl1AdmissionError::UnnormalizedSkinWeights { .. }
        ));

        let sw_nan = [SkinWeight4 { joint_indices: [0, 0, 0, 0], weights: [f32::NAN, 0.0, 0.0, 0.0] }];
        assert!(matches!(
            validate_skin_weights(&sw_nan, 1).unwrap_err(),
            Skl1AdmissionError::InvalidWeightComponent { .. }
        ));
    }

    #[test]
    fn test_normalize_skin_weights_checked() {
        let normalized = normalize_skin_weights_checked([0, 1, 0, 0], [2.0, 2.0, 0.0, 0.0], 2).unwrap();
        assert_eq!(normalized.weights, [0.5, 0.5, 0.0, 0.0]);

        let zero_err = normalize_skin_weights_checked([0, 0, 0, 0], [0.0, 0.0, 0.0, 0.0], 1).unwrap_err();
        assert_eq!(zero_err, Skl1AdmissionError::ZeroWeightError);
    }

    #[test]
    fn test_joint_limits_refuse_and_clamp() {
        let spec = JointLimitSpec {
            flexion: [0.0, 1.0],
            extension: [-1.0, 0.0],
            abduction: [-0.5, 0.5],
            twist: [-0.2, 0.2],
        };
        // Exceeding flexion in Refuse mode:
        assert!(spec.enforce_rotation(1, [1.5, 0.0, 0.0], LimitEnforcementMode::Refuse).is_err());

        // Exceeding flexion in Clamp mode:
        let clamped = spec.enforce_rotation(1, [1.5, 0.8, -0.4], LimitEnforcementMode::Clamp).unwrap();
        assert_eq!(clamped, [1.0, 0.5, -0.2]);
    }
}
