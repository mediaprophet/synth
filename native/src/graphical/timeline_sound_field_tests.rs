use super::super::authoring::{
    sound_field::ClipEmitter,
    timeline::{Clip, Track},
};
use super::*;

fn fixture(
    count: usize,
    entities: usize,
) -> (Timeline, Vec<Association>, SoundField, FieldRequest) {
    let ticks = (timeline_editing::TICKS_PER_SECOND as u64) / RATE as u64;
    let mut clips = Vec::new();
    let mut associations = Vec::new();
    let mut bindings = Vec::new();
    for i in 0..count {
        let id = format!("clip-{i:03}");
        let source = format!("urn:source:{i}");
        clips.push(Clip {
            id: id.clone(),
            source: source.clone(),
            source_start: 3 * ticks,
            start: (i as u64) * 4 * ticks,
            duration: 4 * ticks,
        });
        associations.push(Association {
            clip_id: id.clone(),
            source_identifier: source.clone(),
            resource_key: format!("sound/{i}"),
            resource_digest: [1; 32],
        });
        bindings.push(ClipEmitter {
            clip_id: id,
            source_identifier: source,
            entity_identifier: format!("entity-{}", i % entities),
            position_m: [1., 0., -2.],
            radius_m: 25.,
            gain: 0.2,
            priority: 0,
        });
    }
    (
        Timeline {
            version: 1,
            frame_ticks: 1,
            tracks: vec![Track {
                id: "audio".into(),
                locked: false,
                clips,
            }],
            transitions: vec![],
        },
        associations,
        SoundField {
            listener_position_m: [0.; 3],
            listener_yaw_rad: 0.,
            bindings,
        },
        FieldRequest {
            package: ResourceInspectionRequest {
                filename: "media.hmc".into(),
                package_digest: [2; 32],
                key: "anchor".into(),
                resource_digest: [3; 32],
            },
            start_tick: 0,
            duration_ticks: (count as u64) * 4 * ticks,
        },
    )
}
#[test]
fn many_authored_clips_retain_distinct_offsets_and_do_not_use_the_old_mix_track_cap() {
    let (timeline, associations, field, request) = fixture(130, 2);
    let plan = plan(&timeline, &associations, &field, &request).unwrap();
    assert_eq!(plan.clips.len(), 130);
    assert_eq!(plan.graph.emitter_count, 2);
    assert_eq!(plan.clips[1].start, 4);
    assert_eq!(plan.clips[1].end, 8);
    assert_eq!(
        plan.clips[1].request.timeline_tick,
        timeline.tracks[0].clips[1].start
    );
    assert_eq!(plan.clips[1].request.resource.key, "sound/1");
    let decoded = (0..130)
        .map(|_| PreparedWav {
            rate: RATE,
            channels: 1,
            source_start_frame: 3,
            samples: vec![0.25; 4],
        })
        .collect::<Vec<_>>();
    let (pcm, _, _, _) = render(&plan, &decoded, &request, || Ok(())).unwrap();
    assert_eq!(pcm.frames(), 520);
    assert!(pcm.samples.iter().all(|s| s.is_finite()));
    assert!(pcm.samples.iter().any(|s| s.abs() > 1e-9));
    let mut unsupported = decoded;
    unsupported[0].rate = 44_100;
    assert!(render(&plan, &unsupported, &request, || Ok(())).is_err());
    unsupported[0].rate = RATE;
    unsupported[0].channels = 2;
    assert!(render(&plan, &unsupported, &request, || Ok(())).is_err());
}
#[test]
fn unsupported_residency_absent_binding_and_unaligned_clocks_refuse() {
    let (mut timeline, mut associations, field, request) = fixture(65, 65);
    assert!(plan(&timeline, &associations, &field, &request).is_err());
    let (_, _, field, _) = fixture(65, 2);
    associations.pop();
    assert!(plan(&timeline, &associations, &field, &request).is_err());
    associations.push(Association {
        clip_id: "clip-064".into(),
        source_identifier: "urn:source:64".into(),
        resource_key: "sound/64".into(),
        resource_digest: [1; 32],
    });
    timeline.tracks[0].clips[0].start += 1;
    assert!(plan(&timeline, &associations, &field, &request).is_err());
    let mut bad = request.clone();
    bad.duration_ticks = 0;
    assert!(bad.validate().is_err());
}
#[test]
fn cancellation_during_preparation_or_later_block_withholds_entire_audition() {
    let (timeline, associations, field, mut request) = fixture(1, 1);
    request.duration_ticks = (timeline_editing::TICKS_PER_SECOND as u64) / 5;
    let plan = plan(&timeline, &associations, &field, &request).unwrap();
    let decoded = vec![PreparedWav {
        rate: RATE,
        channels: 1,
        source_start_frame: 3,
        samples: vec![0.25; 4],
    }];
    let calls = std::cell::Cell::new(0);
    let result = render(&plan, &decoded, &request, || {
        let n = calls.get() + 1;
        calls.set(n);
        if n >= 3 {
            Err("revoked during later block".into())
        } else {
            Ok(())
        }
    });
    assert!(result.is_err());
    assert_eq!(calls.get(), 3);
}
