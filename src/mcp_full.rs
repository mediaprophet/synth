//! Comprehensive MCP bridge uniting QualiaDB's full 60-tool `qualia-mcp` suite
//! (`engine-tools`) with Synth's categorical ontology, VibeScript production,
//! and Antigravity HTML presentation (`synth.presentation-html/v1`).
use crate::production::{ProducerRegistry, ProductionLimits};
use qualia_mcp::{
    authoring::{AuthoringBackend, AuthoringTool, ProtocolLimits, ToolError},
    dispatch::enforce_fiduciary_tool_dispatch,
    registry::stable_mcp_tools,
    types::{McpIntentFrame, RawToolPayload},
};
use serde::Deserialize;
use serde_json::{json, Value};
use std::{
    path::PathBuf,
    sync::atomic::AtomicBool,
};

#[derive(Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct MorphismsArg {
    category: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ProducePreviewArg {
    source: String,
    profile: Option<String>,
    out_hmc: Option<String>,
    present_html: Option<String>,
}

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
        Err(err) => Err(ToolError::new(
            "qualia_engine_error",
            format!("QualiaDB MCP tool '{tool_id}' failed: {err:?}"),
        )),
    })
}

/// Standalone comprehensive MCP backend for `synth mcp` (Antigravity IDE, Antigravity 2.0,
/// and `vendor/antigravity-sdk-python` `McpStdioServer`), exposing all 60 QualiaDB MCP
/// tools plus Synth's standalone capabilities, ontology, and HTML presentation tools.
pub struct StandaloneSynthMcp {
    intent: McpIntentFrame,
}

impl Default for StandaloneSynthMcp {
    fn default() -> Self {
        Self {
            intent: McpIntentFrame::default(),
        }
    }
}

impl StandaloneSynthMcp {
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
                description: "Inspect the 114 asset production morphisms, their Domain -> Codomain signatures, AgenticUtility teleology, and NQuin48 contracts".into(),
                input_schema: object(json!({"category": {"type": "string", "maxLength": 64}}), vec![]),
            },
            AuthoringTool {
                name: "synth_operations".into(),
                description: "List the admitted shared-library asset operations and ontological contracts".into(),
                input_schema: object(json!({}), vec![]),
            },
            AuthoringTool {
                name: "synth_profiles".into(),
                description: "List registered AssetProducer profiles and versions".into(),
                input_schema: object(json!({}), vec![]),
            },
            AuthoringTool {
                name: "synth_environment_bootstrap".into(),
                description: "Bootstrap Synth's self-produced Aether Tray & Studio environment (examples/tray_environment.vibe), probe Webizen Desktop presence, and scan the local LLM model vault".into(),
                input_schema: object(json!({}), vec![]),
            },
            AuthoringTool {
                name: "synth_produce_present".into(),
                description: "Evaluate VibeScript source in a bounded ProductionSession, verify strict replay, and optionally write create-only .hmc and Antigravity/Browser Preview .html showcase files".into(),
                input_schema: object(
                    json!({
                        "source": {"type": "string"},
                        "profile": {"type": "string", "maxLength": 64},
                        "out_hmc": {"type": "string", "maxLength": 256},
                        "present_html": {"type": "string", "maxLength": 256}
                    }),
                    vec!["source"],
                ),
            },
        ]
    }
}

impl AuthoringBackend for StandaloneSynthMcp {
    fn tools(&self) -> Vec<AuthoringTool> {
        let mut all = Self::synth_tools();
        all.extend(qualia_engine_authoring_tools());
        all
    }

    fn call(&mut self, name: &str, args: Value) -> Result<Value, ToolError> {
        match name {
            "synth_capabilities" | "synth.capabilities" => {
                crate::vibe_runtime::inspect().map_err(|e| ToolError::new("capabilities_failed", e))
            }
            "synth_ontology_categories" | "synth.ontology.categories" => {
                Ok(crate::production::assets::ontology::categories_json())
            }
            "synth_ontology_morphisms" | "synth.ontology.morphisms" => {
                let parsed: MorphismsArg = serde_json::from_value(args)
                    .map_err(|_| ToolError::new("invalid_arguments", "Invalid category filter"))?;
                Ok(crate::production::assets::ontology::morphisms_json(
                    parsed.category.as_deref(),
                ))
            }
            "synth_operations" | "synth.operations" => Ok(crate::production::assets::catalogue()),
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
                let parsed: ProducePreviewArg = serde_json::from_value(args).map_err(|_| {
                    ToolError::new("invalid_arguments", "Invalid synth_produce_present arguments")
                })?;
                let limits = ProductionLimits::default();
                let cancel = AtomicBool::new(false);
                let registry = ProducerRegistry::with_builtins();
                let mut session = registry
                    .session(
                        parsed.profile.as_deref().unwrap_or("files"),
                        vec![],
                        limits.clone(),
                    )
                    .map_err(|e| ToolError::new("session_error", e))?;
                session
                    .submit(&parsed.source, &cancel)
                    .map_err(|e| ToolError::new("production_failed", e))?;
                let candidate = session
                    .candidate()
                    .ok_or_else(|| ToolError::new("no_candidate", "No candidate produced"))?;
                registry
                    .reopen(candidate.bundle(), limits.clone(), &cancel)
                    .map_err(|e| ToolError::new("replay_failed", e))?;
                if let Some(hmc_str) = &parsed.out_hmc {
                    let hmc_path = PathBuf::from(hmc_str);
                    qualia_core_db::poet_host::invoke::studio_store::checkpoint::save_bundle(
                        &hmc_path,
                        candidate.bundle(),
                        qualia_core_db::poet_host::invoke::studio_store::checkpoint::ExpectedFile::Absent,
                        limits.package_bytes,
                    )
                    .map_err(|e| ToolError::new("hmc_save_failed", e))?;
                }
                if let Some(html_str) = &parsed.present_html {
                    let html_path = PathBuf::from(html_str);
                    let html = crate::present::render_session_html(&session)
                        .map_err(|e| ToolError::new("present_render_failed", e))?;
                    crate::present::write_presentation_file(&html_path, &html, limits.package_bytes)
                        .map_err(|e| ToolError::new("present_save_failed", e))?;
                }
                let mut summary = crate::present::session_presentation_summary(
                    &session,
                    parsed.present_html.as_deref(),
                )
                .map_err(|e| ToolError::new("summary_failed", e))?;
                if let Some(obj) = summary.as_object_mut() {
                    obj.insert("saved_hmc".into(), json!(parsed.out_hmc));
                    obj.insert("strict_replay_verified".into(), json!(true));
                }
                Ok(summary)
            }
            _ => {
                let raw_args = serde_json::to_vec(&args).unwrap_or_else(|_| b"{}".to_vec());
                let payload = RawToolPayload {
                    tool_name: name.as_bytes(),
                    arguments_raw: &raw_args,
                };
                match enforce_fiduciary_tool_dispatch(payload, &self.intent) {
                    Ok(text) => match serde_json::from_str::<Value>(&text) {
                        Ok(Value::Object(map)) => Ok(Value::Object(map)),
                        Ok(other) => Ok(json!({ "status": "ok", "tool": name, "result": other })),
                        Err(_) => Ok(json!({ "status": "ok", "tool": name, "text": text })),
                    },
                    Err(_) => Err(ToolError::new(
                        "unknown_or_failed_tool",
                        "MCP operation is unknown or refused execution",
                    )),
                }
            }
        }
    }
}

/// Default protocol limits accommodating the combined 66+ Synth + QualiaDB MCP tool catalogue.
pub fn comprehensive_protocol_limits() -> ProtocolLimits {
    ProtocolLimits {
        max_message_bytes: 2 * 1024 * 1024,
        max_tools: 128,
        max_id_bytes: 256,
    }
}

/// Run the standalone stdio MCP server (`synth mcp`) exposing all Synth + QualiaDB MCP tools.
pub fn run_standalone_stdio() -> Result<(), String> {
    qualia_mcp::authoring::run_stdio(StandaloneSynthMcp::default(), comprehensive_protocol_limits())
        .map_err(|e| format!("Synth MCP transport ended: {e}"))
}

/// Return the JSON catalogue of all tools exposed by `synth mcp`.
pub fn catalogue_json() -> Value {
    let tools = StandaloneSynthMcp::default().tools();
    json!({
        "schema": "synth.comprehensive-mcp/v1",
        "total_tools": tools.len(),
        "synth_tools": StandaloneSynthMcp::synth_tools().len(),
        "qualia_engine_tools": qualia_engine_authoring_tools().len(),
        "identifier_safe_for_fastmcp": tools.iter().all(|t| !t.name.contains('.')),
        "tools": tools,
    })
}
