//! Bounded PCM preparation for a selected source-associated WAV clip.
//! The source must already have passed the existing authorized HMC read owner.
//! This portion does not open an audio device or claim audible playback.
use super::{checkpoints::TimelineMediaSelection,timeline_audio_preview::admit_wav};
use qualia_audio::{wav::decode_wav,types::SampleFormat};
use std::sync::atomic::{AtomicBool,Ordering};

const MAX_INPUT:usize=16*1024*1024;
const MAX_PAYLOAD:usize=80*1024*1024;

pub(super) struct PreparedWav {pub rate:u32,pub channels:u16,pub source_start_frame:u64,pub samples:Vec<f32>}
impl PreparedWav { pub fn frames(&self)->usize {self.samples.len()/usize::from(self.channels)} }

/// A bounded two-mono-track audition. Qualia Audio's existing block plan
/// supplies equal centre-panned gains (0.5 per input on each output channel).
/// The output stops at the shorter admitted overlap, capped at 15 seconds.
pub(super) fn mix_two_mono(
    mut a:TimelineMediaSelection, mut b:TimelineMediaSelection, cancelled:&AtomicBool,
)->Result<PreparedWav,String>{
    const MAX_MIX_INPUT:usize=8*1024*1024;
    if !matches!(a.selected.kind.as_str(),"audio/wav"|"audio/x-wav"|"audio/wave") || !matches!(b.selected.kind.as_str(),"audio/wav"|"audio/x-wav"|"audio/wave") {return Err("Two-WAV audition requires WAV on both tracks".into());}
    let bytes=a.selected.bytes.len().checked_add(b.selected.bytes.len()).ok_or("Audio input overflow")?;
    if bytes>MAX_MIX_INPUT {return Err("Two-WAV mix exceeds the 8 MiB combined encoded input profile".into());}
    let max_ticks=(timeline_editing::TICKS_PER_SECOND as u64).checked_mul(15).ok_or("Audio span overflow")?;
    let overlap=a.remaining_ticks.min(b.remaining_ticks).min(max_ticks);
    if overlap==0 {return Err("Selected audio clips have no common remaining interval".into());}
    a.remaining_ticks=overlap;b.remaining_ticks=overlap;
    let first=prepare(&a,cancelled)?;
    if first.channels!=1 || first.rate>96_000 {return Err("Two-WAV audition supports mono tracks at 96 kHz or less".into());}
    let second=prepare(&b,cancelled)?;
    if second.channels!=1 || second.rate!=first.rate || second.frames()!=first.frames() {return Err("Two-WAV audition requires equal mono sample rates and exact common frame lengths".into());}
    let n=first.frames();let out_len=n.checked_mul(2).ok_or("Audio mix size overflow")?;
    let mut out=Vec::new();out.try_reserve_exact(out_len).map_err(|_|"Audio mix allocation refused")?;out.resize(out_len,0.0);
    let mut pos=0usize;
    while pos<n {
        if cancelled.load(Ordering::Acquire){return Err("Audio mix cancelled".into());}
        let len=(n-pos).min(qualia_audio::production::MAX_BLOCK);
        let mut plan=qualia_audio::production::ProcessPlan::new(first.rate,len);
        plan.add_track(qualia_audio::production::TrackState::default()).ok_or("Audio track limit")?;
        plan.add_track(qualia_audio::production::TrackState::default()).ok_or("Audio track limit")?;
        plan.process_block(&[&first.samples[pos..pos+len],&second.samples[pos..pos+len]],&mut out[pos*2..(pos+len)*2]).map_err(|e|e.to_string())?;
        if out[pos*2..(pos+len)*2].iter().any(|s|!s.is_finite() || !(-1.0..=1.0).contains(s)){return Err("Audio mix exceeds the finite unit output range".into());}
        pos+=len;
    }
    if cancelled.load(Ordering::Acquire){return Err("Audio mix cancelled; result withheld".into());}
    Ok(PreparedWav{rate:first.rate,channels:2,source_start_frame:0,samples:out})
}

pub(super) fn exact_sample(ticks:u64,rate:u32)->Result<u64,String>{
    let scaled=u128::from(ticks).checked_mul(u128::from(rate)).ok_or("Audio sample time overflow")?;
    let denom=timeline_editing::TICKS_PER_SECOND as u128;
    if scaled%denom!=0 {return Err("Clip boundary does not align to an audio sample; explicit conversion required".into());}
    u64::try_from(scaled/denom).map_err(|_|"Audio sample time overflow".into())
}
/// Seek uses the original selected timeline position and its bounded remainder;
/// this does not infer media timing or round between sample boundaries.
pub(super) fn relative_seek_frame(start_tick:u64,seek_tick:u64,rate:u32,frame_count:usize)->Result<usize,String>{
    if rate==0 || seek_tick<start_tick {return Err("Seek is before the selected clip remainder".into());}
    let frame=exact_sample(seek_tick-start_tick,rate)?;
    let frame=usize::try_from(frame).map_err(|_|"Seek frame overflow")?;
    if frame>=frame_count {return Err("Seek is outside the selected clip remainder".into());}
    Ok(frame)
}

/// Convert only the exact selected clip remainder to channel-preserving PCM.
/// F32 samples outside [-1,1] and nonfinite samples are refused, not clamped.
pub(super) fn prepare(selected:&TimelineMediaSelection,cancelled:&AtomicBool)->Result<PreparedWav,String>{
    if matches!(selected.selected.kind.as_str(),"audio/ogg"|"audio/opus") {return super::timeline_opus_output::prepare(selected,cancelled);}
    if matches!(selected.selected.kind.as_str(),"audio/mp4"|"video/mp4") {return prepare_mp4_aac(selected,cancelled);}
    if matches!(selected.selected.kind.as_str(),"audio/aac"|"audio/aac-adts") {return prepare_adts(selected,cancelled);}
    if cancelled.load(Ordering::Acquire){return Err("WAV preparation cancelled".into());}
    if !matches!(selected.selected.kind.as_str(),"audio/wav"|"audio/x-wav"|"audio/wave") {return Err("Audio output profile supports PCM WAV only".into());}
    let bytes=&selected.selected.bytes;
    if bytes.is_empty()||bytes.len()>MAX_INPUT{return Err("WAV input exceeds encoded-byte profile".into());}
    let admitted=admit_wav(bytes,cancelled)?;
    let decoded=decode_wav(admitted).map_err(|e|format!("WAV decode refused: {e:?}"))?;
    if cancelled.load(Ordering::Acquire){return Err("WAV preparation cancelled".into());}
    let begin=exact_sample(selected.media_tick,decoded.sample_rate)?;
    let end_tick=selected.media_tick.checked_add(selected.remaining_ticks).ok_or("Clip time overflow")?;
    let end=exact_sample(end_tick,decoded.sample_rate)?;
    if end<=begin || end>u64::from(decoded.frames) {return Err("Clip extends outside decoded WAV".into());}
    let channels=usize::from(decoded.channels); let frame_count=usize::try_from(end-begin).map_err(|_|"Clip frame count overflow")?;
    let samples=frame_count.checked_mul(channels).ok_or("Audio sample count overflow")?;
    let bps=match decoded.format {SampleFormat::I16=>2,SampleFormat::I24Packed=>3,SampleFormat::I32|SampleFormat::F32=>4};
    let sample_bytes=samples.checked_mul(4).ok_or("Audio output size overflow")?;
    let logical=bytes.len().checked_add(decoded.pcm.len()).and_then(|n|n.checked_add(sample_bytes)).ok_or("Audio working size overflow")?;
    if logical>MAX_PAYLOAD {return Err("WAV output exceeds bounded logical payload".into());}
    let offset=usize::try_from(begin).map_err(|_|"Audio offset overflow")?.checked_mul(channels).and_then(|n|n.checked_mul(bps)).ok_or("Audio offset overflow")?;
    let len=samples.checked_mul(bps).ok_or("Audio extent overflow")?;
    let source=decoded.pcm.get(offset..offset.checked_add(len).ok_or("Audio extent overflow")?).ok_or("Decoded PCM is shorter than the clip")?;
    let mut output=Vec::new();output.try_reserve_exact(samples).map_err(|_|"Audio sample allocation refused")?;
    for (index,chunk) in source.chunks_exact(bps).enumerate(){
        if index%8192==0 && cancelled.load(Ordering::Acquire){return Err("WAV preparation cancelled during conversion".into());}
        let sample=match decoded.format {
            SampleFormat::I16=>f32::from(i16::from_le_bytes([chunk[0],chunk[1]]))/32768.0,
            SampleFormat::I24Packed=>((i32::from(chunk[0])) | (i32::from(chunk[1])<<8) | (i32::from(chunk[2] as i8)<<16)) as f32 / 8388608.0,
            SampleFormat::I32=>(i32::from_le_bytes([chunk[0],chunk[1],chunk[2],chunk[3]]) as f64/2147483648.0) as f32,
            SampleFormat::F32=>f32::from_le_bytes([chunk[0],chunk[1],chunk[2],chunk[3]]),
        };
        if !sample.is_finite() || !(-1.0..=1.0).contains(&sample){return Err("WAV sample outside finite unit range".into());}
        output.push(sample);
    }
    if cancelled.load(Ordering::Acquire){return Err("WAV preparation cancelled; result withheld".into());}
    Ok(PreparedWav {rate:decoded.sample_rate,channels:decoded.channels,source_start_frame:begin,samples:output})
}

/// Admit a small nonfragmented MP4 and verify the AAC media timeline before
/// passing access units to the existing AAC codec. A single normal-rate edit
/// can map presentation samples onto decoded samples, trimming signalled
/// encoder delay and padding. Unsignalled priming remains unknown.
fn prepare_mp4_aac(selected:&TimelineMediaSelection,cancelled:&AtomicBool)->Result<PreparedWav,String>{
    use media_isobmff::{CodecConfig,DemuxLimits,TrackKind,open_with_limits};
    use audio_aac_codec::{AudioSpecificConfig,Decoder,FRAME_LEN};
    const MAX_INPUT:usize=2*1024*1024;
    const MAX_PACKETS:usize=2048;
    let bytes=&selected.selected.bytes;
    if bytes.is_empty()||bytes.len()>MAX_INPUT{return Err("MP4 AAC input exceeds 2 MiB profile".into());}
    if selected.remaining_ticks==0 || selected.remaining_ticks>(timeline_editing::TICKS_PER_SECOND as u64)*15{return Err("MP4 AAC clip remainder exceeds 15 seconds".into());}
    let mut pos=0usize;let mut count=0usize;let mut ftyp=0;let mut moov=0;let mut mdat=None;
    while pos<bytes.len(){
        if cancelled.load(Ordering::Acquire){return Err("MP4 AAC preparation cancelled".into());}
        count+=1;if count>256||bytes.len()-pos<8{return Err("MP4 top-level box count or framing exceeds profile".into());}
        let size=u32::from_be_bytes(bytes[pos..pos+4].try_into().map_err(|_|"MP4 header")?) as usize;
        let end=pos.checked_add(size).filter(|end|size>=8&&*end<=bytes.len()).ok_or("Invalid MP4 top-level extent")?;
        match &bytes[pos+4..pos+8] {b"ftyp"=>ftyp+=1,b"moov"=>moov+=1,b"mdat"=>{if mdat.is_some(){return Err("Multiple MP4 media-data boxes are not admitted".into());}mdat=Some((pos+8,end));},b"free"|b"skip"|b"wide"=>{},_=>return Err("Unsupported MP4 top-level box".into())}
        pos=end;
    }
    let mdat=mdat.ok_or("MP4 media data missing")?;
    if ftyp!=1||moov!=1{return Err("MP4 requires one file type and one movie header".into());}
    let limits=DemuxLimits{max_input_bytes:MAX_INPUT as u64,max_header_bytes:MAX_INPUT as u64,max_tracks:2,max_fragments:0,max_table_entries:MAX_PACKETS,max_samples_per_track:MAX_PACKETS,reject_audio_composition_offsets:true,require_self_contained_data:true};
    let file=std::panic::catch_unwind(std::panic::AssertUnwindSafe(||open_with_limits(bytes.as_slice(),limits)))
        .map_err(|_|"MP4 demux rejected malformed input")?.map_err(|e|format!("MP4 demux refused: {e}"))?;
    if file.fragmented{return Err("Fragmented MP4 is not admitted".into());}
    let audio:Vec<_>=file.tracks.iter().enumerate().filter(|(_,track)|track.kind==TrackKind::Audio).collect();
    if audio.len()!=1{return Err("MP4 requires exactly one audio track".into());}
    let (track_index,track)=audio[0];
    if track.entries.len()!=1||track.samples.is_empty()||track.samples.len()>MAX_PACKETS{return Err("MP4 audio sample entry or count is outside profile".into());}
    let aac=match track.codec(){Some(CodecConfig::Aac(config))=>config,_=>return Err("MP4 audio track is not AAC".into())};
    if aac.asc.len()!=2 || aac.asc[1]&7!=0{return Err("MP4 AAC configuration or extensions are outside profile".into());}
    let config=AudioSpecificConfig::parse(&aac.asc).map_err(|e|format!("MP4 AAC configuration refused: {e}"))?;
    if config.object_type!=2||config.sbr||config.ps||config.frame_length_flag||!(1..=2).contains(&config.channel_config)||config.sample_rate==0||config.sample_rate>96_000||track.timescale!=config.sample_rate{return Err("MP4 audio requires sample-rate-timed mono/stereo AAC-LC without explicit SBR".into());}
    let rate=config.sample_rate;let channels=config.channel_config;
    let presentation_begin=exact_sample(selected.media_tick,rate)?;
    let end_tick=selected.media_tick.checked_add(selected.remaining_ticks).ok_or("MP4 AAC time overflow")?;
    let presentation_end=exact_sample(end_tick,rate)?;
    let total=(track.samples.len() as u64).checked_mul(FRAME_LEN as u64).ok_or("MP4 AAC samples overflow")?;
    let (edit_start,presentation_length)=mp4_audio_edit_window(&track.edits,file.timescale,rate,total)?;
    if presentation_end<=presentation_begin||presentation_end>presentation_length{return Err("MP4 audio clip exceeds admitted presentation samples".into());}
    let begin=edit_start.checked_add(presentation_begin).ok_or("MP4 AAC edit start overflow")?;
    let end=edit_start.checked_add(presentation_end).ok_or("MP4 AAC edit end overflow")?;
    let output_samples=usize::try_from(end-begin).ok().and_then(|n|n.checked_mul(usize::from(channels))).ok_or("MP4 AAC output overflow")?;
    if output_samples.checked_mul(4).is_none_or(|n|n>16*1024*1024){return Err("MP4 AAC PCM exceeds 16 MiB".into());}
    let mut output=Vec::new();output.try_reserve_exact(output_samples).map_err(|_|"MP4 AAC output allocation refused")?;
    let mut decoder=Decoder::new(&aac.asc).map_err(|e|format!("MP4 AAC decoder refused: {e}"))?;
    let mut previous_end=mdat.0 as u64;
    for (index,sample) in track.samples.iter().enumerate(){
        if cancelled.load(Ordering::Acquire){return Err("MP4 AAC preparation cancelled".into());}
        let expected=(index as u64).checked_mul(FRAME_LEN as u64).ok_or("MP4 sample time overflow")?;
        let sample_end=sample.offset.checked_add(u64::from(sample.size)).ok_or("MP4 sample extent overflow")?;
        if sample.dts!=expected as i64||sample.pts!=sample.dts||sample.duration!=FRAME_LEN as u32||sample.description_index!=1||sample.size==0||sample.size>8192||sample.offset<previous_end||sample.offset<mdat.0 as u64||sample_end>mdat.1 as u64{return Err("MP4 AAC sample layout or timeline is outside profile".into());}
        previous_end=sample_end;
        let access_unit=file.read_sample(bytes.as_slice(),track_index,index).map_err(|e|format!("MP4 sample refused: {e}"))?;
        let decoded=std::panic::catch_unwind(std::panic::AssertUnwindSafe(||decoder.decode(&access_unit)))
            .map_err(|_|"MP4 AAC codec rejected malformed input")?.map_err(|e|format!("MP4 AAC decode refused: {e}"))?;
        if decoder.sbr()||decoded.len()!=usize::from(channels)||decoded.iter().any(|c|c.len()!=FRAME_LEN){return Err("MP4 AAC layout changed or detected unqualified SBR".into());}
        let block_end=expected+FRAME_LEN as u64;let from=begin.max(expected);let until=end.min(block_end);
        if from<until {for frame in (from-expected) as usize..(until-expected) as usize {for channel in &decoded {
            let value=channel[frame];if !value.is_finite()||!(-1.0..=1.0).contains(&value){return Err("MP4 AAC sample outside finite unit range".into());}output.push(value);
        }}}
        if block_end>=end {break;}
    }
    if output.len()!=output_samples||cancelled.load(Ordering::Acquire){return Err("MP4 AAC sample count changed or preparation cancelled".into());}
    Ok(PreparedWav{rate,channels:u16::from(channels),source_start_frame:begin,samples:output})
}

/// Resolve only one non-empty unit-rate edit. Movie duration conversion must
/// land on a sample boundary; no rounding or inference of encoder delay.
fn mp4_audio_edit_window(edits:&[media_isobmff::Edit],movie_rate:u32,rate:u32,total:u64)->Result<(u64,u64),String>{
    if edits.is_empty(){return Ok((0,total));}
    if edits.len()!=1||movie_rate==0||rate==0{return Err("MP4 audio edit list outside supported profile".into());}
    let edit=edits[0];
    if edit.media_time<0||edit.media_rate!=0x10000||edit.segment_duration==0{return Err("MP4 audio requires one non-empty normal-rate edit".into());}
    let scaled=u128::from(edit.segment_duration)*u128::from(rate);
    if scaled%u128::from(movie_rate)!=0{return Err("MP4 audio edit duration does not align to a sample".into());}
    let length=u64::try_from(scaled/u128::from(movie_rate)).map_err(|_|"MP4 audio edit duration overflow")?;
    let start=edit.media_time as u64;
    if length==0||start.checked_add(length).is_none_or(|end|end>total){return Err("MP4 audio edit exceeds decoded samples".into());}
    Ok((start,length))
}

/// Bounded AAC-LC ADTS decode through the existing FilmCraft codec. This is
/// decoded sample time, not a gapless/priming or MP4 edit-list claim.
fn prepare_adts(selected:&TimelineMediaSelection,cancelled:&AtomicBool)->Result<PreparedWav,String>{
    use audio_aac_codec::{AdtsHeader,Decoder,FRAME_LEN,SAMPLE_RATES};
    const MAX_ADTS_BYTES:usize=2*1024*1024;
    const MAX_ADTS_FRAMES:usize=2048;
    let bytes=&selected.selected.bytes;
    if bytes.is_empty()||bytes.len()>MAX_ADTS_BYTES{return Err("AAC ADTS input exceeds 2 MiB profile".into());}
    if selected.remaining_ticks==0 || selected.remaining_ticks>(timeline_editing::TICKS_PER_SECOND as u64)*15{return Err("AAC clip remainder exceeds the 15-second profile".into());}
    let mut packets=Vec::new();let mut position=0usize;let mut baseline=None;
    while position<bytes.len(){
        if cancelled.load(Ordering::Acquire){return Err("AAC preparation cancelled".into());}
        if packets.len()>=MAX_ADTS_FRAMES{return Err("AAC access-unit count exceeds profile".into());}
        let header=AdtsHeader::parse(&bytes[position..]).map_err(|e|format!("AAC ADTS header refused: {e}"))?;
        if header.object_type!=2 || header.header_length!=7 || header.raw_data_blocks!=1 || !(1..=2).contains(&header.channel_config) {return Err("AAC profile requires unprotected single-block mono/stereo AAC-LC".into());}
        let signature=(header.sf_index,header.channel_config);
        if baseline.is_some_and(|v|v!=signature){return Err("AAC ADTS configuration changes within clip".into());}
        baseline=Some(signature);
        let end=position.checked_add(header.frame_length).filter(|end|*end<=bytes.len()).ok_or("Truncated AAC ADTS frame")?;
        let start=position.checked_add(header.header_length).filter(|start|*start<end).ok_or("Empty AAC access unit")?;
        packets.try_reserve(1).map_err(|_|"AAC packet index allocation refused")?;
        packets.push((header,start,end));position=end;
    }
    let (sf_index,channels)=baseline.ok_or("No AAC access units")?;
    let rate=*SAMPLE_RATES.get(usize::from(sf_index)).ok_or("AAC sample rate is unsupported")?;
    if rate==0||rate>96_000{return Err("AAC sample rate exceeds profile".into());}
    let begin=exact_sample(selected.media_tick,rate)?;
    let end_tick=selected.media_tick.checked_add(selected.remaining_ticks).ok_or("AAC clip time overflow")?;
    let end=exact_sample(end_tick,rate)?;
    let total=(packets.len() as u64).checked_mul(FRAME_LEN as u64).ok_or("AAC sample count overflow")?;
    if end<=begin||end>total{return Err("AAC clip extends outside admitted decoded samples".into());}
    let sample_count=usize::try_from(end-begin).ok().and_then(|n|n.checked_mul(usize::from(channels))).ok_or("AAC output size overflow")?;
    if sample_count.checked_mul(4).is_none_or(|n|n>16*1024*1024){return Err("AAC output exceeds 16 MiB PCM profile".into());}
    let mut out=Vec::new();out.try_reserve_exact(sample_count).map_err(|_|"AAC PCM allocation refused")?;
    let mut decoder=Decoder::from_adts(&packets[0].0).map_err(|e|format!("AAC configuration refused: {e}"))?;
    for (index,(_,start,stop)) in packets.iter().enumerate(){
        if cancelled.load(Ordering::Acquire){return Err("AAC preparation cancelled".into());}
        let decoded=std::panic::catch_unwind(std::panic::AssertUnwindSafe(||decoder.decode(&bytes[*start..*stop])))
            .map_err(|_|"AAC codec rejected malformed input")?.map_err(|e|format!("AAC decode refused: {e}"))?;
        if decoder.sbr() || decoded.len()!=usize::from(channels) || decoded.iter().any(|c|c.len()!=FRAME_LEN){return Err("AAC decode changed channel layout or requires unqualified SBR".into());}
        let block_start=(index as u64)*(FRAME_LEN as u64);let block_end=block_start+(FRAME_LEN as u64);
        let from=begin.max(block_start);let until=end.min(block_end);
        if from<until {for frame in (from-block_start) as usize..(until-block_start) as usize {for channel in &decoded {
            let value=channel[frame];if !value.is_finite()||!(-1.0..=1.0).contains(&value){return Err("AAC sample outside finite unit range".into());}out.push(value);
        }}}
        if block_end>=end {break;}
    }
    if out.len()!=sample_count||cancelled.load(Ordering::Acquire){return Err("AAC sample count changed or preparation cancelled".into());}
    Ok(PreparedWav{rate,channels:u16::from(channels),source_start_frame:begin,samples:out})
}

#[cfg(test)]
mod tests {
    use super::*;use crate::graphical::checkpoints::SelectedMedia;
    use qualia_audio::wav::encode_wav_i16_mono;
    fn sample()->TimelineMediaSelection {let mut bytes=vec![0;128];let n=encode_wav_i16_mono(&[0,16384,-16384,32767],8,&mut bytes).unwrap();bytes.truncate(n);
        TimelineMediaSelection{clip_source_identifier:"fixture".into(),media_tick:(timeline_editing::TICKS_PER_SECOND/8) as u64,remaining_ticks:(timeline_editing::TICKS_PER_SECOND/4) as u64,selected:SelectedMedia{kind:"audio/wav".into(),bytes}}}
    #[test]
    fn aac_lc_adts_decode_uses_bounded_exact_sample_window(){
        use audio_aac_codec::{AdtsHeader,Encoder,EncoderConfig,FRAME_LEN,sample_rate_index};
        let rate=48_000;let mut encoder=Encoder::new(EncoderConfig::cbr(rate,1,64_000)).unwrap();
        let input=vec![0.0_f32;FRAME_LEN*4];let mut units=encoder.encode(&[&input]);units.extend(encoder.flush());assert!(!units.is_empty());
        let sf=sample_rate_index(rate).unwrap();let mut bytes=Vec::new();
        for unit in units {bytes.extend_from_slice(&AdtsHeader::write(2,sf,1,unit.len(),0x7ff));bytes.extend_from_slice(&unit);}
        let mut selected=TimelineMediaSelection{clip_source_identifier:"aac-fixture".into(),media_tick:0,
            remaining_ticks:(timeline_editing::TICKS_PER_SECOND as u64/48_000)*FRAME_LEN as u64,
            selected:SelectedMedia{kind:"audio/aac".into(),bytes}};
        let pcm=prepare(&selected,&AtomicBool::new(false)).unwrap();assert_eq!((pcm.rate,pcm.channels,pcm.frames()),(rate,1,FRAME_LEN));
        assert!(prepare(&selected,&AtomicBool::new(true)).is_err());

        selected.selected.bytes.pop();assert!(prepare(&selected,&AtomicBool::new(false)).is_err());
    }
    #[test]
    fn mp4_aac_lc_decodes_exact_samples_and_refuses_oversized_or_truncated_input(){
        use audio_aac_codec::{Encoder,EncoderConfig,FRAME_LEN};
        use media_isobmff::{Brand,Mp4Writer,SampleEntry,TrackConfig,WriteSample,WriterOptions};
        let rate=48_000;let mut encoder=Encoder::new(EncoderConfig::cbr(rate,1,64_000)).unwrap();
        let input:Vec<f32>=(0..FRAME_LEN*4).map(|i| ((i as f32)*0.031).sin()*0.35).collect();
        let mut units=encoder.encode(&[&input]);units.extend(encoder.flush());assert!(units.len()>=4);
        let mut writer=Mp4Writer::new(std::io::Cursor::new(Vec::new()),WriterOptions::new(Brand::Mp4)).unwrap();
        let track=writer.add_track(TrackConfig::new(SampleEntry::aac(encoder.audio_specific_config(),1,rate),rate)).unwrap();
        for unit in &units {writer.write_sample(track,WriteSample{data:unit,duration:FRAME_LEN as u32,composition_offset:0,is_sync:true}).unwrap();}
        let mut selected=TimelineMediaSelection{clip_source_identifier:"mp4-aac-fixture".into(),media_tick:0,
            remaining_ticks:(timeline_editing::TICKS_PER_SECOND as u64/48_000)*FRAME_LEN as u64,
            selected:SelectedMedia{kind:"audio/mp4".into(),bytes:writer.finish().unwrap().into_inner()}};
        let pcm=prepare(&selected,&AtomicBool::new(false)).unwrap();assert_eq!((pcm.rate,pcm.channels,pcm.frames()),(rate,1,FRAME_LEN));
        assert!(prepare(&selected,&AtomicBool::new(true)).is_err());
        let valid=selected.selected.bytes.clone();
        let url=valid.windows(4).position(|w|w==b"url ").unwrap();
        selected.selected.bytes[url+4..url+8].copy_from_slice(&0u32.to_be_bytes());
        assert!(prepare(&selected,&AtomicBool::new(false)).is_err(),"external data reference must not read local mdat");
        selected.selected.bytes=valid.clone();
        let entry=valid.windows(4).position(|w|w==b"mp4a").unwrap();
        selected.selected.bytes[entry+10..entry+12].copy_from_slice(&2u16.to_be_bytes());
        assert!(prepare(&selected,&AtomicBool::new(false)).is_err(),"out-of-range sample data reference must be refused");
        selected.selected.bytes=valid.clone();
        let dref=valid.windows(4).position(|w|w==b"dref").unwrap();
        selected.selected.bytes[dref..dref+4].copy_from_slice(b"skip");
        assert!(prepare(&selected,&AtomicBool::new(false)).is_err(),"missing data reference must be refused");
        selected.selected.bytes=valid;
        let mut opts=WriterOptions::new(Brand::Mp4);opts.movie_timescale=rate;
        let mut edited=Mp4Writer::new(std::io::Cursor::new(Vec::new()),opts).unwrap();
        let mut cfg=TrackConfig::new(SampleEntry::aac(encoder.audio_specific_config(),1,rate),rate);
        // Deliberately non-frame-aligned priming and a tail trim, leaving
        // part of the coded media unpresented at the end.
        let edit_start=FRAME_LEN as u64+7;
        cfg.edits.push(media_isobmff::Edit{segment_duration:(FRAME_LEN*2) as u64,media_time:edit_start as i64,media_rate:0x10000});
        let edited_track=edited.add_track(cfg).unwrap();
        for unit in &units {edited.write_sample(edited_track,WriteSample{data:unit,duration:FRAME_LEN as u32,composition_offset:0,is_sync:true}).unwrap();}
        let tick_per_sample=timeline_editing::TICKS_PER_SECOND as u64/u64::from(rate);
        let edited_selection=TimelineMediaSelection{clip_source_identifier:"mp4-edit".into(),media_tick:37*tick_per_sample,
            remaining_ticks:selected.remaining_ticks,selected:SelectedMedia{kind:"audio/mp4".into(),bytes:edited.finish().unwrap().into_inner()}};
        let edited_pcm=prepare(&edited_selection,&AtomicBool::new(false)).unwrap();
        selected.media_tick=(edit_start+37)*tick_per_sample;
        let raw_pcm=prepare(&selected,&AtomicBool::new(false)).unwrap();
        assert_eq!(edited_pcm.samples,raw_pcm.samples);
        assert_eq!(edited_pcm.source_start_frame,edit_start+37);
        assert!(edited_pcm.samples.iter().any(|sample|sample.abs()>0.00001));
        let mut beyond_edit=edited_selection;
        beyond_edit.media_tick=FRAME_LEN as u64*tick_per_sample+38*tick_per_sample;
        assert!(prepare(&beyond_edit,&AtomicBool::new(false)).is_err());
        selected.media_tick=0;
        selected.selected.bytes.truncate(20);assert!(prepare(&selected,&AtomicBool::new(false)).is_err());
    }
    #[test] fn audio_edit_window_checks_priming_padding_and_sample_alignment(){
        use media_isobmff::Edit;
        let e=Edit{segment_duration:1500,media_time:1024,media_rate:0x10000};
        assert_eq!(mp4_audio_edit_window(&[e],1000,48_000,100_000).unwrap(),(1024,72_000));
        assert!(mp4_audio_edit_window(&[e],1001,48_000,100_000).is_err());
        assert!(mp4_audio_edit_window(&[e],1000,48_000,73_000).is_err());
        assert!(mp4_audio_edit_window(&[Edit{media_rate:0,..e}],1000,48_000,100_000).is_err());
        assert!(mp4_audio_edit_window(&[Edit{media_time:-1,..e}],1000,48_000,100_000).is_err());
        assert!(mp4_audio_edit_window(&[e,e],1000,48_000,200_000).is_err());
        assert_eq!(mp4_audio_edit_window(&[],1000,48_000,123).unwrap(),(0,123));
    }
    #[test] fn packed_i24_stereo_exact_window_preserves_channels_and_sign(){
        let samples=[0i32,0,-8388608,4194304,8388607,-4194304];let data_len=(samples.len()*3) as u32;
        let mut bytes=Vec::new();bytes.extend_from_slice(b"RIFF");bytes.extend_from_slice(&(36+data_len).to_le_bytes());bytes.extend_from_slice(b"WAVEfmt ");
        bytes.extend_from_slice(&16u32.to_le_bytes());bytes.extend_from_slice(&1u16.to_le_bytes());bytes.extend_from_slice(&2u16.to_le_bytes());
        bytes.extend_from_slice(&8u32.to_le_bytes());bytes.extend_from_slice(&48u32.to_le_bytes());bytes.extend_from_slice(&6u16.to_le_bytes());bytes.extend_from_slice(&24u16.to_le_bytes());
        bytes.extend_from_slice(b"data");bytes.extend_from_slice(&data_len.to_le_bytes());for sample in samples {bytes.extend_from_slice(&sample.to_le_bytes()[..3]);}
        let mut selection=sample();selection.selected.bytes=bytes;
        let p=prepare(&selection,&AtomicBool::new(false)).unwrap();
        assert_eq!((p.rate,p.channels,p.source_start_frame,p.frames()),(8,2,1,2));
        assert_eq!(p.samples,vec![-1.0,0.5,8388607.0/8388608.0,-0.5]);
        selection.selected.bytes[32..34].copy_from_slice(&4u16.to_le_bytes());
        assert!(prepare(&selection,&AtomicBool::new(false)).is_err());
    }
    #[test] fn exact_window_preserves_samples_and_channel_count(){let p=prepare(&sample(),&AtomicBool::new(false)).unwrap();assert_eq!((p.rate,p.channels,p.source_start_frame,p.frames()),(8,1,1,2));assert_eq!(p.samples,vec![0.5,-0.5]);}
    #[test] fn two_mono_wavs_use_qualiadb_equal_mix_and_stop_at_common_interval(){
        let a=sample();let mut b=sample();b.remaining_ticks=(timeline_editing::TICKS_PER_SECOND/8) as u64;
        let mixed=mix_two_mono(a,b,&AtomicBool::new(false)).unwrap();
        assert_eq!((mixed.rate,mixed.channels,mixed.frames()),(8,2,1));
        assert_eq!(mixed.samples,vec![0.5,0.5]);
        assert!(mix_two_mono(sample(),sample(),&AtomicBool::new(true)).is_err());
        let mut not_wav=sample();not_wav.selected.kind="audio/aac".into();
        assert!(mix_two_mono(sample(),not_wav,&AtomicBool::new(false)).is_err());
    }
    #[test] fn refuses_partial_sample_out_of_bounds_malformed_and_cancelled(){let mut s=sample();s.media_tick+=1;assert!(prepare(&s,&AtomicBool::new(false)).is_err());let mut s=sample();s.remaining_ticks*=10;assert!(prepare(&s,&AtomicBool::new(false)).is_err());let mut s=sample();s.selected.bytes.truncate(9);assert!(prepare(&s,&AtomicBool::new(false)).is_err());assert!(prepare(&sample(),&AtomicBool::new(true)).is_err());}
    #[test] fn seek_requires_exact_sample_and_stays_in_clip(){let t=timeline_editing::TICKS_PER_SECOND as u64;assert_eq!(relative_seek_frame(100,100+t/8,8,2).unwrap(),1);assert!(relative_seek_frame(100,101,8,2).is_err());assert!(relative_seek_frame(100,100+t/4,8,2).is_err());assert!(relative_seek_frame(100,99,8,2).is_err());}
}
