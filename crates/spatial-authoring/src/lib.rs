//! Bounded cold authoring. Source and native project history remain host-owned.
#![forbid(unsafe_code)]
pub mod assembly;
mod frames;
pub mod geo_studio;
pub mod geometry;
pub mod profile;
pub use assembly::{
    AssemblyLimits, AssemblyPart, CompiledAssembly, CompiledPart, Transform, compile_assembly,
};
pub use geometry::{GeometryLimits, profile_sweep, sphere, torus};
pub use profile::{Profile, ProfileLimits};
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthoringError {
    Invalid(&'static str),
    Budget(&'static str),
    Kernel(String),
}
impl std::fmt::Display for AuthoringError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Invalid(m) | Self::Budget(m) => f.write_str(m),
            Self::Kernel(m) => f.write_str(m),
        }
    }
}
impl std::error::Error for AuthoringError {}
pub(crate) fn finite_positive(value: f64) -> bool {
    value.is_finite() && value > 0.0 && (value as f32).is_finite()
}
#[cfg(test)]
mod tests;
