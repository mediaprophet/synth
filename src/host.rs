use ecopunk_terrain::{
    animal_motion::MotionLimits,
    species_playback::{time::Tick, NativeProjectionBudget, PlaybackRate, SpeciesPlaybackSession},
    species_review_resources::{CorpusOptions, ReviewResourceLimits, ReviewResources},
    wallaby_rig::RigLimits,
};
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
#[cfg(feature = "audio-device")]
#[path = "host_audio.rs"]
mod audio_host;
#[path = "host_package.rs"]
mod package_host;
pub use package_host::{NativeOutputRoot, SavedCandidate};
static NEXT_PROJECTION: AtomicU64 = AtomicU64::new(1);
fn reserve_projection() -> Result<u64, String> {
    NEXT_PROJECTION
        .fetch_update(Ordering::AcqRel, Ordering::Acquire, |n| n.checked_add(1))
        .map_err(|_| "Synth projection generation exhausted".into())
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativeContext {
    pub document: DocumentRevision,
    pub authority_generation: u64,
    pub source_generation: u64,
    pub transport_epoch: Option<u64>,
    pub output_generation: u64,
}
pub struct HostSnapshot {
    pub context: NativeContext,
    pub clips: Vec<String>,
    pub selected_clip: Option<String>,
    pub playing: bool,
    pub position: Option<Tick>,
    pub audio_status: &'static str,
    pub actor_kind: qualia_cooperative_core::authoring::PrincipalKind,
    pub actor_subject: String,
    pub source_cell_bytes: Vec<usize>,
    pub output_configured: bool,
}
pub enum TransportCommand {
    SelectClip(String),
    Play,
    Pause,
    Seek(Tick),
    Rate(PlaybackRate),
    Loop(bool),
}

pub struct SynthHost {
    authority: Arc<NativeAuthority>,
    session: SessionHandle,
    playback: SpeciesPlaybackSession,
    document: DocumentRevision,
    generation: u64,
    resources: ReviewResourceLimits,
    rig: RigLimits,
    motion: MotionLimits,
    budget: NativeProjectionBudget,
    corpus: crate::corpus_policy::CorpusPolicy,
    output_root: Option<NativeOutputRoot>,
    output_generation: u64,
    quarantined: bool,
    pub(crate) output_cancel: Arc<AtomicBool>,
    audio_graph_generation: u64,
    audio_source_start: u64,
    audio_held: AtomicBool,
    #[cfg(feature = "audio-device")]
    pub(crate) output: Option<crate::audio::DeviceOutput>,
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
        return Err("Invalid native source-history digest".into());
    }
    let mut bytes = [0; 32];
    for (i, out) in bytes.iter_mut().enumerate() {
        *out = u8::from_str_radix(&text[i * 2..i * 2 + 2], 16)
            .map_err(|_| "Invalid source-history digest")?;
    }
    Ok(bytes)
}
impl SynthHost {
    pub fn requires_reconciliation(&self) -> bool {
        self.quarantined
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
            return Err("Source reads require 1 through 16384 bytes".into());
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
                    .playback
                    .source_cells()
                    .get(index)
                    .ok_or("Unknown source cell")?;
                let mut end = offset
                    .checked_add(bytes)
                    .ok_or("Source read overflow")?
                    .min(cell.len());
                while end > offset && !cell.is_char_boundary(end) {
                    end -= 1;
                }
                if end == offset && offset < cell.len() {
                    return Err("Source chunk is too small for one UTF-8 character".into());
                }
                cell.get(offset..end)
                    .map(str::to_owned)
                    .ok_or_else(|| "Read offsets must be valid UTF-8 boundaries".into())
            })())
        })
    }
    /// The embedding supplies a real opaque session. Strings, HMCs and MCP cannot mint one.
    /// The native document must explicitly bind the existing Vibe history digest;
    /// a Poet Studio artifact digest is a different contract and is refused.
    pub fn admit(
        authority: Arc<NativeAuthority>,
        session: SessionHandle,
        cells: &[String],
        retained: ReviewResources,
        resources: ReviewResourceLimits,
        rig: RigLimits,
        motion: MotionLimits,
        options: &CorpusOptions<'_>,
        budget: NativeProjectionBudget,
        cancel: &AtomicBool,
    ) -> Result<Self, String> {
        let (document, generation) = authority.transact(&session, |v| {
            transaction(Ok((v.current_document.clone(), v.generation)))
        })?;
        let corpus = crate::corpus_policy::CorpusPolicy::retain(options)?;
        let playback = SpeciesPlaybackSession::from_authoritative_source_cells_at_generation(
            cells,
            retained,
            resources,
            rig,
            motion,
            options,
            reserve_projection()?,
            budget,
            cancel,
        )?;
        if digest(
            playback
                .vibe_source_sha256()
                .ok_or("No source-history binding")?,
        )? != document.source_digest
        {
            return Err(
                "Synth source-history digest differs from the admitted native document".into(),
            );
        }
        authority.transact(&session, |v| {
            transaction(
                if v.current_document == &document && v.generation == generation {
                    Ok(())
                } else {
                    Err("Native admission changed during source preparation".into())
                },
            )
        })?;
        Ok(Self {
            authority,
            session,
            playback,
            document,
            generation,
            resources,
            rig,
            motion,
            budget,
            corpus,
            output_root: None,
            output_generation: 0,
            quarantined: false,
            output_cancel: Arc::new(AtomicBool::new(true)),
            audio_graph_generation: 0,
            audio_source_start: 0,
            audio_held: AtomicBool::new(false),
            #[cfg(feature = "audio-device")]
            output: None,
        })
    }
    fn context(&self) -> NativeContext {
        NativeContext {
            document: self.document.clone(),
            authority_generation: self.generation,
            source_generation: self.playback.source_generation(),
            transport_epoch: self.playback.timeline().map(|t| t.epoch()),
            output_generation: self.output_generation,
        }
    }
    fn guard(&self, expected: &NativeContext) -> Result<(), String> {
        if self.quarantined || &self.context() != expected {
            return Err(
                "Synth source/transport context is stale or held for reconciliation".into(),
            );
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
                Ok(HostSnapshot {
                    context: expected.clone(),
                    clips: self
                        .playback
                        .clip_names()
                        .iter()
                        .map(|s| (*s).to_owned())
                        .collect(),
                    selected_clip: self.playback.timeline().map(|t| t.clip().to_owned()),
                    playing: self.playback.timeline().is_some_and(|t| t.playing()),
                    position: self.playback.timeline().map(|t| t.position()),
                    audio_status: self.audio_status(),
                    actor_kind: v.principal.ok_or("Native principal unavailable")?.kind,
                    actor_subject: v
                        .principal
                        .ok_or("Native principal unavailable")?
                        .subject
                        .to_owned(),
                    source_cell_bytes: self
                        .playback
                        .source_cells()
                        .iter()
                        .map(String::len)
                        .collect(),
                    output_configured: self.output_root.is_some(),
                })
            })())
        })
    }
    pub fn control(
        &mut self,
        expected: &NativeContext,
        command: TransportCommand,
    ) -> Result<(), String> {
        self.guard(expected)?;
        let authority = self.authority.clone();
        let session = self.session.clone();
        authority.transact(&session, |v| {
            transaction((|| {
                authorize(
                    v,
                    &AuthoringRequest {
                        action: AuthoringAction::ExecuteSource,
                        target: &expected.document,
                        expected_authority_generation: expected.authority_generation,
                    },
                )
                .map_err(|e| e.to_string())?;
                match command {
                    TransportCommand::SelectClip(c) => self.playback.select_clip_with_sound(&c)?,
                    TransportCommand::Play => self.playback.play()?,
                    TransportCommand::Pause => self.playback.pause()?,
                    TransportCommand::Seek(t) => self.playback.seek_with_sound(t)?,
                    TransportCommand::Rate(r) => self.playback.set_rate_with_sound(r)?,
                    TransportCommand::Loop(b) => self.playback.set_looping(b)?,
                }
                // Transport commands retire old output; a new epoch is admitted explicitly.
                self.output_cancel.store(true, Ordering::Release);
                Ok(())
            })())
        })
    }
    /// Prepare outside the authority lock; only promote the checked final candidate
    /// under native source/actor admission. Journal failure quarantines the host.
    pub fn submit(
        &mut self,
        expected: &NativeContext,
        source: &str,
        cancel: &AtomicBool,
    ) -> Result<(), String> {
        self.guard(expected)?;
        if source.len() > 1024 * 1024 {
            return Err("Source cell exceeds existing species profile".into());
        }
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
        let next = self.corpus.with_options(|options| {
            self.budget.admit_source_edit(
                self.playback.source_cells(),
                self.playback.resources(),
                source.len(),
                self.resources,
                self.rig,
                self.motion,
                options,
            )?;
            let mut cells = self.playback.source_cells().to_vec();
            cells.push(source.to_owned());
            let retained = ReviewResources::admit(
                self.playback.resources().entries().to_vec(),
                self.resources,
            )?;
            let generation = reserve_projection()?;
            SpeciesPlaybackSession::from_authoritative_source_cells_at_generation(
                &cells,
                retained,
                self.resources,
                self.rig,
                self.motion,
                options,
                generation,
                self.budget,
                cancel,
            )
        })?;
        let source_digest = digest(
            next.vibe_source_sha256()
                .ok_or("Candidate lacks source history")?,
        )?;
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
                    return Err("Source preparation cancelled".into());
                }
                let mut successor = expected.document.clone();
                successor.revision = successor
                    .revision
                    .checked_add(1)
                    .ok_or("Native document revision exhausted")?;
                successor.source_digest = source_digest;
                successor.author_subject = Some(
                    v.principal
                        .ok_or("Native principal unavailable")?
                        .subject
                        .to_owned(),
                );
                self.output_cancel.store(true, Ordering::Release);
                self.playback = next;
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
    pub fn audio_status(&self) -> &'static str {
        if self.audio_held.load(Ordering::Acquire) {
            return "audio held; device shutdown/reset not acknowledged";
        }
        #[cfg(feature = "audio-device")]
        if let Some(output) = &self.output {
            return output.status();
        }
        "no device output admitted"
    }
}
