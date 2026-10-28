//! Typed Vibe source through the native host into existing QualiaDB Sheet owners.
use super::*;
use vibe::session::{SessionLimits, SourceSession};
fn grants() -> Vec<String> { ["capability.invoke", "Sheet.stats", "Sheet.sum_range"].into_iter().map(str::to_owned).collect() }
#[test]
fn source_calls_stats_and_huge_range_over_actual_supplied_cells() {
 let g=grants(); let mut host=GeometryHost::new(g.clone(),200_000); let mut session=SourceSession::new(g,SessionLimits::default());
 let stats=r#"requires [capability("capability.invoke"),capability("Sheet.stats")]; capability.invoke("Sheet.stats",[[1,2],[3],[],[4,5]]);"#;
 let Value::Record(stats)=session.submit(stats,&mut host,vibe::Budget::default()).unwrap().value else { panic!("stats") };
 assert_eq!(stats.get("sum"),Some(&Value::F64(15.0))); assert_eq!(stats.get("count"),Some(&Value::U64(5)));
 host.begin_submission();
 let source=format!(r#"requires [capability("capability.invoke"),capability("Sheet.sum_range")]; capability.invoke("Sheet.sum_range",{{grid:[[1,2],[3],[],[4,5]],range:"A1:ZZZZZZ{}"}});"#,usize::MAX);
 let Value::Record(result)=session.submit(&source,&mut host,vibe::Budget::default()).unwrap().value else { panic!("range") };
 assert_eq!(result.get("sum"),Some(&Value::F64(15.0))); assert_eq!(result.get("count"),Some(&Value::U64(5)));
}
#[test]
fn source_denies_missing_target_and_obeys_work_dispatch_limits() {
 let g=vec!["capability.invoke".into()]; let mut host=GeometryHost::new(g.clone(),200_000); let mut session=SourceSession::new(g,SessionLimits::default());
 let dynamic=r#"requires [capability("capability.invoke")]; let op="Sheet.stats"; capability.invoke(op,[1,2]);"#;
 let error=session.submit(dynamic,&mut host,vibe::Budget::default()).unwrap_err(); assert_eq!(error.diagnostic.code,DiagCode::E300); assert!(error.execution_attempted);
 let g=grants(); let mut host=GeometryHost::with_limits(g.clone(),GeometryHostLimits{max_vertices:10,submission_work:4,dispatches:1,scratch_bytes:4096,profile_points:3}).unwrap(); let mut session=SourceSession::new(g,SessionLimits::default());
 let source=r#"requires [capability("capability.invoke"),capability("Sheet.stats")]; capability.invoke("Sheet.stats",[1,2]);"#;
 assert!(session.submit(source,&mut host,vibe::Budget::default()).is_ok());
 assert!(session.submit(source,&mut host,vibe::Budget::default()).is_err());
 host.begin_submission(); assert!(session.submit(source,&mut host,vibe::Budget::default()).is_ok());
 let g=grants(); let mut host=GeometryHost::with_limits(g.clone(),GeometryHostLimits{max_vertices:10,submission_work:3,dispatches:1,scratch_bytes:4096,profile_points:3}).unwrap(); let mut session=SourceSession::new(g,SessionLimits::default()); assert!(session.submit(source,&mut host,vibe::Budget::default()).is_err());
}
