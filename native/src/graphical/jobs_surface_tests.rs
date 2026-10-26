//! Selected-surface payload publication through the actual owned native worker.
//! The host deliberately changes context after admission; no atlas/render claim.
use super::*;

#[test]
fn surface_inspection_late_source_session_and_epoch_payloads_are_withheld() {
    for change in [1, 2, 3] {
        let (mut host, calls) = fixture(None);
        host.surface_context_change = change;
        let view = host.snapshot.clone();
        let origin = Context::from_snapshot(&view);
        let (worker, handle) = Worker::start(Box::new(host)).unwrap();
        let reply = receive(enqueue(
            &handle,
            Action::InspectSurface {
                id: "fixture-part".into(),
                expected: view.source.clone(),
            },
            origin.clone(),
        ));
        assert!(
            reply.operation_succeeded,
            "admitted read must execute before successor injection"
        );
        assert!(
            reply.surface.is_none(),
            "surface pixels/edges escaped successor context {change}"
        );
        assert_eq!(calls.lock().unwrap().len(), 1);
        if change == 1 {
            assert!(!reply.current);
            assert!(!completion_is_current(&handle, &view, &reply));
        } else {
            let successor = reply.snapshot.as_ref().unwrap();
            assert!(Context::from_snapshot(successor) != origin);
            assert_eq!(successor.source, view.source);
            if change == 2 {
                assert_eq!(successor.session_revision, view.session_revision + 1);
            } else {
                assert_eq!(successor.session_revision, view.session_revision);
                assert_ne!(successor.execution_epoch, view.execution_epoch);
            }
        }
        worker.finish().unwrap();
    }
}

#[test]
fn current_surface_inspection_publishes_once_and_old_completion_cannot_follow_rebuild() {
    let (host, calls) = fixture(None);
    let view = host.snapshot.clone();
    let (worker, handle) = Worker::start(Box::new(host)).unwrap();
    let reply = receive(enqueue(
        &handle,
        Action::InspectSurface {
            id: "fixture-part".into(),
            expected: view.source.clone(),
        },
        Context::from_snapshot(&view),
    ));
    assert!(reply.operation_succeeded);
    assert!(completion_is_current(&handle, &view, &reply));
    let surface = reply.surface.as_ref().unwrap();
    assert_eq!(surface.source, view.source);
    assert_eq!(surface.part_id, "fixture-part");
    assert_eq!(surface.edges, vec![[0, 1]]);
    assert!(surface.uv_image.starts_with("data:image/png;base64,"));
    let mut switched = view.clone();
    switched.session_revision += 1;
    assert!(!completion_is_current(&handle, &switched, &reply));
    let rebuilt = receive(enqueue(
        &handle,
        Action::Rebuild(view.source.clone()),
        Context::from_snapshot(&view),
    ));
    assert!(rebuilt.operation_succeeded);
    assert!(completion_is_current(&handle, &view, &rebuilt));
    assert!(!completion_is_current(&handle, &view, &reply));
    assert_eq!(calls.lock().unwrap().len(), 2);
    worker.finish().unwrap();
}
