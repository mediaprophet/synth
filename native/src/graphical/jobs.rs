//! Explicit native commands use one owned worker with one queued request.
use super::controller::{EditorHost, EditorSnapshot, SessionEpoch};
use super::preview::{native_image_uri, SourceRef};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    mpsc, Arc, Mutex,
};
use std::thread::JoinHandle;
use std::time::Duration;

const MAX_REQUEST_BYTES: usize = 64 * 1024;

#[derive(Clone, PartialEq, Eq)]
pub struct Context {
    pub source: SourceRef,
    pub session_revision: u64,
    pub execution_epoch: SessionEpoch,
    pub reconciliation_required: bool,
    pub authorisation_stamp: Option<super::cooperation::AuthorisationStamp>,
}
impl Context {
    pub fn from_snapshot(snapshot: &EditorSnapshot) -> Self {
        Self {
            source: snapshot.source.clone(),
            session_revision: snapshot.session_revision,
            execution_epoch: snapshot.execution_epoch.clone(),
            reconciliation_required: snapshot.reconciliation_required,
            authorisation_stamp: snapshot.authorisation_stamp.clone(),
        }
    }
}

pub enum Action {
    NumericSheetInspect(SourceRef),
    NumericSheetEdit { edit: super::numeric_sheet::SheetEdit, expected: SourceRef },
    Viewport { request: super::viewport::ViewportCommand, expected: SourceRef },
    InspectSurface {
        id: String,
        expected: SourceRef,
    },
    BindSurfacePackage {
        request: super::surface_host::SurfacePackageRequest,
        expected: SourceRef,
    },
    InspectTyped10d {
        request: super::checkpoints::ResourceInspectionRequest,
        expected: SourceRef,
    },
    EditTyped10d {
        request: super::checkpoints::typed_10d::Typed10dEditRequest,
        expected: SourceRef,
    },
    PackageWorkflow {
        action: super::package_workflow::PackageWorkflowAction,
        expected: SourceRef,
    },
    BrushGeoRaster {
        draft: super::terrain_brush::TerrainBrushDraft,
        expected: SourceRef,
    },
    EditGeoRaster {
        request: super::checkpoints::GeoRasterEditRequest,
        expected: SourceRef,
    },
    CreateGeoRaster {
        request: super::checkpoints::GeoRasterCreateRequest,
        expected: SourceRef,
    },
    InspectResource {
        request: super::checkpoints::ResourceInspectionRequest,
        expected: SourceRef,
    },
    InspectTimelineMedia {
        request: super::checkpoints::TimelineMediaRequest,
        expected: SourceRef,
    },
    PreviewTimelineImage {
        request: super::checkpoints::TimelineMediaRequest,
        expected: SourceRef,
    },
    PrepareTimelineGif { request: super::checkpoints::TimelineMediaRequest, expected: SourceRef, play_cancelled: Arc<AtomicBool> },
    PrepareTimelineMp4 { request: super::checkpoints::TimelineMediaRequest, expected: SourceRef, play_cancelled: Arc<AtomicBool> },
    AnalyzeTimelineWav {
        request: super::checkpoints::TimelineMediaRequest,
        expected: SourceRef,
    },
    PlayTimelineWav { request: super::checkpoints::TimelineMediaRequest, expected: SourceRef, view_closed: Arc<AtomicBool>, play_cancelled: Arc<AtomicBool> },
    PlayTimelineWavMix { tracks: Vec<super::checkpoints::TimelineMediaRequest>, controls: Vec<super::timeline_audio_mix::MixControls>, expected: SourceRef, view_closed: Arc<AtomicBool>, play_cancelled: Arc<AtomicBool> },
    PlayTimelineSoundField { request: super::timeline_sound_field::FieldRequest, expected: SourceRef, view_closed: Arc<AtomicBool>, play_cancelled: Arc<AtomicBool> },
    PauseTimelineWav { paused: bool, expected: SourceRef },
    SeekTimelineWav { request: super::checkpoints::TimelineMediaRequest, expected: SourceRef },
    StopTimelineWav { expected: SourceRef },
    TimelineWavStatus { expected: SourceRef },
    AuthoringInspect(SourceRef),
    AuthoringEdit {
        edit: super::authoring::AuthoringEdit,
        expected: SourceRef,
    },
    CompileAssembly {
        name: String,
        key: String,
        file: qualia_core_db::poet_host::invoke::studio_store::checkpoint::ExpectedFile,
        expected: SourceRef,
    },
    Workflow {
        action: super::cooperative_workflow::WorkflowAction,
        expected: SourceRef,
    },
    SaveInspected {
        name: String,
        file_digest: [u8; 32],
        expected: SourceRef,
    },
    Check(String),
    Submit {
        source: String,
        expected: SourceRef,
    },
    Undo(SourceRef),
    Redo(SourceRef),
    Rebuild(SourceRef),
    Save {
        name: String,
        expected: SourceRef,
    },
    Inventory {
        name: String,
        expected: SourceRef,
    },
    ResourcePlan {
        name: String,
        file_digest: [u8; 32],
        plan: Vec<super::package::LibraryEdit>,
        expected: SourceRef,
    },
    ImportResource {
        name: String,
        file_digest: [u8; 32],
        request: super::package::ImportRequest,
        expected: SourceRef,
    },
    RemoveResource {
        name: String,
        file_digest: [u8; 32],
        key: String,
        resource_digest: [u8; 32],
        expected: SourceRef,
    },
    Refresh,
}

/// Only these operations are guaranteed not to mutate editor/session state.
/// New actions fail closed until explicitly classified.
fn media_read_only(action:&Action)->bool {
    matches!(action,
        Action::NumericSheetInspect(_) | Action::Viewport{..} | Action::InspectSurface{..} | Action::InspectTyped10d{..}
        | Action::InspectResource{..} | Action::InspectTimelineMedia{..} | Action::PreviewTimelineImage{..}
        | Action::PrepareTimelineGif{..} | Action::PrepareTimelineMp4{..} | Action::AnalyzeTimelineWav{..}
        | Action::PlayTimelineWav{..} | Action::PlayTimelineWavMix{..} | Action::PlayTimelineSoundField{..} | Action::PauseTimelineWav{..}
        | Action::SeekTimelineWav{..} | Action::StopTimelineWav{..} | Action::TimelineWavStatus{..}
        | Action::AuthoringInspect(_) | Action::Check(_) | Action::Inventory{..})
}

pub struct Reply {
    pub gif_frames: Option<Vec<super::timeline_gif_preview::GifPlayFrame>>,
    pub mp4_stream: Option<super::timeline_gif_preview::Mp4Stream>,
    pub numeric_sheet: Option<super::numeric_sheet::SheetView>,
    pub viewport: Option<super::viewport_contract::ViewportState>,
    pub surface: Option<super::surfaces::inspection::SurfaceInspection>,
    pub typed_10d: Option<super::checkpoints::typed_10d::Typed10dInspection>,
    pub package_workflow: Option<super::package_workflow::PackageWorkflowView>,
    pub inspection: Option<super::checkpoints::ResourceInspection>,
    /// Actual backend result, independent of human-readable diagnostics.
    pub operation_succeeded: bool,
    pub authoring: Option<super::authoring::AuthoringView>,
    pub origin: Context,
    pub snapshot: Option<EditorSnapshot>,
    pub message: String,
    pub image_uri: Option<String>,
    pub current: bool,
    pub inventory: Option<super::checkpoints::PackageInventory>,
    pub workflow: Option<super::cooperative_workflow::WorkflowView>,
}

struct Request {
    action: Action,
    origin: Context,
    sender: futures_channel::oneshot::Sender<Reply>,
}

#[derive(Clone)]
pub struct HostHandle {
    sender: mpsc::SyncSender<Request>,
    closing: Arc<AtomicBool>,
    // Derived completion guard only; authoritative state stays in EditorHost.
    latest: Arc<Mutex<Option<Context>>>,
    // Retains the last completed host context while unrelated commands run.
    media_context: Arc<Mutex<Option<Context>>>,
    audio_clock: Arc<super::timeline_wav_device::PlaybackClock>,
}
impl HostHandle {
    pub fn estimated_audio_tick(&self,expected:&Context)->Option<u64>{
        if self.closing.load(Ordering::Acquire) {return None;}
        self.audio_clock.estimate_tick(expected)
    }
    pub fn audio_is_paused(&self,expected:&Context)->bool {
        !self.closing.load(Ordering::Acquire) && self.audio_clock.is_paused(expected)
    }
    pub(super) fn estimated_bound_audio_tick(&self,expected:&Context,binding:&super::timeline_wav_device::PlaybackBinding)->Option<u64>{
        if self.closing.load(Ordering::Acquire) {return None;}
        self.audio_clock.estimate_tick_bound(expected,Some(binding))
    }
    pub(super) fn bound_audio_is_paused(&self,expected:&Context,binding:&super::timeline_wav_device::PlaybackBinding)->bool {
        !self.closing.load(Ordering::Acquire) && self.audio_clock.is_paused_bound(expected,Some(binding))
    }
    /// Validate the authority and source of a running read-only presentation
    /// without treating unrelated queued worker activity as a source change.
    pub fn playback_context_is_current(&self,expected:&Context)->bool {
        !self.closing.load(Ordering::Acquire) && !expected.reconciliation_required
            && expected.execution_epoch.is_current() && expected.source.check_current().is_ok()
            && expected.authorisation_stamp.as_ref().is_none_or(|stamp|stamp.is_current())
            && self.media_context.try_lock().is_ok_and(|slot|slot.as_ref()==Some(expected))
    }
}
impl PartialEq for HostHandle {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.closing, &other.closing)
    }
}

/// Owned by launch(), outside the component. Its join runs only after Aether's
/// event loop returns. Started native effects cannot be forcibly cancelled.
pub struct Worker {
    closing: Arc<AtomicBool>,
    audio_inflight: Arc<Mutex<Option<Arc<AtomicBool>>>>,
    thread: Option<JoinHandle<()>>,
}
impl Worker {
    pub fn start(mut host: Box<dyn EditorHost>) -> Result<(Self, HostHandle), String> {
        let (sender, receiver) = mpsc::sync_channel::<Request>(1);
        let closing = Arc::new(AtomicBool::new(false));
        let worker_closing = closing.clone();
        let audio_inflight: Arc<Mutex<Option<Arc<AtomicBool>>>> = Arc::new(Mutex::new(None));
        let worker_audio_inflight = audio_inflight.clone();
        let audio_clock=Arc::new(super::timeline_wav_device::PlaybackClock::default());
        let worker_audio_clock=audio_clock.clone();
        let latest = Arc::new(Mutex::new(Some(Context::from_snapshot(&host.snapshot()?))));
        let worker_latest = latest.clone();
        let media_context=Arc::new(Mutex::new(latest.lock().map_err(|_|"completion cache poisoned")?.clone()));
        let worker_media_context=media_context.clone();
        let thread = std::thread::Builder::new()
            .name("poet-native-authoring".into())
            .spawn(move || {
                let mut wav_device = None;
                let mut visual_lease:Option<Arc<AtomicBool>>=None;
                loop {
                    if worker_closing.load(Ordering::Acquire) {
                        break;
                    }
                    let request = match receiver.recv_timeout(Duration::from_millis(50)) {
                        Ok(request) => request,
                        Err(mpsc::RecvTimeoutError::Timeout) => continue,
                        Err(mpsc::RecvTimeoutError::Disconnected) => break,
                    };
                    // A closed receiver/window abandons work that has not started.
                    if worker_closing.load(Ordering::Acquire) || request.sender.is_canceled() {
                        continue;
                    }
                    if !media_read_only(&request.action) {
                        // Fence playback before any operation that may change the
                        // session, including when it ultimately makes no edit.
                        worker_audio_clock.invalidate_if_context_changed(None);
                        if let Some(token)=visual_lease.take(){token.store(true,Ordering::Release);}
                        *worker_media_context.lock().expect("media context cache poisoned")=None;
                    }
                    if let Action::PrepareTimelineGif{play_cancelled,..} | Action::PrepareTimelineMp4{play_cancelled,..}=&request.action {
                        if let Some(old)=visual_lease.replace(play_cancelled.clone()) {
                            if !Arc::ptr_eq(&old,play_cancelled){old.store(true,Ordering::Release);}
                        }
                    }
                    if let Action::PlayTimelineWav { play_cancelled, .. } | Action::PlayTimelineWavMix { play_cancelled, .. } | Action::PlayTimelineSoundField { play_cancelled, .. } | Action::PrepareTimelineGif { play_cancelled, .. } | Action::PrepareTimelineMp4 { play_cancelled, .. } = &request.action {
                        let mut active=worker_audio_inflight.lock().unwrap_or_else(|e|e.into_inner());
                        if worker_closing.load(Ordering::Acquire) {
                            play_cancelled.store(true,Ordering::Release);
                            continue;
                        }
                        *active=Some(play_cancelled.clone());
                    }
                    // Invalidate already-sent completions while the next job runs.
                    *worker_latest.lock().expect("completion cache poisoned") = None;
                    let reply = execute_with_audio(
                        host.as_mut(),
                        request.action,
                        request.origin,
                        &worker_closing,
                        &mut wav_device,
                        &worker_audio_clock,
                    );
                    *worker_audio_inflight.lock().unwrap_or_else(|e|e.into_inner())=None;
                    let media_next=reply.snapshot.as_ref().filter(|_|reply.current).map(Context::from_snapshot);
                    worker_audio_clock.invalidate_if_context_changed(media_next.as_ref());
                    *worker_media_context.lock().expect("media context cache poisoned")=media_next;
                    *worker_latest.lock().expect("completion cache poisoned") = reply
                        .snapshot
                        .as_ref()
                        .filter(|_| reply.current)
                        .map(Context::from_snapshot);
                    let _ = request.sender.send(reply);
                }
            })
            .map_err(|error| format!("native authoring worker start: {error}"))?;
        Ok((
            Self {
                closing: closing.clone(),
                audio_inflight,
                thread: Some(thread),
            },
            HostHandle {
                sender,
                closing,
                media_context,
                latest,
                audio_clock,
            },
        ))
    }

    pub fn finish(mut self) -> Result<(), String> {
        self.join()
    }
    fn join(&mut self) -> Result<(), String> {
        self.closing.store(true, Ordering::Release);
        if let Some(token)=self.audio_inflight.lock().unwrap_or_else(|e|e.into_inner()).as_ref() {
            token.store(true,Ordering::Release);
        }
        if let Some(thread) = self.thread.take() {
            thread.join().map_err(|_| "native authoring worker panicked; started host effects may require reconciliation".to_string())?;
        }
        Ok(())
    }
}
impl Drop for Worker {
    fn drop(&mut self) {
        // Also join during launch unwinding; never silently detach a started job.
        if let Err(error) = self.join() {
            eprintln!("{error}");
        }
    }
}

struct BoundImage {
    context: Context,
    uri: String,
}

fn image_for_snapshot(
    image: Option<BoundImage>,
    snapshot: Option<&EditorSnapshot>,
    current: bool,
    message: &mut String,
) -> Option<String> {
    image.and_then(|image| {
        if current
            && image.context.execution_epoch.is_current()
            && snapshot.is_some_and(|now| Context::from_snapshot(now) == image.context)
        {
            Some(image.uri)
        } else {
            message.push_str("; completed frame source/session changed; withheld");
            None
        }
    })
}

#[path = "job_actions.rs"]
mod actions;
#[cfg(test)]
use actions::outcome_image;
use actions::perform;

fn execute(
    host: &mut dyn EditorHost,
    action: Action,
    origin: Context,
    cancelled: &AtomicBool,
) -> Reply {
    execute_with_audio(host, action, origin, cancelled, &mut None, &Arc::new(super::timeline_wav_device::PlaybackClock::default()))
}

fn execute_with_audio(
    host: &mut dyn EditorHost,
    action: Action,
    origin: Context,
    cancelled: &AtomicBool,
    wav_device: &mut Option<super::timeline_wav_device::WavDevice>,
    audio_clock: &Arc<super::timeline_wav_device::PlaybackClock>,
) -> Reply {
    let refresh = matches!(action, Action::Refresh);
    let mut gif_frames = None;
    let mut mp4_stream = None;
    let mut inventory = None;
    let mut workflow = None;
    let mut authoring = None;
    let mut surface = None;
    let mut viewport = None;
    let mut numeric_sheet = None;
    let mut inspection = None;
    let mut typed_10d = None;
    let mut package_workflow = None;
    let result = host.snapshot().and_then(|snapshot| {
        if !refresh {
            origin.source.check_current()?;
            if origin.authorisation_stamp.as_ref().is_some_and(|stamp|!stamp.is_current()) {
                return Err("Native authority changed; no command dispatched. Refresh authorised source.".into());
            }
        }
        if !refresh && !origin.execution_epoch.is_current() {
            return Err("Execution epoch retired; no command dispatched. Refresh source.".into());
        }
        let target = match &action {
            Action::Viewport {expected,..} | Action::InspectSurface {expected,..} | Action::BindSurfacePackage {expected,..} | Action::InspectTyped10d{expected,..} | Action::EditTyped10d{expected,..} | Action::BrushGeoRaster{expected,..} | Action::EditGeoRaster{expected,..} | Action::CreateGeoRaster{expected,..} | Action::PackageWorkflow{expected,..} | Action::InspectResource{expected,..} | Action::InspectTimelineMedia{expected,..} | Action::PreviewTimelineImage{expected,..} | Action::PrepareTimelineGif{expected,..} | Action::PrepareTimelineMp4{expected,..} | Action::AnalyzeTimelineWav{expected,..} | Action::PlayTimelineWav{expected,..} | Action::PlayTimelineWavMix{expected,..} | Action::PlayTimelineSoundField{expected,..} | Action::PauseTimelineWav{expected,..} | Action::SeekTimelineWav{expected,..} | Action::StopTimelineWav{expected,..} | Action::TimelineWavStatus{expected,..} | Action::NumericSheetInspect(expected) | Action::NumericSheetEdit{expected,..} | Action::AuthoringInspect(expected) | Action::AuthoringEdit{expected,..} | Action::CompileAssembly{expected,..} | Action::Submit { expected, .. } | Action::Undo(expected) | Action::Redo(expected) | Action::Rebuild(expected) | Action::Save { expected, .. }
            | Action::Inventory{expected,..} | Action::Workflow{expected,..} | Action::SaveInspected{expected,..} | Action::ResourcePlan{expected,..} | Action::ImportResource{expected,..} | Action::RemoveResource{expected,..} => Some(expected),
            _ => None,
        };
        if target.is_some_and(|target| target != &origin.source) {
            return Err("Command target differs from its admitted source context; no command dispatched".into());
        }
        if !refresh && Context::from_snapshot(&snapshot) != origin {
            return Err("Source/session changed before command start; no command dispatched. Refresh source.".into());
        }
        perform(host, action, &mut inventory,&mut workflow,&mut authoring,&mut inspection,&mut package_workflow, &mut typed_10d, &mut surface, &mut viewport, &mut numeric_sheet, &mut gif_frames, &mut mp4_stream, cancelled, wav_device, audio_clock)
    });
    let snapshot = host.snapshot().ok();
    // Heavy host snapshots and encoding run on this worker. The UI repeats
    // only the nonblocking opaque-stamp read immediately before publication.
    let current = snapshot.as_ref().is_some_and(|snapshot| {
        snapshot.publication_is_current()
            && host
                .snapshot()
                .is_ok_and(|now| Context::from_snapshot(&now) == Context::from_snapshot(snapshot))
    });
    let operation_succeeded = result.is_ok();
    let (mut message, bound_image) = match result {
        Ok(result) => result,
        Err(error) => (error, None),
    };
    let image_uri = image_for_snapshot(bound_image, snapshot.as_ref(), current, &mut message);
    // Read-only resource details cannot follow a successor source, session or
    // actor admitted while the job ran. Writes may legitimately advance these
    // contexts; the inspection result has no such successor contract.
    let inspection_is_current = current
        && snapshot
            .as_ref()
            .is_some_and(|now| Context::from_snapshot(now) == origin);
    if surface.is_some() && !inspection_is_current {
        message
            .push_str("; selected surface context changed after read; pixels and edges withheld");
    }
    if inspection.is_some() && !inspection_is_current {
        message.push_str("; resource inspection context changed after read; details withheld");
    }
    if package_workflow
        .as_ref()
        .is_some_and(|view| !view.publication_is_current())
        || (package_workflow.is_some() && !inspection_is_current)
    {
        package_workflow = None;
        message.push_str("; package workflow context changed; inspect before retrying");
    }
    Reply {
        gif_frames: if inspection_is_current && operation_succeeded { gif_frames } else { None },
        mp4_stream: if inspection_is_current && operation_succeeded { mp4_stream } else { None },
        numeric_sheet: if inspection_is_current { numeric_sheet } else { None },
        viewport: if inspection_is_current && image_uri.is_some() { viewport } else { None },
        surface: if inspection_is_current { surface } else { None },
        typed_10d: if inspection_is_current {
            typed_10d
        } else {
            None
        },
        package_workflow,
        inspection: if inspection_is_current {
            inspection
        } else {
            None
        },
        operation_succeeded,
        origin,
        snapshot,
        message,
        image_uri,
        current,
        inventory: if current { inventory } else { None },
        workflow: if current { workflow } else { None },
        authoring: if current { authoring } else { None },
    }
}

pub(super) fn validate_mix_requests(tracks: &[super::checkpoints::TimelineMediaRequest]) -> Result<(), String> {
    if !(2..=qualia_audio::MAX_TRACKS).contains(&tracks.len()) { return Err("Audio mix requires 2–16 tracks".into()); }
    let tick = tracks[0].timeline_tick;
    for (index, track) in tracks.iter().enumerate() {
        track.resource.validate_bounds()?;
        if track.clip_id.is_empty() || track.clip_id.len() > 128 || track.timeline_tick != tick ||
            tracks[..index].iter().any(|prior| prior.clip_id == track.clip_id) {
            return Err("Audio mix needs distinct bounded clip IDs at one timeline tick".into());
        }
    }
    Ok(())
}

pub async fn run(host: HostHandle, action: Action, origin: Context) -> Result<Reply, String> {
    if let Action::PackageWorkflow { action, .. } = &action {
        action.validate()?;
    }

    if let Action::Viewport {request,..} = &action {
        super::viewport_contract::validate_command(request)?;
    }
    match &action {
        Action::PlayTimelineSoundField { request, .. } => request.validate()?,
        Action::PlayTimelineWavMix { tracks, controls, .. } => {
            validate_mix_requests(tracks)?;
            if controls.len()!=tracks.len(){return Err("Each mix track needs controls".into());}
            for control in controls {control.validate()?;}
        },
        Action::BindSurfacePackage { request, .. } => request.validate()?,
        Action::InspectTyped10d { request, .. } => request.validate_bounds()?,
        Action::EditTyped10d { request, .. } => request.validate_bounds()?,
        Action::BrushGeoRaster { draft, .. } => draft.validate()?,
        Action::EditGeoRaster { request, .. } => request.validate_bounds()?,
        Action::CreateGeoRaster { request, .. } => request.validate_bounds()?,
        Action::InspectTimelineMedia { request, .. } | Action::PreviewTimelineImage { request, .. } | Action::PrepareTimelineGif { request, .. } | Action::PrepareTimelineMp4 { request, .. } | Action::AnalyzeTimelineWav { request, .. } | Action::PlayTimelineWav { request, .. } | Action::SeekTimelineWav { request, .. } => {
            request.resource.validate_bounds()?;
            if request.clip_id.is_empty() || request.clip_id.len() > 128 {
                return Err("Timeline clip identifier exceeds native request bounds".into());
            }
        },
        _ => {}
    }
    if let Action::InspectResource { request, .. } = &action {
        request.validate_bounds()?;
    }
    if let Action::Workflow { action, .. } = &action {
        action.validate_bounds()?;
    }
    if host.closing.load(Ordering::Acquire) {
        return Err("Native authoring worker is closing".into());
    }
    if matches!(&action, Action::Check(source) | Action::Submit { source, .. } if source.len() > MAX_REQUEST_BYTES)
    {
        return Err("Source exceeds the native request limit of 64 KiB".into());
    }
    if matches!(&action, Action::Save { name, .. } if name.len()>128) {
        return Err("Checkpoint filename exceeds native request budget".into());
    }
    match &action {
        Action::Inventory { name, .. }
        | Action::SaveInspected { name, .. }
        | Action::ResourcePlan { name, .. }
        | Action::ImportResource { name, .. }
        | Action::RemoveResource { name, .. }
            if name.len() > 128 =>
        {
            return Err("Package filename exceeds native request budget".into())
        }
        Action::ImportResource { request, .. }
            if request.key.len() > 4096
                || request.kind.len() > 4096
                || request.filename.len() > 240 =>
        {
            return Err("Resource import exceeds native request budget".into())
        }
        Action::RemoveResource { key, .. } if key.len() > 4096 => {
            return Err("Resource key exceeds native request budget".into())
        }
        Action::ResourcePlan { plan, .. } if plan.is_empty() || plan.len() > 64 => {
            return Err("Atomic resource plans require one to 64 explicit operations".into())
        }
        _ => {}
    }
    let (sender, receiver) = futures_channel::oneshot::channel();
    host.sender
        .try_send(Request {
            action,
            origin,
            sender,
        })
        .map_err(|error| match error {
            mpsc::TrySendError::Full(_) => {
                "Native authoring queue is full; no command admitted".to_string()
            }
            mpsc::TrySendError::Disconnected(_) => {
                "Native authoring worker ended; started effects may require host reconciliation"
                    .to_string()
            }
        })?;
    receiver.await.map_err(|_| "Native authoring worker ended without a result; started effects may require host reconciliation. No automatic retry.".to_string())
}

/// The UI can install a completion only into the exact view that requested it.
/// The worker separately validated the resulting source incarnation/session.
pub fn completion_is_current(
    host: &HostHandle,
    current_view: &EditorSnapshot,
    reply: &Reply,
) -> bool {
    !host.closing.load(Ordering::Acquire)
        && reply.current
        // The origin source/epoch may be intentionally retired by this exact
        // command. Preserve actor authority, then validate its current successor.
        && current_view.authorisation_stamp.as_ref().is_none_or(|stamp|stamp.is_current())
        && Context::from_snapshot(current_view) == reply.origin
        && reply.snapshot.as_ref().is_some_and(|snapshot| {
            snapshot.publication_is_current()
                && host
                    .latest
                    .try_lock()
                    .is_ok_and(|latest| latest.as_ref() == Some(&Context::from_snapshot(snapshot)))
                && qualia_core_db::poet_host::invoke::studio_store::artifact_stamp_is_current(
                    &snapshot.source.artifact_id,
                    &snapshot.source.stamp,
                )
        })
}

#[cfg(test)]
#[path = "jobs_tests.rs"]
mod tests;
