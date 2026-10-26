//! Same source-owner operation for admitted native agent and operator edits.
use super::*;
use crate::graphical::authoring::{botanical::BotanicalEdit, AuthoringEdit};

pub(super) fn tools() -> Vec<AuthoringTool> {
    let key = json!({"type":"string","minLength":1,"maxLength":128});
    let positive = json!({"type":"number","exclusiveMinimum":0,"maximum":1000000});
    let point = json!({"type":"array","items":{"type":"number","minimum":-1000000,"maximum":1000000},"minItems":3,"maxItems":3});
    let organ = json!({"type":"object","properties":{"key":key,"parent":{"type":["string","null"]},"position_m":point},"required":["key","parent","position_m"],"additionalProperties":false});
    let recipe = json!({"type":"object","properties":{"version":{"const":1},"part_id":key,
        "organs":{"type":"array","items":organ,"minItems":2,"maxItems":256},
        "attractors_m":{"type":"array","items":point,"minItems":1,"maxItems":256},
        "node_length_m":positive,"kill_radius_m":positive,"perception_radius_m":positive,
        "cone_angle_rad":{"type":"number","exclusiveMinimum":0,"maximum":std::f64::consts::TAU},
        "max_iterations":{"type":"integer","minimum":0,"maximum":16},"tip_radius_m":positive,
        "pipe_exponent":{"type":"number","minimum":1,"maximum":8},"ring_slices":{"type":"integer","minimum":3,"maximum":64}},
        "required":["version","part_id","organs","attractors_m","node_length_m","kill_radius_m","perception_radius_m","cone_angle_rad","max_iterations","tip_radius_m","pipe_exponent","ring_slices"],"additionalProperties":false});
    let edit = json!({"oneOf":[
        {"type":"object","properties":{"operation":{"const":"set_recipe"},"recipe":recipe},"required":["operation","recipe"],"additionalProperties":false},
        {"type":"object","properties":{"operation":{"const":"set_tip_radius"},"part_id":key,"tip_radius_m":positive},"required":["operation","part_id","tip_radius_m"],"additionalProperties":false},
        {"type":"object","properties":{"operation":{"const":"set_organ_position"},"part_id":key,"organ_key":key,"position_m":point},"required":["operation","part_id","organ_key","position_m"],"additionalProperties":false}
    ]});
    vec![tool("poet_botanical_inspect", "Inspect adopted organ keys and source-linked graph; not species acceptance", schema(false,true)),
        tool("poet_botanical_edit", "Adopt a bounded graph, set tip radius or move an adopted organ through existing source history and authority; unsupported remapping is refused",
            json!({"type":"object","properties":{"context":{"type":"string","maxLength":128},"edit":edit},"required":["context","edit"],"additionalProperties":false}))]
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Request {
    context: String,
    edit: BotanicalEdit,
}
pub(super) fn call(
    backend: &mut NativeAuthoringBackend,
    name: &str,
    arguments: Value,
) -> Result<Value, ToolError> {
    if name == "poet_botanical_inspect" {
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
        return Ok(
            json!({"source_revision":expected.source_revision,"botanical":view.botanical,"human_review":"unassessed"}),
        );
    }
    let request: Request = args(arguments)?;
    if request.context.len() > 128 {
        return Err(failure("limit", "Context exceeds native bound"));
    }
    let expected = backend.expected(&request.context)?;
    backend.context = None;
    let authority = expected.authorisation_stamp.clone();
    backend
        .host()?
        .authoring_edit(
            AuthoringEdit::Botanical { edit: request.edit },
            expected.source,
        )
        .map_err(editor_error)
        .and_then(|result| outcome(result, &authority))
}
