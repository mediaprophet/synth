//! UI actor selection composes the existing native authority; it grants nothing.
use crate::graphical::{
    cooperation::AuthorityOwner,
    native_project::SessionSlot,
};
use qualia_cooperative_core::authoring::HostAuthorityView;
use qualia_core_db::poet_host::{
    invoke::studio_store::checkpoint::SaveReceipt,
    native_authority::{PackageWriteIntent, SessionHandle},
};
use std::sync::{Arc, MutexGuard};

pub(super) struct SelectedAuthorityOwner {
    native: Arc<dyn AuthorityOwner>,
    selection: SessionSlot,
    session: SessionHandle,
}
impl SelectedAuthorityOwner {
    pub(super) fn new(
        native: Arc<dyn AuthorityOwner>,
        selection: SessionSlot,
        session: SessionHandle,
    ) -> Self {
        Self { native, selection, session }
    }
    fn selected(&self) -> Result<MutexGuard<'_, Option<SessionHandle>>, String> {
        let selection = self.selection.try_lock()
            .map_err(|_| "Native session selection busy or poisoned")?;
        if selection.as_ref() != Some(&self.session) {
            return Err("Native selected actor changed; refresh authorised source".into());
        }
        Ok(selection)
    }
}

impl AuthorityOwner for SelectedAuthorityOwner {
    fn publication_is_current(&self, generation: u64) -> bool {
        let Ok(_selection) = self.selected() else { return false; };
        // Retain the selection guard for the complete nonblocking native check.
        self.native.publication_is_current(generation)
    }

    fn with_current(
        &self,
        operation: &mut dyn FnMut(&HostAuthorityView<'_>) -> Result<(), String>,
    ) -> Result<(), String> {
        let _selection = self.selected()?;
        // Lock order: UI selection -> native authority -> raw shared editor.
        // This inner owner must not re-enter selected UI/session services.
        self.native.with_current(operation)
    }

    fn with_current_package(
        &self,
        intent: &PackageWriteIntent,
        operation: &mut dyn FnMut(&HostAuthorityView<'_>) -> Result<SaveReceipt, String>,
    ) -> Result<SaveReceipt, String> {
        let _selection = self.selected()?;
        // Delegate the native package transaction itself, including its durable
        // audit. The trait's default with_current adapter would lose that audit.
        self.native.with_current_package(intent, operation)
    }
}
