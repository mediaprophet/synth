//! Ordered texture effects use the existing authorised detail/source operation.
use super::*;
use crate::graphical::{
    authoring::AuthoringEdit,
    surfaces::{
        binding::digest_hex,
        detail::{
            required_paint_version, validate_effects, DetailLimits, DetailRecipe, PaintLayer,
            PaintRecipe, TextureEffectStep,
        },
    },
};

pub(super) fn tools() -> Vec<AuthoringTool> {
    let effect = json!({"oneOf":[
        {"type":"object","properties":{"kind":{"const":"grayscale"}},"required":["kind"],"additionalProperties":false},
        {"type":"object","properties":{"kind":{"const":"invert"}},"required":["kind"],"additionalProperties":false},
        {"type":"object","properties":{"kind":{"const":"sepia"},"intensity":{"type":"number","minimum":0,"maximum":1}},"required":["kind","intensity"],"additionalProperties":false},
        {"type":"object","properties":{"kind":{"const":"gaussian_blur"},"sigma":{"type":"number","minimum":0,"maximum":16}},"required":["kind","sigma"],"additionalProperties":false},
        {"type":"object","properties":{"kind":{"const":"sharpen"},"sigma":{"type":"number","minimum":0,"maximum":16},"amount":{"type":"number","minimum":0,"maximum":4}},"required":["kind","sigma","amount"],"additionalProperties":false},
        {"type":"object","properties":{"kind":{"const":"hue_rotate"},"degrees":{"type":"number","minimum":-360,"maximum":360}},"required":["kind","degrees"],"additionalProperties":false},
        {"type":"object","properties":{"kind":{"const":"emboss"},"azimuth_degrees":{"type":"number","minimum":-360,"maximum":360},"strength":{"type":"number","minimum":0,"maximum":1}},"required":["kind","azimuth_degrees","strength"],"additionalProperties":false}
    ]});
    let step = json!({"type":"object","properties":{"enabled":{"type":"boolean"},"effect":effect},"required":["enabled","effect"],"additionalProperties":false});
    let byte = json!({"type":"integer","minimum":0,"maximum":255});
    let mask = json!({"oneOf":[{"type":"null"},
        {"type":"object","properties":{"kind":{"const":"rectangle"},"x":{"type":"integer","minimum":0,"maximum":1024},"y":{"type":"integer","minimum":0,"maximum":1024},"width":{"type":"integer","minimum":1,"maximum":1024},"height":{"type":"integer","minimum":1,"maximum":1024},"coverage":byte},"required":["kind","x","y","width","height","coverage"],"additionalProperties":false},
        {"type":"object","properties":{"kind":{"const":"resource"},"sha256":{"type":"string","pattern":"^(?!0{64}$)[0-9a-f]{64}$"},"channel":{"enum":["red","alpha"]}},"required":["kind","sha256","channel"],"additionalProperties":false}]});
    let stroke = json!({"type":"object","properties":{"size":{"type":"number","minimum":1,"maximum":2048},"colour":{"type":"array","items":byte,"minItems":4,"maxItems":4},"opacity":{"type":"number","minimum":0,"maximum":1},"hardness":{"type":"number","minimum":0,"maximum":1},"points":{"type":"array","minItems":1,"maxItems":4096,"items":{"type":"array","minItems":2,"maxItems":2,"items":{"type":"number","minimum":0,"exclusiveMaximum":1024}}}},"required":["size","colour","opacity","hardness","points"],"additionalProperties":false});
    let layer = json!({"type":"object","properties":{"name":{"type":"string","minLength":1,"maxLength":80},"visible":{"type":"boolean"},"opacity":{"type":"number","minimum":0,"maximum":1},"strokes":{"type":"array","maxItems":128,"items":stroke},"blend_mode":{"enum":["normal","multiply","screen","overlay","soft_light","hard_light","color_dodge","color_burn","difference","exclusion"]},"offset":{"type":"array","minItems":2,"maxItems":2,"items":{"type":"integer","minimum":-2147483648_i64,"maximum":2147483647_i64}},"mask":mask},"required":["name","visible","opacity","strokes"],"additionalProperties":false});
    vec![
        tool("poet_texture_layers_edit", "Replace editable paint layers including blend modes, signed pixel offsets and rectangle/HMC masks through the same native authority and source-history owner",
            json!({"type":"object","properties":{"context":{"type":"string","maxLength":128},"material_id":{"type":"integer","minimum":1},"layers":{"type":"array","maxItems":8,"items":layer}},"required":["context","material_id","layers"],"additionalProperties":false})),
        tool("poet_texture_effects_inspect", "Inspect the selected accepted detail recipe and actual resolved base-colour resource digest; no human or scientific acceptance inferred",
            json!({"type":"object","properties":{"context":{"type":"string","maxLength":128},"part_id":{"type":"string","minLength":1,"maxLength":128}},"required":["context","part_id"],"additionalProperties":false})),
        tool("poet_texture_effects_edit", "Replace only the ordered post-composite effects of an existing imported Paint recipe; preserve source layers and immutable base through the native authority/history operation",
            json!({"type":"object","properties":{"context":{"type":"string","maxLength":128},"material_id":{"type":"integer","minimum":1},"effects":{"type":"array","items":step,"maxItems":8}},"required":["context","material_id","effects"],"additionalProperties":false})),
    ]
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Inspect {
    context: String,
    part_id: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct EditEffects {
    context: String,
    material_id: u64,
    effects: Vec<TextureEffectStep>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct EditLayers {
    context: String,
    material_id: u64,
    layers: Vec<PaintLayer>,
}
enum Patch {
    Effects(Vec<TextureEffectStep>),
    Layers(Vec<PaintLayer>),
}

pub(super) fn call(
    backend: &mut NativeAuthoringBackend,
    name: &str,
    arguments: Value,
) -> Result<Value, ToolError> {
    if name == "poet_texture_effects_inspect" {
        let request: Inspect = args(arguments)?;
        if request.context.len() > 128 || request.part_id.is_empty() || request.part_id.len() > 128
        {
            return Err(failure(
                "invalid_arguments",
                "Texture inspection arguments exceed native bounds",
            ));
        }
        let expected = backend.expected(&request.context)?;
        let view = backend
            .host()?
            .authoring_view(expected.source.clone())
            .map_err(editor_error)?;
        let part = view
            .parts
            .into_iter()
            .find(|p| p.id == request.part_id)
            .ok_or_else(|| failure("native_operation_failed", "Selected part is unavailable"))?;
        let inspection = backend
            .host()?
            .inspect_surface(&request.part_id, expected.source.clone())
            .map_err(editor_error)?;
        receipt_authority(&expected.authorisation_stamp)?;
        expected.source.check_current().map_err(editor_error)?;
        let base = match &part.detail {
            Some(DetailRecipe::Paint(p)) => Some(p.base_sha256.clone()),
            _ => None,
        };
        return Ok(json!({
            "artifact_id":expected.artifact_id,"source_revision":expected.source_revision,
            "session_revision":expected.session_revision,"part_id":part.id,"material_id":part.material_id,
            "detail":part.detail,"base_sha256":base,
            "output_sha256":inspection.texture_sha256.map(|d|digest_hex(&d)),
            "working_space":"encoded sRGB RGBA8; opaque imported square base",
            "human_review":"unassessed"
        }));
    }
    let (context, material_id, patch) = if name == "poet_texture_layers_edit" {
        let request: EditLayers = args(arguments)?;
        if request.layers.len() > 8 {
            return Err(failure("invalid_arguments", "At most eight paint layers"));
        }
        // Shape/range-only preflight at the maximum supported extent. Actual
        // target extent/work/byte/resource admission follows current authority.
        let probe = DetailRecipe::Paint(PaintRecipe {
            version: 3,
            side: 1024,
            base_sha256: "11".repeat(32),
            layers: request.layers,
            effects: vec![],
        });
        probe
            .validate(DetailLimits {
                working_bytes: usize::MAX,
                output_bytes: usize::MAX,
                pixel_visits: usize::MAX,
                ..DetailLimits::default()
            })
            .map_err(|_| {
                failure(
                    "invalid_arguments",
                    "Paint layers violate native shape/range bounds",
                )
            })?;
        let DetailRecipe::Paint(p) = probe else {
            unreachable!()
        };
        (
            request.context,
            request.material_id,
            Patch::Layers(p.layers),
        )
    } else {
        let request: EditEffects = args(arguments)?;
        validate_effects(&request.effects).map_err(|_| {
            failure(
                "invalid_arguments",
                "Texture effects violate the native profile",
            )
        })?;
        (
            request.context,
            request.material_id,
            Patch::Effects(request.effects),
        )
    };
    if context.len() > 128 || material_id == 0 {
        return Err(failure(
            "invalid_arguments",
            "Texture edit arguments exceed native bounds",
        ));
    }
    // A valid-shaped mutation attempt consumes its connection-local context even
    // if a later target, resource, authority or source admission refuses it.
    let expected_result = backend.expected(&context);
    backend.context = None;
    let expected = expected_result?;
    let authority = expected.authorisation_stamp.clone();
    let view = backend
        .host()?
        .authoring_view(expected.source.clone())
        .map_err(editor_error)?;
    let mut detail = None;
    for part in view
        .parts
        .into_iter()
        .filter(|p| p.material_id == material_id)
    {
        let Some(DetailRecipe::Paint(paint)) = part.detail else {
            return Err(failure(
                "native_operation_failed",
                "All shared target parts require the same existing imported Paint recipe",
            ));
        };
        if detail.as_ref().is_some_and(|old| old != &paint) {
            return Err(failure(
                "native_operation_failed",
                "Shared material recipes conflict; accepted source preserved",
            ));
        }
        detail = Some(paint);
    }
    let mut paint = detail.ok_or_else(|| {
        failure(
            "native_operation_failed",
            "Imported paint target is unavailable",
        )
    })?;
    match patch {
        Patch::Effects(effects) => paint.effects = effects,
        Patch::Layers(layers) => paint.layers = layers,
    }
    paint.version = paint.version.max(required_paint_version(&paint));
    let detail = DetailRecipe::Paint(paint);
    detail
        .validate(DetailLimits::default())
        .map_err(editor_error)?;
    backend
        .host()?
        .authoring_edit(
            AuthoringEdit::TextureDetail {
                material_id,
                detail: Some(detail),
            },
            expected.source,
        )
        .map_err(editor_error)
        .and_then(|result| outcome(result, &authority))
}
