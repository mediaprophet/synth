use super::*;
use crate::graphical::authoring::{AuthoringEdit, timeline::{Clip, Timeline, TimelineEdit, Track}};
use crate::graphical::checkpoints::{ResourceInspectionRequest, TimelineMediaRequest};
use std::sync::atomic::AtomicBool;

struct TimelineMediaNoPixels;
impl PreviewProvider for TimelineMediaNoPixels {
    fn retire_retained_viewport(&mut self) -> Result<(), String> { Ok(()) }
    fn preview(&mut self, _: SourceRef, _: &Value) -> Result<PreviewFrame, String> { Err("No pixels in media resource fixture".into()) }
}

#[test]
fn admitted_timeline_media_lookup_is_explicit_guarded_and_revision_bound() {
    let temp = tempfile::tempdir().unwrap();
    for name in ["authority", "hmc", "library"] { std::fs::create_dir(temp.path().join(name)).unwrap(); }
    let (scope, mut admission, authority) = fixture(&temp.path().join("authority"));
    let author = login(&authority, "fixture-author", &AUTHOR);
    let project = NativeProject::open(config(scope, &temp.path().join("hmc"), &temp.path().join("library")), authority.clone(), author.clone(), None, Box::new(TimelineMediaNoPixels)).unwrap();
    let mut ui = project.connect(author).unwrap();
    let timeline = Timeline { version: 1, frame_ticks: 1, tracks: vec![Track { id:"track".into(), locked:false, clips:vec![Clip {
        id:"clip".into(), source:"urn:media:example".into(), source_start:300, start:10, duration:5,
    }] }], transitions: vec![] };
    let initial = ui.snapshot().unwrap();
    let adopted = ui.authoring_edit(AuthoringEdit::Timeline { edit: TimelineEdit::Adopt { timeline } }, initial.source).unwrap();
    let source = adopted.snapshot.source;
    let file = ui.save_checkpoint("media.hmc", ExpectedFile::Absent, source.clone()).unwrap();
    let data = b"fixture media bytes".to_vec();
    let digest: [u8;32] = Sha256::digest(&data).into();
    let saved = ui.edit_package_resources("media.hmc", ExpectedFile::Sha256(file.digest), vec![ResourceEdit::Add(ResourceData {
        key:"selected/picture".into(), kind:"image/png".into(), bytes:data.clone(), meta:None,
    })], source.clone()).unwrap();
    let request = TimelineMediaRequest { clip_id:"clip".into(), timeline_tick:12, resource:ResourceInspectionRequest {
        filename:"media.hmc".into(), package_digest:saved.digest, key:"selected/picture".into(), resource_digest:digest,
    }};
    let cancel = AtomicBool::new(false);
    assert!(ui.timeline_media(request.clone(), source.clone(), &cancel).is_err(), "a caller selection is not a source association");
    let associated = ui.authoring_edit(AuthoringEdit::Timeline { edit: TimelineEdit::AssociateResource {
        clip_id:"clip".into(), resource_key:"selected/picture".into(), resource_digest:digest,
    } }, source).unwrap();
    let source = associated.snapshot.source;
    let saved = ui.save_checkpoint("media.hmc", ExpectedFile::Sha256(saved.digest), source.clone()).unwrap();
    let request = TimelineMediaRequest { resource:ResourceInspectionRequest { package_digest:saved.digest, ..request.resource }, ..request };
    let selected = ui.timeline_media(request.clone(), source.clone(), &cancel).unwrap();
    assert_eq!(selected.clip_source_identifier, "urn:media:example");
    assert_eq!(selected.media_tick, 302);
    assert_eq!(selected.selected.bytes, data);
    let mut out_of_range = request.clone(); out_of_range.timeline_tick = 15;
    assert!(ui.timeline_media(out_of_range, source.clone(), &cancel).is_err());
    let mut stale_package = request.clone(); stale_package.resource.package_digest[0] ^= 1;
    assert!(ui.timeline_media(stale_package, source.clone(), &cancel).is_err());
    let mut wrong_key = request.clone(); wrong_key.resource.key = "other/picture".into();
    assert!(ui.timeline_media(wrong_key, source.clone(), &cancel).is_err());
    let mut wrong_digest = request.clone(); wrong_digest.resource.resource_digest[0] ^= 1;
    assert!(ui.timeline_media(wrong_digest, source.clone(), &cancel).is_err());
    cancel.store(true, Ordering::Release);
    assert!(ui.timeline_media(request.clone(), source.clone(), &cancel).is_err());
    cancel.store(false, Ordering::Release);
    admission.sequence += 1; admission.subjects[0].revoked = true;
    authority.replace_admission(signed(admission)).unwrap();
    assert!(ui.timeline_media(request, source, &cancel).is_err());
}
