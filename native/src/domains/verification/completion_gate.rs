// Clean-room technical native model completion gate (ENG-12, IP-06, IP-08).
// Enforces 193 canonical deliverables, three mandatory delivery profiles (Film, OpenWorldGame, Cartoon),
// human/domain sign-off authority (no agent self-certification), and evidence freshness.

use std::collections::BTreeSet;
use std::fmt;

pub const CANONICAL_BASE_DELIVERABLE_COUNT: usize = 193;
pub const PINNED_CATALOGUE_VERSION: &str = "2026-10-24.2";
pub const PINNED_CATALOGUE_SHA256: [u8; 32] = [
    0xc0, 0xcb, 0x76, 0x15, 0x1b, 0xf5, 0xd0, 0x77,
    0x6a, 0xfa, 0xae, 0x4d, 0x59, 0xd7, 0x42, 0x7e,
    0xfb, 0x65, 0x63, 0x3b, 0xf3, 0x59, 0xd3, 0xf4,
    0xc2, 0xbd, 0xa8, 0x3c, 0x36, 0x1c, 0x91, 0xcb,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TargetProfileKind {
    Film,
    OpenWorldGame,
    Cartoon,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClaimApplicability {
    Required,
    NotApplicableWithRule { rule_id: String, authorized_by_did: u64 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeliverableVerificationStatus {
    Verified,
    Partial,
    Missing,
    Blocked,
    Failed,
    Unassessed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActorKind {
    HumanPrincipal,
    SoftwareAgent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReviewType {
    HumanAcceptance,
    DomainReview,
    TechnicalCheck,
    RightsReview,
    AudioReview,
    AccessibilityReview,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignOffAttestation {
    pub signer_did: u64,
    pub signer_kind: ActorKind,
    pub review_type: ReviewType,
    pub timestamp_epoch_s: u64,
    pub notes: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeliverableClaimRow {
    pub deliverable_id: usize,
    pub target_profile: TargetProfileKind,
    pub applicability: ClaimApplicability,
    pub status: DeliverableVerificationStatus,
    pub artifact_sha256: [u8; 32],
    pub current_artifact_sha256: [u8; 32],
    pub attestations: Vec<SignOffAttestation>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelCompletionLedger {
    pub catalogue_version: String,
    pub catalogue_sha256: [u8; 32],
    pub claims: Vec<DeliverableClaimRow>,
    pub expanded_case_count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompletionGateVerdict {
    CompletePackageAccepted,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompletionGateError {
    EmptyCatalogueVersion,
    CatalogueVersionMismatch { expected: String, found: String },
    EmptyCatalogueSha256,
    CatalogueSha256Mismatch,
    MissingBaseDeliverables { found: usize, expected: usize },
    MissingDeliverableId(usize),
    InsufficientExpandedCases { found: usize, expected: usize },
    MissingTargetProfile(TargetProfileKind),
    UnauthorizedNotApplicableWaiver { deliverable_id: usize },
    AgentSelfCertificationForbidden { deliverable_id: usize },
    StaleArtifactEvidence { deliverable_id: usize },
    DeliverableNotVerified { deliverable_id: usize, status: DeliverableVerificationStatus },
}

impl fmt::Display for CompletionGateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyCatalogueVersion => write!(f, "Catalogue version cannot be empty"),
            Self::CatalogueVersionMismatch { expected, found } => write!(f, "Catalogue version mismatch: expected {expected}, found {found}"),
            Self::EmptyCatalogueSha256 => write!(f, "Catalogue sha256 cannot be zeroed"),
            Self::CatalogueSha256Mismatch => write!(f, "Catalogue sha256 does not match pinned template"),
            Self::MissingBaseDeliverables { found, expected } => write!(f, "Incomplete deliverable count: found {found}, required {expected}"),
            Self::MissingDeliverableId(id) => write!(f, "Missing base deliverable ID: {id}"),
            Self::InsufficientExpandedCases { found, expected } => write!(f, "Insufficient claims: found {found}, required {expected}"),
            Self::MissingTargetProfile(p) => write!(f, "Missing verified target profile: {p:?}"),
            Self::UnauthorizedNotApplicableWaiver { deliverable_id } => write!(f, "Unauthorized N/A waiver on deliverable {deliverable_id}: missing rule_id or authorized_by_did"),
            Self::AgentSelfCertificationForbidden { deliverable_id } => write!(f, "Software agent self-certification forbidden on deliverable {deliverable_id}"),
            Self::StaleArtifactEvidence { deliverable_id } => write!(f, "Stale artifact evidence on deliverable {deliverable_id}: digest mismatch"),
            Self::DeliverableNotVerified { deliverable_id, status } => write!(f, "Required deliverable {deliverable_id} not verified: {status:?}"),
        }
    }
}

impl std::error::Error for CompletionGateError {}

/// Evaluates a model completion ledger against the 193 canonical deliverables,
/// pinned catalogue metadata, mandatory three profiles, and human/domain review rules.
pub fn evaluate_model_completion_gate(
    ledger: &ModelCompletionLedger,
) -> Result<CompletionGateVerdict, CompletionGateError> {
    if ledger.catalogue_version.trim().is_empty() {
        return Err(CompletionGateError::EmptyCatalogueVersion);
    }
    if ledger.catalogue_version != PINNED_CATALOGUE_VERSION {
        return Err(CompletionGateError::CatalogueVersionMismatch {
            expected: PINNED_CATALOGUE_VERSION.to_string(),
            found: ledger.catalogue_version.clone(),
        });
    }
    if ledger.catalogue_sha256 == [0u8; 32] {
        return Err(CompletionGateError::EmptyCatalogueSha256);
    }
    if ledger.catalogue_sha256 != PINNED_CATALOGUE_SHA256 {
        return Err(CompletionGateError::CatalogueSha256Mismatch);
    }

    let mut present_ids = BTreeSet::new();
    for claim in &ledger.claims {
        if claim.deliverable_id >= 1 && claim.deliverable_id <= CANONICAL_BASE_DELIVERABLE_COUNT {
            present_ids.insert(claim.deliverable_id);
        }
    }

    if present_ids.len() < CANONICAL_BASE_DELIVERABLE_COUNT {
        for id in 1..=CANONICAL_BASE_DELIVERABLE_COUNT {
            if !present_ids.contains(&id) {
                return Err(CompletionGateError::MissingDeliverableId(id));
            }
        }
        return Err(CompletionGateError::MissingBaseDeliverables {
            found: present_ids.len(),
            expected: CANONICAL_BASE_DELIVERABLE_COUNT,
        });
    }

    let expected_total = CANONICAL_BASE_DELIVERABLE_COUNT + ledger.expanded_case_count;
    if ledger.claims.len() < expected_total {
        return Err(CompletionGateError::InsufficientExpandedCases {
            found: ledger.claims.len(),
            expected: expected_total,
        });
    }

    let (mut film_verified, mut game_verified, mut cartoon_verified) = (false, false, false);

    for claim in &ledger.claims {
        match &claim.applicability {
            ClaimApplicability::NotApplicableWithRule { rule_id, authorized_by_did } => {
                if rule_id.trim().is_empty() || *authorized_by_did == 0 {
                    return Err(CompletionGateError::UnauthorizedNotApplicableWaiver {
                        deliverable_id: claim.deliverable_id,
                    });
                }
            }
            ClaimApplicability::Required => {
                if claim.status != DeliverableVerificationStatus::Verified {
                    return Err(CompletionGateError::DeliverableNotVerified {
                        deliverable_id: claim.deliverable_id,
                        status: claim.status,
                    });
                }
                match claim.target_profile {
                    TargetProfileKind::Film => film_verified = true,
                    TargetProfileKind::OpenWorldGame => game_verified = true,
                    TargetProfileKind::Cartoon => cartoon_verified = true,
                }
            }
        }

        for attestation in &claim.attestations {
            let is_human_or_domain = matches!(
                attestation.review_type,
                ReviewType::HumanAcceptance | ReviewType::DomainReview
            );
            if is_human_or_domain && attestation.signer_kind == ActorKind::SoftwareAgent {
                return Err(CompletionGateError::AgentSelfCertificationForbidden {
                    deliverable_id: claim.deliverable_id,
                });
            }
        }

        if claim.artifact_sha256 != claim.current_artifact_sha256 {
            return Err(CompletionGateError::StaleArtifactEvidence {
                deliverable_id: claim.deliverable_id,
            });
        }
    }

    if !film_verified {
        return Err(CompletionGateError::MissingTargetProfile(TargetProfileKind::Film));
    }
    if !game_verified {
        return Err(CompletionGateError::MissingTargetProfile(TargetProfileKind::OpenWorldGame));
    }
    if !cartoon_verified {
        return Err(CompletionGateError::MissingTargetProfile(TargetProfileKind::Cartoon));
    }

    Ok(CompletionGateVerdict::CompletePackageAccepted)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_digest(val: u8) -> [u8; 32] { [val; 32] }

    fn make_valid_claim(id: usize, profile: TargetProfileKind) -> DeliverableClaimRow {
        DeliverableClaimRow {
            deliverable_id: id,
            target_profile: profile,
            applicability: ClaimApplicability::Required,
            status: DeliverableVerificationStatus::Verified,
            artifact_sha256: dummy_digest(id as u8),
            current_artifact_sha256: dummy_digest(id as u8),
            attestations: vec![SignOffAttestation {
                signer_did: 42,
                signer_kind: ActorKind::HumanPrincipal,
                review_type: ReviewType::HumanAcceptance,
                timestamp_epoch_s: 1729800000,
                notes: "Approved by lead".into(),
            }],
        }
    }

    fn make_canonical_ledger() -> ModelCompletionLedger {
        let mut claims = Vec::with_capacity(CANONICAL_BASE_DELIVERABLE_COUNT);
        for id in 1..=CANONICAL_BASE_DELIVERABLE_COUNT {
            let profile = match id % 3 {
                0 => TargetProfileKind::Film,
                1 => TargetProfileKind::OpenWorldGame,
                _ => TargetProfileKind::Cartoon,
            };
            claims.push(make_valid_claim(id, profile));
        }
        ModelCompletionLedger {
            catalogue_version: PINNED_CATALOGUE_VERSION.into(),
            catalogue_sha256: PINNED_CATALOGUE_SHA256,
            claims,
            expanded_case_count: 0,
        }
    }

    #[test]
    fn test_valid_complete_package_passes() {
        assert_eq!(evaluate_model_completion_gate(&make_canonical_ledger()), Ok(CompletionGateVerdict::CompletePackageAccepted));
    }

    #[test]
    fn test_missing_catalogue_version_fails() {
        let mut ledger = make_canonical_ledger();
        ledger.catalogue_version = "".into();
        assert_eq!(evaluate_model_completion_gate(&ledger), Err(CompletionGateError::EmptyCatalogueVersion));
    }

    #[test]
    fn test_catalogue_version_mismatch_fails() {
        let mut ledger = make_canonical_ledger();
        ledger.catalogue_version = "2025-01-01.1".into();
        assert!(matches!(evaluate_model_completion_gate(&ledger), Err(CompletionGateError::CatalogueVersionMismatch { .. })));
    }

    #[test]
    fn test_empty_catalogue_sha_fails() {
        let mut ledger = make_canonical_ledger();
        ledger.catalogue_sha256 = [0u8; 32];
        assert_eq!(evaluate_model_completion_gate(&ledger), Err(CompletionGateError::EmptyCatalogueSha256));
    }

    #[test]
    fn test_catalogue_sha_mismatch_fails() {
        let mut ledger = make_canonical_ledger();
        ledger.catalogue_sha256 = [0xabu8; 32];
        assert_eq!(evaluate_model_completion_gate(&ledger), Err(CompletionGateError::CatalogueSha256Mismatch));
    }

    #[test]
    fn test_fewer_than_193_base_deliverables_fails() {
        let mut ledger = make_canonical_ledger();
        ledger.claims.pop();
        assert_eq!(evaluate_model_completion_gate(&ledger), Err(CompletionGateError::MissingDeliverableId(193)));
    }

    #[test]
    fn test_unauthorized_not_applicable_waiver_empty_rule() {
        let mut ledger = make_canonical_ledger();
        ledger.claims[0].applicability = ClaimApplicability::NotApplicableWithRule {
            rule_id: "  ".into(),
            authorized_by_did: 99,
        };
        assert_eq!(evaluate_model_completion_gate(&ledger), Err(CompletionGateError::UnauthorizedNotApplicableWaiver { deliverable_id: 1 }));
    }

    #[test]
    fn test_unauthorized_not_applicable_waiver_zero_did() {
        let mut ledger = make_canonical_ledger();
        ledger.claims[0].applicability = ClaimApplicability::NotApplicableWithRule {
            rule_id: "RULE-BIO-PLANT".into(),
            authorized_by_did: 0,
        };
        assert_eq!(evaluate_model_completion_gate(&ledger), Err(CompletionGateError::UnauthorizedNotApplicableWaiver { deliverable_id: 1 }));
    }

    #[test]
    fn test_authorized_not_applicable_waiver_succeeds() {
        let mut ledger = make_canonical_ledger();
        ledger.claims[0].applicability = ClaimApplicability::NotApplicableWithRule {
            rule_id: "RULE-BIO-PLANT".into(),
            authorized_by_did: 101,
        };
        assert_eq!(evaluate_model_completion_gate(&ledger), Ok(CompletionGateVerdict::CompletePackageAccepted));
    }

    #[test]
    fn test_agent_self_certification_forbidden_human_acceptance() {
        let mut ledger = make_canonical_ledger();
        ledger.claims[5].attestations.push(SignOffAttestation {
            signer_did: 999,
            signer_kind: ActorKind::SoftwareAgent,
            review_type: ReviewType::HumanAcceptance,
            timestamp_epoch_s: 1729800000,
            notes: "AI certifying appearance".into(),
        });
        assert_eq!(evaluate_model_completion_gate(&ledger), Err(CompletionGateError::AgentSelfCertificationForbidden { deliverable_id: 6 }));
    }

    #[test]
    fn test_agent_self_certification_forbidden_domain_review() {
        let mut ledger = make_canonical_ledger();
        ledger.claims[10].attestations.push(SignOffAttestation {
            signer_did: 999,
            signer_kind: ActorKind::SoftwareAgent,
            review_type: ReviewType::DomainReview,
            timestamp_epoch_s: 1729800000,
            notes: "AI certifying botanical anatomy".into(),
        });
        assert_eq!(evaluate_model_completion_gate(&ledger), Err(CompletionGateError::AgentSelfCertificationForbidden { deliverable_id: 11 }));
    }

    #[test]
    fn test_software_agent_technical_check_allowed() {
        let mut ledger = make_canonical_ledger();
        ledger.claims[0].attestations.push(SignOffAttestation {
            signer_did: 999,
            signer_kind: ActorKind::SoftwareAgent,
            review_type: ReviewType::TechnicalCheck,
            timestamp_epoch_s: 1729800000,
            notes: "Automated topology check passed".into(),
        });
        assert_eq!(evaluate_model_completion_gate(&ledger), Ok(CompletionGateVerdict::CompletePackageAccepted));
    }

    #[test]
    fn test_stale_artifact_evidence_fails() {
        let mut ledger = make_canonical_ledger();
        ledger.claims[3].current_artifact_sha256 = dummy_digest(0xee);
        assert_eq!(evaluate_model_completion_gate(&ledger), Err(CompletionGateError::StaleArtifactEvidence { deliverable_id: 4 }));
    }

    #[test]
    fn test_missing_target_profile_cartoon_fails() {
        let mut ledger = make_canonical_ledger();
        for claim in &mut ledger.claims {
            if claim.target_profile == TargetProfileKind::Cartoon {
                claim.target_profile = TargetProfileKind::Film;
            }
        }
        assert_eq!(evaluate_model_completion_gate(&ledger), Err(CompletionGateError::MissingTargetProfile(TargetProfileKind::Cartoon)));
    }

    #[test]
    fn test_unverified_deliverable_fails() {
        let mut ledger = make_canonical_ledger();
        ledger.claims[15].status = DeliverableVerificationStatus::Partial;
        assert_eq!(
            evaluate_model_completion_gate(&ledger),
            Err(CompletionGateError::DeliverableNotVerified {
                deliverable_id: 16,
                status: DeliverableVerificationStatus::Partial,
            })
        );
    }
}
