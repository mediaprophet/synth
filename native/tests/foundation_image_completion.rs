//! Canonical Core APIs, through Poet's actual dependency closure.
#[path = "../../../qualiaDB/crates/qualia-core-db/tests/image_filters_completed.rs"]
mod completed_filters;
#[path = "../../../qualiaDB/crates/qualia-core-db/tests/image_composite_completed.rs"]
mod completed_composition;

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use qualia_core_db::hypermedia_authoring::image::*;

struct CountedAllocator;
thread_local! {
    static MEASURING: Cell<bool> = const { Cell::new(false) };
    static ALLOCATIONS: Cell<usize> = const { Cell::new(0) };
}
fn count_allocation() {
    if MEASURING.try_with(Cell::get).unwrap_or(false) {
        let _ = ALLOCATIONS.try_with(|n| n.set(n.get() + 1));
    }
}
unsafe impl GlobalAlloc for CountedAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        count_allocation();
        unsafe { System.alloc(layout) }
    }
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        count_allocation();
        unsafe { System.alloc_zeroed(layout) }
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) }
    }
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        count_allocation();
        unsafe { System.realloc(ptr, layout, size) }
    }
}
#[global_allocator]
static ALLOCATOR: CountedAllocator = CountedAllocator;

fn measured<T>(operation: impl FnOnce() -> T) -> (T, usize) {
    ALLOCATIONS.with(|n| n.set(0));
    MEASURING.with(|n| n.set(true));
    let result = operation();
    MEASURING.with(|n| n.set(false));
    (result, ALLOCATIONS.with(Cell::get))
}

#[test]
fn image_completed_kernels_use_no_heap_with_caller_buffers() {
    let source = [[110, 70, 30, 180]; 9];
    let mut output = [[0; 4]; 9];
    let mut scratch = [[0.0; 4]; 18];
    for op in [ImageFilterOp::GaussianBlur { sigma: 1.0 },
        ImageFilterOp::Sharpen { sigma: 1.0, amount: 0.5 },
        ImageFilterOp::HueRotate { degrees: 90.0 },
        ImageFilterOp::Emboss { azimuth_degrees: 135.0, strength: 0.5 }] {
        let (result, allocations) = measured(|| filter_rgba8_into(
            &source, 3, 3, op, ImageLimits::default(), &mut output, &mut scratch));
        assert!(result.is_ok());
        assert_eq!(allocations, 0, "caller-buffer filter allocated");
    }
    let mut doc = ImageDocument::new("allocation-oracle", 3, 3);
    let base = doc.add_layer("base");
    base.fill([30, 90, 170, 128]);
    let top = doc.add_layer("top");
    top.fill([180, 100, 20, 180]);
    top.mask = Some(vec![128; 9]);
    top.offset_x = -1;
    for mode in [BlendMode::Normal, BlendMode::Multiply, BlendMode::Screen,
        BlendMode::Overlay, BlendMode::SoftLight, BlendMode::HardLight,
        BlendMode::ColorDodge, BlendMode::ColorBurn, BlendMode::Difference, BlendMode::Exclusion] {
        doc.layers[1].blend_mode = mode;
        let (result, allocations) = measured(|| doc.composite_into(&mut output, ImageLimits::default()));
        assert!(result.is_ok());
        assert_eq!(allocations, 0, "caller-buffer composition allocated");
    }
}

#[test]
fn image_completed_legacy_entry_points_produce_real_outputs() {
    for filter in [ImageFilter::GaussianBlur, ImageFilter::Sharpen, ImageFilter::HueRotate, ImageFilter::Emboss] {
        let mut layer = ImageLayer::new("legacy", "legacy", 3, 3);
        layer.fill([20, 80, 180, 255]);
        layer.set_pixel(1, 1, [220, 30, 10, 255]);
        let before = layer.pixels.clone();
        apply_filter(&mut layer, filter, 0.5);
        assert_ne!(layer.pixels, before, "legacy filter remained a no-op: {filter:?}");
    }
}
