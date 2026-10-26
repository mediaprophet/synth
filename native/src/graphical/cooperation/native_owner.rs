//! Poet source projection of the engine-owned authenticated authority transaction.
use super::{source_digest, AuthorityOwner, GatewayLimits, Scope};
use qualia_cooperative_core::authoring::{DocumentRevision, HostAuthorityView};
use qualia_core_db::poet_host::{
    invoke::studio_store,
    native_authority::{DocumentTransaction, NativeAuthority, SessionHandle},
};
use std::sync::Arc;

pub struct NativeAuthorityOwner {
    authority: Arc<NativeAuthority>,
    session: SessionHandle,
    scope: Scope,
    limits: GatewayLimits,
}
impl NativeAuthorityOwner {
    /// The session is an opaque result of actual native key proof and adopted
    /// project admission. No CLI/MCP/source field can construct it.
    pub fn new(
        authority: Arc<NativeAuthority>,
        session: SessionHandle,
        scope: Scope,
        limits: GatewayLimits,
    ) -> Result<Self, String> {
        super::validate_scope(&scope, limits)?;
        let owner = Self {
            authority,
            session,
            scope,
            limits,
        };
        owner.with_current(&mut |_| Ok(()))?;
        Ok(owner)
    }
}
impl NativeAuthorityOwner {
    fn source_transaction<T>(
        &self,
        view: &HostAuthorityView<'_>,
        operation: &mut dyn FnMut(&HostAuthorityView<'_>) -> Result<T, String>,
    ) -> DocumentTransaction<T> {
        let before = view.current_document;
        let actual = || -> Result<DocumentRevision, String> {
            let artifact = studio_store::get_artifact_stamped(&self.scope.source_id)
                .ok_or("admitted native source disappeared")?
                .0;
            if artifact.domain != self.scope.domain || artifact.id != self.scope.source_id {
                return Err("native source domain/identifier mismatch".into());
            }
            Ok(DocumentRevision {
                project_id: self.scope.project_id.clone(),
                document_id: self.scope.document_id.clone(),
                source_id: artifact.id.clone(),
                revision: artifact.lamport_clock.into(),
                source_digest: source_digest(&artifact, self.limits)?,
                author_subject: before.author_subject.clone(),
            })
        };
        let current = match actual() {
            Ok(current) if &current == before => current,
            _ => {
                return DocumentTransaction {
                    result: Err(
                        "native source differs from durable admission; reconciliation required"
                            .into(),
                    ),
                    successor: None,
                    reconciliation_required: true,
                }
            }
        };
        let result = operation(view);
        match actual() {
            Ok(mut after) if after != current => {
                after.author_subject = view.principal.map(|p| p.subject.into());
                DocumentTransaction {
                    reconciliation_required: result.is_err(),
                    result,
                    successor: Some(after),
                }
            }
            Ok(_) => DocumentTransaction {
                result,
                successor: None,
                reconciliation_required: false,
            },
            Err(error) => DocumentTransaction {
                result: Err(error),
                successor: None,
                reconciliation_required: true,
            },
        }
    }
}
impl AuthorityOwner for NativeAuthorityOwner {
    fn with_current_package(
        &self,
        intent: &qualia_core_db::poet_host::native_authority::PackageWriteIntent,
        operation: &mut dyn FnMut(
            &HostAuthorityView<'_>,
        ) -> Result<studio_store::checkpoint::SaveReceipt, String>,
    ) -> Result<studio_store::checkpoint::SaveReceipt, String> {
        self.authority
            .transact_with_package(&self.session, Some(intent), |view| {
                let transaction = self.source_transaction(view, operation);
                let receipt = transaction.result.as_ref().ok().map(|r| {
                    qualia_core_db::poet_host::native_authority::PackageWriteReceipt {
                        digest: r.digest,
                        bytes: r.bytes,
                    }
                });
                (transaction, receipt)
            })
    }
    fn publication_is_current(&self, generation: u64) -> bool {
        self.authority
            .publication_is_current(&self.session, generation)
    }
    fn with_current(
        &self,
        operation: &mut dyn FnMut(&HostAuthorityView<'_>) -> Result<(), String>,
    ) -> Result<(), String> {
        self.authority.transact(&self.session, |view| {
            self.source_transaction(view, operation)
        })
    }
}

/// Fresh-process source activation uses current native authentication while the
/// regular source adapter deliberately still requires an active Studio record.
pub fn reopen_verified(
    authority: &NativeAuthority,
    session: &SessionHandle,
    scope: &Scope,
    service: &crate::graphical::checkpoints::CheckpointService,
    prepared: crate::graphical::checkpoints::PreparedReopen,
    limits: GatewayLimits,
) -> Result<(studio_store::StudioArtifact, studio_store::StoreStamp), String> {
    struct ReopenOwner<'a> {
        authority: &'a NativeAuthority,
        session: &'a SessionHandle,
        source: &'a str,
    }
    impl AuthorityOwner for ReopenOwner<'_> {
        fn with_current(
            &self,
            operation: &mut dyn FnMut(&HostAuthorityView<'_>) -> Result<(), String>,
        ) -> Result<(), String> {
            self.authority.transact(self.session, |view| {
                let before =
                    studio_store::get_artifact_stamped(self.source).map(|(_, stamp)| stamp);
                let result = operation(view);
                let after = studio_store::get_artifact_stamped(self.source).map(|(_, stamp)| stamp);
                DocumentTransaction {
                    reconciliation_required: result.is_err() && before != after,
                    result,
                    successor: None,
                }
            })
        }
    }
    crate::graphical::checkpoints::authorised_reopen::reopen(
        &ReopenOwner {
            authority,
            session,
            source: &scope.source_id,
        },
        scope,
        service,
        prepared,
        limits,
    )
}
