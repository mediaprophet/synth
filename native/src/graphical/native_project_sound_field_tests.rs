//! Software fixtures: native authority/source/HMC, no human or device acceptance.
use super::*;
use crate::graphical::{
    authoring::{
        sound_field::SoundFieldEdit,
        timeline::{Clip, Timeline, TimelineEdit, Track},
        AuthoringEdit,
    },
    controller::CanonicalEditorHost,
    geometry_host::GeometryHost,
};
struct FieldNoPixels;
impl PreviewProvider for FieldNoPixels {
    fn retire_retained_viewport(&mut self) -> Result<(), String> {
        Ok(())
    }
    fn preview(&mut self, _: SourceRef, _: &Value) -> Result<PreviewFrame, String> {
        Err("Sound-field fixture renders no pixels".into())
    }
}
#[test]
fn native_field_uses_current_authority_persists_in_hmc_and_reconstructs_in_a_fresh_host() {
    let temp = tempfile::tempdir().unwrap();
    for name in ["authority", "hmc", "library"] {
        std::fs::create_dir(temp.path().join(name)).unwrap();
    }
    let (scope, mut admission, authority) = fixture(&temp.path().join("authority"));
    let author = login(&authority, "fixture-author", &AUTHOR);
    let project = NativeProject::open(
        config(
            scope.clone(),
            &temp.path().join("hmc"),
            &temp.path().join("library"),
        ),
        authority.clone(),
        author.clone(),
        None,
        Box::new(FieldNoPixels),
    )
    .unwrap();
    let mut ui = project.connect(author).unwrap();
    let initial = ui.snapshot().unwrap();
    let timeline = Timeline {
        version: 1,
        frame_ticks: 1,
        tracks: vec![Track {
            id: "audio".into(),
            locked: false,
            clips: vec![Clip {
                id: "bird".into(),
                source: "urn:bird:clip".into(),
                source_start: 28,
                start: 56,
                duration: 140,
            }],
        }],
        transitions: vec![],
    };
    let adopted = ui
        .authoring_edit(
            AuthoringEdit::Timeline {
                edit: TimelineEdit::Adopt { timeline },
            },
            initial.source,
        )
        .unwrap();
    let stale = adopted.snapshot.source.clone();
    let assigned = ui
        .authoring_edit(
            AuthoringEdit::SoundField {
                edit: SoundFieldEdit::SetClip {
                    clip_id: "bird".into(),
                    entity_identifier: "urn:entity:bird".into(),
                    position_m: [2., 1., -3.],
                    radius_m: 25.,
                    gain: 0.3,
                    priority: 2,
                },
            },
            stale.clone(),
        )
        .unwrap();
    let edit = || AuthoringEdit::SoundField {
        edit: SoundFieldEdit::SetListener {
            position_m: [1., 0., 0.],
            yaw_rad: 0.5,
        },
    };
    assert!(ui.authoring_edit(edit(), stale).is_err());
    let before = ui
        .authoring_view(assigned.snapshot.source.clone())
        .unwrap()
        .sound_field
        .unwrap();
    let receipt = ui
        .save_checkpoint(
            "field.hmc",
            ExpectedFile::Absent,
            assigned.snapshot.source.clone(),
        )
        .unwrap();
    let saved = checkpoint::load(&receipt.path, Default::default()).unwrap();
    assert_eq!(
        saved.artifact(),
        &studio_store::get_artifact_stamped(&scope.source_id)
            .unwrap()
            .0
    );
    // Reconstruct via the existing explicit native replay owner. This is a
    // fresh host over the saved-equivalent source, not a fresh-process claim.
    let mut fresh = CanonicalEditorHost::open(
        scope.source_id.clone(),
        SessionLimits::default(),
        GeometryHost::new(
            vec![
                "capability.invoke".into(),
                "ComputationalGeometry.extrude_profile".into(),
            ],
            200_000,
        ),
        Box::new(FieldNoPixels),
    )
    .unwrap();
    let held = fresh.snapshot().unwrap();
    assert!(held.reconciliation_required);
    let rebuilt = fresh.rebuild_execution(held.source).unwrap();
    assert_eq!(
        fresh
            .authoring_view(rebuilt.snapshot.source)
            .unwrap()
            .sound_field,
        Some(before)
    );
    admission.sequence += 1;
    admission.subjects[0].revoked = true;
    authority.replace_admission(signed(admission)).unwrap();
    assert!(ui.authoring_edit(edit(), assigned.snapshot.source).is_err());
}
