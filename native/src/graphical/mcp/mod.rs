//! MCP is a transport over the same authorised native editor, not another store.
use super::controller::{EditorHost, EditorOutcome, EditorSnapshot};
use super::cooperation::AuthorizedEditorHost;
use qualia_core_db::poet_host::invoke::studio_store::checkpoint::ExpectedFile;
use qualia_mcp::authoring::{AuthoringBackend, AuthoringTool, ToolError};
use serde::Deserialize;
use serde_json::{json, Value};
use std::hash::{BuildHasher, Hasher};
use std::sync::atomic::{AtomicU64, Ordering};
mod botanical;
mod material_motion;
mod resources;
mod texture_effects;

static NEXT_CONNECTION: AtomicU64 = AtomicU64::new(1);

/// Native embedding supplies a policy-wrapped host. Stdio arguments cannot
/// populate a principal, membership, project grant or native file destination.
pub struct NativeAuthoringBackend {
    host: Option<AuthorizedEditorHost>,
    connection: String,
    sequence: u64,
    context: Option<(String, EditorSnapshot)>,
}

impl NativeAuthoringBackend {
    pub fn unbound() -> Self {
        Self {
            host: None,
            connection: format!(
                "{}-{:016x}{:016x}-{}",
                std::process::id(),
                std::collections::hash_map::RandomState::new()
                    .build_hasher()
                    .finish(),
                std::collections::hash_map::RandomState::new()
                    .build_hasher()
                    .finish(),
                NEXT_CONNECTION.fetch_add(1, Ordering::Relaxed)
            ),
            sequence: 0,
            context: None,
        }
    }
    pub fn bound(host: AuthorizedEditorHost) -> Self {
        Self {
            host: Some(host),
            ..Self::unbound()
        }
    }
    fn host(&mut self) -> Result<&mut AuthorizedEditorHost, ToolError> {
        self.host.as_mut().ok_or_else(|| failure("authority_unavailable",
            "Native authenticated project admission is required; request fields cannot grant access"))
    }
    fn read(&mut self) -> Result<Value, ToolError> {
        let snapshot = self.host()?.snapshot().map_err(editor_error)?;
        let cells = self
            .host()?
            .source_cells(snapshot.source.clone())
            .map_err(editor_error)?;
        if !snapshot.publication_is_current() {
            return Err(failure(
                "stale_authority",
                "Native publication authority/source is no longer current",
            ));
        }
        self.sequence = self
            .sequence
            .checked_add(1)
            .ok_or_else(|| failure("limit", "Connection context exhausted"))?;
        // This token is a connection-local stale-context guard, never a bearer
        // credential. Every operation still runs current native authorisation.
        let token = format!("poet/{}/{}", self.connection, self.sequence);
        let response = json!({"context":token,"artifact_id":snapshot.artifact_id,
            "source_revision":snapshot.source_revision,"session_revision":snapshot.session_revision,
            "reconciliation_required":snapshot.reconciliation_required,
            "material_edit_id":snapshot.material_edit_id,"source_cells":cells});
        self.context = Some((token, snapshot));
        Ok(response)
    }
    fn expected(&mut self, token: &str) -> Result<EditorSnapshot, ToolError> {
        let (admitted, snapshot) = self.context.as_ref().ok_or_else(|| {
            failure(
                "context_required",
                "Read the current source before an operation",
            )
        })?;
        if token != admitted || !snapshot.execution_epoch.is_current() {
            return Err(failure(
                "stale_context",
                "Source context is stale; read again",
            ));
        }
        let expected = snapshot.clone();
        if !expected.publication_is_current() {
            return Err(failure(
                "stale_authority",
                "Native publication authority/source is no longer current",
            ));
        }
        expected.source.check_current().map_err(editor_error)?;
        let now = self.host()?.snapshot().map_err(editor_error)?;
        if now.source != expected.source
            || now.session_revision != expected.session_revision
            || now.execution_epoch != expected.execution_epoch
            || now.reconciliation_required != expected.reconciliation_required
        {
            return Err(failure(
                "stale_context",
                "Native source or execution changed; read again",
            ));
        }
        Ok(expected)
    }
}

fn failure(code: &str, message: &str) -> ToolError {
    ToolError {
        code: code.into(),
        message: message.into(),
    }
}
fn editor_error(_error: String) -> ToolError {
    // Native errors can contain source or filesystem details. Do not disclose
    // them to a denied client; callers can inspect local host diagnostics.
    failure(
        "native_operation_failed",
        "Native operation denied, stale, held or failed; read current state before retrying",
    )
}
fn schema(source: bool, context: bool) -> Value {
    let mut properties = serde_json::Map::new();
    let mut required = Vec::new();
    if source {
        properties.insert("source".into(), json!({"type":"string","maxLength":65536}));
        required.push("source");
    }
    if context {
        properties.insert("context".into(), json!({"type":"string","maxLength":128}));
        required.push("context");
    }
    json!({"type":"object","properties":properties,"required":required,"additionalProperties":false})
}
fn tool(name: &str, description: &str, input_schema: Value) -> AuthoringTool {
    AuthoringTool {
        name: name.into(),
        description: description.into(),
        input_schema,
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Edit {
    context: String,
    source: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Context {
    context: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Save {
    context: String,
    name: String,
    expected_sha256: Option<String>,
}
fn args<T: serde::de::DeserializeOwned>(value: Value) -> Result<T, ToolError> {
    serde_json::from_value(value).map_err(|_| {
        failure(
            "invalid_arguments",
            "Arguments do not match the declared tool schema",
        )
    })
}
fn digest(input: &str) -> Result<[u8; 32], ToolError> {
    if input.len() != 64 {
        return Err(failure(
            "invalid_arguments",
            "Expected SHA256 must contain 64 hexadecimal characters",
        ));
    }
    let mut result = [0; 32];
    for (slot, pair) in result.iter_mut().zip(input.as_bytes().chunks_exact(2)) {
        let value = std::str::from_utf8(pair)
            .ok()
            .and_then(|s| u8::from_str_radix(s, 16).ok())
            .ok_or_else(|| failure("invalid_arguments", "Expected SHA256 is invalid"))?;
        *slot = value;
    }
    Ok(result)
}
fn receipt_authority(
    stamp: &Option<super::cooperation::AuthorisationStamp>,
) -> Result<(), ToolError> {
    if stamp.as_ref().is_none_or(|stamp| !stamp.is_current()) {
        return Err(failure("stale_authority", "Native authority changed after the operation; effects may have committed; read before retrying"));
    }
    Ok(())
}
fn outcome(
    result: EditorOutcome,
    stamp: &Option<super::cooperation::AuthorisationStamp>,
) -> Result<Value, ToolError> {
    receipt_authority(stamp)?;
    if !result.snapshot.execution_epoch.is_current()
        || result.snapshot.source.check_current().is_err()
    {
        return Err(failure(
            "stale_context",
            "Native completion is stale; effects may have committed; read before retrying",
        ));
    }
    Ok(json!({"source_revision":result.snapshot.source_revision,
        "session_revision":result.snapshot.session_revision,
        "reconciliation_required":result.snapshot.reconciliation_required,
        "preview_rendered":result.frame.is_some(),"human_review":"unassessed",
        "message":"Native operation completed; read current authorised state for the next action",
        "read_required":true}))
}

impl AuthoringBackend for NativeAuthoringBackend {
    fn tools(&self) -> Vec<AuthoringTool> {
        let mut tools=vec![
            tool("poet_source_read", "Read authorised current source cells and obtain a connection context",schema(false,false)),
            tool("poet_source_check", "Check a draft against the persistent canonical Vibe session without execution",schema(true,true)),
            tool("poet_source_submit", "Submit source through native policy, canonical Vibe and the source journal",schema(true,true)),
            tool("poet_material_apply", "Submit an explicit material source draft through native material-edit policy",schema(true,true)),
            tool("poet_source_undo", "Undo accepted source; execution becomes held until explicit rebuild",schema(false,true)),
            tool("poet_source_redo", "Redo source; execution becomes held until explicit rebuild",schema(false,true)),
            tool("poet_execution_rebuild", "Explicitly rebuild execution under the unchanged read-only native geometry profile",schema(false,true)),
        ];
        if self
            .host
            .as_ref()
            .is_some_and(EditorHost::checkpoint_supported)
        {
            tools.extend(resources::tools(
                self.host
                    .as_ref()
                    .is_some_and(EditorHost::resource_import_supported),
            ));
            tools.push(tool("poet_hmc_save", "Save the authorised source and undo/redo journal to a host-selected checkpoint directory",
                json!({"type":"object","properties":{"context":{"type":"string","maxLength":128},
                    "name":{"type":"string","maxLength":128},"expected_sha256":{"type":"string","pattern":"^[0-9a-fA-F]{64}$"}},
                    "required":["context","name"],"additionalProperties":false})));
        }
        tools.extend(botanical::tools());
        tools.extend(material_motion::tools());
        tools.extend(texture_effects::tools());
        tools
    }
    fn call(&mut self, name: &str, arguments: Value) -> Result<Value, ToolError> {
        if matches!(name, "poet_botanical_inspect" | "poet_botanical_edit") {
            return botanical::call(self, name, arguments);
        }
        if matches!(
            name,
            "poet_texture_effects_inspect"
                | "poet_texture_effects_edit"
                | "poet_texture_layers_edit"
        ) {
            return texture_effects::call(self, name, arguments);
        }
        if matches!(
            name,
            "poet_material_motion_inspect" | "poet_material_motion_edit"
        ) {
            return material_motion::call(self, name, arguments);
        }
        if matches!(
            name,
            "poet_hmc_inventory"
                | "poet_hmc_import_resource"
                | "poet_hmc_remove_resource"
                | "poet_hmc_edit_resources"
        ) {
            return resources::call(self, name, arguments);
        }
        if name == "poet_source_read" {
            if arguments.as_object().is_none_or(|o| !o.is_empty()) {
                return Err(failure("invalid_arguments", "Read takes no arguments"));
            }
            return self.read();
        }
        if matches!(
            name,
            "poet_source_check" | "poet_source_submit" | "poet_material_apply"
        ) {
            let edit: Edit = args(arguments)?;
            if edit.source.len() > 65536 || edit.context.len() > 128 {
                return Err(failure(
                    "limit",
                    "Source/context exceeds native admission budget",
                ));
            }
            let expected = self.expected(&edit.context)?;
            if name == "poet_source_check" {
                self.host()?.check(&edit.source).map_err(editor_error)?;
                receipt_authority(&expected.authorisation_stamp)?;
                return Ok(
                    json!({"checked":true,"executed":false,"message":"Canonical check completed; no execution attempted"}),
                );
            }
            // Consume the connection context on every attempted mutation,
            // including a failure whose external effects may need reconciliation.
            self.context = None;
            let authority = expected.authorisation_stamp.clone();
            let result = if name == "poet_material_apply" {
                self.host()?.material_edit(&edit.source, expected.source)
            } else {
                self.host()?.submit(&edit.source, expected.source)
            };
            return result
                .map_err(editor_error)
                .and_then(|result| outcome(result, &authority));
        }
        if name == "poet_hmc_save" {
            let save: Save = args(arguments)?;
            if save.name.len() > 128 || save.context.len() > 128 {
                return Err(failure("limit", "Checkpoint name/context exceeds budget"));
            }
            let expected = self.expected(&save.context)?;
            let file = save
                .expected_sha256
                .as_deref()
                .map(digest)
                .transpose()?
                .map_or(ExpectedFile::Absent, ExpectedFile::Sha256);
            self.context = None;
            let authority = expected.authorisation_stamp.clone();
            let receipt = self
                .host()?
                .save_checkpoint(&save.name, file, expected.source)
                .map_err(editor_error)?;
            receipt_authority(&authority)?;
            let hash = receipt
                .digest
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect::<String>();
            return Ok(
                json!({"name":save.name,"sha256":hash,"bytes":receipt.bytes,"durability_warning":receipt.durability_warning,"read_required":true}),
            );
        }
        if matches!(
            name,
            "poet_source_undo" | "poet_source_redo" | "poet_execution_rebuild"
        ) {
            let request: Context = args(arguments)?;
            if request.context.len() > 128 {
                return Err(failure("limit", "Context exceeds budget"));
            }
            let expected = self.expected(&request.context)?;
            self.context = None;
            let authority = expected.authorisation_stamp.clone();
            if name == "poet_execution_rebuild" {
                return self
                    .host()?
                    .rebuild_execution(expected.source)
                    .map_err(editor_error)
                    .and_then(|result| outcome(result, &authority));
            }
            let result = if name == "poet_source_undo" {
                self.host()?.undo(expected.source)
            } else {
                self.host()?.redo(expected.source)
            };
            let snapshot = result.map_err(editor_error)?;
            receipt_authority(&authority)?;
            return Ok(
                json!({"source_revision":snapshot.source_revision,"reconciliation_required":snapshot.reconciliation_required,"read_required":true}),
            );
        }
        Err(failure(
            "unknown_tool",
            "Tool is not supported by native Poet authoring",
        ))
    }
}

/// Stdio entry without a native owner never creates a project/principal or
/// treats a requested grant as authenticated cooperative admission.
pub fn run_unbound_stdio() -> Result<(), String> {
    qualia_mcp::authoring::run_stdio(NativeAuthoringBackend::unbound(), Default::default())
        .map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unbound_transport_does_not_admit_a_principal_or_checkpoint_path() {
        let mut backend = NativeAuthoringBackend::unbound();
        assert!(!backend.tools().iter().any(|t| t.name == "poet_hmc_save"));
        assert_eq!(
            backend
                .call("poet_source_read", json!({}))
                .unwrap_err()
                .code,
            "authority_unavailable"
        );
        assert_eq!(
            backend
                .call("poet_source_read", json!({"principal":"owner"}))
                .unwrap_err()
                .code,
            "invalid_arguments"
        );
        assert!(backend
            .call(
                "poet_source_submit",
                json!({"context":"made-up","source":"1;"})
            )
            .is_err());
    }
    #[test]
    fn overwrite_digest_is_exact_and_bounded() {
        // A process-local counter alone repeats across restarts. Randomised
        // connection labels prevent old context reuse; they grant no authority.
        assert_ne!(
            NativeAuthoringBackend::unbound().connection,
            NativeAuthoringBackend::unbound().connection
        );
        assert_eq!(digest(&"ab".repeat(32)).unwrap(), [0xab; 32]);
        assert!(digest("ab").is_err());
        assert!(digest(&"zz".repeat(32)).is_err());
    }
}
