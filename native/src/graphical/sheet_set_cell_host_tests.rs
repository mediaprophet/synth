//! Public typed Sheet.set_cell capability; no persistence representation.
use qualia_core_db::poet_host::PoetSnapshot;
use vibe::Value;
use std::collections::BTreeMap;
fn request(grid: Value, row: u64, col: u64, value: Value) -> Value {
    Value::Record(BTreeMap::from([("grid".into(), grid), ("row".into(), Value::U64(row)), ("col".into(), Value::U64(col)), ("value".into(), value)]))
}
#[test]
fn native_sheet_set_cell_preserves_other_numeric_values_and_rejects_invalid_edits() {
    let mut host = PoetSnapshot::default();
    let grid = Value::List(vec![Value::List(vec![Value::U64(u64::MAX), Value::I64(i64::MIN)]), Value::List(vec![Value::F64(-0.0),Value::F64(5.0)])]);
    let result = host.invoke_id("Sheet.set_cell", request(grid.clone(), 1, 1, Value::I64(7))).unwrap();
    let Value::List(rows) = result else { panic!("grid") };
    let Value::List(first) = &rows[0] else { panic!("row") };
    let Value::List(second) = &rows[1] else { panic!("row") };
    assert_eq!(first, &vec![Value::U64(u64::MAX),Value::I64(i64::MIN)]);
    assert!(matches!(&second[0], Value::F64(n) if n.to_bits() == (-0.0f64).to_bits()));
    assert_eq!(second[1], Value::I64(7));
    assert!(host.invoke_id("Sheet.set_cell", request(grid.clone(), 2, 0, Value::I64(1))).is_err());
    assert!(host.invoke_id("Sheet.set_cell", request(grid, 0, 0, Value::F64(f64::NAN))).is_err());
    let ragged = Value::List(vec![Value::List(vec![Value::I64(1)]),Value::List(vec![])]);
    assert!(host.invoke_id("Sheet.set_cell", request(ragged, 0, 0, Value::I64(1))).is_err());
    let empty = Value::List(vec![]);
    assert!(host.invoke_id("Sheet.set_cell", request(empty, 0, 0, Value::I64(1))).is_err());
    let wide = Value::List(vec![Value::List(vec![Value::I64(1); 33])]);
    assert!(host.invoke_id("Sheet.set_cell", request(wide, 0, 0, Value::I64(1))).is_err());
    let text = Value::List(vec![Value::List(vec![Value::String("bad".into())])]);
    assert!(host.invoke_id("Sheet.set_cell", request(text, 0, 0, Value::I64(1))).is_err());
    let infinite = Value::List(vec![Value::List(vec![Value::F64(f64::INFINITY)])]);
    assert!(host.invoke_id("Sheet.set_cell", request(infinite, 0, 0, Value::I64(1))).is_err());
    let bad_coord = Value::Record(BTreeMap::from([("grid".into(), Value::List(vec![Value::List(vec![Value::I64(1)])])), ("row".into(), Value::I64(-1)), ("col".into(), Value::U64(0)), ("value".into(), Value::I64(1))]));
    assert!(host.invoke_id("Sheet.set_cell", bad_coord).is_err());
    let oversized = Value::List(vec![Value::List(vec![Value::I64(1)]); 33]);
    assert!(host.invoke_id("Sheet.set_cell", request(oversized, 0, 0, Value::I64(1))).is_err());
}
