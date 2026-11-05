//! Bounded cold authoring. Source and native project history remain host-owned.
#![forbid(unsafe_code)]
pub mod assembly;
mod frames;
pub mod geo_studio;
pub mod geometry;
pub use profile_authoring::profile;
pub use assembly::{
    AssemblyLimits, AssemblyPart, CompiledAssembly, CompiledPart, Transform, compile_assembly,
};
pub use geometry::{GeometryLimits, profile_sweep, sphere, torus};
pub use profile::{Profile, ProfileLimits};
pub use profile_authoring::AuthoringError;
pub(crate) fn finite_positive(value: f64) -> bool {
    value.is_finite() && value > 0.0 && (value as f32).is_finite()
}
#[cfg(test)]
mod tests;
