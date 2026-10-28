//! Host-selected resource library. Filenames express intent, never filesystem authority.
use crate::graphical::checkpoints::typed_10d as typed;
mod intents;
mod library;
mod plan;
pub use intents::parse_resource_plan_json;
pub(crate) use intents::{resource_plan, resource_plan_schema, ResourceIntent};
pub use library::ResourceLibrary;

use super::checkpoints::PackageInventory;
use super::controller::{EditorHost, EditorOutcome, EditorSnapshot};
use super::preview::SourceRef;
use qualia_core_db::poet_host::invoke::studio_store::checkpoint::{
    ExpectedFile, ResourceData, ResourceEdit, SaveReceipt,
};

#[derive(Clone, Debug)]
pub struct ImportRequest {
    pub filename: String,
    pub key: String,
    /// Required for additions; replacing content preserves existing kind and metadata.
    pub kind: String,
    pub expected_resource: Option<[u8; 32]>,
}

#[derive(Clone, Debug)]
pub enum LibraryEdit {
    Import(ImportRequest),
    Remove {
        key: String,
        expected_resource: [u8; 32],
    },
}

pub const MAX_RESOURCE_EDITS: usize = 64;
pub(crate) fn validate_plan(plan: &[LibraryEdit]) -> Result<(), String> {
    plan::validate_requests(plan)
}

/// Place inside SharedEditorHost, with each frontend's AuthorizedEditorHost outside.
pub struct ResourceImportHost {
    inner: Box<dyn EditorHost>,
    library: ResourceLibrary,
}
impl ResourceImportHost {
    pub fn new(inner: Box<dyn EditorHost>, library: ResourceLibrary) -> Self {
        Self { inner, library }
    }
}
impl EditorHost for ResourceImportHost {
    fn retire_retained_viewport(&mut self) -> Result<(), String> { self.inner.retire_retained_viewport() }
    fn viewport_action(&mut self, request: crate::graphical::viewport::ViewportCommand,
        expected: SourceRef, cancelled: &std::sync::atomic::AtomicBool,
    ) -> Result<crate::graphical::viewport_contract::ViewportOutput, String> {
        self.inner.viewport_action(request, expected, cancelled)
    }

    fn retained_surface_carrier(
        &self,
        expected: SourceRef,
        limits: qualia_core_db::poet_host::invoke::studio_store::checkpoint::CheckpointLimits,
    ) -> Result<Option<qualia_core_db::poet_host::invoke::studio_store::checkpoint::VerifiedSourceCarrier>, String> {
        self.inner.retained_surface_carrier(expected, limits)
    }
    fn retained_surface_inputs(
        &self,
        expected: SourceRef,
    ) -> Result<
        Vec<qualia_core_db::poet_host::invoke::studio_store::checkpoint::ResourceData>,
        String,
    > {
        self.inner.retained_surface_inputs(expected)
    }

    fn inspect_surface(
        &self,
        id: &str,
        expected: SourceRef,
    ) -> Result<crate::graphical::surfaces::inspection::SurfaceInspection, String> {
        self.inner.inspect_surface(id, expected)
    }

    fn bind_surface_package(
        &mut self,
        request: crate::graphical::surface_host::SurfacePackageRequest,
        expected: SourceRef,
    ) -> Result<(), String> {
        request.validate()?;
        self.inner.bind_surface_package(request, expected)
    }

    fn bind_surface_package_transaction(
        &mut self,
        request: crate::graphical::surface_host::SurfacePackageRequest,
        expected: SourceRef,
    ) -> Result<(), crate::graphical::surface_host::SurfaceBindingFailure> {
        use crate::graphical::surface_host::SurfaceBindingFailure;
        request.validate().map_err(SurfaceBindingFailure::BeforeMutation)?;
        self.inner.bind_surface_package_transaction(request, expected)
    }

    fn inspect_typed_10d(
        &self,
        request: crate::graphical::checkpoints::ResourceInspectionRequest,
        expected: SourceRef,
        limits: typed::Typed10dLimits,
    ) -> Result<typed::Typed10dInspection, String> {
        self.inner.inspect_typed_10d(request, expected, limits)
    }
    fn edit_typed_10d(
        &mut self,
        request: typed::Typed10dEditRequest,
        expected: SourceRef,
        limits: typed::Typed10dLimits,
        cancelled: &std::sync::atomic::AtomicBool,
    ) -> Result<qualia_core_db::poet_host::invoke::studio_store::checkpoint::SaveReceipt, String>
    {
        self.inner
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
        self.inner
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
        self.inner
            .create_georaster(request, expected, limits, cancelled)
    }

    fn inspect_package_resource(
        &self,
        request: super::checkpoints::ResourceInspectionRequest,
        expected: SourceRef,
        limits: qualia_core_db::q42::m64_matter::M64ReadLimits,
        scratch: &mut [qualia_core_db::q42::m64_matter::M64PayloadInterval],
    ) -> Result<super::checkpoints::ResourceInspection, String> {
        self.inner
            .inspect_package_resource(request, expected, limits, scratch)
    }
    fn numeric_sheet_view(&self, expected: SourceRef) -> Result<crate::graphical::numeric_sheet::SheetView, String> { self.inner.numeric_sheet_view(expected) }
    fn numeric_sheet_edit(&mut self, edit: crate::graphical::numeric_sheet::SheetEdit, expected: SourceRef) -> Result<EditorOutcome, String> { self.inner.numeric_sheet_edit(edit, expected) }
    fn authoring_view(
        &self,
        e: SourceRef,
    ) -> Result<crate::graphical::authoring::AuthoringView, String> {
        self.inner.authoring_view(e)
    }
    fn rig_authoring_edit(&mut self, resource: crate::graphical::checkpoints::ResourceInspectionRequest, control: crate::graphical::rig_binding::RigControl, expected: SourceRef, cancelled: &std::sync::atomic::AtomicBool) -> Result<EditorOutcome,String> {
        self.inner.rig_authoring_edit(resource, control, expected, cancelled)
    }
    fn authoring_edit(
        &mut self,
        a: crate::graphical::authoring::AuthoringEdit,
        e: SourceRef,
    ) -> Result<EditorOutcome, String> {
        self.inner.authoring_edit(a, e)
    }
    fn compiled_assets(
        &self,
        k: &str,
        e: SourceRef,
    ) -> Result<Vec<(String, qualia_core_db::render::compile_10d::CompiledAsset)>, String> {
        self.inner.compiled_assets(k, e)
    }
    fn compile_checkpoint(
        &mut self,
        n: &str,
        k: &str,
        f: qualia_core_db::poet_host::invoke::studio_store::checkpoint::ExpectedFile,
        e: SourceRef,
    ) -> Result<qualia_core_db::poet_host::invoke::studio_store::checkpoint::SaveReceipt, String>
    {
        self.inner.compile_checkpoint(n, k, f, e)
    }

    fn apply_resource_plan(
        &mut self,
        n: &str,
        f: ExpectedFile,
        edits: Vec<LibraryEdit>,
        e: SourceRef,
    ) -> Result<SaveReceipt, String> {
        plan::apply(self, n, f, edits, e)
    }
    fn resource_edit_preflight(
        &self,
        n: &str,
        f: ExpectedFile,
        e: SourceRef,
    ) -> Result<(), String> {
        self.inner.resource_edit_preflight(n, f, e)
    }
    fn resource_import_supported(&self) -> bool {
        true
    }
    fn snapshot(&self) -> Result<EditorSnapshot, String> {
        self.inner.snapshot()
    }
    fn check(&self, s: &str) -> Result<String, String> {
        self.inner.check(s)
    }
    fn submit(&mut self, s: &str, e: SourceRef) -> Result<EditorOutcome, String> {
        self.inner.submit(s, e)
    }
    fn undo(&mut self, e: SourceRef) -> Result<EditorSnapshot, String> {
        self.inner.undo(e)
    }
    fn redo(&mut self, e: SourceRef) -> Result<EditorSnapshot, String> {
        self.inner.redo(e)
    }
    fn rebuild_execution(&mut self, e: SourceRef) -> Result<EditorOutcome, String> {
        self.inner.rebuild_execution(e)
    }
    fn source_cells(&self, e: SourceRef) -> Result<Vec<String>, String> {
        self.inner.source_cells(e)
    }
    fn material_edit(&mut self, s: &str, e: SourceRef) -> Result<EditorOutcome, String> {
        self.inner.material_edit(s, e)
    }
    fn checkpoint_supported(&self) -> bool {
        self.inner.checkpoint_supported()
    }
    fn save_checkpoint(
        &mut self,
        n: &str,
        f: ExpectedFile,
        e: SourceRef,
    ) -> Result<SaveReceipt, String> {
        self.inner.save_checkpoint(n, f, e)
    }
    fn package_inventory(&self, n: &str, e: SourceRef) -> Result<PackageInventory, String> {
        self.inner.package_inventory(n, e)
    }
    fn edit_package_resources(
        &mut self,
        n: &str,
        f: ExpectedFile,
        r: Vec<ResourceEdit>,
        e: SourceRef,
    ) -> Result<SaveReceipt, String> {
        self.inner.edit_package_resources(n, f, r, e)
    }
    fn import_package_resource(
        &mut self,
        n: &str,
        f: ExpectedFile,
        r: ImportRequest,
        e: SourceRef,
    ) -> Result<SaveReceipt, String> {
        if self.inner.snapshot()?.source != e {
            return Err("Resource import source changed before file access".into());
        }
        e.check_current()?;
        // Invalid resource intent must fail before reading the selected native file.
        if r.key == "authoring/native-source.json"
            || r.key.is_empty()
            || r.key.len() > 4096
            || r.kind.len() > 4096
            || r.key.contains('\0')
            || r.kind.contains('\0')
        {
            return Err("Invalid resource key/kind".into());
        }
        if r.expected_resource.is_none() && r.kind.is_empty() {
            return Err("New resources require an explicit kind".into());
        }
        self.inner.resource_edit_preflight(n, f, e.clone())?;
        let inventory = self.inner.package_inventory(n, e.clone())?;
        if !matches!(f,ExpectedFile::Sha256(digest) if digest==inventory.digest) {
            return Err("Resource package digest changed before file access".into());
        }
        let existing = inventory.resources.iter().find(|entry| entry.key == r.key);
        match (r.expected_resource, existing) {
            (Some(expected), Some(entry)) if expected == entry.digest => {}
            (None, None) => {}
            _ => return Err("Resource key/content changed before file access".into()),
        }
        let bytes = self.library.read(&r.filename)?;
        let edit = match r.expected_resource {
            Some(expected) => ResourceEdit::ReplaceBytes {
                key: r.key,
                expected,
                bytes,
            },
            None => ResourceEdit::Add(ResourceData {
                key: r.key,
                kind: r.kind,
                bytes,
                meta: None,
            }),
        };
        self.inner.edit_package_resources(n, f, vec![edit], e)
    }
}

pub fn hex_digest(bytes: &[u8; 32]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
pub(crate) fn durability_note(receipt: &SaveReceipt) -> String {
    receipt
        .durability_warning
        .as_ref()
        .map(|warning| format!("; file installation committed, durability warning: {warning}"))
        .unwrap_or_default()
}
pub fn parse_digest(text: &str) -> Result<[u8; 32], String> {
    if text.len() != 64 || !text.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err("Expected exactly 64 hexadecimal SHA256 characters".into());
    }
    let mut digest = [0; 32];
    for (i, pair) in text.as_bytes().chunks_exact(2).enumerate() {
        let digit = |b: u8| {
            if b <= b'9' {
                b - b'0'
            } else {
                b.to_ascii_lowercase() - b'a' + 10
            }
        };
        digest[i] = (digit(pair[0]) << 4) | digit(pair[1]);
    }
    Ok(digest)
}
#[cfg(test)]
mod tests;
