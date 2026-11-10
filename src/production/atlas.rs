//! Master reference-body pack. The VibeScript names the body. This calls the
//! existing HRA packer and admits the anatomy-pack bytes as file handles.
//! The files producer writes the HMC. Replay in the same process reuses the
//! first fetch so the package bytes stay stable.
//!
//! `quality: master` is the complete discovered body. Lower tiers for a device,
//! a connection, or a task are derived from this master later.

use std::collections::BTreeMap;
use std::sync::{Mutex, OnceLock};

use super::assets::AssetHost;
use vibe::Value;

pub(super) fn pack_reference_body(
    host: &mut AssetHost<'_>,
    args: &Value,
) -> Result<Value, String> {
    #[cfg(feature = "atlas")]
    {
        pack_master(host, args)
    }
    #[cfg(not(feature = "atlas"))]
    {
        let _ = (host, args);
        Err("Rebuild with --features atlas (synth-atlas) to pack the reference body".into())
    }
}

fn arg_string<'a>(args: &'a Value, key: &str) -> Result<&'a str, String> {
    let Value::Record(map) = args else {
        return Err("atlas arguments must be a record".into());
    };
    match map.get(key) {
        Some(Value::String(value)) => Ok(value),
        Some(_) => Err(format!("{key} must be a string")),
        None => Err(format!("missing {key}")),
    }
}

fn optional_string<'a>(args: &'a Value, key: &str) -> Option<&'a str> {
    let Value::Record(map) = args else {
        return None;
    };
    match map.get(key) {
        Some(Value::String(value)) => Some(value),
        _ => None,
    }
}

#[cfg(feature = "atlas")]
#[derive(Clone)]
struct Packed {
    model: String,
    hmc: Vec<u8>,
    q42: Vec<u8>,
    organs_packed: usize,
    bundle_bytes: usize,
    total_10d_bytes: usize,
    q42_quins: usize,
    q42_graph_bytes: usize,
    packed_keys: Vec<String>,
    failed: Vec<(String, String)>,
    curated_not_found: Vec<String>,
}

#[cfg(feature = "atlas")]
fn cache() -> &'static Mutex<BTreeMap<String, Packed>> {
    static CACHE: OnceLock<Mutex<BTreeMap<String, Packed>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(BTreeMap::new()))
}

#[cfg(feature = "atlas")]
fn pack_master(host: &mut AssetHost<'_>, args: &Value) -> Result<Value, String> {
    let model_name = arg_string(args, "model")?;
    let quality = optional_string(args, "quality").unwrap_or("master");
    let scope = optional_string(args, "scope").unwrap_or("complete");
    if quality != "master" {
        return Err(
            "This call packs quality master. Lower tiers are derived from the master pack."
                .into(),
        );
    }
    if scope != "complete" {
        return Err("scope must be complete for the master atlas".into());
    }
    let constitution_json = optional_string(args, "constitution_json").unwrap_or("");
    let (fit_bytes, constitution_bytes) =
        qualia_anatomy_pack::body_constitution::fit_json_for_reference(
            model_name,
            constitution_json,
        )?;
    let packed = load_or_build(model_name)?;
    let report = serde_json::to_vec_pretty(&serde_json::json!({
        "quality": "master",
        "scope": "complete",
        "delivery": "master is the reference body. A person's constitution_json produces male.fit.json / female.fit.json. QualiaPortal set_body_fit_json morphs the organ vertices from that fit. Lower tiers are derived from this master for the device, the connection, and the task.",
        "model": packed.model,
        "organs_packed": packed.organs_packed,
        "bundle_bytes": packed.bundle_bytes,
        "total_10d_bytes": packed.total_10d_bytes,
        "q42_quins": packed.q42_quins,
        "q42_graph_bytes": packed.q42_graph_bytes,
        "packed_keys": packed.packed_keys,
        "failed": packed.failed,
        "curated_not_found": packed.curated_not_found,
        "licence": "CC-BY-4.0",
        "source": "Human Reference Atlas reference organs, complete discovered body",
        "fit": serde_json::from_slice::<serde_json::Value>(&fit_bytes).unwrap_or(serde_json::Value::Null)
    }))
    .map_err(|e| e.to_string())?;
    let hmc = host.admit_encoded("hmc", packed.hmc)?;
    let q42 = host.admit_encoded("q42", packed.q42)?;
    let report = host.admit_encoded("json", report)?;
    let fit = host.admit_encoded("json", fit_bytes)?;
    let constitution = host.admit_encoded("json", constitution_bytes)?;
    Ok(Value::Record(BTreeMap::from([
        ("hmc".into(), hmc),
        ("q42".into(), q42),
        ("report".into(), report),
        ("fit".into(), fit),
        ("constitution".into(), constitution),
        ("quality".into(), Value::String("master".into())),
        ("model".into(), Value::String(model_name.into())),
    ])))
}

#[cfg(feature = "atlas")]
fn load_or_build(model_name: &str) -> Result<Packed, String> {
    {
        let guard = cache().lock().unwrap_or_else(|poison| poison.into_inner());
        if let Some(hit) = guard.get(model_name) {
            return Ok(hit.clone());
        }
    }
    let model = qualia_anatomy_pack::anatomy_pack::model_from_token(model_name)?;
    let dir = std::env::temp_dir().join("synth-anatomy-master");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let out = dir.join(format!("anatomy-{model_name}.hmc"));
    let built =
        qualia_anatomy_pack::anatomy_pack::build_anatomy_pack(model, &out, None)?;
    if built.organs_packed == 0 {
        return Err(format!("no organs packed for {model_name}"));
    }
    let hmc = std::fs::read(&out).map_err(|e| e.to_string())?;
    let q42 = std::fs::read(&built.q42_sidecar_path).map_err(|e| e.to_string())?;
    let packed = Packed {
        model: built.model,
        hmc,
        q42,
        organs_packed: built.organs_packed,
        bundle_bytes: built.bundle_bytes,
        total_10d_bytes: built.total_10d_bytes,
        q42_quins: built.q42_quins,
        q42_graph_bytes: built.q42_graph_bytes,
        packed_keys: built.packed_keys,
        failed: built.failed,
        curated_not_found: built.curated_not_found,
    };
    let mut guard = cache().lock().unwrap_or_else(|poison| poison.into_inner());
    guard.insert(model_name.to_string(), packed.clone());
    Ok(packed)
}
