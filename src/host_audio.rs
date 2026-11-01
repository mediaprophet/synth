use super::*;
use ecopunk_terrain::species_playback::{
    sound::{self, EmitterAnchor, ListenerAnchor},
    time::Tick,
};
use qualia_core_db::{
    audio::scene_audio_graph::{
        pcm_schedule::{ClipScheduleBudget, ClipScheduleState, ClipScheduleWorkspace, PcmClip},
        source_coupling::{SoundBinding, SoundBlockMode, SoundBlockReceipt, SoundScopeState},
        AcousticObstacle, SceneAudioBusGraph,
    },
    render::skeletal::Motor,
};
use sha2::{Digest, Sha256};

impl SynthHost {
    /// Native-only admission: audio bytes are decoded by the existing media
    /// owners before rendering. No wire request supplies raw PCM or a forged lease.
    pub fn prepare_audio(
        &mut self,
        expected: &NativeContext,
        graph_key: &str,
        slots: usize,
        block_frames: usize,
        byte_budget: usize,
    ) -> Result<SoundBinding, String> {
        self.guard(expected)?;
        if self.audio_held.load(Ordering::Acquire) {
            return Err("Native audio recovery required after unacknowledged device reset".into());
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
        let resource = self
            .playback
            .resources()
            .entries()
            .iter()
            .find(|r| r.key == graph_key && r.kind == "q42")
            .ok_or("Select an admitted Q42 sound graph resource")?;
        let (model_sha256, source_sha256) = self
            .playback
            .runtime_binding()
            .ok_or("No source-bound motion runtime")?;
        let timeline = self.playback.timeline().ok_or("No authored transport")?;
        sound::validate_rate(timeline.rate()).map_err(|e| e.to_string())?;
        let source_start =
            sound::exact_source_frame(timeline.timeline()).map_err(|e| e.to_string())?;
        sound::exact_source_frame(timeline.duration()).map_err(|e| e.to_string())?;
        let graph_generation = self
            .audio_graph_generation
            .checked_add(1)
            .ok_or("Sound graph generation exhausted")?;
        let binding = SoundBinding {
            model_sha256,
            source_sha256,
            graph_sha256: Sha256::digest(&resource.bytes).into(),
            source_generation: self.playback.source_generation(),
            graph_generation,
            transport_epoch: expected.transport_epoch.ok_or("No authored transport")?,
        };
        if let Some(old) = self.output.take() {
            // No device wait inside the native authority lock. Never report a
            // successful flush when the selected backend did not acknowledge it.
            if let Err(error) = old.stop() {
                self.audio_held.store(true, Ordering::Release);
                return Err(error);
            }
        }
        self.output_cancel.store(true, Ordering::Release);
        let lease = Arc::new(AtomicBool::new(false));
        let authority = self.authority.clone();
        let session = self.session.clone();
        let document = expected.document.clone();
        let generation = expected.authority_generation;
        let live: Arc<dyn Fn() -> bool + Send + Sync> = Arc::new(move || {
            authority.source_publication_is_current(&session, generation, &document)
        });
        let device = crate::audio::DeviceOutput::prepare(
            binding,
            slots,
            block_frames,
            byte_budget,
            lease.clone(),
            live,
        )?;
        let authority = self.authority.clone();
        let session = self.session.clone();
        let result = authority.transact(&session, |v| {
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
                self.output_cancel = lease.clone();
                self.output = Some(device);
                self.audio_graph_generation = graph_generation;
                self.audio_source_start = source_start;
                Ok(())
            })())
        });
        if result.is_err() {
            lease.store(true, Ordering::Release);
        }
        result.map(|()| binding)
    }
    pub fn play_audio(&self, expected: &NativeContext) -> Result<(), String> {
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
        // The native device worker independently checks the same exact live
        // source/session before starting. It never trusts an MCP context string.
        self.output
            .as_ref()
            .ok_or("No prepared audio device")?
            .play()
    }
    pub fn stop_audio(&self, expected: &NativeContext) -> Result<(), String> {
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
        let result = self.output.as_ref().ok_or("No audio device")?.stop();
        if result.is_err() {
            self.audio_held.store(true, Ordering::Release);
        }
        result
    }
    /// Shared motion clock -> Core sound scheduler -> final native guarded enqueue.
    /// PCM/DSP staging happens outside the authority lock. Failed downstream
    /// acceptance retires both sound history and the owned device epoch.
    pub fn render_sound_block(
        &mut self,
        expected: &NativeContext,
        elapsed: Tick,
        graph: &mut SceneAudioBusGraph,
        scope: &mut SoundScopeState,
        state: &mut ClipScheduleState<'_>,
        workspace: &mut ClipScheduleWorkspace<'_>,
        clips: &[PcmClip<'_>],
        anchors: &[EmitterAnchor],
        listener: ListenerAnchor,
        model_to_world: Motor,
        max_joint_visits: usize,
        first_output_frame: u64,
        mode: SoundBlockMode,
        obstacles: &[AcousticObstacle],
        budget: ClipScheduleBudget,
        l: &mut [f32],
        r: &mut [f32],
        cancel: &AtomicBool,
    ) -> Result<SoundBlockReceipt, String> {
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
        let output = self
            .output
            .as_ref()
            .ok_or("Prepare a native audio device first")?;
        let binding = output.binding();
        if self.output_cancel.load(Ordering::Acquire) {
            return Err("Output epoch is retired".into());
        }
        let authority = self.authority.clone();
        let session = self.session.clone();
        let lease = self.output_cancel.clone();
        let source_tick = ecopunk_terrain::species_playback::time::sample_tick(
            self.audio_source_start
                .checked_add(first_output_frame)
                .ok_or("Sound source frame overflow")?,
            48_000,
        )
        .map_err(str::to_owned)?;
        let held = self.playback.pose_context()?;
        if mode == SoundBlockMode::NaturalTail {
            let timeline = self.playback.timeline().ok_or("No authored transport")?;
            if elapsed != Tick::ZERO
                || timeline.playing()
                || timeline.looping()
                || timeline.position() != timeline.duration()
            {
                return Err("Natural tails require an ended clip and zero motion advance".into());
            }
        }
        let mut staged = false;
        let prepare =
            |motion: &ecopunk_terrain::animal_motion::MotionFrame<'_>,
             context: ecopunk_terrain::species_playback::PlaybackPoseContext| {
                let pose = sound::prepare_pose_for_source(
                    motion,
                    binding,
                    context,
                    anchors,
                    listener,
                    model_to_world,
                    max_joint_visits,
                )
                .map_err(|e| e.to_string())?;
                let receipt = sound::render_at_tick(
                    graph,
                    scope,
                    state,
                    workspace,
                    clips,
                    &pose,
                    source_tick,
                    first_output_frame,
                    mode,
                    context.rate,
                    obstacles,
                    budget,
                    l,
                    r,
                    &|b| {
                        b == binding
                            && !lease.load(Ordering::Acquire)
                            && !cancel.load(Ordering::Acquire)
                    },
                )
                .map_err(|e| e.to_string())?;
                staged = true;
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
                        if cancel.load(Ordering::Acquire) {
                            return Err("Audio block cancelled before enqueue".into());
                        }
                        output.enqueue(&receipt, l, r)?;
                        Ok(())
                    })())
                })?;
                Ok(receipt)
            };
        let result = if mode == SoundBlockMode::NaturalTail {
            self.playback
                .with_current_motion_pose(held, prepare, cancel)
        } else {
            self.playback
                .advance_with_motion_pose(elapsed, prepare, cancel)
        };
        if result.is_err() {
            scope.invalidate();
            lease.store(true, Ordering::Release);
            if staged {
                l.fill(0.);
                r.fill(0.);
            }
        }
        result
    }
}
