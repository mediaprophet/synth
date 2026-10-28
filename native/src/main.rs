//! Poet CLI toolchain: REPL, formatter, linter, evaluator (T62).

mod format;
mod lint;
mod repl;
#[cfg(feature = "native-graphical")]
use poet_cli::graphical;

use std::env;
use std::fs;
use std::process;

fn print_usage() {
    println!(
        "Poet CLI toolchain for VibeScript (v{})",
        vibe::LANGUAGE_VERSION
    );
    println!("\nUsage:");
    println!("  poet repl [--grant <capability>]...  Interactive REPL; default grants: none");
    println!("  poet format <file>  Format a .vibe file");
    println!("  poet lint <file>    Run static analysis checks on a .vibe file");
    println!("  poet eval <file> [--grant <capability>]...    Evaluate a .vibe file and print result");
    println!("  poet help           Show this help message");
    #[cfg(feature = "native-graphical")]
    println!(
        "  poet studio --diagnostic-new-source [--grant <capability>]...  Native Aether Studio"
    );
    #[cfg(feature = "native-graphical")]
    println!("  poet studio --native-project <directory> [--check-project]  Native enrollment, contributor and reviewer workspace");
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        print_usage();
        process::exit(1);
    }

    match args[1].as_str() {
        #[cfg(feature = "native-graphical")]
        "mcp" => {
            if args.len() != 2 {
                eprintln!(
                    "Usage: poet mcp; project admission is supplied only by a native embedding"
                );
                process::exit(1);
            }
            if let Err(error) = graphical::mcp::run_unbound_stdio() {
                eprintln!("Poet MCP: {error}");
                process::exit(1);
            }
        }
        #[cfg(feature = "native-graphical")]
        "studio" => {
            if let Err(error) = graphical::entry::run(&args[2..]) {
                eprintln!("Poet Studio: {error}");
                process::exit(1);
            }
        }
        "repl" => {
            let grants = match repl::parse_repl_grants(&args[2..]) {
                Ok(grants) => grants,
                Err(error) => {
                    eprintln!("REPL option error: {error}");
                    process::exit(1);
                }
            };
            if let Err(e) = repl::run_repl_with_grants(grants) {
                eprintln!("REPL error: {e}");
                process::exit(1);
            }
        }
        "format" => {
            if args.len() < 3 {
                eprintln!("Error: missing file argument for format");
                process::exit(1);
            }
            let path = &args[2];
            match fs::read_to_string(path) {
                Ok(content) => {
                    let formatted = format::format_source(&content);
                    if let Err(e) = fs::write(path, &formatted) {
                        eprintln!("Error writing formatted file: {e}");
                        process::exit(1);
                    }
                    println!("Formatted {}", path);
                }
                Err(e) => {
                    eprintln!("Error reading file '{path}': {e}");
                    process::exit(1);
                }
            }
        }
        "lint" => {
            if args.len() < 3 {
                eprintln!("Error: missing file argument for lint");
                process::exit(1);
            }
            let path = &args[2];
            match fs::read_to_string(path) {
                Ok(content) => {
                    let issues = lint::lint_source(&content);
                    if issues.is_empty() {
                        println!("No issues found in {}", path);
                    } else {
                        for issue in &issues {
                            println!(
                                "{}:{}: {:?}: {}",
                                path, issue.line, issue.severity, issue.message
                            );
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error reading file '{path}': {e}");
                    process::exit(1);
                }
            }
        }
        "eval" => {
            if args.len() < 3 {
                eprintln!("Error: missing file argument for eval");
                process::exit(1);
            }
            let path = &args[2];
            match fs::read_to_string(path) {
                Ok(content) => {
                    let grants = match repl::parse_repl_grants(&args[3..]) {
                        Ok(grants) => grants,
                        Err(error) => { eprintln!("Eval option error: {error}"); process::exit(1); }
                    };
                    let mut host = repl::sheet_host::ReplHost::new(grants.clone());
                    if grants.is_empty() {
                        let mut env = vibe::Env::default();
                        match vibe::eval_cell(&content, &mut host, &mut env) {
                            Ok(val) => println!("{val:?}"),
                            Err(diag) => { eprintln!("Eval error: {}", diag.message); process::exit(1); }
                        }
                    } else {
                        let mut session = vibe::session::SourceSession::new(grants, vibe::session::SessionLimits::default());
                        match repl::eval_session_line(&content, &mut host, &mut session) {
                            Ok(result) => println!("{:?}", result.value),
                            Err(error) => {
                                eprintln!("Eval error: {}", error.diagnostic.message);
                                if error.host_reconciliation_required { eprintln!("Host outcome requires reconciliation; no retry was attempted."); }
                                process::exit(1);
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error reading file '{path}': {e}");
                    process::exit(1);
                }
            }
        }
        "help" | "-h" | "--help" => {
            print_usage();
        }
        other => {
            eprintln!("Unknown command: '{other}'");
            print_usage();
            process::exit(1);
        }
    }
}

