fn main() {
    println!("cargo:rerun-if-changed=assets/branding/icon.ico");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("assets/branding/icon.ico");
        res.set("ProductName", "Webizen AI Synth");
        res.set("FileDescription", "Webizen AI Synth operator");
        res.set("LegalCopyright", "Apache-2.0");
        res.set("CompanyName", "Webizen AI");
        if let Err(err) = res.compile() {
            println!("cargo:warning=winresource: {err}");
        }
    }
}
