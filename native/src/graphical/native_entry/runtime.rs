//! One retained native project/execution owner; sessions and UI leases are ephemeral.
use super::{
    files::{self, HostFiles, Profile, PublicProject},
    types::*,
};
use crate::graphical::{
    controller::EditorHost,
    draft_buffer::DraftBuffer,
    jobs,
    native_project::{NativeProject, SessionSlot},
    session_host::SessionEditorHost,
};
use qualia_core_db::{
    identity::key_vault::KeyVault,
    poet_host::native_authority::{enrollment::NativeSigner, NativeAuthority, SessionHandle},
};
use std::{
    collections::BTreeMap,
    path::Path,
    sync::{Arc, Mutex},
};
#[path = "actions.rs"]
pub(super) mod actions;
pub struct Runtime {
    pub(super) files: HostFiles,
    pub(super) config: Option<PublicProject>,
    pub(super) adopted: bool,
    pub(super) authority: Option<Arc<NativeAuthority>>,
    pub(super) project: Option<Arc<NativeProject>>,
    pub(super) sessions: SessionSlot,
    pub(super) current_profile: Option<String>,
    pub(super) studio: Option<StudioLease>,
    pub(super) worker: Option<jobs::Worker>,
    pub(super) buffers: BTreeMap<String, DraftBuffer>,
    pub(super) active_filename: Option<String>,
    pub(super) generation: u64,
    pub(super) epoch: u64,
}
impl Runtime {
    /// Public metadata only. No vault, source, project adoption or credentials.
    pub fn open(root: &Path) -> Result<Self, String> {
        let files = HostFiles::open(root)?;
        let config = files.project()?;
        Ok(Self {
            files,
            config,
            adopted: false,
            authority: None,
            project: None,
            sessions: Arc::new(Mutex::new(None)),
            current_profile: None,
            studio: None,
            worker: None,
            buffers: BTreeMap::new(),
            active_filename: None,
            generation: 1,
            epoch: 0,
        })
    }
    pub(super) fn guard(&self) -> Result<(), String> {
        self.files.check()?;
        if let Some(config) = &self.config {
            self.files.same_project(config)?;
        }
        Ok(())
    }
    pub(super) fn bump(&mut self) {
        self.generation = self.generation.saturating_add(1);
    }
    pub(super) fn session(&self) -> Result<SessionHandle, String> {
        self.sessions
            .lock()
            .map_err(|_| "Native session slot unavailable")?
            .clone()
            .ok_or("Authenticate an admitted profile first".into())
    }
    pub(super) fn owner(&self) -> Result<Arc<NativeAuthority>, String> {
        if !self.adopted {
            return Err(
                "Explicitly adopt the displayed project authority fingerprint first".into(),
            );
        }
        self.authority
            .clone()
            .ok_or("Native project authority unavailable".into())
    }
    pub(super) fn detach(&mut self) -> Result<(), String> {
        let session = self
            .sessions
            .lock()
            .map_err(|_| "Native session slot unavailable")?
            .take();
        self.studio = None;
        self.current_profile = None;
        let joined = self
            .worker
            .take()
            .map(jobs::Worker::finish)
            .unwrap_or(Ok(()));
        if let (Some(owner), Some(session)) = (&self.authority, session) {
            let _ = owner.disconnect(&session);
        }
        joined
    }
    /// Stale/revoked sessions retire their workers; only public state remains visible.
    /// Private draft buffers and the one canonical execution service are retained.
    pub fn view(&mut self) -> Result<LaunchView, String> {
        if self.guard().is_err() {
            let _ = self.detach();
            self.adopted = false;
            self.bump();
        }
        let mut current_subject = None;
        let mut pending = Vec::new();
        let mut members = Vec::new();
        let mut package_grants = Vec::new();
        if self.studio.is_some() && (self.owner().is_err() || self.session().is_err()) {
            let _ = self.detach();
            self.bump();
        }
        if let (Ok(owner), Ok(session)) = (self.owner(), self.session()) {
            match owner.workflow_evidence(&session) {
                Ok(evidence) => {
                    current_subject = Some(evidence.principal_subject);
                    if let Ok(permission) = owner.enrollment_permission(&session) {
                        if let (Ok(records), Ok(admission)) = (
                            owner.list_enrollments(&permission),
                            owner.enrollment_snapshot(),
                        ) {
                            pending = records;
                            package_grants = admission.snapshot.package_reviews.clone();
                            members = admission
                                .snapshot
                                .subjects
                                .into_iter()
                                .map(|s| MemberView {
                                    subject: s.subject,
                                    kind: s.kind,
                                    role: s.membership.role,
                                    has_review: s.reviews.iter().any(|r| !r.revoked),
                                    revoked: s.revoked,
                                })
                                .collect();
                        }
                    }
                }
                Err(_) => {
                    let _ = self.detach();
                    self.bump();
                }
            }
        }
        Ok(LaunchView {
            package_grants,
            generation: self.generation,
            project_name: self.config.as_ref().map(|c| c.name.clone()),
            authority_fingerprint: self.config.as_ref().map(PublicProject::fingerprint),
            adopted: self.adopted,
            current_subject,
            pending,
            members,
            studio: self.studio.clone(),
        })
    }
    pub(super) fn adopt(&mut self, fingerprint: &str) -> Result<(), String> {
        let config = self
            .config
            .clone()
            .ok_or("Create a local project or open an existing project directory")?;
        if fingerprint != config.fingerprint() {
            return Err(
                "Confirmed authority fingerprint differs from displayed public project key".into(),
            );
        }
        self.detach()?;
        if self.authority.is_none() {
            self.authority = Some(Arc::new(NativeAuthority::open_retained_current(
                &self.files.directory("authority", false)?,
                config.anchor(),
                4 * 1024 * 1024,
                4096,
            )?));
        }
        self.adopted = true;
        Ok(())
    }
    pub(super) fn unlock(&self, label: &str) -> Result<(Profile, KeyVault), String> {
        let profile = self.files.profile(label)?;
        let path = self.files.profile_path(label, false)?;
        let vault = KeyVault::open_native_secure(
            path.to_str().ok_or("Native profile path is not Unicode")?,
            false,
        )?;
        let signer = NativeSigner::from_vault(&vault, files::CONTROLLER_LABEL)?;
        if signer.public_key() != profile.controller_public_key {
            return Err("Unlocked OS controller differs from saved public profile; explicit recovery required".into());
        }
        Ok((profile, vault))
    }
    pub(super) fn login(&mut self, label: &str, filename: &str) -> Result<(), String> {
        files::filename(filename)?;
        let owner = self.owner()?;
        if self.project.is_some() && self.active_filename.as_deref() != Some(filename) {
            return Err("The retained source service already owns another package filename; close and reopen explicitly to change packages".into());
        }
        self.detach()?;
        let (_, vault) = self.unlock(label)?;
        let signer = NativeSigner::from_vault(&vault, files::CONTROLLER_LABEL)?;
        let admission = owner.enrollment_snapshot()?;
        // This association is explicitly signed by the pinned project authority.
        let subject = admission
            .snapshot
            .subjects
            .iter()
            .find(|s| s.controller_public_key == signer.public_key())
            .ok_or(
                "This controller is not admitted; submit enrollment and await a steward's decision",
            )?
            .subject
            .clone();
        let session = super::session_proof::authenticate(&owner, &subject, 3600, |challenge| {
            signer.prove_session(challenge)
        })?;
        let opened = (|| {
            if self.project.is_none() {
                let config = self
                    .config
                    .as_ref()
                    .ok_or("Native project configuration missing")?;
                let project = NativeProject::open_with_rebuild_limits(
                    super::create::project_config(&self.files, config)?,
                    owner.clone(),
                    session.clone(),
                    Some(filename),
                    super::create::preview(),
                    crate::graphical::native_authoring_profile::startup_rebuild_limits(
                        crate::graphical::native_authoring_profile::selected()),
                )?;
                self.project = Some(Arc::new(project));
                self.active_filename = Some(filename.into());
            }
            *self
                .sessions
                .lock()
                .map_err(|_| "Native session slot unavailable")? = Some(session.clone());
            self.current_profile = Some(label.into());
            self.start_studio(label)
        })();
        if opened.is_err() {
            let _ = owner.disconnect(&session);
            let _ = self.detach();
        }
        opened
    }
    pub(super) fn start_studio(&mut self, label: &str) -> Result<(), String> {
        let project = self
            .project
            .clone()
            .ok_or("Native source service unavailable")?;
        let host = SessionEditorHost::new(project, self.sessions.clone());
        let initial = host.snapshot()?;
        if !self.buffers.contains_key(label) && self.buffers.len() >= 64 {
            return Err("Local private draft profile budget exhausted".into());
        }
        let draft_buffer = self
            .buffers
            .entry(label.into())
            .or_insert_with(|| DraftBuffer::new(initial.latest_source.clone()))
            .clone();
        let (worker, host) = jobs::Worker::start(Box::new(host))?;
        self.epoch = self
            .epoch
            .checked_add(1)
            .ok_or("Native studio lease epoch exhausted")?;
        self.worker = Some(worker);
        self.studio = Some(StudioLease {
            host,
            initial,
            draft_buffer,
            epoch: self.epoch,
        });
        Ok(())
    }
}
impl Drop for Runtime {
    fn drop(&mut self) {
        let _ = self.detach();
    }
}
