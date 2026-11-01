//! Existing QualiaDB MCP transport, over the SAME guarded native operations.
//! JSON here is protocol framing, never canonical species/project storage.
use crate::{NativeContext, SynthHost, TransportCommand};
use ecopunk_terrain::species_playback::{time::Tick, PlaybackRate};
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
        })
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
struct ContextArg {
    context: String,
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
struct ReopenArg {
    context: String,
    filename: String,
    package_sha256: String,
}
fn hex(bytes: &[u8; 32]) -> String {
    use std::fmt::Write;
    let mut text = String::with_capacity(64);
    for byte in bytes {
        let _ = write!(&mut text, "{byte:02x}");
    }
    text
}
#[derive(Deserialize)]
#[serde(tag = "operation", rename_all = "snake_case", deny_unknown_fields)]
enum Control {
    Play,
    Pause,
    SelectClip { clip: String },
    Seek { tick: String },
    Rate { numerator: u32, denominator: u32 },
    Loop { enabled: bool },
}
impl AuthoringBackend for SynthAgentBackend {
    fn tools(&self) -> Vec<AuthoringTool> {
        let object = |props: Value, required: Vec<&str>| json!({"type":"object","properties":props,"required":required,"additionalProperties":false});
        let token = json!({"type":"string","maxLength":128});
        let context_schema = object(json!({"context":token}), vec!["context"]);
        let mut tools=vec![
            AuthoringTool {name:"synth.inspect".into(),description:"Inspect the admitted Synth source and obtain a fresh guarded operation context".into(),input_schema:object(json!({}),vec![])},
            AuthoringTool {name:"synth.source.read".into(),description:"Read an authorised Vibe source cell in bounded UTF-8 chunks".into(),input_schema:object(json!({"context":token,"cell":{"type":"integer","minimum":0},"offset":{"type":"integer","minimum":0},"bytes":{"type":"integer","minimum":1,"maximum":16384}}),vec!["context","cell","offset","bytes"])},
            AuthoringTool {name:"synth.source.submit".into(),description:"Prepare a data-only species Vibe edit, then promote under the same native source/actor authority".into(),input_schema:object(json!({"context":token,"source":{"type":"string","maxLength":1048576}}),vec!["context","source"])},
            AuthoringTool {name:"synth.playback.control".into(),description:"Control explicitly authored clips; coupled audio accepts normal speed and exact sample-aligned seeks".into(),input_schema:object(json!({"context":token,"operation":{"enum":["play","pause","select_clip","seek","rate","loop"]},"clip":{"type":"string"},"tick":{"type":"string","pattern":"^[0-9]+$"},"numerator":{"type":"integer"},"denominator":{"type":"integer"},"enabled":{"type":"boolean"}}),vec!["context","operation"])},
            AuthoringTool {name:"synth.audio.status".into(),description:"Read native output status; callback activity is not proof of acoustic presentation".into(),input_schema:context_schema.clone()},
            AuthoringTool {name:"synth.package.save".into(),description:"Create a source/Q42/10D candidate HMC in the native-configured output directory; review remains required".into(),input_schema:object(json!({"context":token,"filename":{"type":"string","maxLength":96}}),vec!["context","filename"])},
            AuthoringTool {name:"synth.package.reopen".into(),description:"Fresh replay of the same admitted source from a saved candidate's exact package digest".into(),input_schema:object(json!({"context":token,"filename":{"type":"string","maxLength":96},"package_sha256":{"type":"string","pattern":"^[0-9a-f]{64}$"}}),vec!["context","filename","package_sha256"])},
        ];
        #[cfg(feature = "audio-device")]
        for (name, description) in [
            (
                "synth.audio.play",
                "Start a native-owned, prefilled sound job",
            ),
            (
                "synth.audio.stop",
                "Stop native output and require acknowledged device reset",
            ),
        ] {
            tools.push(AuthoringTool {
                name: name.into(),
                description: description.into(),
                input_schema: context_schema.clone(),
            });
        }
        tools
    }
    fn call(&mut self, name: &str, value: Value) -> Result<Value, ToolError> {
        if name == "synth.inspect" {
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
                "source_generation":snapshot.context.source_generation.to_string(),"clips":snapshot.clips,
                "selected_clip":snapshot.selected_clip,"playing":snapshot.playing,
                "planned_source_tick":snapshot.position.map(|t|t.0.to_string()),"audio_status":snapshot.audio_status,
                "actor_kind":snapshot.actor_kind,"actor_subject":snapshot.actor_subject,"source_cell_bytes":snapshot.source_cell_bytes,
                "output_configured":snapshot.output_configured});
            self.context = Some((token, snapshot.context));
            return Ok(result);
        }
        match name {
            "synth.package.save" => {
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
                    "review_required":true,"package_complete":false,"refresh_required":true}),
                )
            }
            "synth.package.reopen" => {
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
                Ok(json!({"reopened":true,"paused":true,"refresh_required":true}))
            }
            "synth.source.read" => {
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
            "synth.source.submit" => {
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
            "synth.playback.control" => {
                let mut fields = value
                    .as_object()
                    .cloned()
                    .ok_or_else(|| failure("invalid_arguments", "Expected a control object"))?;
                let token = fields
                    .remove("context")
                    .and_then(|v| v.as_str().map(str::to_owned))
                    .ok_or_else(|| failure("invalid_arguments", "Missing context"))?;
                let expected = self.expected(&token)?;
                let command = match args::<Control>(Value::Object(fields))? {
                    Control::Play => TransportCommand::Play,
                    Control::Pause => TransportCommand::Pause,
                    Control::SelectClip { clip } => TransportCommand::SelectClip(clip),
                    Control::Seek { tick } => TransportCommand::Seek(Tick(
                        tick.parse()
                            .map_err(|_| failure("invalid_arguments", "Invalid tick"))?,
                    )),
                    Control::Rate {
                        numerator,
                        denominator,
                    } => TransportCommand::Rate(PlaybackRate {
                        numerator,
                        denominator,
                    }),
                    Control::Loop { enabled } => TransportCommand::Loop(enabled),
                };
                let result = self
                    .host
                    .try_lock()
                    .map_err(|_| failure("busy", "Synth operation already running"))?
                    .control(&expected, command);
                self.context = None;
                result.map_err(native_error)?;
                Ok(json!({"accepted":true,"refresh_required":true}))
            }
            "synth.audio.status" => {
                let a: ContextArg = args(value)?;
                let expected = self.expected(&a.context)?;
                let host = self
                    .host
                    .try_lock()
                    .map_err(|_| failure("busy", "Synth operation already running"))?;
                let snapshot = host.inspect().map_err(native_error)?;
                if snapshot.context != expected {
                    return Err(failure("stale_context", "Inspect Synth again"));
                }
                Ok(json!({"status":snapshot.audio_status}))
            }
            #[cfg(feature = "audio-device")]
            "synth.audio.play" | "synth.audio.stop" => {
                let a: ContextArg = args(value)?;
                let expected = self.expected(&a.context)?;
                let host = self
                    .host
                    .try_lock()
                    .map_err(|_| failure("busy", "Synth operation already running"))?;
                if name.ends_with(".play") {
                    host.play_audio(&expected)
                } else {
                    host.stop_audio(&expected)
                }
                .map_err(native_error)?;
                Ok(json!({"acknowledged":true}))
            }
            _ => Err(failure(
                "unknown_tool",
                "Synth operation is not implemented or enabled",
            )),
        }
    }
}

/// Native agent entry, using the existing bounded QualiaDB protocol implementation.
pub fn run_stdio(host: Arc<Mutex<SynthHost>>) -> Result<(), String> {
    let backend = SynthAgentBackend::bound(host)?;
    qualia_mcp::authoring::run_stdio(backend, Default::default())
        .map_err(|_| "Synth agent transport ended or refused a protocol frame".into())
}
