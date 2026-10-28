//! Share one canonical editor/session between native UI and MCP projections.
//!
//! Mount raw CanonicalEditorHost (optionally decorated by CheckpointEditorHost)
//! INSIDE this adapter. Mount a separate AuthorizedEditorHost OUTSIDE each clone
//! with its own current native owner/scope. Never wrap an AuthorizedEditorHost
//! here: nested owner acquisition could deadlock and would confuse caller scope.
//! This adapter owns no project, authority, source cache or second session.
use super::controller::{EditorHost, EditorOutcome, EditorSnapshot};
use super::preview::SourceRef;
use crate::graphical::checkpoints::typed_10d as typed;
use qualia_core_db::poet_host::invoke::studio_store::checkpoint::{ExpectedFile, SaveReceipt};
use std::sync::{Arc, Mutex, MutexGuard, TryLockError};
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Clone)]
pub struct SharedEditorHost {
    inner: Arc<Mutex<Box<dyn EditorHost>>>,
    viewport_quarantined: Arc<AtomicBool>,
    // Native service availability is fixed at composition, independently of
    // current user authority or momentary lock contention. No auth is cached.
    checkpoint_supported: bool,
    resource_import_supported: bool,
}

impl SharedEditorHost {
    /// Supply only the raw canonical/checkpoint editor, before authority wraps
    /// each frontend. Existing host grants/state remain in that one editor.
    pub fn new(inner: Box<dyn EditorHost>) -> Self {
        let checkpoint_supported = inner.checkpoint_supported();
        let resource_import_supported = inner.resource_import_supported();
        Self {
            inner: Arc::new(Mutex::new(inner)),
            viewport_quarantined: Arc::new(AtomicBool::new(false)),
            checkpoint_supported,
            resource_import_supported,
        }
    }

    /// Trusted package workflow only: acquire after authority admission and
    /// retain through native ledger commit. Never wrap an authorized host here.
    pub(crate) fn package_review_guard(
        &self,
    ) -> Result<MutexGuard<'_, Box<dyn EditorHost>>, String> {
        self.current()
    }
    pub(crate) fn viewport_guard(&self) -> Result<MutexGuard<'_, Box<dyn EditorHost>>, String> {
        self.current()
    }
    /// Caller holds the shared guard; quarantine is irreversible for this host instance.
    pub(crate) fn quarantine_viewport(&self) { self.viewport_quarantined.store(true, Ordering::Release); }
    fn current(&self) -> Result<MutexGuard<'_, Box<dyn EditorHost>>, String> {
        match self.inner.try_lock() {
            Ok(editor) => {
                // Source edits and rebuilds can also render via the same provider.
                // If retirement failed, quarantine the whole shared instance.
                if self.viewport_quarantined.load(Ordering::Acquire) {
                    Err("Retained viewport is quarantined; shared host reconstruction is required".into())
                } else { Ok(editor) }
            },
            Err(TryLockError::WouldBlock) => Err(
                "Shared native editor is busy; operation was not dispatched; no automatic retry"
                    .into(),
            ),
            Err(TryLockError::Poisoned(_)) => Err(
                "Shared native editor is poisoned; host reconciliation is required before dispatch"
                    .into(),
            ),
        }
    }
}

impl EditorHost for SharedEditorHost {
    fn shared_viewport_host(&self) -> Option<SharedEditorHost> { Some(self.clone()) }
    fn viewport_action(&mut self, request: crate::graphical::viewport::ViewportCommand,
        expected: SourceRef, cancelled: &std::sync::atomic::AtomicBool,
    ) -> Result<crate::graphical::viewport_contract::ViewportOutput, String> {
        self.viewport_guard()?.viewport_action(request, expected, cancelled)
    }

    fn retained_surface_carrier(
        &self,
        expected: SourceRef,
        limits: qualia_core_db::poet_host::invoke::studio_store::checkpoint::CheckpointLimits,
    ) -> Result<
        Option<qualia_core_db::poet_host::invoke::studio_store::checkpoint::VerifiedSourceCarrier>,
        String,
    > {
        self.current()?.retained_surface_carrier(expected, limits)
    }
    fn retained_surface_inputs(
        &self,
        expected: SourceRef,
    ) -> Result<
        Vec<qualia_core_db::poet_host::invoke::studio_store::checkpoint::ResourceData>,
        String,
    > {
        self.current()?.retained_surface_inputs(expected)
    }

    fn inspect_surface(
        &self,
        id: &str,
        expected: SourceRef,
    ) -> Result<crate::graphical::surfaces::inspection::SurfaceInspection, String> {
        self.current()?.inspect_surface(id, expected)
    }

    fn bind_surface_package(
        &mut self,
        request: crate::graphical::surface_host::SurfacePackageRequest,
        expected: SourceRef,
    ) -> Result<(), String> {
        request.validate()?;
        self.current()?.bind_surface_package(request, expected)
    }

    fn inspect_typed_10d(
        &self,
        request: crate::graphical::checkpoints::ResourceInspectionRequest,
        expected: SourceRef,
        limits: typed::Typed10dLimits,
    ) -> Result<typed::Typed10dInspection, String> {
        self.current()?.inspect_typed_10d(request, expected, limits)
    }
    fn edit_typed_10d(
        &mut self,
        request: typed::Typed10dEditRequest,
        expected: SourceRef,
        limits: typed::Typed10dLimits,
        cancelled: &std::sync::atomic::AtomicBool,
    ) -> Result<qualia_core_db::poet_host::invoke::studio_store::checkpoint::SaveReceipt, String>
    {
        self.current()?
            .edit_typed_10d(request, expected, limits, cancelled)
    }

    fn edit_georaster(
        &mut self,
        request: crate::graphical::checkpoints::GeoRasterEditRequest,
        expected: SourceRef,
        limits: crate::graphical::checkpoints::GeoRasterEditLimits,
        cancelled: &std::sync::atomic::AtomicBool,
    ) -> Result<qualia_core_db::poet_host::invoke::studio_store::checkpoint::SaveReceipt, String>
    {
        self.current()?
            .edit_georaster(request, expected, limits, cancelled)
    }
    fn create_georaster(
        &mut self,
        request: crate::graphical::checkpoints::GeoRasterCreateRequest,
        expected: SourceRef,
        limits: crate::graphical::checkpoints::GeoRasterEditLimits,
        cancelled: &std::sync::atomic::AtomicBool,
    ) -> Result<qualia_core_db::poet_host::invoke::studio_store::checkpoint::SaveReceipt, String>
    {
        self.current()?
            .create_georaster(request, expected, limits, cancelled)
    }

    fn inspect_package_resource(
        &self,
        request: super::checkpoints::ResourceInspectionRequest,
        expected: SourceRef,
        limits: qualia_core_db::q42::m64_matter::M64ReadLimits,
        scratch: &mut [qualia_core_db::q42::m64_matter::M64PayloadInterval],
    ) -> Result<super::checkpoints::ResourceInspection, String> {
        self.current()?
            .inspect_package_resource(request, expected, limits, scratch)
    }
    fn numeric_sheet_view(&self, expected: SourceRef) -> Result<crate::graphical::numeric_sheet::SheetView, String> { self.current()?.numeric_sheet_view(expected) }
    fn numeric_sheet_edit(&mut self, edit: crate::graphical::numeric_sheet::SheetEdit, expected: SourceRef) -> Result<EditorOutcome, String> { self.current()?.numeric_sheet_edit(edit, expected) }
    fn authoring_view(
        &self,
        e: SourceRef,
    ) -> Result<crate::graphical::authoring::AuthoringView, String> {
        self.current()?.authoring_view(e)
    }
    fn rig_authoring_edit(&mut self, resource: crate::graphical::checkpoints::ResourceInspectionRequest, control: crate::graphical::rig_binding::RigControl, expected: SourceRef, cancelled: &std::sync::atomic::AtomicBool) -> Result<EditorOutcome,String> {
        self.current()?.rig_authoring_edit(resource, control, expected, cancelled)
    }
    fn authoring_edit(
        &mut self,
        a: crate::graphical::authoring::AuthoringEdit,
        e: SourceRef,
    ) -> Result<EditorOutcome, String> {
        self.current()?.authoring_edit(a, e)
    }
    fn compiled_assets(
        &self,
        k: &str,
        e: SourceRef,
    ) -> Result<Vec<(String, qualia_core_db::render::compile_10d::CompiledAsset)>, String> {
        self.current()?.compiled_assets(k, e)
    }
    fn compile_checkpoint(
        &mut self,
        n: &str,
        k: &str,
        f: qualia_core_db::poet_host::invoke::studio_store::checkpoint::ExpectedFile,
        e: SourceRef,
    ) -> Result<qualia_core_db::poet_host::invoke::studio_store::checkpoint::SaveReceipt, String>
    {
        self.current()?.compile_checkpoint(n, k, f, e)
    }

    fn apply_resource_plan(
        &mut self,
        n: &str,
        f: ExpectedFile,
        plan: Vec<super::package::LibraryEdit>,
        e: SourceRef,
    ) -> Result<SaveReceipt, String> {
        self.current()?.apply_resource_plan(n, f, plan, e)
    }
    fn resource_edit_preflight(
        &self,
        n: &str,
        f: ExpectedFile,
        e: SourceRef,
    ) -> Result<(), String> {
        self.current()?.resource_edit_preflight(n, f, e)
    }
    fn resource_import_supported(&self) -> bool {
        self.resource_import_supported
    }
    fn package_inventory(
        &self,
        name: &str,
        expected: SourceRef,
    ) -> Result<super::checkpoints::PackageInventory, String> {
        self.current()?.package_inventory(name, expected)
    }
    fn edit_package_resources(
        &mut self,
        name: &str,
        file: ExpectedFile,
        edits: Vec<qualia_core_db::poet_host::invoke::studio_store::checkpoint::ResourceEdit>,
        expected: SourceRef,
    ) -> Result<SaveReceipt, String> {
        self.current()?
            .edit_package_resources(name, file, edits, expected)
    }
    fn import_package_resource(
        &mut self,
        name: &str,
        file: ExpectedFile,
        request: super::package::ImportRequest,
        expected: SourceRef,
    ) -> Result<SaveReceipt, String> {
        self.current()?
            .import_package_resource(name, file, request, expected)
    }
    fn snapshot(&self) -> Result<EditorSnapshot, String> {
        self.current()?.snapshot()
    }
    fn check(&self, source: &str) -> Result<String, String> {
        self.current()?.check(source)
    }
    fn source_cells(&self, expected: SourceRef) -> Result<Vec<String>, String> {
        self.current()?.source_cells(expected)
    }
    fn checkpoint_supported(&self) -> bool {
        self.checkpoint_supported
    }
    fn submit(&mut self, source: &str, expected: SourceRef) -> Result<EditorOutcome, String> {
        self.current()?.submit(source, expected)
    }
    fn material_edit(
        &mut self,
        source: &str,
        expected: SourceRef,
    ) -> Result<EditorOutcome, String> {
        self.current()?.material_edit(source, expected)
    }
    fn undo(&mut self, expected: SourceRef) -> Result<EditorSnapshot, String> {
        self.current()?.undo(expected)
    }
    fn redo(&mut self, expected: SourceRef) -> Result<EditorSnapshot, String> {
        self.current()?.redo(expected)
    }
    fn rebuild_execution(&mut self, expected: SourceRef) -> Result<EditorOutcome, String> {
        self.current()?.rebuild_execution(expected)
    }
    fn save_checkpoint(
        &mut self,
        name: &str,
        expected_file: ExpectedFile,
        expected: SourceRef,
    ) -> Result<SaveReceipt, String> {
        self.current()?
            .save_checkpoint(name, expected_file, expected)
    }
}

#[cfg(test)]
#[path = "shared_host_tests.rs"]
mod tests;
