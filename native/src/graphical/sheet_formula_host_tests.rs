//! Real public Sheet formula capability through the native Poet host.
use qualia_core_db::poet_host::PoetSnapshot;
use std::collections::BTreeMap;
use vibe::Value;

fn request(formula: &str, grid: Option<Value>) -> Value {
    let mut fields = BTreeMap::from([("formula".into(), Value::String(formula.into()))]);
    if let Some(grid) = grid { fields.insert("grid".into(), grid); }
    Value::Record(fields)
}

#[test]
fn native_sheet_formula_uses_real_bounded_craft_evaluation() {
    let mut host = PoetSnapshot::default();
    let grid = Value::List(vec![
        Value::List(vec![Value::I64(1),Value::I64(2)]),
        Value::List(vec![Value::I64(3),Value::I64(4)]),
    ]);
    let Value::Record(out) = host.invoke_id("Sheet.eval_formula", request("=SUM(A1:B2)*1.15", Some(grid))).unwrap() else { panic!("record") };
    assert_eq!(out.get("value"), Some(&Value::F64(11.5)));
    assert_eq!(out.get("result_type"), Some(&Value::String("number".into())));
    assert_eq!(out.get("cached"), Some(&Value::Bool(false)));
    let Value::Record(error) = host.invoke_id("Sheet.eval_formula", request("=1/0", None)).unwrap() else { panic!("record") };
    assert_eq!(error.get("value"), Some(&Value::String("#DIV/0!".into())));
    for formula in ["=NOW()", "=SUM(A1:ZZ99999)", "=SUM(A1:B2)"] {
        assert!(host.invoke_id("Sheet.eval_formula", request(formula, None)).is_err());
    }
}
