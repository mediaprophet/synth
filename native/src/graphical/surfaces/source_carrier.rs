//! Immutable snapshot transport only; Core owns source/HMC schemas and merging.
use super::{SurfaceTextureLimits, TextureResources};
use qualia_core_db::poet_host::invoke::studio_store::checkpoint as core;

impl TextureResources {
    /// Called only through the native owner's current source/permission guard.
    /// Decoding is inert: neither source activation nor evaluation occurs here.
    pub(crate) fn retained_carrier(
        &self,
        limits: core::CheckpointLimits,
    ) -> Result<Option<core::VerifiedSourceCarrier>, String> {
        decode_carrier(&self.bytes, self.limits, limits)
    }
}

fn decode_carrier(
    bytes: &[u8],
    surface: SurfaceTextureLimits,
    limits: core::CheckpointLimits,
) -> Result<Option<core::VerifiedSourceCarrier>, String> {
    // The immutable input, decoded resources/metadata and canonical parser
    // scratch are admitted conservatively as a constant multiple. This is a
    // cold copy budget, not a measured peak-process-memory claim.
    if bytes.len() > surface.package_bytes
        || bytes
            .len()
            .checked_mul(4)
            .is_none_or(|n| n > surface.working_bytes)
    {
        return Err("Retained source carrier exceeds surface copy admission".into());
    }
    if bytes.len() > limits.file_bytes {
        return Err("Retained source carrier exceeds checkpoint file admission".into());
    }
    let reader = qualia_core_db::bundle::BundleReader::parse(bytes).map_err(|e| e.to_string())?;
    if reader.entries().len() > limits.entries {
        return Err("Retained source carrier exceeds checkpoint entry admission".into());
    }
    // Preserve source-free texture package compatibility. Ordinary retained
    // inputs still run their existing flags/hash/copy-budget checks separately.
    if reader.entry(core::SOURCE_RESOURCE_KEY).is_none() {
        return Ok(None);
    }
    drop(reader);
    core::decode_source_carrier(bytes, limits).map(Some)
}
