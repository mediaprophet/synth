//! Native package selection; content hashes do not grant authority.
#[path = "device_lease.rs"]
pub mod device_lease;
pub use device_lease::{
    DeviceCapabilityReport, DeviceLeaseError, DeviceLeaseState, SharedDeviceLease,
    SurfaceExtent16x9,
};

use crate::graphical::surfaces::TextureResources;
use qualia_core_db::{container_10d::MaterialRecord, render::assets::TextureDependency};
use vibe::Value;
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SurfacePackageRequest {
    pub filename: String,
    pub digest: [u8; 32],
}
impl SurfacePackageRequest {
    pub fn validate(&self) -> Result<(), String> {
        if self.filename.is_empty()
            || self.filename.len() > 128
            || self.filename.as_bytes().contains(&0)
            || self.digest == [0; 32]
        {
            return Err("Select a bounded HMC filename and its inspected SHA256".into());
        }
        Ok(())
    }
}
/// Trusted raw-host classification for binding; unknown failures must be treated
/// as possibly mutated. BeforeMutation is valid only before resources or execution
/// context have been installed or changed.
#[derive(Debug)]
pub enum SurfaceBindingFailure {
    BeforeMutation(String),
    MayHaveMutated(String),
}
pub fn validate_value(value: &Value, resources: Option<&TextureResources>) -> Result<(), String> {
    if let Value::Record(record) = value {
        if let Some(Value::List(parts)) = record.get("parts") {
            if parts.iter().any(|p|matches!(p,Value::Record(r) if r.get("detail").is_some_and(|d|!matches!(d,Value::Null)))) {
                let resolved=crate::graphical::surfaces::resolution::resolve(value,resources)?;
                return crate::graphical::surfaces::resources::validate_material_resources(&resolved.materials.values().copied().collect::<Vec<_>>(),resolved.resources.as_ref());
            }
            if parts.len() > 256 {
                return Err("Surface part working bound exceeded".into());
            }
            let mut materials = std::collections::BTreeMap::new();
            for part in parts {
                let Value::Record(part) = part else {
                    return Err("Expected assembly part record".into());
                };
                let m = crate::graphical::materials::material_from_value(
                    part.get("material").ok_or("Part lacks material")?,
                )?;
                if let Some(old) = materials.insert(m.id, m) {
                    if old != m {
                        return Err("Conflicting material identifier declarations".into());
                    }
                }
            }
            return crate::graphical::surfaces::resources::validate_material_resources(
                &materials.values().copied().collect::<Vec<_>>(),
                resources,
            );
        }
        let material = if let Some(m) = record.get("material") {
            Some(m)
        } else if record.contains_key("base_color") {
            Some(value)
        } else {
            None
        };
        if let Some(m) = material {
            let m = crate::graphical::materials::material_from_value(m)?;
            crate::graphical::surfaces::resources::validate_material_resources(&[m], resources)?;
        }
    }
    Ok(())
}
pub fn dependencies(
    materials: &[MaterialRecord],
    resources: Option<&TextureResources>,
) -> Result<Vec<TextureDependency>, String> {
    crate::graphical::surfaces::resources::validate_material_resources(materials, resources)?;
    match resources {
        Some(r) => r.dependencies(materials),
        None => Ok(Vec::new()),
    }
}
