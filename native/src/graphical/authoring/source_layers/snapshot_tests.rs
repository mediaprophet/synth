use super::*;
use std::collections::BTreeMap;
fn fixture() -> Value {
    Value::Record(BTreeMap::from([
        ("exact".into(), Value::U64(9007199254740993)),
        ("zero".into(), Value::F64(-0.0)),
        ("text".into(), Value::String("source-owned".into())),
        (
            "items".into(),
            Value::List(vec![Value::I64(-2), Value::Bool(true)]),
        ),
    ]))
}
#[test]
fn exact_byte_boundary_and_caller_selected_allowance_are_enforced_before_copy() {
    let value = fixture();
    let required = cost(Some(&value), Some(&value)).unwrap();
    let error = validation_snapshot_with_budget(Some(&value), Some(&value), required.bytes - 1)
        .unwrap_err();
    assert!(error.contains(&format!(
        "requires {} budgeted structural bytes",
        required.bytes
    )));
    assert!(error.contains(&format!("host admits {} bytes", required.bytes - 1)));
    assert!(error.contains(&format!(
        "observed {} Value nodes and {} text bytes",
        required.nodes, required.text_bytes
    )));
    let (parts, base) =
        validation_snapshot_with_budget(Some(&value), Some(&value), required.bytes).unwrap();
    assert!(super::super::super::parameter_safety::equal(&value, parts.as_ref().unwrap()).unwrap());
    assert!(super::super::super::parameter_safety::equal(&value, base.as_ref().unwrap()).unwrap());
}
#[test]
fn legacy_wrapper_preserves_64_mib_profile_and_exact_value_types() {
    assert_eq!(LEGACY_BYTES, 64 * 1024 * 1024);
    let value = fixture();
    let result = validation_snapshot(Some(&value), None).unwrap();
    let Value::Record(copy) = result.0.unwrap() else {
        panic!()
    };
    assert_eq!(copy["exact"], Value::U64(9007199254740993));
    assert!(matches!(copy["zero"],Value::F64(v) if v.to_bits()==(-0.0f64).to_bits()));
    assert!(result.1.is_none());
}
#[test]
fn explicit_byte_budget_does_not_waive_depth_or_text_profiles() {
    let mut deep = Value::Null;
    for _ in 0..70 {
        deep = Value::List(vec![deep]);
    }
    assert!(
        validation_snapshot_with_budget(Some(&deep), None, 128 * 1024 * 1024)
            .unwrap_err()
            .contains("nesting bound")
    );
    let text = Value::String("x".repeat(1024 * 1024 + 1));
    assert!(
        validation_snapshot_with_budget(Some(&text), None, 128 * 1024 * 1024)
            .unwrap_err()
            .contains("text bound")
    );
    assert!(validation_snapshot_with_budget(None, None, 0)
        .unwrap_err()
        .contains("admitted 0 bytes"));
}
