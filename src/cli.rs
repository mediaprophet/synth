//! Shared CLI entry point for `synth` and `synth-atlas` binaries.
#[path = "asset_cli.rs"]
mod asset_cli;
#[cfg(all(feature = "preview", not(target_arch = "wasm32")))]
#[path = "native_preview.rs"]
mod native_preview;

pub fn run() {
    crate::session_admission::init_gpu_policy_on_startup();
    let args: Vec<String> = std::env::args().skip(1).collect();
    let result = if args.is_empty() || args.as_slice() == ["--help"] {
        println!("Synth commands:\n  find            Search QualiaDB/Etalon/Hypermedia morphisms and MCP tools (synth find xpbd)\n  capabilities    Inspect the admitted Vibe SDK providers and the product card (development tool)\n  ontology        Inspect the category-theoretic ontology taxonomy and morphisms\n  operations      List shared-library asset operations and ontological contracts\n  profiles        List available production profiles\n  mcp             Run the comprehensive QualiaDB + Synth MCP JSON-RPC 2.0 server over stdio\n  mcp-tools       Print the unified QualiaDB + Synth MCP tool catalogue as JSON\n  env             Bootstrap and inspect Synth's self-produced tray & studio environment\n  serve           Run the local Synth Studio HTTP workbench (default: 8787)\n  operator        Webizen AI Synth operator at /operator\n  gpu             Inspect or select GPU adapter preference (discrete-only, discrete, integrated, name:<gpu>) and backend (vulkan, cuda, dx12, metal, gl)\n  bookmark        Semantic datasource bookmarks in .synth/\n  job             Download / wrap / compile jobs with a scheduler\n  compile         Compile an ontology/RDF file (.owl/.rdf/.xml/.n3/.ttl/.nt/.nq/.trig/.jsonld/.yaml) into a .q42 volume via QualiaDB\n  sparql          Query a .q42 volume, resident graph, or remote/bookmarked SPARQL endpoint\n  update          Check GitHub tags; verify/apply a signed release\n  convert-3d      Convert an existing 3D file (.glb/.gltf, .obj, .stl, legacy .10d) to Holographic .10d v2 + license contract\n  validate-model  Run computational model form validation (REQ-8.1)\n  verify-outputs  Run multi-format output & render verification (REQ-9.1)\n  changelog       Inspect or update signed SVN-style changelog & dual-regime license contract (REQ-6)\n  sessions        Inspect multi-agent resource admission slots & GPU runtime profile (REQ-4 & REQ-5)\n  tray            Launch the Webizen AI Synth resident tray & studio (requires native-ui feature)\n  produce         Generate or strictly reopen an asset HMC\n  present         Generate a self-contained HTML showcase for Antigravity / Browser Preview\n  preview         Render .10d PNGs and interactive HTML turntable/motion scrubber");
        asset_cli::run(&["--help".into()])
    } else if args.first().is_some_and(|arg| arg == "gpu") {
        let mut cfg = crate::session_admission::load_gpu_selection_config();
        let mut modified = false;
        let mut parse_err: Option<String> = None;
        let mut i = 1;
        while i < args.len() {
            match args[i].as_str() {
                "list" | "status" => {
                    i += 1;
                }
                "set" => {
                    i += 1;
                }
                "--prefer" | "--preference" if i + 1 < args.len() => {
                    cfg.preference = args[i + 1].clone();
                    modified = true;
                    i += 2;
                }
                "--backend" if i + 1 < args.len() => {
                    cfg.backend = args[i + 1].clone();
                    modified = true;
                    i += 2;
                }
                "--adapter" | "--name" if i + 1 < args.len() => {
                    let v = args[i + 1].trim().to_string();
                    cfg.adapter_name_filter = if v.is_empty() || v == "none" {
                        None
                    } else {
                        Some(v)
                    };
                    modified = true;
                    i += 2;
                }
                "--reject-igpu" => {
                    cfg.reject_igpu = true;
                    modified = true;
                    i += 1;
                }
                "--allow-igpu" => {
                    cfg.reject_igpu = false;
                    if cfg.preference == "discrete-only" {
                        cfg.preference = "discrete".into();
                    }
                    modified = true;
                    i += 1;
                }
                other => {
                    parse_err = Some(format!(
                        "unknown `synth gpu` option `{other}` (Usage: synth gpu [list|set] [--prefer discrete-only|discrete|integrated|name:<gpu>] [--backend vulkan|cuda|dx12|metal|gl|auto] [--adapter <name>] [--reject-igpu|--allow-igpu])"
                    ));
                    break;
                }
            }
        }
        let res = if let Some(err) = parse_err {
            Err(err)
        } else if modified {
            crate::session_admission::save_and_apply_gpu_selection_config(cfg)
        } else {
            Ok(crate::session_admission::detect_runtime_and_gpu_profile(None))
        };
        res.map(|rep| println!("{}", serde_json::to_string_pretty(&rep).unwrap()))
    } else if args.as_slice() == ["--version"] || args.as_slice() == ["version"] {
        println!(
            "{}",
            serde_json::to_string_pretty(&crate::session_admission::build_identity_and_drift_report()).unwrap()
        );
        Ok(())
    } else if args.first().is_some_and(|arg| arg == "sessions") {
        let report = serde_json::json!({
            "sessions": crate::session_admission::session_admission_status(),
            "gpu_runtime_profile": crate::session_admission::detect_runtime_and_gpu_profile(None),
            "build_identity": crate::session_admission::build_identity_and_drift_report(),
        });
        println!("{}", serde_json::to_string_pretty(&report).unwrap());
        Ok(())
    } else if args.first().is_some_and(|arg| arg == "verify-outputs") {
        let dir = args.get(1).cloned().unwrap_or_else(|| ".".into());
        crate::model_validation::verify_multi_format_outputs(serde_json::json!({ "export_dir": dir }))
            .map(|rep| println!("{}", serde_json::to_string_pretty(&rep).unwrap()))
    } else if args.first().is_some_and(|arg| arg == "validate-model") {
        let mut subject = "VascularPlant".to_string();
        let mut asset_path = None;
        let mut i = 1;
        while i < args.len() {
            match args[i].as_str() {
                "--class" if i + 1 < args.len() => {
                    subject = args[i + 1].clone();
                    i += 2;
                }
                "--asset" if i + 1 < args.len() => {
                    asset_path = Some(args[i + 1].clone());
                    i += 2;
                }
                _ => i += 1,
            }
        }
        crate::model_validation::validate_computational_model_form(serde_json::json!({
            "subject_class": subject,
            "asset_10d_path": asset_path
        }))
        .map(|rep| println!("{}", serde_json::to_string_pretty(&rep).unwrap()))
    } else if args.first().is_some_and(|arg| arg == "changelog") {
        let dir = args.get(1).cloned();
        crate::changelog_licensing::dispatch_changelog_license_mcp(serde_json::json!({
            "action": "inspect",
            "project_dir": dir
        }))
        .map(|rep| println!("{}", serde_json::to_string_pretty(&rep).unwrap()))
    } else if args.first().is_some_and(|arg| arg == "convert-3d") {
        let mut src = None;
        let mut out = None;
        let mut license = None;
        let mut creator = "Timothy Charles Holborn".to_string();
        let mut notice = None;
        let mut uri = "urn:qualia:converted:asset".to_string();
        let mut i = 1;
        while i < args.len() {
            match args[i].as_str() {
                "--source" if i + 1 < args.len() => { src = Some(args[i + 1].clone()); i += 2; }
                "--out" if i + 1 < args.len() => { out = Some(args[i + 1].clone()); i += 2; }
                "--license" if i + 1 < args.len() => { license = Some(args[i + 1].clone()); i += 2; }
                "--creator" if i + 1 < args.len() => { creator = args[i + 1].clone(); i += 2; }
                "--attribution" if i + 1 < args.len() => { notice = Some(args[i + 1].clone()); i += 2; }
                "--uri" if i + 1 < args.len() => { uri = args[i + 1].clone(); i += 2; }
                _ => i += 1,
            }
        }
        crate::model_validation::convert_3d_scene_or_asset_cli_or_mcp(serde_json::json!({
            "source_path": src,
            "out_dir": out,
            "spdx_license": license,
            "creator": creator,
            "attribution_notice": notice,
            "asset_uri": uri
        }))
        .map(|rep| println!("{}", serde_json::to_string_pretty(&rep).unwrap()))
    } else if args.first().is_some_and(|arg| arg == "find") {
        let q = args[1..].join(" ");
        if q.trim().is_empty() {
            Err("Usage: synth find <keyword>   e.g. synth find xpbd".into())
        } else {
            println!(
                "{}",
                serde_json::to_string_pretty(&crate::agent_index::search(&q, 20)).unwrap()
            );
            Ok(())
        }
    } else if args.as_slice() == ["capabilities"] {
        crate::vibe_runtime::inspect().and_then(|report| {
            println!(
                "{}",
                serde_json::to_string_pretty(&report).map_err(|e| e.to_string())?
            );
            Ok(())
        })
    } else if args.first().is_some_and(|arg| arg == "ontology") {
        let category = args.get(1).map(String::as_str);
        let payload = if category.is_some() {
            crate::production::assets::ontology::morphisms_json(category)
        } else {
            serde_json::json!({
                "categories": crate::production::assets::ontology::categories_json(),
                "morphisms": crate::production::assets::ontology::morphisms_json(None),
            })
        };
        println!("{}", serde_json::to_string_pretty(&payload).unwrap());
        Ok(())
    } else if args.as_slice() == ["operations"] {
        println!(
            "{}",
            serde_json::to_string_pretty(&crate::production::assets::catalogue()).unwrap()
        );
        Ok(())
    } else if args.as_slice() == ["profiles"] {
        for (id, version) in crate::production::ProducerRegistry::with_builtins().profiles() {
            println!("{id}@{version}");
        }
        Ok(())
    } else if args.as_slice() == ["mcp"] {
        crate::mcp_full::run_standalone_stdio()
    } else if args.as_slice() == ["mcp-tools"] {
        println!(
            "{}",
            serde_json::to_string_pretty(&crate::mcp_full::catalogue_json()).unwrap()
        );
        Ok(())
    } else if args.first().is_some_and(|arg| arg == "env") {
        crate::tray_env::bootstrap_environment().and_then(|env| {
            if args.len() == 3 && args[1] == "--present" {
                std::fs::write(&args[2], env.showcase_html.as_bytes())
                    .map_err(|e| e.to_string())?;
            } else if args.len() != 1 {
                return Err("Usage: synth env [--present OUT.html]".to_owned());
            }
            let report = crate::tray_env::environment_report_json()?;
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
            crate::project::attach(project).and_then(|_| {
                crate::web_server::serve_blocking(port, open_browser)
            })
        }
    } else if args.first().is_some_and(|arg| arg == "operator") {
        crate::operator::run_cli(&args[1..])
    } else if args.first().is_some_and(|arg| arg == "bookmark") {
        crate::operator::bookmarks::run_cli(&args[1..])
    } else if args.first().is_some_and(|arg| arg == "bioportal") {
        #[cfg(not(target_arch = "wasm32"))]
        {
            run_bioportal_cli(&args[1..])
        }
        #[cfg(target_arch = "wasm32")]
        {
            Err("synth bioportal is only available on native targets".to_owned())
        }
    } else if args.first().is_some_and(|arg| arg == "job") {
        crate::operator::jobs::run_cli(&args[1..])
    } else if args
        .first()
        .is_some_and(|arg| arg == "compile" || arg == "compile-q42")
    {
        #[cfg(not(target_arch = "wasm32"))]
        {
            run_compile_q42_cli(&args[1..])
        }
        #[cfg(target_arch = "wasm32")]
        {
            Err("synth compile is only available on native targets".to_owned())
        }
    } else if args
        .first()
        .is_some_and(|arg| arg == "sparql" || arg == "query" || arg == "query-sparql")
    {
        #[cfg(not(target_arch = "wasm32"))]
        {
            run_sparql_cli(&args[1..])
        }
        #[cfg(target_arch = "wasm32")]
        {
            Err("synth sparql is only available on native targets".to_owned())
        }
    } else if args.first().is_some_and(|arg| arg == "update") {
        crate::operator::update::run_cli(&args[1..])
    } else if args.as_slice() == ["tray"] {
        #[cfg(feature = "native-ui")]
        {
            crate::native_ui::launch_standalone_environment(None)
        }
        #[cfg(not(feature = "native-ui"))]
        {
            Err("Webizen AI Synth tray & studio requires building with `--features native-ui` (use `synth env` to inspect the self-produced environment)".to_owned())
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

#[cfg(not(target_arch = "wasm32"))]
fn run_compile_q42_cli(args: &[String]) -> Result<(), String> {
    if args.is_empty() || args.iter().any(|a| a == "--help" || a == "-h") {
        println!(
            "Usage: synth compile --source FILE --out OUT.q42 [--format owl|rdfxml|n3|ttl|nt|nq|trig|jsonld|yaml-ld-q42|q42] [--context-iri IRI] [--segment-mib MIB] [--ingest]\n  Compiles an ontology or RDF dataset (including multi-hundred-MB OWL/RDF/XML files like fma.owl) into a sealed .q42 volume via QualiaDB."
        );
        return Ok(());
    }
    let mut source: Option<String> = None;
    let mut out: Option<String> = None;
    let mut format: Option<String> = None;
    let mut context_iri: Option<String> = None;
    let mut segment_mib: Option<u64> = None;
    let mut ingest = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--source" | "--input" | "-s" | "-i" if i + 1 < args.len() => {
                source = Some(args[i + 1].clone());
                i += 2;
            }
            "--out" | "--out-q42" | "-o" if i + 1 < args.len() => {
                out = Some(args[i + 1].clone());
                i += 2;
            }
            "--format" | "-f" if i + 1 < args.len() => {
                format = Some(args[i + 1].clone());
                i += 2;
            }
            "--context-iri" | "--context" if i + 1 < args.len() => {
                context_iri = Some(args[i + 1].clone());
                i += 2;
            }
            "--segment-mib" if i + 1 < args.len() => {
                segment_mib = Some(
                    args[i + 1]
                        .parse::<u64>()
                        .map_err(|_| "--segment-mib must be a positive integer")?,
                );
                i += 2;
            }
            "--ingest" | "--ingest-to-graph" => {
                ingest = true;
                i += 1;
            }
            other if !other.starts_with('-') && source.is_none() => {
                source = Some(other.to_owned());
                i += 1;
            }
            other if !other.starts_with('-') && out.is_none() => {
                out = Some(other.to_owned());
                i += 1;
            }
            other => {
                return Err(format!(
                    "Unknown `synth compile` flag `{other}` (use `synth compile --help`)"
                ))
            }
        }
    }
    let src_str = source.ok_or("synth compile requires --source <FILE>")?;
    let out_str = out.ok_or("synth compile requires --out <OUT.q42>")?;
    let src_path = std::path::PathBuf::from(&src_str);
    if !src_path.is_file() {
        return Err(format!("Source file does not exist: {}", src_path.display()));
    }
    let file_len = std::fs::metadata(&src_path)
        .map(|m| m.len())
        .map_err(|e| e.to_string())?;
    let dest_path = if std::path::Path::new(&out_str).is_absolute()
        || out_str.contains('/')
        || out_str.contains('\\')
    {
        std::path::PathBuf::from(&out_str)
    } else if let Ok(root) = std::env::var("SYNTH_OUTPUT_ROOT") {
        let r = root.trim();
        if !r.is_empty() {
            std::path::Path::new(r).join(&out_str)
        } else {
            std::path::PathBuf::from(&out_str)
        }
    } else {
        std::path::PathBuf::from(&out_str)
    };
    if let Some(parent) = dest_path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
    }
    let resolved_fmt =
        crate::mcp_full::q42::infer_compile_format(format.as_deref(), Some(&src_str))
            .map_err(|e| e.message)?;
    let (quin_count, lex_terms, sha) = crate::mcp_full::q42::compile_file_to_q42(
        &src_path,
        file_len,
        &dest_path,
        resolved_fmt,
        context_iri.as_deref(),
        ingest,
        segment_mib,
        true,
    )
    .map_err(|e| e.message)?;
    let payload = serde_json::json!({
        "schema": "synth.compile-q42/v1",
        "source": src_path.display().to_string(),
        "format": resolved_fmt.label(),
        "path": dest_path.display().to_string(),
        "quin_count": quin_count,
        "lex_terms": lex_terms,
        "sha256": sha,
        "ingested_to_graph": ingest,
        "graph_quin_count": qualia_core_db::daemon_graph::graph_quin_count(),
        "query": {
            "tool": "query_sparql",
            "vault_path": dest_path.display().to_string(),
            "cli": format!("synth sparql --vault \"{}\" --query \"SELECT ?s ?p ?o WHERE {{ ?s ?p ?o }} LIMIT 20\"", dest_path.display()),
        }
    });
    println!(
        "{}",
        serde_json::to_string_pretty(&payload).map_err(|e| e.to_string())?
    );
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
fn run_sparql_cli(args: &[String]) -> Result<(), String> {
    if args.is_empty() || args.iter().any(|a| a == "--help" || a == "-h") {
        println!(
            "Usage: synth sparql --query \"SELECT ...\" [--vault FILE.q42] [--endpoint URL] [--bookmark ID] [--limit N] [--format json|xml|tsv|csv|ntriples]\n  Executes a SPARQL query against a .q42 volume, the resident daemon graph, or a remote/bookmarked SPARQL endpoint."
        );
        return Ok(());
    }
    let mut query: Option<String> = None;
    let mut vault: Option<String> = None;
    let mut endpoint: Option<String> = None;
    let mut bookmark: Option<String> = None;
    let mut limit: Option<u64> = None;
    let mut format: Option<String> = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--query" | "-q" if i + 1 < args.len() => {
                query = Some(args[i + 1].clone());
                i += 2;
            }
            "--vault" | "--vault-path" | "--q42" | "-v" if i + 1 < args.len() => {
                vault = Some(args[i + 1].clone());
                i += 2;
            }
            "--endpoint" | "-e" if i + 1 < args.len() => {
                endpoint = Some(args[i + 1].clone());
                i += 2;
            }
            "--bookmark" | "--bookmark-id" | "-b" if i + 1 < args.len() => {
                bookmark = Some(args[i + 1].clone());
                i += 2;
            }
            "--limit" | "-l" if i + 1 < args.len() => {
                limit = Some(
                    args[i + 1]
                        .parse::<u64>()
                        .map_err(|_| "--limit must be an integer")?,
                );
                i += 2;
            }
            "--format" | "-f" if i + 1 < args.len() => {
                format = Some(args[i + 1].clone());
                i += 2;
            }
            other if !other.starts_with('-') && query.is_none() => {
                query = Some(other.to_owned());
                i += 1;
            }
            other => {
                return Err(format!(
                    "Unknown `synth sparql` flag `{other}` (use `synth sparql --help`)"
                ))
            }
        }
    }
    let q = query.ok_or("synth sparql requires --query <SPARQL>")?;
    let mut req = serde_json::json!({ "query": q });
    if let Some(v) = vault {
        req["vault_path"] = serde_json::Value::String(v);
    }
    if let Some(ep) = endpoint {
        req["endpoint"] = serde_json::Value::String(ep);
    }
    if let Some(bm) = bookmark {
        req["bookmark_id"] = serde_json::Value::String(bm);
    }
    if let Some(lim) = limit {
        req["limit"] = serde_json::json!(lim);
    }
    if let Some(fmt) = format {
        req["format"] = serde_json::Value::String(fmt);
    }
    let mcp = crate::mcp_full::StandaloneSynthMcp::from_process_env().ok();
    let res = crate::mcp_full::q42::execute_sparql_tool(mcp.as_ref(), req)
        .map_err(|e| e.message)?;
    println!(
        "{}",
        serde_json::to_string_pretty(&res).map_err(|e| e.to_string())?
    );
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
fn run_bioportal_cli(args: &[String]) -> Result<(), String> {
    if args.iter().any(|a| a == "--help" || a == "-h") {
        println!(
            "Usage:\n  synth bioportal status\n  synth bioportal set-key <API_KEY>\n  synth bioportal list-local [--dir C:\\github\\ontology\\biomedical]\n  synth bioportal list-remote [--q QUERY] [--limit N]\n  synth bioportal download <ACRONYM> [--out-dir DIR] [--no-refactor] [--no-q42]\n  synth bioportal mappings <ACRONYM> <CLASS_IRI>\n  synth bioportal refactor <ACRONYM_OR_SOURCE_PATH> [--out-dir DIR] [--no-q42] [--ingest]\n  synth bioportal compile-biomedical [--acronym ACR] [--out-dir DIR] [--no-refactor]"
        );
        return Ok(());
    }
    let sub = args.first().map(String::as_str).unwrap_or("status");
    let mut req = serde_json::json!({});
    match sub {
        "status" => {
            req["action"] = serde_json::json!("status");
        }
        "set-key" | "set_api_key" => {
            let key = args
                .get(1)
                .ok_or("Usage: synth bioportal set-key <API_KEY>")?;
            req["action"] = serde_json::json!("set_api_key");
            req["api_key"] = serde_json::json!(key);
        }
        "list-local" | "list_local" => {
            req["action"] = serde_json::json!("list_local");
            if let Some(pos) = args.iter().position(|a| a == "--dir") {
                if let Some(d) = args.get(pos + 1) {
                    req["biomedical_dir"] = serde_json::json!(d);
                }
            }
        }
        "list-remote" | "list_remote" => {
            req["action"] = serde_json::json!("list_remote");
            let mut i = 1;
            while i < args.len() {
                match args[i].as_str() {
                    "--q" | "-q" if i + 1 < args.len() => {
                        req["q"] = serde_json::json!(args[i + 1]);
                        i += 2;
                    }
                    "--limit" | "-l" if i + 1 < args.len() => {
                        if let Ok(n) = args[i + 1].parse::<u64>() {
                            req["limit"] = serde_json::json!(n);
                        }
                        i += 2;
                    }
                    other if !other.starts_with('-') => {
                        req["q"] = serde_json::json!(other);
                        i += 1;
                    }
                    _ => i += 1,
                }
            }
        }
        "download" => {
            let acr = args
                .get(1)
                .ok_or("Usage: synth bioportal download <ACRONYM> [--out-dir DIR]")?;
            req["action"] = serde_json::json!("download");
            req["acronym"] = serde_json::json!(acr);
            let mut i = 2;
            while i < args.len() {
                match args[i].as_str() {
                    "--out-dir" | "-o" if i + 1 < args.len() => {
                        req["out_dir"] = serde_json::json!(args[i + 1]);
                        i += 2;
                    }
                    "--format" | "-f" if i + 1 < args.len() => {
                        req["download_format"] = serde_json::json!(args[i + 1]);
                        i += 2;
                    }
                    "--no-refactor" => {
                        req["refactor_owl"] = serde_json::json!(false);
                        i += 1;
                    }
                    "--no-q42" => {
                        req["compile_q42"] = serde_json::json!(false);
                        i += 1;
                    }
                    "--ingest" => {
                        req["ingest_to_graph"] = serde_json::json!(true);
                        i += 1;
                    }
                    _ => i += 1,
                }
            }
        }
        "mappings" => {
            let acr = args
                .get(1)
                .ok_or("Usage: synth bioportal mappings <ACRONYM> <CLASS_IRI>")?;
            let iri = args
                .get(2)
                .ok_or("Usage: synth bioportal mappings <ACRONYM> <CLASS_IRI>")?;
            req["action"] = serde_json::json!("mappings");
            req["acronym"] = serde_json::json!(acr);
            req["class_iri"] = serde_json::json!(iri);
        }
        "refactor" | "refactor-owl" | "refactor_owl" => {
            let target = args
                .get(1)
                .ok_or("Usage: synth bioportal refactor <ACRONYM_OR_SOURCE_PATH> [--out-dir DIR]")?;
            req["action"] = serde_json::json!("refactor_owl");
            if std::path::Path::new(target).exists() || target.contains('/') || target.contains('\\') || target.contains('.') {
                req["source_path"] = serde_json::json!(target);
            } else {
                req["acronym"] = serde_json::json!(target);
            }
            let mut i = 2;
            while i < args.len() {
                match args[i].as_str() {
                    "--out-dir" | "-o" if i + 1 < args.len() => {
                        req["out_dir"] = serde_json::json!(args[i + 1]);
                        i += 2;
                    }
                    "--out-q42" if i + 1 < args.len() => {
                        req["out_q42"] = serde_json::json!(args[i + 1]);
                        i += 2;
                    }
                    "--no-q42" => {
                        req["compile_q42"] = serde_json::json!(false);
                        i += 1;
                    }
                    "--ingest" => {
                        req["ingest_to_graph"] = serde_json::json!(true);
                        i += 1;
                    }
                    _ => i += 1,
                }
            }
        }
        "compile-biomedical" | "compile_biomedical" => {
            req["action"] = serde_json::json!("compile_biomedical");
            let mut i = 1;
            while i < args.len() {
                match args[i].as_str() {
                    "--acronym" | "-a" if i + 1 < args.len() => {
                        req["acronym"] = serde_json::json!(args[i + 1]);
                        i += 2;
                    }
                    "--out-dir" | "-o" if i + 1 < args.len() => {
                        req["out_dir"] = serde_json::json!(args[i + 1]);
                        i += 2;
                    }
                    "--dir" if i + 1 < args.len() => {
                        req["biomedical_dir"] = serde_json::json!(args[i + 1]);
                        i += 2;
                    }
                    "--no-refactor" => {
                        req["refactor_owl"] = serde_json::json!(false);
                        i += 1;
                    }
                    "--ingest" => {
                        req["ingest_to_graph"] = serde_json::json!(true);
                        i += 1;
                    }
                    other if !other.starts_with('-') => {
                        req["acronym"] = serde_json::json!(other);
                        i += 1;
                    }
                    _ => i += 1,
                }
            }
        }
        other => {
            return Err(format!(
                "Unknown `synth bioportal` subcommand `{other}` (use `synth bioportal --help`)"
            ))
        }
    }
    let mcp = crate::mcp_full::StandaloneSynthMcp::from_process_env()
        .unwrap_or_else(|_| crate::mcp_full::StandaloneSynthMcp::default());
    let res = crate::mcp_full::bioportal::dispatch_mcp(&mcp, req).map_err(|e| e.message)?;
    println!(
        "{}",
        serde_json::to_string_pretty(&res).map_err(|e| e.to_string())?
    );
    Ok(())
}
