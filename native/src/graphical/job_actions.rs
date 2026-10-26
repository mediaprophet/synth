use super::*;
pub(super) fn perform(
    host: &mut dyn EditorHost,
    action: Action,
    inventory: &mut Option<crate::graphical::checkpoints::PackageInventory>,
    workflow: &mut Option<crate::graphical::cooperative_workflow::WorkflowView>,
    authoring: &mut Option<crate::graphical::authoring::AuthoringView>,
    inspection: &mut Option<crate::graphical::checkpoints::ResourceInspection>,
    package_workflow: &mut Option<crate::graphical::package_workflow::PackageWorkflowView>,
    typed_10d: &mut Option<crate::graphical::checkpoints::typed_10d::Typed10dInspection>,
    surface: &mut Option<crate::graphical::surfaces::inspection::SurfaceInspection>,
    viewport: &mut Option<crate::graphical::viewport_contract::ViewportState>,
    cancelled: &std::sync::atomic::AtomicBool,
) -> Result<(String, Option<BoundImage>), String> {
    let image_uri = None;
    let message = match action {
        Action::Viewport {request,expected} => {
            let output = host.viewport_action(request, expected, cancelled)?;
            let snapshot = host.snapshot()?;
            *viewport = Some(output.state);
            return outcome_image(crate::graphical::controller::EditorOutcome {
                snapshot, frame: Some(output.frame), message: "Native viewport updated; source unchanged.".into()
            });
        }
        Action::InspectSurface { id, expected } => {
            if id.is_empty()
                || id.len() > 128
                || cancelled.load(std::sync::atomic::Ordering::Acquire)
            {
                return Err("Surface inspection refused before dispatch".into());
            }
            *surface = Some(host.inspect_surface(&id, expected)?);
            "Current selected source UV and texture pixels inspected; no asset acceptance claim."
                .into()
        }

        Action::BindSurfacePackage { request, expected } => {
            if cancelled.load(std::sync::atomic::Ordering::Acquire) {
                return Err("Surface binding cancelled before admission".into());
            }
            host.bind_surface_package(request, expected)?;
            "Verified immutable HMC texture snapshot bound; old execution/frame context retired. Rebuild held source explicitly if required.".into()
        }

        Action::InspectTyped10d { request, expected } => {
            *typed_10d = Some(host.inspect_typed_10d(
                request,
                expected,
                crate::graphical::checkpoints::typed_10d::Typed10dLimits::default(),
            )?);
            "Inspected selected .10d material/frame/field snapshot; structural inspection is separate from model acceptance.".into()
        }
        Action::EditTyped10d { request, expected } => {
            let name = request.resource.filename.clone();
            let receipt = host.edit_typed_10d(
                request,
                expected.clone(),
                crate::graphical::checkpoints::typed_10d::Typed10dLimits::default(),
                cancelled,
            )?;
            *inventory = host.package_inventory(&name, expected).ok();
            format!("Saved visual .10d material derivative in {name}; SHA256 {}. Source association is not proof the source regenerates the edited resource. Refresh package review.",crate::graphical::package::hex_digest(&receipt.digest))+&crate::graphical::package::durability_note(&receipt)
        }

        Action::PackageWorkflow { action, expected } => {
            if cancelled.load(std::sync::atomic::Ordering::Acquire) {
                return Err("Worker closed before package workflow admission".into());
            }
            *package_workflow = Some(host.package_workflow(action, expected)?);
            "Saved-package workflow inspected/recorded for the exact digest; refresh after any package or authority change.".into()
        }

        Action::EditGeoRaster { request, expected } => {
            let name = request.resource.filename.clone();
            let receipt = host.edit_georaster(
                request,
                expected.clone(),
                crate::graphical::checkpoints::ui_georaster_edit_limits(),
                cancelled,
            )?;
            *inventory = host.package_inventory(&name, expected).ok();
            format!("Persisted developmental v3 raster edited in {name}; package SHA256 {}. Retained source/resources preserved; refresh inventory. Package review and scientific acceptance remain separate.", crate::graphical::package::hex_digest(&receipt.digest))
                + &crate::graphical::package::durability_note(&receipt)
        }
        Action::CreateGeoRaster { request, expected } => {
            let name = request.filename.clone();
            let receipt = host.create_georaster(
                request,
                expected.clone(),
                crate::graphical::checkpoints::ui_georaster_edit_limits(),
                cancelled,
            )?;
            *inventory = host.package_inventory(&name, expected).ok();
            format!("Explicit source compiled into a new developmental v3 raster resource in {name}; package SHA256 {}. Retained source/resources preserved; refresh inventory. Package review and scientific acceptance remain separate.", crate::graphical::package::hex_digest(&receipt.digest))
                + &crate::graphical::package::durability_note(&receipt)
        }

        Action::InspectResource { request, expected } => {
            use crate::graphical::checkpoints::{ui_inspection_limits, UI_INSPECTION_CHUNKS};
            let mut scratch = [qualia_core_db::q42::m64_matter::M64PayloadInterval::default();
                UI_INSPECTION_CHUNKS];
            *inspection = Some(host.inspect_package_resource(
                request,
                expected,
                ui_inspection_limits(),
                &mut scratch,
            )?);
            "Selected M64 package snapshot inspected. Structural/CRC scope only; scientific and package acceptance remain unassessed.".into()
        }
        Action::AuthoringInspect(expected) => {
            *authoring = Some(host.authoring_view(expected)?);
            "Current accepted assembly inspected".into()
        }
        Action::AuthoringEdit { edit, expected } => {
            let result = host.authoring_edit(edit, expected)?;
            *authoring = host.authoring_view(result.snapshot.source.clone()).ok();
            return outcome_image(result);
        }
        Action::CompileAssembly {
            name,
            key,
            file,
            expected,
        } => {
            let receipt = host.compile_checkpoint(&name, &key, file, expected.clone())?;
            *inventory = host.package_inventory(&name, expected).ok();
            format!("Accepted assembly compiled into {name}; SHA256 {}. Complete model/package acceptance remains separate.",crate::graphical::package::hex_digest(&receipt.digest))+&crate::graphical::package::durability_note(&receipt)
        }
        Action::Workflow { action, .. } => {
            *workflow = Some(host.workflow(action)?);
            "Current source contribution/review evidence updated. Package and master acceptance are separate.".into()
        }
        Action::SaveInspected {
            name,
            file_digest,
            expected,
        } => {
            let receipt = host.save_checkpoint(
                &name,
                qualia_core_db::poet_host::invoke::studio_store::checkpoint::ExpectedFile::Sha256(
                    file_digest,
                ),
                expected.clone(),
            )?;
            *inventory = host.package_inventory(&name, expected).ok();
            format!("Accepted source and complete retained resource inventory saved in {name}; SHA256 {}",crate::graphical::package::hex_digest(&receipt.digest))+&crate::graphical::package::durability_note(&receipt)
        }
        Action::ResourcePlan {
            name,
            file_digest,
            plan,
            expected,
        } => {
            let receipt = host.apply_resource_plan(
                &name,
                qualia_core_db::poet_host::invoke::studio_store::checkpoint::ExpectedFile::Sha256(
                    file_digest,
                ),
                plan,
                expected.clone(),
            )?;
            *inventory = host.package_inventory(&name, expected).ok();
            format!(
                "Atomic resource plan saved in {name}; package SHA256 {}",
                crate::graphical::package::hex_digest(&receipt.digest)
            ) + &crate::graphical::package::durability_note(&receipt)
        }
        Action::Inventory { name, expected } => {
            let result = host.package_inventory(&name, expected)?;
            let message = format!(
                "{} preserved resources in {name}; package SHA256 {}",
                result.resources.len(),
                crate::graphical::package::hex_digest(&result.digest)
            );
            *inventory = Some(result);
            message
        }
        Action::ImportResource {
            name,
            file_digest,
            request,
            expected,
        } => {
            let receipt = host.import_package_resource(
                &name,
                qualia_core_db::poet_host::invoke::studio_store::checkpoint::ExpectedFile::Sha256(
                    file_digest,
                ),
                request,
                expected.clone(),
            )?;
            *inventory = host.package_inventory(&name, expected).ok();
            format!("Resource contents saved in {name}; package SHA256 {}. Unedited entries retained; refresh inventory if unavailable.",crate::graphical::package::hex_digest(&receipt.digest))+&crate::graphical::package::durability_note(&receipt)
        }
        Action::RemoveResource {
            name,
            file_digest,
            key,
            resource_digest,
            expected,
        } => {
            let receipt=host.edit_package_resources(&name,
                qualia_core_db::poet_host::invoke::studio_store::checkpoint::ExpectedFile::Sha256(file_digest),
                vec![qualia_core_db::poet_host::invoke::studio_store::checkpoint::ResourceEdit::Remove{key,expected:resource_digest}],expected.clone())?;
            *inventory = host.package_inventory(&name, expected).ok();
            format!(
                "Explicit resource removal saved in {name}; package SHA256 {}",
                crate::graphical::package::hex_digest(&receipt.digest)
            ) + &crate::graphical::package::durability_note(&receipt)
        }
        Action::Check(source) => host.check(&source)?,
        Action::Submit { source, expected } => {
            let outcome = host.submit(&source, expected)?;
            return outcome_image(outcome);
        }
        Action::Rebuild(expected) => {
            if !host.snapshot()?.rebuild_supported {
                return Err("Execution rebuild is unsupported by this host profile".into());
            }
            return outcome_image(host.rebuild_execution(expected)?);
        }
        Action::Undo(expected) => {
            host.undo(expected)?;
            "Editable source restored; session reconciliation required".into()
        }
        Action::Redo(expected) => {
            host.redo(expected)?;
            "Editable source reapplied; session reconciliation required".into()
        }
        Action::Save { name, expected } => {
            let receipt = host.save_checkpoint(
                &name,
                qualia_core_db::poet_host::invoke::studio_store::checkpoint::ExpectedFile::Absent,
                expected,
            )?;
            let digest = receipt
                .digest
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect::<String>();
            format!("Accepted source/history saved as {name} ({} bytes, SHA256 {digest}); draft and human acceptance are separate",receipt.bytes)+&crate::graphical::package::durability_note(&receipt)
        }
        Action::Refresh => "Current identified source read from canonical store".into(),
    };
    Ok((message, image_uri))
}

pub(super) fn outcome_image(
    outcome: crate::graphical::controller::EditorOutcome,
) -> Result<(String, Option<BoundImage>), String> {
    let mut image_uri = None;
    let mut message = outcome.message;
    if let Some(frame) = outcome.frame {
        if frame.source == outcome.snapshot.source
            && frame.session_revision == Some(outcome.snapshot.session_revision)
            && frame.execution_epoch.as_ref() == Some(&outcome.snapshot.execution_epoch)
            && outcome.snapshot.execution_epoch.is_current()
        {
            match native_image_uri(&frame) {
                Ok(uri) => {
                    image_uri = Some(BoundImage {
                        context: Context::from_snapshot(&outcome.snapshot),
                        uri,
                    })
                }
                Err(error) => message.push_str(&format!("; frame withheld: {error}")),
            }
        } else {
            message.push_str("; frame source/session mismatch; withheld");
        }
    }
    Ok((message, image_uri))
}
