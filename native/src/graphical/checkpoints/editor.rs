//! Decorator mounted inside the native AuthorizedEditorHost. All caller intent
//! must pass that current owner gate before this service reaches filesystem IO.
use super::super::controller::{EditorHost, EditorOutcome, EditorSnapshot};
use super::super::preview::SourceRef;
use super::{CheckpointService, ExpectedFile, PackageInventory, ResourceEdit, SaveReceipt};
use crate::graphical::checkpoints::typed_10d as typed;

pub struct CheckpointEditorHost {
    inner: Box<dyn EditorHost>,
    service: CheckpointService,
}

impl CheckpointEditorHost {
    pub fn new(inner: Box<dyn EditorHost>, service: CheckpointService) -> Result<Self, String> {
        if inner.snapshot()?.artifact_id != service.binding().source_id {
            return Err("checkpoint service does not match the selected native source host".into());
        }
        Ok(Self { inner, service })
    }
}

impl EditorHost for CheckpointEditorHost {
    fn viewport_action(&mut self, request: crate::graphical::viewport::ViewportCommand,
        expected: SourceRef, cancelled: &std::sync::atomic::AtomicBool,
    ) -> Result<crate::graphical::viewport_contract::ViewportOutput, String> {
        self.inner.viewport_action(request, expected, cancelled)
    }

    fn retained_surface_carrier(
        &self,
        expected: SourceRef,
        limits: qualia_core_db::poet_host::invoke::studio_store::checkpoint::CheckpointLimits,
    ) -> Result<
        Option<qualia_core_db::poet_host::invoke::studio_store::checkpoint::VerifiedSourceCarrier>,
        String,
    > {
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
        {
            let resources = self.service.surface_resources(&request, &expected)?;
            self.inner.install_surface_resources(resources, expected)
        }
    }

    fn inspect_typed_10d(
        &self,
        request: crate::graphical::checkpoints::ResourceInspectionRequest,
        expected: SourceRef,
        limits: typed::Typed10dLimits,
    ) -> Result<typed::Typed10dInspection, String> {
        if self.inner.snapshot()?.source != expected {
            return Err("Typed .10d source changed; no read".into());
        }
        expected.check_current()?;
        let result = self.service.inspect_typed_10d(request, &expected, limits)?;
        expected.check_current()?;
        Ok(result)
    }
    fn edit_typed_10d(
        &mut self,
        request: typed::Typed10dEditRequest,
        expected: SourceRef,
        limits: typed::Typed10dLimits,
        cancelled: &std::sync::atomic::AtomicBool,
    ) -> Result<qualia_core_db::poet_host::invoke::studio_store::checkpoint::SaveReceipt, String>
    {
        if self.inner.snapshot()?.source != expected {
            return Err("Typed .10d source changed; no write".into());
        }
        expected.check_current()?;
        self.service
            .edit_typed_10d(request, &expected, limits, cancelled)
    }

    fn create_georaster(
        &mut self,
        request: super::GeoRasterCreateRequest,
        expected: SourceRef,
        limits: super::GeoRasterEditLimits,
        cancelled: &std::sync::atomic::AtomicBool,
    ) -> Result<SaveReceipt, String> {
        if self.inner.snapshot()?.source != expected {
            return Err("Native raster producer source changed; no write".into());
        }
        expected.check_current()?;
        let mut scratch = [qualia_core_db::q42::m64_matter::M64PayloadInterval::default();
            super::UI_INSPECTION_CHUNKS];
        self.service
            .create_georaster(request, &expected.stamp, limits, cancelled, &mut scratch)
    }
    fn edit_georaster(
        &mut self,
        request: super::GeoRasterEditRequest,
        expected: SourceRef,
        limits: super::GeoRasterEditLimits,
        cancelled: &std::sync::atomic::AtomicBool,
    ) -> Result<SaveReceipt, String> {
        if self.inner.snapshot()?.source != expected {
            return Err("Native raster edit source changed; no write".into());
        }
        expected.check_current()?;
        let mut scratch = [qualia_core_db::q42::m64_matter::M64PayloadInterval::default();
            super::UI_INSPECTION_CHUNKS];
        self.service
            .edit_georaster(request, &expected.stamp, limits, cancelled, &mut scratch)
    }
    fn inspect_package_resource(
        &self,
        request: super::ResourceInspectionRequest,
        expected: SourceRef,
        limits: qualia_core_db::q42::m64_matter::M64ReadLimits,
        scratch: &mut [qualia_core_db::q42::m64_matter::M64PayloadInterval],
    ) -> Result<super::ResourceInspection, String> {
        if self.inner.snapshot()?.source != expected {
            return Err("Native resource inspection source changed; no read".into());
        }
        expected.check_current()?;
        let result = self.service.inspect_resource(request, limits, scratch)?;
        expected.check_current()?;
        if self.inner.snapshot()?.source != expected {
            return Err("Native resource inspection source changed; result withheld".into());
        }
        Ok(result)
    }
    fn authoring_view(
        &self,
        e: SourceRef,
    ) -> Result<crate::graphical::authoring::AuthoringView, String> {
        self.inner.authoring_view(e)
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
        crate::graphical::authoring::package::compile_checkpoint(
            self.inner.as_ref(),
            &self.service,
            self.service.limits,
            n,
            k,
            f,
            e,
        )
    }

    fn snapshot(&self) -> Result<EditorSnapshot, String> {
        self.inner.snapshot().map(|mut snapshot| {
            snapshot.checkpoint_supported = true;
            snapshot
        })
    }
    fn check(&self, source: &str) -> Result<String, String> {
        self.inner.check(source)
    }
    fn submit(&mut self, source: &str, expected: SourceRef) -> Result<EditorOutcome, String> {
        self.inner.submit(source, expected)
    }
    fn undo(&mut self, expected: SourceRef) -> Result<EditorSnapshot, String> {
        self.inner.undo(expected)
    }
    fn redo(&mut self, expected: SourceRef) -> Result<EditorSnapshot, String> {
        self.inner.redo(expected)
    }
    fn rebuild_execution(&mut self, expected: SourceRef) -> Result<EditorOutcome, String> {
        self.inner.rebuild_execution(expected)
    }
    fn source_cells(&self, expected: SourceRef) -> Result<Vec<String>, String> {
        self.inner.source_cells(expected)
    }
    fn material_edit(
        &mut self,
        source: &str,
        expected: SourceRef,
    ) -> Result<EditorOutcome, String> {
        self.inner.material_edit(source, expected)
    }
    fn checkpoint_supported(&self) -> bool {
        true
    }
    fn package_inventory(
        &self,
        name: &str,
        expected: SourceRef,
    ) -> Result<PackageInventory, String> {
        if self.inner.snapshot()?.source != expected {
            return Err("native package inventory source changed".into());
        }
        expected.check_current()?;
        self.service.inventory(name)
    }
    fn resource_edit_preflight(
        &self,
        name: &str,
        file: ExpectedFile,
        expected: SourceRef,
    ) -> Result<(), String> {
        if self.inner.snapshot()?.source != expected {
            return Err("native resource import source changed; no import read".into());
        }
        expected.check_current()?;
        self.service.preflight_resource_edit(name, file)
    }
    fn edit_package_resources(
        &mut self,
        name: &str,
        expected_file: ExpectedFile,
        edits: Vec<ResourceEdit>,
        expected: SourceRef,
    ) -> Result<SaveReceipt, String> {
        if self.inner.snapshot()?.source != expected {
            return Err("native resource edit source changed; no write".into());
        }
        expected.check_current()?;
        let parent = match expected_file {
            ExpectedFile::Absent => None,
            ExpectedFile::Sha256(digest) => Some(digest),
        };
        self.service
            .save_with_resource_edits(&expected.stamp, name, expected_file, parent, vec![], edits)
            .map(|saved| saved.receipt)
    }
    fn save_checkpoint(
        &mut self,
        name: &str,
        expected_file: ExpectedFile,
        expected: SourceRef,
    ) -> Result<SaveReceipt, String> {
        let current = self.inner.snapshot()?;
        if current.source != expected {
            return Err("native checkpoint request source changed; no write".into());
        }
        expected.check_current()?;
        // Digest is accepted as lineage only after the service reloads and
        // validates those exact existing bytes against this source association.
        let parent = match expected_file {
            ExpectedFile::Absent => None,
            ExpectedFile::Sha256(digest) => Some(digest),
        };
        // Durable source/history save is available while execution is held.
        // This method does not evaluate source merely to manufacture resources.
        let carrier = self
            .inner
            .retained_surface_carrier(expected.clone(), self.service.limits)?;
        let inputs = if carrier.is_none() {
            self.inner.retained_surface_inputs(expected.clone())?
        } else {
            Vec::new()
        };
        self.service
            .save_with_source_carrier(
                &expected.stamp,
                name,
                expected_file,
                parent,
                Vec::new(),
                Vec::new(),
                inputs,
                carrier.as_ref(),
            )
            .map(|saved| saved.receipt)
    }
}
