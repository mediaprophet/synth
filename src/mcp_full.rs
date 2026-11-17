//! Comprehensive MCP bridge uniting QualiaDB's full 60-tool `qualia-mcp` suite
//! (`engine-tools`) with Synth's categorical ontology, VibeScript production,
//! and Antigravity HTML presentation (`synth.presentation-html/v1`).
use crate::production::{ProducerRegistry, ProductionLimits};
use crate::NativeOutputRoot;
use qualia_mcp::{
    authoring::{AuthoringBackend, AuthoringTool, ProtocolLimits, ToolError},
    dispatch::enforce_fiduciary_tool_dispatch,
    registry::stable_mcp_tools,
    types::{McpIntentFrame, RawToolPayload},
};
use serde::Deserialize;
use serde_json::{json, Value};
use std::{
    path::{Path, PathBuf},
};

#[path = "mcp_full/production.rs"]
mod production;
#[path = "mcp_full/contracts.rs"]
mod contracts;
#[path = "mcp_full/catalog.rs"]
mod catalog;
#[cfg(not(target_arch = "wasm32"))]
#[path = "mcp_full/q42.rs"]
mod q42;


/// Convert `qualia_mcp::stable_mcp_tools()` descriptors into `AuthoringTool` entries.
pub fn qualia_engine_authoring_tools() -> Vec<AuthoringTool> {
    stable_mcp_tools()
        .iter()
        .map(|desc| {
            let schema: Value = serde_json::from_str(desc.input_schema)
                .unwrap_or_else(|_| json!({"type": "object", "properties": {}}));
            AuthoringTool {
                name: desc.name.to_owned(),
                description: desc.description.to_owned(),
                input_schema: schema,
            }
        })
        .collect()
}

/// Dispatch a tool call to `qualia_mcp::dispatch::enforce_fiduciary_tool_dispatch`
/// and normalize the output into a JSON object for `AuthoringServer`.
pub fn dispatch_qualia_engine_tool(name: &str, value: &Value) -> Option<Result<Value, ToolError>> {
    let normalized = name.replace('.', "_");
    let exists = stable_mcp_tools()
        .iter()
        .any(|t| t.name == name || t.name == normalized);
    if !exists {
        return None;
    }
    let tool_id = if stable_mcp_tools().iter().any(|t| t.name == name) {
        name
    } else {
        normalized.as_str()
    };
    let raw_args = serde_json::to_vec(value).unwrap_or_else(|_| b"{}".to_vec());
    let intent = McpIntentFrame::default();
    let payload = RawToolPayload {
        tool_name: tool_id.as_bytes(),
        arguments_raw: &raw_args,
    };
    Some(match enforce_fiduciary_tool_dispatch(payload, &intent) {
        Ok(text) => match serde_json::from_str::<Value>(&text) {
            Ok(Value::Object(map)) => Ok(Value::Object(map)),
            Ok(other) => Ok(json!({ "status": "ok", "tool": tool_id, "result": other })),
            Err(_) => Ok(json!({ "status": "ok", "tool": tool_id, "text": text })),
        },
        Err(err) => {
            use qualia_mcp::types::McpSystemError as E;
            let code = match &err {
                E::InvalidParameters | E::ParseError => "invalid_arguments",
                E::ToolNotFound => "unknown_operation",
                E::ToolNotReady | E::FeatureNotEnabled => "operation_unavailable",
                E::ExecutionFailed(_) => "execution_failed",
                E::SanctuaryGateTriggered | E::IntentFrameViolation => "permission_denied",
            };
            let message: String = err.to_string().chars().take(3500).collect();
            Err(ToolError::new(code, message))
        },
    })
}

/// Standalone comprehensive MCP backend for `synth mcp` (Antigravity IDE, Antigravity 2.0,
/// and `vendor/antigravity-sdk-python` `McpStdioServer`), exposing all 60 QualiaDB MCP
/// tools plus Synth's standalone capabilities, ontology, and HTML presentation tools.
pub struct StandaloneSynthMcp {
    output_root: Option<NativeOutputRoot>,
    input_root: Option<production::InputRoot>,
}

impl Default for StandaloneSynthMcp {
    fn default() -> Self {
        Self {
            output_root: None,
            input_root: None,
        }
    }
}

impl StandaloneSynthMcp {
    pub fn with_input_root(mut self, root: &Path) -> Result<Self, String> {
        self.input_root = Some(production::InputRoot::admit(root)?);
        Ok(self)
    }
    pub fn with_output_root(mut self, root: NativeOutputRoot) -> Self {
        self.output_root = Some(root);
        self
    }

    /// Operator-configured native directory (`SYNTH_OUTPUT_ROOT`). Unset means writes are refused.
    pub fn from_process_env() -> Result<Self, String> {
        let mut mcp = Self::default();
        match std::env::var("SYNTH_OUTPUT_ROOT") {
            Ok(raw) if !raw.trim().is_empty() => {
                mcp.output_root = Some(NativeOutputRoot::admit(Path::new(raw.trim()))?);
            }
            _ => {}
        }
        if let Ok(raw) = std::env::var("SYNTH_INPUT_ROOT") {
            if !raw.trim().is_empty() {
                mcp.input_root = Some(production::InputRoot::admit(Path::new(raw.trim()))?);
            }
        }
        Ok(mcp)
    }

    fn confined_destination(&self, name: &str, ext: &str, err_msg: &str) -> Result<PathBuf, ToolError> {
        let root = self.output_root.as_ref().ok_or_else(|| {
            ToolError::new(
                "native_output_unconfigured",
                "Native output destination not configured",
            )
        })?;
        root.destination_with_ext(name, ext, err_msg)
            .map_err(|e| ToolError::new("native_output_refused", e))
    }

    pub fn synth_tools() -> Vec<AuthoringTool> {
        let object = |props: Value, required: Vec<&str>| {
            json!({
                "type": "object",
                "properties": props,
                "required": required,
                "additionalProperties": false
            })
        };
        vec![
            AuthoringTool {
                name: "synth_capabilities".into(),
                description: "Inspect the admitted Vibe SDK providers and execute the SDK-to-Etalon 5-field NQuin48 parity probe".into(),
                input_schema: object(json!({}), vec![]),
            },
            AuthoringTool {
                name: "synth_ontology_categories".into(),
                description: "Inspect the 12 canonical categories of the Hypermedia/Synth categorical ontology and NQuin48 triple counts".into(),
                input_schema: object(json!({}), vec![]),
            },
            AuthoringTool {
                name: "synth_ontology_morphisms".into(),
                description: format!(
                    "Inspect the {} asset production morphisms (paged: category, id, offset, limit, detail=ids|summary|full). Default 40 summaries.",
                    crate::production::assets::OPERATIONS.len()
                ),
                input_schema: object(catalog::filter_schema(), vec![]),
            },
            AuthoringTool {
                name: "synth_operations".into(),
                description: format!(
                    "List the {} admitted shared-library asset operations (paged: category, id, offset, limit, detail=ids|summary|full). Default 40 summaries so Grok MCP does not overflow.",
                    crate::production::assets::OPERATIONS.len()
                ),
                input_schema: object(catalog::filter_schema(), vec![]),
            },
            AuthoringTool {
                name: "synth_profiles".into(),
                description: "List registered AssetProducer profiles and versions".into(),
                input_schema: object(json!({}), vec![]),
            },
            AuthoringTool {
                name: "synth_environment_bootstrap".into(),
                description: "Bootstrap Webizen AI Synth's self-produced tray & studio environment (examples/tray_environment.vibe), probe Webizen Desktop presence, and scan the local LLM model vault".into(),
                input_schema: object(json!({}), vec![]),
            },
            AuthoringTool {
                name: "synth_operation_contracts".into(),
                description: "Inspect execution host, handle lifetime and explicit encoded/record output recipes (paged). Recipes declare field extraction and encoding; never infer output types from namespaces.".into(),
                input_schema: object(catalog::filter_schema(), vec![]),
            },
            AuthoringTool {
                name: "synth_compile_q42".into(),
                description: "Parse Turtle/N3/N-Triples (or copy an existing .q42) from SYNTH_INPUT_ROOT or inline rdf_data, seal a create-only .q42 under SYNTH_OUTPUT_ROOT, optionally ingest the daemon graph. Query with query_sparql vault_path.".into(),
                input_schema: {
                    let mut schema = object(
                    json!({
                        "source_path": {"type":"string","maxLength":4096,"description":"TTL/N3/NT/Q42 file under SYNTH_INPUT_ROOT"},
                        "rdf_data": {"type":"string","maxLength":16777216,"description":"Inline Turtle when the agent fetched a datasource"},
                        "format": {"type":"string","enum":["ttl","turtle","n3","nt","ntriples","q42"]},
                        "out_q42": {"type":"string","maxLength":96,"pattern":"^[A-Za-z0-9_-][A-Za-z0-9._-]*\\.q42$"},
                        "ingest_to_graph": {"type":"boolean"},
                        "context_iri": {"type":"string","maxLength":1024}
                    }),
                    vec!["out_q42"],
                    );
                    schema["oneOf"] = json!([{ "required": ["source_path", "out_q42"] }, { "required": ["rdf_data", "out_q42"] }]);
                    schema
                },
            },
            AuthoringTool {
                name: "synth_operator_status".into(),
                description: "Operator snapshot: Ecopunk production rules, recent MCP events, local git describe/tags. Pass fetch=true to ls-remote origin tags. Never auto-replaces a locked synth.exe. Browser: GET /operator.".into(),
                input_schema: object(
                    json!({
                        "fetch": {"type":"boolean","description":"Also git ls-remote --tags origin. Reports only; never overwrites the running binary."}
                    }),
                    vec![],
                ),
            },
            AuthoringTool {
                name: "synth_production_rules".into(),
                description: "Evaluate Ecopunk production rules. Biosphere non-fiction taxa require sourced .q42 evidence before model/produce. compile_q42 and graph queries are the evidence path and stay allowed. Encyclopedia is Ecopunk.".into(),
                input_schema: object(
                    json!({
                        "domain": {"type":"string","maxLength":64},
                        "taxon": {"type":"string","maxLength":128},
                        "fiction": {"type":"boolean"},
                        "action": {"type":"string","maxLength":64},
                        "evidence_q42": {"type":"string","maxLength":4096}
                    }),
                    vec![],
                ),
            },
            AuthoringTool {
                name: "synth_bookmarks".into(),
                description: "Semantic bookmarks for RDF dumps, SPARQL endpoints, and similar datasources. Store public metadata in .synth/bookmarks.json. Agent enrichment: data_endpoint when the user pasted a landing page, requires_api_key / api_key_env when auth is needed. action=set_secret writes .synth/secrets.json (gitignored); list/get never return secret values. Fetch with synth_jobs into .synth/inbox; compile with synth_compile_q42.".into(),
                input_schema: object(
                    json!({
                        "action": {"type":"string","enum":["list","get","upsert","forget","set_secret"]},
                        "id": {"type":"string","maxLength":96},
                        "url": {"type":"string","maxLength":2048,"description":"http(s) landing page, SPARQL endpoint, or RDF dump. Credential query params are stripped."},
                        "title": {"type":"string","maxLength":256},
                        "kind": {"type":"string","enum":["sparql","rdf","rest","csv","page"]},
                        "data_endpoint": {"type":"string","maxLength":2048},
                        "formats": {"type":"array","maxItems":8,"items":{"type":"string","maxLength":32}},
                        "requires_api_key": {"type":"boolean"},
                        "api_key_env": {"type":"string","maxLength":128,"description":"Name of the env var or secret slot. Not the key itself."},
                        "auth": {"type":"string","maxLength":32},
                        "licence": {"type":"string","maxLength":256},
                        "notes": {"type":"string","maxLength":8192},
                        "agent_notes": {"type":"string","maxLength":8192},
                        "domain": {"type":"string","maxLength":64},
                        "tags": {"type":"array","maxItems":16,"items":{"type":"string","maxLength":64}},
                        "secret": {"type":"string","maxLength":4096,"description":"Only with action=set_secret. Written to gitignored .synth/secrets.json."}
                    }),
                    vec![],
                ),
            },
            AuthoringTool {
                name: "synth_jobs".into(),
                description: "Downloader and scheduler. Fetch http(s) into .synth/inbox (gitignored staging), wrap PROV/DCT/DCAT, compile Turtle to .q42. action=add|list|get|run|run_due|enable|disable|forget. kind=fetch|ingest|compile. Operator ticks due jobs; synth mcp does not start the scheduler. Max 16 MiB. Secrets stay in .synth/secrets.json.".into(),
                input_schema: object(
                    json!({
                        "action": {"type":"string","enum":["list","get","add","upsert","forget","enable","disable","run","run_due"]},
                        "id": {"type":"string","maxLength":96},
                        "kind": {"type":"string","enum":["fetch","ingest","compile"]},
                        "title": {"type":"string","maxLength":256},
                        "url": {"type":"string","maxLength":2048,"description":"http(s) dump, SPARQL endpoint, or landing URL. Credential query params are rejected."},
                        "bookmark_id": {"type":"string","maxLength":96,"description":"Reuse a semantic bookmark URL / data_endpoint / secret."},
                        "source_path": {"type":"string","maxLength":4096,"description":"Local file under SYNTH_INPUT_ROOT for kind=compile."},
                        "query": {"type":"string","maxLength":32768,"description":"Optional SPARQL CONSTRUCT/DESCRIBE body (POST)."},
                        "out_q42": {"type":"string","maxLength":96,"pattern":"^[A-Za-z0-9_-][A-Za-z0-9._-]*\\.q42$"},
                        "interval_secs": {"type":"integer","minimum":0,"description":"0 = run once (manual or first due). Repeat uses max(interval, 60)."},
                        "enabled": {"type":"boolean"}
                    }),
                    vec![],
                ),
            },
            AuthoringTool {
                name: "synth_update".into(),
                description: "Check GitHub Releases, download+verify+apply the latest signed portable (action=upgrade), or verify/apply a local artifact. Signatures are Ed25519 over SHA-256 (context synth.release.sha256.v1). upgrade/apply copy to a sibling synth.new.exe and never overwrite a locked synth.exe. sign/keygen stay on the CLI.".into(),
                input_schema: object(
                    json!({
                        "action": {"type":"string","enum":["check","upgrade","verify","apply"]},
                        "artifact": {"type":"string","maxLength":4096},
                        "sig": {"type":"string","maxLength":4096},
                        "dest": {"type":"string","maxLength":4096,"description":"Directory to copy a verified artifact into. Defaults to the running binary's parent."}
                    }),
                    vec![],
                ),
            },
            AuthoringTool {
                name: "synth_produce_present".into(),
                description: "Execute Hypermedia asset operations inside one bounded ProductionSession. Bind local inputs under SYNTH_INPUT_ROOT; save create-only HMC/HTML and export resources under SYNTH_OUTPUT_ROOT. Handles expire after source evaluation. Return hashes, absolute resource paths and optional native review receipts. Biosphere non-fiction (domain=biosphere, fiction=false) requires evidence_q42 or ecopunk/entities/taxa/{taxon}/*.q42.".into(),
                input_schema: {
                    let mut schema = object(
                    json!({
                        "source": {
                            "type": "string",
                            "maxLength": 4 * 1024 * 1024
                        },
                        "source_path": {"type":"string", "maxLength":4096, "description":"UTF-8 source file under SYNTH_INPUT_ROOT; provide exactly one of source and source_path."},
                        "profile": {"type": "string", "maxLength": 64},
                        "quality_profile": {"type":"string","enum":["preview","production16"],"description":"preview uses 16/64/80 MiB resource/aggregate/package budgets; production16 explicitly admits 64/256/320 MiB for true 2k RGBA16. Image work budget is 500 million logical units; mesh size caps are unchanged."},
                        "inputs": {"type": "array", "maxItems": ProductionLimits::default().resources,
                            "items": {"type": "object", "additionalProperties": false,
                                "required": ["key", "kind", "path"], "properties": {
                                    "key": {"type": "string", "maxLength": 1024},
                                    "kind": {"type": "string", "maxLength": 64},
                                    "path": {"type": "string", "maxLength": 4096}
                                }}},
                        "export_dir": {"type": "string", "maxLength": 96, "pattern": "^[A-Za-z0-9_-][A-Za-z0-9._-]*$"},
                        "preview_assets": {"type": "array", "maxItems": 8, "items": {"type": "string", "maxLength": 1024}, "description": "Exported .10d resource keys to render with native preview; requires export_dir."},
                        "receipt_json": {"type": "string", "maxLength": 96, "pattern": "^[A-Za-z0-9_-][A-Za-z0-9._-]*\\.json$"},
                        "out_hmc": {
                            "type": "string",
                            "maxLength": 96,
                            "pattern": "^[A-Za-z0-9_-][A-Za-z0-9._-]*\\.hmc$"
                        },
                        "present_html": {
                            "type": "string",
                            "maxLength": 96,
                            "pattern": "^[A-Za-z0-9_-][A-Za-z0-9._-]*\\.html$"
                        },
                        "domain": {"type":"string","maxLength":64,"description":"Encyclopedia domain. biosphere + fiction=false gates geometry until evidence .q42 exists."},
                        "taxon": {"type":"string","maxLength":128,"description":"Ecopunk taxon id, e.g. corymbia-aparrerinja"},
                        "fiction": {"type":"boolean"},
                        "evidence_q42": {"type":"string","maxLength":4096,"description":"Path to a sealed evidence .q42 (under SYNTH_INPUT_ROOT or absolute)"}
                    }),
                    vec![],
                    );
                    schema["oneOf"] = json!([{ "required": ["source"] }, { "required": ["source_path"] }]);
                    schema
                },
            },
        ]
    }
}

impl AuthoringBackend for StandaloneSynthMcp {
    fn instructions(&self) -> Option<String> {
        Some(format!("Synth exposes {} asset morphisms through paged synth_operations / synth_ontology_morphisms / synth_operation_contracts (default 40 summaries; pass category, id, offset, limit, detail=full). Execute morphisms using capability.invoke inside synth_produce_present source (production AssetHost); vibe_invoke/vibe_catalog use a separate generic Poet snapshot. QualiaDB graph tools are on this same server: query_sparql, query_graph, ingest_ontology, parse_rdf, parse_json, parse_csv, serialize_rdf. Compile sourced Turtle/N3 into .q42 with synth_compile_q42 then query_sparql vault_path. Bookmark RDF/SPARQL/landing URLs with synth_bookmarks (public .synth/bookmarks.json; secrets in gitignored .synth/secrets.json). Fetch those URLs with synth_jobs (inbox .synth/inbox, PROV/DCT wrap, optional .q42). The operator scheduler ticks due jobs; synth mcp does not auto-run them. Check or apply signed GitHub Releases with synth_update (never overwrites a locked synth.exe). Biosphere non-fiction taxa (encyclopedia=Ecopunk) require that evidence .q42 before modelling; synth_production_rules and synth_operator_status report the gate. Bind inputs with key/kind/path under SYNTH_INPUT_ROOT; exports/HMC/HTML/Q42 destinations are create-only names under SYNTH_OUTPUT_ROOT. Inspect returned PNG/SVG/native review files before judging quality. Tool names use underscores. Input root configured: {}; output root configured: {}.", crate::production::assets::OPERATIONS.len(), self.input_root.is_some(), self.output_root.is_some()))
    }
    fn tools(&self) -> Vec<AuthoringTool> {
        let mut all = Self::synth_tools();
        all.extend(qualia_engine_authoring_tools());
        all
    }

    fn call(&mut self, name: &str, args: Value) -> Result<Value, ToolError> {
        let result = match name {
            "synth_capabilities" | "synth.capabilities" => {
                crate::vibe_runtime::inspect().map_err(|e| ToolError::new("capabilities_failed", e))
            }
            "synth_ontology_categories" | "synth.ontology.categories" => {
                Ok(crate::production::assets::ontology::categories_json())
            }
            "synth_ontology_morphisms" | "synth.ontology.morphisms" => catalog::morphisms(args),
            "synth_operations" | "synth.operations" => catalog::operations(args),
            "synth_operation_contracts" => catalog::contracts(args),
            #[cfg(not(target_arch = "wasm32"))]
            "synth_compile_q42" | "synth.compile.q42" => q42::compile(self, args),
            #[cfg(not(target_arch = "wasm32"))]
            "synth_operator_status" | "synth.operator.status" => {
                let fetch = args.get("fetch").and_then(Value::as_bool).unwrap_or(false);
                Ok(crate::operator::snapshot_with_fetch(fetch))
            }
            #[cfg(not(target_arch = "wasm32"))]
            "synth_production_rules" | "synth.production.rules" => {
                let req: crate::operator::RuleRequest = serde_json::from_value(args)
                    .map_err(|e| ToolError::new("invalid_arguments", format!("Invalid synth_production_rules arguments: {e}")))?;
                Ok(json!({
                    "rules": crate::operator::rules_json(),
                    "decision": crate::operator::evaluate_rules(&req).to_json(),
                }))
            }
            #[cfg(not(target_arch = "wasm32"))]
            "synth_bookmarks" | "synth.bookmarks" => {
                let req: crate::operator::bookmarks::BookmarkArg = serde_json::from_value(args)
                    .map_err(|e| ToolError::new("invalid_arguments", format!("Invalid synth_bookmarks arguments: {e}")))?;
                crate::operator::bookmarks::dispatch(req)
                    .map_err(|e| ToolError::new("bookmark_failed", e))
            }
            #[cfg(not(target_arch = "wasm32"))]
            "synth_jobs" | "synth.jobs" => {
                let req: crate::operator::jobs::JobArg = serde_json::from_value(args)
                    .map_err(|e| ToolError::new("invalid_arguments", format!("Invalid synth_jobs arguments: {e}")))?;
                crate::operator::jobs::dispatch(req)
                    .map_err(|e| ToolError::new("job_failed", e))
            }
            #[cfg(not(target_arch = "wasm32"))]
            "synth_update" | "synth.update" => {
                let req: crate::operator::update::UpdateArg = serde_json::from_value(args)
                    .map_err(|e| ToolError::new("invalid_arguments", format!("Invalid synth_update arguments: {e}")))?;
                crate::operator::update::dispatch(req)
                    .map_err(|e| ToolError::new("update_failed", e))
            }
            "synth_profiles" | "synth.profiles" => {
                let profiles: Vec<Value> = ProducerRegistry::with_builtins()
                    .profiles()
                    .map(|(id, version)| json!({"id": id, "version": version}))
                    .collect();
                Ok(json!({"profiles": profiles}))
            }
            "synth_environment_bootstrap" | "synth.environment.bootstrap" => {
                crate::tray_env::environment_report_json()
                    .map_err(|e| ToolError::new("environment_bootstrap_failed", e))
            }
            "synth_produce_present" | "synth.produce.present" => {
                production::produce(self, args)
            }
            _ => dispatch_qualia_engine_tool(name, &args).unwrap_or_else(|| {
                Err(ToolError::new("unknown_tool", "Tool is not in the advertised catalog; use underscore tool names"))
            }),
        };
        #[cfg(not(target_arch = "wasm32"))]
        {
            let detail = result.as_ref().err().map(|e| e.message.as_str());
            crate::operator::record_event(name, result.is_ok(), detail);
        }
        result
    }
}

/// Bounded standalone transport for the combined tool and full 150-operation catalog.
/// Authoring admits result objects at one eighth of the wire budget before making
/// text+structured copies. The full operation catalog is ~326 KiB compact, above
/// the former 256 KiB result ceiling; 4 MiB wire keeps a 512 KiB result ceiling.
pub fn comprehensive_protocol_limits() -> ProtocolLimits {
    ProtocolLimits {
        max_message_bytes: 4 * 1024 * 1024,
        max_tools: 128,
        max_id_bytes: 256,
    }
}

/// Run the standalone stdio MCP server (`synth mcp`) exposing all Synth + QualiaDB MCP tools.
pub fn run_standalone_stdio() -> Result<(), String> {
    qualia_mcp::authoring::run_stdio(
        StandaloneSynthMcp::from_process_env()?,
        comprehensive_protocol_limits(),
    )
    .map_err(|e| format!("Synth MCP transport ended: {e}"))
}

/// Return the JSON catalogue of all tools exposed by `synth mcp`.
pub fn catalogue_json() -> Value {
    let tools = StandaloneSynthMcp::default().tools();
    let asset_operations = crate::production::assets::catalogue();
    json!({
        "schema": "synth.comprehensive-mcp/v1",
        "total_tools": tools.len(),
        "synth_tools": StandaloneSynthMcp::synth_tools().len(),
        "qualia_engine_tools": qualia_engine_authoring_tools().len(),
        "admitted_morphisms": crate::production::assets::OPERATIONS.len(),
        "identifier_safe_for_fastmcp": tools.iter().all(|t| !t.name.contains('.')),
        "tools": tools,
        "asset_operations": asset_operations,
    })
}

#[cfg(test)]
#[path = "mcp_full/tests.rs"]
mod interface_tests;

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    const SOURCE: &str =
        r#"let mut asset = { files: [{ key: "note.txt", kind: "txt", text: "native-root" }] };"#;

    fn unique_dir() -> PathBuf {
        let path = std::env::temp_dir().join(format!(
            "synth-mcp-out-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&path).unwrap();
        path
    }

    const PACKING_MORPHISMS: &[&str] = &[
        "hypermedia.radiance.pack_sprite_sheet",
        "hypermedia.radiance.pack_nine_slice",
        "hypermedia.surface.pack_pbr_set",
    ];

    fn packing_ids_present(operations: &[Value]) -> bool {
        PACKING_MORPHISMS.iter().all(|id| {
            operations.iter().any(|op| op["id"].as_str() == Some(*id))
        })
    }

    #[test]
    fn operations_and_catalogue_match_live_150_ontology_including_packing() {
        assert_eq!(crate::production::assets::OPERATIONS.len(), 150);
        assert_eq!(crate::production::assets::ontology::MORPHISMS.len(), 150);
        let catalogue = crate::production::assets::catalogue();
        let operations = catalogue["operations"]
            .as_array()
            .expect("catalogue operations array");
        assert_eq!(operations.len(), 150);
        assert_eq!(catalogue["count"], 150);
        assert!(
            packing_ids_present(operations),
            "live catalogue must include packing morphisms {PACKING_MORPHISMS:?}"
        );

        let tools = StandaloneSynthMcp::synth_tools();
        assert!(
            tools.iter().any(|t| t.name == "synth_compile_q42"),
            "synth_compile_q42 must be advertised so sourced RDF can seal to .q42"
        );
        assert!(
            tools.iter().any(|t| t.name == "synth_operation_contracts"),
            "synth_operation_contracts must be advertised"
        );
        assert!(
            tools.iter().any(|t| t.name == "synth_operator_status"),
            "synth_operator_status must be advertised"
        );
        assert!(
            tools.iter().any(|t| t.name == "synth_production_rules"),
            "synth_production_rules must be advertised"
        );
        assert!(
            tools.iter().any(|t| t.name == "synth_bookmarks"),
            "synth_bookmarks must be advertised for RDF/SPARQL semantic bookmarks"
        );
        assert!(
            tools.iter().any(|t| t.name == "synth_jobs"),
            "synth_jobs must be advertised for the downloader and scheduler"
        );
        assert!(
            tools.iter().any(|t| t.name == "synth_update"),
            "synth_update must be advertised for signed GitHub releases"
        );
        let morphisms = tools
            .iter()
            .find(|t| t.name == "synth_ontology_morphisms")
            .expect("synth_ontology_morphisms");
        let operations_tool = tools
            .iter()
            .find(|t| t.name == "synth_operations")
            .expect("synth_operations");
        assert!(morphisms.description.contains("150"));
        assert!(operations_tool.description.contains("150"));
        assert!(!morphisms.description.contains("114"));
        assert!(!morphisms.description.contains("145"));
        assert!(!operations_tool.description.contains("114"));
        assert!(!operations_tool.description.contains("145"));

        let mut mcp = StandaloneSynthMcp::default();
        let live_ops = mcp.call("synth_operations", json!({})).unwrap();
        assert_eq!(live_ops["count"], 150);
        assert_eq!(live_ops["returned"], 40);
        assert_eq!(live_ops["limit"], 40);
        assert_eq!(live_ops["next_offset"], 40);
        assert_eq!(live_ops["operations"].as_array().unwrap().len(), 40);
        let live_ops_full = mcp
            .call("synth_operations", json!({"detail": "full", "limit": 150}))
            .unwrap();
        let live_ops_arr = live_ops_full["operations"].as_array().unwrap();
        assert_eq!(live_ops_arr.len(), 150);
        assert!(packing_ids_present(live_ops_arr));
        let live_morphisms = mcp
            .call("synth_ontology_morphisms", json!({}))
            .unwrap();
        assert_eq!(live_morphisms["count"], 150);
        assert_eq!(live_morphisms["morphisms"].as_array().unwrap().len(), 40);
        let live_morphisms_full = mcp
            .call(
                "synth_ontology_morphisms",
                json!({"detail": "full", "limit": 150}),
            )
            .unwrap();
        assert_eq!(live_morphisms_full["morphisms"].as_array().unwrap().len(), 150);
        assert!(packing_ids_present(
            live_morphisms_full["morphisms"].as_array().unwrap()
        ));

        let mcp_tools = catalogue_json();
        assert_eq!(mcp_tools["admitted_morphisms"], 150);
        let mcp_ops = mcp_tools["asset_operations"]["operations"]
            .as_array()
            .expect("mcp-tools asset_operations");
        assert_eq!(mcp_ops.len(), 150);
        assert!(
            packing_ids_present(mcp_ops),
            "mcp-tools JSON total must include packing morphisms"
        );
    }

    #[test]
    fn produce_present_schema_caps_source_and_out_hmc_pattern() {
        let tool = StandaloneSynthMcp::synth_tools()
            .into_iter()
            .find(|t| t.name == "synth_produce_present")
            .unwrap();
        let props = &tool.input_schema["properties"];
        assert_eq!(
            props["source"]["maxLength"],
            4 * 1024 * 1024
        );
        assert_eq!(tool.input_schema["oneOf"], json!([{ "required": ["source"] }, { "required": ["source_path"] }]));
        assert_eq!(props["out_hmc"]["maxLength"], 96);
        let pattern = props["out_hmc"]["pattern"].as_str().unwrap();
        assert!(pattern.contains("hmc"));
        assert!(pattern.contains("A-Za-z0-9"));
    }

    #[test]
    fn produce_present_refuses_writes_without_native_root() {
        let mut mcp = StandaloneSynthMcp::default();
        let err = mcp
            .call(
                "synth_produce_present",
                json!({"source": SOURCE, "out_hmc": "note.hmc"}),
            )
            .unwrap_err();
        assert_eq!(err.code, "native_output_unconfigured");
        let ok = mcp
            .call("synth_produce_present", json!({"source": SOURCE}))
            .unwrap();
        assert_eq!(ok["strict_replay_verified"], true);
    }

    #[test]
    fn produce_present_writes_through_native_output_root_create_only() {
        let dir = unique_dir();
        let mut mcp = StandaloneSynthMcp::default()
            .with_output_root(NativeOutputRoot::admit(&dir).unwrap());
        let result = mcp
            .call(
                "synth_produce_present",
                json!({
                    "source": SOURCE,
                    "out_hmc": "note.hmc",
                    "present_html": "note.html"
                }),
            )
            .unwrap();
        assert_eq!(Path::new(result["saved_hmc"].as_str().unwrap()), dir.canonicalize().unwrap().join("note.hmc"));
        assert!(dir.join("note.hmc").is_file());
        assert!(dir.join("note.html").is_file());
        let confined_html = dir
            .join("note.html")
            .canonicalize()
            .unwrap()
            .display()
            .to_string();
        assert_eq!(result["presentation_html"], confined_html);
        let hint = result["antigravity_embed_hint"].as_str().unwrap();
        let src = hint
            .strip_prefix("<agent-embed src=\"")
            .and_then(|s| s.strip_suffix("\"></agent-embed>"))
            .unwrap_or(hint);
        assert!(
            !src.contains("?/C:"),
            "embed hint must not contain verbatim ?/C:, got {hint}"
        );
        #[cfg(windows)]
        assert!(
            src.starts_with("file:///C:"),
            "Windows embed hint must start with file:///C:, got {hint}"
        );
        #[cfg(not(windows))]
        assert!(
            src.starts_with(&format!(
                "file:///{}",
                confined_html.trim_start_matches('/')
            )),
            "unix embed hint must be file:/// + abs path, got {hint}"
        );
        let create_only = mcp
            .call(
                "synth_produce_present",
                json!({"source": SOURCE, "out_hmc": "note.hmc"}),
            )
            .unwrap_err();
        assert_eq!(create_only.code, "native_output_refused");
        let traversal = mcp
            .call(
                "synth_produce_present",
                json!({"source": SOURCE, "out_hmc": "../escape.hmc"}),
            )
            .unwrap_err();
        assert_eq!(traversal.code, "native_output_refused");
        let nested = mcp
            .call(
                "synth_produce_present",
                json!({"source": SOURCE, "out_hmc": "nested/dir.hmc"}),
            )
            .unwrap_err();
        assert_eq!(nested.code, "native_output_refused");
        let win_slash = mcp
            .call(
                "synth_produce_present",
                json!({"source": SOURCE, "out_hmc": r"..\escape.hmc"}),
            )
            .unwrap_err();
        assert_eq!(win_slash.code, "native_output_refused");
        assert!(!dir.join("escape.hmc").exists());
        assert!(!dir.parent().unwrap().join("escape.hmc").exists());
        let _ = std::fs::remove_dir_all(dir);
    }
}
