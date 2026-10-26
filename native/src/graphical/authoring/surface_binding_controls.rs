//! Explicit texture snapshot binding; package names are native-host selected paths.
use crate::graphical::surface_host::SurfacePackageRequest;
use dioxus::prelude::*;
#[component]
pub(super) fn SurfaceBindingControls(
    busy: Signal<bool>,
    onbind: EventHandler<SurfacePackageRequest>,
) -> Element {
    let mut name = use_signal(|| "working.hmc".to_string());
    let mut digest = use_signal(String::new);
    let mut error = use_signal(String::new);
    rsx! {details{summary{"Bind verified HMC texture resources"}
    p{"Use a project HMC filename and its inspected SHA-256. Preview and compilation use immutable verified bytes; edits to that file do not change the bound snapshot. After reopening textured source, bind resources before explicit rebuild."}
    input{disabled:busy(),value:"{name}",aria_label:"Texture resource HMC filename",oninput:move|e|name.set(e.value())}
    input{disabled:busy(),value:"{digest}",aria_label:"Inspected texture resource HMC SHA256",oninput:move|e|digest.set(e.value())}
    button{disabled:busy(),onclick:move |_|{
     let result=(||->Result<SurfacePackageRequest,String>{let text=digest();if text.len()!=64||!text.is_ascii(){return Err("Expected 64 hexadecimal SHA256 characters".into());}let mut hash=[0u8;32];for(i,byte)in hash.iter_mut().enumerate(){*byte=u8::from_str_radix(&text[i*2..i*2+2],16).map_err(|_|"Invalid SHA256")?;}let r=SurfacePackageRequest{filename:name(),digest:hash};r.validate()?;Ok(r)})();
     match result{Ok(r)=>{error.set(String::new());onbind.call(r)},Err(e)=>error.set(e)}
    },"Bind immutable texture snapshot"}
    p{"{error}"}
    }}
}
