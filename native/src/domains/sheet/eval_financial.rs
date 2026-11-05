//! Clean-room spreadsheet financial evaluators.

use super::SheetError;

/// Computes the periodic payment for an annuity based on constant payments and interest rate.
pub fn pmt(
    rate: f64,
    nper: f64,
    pv: f64,
    fv: f64,
    payment_due_start: bool,
) -> Result<f64, SheetError> {
    if !rate.is_finite() || !nper.is_finite() || !pv.is_finite() || !fv.is_finite() {
        return Err(SheetError::InvalidArgument);
    }
    if nper == 0.0 {
        return Err(SheetError::DivisionByZero);
    }
    if rate == 0.0 {
        return Ok(-(pv + fv) / nper);
    }
    let term = (1.0 + rate).powf(nper);
    let factor = if payment_due_start { 1.0 + rate } else { 1.0 };
    let denom = factor * (term - 1.0) / rate;
    if denom == 0.0 || !denom.is_finite() {
        return Err(SheetError::DivisionByZero);
    }
    Ok(-(pv * term + fv) / denom)
}

/// Computes the present value of an investment or loan.
pub fn pv(
    rate: f64,
    nper: f64,
    pmt: f64,
    fv: f64,
    payment_due_start: bool,
) -> Result<f64, SheetError> {
    if !rate.is_finite() || !nper.is_finite() || !pmt.is_finite() || !fv.is_finite() {
        return Err(SheetError::InvalidArgument);
    }
    if rate == 0.0 {
        if nper == 0.0 && fv == 0.0 {
            return Ok(0.0);
        }
        return Ok(-fv - pmt * nper);
    }
    let term = (1.0 + rate).powf(nper);
    if term == 0.0 || !term.is_finite() {
        return Err(SheetError::DivisionByZero);
    }
    let factor = if payment_due_start { 1.0 + rate } else { 1.0 };
    let annuity_factor = factor * (term - 1.0) / rate;
    Ok((-fv - pmt * annuity_factor) / term)
}

/// Computes the future value of an investment based on constant payments and interest rate.
pub fn fv(
    rate: f64,
    nper: f64,
    pmt: f64,
    pv: f64,
    payment_due_start: bool,
) -> Result<f64, SheetError> {
    if !rate.is_finite() || !nper.is_finite() || !pmt.is_finite() || !pv.is_finite() {
        return Err(SheetError::InvalidArgument);
    }
    if rate == 0.0 {
        return Ok(-pv - pmt * nper);
    }
    let term = (1.0 + rate).powf(nper);
    let factor = if payment_due_start { 1.0 + rate } else { 1.0 };
    let annuity_factor = factor * (term - 1.0) / rate;
    Ok(-pv * term - pmt * annuity_factor)
}

/// Computes Net Present Value of cash flows discounted at periodic rate.
pub fn npv(rate: f64, cashflows: &[f64]) -> Result<f64, SheetError> {
    if cashflows.is_empty() || !rate.is_finite() || rate <= -1.0 {
        return Err(SheetError::InvalidArgument);
    }
    let mut sum = 0.0;
    for (i, &cf) in cashflows.iter().enumerate() {
        if !cf.is_finite() {
            return Err(SheetError::ValueError);
        }
        let t = (i + 1) as i32;
        sum += cf / (1.0 + rate).powi(t);
    }
    Ok(sum)
}

/// Computes Internal Rate of Return using iterative Newton-Raphson.
pub fn irr(cashflows: &[f64], max_iterations: usize, tol: f64) -> Result<f64, SheetError> {
    if cashflows.len() < 2 {
        return Err(SheetError::InvalidArgument);
    }
    let has_positive = cashflows.iter().any(|&c| c > 0.0);
    let has_negative = cashflows.iter().any(|&c| c < 0.0);
    if !has_positive || !has_negative {
        return Err(SheetError::InvalidArgument);
    }

    let mut rate: f64 = 0.1; // 10% initial guess
    for _ in 0..max_iterations {
        let mut f = 0.0;
        let mut f_prime = 0.0;
        for (t, &cf) in cashflows.iter().enumerate() {
            let denom = (1.0 + rate).powi(t as i32);
            f += cf / denom;
            if t > 0 {
                f_prime -= (t as f64) * cf / ((1.0 + rate).powi((t + 1) as i32));
            }
        }
        if f_prime.abs() < 1e-15 {
            return Err(SheetError::ValueError);
        }
        let next_rate = rate - f / f_prime;
        if (next_rate - rate).abs() < tol {
            return Ok(next_rate);
        }
        rate = next_rate;
    }
    Err(SheetError::ValueError)
}

/// Straight-line depreciation of an asset for one period.
pub fn sln(cost: f64, salvage: f64, life: f64) -> Result<f64, SheetError> {
    if life <= 0.0 || !cost.is_finite() || !salvage.is_finite() || !life.is_finite() {
        return Err(SheetError::InvalidArgument);
    }
    Ok((cost - salvage) / life)
}

/// Sum-of-years' digits depreciation for a specified period.
pub fn syd(cost: f64, salvage: f64, life: f64, period: f64) -> Result<f64, SheetError> {
    if life <= 0.0 || period <= 0.0 || period > life || !cost.is_finite() || !salvage.is_finite() {
        return Err(SheetError::InvalidArgument);
    }
    let sum_of_years = life * (life + 1.0) / 2.0;
    Ok((cost - salvage) * (life - period + 1.0) / sum_of_years)
}

/// Fixed-declining balance depreciation for a specified period.
pub fn db(cost: f64, salvage: f64, life: f64, period: f64) -> Result<f64, SheetError> {
    if cost <= 0.0 || salvage < 0.0 || life <= 0.0 || period <= 0.0 || period > life {
        return Err(SheetError::InvalidArgument);
    }
    let raw_rate = 1.0 - (salvage / cost).powf(1.0 / life);
    let rate = (raw_rate * 1000.0).round() / 1000.0;
    if rate <= 0.0 || rate >= 1.0 {
        return Err(SheetError::ValueError);
    }

    let mut prior_deprec_total = 0.0;
    let p_int = period.floor() as usize;
    let mut current_deprec = 0.0;
    for _ in 1..=p_int {
        current_deprec = (cost - prior_deprec_total) * rate;
        prior_deprec_total += current_deprec;
    }
    Ok(current_deprec)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pmt_pv_roundtrip() {
        let rate = 0.05 / 12.0;
        let nper = 60.0;
        let principal = 10_000.0;
        let monthly_pmt = pmt(rate, nper, -principal, 0.0, false).expect("pmt");
        assert!(monthly_pmt > 188.0 && monthly_pmt < 189.0);

        let recovered_pv = pv(rate, nper, monthly_pmt, 0.0, false).expect("pv");
        assert!((recovered_pv - (-principal)).abs() < 1e-4);
    }

    #[test]
    fn test_fv_positive() {
        let rate = 0.06 / 12.0;
        let nper = 12.0;
        let pmt_amount = -100.0;
        let future_val = fv(rate, nper, pmt_amount, 0.0, false).expect("fv");
        assert!(future_val > 1230.0 && future_val < 1240.0);
    }

    #[test]
    fn test_npv_and_irr() {
        let cashflows = [-1000.0, 300.0, 420.0, 680.0];
        let calculated_irr = irr(&cashflows, 100, 1e-7).expect("irr");
        assert!(calculated_irr > 0.15 && calculated_irr < 0.20);

        let npv_at_irr = npv(calculated_irr, &cashflows[1..]).expect("npv");
        assert!((npv_at_irr - 1000.0).abs() < 1e-3);
    }

    #[test]
    fn test_irr_negative_no_sign_change() {
        let invalid_cfs = [100.0, 200.0, 300.0];
        assert_eq!(irr(&invalid_cfs, 100, 1e-7), Err(SheetError::InvalidArgument));
    }

    #[test]
    fn test_sln_syd_db() {
        let cost = 10_000.0;
        let salvage = 1_000.0;
        let life = 5.0;

        let sln_val = sln(cost, salvage, life).expect("sln");
        assert_eq!(sln_val, 1800.0);

        let syd_p1 = syd(cost, salvage, life, 1.0).expect("syd p1");
        assert_eq!(syd_p1, 3000.0);

        let db_p1 = db(cost, salvage, life, 1.0).expect("db p1");
        assert!(db_p1 > 3600.0 && db_p1 < 3800.0);
    }

    #[test]
    fn test_negative_invalid_arguments() {
        assert_eq!(sln(1000.0, 100.0, 0.0), Err(SheetError::InvalidArgument));
        assert_eq!(syd(1000.0, 100.0, 5.0, 6.0), Err(SheetError::InvalidArgument));
        assert_eq!(pmt(0.05, 0.0, 100.0, 0.0, false), Err(SheetError::DivisionByZero));
    }
}
