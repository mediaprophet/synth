//! Clean-room proleptic Gregorian date and time evaluators.

use super::SheetError;

pub const SERIAL_EPOCH_OFFSET: i64 = 25568; // 1900-01-01 is serial 1

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DateDifUnit {
    Y,  // Complete years
    M,  // Complete months
    D,  // Complete days
    MD, // Day difference ignoring months and years
    YM, // Month difference ignoring years
    YD, // Day difference ignoring years
}

pub fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

pub fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => if is_leap_year(year) { 29 } else { 28 },
        _ => 0,
    }
}

fn civil_to_days(y: i32, m: u32, d: u32) -> i64 {
    let y = y - if m <= 2 { 1 } else { 0 };
    let era = (if y >= 0 { y } else { y - 399 }) / 400;
    let yoe = (y - era * 400) as u32;
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    (era as i64) * 146097 + (doe as i64) - 719468
}

fn days_to_civil(z: i64) -> (i32, u32, u32) {
    let z = z + 719468;
    let era = (if z >= 0 { z } else { z - 146096 }) / 146097;
    let doe = (z - era * 146097) as u32;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = (yoe as i64) + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = y + if m <= 2 { 1 } else { 0 };
    (y as i32, m, d)
}

/// Converts year, month, day to spreadsheet serial day (1900-01-01 = 1).
pub fn ymd_to_serial(year: i32, month: u32, day: u32) -> Result<i64, SheetError> {
    if month < 1 || month > 12 {
        return Err(SheetError::InvalidArgument);
    }
    let max_day = days_in_month(year, month);
    if day < 1 || day > max_day {
        return Err(SheetError::InvalidArgument);
    }
    let civil_days = civil_to_days(year, month, day);
    Ok(civil_days + SERIAL_EPOCH_OFFSET)
}

/// Converts spreadsheet serial day to proleptic Gregorian (year, month, day).
pub fn serial_to_ymd(serial: i64) -> Result<(i32, u32, u32), SheetError> {
    let civil_days = serial - SERIAL_EPOCH_OFFSET;
    Ok(days_to_civil(civil_days))
}

/// Computes the number of days between two serial dates.
pub fn days_between(start_serial: i64, end_serial: i64) -> i64 {
    end_serial - start_serial
}

/// Computes date difference according to the specified interval unit.
pub fn datedif(
    start_serial: i64,
    end_serial: i64,
    unit: DateDifUnit,
) -> Result<i64, SheetError> {
    if start_serial > end_serial {
        return Err(SheetError::InvalidArgument);
    }
    let (y1, m1, d1) = serial_to_ymd(start_serial)?;
    let (y2, m2, d2) = serial_to_ymd(end_serial)?;

    match unit {
        DateDifUnit::D => Ok(end_serial - start_serial),
        DateDifUnit::Y => {
            let mut years = (y2 - y1) as i64;
            if (m2, d2) < (m1, d1) {
                years -= 1;
            }
            Ok(years)
        }
        DateDifUnit::M => {
            let mut months = ((y2 - y1) as i64) * 12 + (m2 as i64 - m1 as i64);
            if d2 < d1 {
                months -= 1;
            }
            Ok(months)
        }
        DateDifUnit::YM => {
            let mut ym = m2 as i64 - m1 as i64;
            if d2 < d1 {
                ym -= 1;
            }
            if ym < 0 {
                ym += 12;
            }
            Ok(ym)
        }
        DateDifUnit::MD => {
            if d2 >= d1 {
                Ok((d2 - d1) as i64)
            } else {
                let prev_m = if m2 == 1 { 12 } else { m2 - 1 };
                let prev_y = if m2 == 1 { y2 - 1 } else { y2 };
                let dim = days_in_month(prev_y, prev_m) as i64;
                Ok(dim - (d1 as i64) + (d2 as i64))
            }
        }
        DateDifUnit::YD => {
            let target_year = if (m2, d2) < (m1, d1) { y1 + 1 } else { y1 };
            let target_serial = ymd_to_serial(target_year, m2, d2)?;
            Ok(target_serial - start_serial)
        }
    }
}

/// Adds or subtracts months to a serial date, preserving or clamping day of month.
pub fn edate(start_serial: i64, months_delta: i32) -> Result<i64, SheetError> {
    let (y, m, d) = serial_to_ymd(start_serial)?;
    let total_months = (y as i64) * 12 + (m as i64 - 1) + (months_delta as i64);
    let target_y = total_months.div_euclid(12) as i32;
    let target_m = (total_months.rem_euclid(12) + 1) as u32;
    let dim = days_in_month(target_y, target_m);
    let target_d = d.min(dim);
    ymd_to_serial(target_y, target_m, target_d)
}

/// Returns the serial date for the last day of the month after months_delta.
pub fn eomonth(start_serial: i64, months_delta: i32) -> Result<i64, SheetError> {
    let (y, m, _) = serial_to_ymd(start_serial)?;
    let total_months = (y as i64) * 12 + (m as i64 - 1) + (months_delta as i64);
    let target_y = total_months.div_euclid(12) as i32;
    let target_m = (total_months.rem_euclid(12) + 1) as u32;
    let target_d = days_in_month(target_y, target_m);
    ymd_to_serial(target_y, target_m, target_d)
}

fn weekday_mon_1_to_sun_7(serial: i64) -> u32 {
    let civil_days = serial - SERIAL_EPOCH_OFFSET;
    ((civil_days + 3).rem_euclid(7) + 1) as u32
}

/// Calculates working days between two dates, excluding weekends (Sat/Sun) and holidays.
pub fn networkdays(
    start_serial: i64,
    end_serial: i64,
    holidays: &[i64],
) -> Result<i64, SheetError> {
    if start_serial > end_serial {
        let neg = networkdays(end_serial, start_serial, holidays)?;
        return Ok(-neg);
    }
    let mut working_days = 0;
    for s in start_serial..=end_serial {
        let wd = weekday_mon_1_to_sun_7(s);
        if wd <= 5 && !holidays.contains(&s) {
            working_days += 1;
        }
    }
    Ok(working_days)
}

/// Converts hour, minute, second into a day fraction (0.0 .. 1.0).
pub fn hms_to_fraction(hour: u32, minute: u32, second: u32) -> Result<f64, SheetError> {
    if hour >= 24 || minute >= 60 || second >= 60 {
        return Err(SheetError::InvalidArgument);
    }
    Ok((hour as f64 * 3600.0 + minute as f64 * 60.0 + second as f64) / 86400.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ymd_serial_roundtrip() {
        let test_cases = [(1900, 1, 1), (2000, 2, 29), (2024, 2, 29), (2026, 10, 28)];
        for &(y, m, d) in &test_cases {
            let s = ymd_to_serial(y, m, d).expect("valid serial");
            let roundtrip = serial_to_ymd(s).expect("valid ymd");
            assert_eq!(roundtrip, (y, m, d));
        }
    }

    #[test]
    fn test_days_between_and_datedif() {
        let s1 = ymd_to_serial(2020, 1, 1).unwrap();
        let s2 = ymd_to_serial(2023, 7, 15).unwrap();
        assert_eq!(days_between(s1, s2), s2 - s1);

        assert_eq!(datedif(s1, s2, DateDifUnit::Y).unwrap(), 3);
        assert_eq!(datedif(s1, s2, DateDifUnit::M).unwrap(), 42);
        assert_eq!(datedif(s1, s2, DateDifUnit::YM).unwrap(), 6);
        assert_eq!(datedif(s1, s2, DateDifUnit::MD).unwrap(), 14);
    }

    #[test]
    fn test_edate_and_eomonth() {
        let s = ymd_to_serial(2024, 1, 31).unwrap();
        // Leap year February has 29 days, so 31 clamped to 29
        let ed = edate(s, 1).unwrap();
        assert_eq!(serial_to_ymd(ed).unwrap(), (2024, 2, 29));

        let eom = eomonth(s, 1).unwrap();
        assert_eq!(serial_to_ymd(eom).unwrap(), (2024, 2, 29));

        let eom2 = eomonth(s, 2).unwrap();
        assert_eq!(serial_to_ymd(eom2).unwrap(), (2024, 3, 31));
    }

    #[test]
    fn test_networkdays_and_holidays() {
        // Monday 2026-10-26 to Friday 2026-10-30 -> 5 working days
        let mon = ymd_to_serial(2026, 10, 26).unwrap();
        let fri = ymd_to_serial(2026, 10, 30).unwrap();
        assert_eq!(networkdays(mon, fri, &[]).unwrap(), 5);

        // Wednesday is holiday
        let wed = ymd_to_serial(2026, 10, 28).unwrap();
        assert_eq!(networkdays(mon, fri, &[wed]).unwrap(), 4);
    }

    #[test]
    fn test_hms_to_fraction() {
        assert_eq!(hms_to_fraction(12, 0, 0).unwrap(), 0.5);
        assert_eq!(hms_to_fraction(6, 0, 0).unwrap(), 0.25);
        assert_eq!(hms_to_fraction(24, 0, 0), Err(SheetError::InvalidArgument));
    }

    #[test]
    fn test_negative_dates() {
        assert_eq!(ymd_to_serial(2023, 2, 29), Err(SheetError::InvalidArgument));
        assert_eq!(ymd_to_serial(2023, 13, 1), Err(SheetError::InvalidArgument));
        let s1 = ymd_to_serial(2024, 1, 1).unwrap();
        let s2 = ymd_to_serial(2023, 1, 1).unwrap();
        assert_eq!(datedif(s1, s2, DateDifUnit::D), Err(SheetError::InvalidArgument));
    }
}
