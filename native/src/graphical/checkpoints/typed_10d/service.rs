//! Package/source selection guards and one canonical resource transaction.
use super::*;
use crate::graphical::cooperation::{source_digest, GatewayLimits};

impl CheckpointService {
    fn typed_source(&self, expected: &SourceRef) -> Result<[u8; 32], String> {
        if expected.artifact_id != self.binding.source_id {
            return Err("Typed .10d source association differs from native document".into());
        }
        expected.check_current()?;
        let (artifact, stamp) = studio_store::get_artifact_stamped(&self.binding.source_id)
            .ok_or("Typed .10d authored source is absent")?;
        if stamp != expected.stamp {
            return Err("Typed .10d source stamp changed".into());
        }
        source_digest(&artifact, GatewayLimits::default())
    }

    /// Caller holds current ReadSource permission for the trusted document.
    /// Digest equality guards association; it does not prove source reproduces
    /// this resource, its typed scientific interpretation, or a prior edit.
    pub fn inspect_typed_10d(
        &self,
        request: ResourceInspectionRequest,
        expected: &SourceRef,
        mut limits: Typed10dLimits,
    ) -> Result<Typed10dInspection, String> {
        request.validate_bounds()?;
        let current_source = self.typed_source(expected)?;
        limits.max_file_bytes = limits.max_file_bytes.min(self.limits.file_bytes);
        let checkpoint = core::load(&self.path(&request.filename)?, self.limits)?;
        self.matches(checkpoint.artifact())?;
        if checkpoint.file_digest() != Some(request.package_digest) {
            return Err("Typed .10d package digest changed; refresh inventory".into());
        }
        self.check_release_input(&request.filename, request.package_digest)?;
        if source_digest(checkpoint.artifact(), GatewayLimits::default())? != current_source {
            return Err("Saved package source differs from current authored source; save/reopen a coherent base first".into());
        }
        let selected = checkpoint
            .resources()
            .iter()
            .find(|r| r.key == request.key)
            .ok_or("Selected .10d resource absent")?;
        if selected.kind != "10d" || selected.summary().digest != request.resource_digest {
            return Err("Selected resource kind/digest changed or is not canonical 10d".into());
        }
        let payload = payload::inspect(&selected.bytes, limits)?;
        let skeletal = payload::inspect_skeletal(&selected.bytes);
        let mut edit_blocker = payload.blocker;
        if selected.meta.is_some() {
            edit_blocker = Some("Selected resource metadata has no qualified derivative contract; use authored source".into());
        }
        // Never pretend an unchanged companion is consistent with a new
        // compiled digest. Canonical source compilation owns that graph.
        if checkpoint.resources().iter().any(|r| r.kind == "q42") {
            edit_blocker = Some("Source-compiled .10d/Q42 pair requires a coherent authored-source rebuild; direct material replacement refused".into());
        }
        let saved_source_revision = checkpoint.artifact().lamport_clock;
        drop(checkpoint);
        let current = core::load(&self.path(&request.filename)?, self.limits)?;
        if current.file_digest() != Some(request.package_digest)
            || self.typed_source(expected)? != current_source
        {
            return Err("Typed .10d selection changed during inspection; result withheld".into());
        }
        Ok(Typed10dInspection {
            request,
            saved_source_revision,
            source_digest: current_source,
            version: payload.version,
            vertex_count: payload.vertices,
            index_count: payload.indices,
            materials: payload.materials,
            ranges: payload.ranges,
            fields: payload.fields,
            frames: payload.frames,
            skeletal,
            edit_blocker,
        })
    }

    /// Visual factors create a selected-resource derivative. The retained
    /// authored source is not represented as regenerating the edited resource.
    /// Outer authorized owner retains WritePackage permission across the call,
    /// records the operation's before/after package evidence and invalidates
    /// package-review/publication views. No authority comes from these bytes.
    pub fn edit_typed_10d(
        &self,
        request: Typed10dEditRequest,
        expected: &SourceRef,
        mut limits: Typed10dLimits,
        cancelled: &AtomicBool,
    ) -> Result<SaveReceipt, String> {
        request.validate_bounds()?;
        live(cancelled)?;
        if self.typed_source(expected)? != request.source_digest {
            return Err("Typed .10d source digest changed; no write".into());
        }
        self.preflight_resource_edit(
            &request.resource.filename,
            ExpectedFile::Sha256(request.resource.package_digest),
        )?;
        let view = self.inspect_typed_10d(request.resource.clone(), expected, limits)?;
        if let Some(blocker) = &view.edit_blocker {
            return Err(blocker.clone());
        }
        drop(view);
        limits.max_file_bytes = limits.max_file_bytes.min(self.limits.file_bytes);
        let selected = &request.resource;
        let checkpoint = core::load(&self.path(&selected.filename)?, self.limits)?;
        if checkpoint.file_digest() != Some(selected.package_digest) {
            return Err("Typed .10d package changed before edit".into());
        }
        let resource = checkpoint
            .resources()
            .iter()
            .find(|r| r.key == selected.key)
            .ok_or("Selected resource absent")?;
        if resource.summary().digest != selected.resource_digest
            || resource.kind != "10d"
            || resource.meta.is_some()
        {
            return Err("Typed .10d resource changed before edit".into());
        }
        let bytes = payload::replace(&resource.bytes, &request, limits, cancelled)?;
        drop(checkpoint);
        live(cancelled)?;
        if self.typed_source(expected)? != request.source_digest {
            return Err("Typed .10d source changed before save".into());
        }
        self.save_with_resource_edits(
            &expected.stamp,
            &selected.filename,
            ExpectedFile::Sha256(selected.package_digest),
            Some(selected.package_digest),
            vec![],
            vec![ResourceEdit::ReplaceBytes {
                key: selected.key.clone(),
                expected: selected.resource_digest,
                bytes,
            }],
        )
        .map(|saved| saved.receipt)
    }
}
