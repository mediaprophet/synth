//! Native embedding for authenticated Poet projects. No CLI/MCP field can mint a session.
use super::{
    checkpoints::{
        CheckpointEditorHost, CheckpointLimits, CheckpointService, PackageWritePolicy,
        TrustedBinding,
    },
    controller::{CanonicalEditorHost, EditorHost, PreviewProvider, RebuildLimits},
    cooperation::{native_owner::NativeAuthorityOwner, AuthorizedEditorHost, GatewayLimits, Scope},
    geometry_host::GeometryHost,
    package::{ResourceImportHost, ResourceLibrary},
    shared_host::SharedEditorHost,
};
use qualia_cooperative_core::authoring::{authorize, AuthoringAction, AuthoringRequest};
use qualia_core_db::poet_host::native_authority::{
    DocumentTransaction, NativeAuthority, SessionHandle,
};
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};
/// Host projection of the current opaque native session, never a principal roster.
pub type SessionSlot = Arc<Mutex<Option<SessionHandle>>>;
#[path = "cooperation/selected_owner.rs"]
mod selected_owner;

use vibe::session::SessionLimits;

/// All paths, execution grants and resource budgets are selected by the native
/// host after project admission, never populated from source or MCP requests.
pub struct ProjectConfig {
    pub scope: Scope,
    pub checkpoint_root: PathBuf,
    pub checkpoint_limits: CheckpointLimits,
    pub write_policy: PackageWritePolicy,
    pub import_library: Option<ResourceLibrary>,
    pub session_limits: SessionLimits,
    pub gateway_limits: GatewayLimits,
    pub geometry_grants: Vec<String>,
    pub max_vertices: usize,
}

pub struct NativeProject {
    shared: SharedEditorHost,
    authority: Arc<NativeAuthority>,
    scope: Scope,
    limits: GatewayLimits,
}
impl NativeProject {
    pub(crate) fn package_workflow_service(
        &self,
        sessions: SessionSlot,
    ) -> super::package_workflow::PackageWorkflowService {
        super::package_workflow::PackageWorkflowService::new(
            self.authority.clone(),
            sessions,
            self.shared.clone(),
            self.scope.clone(),
            self.limits,
        )
    }
    pub(crate) fn verify_package_target(
        &self,
        target: &qualia_core_db::poet_host::native_authority::package_review::PackageRevision,
    ) -> Result<(), String> {
        let guard = self.shared.package_review_guard()?;
        let source = guard.snapshot()?.source;
        super::package_workflow::verify_package_guard(
            guard.as_ref(),
            &self.scope,
            self.limits,
            target,
            &source,
        )
    }

    pub(crate) fn workflow_service(
        &self,
        sessions: SessionSlot,
    ) -> super::cooperative_workflow::WorkflowService {
        super::cooperative_workflow::WorkflowService::new(
            self.authority.clone(),
            sessions,
            self.shared.clone(),
            self.scope.clone(),
            self.limits,
        )
    }
    /// Bind an already admitted source or freshly reopen a verified HMC under
    /// the current native owner. Existing documents are never replaced.
    /// Reopened execution stays held until explicit authorised reconstruction.
    pub fn open(
        config: ProjectConfig,
        authority: Arc<NativeAuthority>,
        session: SessionHandle,
        reopen: Option<&str>,
        preview: Box<dyn PreviewProvider>,
    ) -> Result<Self, String> {
        Self::open_with_rebuild_limits(
            config,
            authority,
            session,
            reopen,
            preview,
            RebuildLimits::default(),
        )
    }
    /// Rebuild work is chosen by the admitted native host, never restored from HMC.
    pub fn open_with_rebuild_limits(
        config: ProjectConfig,
        authority: Arc<NativeAuthority>,
        session: SessionHandle,
        reopen: Option<&str>,
        preview: Box<dyn PreviewProvider>,
        rebuild_limits: RebuildLimits,
    ) -> Result<Self, String> {
        if config.max_vertices == 0 {
            return Err("Native geometry working budget must be nonzero".into());
        }
        let binding = TrustedBinding {
            project_id: config.scope.project_id.clone(),
            document_id: config.scope.document_id.clone(),
            source_id: config.scope.source_id.clone(),
            domain: config.scope.domain.clone(),
        };
        let service = CheckpointService::new_with_policy(
            &config.checkpoint_root,
            binding,
            config.checkpoint_limits,
            config.write_policy,
        )?;
        if let Some(name) = reopen {
            let prepared = authority.transact(&session, |view| {
                let result = (|| {
                    let document = view.current_document;
                    if document.project_id != config.scope.project_id
                        || document.document_id != config.scope.document_id
                        || document.source_id != config.scope.source_id
                    {
                        return Err(
                            "Native reopen scope differs from admitted project/document/source"
                                .into(),
                        );
                    }
                    authorize(
                        view,
                        &AuthoringRequest {
                            action: AuthoringAction::ReopenCheckpoint,
                            target: document,
                            expected_authority_generation: view.generation,
                        },
                    )
                    .map_err(|e| e.to_string())?;
                    service.prepare_reopen(name)
                })();
                DocumentTransaction {
                    result,
                    successor: None,
                    reconciliation_required: false,
                }
            })?;
            super::cooperation::native_owner::reopen_verified(
                &authority,
                &session,
                &config.scope,
                &service,
                prepared,
                config.gateway_limits,
            )?;
        }
        // Check real key/session/admission and the exact canonical source before
        // constructing interpreter/renderer resources. No evaluation occurs here.
        let owner = Arc::new(NativeAuthorityOwner::new(
            authority.clone(),
            session.clone(),
            config.scope.clone(),
            config.gateway_limits,
        )?);
        let raw = CanonicalEditorHost::open_with_rebuild_limits(
            config.scope.source_id.clone(),
            config.session_limits,
            GeometryHost::new(config.geometry_grants, config.max_vertices),
            preview,
            rebuild_limits,
        )?;
        let checkpoint = CheckpointEditorHost::new(Box::new(raw), service)?;
        let inner: Box<dyn EditorHost> = match config.import_library {
            Some(library) => Box::new(ResourceImportHost::new(Box::new(checkpoint), library)),
            None => Box::new(checkpoint),
        };
        let project = Self {
            shared: SharedEditorHost::new(inner),
            authority,
            scope: config.scope,
            limits: config.gateway_limits,
        };
        // A native session is still revalidated at every subsequent operation.
        AuthorizedEditorHost::new(
            Box::new(project.shared.clone()),
            project.scope.clone(),
            owner,
            project.limits,
        )?
        .snapshot()?;
        Ok(project)
    }
    pub fn connect(&self, session: SessionHandle) -> Result<AuthorizedEditorHost, String> {
        let owner = Arc::new(NativeAuthorityOwner::new(
            self.authority.clone(),
            session,
            self.scope.clone(),
            self.limits,
        )?);
        let host = AuthorizedEditorHost::new(
            Box::new(self.shared.clone()),
            self.scope.clone(),
            owner,
            self.limits,
        )?;
        host.snapshot()?;
        Ok(host)
    }
    /// A UI selection is an additional publication/operation constraint. It
    /// never revokes other admitted sessions or alters direct connections.
    pub(crate) fn connect_selected(
        &self,
        session: SessionHandle,
        selection: SessionSlot,
    ) -> Result<AuthorizedEditorHost, String> {
        let native = Arc::new(NativeAuthorityOwner::new(
            self.authority.clone(), session.clone(), self.scope.clone(), self.limits,
        )?);
        let owner = Arc::new(selected_owner::SelectedAuthorityOwner::new(native, selection, session));
        let host = AuthorizedEditorHost::new(
            Box::new(self.shared.clone()), self.scope.clone(), owner, self.limits,
        )?;
        host.snapshot()?;
        Ok(host)
    }
    pub fn mcp_backend(
        &self,
        session: SessionHandle,
    ) -> Result<super::mcp::NativeAuthoringBackend, String> {
        self.connect(session)
            .map(super::mcp::NativeAuthoringBackend::bound)
    }
    /// Main-thread native Aether projection, using the same source/session as MCP.
    pub fn launch(&self, session: SessionHandle) -> Result<(), String> {
        let host = self.connect(session)?;
        let draft = host.snapshot()?.latest_source;
        super::launch(Box::new(host), draft)
    }
}

#[cfg(test)]
#[path = "native_project_tests.rs"]
mod tests;
