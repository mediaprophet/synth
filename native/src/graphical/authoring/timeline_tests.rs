use super::timeline::{self, Clip, Timeline, TimelineEdit, Track, Transition};
use vibe::{session::SourceSession, Budget, LocalHost};

fn example(locked: bool) -> Timeline {
    Timeline { version: 1, frame_ticks: 10, tracks: vec![Track { id:"track1".into(), locked, clips: vec![
        Clip { id:"a".into(), source:"media:a".into(), source_start:300, start:0, duration:100 },
        Clip { id:"b".into(), source:"media:b".into(), source_start:700, start:100, duration:100 },
    ]}], transitions: vec![Transition { id:"x".into(),kind:"cross_dissolve".into(),track:"track1".into(),from:"a".into(),to:"b".into(),start:90,duration:20 }] }
}

#[test]
fn vibe_source_round_trip_preserves_source_offsets_and_transition_edit() {
    let mut session=SourceSession::default(); let mut host=LocalHost::default();
    session.submit("let mut poet_parts = [];", &mut host, Budget::default()).unwrap();
    let create=timeline::edit_source(None,&TimelineEdit::Adopt{timeline:example(false)}).unwrap();
    session.submit(&create,&mut host,Budget::default()).unwrap();
    assert_eq!(timeline::read(session.value("poet_timeline")).unwrap(),Some(example(false)));
    let edit=timeline::edit_source(session.value("poet_timeline"),&TimelineEdit::SetTransition{id:"x".into(),start:80,duration:40}).unwrap();
    session.submit(&edit,&mut host,Budget::default()).unwrap();
    let actual=timeline::read(session.value("poet_timeline")).unwrap().unwrap();
    assert_eq!(actual.tracks[0].clips[0].source_start,300);
    assert_eq!(actual.tracks[0].clips[1].start,100);
    assert_eq!((actual.transitions[0].start,actual.transitions[0].duration),(80,40));
}

#[test]
fn locked_track_refuses_edit_without_mutating_source() {
    let mut session=SourceSession::default(); let mut host=LocalHost::default();
    session.submit("let mut poet_parts = [];",&mut host,Budget::default()).unwrap();
    let create=timeline::edit_source(None,&TimelineEdit::Adopt{timeline:example(true)}).unwrap();
    session.submit(&create,&mut host,Budget::default()).unwrap();
    let before=session.value("poet_timeline").cloned();
    assert!(timeline::edit_source(before.as_ref(),&TimelineEdit::SetTransition{id:"x".into(),start:80,duration:40}).is_err());
    assert_eq!(before,session.value("poet_timeline").cloned());
}

#[test]
fn rejects_bad_cut_overflow_and_duplicate_identity() {
    let mut t=example(false); t.tracks[0].clips[1].start=120;
    assert!(t.validate().is_err());
    let mut t=example(false); t.tracks[0].clips[0].source_start=i64::MAX as u64;
    assert!(t.validate().is_err());
    let mut t=example(false); t.tracks[0].clips[1].id="a".into();
    assert!(t.validate().is_err());
    let mut t=example(false); t.transitions[0].duration=1;
    assert!(t.validate().is_err());
}

#[test]
fn media_identifier_round_trips_iri_and_cannot_inject_source() {
    let mut original=example(false);
    original.tracks[0].clips[0].source="https://example.org/物種?x=%20#音声\";poet_parts=[];//\\tail".into();
    let mut session=SourceSession::default(); let mut host=LocalHost::default();
    session.submit("let mut poet_parts = [1];",&mut host,Budget::default()).unwrap();
    let text=timeline::edit_source(None,&TimelineEdit::Adopt{timeline:original.clone()}).unwrap();
    session.submit(&text,&mut host,Budget::default()).unwrap();
    assert_eq!(timeline::read(session.value("poet_timeline")).unwrap(),Some(original));
    assert_eq!(session.value("poet_parts"),Some(&vibe::Value::List(vec![vibe::Value::I64(1)])));
}

#[test]
fn media_identifier_rejects_control_characters_and_excess_length() {
    let mut value=example(false); value.tracks[0].clips[0].source="bad\nidentifier".into();
    assert!(value.validate().is_err());
    value.tracks[0].clips[0].source="x".repeat(2049);
    assert!(value.validate().is_err());
}

#[test]
fn timeline_only_source_does_not_require_poet_parts() {
    let mut session=SourceSession::default(); let mut host=LocalHost::default();
    let text=timeline::edit_source_with_output(None,&TimelineEdit::Adopt{timeline:example(false)},false).unwrap();
    session.submit(&text,&mut host,Budget::default()).unwrap();
    assert!(session.value("poet_parts").is_none());
    assert_eq!(timeline::read(session.value("poet_timeline")).unwrap(),Some(example(false)));
}

fn apply_timeline(session:&mut SourceSession, host:&mut LocalHost, edit:TimelineEdit) -> Result<(),String> {
    let text=timeline::edit_source_with_output(session.value("poet_timeline"),&edit,false)?;
    session.submit(&text,host,Budget::default()).map_err(|e|format!("{e:?}"))?; Ok(())
}

#[test]
fn timeline_creates_tracks_clips_and_transition_through_source_history() {
    let mut session=SourceSession::default(); let mut host=LocalHost::default();
    apply_timeline(&mut session,&mut host,TimelineEdit::Adopt{timeline:Timeline{version:1,frame_ticks:10,tracks:vec![],transitions:vec![]}}).unwrap();
    apply_timeline(&mut session,&mut host,TimelineEdit::AddTrack{id:"track1".into()}).unwrap();
    for clip in example(false).tracks[0].clips.clone() {
        apply_timeline(&mut session,&mut host,TimelineEdit::AddClip{track:"track1".into(),clip}).unwrap();
    }
    apply_timeline(&mut session,&mut host,TimelineEdit::AddTransition{transition:example(false).transitions[0].clone()}).unwrap();
    assert_eq!(timeline::read(session.value("poet_timeline")).unwrap(),Some(example(false)));
    let before=session.value("poet_timeline").cloned();
    assert!(apply_timeline(&mut session,&mut host,TimelineEdit::AddTrack{id:"track1".into()}).is_err());
    assert!(apply_timeline(&mut session,&mut host,TimelineEdit::AddClip{track:"track1".into(),clip:example(false).tracks[0].clips[0].clone()}).is_err());
    assert!(apply_timeline(&mut session,&mut host,TimelineEdit::AddTransition{transition:example(false).transitions[0].clone()}).is_err());
    assert_eq!(before,session.value("poet_timeline").cloned());
}

#[test]
fn locked_track_refuses_clip_and_transition_creation() {
    let mut session=SourceSession::default(); let mut host=LocalHost::default();
    let locked=example(true); let before_clip=locked.tracks[0].clips[0].clone();
    apply_timeline(&mut session,&mut host,TimelineEdit::Adopt{timeline:locked}).unwrap();
    let before=session.value("poet_timeline").cloned();
    let mut clip=before_clip; clip.id="new_clip".into();
    assert!(apply_timeline(&mut session,&mut host,TimelineEdit::AddClip{track:"track1".into(),clip}).is_err());
    let mut transition=example(false).transitions[0].clone(); transition.id="new_transition".into();
    assert!(apply_timeline(&mut session,&mut host,TimelineEdit::AddTransition{transition}).is_err());
    assert_eq!(before,session.value("poet_timeline").cloned());
}

#[test]
fn timeline_edits_preserve_dependencies_and_enforce_track_locks() {
    let mut session=SourceSession::default(); let mut host=LocalHost::default();
    apply_timeline(&mut session,&mut host,TimelineEdit::Adopt{timeline:example(false)}).unwrap();
    let initial=session.value("poet_timeline").cloned();
    assert!(apply_timeline(&mut session,&mut host,TimelineEdit::RemoveClip{track:"track1".into(),id:"a".into()}).is_err());
    assert!(apply_timeline(&mut session,&mut host,TimelineEdit::RemoveTrack{id:"track1".into()}).is_err());
    let mut invalid=example(false).tracks[0].clips[0].clone(); invalid.duration=50;
    assert!(apply_timeline(&mut session,&mut host,TimelineEdit::UpdateClip{track:"track1".into(),clip:invalid}).is_err());
    assert_eq!(initial,session.value("poet_timeline").cloned());
    apply_timeline(&mut session,&mut host,TimelineEdit::SetTrackLocked{id:"track1".into(),locked:true}).unwrap();
    let locked=session.value("poet_timeline").cloned();
    assert!(apply_timeline(&mut session,&mut host,TimelineEdit::RemoveTransition{id:"x".into()}).is_err());
    let mut changed=example(false).tracks[0].clips[0].clone(); changed.source_start=400;
    assert!(apply_timeline(&mut session,&mut host,TimelineEdit::UpdateClip{track:"track1".into(),clip:changed.clone()}).is_err());
    assert_eq!(locked,session.value("poet_timeline").cloned());
    apply_timeline(&mut session,&mut host,TimelineEdit::SetTrackLocked{id:"track1".into(),locked:false}).unwrap();
    apply_timeline(&mut session,&mut host,TimelineEdit::UpdateClip{track:"track1".into(),clip:changed}).unwrap();
    let updated=timeline::read(session.value("poet_timeline")).unwrap().unwrap();
    assert_eq!(updated.tracks[0].clips[0].source_start,400);
    apply_timeline(&mut session,&mut host,TimelineEdit::RemoveTransition{id:"x".into()}).unwrap();
    for id in ["a","b"] { apply_timeline(&mut session,&mut host,TimelineEdit::RemoveClip{track:"track1".into(),id:id.into()}).unwrap(); }
    apply_timeline(&mut session,&mut host,TimelineEdit::RemoveTrack{id:"track1".into()}).unwrap();
    let empty=timeline::read(session.value("poet_timeline")).unwrap().unwrap();
    assert!(empty.tracks.is_empty() && empty.transitions.is_empty());
}
