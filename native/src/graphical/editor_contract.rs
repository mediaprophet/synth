//! Shared native editor contract; canonical execution remains in controller.
use super::SessionEpoch;
use crate::graphical::checkpoints::typed_10d as typed;
use crate::graphical::preview::{self, PreviewFrame, SourceRef};
use qualia_core_db::poet_host::invoke::studio_store;
use vibe::Value;

#[derive(Clone)]
pub struct EditorSnapshot {
    pub artifact_id: String,
    pub title: String,
    pub source_revision: u32,
    pub source: SourceRef,
    pub session_revision: u64,
    pub execution_epoch: SessionEpoch,
    pub rebuild_supported: bool,
    pub latest_source: String,
    pub reconciliation_required: bool,
    pub material_edit_id: Option<u64>,
    pub checkpoint_supported: bool,
    pub authorisation_stamp: Option<crate::graphical::cooperation::AuthorisationStamp>,
}

impl EditorSnapshot {
    /// Nonblocking source/execution/authority validation at publication.
    pub fn publication_is_current(&self) -> bool {
        self.execution_epoch.is_current()
            && self.source.check_current().is_ok()
            && self
                .authorisation_stamp
                .as_ref()
                .is_none_or(|stamp| stamp.is_current())
    }
}

pub struct EditorOutcome {
    pub snapshot: EditorSnapshot,
    pub message: String,
    pub frame: Option<PreviewFrame>,
}

pub trait PreviewProvider: Send {
    fn preview_selected_rig(&mut self, _source: SourceRef, _descriptor: crate::graphical::rig_binding::RigSourceDescriptor, _bytes: &[u8], _delta_seconds: f64, _cancelled: &std::sync::atomic::AtomicBool) -> Result<crate::graphical::viewport_contract::ViewportOutput, String> {
        Err("Rig preview is unavailable for this host".into())
    }
    fn stage_rig_control(&mut self, _source: SourceRef, _descriptor: crate::graphical::rig_binding::RigSourceDescriptor, _bytes: &[u8], _action: crate::graphical::rig_binding::RigControl, _cancelled: &std::sync::atomic::AtomicBool) -> Result<crate::graphical::rig_binding::rig_candidate::StagedRigControl,String> {
        Err("This preview provider cannot safely admit selected rig geometry".into())
    }
    /// Retire any state changed by an unpublishable rig frame. Stateless providers need no action.
    fn invalidate_rig_preview(&mut self) {}
    /// Must explicitly retire all retained viewport state or report failure.
    /// Stateless providers can return success; custom providers must override
    /// this to avoid quarantine after an unpublishable source or viewport result.
    fn retire_retained_viewport(&mut self) -> Result<(), String> { Err("preview provider cannot retire retained viewport".into()) }
    fn set_execution_context(&mut self, _epoch: &SessionEpoch) {}
    fn viewport_cached(&self, _source: &SourceRef) -> bool { false }
    fn viewport(&mut self, _request: crate::graphical::viewport::ViewportCommand,
        _source: SourceRef, _value: &Value,
        _resources: Option<&crate::graphical::surfaces::TextureResources>,
        _cancelled: &std::sync::atomic::AtomicBool,
    ) -> Result<crate::graphical::viewport_contract::ViewportOutput, String> {
        Err("This preview provider does not support native viewport interaction".into())
    }

    fn validate_with_resources(
        &self,
        value: &Value,
        resources: Option<&crate::graphical::surfaces::TextureResources>,
    ) -> Result<(), String> {
        self.validate(value)?;
        crate::graphical::surface_host::validate_value(value, resources)
    }
    fn preview_with_resources(
        &mut self,
        source: SourceRef,
        value: &Value,
        _resources: Option<&crate::graphical::surfaces::TextureResources>,
    ) -> Result<PreviewFrame, String> {
        self.preview(source, value)
    }

    /// Native output admission before local session/source promotion; no GPU.
    fn validate(&self, _value: &Value) -> Result<(), String> {
        Ok(())
    }
    /// Called with actual evaluated engine output after checked source commit.
    fn preview(&mut self, source: SourceRef, value: &Value) -> Result<PreviewFrame, String>;
}

pub struct MeshPreviewProvider {
    /// Caller-supplied native material/view inputs; no inferred physical rights.
    pub material: qualia_core_db::container_10d::MaterialRecord,
    pub camera: preview::PreviewCamera,
    pub limits: preview::PreviewLimits,
}

impl PreviewProvider for MeshPreviewProvider {
    // This provider returns newly rendered frames and retains no viewport state.
    fn retire_retained_viewport(&mut self) -> Result<(), String> { Ok(()) }
    fn validate_with_resources(
        &self,
        value: &Value,
        resources: Option<&crate::graphical::surfaces::TextureResources>,
    ) -> Result<(), String> {
        if crate::graphical::assembly::is_assembly(value) {
            let resolved = crate::graphical::surfaces::resolution::resolve(value, resources)?;
            let scene = crate::graphical::assembly::admit_assembly_with_materials(
                value,
                self.limits,
                &resolved.materials,
            )?;
            crate::graphical::surfaces::resources::validate_material_resources(
                &scene.materials,
                resolved.resources.as_ref(),
            )
        } else {
            self.validate(value)?;
            crate::graphical::surface_host::validate_value(value, resources)
        }
    }

    fn preview_with_resources(
        &mut self,
        source: SourceRef,
        value: &Value,
        resources: Option<&crate::graphical::surfaces::TextureResources>,
    ) -> Result<PreviewFrame, String> {
        if crate::graphical::assembly::is_assembly(value) {
            let resolved = crate::graphical::surfaces::resolution::resolve(value, resources)?;
            let scene = crate::graphical::assembly::admit_assembly_with_materials(
                value,
                self.limits,
                &resolved.materials,
            )?;
            return crate::graphical::assembly::render_assembly_with_resources(
                source,
                &scene,
                crate::graphical::assembly::AssemblyRenderSettings::for_camera(self.camera),
                self.limits,
                None,
                resolved.resources.as_ref(),
            );
        }
        self.preview(source, value)
    }

    fn validate(&self, value: &Value) -> Result<(), String> {
        if crate::graphical::assembly::is_assembly(value) {
            return crate::graphical::assembly::admit_assembly(value, self.limits).map(|_| ());
        }
        if let Value::Record(record) = value {
            if record.contains_key("geometry") || record.contains_key("material") {
                return crate::graphical::materials::authored_scene(
                    value,
                    self.material,
                    self.limits,
                )
                .map(|_| ());
            }
            if record.contains_key("base_color")
                && record.contains_key("metallic")
                && record.contains_key("roughness")
            {
                return crate::graphical::materials::material_from_value(value).map(|_| ());
            }
        }
        // Ordinary helper/function/scalar cells remain valid persistent REPL
        // submissions. A non-scene result simply cannot produce a mesh frame.
        Ok(())
    }
    fn preview(&mut self, source: SourceRef, value: &Value) -> Result<PreviewFrame, String> {
        if crate::graphical::assembly::is_assembly(value) {
            let assembly = crate::graphical::assembly::admit_assembly(value, self.limits)?;
            return crate::graphical::assembly::render_assembly(
                source,
                &assembly,
                self.camera,
                self.limits,
            );
        }
        let (mesh, material) =
            crate::graphical::materials::authored_scene(value, self.material, self.limits)?;
        preview::render_mesh(source, &mesh, material, self.camera, self.limits)
    }
}

pub trait EditorHost: Send {
    /// Resolve a caller-selected HMC resource for an admitted timeline clip.
    /// Checks the selected key/digest against the current source-owned clip
    /// association. That assertion is not proof of rights or universal identity.
    fn timeline_media(&self, _request: crate::graphical::checkpoints::TimelineMediaRequest, _expected: SourceRef, _cancelled: &std::sync::atomic::AtomicBool) -> Result<crate::graphical::checkpoints::TimelineMediaSelection, String> {
        Err("Timeline media inspection is unavailable for this host".into())
    }
    /// Trusted capability to acquire the same shared owner after authority admission.
    fn shared_viewport_host(&self) -> Option<crate::graphical::shared_host::SharedEditorHost> { None }
    /// Retire tentative derived state, or fail so the shared viewport can be quarantined.
    fn retire_retained_viewport(&mut self) -> Result<(), String> { Err("host cannot retire retained viewport".into()) }
    /// Internal checkpoint-to-preview handoff, only under current project read authority.
    fn preview_rig_from_selected_resource(&mut self, _resource_key: &str, _bytes: &[u8], _expected: SourceRef, _delta_seconds: f64, _cancelled: &std::sync::atomic::AtomicBool) -> Result<crate::graphical::viewport_contract::ViewportOutput, String> {
        Err("Selected rig preview is unavailable for this host".into())
    }
    /// Rig edits must use this cancellation-aware, authority-wrapped route.
    fn rig_authoring_edit(&mut self, _resource: crate::graphical::checkpoints::ResourceInspectionRequest, _control: crate::graphical::rig_binding::RigControl, _expected: SourceRef, _cancelled: &std::sync::atomic::AtomicBool) -> Result<EditorOutcome,String> {
        Err("This host cannot perform a cancellation-aware rig edit".into())
    }
    /// Internal checkpoint-to-source handoff; outer authority and selected-resource checks are mandatory.
    fn edit_rig_from_selected_resource(&mut self, _resource_key: &str, _bytes: &[u8], _control: crate::graphical::rig_binding::RigControl, _expected: SourceRef, _cancelled: &std::sync::atomic::AtomicBool) -> Result<EditorOutcome,String> {
        Err("This source owner cannot edit a selected rig".into())
    }
    fn viewport_action(&mut self, _request: crate::graphical::viewport::ViewportCommand,
        _expected: SourceRef, _cancelled: &std::sync::atomic::AtomicBool,
    ) -> Result<crate::graphical::viewport_contract::ViewportOutput, String> {
        Err("Native viewport is not available from this host".into())
    }

    /// Inert original source/package transport; does not activate or admit
    /// execution and must retain the current native ReadSource owner guard.
    fn retained_surface_carrier(
        &self,
        expected: SourceRef,
        limits: qualia_core_db::poet_host::invoke::studio_store::checkpoint::CheckpointLimits,
    ) -> Result<
        Option<qualia_core_db::poet_host::invoke::studio_store::checkpoint::VerifiedSourceCarrier>,
        String,
    > {
        let _ = (expected, limits);
        Ok(None)
    }

    fn retained_surface_inputs(
        &self,
        expected: SourceRef,
    ) -> Result<
        Vec<qualia_core_db::poet_host::invoke::studio_store::checkpoint::ResourceData>,
        String,
    > {
        let _ = expected;
        Ok(Vec::new())
    }

    fn inspect_surface(
        &self,
        _id: &str,
        _expected: SourceRef,
    ) -> Result<crate::graphical::surfaces::inspection::SurfaceInspection, String> {
        Err("Native surface inspection unavailable".into())
    }

    fn bind_surface_package(
        &mut self,
        _request: crate::graphical::surface_host::SurfacePackageRequest,
        _expected: SourceRef,
    ) -> Result<(), String> {
        Err("Native surface package binding unavailable".into())
    }
    /// Raw binding seam: the default cannot prove failure occurred before mutation.
    fn bind_surface_package_transaction(
        &mut self,
        request: crate::graphical::surface_host::SurfacePackageRequest,
        expected: SourceRef,
    ) -> Result<(), crate::graphical::surface_host::SurfaceBindingFailure> {
        self.bind_surface_package(request, expected)
            .map_err(crate::graphical::surface_host::SurfaceBindingFailure::MayHaveMutated)
    }
    /// Internal raw owner only; no command deserializes resources or forwards this through authority wrappers.
    fn install_surface_resources(
        &mut self,
        _resources: crate::graphical::surfaces::TextureResources,
        _expected: SourceRef,
    ) -> Result<(), String> {
        Err("Raw surface resource owner unavailable".into())
    }
    /// Raw owners may prove that validation failed before changing resources.
    /// Unknown implementations remain conservatively classified as mutated.
    fn install_surface_resources_transaction(
        &mut self,
        resources: crate::graphical::surfaces::TextureResources,
        expected: SourceRef,
    ) -> Result<(), crate::graphical::surface_host::SurfaceBindingFailure> {
        self.install_surface_resources(resources, expected)
            .map_err(crate::graphical::surface_host::SurfaceBindingFailure::MayHaveMutated)
    }

    fn inspect_typed_10d(
        &self,
        request: crate::graphical::checkpoints::ResourceInspectionRequest,
        expected: SourceRef,
        limits: typed::Typed10dLimits,
    ) -> Result<typed::Typed10dInspection, String> {
        let _ = (request, expected, limits);
        Err("Native typed .10d inspection unavailable".into())
    }
    fn edit_typed_10d(
        &mut self,
        request: typed::Typed10dEditRequest,
        expected: SourceRef,
        limits: typed::Typed10dLimits,
        cancelled: &std::sync::atomic::AtomicBool,
    ) -> Result<qualia_core_db::poet_host::invoke::studio_store::checkpoint::SaveReceipt, String>
    {
        let _ = (request, expected, limits, cancelled);
        Err("Native typed .10d editing unavailable".into())
    }

    fn package_workflow(
        &mut self,
        _action: crate::graphical::package_workflow::PackageWorkflowAction,
        _expected: SourceRef,
    ) -> Result<crate::graphical::package_workflow::PackageWorkflowView, String> {
        Err("Native exact-package workflow unavailable for this host".into())
    }

    fn edit_georaster(
        &mut self,
        request: crate::graphical::checkpoints::GeoRasterEditRequest,
        expected: SourceRef,
        limits: crate::graphical::checkpoints::GeoRasterEditLimits,
        cancelled: &std::sync::atomic::AtomicBool,
    ) -> Result<qualia_core_db::poet_host::invoke::studio_store::checkpoint::SaveReceipt, String>
    {
        let _ = (request, expected, limits, cancelled);
        Err("Native georaster authoring is unavailable for this host".into())
    }

    fn brush_georaster(
        &mut self,
        request: crate::graphical::checkpoints::GeoRasterBrushRequest,
        expected: SourceRef,
        limits: crate::graphical::checkpoints::GeoRasterEditLimits,
        cancelled: &std::sync::atomic::AtomicBool,
    ) -> Result<qualia_core_db::poet_host::invoke::studio_store::checkpoint::SaveReceipt, String>
    {
        let _ = (request, expected, limits, cancelled);
        Err("Native georaster authoring is unavailable for this host".into())
    }
    fn create_georaster(
        &mut self,
        request: crate::graphical::checkpoints::GeoRasterCreateRequest,
        expected: SourceRef,
        limits: crate::graphical::checkpoints::GeoRasterEditLimits,
        cancelled: &std::sync::atomic::AtomicBool,
    ) -> Result<qualia_core_db::poet_host::invoke::studio_store::checkpoint::SaveReceipt, String>
    {
        let _ = (request, expected, limits, cancelled);
        Err("Native georaster authoring is unavailable for this host".into())
    }

    fn numeric_sheet_view(&self, _expected: SourceRef) -> Result<crate::graphical::numeric_sheet::SheetView, String> { Err("Numeric sheet is unavailable for this host".into()) }
    fn numeric_sheet_edit(&mut self, _edit: crate::graphical::numeric_sheet::SheetEdit, _expected: SourceRef) -> Result<EditorOutcome, String> { Err("Numeric sheet editing is unavailable for this host".into()) }
    fn authoring_view(
        &self,
        _expected: SourceRef,
    ) -> Result<crate::graphical::authoring::AuthoringView, String> {
        Err("Native structured authoring is not configured for this host".into())
    }
    fn authoring_edit(
        &mut self,
        _edit: crate::graphical::authoring::AuthoringEdit,
        _expected: SourceRef,
    ) -> Result<EditorOutcome, String> {
        Err("Native structured authoring is not configured for this host".into())
    }
    fn compiled_assets(
        &self,
        _key: &str,
        _expected: SourceRef,
    ) -> Result<Vec<(String, qualia_core_db::render::compile_10d::CompiledAsset)>, String> {
        Err("Native source-bound asset compilation is not configured for this host".into())
    }
    fn compile_checkpoint(
        &mut self,
        _name: &str,
        _key: &str,
        _file: studio_store::checkpoint::ExpectedFile,
        _expected: SourceRef,
    ) -> Result<studio_store::checkpoint::SaveReceipt, String> {
        Err("Native compiled checkpoint persistence is not configured for this host".into())
    }
    fn workflow(
        &mut self,
        _action: crate::graphical::cooperative_workflow::WorkflowAction,
    ) -> Result<crate::graphical::cooperative_workflow::WorkflowView, String> {
        Err("Native cooperative operator service is not configured for this host".into())
    }
    fn apply_resource_plan(
        &mut self,
        name: &str,
        file: studio_store::checkpoint::ExpectedFile,
        plan: Vec<crate::graphical::package::LibraryEdit>,
        expected: SourceRef,
    ) -> Result<studio_store::checkpoint::SaveReceipt, String> {
        crate::graphical::package::validate_plan(&plan)?;
        let mut edits = Vec::with_capacity(plan.len());
        for edit in plan {
            match edit {
                crate::graphical::package::LibraryEdit::Remove {
                    key,
                    expected_resource,
                } => edits.push(studio_store::checkpoint::ResourceEdit::Remove {
                    key,
                    expected: expected_resource,
                }),
                _ => {
                    return Err(
                        "The native host has not configured a resource import library".into(),
                    )
                }
            }
        }
        self.resource_edit_preflight(name, file, expected.clone())?;
        self.edit_package_resources(name, file, edits, expected)
    }
    fn resource_edit_preflight(
        &self,
        _name: &str,
        _file: studio_store::checkpoint::ExpectedFile,
        _expected: SourceRef,
    ) -> Result<(), String> {
        Err("Resource editing preflight is unavailable for this host".into())
    }
    fn resource_import_supported(&self) -> bool {
        false
    }
    fn package_inventory(
        &self,
        _name: &str,
        _expected: SourceRef,
    ) -> Result<crate::graphical::checkpoints::PackageInventory, String> {
        Err("HMC resource inspection is unavailable for this host".into())
    }
    fn inspect_package_resource(
        &self,
        _request: crate::graphical::checkpoints::ResourceInspectionRequest,
        _expected: SourceRef,
        _limits: qualia_core_db::q42::m64_matter::M64ReadLimits,
        _scratch: &mut [qualia_core_db::q42::m64_matter::M64PayloadInterval],
    ) -> Result<crate::graphical::checkpoints::ResourceInspection, String> {
        Err("Selected resource inspection is unavailable for this host".into())
    }
    fn edit_package_resources(
        &mut self,
        _name: &str,
        _expected_file: studio_store::checkpoint::ExpectedFile,
        _edits: Vec<studio_store::checkpoint::ResourceEdit>,
        _expected: SourceRef,
    ) -> Result<studio_store::checkpoint::SaveReceipt, String> {
        Err("HMC resource editing is unavailable for this host".into())
    }
    fn import_package_resource(
        &mut self,
        _name: &str,
        _expected_file: studio_store::checkpoint::ExpectedFile,
        _request: crate::graphical::package::ImportRequest,
        _expected: SourceRef,
    ) -> Result<studio_store::checkpoint::SaveReceipt, String> {
        Err("The native host has not configured a resource import library".into())
    }
    fn source_cells(&self, _expected: SourceRef) -> Result<Vec<String>, String> {
        Err("identified source-cell read is unavailable for this host".into())
    }
    fn material_edit(
        &mut self,
        source: &str,
        expected: SourceRef,
    ) -> Result<EditorOutcome, String> {
        let _ = (source, expected);
        Err("material-only source admission is unavailable for this host".into())
    }
    fn checkpoint_supported(&self) -> bool {
        false
    }
    fn save_checkpoint(
        &mut self,
        _name: &str,
        _expected_file: studio_store::checkpoint::ExpectedFile,
        _expected: SourceRef,
    ) -> Result<studio_store::checkpoint::SaveReceipt, String> {
        Err("native checkpoint destination/admission is unavailable for this host".into())
    }
    fn snapshot(&self) -> Result<EditorSnapshot, String>;
    fn check(&self, source: &str) -> Result<String, String>;
    fn submit(&mut self, source: &str, expected: SourceRef) -> Result<EditorOutcome, String>;
    fn undo(&mut self, expected: SourceRef) -> Result<EditorSnapshot, String>;
    fn redo(&mut self, expected: SourceRef) -> Result<EditorSnapshot, String>;
    fn rebuild_execution(&mut self, _expected: SourceRef) -> Result<EditorOutcome, String> {
        Err("explicit execution rebuild is unavailable for this host profile".into())
    }
}
