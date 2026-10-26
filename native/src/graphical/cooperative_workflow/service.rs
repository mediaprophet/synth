use super::{WorkflowAction, WorkflowView};
use crate::graphical::{
    controller::EditorHost,
    cooperation::{source_digest, GatewayLimits, Scope},
    native_project::SessionSlot,
    shared_host::SharedEditorHost,
};
use qualia_cooperative_core::authoring::{
    authorize, AuthoringAction, AuthoringRequest, DocumentRevision,
};
use qualia_core_db::poet_host::{
    invoke::studio_store,
    native_authority::{DocumentTransaction, NativeAuthority, SessionHandle},
};
use std::sync::Arc;

/// Uses the native shell's one opaque session slot and the one raw shared
/// editor. No local principal cache, policy, membership roster or second session.
pub struct WorkflowService {
    authority: Arc<NativeAuthority>,
    sessions: SessionSlot,
    shared: SharedEditorHost,
    scope: Scope,
    limits: GatewayLimits,
}

impl WorkflowService {
    pub fn new(
        authority: Arc<NativeAuthority>,
        sessions: SessionSlot,
        shared: SharedEditorHost,
        scope: Scope,
        limits: GatewayLimits,
    ) -> Self {
        Self {
            authority,
            sessions,
            shared,
            scope,
            limits,
        }
    }
    pub fn run(&self, action: WorkflowAction) -> Result<WorkflowView, String> {
        action.validate_bounds()?;
        // A selected actor cannot change between source inspection and durable
        // contribution/review commit. Independent project sessions coexist.
        // Match the selection -> authority -> raw editor lock order.
        let selection = self
            .sessions
            .try_lock()
            .map_err(|_| "Native workflow actor selection busy or poisoned")?;
        let session = selection
            .as_ref()
            .ok_or("Sign in before cooperative work")?
            .clone();
        // This first inspection binds user intent to the actual source. The
        // record API repeats exact-target/current-session admission at commit.
        let before = self.inspect(&session)?;
        let target = match &action {
            WorkflowAction::Inspect => None,
            WorkflowAction::Contribution { target, .. }
            | WorkflowAction::RequestReview { target, .. }
            | WorkflowAction::Review { target, .. } => Some(target),
        };
        if target.is_some_and(|target| target != &before.evidence.target) {
            return Err(
                "Source changed since workflow inspection; refresh; draft preserved".into(),
            );
        }
        match action {
            WorkflowAction::Inspect => return Ok(before),
            WorkflowAction::Contribution {
                target,
                id,
                description,
                minutes,
                work_status,
            } => {
                self.authority.record_contribution_at(
                    &session,
                    &target,
                    &id,
                    &description,
                    minutes,
                    work_status,
                )?;
            }
            WorkflowAction::RequestReview {
                target,
                id,
                evidence,
            } => {
                self.authority
                    .request_source_review(&session, &target, &id, &evidence)?;
            }
            WorkflowAction::Review {
                target,
                request_id,
                id,
                decision,
                rationale,
                evidence,
            } => {
                self.authority.record_requested_source_review(
                    &session,
                    &target,
                    &request_id,
                    &id,
                    decision,
                    &rationale,
                    &evidence,
                )?;
            }
        }
        self.inspect(&session).map_err(|error| {
            format!("Workflow record may have committed; refresh evidence before retrying: {error}")
        })
    }
    fn inspect(&self, session: &SessionHandle) -> Result<WorkflowView, String> {
        let source = self.authority.transact(session, |view| {
            let result = (|| {
                authorize(view, &AuthoringRequest { action: AuthoringAction::ReadSource,
                    target: view.current_document, expected_authority_generation: view.generation })
                    .map_err(|e| e.to_string())?;
                let snapshot = self.shared.snapshot()?;
                let (artifact, stamp) = studio_store::get_artifact_stamped(&self.scope.source_id)
                    .ok_or("admitted workflow source is absent")?;
                let current = DocumentRevision {
                    project_id: self.scope.project_id.clone(), document_id: self.scope.document_id.clone(),
                    source_id: artifact.id.clone(), revision: artifact.lamport_clock.into(),
                    source_digest: source_digest(&artifact, self.limits)?,
                    author_subject: view.current_document.author_subject.clone(),
                };
                if &current != view.current_document || artifact.domain != self.scope.domain
                    || snapshot.source.stamp != stamp || snapshot.source.artifact_id != artifact.id {
                    return Err("Workflow source differs from current native admission; reconciliation required".into());
                }
                let cells = self.shared.source_cells(snapshot.source.clone())?;
                let previous = studio_store::previous_source_snapshot(&artifact.id, &stamp)?
                    .map(|source| serde_json::to_string_pretty(&source).map_err(|e| e.to_string())).transpose()?;
                let source = studio_store::EditableSourceSnapshot { title: artifact.title.clone(),
                    nodes: artifact.nodes.clone(), properties: artifact.properties.clone() };
                let current_json = serde_json::to_string_pretty(&source).map_err(|e| e.to_string())?;
                if current_json.len() > 8*1024*1024 || previous.as_ref().is_some_and(|source| source.len() > 8*1024*1024) {
                    return Err("Source comparison exceeds native inspection byte budget".into());
                }
                snapshot.source.check_current()?;
                Ok((current, view.generation, snapshot.source, cells, artifact.history, previous, current_json))
            })();
            DocumentTransaction { result, successor: None, reconciliation_required: false }
        })?;
        let evidence = self.authority.workflow_evidence(session)?;
        if evidence.target != source.0
            || evidence.authority_generation != source.1
            || !self
                .authority
                .publication_is_current(session, evidence.authority_generation)
        {
            return Err(
                "Native source/session changed during workflow inspection; result withheld".into(),
            );
        }
        source.2.check_current()?;
        Ok(WorkflowView {
            evidence,
            source: source.2,
            source_cells: source.3,
            source_history: source.4,
            previous_source_json: source.5,
            current_source_json: source.6,
        })
    }
}
