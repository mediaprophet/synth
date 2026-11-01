//! Synth is a separate agent-driven production consumer. No Poet shell dependency.
pub mod agent;
#[cfg(feature = "audio-device")]
mod audio;
mod corpus_policy;
mod host;
#[cfg(feature = "native-ui")]
pub mod native_ui;
pub use host::{
    HostSnapshot, NativeContext, NativeOutputRoot, SavedCandidate, SynthHost, TransportCommand,
};
