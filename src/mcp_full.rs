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

#[derive(Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct MorphismsArg {
    category: Option<String>,
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
                name: "synth_operation_contracts".into(),
                description: "Inspect execution host, handle lifetime and explicit encoded/record output recipes. Recipes declare field extraction and encoding; never infer output types from namespaces.".into(),
                input_schema: object(json!({}), vec![]),
            },
            AuthoringTool {
                name: "synth_produce_present".into(),
                description: "Execute Hypermedia asset operations inside one bounded ProductionSession. Bind local inputs under SYNTH_INPUT_ROOT; save create-only HMC/HTML and export resources under SYNTH_OUTPUT_ROOT. Handles expire after source evaluation. Return hashes, absolute resource paths and optional native review receipts.".into(),
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
                        }
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
        Some(format!("Synth exposes {} asset morphisms through synth_operations and synth_ontology_morphisms. Execute them using capability.invoke inside synth_produce_present source (production AssetHost); vibe_invoke/vibe_catalog use a separate generic Poet snapshot. Handles live only within one source evaluation. Use synth_operation_contracts for explicit output recipes and emit encoded resources via asset.files. Bind inputs with key/kind/path under SYNTH_INPUT_ROOT; exports/HMC/HTML/receipt destinations are create-only names under SYNTH_OUTPUT_ROOT. Inspect returned PNG/SVG/native review files before judging quality. Tool names use underscores. Input root configured: {}; output root configured: {}.", crate::production::assets::OPERATIONS.len(), self.input_root.is_some(), self.output_root.is_some()))
    }
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
            "synth_operation_contracts" => Ok(contracts::catalogue()),
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
        }
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
        let live_ops_arr = live_ops["operations"].as_array().unwrap();
        assert_eq!(live_ops_arr.len(), 150);
        assert_eq!(live_ops["count"], 150);
        assert!(packing_ids_present(live_ops_arr));
        let live_morphisms = mcp
            .call("synth_ontology_morphisms", json!({}))
            .unwrap();
        assert_eq!(live_morphisms["count"], 150);
        assert_eq!(live_morphisms["morphisms"].as_array().unwrap().len(), 150);
        assert!(packing_ids_present(
            live_morphisms["morphisms"].as_array().unwrap()
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
