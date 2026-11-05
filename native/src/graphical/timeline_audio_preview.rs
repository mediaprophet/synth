//! Bounded waveform statistics from an authorized source-associated WAV clip.
//! No samples are logged, persisted, or sent to a device by this module.
use crate::graphical::checkpoints::TimelineMediaSelection;
use qualia_audio::{convert::to_mono_f32, types::AudioView, wav::decode_wav};
use std::sync::atomic::{AtomicBool, Ordering};

const MAX_INPUT: usize = 16 * 1024 * 1024;
const MAX_CHANNELS: u16 = 8;
const MAX_RATE: u32 = 384_000;
const MAX_WINDOW_FRAMES: usize = 4096;
const MAX_PAYLOAD: usize = 40 * 1024 * 1024;
const MAX_RIFF_CHUNKS: usize = 65_536;

/// Statistics for a source-time window; mix is the existing unweighted mono
/// channel average. They are not calibrated loudness or evidence of playback.
pub(super) struct WavAnalysis {
    pub rate: u32, pub channels: u16, pub source_frames: u32,
    pub window_start: u64, pub window_frames: usize,
    pub minimum: f32, pub maximum: f32, pub rms: f64,
}

fn ceil_sample(ticks:u64, rate:u32) -> Result<u64,String> {
    let denom=timeline_editing::TICKS_PER_SECOND as u128;
    let n=(ticks as u128).checked_mul(rate as u128).ok_or("Audio time overflow")?;
    let q=n / denom; let extra=if n % denom == 0 {0} else {1};
    u64::try_from(q.checked_add(extra).ok_or("Audio time overflow")?).map_err(|_|"Audio sample index overflow".into())
}

// QualiaDB owns the decoder. This bounded admission validates RIFF container
// geometry and the PCM layout that that decoder does not itself cross-check.
// Unknown ancillary chunks are retained and ignored, not interpreted or removed.
pub(super) fn admit_wav<'a>(bytes:&'a [u8],cancelled:&AtomicBool)->Result<&'a [u8],String>{
    if bytes.len()<12 || &bytes[0..4]!=b"RIFF" || &bytes[8..12]!=b"WAVE" {return Err("Expected RIFF/WAVE".into());}
    let declared=u32::from_le_bytes(bytes[4..8].try_into().map_err(|_|"Invalid RIFF length")?) as usize;
    let end=declared.checked_add(8).ok_or("RIFF size overflow")?;
    if end<12 || end>bytes.len() {return Err("RIFF declared size exceeds available bytes".into());}
    let mut pos=12usize;let mut chunks=0usize;let mut format=None;let mut data_len=None;
    while pos<end {
        chunks+=1;if chunks>MAX_RIFF_CHUNKS{return Err("RIFF chunk count exceeds analysis profile".into());}
        if chunks%256==0 && cancelled.load(Ordering::Acquire){return Err("WAV analysis cancelled during admission".into());}
        let header_end=pos.checked_add(8).filter(|n|*n<=end).ok_or("Truncated RIFF chunk header")?;
        let kind=&bytes[pos..pos+4];let size=u32::from_le_bytes(bytes[pos+4..header_end].try_into().map_err(|_|"Invalid RIFF chunk size")?) as usize;
        let chunk_end=header_end.checked_add(size).filter(|n|*n<=end).ok_or("RIFF chunk exceeds declared container")?;
        let padded_end=chunk_end.checked_add(size%2).filter(|n|*n<=end).ok_or("Missing RIFF chunk alignment byte")?;
        if kind==b"fmt " {
            if format.is_some() || size<16 {return Err("WAV requires one supported format chunk".into());}
            let c=&bytes[header_end..chunk_end];
            let code=u16::from_le_bytes([c[0],c[1]]);let channels=u16::from_le_bytes([c[2],c[3]]);
            let rate=u32::from_le_bytes([c[4],c[5],c[6],c[7]]);let byte_rate=u32::from_le_bytes([c[8],c[9],c[10],c[11]]);
            let align=u16::from_le_bytes([c[12],c[13]]);let bits=u16::from_le_bytes([c[14],c[15]]);
            if !matches!((code,bits),(1,16)|(1,24)|(1,32)|(3,32)) || channels==0 || channels>MAX_CHANNELS || rate==0 || rate>MAX_RATE {
                return Err("WAV format exceeds supported PCM analysis profile".into());
            }
            let expected_align=channels.checked_mul(bits/8).ok_or("WAV block alignment overflow")?;
            if align!=expected_align || rate.checked_mul(u32::from(align))!=Some(byte_rate) {return Err("WAV block alignment or byte rate disagrees with format".into());}
            format=Some(align as usize);
        } else if kind==b"data" {
            if data_len.replace(size).is_some(){return Err("Multiple WAV data chunks require a qualified profile".into());}
        }
        pos=padded_end;
    }
    let align=format.ok_or("WAV format chunk missing")?;let data_len=data_len.ok_or("WAV data chunk missing")?;
    if data_len%align!=0 {return Err("WAV data does not contain complete sample frames".into());}
    Ok(&bytes[..end])
}

pub(super) fn analyze(selected:&TimelineMediaSelection,cancelled:&AtomicBool)->Result<WavAnalysis,String>{
    if cancelled.load(Ordering::Acquire){return Err("WAV analysis cancelled".into());}
    if !matches!(selected.selected.kind.as_str(),"audio/wav"|"audio/x-wav"|"audio/wave") {return Err("Analysis supports PCM WAV only".into());}
    let input=&selected.selected.bytes;
    if input.is_empty() || input.len()>MAX_INPUT {return Err("WAV input exceeds encoded-byte profile".into());}
    // Core's cold WAV decoder copies at most the supplied encoded bytes. Charge
    // input + decoded PCM + bounded mono window before calling it. This is a
    // logical payload bound, not a measurement of codec/internal heap overhead.
    let payload=input.len().checked_mul(2).and_then(|n|n.checked_add(MAX_WINDOW_FRAMES*4)).ok_or("WAV work-size overflow")?;
    if payload>MAX_PAYLOAD{return Err("WAV working-byte profile exceeded".into());}
    let admitted=admit_wav(input,cancelled)?;
    let decoded=decode_wav(admitted).map_err(|e|format!("WAV decode refused: {e:?}"))?;
    if cancelled.load(Ordering::Acquire){return Err("WAV analysis cancelled after decode".into());}
    if decoded.channels==0 || decoded.channels>MAX_CHANNELS || decoded.sample_rate==0 || decoded.sample_rate>MAX_RATE {return Err("WAV channel count or sample rate exceeds analysis profile".into());}
    let original=decoded.view();
    if !original.is_well_formed(){return Err("WAV sample view is malformed".into());}
    let stride=original.frame_stride_bytes as usize;
    let expected=(decoded.frames as usize).checked_mul(stride).ok_or("WAV sample size overflow")?;
    if stride==0 || expected!=decoded.pcm.len(){return Err("WAV data does not contain complete sample frames".into());}
    let end_tick=selected.media_tick.checked_add(selected.remaining_ticks).ok_or("Clip media range overflow")?;
    let begin=ceil_sample(selected.media_tick,decoded.sample_rate)?;
    let end=ceil_sample(end_tick,decoded.sample_rate)?.min(u64::from(decoded.frames));
    if begin>=end {return Err("No sample frame is available inside this clip at the selected tick".into());}
    let count=usize::try_from((end-begin).min(MAX_WINDOW_FRAMES as u64)).map_err(|_|"Audio window overflow")?;
    let first=usize::try_from(begin).map_err(|_|"Audio sample offset overflow")?.checked_mul(stride).ok_or("Audio sample offset overflow")?;
    let last=count.checked_mul(stride).and_then(|n|first.checked_add(n)).ok_or("Audio sample extent overflow")?;
    let bytes=decoded.pcm.get(first..last).ok_or("Audio sample extent exceeds decoded WAV")?;
    let view=AudioView {bytes,frames:count as u32,channels:decoded.channels,sample_rate:decoded.sample_rate,frame_stride_bytes:original.frame_stride_bytes,format:decoded.format};
    let mut mono=Vec::new();mono.try_reserve_exact(count).map_err(|_|"Audio window allocation refused")?;mono.resize(count,0.0);
    to_mono_f32(view,&mut mono).map_err(|e|format!("Audio channel conversion refused: {e:?}"))?;
    if cancelled.load(Ordering::Acquire){return Err("WAV analysis cancelled after conversion".into());}
    if mono.iter().any(|n|!n.is_finite()){return Err("WAV contains non-finite samples in the selected window".into());}
    let mut min=f32::INFINITY;let mut max=f32::NEG_INFINITY;let mut squares=0.0f64;
    for sample in mono {min=min.min(sample);max=max.max(sample);squares+=f64::from(sample)*f64::from(sample);}
    let rms=(squares/count as f64).sqrt();
    Ok(WavAnalysis {rate:decoded.sample_rate,channels:decoded.channels,source_frames:decoded.frames,window_start:begin,window_frames:count,minimum:min,maximum:max,rms})
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graphical::checkpoints::SelectedMedia;
    use qualia_audio::wav::encode_wav_i16_mono;
    fn sample() -> TimelineMediaSelection {
        let pcm=[0i16,16384,-16384,32767];let mut wav=vec![0;128];
        let size=encode_wav_i16_mono(&pcm,8,&mut wav).unwrap();wav.truncate(size);
        TimelineMediaSelection {clip_source_identifier:"urn:private".into(), media_tick:(timeline_editing::TICKS_PER_SECOND/8) as u64,
            remaining_ticks:(timeline_editing::TICKS_PER_SECOND/4) as u64,selected:SelectedMedia{kind:"audio/wav".into(),bytes:wav}}
    }
    fn stereo_32(code:u16,samples:&[u32])->TimelineMediaSelection {
        assert_eq!(samples.len()%2,0);
        let data_len=(samples.len()*4) as u32;
        let mut bytes=Vec::new();
        bytes.extend_from_slice(b"RIFF");bytes.extend_from_slice(&(36+data_len).to_le_bytes());bytes.extend_from_slice(b"WAVEfmt ");
        bytes.extend_from_slice(&16u32.to_le_bytes());bytes.extend_from_slice(&code.to_le_bytes());
        bytes.extend_from_slice(&2u16.to_le_bytes());bytes.extend_from_slice(&8u32.to_le_bytes());
        bytes.extend_from_slice(&64u32.to_le_bytes());bytes.extend_from_slice(&8u16.to_le_bytes());bytes.extend_from_slice(&32u16.to_le_bytes());
        bytes.extend_from_slice(b"data");bytes.extend_from_slice(&data_len.to_le_bytes());
        for sample in samples {bytes.extend_from_slice(&sample.to_le_bytes());}
        TimelineMediaSelection {clip_source_identifier:"urn:private".into(),media_tick:(timeline_editing::TICKS_PER_SECOND/8) as u64,
            remaining_ticks:(timeline_editing::TICKS_PER_SECOND/4) as u64,selected:SelectedMedia{kind:"audio/wav".into(),bytes}}
    }
    #[test] fn window_respects_source_offset_and_clip_end(){
        let result=analyze(&sample(),&AtomicBool::new(false)).unwrap();
        assert_eq!((result.rate,result.channels,result.source_frames,result.window_start,result.window_frames),(8,1,4,1,2));
        assert_eq!((result.minimum,result.maximum),(-0.5,0.5));assert!((result.rms-0.5).abs()<1e-9);
    }
    #[test] fn invalid_or_outside_wav_refuses(){
        let mut s=sample();s.remaining_ticks=0;assert!(analyze(&s,&AtomicBool::new(false)).is_err());
        let mut s=sample();s.selected.bytes.truncate(10);assert!(analyze(&s,&AtomicBool::new(false)).is_err());
        let mut s=sample();s.selected.kind="video/mp4".into();assert!(analyze(&s,&AtomicBool::new(false)).is_err());
        assert!(analyze(&sample(),&AtomicBool::new(true)).is_err());
    }
    #[test] fn malformed_riff_length_and_pcm_layout_refuse_before_decode(){
        let mut s=sample();s.selected.bytes[4..8].copy_from_slice(&u32::MAX.to_le_bytes());
        assert!(analyze(&s,&AtomicBool::new(false)).is_err());
        let mut s=sample();s.selected.bytes[28..32].copy_from_slice(&0u32.to_le_bytes());
        assert!(analyze(&s,&AtomicBool::new(false)).is_err());
        let mut s=sample();s.selected.bytes[32..34].copy_from_slice(&1u16.to_le_bytes());
        assert!(analyze(&s,&AtomicBool::new(false)).is_err());
        // Bytes beyond the declared RIFF are not presented to the decoder.
        let mut s=sample();s.selected.bytes.extend_from_slice(b"trailing");
        assert!(analyze(&s,&AtomicBool::new(false)).is_ok());
    }
    #[test] fn packed_i24_stereo_analysis_uses_signed_channel_average(){
        let samples=[0i32,0,-8388608,4194304,8388607,-4194304];
        let data_len=(samples.len()*3) as u32;
        let mut bytes=Vec::new();
        bytes.extend_from_slice(b"RIFF");bytes.extend_from_slice(&(36+data_len).to_le_bytes());bytes.extend_from_slice(b"WAVEfmt ");
        bytes.extend_from_slice(&16u32.to_le_bytes());bytes.extend_from_slice(&1u16.to_le_bytes());bytes.extend_from_slice(&2u16.to_le_bytes());
        bytes.extend_from_slice(&8u32.to_le_bytes());bytes.extend_from_slice(&48u32.to_le_bytes());bytes.extend_from_slice(&6u16.to_le_bytes());bytes.extend_from_slice(&24u16.to_le_bytes());
        bytes.extend_from_slice(b"data");bytes.extend_from_slice(&data_len.to_le_bytes());
        for sample in samples {bytes.extend_from_slice(&sample.to_le_bytes()[..3]);}
        let s=TimelineMediaSelection{clip_source_identifier:"fixture-i24".into(),media_tick:(timeline_editing::TICKS_PER_SECOND/8) as u64,
            remaining_ticks:(timeline_editing::TICKS_PER_SECOND/4) as u64,selected:SelectedMedia{kind:"audio/wav".into(),bytes}};
        let r=analyze(&s,&AtomicBool::new(false)).unwrap();
        assert_eq!((r.channels,r.rate,r.window_start,r.window_frames),(2,8,1,2));
        assert_eq!(r.minimum,-0.25);assert!((r.maximum-0.25).abs()<1e-6);
        let mut bad=s;bad.selected.bytes[32..34].copy_from_slice(&4u16.to_le_bytes());
        assert!(analyze(&bad,&AtomicBool::new(false)).is_err());
    }
    #[test] fn stereo_i32_window_uses_existing_channel_average(){
        let s=stereo_32(1,&[0,0,1_073_741_824i32 as u32,(-1_073_741_824i32) as u32,i32::MAX as u32,0]);
        let r=analyze(&s,&AtomicBool::new(false)).unwrap();
        assert_eq!((r.channels,r.rate,r.window_start,r.window_frames),(2,8,1,2));
        assert_eq!(r.minimum,0.0);assert!((r.maximum-0.5).abs()<1e-6);
        assert!((r.rms-(0.125f64).sqrt()).abs()<1e-6);
    }
    #[test] fn stereo_f32_window_refuses_nonfinite_selected_samples(){
        let bits=|x:f32|x.to_bits();
        let s=stereo_32(3,&[bits(f32::NAN),bits(0.0),bits(0.75),bits(0.25),bits(-0.75),bits(-0.25)]);
        let r=analyze(&s,&AtomicBool::new(false)).unwrap();
        assert_eq!((r.channels,r.window_start,r.window_frames),(2,1,2));
        assert_eq!((r.minimum,r.maximum),(-0.5,0.5));assert!((r.rms-0.5).abs()<1e-9);
        // A non-finite value outside the selected interval did not taint it;
        // moving that value into the active window is refused.
        let bad=stereo_32(3,&[bits(0.0),bits(0.0),bits(f32::NAN),bits(0.25),bits(-0.75),bits(-0.25)]);
        assert!(analyze(&bad,&AtomicBool::new(false)).is_err());
    }
}
