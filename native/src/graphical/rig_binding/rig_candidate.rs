//! Disposable rig-control candidate for the native source owner.
//!
//! A candidate is neither an authorization nor a source commit. The caller
//! must resolve the exact selected HMC resource under current project rights,
//! bind the actual compiled part/span, and promote this literal through the
//! existing expected-source authoring route. Re-admit a fresh preview only
//! after the source owner has accepted the edit.
use super::{
    admit_bound_rig, apply_control, descriptor_source_literal, BindMesh, RigControl,
    RigSourceDescriptor, ViewportSpan,
};
use crate::graphical::preview::SourceRef;

/// Compute a candidate without mutating the retained viewport player.
/// All source, resource, mesh, lineage, span and cold-budget checks in the
/// normal rig admission apply. `expected` must be checked again by the source
/// owner when it promotes the result; this function does not grant authority.
pub struct StagedRigControl {
    expected: SourceRef,
    before: RigSourceDescriptor,
    after: RigSourceDescriptor,
    literal: String,
}

impl StagedRigControl {
    pub(crate) fn expected(&self) -> &SourceRef { &self.expected }
    pub(crate) fn before(&self) -> &RigSourceDescriptor { &self.before }
    pub fn after(&self) -> &RigSourceDescriptor { &self.after }
    pub(crate) fn literal(&self) -> &str { &self.literal }
}

pub fn stage_rig_control(
    expected: SourceRef,
    descriptor: RigSourceDescriptor,
    selected_resource_key: &str,
    resource_bytes: &[u8],
    mesh: BindMesh<'_>,
    span: ViewportSpan,
    resident_budget: usize,
    action: RigControl,
) -> Result<StagedRigControl, String> {
    let before = descriptor.clone();
    let mut candidate = admit_bound_rig(
        expected.clone(), descriptor, selected_resource_key, resource_bytes, mesh, span,
        resident_budget,
    )?;
    let descriptor = apply_control(&mut candidate, action)?.clone();
    let literal = descriptor_source_literal(&descriptor)?;
    expected.check_current()?;
    Ok(StagedRigControl { expected, before, after: descriptor, literal })
}

/// Stage an already viewport-admitted, disposable rig without altering retained state.
pub(crate) fn stage_admitted(mut rig: super::BoundRig, action: RigControl) -> Result<StagedRigControl,String> {
    let expected = rig.source().clone();
    let before = rig.descriptor().clone();
    let after = apply_control(&mut rig, action)?.clone();
    let literal = descriptor_source_literal(&after)?;
    expected.check_current()?;
    Ok(StagedRigControl { expected, before, after, literal })
}
