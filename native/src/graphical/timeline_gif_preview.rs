//! Authorized, bounded animated-GIF frame sampling for native timeline preview.
//! Uses the same GIF codec already used in FilmCraft's media source; does not
//! create a separate application, resource format, playback clock, or store.
use super::checkpoints::SelectedMedia;
use super::timeline_image_preview::PaddedImage;
use image::{AnimationDecoder, ImageDecoder};
use std::io::Cursor;
use std::sync::atomic::{AtomicBool, Ordering};

const MAX_INPUT: usize = 8 * 1024 * 1024;
const MAX_SIDE: u32 = 1024;
const MAX_FRAMES: usize = 256;
const MAX_FRAME_PIXELS: usize = 16 * 1024 * 1024;
const MAX_CODEC_ALLOCATION: u64 = 24 * 1024 * 1024;
const MAX_PLAY_FRAMES: usize = 32;
const MAX_PLAY_URI_BYTES: usize = 8 * 1024 * 1024;
const MAX_PLAY_SECONDS: u64 = 15;

/// Encoded display frames for a single admitted playback session, never a
/// persisted resource. Frame durations use exact editorial ticks.
pub(super) struct GifPlayFrame { pub uri: String, pub duration_ticks: u64 }

static MP4_STREAM_ACTIVE:AtomicBool=AtomicBool::new(false);
struct StreamLease;
impl Drop for StreamLease {fn drop(&mut self){MP4_STREAM_ACTIVE.store(false,Ordering::Release);}}
pub(super) struct Mp4Stream {
    pub frames:std::sync::mpsc::Receiver<Result<GifPlayFrame,String>>,
    cancelled:std::sync::Arc<AtomicBool>,
}
impl Drop for Mp4Stream {fn drop(&mut self){self.cancelled.store(true,Ordering::Release);}}
fn send_mp4_item<T>(tx:&std::sync::mpsc::SyncSender<T>,mut item:T,token:&AtomicBool,current:impl Fn()->bool)->Result<(),String>{loop{
    if token.load(Ordering::Acquire)||!current(){return Err("MP4 source changed or playback cancelled".into());}
    match tx.try_send(item){Ok(())=>return Ok(()),Err(std::sync::mpsc::TrySendError::Full(next))=>{item=next;std::thread::sleep(std::time::Duration::from_millis(10));},Err(std::sync::mpsc::TrySendError::Disconnected(_))=>return Err("MP4 consumer closed".into())}
}}
#[cfg(test)] mod mp4_stream_queue_tests {
    use super::*;
    #[test] fn full_queue_is_released_by_cancellation_and_dropped_receiver(){
        let (tx,rx)=std::sync::mpsc::sync_channel(1);tx.send(1).unwrap();
        let token=std::sync::Arc::new(AtomicBool::new(false));let copy=token.clone();let worker=std::thread::spawn(move||send_mp4_item(&tx,2,&copy,||true));
        token.store(true,Ordering::Release);assert!(worker.join().unwrap().is_err());drop(rx);
        let (tx,rx)=std::sync::mpsc::sync_channel::<u8>(1);drop(rx);assert!(send_mp4_item(&tx,1,&AtomicBool::new(false),||true).is_err());
    }
}

/// One bounded producer retains the already-authorized immutable HMC bytes.
/// Dropping the receiver or cancelling unblocks backpressure within 10ms.
pub(super) fn start_mp4_stream(selected:super::checkpoints::TimelineMediaSelection,snapshot:super::controller::EditorSnapshot,cancelled:std::sync::Arc<AtomicBool>)->Result<Mp4Stream,String>{
    if selected.selected.kind!="video/mp4"||cancelled.load(Ordering::Acquire)||!snapshot.publication_is_current(){return Err("MP4 playback cancelled or source changed".into());}
    if MP4_STREAM_ACTIVE.compare_exchange(false,true,Ordering::AcqRel,Ordering::Acquire).is_err(){return Err("A video stream is already active or shutting down".into());}
    let lease=StreamLease;let (tx,rx)=std::sync::mpsc::sync_channel(2);let token=cancelled.clone();
    std::thread::Builder::new().name("poet-mp4-preview".into()).spawn(move||{
        let _lease=lease;
        let send=|item:Result<GifPlayFrame,String>|send_mp4_item(&tx,item,&token,||snapshot.publication_is_current());
        let result=video_h264_preview::decode_mp4_stream(&selected.selected.bytes,selected.media_tick,selected.remaining_ticks,&token,|frame|{
            if token.load(Ordering::Acquire)||!snapshot.publication_is_current(){return Err("MP4 source changed or playback cancelled".into());}
            let side=frame.width.max(frame.height).max(8);let rgba=super::timeline_image_preview::pad(frame.width,frame.height,&frame.rgba,side)?;
            let view=super::preview::PreviewFrame{source:snapshot.source.clone(),session_revision:Some(snapshot.session_revision),execution_epoch:Some(snapshot.execution_epoch.clone()),side,rgba8:rgba};
            let uri=super::preview::native_image_uri(&view)?;if uri.len()>MAX_PLAY_URI_BYTES{return Err("Video frame exceeds encoded byte budget".into());}
            send(Ok(GifPlayFrame{uri,duration_ticks:frame.duration_ticks}))
        });
        if let Err(error)=result{let _=send(Err(error));}
    }).map_err(|_|"Cannot start video stream worker")?;
    Ok(Mp4Stream{frames:rx,cancelled})
}

/// Silent MP4 frames use the same bounded native presentation queue as GIF.
/// Container/digest/actor admission remains with the existing HMC owner.
pub(super) fn prepare_mp4(selected:&super::checkpoints::TimelineMediaSelection,snapshot:&super::controller::EditorSnapshot,cancelled:&AtomicBool)->Result<Vec<GifPlayFrame>,String>{
    if selected.selected.kind!="video/mp4" {return Err("Only selected MP4 video is admitted".into());}
    if cancelled.load(Ordering::Acquire)||!snapshot.publication_is_current(){return Err("Video playback cancelled or source changed".into());}
    let frames=video_h264_preview::decode_mp4(&selected.selected.bytes,selected.media_tick,selected.remaining_ticks,cancelled)?;
    let mut result=Vec::new();let mut uri_bytes=0usize;
    for frame in frames {
        if cancelled.load(Ordering::Acquire)||!snapshot.publication_is_current(){return Err("Video playback cancelled or source changed".into());}
        let side=frame.width.max(frame.height).max(8);
        let rgba=super::timeline_image_preview::pad(frame.width,frame.height,&frame.rgba,side)?;
        let view=super::preview::PreviewFrame{source:snapshot.source.clone(),session_revision:Some(snapshot.session_revision),execution_epoch:Some(snapshot.execution_epoch.clone()),side,rgba8:rgba};
        let uri=super::preview::native_image_uri(&view)?;
        uri_bytes=uri_bytes.checked_add(uri.len()).ok_or("Video preview size overflow")?;
        if uri_bytes>MAX_PLAY_URI_BYTES {return Err("Video encoded frame budget exceeded".into());}
        result.push(GifPlayFrame{uri,duration_ticks:frame.duration_ticks});
    }
    if result.is_empty(){return Err("No MP4 frames in selected range".into());}
    Ok(result)
}

pub(super) fn prepare_play(
    selected: &super::checkpoints::TimelineMediaSelection,
    snapshot: &super::controller::EditorSnapshot,
    cancelled: &AtomicBool,
) -> Result<Vec<GifPlayFrame>, String> {
    let bytes=&selected.selected.bytes;
    if selected.selected.kind!="image/gif" || bytes.len()>MAX_INPUT || bytes.len()<13 {return Err("GIF playback requires a bounded GIF resource".into());}
    if !matches!(&bytes[..6], b"GIF87a" | b"GIF89a") {return Err("Invalid GIF signature".into());}
    let width=u16::from_le_bytes([bytes[6],bytes[7]]) as u32; let height=u16::from_le_bytes([bytes[8],bytes[9]]) as u32;
    // Playback is intentionally narrower than still inspection to bound
    // retained decoded/encoded frames and UI property copies.
    if width==0||height==0||width>256||height>256 {return Err("GIF playback is limited to 256 × 256 pixels".into());}
    let rate=timeline_editing::TICKS_PER_SECOND as u64;
    if selected.remaining_ticks==0 || selected.remaining_ticks>rate*MAX_PLAY_SECONDS {return Err("GIF clip remainder exceeds the 15-second playback profile".into());}
    let start=selected.media_tick; let end=start.checked_add(selected.remaining_ticks).ok_or("GIF playback range overflow")?;
    preflight(bytes,width,height,cancelled)?;
    let mut decoder=image::codecs::gif::GifDecoder::new(Cursor::new(bytes)).map_err(|_|"Invalid GIF container")?;
    let mut limits=image::Limits::default(); limits.max_image_width=Some(256); limits.max_image_height=Some(256); limits.max_alloc=Some(MAX_CODEC_ALLOCATION);
    decoder.set_limits(limits).map_err(|_|"GIF decoder limits refused")?;
    let mut frames=decoder.into_frames(); let mut elapsed=0u64; let mut result=Vec::new(); let mut uri_bytes=0usize;
    for _ in 0..MAX_FRAMES {
        if cancelled.load(Ordering::Acquire) || !snapshot.publication_is_current() {return Err("GIF playback cancelled or source changed".into());}
        let Some(frame)=frames.next() else {break}; let frame=frame.map_err(|_|"GIF frame decode failed")?;
        let (num,den)=frame.delay().numer_denom_ms(); if num==0||den==0 {return Err("GIF frame has zero or invalid delay".into());}
        let product=u128::from(num)*u128::from(rate); let divisor=u128::from(den)*1000;
        if product%divisor!=0 {return Err("GIF delay is not exact in editorial ticks".into());}
        let duration=u64::try_from(product/divisor).map_err(|_|"GIF duration overflow")?;
        let next=elapsed.checked_add(duration).ok_or("GIF duration overflow")?;
        if next>start && elapsed<end {
            if result.len()>=MAX_PLAY_FRAMES {return Err("GIF playback frame count exceeds 32".into());}
            let effective=next.min(end)-elapsed.max(start); let rgba=frame.into_buffer().into_raw();
            let side=width.max(height).max(8);let rgba=super::timeline_image_preview::pad(width,height,&rgba,side)?;
            let frame=super::preview::PreviewFrame{source:snapshot.source.clone(),session_revision:Some(snapshot.session_revision),execution_epoch:Some(snapshot.execution_epoch.clone()),side,rgba8:rgba};
            let uri=super::preview::native_image_uri(&frame)?;
            uri_bytes=uri_bytes.checked_add(uri.len()).ok_or("GIF preview size overflow")?;
            if uri_bytes>MAX_PLAY_URI_BYTES {return Err("GIF playback encoded frame budget exceeded".into());}
            result.push(GifPlayFrame{uri,duration_ticks:effective});
        }
        elapsed=next;
        if elapsed>=end {break;}
    }
    if elapsed<end || result.is_empty() {return Err("Clip range exceeds the GIF's admitted single traversal".into());}
    if cancelled.load(Ordering::Acquire) || !snapshot.publication_is_current() {return Err("GIF playback cancelled or source changed".into());}
    Ok(result)
}

/// Walk GIF block framing before invoking the codec. The codec's allocation
/// limit is best-effort, so bound the actual image descriptors as well as the
/// logical screen. Pixel decompression and compositing stay in the codec.
fn preflight(bytes: &[u8], width: u32, height: u32, cancelled: &AtomicBool) -> Result<(), String> {
    let take = |pos: &mut usize, count: usize| -> Result<(), String> {
        *pos=pos.checked_add(count).filter(|end|*end<=bytes.len()).ok_or("Truncated GIF block")?; Ok(())
    };
    let subblocks = |pos: &mut usize| -> Result<(), String> {
        loop {
            if cancelled.load(Ordering::Acquire) { return Err("GIF preview cancelled".into()); }
            let size=*bytes.get(*pos).ok_or("Truncated GIF sub-block")? as usize; take(pos,1)?;
            if size==0 {return Ok(());} take(pos,size)?;
        }
    };
    let mut pos=13usize;
    let packed=bytes[10];
    if packed & 0x80 != 0 { take(&mut pos,3usize << (usize::from(packed & 7)+1))?; }
    let mut total_pixels=0usize; let mut count=0usize;
    loop {
        if cancelled.load(Ordering::Acquire) {return Err("GIF preview cancelled".into());}
        let marker=*bytes.get(pos).ok_or("GIF has no trailer")?; take(&mut pos,1)?;
        match marker {
            0x3b => return if count>0 {Ok(())} else {Err("GIF has no image frames".into())},
            0x21 => {take(&mut pos,1)?; subblocks(&mut pos)?;},
            0x2c => {
                let start=pos; take(&mut pos,9)?;
                let le=|i:usize| u32::from(u16::from_le_bytes([bytes[start+i],bytes[start+i+1]]));
                let (left,top,w,h)=(le(0),le(2),le(4),le(6));
                if w==0 || h==0 || left.checked_add(w).is_none_or(|x|x>width) || top.checked_add(h).is_none_or(|y|y>height) {
                    return Err("GIF frame rectangle exceeds its logical screen".into());
                }
                count=count.checked_add(1).ok_or("GIF frame count overflow")?;
                let pixels=(w as usize).checked_mul(h as usize).ok_or("GIF frame size overflow")?;
                // Compositing may touch the whole logical screen, so charge the
                // larger of frame and screen pixels for every admitted frame.
                let screen=(width as usize).checked_mul(height as usize).ok_or("GIF screen overflow")?;
                total_pixels=total_pixels.checked_add(pixels.max(screen)).ok_or("GIF work overflow")?;
                if count>MAX_FRAMES || total_pixels>MAX_FRAME_PIXELS { return Err("GIF frame decode work exceeds preview profile".into()); }
                let flags=bytes[start+8];
                if flags & 0x80 != 0 {take(&mut pos,3usize << (usize::from(flags & 7)+1))?;}
                let code=*bytes.get(pos).ok_or("GIF LZW code size is missing")?; take(&mut pos,1)?;
                if !(2..=8).contains(&code) {return Err("Unsupported GIF LZW code size".into());}
                subblocks(&mut pos)?;
            },
            _ => return Err("Unsupported GIF block marker".into()),
        }
    }
}

/// Resolve a frame at source time. GIF's nonzero frame delays are exact
/// centiseconds. We do not invent a duration for a zero-delay frame, loop a
/// finite animation, or report this one-shot operation as playback.
pub(super) fn decode(selected: &SelectedMedia, media_tick: u64, cancelled: &AtomicBool) -> Result<PaddedImage, String> {
    if selected.kind != "image/gif" || selected.bytes.len() > MAX_INPUT || selected.bytes.len() < 13 {
        return Err("GIF preview requires a bounded GIF resource".into());
    }
    if cancelled.load(Ordering::Acquire) { return Err("GIF preview cancelled".into()); }
    // Dimensions are in the GIF logical screen descriptor. Check them before
    // the existing decoder initializes to avoid oversized image buffers.
    if !matches!(&selected.bytes[..6], b"GIF87a" | b"GIF89a") { return Err("Invalid GIF signature".into()); }
    let width=u16::from_le_bytes([selected.bytes[6],selected.bytes[7]]) as u32;
    let height=u16::from_le_bytes([selected.bytes[8],selected.bytes[9]]) as u32;
    if width == 0 || height == 0 || width > MAX_SIDE || height > MAX_SIDE { return Err("GIF dimensions exceed preview profile".into()); }
    let pixels=(width as usize).checked_mul(height as usize).ok_or("GIF dimensions overflow")?;
    preflight(&selected.bytes,width,height,cancelled)?;
    let mut decoder=image::codecs::gif::GifDecoder::new(Cursor::new(&selected.bytes)).map_err(|_|"Invalid GIF container")?;
    let mut limits=image::Limits::default();
    limits.max_image_width=Some(MAX_SIDE); limits.max_image_height=Some(MAX_SIDE); limits.max_alloc=Some(MAX_CODEC_ALLOCATION);
    decoder.set_limits(limits).map_err(|_|"GIF decoder limits refused")?;
    let mut elapsed=0u128;
    let tick=u128::from(media_tick);
    let rate=timeline_editing::TICKS_PER_SECOND as u128;
    let mut frames=decoder.into_frames();
    for index in 0..MAX_FRAMES {
        if cancelled.load(Ordering::Acquire) { return Err("GIF preview cancelled; result withheld".into()); }
        if pixels.checked_mul(index+1).is_none_or(|n|n>MAX_FRAME_PIXELS) {
            return Err("GIF frame decode work exceeds preview profile".into());
        }
        let Some(result)=frames.next() else {return Err("Timeline position is outside the admitted GIF animation".into());};
        let frame=result.map_err(|_|"GIF frame decode failed")?;
        let (num,den)=frame.delay().numer_denom_ms();
        if num == 0 || den == 0 { return Err("GIF zero or invalid frame delay is not admitted".into()); }
        let duration=u128::from(num).checked_mul(rate).ok_or("GIF duration overflow")?;
        let divisor=u128::from(den).checked_mul(1000).ok_or("GIF duration overflow")?;
        if duration % divisor != 0 { return Err("GIF frame delay is not exact in timeline ticks".into()); }
        let end=elapsed.checked_add(duration/divisor).ok_or("GIF duration overflow")?;
        if tick < end {
            let rgba=frame.into_buffer().into_raw();
            let side=width.max(height).max(8);
            let out=super::timeline_image_preview::pad(width,height,&rgba,side)?;
            if cancelled.load(Ordering::Acquire) { return Err("GIF preview cancelled; result withheld".into()); }
            return Ok(PaddedImage {side,width,height,rgba:out});
        }
        elapsed=end;
    }
    Err("GIF frame decode work exceeds preview profile".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{codecs::gif::GifEncoder, Delay, Frame, RgbaImage};

    fn sample() -> SelectedMedia {
        let mut bytes=Vec::new();
        { let mut encoder=GifEncoder::new(&mut bytes);
          for color in [[255,0,0,255],[0,255,0,255]] {
              let image=RgbaImage::from_pixel(8,8,image::Rgba(color));
              encoder.encode_frame(Frame::from_parts(image,0,0,Delay::from_numer_denom_ms(100,1))).unwrap();
          }
        }
        SelectedMedia {kind:"image/gif".into(),bytes}
    }
    #[test]
    fn select_frame_on_half_open_timeline_boundaries() {
        let media=sample(); let cancelled=AtomicBool::new(false);
        let duration=timeline_editing::TICKS_PER_SECOND as u64 / 10;
        let a=decode(&media,duration-1,&cancelled).unwrap();
        let b=decode(&media,duration,&cancelled).unwrap();
        assert_eq!(&a.rgba[..4],&[255,0,0,255]);
        assert_eq!(&b.rgba[..4],&[0,255,0,255]);
        assert!(decode(&media,2*duration,&cancelled).is_err());
    }
    #[test]
    fn refuse_cancelled_and_wrong_format_without_decoding() {
        let mut media=sample(); let cancelled=AtomicBool::new(true);
        assert!(decode(&media,0,&cancelled).is_err());
        media.kind="video/mp4".into();
        assert!(decode(&media,0,&AtomicBool::new(false)).is_err());
    }
    #[test]
    fn refuse_frame_rectangle_larger_than_screen_before_codec() {
        let mut media=sample();
        // Find the image descriptor after the logical screen and colour table.
        let mut offset=13usize;
        if media.bytes[10]&0x80 != 0 { offset+=3usize << (usize::from(media.bytes[10]&7)+1); }
        // The encoder writes a graphics-control extension before the frame.
        if media.bytes[offset]==0x21 { offset+=8; }
        assert_eq!(media.bytes[offset],0x2c);
        media.bytes[offset+5..offset+7].copy_from_slice(&1024u16.to_le_bytes());
        assert!(decode(&media,0,&AtomicBool::new(false)).is_err());
    }
}
