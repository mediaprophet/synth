use super::*;
use crate::graphical::{
    controller::EditorHost,
    cooperation::{source_digest, GatewayLimits, Scope},
    native_project::SessionSlot,
    shared_host::SharedEditorHost,
};
use qualia_core_db::poet_host::{
    invoke::studio_store,
    native_authority::{NativeAuthority, SessionHandle},
};
use std::sync::Arc;

/// Raw-owner verification only. Never call an AuthorizedEditorHost from here:
/// the native authority is already held by the caller's package transaction.
pub fn verify_package_guard(
    host: &dyn EditorHost,
    scope: &Scope,
    limits: GatewayLimits,
    target: &PackageRevision,
    expected: &SourceRef,
) -> Result<(), String> {
    expected.check_current()?;
    if host.snapshot()?.source != *expected {
        return Err("Package source changed; refresh".into());
    }
    let (artifact, stamp) =
        studio_store::get_artifact_stamped(&scope.source_id).ok_or("Native source absent")?;
    if target.source.project_id != scope.project_id
        || target.source.document_id != scope.document_id
        || target.source.source_id != scope.source_id
        || artifact.domain != scope.domain
        || stamp != expected.stamp
        || artifact.id != expected.artifact_id
        || u64::from(artifact.lamport_clock) != target.source.revision
        || source_digest(&artifact, limits)? != target.source.source_digest
    {
        return Err(
            "Package review source/project/revision differs from current native source".into(),
        );
    }
    let inventory = host.package_inventory(&target.filename, expected.clone())?;
    if inventory.digest != target.package_digest {
        return Err("Saved HMC bytes changed; package review withheld".into());
    }
    // Checkpoint inventory performs bounded canonical load, source/domain and
    // immutable-release checks. Native audit separately binds the byte count.
    expected.check_current()?;
    if host.snapshot()?.source != *expected {
        return Err("Source changed during HMC verification".into());
    }
    Ok(())
}

pub struct PackageWorkflowService {
    authority: Arc<NativeAuthority>,
    sessions: SessionSlot,
    shared: SharedEditorHost,
    scope: Scope,
    limits: GatewayLimits,
}
impl PackageWorkflowService {
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
    fn inspect(
        &self,
        session: &SessionHandle,
        target: &PackageRevision,
        expected: &SourceRef,
    ) -> Result<PackageWorkflowView, String> {
        // Acquire authority -> raw editor, and RETAIN that guard until the
        // authority call (including any commit) has returned. No recursive lock.
        let mut retained = None;
        let evidence = self
            .authority
            .package_workflow_evidence(session, target, |target| {
                let guard = self.shared.package_review_guard()?;
                verify_package_guard(guard.as_ref(), &self.scope, self.limits, target, expected)?;
                retained = Some(guard);
                Ok(())
            })?;
        drop(retained);
        if !self
            .authority
            .publication_is_current(session, evidence.authority_generation)
        {
            return Err("Native authority changed; package workflow withheld".into());
        }
        expected.check_current()?;
        let publication = PackagePublicationStamp {
            selection: self.sessions.clone(),
            authority: self.authority.clone(),
            session: session.clone(),
            generation: evidence.authority_generation,
            target: evidence.target.clone(),
        };
        Ok(PackageWorkflowView {
            source: expected.clone(),
            evidence,
            publication,
        })
    }
    pub fn run(
        &self,
        action: PackageWorkflowAction,
        expected: SourceRef,
    ) -> Result<PackageWorkflowView, String> {
        action.validate()?;
        // Retain selection through verification, record commit and final
        // inspection. No callback re-enters this selected-session service.
        // Consistent lock order: selection -> authority -> raw shared editor.
        let selected = self.sessions.try_lock().map_err(|_| "Native session selection busy")?;
        let session = selected.clone().ok_or("Sign in with an admitted contributor or package reviewer")?;
        if let PackageWorkflowAction::Inspect(selection) = &action {
            let target = self.authority.saved_package_target(
                &session,
                &selection.filename,
                &selection.package_id,
            )?;
            if target.package_digest != selection.digest {
                return Err("Package changed since inventory; refresh".into());
            }
            return self.inspect(&session, &target, &expected);
        }
        let target = match &action {
            PackageWorkflowAction::Contribution { target, .. }
            | PackageWorkflowAction::Request { target, .. }
            | PackageWorkflowAction::Decide { target, .. } => target.clone(),
            _ => unreachable!(),
        };
        let mut retained = None;
        let verify = |target: &PackageRevision| {
            let guard = self.shared.package_review_guard()?;
            verify_package_guard(guard.as_ref(), &self.scope, self.limits, target, &expected)?;
            retained = Some(guard);
            Ok(())
        };
        match action {
            PackageWorkflowAction::Contribution {
                generation,
                id,
                description,
                minutes,
                status,
                ..
            } => {
                self.authority.record_package_contribution(
                    &session,
                    generation,
                    &target,
                    &id,
                    &description,
                    minutes,
                    status,
                    verify,
                )?;
            }
            PackageWorkflowAction::Request {
                generation,
                contribution_id,
                id,
                evidence,
                ..
            } => {
                self.authority.request_package_review(
                    &session,
                    generation,
                    &target,
                    &contribution_id,
                    &id,
                    &evidence,
                    verify,
                )?;
            }
            PackageWorkflowAction::Decide {
                generation,
                request_id,
                id,
                decision,
                rationale,
                evidence,
                ..
            } => {
                self.authority.record_requested_package_review(
                    &session,
                    generation,
                    &target,
                    &request_id,
                    &id,
                    decision,
                    &rationale,
                    &evidence,
                    verify,
                )?;
            }
            _ => unreachable!(),
        }
        drop(retained);
        self.inspect(&session, &target, &expected).map_err(|error| {
            format!("Package record may have committed; inspect before retrying: {error}")
        })
    }
}
