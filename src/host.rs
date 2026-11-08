//! Asset-neutral authenticated production host. The embedding chooses the producer.
use crate::production::ProductionSession;
use qualia_cooperative_core::authoring::{
    authorize, AuthoringAction, AuthoringRequest, DocumentRevision,
};
use qualia_core_db::poet_host::native_authority::{
    DocumentTransaction, NativeAuthority, SessionHandle,
};
use std::sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    Arc,
};
#[path = "host_package.rs"]
mod package_host;
pub use package_host::{NativeOutputRoot, SavedCandidate};

static NEXT_GENERATION: AtomicU64 = AtomicU64::new(1);
fn reserve_generation() -> Result<u64, String> {
    NEXT_GENERATION
        .fetch_update(Ordering::AcqRel, Ordering::Acquire, |n| n.checked_add(1))
        .map_err(|_| "Production generation exhausted".into())
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativeContext {
    pub document: DocumentRevision,
    pub authority_generation: u64,
    pub source_generation: u64,
    pub output_generation: u64,
}
pub struct HostSnapshot {
    pub context: NativeContext,
    pub producer: String,
    pub producer_version: String,
    pub resources: Vec<(String, String, usize)>,
    pub actor_kind: qualia_cooperative_core::authoring::PrincipalKind,
    pub actor_subject: String,
    pub source_cell_bytes: Vec<usize>,
    pub output_configured: bool,
}
pub struct SynthHost {
    authority: Arc<NativeAuthority>,
    session: SessionHandle,
    production: ProductionSession,
    document: DocumentRevision,
    generation: u64,
    source_generation: u64,
    output_root: Option<NativeOutputRoot>,
    output_generation: u64,
    quarantined: bool,
}
fn transaction<T>(result: Result<T, String>) -> DocumentTransaction<T> {
    DocumentTransaction {
        result,
        successor: None,
        reconciliation_required: false,
    }
}
pub(crate) fn digest(text: &str) -> Result<[u8; 32], String> {
    if text.len() != 64 || !text.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err("Invalid SHA-256".into());
    }
    let mut bytes = [0; 32];
    for (i, out) in bytes.iter_mut().enumerate() {
        *out = u8::from_str_radix(&text[i * 2..i * 2 + 2], 16).map_err(|_| "Invalid SHA-256")?;
    }
    Ok(bytes)
}
impl SynthHost {
    /// The native document must bind `production.document_digest()`: producer,
    /// retained input hashes and source history. No source can create authority.
    pub fn admit(
        authority: Arc<NativeAuthority>,
        session: SessionHandle,
        production: ProductionSession,
    ) -> Result<Self, String> {
        let binding = production.document_digest()?;
        let (document, generation) = authority.transact(&session, |v| {
            transaction((|| {
                authorize(
                    v,
                    &AuthoringRequest {
                        action: AuthoringAction::ReadSource,
                        target: v.current_document,
                        expected_authority_generation: v.generation,
                    },
                )
                .map_err(|e| e.to_string())?;
                if v.current_document.source_digest != binding {
                    return Err("Production binding differs from native document".into());
                }
                Ok((v.current_document.clone(), v.generation))
            })())
        })?;
        Ok(Self {
            authority,
            session,
            production,
            document,
            generation,
            source_generation: reserve_generation()?,
            output_root: None,
            output_generation: 0,
            quarantined: false,
        })
    }
    pub fn requires_reconciliation(&self) -> bool {
        self.quarantined
    }
    fn context(&self) -> NativeContext {
        NativeContext {
            document: self.document.clone(),
            authority_generation: self.generation,
            source_generation: self.source_generation,
            output_generation: self.output_generation,
        }
    }
    fn guard(&self, expected: &NativeContext) -> Result<(), String> {
        if self.quarantined || &self.context() != expected {
            return Err("Production context is stale or requires reconciliation".into());
        }
        Ok(())
    }
    pub fn inspect(&self) -> Result<HostSnapshot, String> {
        let expected = self.context();
        self.guard(&expected)?;
        self.authority.transact(&self.session, |v| {
            transaction((|| {
                authorize(
                    v,
                    &AuthoringRequest {
                        action: AuthoringAction::ReadSource,
                        target: &expected.document,
                        expected_authority_generation: expected.authority_generation,
                    },
                )
                .map_err(|e| e.to_string())?;
                let principal = v.principal.ok_or("Native principal unavailable")?;
                Ok(HostSnapshot {
                    context: expected.clone(),
                    producer: self.production.profile().0.into(),
                    producer_version: self.production.profile().1.into(),
                    resources: self
                        .production
                        .candidate()
                        .ok_or("No admitted candidate")?
                        .outputs()
                        .iter()
                        .map(|r| (r.key.clone(), r.kind.clone(), r.bytes.len()))
                        .collect(),
                    actor_kind: principal.kind,
                    actor_subject: principal.subject.to_owned(),
                    source_cell_bytes: self.production.sources().iter().map(String::len).collect(),
                    output_configured: self.output_root.is_some(),
                })
            })())
        })
    }
    pub fn read_source(
        &self,
        expected: &NativeContext,
        index: usize,
        offset: usize,
        bytes: usize,
    ) -> Result<String, String> {
        self.guard(expected)?;
        if bytes == 0 || bytes > 16 * 1024 {
            return Err("Source reads require 1..16384 bytes".into());
        }
        self.authority.transact(&self.session, |v| {
            transaction((|| {
                authorize(
                    v,
                    &AuthoringRequest {
                        action: AuthoringAction::ReadSource,
                        target: &expected.document,
                        expected_authority_generation: expected.authority_generation,
                    },
                )
                .map_err(|e| e.to_string())?;
                let cell = self
                    .production
                    .sources()
                    .get(index)
                    .ok_or("Unknown source cell")?;
                let mut end = offset
                    .checked_add(bytes)
                    .ok_or("Read overflow")?
                    .min(cell.len());
                while end > offset && !cell.is_char_boundary(end) {
                    end -= 1;
                }
                if end == offset && offset < cell.len() {
                    return Err("Chunk too small for a UTF-8 character".into());
                }
                cell.get(offset..end)
                    .map(str::to_owned)
                    .ok_or_else(|| "Invalid UTF-8 offsets".into())
            })())
        })
    }
    /// Prepare outside the authority lock; recheck authority on promotion.
    /// Journal failure after promotion quarantines the host.
    pub fn submit(
        &mut self,
        expected: &NativeContext,
        source: &str,
        cancel: &AtomicBool,
    ) -> Result<(), String> {
        self.guard(expected)?;
        self.authority.transact(&self.session, |v| {
            transaction(
                authorize(
                    v,
                    &AuthoringRequest {
                        action: AuthoringAction::ExecuteSource,
                        target: &expected.document,
                        expected_authority_generation: expected.authority_generation,
                    },
                )
                .map(|_| ())
                .map_err(|e| e.to_string()),
            )
        })?;
        let next = self.production.prepare_edit(source, cancel)?;
        let binding = next.document_digest()?;
        let source_generation = reserve_generation()?;
        let mut promoted = false;
        let authority = self.authority.clone();
        let session = self.session.clone();
        let result = authority.transact(&session, |v| {
            let checked = (|| {
                authorize(
                    v,
                    &AuthoringRequest {
                        action: AuthoringAction::ExecuteSource,
                        target: &expected.document,
                        expected_authority_generation: expected.authority_generation,
                    },
                )
                .map_err(|e| e.to_string())?;
                if cancel.load(Ordering::Acquire) {
                    return Err("Production cancelled before promotion".into());
                }
                let mut successor = expected.document.clone();
                successor.revision = successor
                    .revision
                    .checked_add(1)
                    .ok_or("Document revision exhausted")?;
                successor.source_digest = binding;
                successor.author_subject = Some(
                    v.principal
                        .ok_or("Native principal unavailable")?
                        .subject
                        .to_owned(),
                );
                self.production = next;
                self.source_generation = source_generation;
                self.document = successor.clone();
                promoted = true;
                Ok(successor)
            })();
            match checked {
                Ok(successor) => DocumentTransaction {
                    result: Ok(()),
                    successor: Some(successor),
                    reconciliation_required: false,
                },
                Err(error) => transaction(Err(error)),
            }
        });
        if result.is_err() && promoted {
            self.quarantined = true;
        }
        result
    }
}
