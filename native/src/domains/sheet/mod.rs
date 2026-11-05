//! Clean-room spreadsheet domain engine: 112 canonical functions, cycle detection, and protection.

pub mod eval_database;
pub mod eval_datetime;
pub mod eval_financial;
pub mod protection;

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CellCoord {
    pub row: u32,
    pub col: u32,
}

impl CellCoord {
    pub const fn new(row: u32, col: u32) -> Self { Self { row, col } }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SheetError {
    CycleDetected,
    ProtectedCellRefused,
    DivisionByZero,
    InvalidArgument,
    ValueError,
    NameError,
}

impl fmt::Display for SheetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CycleDetected => write!(f, "#CYCLE! Topological dependency cycle detected"),
            Self::ProtectedCellRefused => write!(f, "#PROTECTED! Cell or range modification refused"),
            Self::DivisionByZero => write!(f, "#DIV/0! Division by zero"),
            Self::InvalidArgument => write!(f, "#ARG! Invalid function argument"),
            Self::ValueError => write!(f, "#VALUE! Value conversion or evaluation error"),
            Self::NameError => write!(f, "#NAME? Unrecognized formula or identifier"),
        }
    }
}

impl std::error::Error for SheetError {}

#[derive(Debug, Clone, PartialEq)]
pub enum CellValue {
    Number(f64),
    Text(String),
    Bool(bool),
    Error(SheetError),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FunctionCategory {
    MathTrig,
    Statistical,
    Logical,
    Text,
    Lookup,
    Financial,
    DateTime,
    Database,
}

pub struct FormulaCatalog;

const CANONICAL_CATALOG: &[(&str, FunctionCategory)] = &[
    // Math & Trig (27)
    ("ABS", FunctionCategory::MathTrig), ("ACOS", FunctionCategory::MathTrig),
    ("ASIN", FunctionCategory::MathTrig), ("ATAN", FunctionCategory::MathTrig),
    ("ATAN2", FunctionCategory::MathTrig), ("CEILING", FunctionCategory::MathTrig),
    ("COS", FunctionCategory::MathTrig), ("DEGREES", FunctionCategory::MathTrig),
    ("EXP", FunctionCategory::MathTrig), ("FACT", FunctionCategory::MathTrig),
    ("FLOOR", FunctionCategory::MathTrig), ("LN", FunctionCategory::MathTrig),
    ("LOG", FunctionCategory::MathTrig), ("LOG10", FunctionCategory::MathTrig),
    ("MOD", FunctionCategory::MathTrig), ("PI", FunctionCategory::MathTrig),
    ("POWER", FunctionCategory::MathTrig), ("RADIANS", FunctionCategory::MathTrig),
    ("ROUND", FunctionCategory::MathTrig), ("ROUNDDOWN", FunctionCategory::MathTrig),
    ("ROUNDUP", FunctionCategory::MathTrig), ("SIGN", FunctionCategory::MathTrig),
    ("SIN", FunctionCategory::MathTrig), ("SQRT", FunctionCategory::MathTrig),
    ("SUM", FunctionCategory::MathTrig), ("SUMIF", FunctionCategory::MathTrig),
    ("TAN", FunctionCategory::MathTrig),
    // Statistical (18)
    ("AVERAGE", FunctionCategory::Statistical), ("AVERAGEA", FunctionCategory::Statistical),
    ("AVERAGEIF", FunctionCategory::Statistical), ("COUNT", FunctionCategory::Statistical),
    ("COUNTA", FunctionCategory::Statistical), ("COUNTBLANK", FunctionCategory::Statistical),
    ("COUNTIF", FunctionCategory::Statistical), ("COUNTIFS", FunctionCategory::Statistical),
    ("LARGE", FunctionCategory::Statistical), ("MAX", FunctionCategory::Statistical),
    ("MAXA", FunctionCategory::Statistical), ("MEDIAN", FunctionCategory::Statistical),
    ("MIN", FunctionCategory::Statistical), ("MINA", FunctionCategory::Statistical),
    ("MODE", FunctionCategory::Statistical), ("RANK", FunctionCategory::Statistical),
    ("SMALL", FunctionCategory::Statistical), ("STDEV", FunctionCategory::Statistical),
    // Logical (9)
    ("AND", FunctionCategory::Logical), ("FALSE", FunctionCategory::Logical),
    ("IF", FunctionCategory::Logical), ("IFERROR", FunctionCategory::Logical),
    ("NOT", FunctionCategory::Logical), ("OR", FunctionCategory::Logical),
    ("SWITCH", FunctionCategory::Logical), ("TRUE", FunctionCategory::Logical),
    ("XOR", FunctionCategory::Logical),
    // Text (16)
    ("CHAR", FunctionCategory::Text), ("CLEAN", FunctionCategory::Text),
    ("CONCAT", FunctionCategory::Text), ("CONCATENATE", FunctionCategory::Text),
    ("EXACT", FunctionCategory::Text), ("FIND", FunctionCategory::Text),
    ("LEFT", FunctionCategory::Text), ("LEN", FunctionCategory::Text),
    ("LOWER", FunctionCategory::Text), ("MID", FunctionCategory::Text),
    ("REPLACE", FunctionCategory::Text), ("RIGHT", FunctionCategory::Text),
    ("SEARCH", FunctionCategory::Text), ("SUBSTITUTE", FunctionCategory::Text),
    ("TRIM", FunctionCategory::Text), ("UPPER", FunctionCategory::Text),
    // Lookup & Reference (10)
    ("CHOOSE", FunctionCategory::Lookup), ("COLUMN", FunctionCategory::Lookup),
    ("COLUMNS", FunctionCategory::Lookup), ("HLOOKUP", FunctionCategory::Lookup),
    ("INDEX", FunctionCategory::Lookup), ("LOOKUP", FunctionCategory::Lookup),
    ("MATCH", FunctionCategory::Lookup), ("ROW", FunctionCategory::Lookup),
    ("ROWS", FunctionCategory::Lookup), ("VLOOKUP", FunctionCategory::Lookup),
    // Financial (12)
    ("DB", FunctionCategory::Financial), ("FV", FunctionCategory::Financial),
    ("IPMT", FunctionCategory::Financial), ("IRR", FunctionCategory::Financial),
    ("NPER", FunctionCategory::Financial), ("NPV", FunctionCategory::Financial),
    ("PMT", FunctionCategory::Financial), ("PPMT", FunctionCategory::Financial),
    ("PV", FunctionCategory::Financial), ("RATE", FunctionCategory::Financial),
    ("SLN", FunctionCategory::Financial), ("SYD", FunctionCategory::Financial),
    // Date & Time (12)
    ("DATE", FunctionCategory::DateTime), ("DATEDIF", FunctionCategory::DateTime),
    ("DAY", FunctionCategory::DateTime), ("DAYS", FunctionCategory::DateTime),
    ("EDATE", FunctionCategory::DateTime), ("EOMONTH", FunctionCategory::DateTime),
    ("HOUR", FunctionCategory::DateTime), ("MINUTE", FunctionCategory::DateTime),
    ("MONTH", FunctionCategory::DateTime), ("NETWORKDAYS", FunctionCategory::DateTime),
    ("NOW", FunctionCategory::DateTime), ("TODAY", FunctionCategory::DateTime),
    // Database (8)
    ("DAVERAGE", FunctionCategory::Database), ("DCOUNT", FunctionCategory::Database),
    ("DCOUNTA", FunctionCategory::Database), ("DGET", FunctionCategory::Database),
    ("DMAX", FunctionCategory::Database), ("DMIN", FunctionCategory::Database),
    ("DPRODUCT", FunctionCategory::Database), ("DSUM", FunctionCategory::Database),
];

impl FormulaCatalog {
    pub const fn catalog_function_count() -> usize { CANONICAL_CATALOG.len() }
    pub fn is_registered(name: &str) -> bool { Self::lookup(name).is_some() }
    pub fn lookup(name: &str) -> Option<FunctionCategory> {
        let upper = name.to_ascii_uppercase();
        for &(func, cat) in CANONICAL_CATALOG {
            if func == upper { return Some(cat); }
        }
        None
    }
    pub fn entries() -> &'static [(&'static str, FunctionCategory)] { CANONICAL_CATALOG }
}

pub fn catalog_function_count() -> usize { FormulaCatalog::catalog_function_count() }

/// Iterative cycle detector using Kahn's algorithm (zero unbounded recursion).
pub fn detect_formula_cycle(edges: &[(CellCoord, CellCoord)]) -> Result<(), SheetError> {
    if edges.is_empty() { return Ok(()); }
    use std::collections::{HashMap, VecDeque};
    let mut adj: HashMap<CellCoord, Vec<CellCoord>> = HashMap::new();
    let mut in_degree: HashMap<CellCoord, usize> = HashMap::new();

    for &(from, to) in edges {
        if from == to { return Err(SheetError::CycleDetected); }
        adj.entry(from).or_default().push(to);
        in_degree.entry(from).or_insert(0);
        *in_degree.entry(to).or_insert(0) += 1;
    }

    let mut queue = VecDeque::new();
    for (&node, &deg) in &in_degree {
        if deg == 0 { queue.push_back(node); }
    }

    let total_nodes = in_degree.len();
    let mut visited_count = 0;

    while let Some(node) = queue.pop_front() {
        visited_count += 1;
        if let Some(neighbors) = adj.get(&node) {
            for &next in neighbors {
                if let Some(deg) = in_degree.get_mut(&next) {
                    *deg -= 1;
                    if *deg == 0 { queue.push_back(next); }
                }
            }
        }
    }

    if visited_count == total_nodes { Ok(()) } else { Err(SheetError::CycleDetected) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_catalog_112_functions() {
        assert_eq!(FormulaCatalog::catalog_function_count(), 112);
        assert_eq!(catalog_function_count(), 112);
        assert!(FormulaCatalog::is_registered("PMT"));
        assert!(FormulaCatalog::is_registered("datedif"));
        assert!(FormulaCatalog::is_registered("DSUM"));
        assert!(!FormulaCatalog::is_registered("NON_EXISTENT_FUNC"));
    }

    #[test]
    fn test_cycle_detection_positive() {
        let edges = [
            (CellCoord::new(0, 0), CellCoord::new(0, 1)),
            (CellCoord::new(0, 1), CellCoord::new(0, 2)),
        ];
        assert_eq!(detect_formula_cycle(&edges), Ok(()));
    }

    #[test]
    fn test_cycle_detection_negative_direct() {
        let a1 = CellCoord::new(0, 0);
        let b1 = CellCoord::new(0, 1);
        let edges = [(a1, b1), (b1, a1)];
        assert_eq!(detect_formula_cycle(&edges), Err(SheetError::CycleDetected));
    }

    #[test]
    fn test_cycle_detection_self_loop() {
        let a1 = CellCoord::new(0, 0);
        assert_eq!(detect_formula_cycle(&[(a1, a1)]), Err(SheetError::CycleDetected));
    }
}
