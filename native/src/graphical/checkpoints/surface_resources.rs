//! Bounded immutable HMC selection inside the existing checkpoint owner.
use super::*;
use crate::graphical::{
    preview::SourceRef,
    surface_host::SurfacePackageRequest,
    surfaces::{SurfaceTextureLimits, TextureResources},
};
use std::io::Read;
impl CheckpointService {
    pub fn surface_resources(
        &self,
        request: &SurfacePackageRequest,
        expected: &SourceRef,
    ) -> Result<TextureResources, String> {
        request.validate()?;
        expected.check_current()?;
        if expected.artifact_id != self.binding.source_id {
            return Err("Foreign surface resource source".into());
        }
        let limits = SurfaceTextureLimits::default();
        let maximum = self.limits.file_bytes.min(limits.package_bytes);
        let file = std::fs::File::open(self.path(&request.filename)?).map_err(|e| e.to_string())?;
        let length = usize::try_from(file.metadata().map_err(|e| e.to_string())?.len())
            .map_err(|_| "HMC length overflow")?;
        if length > maximum {
            return Err("Surface HMC exceeds native snapshot budget".into());
        }
        let mut bytes = Vec::new();
        bytes
            .try_reserve_exact(length.checked_add(1).ok_or("Surface capacity overflow")?)
            .map_err(|_| "Surface allocation refused")?;
        file.take(maximum as u64 + 1)
            .read_to_end(&mut bytes)
            .map_err(|e| e.to_string())?;
        if bytes.len() > maximum {
            return Err("Surface HMC grew beyond snapshot budget".into());
        }
        let checkpoint = core::decode(&bytes, self.limits)?;
        self.matches(checkpoint.artifact())?;
        if checkpoint.file_digest() != Some(request.digest) {
            return Err("Surface HMC digest changed; refresh inventory".into());
        }
        self.check_release_input(&request.filename, request.digest)?;
        drop(checkpoint);
        let resources = TextureResources::from_hmc_bytes(&bytes, limits)?;
        // A reopened source history can reference masks absent from its current
        // recipe. Compare before even the first surface bind so those inputs
        // cannot disappear before undo/redo has been reconstructed.
        {
            let retained = self.retained.lock().map_err(|_| "checkpoint resource carrier poisoned")?;
            if let Some(carrier) = retained.as_ref() {
                resources.require_retained_images(carrier.checkpoint().resources())?;
            }
        }
        expected.check_current()?;
        Ok(resources)
    }
}
