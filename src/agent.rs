//! Existing QualiaDB MCP transport, over the SAME guarded native operations.
//! JSON here is protocol framing, never canonical asset storage.
use crate::{NativeContext, SynthHost};
use qualia_mcp::authoring::{AuthoringBackend, AuthoringTool, ToolError};
use serde::Deserialize;
use serde_json::{json, Value};
use std::{
    sync::atomic::{AtomicBool, Ordering},
    sync::{Arc, Mutex},
};

pub struct SynthAgentBackend {
    host: Arc<Mutex<SynthHost>>,
    context: Option<(String, NativeContext)>,
    connection: String,
    sequence: u64,
    cancel: Arc<AtomicBool>,
    identifier_safe_tools: bool,
    include_qualia_engine_tools: bool,
}
impl SynthAgentBackend {
    /// Native embedding injects an already admitted host. No stdio field or
    /// source string can create principal/session/project authority.
    pub fn bound(host: Arc<Mutex<SynthHost>>) -> Result<Self, String> {
        use std::hash::{BuildHasher, Hasher};
        let snapshot = host.try_lock().map_err(|_| "Synth host busy")?.inspect()?;
        if snapshot.actor_kind != qualia_cooperative_core::authoring::PrincipalKind::SoftwareAgent {
            return Err("Agent transport requires its own native authenticated software-agent session; a human session is not impersonated".into());
        }
        let connection = format!(
            "{:016x}",
            std::collections::hash_map::RandomState::new()
                .build_hasher()
                .finish()
        );
        Ok(Self {
            host,
            context: None,
            connection,
            sequence: 0,
            cancel: Arc::new(AtomicBool::new(false)),
            identifier_safe_tools: false,
            include_qualia_engine_tools: false,
        })
    }
    /// Advertise Python-identifier-safe tool names (`synth_inspect`, etc.) required by
    /// FastMCP and the Google Antigravity Python SDK (`vendor/antigravity-sdk-python`),
    /// while `call()` continues to accept both dotted and underscore tool names.
    pub fn with_identifier_safe_tools(mut self) -> Self {
        self.identifier_safe_tools = true;
        self
    }
    /// Include the full QualiaDB MCP engine tool catalogue (`qualia_mcp::stable_mcp_tools()`)
    /// alongside Synth's authenticated authoring tools.
    pub fn with_qualia_engine_tools(mut self) -> Self {
        self.include_qualia_engine_tools = true;
        self
    }
    /// Native supervision can cancel preparation without waiting for the host mutex.
    pub fn cancellation(&self) -> Arc<AtomicBool> {
        self.cancel.clone()
    }
    fn expected(&self, token: &str) -> Result<NativeContext, ToolError> {
        self.context
            .as_ref()
            .filter(|(known, _)| known == token)
            .map(|(_, c)| c.clone())
            .ok_or_else(|| failure("stale_context", "Inspect Synth again before acting"))
    }
}
fn failure(code: &str, message: &str) -> ToolError {
    ToolError::new(code, message)
}
fn native_error(_: String) -> ToolError {
    failure(
        "native_operation_refused",
        "Operation refused or context changed; inspect the admitted native job",
    )
}
fn args<T: serde::de::DeserializeOwned>(value: Value) -> Result<T, ToolError> {
    serde_json::from_value(value).map_err(|_| {
        failure(
            "invalid_arguments",
            "Invalid or unsupported operation arguments",
        )
    })
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ReadArg {
    context: String,
    cell: usize,
    offset: usize,
    bytes: usize,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SubmitArg {
    context: String,
    source: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SaveArg {
    context: String,
    filename: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PresentArg {
    context: String,
    filename: Option<String>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ReopenArg {
    context: String,
    filename: String,
    package_sha256: String,
}
#[derive(Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct MorphismsArg {
    category: Option<String>,
}
fn hex(bytes: &[u8; 32]) -> String {
    use std::fmt::Write;
    let mut text = String::with_capacity(64);
    for byte in bytes {
        let _ = write!(&mut text, "{byte:02x}");
    }
    text
}
impl AuthoringBackend for SynthAgentBackend {
    fn tools(&self) -> Vec<AuthoringTool> {
        let object = |props: Value, required: Vec<&str>| json!({"type":"object","properties":props,"required":required,"additionalProperties":false});
        let token = json!({"type":"string","maxLength":128});
        let tool_name = |dotted: &str| -> String {
            if self.identifier_safe_tools {
                dotted.replace('.', "_")
            } else {
                dotted.into()
            }
        };
        let mut tools = vec![
            AuthoringTool {name:tool_name("synth.inspect"),description:"Inspect the admitted producer, resources, ontological categories, and source context".into(),input_schema:object(json!({}),vec![])},
            AuthoringTool {name:tool_name("synth.ontology.categories"),description:"Inspect the category-theoretic ontology taxonomy (Objects, Morphism purposes, and NQuin48 counts)".into(),input_schema:object(json!({}),vec![])},
            AuthoringTool {name:tool_name("synth.ontology.morphisms"),description:format!("Inspect the {} admitted asset production morphisms, their Domain -> Codomain signatures, AgenticUtility teleology, and NQuin48 ontological contracts", crate::production::assets::OPERATIONS.len()),input_schema:object(json!({"category":{"type":"string","maxLength":64}}),vec![])},
            AuthoringTool {name:tool_name("synth.source.read"),description:"Read an authorized Vibe source cell in bounded UTF-8 chunks".into(),input_schema:object(json!({"context":token,"cell":{"type":"integer","minimum":0},"offset":{"type":"integer","minimum":0},"bytes":{"type":"integer","minimum":1,"maximum":16384}}),vec!["context","cell","offset","bytes"])},
            AuthoringTool {name:tool_name("synth.source.submit"),description:"Prepare an asset edit with the admitted producer, then promote under native authority".into(),input_schema:object(json!({"context":token,"source":{"type":"string"}}),vec!["context","source"])},
            AuthoringTool {name:tool_name("synth.package.save"),description:"Save an admitted asset HMC in the native-configured output directory".into(),input_schema:object(json!({"context":token,"filename":{"type":"string","maxLength":96}}),vec!["context","filename"])},
            AuthoringTool {name:tool_name("synth.package.reopen"),description:"Verify the same admitted production binding from saved package bytes".into(),input_schema:object(json!({"context":token,"filename":{"type":"string","maxLength":96},"package_sha256":{"type":"string","pattern":"^[0-9a-f]{64}$"}}),vec!["context","filename","package_sha256"])},
            AuthoringTool {name:tool_name("synth.package.present"),description:"Generate a self-contained HTML work presentation (Antigravity Auxiliary Pane / <agent-embed> / Browser Preview) in the native output directory".into(),input_schema:object(json!({"context":token,"filename":{"type":"string","maxLength":96}}),vec!["context"])},
        ];
        if self.include_qualia_engine_tools {
            tools.extend(crate::mcp_full::qualia_engine_authoring_tools());
        }
        tools
    }
    fn call(&mut self, name: &str, value: Value) -> Result<Value, ToolError> {
        if matches!(name, "synth.ontology.categories" | "synth_ontology_categories") {
            if value.as_object().is_none_or(|v| !v.is_empty()) {
                return Err(failure(
                    "invalid_arguments",
                    "Ontology categories takes an empty object",
                ));
            }
            return Ok(crate::production::assets::ontology::categories_json());
        }
        if matches!(name, "synth.ontology.morphisms" | "synth_ontology_morphisms") {
            let a: MorphismsArg = args(value)?;
            return Ok(crate::production::assets::ontology::morphisms_json(
                a.category.as_deref(),
            ));
        }
        if matches!(name, "synth.inspect" | "synth_inspect") {
            if value.as_object().is_none_or(|v| !v.is_empty()) {
                return Err(failure(
                    "invalid_arguments",
                    "Inspect takes an empty object",
                ));
            }
            let snapshot = self
                .host
                .try_lock()
                .map_err(|_| failure("busy", "Synth operation already running"))?
                .inspect()
                .map_err(native_error)?;
            self.sequence = self
                .sequence
                .checked_add(1)
                .ok_or_else(|| failure("exhausted", "Context sequence exhausted"))?;
            let token = format!("{}:{}", self.connection, self.sequence);
            let result = json!({"product":"Synth","context":token,"source_revision":snapshot.context.document.revision.to_string(),
                "source_generation":snapshot.context.source_generation.to_string(),"producer":snapshot.producer,
                "producer_version":snapshot.producer_version,"resources":snapshot.resources,
                "actor_kind":snapshot.actor_kind,"actor_subject":snapshot.actor_subject,"source_cell_bytes":snapshot.source_cell_bytes,
                "output_configured":snapshot.output_configured,
                "asset_operations":crate::production::assets::catalogue()});
            self.context = Some((token, snapshot.context));
            return Ok(result);
        }
        match name {
            "synth.package.save" | "synth_package_save" => {
                let a: SaveArg = args(value)?;
                let expected = self.expected(&a.context)?;
                let result = self
                    .host
                    .try_lock()
                    .map_err(|_| failure("busy", "Synth operation already running"))?
                    .save_candidate(&expected, &a.filename);
                self.context = None;
                let saved = result.map_err(native_error)?;
                Ok(
                    json!({"filename":saved.filename,"package_sha256":hex(&saved.digest),"bytes":saved.bytes.to_string(),
                    "audit_recorded":saved.audit_recorded,"durability_warning":saved.durability_warning.is_some(),
                    "review_required":true,"refresh_required":true}),
                )
            }
            "synth.package.present" | "synth_package_present" => {
                let a: PresentArg = args(value)?;
                let expected = self.expected(&a.context)?;
                let summary = self
                    .host
                    .try_lock()
                    .map_err(|_| failure("busy", "Synth operation already running"))?
                    .present_candidate(&expected, a.filename.as_deref())
                    .map_err(native_error)?;
                Ok(summary)
            }
            "synth.package.reopen" | "synth_package_reopen" => {
                let a: ReopenArg = args(value)?;
                let expected = self.expected(&a.context)?;
                let sha = crate::host::digest(&a.package_sha256)
                    .map_err(|_| failure("invalid_arguments", "Invalid package SHA-256"))?;
                self.cancel.store(false, Ordering::Release);
                let result = self
                    .host
                    .try_lock()
                    .map_err(|_| failure("busy", "Synth operation already running"))?
                    .reopen_candidate(&expected, &a.filename, sha, &self.cancel);
                self.context = None;
                result.map_err(native_error)?;
                Ok(json!({"reopened":true,"refresh_required":true}))
            }
            "synth.source.read" | "synth_source_read" => {
                let a: ReadArg = args(value)?;
                let expected = self.expected(&a.context)?;
                let text = self
                    .host
                    .try_lock()
                    .map_err(|_| failure("busy", "Synth operation already running"))?
                    .read_source(&expected, a.cell, a.offset, a.bytes)
                    .map_err(native_error)?;
                Ok(json!({"source":text,"offset":a.offset,"next_offset":a.offset+text.len()}))
            }
            "synth.source.submit" | "synth_source_submit" => {
                let a: SubmitArg = args(value)?;
                let expected = self.expected(&a.context)?;
                self.cancel.store(false, Ordering::Release);
                let mut host = self
                    .host
                    .try_lock()
                    .map_err(|_| failure("busy", "Synth operation already running"))?;
                let result = host.submit(&expected, &a.source, &self.cancel);
                let reconciliation = host.requires_reconciliation();
                drop(host);
                self.context = None;
                if result.is_err() && reconciliation {
                    return Err(failure("source_commit_requires_reconciliation",
                    "Source may have committed; native source/audit reconciliation is required. Do not retry the edit blindly"));
                }
                result.map_err(native_error)?;
                Ok(json!({"source_promoted":true,"refresh_required":true}))
            }
            _ => {
                if self.include_qualia_engine_tools {
                    if let Some(res) = crate::mcp_full::dispatch_qualia_engine_tool(name, &value) {
                        return res;
                    }
                }
                Err(failure(
                    "unknown_tool",
                    "Synth operation is not implemented or enabled",
                ))
            }
        }
    }
}

/// Native agent entry, using the existing bounded QualiaDB protocol implementation.
pub fn run_stdio(host: Arc<Mutex<SynthHost>>) -> Result<(), String> {
    let backend = SynthAgentBackend::bound(host)?;
    qualia_mcp::authoring::run_stdio(backend, Default::default())
        .map_err(|_| "Synth agent transport ended or refused a protocol frame".into())
}

/// Native agent entry with Python-identifier-safe tool names (`synth_inspect`, etc.)
/// for FastMCP / Antigravity Python SDK (`McpStdioServer`) clients.
pub fn run_stdio_identifier_safe(host: Arc<Mutex<SynthHost>>) -> Result<(), String> {
    let backend = SynthAgentBackend::bound(host)?.with_identifier_safe_tools();
    qualia_mcp::authoring::run_stdio(backend, Default::default())
        .map_err(|_| "Synth agent transport ended or refused a protocol frame".into())
}

/// Native agent entry exposing both authenticated Synth authoring tools and all 63
/// QualiaDB MCP engine tools (`qualia_mcp::stable_mcp_tools()`).
pub fn run_stdio_comprehensive(host: Arc<Mutex<SynthHost>>) -> Result<(), String> {
    let backend = SynthAgentBackend::bound(host)?
        .with_identifier_safe_tools()
        .with_qualia_engine_tools();
    qualia_mcp::authoring::run_stdio(backend, crate::mcp_full::comprehensive_protocol_limits())
        .map_err(|_| "Synth agent transport ended or refused a protocol frame".into())
}
