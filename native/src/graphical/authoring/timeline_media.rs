//! Checked editorial-to-source clock projection for an admitted timeline.
//! This module does not resolve identifiers, open files, or grant authority.
use super::timeline::Timeline;

/// A source identifier and its exact media tick. The identifier is carried
/// through unchanged and is not automatically a path, digest, or HMC key.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct ClipMediaTime<'a> {
    pub source_identifier: &'a str,
    pub media_tick: u64,
    /// Remaining timeline ticks in this half-open clip after the requested tick.
    pub remaining_ticks: u64,
}

/// Resolve a unique admitted clip at a timeline tick. The active range is
/// half-open `[start, start + duration)`: the tick at the end belongs to the
/// next clip, never both. Media time includes the source trim offset. This is a
/// pure projection; an authorized host must separately resolve the identifier.
pub(crate) fn map_clip_time<'a>(timeline: &'a Timeline, clip_id: &str, timeline_tick: u64) -> Result<ClipMediaTime<'a>, String> {
    timeline.validate()?;
    if clip_id.is_empty() || clip_id.len() > 128 {
        return Err("Invalid clip identifier".into());
    }
    let clip = timeline.tracks.iter().flat_map(|track| track.clips.iter())
        .find(|clip| clip.id == clip_id).ok_or("Timeline clip is absent")?;
    let end = clip.start.checked_add(clip.duration).ok_or("Timeline clip range overflow")?;
    if timeline_tick < clip.start || timeline_tick >= end {
        return Err("Timeline tick is outside the selected clip".into());
    }
    let relative = timeline_tick.checked_sub(clip.start).ok_or("Timeline offset underflow")?;
    let media_tick = clip.source_start.checked_add(relative).ok_or("Media source offset overflow")?;
    Ok(ClipMediaTime { source_identifier: &clip.source, media_tick, remaining_ticks: end - timeline_tick })
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::timeline::{Clip, Track};

    fn timeline() -> Timeline {
        Timeline { version: 1, frame_ticks: 1, tracks: vec![Track {
            id: "track".into(), locked: false, clips: vec![
                Clip { id: "a".into(), source: "https://example.test/物種?q=1".into(), source_start: 300, start: 10, duration: 5 },
                Clip { id: "b".into(), source: "urn:media:b".into(), source_start: 12, start: 15, duration: 2 },
            ],
        }], transitions: vec![] }
    }

    #[test]
    fn maps_half_open_clip_boundaries_and_source_trim() {
        let t = timeline();
        assert_eq!(map_clip_time(&t, "a", 10).unwrap(), ClipMediaTime { source_identifier: "https://example.test/物種?q=1", media_tick: 300, remaining_ticks: 5 });
        assert_eq!(map_clip_time(&t, "a", 14).unwrap().media_tick, 304);
        assert!(map_clip_time(&t, "a", 9).is_err());
        assert!(map_clip_time(&t, "a", 15).is_err());
        assert_eq!(map_clip_time(&t, "b", 15).unwrap().media_tick, 12);
        assert!(map_clip_time(&t, "missing", 10).is_err());
    }

    #[test]
    fn rejects_duplicate_ids_and_source_offset_overflow() {
        let mut t = timeline();
        t.tracks[0].clips[1].id = "a".into();
        assert!(map_clip_time(&t, "a", 10).is_err());
        let mut t = timeline();
        t.tracks[0].clips[0].source_start = i64::MAX as u64;
        assert!(map_clip_time(&t, "a", 10).is_err());
    }
}
