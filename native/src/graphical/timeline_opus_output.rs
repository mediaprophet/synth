//! Bounded Ogg Opus preparation after an authorized HMC selection.
use super::{checkpoints::TimelineMediaSelection, timeline_audio_output::{exact_sample, PreparedWav}};
use std::sync::atomic::{AtomicBool, Ordering};

fn append_window(channels: &[Vec<f32>], position: u64, duration: u64, raw_begin: u64, raw_end: u64, output: &mut Vec<f32>) -> Result<(), String> {
    let next = position.checked_add(duration).ok_or("Opus packet time overflow")?;
    let start = raw_begin.max(position);
    let finish = raw_end.min(next);
    if start < finish {
        for frame in (start - position) as usize..(finish - position) as usize {
            for channel in channels {
                let value = *channel.get(frame).ok_or("Opus channel shorter than declared duration")?;
                if !value.is_finite() || !(-1.0..=1.0).contains(&value) { return Err("Opus sample outside finite unit range".into()); }
                output.push(value);
            }
        }
    }
    Ok(())
}
pub(super) fn prepare(selected: &TimelineMediaSelection, cancelled: &AtomicBool) -> Result<PreparedWav, String> {
    const RATE: u32 = 48_000;
    let bytes = &selected.selected.bytes;
    if cancelled.load(Ordering::Acquire) { return Err("Opus preparation cancelled".into()); }
    if bytes.is_empty() || bytes.len() > 2 * 1024 * 1024 || selected.remaining_ticks == 0 ||
        selected.remaining_ticks > timeline_editing::TICKS_PER_SECOND as u64 * 15 {
        return Err("Ogg Opus input or selected duration exceeds profile".into());
    }
    let file = media_ogg::strict_opus::open_bounded_opus(bytes, media_ogg::strict_opus::OpusLimits {
        input_bytes: 2 * 1024 * 1024, pages: 4096, packets: 2048, packet_bytes: 65_025,
    })?;
    if cancelled.load(Ordering::Acquire) { return Err("Opus preparation cancelled after admission".into()); }
    let stream = &file.streams[0];
    let head = audio_opus_codec::OpusHead::parse(&stream.headers[0]).map_err(|e| format!("Opus header refused: {e}"))?;
    let final_granule = u64::try_from(stream.last_granule.ok_or("Opus final granule missing")?).map_err(|_| "Invalid Opus final granule")?;
    let total_decoded: u64 = stream.packets.iter().try_fold(0u64, |sum, packet| {
        let n = media_ogg::opus_packet_samples(packet.prefix, packet.size).ok_or("Opus duration unavailable")?;
        sum.checked_add(u64::from(n)).ok_or("Opus duration overflow")
    })?;
    if total_decoded > u64::from(RATE) * 120 { return Err("Opus decoded work exceeds 120 seconds".into()); }
    let playable = final_granule.checked_sub(u64::from(head.pre_skip)).ok_or("Opus pre-skip exceeds final granule")?;
    let begin = exact_sample(selected.media_tick, RATE)?;
    let end_tick = selected.media_tick.checked_add(selected.remaining_ticks).ok_or("Opus clip time overflow")?;
    let end = exact_sample(end_tick, RATE)?;
    if end <= begin || end > playable { return Err("Clip extends beyond Opus presentation interval".into()); }
    let raw_begin = begin.checked_add(u64::from(head.pre_skip)).ok_or("Opus time overflow")?;
    let raw_end = end.checked_add(u64::from(head.pre_skip)).ok_or("Opus time overflow")?;
    let sample_count = usize::try_from(end - begin).ok().and_then(|n| n.checked_mul(head.channels)).ok_or("Opus output overflow")?;
    if sample_count.checked_mul(4).is_none_or(|n| n > 16 * 1024 * 1024) { return Err("Opus output exceeds PCM profile".into()); }
    let mut output = Vec::new();
    output.try_reserve_exact(sample_count).map_err(|_| "Opus output allocation refused")?;
    let mut decoder = audio_opus_codec::Decoder::from_head(head.clone(), RATE).map_err(|e| format!("Opus decoder refused: {e}"))?;
    let mut position = 0u64;
    for (index, packet) in stream.packets.iter().enumerate() {
        if cancelled.load(Ordering::Acquire) { return Err("Opus preparation cancelled".into()); }
        let duration = u64::from(media_ogg::opus_packet_samples(packet.prefix, packet.size).ok_or("Opus duration unavailable")?);
        let next = position.checked_add(duration).ok_or("Opus packet time overflow")?;
        let encoded = file.read_packet(bytes, 0, index).map_err(|e| format!("Opus packet read failed: {e}"))?;
        let channels = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| decoder.decode(Some(&encoded))))
            .map_err(|_| "Opus codec rejected malformed input")?.map_err(|e| format!("Opus decode refused: {e}"))?;
        if channels.len() != head.channels || channels.iter().any(|c| c.len() != duration as usize) { return Err("Opus decoder output changed shape".into()); }
        append_window(&channels, position, duration, raw_begin, raw_end, &mut output)?;
        position = next;
        if position >= raw_end { break; }
    }
    if output.len() != sample_count || cancelled.load(Ordering::Acquire) { return Err("Opus sample count changed or preparation cancelled".into()); }
    Ok(PreparedWav { rate: RATE, channels: head.channels as u16, source_start_frame: begin, samples: output })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graphical::checkpoints::SelectedMedia;
    fn page(out: &mut Vec<u8>, flags: u8, granule: i64, seq: u32, packet: &[u8]) {
        let start=out.len();out.extend_from_slice(b"OggS");out.push(0);out.push(flags);out.extend_from_slice(&granule.to_le_bytes());
        out.extend_from_slice(&42u32.to_le_bytes());out.extend_from_slice(&seq.to_le_bytes());out.extend_from_slice(&[0;4]);
        out.push(1);out.push(packet.len() as u8);out.extend_from_slice(packet);let crc=media_ogg::crc32(&out[start..]);out[start+22..start+26].copy_from_slice(&crc.to_le_bytes());
    }
    fn selection() -> TimelineMediaSelection {
        let mut bytes=Vec::new();let mut head=b"OpusHead".to_vec();head.extend_from_slice(&[1,1,120,0]);
        head.extend_from_slice(&48_000u32.to_le_bytes());head.extend_from_slice(&[0,0,0]);
        let mut tags=b"OpusTags".to_vec();tags.extend_from_slice(&[0;8]);
        page(&mut bytes,2,0,0,&head);page(&mut bytes,0,0,1,&tags);page(&mut bytes,4,900,2,&[0xf8,0xff,0xfe]);
        let tick=timeline_editing::TICKS_PER_SECOND as u64/48_000;
        TimelineMediaSelection{clip_source_identifier:"opus-fixture".into(),media_tick:10*tick,remaining_ticks:400*tick,selected:SelectedMedia{kind:"audio/ogg".into(),bytes}}
    }
    #[test] fn packet_window_copies_distinct_stereo_samples_across_boundary(){
        let left=vec![0.0,0.1,0.2,0.3];let right=vec![0.0,-0.1,-0.2,-0.3];
        let mut out=Vec::new();
        append_window(&[left.clone(),right.clone()],0,4,2,6,&mut out).unwrap();
        append_window(&[left,right],4,4,2,6,&mut out).unwrap();
        assert_eq!(out,vec![0.2,-0.2,0.3,-0.3,0.0,0.0,0.1,-0.1]);
    }
    #[test] fn clipped_opus_prepares_without_exposing_priming_or_tail(){
        let s=selection();let result=prepare(&s,&AtomicBool::new(false)).unwrap();
        assert_eq!((result.rate,result.channels,result.source_start_frame,result.frames()),(48_000,1,10,400));
        let mut beyond=selection();beyond.remaining_ticks=(timeline_editing::TICKS_PER_SECOND as u64/48_000)*780;
        assert!(prepare(&beyond,&AtomicBool::new(false)).is_err());
        assert!(prepare(&s,&AtomicBool::new(true)).is_err());
    }
}
