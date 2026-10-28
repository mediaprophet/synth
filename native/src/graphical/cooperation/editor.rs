use super::*;
use crate::graphical::checkpoints::typed_10d as typed;

impl EditorHost for AuthorizedEditorHost {
    fn viewport_action(&mut self, request: crate::graphical::viewport::ViewportCommand,
        expected: SourceRef, cancelled: &std::sync::atomic::AtomicBool,
    ) -> Result<crate::graphical::viewport_contract::ViewportOutput, String> {
        crate::graphical::viewport_contract::validate_command(&request)?;
        if cancelled.load(std::sync::atomic::Ordering::Acquire) { return Err("Viewport cancelled before authorization".into()); }
        // Authority -> shared-editor is the established lock order. Keep the
        // same guard through every owner/post-source check, so no other frontend
        // can observe tentative state or be invalidated by delayed cleanup.
        let shared = self.inner.shared_viewport_host().ok_or(
            "Viewport requires a shared transaction-capable editor owner")?;
        let mut guard = None;
        let result = with_authorized(self.owner.as_ref(), &self.scope, Some(&expected),
            AuthoringAction::ReadSource, self.limits, |_, _| {
                guard = Some(shared.viewport_guard()?);
                if cancelled.load(std::sync::atomic::Ordering::Acquire) { return Err("Viewport cancelled before dispatch".into()); }
                guard.as_mut().expect("guard just acquired").viewport_action(request, expected.clone(), cancelled)
            });
        // This final checked transition is the publication point. Cancellation
        // after it cannot retroactively revoke a frame already returned.
        let result = if result.is_ok() && cancelled.load(std::sync::atomic::Ordering::Acquire) {
            Err("Viewport cancelled before publication".into())
        } else { result };
        match result {
            Ok(output) => Ok(output),
            Err(error) => {
                if let Some(editor) = guard.as_mut() {
                    if editor.retire_retained_viewport().is_err() {
                        shared.quarantine_viewport();
                        Err(format!("{error}; retained viewport quarantined, shared host reconstruction required"))
                    } else {
                        Err(format!("{error}; viewport reset; redraw and reselect to continue"))
                    }
                } else { Err(error) }
            }
        }
    }

    fn retained_surface_carrier(
        &self,
        expected: SourceRef,
        limits: qualia_core_db::poet_host::invoke::studio_store::checkpoint::CheckpointLimits,
    ) -> Result<
        Option<qualia_core_db::poet_host::invoke::studio_store::checkpoint::VerifiedSourceCarrier>,
        String,
    > {
        with_authorized(
            self.owner.as_ref(),
            &self.scope,
            Some(&expected),
            AuthoringAction::ReadSource,
            self.limits,
            |_, _| {
                self.inner
                    .retained_surface_carrier(expected.clone(), limits)
            },
        )
    }

    fn retained_surface_inputs(
        &self,
        expected: SourceRef,
    ) -> Result<
        Vec<qualia_core_db::poet_host::invoke::studio_store::checkpoint::ResourceData>,
        String,
    > {
        with_authorized(
            self.owner.as_ref(),
            &self.scope,
            Some(&expected),
            AuthoringAction::ReadSource,
            self.limits,
            |_, _| self.inner.retained_surface_inputs(expected.clone()),
        )
    }

    fn inspect_surface(
        &self,
        id: &str,
        expected: SourceRef,
    ) -> Result<crate::graphical::surfaces::inspection::SurfaceInspection, String> {
        with_authorized(
            self.owner.as_ref(),
            &self.scope,
            Some(&expected),
            AuthoringAction::ReadSource,
            self.limits,
            |_, _| self.inner.inspect_surface(id, expected.clone()),
        )
    }

    fn bind_surface_package(
        &mut self,
        request: crate::graphical::surface_host::SurfacePackageRequest,
        expected: SourceRef,
    ) -> Result<(), String> {
        request.validate()?;
        // Acquire after authority admission and retain the same shared owner
        // through the owner's final source and admission checks. Binding can
        // replace resources and retire an epoch, neither of which is undone by
        // invalidating only a viewport frame.
        let shared = self.inner.shared_viewport_host().ok_or(
            "Surface binding requires a shared transaction-capable editor owner")?;
        let mut guard = None;
        let mut may_have_changed = false;
        let result = with_authorized(
            self.owner.as_ref(), &self.scope, Some(&expected),
            AuthoringAction::ReadSource, self.limits, |_, _| {
                guard = Some(shared.viewport_guard()?);
                use crate::graphical::surface_host::SurfaceBindingFailure;
                match guard.as_mut().expect("guard just acquired")
                    .bind_surface_package_transaction(request, expected.clone()) {
                    Ok(()) => { may_have_changed = true; Ok(()) }
                    Err(SurfaceBindingFailure::BeforeMutation(error)) => Err(error),
                    Err(SurfaceBindingFailure::MayHaveMutated(error)) => { may_have_changed = true; Err(error) }
                }
            },
        );
        match result {
            Ok(()) => Ok(()),
            Err(error) if may_have_changed => {
                // A failing raw host may already have replaced the resources or
                // epoch. Never expose that state and do not claim rollback.
                shared.quarantine_viewport();
                Err(format!("{error}; surface binding may have changed host state; shared host quarantined; reconstruct and reconcile before retry"))
            }
            Err(error) => Err(error),
        }
    }

    fn inspect_typed_10d(
        &self,
        request: crate::graphical::checkpoints::ResourceInspectionRequest,
        expected: SourceRef,
        limits: typed::Typed10dLimits,
    ) -> Result<typed::Typed10dInspection, String> {
        request.validate_bounds()?;
        with_authorized(
            self.owner.as_ref(),
            &self.scope,
            Some(&expected),
            AuthoringAction::ReadSource,
            self.limits,
            |_, _| {
                self.inner
                    .inspect_typed_10d(request, expected.clone(), limits)
            },
        )
    }
    fn edit_typed_10d(
        &mut self,
        request: typed::Typed10dEditRequest,
        expected: SourceRef,
        limits: typed::Typed10dLimits,
        cancelled: &std::sync::atomic::AtomicBool,
    ) -> Result<qualia_core_db::poet_host::invoke::studio_store::checkpoint::SaveReceipt, String>
    {
        request.validate_bounds()?;
        let name = request.resource.filename.clone();
        let file = studio_store::checkpoint::ExpectedFile::Sha256(request.resource.package_digest);
        self.write_package(&name, file, &expected, |inner| {
            inner.edit_typed_10d(request, expected.clone(), limits, cancelled)
        })
    }

    fn edit_georaster(
        &mut self,
        request: crate::graphical::checkpoints::GeoRasterEditRequest,
        expected: SourceRef,
        limits: crate::graphical::checkpoints::GeoRasterEditLimits,
        cancelled: &std::sync::atomic::AtomicBool,
    ) -> Result<qualia_core_db::poet_host::invoke::studio_store::checkpoint::SaveReceipt, String>
    {
        request.validate_bounds()?;
        let name = request.resource.filename.clone();
        let file = studio_store::checkpoint::ExpectedFile::Sha256(request.resource.package_digest);
        self.write_package(&name, file, &expected, |inner| {
            inner.edit_georaster(request, expected.clone(), limits, cancelled)
        })
    }
    fn create_georaster(
        &mut self,
        request: crate::graphical::checkpoints::GeoRasterCreateRequest,
        expected: SourceRef,
        limits: crate::graphical::checkpoints::GeoRasterEditLimits,
        cancelled: &std::sync::atomic::AtomicBool,
    ) -> Result<qualia_core_db::poet_host::invoke::studio_store::checkpoint::SaveReceipt, String>
    {
        request.validate_bounds()?;
        let name = request.filename.clone();
        let file = studio_store::checkpoint::ExpectedFile::Sha256(request.package_digest);
        self.write_package(&name, file, &expected, |inner| {
            inner.create_georaster(request, expected.clone(), limits, cancelled)
        })
    }

    fn inspect_package_resource(
        &self,
        request: crate::graphical::checkpoints::ResourceInspectionRequest,
        expected: SourceRef,
        limits: qualia_core_db::q42::m64_matter::M64ReadLimits,
        scratch: &mut [qualia_core_db::q42::m64_matter::M64PayloadInterval],
    ) -> Result<crate::graphical::checkpoints::ResourceInspection, String> {
        with_authorized(
            self.owner.as_ref(),
            &self.scope,
            Some(&expected),
            AuthoringAction::ReadSource,
            self.limits,
            |_, _| {
                self.inner
                    .inspect_package_resource(request, expected.clone(), limits, scratch)
            },
        )
    }
    fn numeric_sheet_view(&self, expected: SourceRef) -> Result<crate::graphical::numeric_sheet::SheetView, String> {
        with_authorized(self.owner.as_ref(), &self.scope, Some(&expected), AuthoringAction::ReadSource, self.limits, |_, _| self.inner.numeric_sheet_view(expected.clone()))
    }
    fn numeric_sheet_edit(&mut self, edit: crate::graphical::numeric_sheet::SheetEdit, expected: SourceRef) -> Result<EditorOutcome, String> {
        self.write_with_view(&expected, AuthoringAction::ExecuteSource, |view, inner| {
            authoring::authorize(view, &AuthoringRequest { action: AuthoringAction::ReadSource, target: view.current_document, expected_authority_generation: view.generation }).map_err(|error| error.to_string())?;
            inner.numeric_sheet_edit(edit, expected.clone())
        })
    }
    fn authoring_view(
        &self,
        e: SourceRef,
    ) -> Result<crate::graphical::authoring::AuthoringView, String> {
        with_authorized(
            self.owner.as_ref(),
            &self.scope,
            Some(&e),
            AuthoringAction::ReadSource,
            self.limits,
            |_, _| self.inner.authoring_view(e.clone()),
        )
    }
    fn rig_authoring_edit(&mut self, resource: crate::graphical::checkpoints::ResourceInspectionRequest, control: crate::graphical::rig_binding::RigControl, expected: SourceRef, cancelled: &std::sync::atomic::AtomicBool) -> Result<EditorOutcome,String> {
        if cancelled.load(std::sync::atomic::Ordering::Acquire) { return Err("Rig edit cancelled before authorization".into()); }
        resource.validate_bounds()?;
        self.write_with_view(&expected, AuthoringAction::ExecuteSource, |view, inner| {
            authoring::authorize(view, &AuthoringRequest { action: AuthoringAction::ReadSource, target: view.current_document, expected_authority_generation: view.generation }).map_err(|error| error.to_string())?;
            if cancelled.load(std::sync::atomic::Ordering::Acquire) { return Err("Rig edit cancelled before resource admission".into()); }
            inner.rig_authoring_edit(resource, control, expected.clone(), cancelled)
        })
    }
    fn authoring_edit(
        &mut self,
        a: crate::graphical::authoring::AuthoringEdit,
        e: SourceRef,
    ) -> Result<EditorOutcome, String> {
        if matches!(&a, crate::graphical::authoring::AuthoringEdit::Rig { .. }) { return Err("Rig edit requires the cancellation-aware route".into()); }
        self.write(&e, AuthoringAction::ExecuteSource, |inner| { inner.authoring_edit(a, e.clone()) })
    }
    fn compiled_assets(
        &self,
        k: &str,
        e: SourceRef,
    ) -> Result<Vec<(String, qualia_core_db::render::compile_10d::CompiledAsset)>, String> {
        with_authorized(
            self.owner.as_ref(),
            &self.scope,
            Some(&e),
            AuthoringAction::ReadSource,
            self.limits,
            |_, _| self.inner.compiled_assets(k, e.clone()),
        )
    }
    fn compile_checkpoint(
        &mut self,
        n: &str,
        k: &str,
        f: qualia_core_db::poet_host::invoke::studio_store::checkpoint::ExpectedFile,
        e: SourceRef,
    ) -> Result<qualia_core_db::poet_host::invoke::studio_store::checkpoint::SaveReceipt, String>
    {
        self.write_package(n, f, &e, |inner| {
            inner.compile_checkpoint(n, k, f, e.clone())
        })
    }

    fn apply_resource_plan(
        &mut self,
        name: &str,
        file: studio_store::checkpoint::ExpectedFile,
        plan: Vec<crate::graphical::package::LibraryEdit>,
        expected: SourceRef,
    ) -> Result<studio_store::checkpoint::SaveReceipt, String> {
        self.write_package(name, file, &expected, |inner| {
            inner.apply_resource_plan(name, file, plan, expected.clone())
        })
    }
    fn resource_edit_preflight(
        &self,
        name: &str,
        file: studio_store::checkpoint::ExpectedFile,
        expected: SourceRef,
    ) -> Result<(), String> {
        with_authorized(
            self.owner.as_ref(),
            &self.scope,
            Some(&expected),
            AuthoringAction::SaveCheckpoint,
            self.limits,
            |_, _| {
                self.inner
                    .resource_edit_preflight(name, file, expected.clone())
            },
        )
    }
    fn resource_import_supported(&self) -> bool {
        self.inner.resource_import_supported()
    }
    fn package_inventory(
        &self,
        name: &str,
        expected: SourceRef,
    ) -> Result<crate::graphical::checkpoints::PackageInventory, String> {
        with_authorized(
            self.owner.as_ref(),
            &self.scope,
            Some(&expected),
            AuthoringAction::ReadSource,
            self.limits,
            |_, _| self.inner.package_inventory(name, expected.clone()),
        )
    }
    fn edit_package_resources(
        &mut self,
        name: &str,
        expected_file: studio_store::checkpoint::ExpectedFile,
        edits: Vec<studio_store::checkpoint::ResourceEdit>,
        expected: SourceRef,
    ) -> Result<studio_store::checkpoint::SaveReceipt, String> {
        self.write_package(name, expected_file, &expected, |inner| {
            inner.edit_package_resources(name, expected_file, edits, expected.clone())
        })
    }
    fn import_package_resource(
        &mut self,
        name: &str,
        expected_file: studio_store::checkpoint::ExpectedFile,
        request: crate::graphical::package::ImportRequest,
        expected: SourceRef,
    ) -> Result<studio_store::checkpoint::SaveReceipt, String> {
        self.write_package(name, expected_file, &expected, |inner| {
            inner.import_package_resource(name, expected_file, request, expected.clone())
        })
    }
    fn snapshot(&self) -> Result<EditorSnapshot, String> {
        self.read(|inner, generation| {
            let mut snapshot = inner.snapshot()?;
            snapshot.authorisation_stamp = Some(AuthorisationStamp {
                owner: self.owner.clone(),
                generation,
            });
            Ok(snapshot)
        })
    }
    fn check(&self, source: &str) -> Result<String, String> {
        self.read(|inner, _| inner.check(source))
    }
    fn source_cells(&self, expected: SourceRef) -> Result<Vec<String>, String> {
        with_authorized(
            self.owner.as_ref(),
            &self.scope,
            Some(&expected),
            AuthoringAction::ReadSource,
            self.limits,
            |_, _| self.inner.source_cells(expected.clone()),
        )
    }
    fn checkpoint_supported(&self) -> bool {
        self.inner.checkpoint_supported()
    }
    fn submit(&mut self, source: &str, expected: SourceRef) -> Result<EditorOutcome, String> {
        self.write(&expected, AuthoringAction::ExecuteSource, |inner| {
            inner.submit(source, expected.clone())
        })
    }
    fn material_edit(
        &mut self,
        source: &str,
        expected: SourceRef,
    ) -> Result<EditorOutcome, String> {
        self.write(&expected, AuthoringAction::MaterialEdit, |inner| {
            inner.material_edit(source, expected.clone())
        })
    }
    fn undo(&mut self, expected: SourceRef) -> Result<EditorSnapshot, String> {
        self.write(&expected, AuthoringAction::Undo, |inner| {
            inner.undo(expected.clone())
        })
    }
    fn redo(&mut self, expected: SourceRef) -> Result<EditorSnapshot, String> {
        self.write(&expected, AuthoringAction::Redo, |inner| {
            inner.redo(expected.clone())
        })
    }
    fn rebuild_execution(&mut self, expected: SourceRef) -> Result<EditorOutcome, String> {
        self.write(&expected, AuthoringAction::Rebuild, |inner| {
            inner.rebuild_execution(expected.clone())
        })
    }
    fn save_checkpoint(
        &mut self,
        name: &str,
        expected_file: studio_store::checkpoint::ExpectedFile,
        expected: SourceRef,
    ) -> Result<studio_store::checkpoint::SaveReceipt, String> {
        self.write_package(name, expected_file, &expected, |inner| {
            inner.save_checkpoint(name, expected_file, expected.clone())
        })
    }
}
