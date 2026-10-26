//! Existing Poet payloads enter the same authorized native worker and source context.
use super::core::{
    ActionType, ContextRef, IntentBus, IntentReceipt, IntentStatus, Provenance, TargetKind,
    VibeScriptPayload,
};
use crate::graphical::{
    authoring::AuthoringEdit,
    cooperative_workflow::WorkflowAction,
    jobs::{self, Action, Context, HostHandle, Reply},
};
use qualia_core_db::poet_host::invoke::studio_store::checkpoint::ExpectedFile;
use serde::{Deserialize, Serialize};
use std::{
    io::Write,
    sync::atomic::{AtomicU64, Ordering},
};
const MAX_ENVELOPE_BYTES: usize = 96 * 1024;
static NEXT_DISPATCH: AtomicU64 = AtomicU64::new(1);
#[derive(Clone)]
pub struct NativeIntentBus {
    host: HostHandle,
    origin: Context,
}
pub struct NativeDispatch {
    pub receipt: IntentReceipt,
    pub reply: Option<Reply>,
}
#[path = "native_commands.rs"]
mod commands;
use commands::Command;
struct Bounded {
    bytes: Vec<u8>,
}
impl Write for Bounded {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        if self
            .bytes
            .len()
            .checked_add(bytes.len())
            .is_none_or(|n| n > MAX_ENVELOPE_BYTES)
        {
            return Err(std::io::Error::other(
                "Poet intent exceeds bounded envelope",
            ));
        }
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
fn rejection(
    id: u64,
    reason: impl Into<String>,
    provenance: Option<Provenance>,
    reply: Option<Reply>,
) -> NativeDispatch {
    NativeDispatch {
        receipt: IntentReceipt {
            dispatch_id: id,
            status: IntentStatus::Rejected(reason.into()),
            provenance,
        },
        reply,
    }
}
impl NativeIntentBus {
    pub fn new(host: HostHandle, origin: Context) -> Result<Self, String> {
        if origin.source.artifact_id.is_empty() || origin.source.artifact_id.len() > 4096 {
            return Err("Poet intent bus requires an actual bounded source component".into());
        }
        Ok(Self { host, origin })
    }
    fn current_context(&self) -> Result<(), String> {
        self.origin.source.check_current()?;
        if !self.origin.execution_epoch.is_current()
            || self
                .origin
                .authorisation_stamp
                .as_ref()
                .is_none_or(|s| !s.is_current())
        {
            return Err(
                "Poet intent requires the same current authenticated native source/session context"
                    .into(),
            );
        }
        Ok(())
    }
    async fn inspect_actor(&self, context: &Context) -> Result<String, String> {
        let inspection = jobs::run(
            self.host.clone(),
            Action::Workflow {
                action: WorkflowAction::Inspect,
                expected: context.source.clone(),
            },
            context.clone(),
        )
        .await?;
        let subject = match (&inspection.workflow, &inspection.snapshot) {
            (Some(view), Some(snapshot))
                if inspection.operation_succeeded
                    && inspection.current
                    && Context::from_snapshot(snapshot) == *context
                    && view.source == context.source
                    && view.evidence.target.source_id == context.source.artifact_id
                    && !view.evidence.principal_subject.is_empty()
                    && view.evidence.principal_subject.len() <= 4096 =>
            {
                view.evidence.principal_subject.clone()
            }
            _ => {
                return Err("Authenticated Poet actor inspection failed or context changed".into())
            }
        };
        context.source.check_current()?;
        if !context.execution_epoch.is_current()
            || context
                .authorisation_stamp
                .as_ref()
                .is_none_or(|s| !s.is_current())
        {
            return Err("Poet actor/source context changed during inspection".into());
        }
        Ok(subject)
    }
    fn provenance(id: u64, subject: String, scope: &str) -> Provenance {
        // Preserve the legacy field spelling without inventing a DID.
        Provenance {
            emitter_did: subject,
            component_label: "Poet native Studio".into(),
            intent_counter: id,
            capability_scope: Some(scope.into()),
        }
    }
    async fn refresh(&self, id: u64, scope: &str) -> Result<NativeDispatch, String> {
        let Some(old_actor) = &self.origin.authorisation_stamp else {
            return Ok(rejection(
                id,
                "Poet refresh requires an authenticated native actor",
                None,
                None,
            ));
        };
        if !old_actor.is_current() {
            return Ok(rejection(
                id,
                "Poet refresh actor changed or expired",
                None,
                None,
            ));
        }
        // Read-only authorized discovery retains the original refresh semantics.
        // It may refresh source/epoch, never replace the admitted actor.
        let reply = jobs::run(self.host.clone(), Action::Refresh, self.origin.clone()).await?;
        let current = match &reply.snapshot {
            Some(snapshot)
                if reply.operation_succeeded
                    && reply.current
                    && snapshot.source.artifact_id == self.origin.source.artifact_id
                    && snapshot.authorisation_stamp.as_ref() == Some(old_actor)
                    && old_actor.is_current()
                    && snapshot
                        .authorisation_stamp
                        .as_ref()
                        .is_some_and(|s| s.is_current()) =>
            {
                Context::from_snapshot(snapshot)
            }
            _ => {
                return Ok(rejection(
                    id,
                    "Poet refresh could not preserve the same opaque actor",
                    None,
                    None,
                ))
            }
        };
        let subject = match self.inspect_actor(&current).await {
            Ok(subject) => subject,
            Err(e) => return Ok(rejection(id, e, None, None)),
        };
        if !old_actor.is_current() || current.source.check_current().is_err() {
            return Ok(rejection(
                id,
                "Poet refresh changed during actor inspection",
                None,
                None,
            ));
        }
        Ok(NativeDispatch {
            receipt: IntentReceipt {
                dispatch_id: id,
                status: IntentStatus::Accepted,
                provenance: Some(Self::provenance(id, subject, scope)),
            },
            reply: Some(reply),
        })
    }
    fn envelope<P: Serialize>(&self, payload: VibeScriptPayload<P>) -> Result<Command, String> {
        if payload.provenance.is_some() {
            return Err("Client-supplied Poet provenance cannot establish an actor".into());
        }
        if !matches!(&payload.context,ContextRef::Iri(iri)if iri=="https://qualiadb.org/schema/vibe#")
        {
            return Err(
                "Poet intent context differs from the admitted original schema context".into(),
            );
        }
        if payload.target_identifier.kind != TargetKind::ComponentId
            || payload.target_identifier.value != self.origin.source.artifact_id
        {
            return Err("Poet intent targets a foreign component or identifier kind".into());
        }
        let mut output = Bounded { bytes: Vec::new() };
        serde_json::to_writer(&mut output, &payload).map_err(|e| e.to_string())?;
        let payload: VibeScriptPayload<serde_json::Value> =
            serde_json::from_slice(&output.bytes).map_err(|e| e.to_string())?;
        let command: Command = serde_json::from_value(payload.parameters)
            .map_err(|e| format!("Poet native command refused: {e}"))?;
        if payload.action_type != command.action_type() {
            return Err("Poet action classification differs from the native command".into());
        }
        Ok(command)
    }
    pub async fn dispatch_with_reply<P>(
        &self,
        payload: VibeScriptPayload<P>,
    ) -> Result<NativeDispatch, String>
    where
        P: Serialize + Send + Sync,
    {
        let id = NEXT_DISPATCH
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |n| n.checked_add(1))
            .map_err(|_| "Poet dispatch counter exhausted")?;
        let command = match self.envelope(payload) {
            Ok(command) => command,
            Err(e) => return Ok(rejection(id, e, None, None)),
        };
        let scope = command.scope();
        let action = match command.action(&self.origin) {
            Ok(action) => action,
            Err(e) => return Ok(rejection(id, e, None, None)),
        };
        if matches!(&action, Action::Refresh) {
            return self.refresh(id, scope).await;
        }
        if let Err(e) = self.current_context() {
            return Ok(rejection(id, e, None, None));
        }
        let subject = match self.inspect_actor(&self.origin).await {
            Ok(subject) => subject,
            Err(e) => return Ok(rejection(id, e, None, None)),
        };
        if let Err(e) = self.current_context() {
            return Ok(rejection(id, e, None, None));
        }
        // The legacy field name is preserved. This value is the qualified
        // admitted subject, not a generated DID or a current_user fallback.
        let provenance = Self::provenance(id, subject, scope);
        let reply = jobs::run(self.host.clone(), action, self.origin.clone()).await?;
        if !reply.current {
            return Ok(rejection(id, reply.message.clone(), Some(provenance), None));
        }
        if !reply.operation_succeeded {
            return Ok(rejection(
                id,
                reply.message.clone(),
                Some(provenance),
                Some(reply),
            ));
        }
        Ok(NativeDispatch {
            receipt: IntentReceipt {
                dispatch_id: id,
                status: IntentStatus::Accepted,
                provenance: Some(provenance),
            },
            reply: Some(reply),
        })
    }
}
impl IntentBus for NativeIntentBus {
    type Error = String;
    async fn dispatch<P>(&self, payload: VibeScriptPayload<P>) -> Result<IntentReceipt, Self::Error>
    where
        P: Serialize + Send + Sync,
    {
        self.dispatch_with_reply(payload).await.map(|d| d.receipt)
    }
    async fn cancel(&self, dispatch_id: u64) -> Result<IntentReceipt, Self::Error> {
        Ok(IntentReceipt {
            dispatch_id,
            status: IntentStatus::Rejected(
                "Native cancellation is unsupported; no command was cancelled".into(),
            ),
            provenance: None,
        })
    }
}
