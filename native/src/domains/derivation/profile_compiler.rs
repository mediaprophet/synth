//! Native Master-to-Profile derivation compiler (AR-05, ENG-11).
//!
//! Compiles high-grade Master embodiments into Film, Open-World-Game,
//! and Cartoon delivery profiles, enforcing exact triangle, texture, joint,
//! error metric, and C4 security classification budgets.
//!
//! Invariant: In accordance with project governance (rule 5), software derivation
//! checks never self-certify human_acceptance or domain_review; delivery packages
//! require independent human-principal review prior to final catalog sign-off.

use super::error_metrics::{
    evaluate_derivation_error_metrics, DerivationErrorMetrics, DerivationMetricError,
    MeshSurfaceSample, ProfileErrorThresholds,
};
use std::fmt;

pub const SENSITIVITY_PUBLIC: u8 = 0;
pub const SENSITIVITY_RESTRICTED: u8 = 1;
pub const SENSITIVITY_CLASSIFIED: u8 = 2;

/// Delivery target profiles for derived packages (AR-05).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DeliveryTargetProfile {
    Film,
    OpenWorldGame,
    Cartoon,
}

impl fmt::Display for DeliveryTargetProfile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Film => write!(f, "Film"),
            Self::OpenWorldGame => write!(f, "OpenWorldGame"),
            Self::Cartoon => write!(f, "Cartoon"),
        }
    }
}

/// Visual style presets (kept distinct from delivery target and LOD).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum StylePreset {
    PhotorealisticSpectral,
    StylizedCelToon,
    BotanicalIllustration,
}

impl fmt::Display for StylePreset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PhotorealisticSpectral => write!(f, "PhotorealisticSpectral"),
            Self::StylizedCelToon => write!(f, "StylizedCelToon"),
            Self::BotanicalIllustration => write!(f, "BotanicalIllustration"),
        }
    }
}

/// High-grade master embodiment specification.
#[derive(Debug, Clone, PartialEq)]
pub struct MasterEmbodimentSpec {
    pub master_id: String,
    pub master_sha256: [u8; 32],
    pub q42_did: u64,
    pub sensitivity_class: u8,
    pub surface_samples: Vec<MeshSurfaceSample>,
    pub critical_organ_ids: Vec<u64>,
    pub vertex_count: usize,
    pub triangle_count: usize,
    pub texture_bytes: usize,
    pub joint_count: usize,
    pub biological_state: String,
    pub simulation_fidelity: String,
    pub style: StylePreset,
}

/// Target profile budget configuration (AR-05, ENG-11).
#[derive(Debug, Clone, PartialEq)]
pub struct TargetProfileBudget {
    pub delivery_target: DeliveryTargetProfile,
    pub max_triangles: usize,
    pub max_texture_bytes: usize,
    pub max_joints: usize,
    pub target_lod: u8,
    pub error_thresholds: ProfileErrorThresholds,
    pub public_distribution: bool,
    pub declassification_grant_id: Option<u64>,
}

impl TargetProfileBudget {
    pub fn film_reference() -> Self {
        Self {
            delivery_target: DeliveryTargetProfile::Film, max_triangles: 2_000_000,
            max_texture_bytes: 256 * 1024 * 1024, max_joints: 512, target_lod: 0,
            error_thresholds: ProfileErrorThresholds::film(), public_distribution: false,
            declassification_grant_id: None,
        }
    }

    pub fn open_world_game_reference() -> Self {
        Self {
            delivery_target: DeliveryTargetProfile::OpenWorldGame, max_triangles: 80_000,
            max_texture_bytes: 32 * 1024 * 1024, max_joints: 96, target_lod: 2,
            error_thresholds: ProfileErrorThresholds::open_world_game(), public_distribution: true,
            declassification_grant_id: None,
        }
    }

    pub fn cartoon_reference() -> Self {
        Self {
            delivery_target: DeliveryTargetProfile::Cartoon, max_triangles: 30_000,
            max_texture_bytes: 16 * 1024 * 1024, max_joints: 48, target_lod: 3,
            error_thresholds: ProfileErrorThresholds::cartoon(), public_distribution: true,
            declassification_grant_id: None,
        }
    }
}

/// Candidate geometry and attributes for derivation.
#[derive(Debug, Clone, PartialEq)]
pub struct DerivedGeometryCandidate {
    pub candidate_id: String,
    pub style: StylePreset,
    pub lod_tier: u8,
    pub biological_state: String,
    pub simulation_fidelity: String,
    pub surface_samples: Vec<MeshSurfaceSample>,
    pub vertex_count: usize,
    pub triangle_count: usize,
    pub texture_bytes: usize,
    pub joint_count: usize,
}

/// A fully compiled and validated derived profile package.
#[derive(Debug, Clone, PartialEq)]
pub struct DerivedProfilePackage {
    pub package_id: String,
    pub delivery_target: DeliveryTargetProfile,
    pub style: StylePreset,
    pub lod_tier: u8,
    pub biological_state: String,
    pub simulation_fidelity: String,
    pub vertex_count: usize,
    pub triangle_count: usize,
    pub texture_bytes: usize,
    pub joint_count: usize,
    pub derives_from_master_sha256: [u8; 32],
    pub derives_from_q42_did: u64,
    pub lineage_quin_hash: u64,
    pub metrics: DerivationErrorMetrics,
    pub declassification_grant_id: Option<u64>,
}

/// Bundle of all three mandatory delivery profile packages (Film, OpenWorldGame, Cartoon).
#[derive(Debug, Clone, PartialEq)]
pub struct MandatoryProfileBundle {
    pub film: DerivedProfilePackage,
    pub open_world_game: DerivedProfilePackage,
    pub cartoon: DerivedProfilePackage,
}

/// Errors returned during profile compilation.
#[derive(Debug, Clone, PartialEq)]
pub enum ProfileCompileError {
    BudgetExceededTriangles { target: DeliveryTargetProfile, actual: usize, max: usize },
    BudgetExceededTextureBytes { target: DeliveryTargetProfile, actual: usize, max: usize },
    BudgetExceededJoints { target: DeliveryTargetProfile, actual: usize, max: usize },
    MetricFailure(DerivationMetricError),
    RestrictedSourceBlockedFromPublicDerivation { sensitivity_class: u8 },
}

impl fmt::Display for ProfileCompileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BudgetExceededTriangles { target, actual, max } => write!(f, "{target} tris {actual} > max {max}"),
            Self::BudgetExceededTextureBytes { target, actual, max } => write!(f, "{target} texture bytes {actual} > max {max}"),
            Self::BudgetExceededJoints { target, actual, max } => write!(f, "{target} joints {actual} > max {max}"),
            Self::MetricFailure(err) => write!(f, "metric failure: {err}"),
            Self::RestrictedSourceBlockedFromPublicDerivation { sensitivity_class } => {
                write!(f, "C4 violation: sensitivity class {sensitivity_class} blocked from public derivation without grant")
            }
        }
    }
}

impl std::error::Error for ProfileCompileError {}

impl From<DerivationMetricError> for ProfileCompileError {
    fn from(err: DerivationMetricError) -> Self {
        Self::MetricFailure(err)
    }
}

/// Computes the 64-bit deterministic lineage quin hash for the derivation.
pub fn compute_lineage_quin_hash(
    master_q42_did: u64,
    target: DeliveryTargetProfile,
    lod_tier: u8,
    master_sha256: &[u8; 32],
) -> u64 {
    let mut h = 0xcbf29ce484222325u64;
    for &b in &master_q42_did.to_le_bytes() { h = (h ^ (b as u64)).wrapping_mul(0x100000001b3); }
    let code = match target { DeliveryTargetProfile::Film => 1u8, DeliveryTargetProfile::OpenWorldGame => 2u8, DeliveryTargetProfile::Cartoon => 3u8 };
    h = (h ^ (code as u64)).wrapping_mul(0x100000001b3);
    h = (h ^ (lod_tier as u64)).wrapping_mul(0x100000001b3);
    for &b in master_sha256 { h = (h ^ (b as u64)).wrapping_mul(0x100000001b3); }
    h
}

/// Compiles a high-grade Master embodiment into a single validated delivery profile package.
pub fn compile_master_to_target_profile(
    master: &MasterEmbodimentSpec,
    candidate: &DerivedGeometryCandidate,
    budget: &TargetProfileBudget,
) -> Result<DerivedProfilePackage, ProfileCompileError> {
    // 1. C4 Sensitivity Gate (ENG-11, IP-03 C4)
    if master.sensitivity_class > SENSITIVITY_PUBLIC && budget.public_distribution && budget.declassification_grant_id.is_none() {
        return Err(ProfileCompileError::RestrictedSourceBlockedFromPublicDerivation {
            sensitivity_class: master.sensitivity_class,
        });
    }

    // 2. Target Profile Budget Gate (AR-05, ENG-11)
    if candidate.triangle_count > budget.max_triangles {
        return Err(ProfileCompileError::BudgetExceededTriangles { target: budget.delivery_target, actual: candidate.triangle_count, max: budget.max_triangles });
    }
    if candidate.texture_bytes > budget.max_texture_bytes {
        return Err(ProfileCompileError::BudgetExceededTextureBytes { target: budget.delivery_target, actual: candidate.texture_bytes, max: budget.max_texture_bytes });
    }
    if candidate.joint_count > budget.max_joints {
        return Err(ProfileCompileError::BudgetExceededJoints { target: budget.delivery_target, actual: candidate.joint_count, max: budget.max_joints });
    }

    // 3. Metric Evaluation and Threshold Verification
    let metrics = evaluate_derivation_error_metrics(
        &master.surface_samples,
        &candidate.surface_samples,
        &master.critical_organ_ids,
        &budget.error_thresholds,
    )?;

    // 4. Lineage Quin Hash Computation
    let lineage_quin_hash = compute_lineage_quin_hash(master.q42_did, budget.delivery_target, candidate.lod_tier, &master.master_sha256);

    Ok(DerivedProfilePackage {
        package_id: candidate.candidate_id.clone(),
        delivery_target: budget.delivery_target,
        style: candidate.style,
        lod_tier: candidate.lod_tier,
        biological_state: candidate.biological_state.clone(),
        simulation_fidelity: candidate.simulation_fidelity.clone(),
        vertex_count: candidate.vertex_count,
        triangle_count: candidate.triangle_count,
        texture_bytes: candidate.texture_bytes,
        joint_count: candidate.joint_count,
        derives_from_master_sha256: master.master_sha256,
        derives_from_q42_did: master.q42_did,
        lineage_quin_hash,
        metrics,
        declassification_grant_id: budget.declassification_grant_id,
    })
}

/// Compiles all three mandatory profiles (Film, Open-World-Game, Cartoon) from an admitted Master.
pub fn compile_all_three_mandatory_profiles(
    master: &MasterEmbodimentSpec,
    film_candidate: &DerivedGeometryCandidate,
    film_budget: &TargetProfileBudget,
    game_candidate: &DerivedGeometryCandidate,
    game_budget: &TargetProfileBudget,
    cartoon_candidate: &DerivedGeometryCandidate,
    cartoon_budget: &TargetProfileBudget,
) -> Result<MandatoryProfileBundle, ProfileCompileError> {
    let film = compile_master_to_target_profile(master, film_candidate, film_budget)?;
    let open_world_game = compile_master_to_target_profile(master, game_candidate, game_budget)?;
    let cartoon = compile_master_to_target_profile(master, cartoon_candidate, cartoon_budget)?;
    Ok(MandatoryProfileBundle { film, open_world_game, cartoon })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_samples(organ_ids: &[u64]) -> Vec<MeshSurfaceSample> {
        organ_ids.iter().enumerate().map(|(i, &oid)| {
            MeshSurfaceSample::new([i as f32 * 5.0, 0.0, 0.0], [0.0, 0.0, 1.0], [0.1 * i as f32, 0.1 * i as f32], oid)
        }).collect()
    }

    fn sample_master() -> MasterEmbodimentSpec {
        MasterEmbodimentSpec {
            master_id: "eucalyptus_regnans_master_v1".to_string(),
            master_sha256: [0x42; 32],
            q42_did: 0x9001_8002_7003_u64,
            sensitivity_class: SENSITIVITY_PUBLIC,
            surface_samples: dummy_samples(&[10, 20, 30]),
            critical_organ_ids: vec![10, 20],
            vertex_count: 500_000,
            triangle_count: 900_000,
            texture_bytes: 128 * 1024 * 1024,
            joint_count: 128,
            biological_state: "AdultFlowering".to_string(),
            simulation_fidelity: "FullFEMRig".to_string(),
            style: StylePreset::PhotorealisticSpectral,
        }
    }

    fn dummy_candidate(id: &str, style: StylePreset, lod: u8, tris: usize, tex_mb: usize, joints: usize) -> DerivedGeometryCandidate {
        DerivedGeometryCandidate {
            candidate_id: id.to_string(), style, lod_tier: lod,
            biological_state: "AdultFlowering".to_string(), simulation_fidelity: "KinematicSkeletal".to_string(),
            surface_samples: dummy_samples(&[10, 20, 30]), vertex_count: tris / 2, triangle_count: tris,
            texture_bytes: tex_mb * 1024 * 1024, joint_count: joints,
        }
    }

    #[test]
    fn test_compile_all_three_mandatory_profiles_success() {
        let master = sample_master();
        let film_cand = dummy_candidate("film_0", StylePreset::PhotorealisticSpectral, 0, 800_000, 128, 128);
        let game_cand = dummy_candidate("game_lod2", StylePreset::BotanicalIllustration, 2, 65_000, 24, 64);
        let cartoon_cand = dummy_candidate("toon_lod3", StylePreset::StylizedCelToon, 3, 22_000, 8, 32);

        let bundle = compile_all_three_mandatory_profiles(
            &master,
            &film_cand, &TargetProfileBudget::film_reference(),
            &game_cand, &TargetProfileBudget::open_world_game_reference(),
            &cartoon_cand, &TargetProfileBudget::cartoon_reference(),
        ).unwrap();

        assert_eq!(bundle.film.delivery_target, DeliveryTargetProfile::Film);
        assert_eq!(bundle.open_world_game.delivery_target, DeliveryTargetProfile::OpenWorldGame);
        assert_eq!(bundle.cartoon.delivery_target, DeliveryTargetProfile::Cartoon);
        assert_eq!(bundle.film.style, StylePreset::PhotorealisticSpectral);
        assert_eq!(bundle.open_world_game.style, StylePreset::BotanicalIllustration);
        assert_eq!(bundle.cartoon.style, StylePreset::StylizedCelToon);
        assert_eq!(bundle.film.lod_tier, 0);
        assert_eq!(bundle.open_world_game.lod_tier, 2);
        assert_eq!(bundle.cartoon.lod_tier, 3);
        assert_eq!(bundle.film.derives_from_master_sha256, master.master_sha256);
        assert_eq!(bundle.open_world_game.derives_from_q42_did, master.q42_did);
        assert_ne!(bundle.cartoon.lineage_quin_hash, 0);
    }

    #[test]
    fn test_budget_overflow_triangles_fails() {
        let master = sample_master();
        let cand = dummy_candidate("overflow_tris", StylePreset::BotanicalIllustration, 2, 95_000, 16, 48);
        let res = compile_master_to_target_profile(&master, &cand, &TargetProfileBudget::open_world_game_reference());
        assert!(matches!(res, Err(ProfileCompileError::BudgetExceededTriangles { .. })));
    }

    #[test]
    fn test_budget_overflow_texture_bytes_fails() {
        let master = sample_master();
        let cand = dummy_candidate("overflow_tex", StylePreset::BotanicalIllustration, 2, 50_000, 64, 48);
        let res = compile_master_to_target_profile(&master, &cand, &TargetProfileBudget::open_world_game_reference());
        assert!(matches!(res, Err(ProfileCompileError::BudgetExceededTextureBytes { .. })));
    }

    #[test]
    fn test_budget_overflow_joints_fails() {
        let master = sample_master();
        let cand = dummy_candidate("overflow_joints", StylePreset::StylizedCelToon, 3, 20_000, 8, 60);
        let res = compile_master_to_target_profile(&master, &cand, &TargetProfileBudget::cartoon_reference());
        assert!(matches!(res, Err(ProfileCompileError::BudgetExceededJoints { .. })));
    }

    #[test]
    fn test_c4_restricted_master_blocked_from_public_game_derivation() {
        let mut master = sample_master();
        master.sensitivity_class = SENSITIVITY_RESTRICTED;
        let cand = dummy_candidate("game_cand", StylePreset::BotanicalIllustration, 2, 50_000, 16, 48);
        let mut budget = TargetProfileBudget::open_world_game_reference();
        budget.public_distribution = true;
        budget.declassification_grant_id = None;
        let res = compile_master_to_target_profile(&master, &cand, &budget);
        assert_eq!(res, Err(ProfileCompileError::RestrictedSourceBlockedFromPublicDerivation { sensitivity_class: 1 }));
    }

    #[test]
    fn test_c4_classified_master_blocked_from_public_derivation() {
        let mut master = sample_master();
        master.sensitivity_class = SENSITIVITY_CLASSIFIED;
        let cand = dummy_candidate("game_cand", StylePreset::BotanicalIllustration, 2, 50_000, 16, 48);
        let mut budget = TargetProfileBudget::open_world_game_reference();
        budget.public_distribution = true;
        budget.declassification_grant_id = None;
        let res = compile_master_to_target_profile(&master, &cand, &budget);
        assert_eq!(res, Err(ProfileCompileError::RestrictedSourceBlockedFromPublicDerivation { sensitivity_class: 2 }));
    }

    #[test]
    fn test_c4_restricted_master_allowed_with_declassification_grant() {
        let mut master = sample_master();
        master.sensitivity_class = SENSITIVITY_RESTRICTED;
        let cand = dummy_candidate("game_cand", StylePreset::BotanicalIllustration, 2, 50_000, 16, 48);
        let mut budget = TargetProfileBudget::open_world_game_reference();
        budget.public_distribution = true;
        budget.declassification_grant_id = Some(0xDEAD_BEEF_u64);
        let res = compile_master_to_target_profile(&master, &cand, &budget);
        assert!(res.is_ok());
        assert_eq!(res.unwrap().declassification_grant_id, Some(0xDEAD_BEEF_u64));
    }

    #[test]
    fn test_c4_restricted_master_allowed_for_private_derivation() {
        let mut master = sample_master();
        master.sensitivity_class = SENSITIVITY_RESTRICTED;
        let cand = dummy_candidate("film_cand", StylePreset::PhotorealisticSpectral, 0, 500_000, 64, 96);
        let budget = TargetProfileBudget::film_reference(); // public_distribution = false
        let res = compile_master_to_target_profile(&master, &cand, &budget);
        assert!(res.is_ok());
    }
}
