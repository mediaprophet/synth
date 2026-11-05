//! Clean-room Poet domain engines (PA-03..PA-09, IP-03 C1-C8, AUTH-H1..H6).
//!
//! Shared across native graphical (Aether/Blitz), headless CLI, and WASM host bridges.

pub mod sheet;

pub mod document {
    pub mod annotations;
}

pub mod deck {
    pub mod storyboard;
}

pub mod vector {
    pub mod boolean_ops;
    pub mod path_offset;
}

pub mod raster {
    pub mod raw_to_pbr;
}

pub mod audio {
    pub mod spectral_session;
}

pub mod timeline {
    pub mod compositor_pass;
    pub mod keyframe_graph;
}

pub mod cooperative {
    pub mod cooperative_c1_c8;
}

pub mod derivation {
    pub mod error_metrics;
    pub mod profile_compiler;
}

pub mod verification {
    pub mod completion_gate;
    pub mod consumer_loader_seam;
    pub mod obligation_gate;
}
