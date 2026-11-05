//! Clean-room spreadsheet database criteria range evaluators.

use super::{CellValue, SheetError};

#[derive(Debug, Clone, PartialEq)]
pub enum CriteriaPredicate {
    Equals(CellValue),
    NotEquals(CellValue),
    GreaterThan(f64),
    LessThan(f64),
    GreaterOrEqual(f64),
    LessOrEqual(f64),
}

impl CriteriaPredicate {
    pub fn matches(&self, value: &CellValue) -> bool {
        match self {
            Self::Equals(expected) => match (expected, value) {
                (CellValue::Number(a), CellValue::Number(b)) => (a - b).abs() < 1e-9,
                (CellValue::Text(a), CellValue::Text(b)) => a.eq_ignore_ascii_case(b),
                (CellValue::Bool(a), CellValue::Bool(b)) => a == b,
                (CellValue::Error(a), CellValue::Error(b)) => a == b,
                _ => false,
            },
            Self::NotEquals(expected) => !Self::Equals(expected.clone()).matches(value),
            Self::GreaterThan(threshold) => match value {
                CellValue::Number(n) => n > threshold,
                _ => false,
            },
            Self::LessThan(threshold) => match value {
                CellValue::Number(n) => n < threshold,
                _ => false,
            },
            Self::GreaterOrEqual(threshold) => match value {
                CellValue::Number(n) => n >= threshold,
                _ => false,
            },
            Self::LessOrEqual(threshold) => match value {
                CellValue::Number(n) => n <= threshold,
                _ => false,
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Criterion {
    pub field_name: String,
    pub predicate: CriteriaPredicate,
}

impl Criterion {
    pub fn new(field_name: impl Into<String>, predicate: CriteriaPredicate) -> Self {
        Self {
            field_name: field_name.into(),
            predicate,
        }
    }
}

pub struct DatabaseTable<'a> {
    pub headers: &'a [&'a str],
    pub records: &'a [&'a [CellValue]],
}

impl<'a> DatabaseTable<'a> {
    pub fn new(headers: &'a [&'a str], records: &'a [&'a [CellValue]]) -> Self {
        Self { headers, records }
    }

    pub fn column_index(&self, field: &str) -> Option<usize> {
        self.headers.iter().position(|&h| h.eq_ignore_ascii_case(field))
    }

    pub fn matches_row(&self, row: &[CellValue], criteria: &[Criterion]) -> bool {
        for crit in criteria {
            if let Some(col_idx) = self.column_index(&crit.field_name) {
                if let Some(cell) = row.get(col_idx) {
                    if !crit.predicate.matches(cell) {
                        return false;
                    }
                } else {
                    return false;
                }
            } else {
                return false;
            }
        }
        true
    }
}

/// Computes the sum of values in a field for records matching criteria.
pub fn dsum(
    table: &DatabaseTable,
    field: &str,
    criteria: &[Criterion],
) -> Result<f64, SheetError> {
    let col = table.column_index(field).ok_or(SheetError::NameError)?;
    let mut sum = 0.0;
    for &row in table.records {
        if table.matches_row(row, criteria) {
            if let Some(CellValue::Number(val)) = row.get(col) {
                sum += val;
            }
        }
    }
    Ok(sum)
}

/// Computes the average of values in a field for records matching criteria.
pub fn daverage(
    table: &DatabaseTable,
    field: &str,
    criteria: &[Criterion],
) -> Result<f64, SheetError> {
    let col = table.column_index(field).ok_or(SheetError::NameError)?;
    let mut sum = 0.0;
    let mut count = 0;
    for &row in table.records {
        if table.matches_row(row, criteria) {
            if let Some(CellValue::Number(val)) = row.get(col) {
                sum += val;
                count += 1;
            }
        }
    }
    if count == 0 {
        return Err(SheetError::DivisionByZero);
    }
    Ok(sum / (count as f64))
}

/// Counts cells containing numbers in a field for records matching criteria.
pub fn dcount(
    table: &DatabaseTable,
    field: &str,
    criteria: &[Criterion],
) -> Result<usize, SheetError> {
    let col = table.column_index(field).ok_or(SheetError::NameError)?;
    let mut count = 0;
    for &row in table.records {
        if table.matches_row(row, criteria) {
            if let Some(CellValue::Number(_)) = row.get(col) {
                count += 1;
            }
        }
    }
    Ok(count)
}

/// Finds the minimum numeric value in a field for records matching criteria.
pub fn dmin(
    table: &DatabaseTable,
    field: &str,
    criteria: &[Criterion],
) -> Result<f64, SheetError> {
    let col = table.column_index(field).ok_or(SheetError::NameError)?;
    let mut min_val: Option<f64> = None;
    for &row in table.records {
        if table.matches_row(row, criteria) {
            if let Some(CellValue::Number(val)) = row.get(col) {
                min_val = Some(min_val.map_or(*val, |m| m.min(*val)));
            }
        }
    }
    min_val.ok_or(SheetError::ValueError)
}

/// Finds the maximum numeric value in a field for records matching criteria.
pub fn dmax(
    table: &DatabaseTable,
    field: &str,
    criteria: &[Criterion],
) -> Result<f64, SheetError> {
    let col = table.column_index(field).ok_or(SheetError::NameError)?;
    let mut max_val: Option<f64> = None;
    for &row in table.records {
        if table.matches_row(row, criteria) {
            if let Some(CellValue::Number(val)) = row.get(col) {
                max_val = Some(max_val.map_or(*val, |m| m.max(*val)));
            }
        }
    }
    max_val.ok_or(SheetError::ValueError)
}

/// Retrieves a single value from a specified field of a single record matching criteria.
pub fn dget(
    table: &DatabaseTable,
    field: &str,
    criteria: &[Criterion],
) -> Result<CellValue, SheetError> {
    let col = table.column_index(field).ok_or(SheetError::NameError)?;
    let mut matched_val: Option<CellValue> = None;
    for &row in table.records {
        if table.matches_row(row, criteria) {
            if matched_val.is_some() {
                // Multiple matches: standard spreadsheet #NUM! error
                return Err(SheetError::ValueError);
            }
            if let Some(val) = row.get(col) {
                matched_val = Some(val.clone());
            }
        }
    }
    matched_val.ok_or(SheetError::ValueError)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_evaluators() {
        let headers = ["Tree", "Height", "Age", "Yield", "Profit"];
        let row1 = [
            CellValue::Text("Apple".into()),
            CellValue::Number(18.0),
            CellValue::Number(20.0),
            CellValue::Number(14.0),
            CellValue::Number(105.0),
        ];
        let row2 = [
            CellValue::Text("Pear".into()),
            CellValue::Number(12.0),
            CellValue::Number(12.0),
            CellValue::Number(10.0),
            CellValue::Number(96.0),
        ];
        let row3 = [
            CellValue::Text("Apple".into()),
            CellValue::Number(15.0),
            CellValue::Number(16.0),
            CellValue::Number(12.0),
            CellValue::Number(75.0),
        ];

        let records: [&[CellValue]; 3] = [&row1, &row2, &row3];
        let table = DatabaseTable::new(&headers, &records);

        let criteria = [Criterion::new("Tree", CriteriaPredicate::Equals(CellValue::Text("Apple".into())))];

        // Apple profit: 105 + 75 = 180
        assert_eq!(dsum(&table, "Profit", &criteria).unwrap(), 180.0);
        // Apple average height: (18 + 15) / 2 = 16.5
        assert_eq!(daverage(&table, "Height", &criteria).unwrap(), 16.5);
        // Apple count
        assert_eq!(dcount(&table, "Age", &criteria).unwrap(), 2);
        // Apple min height
        assert_eq!(dmin(&table, "Height", &criteria).unwrap(), 15.0);
        // Apple max profit
        assert_eq!(dmax(&table, "Profit", &criteria).unwrap(), 105.0);
    }

    #[test]
    fn test_dget_positive_and_negative() {
        let headers = ["Name", "Score"];
        let row1 = [CellValue::Text("Alice".into()), CellValue::Number(95.0)];
        let row2 = [CellValue::Text("Bob".into()), CellValue::Number(82.0)];
        let row3 = [CellValue::Text("Alice".into()), CellValue::Number(88.0)];

        let records: [&[CellValue]; 3] = [&row1, &row2, &row3];
        let table = DatabaseTable::new(&headers, &records);

        // Unique match for Bob
        let crit_bob = [Criterion::new("Name", CriteriaPredicate::Equals(CellValue::Text("Bob".into())))];
        assert_eq!(dget(&table, "Score", &crit_bob).unwrap(), CellValue::Number(82.0));

        // Multiple matches for Alice -> ValueError
        let crit_alice = [Criterion::new("Name", CriteriaPredicate::Equals(CellValue::Text("Alice".into())))];
        assert_eq!(dget(&table, "Score", &crit_alice), Err(SheetError::ValueError));

        // No match for Charlie -> ValueError
        let crit_charlie = [Criterion::new("Name", CriteriaPredicate::Equals(CellValue::Text("Charlie".into())))];
        assert_eq!(dget(&table, "Score", &crit_charlie), Err(SheetError::ValueError));
    }
}
