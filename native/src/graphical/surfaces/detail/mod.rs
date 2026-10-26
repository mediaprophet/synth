//! Cold source-replayed texture editing. Canonical Core owns pixels and PNG.
mod recipe;
mod render;
pub use recipe::{
    required_paint_version, validate_effects, DetailCost, DetailLimits, DetailRecipe,
    LayerBlendMode, LayerMask, MaskChannel, PaintLayer, PaintRecipe, PaintStroke, ProceduralRecipe,
    TextureEffect, TextureEffectStep,
};
pub use render::{
    render_detail, render_detail_with_masks, DetailBase, DetailMaskBase, DetailOutput,
};
#[cfg(test)]
mod tests;

#[cfg(test)]
mod completed_tests;
#[cfg(test)]
mod effect_tests;
