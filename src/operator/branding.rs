//! Compiled Webizen AI Synth marks used by the operator page, tray, and installers.
pub const APP_DISPLAY_NAME: &str = "Webizen AI Synth";
pub const ICON_PNG: &[u8] = include_bytes!("../../assets/branding/icon-256.png");
pub const ICON_64_PNG: &[u8] = include_bytes!("../../assets/branding/icon-64.png");
pub const ICON_32_PNG: &[u8] = include_bytes!("../../assets/branding/icon-32.png");
pub const ICON_ICO: &[u8] = include_bytes!("../../assets/branding/icon.ico");
pub const EMBLEM_SVG: &[u8] = include_bytes!("../../assets/branding/emblem.svg");
pub const BANNER_PNG: &[u8] = include_bytes!("../../assets/branding/banner.png");

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
