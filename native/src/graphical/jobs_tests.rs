use super::super::controller::EditorOutcome;
use super::*;
use qualia_core_db::poet_host::invoke::studio_store::{self, StudioArtifact};
use std::sync::{atomic::AtomicUsize, Barrier, Mutex};

static NEXT: AtomicUsize = AtomicUsize::new(0);
struct ProbeHost {
    snapshot: EditorSnapshot,
    calls: Arc<Mutex<Vec<std::thread::ThreadId>>>,
    gate: Option<Arc<Barrier>>,
    inspection_source_successor: bool,
    surface_context_change: u8,
    surface_successor: Mutex<Option<EditorSnapshot>>,
}
impl EditorHost for ProbeHost {
    fn inspect_surface(
        &self,
        id: &str,
        expected: SourceRef,
    ) -> Result<super::super::surfaces::inspection::SurfaceInspection, String> {
        self.calls.lock().unwrap().push(std::thread::current().id());
        // Inject a successor only after worker origin admission and before its
        // final publication checks. No barriers or blocking test threads.
        match self.surface_context_change {
            1 => {
                let source = studio_store::get_artifact(&self.snapshot.artifact_id)
                    .ok_or("surface fixture source disappeared")?;
                studio_store::upsert_artifact(source);
            }
            2 | 3 => {
                let mut successor = self.snapshot.clone();
                if self.surface_context_change == 2 {
                    successor.session_revision += 1;
                } else {
                    successor.execution_epoch = SessionEpoch::new();
                }
                *self.surface_successor.lock().unwrap() = Some(successor);
            }
            _ => {}
        }
        Ok(super::super::surfaces::inspection::SurfaceInspection {
            source: expected,
            part_id: id.into(),
            // Bounded synthetic delivery witness; not a rendered atlas claim.
            uv_image: "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAwMCAO+jWZkAAAAASUVORK5CYII=".into(),
            texture_sha256: None,
            texture_image: None,
            summary: "Synthetic UV inspection delivery fixture".into(),
            edges: vec![[0, 1]],
        })
    }
    fn inspect_package_resource(
        &self,
        request: super::super::checkpoints::ResourceInspectionRequest,
        _: SourceRef,
        _: qualia_core_db::q42::m64_matter::M64ReadLimits,
        _: &mut [qualia_core_db::q42::m64_matter::M64PayloadInterval],
    ) -> Result<super::super::checkpoints::ResourceInspection, String> {
        self.calls.lock().unwrap().push(std::thread::current().id());
        if self.inspection_source_successor {
            // Deterministic post-admission invalidation. No two-party wait:
            // a refused dispatch must produce a bounded test failure instead
            // of leaving the caller blocked on a barrier the worker never reaches.
            let source = studio_store::get_artifact(&self.snapshot.artifact_id)
                .ok_or("inspection fixture source disappeared")?;
            studio_store::upsert_artifact(source);
        }
        // Delivery fixture only: canonical decoding has separate byte tests.
        Ok(super::super::checkpoints::ResourceInspection {
            request,
            saved_source_revision: 0,
            payload: super::super::checkpoints::M64PayloadInspection {
                header: Default::default(),
                verification: Default::default(),
                georaster: None,
                first_source_sample: None,
            },
        })
    }
    fn snapshot(&self) -> Result<EditorSnapshot, String> {
        Ok(self
            .surface_successor
            .lock()
            .unwrap()
            .clone()
            .unwrap_or_else(|| self.snapshot.clone()))
    }
    fn check(&self, source: &str) -> Result<String, String> {
        self.calls.lock().unwrap().push(std::thread::current().id());
        if source == "hold" || source == "panic" {
            if let Some(gate) = &self.gate {
                gate.wait();
                gate.wait();
            }
        }
        if source == "panic" {
            panic!("worker lifecycle probe");
        }
        Ok("checked without execution".into())
    }
    fn submit(&mut self, _: &str, _: SourceRef) -> Result<EditorOutcome, String> {
        self.snapshot.reconciliation_required = true;
        Err("host attempt failed; reconciliation required".into())
    }
    fn undo(&mut self, _: SourceRef) -> Result<EditorSnapshot, String> {
        Err("no source history".into())
    }
    fn redo(&mut self, _: SourceRef) -> Result<EditorSnapshot, String> {
        Err("no redo source".into())
    }
    fn rebuild_execution(&mut self, _: SourceRef) -> Result<EditorOutcome, String> {
        self.calls.lock().unwrap().push(std::thread::current().id());
        if let Some(gate) = &self.gate {
            gate.wait();
            gate.wait();
        }
        self.snapshot.execution_epoch = SessionEpoch::new();
        self.snapshot.reconciliation_required = false;
        Ok(EditorOutcome {
            snapshot: self.snapshot.clone(),
            message: "geometry-only rebuild completed".into(),
            frame: None,
        })
    }
}
fn fixture(gate: Option<Arc<Barrier>>) -> (ProbeHost, Arc<Mutex<Vec<std::thread::ThreadId>>>) {
    let id = format!("poet-ui-worker-{}", NEXT.fetch_add(1, Ordering::Relaxed));
    let (artifact, stamp) = studio_store::create_artifact_if_absent(StudioArtifact::new(
        &id,
        "Geometry",
        "Worker test",
    ))
    .unwrap();
    let snapshot = EditorSnapshot {
        artifact_id: id.clone(),
        title: artifact.title,
        source_revision: artifact.lamport_clock,
        source: SourceRef {
            artifact_id: id,
            stamp,
        },
        session_revision: 0,
        execution_epoch: SessionEpoch::new(),
        rebuild_supported: true,
        latest_source: String::new(),
        reconciliation_required: false,
        material_edit_id: None,
        checkpoint_supported: false,
        authorisation_stamp: None,
    };
    let calls = Arc::new(Mutex::new(Vec::new()));
    (
        ProbeHost {
            snapshot,
            calls: calls.clone(),
            gate,
            inspection_source_successor: false,
            surface_context_change: 0,
            surface_successor: Mutex::new(None),
        },
        calls,
    )
}
fn enqueue(
    handle: &HostHandle,
    action: Action,
    origin: Context,
) -> futures_channel::oneshot::Receiver<Reply> {
    let (sender, receiver) = futures_channel::oneshot::channel();
    assert!(handle
        .sender
        .try_send(Request {
            action,
            origin,
            sender
        })
        .is_ok());
    receiver
}
fn receive(mut receiver: futures_channel::oneshot::Receiver<Reply>) -> Reply {
    for _ in 0..200 {
        if let Some(reply) = receiver.try_recv().unwrap() {
            return reply;
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    panic!("worker did not deliver bounded test reply");
}

#[test]
fn one_worker_one_queue_and_cancelled_queued_work_never_starts() {
    let gate = Arc::new(Barrier::new(2));
    let (host, calls) = fixture(Some(gate.clone()));
    let origin = Context::from_snapshot(&host.snapshot);
    let (worker, handle) = Worker::start(Box::new(host)).unwrap();
    let first = enqueue(&handle, Action::Check("hold".into()), origin.clone());
    gate.wait(); // Started work is on the native worker, not this caller.
    let cancelled = enqueue(&handle, Action::Check("cancelled".into()), origin.clone());
    let (sender, _receiver) = futures_channel::oneshot::channel();
    assert!(matches!(
        handle.sender.try_send(Request {
            action: Action::Refresh,
            origin,
            sender
        }),
        Err(mpsc::TrySendError::Full(_))
    ));
    drop(cancelled);
    gate.wait();
    assert!(receive(first).current);
    worker.finish().unwrap();
    let calls = calls.lock().unwrap();
    assert_eq!(calls.len(), 1);
    assert_ne!(calls[0], std::thread::current().id());
}

#[test]
fn stale_session_or_replaced_incarnation_cannot_dispatch_or_install() {
    let (host, calls) = fixture(None);
    let view = host.snapshot.clone();
    let mut stale = Context::from_snapshot(&view);
    stale.session_revision += 1;
    let (worker, handle) = Worker::start(Box::new(host)).unwrap();
    let reply = receive(enqueue(&handle, Action::Check("unused".into()), stale));
    assert!(reply.message.contains("no command dispatched"));
    assert!(!completion_is_current(&handle, &view, &reply));
    assert!(calls.lock().unwrap().is_empty());
    // Identical id/revision replacement still has a different opaque stamp.
    let replacement = studio_store::get_artifact(&view.artifact_id).unwrap();
    studio_store::upsert_artifact(replacement);
    let reply = receive(enqueue(
        &handle,
        Action::Check("unused".into()),
        Context::from_snapshot(&view),
    ));
    assert!(!reply.current);
    assert!(!completion_is_current(&handle, &view, &reply));
    assert!(calls.lock().unwrap().is_empty());
    worker.finish().unwrap();
}

#[test]
fn failed_host_attempt_delivers_reconciliation_snapshot_without_retry() {
    let (host, _) = fixture(None);
    let view = host.snapshot.clone();
    let (worker, handle) = Worker::start(Box::new(host)).unwrap();
    let reply = receive(enqueue(
        &handle,
        Action::Submit {
            source: "effect".into(),
            expected: view.source.clone(),
        },
        Context::from_snapshot(&view),
    ));
    assert!(completion_is_current(&handle, &view, &reply));
    assert!(reply.snapshot.unwrap().reconciliation_required);
    assert!(reply.image_uri.is_none());
    assert!(reply.message.contains("host attempt failed"));
    worker.finish().unwrap();
}

#[test]
fn completed_image_cannot_follow_a_new_source_or_session_snapshot() {
    let (host, _) = fixture(None);
    let old = host.snapshot.clone();
    let image = || {
        Some(BoundImage {
            context: Context::from_snapshot(&old),
            uri: "old-frame".into(),
        })
    };
    let mut message = String::new();
    assert_eq!(
        image_for_snapshot(image(), Some(&old), true, &mut message).as_deref(),
        Some("old-frame")
    );
    let mut new_session = old.clone();
    new_session.session_revision += 1;
    assert!(image_for_snapshot(image(), Some(&new_session), true, &mut message).is_none());
    // Same artifact id and numeric revision replacement still changes incarnation.
    let artifact = studio_store::get_artifact(&old.artifact_id).unwrap();
    studio_store::upsert_artifact(artifact);
    let (_, stamp) = studio_store::get_artifact_stamped(&old.artifact_id).unwrap();
    let mut replacement = old.clone();
    replacement.source.stamp = stamp;
    assert!(image_for_snapshot(image(), Some(&replacement), true, &mut message).is_none());
    assert!(image_for_snapshot(image(), Some(&old), false, &mut message).is_none());
    assert!(message.contains("frame source/session changed; withheld"));
}

#[test]
fn publication_rechecks_actual_store_after_worker_completion() {
    let (host, _) = fixture(None);
    let view = host.snapshot.clone();
    let (worker, handle) = Worker::start(Box::new(host)).unwrap();
    let reply = Reply {
        viewport: None,
        surface: None,
        typed_10d: None,
        package_workflow: None,
        inspection: None,
        operation_succeeded: true,
        origin: Context::from_snapshot(&view),
        snapshot: Some(view.clone()),
        message: String::new(),
        image_uri: Some("frame".into()),
        current: true,
        inventory: None,
        workflow: None,
        authoring: None,
    };
    assert!(completion_is_current(&handle, &view, &reply));
    studio_store::upsert_artifact(studio_store::get_artifact(&view.artifact_id).unwrap());
    assert!(!completion_is_current(&handle, &view, &reply));
    worker.finish().unwrap();
}

#[path = "jobs_surface_tests.rs"]
mod surface_tests;

#[test]
fn resource_inspection_late_source_and_session_completions_are_withheld() {
    use crate::graphical::checkpoints::ResourceInspectionRequest;
    let request = || ResourceInspectionRequest {
        filename: "snapshot.hmc".into(),
        package_digest: [0; 32],
        key: "raster.m64".into(),
        resource_digest: [0; 32],
    };
    let (mut host, calls) = fixture(None);
    host.inspection_source_successor = true;
    let view = host.snapshot.clone();
    let (worker, handle) = Worker::start(Box::new(host)).unwrap();
    let pending = enqueue(
        &handle,
        Action::InspectResource {
            request: request(),
            expected: view.source.clone(),
        },
        Context::from_snapshot(&view),
    );
    let reply = receive(pending);
    assert!(reply.operation_succeeded);
    assert!(reply.inspection.is_none());
    assert!(!completion_is_current(&handle, &view, &reply));
    assert_eq!(calls.lock().unwrap().len(), 1);
    worker.finish().unwrap();
    let (host, _) = fixture(None);
    let view = host.snapshot.clone();
    let (worker, handle) = Worker::start(Box::new(host)).unwrap();
    let reply = receive(enqueue(
        &handle,
        Action::InspectResource {
            request: request(),
            expected: view.source.clone(),
        },
        Context::from_snapshot(&view),
    ));
    assert!(reply.inspection.is_some());
    assert!(completion_is_current(&handle, &view, &reply));
    let mut switched = view.clone();
    switched.session_revision += 1;
    assert!(!completion_is_current(&handle, &switched, &reply));
    // Use the supported worker rebuild lifecycle. The fixture replaces its
    // epoch; production retirement itself is covered below by the actual
    // CanonicalEditorHost source/rebuild test, without exposing private APIs.
    let rebuilt = receive(enqueue(
        &handle,
        Action::Rebuild(view.source.clone()),
        Context::from_snapshot(&view),
    ));
    assert_ne!(
        rebuilt.snapshot.as_ref().unwrap().execution_epoch,
        view.execution_epoch
    );
    assert!(!completion_is_current(&handle, &view, &reply));
    worker.finish().unwrap();
}

#[test]
fn rebuilt_epoch_rejects_old_queued_reply_and_frame_even_when_revision_repeats() {
    let (host, calls) = fixture(None);
    let view = host.snapshot.clone();
    let origin = Context::from_snapshot(&view);
    let (worker, handle) = Worker::start(Box::new(host)).unwrap();
    let old_reply = receive(enqueue(
        &handle,
        Action::Check("old".into()),
        origin.clone(),
    ));
    assert!(completion_is_current(&handle, &view, &old_reply));
    let rebuilt = receive(enqueue(
        &handle,
        Action::Rebuild(view.source.clone()),
        origin.clone(),
    ));
    let new = rebuilt.snapshot.as_ref().unwrap();
    assert_eq!(new.source, view.source);
    assert_eq!(new.session_revision, view.session_revision);
    assert_ne!(new.execution_epoch, view.execution_epoch);
    assert!(completion_is_current(&handle, &view, &rebuilt));
    assert!(!completion_is_current(&handle, &view, &old_reply));
    let image = Some(BoundImage {
        context: origin.clone(),
        uri: "old-epoch-frame".into(),
    });
    assert!(image_for_snapshot(image, Some(new), true, &mut String::new()).is_none());
    let queued_old = receive(enqueue(&handle, Action::Check("old queued".into()), origin));
    assert!(queued_old.message.contains("no command dispatched"));
    assert_eq!(calls.lock().unwrap().len(), 2); // one check, one rebuild
    worker.finish().unwrap();
}

#[test]
fn actual_source_successor_and_retired_rebuild_epoch_publish_into_the_requesting_view() {
    use crate::graphical::{
        controller::{CanonicalEditorHost, PreviewProvider},
        geometry_host::GeometryHost,
        preview::PreviewFrame,
    };
    struct NoPixels;
    impl PreviewProvider for NoPixels {
        fn preview(&mut self, _: SourceRef, _: &vibe::Value) -> Result<PreviewFrame, String> {
            Err("headless source lifecycle test; no pixels claimed".into())
        }
    }
    let (probe, _) = fixture(None);
    let host = CanonicalEditorHost::open(
        probe.snapshot.artifact_id.clone(),
        vibe::session::SessionLimits::default(),
        GeometryHost::default(),
        Box::new(NoPixels),
    )
    .unwrap();
    let initial = host.snapshot().unwrap();
    let (worker, handle) = Worker::start(Box::new(host)).unwrap();
    let submitted = receive(enqueue(
        &handle,
        Action::Submit {
            source: "let ready = 1; ready;".into(),
            expected: initial.source.clone(),
        },
        Context::from_snapshot(&initial),
    ));
    let accepted = submitted.snapshot.as_ref().unwrap().clone();
    assert_ne!(accepted.source, initial.source);
    assert!(!initial.publication_is_current());
    assert!(accepted.publication_is_current());
    assert!(completion_is_current(&handle, &initial, &submitted));
    let undone = receive(enqueue(
        &handle,
        Action::Undo(accepted.source.clone()),
        Context::from_snapshot(&accepted),
    ));
    let held = undone.snapshot.as_ref().unwrap().clone();
    assert!(completion_is_current(&handle, &accepted, &undone));
    assert!(!completion_is_current(&handle, &initial, &submitted));
    let rebuilt = receive(enqueue(
        &handle,
        Action::Rebuild(held.source.clone()),
        Context::from_snapshot(&held),
    ));
    assert!(!held.execution_epoch.is_current());
    assert!(completion_is_current(&handle, &held, &rebuilt));
    let current = rebuilt.snapshot.as_ref().unwrap();
    studio_store::upsert_artifact(studio_store::get_artifact(&current.artifact_id).unwrap());
    assert!(!completion_is_current(&handle, &held, &rebuilt));
    worker.finish().unwrap();
    assert!(!completion_is_current(&handle, &held, &rebuilt));
}

#[test]
fn held_state_transition_rejects_prior_success_with_unchanged_epoch_revision_and_source() {
    let (host, _) = fixture(None);
    let view = host.snapshot.clone();
    let origin = Context::from_snapshot(&view);
    let (worker, handle) = Worker::start(Box::new(host)).unwrap();
    let prior = receive(enqueue(
        &handle,
        Action::Check("valid".into()),
        origin.clone(),
    ));
    let held = receive(enqueue(
        &handle,
        Action::Submit {
            source: "attempt".into(),
            expected: view.source.clone(),
        },
        origin,
    ));
    let now = held.snapshot.as_ref().unwrap();
    assert_eq!(now.execution_epoch, view.execution_epoch);
    assert_eq!(now.session_revision, view.session_revision);
    assert_eq!(now.source, view.source);
    assert!(now.reconciliation_required);
    assert!(!completion_is_current(&handle, &view, &prior));
    assert!(completion_is_current(&handle, &view, &held));
    worker.finish().unwrap();
}

#[test]
fn unsupported_profile_has_no_rebuild_dispatch() {
    let (mut host, calls) = fixture(None);
    host.snapshot.rebuild_supported = false;
    let origin = Context::from_snapshot(&host.snapshot);
    let target = host.snapshot.source.clone();
    let (worker, handle) = Worker::start(Box::new(host)).unwrap();
    let reply = receive(enqueue(&handle, Action::Rebuild(target), origin));
    assert!(reply.message.contains("unsupported"));
    assert!(calls.lock().unwrap().is_empty());
    worker.finish().unwrap();
}

#[test]
fn preview_with_another_execution_epoch_is_withheld_before_encoding() {
    let (host, _) = fixture(None);
    let snapshot = host.snapshot.clone();
    let frame = super::super::preview::PreviewFrame {
        source: snapshot.source.clone(),
        session_revision: Some(snapshot.session_revision),
        execution_epoch: Some(SessionEpoch::new()),
        side: 8,
        rgba8: vec![0; 8 * 8 * 4],
    };
    let (message, image) = outcome_image(EditorOutcome {
        snapshot,
        message: "result".into(),
        frame: Some(frame),
    })
    .unwrap();
    assert!(image.is_none());
    assert!(message.contains("mismatch; withheld"));
}

#[test]
fn closing_joins_started_rebuild_and_cancels_queued_command() {
    let gate = Arc::new(Barrier::new(2));
    let (host, calls) = fixture(Some(gate.clone()));
    let origin = Context::from_snapshot(&host.snapshot);
    let target = host.snapshot.source.clone();
    let (worker, handle) = Worker::start(Box::new(host)).unwrap();
    let first = enqueue(&handle, Action::Rebuild(target), origin.clone());
    gate.wait();
    let mut queued = enqueue(&handle, Action::Check("queued".into()), origin);
    let finished = Arc::new(AtomicBool::new(false));
    let finished_thread = finished.clone();
    let joined = std::thread::spawn(move || {
        worker.finish().unwrap();
        finished_thread.store(true, Ordering::Release);
    });
    for _ in 0..200 {
        if handle.closing.load(Ordering::Acquire) {
            break;
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    assert!(handle.closing.load(Ordering::Acquire));
    assert!(!finished.load(Ordering::Acquire));
    gate.wait();
    joined.join().unwrap();
    assert!(receive(first).current);
    assert!(queued.try_recv().is_err());
    assert_eq!(calls.lock().unwrap().len(), 1);
}

#[test]
fn worker_panic_closes_reply_and_is_reported_when_owner_joins() {
    let gate = Arc::new(Barrier::new(2));
    let (host, calls) = fixture(Some(gate.clone()));
    let origin = Context::from_snapshot(&host.snapshot);
    let (worker, handle) = Worker::start(Box::new(host)).unwrap();
    let mut receiver = enqueue(&handle, Action::Check("panic".into()), origin);
    gate.wait();
    gate.wait();
    assert!(worker.finish().is_err());
    assert!(receiver.try_recv().is_err());
    assert_eq!(calls.lock().unwrap().len(), 1);
}
