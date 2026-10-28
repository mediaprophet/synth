//! One native admission gateway for UI/MCP operations on identified source.
//! Authority is injected by a trusted owner, never restored from source/HMC.
mod editor;
pub mod native_owner;
use super::controller::{EditorHost, EditorOutcome, EditorSnapshot};
use super::preview::SourceRef;
use qualia_cooperative_core::authoring::{
    self, AuthoringAction, AuthoringPermit, AuthoringRequest, HostAuthorityView,
};
use qualia_core_db::poet_host::invoke::studio_store::{self, StudioArtifact};
use sha2::{Digest, Sha256};
use std::{io::Write, sync::Arc};

/// Host binding, not an authentication or permission claim. No Deserialize.
#[derive(Clone, Debug)]
pub struct Scope {
    pub project_id: String,
    pub document_id: String,
    pub source_id: String,
    pub domain: String,
}

#[derive(Clone, Copy)]
pub struct GatewayLimits {
    pub max_scope_text_bytes: usize,
    pub max_source_cells: usize,
    pub max_nodes: usize,
    pub max_digest_bytes: usize,
}
impl Default for GatewayLimits {
    fn default() -> Self {
        Self {
            max_scope_text_bytes: 4096,
            max_source_cells: 64,
            max_nodes: 65_536,
            max_digest_bytes: 4 * 1024 * 1024,
        }
    }
}

/// Native provider must hold its current-state lock/transaction for the entire
/// callback, including source/evidence commit. It must call exactly once and
/// update its admitted document metadata after a write before allowing another
/// action. This trait is not an authentication implementation or cross-store
/// atomicity mechanism. Failed postcommit publication must require reconciliation.
pub trait AuthorityOwner: Send + Sync {
    /// A production native owner persists package actor/digest evidence before
    /// returning. The default supports explicit test/custom owners only and
    /// carries no durable native package-audit claim.
    fn with_current_package(
        &self,
        _intent: &qualia_core_db::poet_host::native_authority::PackageWriteIntent,
        operation: &mut dyn FnMut(
            &HostAuthorityView<'_>,
        ) -> Result<studio_store::checkpoint::SaveReceipt, String>,
    ) -> Result<studio_store::checkpoint::SaveReceipt, String> {
        let mut output = None;
        let mut called = false;
        self.with_current(&mut |view| {
            if called {
                return Err("package callback repeated; reconciliation required".into());
            }
            called = true;
            let receipt = operation(view)?;
            output = Some(receipt);
            Ok(())
        })?;
        output.ok_or("native package operation was not admitted".into())
    }
    /// Fast publication guard, not a fresh authorization or permit. Must never
    /// block or acquire a contended owner lock: return false while busy, expired,
    /// revoked, disconnected, or on any generation mismatch. The native owner
    /// must cover every relevant identifier/session/membership/review change.
    /// Providers without this capability cannot publish delayed authorised data.
    fn publication_is_current(&self, _generation: u64) -> bool {
        false
    }
    fn with_current(
        &self,
        operation: &mut dyn FnMut(&HostAuthorityView<'_>) -> Result<(), String>,
    ) -> Result<(), String>;
}

/// Opaque native publication context; no serialization or authority restoration.
/// Equality identifies the same current owner incarnation and generation only.
#[derive(Clone)]
pub struct AuthorisationStamp {
    owner: Arc<dyn AuthorityOwner>,
    generation: u64,
}
impl AuthorisationStamp {
    pub fn is_current(&self) -> bool {
        self.owner.publication_is_current(self.generation)
    }
}
impl std::fmt::Debug for AuthorisationStamp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AuthorisationStamp")
            .field("generation", &self.generation)
            .finish_non_exhaustive()
    }
}
impl PartialEq for AuthorisationStamp {
    fn eq(&self, other: &Self) -> bool {
        self.generation == other.generation && Arc::ptr_eq(&self.owner, &other.owner)
    }
}
impl Eq for AuthorisationStamp {}

pub struct UnboundAuthorityOwner;
impl AuthorityOwner for UnboundAuthorityOwner {
    fn with_current(
        &self,
        _: &mut dyn FnMut(&HostAuthorityView<'_>) -> Result<(), String>,
    ) -> Result<(), String> {
        Err("No authenticated cooperative authority owner is bound; operation denied".into())
    }
}

struct DigestWriter {
    digest: Sha256,
    bytes: usize,
    limit: usize,
}
impl Write for DigestWriter {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        let next = self
            .bytes
            .checked_add(bytes.len())
            .filter(|n| *n <= self.limit)
            .ok_or_else(|| std::io::Error::other("source digest byte budget exceeded"))?;
        self.digest.update(bytes);
        self.bytes = next;
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

/// Deterministic, bounded hash projection of actual current authored state.
/// BTreeMap ordering is canonical here. Revision is separately bound to Lamport;
/// history/redo are excluded. This is not a saved authority record or file codec.
pub fn source_digest(artifact: &StudioArtifact, limits: GatewayLimits) -> Result<[u8; 32], String> {
    if artifact.nodes.len() > limits.max_nodes {
        return Err("source node count exceeds gateway bounds".into());
    }
    let mut cells = 0usize;
    for (key, node) in &artifact.nodes {
        if key.starts_with("poet.native.submission/") {
            cells = cells.checked_add(1).ok_or("source cell count overflow")?;
            if cells > limits.max_source_cells
                || key != &format!("poet.native.submission/{cells:020}")
                || !node.contains_key("text")
            {
                return Err(
                    "source cells exceed bounds or are not an identified contiguous sequence"
                        .into(),
                );
            }
        }
    }
    let mut writer = DigestWriter {
        digest: Sha256::new(),
        bytes: 0,
        limit: limits.max_digest_bytes,
    };
    serde_json::to_writer(
        &mut writer,
        &(
            "poet.source-authority.v1",
            &artifact.id,
            artifact.did_q42,
            &artifact.domain,
            &artifact.title,
            artifact.routing_lane,
            artifact.sensitivity,
            artifact.context_did_hash,
            &artifact.nodes,
            &artifact.properties,
        ),
    )
    .map_err(|error| format!("source digest projection refused: {error}"))?;
    Ok(writer.digest.finalize().into())
}

fn validate_scope(scope: &Scope, limits: GatewayLimits) -> Result<(), String> {
    if [
        &scope.project_id,
        &scope.document_id,
        &scope.source_id,
        &scope.domain,
    ]
    .into_iter()
    .any(|s| s.is_empty() || s.len() > limits.max_scope_text_bytes)
        || limits.max_source_cells == 0
        || limits.max_nodes == 0
        || limits.max_digest_bytes == 0
    {
        return Err("invalid native project/document/source scope or bounds".into());
    }
    Ok(())
}

/// The provider's current generation is authoritative; request arguments cannot
/// grant authority. Every operation receives one borrowed current permit. Saved
/// records cannot create this permit. A provider error after callback execution
/// is reported honestly: source/effects may have committed and need reconciliation.
pub fn with_authorized<T>(
    owner: &dyn AuthorityOwner,
    scope: &Scope,
    expected: Option<&SourceRef>,
    action: AuthoringAction,
    limits: GatewayLimits,
    operation: impl FnOnce(&HostAuthorityView<'_>, AuthoringPermit<'_>) -> Result<T, String>,
) -> Result<T, String> {
    validate_scope(scope, limits)?;
    let mut calls = 0usize;
    let mut duplicate = false;
    let mut operation = Some(operation);
    let mut output = None;
    let mut callback = |view: &HostAuthorityView<'_>| {
        calls = calls.saturating_add(1);
        if calls != 1 {
            duplicate = true;
            return Err(
                "authority owner invoked operation more than once; no repeated dispatch".into(),
            );
        }
        let result = (|| {
            let current = view.current_document;
            if current.project_id != scope.project_id
                || current.document_id != scope.document_id
                || current.source_id != scope.source_id
            {
                return Err("current cooperative document differs from native host scope".into());
            }
            // Authenticate before cloning/hashing actual source. The permit is
            // not used until the actual source digest/stamp is also bound.
            let permit = authoring::authorize(
                view,
                &AuthoringRequest {
                    action,
                    target: current,
                    expected_authority_generation: view.generation,
                },
            )
            .map_err(|error| error.to_string())?;
            let (artifact, stamp) = studio_store::get_artifact_stamped(&scope.source_id)
                .ok_or("native source disappeared")?;
            if artifact.id != scope.source_id || artifact.domain != scope.domain {
                return Err("native source scope/domain differs from its host binding".into());
            }
            let source = SourceRef {
                artifact_id: artifact.id.clone(),
                stamp,
            };
            if expected.is_some_and(|expected| expected != &source) {
                return Err(
                    "source stamp differs from requested native operation; no dispatch".into(),
                );
            }
            if current.revision != u64::from(artifact.lamport_clock)
                || current.source_digest != source_digest(&artifact, limits)?
            {
                return Err("current cooperative document does not bind the actual native source revision/digest".into());
            }
            source.check_current()?;
            let result =
                operation
                    .take()
                    .ok_or("native operation was already consumed")?(view, permit)?;
            if action == AuthoringAction::ReadSource {
                source.check_current()?;
            }
            Ok(result)
        })();
        let status = result.as_ref().map(|_| ()).map_err(Clone::clone);
        output = Some(result);
        status
    };
    let provider_result = owner.with_current(&mut callback);
    drop(callback);
    if duplicate {
        return Err("authority owner invoked callback repeatedly; operation was not repeated; first operation may have committed, reconciliation required".into());
    }
    if let Err(error) = provider_result {
        return Err(if calls == 0 {
            error
        } else {
            format!("authority owner failed after admission callback; operation may have committed; reconciliation required: {error}")
        });
    }
    output.ok_or("authority owner did not admit any operation")?
}

/// Wrap before passing a host to either native UI or MCP. This wrapper cannot
/// bypass the underlying canonical source stamp/session checks. No bootstrap
/// principal, caller-supplied identifiers, local roster or checkpoint-derived grant is supplied.
pub struct AuthorizedEditorHost {
    inner: Box<dyn EditorHost>,
    scope: Scope,
    owner: Arc<dyn AuthorityOwner>,
    limits: GatewayLimits,
}

// Every source-bearing result carries the same admission context as its final
// wrapped snapshot. Otherwise worker frame/result context equality would reject
// an authorised outcome merely because its inner host supplied no native stamp.
trait AuthorisedResult {
    fn attach_authorisation(&mut self, stamp: AuthorisationStamp);
    // A result without an exact bound frame cannot publish retained preview state.
    fn has_bound_frame(&self) -> bool { false }
    fn withhold_unbound_frame(&mut self) {}
}
impl AuthorisedResult for EditorSnapshot {
    fn attach_authorisation(&mut self, stamp: AuthorisationStamp) {
        self.authorisation_stamp = Some(stamp);
    }
}
impl AuthorisedResult for EditorOutcome {
    fn has_bound_frame(&self) -> bool {
        self.frame.as_ref().is_some_and(|frame| frame.source == self.snapshot.source
            && frame.session_revision == Some(self.snapshot.session_revision)
            && frame.execution_epoch.as_ref() == Some(&self.snapshot.execution_epoch)
            && self.snapshot.execution_epoch.is_current() && self.snapshot.source.check_current().is_ok())
    }
    fn withhold_unbound_frame(&mut self) {
        if self.frame.take().is_some() { self.message.push_str("; mismatched preview withheld"); }
    }
    fn attach_authorisation(&mut self, stamp: AuthorisationStamp) {
        self.snapshot.attach_authorisation(stamp);
    }
}
impl AuthorisedResult for studio_store::checkpoint::SaveReceipt {
    fn attach_authorisation(&mut self, _: AuthorisationStamp) {}
}
impl AuthorizedEditorHost {
    fn write_package(
        &mut self,
        name: &str,
        expected_file: studio_store::checkpoint::ExpectedFile,
        expected: &SourceRef,
        operation: impl FnOnce(
            &mut dyn EditorHost,
        ) -> Result<studio_store::checkpoint::SaveReceipt, String>,
    ) -> Result<studio_store::checkpoint::SaveReceipt, String> {
        struct Current<'a, 'b>(&'a HostAuthorityView<'b>);
        impl AuthorityOwner for Current<'_, '_> {
            fn with_current(
                &self,
                op: &mut dyn FnMut(&HostAuthorityView<'_>) -> Result<(), String>,
            ) -> Result<(), String> {
                op(self.0)
            }
        }
        let intent = qualia_core_db::poet_host::native_authority::PackageWriteIntent {
            filename: name.into(),
            expected_digest: match expected_file {
                studio_store::checkpoint::ExpectedFile::Absent => None,
                studio_store::checkpoint::ExpectedFile::Sha256(digest) => Some(digest),
            },
        };
        let inner = &mut self.inner;
        let scope = &self.scope;
        let limits = self.limits;
        let mut operation = Some(operation);
        self.owner.with_current_package(&intent, &mut |view| {
            with_authorized(
                &Current(view),
                scope,
                Some(expected),
                AuthoringAction::SaveCheckpoint,
                limits,
                |_, _| {
                    operation
                        .take()
                        .ok_or("package operation already consumed")?(
                        inner.as_mut()
                    )
                },
            )
        })
    }
    pub fn new(
        inner: Box<dyn EditorHost>,
        scope: Scope,
        owner: Arc<dyn AuthorityOwner>,
        limits: GatewayLimits,
    ) -> Result<Self, String> {
        validate_scope(&scope, limits)?;
        Ok(Self {
            inner,
            scope,
            owner,
            limits,
        })
    }
    fn read<T>(
        &self,
        operation: impl FnOnce(&dyn EditorHost, u64) -> Result<T, String>,
    ) -> Result<T, String> {
        with_authorized(
            self.owner.as_ref(),
            &self.scope,
            None,
            AuthoringAction::ReadSource,
            self.limits,
            |view, _| {
                let snapshot = self.inner.snapshot()?;
                if snapshot.source.artifact_id != self.scope.source_id
                    || snapshot.artifact_id != self.scope.source_id
                    || u64::from(snapshot.source_revision) != view.current_document.revision
                {
                    return Err("foreign inner editor source".into());
                }
                snapshot.source.check_current()?;
                operation(self.inner.as_ref(), view.generation)
            },
        )
    }
    fn write<T: AuthorisedResult>(
        &mut self,
        expected: &SourceRef,
        action: AuthoringAction,
        operation: impl FnOnce(&mut dyn EditorHost) -> Result<T, String>,
    ) -> Result<T, String> {
        self.write_with_view(expected, action, |_, inner| operation(inner))
    }

    // The shared guard is acquired only inside the admitted authority callback
    // and survives owner post-checks. Source and runtime effects are never rolled
    // back or retried if the owner refuses after operation dispatch.
    fn write_with_view<T: AuthorisedResult>(
        &mut self,
        expected: &SourceRef,
        action: AuthoringAction,
        operation: impl FnOnce(&HostAuthorityView<'_>, &mut dyn EditorHost) -> Result<T, String>,
    ) -> Result<T, String> {
        let stamp_owner = self.owner.clone();
        if let Some(shared) = self.inner.shared_viewport_host() {
            let mut guard = None;
            let mut result = with_authorized(self.owner.as_ref(), &self.scope, Some(expected), action, self.limits, |view, _| {
                guard = Some(shared.viewport_guard()?);
                let mut result = operation(view, guard.as_mut().expect("guard just acquired").as_mut())?;
                result.attach_authorisation(AuthorisationStamp { owner: stamp_owner, generation: view.generation });
                Ok(result)
            });
            // A failed dispatch can have changed session, source, effects or
            // retained preview; a successful dispatch with no bound frame can
            // likewise have changed preview before failing to render.
            let retire = match &result { Ok(outcome) => !outcome.has_bound_frame(), Err(_) => true };
            if retire {
                if let Ok(outcome) = &mut result { outcome.withhold_unbound_frame(); }
                if let Some(editor) = guard.as_mut() {
                    if editor.retire_retained_viewport().is_err() {
                        shared.quarantine_viewport();
                        return Err("Source operation may have committed or produced effects; retained viewport quarantined, shared host reconstruction and reconciliation required; no retry".into());
                    }
                    if let Err(ref error) = result {
                        return Err(format!("{error}; retained viewport reset; source or effects may have committed; reconcile before retry"));
                    }
                }
            }
            return result;
        }
        // Compatibility for existing single-owner, nonshared hosts. No shared
        // viewport atomicity is claimed for independently constructed wrappers.
        let inner = &mut self.inner;
        with_authorized(self.owner.as_ref(), &self.scope, Some(expected), action, self.limits, |view, _| {
            let mut result = operation(view, inner.as_mut())?;
            result.attach_authorisation(AuthorisationStamp { owner: stamp_owner, generation: view.generation });
            Ok(result)
        })
    }

}
#[cfg(test)]
mod tests;
