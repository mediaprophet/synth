//! Shared borrowed admission before selected inspection or full scene construction.
use super::admission::{fields, geometry_counts, get, record};
use crate::graphical::{preview::PreviewLimits, surfaces::uv_atlas};
use qualia_core_db::{
    container_10d::TextureTransform,
    specialized_libs::computational_geometry::uv_atlas::AtlasLimits,
};
use vibe::Value;
pub(super) const PART_FIELDS: &[&str] = &[
    "id",
    "semantic_id",
    "parent",
    "geometry",
    "material",
    "translation",
    "rotation",
    "scale",
    "uv_transform",
    "uv_projection",
    "uv_atlas",
    "detail",
];
pub(crate) fn preflight(
    value: &Value,
    limits: PreviewLimits,
    atlas_limits: AtlasLimits,
) -> Result<(usize, usize, usize), String> {
    let part = record(value)?;
    fields(part, PART_FIELDS)?;
    let (v, t, mut out) = geometry_counts(record(get(part, "geometry")?)?, limits)?;
    if let Some(transform) = part.get("uv_transform") {
        crate::graphical::surfaces::uv::from_value(transform)?;
    }
    if let Some(projection) = part
        .get("uv_projection")
        .filter(|v| !matches!(v, Value::Null))
    {
        crate::graphical::surfaces::uv::projection_from_value(projection)?;
    }
    if let Some(atlas) = part.get("uv_atlas").filter(|v| !matches!(v, Value::Null)) {
        if part
            .get("uv_projection")
            .is_some_and(|v| !matches!(v, Value::Null))
        {
            return Err("Remove planar projection explicitly before applying an atlas".into());
        }
        if part
            .get("uv_transform")
            .map(crate::graphical::surfaces::uv::from_value)
            .transpose()?
            .is_some_and(|t| t != TextureTransform::IDENTITY)
        {
            return Err(
                "Reset the part UV affine transform explicitly before applying an atlas".into(),
            );
        }
        out = uv_atlas::preflight(&uv_atlas::from_value(atlas)?, v, t, atlas_limits)?;
    }
    if v > limits.vertices
        || out > limits.vertices
        || t.checked_mul(3).is_none_or(|i| i > limits.indices)
    {
        return Err("Selected geometry exceeds host working budget".into());
    }
    Ok((v, t, out))
}
