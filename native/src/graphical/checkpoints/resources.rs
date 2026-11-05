//! Native working-copy persistence; no resource type is silently discarded.
use super::*;
use qualia_core_db::render::compile_10d::CompiledAsset;

#[path = "resource_verifier.rs"]
pub mod resource_verifier;
pub use resource_verifier::{
    verify_atomic_resource_batch, AtomicResourceVerificationReport,
};

/// Selected by trusted host configuration, never inferred from incoming HMC.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PackageWritePolicy {
    WorkingCopy,
    ImmutableRelease { filename: String, digest: [u8; 32] },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackageInventory {
    pub digest: [u8; 32],
    pub resources: Vec<ResourceSummary>,
}

impl CheckpointService {
    fn check_writable_target(
        &self,
        filename: &str,
        expected_file: ExpectedFile,
    ) -> Result<(), String> {
        if let PackageWritePolicy::ImmutableRelease {
            filename: origin,
            digest,
        } = &self.policy
        {
            if origin.eq_ignore_ascii_case(filename)
                || matches!(expected_file, ExpectedFile::Sha256(expected) if expected == *digest)
            {
                return Err("immutable publication/release cannot be overwritten; choose a new working-copy filename".into());
            }
        }
        Ok(())
    }

    /// Current native owner calls this before reading a selected import file.
    /// Final save repeats every guard; this does not reserve or mutate a file.
    pub fn preflight_resource_edit(
        &self,
        filename: &str,
        expected_file: ExpectedFile,
    ) -> Result<(), String> {
        let ExpectedFile::Sha256(expected) = expected_file else {
            return Err("resource edits require the expected existing package digest".into());
        };
        self.path(filename)?;
        self.check_writable_target(filename, expected_file)?;
        let inventory = self.inventory(filename)?;
        if inventory.digest != expected {
            return Err("resource edit package digest changed; no import read".into());
        }
        if self
            .retained
            .lock()
            .map_err(|_| "checkpoint resource carrier poisoned")?
            .as_ref()
            .is_some_and(|base| base.origin_digest() != expected)
        {
            return Err("resource edit differs from retained package base; no import read".into());
        }
        Ok(())
    }

    pub(super) fn check_release_input(
        &self,
        filename: &str,
        digest: [u8; 32],
    ) -> Result<(), String> {
        if let PackageWritePolicy::ImmutableRelease {
            filename: origin,
            digest: pinned,
        } = &self.policy
        {
            if origin.eq_ignore_ascii_case(filename) && *pinned != digest {
                return Err(
                    "immutable release bytes differ from the native host's pinned digest".into(),
                );
            }
        }
        Ok(())
    }

    /// Caller holds a current ReadSource owner guard for the bound document.
    /// Listing neither activates source nor decodes arbitrary resources in UI.
    pub fn inventory(&self, filename: &str) -> Result<PackageInventory, String> {
        let checkpoint = core::load(&self.path(filename)?, self.limits)?;
        self.matches(checkpoint.artifact())?;
        let digest = checkpoint.file_digest().ok_or("checkpoint digest absent")?;
        self.check_release_input(filename, digest)?;
        Ok(PackageInventory {
            digest,
            resources: checkpoint.inventory(),
        })
    }

    /// Pre-commit validation of an edit batch against caller limits without disk or carrier mutation.
    pub fn verify_resource_edits(
        &self,
        edits: &[ResourceEdit],
    ) -> Result<AtomicResourceVerificationReport, String> {
        verify_atomic_resource_batch(edits, self.limits.file_bytes)
    }

    pub fn save(
        &self,
        expected: &StoreStamp,
        filename: &str,
        expected_file: ExpectedFile,
        parent_digest: Option<[u8; 32]>,
        assets: Vec<(String, CompiledAsset)>,
    ) -> Result<SavedCheckpoint, String> {
        self.save_with_resource_edits(
            expected,
            filename,
            expected_file,
            parent_digest,
            assets,
            vec![],
        )
    }

    /// Current native owner must retain its SaveCheckpoint/EditSource guard
    /// through this entire call. Source and resources install as one HMC rename;
    /// errors before install leave disk and the retained carrier unchanged.
    /// Core IO documents the portable external-writer and power-loss limits.
    pub fn save_with_resource_edits(
        &self,
        expected: &StoreStamp,
        filename: &str,
        expected_file: ExpectedFile,
        parent_digest: Option<[u8; 32]>,
        assets: Vec<(String, CompiledAsset)>,
        edits: Vec<ResourceEdit>,
    ) -> Result<SavedCheckpoint, String> {
        self.save_with_retained_inputs(
            expected,
            filename,
            expected_file,
            parent_digest,
            assets,
            edits,
            vec![],
        )
    }

    /// Exact inert compiler-input retention. These records come from the
    /// authorized immutable snapshot owner, never a caller resource-edit request.
    /// Existing records must agree in payload, kind AND opaque metadata.
    pub(crate) fn merge_retained_inputs(
        checkpoint: core::Checkpoint,
        inputs: Vec<core::ResourceData>,
        limits: core::CheckpointLimits,
    ) -> Result<core::Checkpoint, String> {
        if inputs.is_empty() {
            return Ok(checkpoint);
        }
        if inputs.len() > limits.entries {
            return Err("Retained compiler input count exceeds checkpoint budget".into());
        }
        let mut additions = std::collections::BTreeMap::<String, core::ResourceData>::new();
        for input in inputs {
            if let Some(old) = checkpoint.resources().iter().find(|r| r.key == input.key) {
                if old != &input {
                    return Err(
                        "Retained compiler input conflicts with exact existing resource/metadata"
                            .into(),
                    );
                }
                continue;
            }
            if let Some(old) = additions.get(&input.key) {
                if old != &input {
                    return Err("Conflicting retained compiler input declarations".into());
                }
                continue;
            }
            additions.insert(input.key.clone(), input);
        }
        // Core checks reserved source keys, aggregate bytes/entries, graph
        // validity and dependency closure before cloning the detached carrier.
        checkpoint.apply_resource_edits(
            additions.into_values().map(ResourceEdit::Add).collect(),
            limits,
        )
    }

    /// Trusted source compilation may retain verified bound inputs when creating
    /// a fresh package. Ordinary resource edits still require an existing digest.
    pub(crate) fn save_with_retained_inputs(
        &self,
        expected: &StoreStamp,
        filename: &str,
        expected_file: ExpectedFile,
        parent_digest: Option<[u8; 32]>,
        assets: Vec<(String, CompiledAsset)>,
        edits: Vec<ResourceEdit>,
        inputs: Vec<core::ResourceData>,
    ) -> Result<SavedCheckpoint, String> {
        self.save_with_source_carrier(
            expected,
            filename,
            expected_file,
            parent_digest,
            assets,
            edits,
            inputs,
            None,
        )
    }

    /// Exact original package transport is separate from current accepted source.
    /// The current native owner guard remains held through final source capture,
    /// all inert carrier admission, the disk SHA guard and atomic installation.
    pub(crate) fn save_with_source_carrier(
        &self,
        expected: &StoreStamp,
        filename: &str,
        expected_file: ExpectedFile,
        parent_digest: Option<[u8; 32]>,
        assets: Vec<(String, CompiledAsset)>,
        edits: Vec<ResourceEdit>,
        inputs: Vec<core::ResourceData>,
        source_carrier: Option<&core::VerifiedSourceCarrier>,
    ) -> Result<SavedCheckpoint, String> {
        let path = self.path(filename)?;
        if !edits.is_empty() && !matches!(expected_file, ExpectedFile::Sha256(_)) {
            return Err("resource edits require the expected existing package digest".into());
        }
        self.check_writable_target(filename, expected_file)?;
        verify_atomic_resource_batch(&edits, self.limits.file_bytes)?;
        let mut retained = self
            .retained
            .lock()
            .map_err(|_| "checkpoint resource carrier poisoned")?;
        let disk_base = match expected_file {
            ExpectedFile::Absent => None,
            ExpectedFile::Sha256(digest) => {
                let existing = core::load_source_carrier(&path, self.limits)?;
                self.matches(existing.checkpoint().artifact())?;
                if existing.origin_digest() != digest {
                    return Err(
                        "checkpoint replacement digest changed before source capture".into(),
                    );
                }
                if retained
                    .as_ref()
                    .is_some_and(|base| base.origin_digest() != digest)
                {
                    return Err("selected package differs from retained resource base; reopen the intended base explicitly".into());
                }
                Some(existing)
            }
        };
        let release_base = if disk_base.is_none() && retained.is_none() {
            match &self.policy {
                PackageWritePolicy::ImmutableRelease {
                    filename: origin,
                    digest,
                } => {
                    let input = core::load_source_carrier(&self.path(origin)?, self.limits)?;
                    self.matches(input.checkpoint().artifact())?;
                    if input.origin_digest() != *digest {
                        return Err(
                            "immutable release base differs from native pinned digest".into()
                        );
                    }
                    Some(input)
                }
                PackageWritePolicy::WorkingCopy => None,
            }
        } else {
            None
        };
        let base = disk_base
            .as_ref()
            .or(retained.as_ref())
            .or(release_base.as_ref());
        let mut checkpoint = core::capture(&self.binding.source_id, expected, self.limits)?;
        self.matches(checkpoint.artifact())?;
        let source_revision = checkpoint.artifact().lamport_clock;
        if let Some(base) = base {
            let origin = base.origin_digest();
            if parent_digest.is_some_and(|parent| parent != origin) {
                return Err("checkpoint parent differs from exact retained resource base".into());
            }
            checkpoint = checkpoint
                .with_parent_digest(origin)
                .with_source_carrier(base, self.limits)?;
        } else if source_carrier.is_none() && parent_digest.is_some() {
            return Err("checkpoint lineage requires a verified retained package base".into());
        }
        if let Some(input) = source_carrier {
            if base.is_none() {
                let origin = input.origin_digest();
                if parent_digest.is_some_and(|parent| parent != origin) {
                    return Err(
                        "checkpoint parent differs from verified compiler input base".into(),
                    );
                }
                checkpoint = checkpoint.with_parent_digest(origin);
            }
            checkpoint = checkpoint.with_source_carrier(input, self.limits)?;
        }
        let checkpoint = Self::merge_retained_inputs(checkpoint, inputs, self.limits)?;
        let checkpoint = checkpoint.apply_resource_edits(edits, self.limits)?;
        // Core writer rejects collisions with retained entries. Replacement is
        // available only through an explicit guarded ResourceEdit, never by an
        // incidental compiler output sharing an old entry's key.
        let bytes = core::encode(&checkpoint, assets, self.limits)?;
        let next_carrier = core::decode_source_carrier(&bytes, self.limits)?;
        if !studio_store::artifact_stamp_is_current(&self.binding.source_id, expected) {
            return Err("native source changed before checkpoint write; no write attempted".into());
        }
        if self.path(filename)? != path {
            return Err("checkpoint path changed".into());
        }
        let receipt = core::save(&path, &bytes, expected_file, self.limits)?;
        *retained = Some(next_carrier);
        Ok(SavedCheckpoint {
            receipt,
            source_revision,
        })
    }
}
