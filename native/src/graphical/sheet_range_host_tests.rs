//! Public Qualia Sheet operators through the native Poet host; no JSON or UI claim.
use qualia_core_db::poet_host::PoetSnapshot;
use std::collections::BTreeMap;
use vibe::Value;

fn rows(values: &[&[f64]]) -> Value {
    Value::List(values.iter().map(|row| Value::List(row.iter().copied().map(Value::F64).collect())).collect())
}
fn range(values: Value, address: &str) -> Value {
    Value::Record(BTreeMap::from([("grid".into(), values), ("range".into(), Value::String(address.into()))]))
}

#[test]
fn native_sheet_operators_bound_requested_range_to_supplied_cells() {
    let mut host = PoetSnapshot::default();
    let grid = rows(&[&[1.0, 2.0], &[3.0], &[], &[4.0, 5.0]]);
    let stats = host.invoke_id("Sheet.stats", grid.clone()).unwrap();
    let Value::Record(stats) = stats else { panic!("stats record") };
    assert_eq!(stats.get("count"), Some(&Value::U64(5)));
    assert_eq!(stats.get("sum"), Some(&Value::F64(15.0)));
    assert_eq!(stats.get("mean"), Some(&Value::F64(3.0)));
    let huge = format!("A1:ZZZZZZ{}", usize::MAX);
    let Value::Record(sum) = host.invoke_id("Sheet.sum_range", range(grid.clone(), &huge)).unwrap() else { panic!("sum record") };
    assert_eq!(sum.get("sum"), Some(&Value::F64(15.0)));
    assert_eq!(sum.get("count"), Some(&Value::U64(5)));
    let Value::Record(reversed) = host.invoke_id("Sheet.sum_range", range(grid.clone(), "B4:A2")).unwrap() else { panic!("sum record") };
    assert_eq!(reversed.get("sum"), Some(&Value::F64(12.0)));
    assert_eq!(reversed.get("count"), Some(&Value::U64(3)));
    let Value::Record(disjoint) = host.invoke_id("Sheet.sum_range", range(grid.clone(), "ZZ999:ZZ1000")).unwrap() else { panic!("sum record") };
    assert_eq!(disjoint.get("sum"), Some(&Value::F64(0.0)));
    assert_eq!(disjoint.get("count"), Some(&Value::U64(0)));
    let overflowing = format!("A1:{}1", "Z".repeat(64));
    assert!(host.invoke_id("Sheet.sum_range", range(grid, &overflowing)).is_err());
}
