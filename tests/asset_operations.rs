pub use qualia_core_db::{
    render::compile_10d::decode_10d_mesh,
    specialized_libs::computer_vision::cv::codecs::png_decode::decode_png,
};
pub use std::sync::atomic::{AtomicBool, Ordering};
pub use synth::production::{bundle_read_limits, ProducerRegistry, ProductionLimits, Resource};

pub const EXAMPLE: &str = include_str!("../examples/production.vibe");
pub const GRANT: &str = "requires [capability(\"capability.invoke\")];";

#[path = "asset_operations_suites/waves_0_to_3.rs"]
mod waves_0_to_3;

#[path = "asset_operations_suites/waves_4_to_7.rs"]
mod waves_4_to_7;

#[path = "asset_operations_suites/waves_8_to_10.rs"]
mod waves_8_to_10;

#[path = "asset_operations_suites/waves_12_and_13.rs"]
mod waves_12_and_13;
