//! Explicit, read-only website mount alongside the existing native production API.
//! `/` remains the QualiaDB studio; a website opens at `/aaa-proof.html`.
use std::{net::TcpStream, path::{Path, PathBuf}, sync::OnceLock};
static ROOT: OnceLock<PathBuf> = OnceLock::new();
const MAX_STATIC_BYTES: u64 = 64 * 1024 * 1024;

pub fn configure_website_root(root: &Path) -> Result<(), String> {
    let root = root.canonicalize().map_err(|e| format!("Website root: {e}"))?;
    if !root.join("site/aaa-proof.html").is_file() {
        return Err("Website root must contain site/aaa-proof.html".into());
    }
    ROOT.set(root).map_err(|_| "Website root is already configured".into())
}

fn mapping(path: &str) -> Option<(&'static str, String)> {
    match path {
        "/aaa-proof.html" | "/aaa-full-proof.html" | "/aaa-instanced-proof.html" | "/terrain-proof.html" | "/living-habitat-proof.html" => Some(("site", path[1..].into())),
        "/atlas" | "/atlas/" => Some(("site", "atlas/index.html".into())),
        "/journey" | "/journey/" => Some(("site", "journey/index.html".into())),
        _ if path.starts_with("/publication/") => Some(("publication/generated", path[13..].into())),
        _ if ["/world/", "/pkg/native-world/", "/pkg/horizon-world/", "/atlas/", "/journey/"].iter().any(|p| path.starts_with(p)) => Some(("site", path[1..].into())),
        _ if ["/data/publications/", "/production/aaa-proof/", "/production/aaa-instancing/", "/production/native-world/", "/production/aaa-field/", "/production/living-habitat/"].iter().any(|p| path.starts_with(p)) => Some(("", path[1..].into())),
        _ => None,
    }
}

pub(super) fn handles(path: &str) -> bool { ROOT.get().is_some() && mapping(path).is_some() }

fn resolved(root: &Path, path: &str) -> Option<PathBuf> {
    let (base, relative) = mapping(path)?;
    let relative = super::confine_qualia_rel(&relative)?;
    let base = root.join(base).canonicalize().ok()?;
    let file = base.join(relative).canonicalize().ok()?;
    // Reparse points must not escape the explicitly attached project.
    (base.starts_with(root) && file.starts_with(root) && file.starts_with(&base) && file.is_file()).then_some(file)
}

pub(super) fn serve(stream: TcpStream, path: &str, range: Option<&str>) -> Result<(), String> {
    let file = ROOT.get().and_then(|root| resolved(root, path));
    let Some(file) = file else {
        return super::write_http_response(stream,"404 Not Found","application/json",br#"{"ok":false,"error":"Public website file not found"}"#);
    };
    if std::fs::metadata(&file).map_err(|e| e.to_string())?.len() > MAX_STATIC_BYTES {
        return super::write_http_response(stream,"413 Content Too Large","application/json",br#"{"ok":false,"error":"Public static file exceeds 64 MiB"}"#);
    }
    let bytes = std::fs::read(&file).map_err(|e| e.to_string())?;
    super::write_ranged_bytes(stream, super::mime_for_qualia(&file.to_string_lossy()), &bytes, range)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn mount_is_opt_in_and_keeps_studio_and_private_data_out() {
        for p in ["/", "/index.html", "/project", "/.git/config", "/data/acquisition/Timeline.json", "/production/private/photo.jpg"] { assert!(mapping(p).is_none(), "{p}"); }
        assert_eq!(mapping("/publication/current.json"),Some(("publication/generated","current.json".into())));
        assert_eq!(mapping("/atlas/"),Some(("site","atlas/index.html".into())));
    }
    #[test]
    fn attached_site_paths_reject_traversal() {
        for p in ["/world/../../.git/config", "/world/%2e%2e/%2e%2e/.git/config", "/world/C:/secret", "/world/%5csecret"] {
            let (_, rel)=mapping(p).unwrap();
            assert!(super::super::confine_qualia_rel(&rel).is_none(), "{p}");
        }
    }
    #[test]
    fn horizon_review_has_separate_public_routes() {
        assert_eq!(mapping("/living-habitat-proof.html"),Some(("site","living-habitat-proof.html".into())));
        assert_eq!(mapping("/pkg/horizon-world/runtime-identity.json"),Some(("site","pkg/horizon-world/runtime-identity.json".into())));
        assert_eq!(mapping("/production/living-habitat/scene.json"),Some(("","production/living-habitat/scene.json".into())));
        assert!(mapping("/pkg/other-private-runtime/a.wasm").is_none());
        let (_,rel)=mapping("/pkg/horizon-world/../../.git/config").unwrap();
        assert!(super::super::confine_qualia_rel(&rel).is_none());
    }
}
