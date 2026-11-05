use super::super::timeline::{Clip, TimelineEdit, Track};
use super::*;
use vibe::{session::SourceSession, Budget, LocalHost};

fn timeline() -> Timeline {
    Timeline {
        version: 1,
        frame_ticks: 1,
        tracks: vec![Track {
            id: "audio".into(),
            locked: false,
            clips: vec![
                Clip {
                    id: "a".into(),
                    source: "https://example.test/物種#audio".into(),
                    source_start: 14,
                    start: 28,
                    duration: 42,
                },
                Clip {
                    id: "b".into(),
                    source: "urn:source:b".into(),
                    source_start: 0,
                    start: 70,
                    duration: 28,
                },
            ],
        }],
        transitions: vec![],
    }
}
fn set(clip_id: &str, entity: &str) -> SoundFieldEdit {
    SoundFieldEdit::SetClip {
        clip_id: clip_id.into(),
        entity_identifier: entity.into(),
        position_m: [2., 0., -1.],
        radius_m: 25.,
        gain: 0.25,
        priority: -3,
    }
}
#[test]
fn authored_source_roundtrip_and_retained_cell_replay_preserve_exact_clocks_and_field() {
    let mut session = SourceSession::default();
    let mut host = LocalHost::default();
    let timeline_cell = super::super::timeline::edit_source_with_output(
        None,
        &TimelineEdit::Adopt {
            timeline: timeline(),
        },
        false,
    )
    .unwrap();
    session
        .submit(&timeline_cell, &mut host, Budget::default())
        .unwrap();
    let field_cell =
        edit_source(None, &timeline(), &set("a", "urn:entity:bird/物種"), false).unwrap();
    session
        .submit(&field_cell, &mut host, Budget::default())
        .unwrap();
    let changed = edit_source(
        session.value("poet_sound_field"),
        &timeline(),
        &SoundFieldEdit::SetListener {
            position_m: [1., 2., 3.],
            yaw_rad: 0.5,
        },
        false,
    )
    .unwrap();
    session
        .submit(&changed, &mut host, Budget::default())
        .unwrap();
    let expected = read(session.value("poet_sound_field"), &timeline())
        .unwrap()
        .unwrap();
    assert_eq!(
        expected.bindings[0].source_identifier,
        "https://example.test/物種#audio"
    );
    assert_eq!(
        expected.bindings[0].entity_identifier,
        "urn:entity:bird/物種"
    );
    let mut reopened = SourceSession::default();
    let mut fresh_host = LocalHost::default();
    for cell in [&timeline_cell, &field_cell, &changed] {
        reopened
            .submit(cell, &mut fresh_host, Budget::default())
            .unwrap();
    }
    assert_eq!(
        read(reopened.value("poet_sound_field"), &timeline()).unwrap(),
        Some(expected)
    );
    assert_eq!(
        super::super::timeline::read(reopened.value("poet_timeline")).unwrap(),
        Some(timeline())
    );
    let previous = reopened.value("poet_sound_field").cloned();
    let stale = field_cell
        .replace("let mut poet_sound_field", "poet_sound_field")
        .replace("https://example.test/物種#audio", "urn:stale-source");
    let refused = reopened.submit_validated_bindings(
        &stale,
        &mut fresh_host,
        Budget::default(),
        |_, bindings, _| {
            validate_pair(
                bindings.value("poet_timeline"),
                bindings.value("poet_sound_field"),
            )
            .map_err(|message| {
                vibe::Diagnostic::new(vibe::DiagCode::E701, vibe::Span::point(0), message)
            })
        },
    );
    assert!(refused.is_err());
    assert_eq!(reopened.value("poet_sound_field").cloned(), previous);
}
#[test]
fn locked_stale_missing_duplicate_and_numeric_assignments_are_refused() {
    let mut locked = timeline();
    locked.tracks[0].locked = true;
    assert!(edit_source(None, &locked, &set("a", "entity"), false).is_err());
    assert!(edit_source(None, &timeline(), &set("absent", "entity"), false).is_err());
    assert!(edit_source(
        None,
        &timeline(),
        &SoundFieldEdit::SetListener {
            position_m: [f64::NAN, 0., 0.],
            yaw_rad: 0.
        },
        false
    )
    .is_err());
    let mut field = SoundField {
        listener_position_m: [0.; 3],
        listener_yaw_rad: 0.,
        bindings: vec![ClipEmitter {
            clip_id: "a".into(),
            source_identifier: timeline().tracks[0].clips[0].source.clone(),
            entity_identifier: "entity".into(),
            position_m: [0.; 3],
            radius_m: 25.,
            gain: 0.5,
            priority: 0,
        }],
    };
    field.validate(&timeline()).unwrap();
    field.bindings.push(field.bindings[0].clone());
    assert!(field.validate(&timeline()).is_err());
    field.bindings.pop();
    field.bindings[0].source_identifier = "changed".into();
    assert!(field.validate(&timeline()).is_err());
    field.bindings[0].source_identifier = timeline().tracks[0].clips[0].source.clone();
    field.bindings[0].gain = f64::INFINITY;
    assert!(field.validate(&timeline()).is_err());
    assert!(validate_pair(None, Some(&Value::Null)).is_err());
}
#[test]
fn conflicting_shared_entity_placement_requires_an_explicit_edit() {
    let mut session = SourceSession::default();
    let mut host = LocalHost::default();
    let initial = super::super::timeline::edit_source_with_output(
        None,
        &TimelineEdit::Adopt {
            timeline: timeline(),
        },
        false,
    )
    .unwrap();
    session
        .submit(&initial, &mut host, Budget::default())
        .unwrap();
    let cell = edit_source(None, &timeline(), &set("a", "same-entity"), false).unwrap();
    session.submit(&cell, &mut host, Budget::default()).unwrap();
    let mut edit = set("b", "same-entity");
    if let SoundFieldEdit::SetClip { position_m, .. } = &mut edit {
        *position_m = [3., 0., 0.];
    }
    assert!(edit_source(session.value("poet_sound_field"), &timeline(), &edit, false).is_err());
}
