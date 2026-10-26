//! Negative fixtures only; no fixture principal is production authentication.
use super::*;
use crate::graphical::{
    checkpoints::{CheckpointLimits, ExpectedFile, TrustedBinding},
    cooperation::UnboundAuthorityOwner,
};
use qualia_core_db::poet_host::invoke::studio_store;
use std::{
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};

struct Fixture {
    path: PathBuf,
    service: CheckpointService,
    scope: Scope,
}
impl Fixture {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(1);
        let serial = NEXT.fetch_add(1, Ordering::Relaxed);
        let root = std::env::var_os("POET_NATIVE_CHECKPOINT_TEST_ROOT")
            .map(PathBuf::from)
            .unwrap_or_else(std::env::temp_dir)
            .canonicalize()
            .unwrap();
        let path = root.join(format!(
            "poet-reopen-negative-{}-{serial}",
            std::process::id()
        ));
        std::fs::create_dir(&path).unwrap();
        let scope = Scope {
            project_id: "fixture-project".into(),
            document_id: format!("fixture-document-{serial}"),
            source_id: format!("urn:poet:reopen-negative:{}:{serial}", std::process::id()),
            domain: "Geometry".into(),
        };
        let (_, stamp) = studio_store::create_artifact_if_absent(StudioArtifact::new(
            &scope.source_id,
            &scope.domain,
            "negative reopen fixture",
        ))
        .unwrap();
        let service = CheckpointService::new(
            &path,
            TrustedBinding {
                project_id: scope.project_id.clone(),
                document_id: scope.document_id.clone(),
                source_id: scope.source_id.clone(),
                domain: scope.domain.clone(),
            },
            CheckpointLimits::default(),
        )
        .unwrap();
        service
            .save(&stamp, "source.hmc", ExpectedFile::Absent, None, vec![])
            .unwrap();
        Self {
            path,
            service,
            scope,
        }
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        // Exactly these two files were created in this uniquely owned directory;
        // no recursive removal or caller-provided arbitrary path is used.
        for name in ["source.hmc", "source.hmc.poet-source.lock"] {
            let _ = std::fs::remove_file(self.path.join(name));
        }
        let _ = std::fs::remove_dir(&self.path);
    }
}

#[test]
fn unbound_owner_cannot_activate_prepared_checkpoint() {
    let fixture = Fixture::new();
    let before = studio_store::get_artifact_stamped(&fixture.scope.source_id).unwrap();
    let prepared = fixture.service.prepare_reopen("source.hmc").unwrap();
    let error = reopen(
        &UnboundAuthorityOwner,
        &fixture.scope,
        &fixture.service,
        prepared,
        GatewayLimits::default(),
    )
    .unwrap_err();
    assert!(error.contains("No authenticated cooperative authority owner"));
    assert_eq!(
        studio_store::get_artifact_stamped(&fixture.scope.source_id).unwrap(),
        before
    );
}

struct NotInvokedOwner(AtomicUsize);
impl AuthorityOwner for NotInvokedOwner {
    fn with_current(
        &self,
        _: &mut dyn FnMut(&HostAuthorityView<'_>) -> Result<(), String>,
    ) -> Result<(), String> {
        self.0.fetch_add(1, Ordering::Relaxed);
        Err("negative fixture: lookup must not occur for a foreign scope".into())
    }
}
#[test]
fn foreign_host_scope_is_rejected_before_authority_lookup() {
    let fixture = Fixture::new();
    let before = studio_store::get_artifact_stamped(&fixture.scope.source_id).unwrap();
    let prepared = fixture.service.prepare_reopen("source.hmc").unwrap();
    let mut foreign = fixture.scope.clone();
    foreign.project_id = "foreign-project".into();
    let owner = NotInvokedOwner(AtomicUsize::new(0));
    assert!(reopen(
        &owner,
        &foreign,
        &fixture.service,
        prepared,
        GatewayLimits::default()
    )
    .is_err());
    assert_eq!(owner.0.load(Ordering::Relaxed), 0);
    assert_eq!(
        studio_store::get_artifact_stamped(&fixture.scope.source_id).unwrap(),
        before
    );
}
