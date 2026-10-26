//! Resource intents pass the same current native project owner as UI commands.
use super::*;
use crate::graphical::package::{
    hex_digest, resource_plan, resource_plan_schema, ImportRequest, ResourceIntent,
};
use qualia_core_db::poet_host::invoke::studio_store::checkpoint::ResourceEdit;

pub(super) fn tools(import: bool) -> Vec<AuthoringTool> {
    let common = json!({"context":{"type":"string","maxLength":128},"name":{"type":"string","maxLength":128},
        "expected_sha256":{"type":"string","pattern":"^[0-9a-fA-F]{64}$"},
        "key":{"type":"string","maxLength":4096},"expected_resource_sha256":{"type":"string","pattern":"^[0-9a-fA-F]{64}$"}});
    let mut remove = common.clone();
    let mut result = vec![
        tool("poet_hmc_edit_resources", "Apply one atomic plan of up to 64 guarded resource additions, replacements or removals; imports use only the native selected library",
            json!({"type":"object","properties":{"context":{"type":"string","maxLength":128},"name":{"type":"string","maxLength":128},"expected_sha256":{"type":"string","pattern":"^[0-9a-fA-F]{64}$"},"edits":resource_plan_schema()},"required":["context","name","expected_sha256","edits"],"additionalProperties":false})),
        tool(
            "poet_hmc_inventory",
            "Inspect authorised HMC resource keys, kinds, sizes and content/metadata digests",
            json!({"type":"object","properties":{"context":{"type":"string","maxLength":128},"name":{"type":"string","maxLength":128}},"required":["context","name"],"additionalProperties":false}),
        ),
        tool(
            "poet_hmc_remove_resource",
            "Explicitly remove one resource with both package and resource SHA256 guards",
            json!({"type":"object","properties":remove,"required":["context","name","expected_sha256","key","expected_resource_sha256"],"additionalProperties":false}),
        ),
    ];
    if import {
        remove["filename"] = json!({"type":"string","maxLength":240});
        remove["kind"] = json!({"type":"string","maxLength":4096});
        result.push(tool("poet_hmc_import_resource","Import one file from the native host-selected library; replace bytes while retaining kind and metadata",
            json!({"type":"object","properties":remove,"required":["context","name","expected_sha256","key","filename","kind"],"additionalProperties":false})));
    }
    result
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Inspect {
    context: String,
    name: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Plan {
    context: String,
    name: String,
    expected_sha256: String,
    edits: Vec<ResourceIntent>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Remove {
    context: String,
    name: String,
    expected_sha256: String,
    key: String,
    expected_resource_sha256: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Import {
    context: String,
    name: String,
    expected_sha256: String,
    key: String,
    filename: String,
    kind: String,
    expected_resource_sha256: Option<String>,
}
fn bounded(context: &str, name: &str, key: &str) -> Result<(), ToolError> {
    if context.len() > 128 || name.len() > 128 || key.len() > 4096 {
        return Err(failure(
            "limit",
            "Resource intent exceeds native message bounds",
        ));
    }
    Ok(())
}
pub(super) fn call(
    backend: &mut NativeAuthoringBackend,
    name: &str,
    arguments: Value,
) -> Result<Value, ToolError> {
    if name == "poet_hmc_edit_resources" {
        let request: Plan = args(arguments)?;
        bounded(&request.context, &request.name, "")?;
        let plan = resource_plan(request.edits)
            .map_err(|_| failure("invalid_arguments", "Invalid bounded resource plan"))?;
        let file = ExpectedFile::Sha256(digest(&request.expected_sha256)?);
        let expected = backend.expected(&request.context)?;
        backend.context = None;
        let authority = expected.authorisation_stamp.clone();
        let receipt = backend
            .host()?
            .apply_resource_plan(&request.name, file, plan, expected.source)
            .map_err(editor_error)?;
        receipt_authority(&authority)?;
        return Ok(
            json!({"name":request.name,"sha256":hex_digest(&receipt.digest),"bytes":receipt.bytes,"durability_warning":receipt.durability_warning,"read_required":true,"human_review":"unassessed"}),
        );
    }
    if name == "poet_hmc_inventory" {
        let request: Inspect = args(arguments)?;
        bounded(&request.context, &request.name, "")?;
        let expected = backend.expected(&request.context)?;
        let inventory = backend
            .host()?
            .package_inventory(&request.name, expected.source.clone())
            .map_err(editor_error)?;
        receipt_authority(&expected.authorisation_stamp)?;
        expected.source.check_current().map_err(editor_error)?;
        return Ok(
            json!({"name":request.name,"sha256":hex_digest(&inventory.digest),"resources":inventory.resources.into_iter().map(|r|json!({
            "key":r.key,"kind":r.kind,"bytes":r.bytes,"sha256":hex_digest(&r.digest),"metadata_bytes":r.metadata_bytes,
            "metadata_sha256":r.metadata_digest.map(|d|hex_digest(&d))})).collect::<Vec<_>>(),"human_review":"unassessed"}),
        );
    }
    let (context, name, file, edit, import) = if name == "poet_hmc_remove_resource" {
        let request: Remove = args(arguments)?;
        bounded(&request.context, &request.name, &request.key)?;
        (
            request.context,
            request.name,
            ExpectedFile::Sha256(digest(&request.expected_sha256)?),
            Some(ResourceEdit::Remove {
                key: request.key,
                expected: digest(&request.expected_resource_sha256)?,
            }),
            None,
        )
    } else {
        let request: Import = args(arguments)?;
        bounded(&request.context, &request.name, &request.key)?;
        if request.filename.len() > 240 || request.kind.len() > 4096 {
            return Err(failure("limit", "Resource import exceeds native bounds"));
        }
        (
            request.context,
            request.name,
            ExpectedFile::Sha256(digest(&request.expected_sha256)?),
            None,
            Some(ImportRequest {
                filename: request.filename,
                key: request.key,
                kind: request.kind,
                expected_resource: request
                    .expected_resource_sha256
                    .as_deref()
                    .map(digest)
                    .transpose()?,
            }),
        )
    };
    let expected = backend.expected(&context)?;
    backend.context = None;
    let authority = expected.authorisation_stamp.clone();
    let receipt = if let Some(edit) = edit {
        backend
            .host()?
            .edit_package_resources(&name, file, vec![edit], expected.source)
    } else {
        backend.host()?.import_package_resource(
            &name,
            file,
            import.ok_or_else(|| failure("invalid_arguments", "Missing resource intent"))?,
            expected.source,
        )
    }
    .map_err(editor_error)?;
    receipt_authority(&authority)?;
    Ok(
        json!({"name":name,"sha256":hex_digest(&receipt.digest),"bytes":receipt.bytes,"durability_warning":receipt.durability_warning,"read_required":true,"human_review":"unassessed"}),
    )
}
