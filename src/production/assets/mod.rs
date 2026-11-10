//! Application adapter. Production algorithms, ontological contracts, and operation
//! descriptors live in Hypermedia.
pub use asset_production_vibe::{ontology, AssetWorkspace, OPERATIONS};
use std::borrow::Cow;

/// Resolves any canonical categorical morphism ID (`hypermedia.<category>.<morphism>`)
/// or historical `hypermedia.*`/`synth.*` alias to its canonical ID so saved `.hmc`
/// source cells retain byte-exact replay while new cells use the categorical taxonomy.
pub fn canonical(id: &str) -> Cow<'_, str> {
    if let Some(canonical_id) = ontology::canonical_morphism_id(id) {
        return Cow::Borrowed(canonical_id);
    }
    Cow::Borrowed(id)
}
pub fn catalogue() -> serde_json::Value {
    let mut value = asset_production_vibe::catalogue();
    value["count"] = serde_json::json!(OPERATIONS.len());
    value["legacy_aliases"] = serde_json::json!(ontology::MORPHISMS
        .iter()
        .flat_map(|m| m.aliases.iter().map(move |alias| {
            serde_json::json!({"id": alias, "canonical": m.id})
        }))
        .collect::<Vec<_>>());
    value
}
pub(crate) fn arguments(id: &str) -> &'static str {
    asset_production_vibe::arguments(&canonical(id))
}
pub(crate) fn tool_limits(limits: &super::ProductionLimits) -> asset_production_vibe::ToolLimits {
    asset_production_vibe::ToolLimits {
        resources: limits.resources,
        resource_bytes: limits.resource_bytes,
        total_resource_bytes: limits.total_resource_bytes,
        asset_work_units: limits.asset_work_units,
    }
}
pub(crate) struct AssetHost<'a>(pub asset_production_vibe::AssetHost<'a>);
impl<'a> std::ops::Deref for AssetHost<'a> {
    type Target = asset_production_vibe::AssetHost<'a>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl std::ops::DerefMut for AssetHost<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
