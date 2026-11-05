//! Explicit selected-resource lookup against an already-associated timeline clip.
use crate::graphical::{
    checkpoints::{ResourceInspectionRequest, TimelineMediaRequest},
    controller::EditorSnapshot,
    jobs::{self, Action, Context, HostHandle},
};
use dioxus::prelude::*;

type AudioCancelSlot=std::sync::Arc<std::sync::Mutex<Option<std::sync::Arc<std::sync::atomic::AtomicBool>>>>;
fn cancel_current_audio(slot:&AudioCancelSlot) {
    if let Some(token)=slot.lock().unwrap_or_else(|e|e.into_inner()).as_ref() {
        token.store(true,std::sync::atomic::Ordering::Release);
    }
}
fn replace_audio_cancel(slot:&AudioCancelSlot,token:std::sync::Arc<std::sync::atomic::AtomicBool>) {
    let mut prior=slot.lock().unwrap_or_else(|e|e.into_inner());
    if let Some(old)=prior.as_ref() {old.store(true,std::sync::atomic::Ordering::Release);}
    *prior=Some(token);
}
fn paired_start_allowed(succeeded:bool,current:bool,cancelled:bool,closed:bool)->bool {
    succeeded && current && !cancelled && !closed
}
fn consume_video_duration(elapsed:&mut u128,duration:u128)->bool {
    if *elapsed>=duration {*elapsed-=duration;true}else{false}
}
fn stream_audio_position(tick:u64,start:u64,previous:Option<u64>)->Result<Option<u128>, &'static str> {
    if previous.is_some_and(|last|tick<last) {return Err("Audio moved backward; restart visual playback at the new tick.");}
    Ok(tick.checked_sub(start).map(u128::from))
}
#[cfg(test)]
mod audio_cancel_tests {
    use super::*;
    #[test]
    fn closing_a_view_cancels_its_current_inflight_play() {
        let token=std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let slot=AudioCancelSlot::new(std::sync::Mutex::new(Some(token.clone())));
        cancel_current_audio(&slot);
        assert!(token.load(std::sync::atomic::Ordering::Acquire));
    }
    #[test]
    fn closing_a_view_cancels_its_current_inflight_gif() {
        let token=std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let slot=AudioCancelSlot::new(std::sync::Mutex::new(Some(token.clone())));
        cancel_current_audio(&slot);
        assert!(token.load(std::sync::atomic::Ordering::Acquire));
    }
    #[test]
    fn paired_start_shares_cancellation_and_refuses_failed_or_stale_audio() {
        use std::sync::atomic::Ordering;
        let slot=AudioCancelSlot::new(std::sync::Mutex::new(None));
        let old=std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let paired=std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        replace_audio_cancel(&slot,old.clone());replace_audio_cancel(&slot,paired.clone());
        assert!(old.load(Ordering::Acquire));
        assert!(paired_start_allowed(true,true,false,false));
        assert!(!paired_start_allowed(false,true,false,false));
        assert!(!paired_start_allowed(true,false,false,false));
        assert!(!paired_start_allowed(true,true,true,false));
        assert!(!paired_start_allowed(true,true,false,true));
        cancel_current_audio(&slot);assert!(paired.load(Ordering::Acquire));
    }
    #[test]
    fn playback_keeps_polling_overshoot_instead_of_slowing_video() {
        // 100 polling intervals of 0.6 frames must advance exactly 60 frames.
        let mut elapsed=0u128;let mut advanced=0;
        for _ in 0..100 {elapsed+=600_000;while consume_video_duration(&mut elapsed,1_000_000){advanced+=1;}}
        assert_eq!(advanced,60);assert_eq!(elapsed,0);
    }
    #[test]
    fn stream_audio_position_refuses_backward_seek_and_waits_before_start() {
        assert_eq!(stream_audio_position(9,10,None),Ok(None));
        assert_eq!(stream_audio_position(15,10,Some(12)),Ok(Some(5)));
        assert!(stream_audio_position(11,10,Some(12)).is_err());
    }
}

#[derive(Clone)]
enum AudioControl { Pause, Resume, Seek(TimelineMediaRequest), Stop, Status }
fn audio_control(host: HostHandle, current: EditorSnapshot, control: AudioControl, mut busy: Signal<bool>, mut status: Signal<String>) {
    if busy() || !current.publication_is_current() {return;}
    let guard=host.clone();let origin=Context::from_snapshot(&current);
    let action=match control {
        AudioControl::Pause=>Action::PauseTimelineWav{paused:true,expected:current.source.clone()},
        AudioControl::Resume=>Action::PauseTimelineWav{paused:false,expected:current.source.clone()},
        AudioControl::Seek(request)=>Action::SeekTimelineWav{request,expected:current.source.clone()},
        AudioControl::Stop=>Action::StopTimelineWav{expected:current.source.clone()},
        AudioControl::Status=>Action::TimelineWavStatus{expected:current.source.clone()},
    };
    busy.set(true);
    spawn(async move {
        match jobs::run(host,action,origin).await {
            Ok(reply) if jobs::completion_is_current(&guard,&current,&reply)=>status.set(reply.message),
            Ok(_)=>status.set("Source or authority changed during audio control.".into()),
            Err(error)=>status.set(error),
        }
        busy.set(false);
    });
}

#[component]
pub(super) fn TimelineMediaView(
    host: HostHandle,
    current: EditorSnapshot,
    resource: ResourceInspectionRequest,
    can_preview: bool,
    can_play_gif: bool,
    can_play_mp4: bool,
    can_analyze_audio: bool,
    can_play_aac: bool,
    busy: Signal<bool>,
    status: Signal<String>,
) -> Element {
    let mut clip = use_signal(String::new);
    let mut tick = use_signal(|| "0".to_owned());
    let mut field_duration = use_signal(|| (timeline_editing::TICKS_PER_SECOND as u64).to_string());
    let mut mix_clip=use_signal(String::new);
    let mut mix_key=use_signal(String::new);
    let mut mix_digest=use_signal(String::new);
    let mut mix_extra=use_signal(String::new);
    let mut first_gain=use_signal(||"1".to_owned());let mut first_pan=use_signal(||"0".to_owned());
    let mut second_gain=use_signal(||"1".to_owned());let mut second_pan=use_signal(||"0".to_owned());
    let mut image = use_signal(|| None::<(Context, ResourceInspectionRequest, String)>);
    let mut busy = busy;
    let mut status = status;
    let play_cancel=use_hook(|| std::sync::Arc::new(std::sync::Mutex::new(None::<std::sync::Arc<std::sync::atomic::AtomicBool>>)));
    let mut gif_cancel=use_signal(|| None::<std::sync::Arc<std::sync::atomic::AtomicBool>>);
    let mut gif_playing=use_signal(|| false);
    let mut gif_paused=use_signal(|| false);
    let mut follow_audio=use_signal(|| false);
    let mut paired_audio=use_signal(|| false);
    let gif_inflight=use_hook(|| std::sync::Arc::new(std::sync::Mutex::new(None::<std::sync::Arc<std::sync::atomic::AtomicBool>>)));
    // Any pending start or playing callback is invalidated when this resource
    // row disappears, including package switches and native-window teardown.
    let view_closed=use_hook(|| std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)));
    let close_audio=view_closed.clone();
    let close_cancel=play_cancel.clone();
    let close_gif=gif_inflight.clone();
    use_drop(move || {
        close_audio.store(true,std::sync::atomic::Ordering::Release);
        cancel_current_audio(&close_cancel);
        cancel_current_audio(&close_gif);
    });
    let ready = current.publication_is_current();
    let visible_context=Context::from_snapshot(&current); let visible_resource=resource.clone();
    let gif_host=host.clone(); let gif_current=current.clone(); let gif_resource=resource.clone(); let gif_closed=view_closed.clone(); let gif_slot=gif_inflight.clone();let gif_audio_slot=play_cancel.clone();
    let gif_play = move |_| {
        use std::sync::atomic::Ordering;
        if busy() || !gif_current.publication_is_current() {return;}
        let clip_id=clip(); if clip_id.is_empty()||clip_id.len()>128 {status.set("Enter the existing clip ID.".into());return;}
        let timeline_tick=match tick().parse::<u64>(){Ok(n)=>n,Err(_)=>{status.set("Enter a whole nonnegative timeline tick.".into());return;}};
        let binding=super::timeline_wav_device::PlaybackBinding{clip_id:clip_id.clone(),resource:gif_resource.clone(),secondary:Vec::new()};
        let request=TimelineMediaRequest{clip_id,timeline_tick,resource:gif_resource.clone()};
        let paired_at_start=can_play_mp4 && paired_audio();
        let follow_audio_at_start=follow_audio() || paired_at_start;
        if let Some(old)=gif_cancel(){old.store(true,Ordering::Release);}
        let token=std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));gif_cancel.set(Some(token.clone()));
        *gif_slot.lock().unwrap_or_else(|e|e.into_inner())=Some(token.clone());
        if paired_at_start {replace_audio_cancel(&gif_audio_slot,token.clone());}
        gif_paused.set(false);gif_playing.set(true);image.set(None);busy.set(true);
        let host=gif_host.clone();let guard=host.clone();let current=gif_current.clone();let origin=Context::from_snapshot(&current);let resource=gif_resource.clone();let closed=gif_closed.clone();
        spawn(async move {
            if paired_at_start {
                let audio=jobs::run(host.clone(),Action::PlayTimelineWav{request:request.clone(),expected:current.source.clone(),view_closed:closed.clone(),play_cancelled:token.clone()},origin.clone()).await;
                let outcome=match audio {Ok(reply)=>{
                    let current_reply=jobs::completion_is_current(&guard,&current,&reply);
                    if paired_start_allowed(reply.operation_succeeded,current_reply,token.load(Ordering::Acquire),closed.load(Ordering::Acquire)){Ok(())}
                    else {Err(if current_reply {reply.message} else {"Source or authority changed while starting paired audio.".into()})}
                },Err(error)=>Err(error)};
                if let Err(error)=outcome {
                    token.store(true,Ordering::Release);busy.set(false);
                    if gif_cancel().as_ref().is_some_and(|t|std::sync::Arc::ptr_eq(t,&token)){gif_playing.set(false);status.set(error);}
                    return;
                }
            }
            let action=if can_play_mp4 {Action::PrepareTimelineMp4{request,expected:current.source.clone(),play_cancelled:token.clone()}} else {Action::PrepareTimelineGif{request,expected:current.source.clone(),play_cancelled:token.clone()}};
            let result=jobs::run(host,action,origin.clone()).await;
            busy.set(false);
            let mut reply=match result {Ok(r)=>r,Err(e)=>{if paired_at_start {token.store(true,Ordering::Release);}if gif_cancel().as_ref().is_some_and(|t|std::sync::Arc::ptr_eq(t,&token)){gif_playing.set(false);status.set(e);}return;}};
            if !jobs::completion_is_current(&guard,&current,&reply) || token.load(Ordering::Acquire) || closed.load(Ordering::Acquire) {if paired_at_start {token.store(true,Ordering::Release);}if gif_cancel().as_ref().is_some_and(|t|std::sync::Arc::ptr_eq(t,&token)){gif_playing.set(false);}return;}
            if let Some(stream)=reply.mp4_stream.take() {
                if follow_audio_at_start {
                    status.set("Following this clip's predicted audio clock; acoustic and display synchronization, codec priming and gapless timing are unqualified.".into());
                    let mut frame_end=0u128;let mut active=false;let mut last_tick=None;let mut missing=None::<std::time::Instant>;let mut failure=None::<String>;
                    'sync: loop {
                        if token.load(Ordering::Acquire)||closed.load(Ordering::Acquire)||!guard.playback_context_is_current(&origin){break;}
                        let now=std::time::Instant::now();
                        let target=match guard.estimated_bound_audio_tick(&origin,&binding) {
                            Some(audio_tick)=>{
                                missing=None;
                                let target=match stream_audio_position(audio_tick,timeline_tick,last_tick){Ok(value)=>value,Err(error)=>{failure=Some(error.into());break;}};
                                last_tick=Some(audio_tick);
                                match target {Some(value)=>value,None=>{image.set(None);futures_timer::Delay::new(std::time::Duration::from_millis(10)).await;continue;}}
                            }
                            None=>{
                                if guard.bound_audio_is_paused(&origin,&binding){missing=None;futures_timer::Delay::new(std::time::Duration::from_millis(10)).await;continue;}
                                if now.saturating_duration_since(*missing.get_or_insert(now))>std::time::Duration::from_secs(2){failure=Some("Matching audio clock unavailable; visual playback stopped.".into());break;}
                                futures_timer::Delay::new(std::time::Duration::from_millis(10)).await;continue;
                            }
                        };
                        while !active || target>=frame_end {
                            match stream.frames.try_recv(){
                                Ok(Ok(frame))=>{
                                    if !guard.playback_context_is_current(&origin)||token.load(Ordering::Acquire)||closed.load(Ordering::Acquire){break 'sync;}
                                    frame_end=frame_end.saturating_add(u128::from(frame.duration_ticks));active=true;
                                    if target<frame_end {image.set(Some((origin.clone(),resource.clone(),frame.uri)));}
                                }
                                Ok(Err(error))=>{failure=Some(error);break 'sync;}
                                Err(std::sync::mpsc::TryRecvError::Empty)=>{image.set(None);break;},
                                Err(std::sync::mpsc::TryRecvError::Disconnected)=>break 'sync,
                            }
                        }
                        futures_timer::Delay::new(std::time::Duration::from_millis(10)).await;
                    }
                    drop(stream);
                    if gif_cancel().as_ref().is_some_and(|t|std::sync::Arc::ptr_eq(t,&token)){gif_playing.set(false);image.set(None);status.set(failure.unwrap_or_else(||"Visual playback ended or stopped.".into()));}
                    return;
                }
                status.set("Streaming bounded silent video; decoding may pause presentation.".into());
                let mut active=None::<u128>;let mut elapsed=0u128;let mut last=std::time::Instant::now();let mut failure=None::<String>;
                'playback: loop {
                    if token.load(Ordering::Acquire)||closed.load(Ordering::Acquire)||!guard.playback_context_is_current(&origin){break;}
                    let now=std::time::Instant::now();
                    if active.is_some()&&!gif_paused(){elapsed=elapsed.saturating_add(now.saturating_duration_since(last).as_nanos().saturating_mul(timeline_editing::TICKS_PER_SECOND as u128)/1_000_000_000);}
                    last=now;
                    if let Some(duration)=active {if consume_video_duration(&mut elapsed,duration){active=None;}}
                    while active.is_none()&&!gif_paused(){
                        match stream.frames.try_recv(){
                            Ok(Ok(frame))=>{
                                if !guard.playback_context_is_current(&origin)||token.load(Ordering::Acquire)||closed.load(Ordering::Acquire){break 'playback;}
                                let duration=u128::from(frame.duration_ticks);
                                if consume_video_duration(&mut elapsed,duration) {continue;}
                                active=Some(duration);image.set(Some((origin.clone(),resource.clone(),frame.uri)));
                            },
                            Ok(Err(error))=>{failure=Some(error);break 'playback;},
                            Err(std::sync::mpsc::TryRecvError::Empty)=>{elapsed=0;last=std::time::Instant::now();break;},
                            Err(std::sync::mpsc::TryRecvError::Disconnected)=>break 'playback,
                        }
                    }
                    futures_timer::Delay::new(std::time::Duration::from_millis(10)).await;
                }
                drop(stream);
                if gif_cancel().as_ref().is_some_and(|t|std::sync::Arc::ptr_eq(t,&token)){gif_playing.set(false);image.set(None);status.set(failure.unwrap_or_else(||"Video stream ended or stopped.".into()));}
                return;
            }
            let Some(frames)=reply.gif_frames.take() else {if paired_at_start {token.store(true,Ordering::Release);}gif_playing.set(false);status.set(reply.message);return;};
            status.set(if follow_audio_at_start {"Following the predicted audio-device clock; acoustic and display synchronization are not qualified."} else {"Playing bounded video without audio; colour/presentation remain unqualified."}.into());
            let mut elapsed=0u128; let mut last=std::time::Instant::now(); let mut index=0usize; let mut shown=None; let mut frame_end=u128::from(frames[0].duration_ticks);
            let mut missing_clock_since=None::<std::time::Instant>;let mut clock_unavailable=false;
            loop {
                if token.load(Ordering::Acquire)||closed.load(Ordering::Acquire)||!guard.playback_context_is_current(&origin) {break;}
                let now=std::time::Instant::now();
                if follow_audio_at_start {
                    match guard.estimated_audio_tick(&origin) {
                        Some(audio_tick)=>{
                            missing_clock_since=None;
                            if audio_tick<timeline_tick {image.set(None);futures_timer::Delay::new(std::time::Duration::from_millis(20)).await;continue;}
                            elapsed=u128::from(audio_tick-timeline_tick);
                        }
                        None=>{
                            if guard.audio_is_paused(&origin) {
                                // Retain the last presented visual frame while
                                // audio is paused; no new timing is asserted.
                                missing_clock_since=None;
                                futures_timer::Delay::new(std::time::Duration::from_millis(20)).await;
                                continue;
                            }
                            let since=*missing_clock_since.get_or_insert(now);
                            if now.saturating_duration_since(since)>std::time::Duration::from_secs(2) {clock_unavailable=true;break;}
                            futures_timer::Delay::new(std::time::Duration::from_millis(20)).await;
                            continue;
                        }
                    }
                } else if !gif_paused() {let delta=now.saturating_duration_since(last).as_nanos();elapsed=elapsed.saturating_add(delta.saturating_mul(timeline_editing::TICKS_PER_SECOND as u128)/1_000_000_000);}
                last=now;
                // Recompute because an accepted audio seek can move backward.
                index=0;frame_end=u128::from(frames[0].duration_ticks);
                while elapsed>=frame_end {index+=1;if index>=frames.len(){break;}frame_end=frame_end.saturating_add(u128::from(frames[index].duration_ticks));}
                if index>=frames.len(){break;}
                if shown!=Some(index) {image.set(Some((origin.clone(),resource.clone(),frames[index].uri.clone())));shown=Some(index);}
                futures_timer::Delay::new(std::time::Duration::from_millis(20)).await;
            }
            if gif_cancel().as_ref().is_some_and(|t|std::sync::Arc::ptr_eq(t,&token)) {gif_playing.set(false);image.set(None);status.set(if clock_unavailable {"Predicted audio clock unavailable; visual playback stopped."} else {"Visual playback ended or stopped."}.into());}
        });
    };
    let play_host=host.clone(); let play_current=current.clone(); let play_resource=resource.clone();
    let mix_host=host.clone();let mix_current=current.clone();let mix_resource=resource.clone();
    let mix_closed=view_closed.clone();let mix_slot=play_cancel.clone();
    let field_host=host.clone();let field_current=current.clone();let field_resource=resource.clone();
    let field_closed=view_closed.clone();let field_slot=play_cancel.clone();
    let field_pause_host=host.clone();let field_pause_current=current.clone();
    let field_resume_host=host.clone();let field_resume_current=current.clone();
    let field_stop_host=host.clone();let field_stop_current=current.clone();let field_stop_slot=play_cancel.clone();
    let play_field=move |_| {
        if busy() || !field_current.publication_is_current() { return; }
        let request=(||Ok::<_,String>(crate::graphical::timeline_sound_field::FieldRequest {
            package:field_resource.clone(),start_tick:tick().parse().map_err(|_|"Enter a whole sound-field start tick")?,
            duration_ticks:field_duration().parse().map_err(|_|"Enter a whole sound-field duration")? }))();
        let request=match request.and_then(|r|{r.validate()?;Ok(r)}) {Ok(r)=>r,Err(error)=>{status.set(error);return;}};
        let token=std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        replace_audio_cancel(&field_slot,token.clone());
        let host=field_host.clone();let guard=host.clone();let current=field_current.clone();let closed=field_closed.clone();
        let origin=Context::from_snapshot(&current);busy.set(true);
        spawn(async move {
            match jobs::run(host,Action::PlayTimelineSoundField {request,expected:current.source.clone(),view_closed:closed,play_cancelled:token},origin).await {
                Ok(reply) if jobs::completion_is_current(&guard,&current,&reply)=>status.set(reply.message),
                Ok(_)=>status.set("Sound-field source or authority changed; result withheld.".into()),Err(error)=>status.set(error),
            }
            busy.set(false);
        });
    };
    let play_mix=move |_|{
        use std::sync::atomic::Ordering;
        if busy() || !mix_current.publication_is_current(){return;}
        let first_id=clip();let second_id=mix_clip();
        if first_id.is_empty() || second_id.is_empty() || first_id==second_id || first_id.len()>128 || second_id.len()>128 {status.set("Enter two different existing clip IDs.".into());return;}
        let timeline_tick=match tick().parse::<u64>() {Ok(v)=>v,Err(_)=>{status.set("Enter a whole nonnegative timeline tick.".into());return;}};
        let digest=match crate::graphical::package::parse_digest(&mix_digest()) {Ok(v)=>v,Err(error)=>{status.set(error);return;}};
        let second_resource=ResourceInspectionRequest{filename:mix_resource.filename.clone(),package_digest:mix_resource.package_digest,key:mix_key(),resource_digest:digest};
        if let Err(error)=second_resource.validate_bounds(){status.set(error);return;}
        let first=TimelineMediaRequest{clip_id:first_id,timeline_tick,resource:mix_resource.clone()};
        let second=TimelineMediaRequest{clip_id:second_id,timeline_tick,resource:second_resource};
        let mut tracks=vec![first,second];
        let parse_control=|gain:&str,pan:&str|->Result<super::timeline_audio_mix::MixControls,String>{
            let gain=gain.parse::<f32>().map_err(|_|"Enter a numeric track gain")?;
            let pan=pan.parse::<f32>().map_err(|_|"Enter a numeric track pan")?;
            let control=super::timeline_audio_mix::MixControls{gain,pan};control.validate()?;Ok(control)
        };
        let mut controls=Vec::new();
        for (gain,pan) in [(first_gain(),first_pan()),(second_gain(),second_pan())] {
            match parse_control(&gain,&pan) {Ok(control)=>controls.push(control),Err(error)=>{status.set(error);return;}}
        }
        let extra=mix_extra();
        if extra.len()>8192 {status.set("Additional track list exceeds 8192 bytes.".into());return;}
        for line in extra.lines().filter(|line|!line.trim().is_empty()) {
            let fields:Vec<_>=line.split('|').map(str::trim).collect();
            if !(fields.len()==3 || fields.len()==5) || fields[0].is_empty() || fields[1].is_empty() {status.set("Each additional track needs clip ID | resource key | SHA256, optionally followed by | gain | pan.".into());return;}
            let resource_digest=match crate::graphical::package::parse_digest(fields[2]) {Ok(v)=>v,Err(error)=>{status.set(error);return;}};
            let control=if fields.len()==5 {match parse_control(fields[3],fields[4]) {Ok(c)=>c,Err(error)=>{status.set(error);return;}}} else {super::timeline_audio_mix::MixControls::default()};
            tracks.push(TimelineMediaRequest{clip_id:fields[0].into(),timeline_tick,resource:ResourceInspectionRequest{filename:mix_resource.filename.clone(),package_digest:mix_resource.package_digest,key:fields[1].into(),resource_digest}});
            controls.push(control);
            if tracks.len()>qualia_audio::MAX_TRACKS {status.set("Audio mix supports at most 16 tracks.".into());return;}
        }
        if let Err(error)=jobs::validate_mix_requests(&tracks){status.set(error);return;}
        let token=std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        {let mut prior=mix_slot.lock().unwrap_or_else(|e|e.into_inner());if let Some(old)=prior.as_ref(){old.store(true,Ordering::Release);}*prior=Some(token.clone());}
        let host=mix_host.clone();let guard=host.clone();let current=mix_current.clone();let origin=Context::from_snapshot(&current);let view_closed=mix_closed.clone();busy.set(true);
        spawn(async move {
            match jobs::run(host,Action::PlayTimelineWavMix{tracks,controls,expected:current.source.clone(),view_closed,play_cancelled:token},origin).await {
                Ok(reply) if jobs::completion_is_current(&guard,&current,&reply)=>status.set(reply.message),
                Ok(_)=>status.set("Source or authority changed while starting the mix; result withheld.".into()),
                Err(error)=>status.set(error),
            }
            busy.set(false);
        });
    };
    let play_cancel_for_start=play_cancel.clone();
    let play = move |_| {
        if busy() || !play_current.publication_is_current() { return; }
        let clip_id=clip();
        if clip_id.is_empty() || clip_id.len()>128 { status.set("Enter the existing timeline clip ID.".into()); return; }
        let timeline_tick=match tick().parse::<u64>() {Ok(v)=>v,Err(_)=>{status.set("Enter a whole nonnegative timeline tick.".into());return;}};
        let request=TimelineMediaRequest{clip_id,timeline_tick,resource:play_resource.clone()};
        let host=play_host.clone();let guard=host.clone();let current=play_current.clone();let origin=Context::from_snapshot(&current);
        let view_closed=view_closed.clone();
        let play_cancelled=std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        {
            let mut prior=play_cancel_for_start.lock().unwrap_or_else(|e|e.into_inner());
            if let Some(token)=prior.as_ref() {token.store(true,std::sync::atomic::Ordering::Release);}
            *prior=Some(play_cancelled.clone());
        }
        busy.set(true);
        spawn(async move {
            match jobs::run(host,Action::PlayTimelineWav{request,expected:current.source.clone(),view_closed,play_cancelled},origin).await {
                Ok(reply) if jobs::completion_is_current(&guard,&current,&reply)=>status.set(reply.message),
                Ok(_)=>status.set("Source or authority changed while starting audio; result withheld.".into()),
                Err(error)=>status.set(error),
            }
            busy.set(false);
        });
    };
    let pause_host=host.clone();let pause_current=current.clone();
    let resume_host=host.clone();let resume_current=current.clone();
    let stop_host=host.clone();let stop_current=current.clone();
    let seek_host=host.clone();let seek_current=current.clone();let seek_resource=resource.clone();
    let play_cancel_for_stop=play_cancel.clone();
    let status_host=host.clone();let status_current=current.clone();
    let preview_host=host.clone(); let preview_current=current.clone(); let preview_resource=resource.clone();
    let audio_host=host.clone(); let audio_current=current.clone(); let audio_resource=resource.clone();
    let audio = move |_| {
        if busy() || !audio_current.publication_is_current() { return; }
        let clip_id=clip();
        if clip_id.is_empty() || clip_id.len()>128 {status.set("Enter the existing timeline clip ID (1–128 bytes).".into());return;}
        let timeline_tick=match tick().parse::<u64>() {Ok(v)=>v,Err(_)=>{status.set("Enter a whole nonnegative timeline tick.".into());return;}};
        let request=TimelineMediaRequest{clip_id,timeline_tick,resource:audio_resource.clone()};
        let host=audio_host.clone(); let guard=host.clone(); let current=audio_current.clone(); let origin=Context::from_snapshot(&current);
        busy.set(true);
        spawn(async move {
            match jobs::run(host,Action::AnalyzeTimelineWav{request,expected:current.source.clone()},origin).await {
                Ok(reply) if jobs::completion_is_current(&guard,&current,&reply)=>status.set(reply.message),
                Ok(_)=>status.set("Source/session/authority changed; audio analysis withheld. Refresh inventory.".into()),
                Err(error)=>status.set(error),
            }
            busy.set(false);
        });
    };
    let preview = move |_| {
        if busy() || !preview_current.publication_is_current() { return; }
        // Retire the old animation before a one-shot preview owns the same
        // image signal; its finishing task must not erase the new preview.
        if let Some(old)=gif_cancel(){old.store(true,std::sync::atomic::Ordering::Release);}
        gif_cancel.set(None); gif_playing.set(false);
        let clip_id=clip();
        if clip_id.is_empty() || clip_id.len()>128 {status.set("Enter the existing timeline clip ID (1–128 bytes).".into());return;}
        let timeline_tick=match tick().parse::<u64>() {Ok(v)=>v,Err(_)=>{status.set("Enter a whole nonnegative timeline tick.".into());return;}};
        let request=TimelineMediaRequest{clip_id,timeline_tick,resource:preview_resource.clone()};
        let host=preview_host.clone(); let guard=host.clone(); let current=preview_current.clone();
        let origin=Context::from_snapshot(&current); let resource=preview_resource.clone();
        image.set(None);busy.set(true);
        spawn(async move {
            match jobs::run(host,Action::PreviewTimelineImage{request,expected:current.source.clone()},origin.clone()).await {
                Ok(reply) if jobs::completion_is_current(&guard,&current,&reply)=>{
                    if let Some(uri)=reply.image_uri { image.set(Some((origin,resource,uri))); }
                    status.set(reply.message);
                }
                Ok(_)=>status.set("Source/session/authority changed; image withheld. Refresh inventory.".into()),
                Err(error)=>status.set(error),
            }
            busy.set(false);
        });
    };
    let run = move |_| {
        if busy() || !current.publication_is_current() { return; }
        let clip_id = clip();
        if clip_id.is_empty() || clip_id.len() > 128 {
            status.set("Enter the existing timeline clip ID (1–128 bytes).".into()); return;
        }
        let timeline_tick = match tick().parse::<u64>() {
            Ok(v) => v,
            Err(_) => { status.set("Enter a whole nonnegative timeline tick.".into()); return; }
        };
        let request = TimelineMediaRequest { clip_id, timeline_tick, resource: resource.clone() };
        let host = host.clone(); let guard = host.clone(); let current = current.clone();
        let origin = Context::from_snapshot(&current);
        busy.set(true);
        spawn(async move {
            match jobs::run(host, Action::InspectTimelineMedia { request, expected: current.source.clone() }, origin).await {
                Ok(reply) if jobs::completion_is_current(&guard, &current, &reply) => status.set(reply.message),
                Ok(_) => status.set("Source/session/authority changed; media result withheld. Refresh inventory.".into()),
                Err(error) => status.set(error),
            }
            busy.set(false);
        });
    };
    rsx! {
        fieldset { disabled:!ready,
            legend { "Inspect associated timeline media" }
            p { "Uses the current source-owned clip association and this exact HMC resource. Inspection reads bytes without decoding, displaying or playing them." }
            label { "Clip ID " input { value:"{clip}", maxlength:128, aria_label:"Timeline media clip ID", oninput:move |e| {if let Some(t)=gif_cancel(){t.store(true,std::sync::atomic::Ordering::Release);} clip.set(e.value());image.set(None);} } }
            label { "Timeline tick " input { value:"{tick}", maxlength:20, aria_label:"Timeline media tick", oninput:move |e| {if let Some(t)=gif_cancel(){t.store(true,std::sync::atomic::Ordering::Release);} tick.set(e.value());image.set(None);} } }
            button { onclick:run, "Inspect associated media" }
            p { "Audition the source-authored spatial field using every assigned clip and its own digest-bound resource in this selected HMC. The clip ID above is not a field selector. Unassigned timeline clips are explicitly outside this field; no resident entity is silently dropped. Requires 48 kHz mono sources; the output is stereo. Pause/resume/stop use the existing audio controls; field seek requires a new exact audition interval." }
            label { "Sound-field duration (exact ticks, at most 15 seconds)" input { value:"{field_duration}",maxlength:20,oninput:move|e|field_duration.set(e.value()) } }
            button { disabled:busy(),onclick:play_field,"Audition authored sound field" }
            button { onclick:move |_| audio_control(field_pause_host.clone(),field_pause_current.clone(),AudioControl::Pause,busy,status), "Pause sound field" }
            button { onclick:move |_| audio_control(field_resume_host.clone(),field_resume_current.clone(),AudioControl::Resume,busy,status), "Resume sound field" }
            button { onclick:move |_| {
                cancel_current_audio(&field_stop_slot);
                if busy() {status.set("Sound-field cancellation requested; pending preparation will stop at its next check.".into());}
                else {audio_control(field_stop_host.clone(),field_stop_current.clone(),AudioControl::Stop,busy,status);}
            }, "Stop sound field" }
            if can_preview {button { onclick:preview, "Preview associated image or GIF frame" }}
            if can_play_gif || can_play_mp4 {
                if can_play_mp4 {label {input {r#type:"checkbox",checked:paired_audio(),disabled:gif_playing(),onchange:move |e|paired_audio.set(e.checked())} " Start this MP4's audio with its visual" }}
                label { input {r#type:"checkbox",checked:follow_audio(),disabled:gif_playing(),onchange:move |e|follow_audio.set(e.checked())} " Follow active audio clock (estimated; MP4 requires the same clip and resource)" }
                button {disabled:busy(),onclick:gif_play,"Play visual"}
                button {disabled:!gif_playing()||follow_audio()||(can_play_mp4 && paired_audio()),onclick:move |_| {let next=!gif_paused();gif_paused.set(next);},if gif_paused() {"Resume visual"} else {"Pause visual"}}
                button {disabled:!gif_playing(),onclick:move |_| {if let Some(token)=gif_cancel(){token.store(true,std::sync::atomic::Ordering::Release);}},"Stop visual"}
                p {"Visual playback is bounded to 15 seconds and 256 × 256 pixels; MP4 must use the admitted IDR-started H.264 profile. Paired playback starts audio, then visual follows its estimated clock; both must admit the same clip and resource. Backward audio seek requires restarting visual playback. Looping is not included."}
            }
            if can_analyze_audio {button { onclick:audio, "Analyze associated WAV window" }}
            if can_analyze_audio || can_play_aac {
                p { "Playback requires a matching f32 output device, sample rate and channel count. AAC accepts bounded unprotected mono/stereo ADTS AAC-LC core and nonfragmented MP4 profiles; detection of implicit SBR/PS is not comprehensive, and priming and gapless timing are unqualified. Ogg Opus accepts bounded, intact single-stream family-0 mono/stereo at 48 kHz with signalled pre-skip and final granule trimming; audio timing, gain and device output remain unverified. Seek is limited to the loaded clip remainder and resets the pending WASAPI stream buffer; previously rendered downstream audio may remain." }
                button { onclick:play, "Play associated audio" }
                if can_analyze_audio {
                details { summary {"Audition multiple mono WAV clips"}
                    p {"Audition 2–16 distinct clips at the same timeline tick, up to 15 seconds or their common overlap. Gain 0–1 and pan -1 (left) to 1 (right) apply only to this audition; center gain 1 preserves the equal-level mix. If the mix would clip, reduce gains. Each clip must be associated with a WAV resource in this package; the output must match their sample rate and stereo channels. Mixed playback cannot be sought."}
                    label {"First clip gain " input {value:"{first_gain}",maxlength:16,oninput:move|e|first_gain.set(e.value())} }
                    label {"First clip pan " input {value:"{first_pan}",maxlength:16,oninput:move|e|first_pan.set(e.value())} }
                    label {"Second clip ID " input {value:"{mix_clip}",maxlength:128,oninput:move|e|mix_clip.set(e.value())} }
                    label {"Second resource key " input {value:"{mix_key}",maxlength:256,oninput:move|e|mix_key.set(e.value())} }
                    label {"Second resource SHA256 " input {value:"{mix_digest}",maxlength:64,oninput:move|e|mix_digest.set(e.value())} }
                    label {"Second clip gain " input {value:"{second_gain}",maxlength:16,oninput:move|e|second_gain.set(e.value())} }
                    label {"Second clip pan " input {value:"{second_pan}",maxlength:16,oninput:move|e|second_pan.set(e.value())} }
                    label {"Additional tracks (one per line: clip ID | resource key | SHA256 | gain | pan; last two optional)" textarea {value:"{mix_extra}",maxlength:8192,oninput:move|e|mix_extra.set(e.value())} }
                    button {onclick:play_mix,"Play WAV mix"}
                }
                }
                button { onclick:move |_| audio_control(pause_host.clone(),pause_current.clone(),AudioControl::Pause,busy,status), "Pause audio" }
                button { onclick:move |_| audio_control(resume_host.clone(),resume_current.clone(),AudioControl::Resume,busy,status), "Resume audio" }
                button { onclick:move |_| {
                    match tick().parse::<u64>() {
                        Ok(value)=>audio_control(seek_host.clone(),seek_current.clone(),AudioControl::Seek(TimelineMediaRequest {clip_id:clip(),timeline_tick:value,resource:seek_resource.clone()}),busy,status),
                        Err(_)=>status.set("Enter a whole nonnegative timeline tick.".into()),
                    }
                }, "Seek audio to timeline tick" }
                button { onclick:move |_| {
                    cancel_current_audio(&play_cancel_for_stop);
                    if busy() {status.set("Audio cancellation requested; pending work will stop at its next cancellation check.".into());}
                    else {audio_control(stop_host.clone(),stop_current.clone(),AudioControl::Stop,busy,status);}
                }, "Stop audio" }
                button { onclick:move |_| audio_control(status_host.clone(),status_current.clone(),AudioControl::Status,busy,status), "Audio status" }
            }
            if let Some((origin,selected,uri))=image() {
                if ready && origin==visible_context && selected==visible_resource {
                    img { src:"{uri}", alt:"Associated image or GIF frame preview; colour interpretation is unqualified", style:"max-width:100%;max-height:320px;object-fit:contain;" }
                }
            }
        }
    }
}
