//! Compiled Webizen AI Synth marks used by the operator page, tray, and installers.
//!
//! Aether (Dioxus fork) reads `AETHER_APP_TITLE` / `AETHER_PRODUCT_NAME` for the
//! desktop window when the app has not passed a `WindowBuilder`. Claim those
//! before `Config::new()` so the toolkit never falls back to "Aether App".
pub const APP_DISPLAY_NAME: &str = "Webizen AI Synth";
pub const APP_BUNDLE_ID: &str = "au.webizen.synth";
pub const APP_PUBLISHER: &str = "Webizen AI";
pub const ICON_PNG: &[u8] = include_bytes!("../../assets/branding/icon-256.png");
pub const ICON_64_PNG: &[u8] = include_bytes!("../../assets/branding/icon-64.png");
pub const ICON_32_PNG: &[u8] = include_bytes!("../../assets/branding/icon-32.png");
pub const ICON_ICO: &[u8] = include_bytes!("../../assets/branding/icon.ico");
pub const EMBLEM_SVG: &[u8] = include_bytes!("../../assets/branding/emblem.svg");
pub const BANNER_PNG: &[u8] = include_bytes!("../../assets/branding/banner.png");

/// Tell the Aether toolkit which application it is hosting.
pub fn claim_toolkit_identity() {
    std::env::set_var("AETHER_APP_TITLE", APP_DISPLAY_NAME);
    std::env::set_var("AETHER_PRODUCT_NAME", APP_DISPLAY_NAME);
}

/// WebView2 / WebKit profile directory for this app, not next to `synth.exe`.
pub fn webview_data_dir() -> Option<std::path::PathBuf> {
    let base = if cfg!(windows) {
        std::env::var_os("LOCALAPPDATA").map(std::path::PathBuf::from)
    } else if cfg!(target_os = "macos") {
        std::env::var_os("HOME").map(|home| {
            std::path::PathBuf::from(home).join("Library").join("Application Support")
        })
    } else {
        std::env::var_os("XDG_DATA_HOME")
            .map(std::path::PathBuf::from)
            .or_else(|| {
                std::env::var_os("HOME")
                    .map(|home| std::path::PathBuf::from(home).join(".local").join("share"))
            })
    }?;
    Some(base.join(APP_DISPLAY_NAME))
}

pub fn asset(name: &str) -> Option<(&'static str, &'static [u8])> {
    match name {
        "icon.png" | "icon-256.png" => Some(("image/png", ICON_PNG)),
        "icon-64.png" => Some(("image/png", ICON_64_PNG)),
        "icon-32.png" => Some(("image/png", ICON_32_PNG)),
        "favicon.ico" | "icon.ico" => Some(("image/x-icon", ICON_ICO)),
        "emblem.svg" => Some(("image/svg+xml", EMBLEM_SVG)),
        "banner.png" => Some(("image/png", BANNER_PNG)),
        _ => None,
    }
}

fn b64(bytes: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = chunk.get(1).copied().unwrap_or(0) as u32;
        let b2 = chunk.get(2).copied().unwrap_or(0) as u32;
        let triple = (b0 << 16) | (b1 << 8) | b2;
        out.push(TABLE[((triple >> 18) & 0x3f) as usize] as char);
        out.push(TABLE[((triple >> 12) & 0x3f) as usize] as char);
        out.push(if chunk.len() > 1 {
            TABLE[((triple >> 6) & 0x3f) as usize] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            TABLE[(triple & 0x3f) as usize] as char
        } else {
            '='
        });
    }
    out
}

pub fn emblem_data_uri() -> String {
    format!("data:image/svg+xml;base64,{}", b64(EMBLEM_SVG))
}

pub fn banner_data_uri() -> String {
    format!("data:image/png;base64,{}", b64(BANNER_PNG))
}
