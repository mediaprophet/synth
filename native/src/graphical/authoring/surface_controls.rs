//! Native full visual surface/UV controls submit revision-bound authoring intents.
use super::{AuthoringEdit, AuthoringView};
use crate::graphical::{
    preview::SourceRef,
    surfaces::{uv, SurfaceDraft, TextureBindingDraft, TextureRole},
};
use dioxus::prelude::*;
use qualia_core_db::container_10d::{MaterialRecord, OpacityMode, ShadingModel, TextureTransform};
use vibe::Value;
const FACTORS: [&str; 13] = [
    "Linear red",
    "Linear green",
    "Linear blue",
    "Alpha",
    "Emissive red",
    "Emissive green",
    "Emissive blue",
    "Metallic",
    "Roughness",
    "Specular",
    "Normal scale",
    "Occlusion strength",
    "Alpha cutoff",
];
const FLAGS: [&str; 6] = [
    "Double sided",
    "Casts shadow",
    "Receives shadow",
    "Pickable",
    "Multiply vertex colour",
    "Wind animated",
];
const MAP_FIELDS: [&str; 10] = [
    "SHA-256 (blank removes)",
    "Offset U",
    "Offset V",
    "Scale U",
    "Scale V",
    "Rotation radians",
    "Wrap S",
    "Wrap T",
    "Mag filter",
    "Min filter",
];
fn numbers<const N: usize>(xs: &[String]) -> Result<[f32; N], String> {
    if xs.len() != N {
        return Err("Incomplete native surface fields".into());
    }
    let mut out = [0.0; N];
    for (o, s) in out.iter_mut().zip(xs) {
        *o = s
            .parse::<f32>()
            .map_err(|_| "Surface inputs must be numeric")?;
        if !o.is_finite() {
            return Err("Surface inputs must be finite".into());
        }
    }
    Ok(out)
}
fn factors(m: &MaterialRecord) -> Vec<String> {
    m.base_color
        .iter()
        .chain(&m.emissive)
        .copied()
        .chain([
            m.metallic,
            m.roughness,
            m.specular,
            m.normal_scale,
            m.occlusion_strength,
            m.alpha_cutoff,
        ])
        .map(|v| v.to_string())
        .collect()
}
fn map_fields(role: TextureRole, m: &MaterialRecord) -> Vec<String> {
    let binding = TextureBindingDraft::from_material(role, m);
    let t = binding
        .map(|b| b.transform)
        .unwrap_or(TextureTransform::IDENTITY);
    let sampler = m.texture_samplers[role.index()];
    vec![
        binding
            .map(|b| crate::graphical::surfaces::binding::digest_hex(&b.digest))
            .unwrap_or_default(),
        t.offset[0].to_string(),
        t.offset[1].to_string(),
        t.scale[0].to_string(),
        t.scale[1].to_string(),
        t.rotation.to_string(),
        format!("{:?}", sampler.wrap_s),
        format!("{:?}", sampler.wrap_t),
        format!("{:?}", sampler.mag_filter),
        format!("{:?}", sampler.min_filter),
    ]
}
fn map_binding(role: TextureRole, xs: &[String]) -> Result<Option<TextureBindingDraft>, String> {
    if xs.len() != 10 {
        return Err("Incomplete native texture fields".into());
    }
    if xs[0].trim().is_empty() {
        return Ok(None);
    }
    let n = numbers::<5>(&xs[1..6])?;
    let string = |s: &str| Value::String(s.into());
    let transform = Value::Record(std::collections::BTreeMap::from([
        (
            "offset".into(),
            Value::List(n[..2].iter().map(|v| Value::F64(*v as f64)).collect()),
        ),
        (
            "scale".into(),
            Value::List(n[2..4].iter().map(|v| Value::F64(*v as f64)).collect()),
        ),
        ("rotation".into(), Value::F64(n[4] as f64)),
        ("tex_coord".into(), Value::I64(0)),
    ]));
    let r = Value::Record(std::collections::BTreeMap::from([
        ("role".into(), string(role.name())),
        ("sha256".into(), string(xs[0].trim())),
        ("colour_space".into(), string(role.colour_space())),
        ("channels".into(), string(role.channels())),
        ("transform".into(), transform),
        ("wrap_s".into(), string(&xs[6])),
        ("wrap_t".into(), string(&xs[7])),
        ("mag_filter".into(), string(&xs[8])),
        ("min_filter".into(), string(&xs[9])),
    ]));
    TextureBindingDraft::from_value(role, &r).map(Some)
}
fn edited(
    template: &SurfaceDraft,
    xs: &[String],
    flags: &[bool],
    mode: &str,
    shading: &str,
    maps: &[Vec<String>],
) -> Result<SurfaceDraft, String> {
    let n = numbers::<13>(xs)?;
    if flags.len() != 6 || maps.len() != 6 {
        return Err("Incomplete visual surface draft".into());
    }
    let mut m = template.material;
    m.base_color = n[..4].try_into().expect("fixed fields");
    m.emissive = n[4..7].try_into().expect("fixed fields");
    m.metallic = n[7];
    m.roughness = n[8];
    m.specular = n[9];
    m.normal_scale = n[10];
    m.occlusion_strength = n[11];
    m.alpha_cutoff = n[12];
    m.double_sided = flags[0];
    m.casts_shadow = flags[1];
    m.receives_shadow = flags[2];
    m.pickable = flags[3];
    m.multiply_vertex_color = flags[4];
    m.wind_animated = flags[5];
    m.opacity_mode = match mode {
        "Opaque" => OpacityMode::Opaque,
        "Mask" => OpacityMode::Mask,
        "Blend" => OpacityMode::Blend,
        _ => return Err("Unknown opacity mode".into()),
    };
    m.shading_model = match shading {
        "MetallicRoughness" => ShadingModel::MetallicRoughness,
        "Stylized" => ShadingModel::Stylized,
        _ => return Err("Unknown shading model".into()),
    };
    for role in TextureRole::ALL {
        if let Some(binding) = map_binding(role, &maps[role.index()])? {
            binding.apply(&mut m)?;
        } else {
            role.set_digest(&mut m, [0; 32]);
            m.texture_transforms[role.index()] = TextureTransform::IDENTITY;
            m.texture_samplers[role.index()] =
                qualia_core_db::container_10d::TextureSampler::GLTF_DEFAULT;
        }
    }
    crate::graphical::surfaces::draft::validate(&m)?;
    Ok(SurfaceDraft::from_material(&m))
}
#[component]
pub(super) fn SurfaceControls(
    selected: Signal<String>,
    inspected: Signal<Option<AuthoringView>>,
    fresh: bool,
    onedit: EventHandler<AuthoringEdit>,
) -> Element {
    let mut loaded = use_signal(|| None::<(SourceRef, String, SurfaceDraft)>);
    let mut values = use_signal(|| Vec::<String>::new());
    let mut flags = use_signal(|| vec![false; 6]);
    let mut maps = use_signal(|| vec![vec![String::new(); 10]; 6]);
    let mut mode = use_signal(|| "Opaque".to_owned());
    let mut shading = use_signal(|| "MetallicRoughness".to_owned());
    let mut uv_fields = use_signal(|| {
        vec![
            "0".to_owned(),
            "0".to_owned(),
            "1".to_owned(),
            "1".to_owned(),
            "0".to_owned(),
        ]
    });
    let mut axes = use_signal(|| "Stored UV0".to_owned());
    let mut metres = use_signal(|| "1".to_owned());
    let mut message = use_signal(String::new);
    let ready = fresh
        && loaded().as_ref().is_some_and(|(source, id, _)| {
            id == &selected() && inspected().as_ref().is_some_and(|v| &v.source == source)
        });
    rsx! {
        fieldset { disabled:!fresh,style:"margin-top:10px;border:1px solid #405264;",
            legend {"UV0 and full visual material"}
            p {"UV projection uses part-local metres. Projection overrides stored UV0; removing it restores stored UV0. Map transforms use UV0 only. Visual factors do not assert physical material properties."}
            button { disabled:!fresh||selected().is_empty(),onclick:move |_|{
                if let Some((source,part))=inspected().and_then(|v|v.parts.iter().find(|p|p.id==selected()).cloned().map(|p|(v.source,p))) {
                    let m=part.surface.material;
                    values.set(factors(&m));flags.set(vec![m.double_sided,m.casts_shadow,m.receives_shadow,m.pickable,m.multiply_vertex_color,m.wind_animated]);
                    maps.set(TextureRole::ALL.into_iter().map(|r|map_fields(r,&m)).collect());
                    mode.set(format!("{:?}",m.opacity_mode));shading.set(format!("{:?}",m.shading_model));
                    let t=part.uv_transform;uv_fields.set(vec![t.offset[0].to_string(),t.offset[1].to_string(),t.scale[0].to_string(),t.scale[1].to_string(),t.rotation.to_string()]);
                    axes.set(part.uv_projection.map(|p|match p.axes {[0,1]=>"XY",[0,2]=>"XZ",_=>"YZ"}).unwrap_or("Stored UV0").into());
                    metres.set(part.uv_projection.map(|p|p.metres_per_repeat.to_string()).unwrap_or_else(||"1".into()));
                    loaded.set(Some((source,part.id,part.surface)));message.set("Selected accepted surface loaded. Applying a shared material changes every part using its identifier.".into());
                }
            },"Load selected surface and UV"}
            if loaded().is_some() {
                div { style:"display:flex;flex-wrap:wrap;gap:6px;",
                    for (i,label) in FACTORS.iter().enumerate() {label {style:"width:110px;font-size:12px;","{label}" input {disabled:!ready,value:"{values()[i]}",aria_label:"Visual {label}",style:"width:104px;",oninput:move |e|values.write()[i]=e.value()}}}
                }
                select {disabled:!ready,value:"{mode}",aria_label:"Visual opacity mode",onchange:move |e|mode.set(e.value()),for name in ["Opaque","Mask","Blend"] {option {value:"{name}","{name}"}}}
                select {disabled:!ready,value:"{shading}",aria_label:"Visual shading model",onchange:move |e|shading.set(e.value()),for name in ["MetallicRoughness","Stylized"] {option {value:"{name}","{name}"}}}
                for (i,label) in FLAGS.iter().enumerate() {label {"{label}" input {r#type:"checkbox",disabled:!ready,checked:flags()[i],aria_label:"Visual {label}",onchange:move |e|flags.write()[i]=e.checked()}}}
                for role in TextureRole::ALL {
                    details {summary {"{role.name()} · {role.colour_space()} · {role.channels()}"}
                        p {"Choose an existing digest from the bound authorised HMC. Missing/corrupt or mismatched data refuses the whole edit."}
                        div {style:"display:flex;flex-wrap:wrap;gap:6px;",for (i,label) in MAP_FIELDS.iter().enumerate() {
                            label {style:"font-size:12px;","{label}"
                                if i<6 {input {disabled:!ready,value:"{maps()[role.index()][i]}",aria_label:"{role.name()} {label}",oninput:move |e|maps.write()[role.index()][i]=e.value()}}
                                else if i<8 {select {disabled:!ready,value:"{maps()[role.index()][i]}",aria_label:"{role.name()} {label}",onchange:move |e|maps.write()[role.index()][i]=e.value(),for name in ["Repeat","ClampToEdge","MirroredRepeat"] {option {value:"{name}","{name}"}}}}
                                else if i==8 {select {disabled:!ready,value:"{maps()[role.index()][i]}",aria_label:"{role.name()} {label}",onchange:move |e|maps.write()[role.index()][i]=e.value(),for name in ["Nearest","Linear"] {option {value:"{name}","{name}"}}}}
                                else {select {disabled:!ready,value:"{maps()[role.index()][i]}",aria_label:"{role.name()} {label}",onchange:move |e|maps.write()[role.index()][i]=e.value(),for name in ["Nearest","Linear","NearestMipmapNearest","LinearMipmapNearest","NearestMipmapLinear","LinearMipmapLinear"] {option {value:"{name}","{name}"}}}}
                            }
                        }}
                    }
                }
                button {disabled:!ready,onclick:move |_|{
                    if !ready {message.set("Reload the current selected surface before applying".into());return;}
                    if let Some((_,_,template))=loaded() {match edited(&template,&values(),&flags(),&mode(),&shading(),&maps()) {
                        Ok(surface)=>onedit.call(AuthoringEdit::Surface {material_id:template.material.id,surface}),Err(e)=>message.set(e),
                    }}
                },"Apply shared visual material"}
                div {style:"margin-top:8px;",
                    select {disabled:!ready,value:"{axes}",aria_label:"Part UV projection axes",onchange:move |e|axes.set(e.value()),for name in ["Stored UV0","XY","XZ","YZ"] {option {value:"{name}","{name}"}}}
                    input {disabled:!ready,value:"{metres}",aria_label:"UV metres per repeat",oninput:move |e|metres.set(e.value())}
                    button {disabled:!ready,onclick:move |_|{
                        if !ready {return;}
                        let projection=if axes()=="Stored UV0" {Ok(None)} else {metres().parse::<f32>().map_err(|_|"Enter finite metres per repeat".to_string()).and_then(|metres_per_repeat|{
                            let p=uv::PlanarUvDraft {axes:match axes().as_str() {"XY"=>[0,1],"XZ"=>[0,2],_=>[1,2]},metres_per_repeat};uv::projection_source(&p).map(|_|Some(p))
                        })};
                        match projection {Ok(projection)=>onedit.call(AuthoringEdit::UvProjection {id:selected(),projection}),Err(e)=>message.set(e)}
                    },"Apply part UV projection"}
                    for (i,label) in ["Part UV offset U","Part UV offset V","Part UV scale U","Part UV scale V","Part UV radians"].iter().enumerate() {
                        label {"{label}" input {disabled:!ready,value:"{uv_fields()[i]}",aria_label:"{label}",oninput:move |e|uv_fields.write()[i]=e.value()}}
                    }
                    button {disabled:!ready,onclick:move |_|{
                        if !ready {return;}
                        match numbers::<5>(&uv_fields()).and_then(|n|{let t=TextureTransform {offset:[n[0],n[1]],scale:[n[2],n[3]],rotation:n[4],tex_coord:0};uv::validate(&t).map(|_|t)}) {
                            Ok(transform)=>onedit.call(AuthoringEdit::UvTransform {id:selected(),transform}),Err(e)=>message.set(e),
                        }
                    },"Apply part UV transform"}
                }
            }
            p {role:"status","{message}"}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn full_controls_preserve_material_fields_and_refuse_bad_map_semantics() {
        let template = SurfaceDraft::from_material(&MaterialRecord::legacy_default());
        let mut maps = TextureRole::ALL
            .into_iter()
            .map(|r| map_fields(r, &template.material))
            .collect::<Vec<_>>();
        let flags = [false, true, true, true, true, false];
        assert_eq!(
            edited(
                &template,
                &factors(&template.material),
                &flags,
                "Opaque",
                "MetallicRoughness",
                &maps
            )
            .unwrap()
            .material,
            template.material
        );
        maps[0][0] = "C:/private/image.png".into();
        assert!(edited(
            &template,
            &factors(&template.material),
            &flags,
            "Opaque",
            "MetallicRoughness",
            &maps
        )
        .is_err());
    }
}
