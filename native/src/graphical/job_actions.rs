use super::*;
fn summarize_timeline_media(selected: &crate::graphical::checkpoints::TimelineMediaSelection) -> String {
    format!("Associated HMC bytes matched the selected resource digest at inspection: {} bytes, kind {}, media tick {}. Content was not displayed or decoded.",
        selected.selected.bytes.len(), selected.selected.kind, selected.media_tick)
}
fn call_or_retire<T>(owner:&mut Option<T>, call:impl FnOnce(&mut T)->Result<(),String>)->Result<(),String> {
    let result=call(owner.as_mut().ok_or("Audio device unavailable")?);
    if result.is_err() { *owner=None; }
    result
}
fn retire_if<T>(owner:&mut Option<T>, is_retired:impl FnOnce(&T)->bool) {
    if owner.as_ref().is_some_and(is_retired) { *owner=None; }
}
// Keep the actual encoding inside this boundary so cancellation/context can be
// checked after a potentially long codec call and before publishing its result.
fn encode_and_confirm<T>(cancelled: &std::sync::atomic::AtomicBool, encode: impl FnOnce() -> Result<T,String>, current: impl FnOnce() -> Result<bool,String>) -> Result<T,String> {
    let output=encode()?;
    if cancelled.load(std::sync::atomic::Ordering::Acquire) {return Err("Image preview cancelled after encoding; result withheld".into());}
    if !current()? {return Err("Image preview context changed after encoding; result withheld".into());}
    if cancelled.load(std::sync::atomic::Ordering::Acquire) {return Err("Image preview cancelled after context check; result withheld".into());}
    Ok(output)
}
#[cfg(test)]
mod media_summary_tests {
    use super::*;
    #[test]
    fn media_summary_excludes_content_and_source_identifier() {
        let selected = crate::graphical::checkpoints::TimelineMediaSelection {
            clip_source_identifier: "private-source-identifier".into(), media_tick: 42, remaining_ticks: 1,
            selected: crate::graphical::checkpoints::SelectedMedia { kind:"audio/wav".into(), bytes:b"PRIVATE-MEDIA-CONTENT".to_vec() },
        };
        let summary = summarize_timeline_media(&selected);
        assert!(summary.contains("audio/wav") && summary.contains("42"));
        assert!(!summary.contains("PRIVATE-MEDIA-CONTENT"));
        assert!(!summary.contains("private-source-identifier"));
    }
    #[test]
    fn cancellation_during_encoding_withholds_encoded_output() {
        let cancelled=std::sync::atomic::AtomicBool::new(false);
        let result=encode_and_confirm(&cancelled,||{
            // Simulate cancellation while the encoder is in progress.
            cancelled.store(true,std::sync::atomic::Ordering::Release);
            Ok("private encoded image")
        },||Ok(true));
        assert!(result.is_err());
    }
    #[test]
    fn context_change_during_encoding_withholds_encoded_output() {
        let cancelled=std::sync::atomic::AtomicBool::new(false);
        let result=encode_and_confirm(&cancelled,||Ok("private encoded image"),||Ok(false));
        assert!(result.is_err());
    }
    #[test]
    fn failed_audio_owner_is_retired_so_a_later_owner_can_be_created() {
        let mut owner=Some(1_u8);
        assert!(call_or_retire(&mut owner, |_|Err("closed".into())).is_err());
        assert!(owner.is_none());
        owner=Some(2);
        assert!(call_or_retire(&mut owner, |owner|{*owner=3;Ok(())}).is_ok());
        assert_eq!(owner,Some(3));
    }
    #[test]
    fn stale_audio_owner_is_retired_before_a_new_start() {
        let mut owner=Some(true);
        retire_if(&mut owner, |retired|*retired);
        assert!(owner.is_none());
        owner=Some(false);
        retire_if(&mut owner, |retired|*retired);
        assert_eq!(owner,Some(false));
    }
}
pub(super) fn perform(
    host: &mut dyn EditorHost,
    action: Action,
    inventory: &mut Option<crate::graphical::checkpoints::PackageInventory>,
    workflow: &mut Option<crate::graphical::cooperative_workflow::WorkflowView>,
    authoring: &mut Option<crate::graphical::authoring::AuthoringView>,
    inspection: &mut Option<crate::graphical::checkpoints::ResourceInspection>,
    package_workflow: &mut Option<crate::graphical::package_workflow::PackageWorkflowView>,
    typed_10d: &mut Option<crate::graphical::checkpoints::typed_10d::Typed10dInspection>,
    surface: &mut Option<crate::graphical::surfaces::inspection::SurfaceInspection>,
    viewport: &mut Option<crate::graphical::viewport_contract::ViewportState>,
    numeric_sheet: &mut Option<crate::graphical::numeric_sheet::SheetView>,
    gif_frames: &mut Option<Vec<crate::graphical::timeline_gif_preview::GifPlayFrame>>,
    mp4_stream: &mut Option<crate::graphical::timeline_gif_preview::Mp4Stream>,
    cancelled: &std::sync::atomic::AtomicBool,
    wav_device: &mut Option<crate::graphical::timeline_wav_device::WavDevice>,
    audio_clock: &std::sync::Arc<crate::graphical::timeline_wav_device::PlaybackClock>,
) -> Result<(String, Option<BoundImage>), String> {
    let image_uri = None;
    let message = match action {
        Action::PlayTimelineSoundField { request, expected, view_closed, play_cancelled } => {
            let prepared = crate::graphical::timeline_sound_field::prepare(host, &request, &expected, &view_closed, &play_cancelled, cancelled)?;
            let now = host.snapshot()?;
            if cancelled.load(std::sync::atomic::Ordering::Acquire) || view_closed.load(std::sync::atomic::Ordering::Acquire) || play_cancelled.load(std::sync::atomic::Ordering::Acquire)
                || !now.publication_is_current() || Context::from_snapshot(&now) != prepared.context { return Err("Sound-field source, authority or view changed before device admission".into()); }
            retire_if(wav_device, |device|device.is_retired());
            if wav_device.is_none() { *wav_device=Some(crate::graphical::timeline_wav_device::WavDevice::new(audio_clock.clone())?); }
            let summary = prepared.summary;
            call_or_retire(wav_device, |device|device.play(prepared.pcm,prepared.context,prepared.binding,request.start_tick,view_closed.clone(),play_cancelled.clone()))?;
            if cancelled.load(std::sync::atomic::Ordering::Acquire) || view_closed.load(std::sync::atomic::Ordering::Acquire) || play_cancelled.load(std::sync::atomic::Ordering::Acquire) {
                if let Some(device)=wav_device.take(){let _=device.stop();}return Err("Sound-field audition cancelled; stop requested".into());
            }
            summary
        }
        Action::PrepareTimelineMp4 {request,expected,play_cancelled} => {
            if cancelled.load(std::sync::atomic::Ordering::Acquire) || play_cancelled.load(std::sync::atomic::Ordering::Acquire) {return Err("MP4 playback cancelled".into());}
            let selected=host.timeline_media(request,expected.clone(),&play_cancelled)?;
            let snapshot=host.snapshot()?;
            if snapshot.source!=expected || snapshot.reconciliation_required || !snapshot.publication_is_current() || cancelled.load(std::sync::atomic::Ordering::Acquire) {return Err("MP4 source changed or playback cancelled".into());}
            let admitted=Context::from_snapshot(&snapshot);
            let now=host.snapshot()?;
            if cancelled.load(std::sync::atomic::Ordering::Acquire) || play_cancelled.load(std::sync::atomic::Ordering::Acquire) || !now.publication_is_current() || Context::from_snapshot(&now)!=admitted {return Err("MP4 source changed or playback cancelled; frames withheld".into());}
            *mp4_stream=Some(crate::graphical::timeline_gif_preview::start_mp4_stream(selected,snapshot,play_cancelled)?);
            "MP4/H.264 stream opened for bounded silent native playback.".into()
        }
        Action::PrepareTimelineGif {request,expected,play_cancelled} => {
            if cancelled.load(std::sync::atomic::Ordering::Acquire) || play_cancelled.load(std::sync::atomic::Ordering::Acquire) {return Err("GIF playback cancelled".into());}
            let selected=host.timeline_media(request,expected.clone(),&play_cancelled)?;
            let snapshot=host.snapshot()?;
            if snapshot.source!=expected || snapshot.reconciliation_required || !snapshot.publication_is_current() || cancelled.load(std::sync::atomic::Ordering::Acquire) {return Err("GIF source changed or playback cancelled".into());}
            let admitted=Context::from_snapshot(&snapshot);
            let frames=crate::graphical::timeline_gif_preview::prepare_play(&selected,&snapshot,&play_cancelled)?;
            let now=host.snapshot()?;
            if cancelled.load(std::sync::atomic::Ordering::Acquire) || play_cancelled.load(std::sync::atomic::Ordering::Acquire) || !now.publication_is_current() || Context::from_snapshot(&now)!=admitted {return Err("GIF source changed or playback cancelled; frames withheld".into());}
            *gif_frames=Some(frames);
            "GIF frames prepared for bounded native playback; no audio.".into()
        }
        Action::PlayTimelineWavMix { tracks, controls, expected, view_closed, play_cancelled } => {
            super::validate_mix_requests(&tracks)?;
            if controls.len()!=tracks.len(){return Err("Each mix track needs controls".into());}
            for control in &controls {control.validate()?;}
            if cancelled.load(std::sync::atomic::Ordering::Acquire) || view_closed.load(std::sync::atomic::Ordering::Acquire) || play_cancelled.load(std::sync::atomic::Ordering::Acquire) {return Err("Audio mix cancelled before read".into());}
            let initial=host.snapshot()?;let admitted=Context::from_snapshot(&initial);
            if initial.source!=expected || initial.reconciliation_required || !initial.publication_is_current(){return Err("Audio source or authority changed before mix".into());}
            let timeline_start=tracks[0].timeline_tick;
            let binding=crate::graphical::timeline_wav_device::PlaybackBinding{clip_id:tracks[0].clip_id.clone(),resource:tracks[0].resource.clone(),secondary:tracks[1..].iter().map(|t|(t.clip_id.clone(),t.resource.clone())).collect()};
            let mut selections=Vec::with_capacity(tracks.len());
            let mut selected_bytes=0usize;
            for track in tracks {
                if cancelled.load(std::sync::atomic::Ordering::Acquire) || view_closed.load(std::sync::atomic::Ordering::Acquire) || play_cancelled.load(std::sync::atomic::Ordering::Acquire) {return Err("Audio mix cancelled during read".into());}
                let selection=host.timeline_media(track,expected.clone(),&play_cancelled)?;
                selected_bytes=selected_bytes.checked_add(selection.selected.bytes.len()).ok_or("Audio mix input size overflow")?;
                if selected_bytes>8*1024*1024 {return Err("Audio mix exceeds 8 MiB combined encoded input".into());}
                selections.push(selection);
                let mid=host.snapshot()?;
                if cancelled.load(std::sync::atomic::Ordering::Acquire) || view_closed.load(std::sync::atomic::Ordering::Acquire) || play_cancelled.load(std::sync::atomic::Ordering::Acquire) || !mid.publication_is_current() || Context::from_snapshot(&mid)!=admitted {return Err("Source, authority or view changed during audio mix".into());}
            }
            let prepared=crate::graphical::timeline_audio_mix::mix_mono(selections,&controls,&play_cancelled)?;
            let now=host.snapshot()?;
            if cancelled.load(std::sync::atomic::Ordering::Acquire) || view_closed.load(std::sync::atomic::Ordering::Acquire) || play_cancelled.load(std::sync::atomic::Ordering::Acquire) || !now.publication_is_current() || Context::from_snapshot(&now)!=admitted {return Err("Source, authority or view changed before audio mix playback".into());}
            retire_if(wav_device, |device|device.is_retired());
            if wav_device.is_none() { *wav_device=Some(crate::graphical::timeline_wav_device::WavDevice::new(audio_clock.clone())?); }
            call_or_retire(wav_device, |device|device.play(prepared,admitted,binding,timeline_start,view_closed.clone(),play_cancelled.clone()))?;
            if cancelled.load(std::sync::atomic::Ordering::Acquire) || view_closed.load(std::sync::atomic::Ordering::Acquire) || play_cancelled.load(std::sync::atomic::Ordering::Acquire) {if let Some(device)=wav_device.take(){let _=device.stop();}return Err("Audio mix cancelled; stop requested".into());}
            "Mono WAV clips started as a gain/pan stereo audition, limited to their common overlap and at most 15 seconds; output and timing remain unqualified.".into()
        }
        Action::PlayTimelineWav { request, expected, view_closed, play_cancelled } => {
            if cancelled.load(std::sync::atomic::Ordering::Acquire) || view_closed.load(std::sync::atomic::Ordering::Acquire) || play_cancelled.load(std::sync::atomic::Ordering::Acquire) { return Err("Audio playback cancelled before read".into()); }
            let initial=host.snapshot()?;
            if initial.source!=expected || initial.reconciliation_required || !initial.publication_is_current() {return Err("Audio source or authority changed before read".into());}
            let admitted=Context::from_snapshot(&initial);
            let timeline_start=request.timeline_tick;
            let binding=crate::graphical::timeline_wav_device::PlaybackBinding{clip_id:request.clip_id.clone(),resource:request.resource.clone(),secondary:Vec::new()};
            let selected=host.timeline_media(request,expected.clone(),&play_cancelled)?;
            let is_aac=matches!(selected.selected.kind.as_str(),"audio/aac"|"audio/aac-adts"|"audio/mp4"|"video/mp4");
            let is_opus=matches!(selected.selected.kind.as_str(),"audio/ogg"|"audio/opus");
            let prepared=crate::graphical::timeline_audio_output::prepare(&selected,&play_cancelled)?;
            let snapshot=host.snapshot()?;
            if cancelled.load(std::sync::atomic::Ordering::Acquire) || view_closed.load(std::sync::atomic::Ordering::Acquire) || play_cancelled.load(std::sync::atomic::Ordering::Acquire) || snapshot.reconciliation_required || !snapshot.publication_is_current() || Context::from_snapshot(&snapshot)!=admitted { return Err("Audio source, session, view or authority changed before playback".into()); }
            retire_if(wav_device, |device|device.is_retired());
            if wav_device.is_none() { *wav_device=Some(crate::graphical::timeline_wav_device::WavDevice::new(audio_clock.clone())?); }
            call_or_retire(wav_device, |device|device.play(prepared,admitted,binding,timeline_start,view_closed.clone(),play_cancelled.clone()))?;
            if cancelled.load(std::sync::atomic::Ordering::Acquire) || view_closed.load(std::sync::atomic::Ordering::Acquire) || play_cancelled.load(std::sync::atomic::Ordering::Acquire) {
                if let Some(device)=wav_device.take(){let _=device.stop();}
                return Err("Audio playback cancelled; device stop requested".into());
            }
            if is_aac {"AAC core playback started on the exact matching f32 output profile; implicit SBR/PS detection, priming, device buffering and audibility are not qualified.".into()} else if is_opus {"Ogg Opus playback started on the exact matching f32 output profile; timing, device buffering and audibility are not qualified.".into()} else {"WAV playback started on the exact matching f32 output profile; device buffering and audibility are not measured.".into()}
        }
        Action::PauseTimelineWav { paused, expected } => {
            let snapshot=host.snapshot()?;
            if snapshot.source!=expected || !snapshot.publication_is_current() {return Err("Audio source or authority changed".into());}
            let device=wav_device.as_ref().ok_or("No active audio device")?;
            device.set_paused(Context::from_snapshot(&snapshot),paused)?;
            format!("Audio {} requested; queued device samples may remain.",if paused {"pause"} else {"resume"})
        }
        Action::SeekTimelineWav { request, expected } => {
            let snapshot=host.snapshot()?;
            if snapshot.source!=expected || !snapshot.publication_is_current() {return Err("Audio source or authority changed".into());}
            let binding=crate::graphical::timeline_wav_device::PlaybackBinding{clip_id:request.clip_id,resource:request.resource,secondary:Vec::new()};
            let result=wav_device.as_ref().ok_or("No active audio device")?.seek_tick(Context::from_snapshot(&snapshot),binding,request.timeline_tick);
            if let Err(error)=result {
                if wav_device.as_ref().is_some_and(|device|device.is_retired()) { *wav_device=None; }
                return Err(error);
            }
            "WASAPI stream stopped and its pending buffer reset; the selected tick was submitted. Already rendered downstream audio and audibility are not measured.".into()
        }
        Action::StopTimelineWav { .. } => {
            if let Some(device)=wav_device.take() {device.stop()?;}
            "Audio stop requested; queued device samples may remain.".into()
        }
        Action::TimelineWavStatus { .. } => format!("Audio device: {}", wav_device.as_ref().map_or("idle", |device|device.status())),
        Action::AnalyzeTimelineWav { request, expected } => {
            if cancelled.load(std::sync::atomic::Ordering::Acquire) {return Err("WAV analysis cancelled".into());}
            let selected=host.timeline_media(request,expected.clone(),cancelled)?;
            let analysis=crate::graphical::timeline_audio_preview::analyze(&selected,cancelled)?;
            let snapshot=host.snapshot()?;
            if cancelled.load(std::sync::atomic::Ordering::Acquire) || snapshot.source!=expected || !snapshot.publication_is_current() {
                return Err("WAV analysis cancelled or source changed; result withheld".into());
            }
            format!("WAV: {} Hz, {} channels, {} source frames. Clip window: {} frames from source frame {}; mono sample range {:.4} to {:.4}, RMS {:.4}. No audio played.",
                analysis.rate,analysis.channels,analysis.source_frames,analysis.window_frames,analysis.window_start,analysis.minimum,analysis.maximum,analysis.rms)
        }
        Action::PreviewTimelineImage { request, expected } => {
            if cancelled.load(std::sync::atomic::Ordering::Acquire) { return Err("Image preview cancelled".into()); }
            let digest=request.resource.resource_digest;
            let selected=host.timeline_media(request,expected.clone(),cancelled)?;
            let is_gif=selected.selected.kind == "image/gif";
            let image=if is_gif {
                crate::graphical::timeline_gif_preview::decode(&selected.selected,selected.media_tick,cancelled)?
            } else {
                crate::graphical::timeline_image_preview::decode(&selected.selected,digest,cancelled)?
            };
            if cancelled.load(std::sync::atomic::Ordering::Acquire) { return Err("Image preview cancelled; result withheld".into()); }
            let snapshot=host.snapshot()?;
            if snapshot.source!=expected || !snapshot.publication_is_current() { return Err("Image preview source changed; result withheld".into()); }
            let admitted=Context::from_snapshot(&snapshot);
            let label=if is_gif {"Animated GIF frame at the selected media time"} else {"Associated still image"};
            let message=format!("{label} decoded at {} × {} pixels and padded into a {} × {} preview; colour interpretation is unqualified.",image.width,image.height,image.side,image.side);
            return encode_and_confirm(cancelled, || outcome_image(crate::graphical::controller::EditorOutcome {
                frame:Some(crate::graphical::preview::PreviewFrame {source:expected,session_revision:Some(snapshot.session_revision),execution_epoch:Some(snapshot.execution_epoch.clone()),side:image.side,rgba8:image.rgba}),
                snapshot,message,
            }), || {
                let current=host.snapshot()?;
                Ok(current.publication_is_current() && Context::from_snapshot(&current)==admitted)
            });
        }
        Action::InspectTimelineMedia { request, expected } => {
            if cancelled.load(std::sync::atomic::Ordering::Acquire) { return Err("Timeline media inspection cancelled".into()); }
            let selected = host.timeline_media(request, expected, cancelled)?;
            if cancelled.load(std::sync::atomic::Ordering::Acquire) { return Err("Timeline media inspection cancelled; result withheld".into()); }
            summarize_timeline_media(&selected)
        }
        Action::NumericSheetInspect(expected) => {
            if cancelled.load(std::sync::atomic::Ordering::Acquire) { return Err("Sheet inspection cancelled".into()); }
            *numeric_sheet = Some(host.numeric_sheet_view(expected)?);
            "Inspected existing numeric source grid".into()
        }
        Action::NumericSheetEdit { edit, expected } => {
            if cancelled.load(std::sync::atomic::Ordering::Acquire) { return Err("Sheet edit cancelled before admission".into()); }
            return outcome_image(host.numeric_sheet_edit(edit, expected)?);
        }
        Action::Viewport {request,expected} => {
            let output = host.viewport_action(request, expected, cancelled)?;
            let snapshot = host.snapshot()?;
            *viewport = Some(output.state);
            return outcome_image(crate::graphical::controller::EditorOutcome {
                snapshot, frame: Some(output.frame), message: "Native viewport updated; source unchanged.".into()
            });
        }
        Action::InspectSurface { id, expected } => {
            if id.is_empty()
                || id.len() > 128
                || cancelled.load(std::sync::atomic::Ordering::Acquire)
            {
                return Err("Surface inspection refused before dispatch".into());
            }
            *surface = Some(host.inspect_surface(&id, expected)?);
            "Current selected source UV and texture pixels inspected; no asset acceptance claim."
                .into()
        }

        Action::BindSurfacePackage { request, expected } => {
            if cancelled.load(std::sync::atomic::Ordering::Acquire) {
                return Err("Surface binding cancelled before admission".into());
            }
            host.bind_surface_package(request, expected)?;
            "Verified immutable HMC texture snapshot bound; old execution/frame context retired. Rebuild held source explicitly if required.".into()
        }

        Action::InspectTyped10d { request, expected } => {
            if cancelled.load(std::sync::atomic::Ordering::Acquire) {
                return Err("Selected .10d inspection cancelled before read".into());
            }
            let result = host.inspect_typed_10d(
                request,
                expected,
                crate::graphical::checkpoints::typed_10d::Typed10dLimits::default(),
            )?;
            if cancelled.load(std::sync::atomic::Ordering::Acquire) {
                return Err("Selected .10d inspection cancelled; result withheld".into());
            }
            *typed_10d = Some(result);
            "Inspected selected .10d material/frame/skeletal/field snapshot; structural inspection is separate from model acceptance.".into()
        }
        Action::EditTyped10d { request, expected } => {
            let name = request.resource.filename.clone();
            let receipt = host.edit_typed_10d(
                request,
                expected.clone(),
                crate::graphical::checkpoints::typed_10d::Typed10dLimits::default(),
                cancelled,
            )?;
            *inventory = host.package_inventory(&name, expected).ok();
            format!("Saved visual .10d material derivative in {name}; SHA256 {}. Source association is not proof the source regenerates the edited resource. Refresh package review.",crate::graphical::package::hex_digest(&receipt.digest))+&crate::graphical::package::durability_note(&receipt)
        }

        Action::PackageWorkflow { action, expected } => {
            if cancelled.load(std::sync::atomic::Ordering::Acquire) {
                return Err("Worker closed before package workflow admission".into());
            }
            *package_workflow = Some(host.package_workflow(action, expected)?);
            "Saved-package workflow inspected/recorded for the exact digest; refresh after any package or authority change.".into()
        }

        Action::BrushGeoRaster { draft, expected } => {
            if cancelled.load(std::sync::atomic::Ordering::Acquire) {
                return Err("Terrain brush cancelled before dispatch".into());
            }
            let request = draft.into_request()?;
            let name = request.resource.filename.clone();
            let receipt = host.brush_georaster(request, expected.clone(),
                crate::graphical::checkpoints::ui_georaster_edit_limits(), cancelled)?;
            *inventory = host.package_inventory(&name, expected).ok();
            format!("Terrain stroke saved into working copy {name}; package SHA256 {}. Refresh inventory and inspect the new raster before another stroke.",crate::graphical::package::hex_digest(&receipt.digest))
                + &crate::graphical::package::durability_note(&receipt)
        }

        Action::EditGeoRaster { request, expected } => {
            let name = request.resource.filename.clone();
            let receipt = host.edit_georaster(
                request,
                expected.clone(),
                crate::graphical::checkpoints::ui_georaster_edit_limits(),
                cancelled,
            )?;
            *inventory = host.package_inventory(&name, expected).ok();
            format!("Persisted developmental v3 raster edited in {name}; package SHA256 {}. Retained source/resources preserved; refresh inventory. Package review and scientific acceptance remain separate.", crate::graphical::package::hex_digest(&receipt.digest))
                + &crate::graphical::package::durability_note(&receipt)
        }
        Action::CreateGeoRaster { request, expected } => {
            let name = request.filename.clone();
            let receipt = host.create_georaster(
                request,
                expected.clone(),
                crate::graphical::checkpoints::ui_georaster_edit_limits(),
                cancelled,
            )?;
            *inventory = host.package_inventory(&name, expected).ok();
            format!("Explicit source compiled into a new developmental v3 raster resource in {name}; package SHA256 {}. Retained source/resources preserved; refresh inventory. Package review and scientific acceptance remain separate.", crate::graphical::package::hex_digest(&receipt.digest))
                + &crate::graphical::package::durability_note(&receipt)
        }

        Action::InspectResource { request, expected } => {
            use crate::graphical::checkpoints::{ui_inspection_limits, UI_INSPECTION_CHUNKS};
            let mut scratch = [qualia_core_db::q42::m64_matter::M64PayloadInterval::default();
                UI_INSPECTION_CHUNKS];
            *inspection = Some(host.inspect_package_resource(
                request,
                expected,
                ui_inspection_limits(),
                &mut scratch,
            )?);
            "Selected M64 package snapshot inspected. Structural/CRC scope only; scientific and package acceptance remain unassessed.".into()
        }
        Action::AuthoringInspect(expected) => {
            *authoring = Some(host.authoring_view(expected)?);
            "Current accepted assembly inspected".into()
        }
        Action::AuthoringEdit { edit, expected } => {
            let result = match edit {
                crate::graphical::authoring::AuthoringEdit::Rig { resource, control } => host.rig_authoring_edit(resource, control, expected, cancelled)?,
                other => host.authoring_edit(other, expected)?,
            };
            *authoring = host.authoring_view(result.snapshot.source.clone()).ok();
            return outcome_image(result);
        }
        Action::CompileAssembly {
            name,
            key,
            file,
            expected,
        } => {
            let receipt = host.compile_checkpoint(&name, &key, file, expected.clone())?;
            *inventory = host.package_inventory(&name, expected).ok();
            format!("Accepted assembly compiled into {name}; SHA256 {}. Complete model/package acceptance remains separate.",crate::graphical::package::hex_digest(&receipt.digest))+&crate::graphical::package::durability_note(&receipt)
        }
        Action::Workflow { action, .. } => {
            *workflow = Some(host.workflow(action)?);
            "Current source contribution/review evidence updated. Package and master acceptance are separate.".into()
        }
        Action::SaveInspected {
            name,
            file_digest,
            expected,
        } => {
            let receipt = host.save_checkpoint(
                &name,
                qualia_core_db::poet_host::invoke::studio_store::checkpoint::ExpectedFile::Sha256(
                    file_digest,
                ),
                expected.clone(),
            )?;
            *inventory = host.package_inventory(&name, expected).ok();
            format!("Accepted source and complete retained resource inventory saved in {name}; SHA256 {}",crate::graphical::package::hex_digest(&receipt.digest))+&crate::graphical::package::durability_note(&receipt)
        }
        Action::ResourcePlan {
            name,
            file_digest,
            plan,
            expected,
        } => {
            let receipt = host.apply_resource_plan(
                &name,
                qualia_core_db::poet_host::invoke::studio_store::checkpoint::ExpectedFile::Sha256(
                    file_digest,
                ),
                plan,
                expected.clone(),
            )?;
            *inventory = host.package_inventory(&name, expected).ok();
            format!(
                "Atomic resource plan saved in {name}; package SHA256 {}",
                crate::graphical::package::hex_digest(&receipt.digest)
            ) + &crate::graphical::package::durability_note(&receipt)
        }
        Action::Inventory { name, expected } => {
            let result = host.package_inventory(&name, expected)?;
            let message = format!(
                "{} preserved resources in {name}; package SHA256 {}",
                result.resources.len(),
                crate::graphical::package::hex_digest(&result.digest)
            );
            *inventory = Some(result);
            message
        }
        Action::ImportResource {
            name,
            file_digest,
            request,
            expected,
        } => {
            let receipt = host.import_package_resource(
                &name,
                qualia_core_db::poet_host::invoke::studio_store::checkpoint::ExpectedFile::Sha256(
                    file_digest,
                ),
                request,
                expected.clone(),
            )?;
            *inventory = host.package_inventory(&name, expected).ok();
            format!("Resource contents saved in {name}; package SHA256 {}. Unedited entries retained; refresh inventory if unavailable.",crate::graphical::package::hex_digest(&receipt.digest))+&crate::graphical::package::durability_note(&receipt)
        }
        Action::RemoveResource {
            name,
            file_digest,
            key,
            resource_digest,
            expected,
        } => {
            let receipt=host.edit_package_resources(&name,
                qualia_core_db::poet_host::invoke::studio_store::checkpoint::ExpectedFile::Sha256(file_digest),
                vec![qualia_core_db::poet_host::invoke::studio_store::checkpoint::ResourceEdit::Remove{key,expected:resource_digest}],expected.clone())?;
            *inventory = host.package_inventory(&name, expected).ok();
            format!(
                "Explicit resource removal saved in {name}; package SHA256 {}",
                crate::graphical::package::hex_digest(&receipt.digest)
            ) + &crate::graphical::package::durability_note(&receipt)
        }
        Action::Check(source) => host.check(&source)?,
        Action::Submit { source, expected } => {
            let outcome = host.submit(&source, expected)?;
            return outcome_image(outcome);
        }
        Action::Rebuild(expected) => {
            if !host.snapshot()?.rebuild_supported {
                return Err("Execution rebuild is unsupported by this host profile".into());
            }
            return outcome_image(host.rebuild_execution(expected)?);
        }
        Action::Undo(expected) => {
            host.undo(expected)?;
            "Editable source restored; session reconciliation required".into()
        }
        Action::Redo(expected) => {
            host.redo(expected)?;
            "Editable source reapplied; session reconciliation required".into()
        }
        Action::Save { name, expected } => {
            let receipt = host.save_checkpoint(
                &name,
                qualia_core_db::poet_host::invoke::studio_store::checkpoint::ExpectedFile::Absent,
                expected,
            )?;
            let digest = receipt
                .digest
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect::<String>();
            format!("Accepted source/history saved as {name} ({} bytes, SHA256 {digest}); draft and human acceptance are separate",receipt.bytes)+&crate::graphical::package::durability_note(&receipt)
        }
        Action::Refresh => "Current identified source read from canonical store".into(),
    };
    Ok((message, image_uri))
}

pub(super) fn outcome_image(
    outcome: crate::graphical::controller::EditorOutcome,
) -> Result<(String, Option<BoundImage>), String> {
    let mut image_uri = None;
    let mut message = outcome.message;
    if let Some(frame) = outcome.frame {
        if frame.source == outcome.snapshot.source
            && frame.session_revision == Some(outcome.snapshot.session_revision)
            && frame.execution_epoch.as_ref() == Some(&outcome.snapshot.execution_epoch)
            && outcome.snapshot.execution_epoch.is_current()
        {
            match native_image_uri(&frame) {
                Ok(uri) => {
                    image_uri = Some(BoundImage {
                        context: Context::from_snapshot(&outcome.snapshot),
                        uri,
                    })
                }
                Err(error) => message.push_str(&format!("; frame withheld: {error}")),
            }
        } else {
            message.push_str("; frame source/session mismatch; withheld");
        }
    }
    Ok((message, image_uri))
}
