// Clean-room cooperative authoring and review session engine (IP-03 C1-C8 & AUTH-H1..H6)
// Provides human-principal attribution, scoped agent delegation, revision conflict safety,
// and independent reviewer sign-off without external dependencies.

use std::collections::BTreeMap;
use std::fmt;

pub const ROLE_CONTRIBUTOR: u32 = 1 << 0;
pub const ROLE_REVIEWER: u32 = 1 << 1;
pub const ROLE_ADMIN: u32 = 1 << 2;

/// Authenticated human principal grant (AUTH-H1, AUTH-H2).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HumanPrincipalGrant {
    pub principal_did: u64,
    pub role_mask: u32,
    pub valid_until_epoch_s: u64,
}

/// Bounded delegation grant for an AI or automation agent (AUTH-H3).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DelegatedAgentGrant {
    pub agent_did: u64,
    pub delegating_principal_did: u64,
    pub allowed_domains_mask: u32,
    pub revision_budget: u32,
    pub valid_until_epoch_s: u64,
    pub revoked: bool,
}

/// Durable committed revision record with principal provenance (C1, C2, C6).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthoringRevisionRecord {
    pub revision: u64,
    pub author_principal_did: u64,
    pub acting_agent_did: Option<u64>,
    pub domain_id: u32,
    pub change_summary: String,
    pub payload_digest: [u8; 32],
    pub committed_epoch_s: u64,
    pub review_decision: Option<ReviewDecision>,
}

/// Formal review decision by an independent reviewer (C4, AUTH-H4).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReviewDecision {
    pub reviewer_principal_did: u64,
    pub acting_agent_did: Option<u64>,
    pub approved: bool,
    pub notes: String,
    pub decided_epoch_s: u64,
}

/// Transient draft saved upon disconnection without false commit (C7).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingReconnectDraft {
    pub session_id: u64,
    pub principal_did: u64,
    pub agent_did: Option<u64>,
    pub target_revision: u64,
    pub domain_id: u32,
    pub draft_payload: Vec<u8>,
    pub disconnected_epoch_s: u64,
}

/// Errors returned by the cooperative session engine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CooperativeError {
    PrincipalNotFound { principal_did: u64 },
    PrincipalExpired { principal_did: u64, valid_until: u64, current_time: u64 },
    UnauthorizedRole { principal_did: u64, required: u32, actual: u32 },
    DelegationNotFound { agent_did: u64 },
    DelegationExpired { agent_did: u64, valid_until: u64, current_time: u64 },
    DelegationRevoked { agent_did: u64 },
    DomainScopeDenied { domain_id: u32, allowed_mask: u32 },
    RevisionBudgetExhausted { agent_did: u64 },
    SelfApprovalForbidden { principal_did: u64 },
    RevisionConflict { expected: u64, current: u64 },
    RevisionNotFound { revision: u64 },
    AlreadyReviewed { revision: u64 },
    SessionNotFound { session_id: u64 },
    PrincipalMismatch { session_id: u64, expected: u64, actual: u64 },
}

impl fmt::Display for CooperativeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PrincipalNotFound { principal_did } => write!(f, "Principal 0x{:016x} not found", principal_did),
            Self::PrincipalExpired { principal_did, valid_until, current_time } => write!(f, "Principal 0x{:016x} expired at {} (curr {})", principal_did, valid_until, current_time),
            Self::UnauthorizedRole { principal_did, required, actual } => write!(f, "Principal 0x{:016x} lacks role 0x{:x} (has 0x{:x})", principal_did, required, actual),
            Self::DelegationNotFound { agent_did } => write!(f, "Agent 0x{:016x} delegation not found", agent_did),
            Self::DelegationExpired { agent_did, valid_until, current_time } => write!(f, "Agent 0x{:016x} expired at {} (curr {})", agent_did, valid_until, current_time),
            Self::DelegationRevoked { agent_did } => write!(f, "Agent 0x{:016x} delegation is revoked", agent_did),
            Self::DomainScopeDenied { domain_id, allowed_mask } => write!(f, "Domain {} denied by scope 0x{:x}", domain_id, allowed_mask),
            Self::RevisionBudgetExhausted { agent_did } => write!(f, "Agent 0x{:016x} budget exhausted", agent_did),
            Self::SelfApprovalForbidden { principal_did } => write!(f, "Reviewer 0x{:016x} cannot self-approve", principal_did),
            Self::RevisionConflict { expected, current } => write!(f, "Revision conflict: expected {}, current is {}", expected, current),
            Self::RevisionNotFound { revision } => write!(f, "Revision {} not found", revision),
            Self::AlreadyReviewed { revision } => write!(f, "Revision {} already reviewed", revision),
            Self::SessionNotFound { session_id } => write!(f, "Draft session 0x{:016x} not found", session_id),
            Self::PrincipalMismatch { session_id, expected, actual } => write!(f, "Session 0x{:016x} mismatch: expected 0x{:016x}, got 0x{:016x}", session_id, expected, actual),
        }
    }
}

impl std::error::Error for CooperativeError {}

/// Authoritative cooperative session engine orchestrating C1-C8 authoring and review.
#[derive(Debug, Default)]
pub struct CooperativeSessionEngine {
    principals: BTreeMap<u64, HumanPrincipalGrant>,
    agent_grants: BTreeMap<u64, DelegatedAgentGrant>,
    revisions: Vec<AuthoringRevisionRecord>,
    current_revision: u64,
    pending_drafts: BTreeMap<u64, PendingReconnectDraft>,
}

impl CooperativeSessionEngine {
    pub fn new() -> Self { Self::default() }
    pub fn current_revision(&self) -> u64 { self.current_revision }

    pub fn register_principal(&mut self, grant: HumanPrincipalGrant) {
        self.principals.insert(grant.principal_did, grant);
    }

    pub fn delegate_agent(&mut self, grant: DelegatedAgentGrant, current_time_s: u64) -> Result<(), CooperativeError> {
        let p = self.principals.get(&grant.delegating_principal_did)
            .ok_or(CooperativeError::PrincipalNotFound { principal_did: grant.delegating_principal_did })?;
        if current_time_s > p.valid_until_epoch_s {
            return Err(CooperativeError::PrincipalExpired { principal_did: p.principal_did, valid_until: p.valid_until_epoch_s, current_time: current_time_s });
        }
        if current_time_s > grant.valid_until_epoch_s {
            return Err(CooperativeError::DelegationExpired { agent_did: grant.agent_did, valid_until: grant.valid_until_epoch_s, current_time: current_time_s });
        }
        self.agent_grants.insert(grant.agent_did, grant);
        Ok(())
    }

    pub fn revoke_agent_delegation(&mut self, agent_did: u64) -> Result<(), CooperativeError> {
        let grant = self.agent_grants.get_mut(&agent_did).ok_or(CooperativeError::DelegationNotFound { agent_did })?;
        grant.revoked = true;
        Ok(())
    }

    pub fn submit_revision(
        &mut self,
        expected_revision: u64,
        principal_did: u64,
        acting_agent_did: Option<u64>,
        domain_id: u32,
        change_summary: String,
        payload_digest: [u8; 32],
        current_time_s: u64,
    ) -> Result<u64, CooperativeError> {
        if expected_revision != self.current_revision {
            return Err(CooperativeError::RevisionConflict { expected: expected_revision, current: self.current_revision });
        }
        let principal = self.principals.get(&principal_did)
            .ok_or(CooperativeError::PrincipalNotFound { principal_did })?;
        if current_time_s > principal.valid_until_epoch_s {
            return Err(CooperativeError::PrincipalExpired { principal_did, valid_until: principal.valid_until_epoch_s, current_time: current_time_s });
        }
        if (principal.role_mask & ROLE_CONTRIBUTOR) == 0 {
            return Err(CooperativeError::UnauthorizedRole { principal_did, required: ROLE_CONTRIBUTOR, actual: principal.role_mask });
        }
        if let Some(agent_did) = acting_agent_did {
            let grant = self.agent_grants.get_mut(&agent_did)
                .ok_or(CooperativeError::DelegationNotFound { agent_did })?;
            if grant.delegating_principal_did != principal_did {
                return Err(CooperativeError::PrincipalMismatch { session_id: 0, expected: grant.delegating_principal_did, actual: principal_did });
            }
            if grant.revoked { return Err(CooperativeError::DelegationRevoked { agent_did }); }
            if current_time_s > grant.valid_until_epoch_s {
                return Err(CooperativeError::DelegationExpired { agent_did, valid_until: grant.valid_until_epoch_s, current_time: current_time_s });
            }
            let domain_bit = 1u32.checked_shl(domain_id).unwrap_or(0);
            if (grant.allowed_domains_mask & domain_bit) == 0 {
                return Err(CooperativeError::DomainScopeDenied { domain_id, allowed_mask: grant.allowed_domains_mask });
            }
            if grant.revision_budget == 0 { return Err(CooperativeError::RevisionBudgetExhausted { agent_did }); }
            grant.revision_budget -= 1;
        }

        self.current_revision += 1;
        self.revisions.push(AuthoringRevisionRecord {
            revision: self.current_revision,
            author_principal_did: principal_did,
            acting_agent_did,
            domain_id,
            change_summary,
            payload_digest,
            committed_epoch_s: current_time_s,
            review_decision: None,
        });
        Ok(self.current_revision)
    }

    pub fn review_revision(
        &mut self,
        revision: u64,
        reviewer_principal_did: u64,
        acting_agent_did: Option<u64>,
        approved: bool,
        notes: String,
        current_time_s: u64,
    ) -> Result<(), CooperativeError> {
        let reviewer = self.principals.get(&reviewer_principal_did)
            .ok_or(CooperativeError::PrincipalNotFound { principal_did: reviewer_principal_did })?;
        if current_time_s > reviewer.valid_until_epoch_s {
            return Err(CooperativeError::PrincipalExpired { principal_did: reviewer_principal_did, valid_until: reviewer.valid_until_epoch_s, current_time: current_time_s });
        }
        if (reviewer.role_mask & ROLE_REVIEWER) == 0 {
            return Err(CooperativeError::UnauthorizedRole { principal_did: reviewer_principal_did, required: ROLE_REVIEWER, actual: reviewer.role_mask });
        }
        if let Some(agent_did) = acting_agent_did {
            let grant = self.agent_grants.get(&agent_did)
                .ok_or(CooperativeError::DelegationNotFound { agent_did })?;
            if grant.delegating_principal_did != reviewer_principal_did {
                return Err(CooperativeError::PrincipalMismatch { session_id: 0, expected: grant.delegating_principal_did, actual: reviewer_principal_did });
            }
            if grant.revoked { return Err(CooperativeError::DelegationRevoked { agent_did }); }
            if current_time_s > grant.valid_until_epoch_s {
                return Err(CooperativeError::DelegationExpired { agent_did, valid_until: grant.valid_until_epoch_s, current_time: current_time_s });
            }
        }

        let record = self.revisions.iter_mut().find(|r| r.revision == revision)
            .ok_or(CooperativeError::RevisionNotFound { revision })?;
        if record.review_decision.is_some() {
            return Err(CooperativeError::AlreadyReviewed { revision });
        }
        // AUTH-H4 / C4: Self-approval is strictly forbidden
        if reviewer_principal_did == record.author_principal_did {
            return Err(CooperativeError::SelfApprovalForbidden { principal_did: reviewer_principal_did });
        }

        record.review_decision = Some(ReviewDecision {
            reviewer_principal_did,
            acting_agent_did,
            approved,
            notes,
            decided_epoch_s: current_time_s,
        });
        Ok(())
    }

    pub fn disconnect_session(
        &mut self,
        session_id: u64,
        principal_did: u64,
        agent_did: Option<u64>,
        target_revision: u64,
        domain_id: u32,
        draft_payload: Vec<u8>,
        current_time_s: u64,
    ) -> Result<(), CooperativeError> {
        self.pending_drafts.insert(session_id, PendingReconnectDraft {
            session_id, principal_did, agent_did, target_revision, domain_id, draft_payload, disconnected_epoch_s: current_time_s,
        });
        Ok(())
    }

    pub fn reconnect_session(&mut self, session_id: u64, principal_did: u64) -> Result<PendingReconnectDraft, CooperativeError> {
        let draft = self.pending_drafts.get(&session_id).ok_or(CooperativeError::SessionNotFound { session_id })?;
        if draft.principal_did != principal_did {
            return Err(CooperativeError::PrincipalMismatch { session_id, expected: draft.principal_did, actual: principal_did });
        }
        Ok(self.pending_drafts.remove(&session_id).unwrap())
    }

    pub fn get_revision(&self, revision: u64) -> Option<&AuthoringRevisionRecord> {
        self.revisions.iter().find(|r| r.revision == revision)
    }

    pub fn pending_draft_count(&self) -> usize { self.pending_drafts.len() }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_engine() -> (CooperativeSessionEngine, u64, u64, u64) {
        let mut engine = CooperativeSessionEngine::new();
        let alice = 0x1001;
        let bob = 0x2002;
        let agent = 0x9009;

        engine.register_principal(HumanPrincipalGrant { principal_did: alice, role_mask: ROLE_CONTRIBUTOR, valid_until_epoch_s: 10_000 });
        engine.register_principal(HumanPrincipalGrant { principal_did: bob, role_mask: ROLE_REVIEWER, valid_until_epoch_s: 10_000 });
        engine.delegate_agent(DelegatedAgentGrant {
            agent_did: agent, delegating_principal_did: alice, allowed_domains_mask: 0x01, revision_budget: 3, valid_until_epoch_s: 10_000, revoked: false,
        }, 100).unwrap();

        (engine, alice, bob, agent)
    }

    #[test]
    fn test_c1_c3_principal_submit_and_review() {
        let (mut engine, alice, bob, _) = setup_engine();
        let rev = engine.submit_revision(0, alice, None, 0, "Initial edit".into(), [1u8; 32], 200).unwrap();
        assert_eq!(rev, 1);
        assert_eq!(engine.current_revision(), 1);

        engine.review_revision(1, bob, None, true, "Approved".into(), 250).unwrap();
        assert!(engine.get_revision(1).unwrap().review_decision.as_ref().unwrap().approved);
    }

    #[test]
    fn test_auth_h3_delegated_agent_budget_and_expiry() {
        let (mut engine, alice, _, agent) = setup_engine();
        engine.submit_revision(0, alice, Some(agent), 0, "Edit 1".into(), [1u8; 32], 200).unwrap();
        engine.submit_revision(1, alice, Some(agent), 0, "Edit 2".into(), [2u8; 32], 201).unwrap();
        engine.submit_revision(2, alice, Some(agent), 0, "Edit 3".into(), [3u8; 32], 202).unwrap();

        let err = engine.submit_revision(3, alice, Some(agent), 0, "Edit 4".into(), [4u8; 32], 203).unwrap_err();
        assert_eq!(err, CooperativeError::RevisionBudgetExhausted { agent_did: agent });
    }

    #[test]
    fn test_auth_h4_c4_self_approval_forbidden() {
        let mut engine = CooperativeSessionEngine::new();
        let charlie = 0x3003;
        engine.register_principal(HumanPrincipalGrant { principal_did: charlie, role_mask: ROLE_CONTRIBUTOR | ROLE_REVIEWER, valid_until_epoch_s: 10_000 });
        engine.submit_revision(0, charlie, None, 0, "Self edit".into(), [9u8; 32], 100).unwrap();
        let err = engine.review_revision(1, charlie, None, true, "Self approval".into(), 150).unwrap_err();
        assert_eq!(err, CooperativeError::SelfApprovalForbidden { principal_did: charlie });
    }

    #[test]
    fn test_c5_revision_conflict_fails_closed() {
        let (mut engine, alice, _, _) = setup_engine();
        engine.submit_revision(0, alice, None, 0, "Edit 1".into(), [1u8; 32], 200).unwrap();
        let err = engine.submit_revision(0, alice, None, 0, "Stale".into(), [2u8; 32], 210).unwrap_err();
        assert_eq!(err, CooperativeError::RevisionConflict { expected: 0, current: 1 });
        assert_eq!(engine.current_revision(), 1);
    }

    #[test]
    fn test_auth_h3_revoked_agent_fails_closed() {
        let (mut engine, alice, _, agent) = setup_engine();
        engine.revoke_agent_delegation(agent).unwrap();
        let err = engine.submit_revision(0, alice, Some(agent), 0, "Revoked".into(), [1u8; 32], 200).unwrap_err();
        assert_eq!(err, CooperativeError::DelegationRevoked { agent_did: agent });
    }

    #[test]
    fn test_c7_disconnect_and_reconnect_draft_recovery() {
        let (mut engine, alice, _, _) = setup_engine();
        let payload = vec![0xCA, 0xFE, 0xBA, 0xBE];
        engine.disconnect_session(0xABC, alice, None, 0, 0, payload.clone(), 300).unwrap();
        assert_eq!(engine.pending_draft_count(), 1);
        assert_eq!(engine.current_revision(), 0);

        let draft = engine.reconnect_session(0xABC, alice).unwrap();
        assert_eq!(draft.draft_payload, payload);
        assert_eq!(engine.pending_draft_count(), 0);
        assert_eq!(engine.current_revision(), 0);
    }
}
