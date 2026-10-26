use super::core;
use super::core::{ConstructSeed, ContextRef, ManifoldSeed, SeedContainer, TargetIdentifier};
#[allow(dead_code)]
#[path = "../../../../wasm/src/tool_chest/constructs/mod.rs"]
mod construct_seeds;
#[path = "../../../../wasm/src/tool_chest/manifolds/studio.rs"]
mod studio_seed;

/// A selected native projection, not a replacement registry or document owner.
#[derive(Clone)]
pub struct WorkspaceProjection {
    pub construct: ConstructSeed,
    pub manifold: ManifoldSeed,
    pub context: ContextRef,
    pub source_target: TargetIdentifier,
}
impl WorkspaceProjection {
    pub fn for_source(artifact_id: &str) -> Result<Self, String> {
        if artifact_id.is_empty() || artifact_id.len() > 4096 || artifact_id.as_bytes().contains(&0)
        {
            return Err(
                "Poet workspace requires a bounded actual source component identifier".into(),
            );
        }
        // Reuse the original Studio seed, retaining its Dual Studio, Scene and
        // Audio session containers. Unmounted native services remain visible/held.
        let mut manifold = studio_seed::studio_manifold_seed();
        for (index, container) in manifold.containers.iter_mut().enumerate() {
            container.id = format!("studio-source-{index}:{artifact_id}");
            container.honesty = if container.container_type == "audio_session" {
                "held"
            } else {
                "partial"
            }
            .into();
            container.semantic_uri = artifact_id.into();
        }
        let mut inspector = SeedContainer::new(
            "inspector",
            "Spatial source and part context",
            1420.0,
            30.0,
            360.0,
            520.0,
        );
        inspector.id = format!("studio-inspector:{artifact_id}");
        inspector.honesty = "partial".into();
        inspector.semantic_uri = artifact_id.into();
        manifold.containers.push(inspector);
        // A source record is not a new observer construct or an authorship claim.
        // Retain the real bundled Poet construct and every existing lens identifier.
        let mut construct = construct_seeds::poet_construct();
        construct.honesty = "partial".into();
        construct.description.push_str(" Native projection currently selects Studio source/spatial context; other native lenses and graphical usability remain separately qualified.");
        Ok(Self {
            construct,
            manifold,
            context: ContextRef::default(),
            source_target: TargetIdentifier::component_id(artifact_id),
        })
    }
}
