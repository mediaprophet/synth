//! Native Poet surface projection. MAT4, codecs, HMC keys and GPU remain Core-owned.
pub mod binding;
pub mod detail;
#[cfg(test)]
mod detail_pixels;
pub mod draft;
pub mod inspection;
#[cfg(test)]
mod integration_tests;
pub mod resolution;
pub mod resources;
pub mod texture_inputs;
pub mod uv;
pub mod uv_atlas;
pub mod wire;
pub use binding::{TextureBindingDraft, TextureRole};
pub use draft::SurfaceDraft;
pub use resources::{SurfaceTextureLimits, TextureResources};
pub use uv::UvTransformDraft;

#[cfg(test)]
mod pixels;
#[cfg(test)]
mod tests;
