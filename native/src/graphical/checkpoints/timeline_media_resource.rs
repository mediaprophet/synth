//! Selected, bounded HMC media bytes under the existing checkpoint owner.
//! A caller must establish clip-to-resource association separately; a key or
//! digest is not authority, provenance, a codec qualification, or an IRI match.
use super::*;
use crate::graphical::preview::SourceRef;
use std::sync::atomic::{AtomicBool, Ordering};

/// Immutable, digest-checked source bytes. No decoding or playback is implied.
pub struct SelectedMedia {
    pub kind: String,
    pub bytes: Vec<u8>,
}

/// Explicit caller-selected resource; no IRI-to-key equivalence is inferred.
#[derive(Clone, Debug)]
pub struct TimelineMediaRequest {
    pub clip_id: String,
    pub timeline_tick: u64,
    pub resource: ResourceInspectionRequest,
}

/// The checked clip identity/time and explicitly selected immutable bytes.
pub struct TimelineMediaSelection {
    pub clip_source_identifier: String,
    pub media_tick: u64,
    pub remaining_ticks: u64,
    pub selected: SelectedMedia,
}

impl CheckpointService {
    /// The caller must retain the current native ReadSource owner guard and
    /// resolve the selected clip's identifier to this explicit request under
    /// the same current source revision. This function validates the selection,
    /// not the semantic association between the clip identifier and HMC key.
    pub(crate) fn read_selected_media(
        &self,
        request: ResourceInspectionRequest,
        expected: &SourceRef,
        max_resource_bytes: usize,
        cancelled: &AtomicBool,
    ) -> Result<SelectedMedia, String> {
        let live = || if cancelled.load(Ordering::Acquire) { Err("Media resource read cancelled") } else { Ok(()) };
        live()?;
        request.validate_bounds()?;
        expected.check_current()?;
        if expected.artifact_id != self.binding.source_id { return Err("Media resource belongs to another source".into()); }
        if max_resource_bytes == 0 || max_resource_bytes > self.limits.file_bytes { return Err("Media resource byte budget is invalid".into()); }
        let (artifact, stamp) = studio_store::get_artifact_stamped(&expected.artifact_id).ok_or("Media resource source disappeared")?;
        if stamp != expected.stamp { return Err("Media resource source changed".into()); }
        let source_digest = crate::graphical::cooperation::source_digest(&artifact, crate::graphical::cooperation::GatewayLimits::default())?;
        let path = self.path(&request.filename)?;
        let checkpoint = core::load_cancellable(&path, self.limits, cancelled)?;
        live()?;
        self.matches(checkpoint.artifact())?;
        if crate::graphical::cooperation::source_digest(checkpoint.artifact(), crate::graphical::cooperation::GatewayLimits::default())? != source_digest {
            return Err("Selected package source differs from current source; save/reopen a coherent base first".into());
        }
        if checkpoint.file_digest() != Some(request.package_digest) { return Err("Media resource package changed; refresh inventory".into()); }
        self.check_release_input(&request.filename, request.package_digest)?;
        let selected = checkpoint.resources().iter().find(|r| r.key == request.key).ok_or("Selected media resource is absent")?;
        if selected.summary().digest != request.resource_digest { return Err("Media resource digest changed; refresh inventory".into()); }
        if !(selected.kind.starts_with("image/") || selected.kind.starts_with("audio/") || selected.kind.starts_with("video/")) {
            return Err("Selected resource is not declared as image, audio, or video".into());
        }
        if selected.kind.len() > 128 || selected.bytes.len() > max_resource_bytes { return Err("Media resource exceeds caller profile".into()); }
        let mut bytes = Vec::new();
        bytes.try_reserve_exact(selected.bytes.len()).map_err(|_| "Media allocation refused")?;
        bytes.extend_from_slice(&selected.bytes);
        let kind = selected.kind.clone();
        drop(checkpoint);
        expected.check_current()?;
        let current = core::load_cancellable(&self.path(&request.filename)?, self.limits, cancelled)?;
        live()?;
        self.matches(current.artifact())?;
        self.check_release_input(&request.filename, request.package_digest)?;
        if current.file_digest() != Some(request.package_digest) { return Err("Media package changed during read; result withheld".into()); }
        expected.check_current()?;
        live()?;
        Ok(SelectedMedia { kind, bytes })
    }
}

#[cfg(test)]
#[path = "timeline_media_resource_tests.rs"]
mod tests;
