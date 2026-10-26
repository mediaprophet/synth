//! Fresh-store checkpoint admission under the CURRENT native cooperative owner.
//! Prepared file state is inert source, never an authority record. The provider
//! owns authenticated context, its lock and persistence of admitted metadata.
use super::{CheckpointService, PreparedReopen};
use crate::graphical::cooperation::{source_digest, AuthorityOwner, GatewayLimits, Scope};
use qualia_cooperative_core::authoring::{
    self, AuthoringAction, AuthoringRequest, HostAuthorityView,
};
use qualia_core_db::poet_host::invoke::studio_store::{StoreStamp, StudioArtifact};

/// Reopen into an absent Studio record while the owner holds current authority.
/// Does not require an already active source, infer principals/grants from HMC,
/// replay effects or replace accepted work. A reopened interpreter remains held.
///
/// The provider must invoke the callback exactly once under its native lock and
/// persist its admitted document/proof metadata before releasing that lock.
/// This function cannot invent a cross-owner transaction or authenticate humans.
pub fn reopen(
    owner: &dyn AuthorityOwner,
    scope: &Scope,
    service: &CheckpointService,
    prepared: PreparedReopen,
    limits: GatewayLimits,
) -> Result<(StudioArtifact, StoreStamp), String> {
    if [
        &scope.project_id,
        &scope.document_id,
        &scope.source_id,
        &scope.domain,
    ]
    .iter()
    .any(|s| s.is_empty() || s.len() > limits.max_scope_text_bytes || s.as_bytes().contains(&0))
        || limits.max_source_cells == 0
        || limits.max_nodes == 0
        || limits.max_digest_bytes == 0
    {
        return Err("invalid native checkpoint scope or gateway bounds".into());
    }
    let binding = service.binding();
    if binding.project_id != scope.project_id
        || binding.document_id != scope.document_id
        || binding.source_id != scope.source_id
        || binding.domain != scope.domain
        || prepared.artifact().id != scope.source_id
        || prepared.artifact().domain != scope.domain
    {
        return Err(
            "checkpoint service/source differs from the trusted native reopen scope".into(),
        );
    }
    let mut prepared = Some(prepared);
    let mut calls = 0usize;
    let mut duplicate = false;
    let mut output = None;
    let mut callback = |view: &HostAuthorityView<'_>| {
        calls = calls.saturating_add(1);
        if calls != 1 {
            duplicate = true;
            return Err(
                "authority owner invoked reopen more than once; no repeated activation".into(),
            );
        }
        let result = (|| {
            let document = view.current_document;
            if document.project_id != scope.project_id
                || document.document_id != scope.document_id
                || document.source_id != scope.source_id
            {
                return Err(
                    "current cooperative document differs from the trusted reopen scope".into(),
                );
            }
            // Only current native state can produce this borrowed permit.
            let _permit = authoring::authorize(
                view,
                &AuthoringRequest {
                    action: AuthoringAction::ReopenCheckpoint,
                    target: document,
                    expected_authority_generation: view.generation,
                },
            )
            .map_err(|error| error.to_string())?;
            let candidate = prepared
                .as_ref()
                .ok_or("prepared reopen was already consumed")?;
            if document.revision != u64::from(candidate.artifact().lamport_clock)
                || document.source_digest != source_digest(candidate.artifact(), limits)?
            {
                return Err(
                    "current native owner does not bind the prepared source revision/digest".into(),
                );
            }
            // Service rereads exact prepared disk SHA and core atomically
            // creates an absent source with a fresh process-local incarnation.
            service.activate(
                prepared
                    .take()
                    .ok_or("prepared reopen was already consumed")?,
            )
        })();
        let status = result.as_ref().map(|_| ()).map_err(Clone::clone);
        output = Some(result);
        status
    };
    let provider_result = owner.with_current(&mut callback);
    drop(callback);
    if duplicate {
        return Err("authority owner repeated reopen callback; activation was not repeated; first activation may have committed, reconciliation required".into());
    }
    if let Err(error) = provider_result {
        return Err(if calls == 0 {
            error
        } else {
            format!("authority owner failed after reopen callback; activation may have committed; reconciliation required: {error}")
        });
    }
    output.ok_or("authority owner did not admit any reopen operation")?
}

#[cfg(test)]
#[path = "authorised_reopen_tests.rs"]
mod tests;
