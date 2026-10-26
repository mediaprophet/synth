//! Adapt native controls to Poet's existing payload contract and the same worker.
use super::{
    authoring::AuthoringEdit,
    jobs::{self, Action, Context, HostHandle, Reply},
    poet_workspace::{
        core::{ActionType, TargetIdentifier, VibeScriptPayload},
        NativeIntentBus,
    },
};
use qualia_core_db::poet_host::invoke::studio_store::checkpoint::ExpectedFile;
use serde_json::json;
pub(crate) async fn run(
    host: HostHandle,
    action: Action,
    origin: Context,
) -> Result<Reply, String> {
    let expected = match &action {
        Action::Viewport { expected, .. }
        | Action::InspectSurface { expected, .. }
        | Action::BindSurfacePackage { expected, .. }
        | Action::InspectTyped10d { expected, .. }
        | Action::EditTyped10d { expected, .. }
        | Action::PackageWorkflow { expected, .. }
        | Action::EditGeoRaster { expected, .. }
        | Action::CreateGeoRaster { expected, .. }
        | Action::InspectResource { expected, .. }
        | Action::AuthoringInspect(expected)
        | Action::AuthoringEdit { expected, .. }
        | Action::CompileAssembly { expected, .. }
        | Action::Submit { expected, .. }
        | Action::Undo(expected)
        | Action::Redo(expected)
        | Action::Rebuild(expected)
        | Action::Save { expected, .. }
        | Action::Inventory { expected, .. }
        | Action::Workflow { expected, .. }
        | Action::SaveInspected { expected, .. }
        | Action::ResourcePlan { expected, .. }
        | Action::ImportResource { expected, .. }
        | Action::RemoveResource { expected, .. } => Some(expected),
        _ => None,
    };
    if expected.is_some_and(|e| e != &origin.source) {
        return Err(
            "Poet intent target differs from admitted source context; no retargeting or dispatch"
                .into(),
        );
    }
    // An explicitly labelled local diagnostic has no cooperative actor to stamp.
    // Authenticated native projects always use the original Poet intent contract.
    if origin.authorisation_stamp.is_none() {
        return jobs::run(host, action, origin).await;
    }
    let (kind, parameters) = match &action {
        Action::Viewport { request, .. } => (
            ActionType::Query,
            json!({"command":"viewport.navigate","request":request}),
        ),
        Action::InspectSurface { id, .. } => (
            ActionType::Query,
            json!({"command":"authoring.inspect_surface","id":id}),
        ),
        Action::BindSurfacePackage { request, .. } => (
            ActionType::Query,
            json!({"command":"authoring.bind_surface_package","request":request}),
        ),
        Action::AuthoringInspect(_) => (ActionType::Query, json!({"command":"authoring.inspect"})),
        Action::AuthoringEdit { edit, .. } => {
            let p = match edit {
                AuthoringEdit::Botanical { edit } => {
                    json!({"command":"authoring.botanical","edit":edit})
                }
                AuthoringEdit::MaterialMotion { edit } => {
                    json!({"command":"authoring.material_motion","edit":edit})
                }
                AuthoringEdit::EnableSourceLayers => {
                    json!({"command":"authoring.enable_source_layers"})
                }
                AuthoringEdit::UvAtlas { id, atlas } => {
                    json!({"command":"authoring.uv_atlas","id":id,"atlas":atlas})
                }
                AuthoringEdit::TextureDetail {
                    material_id,
                    detail,
                } => {
                    json!({"command":"authoring.texture_detail","material_id":material_id,"detail":detail})
                }
                AuthoringEdit::Transform {
                    id,
                    translation,
                    rotation,
                    scale,
                } => {
                    json!({"command":"authoring.transform","id":id,"translation":translation,"rotation":rotation,"scale":scale})
                }
                AuthoringEdit::Material {
                    material_id,
                    base_color,
                    roughness,
                    metallic,
                } => {
                    json!({"command":"authoring.material","material_id":material_id,"base_color":base_color,"roughness":roughness,"metallic":metallic})
                }
                AuthoringEdit::Surface {
                    material_id,
                    surface,
                } => {
                    json!({"command":"authoring.surface","material_id":material_id,"surface":surface})
                }
                AuthoringEdit::UvTransform { id, transform } => {
                    json!({"command":"authoring.uv_transform","id":id,"transform":crate::graphical::surfaces::wire::UvTransformWire::from(*transform)})
                }
                AuthoringEdit::UvProjection { id, projection } => {
                    json!({"command":"authoring.uv_projection","id":id,"projection":projection})
                }
                AuthoringEdit::Reparent { id, parent } => {
                    json!({"command":"authoring.reparent","id":id,"parent":parent})
                }
                AuthoringEdit::RemoveLeaf { id } => {
                    json!({"command":"authoring.remove_leaf","id":id})
                }
                AuthoringEdit::ReplaceGeometry { id, geometry } => {
                    json!({"command":"authoring.replace_geometry","id":id,"geometry":geometry})
                }
                AuthoringEdit::AddGeometry {
                    id,
                    semantic_id,
                    material_from,
                    geometry,
                } => {
                    json!({"command":"authoring.add_geometry","id":id,"semantic_id":semantic_id,"material_from":material_from,"geometry":geometry})
                }
                AuthoringEdit::UniqueMaterial { id, material_id } => {
                    json!({"command":"authoring.unique_material","id":id,"material_id":material_id})
                }
                AuthoringEdit::Parameter { key, value } => {
                    json!({"command":"authoring.parameter","key":key,"value":value})
                }
            };
            (ActionType::Invoke, p)
        }
        Action::CompileAssembly {
            name, key, file, ..
        } => (
            ActionType::Invoke,
            json!({"command":"authoring.compile","name":name,"key":key,"expected_sha256":match file{ExpectedFile::Absent=>None,ExpectedFile::Sha256(d)=>Some(*d)}}),
        ),
        Action::Check(source) => (
            ActionType::Query,
            json!({"command":"source.check","source":source}),
        ),
        Action::Submit { source, .. } => (
            ActionType::Invoke,
            json!({"command":"source.submit","source":source}),
        ),
        Action::Undo(_) => (ActionType::Invoke, json!({"command":"source.undo"})),
        Action::Redo(_) => (ActionType::Invoke, json!({"command":"source.redo"})),
        Action::Rebuild(_) => (ActionType::Invoke, json!({"command":"source.rebuild"})),
        Action::Refresh => (ActionType::Query, json!({"command":"source.refresh"})),
        Action::Save { name, .. } => (
            ActionType::Invoke,
            json!({"command":"package.save","name":name}),
        ),
        // Existing package/cooperation controls keep their qualified same-owner
        // adapters until their original Poet tool payloads are ported explicitly.
        _ => return jobs::run(host, action, origin).await,
    };
    let payload = VibeScriptPayload::new(
        kind,
        TargetIdentifier::component_id(&origin.source.artifact_id),
        parameters,
    );
    let result = NativeIntentBus::new(host, origin)?
        .dispatch_with_reply(payload)
        .await?;
    result
        .reply
        .ok_or_else(|| format!("Poet intent rejected: {:?}", result.receipt.status))
}
