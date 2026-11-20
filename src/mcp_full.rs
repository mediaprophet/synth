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
        crate::session_admission::init_gpu_policy_on_startup();
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
                name: "synth_find".into(),
                description: "Find QualiaDB / Etalon / Hypermedia morphisms and Synth MCP tools by keyword. q=xpbd|nla|ik|navmesh|camera|input|gis|osm|n3|sparql|hmc|q42. Synth is the development tool; QualiaPortal is the visitor runtime.".into(),
                input_schema: object(crate::agent_index::find_arg_schema(), vec!["q"]),
            },
            AuthoringTool {
                name: "synth_capabilities".into(),
                description: "Webizen AI Synth product card (development tool, not the visitor runtime) plus admitted Vibe/Etalon providers and the SDK-to-Etalon NQuin48 parity probe".into(),
                input_schema: object(json!({}), vec![]),
            },
            AuthoringTool {
                name: "synth_ontology_categories".into(),
                description: "Inspect the 12 Hypermedia/Synth categories (locus manifold surface radiance glyph layout kinetics acoustics electromagnetics geodesy discourse chronicle) with example morphism ids. Use synth_find to search.".into(),
                input_schema: object(json!({}), vec![]),
            },
            AuthoringTool {
                name: "synth_ontology_morphisms".into(),
                description: format!(
                    "Inspect the {} asset production morphisms (paged: category, id, q, offset, limit, detail=ids|summary|full). Default 40 summaries. q=xpbd searches kinetics.",
                    crate::production::assets::OPERATIONS.len()
                ),
                input_schema: object(catalog::filter_schema(), vec![]),
            },
            AuthoringTool {
                name: "synth_operations".into(),
                description: format!(
                    "List the {} admitted QualiaDB/Etalon/Hypermedia asset operations (paged: category, id, q, offset, limit, detail=ids|summary|full). Default 40 summaries so Grok MCP does not overflow. Prefer synth_find for keyword lookup.",
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
                description: "Parse N3 (preferred; N3Logic + SHACL), Turtle, N-Triples, or yaml-ld-q42 (.yaml/.yml/.q42.yaml) — or copy an existing .q42 — from SYNTH_INPUT_ROOT or inline rdf_data, seal a create-only .q42 under SYNTH_OUTPUT_ROOT, optionally ingest the daemon graph. Query with query_sparql vault_path.".into(),
                input_schema: {
                    let mut schema = object(
                    json!({
                        "source_path": {"type":"string","maxLength":4096,"description":"N3/TTL/NT/YAML-LD-Q42/Q42 file under SYNTH_INPUT_ROOT. Prefer .n3 or .q42.yaml."},
                        "rdf_data": {"type":"string","maxLength":16777216,"description":"Inline N3 (preferred), Turtle, or yaml-ld-q42 when the agent fetched a datasource"},
                        "format": {"type":"string","enum":["ttl","turtle","n3","nt","ntriples","yaml","yml","yaml-ld-q42","q42"]},
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
                description: "Semantic bookmarks for N3/RDF dumps, SPARQL endpoints, and GIS sources (OSM/DEM/STAC later). Store public metadata in .synth/bookmarks.json. Agent enrichment: data_endpoint when the user pasted a landing page, requires_api_key / api_key_env when auth is needed. action=set_secret writes .synth/secrets.json (gitignored); list/get never return secret values. Fetch with synth_jobs into .synth/inbox; compile N3 with synth_compile_q42.".into(),
                input_schema: object(
                    json!({
                        "action": {"type":"string","enum":["list","get","upsert","forget","set_secret"]},
                        "id": {"type":"string","maxLength":96},
                        "url": {"type":"string","maxLength":2048,"description":"http(s) landing page, SPARQL endpoint, or RDF dump. Credential query params are stripped."},
                        "title": {"type":"string","maxLength":256},
                        "kind": {"type":"string","enum":["sparql","rdf","gis","rest","csv","page"]},
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
                description: "Downloader and scheduler. Fetch http(s) into .synth/inbox (gitignored staging), wrap PROV/DCT/DCAT, compile N3/Turtle to .q42. action=add|list|get|run|run_due|enable|disable|forget. kind=fetch|ingest|compile. GIS/OSM/DEM/STAC ingest is opt-in later; geodesy morphisms bake maps now. Operator ticks due jobs; synth mcp does not start the scheduler. Max 16 MiB. Secrets stay in .synth/secrets.json.".into(),
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
                name: "synth_annotations".into(),
                description: "List, add, or mark addressed spatial/temporal viewport annotations (.synth/annotations.json) so agents can close the visual/motion review feedback loop (REQ-2.4).".into(),
                input_schema: object(
                    json!({
                        "action": {"type":"string","enum":["list","add","mark_addressed"]},
                        "project_dir": {"type":"string","maxLength":4096},
                        "id": {"type":"string","maxLength":128},
                        "lane": {"type":"string","enum":["working","delivered"]},
                        "file": {"type":"string","maxLength":512},
                        "comment": {"type":"string","maxLength":8192},
                        "rating": {"type":"integer","minimum":0,"maximum":5},
                        "camera_yaw": {"type":"number"},
                        "camera_pitch": {"type":"number"},
                        "camera_distance": {"type":"number"},
                        "time_seconds": {"type":"number"},
                        "frame_index": {"type":"integer"}
                    }),
                    vec![],
                ),
            },
            AuthoringTool {
                name: "synth_sessions".into(),
                description: "Inspect multi-agent concurrent production sessions, resource admission slots (SYNTH_OUTPUT_ROOT/.synth/slots.json), and Local GPU / Port 4242 runtime profile (REQ-4 & REQ-5).".into(),
                input_schema: object(
                    json!({
                        "gpu_backend": {"type":"string","enum":["auto","vulkan","dx12","metal","gl","webgpu-wasm","webgl2-wasm","cpu"]}
                    }),
                    vec![],
                ),
            },
            AuthoringTool {
                name: "synth_gpu".into(),
                description: "Inspect or select Synth's physical GPU adapter preference (default: discrete-only, refusing Intel/on-board integrated GPUs so the host desktop never throttles or locks up) and graphics backend (default: vulkan; also dx12, metal, gl, auto). Persists in .synth/gpu-config.json.".into(),
                input_schema: object(
                    json!({
                        "action": {"type":"string","enum":["status","list","set"]},
                        "preference": {"type":"string","description":"discrete-only (default; blocks on-board iGPU), discrete, integrated, or name:<substring>"},
                        "backend": {"type":"string","enum":["vulkan","dx12","metal","gl","auto","cpu"]},
                        "reject_igpu": {"type":"boolean","description":"When true (default), strictly refuse DeviceType::IntegratedGpu adapters"},
                        "adapter_name_filter": {"type":"string","maxLength":128,"description":"Optional substring filter for adapter name (e.g. RTX, NVIDIA)"}
                    }),
                    vec![],
                ),
            },
            AuthoringTool {
                name: "synth_changelog_license".into(),
                description: "Inspect or update an asset's SVN-style signed revision changelog (.synth/changelog.json) and Dual-Regime Semantic License Contract (.synth/license-contract.json: PermissiveCommonsPaydown vs. PerpetualCreativeRoyalty vs. CC-BY-NC-ND-4.0) (REQ-6).".into(),
                input_schema: object(
                    json!({
                        "action": {"type":"string","enum":["inspect","set_license"]},
                        "project_dir": {"type":"string","maxLength":4096},
                        "asset_id": {"type":"string","maxLength":512},
                        "regime": {"type":"string","enum":["commons_paydown","perpetual_royalty","cc_by_nc_nd","qualia_cc_by_nc_nd","sanctuary"]},
                        "target_pool_t_cents": {"type":"integer","minimum":0},
                        "roi_multiplier": {"type":"number","minimum":1.0,"maximum":10.0},
                        "settled_s_cents": {"type":"integer","minimum":0},
                        "upstream_source_license": {"type":"string","maxLength":128}
                    }),
                    vec![],
                ),
            },
            AuthoringTool {
                name: "synth_validate_model".into(),
                description: "Validate an asset's computational model parameters (required physical/anatomical/kinetic fields by subject_class, ClaimBasis/EpistemicTier classification, 1..N reference bindings, QUDT bounds, and .10d HOLO/MSTR/CBLD/KLOD0..KLOD3 sections) (REQ-7 & REQ-8).".into(),
                input_schema: object(
                    json!({
                        "subject_class": {"type":"string","enum":["VascularPlant","Flora","VertebrateFauna","HumanAnatomy","MechanicalVehicle","MadeArtefact"]},
                        "asset_10d_path": {"type":"string","maxLength":4096},
                        "fields": {"type":"array","items":{"type":"object"}}
                    }),
                    vec![],
                ),
            },
            AuthoringTool {
                name: "synth_verify_outputs".into(),
                description: "Run automated multi-format render and output verification (.10d CRC-32C/frustum bounds/KLOD, .png, .svg, .emf, .m64, .q42, .hmc, .mp4, .wav, .qac, .dxf) and return a pass|fail|error matrix (REQ-9).".into(),
                input_schema: object(
                    json!({
                        "export_dir": {"type":"string","maxLength":4096}
                    }),
                    vec![],
                ),
            },
            AuthoringTool {
                name: "synth_convert_3d".into(),
                description: "Convert an existing 3D formatted asset or multi-node scene (.glb/.gltf static or living scene with skins/anims/morphs, .obj, .stl, or legacy .10d) into a Holographic .10d v2 Mode-A Archetype + Mode-B scene instances + attested PRV1/CBLD source-licensing & provenance package (Phase 1).".into(),
                input_schema: object(
                    json!({
                        "source_path": {"type":"string","maxLength":4096},
                        "out_dir": {"type":"string","maxLength":4096},
                        "format": {"type":"string","enum":["glb","gltf","obj","stl","10d"]},
                        "asset_uri": {"type":"string","maxLength":1024},
                        "domain_id": {"type":"integer","minimum":1},
                        "spdx_license": {"type":"string","maxLength":128,"description":"Mandatory source license identifier (e.g. CC-BY-NC-ND-4.0, CC-BY-4.0, PC-PAYDOWN-1.0)"},
                        "creator": {"type":"string","maxLength":512},
                        "attribution_notice": {"type":"string","maxLength":2048,"description":"Mandatory UTF-8 attribution and copyright notice"},
                        "origin_uri": {"type":"string","maxLength":2048},
                        "embed_full_source_bytes": {"type":"boolean"}
                    }),
                    vec!["source_path","out_dir","spdx_license","attribution_notice"],
                ),
            },
            AuthoringTool {
                name: "synth_produce_present".into(),
                description: "Execute QualiaDB/Etalon/Hypermedia asset operations inside one bounded ProductionSession (development tool, not the visitor runtime). Bind local inputs under SYNTH_INPUT_ROOT; save create-only HMC/HTML and export resources under SYNTH_OUTPUT_ROOT. New scene assets are extra HMC/.10d/.q42 files; the host does not rebuild. Handles expire after source evaluation. Biosphere non-fiction (domain=biosphere, fiction=false) requires evidence_q42 or ecopunk/entities/taxa/{taxon}/*.q42.".into(),
                input_schema: {
                    let mut schema = object(
                    json!({
                        "source": {
                            "type": "string",
                            "maxLength": 4 * 1024 * 1024
                        },
                        "source_path": {"type":"string", "maxLength":4096, "description":"UTF-8 source file under SYNTH_INPUT_ROOT; provide exactly one of source and source_path."},
                        "profile": {"type": "string", "maxLength": 64},
                        "quality_profile": {"type":"string","enum":["preview","production16","authoring"],"description":"Default production16/authoring admits a 128 MiB packed .10d ring, 2 GiB aggregate, and 2304 MiB package so raw CAD (boolean unions) can stay complicated. The 128 MiB ring is the graphics-container exception; it is not the 42 MiB Q42 Sentinel and it is not the p64/LLM VRAM ledger (6 GiB default). preview is the conservative 16/64/80 MiB leftover. Mesh size caps stay 200k/400k. Runtime cost uses hypermedia.manifold.simplify mode=optimize; that does not replace or degrade the source solid."},
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
                        "evidence_q42": {"type":"string","maxLength":4096,"description":"Path to a sealed evidence .q42 (under SYNTH_INPUT_ROOT or absolute)"},
                        "session_id": {"type":"string","maxLength":128,"description":"Optional multi-agent session ID for slot lease management"},
                        "agent_id": {"type":"string","maxLength":128,"description":"Optional calling agent ID for slot attribution"},
                        "wait_ms": {"type":"integer","minimum":0,"maximum":15000,"description":"Optional FIFO wait time in ms when production slots are busy"},
                        "gpu_backend": {"type":"string","enum":["auto","vulkan","dx12","metal","gl","webgpu-wasm","webgl2-wasm","cpu"]},
                        "verify_outputs": {"type":"boolean","description":"Automatically run multi-format output verification (REQ-9.1) on export_dir"}
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
        Some(crate::agent_index::mcp_instructions(
            self.input_root.is_some(),
            self.output_root.is_some(),
        ))
    }
    fn tools(&self) -> Vec<AuthoringTool> {
        let mut all = Self::synth_tools();
        all.extend(qualia_engine_authoring_tools());
        all
    }

    fn call(&mut self, name: &str, args: Value) -> Result<Value, ToolError> {
        let result = match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            self.call_inner(name, args)
        })) {
            Ok(res) => res,
            Err(_) => Err(ToolError::new(
                "internal_panic",
                format!("Tool `{name}` caught an internal panic; MCP daemon remains online"),
            )),
        };
        #[cfg(not(target_arch = "wasm32"))]
        {
            let detail = result.as_ref().err().map(|e| e.message.as_str());
            crate::operator::record_event(name, result.is_ok(), detail);
        }
        result
    }
}

impl StandaloneSynthMcp {
    fn call_inner(&mut self, name: &str, args: Value) -> Result<Value, ToolError> {
        match name {
            "synth_find" | "synth.find" => {
                let q = args
                    .get("q")
                    .and_then(Value::as_str)
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .ok_or_else(|| ToolError::new("invalid_arguments", "q is required (e.g. xpbd, nla, n3, gis)"))?;
                let limit = args.get("limit").and_then(Value::as_u64).unwrap_or(20) as usize;
                Ok(crate::agent_index::search(q, limit))
            }
            "synth_capabilities" | "synth.capabilities" => {
                crate::vibe_runtime::inspect().map_err(|e| ToolError::new("capabilities_failed", e))
            }
            "synth_ontology_categories" | "synth.ontology.categories" => {
                Ok(crate::agent_index::categories_for_agents())
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
            "synth_annotations" | "synth.annotations" => {
                crate::project::dispatch_annotations_mcp(args)
                    .map_err(|e| ToolError::new("annotations_failed", e))
            }
            "synth_sessions" | "synth.sessions" => {
                let backend = args.get("gpu_backend").and_then(Value::as_str);
                Ok(json!({
                    "sessions": crate::session_admission::session_admission_status(),
                    "gpu_runtime_profile": crate::session_admission::detect_runtime_and_gpu_profile(backend),
                    "build_identity": crate::session_admission::build_identity_and_drift_report(),
                }))
            }
            "synth_gpu" | "synth.gpu" => {
                let action = args.get("action").and_then(Value::as_str).unwrap_or("status");
                let has_updates = args.get("preference").is_some()
                    || args.get("backend").is_some()
                    || args.get("reject_igpu").is_some()
                    || args.get("adapter_name_filter").is_some();
                if action == "set" || has_updates {
                    let mut cfg = crate::session_admission::load_gpu_selection_config();
                    if let Some(p) = args.get("preference").and_then(Value::as_str) {
                        cfg.preference = p.to_string();
                        if cfg.preference == "discrete-only" {
                            cfg.reject_igpu = true;
                        } else if cfg.preference == "integrated" {
                            cfg.reject_igpu = false;
                        }
                    }
                    if let Some(b) = args.get("backend").and_then(Value::as_str) {
                        cfg.backend = b.to_string();
                    }
                    if let Some(r) = args.get("reject_igpu").and_then(Value::as_bool) {
                        cfg.reject_igpu = r;
                    }
                    if let Some(f) = args.get("adapter_name_filter") {
                        cfg.adapter_name_filter = f
                            .as_str()
                            .map(str::trim)
                            .filter(|s| !s.is_empty() && *s != "none")
                            .map(str::to_string);
                    }
                    crate::session_admission::save_and_apply_gpu_selection_config(cfg)
                        .map_err(|e| ToolError::new("gpu_selection_failed", e))
                } else {
                    Ok(crate::session_admission::detect_runtime_and_gpu_profile(None))
                }
            }
            "synth_changelog_license" | "synth.changelog.license" => {
                crate::changelog_licensing::dispatch_changelog_license_mcp(args)
                    .map_err(|e| ToolError::new("changelog_license_failed", e))
            }
            "synth_validate_model" | "synth.validate.model" => {
                crate::model_validation::validate_computational_model_form(args)
                    .map_err(|e| ToolError::new("validate_model_failed", e))
            }
            "synth_verify_outputs" | "synth.verify.outputs" => {
                crate::model_validation::verify_multi_format_outputs(args)
                    .map_err(|e| ToolError::new("verify_outputs_failed", e))
            }
            "synth_convert_3d" | "synth.convert.3d" => {
                crate::model_validation::convert_3d_scene_or_asset_cli_or_mcp(args)
                    .map_err(|e| ToolError::new("convert_3d_failed", e))
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
        assert!(
            tools.iter().any(|t| t.name == "synth_find"),
            "synth_find must be advertised so agents can keyword-search morphisms"
        );
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
        let found = mcp.call("synth_find", json!({"q": "xpbd"})).unwrap();
        assert_eq!(found["schema"], "synth.find/v1");
        assert!(found["morphisms"].as_array().unwrap().iter().any(|m| {
            m["id"] == "hypermedia.kinetics.step_island_xpbd"
        }));
        let ops_q = mcp
            .call("synth_operations", json!({"q": "xpbd", "limit": 20}))
            .unwrap();
        assert!(ops_q["count"].as_u64().unwrap() >= 1);
        assert!(ops_q["count"].as_u64().unwrap() < 150);
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

    #[test]
    fn synth_2026_11_requirements_sessions_changelog_validation_verification_and_convert3d_pass() {
        let dir = unique_dir();
        let mut mcp = StandaloneSynthMcp::default()
            .with_output_root(NativeOutputRoot::admit(&dir).unwrap());

        // REQ-1, REQ-4, REQ-5: synth_sessions returns build_identity, gpu_runtime_profile, and session slots
        let sess = mcp.call("synth_sessions", json!({"gpu_backend": "vulkan"})).unwrap();
        assert_eq!(sess["build_identity"]["license"], "CC-BY-NC-ND-4.0");
        assert_eq!(sess["gpu_runtime_profile"]["selected_gpu_backend"], "vulkan");
        assert_eq!(sess["sessions"]["schema"], "synth.resource-admission-slots/v1");

        // REQ-6: synth_changelog_license sets & validates a Dual-Regime License Contract (SHACL roi_multiplier cap)
        let lic = mcp
            .call(
                "synth_changelog_license",
                json!({
                    "action": "set_license",
                    "project_dir": dir.to_string_lossy(),
                    "asset_id": "urn:qualia:test:work",
                    "regime": "commons_paydown",
                    "target_pool_t_cents": 100000,
                    "roi_multiplier": 2.5,
                    "settled_s_cents": 40000
                }),
            )
            .unwrap();
        assert_eq!(lic["status"], "ok");
        assert_eq!(lic["remaining_recovery_cents"], 60000);
        assert_eq!(lic["is_commons_discharged"], false);

        // REQ-8: synth_validate_model checks required fields, ClaimBasis, and ReferenceBindings
        let form = mcp
            .call(
                "synth_validate_model",
                json!({
                    "subject_class": "MechanicalVehicle",
                    "fields": [
                        {"name": "length_m", "value": 4.95, "qudt_unit": "qudt:Meter", "claim_basis": "Measured", "references": [{"source_id": "spec-01", "kind": "standard"}]},
                        {"name": "width_m", "value": 1.85, "qudt_unit": "qudt:Meter", "claim_basis": "Measured", "references": [{"source_id": "spec-01", "kind": "standard"}]},
                        {"name": "height_m", "value": 1.92, "qudt_unit": "qudt:Meter", "claim_basis": "Measured", "references": [{"source_id": "spec-01", "kind": "standard"}]},
                        {"name": "dry_mass_kg", "value": 2150.0, "qudt_unit": "qudt:Kilogram", "claim_basis": "Measured", "references": [{"source_id": "spec-01", "kind": "standard"}]},
                        {"name": "youngs_modulus_mpa", "value": 210000.0, "claim_basis": "Derived"},
                        {"name": "yield_stress_mpa", "value": 350.0, "claim_basis": "Derived"},
                        {"name": "kinematic_solver_regime", "value": "ArticulatedRigidBodyDca", "claim_basis": "ModelAssumption"},
                        {"name": "multiscale_strata_l0_l6", "value": ["L0","L1","L2","L5","L6"], "claim_basis": "Described", "references": [{"source_id": "q42-01", "kind": "q42_assertion"}]}
                    ]
                }),
            )
            .unwrap();
        assert_eq!(form["status"], "pass");
        assert_eq!(form["populated_required"], 8);

        // Phase 1 & REQ-9: synth_convert_3d converts an OBJ with CC-BY-NC-ND-4.0 license and synth_verify_outputs passes
        let obj_path = dir.join("source_tri.obj");
        std::fs::write(&obj_path, b"v 0 0 0\nv 1 0 0\nv 0 1 0\nf 1 2 3\n").unwrap();
        let conv_out = dir.join("converted_out");
        let conv = mcp
            .call(
                "synth_convert_3d",
                json!({
                    "source_path": obj_path.to_string_lossy(),
                    "out_dir": conv_out.to_string_lossy(),
                    "spdx_license": "CC-BY-NC-ND-4.0",
                    "creator": "Timothy Charles Holborn",
                    "attribution_notice": "Copyright (c) 2026 Timothy Charles Holborn — CC BY-NC-ND 4.0",
                    "asset_uri": "urn:qualia:test:converted_tri"
                }),
            )
            .unwrap();
        assert_eq!(conv["status"], "ok");
        assert_eq!(conv["changelog_head_rev"], 1);

        let ver = mcp
            .call(
                "synth_verify_outputs",
                json!({ "export_dir": conv_out.to_string_lossy() }),
            )
            .unwrap();
        assert_eq!(ver["overall_technical_status"], "pass");
        assert!(ver["checked_count"].as_u64().unwrap() >= 1);

        let _ = std::fs::remove_dir_all(dir);
    }
}
