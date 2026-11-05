//! Explicit seam/density editing; display data comes from the native worker.
use crate::graphical::surfaces::{inspection::SurfaceInspection, uv_atlas::AtlasDraft};
use dioxus::prelude::*;
#[component]
pub(super) fn UvAtlasControls(
    atlas: Option<AtlasDraft>,
    context_token: String,
    fresh: bool,
    inspection: Option<SurfaceInspection>,
    onapply: EventHandler<Option<AtlasDraft>>,
) -> Element {
    let mut loaded = use_signal(|| None::<(String, Option<AtlasDraft>)>);
    let mut seams = use_signal(Vec::<[u32; 2]>::new);
    let mut fields = use_signal(|| vec!["1024".into(), "4".into(), "64".into(), "64".into()]);
    let mut pair = use_signal(|| [String::new(), String::new()]);
    let mut page = use_signal(|| 0usize);
    let mut message = use_signal(String::new);
    let mut smooth_frames = use_signal(|| false);
    let ready = fresh
        && loaded()
            .as_ref()
            .is_some_and(|(context, original)| context == &context_token && original == &atlas);
    rsx! {fieldset{disabled:!fresh,style:"margin-top:8px;border:1px solid #405264;",
        legend{"UV unwrap and atlas · explicit seams"}
        p{"Seams must cut oriented manifold disk charts. Density uses part-local metres before TRS. Harmonic mapping can distort; packing may refuse at the chosen resolution/density. Reset planar projection and part UV affine explicitly before enabling an atlas."}
        button{onclick:move |_|{let a=atlas.clone().unwrap_or_default();smooth_frames.set(a.version == 2);seams.set(a.seams.clone());fields.set(vec![a.atlas_size.to_string(),a.gutter_pixels.to_string(),a.texels_per_metre.to_string(),a.iterations.to_string()]);loaded.set(Some((context_token.clone(),atlas.clone())));message.set("Accepted atlas settings loaded; base geometry and stored UVs stay retained.".into());},"Load accepted atlas or defaults"}
        for (i,name) in ["Atlas side pixels","Outer gutter pixels","Area texels per local metre","Harmonic iterations"].iter().enumerate(){label{"{name}" input{disabled:!ready,value:"{fields()[i]}",aria_label:"{name}",oninput:move|e|fields.write()[i]=e.value()}}}
        label{input{r#type:"checkbox",disabled:!ready,checked:smooth_frames(),onchange:move|e|smooth_frames.set(e.checked())} "Keep smooth shading across UV seams; preserve supplied normals"}
        p{"{seams().len()} explicit seam edges"}
        label{"First source vertex" input{disabled:!ready,value:"{pair()[0]}",aria_label:"Seam first source vertex",oninput:move|e|pair.write()[0]=e.value()}}
        label{"Second source vertex" input{disabled:!ready,value:"{pair()[1]}",aria_label:"Seam second source vertex",oninput:move|e|pair.write()[1]=e.value()}}
        button{disabled:!ready,onclick:move |_|{match(pair()[0].parse::<u32>(),pair()[1].parse::<u32>()){
            (Ok(a),Ok(b)) if a!=b=>{let edge=[a.min(b),a.max(b)];let mut next=seams();match next.binary_search(&edge){Ok(i)=>{next.remove(i);},Err(i)=>next.insert(i,edge)}seams.set(next);},_=>message.set("Enter two distinct source vertex indices".into())}},"Toggle seam pair"}
        if let Some(data)=inspection {
            p{"Available source edges; 64 per page. Select edges then apply and inspect again."}
            button{disabled:!ready||page()==0,onclick:move |_|page.set(page().saturating_sub(1)),"Previous edges"}
            button{disabled:!ready||(page()+1)*64>=data.edges.len(),onclick:move |_|page.set(page()+1),"Next edges"}
            for edge in data.edges.iter().skip(page()*64).take(64).copied(){label{style:"display:inline-block;margin:3px;",input{r#type:"checkbox",disabled:!ready,checked:seams().binary_search(&edge).is_ok(),aria_label:"Seam edge {edge[0]} to {edge[1]}",onchange:move|e|{let mut next=seams();match(next.binary_search(&edge),e.checked()){(Err(i),true)=>next.insert(i,edge),(Ok(i),false)=>{next.remove(i);},_=>{}}seams.set(next);}}"{edge[0]}–{edge[1]}"}}
        }
        button{disabled:!ready,onclick:move |_|{let draft=(||{let f=fields();let a=AtlasDraft{version:if smooth_frames(){2}else{1},seams:seams(),atlas_size:f[0].parse().map_err(|_|"Invalid atlas size")?,gutter_pixels:f[1].parse().map_err(|_|"Invalid gutter")?,texels_per_metre:f[2].parse().map_err(|_|"Invalid density")?,iterations:f[3].parse().map_err(|_|"Invalid iteration count")?};a.options()?;Ok::<_,String>(a)})();match draft{Ok(a)=>onapply.call(Some(a)),Err(e)=>message.set(e)}},"Apply source atlas recipe"}
        button{disabled:!ready,onclick:move |_|onapply.call(None),"Remove atlas; restore stored UV path"}
        p{role:"status","{message}"}
    }}
}
