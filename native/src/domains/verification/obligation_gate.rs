// Clean-room unit-safe agreement, cooperative commons obligation,
// and periodic credit allocation gate (AGR-01..11, FUND-01..06).
// Enforces explicit currency valuation, no-charge human-rights protections without false discharge,
// and zero-overdraft pre-funded credit limits.

use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct ContributionRecord {
    pub contributor_did: u64,
    pub role: String,
    pub authority_scope: String,
    pub is_paid_work: bool,
    pub effort_units: f64,
    pub explicit_valuation_currency: Option<String>,
    pub monetary_amount_minor_units: Option<i64>,
}

impl ContributionRecord {
    /// Validates whether this contribution can be presented as monetary compensation.
    /// Effort units or paid claims without an explicit currency and minor-unit amount MUST fail.
    pub fn validate_monetary_compensation(&self) -> Result<i64, ObligationGateError> {
        let has_effort = self.effort_units > 0.0;
        let claims_pay = self.is_paid_work || self.monetary_amount_minor_units.is_some();

        if has_effort || claims_pay {
            let currency_valid = self
                .explicit_valuation_currency
                .as_deref()
                .map(|s| !s.trim().is_empty())
                .unwrap_or(false);

            match (currency_valid, self.monetary_amount_minor_units) {
                (true, Some(amount)) if amount >= 0 => Ok(amount),
                _ => Err(ObligationGateError::UnvaluedEffortPresentedAsMoney),
            }
        } else {
            Ok(0)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UsageLicenseContext {
    CommercialQualifyingUse,
    HumanRightsNoChargeUse,
    InternalResearchUse,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObligationPoolLedger {
    pub remaining_obligation_minor_units: i64,
    pub originating_contributors: Vec<u64>,
    pub discharged: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CarryInstruction {
    ReturnToProvider,
    CarryForwardToNextPeriod,
    DelegateToPurpose {
        recipient_did: u64,
        purpose_scope: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreFundedCreditAllocation {
    pub provider_did: u64,
    pub authorized_backing_minor_units: i64,
    pub drawn_minor_units: i64,
    pub period_end_epoch_s: u64,
    pub return_or_carry_instruction: CarryInstruction,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReconciledResidual {
    pub provider_did: u64,
    pub total_authorized_minor_units: i64,
    pub total_consumed_minor_units: i64,
    pub residual_minor_units: i64,
    pub disposition: CarryInstruction,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ObligationGateError {
    UnvaluedEffortPresentedAsMoney,
    NoChargeUseIsNotDischarge,
    CreditAllocationOverdraftForbidden { requested: i64, available: i64 },
    InvalidDrawdownAmount(i64),
    ObligationAlreadyDischarged,
    NegativeSettlementAmount(i64),
}

impl fmt::Display for ObligationGateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnvaluedEffortPresentedAsMoney => {
                write!(f, "Cannot present effort units as monetary compensation without explicit currency and minor-unit valuation")
            }
            Self::NoChargeUseIsNotDischarge => {
                write!(f, "Human rights no-charge use is zero charge and cannot reduce obligation pool or mark discharged")
            }
            Self::CreditAllocationOverdraftForbidden { requested, available } => {
                write!(f, "Credit allocation overdraft forbidden: requested {requested}, available {available}")
            }
            Self::InvalidDrawdownAmount(amt) => write!(f, "Invalid negative drawdown amount: {amt}"),
            Self::ObligationAlreadyDischarged => write!(f, "Obligation pool is already fully discharged"),
            Self::NegativeSettlementAmount(amt) => write!(f, "Negative settlement amount: {amt}"),
        }
    }
}

impl std::error::Error for ObligationGateError {}

/// Records a usage event and applies settlement to the obligation pool under the specified license context.
pub fn record_usage_and_settle(
    pool: &mut ObligationPoolLedger,
    context: UsageLicenseContext,
    proposed_charge_minor_units: i64,
    mark_discharged: bool,
) -> Result<i64, ObligationGateError> {
    match context {
        UsageLicenseContext::HumanRightsNoChargeUse => {
            if mark_discharged || proposed_charge_minor_units > 0 {
                return Err(ObligationGateError::NoChargeUseIsNotDischarge);
            }
            Ok(0)
        }
        UsageLicenseContext::InternalResearchUse => {
            if mark_discharged {
                return Err(ObligationGateError::NoChargeUseIsNotDischarge);
            }
            Ok(0)
        }
        UsageLicenseContext::CommercialQualifyingUse => {
            if pool.discharged {
                return Err(ObligationGateError::ObligationAlreadyDischarged);
            }
            if proposed_charge_minor_units < 0 {
                return Err(ObligationGateError::NegativeSettlementAmount(proposed_charge_minor_units));
            }
            let actual_recovery = proposed_charge_minor_units.min(pool.remaining_obligation_minor_units);
            pool.remaining_obligation_minor_units -= actual_recovery;
            if pool.remaining_obligation_minor_units == 0 || mark_discharged {
                pool.discharged = true;
            }
            Ok(actual_recovery)
        }
    }
}

impl PreFundedCreditAllocation {
    /// Attempts to draw down credit from the allocation. Fails closed on any overdraft.
    pub fn draw_down(&mut self, request_minor_units: i64) -> Result<i64, ObligationGateError> {
        if request_minor_units < 0 {
            return Err(ObligationGateError::InvalidDrawdownAmount(request_minor_units));
        }
        let available = self.authorized_backing_minor_units.saturating_sub(self.drawn_minor_units);
        if request_minor_units > available {
            return Err(ObligationGateError::CreditAllocationOverdraftForbidden {
                requested: request_minor_units,
                available,
            });
        }
        self.drawn_minor_units += request_minor_units;
        Ok(self.drawn_minor_units)
    }

    /// Reconciles residual at period cutoff, honoring provider disposition instructions.
    pub fn reconcile_period(&self, _current_epoch_s: u64) -> Result<ReconciledResidual, ObligationGateError> {
        let residual = self.authorized_backing_minor_units.saturating_sub(self.drawn_minor_units);
        Ok(ReconciledResidual {
            provider_did: self.provider_did,
            total_authorized_minor_units: self.authorized_backing_minor_units,
            total_consumed_minor_units: self.drawn_minor_units,
            residual_minor_units: residual,
            disposition: self.return_or_carry_instruction.clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unvalued_effort_as_money_rejected() {
        let record = ContributionRecord {
            contributor_did: 100,
            role: "3D Modeler".into(),
            authority_scope: "fauna/avian".into(),
            is_paid_work: false,
            effort_units: 120.0,
            explicit_valuation_currency: None,
            monetary_amount_minor_units: None,
        };
        assert_eq!(
            record.validate_monetary_compensation(),
            Err(ObligationGateError::UnvaluedEffortPresentedAsMoney)
        );
    }

    #[test]
    fn test_empty_currency_valuation_rejected() {
        let record = ContributionRecord {
            contributor_did: 100,
            role: "Animator".into(),
            authority_scope: "fauna/avian".into(),
            is_paid_work: true,
            effort_units: 80.0,
            explicit_valuation_currency: Some("   ".into()),
            monetary_amount_minor_units: Some(5000),
        };
        assert_eq!(
            record.validate_monetary_compensation(),
            Err(ObligationGateError::UnvaluedEffortPresentedAsMoney)
        );
    }

    #[test]
    fn test_valued_effort_with_currency_succeeds() {
        let record = ContributionRecord {
            contributor_did: 101,
            role: "Texture Artist".into(),
            authority_scope: "fauna/avian".into(),
            is_paid_work: true,
            effort_units: 60.0,
            explicit_valuation_currency: Some("AUD".into()),
            monetary_amount_minor_units: Some(45000), // $450.00
        };
        assert_eq!(record.validate_monetary_compensation(), Ok(45000));
    }

    #[test]
    fn test_human_rights_no_charge_use_permitted_at_zero() {
        let mut pool = ObligationPoolLedger {
            remaining_obligation_minor_units: 100_000,
            originating_contributors: vec![101, 102],
            discharged: false,
        };
        let res = record_usage_and_settle(
            &mut pool,
            UsageLicenseContext::HumanRightsNoChargeUse,
            0,
            false,
        );
        assert_eq!(res, Ok(0));
        assert_eq!(pool.remaining_obligation_minor_units, 100_000);
        assert!(!pool.discharged);
    }

    #[test]
    fn test_human_rights_no_charge_cannot_discharge_pool() {
        let mut pool = ObligationPoolLedger {
            remaining_obligation_minor_units: 100_000,
            originating_contributors: vec![101],
            discharged: false,
        };
        let res = record_usage_and_settle(
            &mut pool,
            UsageLicenseContext::HumanRightsNoChargeUse,
            0,
            true, // attempting discharge
        );
        assert_eq!(res, Err(ObligationGateError::NoChargeUseIsNotDischarge));
        assert!(!pool.discharged);
    }

    #[test]
    fn test_human_rights_no_charge_cannot_reduce_pool() {
        let mut pool = ObligationPoolLedger {
            remaining_obligation_minor_units: 100_000,
            originating_contributors: vec![101],
            discharged: false,
        };
        let res = record_usage_and_settle(
            &mut pool,
            UsageLicenseContext::HumanRightsNoChargeUse,
            5000, // attempting charge reduction
            false,
        );
        assert_eq!(res, Err(ObligationGateError::NoChargeUseIsNotDischarge));
        assert_eq!(pool.remaining_obligation_minor_units, 100_000);
    }

    #[test]
    fn test_commercial_use_reduces_pool_and_discharges() {
        let mut pool = ObligationPoolLedger {
            remaining_obligation_minor_units: 50_000,
            originating_contributors: vec![101, 102],
            discharged: false,
        };
        let res1 = record_usage_and_settle(
            &mut pool,
            UsageLicenseContext::CommercialQualifyingUse,
            30_000,
            false,
        );
        assert_eq!(res1, Ok(30_000));
        assert_eq!(pool.remaining_obligation_minor_units, 20_000);
        assert!(!pool.discharged);

        let res2 = record_usage_and_settle(
            &mut pool,
            UsageLicenseContext::CommercialQualifyingUse,
            25_000,
            false,
        );
        assert_eq!(res2, Ok(20_000)); // capped at remaining
        assert_eq!(pool.remaining_obligation_minor_units, 0);
        assert!(pool.discharged);
    }

    #[test]
    fn test_commercial_use_on_already_discharged_pool_rejected() {
        let mut pool = ObligationPoolLedger {
            remaining_obligation_minor_units: 0,
            originating_contributors: vec![101],
            discharged: true,
        };
        let res = record_usage_and_settle(
            &mut pool,
            UsageLicenseContext::CommercialQualifyingUse,
            10_000,
            false,
        );
        assert_eq!(res, Err(ObligationGateError::ObligationAlreadyDischarged));
    }

    #[test]
    fn test_credit_allocation_drawdown_success() {
        let mut alloc = PreFundedCreditAllocation {
            provider_did: 200,
            authorized_backing_minor_units: 100_000,
            drawn_minor_units: 20_000,
            period_end_epoch_s: 1730000000,
            return_or_carry_instruction: CarryInstruction::ReturnToProvider,
        };
        let drawn = alloc.draw_down(30_000);
        assert_eq!(drawn, Ok(50_000));
        assert_eq!(alloc.drawn_minor_units, 50_000);
    }

    #[test]
    fn test_credit_allocation_overdraft_forbidden() {
        let mut alloc = PreFundedCreditAllocation {
            provider_did: 200,
            authorized_backing_minor_units: 100_000,
            drawn_minor_units: 90_000,
            period_end_epoch_s: 1730000000,
            return_or_carry_instruction: CarryInstruction::ReturnToProvider,
        };
        let res = alloc.draw_down(15_000);
        assert_eq!(
            res,
            Err(ObligationGateError::CreditAllocationOverdraftForbidden {
                requested: 15_000,
                available: 10_000,
            })
        );
        assert_eq!(alloc.drawn_minor_units, 90_000); // unchanged
    }

    #[test]
    fn test_credit_allocation_negative_drawdown_rejected() {
        let mut alloc = PreFundedCreditAllocation {
            provider_did: 200,
            authorized_backing_minor_units: 100_000,
            drawn_minor_units: 0,
            period_end_epoch_s: 1730000000,
            return_or_carry_instruction: CarryInstruction::ReturnToProvider,
        };
        assert_eq!(alloc.draw_down(-500), Err(ObligationGateError::InvalidDrawdownAmount(-500)));
    }

    #[test]
    fn test_period_reconciliation_return_residual() {
        let alloc = PreFundedCreditAllocation {
            provider_did: 200,
            authorized_backing_minor_units: 100_000,
            drawn_minor_units: 60_000,
            period_end_epoch_s: 1730000000,
            return_or_carry_instruction: CarryInstruction::ReturnToProvider,
        };
        let reconciled = alloc.reconcile_period(1730000000).unwrap();
        assert_eq!(reconciled.residual_minor_units, 40_000);
        assert_eq!(reconciled.disposition, CarryInstruction::ReturnToProvider);
    }

    #[test]
    fn test_period_reconciliation_delegate_residual() {
        let alloc = PreFundedCreditAllocation {
            provider_did: 200,
            authorized_backing_minor_units: 100_000,
            drawn_minor_units: 75_000,
            period_end_epoch_s: 1730000000,
            return_or_carry_instruction: CarryInstruction::DelegateToPurpose {
                recipient_did: 505,
                purpose_scope: "indigenous-land-care".into(),
            },
        };
        let reconciled = alloc.reconcile_period(1730000000).unwrap();
        assert_eq!(reconciled.residual_minor_units, 25_000);
        assert!(matches!(reconciled.disposition, CarryInstruction::DelegateToPurpose { recipient_did: 505, .. }));
    }
}
