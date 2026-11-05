// Clean-room game-demo civics scenario derivative loader and admission verifier (ENG-12, IP-06).
// Ensures game-demo consumes only fully reviewed OpenWorldGame derivatives with unbroken master provenance,
// public sensitivity clearance (class 0), and complete civics scenario anchors.

use std::fmt;
use super::completion_gate::{CompletionGateVerdict, TargetProfileKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReviewedDerivativeManifest {
    pub derivative_id: String,
    pub target_profile: TargetProfileKind,
    pub derives_from_master_sha256: [u8; 32],
    pub derivative_artifact_sha256: [u8; 32],
    pub completion_verdict: CompletionGateVerdict,
    pub is_reviewed: bool,
    pub reviewer_did: u64,
    pub sensitivity_class: u8,
    pub available_anchor_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CivicsScenarioRequirement {
    pub scenario_id: String,
    pub required_master_sha256: Option<[u8; 32]>,
    pub required_anchor_ids: Vec<String>,
    pub max_permitted_sensitivity_class: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdmittedGameDemoDerivative {
    pub derivative_id: String,
    pub derives_from_master_sha256: [u8; 32],
    pub bound_scenario_id: String,
    pub admitted_epoch_s: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConsumerLoaderError {
    TargetProfileMismatch {
        expected: TargetProfileKind,
        found: TargetProfileKind,
    },
    InvalidMasterDigest,
    MasterDigestMismatch {
        expected: [u8; 32],
        found: [u8; 32],
    },
    UnreviewedDerivative,
    CompletionGateNotPassed,
    SensitivityClassNotPermitted {
        class: u8,
        max_permitted: u8,
    },
    MissingScenarioAnchor {
        anchor_id: String,
    },
}

impl fmt::Display for ConsumerLoaderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TargetProfileMismatch { expected, found } => {
                write!(f, "Target profile mismatch: expected {expected:?}, found {found:?}")
            }
            Self::InvalidMasterDigest => write!(f, "Invalid master derivative digest: zeroed hash"),
            Self::MasterDigestMismatch { expected, found } => {
                write!(f, "Master digest mismatch: expected {expected:?}, found {found:?}")
            }
            Self::UnreviewedDerivative => write!(f, "Derivative has not received formal reviewer sign-off"),
            Self::CompletionGateNotPassed => write!(f, "Derivative completion gate verdict is not accepted"),
            Self::SensitivityClassNotPermitted { class, max_permitted } => {
                write!(f, "Sensitivity class {class} exceeds permitted maximum {max_permitted}")
            }
            Self::MissingScenarioAnchor { anchor_id } => {
                write!(f, "Missing required civics scenario anchor: {anchor_id}")
            }
        }
    }
}

impl std::error::Error for ConsumerLoaderError {}

/// Verifies and admits a high-grade derivative asset for game-demo civics scenario consumption.
pub fn verify_and_admit_game_demo_derivative(
    manifest: &ReviewedDerivativeManifest,
    requirement: &CivicsScenarioRequirement,
    current_epoch_s: u64,
) -> Result<AdmittedGameDemoDerivative, ConsumerLoaderError> {
    if manifest.target_profile != TargetProfileKind::OpenWorldGame {
        return Err(ConsumerLoaderError::TargetProfileMismatch {
            expected: TargetProfileKind::OpenWorldGame,
            found: manifest.target_profile,
        });
    }

    if manifest.derives_from_master_sha256 == [0u8; 32] {
        return Err(ConsumerLoaderError::InvalidMasterDigest);
    }

    if let Some(expected_master) = requirement.required_master_sha256 {
        if manifest.derives_from_master_sha256 != expected_master {
            return Err(ConsumerLoaderError::MasterDigestMismatch {
                expected: expected_master,
                found: manifest.derives_from_master_sha256,
            });
        }
    }

    if !manifest.is_reviewed || manifest.reviewer_did == 0 {
        return Err(ConsumerLoaderError::UnreviewedDerivative);
    }

    if manifest.completion_verdict != CompletionGateVerdict::CompletePackageAccepted {
        return Err(ConsumerLoaderError::CompletionGateNotPassed);
    }

    if manifest.sensitivity_class > requirement.max_permitted_sensitivity_class {
        return Err(ConsumerLoaderError::SensitivityClassNotPermitted {
            class: manifest.sensitivity_class,
            max_permitted: requirement.max_permitted_sensitivity_class,
        });
    }

    for required_anchor in &requirement.required_anchor_ids {
        if !manifest.available_anchor_ids.iter().any(|a| a == required_anchor) {
            return Err(ConsumerLoaderError::MissingScenarioAnchor {
                anchor_id: required_anchor.clone(),
            });
        }
    }

    Ok(AdmittedGameDemoDerivative {
        derivative_id: manifest.derivative_id.clone(),
        derives_from_master_sha256: manifest.derives_from_master_sha256,
        bound_scenario_id: requirement.scenario_id.clone(),
        admitted_epoch_s: current_epoch_s,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_hash(byte: u8) -> [u8; 32] {
        [byte; 32]
    }

    fn make_valid_manifest() -> ReviewedDerivativeManifest {
        ReviewedDerivativeManifest {
            derivative_id: "species-eucalyptus-regnans-rt-lod0".into(),
            target_profile: TargetProfileKind::OpenWorldGame,
            derives_from_master_sha256: dummy_hash(0x11),
            derivative_artifact_sha256: dummy_hash(0x22),
            completion_verdict: CompletionGateVerdict::CompletePackageAccepted,
            is_reviewed: true,
            reviewer_did: 8888,
            sensitivity_class: 0,
            available_anchor_ids: vec![
                "anchor-tree-hollow-nesting".into(),
                "anchor-fire-bark-shed".into(),
                "anchor-canopy-density-zone".into(),
            ],
        }
    }

    fn make_valid_requirement() -> CivicsScenarioRequirement {
        CivicsScenarioRequirement {
            scenario_id: "civics-scen-yarra-ranges-restoration".into(),
            required_master_sha256: Some(dummy_hash(0x11)),
            required_anchor_ids: vec![
                "anchor-tree-hollow-nesting".into(),
                "anchor-canopy-density-zone".into(),
            ],
            max_permitted_sensitivity_class: 0,
        }
    }

    #[test]
    fn test_valid_admission_succeeds() {
        let manifest = make_valid_manifest();
        let requirement = make_valid_requirement();
        let res = verify_and_admit_game_demo_derivative(&manifest, &requirement, 1729800000);
        assert!(res.is_ok());
        let admitted = res.unwrap();
        assert_eq!(admitted.derivative_id, "species-eucalyptus-regnans-rt-lod0");
        assert_eq!(admitted.bound_scenario_id, "civics-scen-yarra-ranges-restoration");
        assert_eq!(admitted.derives_from_master_sha256, dummy_hash(0x11));
    }

    #[test]
    fn test_target_profile_mismatch_film_rejected() {
        let mut manifest = make_valid_manifest();
        manifest.target_profile = TargetProfileKind::Film;
        let requirement = make_valid_requirement();
        let res = verify_and_admit_game_demo_derivative(&manifest, &requirement, 1729800000);
        assert_eq!(
            res,
            Err(ConsumerLoaderError::TargetProfileMismatch {
                expected: TargetProfileKind::OpenWorldGame,
                found: TargetProfileKind::Film,
            })
        );
    }

    #[test]
    fn test_target_profile_mismatch_cartoon_rejected() {
        let mut manifest = make_valid_manifest();
        manifest.target_profile = TargetProfileKind::Cartoon;
        let requirement = make_valid_requirement();
        let res = verify_and_admit_game_demo_derivative(&manifest, &requirement, 1729800000);
        assert_eq!(
            res,
            Err(ConsumerLoaderError::TargetProfileMismatch {
                expected: TargetProfileKind::OpenWorldGame,
                found: TargetProfileKind::Cartoon,
            })
        );
    }

    #[test]
    fn test_zeroed_master_digest_rejected() {
        let mut manifest = make_valid_manifest();
        manifest.derives_from_master_sha256 = [0u8; 32];
        let requirement = make_valid_requirement();
        let res = verify_and_admit_game_demo_derivative(&manifest, &requirement, 1729800000);
        assert_eq!(res, Err(ConsumerLoaderError::InvalidMasterDigest));
    }

    #[test]
    fn test_mismatched_master_digest_rejected() {
        let mut manifest = make_valid_manifest();
        manifest.derives_from_master_sha256 = dummy_hash(0x99);
        let requirement = make_valid_requirement();
        let res = verify_and_admit_game_demo_derivative(&manifest, &requirement, 1729800000);
        assert_eq!(
            res,
            Err(ConsumerLoaderError::MasterDigestMismatch {
                expected: dummy_hash(0x11),
                found: dummy_hash(0x99),
            })
        );
    }

    #[test]
    fn test_unreviewed_derivative_rejected() {
        let mut manifest = make_valid_manifest();
        manifest.is_reviewed = false;
        let requirement = make_valid_requirement();
        let res = verify_and_admit_game_demo_derivative(&manifest, &requirement, 1729800000);
        assert_eq!(res, Err(ConsumerLoaderError::UnreviewedDerivative));
    }

    #[test]
    fn test_zero_reviewer_did_rejected() {
        let mut manifest = make_valid_manifest();
        manifest.reviewer_did = 0;
        let requirement = make_valid_requirement();
        let res = verify_and_admit_game_demo_derivative(&manifest, &requirement, 1729800000);
        assert_eq!(res, Err(ConsumerLoaderError::UnreviewedDerivative));
    }

    #[test]
    fn test_restricted_sensitivity_class_rejected() {
        let mut manifest = make_valid_manifest();
        manifest.sensitivity_class = 1; // Restricted
        let requirement = make_valid_requirement(); // max allowed = 0
        let res = verify_and_admit_game_demo_derivative(&manifest, &requirement, 1729800000);
        assert_eq!(
            res,
            Err(ConsumerLoaderError::SensitivityClassNotPermitted {
                class: 1,
                max_permitted: 0,
            })
        );
    }

    #[test]
    fn test_missing_scenario_anchor_rejected() {
        let manifest = make_valid_manifest();
        let mut requirement = make_valid_requirement();
        requirement.required_anchor_ids.push("anchor-non-existent-habitat".into());
        let res = verify_and_admit_game_demo_derivative(&manifest, &requirement, 1729800000);
        assert_eq!(
            res,
            Err(ConsumerLoaderError::MissingScenarioAnchor {
                anchor_id: "anchor-non-existent-habitat".into(),
            })
        );
    }
}
