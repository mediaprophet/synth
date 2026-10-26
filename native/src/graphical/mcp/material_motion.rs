//! Material keys use the same admitted authoring operation as native controls.
use super::*;
use crate::graphical::authoring::{material_motion::MaterialMotionEdit, AuthoringEdit};

pub(super) fn tools() -> Vec<AuthoringTool> {
    let factor = json!({"type":"number","minimum":0,"maximum":1});
    let milliseconds = json!({"type":"integer","minimum":0,"maximum":86400000});
    let identifier = json!({"type":"integer","minimum":1});
    let key = json!({"type":"object","properties":{
        "time_ms":milliseconds,"base_color":{"type":"array","items":factor,"minItems":4,"maxItems":4},
        "roughness":factor,"metallic":factor,"interpolation":{"enum":["hold","linear","cubic_in_out"]}},
        "required":["time_ms","base_color","roughness","metallic","interpolation"],"additionalProperties":false});
    let draft = json!({"type":"object","properties":{
        "version":{"const":1},"material_id":identifier,"keys":{"type":"array","items":key,"minItems":1,"maxItems":64}},
        "required":["version","material_id","keys"],"additionalProperties":false});
    let edit = json!({"oneOf":[
        {"type":"object","properties":{"operation":{"const":"set"},"draft":draft,"sample_ms":milliseconds},"required":["operation","draft","sample_ms"],"additionalProperties":false},
        {"type":"object","properties":{"operation":{"const":"sample"},"material_id":identifier,"sample_ms":milliseconds},"required":["operation","material_id","sample_ms"],"additionalProperties":false},
        {"type":"object","properties":{"operation":{"const":"remove"},"material_id":identifier},"required":["operation","material_id"],"additionalProperties":false}
    ]});
    vec![
        tool("poet_material_motion_inspect", "Inspect editable material keys and actual sampled factors in the current native source; no playback or scientific-model claim", schema(false,true)),
        tool("poet_material_motion_edit", "Set, sample/apply or remove one material track through the native revision/authority and source-history owner; explicit milliseconds, linear factors, no automatic playback",
            json!({"type":"object","properties":{"context":{"type":"string","maxLength":128},"edit":edit},"required":["context","edit"],"additionalProperties":false})),
    ]
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Request {
    context: String,
    edit: MaterialMotionEdit,
}

pub(super) fn call(
    backend: &mut NativeAuthoringBackend,
    name: &str,
    arguments: Value,
) -> Result<Value, ToolError> {
    if name == "poet_material_motion_inspect" {
        let request: Context = args(arguments)?;
        if request.context.len() > 128 {
            return Err(failure("limit", "Context exceeds native bound"));
        }
        let expected = backend.expected(&request.context)?;
        let view = backend
            .host()?
            .authoring_view(expected.source.clone())
            .map_err(editor_error)?;
        receipt_authority(&expected.authorisation_stamp)?;
        expected.source.check_current().map_err(editor_error)?;
        return Ok(json!({"artifact_id":expected.artifact_id,
            "source_revision":expected.source_revision,"session_revision":expected.session_revision,
            "material_motion":view.material_motion,
            "parts":view.parts.into_iter().map(|p|json!({"id":p.id,"material_id":p.material_id,"base_color":p.base_color,"roughness":p.roughness,"metallic":p.metallic})).collect::<Vec<_>>(),
            "time_unit":"milliseconds","colour_space":"linear","human_review":"unassessed"}));
    }
    let request: Request = args(arguments)?;
    if request.context.len() > 128 {
        return Err(failure("limit", "Context exceeds native bound"));
    }
    request.edit.validate().map_err(|_| {
        failure(
            "invalid_arguments",
            "Material keys violate the supported native profile",
        )
    })?;
    let expected = backend.expected(&request.context)?;
    backend.context = None;
    let authority = expected.authorisation_stamp.clone();
    backend
        .host()?
        .authoring_edit(
            AuthoringEdit::MaterialMotion { edit: request.edit },
            expected.source,
        )
        .map_err(editor_error)
        .and_then(|result| outcome(result, &authority))
}
