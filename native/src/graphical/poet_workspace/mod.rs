//! Native projection of the existing Poet Tool-Chest model and intent contract.
//! Original portable sources retain their own copyright and licence unchanged.
#[allow(dead_code, unused_imports)]
#[path = "../../../../wasm/src/tool_chest/core/mod.rs"]
pub mod core;
pub mod layout;
#[path = "../../../../wasm/src/browser/material_upload.rs"]
pub mod material_upload;
pub mod viewport_mount;
mod intents;
mod model;

pub use intents::{NativeDispatch, NativeIntentBus, NumericSheetIntent};
pub use layout::{
    DockRegionSpec, HonestyBadge, ManifoldLensTab, ToolChestSection, UnifiedWorkspaceLayout,
};
pub use model::WorkspaceProjection;
pub use viewport_mount::{
    mount_16x9_portal_viewport, AspectFramingPolicy, ViewportMountContract, ViewportMountError,
    ViewportRenderReceipt,
};

#[cfg(test)]
mod tests;
