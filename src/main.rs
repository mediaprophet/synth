//! Direct local candidate production over existing owners. Does not invent an
//! authenticated project session or claim human/domain/package acceptance.
mod asset_cli;
#[cfg(all(feature = "preview", not(target_arch = "wasm32")))]
mod native_preview;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let result = if args.is_empty() || args.as_slice() == ["--help"] {
        println!("Synth commands:\n  capabilities  Inspect the admitted Vibe SDK providers\n  ontology      Inspect the category-theoretic ontology taxonomy and morphisms\n  operations    List shared-library asset operations and ontological contracts\n  profiles      List available production profiles\n  mcp           Run the comprehensive QualiaDB + Synth MCP JSON-RPC 2.0 server over stdio\n  mcp-tools     Print the unified QualiaDB + Synth MCP tool catalogue as JSON\n  env           Bootstrap and inspect Synth's self-produced tray & studio environment\n  serve         Run the local Synth Studio HTTP workbench on a non-conflicting port (default: 8787). QualiaDB demo at / ; current project at /project when --project DIR is set\n  tray          Launch the Aether resident tray & studio environment (requires native-ui feature)\n  produce       Generate or strictly reopen an asset HMC\n  present       Generate a self-contained HTML showcase for Antigravity / Browser Preview\n  preview       Render .10d PNGs and interactive HTML turntable (requires preview feature)");
        asset_cli::run(&["--help".into()])
    } else if args.as_slice() == ["capabilities"] {
        synth::vibe_runtime::inspect().and_then(|report| {
            println!(
                "{}",
                serde_json::to_string_pretty(&report).map_err(|e| e.to_string())?
            );
            Ok(())
        })
    } else if args.first().is_some_and(|arg| arg == "ontology") {
        let category = args.get(1).map(String::as_str);
        let payload = if category.is_some() {
            synth::production::assets::ontology::morphisms_json(category)
        } else {
            serde_json::json!({
                "categories": synth::production::assets::ontology::categories_json(),
                "morphisms": synth::production::assets::ontology::morphisms_json(None),
            })
        };
        println!("{}", serde_json::to_string_pretty(&payload).unwrap());
        Ok(())
    } else if args.as_slice() == ["operations"] {
        println!(
            "{}",
            serde_json::to_string_pretty(&synth::production::assets::catalogue()).unwrap()
        );
        Ok(())
    } else if args.as_slice() == ["profiles"] {
        for (id, version) in synth::production::ProducerRegistry::with_builtins().profiles() {
            println!("{id}@{version}");
        }
        Ok(())
    } else if args.as_slice() == ["mcp"] {
        synth::mcp_full::run_standalone_stdio()
    } else if args.as_slice() == ["mcp-tools"] {
        println!(
            "{}",
            serde_json::to_string_pretty(&synth::mcp_full::catalogue_json()).unwrap()
        );
        Ok(())
    } else if args.first().is_some_and(|arg| arg == "env") {
        synth::tray_env::bootstrap_environment().and_then(|env| {
            if args.len() == 3 && args[1] == "--present" {
                std::fs::write(&args[2], env.showcase_html.as_bytes())
                    .map_err(|e| e.to_string())?;
            } else if args.len() != 1 {
                return Err("Usage: synth env [--present OUT.html]".to_owned());
            }
            let report = synth::tray_env::environment_report_json()?;
            println!(
                "{}",
                serde_json::to_string_pretty(&report).map_err(|e| e.to_string())?
            );
            Ok(())
        })
    } else if args.first().is_some_and(|arg| arg == "serve") {
        let mut port: Option<u16> = None;
        let mut open_browser = false;
        let mut project: Option<std::path::PathBuf> = None;
        let mut idx = 1;
        let mut parse_err: Option<String> = None;
        while idx < args.len() {
            match args[idx].as_str() {
                "--open" => {
                    open_browser = true;
                    idx += 1;
                }
                "--port" => {
                    if let Some(p_str) = args.get(idx + 1) {
                        match p_str.parse::<u16>() {
                            Ok(p) => port = Some(p),
                            Err(_) => {
                                parse_err = Some(format!("Invalid --port value: {p_str}"));
                                break;
                            }
                        }
                        idx += 2;
                    } else {
                        parse_err = Some(
                            "Usage: synth serve [--port PORT] [--open] [--project DIR]".to_owned(),
                        );
                        break;
                    }
                }
                "--project" => {
                    if let Some(p) = args.get(idx + 1) {
                        project = Some(std::path::PathBuf::from(p));
                        idx += 2;
                    } else {
                        parse_err = Some(
                            "Usage: synth serve [--port PORT] [--open] [--project DIR]".to_owned(),
                        );
                        break;
                    }
                }
                other => {
                    parse_err = Some(format!(
                        "Unknown `synth serve` flag `{other}` (Usage: synth serve [--port PORT] [--open] [--project DIR])"
                    ));
                    break;
                }
            }
        }
        if let Some(err) = parse_err {
            Err(err)
        } else {
            synth::project::attach(project).and_then(|_| {
                synth::web_server::serve_blocking(port, open_browser)
            })
        }
    } else if args.as_slice() == ["tray"] {
        #[cfg(feature = "native-ui")]
        {
            synth::native_ui::launch_standalone_environment(None)
        }
        #[cfg(not(feature = "native-ui"))]
        {
            Err("Aether tray & studio requires building with `--features native-ui` (use `synth env` to inspect the self-produced environment)".to_owned())
        }
    } else if args.first().is_some_and(|arg| arg == "preview") {
        #[cfg(all(feature = "preview", not(target_arch = "wasm32")))]
        {
            native_preview::run(&args[1..])
        }
        #[cfg(any(not(feature = "preview"), target_arch = "wasm32"))]
        {
            Err("Native `.10d` preview requires a native build with --features preview".to_owned())
        }
    } else {
        asset_cli::run(&args)
    };
    if let Err(error) = result {
        eprintln!("Synth: {error}");
        std::process::exit(1);
    }
}
