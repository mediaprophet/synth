//! Poet-owned, single-device-thread WAV output adapter.
//!
//! The host must submit already-authorized prepared PCM and an exact live
//! context. FilmCraft's MIT-licensed `apps/filmcraft/src/audio.rs` is the
//! reference for CPAL output ownership/device error handling; no FilmCraft
//! application/session, clock, UI or serializer is used here. Preserve the
//! FilmCraft MIT notice when integrating this adapter.
//!
//! This candidate is not yet mounted in the native worker or UI. No playback
//! is established until the caller wires lifecycle/authority and qualifies it.
use super::{jobs::Context,timeline_audio_output::PreparedWav};
use cpal::traits::{DeviceTrait,HostTrait,StreamTrait};
use std::sync::{Arc,Mutex,mpsc,atomic::{AtomicBool,AtomicU8,Ordering}};
use std::time::Duration;
use std::time::Instant;

const IDLE:u8=0; const PLAYING:u8=1; const PAUSED:u8=2; const ENDED:u8=3; const INVALID:u8=4; const FAILED:u8=5;

// This is a predicted callback/device clock, never an acoustic or display
// presentation timestamp. It is scoped to one accepted source and lease.
const CLOCK_BLOCKS:usize=8;
#[derive(Clone,Copy)]
struct ClockBlock { first_frame:usize, end_frame:usize, predicted_first:Instant }
struct ClockAnchor { context:Context, binding:PlaybackBinding, lease:Arc<AtomicBool>, view:Arc<AtomicBool>, start_tick:u64, rate:u32,
    blocks:[Option<ClockBlock>;CLOCK_BLOCKS], next:usize }
fn binding_matches(actual:&PlaybackBinding,expected:Option<&PlaybackBinding>)->bool {
    expected.is_none_or(|requested|requested==actual)
}
fn predicted_frame(first_frame:usize,end_frame:usize,rate:u32,first:Instant,now:Instant)->Option<u128>{
    if rate==0{return None;}
    if now<first{return None;}
    let frames=end_frame.checked_sub(first_frame)? as u128;
    let block_ns=frames.checked_mul(1_000_000_000)?.checked_add(u128::from(rate)-1)?/u128::from(rate);
    let end=first.checked_add(Duration::from_nanos(u64::try_from(block_ns).ok()?))?;
    if now>end {return None;}
    let nanos=now.checked_duration_since(first)?.as_nanos();
    let advanced=nanos.checked_mul(u128::from(rate))?/1_000_000_000;
    (first_frame as u128).checked_add(advanced.min(frames))
}
fn select_predicted_frame(blocks:&[Option<ClockBlock>;CLOCK_BLOCKS],rate:u32,now:Instant)->Option<u128>{
    blocks.iter().flatten().filter_map(|block|predicted_frame(block.first_frame,block.end_frame,rate,block.predicted_first,now).map(|frame|(block.predicted_first,frame)))
        .max_by_key(|(first,_)|*first).map(|(_,frame)|frame)
}
#[derive(Default)]
pub(super) struct PlaybackClock { anchor:Mutex<Option<ClockAnchor>>, blocked:AtomicBool, paused:AtomicBool }
impl PlaybackClock {
    #[cfg(test)]
    pub(super) fn initialize_test(&self,context:Context,binding:PlaybackBinding,lease:Arc<AtomicBool>,view:Arc<AtomicBool>)->Result<(),String> {
        self.initialize(context,binding,lease,view,0,48_000)
    }
    #[cfg(test)]
    pub(super) fn poison_anchor_test(&self) {
        let _=std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {let _guard=self.anchor.lock().unwrap();panic!("poison clock test");}));
    }
    /// The worker calls this after publishing each completed host operation.
    /// Source/session changes cancel the active lease; unchanged work does not.
    pub(super) fn invalidate_if_context_changed(&self,next:Option<&Context>) {
        let slot=self.anchor.lock().unwrap_or_else(|poisoned|poisoned.into_inner());
        if let Some(a)=slot.as_ref() {
            if next != Some(&a.context) {
                a.lease.store(true,Ordering::Release);
                self.blocked.store(true,Ordering::Release);
            }
        }
    }
    fn initialize(&self,context:Context,binding:PlaybackBinding,lease:Arc<AtomicBool>,view:Arc<AtomicBool>,start_tick:u64,rate:u32)->Result<(),String> {
        let mut slot=self.anchor.lock().map_err(|_|"Audio clock is unavailable; playback refused")?;
        *slot=Some(ClockAnchor{context,binding,lease,view,start_tick,rate,blocks:[None;CLOCK_BLOCKS],next:0});
        self.blocked.store(false,Ordering::Release);
        self.paused.store(false,Ordering::Release);
        Ok(())
    }
    fn clear(&self){self.blocked.store(true,Ordering::Release);self.paused.store(false,Ordering::Release);if let Ok(mut slot)=self.anchor.lock(){*slot=None;}}
    // Called only on owner/control threads; callback publication is nonblocking.
    fn suspend(&self){self.blocked.store(true,Ordering::Release);if let Ok(mut slot)=self.anchor.lock(){if let Some(a)=slot.as_mut(){a.blocks=[None;CLOCK_BLOCKS];a.next=0;}}}
    fn resume(&self){self.blocked.store(false,Ordering::Release);}
    pub(super) fn is_paused(&self,expected:&Context)->bool {
        self.is_paused_bound(expected,None)
    }
    pub(super) fn is_paused_bound(&self,expected:&Context,binding:Option<&PlaybackBinding>)->bool {
        if !self.paused.load(Ordering::Acquire){return false;}
        self.anchor.try_lock().ok().and_then(|slot|slot.as_ref().map(|a|&a.context==expected && binding_matches(&a.binding,binding) && !a.lease.load(Ordering::Acquire) && !a.view.load(Ordering::Acquire) && current(&a.context))).unwrap_or(false)
    }
    fn invalidate_frame(&self){if let Ok(mut slot)=self.anchor.try_lock(){if let Some(a)=slot.as_mut(){a.blocks=[None;CLOCK_BLOCKS];a.next=0;}}}
    fn publish(&self,first:usize,end:usize,lead:Duration){
        if self.blocked.load(Ordering::Acquire){return;}
        if lead>Duration::from_secs(2) {self.invalidate_frame();return;}
        // Keep the final predicted block until its predicted end; a later
        // silent callback does not mean already-queued samples were heard.
        if end<=first {return;}
        if let Some(predicted_first)=Instant::now().checked_add(lead) {
            if let Ok(mut slot)=self.anchor.try_lock(){if !self.blocked.load(Ordering::Acquire){if let Some(a)=slot.as_mut(){a.blocks[a.next]=Some(ClockBlock{first_frame:first,end_frame:end,predicted_first});a.next=(a.next+1)%CLOCK_BLOCKS;}}}
        } else {self.invalidate_frame();}
    }
    pub(super) fn estimate_tick(&self,expected:&Context)->Option<u64>{
        self.estimate_tick_bound(expected,None)
    }
    pub(super) fn estimate_tick_bound(&self,expected:&Context,binding:Option<&PlaybackBinding>)->Option<u64>{
        if self.blocked.load(Ordering::Acquire){return None;}
        let slot=self.anchor.try_lock().ok()?;if self.blocked.load(Ordering::Acquire){return None;}let a=slot.as_ref()?;
        if &a.context!=expected || !binding_matches(&a.binding,binding) || a.lease.load(Ordering::Acquire) || a.view.load(Ordering::Acquire) || !current(&a.context) || a.rate==0 {return None;}
        let frame=select_predicted_frame(&a.blocks,a.rate,Instant::now())?;
        let ticks=frame.checked_mul(timeline_editing::TICKS_PER_SECOND as u128)?/u128::from(a.rate);
        u64::try_from(u128::from(a.start_tick).checked_add(ticks)?).ok()
    }
}
// A blocked platform driver must not allow repeated launches to accumulate
// detached device threads. The slot is held until the owner thread really exits.
static DEVICE_SLOT: AtomicBool = AtomicBool::new(false);
struct SlotGuard;
impl Drop for SlotGuard { fn drop(&mut self) { DEVICE_SLOT.store(false, Ordering::Release); } }

fn current(c:&Context)->bool {
    !c.reconciliation_required && c.execution_epoch.is_current() && c.source.check_current().is_ok()
        && c.authorisation_stamp.as_ref().is_some_and(|s|s.is_current())
}
fn latch_live(cancelled:&AtomicBool, context_live:bool)->bool {
    if !context_live { cancelled.store(true,Ordering::Release); }
    !cancelled.load(Ordering::Acquire)
}
struct Samples {data:Vec<f32>,channels:usize,frame:usize,paused:bool,context:Context,ended:bool,status:Arc<AtomicU8>,cancelled:Arc<AtomicBool>,view_closed:Arc<AtomicBool>,clock:Arc<PlaybackClock>}
impl Samples {
    fn render(&mut self,output:&mut[f32],lead:Option<Duration>) {
        let first=self.frame;
        let live=latch_live(&self.cancelled,!self.view_closed.load(Ordering::Acquire) && current(&self.context));
        fill_pcm(&self.data,self.channels,&mut self.frame,self.paused,&mut self.ended,live,output);
        if !live {self.cancelled.store(true,Ordering::Release);self.status.store(INVALID,Ordering::Release);} else if self.ended {self.status.store(ENDED,Ordering::Release);}
        // Source/authority may change while constructing this output buffer.
        if !latch_live(&self.cancelled,!self.view_closed.load(Ordering::Acquire) && current(&self.context)) {output.fill(0.0);self.status.store(INVALID,Ordering::Release);}
        if self.cancelled.load(Ordering::Acquire) || self.paused {self.clock.invalidate_frame();}
        else if let Some(lead)=lead {self.clock.publish(first,self.frame,lead);} else {self.clock.invalidate_frame();}
    }
}
fn fill_pcm(data:&[f32],channels:usize,position:&mut usize,paused:bool,ended:&mut bool,live:bool,output:&mut[f32]) {
        output.fill(0.0);
        if paused || *ended || !live || channels==0 || output.len()%channels!=0 {return;}
        for frame in output.chunks_exact_mut(channels) {
            let Some(start)=position.checked_mul(channels) else {*ended=true;break;};
            let Some(end)=start.checked_add(channels) else {*ended=true;break;};
            let Some(source)=data.get(start..end) else {*ended=true;break;};
            frame.copy_from_slice(source); *position+=1;
        }
}
#[derive(Clone,PartialEq,Eq)]
pub(super) struct PlaybackBinding { pub clip_id:String, pub resource:super::checkpoints::ResourceInspectionRequest, pub secondary:Vec<(String,super::checkpoints::ResourceInspectionRequest)> }
struct Active {stream:cpal::Stream,samples:Arc<Mutex<Samples>>,context:Context,binding:PlaybackBinding,failed:Arc<AtomicBool>,cancelled:Arc<AtomicBool>,view_closed:Arc<AtomicBool>,timeline_start:u64,rate:u32,frame_count:usize}
enum Command {
    Start {pcm:PreparedWav,context:Context,binding:PlaybackBinding,timeline_start:u64,cancelled:Arc<AtomicBool>,view_closed:Arc<AtomicBool>,reply:mpsc::Sender<Result<(),String>>},
    Pause {context:Context,paused:bool,reply:mpsc::Sender<Result<(),String>>},
    Seek {context:Context,binding:PlaybackBinding,timeline_tick:u64,reply:mpsc::Sender<Result<(),String>>},
    Stop {reply:mpsc::Sender<Result<(),String>>},
    Shutdown,
}
pub(super) struct WavDevice {sender:mpsc::SyncSender<Command>,shutdown:Arc<AtomicBool>,status:Arc<AtomicU8>,lease:Arc<AtomicBool>,clock:Arc<PlaybackClock>}
impl WavDevice {
    pub fn new(clock:Arc<PlaybackClock>)->Result<Self,String>{
        // Give a recently retired owner time to drop its stream, without
        // waiting indefinitely on a blocked platform driver.
        let mut acquired=false;
        for _ in 0..=25 {
            if DEVICE_SLOT.compare_exchange(false,true,Ordering::AcqRel,Ordering::Acquire).is_ok() {acquired=true;break;}
            std::thread::sleep(Duration::from_millis(10));
        }
        if !acquired {return Err("Audio device owner is already active or still shutting down".into());}
        let (sender,receiver)=mpsc::sync_channel(1);
        let shutdown=Arc::new(AtomicBool::new(false));let thread_shutdown=shutdown.clone();
        let status=Arc::new(AtomicU8::new(IDLE));let thread_status=status.clone();
        let thread_clock=clock.clone();
        let thread=std::thread::Builder::new().name("poet-wav-device".into()).spawn(move||{
            let _slot=SlotGuard;
            let mut active:Option<Active>=None;
            while !thread_shutdown.load(Ordering::Acquire) {
              let command=match receiver.recv_timeout(Duration::from_millis(50)){
                Ok(command)=>command,
                Err(mpsc::RecvTimeoutError::Timeout)=>{
                    if active.as_ref().is_some_and(|a|a.cancelled.load(Ordering::Acquire)||a.view_closed.load(Ordering::Acquire)||!current(&a.context)) {
                        thread_status.store(INVALID,Ordering::Release);
                        active=None;
                        break;
                    }
                    continue
                },Err(_)=>break};
              match command {
                Command::Start{pcm,context,binding,timeline_start,cancelled,view_closed,reply}=>{active=None;thread_clock.clear();thread_status.store(IDLE,Ordering::Release); let result=start(pcm,context,binding,timeline_start,thread_status.clone(),cancelled,view_closed,thread_clock.clone()).map(|next|{active=Some(next);});if result.is_err(){thread_clock.clear();thread_status.store(FAILED,Ordering::Release);}let _=reply.send(result);},
                Command::Pause{context,paused,reply}=>{let result=with_active(&mut active,&context,|a|{
                    thread_clock.suspend();
                    let mut s=a.samples.lock().map_err(|_|"Audio state poisoned")?;s.paused=paused;drop(s);
                    let result=if paused {a.stream.pause()}else{a.stream.play()};
                    if let Err(error)=result {a.cancelled.store(true,Ordering::Release);thread_status.store(FAILED,Ordering::Release);return Err(format!("Audio pause/resume failed: {error}"));}
                    thread_clock.paused.store(paused,Ordering::Release);
                    if !paused {thread_clock.resume();}thread_status.store(if paused {PAUSED}else{PLAYING},Ordering::Release); Ok(())});let _=reply.send(result);},
                Command::Seek{context,binding,timeline_tick,reply}=>{
                    thread_clock.suspend();
                    let result=with_active(&mut active,&context,|a|{if a.binding!=binding {return Err("Selected clip or HMC resource differs from playing audio".into());}seek_active(a,timeline_tick,&thread_status)});
                    if active.as_ref().is_some_and(|a|!a.cancelled.load(Ordering::Acquire) && thread_status.load(Ordering::Acquire)==PLAYING) {thread_clock.resume();}
                    let _=reply.send(result);
                },
                Command::Stop{reply}=>{thread_clock.clear();active=None;thread_status.store(IDLE,Ordering::Release);let _=reply.send(Ok(()));break;},
                Command::Shutdown=>break,
            }}
        });
        if let Err(error)=thread { DEVICE_SLOT.store(false,Ordering::Release); return Err(format!("Audio device thread unavailable: {error}")); }
        // Drop JoinHandle without joining: a stuck driver is bounded to one
        // owner thread, and the audio lease is cancelled before shutdown.
        Ok(Self{sender,shutdown,status,lease:Arc::new(AtomicBool::new(true)),clock})
    }
    fn request(&self,command:impl FnOnce(mpsc::Sender<Result<(),String>>)->Command)->Result<(),String>{
        let (send,receive)=mpsc::channel();if self.sender.try_send(command(send)).is_err(){self.lease.store(true,Ordering::Release);return Err("Audio device queue is busy or closed; playback lease cancelled".into());}
        match receive.recv_timeout(Duration::from_secs(2)) {Ok(result)=>result,Err(_)=>{self.lease.store(true,Ordering::Release);Err("Audio device command timed out; playback lease cancelled and device state must be reconciled".into())}}
    }
    pub fn play(&mut self,pcm:PreparedWav,context:Context,binding:PlaybackBinding,timeline_start:u64,view_closed:Arc<AtomicBool>,play_cancelled:Arc<AtomicBool>)->Result<(),String>{self.lease.store(true,Ordering::Release);self.lease=play_cancelled;let cancelled=self.lease.clone();self.request(|reply|Command::Start{pcm,context,binding,timeline_start,cancelled,view_closed,reply})}
    pub fn set_paused(&self,context:Context,paused:bool)->Result<(),String>{self.request(|reply|Command::Pause{context,paused,reply})}
    /// WASAPI can reset its stopped stream through the local CPAL owner. Other
    /// backends fail closed until they provide an equivalent acknowledgment.
    pub fn seek_tick(&self,context:Context,binding:PlaybackBinding,timeline_tick:u64)->Result<(),String>{self.request(|reply|Command::Seek{context,binding,timeline_tick,reply})}
    pub fn stop(&self)->Result<(),String>{self.lease.store(true,Ordering::Release);self.request(|reply|Command::Stop{reply})}
    pub fn is_retired(&self)->bool {self.lease.load(Ordering::Acquire) || self.shutdown.load(Ordering::Acquire) || matches!(self.status.load(Ordering::Acquire),INVALID|FAILED)}
    pub fn status(&self)->&'static str {match self.status.load(Ordering::Acquire){PLAYING=>"playing",PAUSED=>"pause requested; buffered audio may remain",ENDED=>"samples queued; device completion is not established",INVALID=>"view closed or source/authority unavailable or changed; buffered audio may remain",FAILED=>"device failed",_=>"idle"}}
}
impl Drop for WavDevice {fn drop(&mut self){self.lease.store(true,Ordering::Release);self.clock.clear();self.shutdown.store(true,Ordering::Release);let _=self.sender.try_send(Command::Shutdown);}}
fn with_active(active:&mut Option<Active>,context:&Context,f:impl FnOnce(&mut Active)->Result<(),String>)->Result<(),String>{
    let device=active.as_mut().ok_or("No active WAV output")?;
    if device.failed.load(Ordering::Acquire) {return Err("Audio device reported failure; stop and select again".into());}
    if device.cancelled.load(Ordering::Acquire)||&device.context!=context || !current(context){return Err("Audio source or authority changed; stop and select again".into());}
    f(device)
}

#[cfg(windows)]
fn stop_and_reset(stream:&cpal::Stream)->Result<(),String>{
    match stream.as_inner() {
        cpal::platform::StreamInner::Wasapi(stream)=>stream.stop_and_reset(Duration::from_secs(1)),
        #[allow(unreachable_patterns)]
        _=>Err("Audio seek requires the WASAPI reset-capable backend".into()),
    }
}
#[cfg(not(windows))]
fn stop_and_reset(_stream:&cpal::Stream)->Result<(),String>{Err("Audio seek is not supported by this backend".into())}

#[cfg(windows)]
fn supports_reset(stream:&cpal::Stream)->bool {
    matches!(stream.as_inner(),cpal::platform::StreamInner::Wasapi(_))
}
#[cfg(not(windows))]
fn supports_reset(_stream:&cpal::Stream)->bool {false}

fn reset_then_commit(reset:impl FnOnce()->Result<(),String>,commit:impl FnOnce()->Result<(),String>)->Result<(),String>{
    reset()?;
    commit()
}

fn seek_active(a:&mut Active,timeline_tick:u64,status:&AtomicU8)->Result<(),String>{
    // Admit the target before affecting the live stream.
    let frame=super::timeline_audio_output::relative_seek_frame(a.timeline_start,timeline_tick,a.rate,a.frame_count)?;
    if !supports_reset(&a.stream) {return Err("Audio seek is not supported by this backend; playback is unchanged".into());}
    let mut paused=false;
    let reset_result=reset_then_commit(||stop_and_reset(&a.stream),||{
        if a.cancelled.load(Ordering::Acquire)||a.view_closed.load(Ordering::Acquire)||!current(&a.context) {
            status.store(INVALID,Ordering::Release);
            return Err("Audio source, view or authority changed during stream reset".into());
        }
        let mut samples=a.samples.lock().map_err(|_|"Audio state poisoned")?;
        samples.frame=frame;samples.ended=false;paused=samples.paused;
        Ok(())
    });
    if let Err(error)=reset_result {
        a.cancelled.store(true,Ordering::Release);
        if status.load(Ordering::Acquire)!=INVALID {status.store(FAILED,Ordering::Release);}
        return Err(error);
    }
    if !paused {
        if let Err(error)=a.stream.play() {a.cancelled.store(true,Ordering::Release);status.store(FAILED,Ordering::Release);return Err(format!("Audio restart after seek failed: {error}"));}
    }
    if a.cancelled.load(Ordering::Acquire)||a.view_closed.load(Ordering::Acquire)||!current(&a.context) {
        a.cancelled.store(true,Ordering::Release);status.store(INVALID,Ordering::Release);
        return Err("Audio source, view or authority changed after stream reset".into());
    }
    status.store(if paused {PAUSED}else{PLAYING},Ordering::Release);
    Ok(())
}
fn start(pcm:PreparedWav,context:Context,binding:PlaybackBinding,timeline_start:u64,status:Arc<AtomicU8>,cancelled:Arc<AtomicBool>,view_closed:Arc<AtomicBool>,clock:Arc<PlaybackClock>)->Result<Active,String>{
    if cancelled.load(Ordering::Acquire)||view_closed.load(Ordering::Acquire)||!current(&context){return Err("Audio source, view, authority or request is not current".into());}
    if pcm.channels==0 || pcm.rate==0 || pcm.samples.is_empty() || pcm.samples.len()%usize::from(pcm.channels)!=0 {return Err("Invalid prepared WAV".into());}
    let device=cpal::default_host().default_output_device().ok_or("No default audio output device")?;
    let supported=device.default_output_config().map_err(|e|format!("Audio output configuration: {e}"))?;
    if supported.sample_format()!=cpal::SampleFormat::F32 || supported.channels()!=pcm.channels || supported.sample_rate().0!=pcm.rate {
        return Err("Default device must expose the WAV's exact rate/channels and f32 output; conversion is not enabled".into());
    }
    let config:cpal::StreamConfig=supported.into();let rate=pcm.rate;let frame_count=pcm.frames();
    clock.initialize(context.clone(),binding.clone(),cancelled.clone(),view_closed.clone(),timeline_start,rate)?;
    let samples=Arc::new(Mutex::new(Samples{data:pcm.samples,channels:usize::from(pcm.channels),frame:0,paused:false,context:context.clone(),ended:false,status:status.clone(),cancelled:cancelled.clone(),view_closed:view_closed.clone(),clock:clock.clone()}));
    let callback=samples.clone();
    let failed=Arc::new(AtomicBool::new(false));let callback_failed=failed.clone();let error_failed=failed.clone();let error_status=status.clone();let error_cancel=cancelled.clone();let error_clock=clock.clone();
    let callback_clock=clock.clone();
    let stream=device.build_output_stream(&config,move |out:&mut[f32],info:&cpal::OutputCallbackInfo|{
        if callback_failed.load(Ordering::Acquire){out.fill(0.0);callback_clock.invalidate_frame();}
        else if let Ok(mut state)=callback.try_lock(){let stamp=info.timestamp();state.render(out,stamp.playback.duration_since(&stamp.callback));}else{out.fill(0.0);callback_clock.invalidate_frame();}
    },move |_error|{error_failed.store(true,Ordering::Release);error_cancel.store(true,Ordering::Release);error_clock.invalidate_frame();error_status.store(FAILED,Ordering::Release);},None).map_err(|e|format!("Audio stream creation failed: {e}"))?;
    if cancelled.load(Ordering::Acquire)||view_closed.load(Ordering::Acquire)||!current(&context){return Err("Audio source, view, authority or request changed before playback".into());}
    status.store(PLAYING,Ordering::Release);
    stream.play().map_err(|e|format!("Audio stream start failed: {e}"))?;
    if failed.load(Ordering::Acquire)||cancelled.load(Ordering::Acquire){return Err("Audio device failed or request was cancelled during start".into());}
    Ok(Active{stream,samples,context,binding,failed,cancelled,view_closed,timeline_start,rate,frame_count})
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn predicted_audio_frame_stays_inside_the_published_block() {
        let start=Instant::now();
        assert_eq!(predicted_frame(8,12,4,start+Duration::from_millis(1),start),None);
        assert_eq!(predicted_frame(8,12,4,start,start),Some(8));
        assert_eq!(predicted_frame(8,12,4,start,start+Duration::from_millis(500)),Some(10));
        assert_eq!(predicted_frame(8,12,4,start,start+Duration::from_secs(1)),Some(12));
        assert_eq!(predicted_frame(8,12,4,start,start+Duration::from_millis(1001)),None);
        assert_eq!(predicted_frame(8,12,0,start,start),None);
    }
    #[test] fn queued_future_audio_blocks_do_not_hide_the_current_block() {
        let start=Instant::now();let mut blocks=[None;CLOCK_BLOCKS];
        blocks[0]=Some(ClockBlock{first_frame:0,end_frame:4,predicted_first:start});
        blocks[1]=Some(ClockBlock{first_frame:4,end_frame:8,predicted_first:start+Duration::from_secs(1)});
        assert_eq!(select_predicted_frame(&blocks,4,start+Duration::from_millis(500)),Some(2));
        assert_eq!(select_predicted_frame(&blocks,4,start+Duration::from_millis(1500)),Some(6));
        assert_eq!(select_predicted_frame(&blocks,4,start+Duration::from_secs(3)),None);
    }
    #[test] fn seek_commits_only_after_confirmed_reset() {
        let mut committed=false;
        assert!(reset_then_commit(||Err("reset refused".into()),||{committed=true;Ok(())}).is_err());
        assert!(!committed);
        assert!(reset_then_commit(||Ok(()),||{committed=true;Ok(())}).is_ok());
        assert!(committed);
    }
    #[test] fn seek_binding_distinguishes_clip_package_and_resource() {
        let resource=super::super::checkpoints::ResourceInspectionRequest {
            filename:"media.hmc".into(),package_digest:[1;32],key:"wav".into(),resource_digest:[2;32],
        };
        let original=PlaybackBinding{clip_id:"first".into(),resource,secondary:Vec::new()};
        assert!(binding_matches(&original,Some(&original)));
        let mut other=original.clone();other.clip_id="second".into();assert!(!binding_matches(&original,Some(&other)));
        let mut other=original.clone();other.resource.package_digest=[3;32];assert!(!binding_matches(&original,Some(&other)));
        let mut other=original.clone();other.resource.resource_digest=[4;32];assert!(!binding_matches(&original,Some(&other)));
        let mut other=original.clone();other.secondary.push(("mix".into(),other.resource.clone()));assert!(!binding_matches(&original,Some(&other)));
    }
    #[test] fn context_invalidation_is_terminal_for_audio_lease() {
        let lease=AtomicBool::new(false);
        assert!(latch_live(&lease,true));
        assert!(!latch_live(&lease,false));
        assert!(!latch_live(&lease,true));
    }
    #[test] fn bounded_pcm_pauses_and_silences_stale_and_pads_end() {
        let source=[0.1,0.2,0.3,0.4]; let mut frame=0; let mut ended=false; let mut out=[1.0;6];
        fill_pcm(&source,2,&mut frame,true,&mut ended,true,&mut out); assert_eq!(out,[0.0;6]); assert_eq!(frame,0);
        fill_pcm(&source,2,&mut frame,false,&mut ended,false,&mut out); assert_eq!(out,[0.0;6]); assert_eq!(frame,0);
        fill_pcm(&source,2,&mut frame,false,&mut ended,true,&mut out); assert_eq!(out,[0.1,0.2,0.3,0.4,0.0,0.0]); assert_eq!(frame,2); assert!(ended);
    }
}
