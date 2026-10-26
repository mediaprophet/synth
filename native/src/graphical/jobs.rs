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

pub struct Reply {
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
    thread: Option<JoinHandle<()>>,
}
impl Worker {
    pub fn start(mut host: Box<dyn EditorHost>) -> Result<(Self, HostHandle), String> {
        let (sender, receiver) = mpsc::sync_channel::<Request>(1);
        let closing = Arc::new(AtomicBool::new(false));
        let worker_closing = closing.clone();
        let latest = Arc::new(Mutex::new(Some(Context::from_snapshot(&host.snapshot()?))));
        let worker_latest = latest.clone();
        let thread = std::thread::Builder::new()
            .name("poet-native-authoring".into())
            .spawn(move || {
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
                    // Invalidate already-sent completions while the next job runs.
                    *worker_latest.lock().expect("completion cache poisoned") = None;
                    let reply = execute(
                        host.as_mut(),
                        request.action,
                        request.origin,
                        &worker_closing,
                    );
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
                thread: Some(thread),
            },
            HostHandle {
                sender,
                closing,
                latest,
            },
        ))
    }

    pub fn finish(mut self) -> Result<(), String> {
        self.join()
    }
    fn join(&mut self) -> Result<(), String> {
        self.closing.store(true, Ordering::Release);
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
    let refresh = matches!(action, Action::Refresh);
    let mut inventory = None;
    let mut workflow = None;
    let mut authoring = None;
    let mut surface = None;
    let mut viewport = None;
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
            Action::Viewport {expected,..} | Action::InspectSurface {expected,..} | Action::BindSurfacePackage {expected,..} | Action::InspectTyped10d{expected,..} | Action::EditTyped10d{expected,..} | Action::EditGeoRaster{expected,..} | Action::CreateGeoRaster{expected,..} | Action::PackageWorkflow{expected,..} | Action::InspectResource{expected,..} | Action::AuthoringInspect(expected) | Action::AuthoringEdit{expected,..} | Action::CompileAssembly{expected,..} | Action::Submit { expected, .. } | Action::Undo(expected) | Action::Redo(expected) | Action::Rebuild(expected) | Action::Save { expected, .. }
            | Action::Inventory{expected,..} | Action::Workflow{expected,..} | Action::SaveInspected{expected,..} | Action::ResourcePlan{expected,..} | Action::ImportResource{expected,..} | Action::RemoveResource{expected,..} => Some(expected),
            _ => None,
        };
        if target.is_some_and(|target| target != &origin.source) {
            return Err("Command target differs from its admitted source context; no command dispatched".into());
        }
        if !refresh && Context::from_snapshot(&snapshot) != origin {
            return Err("Source/session changed before command start; no command dispatched. Refresh source.".into());
        }
        perform(host, action, &mut inventory,&mut workflow,&mut authoring,&mut inspection,&mut package_workflow, &mut typed_10d, &mut surface, &mut viewport, cancelled)
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

pub async fn run(host: HostHandle, action: Action, origin: Context) -> Result<Reply, String> {
    if let Action::PackageWorkflow { action, .. } = &action {
        action.validate()?;
    }

    if let Action::Viewport {request,..} = &action {
        super::viewport_contract::validate_command(request)?;
    }
    match &action {
        Action::BindSurfacePackage { request, .. } => request.validate()?,
        Action::InspectTyped10d { request, .. } => request.validate_bounds()?,
        Action::EditTyped10d { request, .. } => request.validate_bounds()?,
        Action::EditGeoRaster { request, .. } => request.validate_bounds()?,
        Action::CreateGeoRaster { request, .. } => request.validate_bounds()?,
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
