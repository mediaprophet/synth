//! Revision-bound compilation uses the existing core HMC codec and resource transaction.
use crate::graphical::checkpoints::{CheckpointService, ExpectedFile, SaveReceipt};
use crate::graphical::controller::EditorHost;
use crate::graphical::preview::SourceRef;
use qualia_core_db::poet_host::invoke::studio_store::checkpoint as core;

pub(crate) fn compile_checkpoint(
    inner: &dyn EditorHost,
    service: &CheckpointService,
    limits: core::CheckpointLimits,
    name: &str,
    key: &str,
    file: ExpectedFile,
    expected: SourceRef,
) -> Result<SaveReceipt, String> {
    let current = inner.snapshot()?;
    if current.source != expected
        || current.reconciliation_required
        || !current.publication_is_current()
    {
        return Err("Compilation requires current reconstructed accepted source".into());
    }
    expected.check_current()?;
    let carrier = inner.retained_surface_carrier(expected.clone(), limits)?;
    let inputs = if carrier.is_none() {
        inner.retained_surface_inputs(expected.clone())?
    } else {
        Vec::new()
    };
    let assets = inner.compiled_assets(key, expected.clone())?;
    if matches!(file, ExpectedFile::Absent) {
        return service
            .save_with_source_carrier(
                &expected.stamp,
                name,
                file,
                None,
                assets,
                vec![],
                inputs,
                carrier.as_ref(),
            )
            .map(|r| r.receipt);
    }
    // Build a detached candidate through the canonical codec. It supplies both
    // native .10d and Q42 companion resources; no application-owned wire format.
    let mut candidate = core::capture(&expected.artifact_id, &expected.stamp, limits)?;
    if let Some(input) = carrier.as_ref() {
        candidate = candidate.with_source_carrier(input, limits)?;
    }
    let candidate = CheckpointService::merge_retained_inputs(candidate, inputs, limits)?;
    // Retained copies follow the canonical resource-budget admission above.
    let inputs = if carrier.is_none() {
        candidate.resources().to_vec()
    } else {
        Vec::new()
    };
    let bytes = core::encode(&candidate, assets, limits)?;
    let candidate = core::decode(&bytes, limits)?;
    let inventory = candidate.inventory();
    if inventory.is_empty() || inventory.len() > 64 {
        return Err("Compiled resource plan is empty or exceeds host entry bound".into());
    }
    let existing = match file {
        ExpectedFile::Absent => None,
        ExpectedFile::Sha256(digest) => {
            let selected = service.inventory(name)?;
            if selected.digest != digest {
                return Err("Inspected package changed before compile".into());
            }
            Some(selected)
        }
    };
    let mut edits = Vec::with_capacity(inventory.len());
    let origin_key = carrier.as_ref().map(|c| c.origin_key());
    for resource in inventory {
        // Final service admission unions all inert inputs with the selected
        // destination, including protected origins. They are never raw edits.
        if carrier.as_ref().is_some_and(|input| {
            input.resources().iter().any(|r| r.key == resource.key)
                || origin_key.as_deref() == Some(resource.key.as_str())
        }) || inputs.iter().any(|r| r.key == resource.key)
        {
            continue;
        }
        let data = candidate
            .resources()
            .iter()
            .find(|r| r.key == resource.key)
            .cloned()
            .ok_or("Compiled resource disappeared")?;
        if let Some(old) = existing
            .as_ref()
            .and_then(|p| p.resources.iter().find(|r| r.key == resource.key))
        {
            if old.kind != resource.kind {
                return Err("Compiled resource cannot change an existing entry kind".into());
            }
            if old.metadata_digest != resource.metadata_digest
                || old.metadata_bytes != resource.metadata_bytes
            {
                return Err("Compiled resource cannot change existing opaque metadata; retain the exact input or use a new key".into());
            }
            // A digest guard is permission to replace the selected file, not
            // proof every richer section or graph fact is compiler-owned.
            // Fresh explicit keys retain the old complete representation.
            if old.digest != resource.digest {
                return Err("Existing compiled resource differs; choose a new asset key to retain its complete .10d sections and graph facts. Same-key semantic merge is not yet qualified".into());
            }
            edits.push(core::ResourceEdit::ReplaceBytes {
                key: resource.key,
                expected: old.digest,
                bytes: data.bytes,
            });
        } else {
            edits.push(core::ResourceEdit::Add(data));
        }
    }
    expected.check_current()?;
    let parent = match file {
        ExpectedFile::Absent => None,
        ExpectedFile::Sha256(d) => Some(d),
    };
    service
        .save_with_source_carrier(
            &expected.stamp,
            name,
            file,
            parent,
            vec![],
            edits,
            inputs,
            carrier.as_ref(),
        )
        .map(|r| r.receipt)
}
