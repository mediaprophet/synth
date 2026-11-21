//! General-purpose VibeScript authoring, pluggable asset production and native HMC packaging.
//! Domain-specific generators implement [`production::AssetProducer`].
#![recursion_limit = "256"]
pub mod agent;
pub mod agent_index;
pub mod changelog_licensing;
#[cfg(not(target_arch = "wasm32"))]
pub mod cli;
mod host;
pub mod mcp_full;
pub mod model_validation;
#[cfg(not(target_arch = "wasm32"))]
pub mod operator;
#[cfg(feature = "native-ui")]
pub mod native_ui;
pub mod present;
pub mod present_3d;
pub mod present_portal;
pub mod present_project;
pub mod present_qac;
pub mod project;
pub mod present_capabilities;
pub mod present_style;
pub mod production;
pub mod session_admission;
pub mod tray_env;
pub mod vibe_runtime;
pub mod wasm_lite;
pub mod web_server;
pub use host::{HostSnapshot, NativeContext, NativeOutputRoot, SavedCandidate, SynthHost};
