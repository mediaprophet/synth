use super::{timeline::{Clip,Timeline,TimelineEdit,Track},timeline_associations as media};
use vibe::{session::SourceSession,LocalHost,Budget,Value};
fn timeline()->Timeline{Timeline{version:1,frame_ticks:1,tracks:vec![Track{id:"t".into(),locked:false,clips:vec![Clip{id:"c".into(),source:"https://example.test/物種#source".into(),source_start:7,start:0,duration:10}]}],transitions:vec![]}}

#[test]
fn explicit_association_is_source_owned_and_distinguishes_clip_source_key_digest(){
    let mut session=SourceSession::default();let mut host=LocalHost::default();
    let initial=super::timeline::edit_source_with_output(None,&TimelineEdit::Adopt{timeline:timeline()},false).unwrap();
    session.submit(&initial,&mut host,Budget::default()).unwrap();
    let cell=media::bind(None,&timeline(),"c","assets/音声\".wav",[7;32],false).unwrap();
    session.submit(&cell,&mut host,Budget::default()).unwrap();
    let bindings=media::read(session.value("poet_timeline_media"),&timeline()).unwrap();
    assert_eq!(bindings.len(),1);assert_eq!(bindings[0].clip_id,"c");assert_eq!(bindings[0].source_identifier,"https://example.test/物種#source");
    assert_eq!(bindings[0].resource_key,"assets/音声\".wav");assert_eq!(bindings[0].resource_digest,[7;32]);
    assert!(media::bind(session.value("poet_timeline_media"),&timeline(),"c","other",[1;32],false).is_err());
    let mut changed=timeline();changed.tracks[0].clips[0].source="different".into();
    assert!(media::read(session.value("poet_timeline_media"),&changed).is_err());
    let unbind=media::unbind(session.value("poet_timeline_media"),&timeline(),"c",false).unwrap();
    session.submit(&unbind,&mut host,Budget::default()).unwrap();
    assert!(media::read(session.value("poet_timeline_media"),&timeline()).unwrap().is_empty());
}
#[test]
fn locked_or_unknown_clip_and_bad_resource_are_refused(){
    let mut locked=timeline();locked.tracks[0].locked=true;
    assert!(media::bind(None,&locked,"c","resource",[1;32],false).is_err());
    assert!(media::bind(None,&timeline(),"absent","resource",[1;32],false).is_err());
    assert!(media::bind(None,&timeline(),"c","resource",[0;32],false).is_err());
    assert!(media::bind(None,&timeline(),"c","bad\nkey",[1;32],false).is_err());
    assert!(media::validate_pair(None,Some(&Value::Null)).is_err());
}
