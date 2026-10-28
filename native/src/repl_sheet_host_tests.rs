use super::*;
use super::sheet_host::ReplHost;
use vibe::{Span, Value};
fn grants() -> Vec<String> { ["capability.invoke", "Sheet.stats", "Sheet.sum_range", "Sheet.set_cell", "Sheet.eval_formula", "Sheet.create"].into_iter().map(str::to_string).collect() }
fn call(host: &mut ReplHost, session: &mut SourceSession, src: &str) -> Result<SubmissionResult, SubmissionError> {
    host.begin_submission(); eval_session_line(src, host, session)
}
#[test]
fn unsupported_receipt_only_sheet_never_returns_local_echo() {
    let g=grants(); let mut host=ReplHost::new(g.clone()); let mut session=SourceSession::new(g, SessionLimits::default());
    let src=r#"requires [capability("capability.invoke"), capability("Sheet.create")]; capability.invoke("Sheet.create",{});"#;
    assert!(call(&mut host,&mut session,src).is_err());
    assert!(host.capability_invoke("Sheet.create", &Value::Null, Span::point(0)).is_err());
}
#[test]
fn ordinary_repl_arithmetic_still_works() {
    let mut host=ReplHost::new(vec![]); let mut session=SourceSession::new(vec![], SessionLimits::default());
    assert_eq!(call(&mut host,&mut session,"= 1 + 2").unwrap().value,Value::I64(3));
}
#[cfg(not(feature = "native-graphical"))]
#[test]
fn lightweight_repl_refuses_even_known_sheet_instead_of_echoing() {
    let g=grants(); let mut host=ReplHost::new(g.clone()); let mut session=SourceSession::new(g,SessionLimits::default());
    let src=r#"requires [capability("capability.invoke"), capability("Sheet.stats")]; capability.invoke("Sheet.stats",{grid:[[1,2]]});"#;
    assert!(call(&mut host,&mut session,src).is_err());
}
#[cfg(feature = "native-graphical")]
#[test]
fn native_repl_calls_existing_sheet_owner_by_dotted_and_capability_paths() {
    let g=grants(); let mut host=ReplHost::new(g.clone()); let mut session=SourceSession::new(g,SessionLimits::default());
    let direct=r#"requires [capability("Sheet.stats")]; Sheet.stats({grid:[[1,2],[3,4]]});"#;
    let Value::Record(r)=call(&mut host,&mut session,direct).unwrap().value else { panic!("record") };
    assert_eq!(r.get("sum"),Some(&Value::F64(10.0)));
    for (src, sum) in [
        (r#"requires [capability("Sheet.stats")]; Sheet.stats([1,2]);"#, 3.0),
        (r#"requires [capability("Sheet.stats")]; Sheet.stats([[1,2],[3,4]]);"#, 10.0),
    ] {
        let Value::Record(r)=call(&mut host,&mut session,src).unwrap().value else { panic!("record") };
        assert_eq!(r.get("sum"),Some(&Value::F64(sum)));
    }
    let indirect=r#"requires [capability("capability.invoke"), capability("Sheet.eval_formula")]; capability.invoke("Sheet.eval_formula",{formula:"=SUM(A1:B2)*1.15",grid:[[1,2],[3,4]]});"#;
    let Value::Record(r)=call(&mut host,&mut session,indirect).unwrap().value else { panic!("record") };
    assert_eq!(r.get("value"),Some(&Value::F64(11.5)));
    let edit=r#"requires [capability("Sheet.set_cell")]; Sheet.set_cell({grid:[[1,2]],row:0,col:1,value:7});"#;
    assert_eq!(call(&mut host,&mut session,edit).unwrap().value,Value::List(vec![Value::List(vec![Value::I64(1),Value::I64(7)])]));
    let no_target=vec!["capability.invoke".into()]; let mut denied=ReplHost::new(no_target.clone()); let mut s=SourceSession::new(no_target,SessionLimits::default());
    let dynamic=r#"requires [capability("capability.invoke")]; let id="Sheet.stats"; capability.invoke(id,{grid:[[1]]});"#;
    assert!(call(&mut denied,&mut s,dynamic).is_err());
    let huge=Value::List(vec![Value::I64(0); 33]);
    let malformed=Value::Record(std::collections::BTreeMap::from([("t".into(),huge.clone()),("value".into(),huge)]));
    assert!(host.capability_invoke("Sheet.stats",&malformed,Span::point(0)).is_err());
}

#[cfg(feature = "native-graphical")]
#[test]
fn positional_stats_wrapper_keeps_raw_nonfinite_numeric_semantics() {
    let g=grants(); let mut host=ReplHost::new(g);
    for n in [f64::INFINITY, f64::NAN, -0.0] {
        let values=Value::List(vec![Value::F64(n)]);
        let wrapped=Value::Record(std::collections::BTreeMap::from([("t".into(),values.clone()),("value".into(),values.clone())]));
        let raw=host.capability_invoke("Sheet.stats",&values,Span::point(0)).unwrap();
        let dotted=host.capability_invoke("Sheet.stats",&wrapped,Span::point(0)).unwrap();
        let (Value::Record(raw),Value::Record(dotted))=(raw,dotted) else { panic!("records") };
        let (Some(Value::F64(a)),Some(Value::F64(b)))=(raw.get("sum"),dotted.get("sum")) else { panic!("numeric sums") };
        assert!((a.is_nan() && b.is_nan()) || a.to_bits()==b.to_bits());
    }
}
