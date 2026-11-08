//! General-purpose VibeScript authoring, pluggable asset production and native HMC packaging.
//! Domain-specific generators implement [`production::AssetProducer`].
pub mod agent;
mod host;
#[cfg(feature = "native-ui")]
pub mod native_ui;
pub mod production;
pub mod vibe_runtime;
pub use host::{HostSnapshot, NativeContext, NativeOutputRoot, SavedCandidate, SynthHost};
