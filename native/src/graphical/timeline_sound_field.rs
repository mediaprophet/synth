//! Bounded cold audition of authored field clips through existing Core DSP.
//! Decoders, source reads, device lifecycle and source persistence stay owned.
use super::{
    authoring::{sound_field::SoundField, timeline::Timeline, timeline_associations::Association},
    checkpoints::{ResourceInspectionRequest, TimelineMediaRequest},
    controller::EditorHost,
    jobs::Context,
    timeline_audio_output::{exact_sample, PreparedWav},
    timeline_wav_device::PlaybackBinding,
};
use qualia_core_db::audio::{
    holosonic::HolosonicEmitter,
    scene_audio_graph::{
        pcm::{PcmRenderBudget, PcmRenderState, PcmSourceBinding},
        pcm_schedule::{
            ClipAssignment, ClipScheduleBudget, ClipScheduleState, ClipScheduleWorkspace,
            ClipSpanReceipt, PcmClip,
        },
        AudioListenerPose, HolosonicNode, SceneAudioBusGraph, MAX_SCENE_AUDIO_EMITTERS,
    },
};
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicBool, Ordering};

const RATE: u32 = 48_000;
const BLOCK: usize = 4096;
const MAX_ENCODED: usize = 16 * 1024 * 1024;
const MAX_DECODED: usize = 64 * 1024 * 1024;
const MAX_WORKING: usize = 128 * 1024 * 1024;

#[derive(Clone)]
pub(super) struct FieldRequest {
    pub package: ResourceInspectionRequest,
    pub start_tick: u64,
    pub duration_ticks: u64,
}
impl FieldRequest {
    pub(super) fn validate(&self) -> Result<(), String> {
        self.package.validate_bounds()?;
        if self.duration_ticks == 0
            || self.duration_ticks > (timeline_editing::TICKS_PER_SECOND as u64) * 15
        {
            return Err(
                "Sound-field audition must be an explicit interval of at most 15 seconds".into(),
            );
        }
        exact_sample(self.start_tick, RATE)?;
        exact_sample(
            self.start_tick
                .checked_add(self.duration_ticks)
                .ok_or("Sound-field interval overflow")?,
            RATE,
        )?;
        Ok(())
    }
}
struct PlannedClip {
    request: TimelineMediaRequest,
    start: u64,
    end: u64,
    slot: usize,
    gain: f32,
    priority: i32,
}
struct Plan {
    graph: SceneAudioBusGraph,
    clips: Vec<PlannedClip>,
    decoded_bytes: usize,
    omitted_timeline_clips: usize,
}

fn plan(
    timeline: &Timeline,
    associations: &[Association],
    field: &SoundField,
    request: &FieldRequest,
) -> Result<Plan, String> {
    request.validate()?;
    field.validate(timeline)?;
    if field.bindings.is_empty() {
        return Err("Assign at least one authored field clip".into());
    }
    let mut entities = BTreeMap::new();
    for b in &field.bindings {
        entities.insert(b.entity_identifier.as_str(), (b.position_m, b.radius_m));
    }
    if entities.len() > MAX_SCENE_AUDIO_EMITTERS {
        return Err(format!("This audition requires {} resident entities; the existing Core profile supports {}. No emitter was dropped. Use an explicit authored subset or a future reviewed residency policy.", entities.len(), MAX_SCENE_AUDIO_EMITTERS));
    }
    let mut graph = SceneAudioBusGraph::new();
    let yaw = field.listener_yaw_rad as f32;
    graph.listener = AudioListenerPose {
        position: field.listener_position_m.map(|v| v as f32),
        yaw_rad: yaw,
        forward: [yaw.sin(), 0.0, -yaw.cos()],
        right: [yaw.cos(), 0.0, yaw.sin()],
        ..Default::default()
    };
    let mut slots = BTreeMap::new();
    for (index, (id, (position, radius))) in entities.into_iter().enumerate() {
        let node = HolosonicNode {
            node_id: index as u32,
            origin_quin_hash: 0,
            deontic_permit: 0,
            emitter: HolosonicEmitter {
                position: position.map(|v| v as f32),
                coherence_radius: radius as f32,
                amplitude: 1.0,
                ..Default::default()
            },
            ..Default::default()
        };
        if !graph.push_emitter(node) {
            return Err("Sound-field resident admission refused; no emitter dropped".into());
        }
        slots.insert(id, index);
    }
    let mut bindings = field.bindings.iter().collect::<Vec<_>>();
    bindings.sort_by(|a, b| a.clip_id.cmp(&b.clip_id));
    let mut clips = Vec::with_capacity(bindings.len());
    let mut decoded_bytes = 0usize;
    for b in bindings {
        let c = timeline
            .tracks
            .iter()
            .flat_map(|t| &t.clips)
            .find(|c| c.id == b.clip_id)
            .ok_or("Field clip disappeared")?;
        let association = associations
            .iter()
            .find(|a| a.clip_id == b.clip_id && a.source_identifier == c.source)
            .ok_or("Every field clip requires its own current source-owned HMC association")?;
        let frames = usize::try_from(exact_sample(c.duration, RATE)?)
            .map_err(|_| "Field source frame count overflow")?;
        if frames == 0 {
            return Err("Field clip has no samples".into());
        }
        decoded_bytes = decoded_bytes
            .checked_add(frames.checked_mul(4).ok_or("Field source bytes overflow")?)
            .ok_or("Field source bytes overflow")?;
        if decoded_bytes > MAX_DECODED {
            return Err(
                "Authored field source windows exceed the 64 MiB decoded audition profile".into(),
            );
        }
        // Both trim and event clocks must align. They remain distinct owners.
        exact_sample(c.source_start, RATE)?;
        let start = exact_sample(c.start, RATE)?;
        let end = exact_sample(
            c.start
                .checked_add(c.duration)
                .ok_or("Field clip end overflow")?,
            RATE,
        )?;
        clips.push(PlannedClip {
            request: TimelineMediaRequest {
                clip_id: c.id.clone(),
                timeline_tick: c.start,
                resource: ResourceInspectionRequest {
                    filename: request.package.filename.clone(),
                    package_digest: request.package.package_digest,
                    key: association.resource_key.clone(),
                    resource_digest: association.resource_digest,
                },
            },
            start,
            end,
            slot: *slots
                .get(b.entity_identifier.as_str())
                .ok_or("Field entity disappeared")?,
            gain: b.gain as f32,
            priority: b.priority,
        });
    }
    Ok(Plan {
        graph,
        clips,
        decoded_bytes,
        omitted_timeline_clips: timeline.tracks.iter().map(|t| t.clips.len()).sum::<usize>()
            - field.bindings.len(),
    })
}
fn live(
    host: &dyn EditorHost,
    context: &Context,
    closed: &AtomicBool,
    cancelled: &AtomicBool,
    worker: &AtomicBool,
) -> Result<(), String> {
    if closed.load(Ordering::Acquire)
        || cancelled.load(Ordering::Acquire)
        || worker.load(Ordering::Acquire)
    {
        return Err("Sound-field audition cancelled".into());
    }
    let now = host.snapshot()?;
    if !now.publication_is_current() || Context::from_snapshot(&now) != *context {
        return Err("Sound-field source/session/authority changed; output withheld".into());
    }
    Ok(())
}
pub(super) struct PreparedField {
    pub pcm: PreparedWav,
    pub context: Context,
    pub binding: PlaybackBinding,
    pub summary: String,
}
pub(super) fn prepare(
    host: &dyn EditorHost,
    request: &FieldRequest,
    expected: &super::preview::SourceRef,
    closed: &AtomicBool,
    cancelled: &AtomicBool,
    worker: &AtomicBool,
) -> Result<PreparedField, String> {
    request.validate()?;
    let initial = host.snapshot()?;
    if initial.source != *expected
        || initial.reconciliation_required
        || !initial.publication_is_current()
    {
        return Err("Sound-field source or authority is not current".into());
    }
    let context = Context::from_snapshot(&initial);
    live(host, &context, closed, cancelled, worker)?;
    let view = host.authoring_view(expected.clone())?;
    let timeline = view.timeline.as_ref().ok_or("No admitted timeline")?;
    let field = view.sound_field.as_ref().ok_or("No authored sound-field")?;
    let plan = plan(timeline, &view.timeline_media, field, request)?;
    let mut decoded = Vec::with_capacity(plan.clips.len());
    let mut encoded_bytes = 0usize;
    for c in &plan.clips {
        live(host, &context, closed, cancelled, worker)?;
        let selected = host.timeline_media(c.request.clone(), expected.clone(), cancelled)?;
        encoded_bytes = encoded_bytes
            .checked_add(selected.selected.bytes.len())
            .ok_or("Field input size overflow")?;
        if encoded_bytes > MAX_ENCODED {
            return Err(
                "Sound-field combined encoded resources exceed 16 MiB audition profile".into(),
            );
        }
        let pcm = super::timeline_audio_output::prepare(&selected, cancelled)?;
        if pcm.rate != RATE || pcm.channels != 1 || pcm.frames() as u64 != c.end - c.start {
            return Err("Sound-field requires exact 48 kHz mono clip windows; explicit conversion is required for other clocks/channels".into());
        }
        decoded.push(pcm);
        live(host, &context, closed, cancelled, worker)?;
    }
    let (pcm, limited_input, limited_output, peak) = render(&plan, &decoded, request, || {
        live(host, &context, closed, cancelled, worker)
    })?;
    live(host, &context, closed, cancelled, worker)?;
    let first = &plan.clips[0].request;
    let binding = PlaybackBinding {
        clip_id: first.clip_id.clone(),
        resource: first.resource.clone(),
        secondary: plan
            .clips
            .iter()
            .skip(1)
            .map(|c| (c.request.clip_id.clone(), c.request.resource.clone()))
            .collect(),
    };
    let summary = format!("Authored sound-field audition: {} assigned clips, {} resident declared entities, {} timeline clips outside this explicit field; exact 48 kHz mono sources to stereo, static spatial mapping, up to 15 seconds. Input limited samples {}, output limited samples {}, pre-limit peak {}. Limiting requires mix review; device/quality/rights acceptance remains pending.",plan.clips.len(),plan.graph.emitter_count,plan.omitted_timeline_clips,limited_input,limited_output,peak);
    Ok(PreparedField {
        pcm,
        context,
        binding,
        summary,
    })
}
fn render(
    plan: &Plan,
    decoded: &[PreparedWav],
    request: &FieldRequest,
    live: impl Fn() -> Result<(), String>,
) -> Result<(PreparedWav, usize, usize, f32), String> {
    request.validate()?;
    if decoded.len() != plan.clips.len()
        || decoded.iter().zip(&plan.clips).any(|(pcm, c)| {
            pcm.rate != RATE || pcm.channels != 1 || pcm.samples.len() as u64 != c.end - c.start
        })
    {
        return Err("Sound-field requires complete exact 48 kHz mono decoded clip windows".into());
    }
    let start = exact_sample(request.start_tick, RATE)?;
    let frames = usize::try_from(exact_sample(request.duration_ticks, RATE)?)
        .map_err(|_| "Field output frames overflow")?;
    let clips = plan
        .clips
        .iter()
        .zip(decoded)
        .enumerate()
        .map(|(i, (c, pcm))| PcmClip {
            clip_key: i as u64 + 1,
            binding: PcmSourceBinding {
                source_key: i as u64 + 1,
                generation: 1,
                node_id: c.slot as u32,
                origin_quin_hash: 0,
            },
            emitter_slot: c.slot,
            start_frame: c.start,
            end_frame: c.end,
            source_start_frame: 0,
            loop_frames: None,
            gain: c.gain,
            priority: c.priority,
            samples: &pcm.samples,
        })
        .collect::<Vec<_>>();
    let spans = clips
        .len()
        .checked_mul(2)
        .and_then(|n| n.checked_add(1))
        .ok_or("Field span count overflow")?;
    let selections = spans
        .checked_mul(clips.len())
        .ok_or("Field selection count overflow")?;
    let logical = plan
        .decoded_bytes
        .checked_add(frames.checked_mul(8).ok_or("Field output size overflow")?)
        .and_then(|n| n.checked_add(selections.checked_mul(std::mem::size_of::<usize>())?))
        .and_then(|n| {
            n.checked_add(
                plan.graph
                    .emitter_count
                    .checked_mul(BLOCK)?
                    .checked_mul(4)?,
            )
        })
        .and_then(|n| n.checked_add(BLOCK.checked_mul(16)?))
        .and_then(|n| n.checked_add(spans.checked_mul(std::mem::size_of::<ClipSpanReceipt>())?))
        .and_then(|n| {
            n.checked_add(
                clips
                    .len()
                    .checked_mul(std::mem::size_of::<Option<ClipAssignment>>())?,
            )
        })
        .and_then(|n| {
            n.checked_add(
                clips
                    .len()
                    .checked_mul(std::mem::size_of::<PcmClip<'_>>())?,
            )
        })
        .and_then(|n| n.checked_add(std::mem::size_of::<SceneAudioBusGraph>()))
        .and_then(|n| n.checked_add(std::mem::size_of::<PcmRenderState>().checked_mul(2)?))
        .ok_or("Field working set overflow")?;
    if logical > MAX_WORKING {
        return Err("Sound-field logical audition buffers exceed 128 MiB".into());
    }
    let mut assignments = filled(clips.len(), None)?;
    let mut state = ClipScheduleState::new(&mut assignments);
    let mut mono = filled(
        plan.graph
            .emitter_count
            .checked_mul(BLOCK)
            .ok_or("Field mono size overflow")?,
        0.0,
    )?;
    let mut scratch_l = filled(BLOCK, 0.0)?;
    let mut scratch_r = filled(BLOCK, 0.0)?;
    let mut selected = filled(selections, 0)?;
    let mut span_receipts = filled(spans, ClipSpanReceipt::default())?;
    let mut work = ClipScheduleWorkspace::new(
        &mut mono,
        &mut scratch_l,
        &mut scratch_r,
        &mut selected,
        &mut span_receipts,
    );
    let mut l = filled(BLOCK, 0.0)?;
    let mut r = filled(BLOCK, 0.0)?;
    let mut output = Vec::new();
    output
        .try_reserve_exact(frames.checked_mul(2).ok_or("Field output size overflow")?)
        .map_err(|_| "Field output allocation refused")?;
    let mut offset = 0usize;
    let mut limited_input = 0usize;
    let mut limited_output = 0usize;
    let mut peak = 0.0f32;
    while offset < frames {
        live()?;
        let count = (frames - offset).min(BLOCK);
        let block_start = start
            .checked_add(offset as u64)
            .ok_or("Field clock overflow")?;
        let report = plan
            .graph
            .render_scheduled_pcm_quantum_zero_heap(
                &mut state,
                &mut work,
                &clips,
                block_start,
                1,
                &[],
                ClipScheduleBudget {
                    max_authored_clips: clips.len(),
                    max_selected_clips: clips.len(),
                    max_event_spans: spans,
                    max_work: 1_000_000_000,
                    max_transaction_bytes: 16 * 1024 * 1024,
                    render: PcmRenderBudget {
                        max_frames: BLOCK,
                        max_active_voices: plan.graph.emitter_count,
                        max_obstacles: 0,
                        sample_rate: RATE,
                    },
                },
                &mut l[..count],
                &mut r[..count],
            )
            .map_err(|e| format!("Sound-field scheduling/DSP refused: {e:?}"))?;
        limited_input = limited_input
            .checked_add(report.limited_input_samples)
            .ok_or("Field limit count overflow")?;
        limited_output = limited_output
            .checked_add(report.limited_output_samples)
            .ok_or("Field limit count overflow")?;
        peak = peak.max(report.peak_before_limit);
        live()?;
        for (&left, &right) in l[..count].iter().zip(&r[..count]) {
            output.push(left);
            output.push(right);
        }
        offset += count;
    }
    Ok((
        PreparedWav {
            rate: RATE,
            channels: 2,
            source_start_frame: 0,
            samples: output,
        },
        limited_input,
        limited_output,
        peak,
    ))
}

fn filled<T: Clone>(count: usize, value: T) -> Result<Vec<T>, String> {
    let mut out = Vec::new();
    out.try_reserve_exact(count)
        .map_err(|_| "Sound-field buffer allocation refused")?;
    out.resize(count, value);
    Ok(out)
}

#[cfg(test)]
#[path = "timeline_sound_field_tests.rs"]
mod tests;
