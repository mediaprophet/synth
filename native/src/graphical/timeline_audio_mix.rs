//! Bounded, already-authorized timeline audio preparation and mixing.
//!
//! The caller owns source/authority checks for every selection and rechecks
//! its context before playback. This module does not open resources or devices.
use super::{checkpoints::TimelineMediaSelection, timeline_audio_output::{prepare, PreparedWav}};
use qualia_audio::{ProcessPlan, StatefulMixer, TrackState, MAX_BLOCK, MAX_TRACKS};
use std::sync::atomic::{AtomicBool, Ordering};

/// Ephemeral audition controls; these do not alter the authored timeline.
#[derive(Clone, Copy, Debug)]
pub(super) struct MixControls { pub gain: f32, pub pan: f32 }
impl Default for MixControls { fn default() -> Self { Self { gain: 1.0, pan: 0.0 } } }
impl MixControls {
    pub(super) fn validate(self) -> Result<(), String> {
        if !(0.0..=1.0).contains(&self.gain) || !(-1.0..=1.0).contains(&self.pan) {
            return Err("Track gain must be 0–1 and pan -1–1".into());
        }
        Ok(())
    }
}

/// Mix two to sixteen already-authorized mono WAV selections over their shared
/// remaining interval, capped at 15 seconds. Selections must correspond to the
/// same timeline tick; the caller enforces that and retains all source bindings.
pub(super) fn mix_mono(mut selections: Vec<TimelineMediaSelection>, controls: &[MixControls], cancelled: &AtomicBool) -> Result<PreparedWav, String> {
    if !(2..=MAX_TRACKS).contains(&selections.len()) { return Err("Audio mix requires 2–16 tracks".into()); }
    if controls.len()!=selections.len() {return Err("Each mix track needs controls".into());}
    for control in controls {control.validate()?;}
    let mut encoded = 0usize;
    let mut overlap = 15u64.checked_mul(timeline_editing::TICKS_PER_SECOND as u64).ok_or("Audio span overflow")?;
    for selection in &selections {
        if !matches!(selection.selected.kind.as_str(), "audio/wav" | "audio/x-wav" | "audio/wave") {
            return Err("Audio mix requires mono WAV tracks".into());
        }
        encoded = encoded.checked_add(selection.selected.bytes.len()).ok_or("Audio input overflow")?;
        overlap = overlap.min(selection.remaining_ticks);
    }
    if encoded > 8 * 1024 * 1024 { return Err("Audio mix exceeds 8 MiB combined encoded input".into()); }
    if overlap == 0 { return Err("Audio clips have no common remaining interval".into()); }
    let mut tracks = Vec::with_capacity(selections.len());
    for selection in &mut selections {
        if cancelled.load(Ordering::Acquire) { return Err("Audio mix cancelled".into()); }
        selection.remaining_ticks = overlap;
        tracks.push(prepare(selection, cancelled)?);
    }
    mix_prepared(&tracks, controls, cancelled)
}

fn mix_prepared(tracks: &[PreparedWav], controls: &[MixControls], cancelled: &AtomicBool) -> Result<PreparedWav, String> {
    if !(2..=MAX_TRACKS).contains(&tracks.len()) { return Err("Audio mix requires 2–16 tracks".into()); }
    if controls.len()!=tracks.len(){return Err("Each mix track needs controls".into());}
    for control in controls {control.validate()?;}
    let first = &tracks[0];
    if first.channels != 1 || first.rate == 0 || first.rate > 96_000 { return Err("Audio mix supports mono up to 96 kHz".into()); }
    let frames = first.frames();
    if frames == 0 || frames > (first.rate as usize) * 15 { return Err("Audio mix exceeds the 15-second profile".into()); }
    if tracks.iter().any(|t| t.channels != 1 || t.rate != first.rate || t.frames() != frames) {
        return Err("Audio mix requires equal mono rates and exact common frame lengths".into());
    }
    let output_len = frames.checked_mul(2).ok_or("Audio mix size overflow")?;
    let mut output = Vec::new();
    output.try_reserve_exact(output_len).map_err(|_| "Audio mix allocation refused")?;
    output.resize(output_len, 0.0);
    let mut plan = ProcessPlan::new(first.rate, MAX_BLOCK);
    // The QualiaDB centre pan has 0.5 gain per channel. Dividing by
    // track count therefore gives the arithmetic mean per output channel.
    for control in controls {
        plan.add_track(TrackState { gain: 2.0 * control.gain / tracks.len() as f32, pan:control.pan, ..Default::default() }).ok_or("Audio track limit")?;
    }
    let mut mixer = StatefulMixer::new();
    let mut position = 0;
    while position < frames {
        if cancelled.load(Ordering::Acquire) { return Err("Audio mix cancelled".into()); }
        let count = (frames - position).min(MAX_BLOCK);
        let mut input: [&[f32]; MAX_TRACKS] = [&[]; MAX_TRACKS];
        for (slot, track) in input.iter_mut().zip(tracks) { *slot = &track.samples[position..position + count]; }
        mixer.process_block(&plan, &input[..tracks.len()], count, &mut output[position * 2..(position + count) * 2]).map_err(str::to_owned)?;
        for sample in &mut output[position * 2..(position + count) * 2] {
            // Floating-point addition can exceed unity by a few floating-point
            // ulps even when every source is exactly one. Permit only this
            // arithmetic tolerance, then clamp; reject genuine over-range.
            if !sample.is_finite() || sample.abs() > 1.000001 {
                return Err("Audio mix exceeds finite unit output range".into());
            }
            *sample = sample.clamp(-1.0, 1.0);
        }
        position += count;
    }
    if cancelled.load(Ordering::Acquire) { return Err("Audio mix cancelled; result withheld".into()); }
    Ok(PreparedWav { rate: first.rate, channels: 2, source_start_frame: 0, samples: output })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn track(value: f32) -> PreparedWav {
        PreparedWav { rate: 48000, channels: 1, source_start_frame: 0, samples: vec![value; MAX_BLOCK + 1] }
    }
    #[test]
    fn averages_three_tracks_through_a_partial_block() {
        let result = mix_prepared(&[track(0.25), track(0.5), track(-0.75)], &[MixControls::default();3], &AtomicBool::new(false)).unwrap();
        assert_eq!(result.frames(), MAX_BLOCK + 1);
        assert!(result.samples.iter().all(|s| s.abs() < 1e-6));
    }
    #[test]
    fn refuses_cancelled_or_misaligned_tracks() {
        assert!(mix_prepared(&[track(0.0), track(0.0)], &[MixControls::default();2], &AtomicBool::new(true)).is_err());
        let a = track(0.0);
        let mut b = track(0.0);
        b.samples.pop();
        assert!(mix_prepared(&[a, b], &[MixControls::default();2], &AtomicBool::new(false)).is_err());
    }
    #[test]
    fn full_scale_multitrack_roundoff_is_bounded() {
        for n in 2..=MAX_TRACKS {
            let tracks: Vec<_> = (0..n).map(|_| track(1.0)).collect();
            let output = mix_prepared(&tracks, &vec![MixControls::default();n], &AtomicBool::new(false)).unwrap();
            assert!(output.samples.iter().all(|s| (0.999999..=1.0).contains(s)));
        }
    }
    #[test]
    fn low_sample_rate_with_default_bypassed_eq_is_admitted() {
        let mut a = track(0.5);
        let mut b = track(0.5);
        a.rate = 8;
        b.rate = 8;
        a.samples.truncate(8);
        b.samples.truncate(8);
        let result = mix_prepared(&[a, b], &[MixControls::default();2], &AtomicBool::new(false)).unwrap();
        assert_eq!(result.samples[0], 0.5);
    }
    #[test]
    fn audition_controls_apply_gain_and_pan_and_reject_clipping() {
        let c=[MixControls{gain:0.5,pan:-1.0},MixControls{gain:1.0,pan:1.0}];
        let result=mix_prepared(&[track(0.5),track(0.5)],&c,&AtomicBool::new(false)).unwrap();
        assert_eq!(&result.samples[..2],&[0.25,0.5]);
        assert!(mix_prepared(&[track(1.0),track(1.0)],&[MixControls{gain:1.0,pan:-1.0};2],&AtomicBool::new(false)).is_err());
        assert!(mix_prepared(&[track(0.0),track(0.0)],&[MixControls{gain:f32::NAN,pan:0.0};2],&AtomicBool::new(false)).is_err());
    }
}
