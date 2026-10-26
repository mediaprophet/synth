//! Selected dependent metadata must not silently survive a semantic payload edit.
use super::*;

#[test]
fn georaster_edit_refuses_selected_opaque_provided_unknown_and_observation_metadata() {
    for variant in 0..4 {
        let root = tempfile::tempdir().unwrap();
        let scope = scope();
        let owner = Arc::new(FixtureOwner::new(&scope, ProjectRole::Contributor));
        let mut host = host(
            root.path(),
            &scope,
            owner.clone(),
            service(root.path(), &scope),
        );
        seed(root.path(), &scope, &mut host, owner.as_ref());
        let limits = CheckpointLimits::default();
        let base = core::load(&root.path().join(FILE), limits).unwrap();
        let mut resource = base
            .resources()
            .iter()
            .find(|r| r.key == KEY)
            .unwrap()
            .clone();
        let old_digest = resource.summary().digest;
        let meta = match variant {
            0 => vec![0xa1, 0x61, b'x', 1],
            1 => source_annotation(&resource.bytes, provided_source()),
            2 => source_annotation(
                &resource.bytes,
                SourceDeclaration {
                    status: SourceStatus::Unknown,
                    description: "origin unknown; do not infer sample semantics".into(),
                    evidence_ref: None,
                    observation_claim: false,
                },
            ),
            _ => source_annotation(
                &resource.bytes,
                SourceDeclaration {
                    status: SourceStatus::Provided,
                    description: "provided data without observation validation".into(),
                    evidence_ref: None,
                    observation_claim: false,
                },
            ),
        };
        resource.meta = Some(meta);
        // Fixture setup uses the canonical inert package editor. This does not
        // claim actor audit or scientific metadata admission for the fixture.
        let changed = base
            .apply_resource_edits(
                vec![ResourceEdit::Replace {
                    expected: old_digest,
                    resource,
                }],
                limits,
            )
            .unwrap();
        core::save(
            &root.path().join(FILE),
            &core::encode(&changed, vec![], limits).unwrap(),
            ExpectedFile::Sha256(base.file_digest().unwrap()),
            limits,
        )
        .unwrap();
        let request = selected(&host, &scope);
        let before = std::fs::read(root.path().join(FILE)).unwrap();
        let error = host
            .edit_georaster(
                request,
                reference(&scope),
                ui_georaster_edit_limits(),
                &AtomicBool::new(false),
            )
            .unwrap_err();
        assert!(
            error.contains("metadata") || error.contains("derivation"),
            "{error}"
        );
        assert_eq!(std::fs::read(root.path().join(FILE)).unwrap(), before);
    }
}
