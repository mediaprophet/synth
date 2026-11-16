//! Encoded native outputs for clients that implement their own native UI.
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use crate::production::Resource;

pub(super) fn resources(outputs: &[Resource]) -> Value {
    Value::Array(outputs.iter().map(|r| json!({
        "key": r.key,
        "kind": r.kind,
        "byte_length": r.bytes.len(),
        "sha256": format!("{:x}", Sha256::digest(&r.bytes)),
        "base64": crate::present::base64_encode(&r.bytes),
    })).collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    const SOURCE: &str = r#"requires [capability("capability.invoke")];
let image = capability.invoke("hypermedia.radiance.create_rgba", {width:8,height:8,color:[51,102,153,255]});
let png = capability.invoke("hypermedia.radiance.encode_rgba_png", {image:image});
let mut asset = {files:[{key:"image.png",resource:png}]};"#;

    #[test]
    fn native_resources_and_package_are_replayed_without_html() {
        let body = serde_json::to_vec(&json!({"source":SOURCE,"include_html":false,"include_resources":true,"include_package":true})).unwrap();
        let first = super::super::handle_produce(&body);
        assert_eq!(first["ok"], true, "{first}");
        assert_eq!(first["strict_replay_verified"], true);
        assert!(first.get("cards_html").is_none());
        assert_eq!(first["resources"][0]["key"], "image.png");
        assert_eq!(first["resources"][0]["kind"], "png");
        assert!(first["resources"][0]["base64"].as_str().unwrap().starts_with("iVBORw0KGgo"));
        assert!(first["resources"][0]["byte_length"].as_u64().unwrap() > 8);
        assert_eq!(first["resources"][0]["sha256"].as_str().unwrap().len(), 64);
        assert!(!first["package_base64"].as_str().unwrap().is_empty());
        let second = super::super::handle_produce(&body);
        assert_eq!(first["package_sha256"], second["package_sha256"]);
        assert_eq!(first["resources"], second["resources"]);
    }

    #[test]
    fn existing_html_client_contract_is_preserved() {
        let body = serde_json::to_vec(&json!({"source":SOURCE})).unwrap();
        let result = super::super::handle_produce(&body);
        assert_eq!(result["ok"], true, "{result}");
        assert!(result["cards_html"].as_str().unwrap().contains("image.png"));
        assert!(result.get("resources").is_none());
        assert!(result.get("package_base64").is_none());
    }
}
