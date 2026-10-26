//! Native projection of the existing Poet Tool-Chest model and intent contract.
//! Original portable sources retain their own copyright and licence unchanged.
#[allow(dead_code, unused_imports)]
#[path = "../../../../wasm/src/tool_chest/core/mod.rs"]
pub mod core;
mod intents;
mod model;
pub use intents::{NativeDispatch, NativeIntentBus};
pub use model::WorkspaceProjection;
#[cfg(test)]
mod tests;
