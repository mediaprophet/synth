fn main() {
    println!("cargo:rerun-if-changed=assets/branding/icon.ico");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("assets/branding/icon.ico");
        res.set("ProductName", "Webizen AI Synth");
        res.set("FileDescription", "Webizen AI Synth operator");
        res.set("LegalCopyright", "Copyright (c) 2026 Timothy Charles Holborn (CC-BY-NC-ND-4.0)");
        res.set("CompanyName", "Webizen AI");
        res.set("FileVersion", "0.0.41.13");
        res.set("ProductVersion", "0.0.41.13");
        if let Err(err) = res.compile() {
            println!("cargo:warning=winresource: {err}");
        }
    }
}
