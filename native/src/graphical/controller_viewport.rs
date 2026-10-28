//! Read-only viewport projection under the same admitted canonical source owner.
use super::*;
use crate::graphical::{viewport::ViewportCommand, viewport_contract::ViewportOutput};
use std::sync::atomic::{AtomicBool, Ordering};
impl CanonicalEditorHost {
    pub(super) fn viewport_native(&mut self, request: ViewportCommand, expected: SourceRef,
        cancelled: &AtomicBool) -> Result<ViewportOutput, String> {
        crate::graphical::viewport_contract::validate_command(&request)?;
        self.expected(&expected)?;
        if self.reconciliation_required || !self.execution_epoch.is_current()
            || cancelled.load(Ordering::Acquire) {
            return Err("Rebuild current execution before interacting with the viewport".into());
        }
        let fallback;
        let empty = Value::Null;
        let value = if !matches!(&request, ViewportCommand::RestoreSourceGeometry) && self.preview.viewport_cached(&expected) { &empty }
        else if let Some(parts) = self.session.value("poet_parts") { parts }
        else {
            let geometry = self.session.value("poet_geometry").ok_or("No current authored scene")?;
            let material = self.session.value("poet_material").ok_or("No current authored material")?;
            fallback = Value::Record(std::collections::BTreeMap::from([
                ("geometry".into(), geometry.clone()), ("material".into(), material.clone())]));
            &fallback
        };
        let mut output = self.preview.viewport(request, expected.clone(), value,
            self.surface_resources.as_ref(), cancelled)?;
        expected.check_current()?;
        if output.frame.source != expected || output.state.source != expected
            || !self.execution_epoch.is_current() || cancelled.load(Ordering::Acquire) {
            return Err("Viewport result changed context; derived output withheld".into());
        }
        output.frame.session_revision = Some(self.session.revision());
        output.frame.execution_epoch = Some(self.execution_epoch.clone());
        output.state.execution_epoch = Some(self.execution_epoch.clone());
        Ok(output)
    }
}
