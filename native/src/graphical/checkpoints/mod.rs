//! Native checkpoint service. The cooperative EditorHost must hold its current
//! trusted project/document authority guard across every call. This service
//! checks source/path consistency; it never derives authority from a file.
use qualia_core_db::poet_host::invoke::studio_store::{
    self, checkpoint as core, StoreStamp, StudioArtifact,
};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

mod editor;
pub use editor::CheckpointEditorHost;
pub mod authorised_reopen;
mod resources;
mod surface_resources;
mod timeline_media_resource;
pub use timeline_media_resource::{SelectedMedia, TimelineMediaRequest, TimelineMediaSelection};
pub use resources::{PackageInventory, PackageWritePolicy};
mod georaster_edit;
mod inspection;
pub mod typed_10d;
pub use georaster_edit::{
    ui_georaster_edit_limits, GeoRasterBrushRequest, GeoRasterCreateRequest, GeoRasterEditLimits, GeoRasterEditRequest,
    SourceAnnotation, SourceDeclaration, SourceQualification, SourceStatus,
};
pub use inspection::{
    ui_inspection_limits, M64PayloadInspection, ResourceInspection, ResourceInspectionRequest,
    UI_INSPECTION_CHUNKS,
};

pub use core::{
    CheckpointLimits, ExpectedFile, ResourceData, ResourceEdit, ResourceSummary, SaveReceipt,
};

/// Supplied by the current trusted native owner, never deserialized from HMC.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TrustedBinding {
    pub project_id: String,
    pub document_id: String,
    pub source_id: String,
    pub domain: String,
}

pub struct CheckpointService {
    configured_root: PathBuf,
    canonical_root: PathBuf,
    binding: TrustedBinding,
    limits: CheckpointLimits,
    policy: PackageWritePolicy,
    retained: Mutex<Option<core::VerifiedSourceCarrier>>,
}

pub struct PreparedReopen {
    checkpoint: core::VerifiedSourceCarrier,
    binding: TrustedBinding,
    filename: String,
    digest: [u8; 32],
}
impl PreparedReopen {
    pub fn artifact(&self) -> &StudioArtifact {
        self.checkpoint.checkpoint().artifact()
    }
    pub fn file_digest(&self) -> [u8; 32] {
        self.digest
    }
    pub fn parent_digest(&self) -> Option<[u8; 32]> {
        self.checkpoint.checkpoint().parent_digest()
    }
    pub fn resources(&self) -> Vec<ResourceSummary> {
        self.checkpoint.checkpoint().inventory()
    }
}

pub struct SavedCheckpoint {
    pub receipt: SaveReceipt,
    pub source_revision: u32,
}

impl CheckpointService {
    /// Root comes from native host configuration, not an MCP/source path.
    /// Construction checks consistency only; the caller supplies authority.
    pub fn new(
        root: &Path,
        binding: TrustedBinding,
        limits: CheckpointLimits,
    ) -> Result<Self, String> {
        Self::new_with_policy(root, binding, limits, PackageWritePolicy::WorkingCopy)
    }

    /// The current native host selects release status; file contents cannot
    /// grant edit authority or self-declare that a publication is mutable.
    pub fn new_with_policy(
        root: &Path,
        binding: TrustedBinding,
        limits: CheckpointLimits,
        policy: PackageWritePolicy,
    ) -> Result<Self, String> {
        if [
            &binding.project_id,
            &binding.document_id,
            &binding.source_id,
            &binding.domain,
        ]
        .iter()
        .any(|id| id.is_empty() || id.as_bytes().contains(&0))
        {
            return Err(
                "trusted checkpoint association contains invalid identifiers/domain".into(),
            );
        }
        let canonical_root = root.canonicalize().map_err(|e| e.to_string())?;
        if !canonical_root.is_dir() {
            return Err("native checkpoint root must be an existing host-owned directory".into());
        }
        let service = Self {
            configured_root: root.to_path_buf(),
            canonical_root,
            binding,
            limits,
            policy,
            retained: Mutex::new(None),
        };
        if let PackageWritePolicy::ImmutableRelease { filename, .. } = &service.policy {
            service.path(filename)?;
        }
        Ok(service)
    }

    pub fn binding(&self) -> &TrustedBinding {
        &self.binding
    }

    fn path(&self, filename: &str) -> Result<PathBuf, String> {
        if filename.len() > 240
            || !filename.to_ascii_lowercase().ends_with(".hmc")
            || filename.starts_with('.')
            || filename.len() <= 4
            || !filename
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.'))
        {
            return Err(
                "choose one plain .hmc filename within the host checkpoint directory".into(),
            );
        }
        let stem = filename
            .split('.')
            .next()
            .unwrap_or("")
            .to_ascii_uppercase();
        if matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL")
            || (stem.len() == 4
                && (stem.starts_with("COM") || stem.starts_with("LPT"))
                && matches!(stem.as_bytes()[3], b'1'..=b'9'))
        {
            return Err("checkpoint filename is an OS device name".into());
        }
        // Recheck root resolution on every operation. An incoming file never
        // selects a directory. The host owns this root and its ancestors; a
        // malicious outside filesystem mutation cannot be made portable-atomic.
        if self
            .configured_root
            .canonicalize()
            .map_err(|e| e.to_string())?
            != self.canonical_root
            || self
                .canonical_root
                .canonicalize()
                .map_err(|e| e.to_string())?
                != self.canonical_root
        {
            return Err("native checkpoint root resolution changed".into());
        }
        let path = self.canonical_root.join(filename);
        if std::fs::symlink_metadata(&path)
            .is_ok_and(|m| m.file_type().is_symlink() || !m.is_file())
        {
            return Err(
                "checkpoint target must be a regular file, never a link/junction/directory".into(),
            );
        }
        Ok(path)
    }

    fn matches(&self, artifact: &StudioArtifact) -> Result<(), String> {
        if artifact.id != self.binding.source_id || artifact.domain != self.binding.domain {
            return Err("checkpoint source identifiers/domain do not match the trusted native document association".into());
        }
        Ok(())
    }

    /// Caller holds the live ReopenCheckpoint owner guard. Preparation reads
    /// and validates source/history/resources only. It cannot activate or replay.
    pub fn prepare_reopen(&self, filename: &str) -> Result<PreparedReopen, String> {
        let checkpoint = core::load_source_carrier(&self.path(filename)?, self.limits)?;
        self.matches(checkpoint.checkpoint().artifact())?;
        let digest = checkpoint.origin_digest();
        self.check_release_input(filename, digest)?;
        Ok(PreparedReopen {
            checkpoint,
            binding: self.binding.clone(),
            filename: filename.into(),
            digest,
        })
    }

    /// Caller rechecks its current ReopenCheckpoint owner decision for the
    /// prepared source/revision/digest and retains that guard through activation.
    /// Existing accepted source is never replaced. A fresh interpreter opens
    /// held; source-triggered grants and arbitrary effect replay are excluded.
    pub fn activate(
        &self,
        prepared: PreparedReopen,
    ) -> Result<(StudioArtifact, StoreStamp), String> {
        if prepared.binding != self.binding {
            return Err(
                "prepared checkpoint belongs to another trusted document association".into(),
            );
        }
        self.matches(prepared.checkpoint.checkpoint().artifact())?;
        // A stale disk selection cannot be silently activated after review.
        let current = core::load(&self.path(&prepared.filename)?, self.limits)?;
        if current.file_digest() != Some(prepared.digest) {
            return Err("checkpoint bytes changed since preparation; no activation".into());
        }
        self.check_release_input(&prepared.filename, prepared.digest)?;
        // Lock before activation, so no fallible lock step follows source
        // installation. Resources remain inert and bound to this live service.
        let mut retained = self
            .retained
            .lock()
            .map_err(|_| "checkpoint resource carrier poisoned")?;
        let carrier = prepared.checkpoint;
        let result = core::activate(carrier.checkpoint().clone(), self.limits)?;
        *retained = Some(carrier);
        Ok(result)
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod restart_tests;

#[cfg(test)]
mod resource_preservation_tests;
