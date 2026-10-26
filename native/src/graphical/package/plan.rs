//! One bounded import plan, one guarded inventory, one raw atomic core save.
use super::*;
use std::collections::BTreeSet;

pub(super) fn validate_requests(plan: &[LibraryEdit]) -> Result<(), String> {
    if plan.is_empty() || plan.len() > MAX_RESOURCE_EDITS {
        return Err("Choose between 1 and 64 resource edits in one atomic plan".into());
    }
    let mut keys = BTreeSet::new();
    for edit in plan {
        let key = match edit {
            LibraryEdit::Import(request) => {
                ResourceLibrary::validate_filename(&request.filename)?;
                if request.kind.len() > 4096
                    || request.kind.contains('\0')
                    || (request.expected_resource.is_none() && request.kind.is_empty())
                {
                    return Err("New resource kind is missing or invalid".into());
                }
                &request.key
            }
            LibraryEdit::Remove { key, .. } => key,
        };
        if key.is_empty()
            || key.len() > 4096
            || key.contains('\0')
            || key == "authoring/native-source.json"
            || !keys.insert(key.as_str())
        {
            return Err("Resource plan has an invalid, reserved or duplicate key".into());
        }
    }
    Ok(())
}

pub(super) fn apply(
    host: &mut ResourceImportHost,
    name: &str,
    file: ExpectedFile,
    plan: Vec<LibraryEdit>,
    expected: SourceRef,
) -> Result<SaveReceipt, String> {
    // Validate ALL logical intents, including filenames, before ANY library IO.
    validate_requests(&plan)?;
    if host.inner.snapshot()?.source != expected {
        return Err("Resource plan source changed before file access".into());
    }
    expected.check_current()?;
    host.inner
        .resource_edit_preflight(name, file, expected.clone())?;
    let inventory = host.inner.package_inventory(name, expected.clone())?;
    if !matches!(file, ExpectedFile::Sha256(digest) if digest == inventory.digest) {
        return Err("Resource plan package digest changed before file access".into());
    }
    for edit in &plan {
        let (key, digest) = match edit {
            LibraryEdit::Import(request) => (&request.key, request.expected_resource),
            LibraryEdit::Remove {
                key,
                expected_resource,
            } => (key, Some(*expected_resource)),
        };
        let existing = inventory.resources.iter().find(|entry| &entry.key == key);
        match (digest, existing) {
            (Some(expected), Some(entry)) if expected == entry.digest => {}
            (None, None) => {}
            _ => return Err("Resource plan key/content changed before any file access".into()),
        }
    }
    // A single native-configured byte budget covers the aggregate payload,
    // rather than granting the complete cap independently to 64 imports.
    let mut remaining = host.library.byte_budget();
    let mut edits = Vec::with_capacity(plan.len());
    for edit in plan {
        edits.push(match edit {
            LibraryEdit::Remove {
                key,
                expected_resource,
            } => ResourceEdit::Remove {
                key,
                expected: expected_resource,
            },
            LibraryEdit::Import(request) => {
                let bytes = host
                    .library
                    .read_with_budget(&request.filename, remaining)?;
                remaining = remaining
                    .checked_sub(bytes.len())
                    .ok_or("Resource plan aggregate byte budget exceeded")?;
                match request.expected_resource {
                    Some(expected) => ResourceEdit::ReplaceBytes {
                        key: request.key,
                        expected,
                        bytes,
                    },
                    None => ResourceEdit::Add(ResourceData {
                        key: request.key,
                        kind: request.kind,
                        bytes,
                        meta: None,
                    }),
                }
            }
        });
    }
    // Recheck the actual source and package/policy immediately before save. The
    // raw checkpoint owner repeats its source/SHA/dependency guards and rename.
    if host.inner.snapshot()?.source != expected {
        return Err("Source changed while reading resource plan; no save".into());
    }
    expected.check_current()?;
    host.inner
        .resource_edit_preflight(name, file, expected.clone())?;
    host.inner
        .edit_package_resources(name, file, edits, expected)
}
