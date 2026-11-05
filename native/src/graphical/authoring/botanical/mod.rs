//! Versioned source-owned botanical graph editing for the native Poet owner.
mod controls;
pub(crate) mod typed_controls;
mod recipe;
pub(crate) mod junction_source;
pub(crate) mod junction_edit;
pub mod multi_junction;
pub mod organ_state;
mod source_edit;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod junction_tests;

pub use controls::BotanicalControls;
pub use junction_source::{JunctionSettings, JunctionSource};
pub use multi_junction::{
    weld_multi_junction, ChildBranchAttachment, MultiJunctionBudget, MultiJunctionError,
    MultiJunctionSpec, WeldedMultiJunctionMesh,
};
pub use organ_state::{
    BotanicalDevelopmentalStage, BotanicalOrganStateGraph, OrganKind, OrganStateError,
    PersistentOrganDescriptor,
};
pub use recipe::{read_recipe, BotanicalLimits, BotanicalRecipe, Organ, SCHEMA};
pub use source_edit::{edit_source, validate_projection, BotanicalEdit};
