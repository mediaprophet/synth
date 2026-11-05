//! Additive native Poet Tool-Chest/Manifold source authoring surface.
pub mod animated_rig_preview;
pub mod assembly;
pub mod numeric_sheet;
pub mod terrain_brush;
pub mod authoring;
pub mod botanical_graph_host;
pub mod checkpoints;
pub mod controller;
pub mod cooperation;
mod cooperation_selection;
mod cooperative_controls;
pub mod cooperative_workbench;
pub mod cooperative_workflow;
pub mod draft_buffer;
pub mod entry;
pub mod geometry_host;
mod jobs;
mod material_controls;
mod material_edit;
pub mod materials;
pub mod mcp;
pub mod native_authoring_profile;
mod native_dispatch;
pub mod native_entry;
pub mod native_project;
pub mod package;
mod package_batch_controls;
mod package_controls;
mod timeline_image_preview;
mod timeline_gif_preview;
mod timeline_audio_preview;
mod timeline_audio_output;
mod timeline_opus_output;
mod timeline_audio_mix;
mod timeline_sound_field;
mod timeline_wav_device;
pub(crate) mod timeline_resource_selection;
mod package_review_controls;
pub mod package_workflow;
pub mod poet_workspace;
pub mod preview;
pub mod rig_binding;
pub mod session_host;
pub mod shared_host;
pub mod surface_host;
pub mod surfaces;
mod ui;
pub mod viewport;
pub mod viewport_contract;
mod viewport_controls;
mod viewport_input;
pub mod viewport_provider;

use controller::EditorHost;

/// Main-thread Aether/Blitz native entry. Host grants are supplied by the caller.
/// Closing the window stops queued admission, then joins the one native worker.
pub fn launch(host: Box<dyn EditorHost>, initial_draft: String) -> Result<(), String> {
    let initial = host.snapshot()?;
    let (worker, host) = jobs::Worker::start(host)?;
    let attributes = dioxus_native::WindowAttributes::default()
        .with_title("Poet · Studio")
        .with_surface_size(dioxus_native::LogicalSize::new(1440, 900));
    dioxus_native::launch_cfg_with_props(
        ui::NativeStudio,
        ui::NativeStudioProps {
            host,
            initial,
            initial_draft,
            draft_buffer: None,
            shared_busy: None,
        },
        Vec::new(),
        vec![Box::new(attributes)],
    );
    worker.finish()
}

#[cfg(test)]
mod reference_authoring_tests;

#[cfg(test)]
mod biological_reference_tests;

mod lookdev_controls;

mod numeric_sheet_controls;
#[cfg(test)]
mod sheet_formula_host_tests;
#[cfg(test)]
mod sheet_set_cell_host_tests;
#[cfg(test)]
#[path = "hard_cap_sweep_public_tests.rs"]
mod hard_cap_sweep_public_tests;

#[cfg(test)]
#[path = "polygon_soup_public_tests.rs"]
mod polygon_soup_public_tests;

#[cfg(test)]
#[path = "indexed_tetra_public_tests.rs"]
mod indexed_tetra_public_tests;

#[cfg(test)]
#[path = "surface_topology_public_tests.rs"]
mod surface_topology_public_tests;

#[cfg(test)]
#[path = "tapered_branch_field_public_tests.rs"]
mod tapered_branch_field_public_tests;

pub(crate) mod fork_adopted_adapter;

pub(crate) mod junction_generate;

pub(crate) mod junction_budget;
