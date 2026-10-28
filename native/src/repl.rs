//! VibeScript interactive REPL (T62).

use std::io::{self, BufRead, Write};
use vibe::session::{SessionLimits, SourceSession, SubmissionError, SubmissionResult};
#[cfg(test)]
use vibe::{check_cell, check_program, parse_cell, parse_program, Diagnostic, Engine, Env, Value};
use vibe::{Budget, Host, LocalHost};
#[path = "repl_sheet_host.rs"]
pub(crate) mod sheet_host;

/// Evaluate a single REPL line/cell within a persistent host and environment.
#[cfg(test)]
pub fn eval_line(line: &str, host: &mut impl Host, env: &mut Env) -> Result<Value, Diagnostic> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return Ok(Value::Null);
    }

    // Choose a mode using parsing only. A complete program (including a
    // semicolon-terminated expression) uses program checking. An explicit '='
    // always selects a Pure cell. Never retry after checking or evaluation:
    // even a failed host call may already have committed an external effect.
    if !trimmed.starts_with('=') {
        if let Ok(program) = parse_program(trimmed) {
            check_program(&program)?;
            let mut engine = Engine::new(host, Budget::default());
            return engine.eval_program(&program, env);
        }
    }

    let cell_src = if trimmed.starts_with('=') {
        trimmed.to_string()
    } else {
        format!("= {trimmed}")
    };

    let expr = parse_cell(&cell_src)?;
    check_cell(&expr)?;
    let mut engine = Engine::new(host, Budget::default());
    engine.eval_expr(&expr, env)
}

/// Evaluate once through the canonical persistent checked source gateway.
/// The caller supplies a session whose actual grants were admitted by its host.
pub fn eval_session_line(
    line: &str,
    host: &mut impl Host,
    session: &mut SourceSession,
) -> Result<SubmissionResult, SubmissionError> {
    session.submit(line, host, Budget::default())
}

/// Parse explicit host-level local grants; source requests are separate.
pub fn parse_repl_grants(args: &[String]) -> Result<Vec<String>, String> {
    let mut grants = Vec::new();
    let mut index = 0;
    while index < args.len() {
        if args[index] != "--grant" {
            return Err(format!("unknown repl option: {}", args[index]));
        }
        let cap = args
            .get(index + 1)
            .filter(|s| !s.trim().is_empty() && !s.starts_with("--"))
            .ok_or_else(|| "--grant requires an explicit capability identifier".to_string())?;
        if !grants.contains(cap) {
            grants.push(cap.clone());
        }
        index += 2;
    }
    Ok(grants)
}

/// Host-admitted local capability profile. Source requests never grant authority.
/// This local host profile is not external project/provider authorization.
pub fn run_repl_with_grants(grants: Vec<String>) -> io::Result<()> {
    println!("Poet — VibeScript REPL v{}", vibe::LANGUAGE_VERSION);
    println!("Vibe is the language; Poet is this creative shell.");
    println!("Type an expression or cell statement to evaluate. Type 'exit' or Ctrl+D to quit.\n");
    println!(
        "Local REPL grants: {}",
        if grants.is_empty() {
            "none".to_string()
        } else {
            grants.join(", ")
        }
    );

    let mut host = sheet_host::ReplHost::new(grants.clone());
    let mut session = SourceSession::new(grants, SessionLimits::default());
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    loop {
        print!("vibe> ");
        stdout.flush()?;

        let mut line = String::new();
        if stdin.lock().read_line(&mut line)? == 0 {
            break;
        }

        let trimmed = line.trim();
        if trimmed == "exit" || trimmed == "quit" {
            break;
        }
        if trimmed.is_empty() {
            continue;
        }

        host.begin_submission();
        match eval_session_line(trimmed, &mut host, &mut session) {
            Ok(result) => println!("{}", result.value),
            Err(error) => {
                println!("Error: {}", error.diagnostic.message);
                if error.host_reconciliation_required {
                    println!("Host outcome requires reconciliation; no retry was attempted.");
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use vibe::{DiagCode, Span};

    #[derive(Default)]
    struct FailingSealHost {
        calls: usize,
        local: LocalHost,
    }

    fn seal_failure(span: Span) -> Diagnostic {
        Diagnostic::new(DiagCode::E600, span, "seal failed after dispatch")
            .with_fix("inspect the original request before retrying")
            .with_evidential(0.25, 0.75)
    }

    impl Host for FailingSealHost {
        fn graph_query(
            &mut self,
            args: &[Value],
            take: u64,
            span: Span,
        ) -> Result<Value, Diagnostic> {
            self.local.graph_query(args, take, span)
        }

        fn graph_stage(&mut self, term: &Value, span: Span) -> Result<Value, Diagnostic> {
            self.local.graph_stage(term, span)
        }

        fn graph_commit(&mut self, span: Span) -> Result<Value, Diagnostic> {
            self.local.graph_commit(span)
        }

        fn aura_validate(
            &mut self,
            node: &Value,
            shape: &Value,
            span: Span,
        ) -> Result<Value, Diagnostic> {
            self.local.aura_validate(node, shape, span)
        }

        fn pulse_publish(
            &mut self,
            topic: &str,
            payload: &Value,
            span: Span,
        ) -> Result<Value, Diagnostic> {
            self.local.pulse_publish(topic, payload, span)
        }

        fn quin_seal(
            &mut self,
            _subject: u64,
            _predicate: u64,
            _object: u64,
            _context: u64,
            span: Span,
        ) -> Result<Value, Diagnostic> {
            self.calls += 1;
            Err(seal_failure(span))
        }
    }

    const SEAL: &str = "quin.statement(subject: 1, predicate: 2, object: 3, context: 4)";

    #[test]
    fn failing_host_call_runs_once_in_each_mode() {
        for source in [SEAL.to_string(), format!("= {SEAL}"), format!("{SEAL};")] {
            let mut host = FailingSealHost::default();
            let mut env = Env::default();
            let error = eval_line(&source, &mut host, &mut env).unwrap_err();
            assert_eq!(host.calls, 1, "source: {source}");
            assert_eq!(error, seal_failure(error.span), "source: {source}");
        }
    }

    #[test]
    fn authorized_effectful_program_is_not_checked_as_a_pure_cell() {
        let mut host = LocalHost::default();
        let mut env = Env::default();
        eval_line(
            "requires [ capability(\"pulse.publish\") ]; pulse.publish(\"once\", 1);",
            &mut host,
            &mut env,
        )
        .unwrap();
        assert_eq!(host.published, ["once"]);
    }

    #[test]
    fn program_check_failure_does_not_dispatch() {
        let mut host = LocalHost::default();
        let mut env = Env::default();
        let error = eval_line("pulse.publish(\"denied\", 1);", &mut host, &mut env).unwrap_err();
        assert_eq!(error.code, DiagCode::E300);
        assert!(host.published.is_empty());
    }

    #[test]
    fn explicit_cell_does_not_fall_back_to_program() {
        let mut host = LocalHost::default();
        let mut env = Env::default();
        let error = eval_line("= pulse.publish(\"denied\", 1);", &mut host, &mut env).unwrap_err();
        assert_eq!(error.code, DiagCode::E200);
        assert!(host.published.is_empty());
    }

    #[test]
    fn malformed_program_does_not_execute_a_valid_prefix() {
        let mut host = LocalHost::default();
        let mut env = Env::default();
        assert!(eval_line(
            "requires [ capability(\"pulse.publish\") ]; pulse.publish(\"never\", 1); let =",
            &mut host,
            &mut env,
        )
        .is_err());
        assert!(host.published.is_empty());
        assert!(env.vars.is_empty());
    }

    #[test]
    fn failed_program_keeps_effects_without_replaying_them() {
        let mut host = LocalHost::default();
        let mut env = Env::default();
        let error = eval_line(
            "requires [ capability(\"pulse.publish\") ]; pulse.publish(\"committed\", 1); missing;",
            &mut host,
            &mut env,
        )
        .unwrap_err();
        assert_eq!(error.code, DiagCode::E600);
        assert_eq!(error.message, "undefined missing");
        assert_eq!(host.published, ["committed"]);
    }

    #[test]
    fn declarations_survive_later_submissions() {
        let mut host = LocalHost::default();
        let mut env = Env::default();
        eval_line("let a: i64 = 10;", &mut host, &mut env).unwrap();
        assert_eq!(
            eval_line("a * 2", &mut host, &mut env).unwrap(),
            Value::I64(20)
        );
    }

    #[test]
    fn bare_condition_preserves_vibe_0_1_truthiness() {
        let mut host = LocalHost::default();
        let mut env = Env::default();
        assert_eq!(
            eval_line("if 0 { 1 } else { 2 }", &mut host, &mut env).unwrap(),
            Value::I64(2)
        );
    }

    #[test]
    fn repl_eval_simple_math() {
        let mut host = LocalHost::default();
        let mut env = Env::default();
        let val = eval_line("1 + 2 * 3", &mut host, &mut env).unwrap();
        assert_eq!(val, Value::I64(7));
    }

    #[test]
    fn repl_maintains_env_bindings() {
        let mut host = LocalHost::default();
        let mut env = Env::default();
        env.vars.insert("a".into(), Value::I64(10));
        let val = eval_line("a * 2", &mut host, &mut env).unwrap();
        assert_eq!(val, Value::I64(20));
    }
}





#[cfg(test)]
#[path = "repl_sheet_host_tests.rs"]
mod sheet_host_tests;

