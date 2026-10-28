//! Reconnect each native operation to the current admitted session.
use super::{
    checkpoints::PackageInventory,
    controller::{EditorHost, EditorOutcome, EditorSnapshot},
    cooperation::AuthorizedEditorHost,
    cooperative_workflow::{WorkflowAction, WorkflowService, WorkflowView},
    native_project::{NativeProject, SessionSlot},
    package::{ImportRequest, LibraryEdit},
    preview::SourceRef,
};
use crate::graphical::checkpoints::typed_10d as typed;
use qualia_core_db::poet_host::invoke::studio_store::checkpoint::{
    ExpectedFile, ResourceEdit, SaveReceipt,
};
use qualia_core_db::poet_host::native_authority::SessionHandle;
use std::{
    ops::{Deref, DerefMut},
    sync::{Arc, Mutex, MutexGuard},
};

struct CurrentHost<'a> {
    guard: MutexGuard<'a, Option<(SessionHandle, AuthorizedEditorHost)>>,
}
impl Deref for CurrentHost<'_> {
    type Target = AuthorizedEditorHost;
    fn deref(&self) -> &Self::Target {
        &self.guard.as_ref().expect("bound native frontend").1
    }
}
impl DerefMut for CurrentHost<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.guard.as_mut().expect("bound native frontend").1
    }
}

pub struct SessionEditorHost {
    project: Arc<NativeProject>,
    sessions: SessionSlot,
    workflow: WorkflowService,
    connection: Mutex<Option<(SessionHandle, AuthorizedEditorHost)>>,
}
impl SessionEditorHost {
    pub fn new(project: Arc<NativeProject>, sessions: SessionSlot) -> Self {
        let workflow = project.workflow_service(sessions.clone());
        Self {
            project,
            sessions,
            workflow,
            connection: Mutex::new(None),
        }
    }
    fn current(&self) -> Result<CurrentHost<'_>, String> {
        let session = self
            .sessions
            .try_lock()
            .map_err(|_| "Native session selection is busy or unavailable")?
            .clone()
            .ok_or(
                "Unlock and authenticate an admitted contributor or reviewer before authoring",
            )?;
        let mut guard = self
            .connection
            .try_lock()
            .map_err(|_| "Native frontend binding is busy or poisoned")?;
        if guard
            .as_ref()
            .is_none_or(|(current, _)| current != &session)
        {
            let host = self.project.connect_selected(session.clone(), self.sessions.clone())?;
            *guard = Some((session, host));
        }
        // Keep the derived frontend owner stable while the same opaque session
        // is selected. Its native authorization is still checked on every call;
        // rebuilding the owner on every read would invalidate publication stamps.
        Ok(CurrentHost { guard })
    }
}
impl EditorHost for SessionEditorHost {
    fn viewport_action(&mut self, request: crate::graphical::viewport::ViewportCommand,
        expected: SourceRef, cancelled: &std::sync::atomic::AtomicBool,
    ) -> Result<crate::graphical::viewport_contract::ViewportOutput, String> {
        self.current()?.viewport_action(request, expected, cancelled)
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

    fn package_workflow(
        &mut self,
        action: super::package_workflow::PackageWorkflowAction,
        expected: SourceRef,
    ) -> Result<super::package_workflow::PackageWorkflowView, String> {
        self.project
            .package_workflow_service(self.sessions.clone())
            .run(action, expected)
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

    fn workflow(&mut self, action: WorkflowAction) -> Result<WorkflowView, String> {
        self.workflow.run(action)
    }
    fn snapshot(&self) -> Result<EditorSnapshot, String> {
        self.current()?.snapshot()
    }
    fn check(&self, s: &str) -> Result<String, String> {
        self.current()?.check(s)
    }
    fn submit(&mut self, s: &str, e: SourceRef) -> Result<EditorOutcome, String> {
        self.current()?.submit(s, e)
    }
    fn undo(&mut self, e: SourceRef) -> Result<EditorSnapshot, String> {
        self.current()?.undo(e)
    }
    fn redo(&mut self, e: SourceRef) -> Result<EditorSnapshot, String> {
        self.current()?.redo(e)
    }
    fn rebuild_execution(&mut self, e: SourceRef) -> Result<EditorOutcome, String> {
        self.current()?.rebuild_execution(e)
    }
    fn material_edit(&mut self, s: &str, e: SourceRef) -> Result<EditorOutcome, String> {
        self.current()?.material_edit(s, e)
    }
    fn source_cells(&self, e: SourceRef) -> Result<Vec<String>, String> {
        self.current()?.source_cells(e)
    }
    fn checkpoint_supported(&self) -> bool {
        true
    }
    fn resource_import_supported(&self) -> bool {
        self.current()
            .is_ok_and(|host| host.resource_import_supported())
    }
    fn save_checkpoint(
        &mut self,
        n: &str,
        f: ExpectedFile,
        e: SourceRef,
    ) -> Result<SaveReceipt, String> {
        self.current()?.save_checkpoint(n, f, e)
    }
    fn package_inventory(&self, n: &str, e: SourceRef) -> Result<PackageInventory, String> {
        self.current()?.package_inventory(n, e)
    }
    fn resource_edit_preflight(
        &self,
        n: &str,
        f: ExpectedFile,
        e: SourceRef,
    ) -> Result<(), String> {
        self.current()?.resource_edit_preflight(n, f, e)
    }
    fn edit_package_resources(
        &mut self,
        n: &str,
        f: ExpectedFile,
        r: Vec<ResourceEdit>,
        e: SourceRef,
    ) -> Result<SaveReceipt, String> {
        self.current()?.edit_package_resources(n, f, r, e)
    }
    fn import_package_resource(
        &mut self,
        n: &str,
        f: ExpectedFile,
        r: ImportRequest,
        e: SourceRef,
    ) -> Result<SaveReceipt, String> {
        self.current()?.import_package_resource(n, f, r, e)
    }
    fn apply_resource_plan(
        &mut self,
        n: &str,
        f: ExpectedFile,
        r: Vec<LibraryEdit>,
        e: SourceRef,
    ) -> Result<SaveReceipt, String> {
        self.current()?.apply_resource_plan(n, f, r, e)
    }
}
