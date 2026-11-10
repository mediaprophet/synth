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
    output_root: Option<NativeOutputRoot>,
}

impl Default for StandaloneSynthMcp {
    fn default() -> Self {
        Self {
            intent: McpIntentFrame::default(),
            output_root: None,
        }
    }
}

impl StandaloneSynthMcp {
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
                    "Inspect the {} asset production morphisms, their Domain -> Codomain signatures, AgenticUtility teleology, and NQuin48 contracts",
                    crate::production::assets::OPERATIONS.len()
                ),
                input_schema: object(json!({"category": {"type": "string", "maxLength": 64}}), vec![]),
            },
            AuthoringTool {
                name: "synth_operations".into(),
                description: format!(
                    "List the {} admitted shared-library asset operations and ontological contracts from the live catalogue",
                    crate::production::assets::OPERATIONS.len()
                ),
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
                        "source": {
                            "type": "string",
                            "maxLength": ProductionLimits::default().source.submission_bytes
                        },
                        "profile": {"type": "string", "maxLength": 64},
                        "out_hmc": {
                            "type": "string",
                            "maxLength": 96,
                            "pattern": "^[A-Za-z0-9_-][A-Za-z0-9._-]*\\.hmc$"
                        },
                        "present_html": {
                            "type": "string",
                            "maxLength": 96,
                            "pattern": "^[A-Za-z0-9_-][A-Za-z0-9._-]*\\.html$"
                        }
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
                    let hmc_path = self.confined_destination(
                        hmc_str,
                        ".hmc",
                        "Use a visible HMC filename without directory components",
                    )?;
                    qualia_core_db::poet_host::invoke::studio_store::checkpoint::save_bundle(
                        &hmc_path,
                        candidate.bundle(),
                        qualia_core_db::poet_host::invoke::studio_store::checkpoint::ExpectedFile::Absent,
                        limits.package_bytes,
                    )
                    .map_err(|e| ToolError::new("hmc_save_failed", e))?;
                }
                let mut saved_html = None;
                if let Some(html_str) = &parsed.present_html {
                    let html_path = self.confined_destination(
                        html_str,
                        ".html",
                        "Use a visible HTML filename without directory components",
                    )?;
                    let html = crate::present::render_session_html(&session)
                        .map_err(|e| ToolError::new("present_render_failed", e))?;
                    crate::present::write_presentation_file(&html_path, &html, limits.package_bytes)
                        .map_err(|e| ToolError::new("present_save_failed", e))?;
                    saved_html = Some(html_path.display().to_string());
                }
                let mut summary = crate::present::session_presentation_summary(
                    &session,
                    saved_html.as_deref(),
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
    fn operations_and_catalogue_match_live_148_ontology_including_packing() {
        assert_eq!(crate::production::assets::OPERATIONS.len(), 148);
        assert_eq!(crate::production::assets::ontology::MORPHISMS.len(), 148);
        let catalogue = crate::production::assets::catalogue();
        let operations = catalogue["operations"]
            .as_array()
            .expect("catalogue operations array");
        assert_eq!(operations.len(), 148);
        assert_eq!(catalogue["count"], 148);
        assert!(
            packing_ids_present(operations),
            "live catalogue must include packing morphisms {PACKING_MORPHISMS:?}"
        );

        let tools = StandaloneSynthMcp::synth_tools();
        let morphisms = tools
            .iter()
            .find(|t| t.name == "synth_ontology_morphisms")
            .expect("synth_ontology_morphisms");
        let operations_tool = tools
            .iter()
            .find(|t| t.name == "synth_operations")
            .expect("synth_operations");
        assert!(morphisms.description.contains("148"));
        assert!(operations_tool.description.contains("148"));
        assert!(!morphisms.description.contains("114"));
        assert!(!morphisms.description.contains("145"));
        assert!(!operations_tool.description.contains("114"));
        assert!(!operations_tool.description.contains("145"));

        let mut mcp = StandaloneSynthMcp::default();
        let live_ops = mcp.call("synth_operations", json!({})).unwrap();
        let live_ops_arr = live_ops["operations"].as_array().unwrap();
        assert_eq!(live_ops_arr.len(), 148);
        assert_eq!(live_ops["count"], 148);
        assert!(packing_ids_present(live_ops_arr));
        let live_morphisms = mcp
            .call("synth_ontology_morphisms", json!({}))
            .unwrap();
        assert_eq!(live_morphisms["count"], 148);
        assert_eq!(live_morphisms["morphisms"].as_array().unwrap().len(), 148);
        assert!(packing_ids_present(
            live_morphisms["morphisms"].as_array().unwrap()
        ));

        let mcp_tools = catalogue_json();
        assert_eq!(mcp_tools["admitted_morphisms"], 148);
        let mcp_ops = mcp_tools["asset_operations"]["operations"]
            .as_array()
            .expect("mcp-tools asset_operations");
        assert_eq!(mcp_ops.len(), 148);
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
            ProductionLimits::default().source.submission_bytes
        );
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
        assert_eq!(result["saved_hmc"], "note.hmc");
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
        assert_eq!(create_only.code, "hmc_save_failed");
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
