use super::*;
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT: AtomicUsize = AtomicUsize::new(0);

#[test]
fn selected_media_refuses_stale_mismatch_kind_budget_and_cancellation() {
    let root = tempfile::tempdir().unwrap();
    let id = format!("urn:poet:media-read:{}:{}", std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed));
    let (_, stamp) = studio_store::create_artifact_if_absent(StudioArtifact::new(&id, "Geometry", "Media fixture")).unwrap();
    let expected = SourceRef { artifact_id: id.clone(), stamp: stamp.clone() };
    let limits = CheckpointLimits::default();
    let checkpoint = core::capture(&id, &stamp, limits).unwrap().apply_resource_edits(vec![
        ResourceEdit::Add(ResourceData { key: "media/image".into(), kind: "image/png".into(), bytes: b"fixture media bytes".to_vec(), meta: None }),
        ResourceEdit::Add(ResourceData { key: "data/raw".into(), kind: "data/raw".into(), bytes: b"nonmedia".to_vec(), meta: None }),
    ], limits).unwrap();
    let inventory = checkpoint.inventory();
    let image_digest = inventory.iter().find(|r| r.key == "media/image").unwrap().digest;
    let raw_digest = inventory.iter().find(|r| r.key == "data/raw").unwrap().digest;
    let bytes = core::encode(&checkpoint, vec![], limits).unwrap();
    let receipt = core::save(&root.path().join("media.hmc"), &bytes, ExpectedFile::Absent, limits).unwrap();
    let service = CheckpointService::new(root.path(), TrustedBinding {
        project_id: "fixture-project".into(), document_id: "fixture-document".into(), source_id: id.clone(), domain: "Geometry".into(),
    }, limits).unwrap();
    let request = ResourceInspectionRequest { filename: "media.hmc".into(), package_digest: receipt.digest, key: "media/image".into(), resource_digest: image_digest };
    let cancel = AtomicBool::new(false);
    let result = service.read_selected_media(request.clone(), &expected, 100, &cancel).unwrap();
    assert_eq!(result.kind, "image/png");
    assert_eq!(result.bytes, b"fixture media bytes");
    assert!(service.read_selected_media(request.clone(), &expected, 1, &cancel).is_err());
    let mut changed = request.clone(); changed.package_digest[0] ^= 1;
    assert!(service.read_selected_media(changed, &expected, 100, &cancel).is_err());
    let mut changed = request.clone(); changed.resource_digest[0] ^= 1;
    assert!(service.read_selected_media(changed, &expected, 100, &cancel).is_err());
    let mut changed = request.clone(); changed.key = "data/raw".into(); changed.resource_digest = raw_digest;
    assert!(service.read_selected_media(changed, &expected, 100, &cancel).is_err());
    cancel.store(true, Ordering::Release);
    assert!(service.read_selected_media(request.clone(), &expected, 100, &cancel).is_err());
    cancel.store(false, Ordering::Release);
    studio_store::mutate_artifact_stamped(&id, &stamp, |artifact| { artifact.title = "changed".into(); Ok(()) }).unwrap();
    assert!(service.read_selected_media(request, &expected, 100, &cancel).is_err());
}
