use super::*;
use qualia_core_db::poet_host::{
    invoke::studio_store::checkpoint as io,
    native_authority::{PackageWriteIntent, PackageWriteReceipt},
};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

/// A native-configured destination, never deserialised from an agent request.
pub struct NativeOutputRoot {
    configured: PathBuf,
    canonical: PathBuf,
}
impl NativeOutputRoot {
    pub fn admit(root: &Path) -> Result<Self, String> {
        let meta = std::fs::symlink_metadata(root).map_err(|_| "Output directory unavailable")?;
        if !meta.is_dir() || meta.file_type().is_symlink() {
            return Err("Output root must be an existing native directory".into());
        }
        Ok(Self {
            configured: root.to_owned(),
            canonical: root
                .canonicalize()
                .map_err(|_| "Output root cannot be resolved")?,
        })
    }
    fn destination(&self, name: &str) -> Result<PathBuf, String> {
        self.destination_with_ext(name, ".hmc", "Use a visible HMC filename without directory components")
    }
    fn destination_html(&self, name: &str) -> Result<PathBuf, String> {
        self.destination_with_ext(name, ".html", "Use a visible HTML filename without directory components")
    }
    /// Confine an agent-supplied filename to this native root. Never join caller paths.
    pub fn destination_with_ext(&self, name: &str, ext: &str, err_msg: &str) -> Result<PathBuf, String> {
        if name.len() > 96
            || name.starts_with('.')
            || !name.ends_with(ext)
            || name.len() <= ext.len()
            || !name
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, b'-' | b'_' | b'.'))
        {
            return Err(err_msg.into());
        }
        if self
            .configured
            .canonicalize()
            .map_err(|_| "Output root unavailable")?
            != self.canonical
        {
            return Err("Native output root changed".into());
        }
        Ok(self.canonical.join(name))
    }
}
pub struct SavedCandidate {
    pub filename: String,
    pub digest: [u8; 32],
    pub bytes: usize,
    pub audit_recorded: bool,
    pub durability_warning: Option<String>,
}
impl SynthHost {
    pub fn configure_output(&mut self, root: NativeOutputRoot) -> Result<(), String> {
        let generation = self
            .output_generation
            .checked_add(1)
            .ok_or("Output scope generation exhausted")?;
        self.output_root = Some(root);
        self.output_generation = generation;
        Ok(())
    }
    /// Create-only candidate save through Core's existing atomic writer and
    /// native package audit. Existing masters and media are never overwritten.
    pub fn save_candidate(
        &mut self,
        expected: &NativeContext,
        name: &str,
    ) -> Result<SavedCandidate, String> {
        self.guard(expected)?;
        let path = self
            .output_root
            .as_ref()
            .ok_or("Native output destination not configured")?
            .destination(name)?;
        let intent = PackageWriteIntent {
            filename: path
                .to_str()
                .ok_or("Output location cannot be represented in native package evidence")?
                .to_owned(),
            expected_digest: None,
        };
        let authority = self.authority.clone();
        let session = self.session.clone();
        let mut committed = None;
        let result = authority.transact_with_package(&session, Some(&intent), |v| {
            let outcome = (|| {
                authorize(
                    v,
                    &AuthoringRequest {
                        action: AuthoringAction::SaveCheckpoint,
                        target: &expected.document,
                        expected_authority_generation: expected.authority_generation,
                    },
                )
                .map_err(|e| e.to_string())?;
                let bytes = self
                    .production
                    .candidate()
                    .ok_or("No admitted candidate")?
                    .bundle();
                let receipt = io::save_bundle(
                    &path,
                    bytes,
                    io::ExpectedFile::Absent,
                    self.production.limits().package_bytes,
                )?;
                let evidence = PackageWriteReceipt {
                    digest: receipt.digest,
                    bytes: receipt.bytes,
                };
                committed = Some(SavedCandidate {
                    filename: name.to_owned(),
                    digest: receipt.digest,
                    bytes: receipt.bytes,
                    audit_recorded: false,
                    durability_warning: receipt.durability_warning,
                });
                Ok(evidence)
            })();
            match outcome {
                Ok(evidence) => (transaction(Ok(())), Some(evidence)),
                Err(error) => (transaction(Err(error)), None),
            }
        });
        match (result, committed) {
            (Ok(()), Some(mut saved)) => {
                saved.audit_recorded = true;
                Ok(saved)
            }
            (Err(_), Some(saved)) => {
                self.quarantined = true;
                Ok(saved)
            }
            (Err(error), None) => Err(error),
            _ => Err("Native package transaction returned no receipt".into()),
        }
    }
    /// Export a self-contained HTML presentation for Antigravity IDE / Auxiliary Pane / Browser Preview.
    pub fn present_candidate(
        &self,
        expected: &NativeContext,
        name: Option<&str>,
    ) -> Result<serde_json::Value, String> {
        self.guard(expected)?;
        let destination = match name {
            Some(filename) => Some(
                self.output_root
                    .as_ref()
                    .ok_or("Native output destination not configured")?
                    .destination_html(filename)?,
            ),
            None => None,
        };
        self.authority.transact(&self.session, |v| {
            transaction((|| {
                authorize(
                    v,
                    &AuthoringRequest {
                        action: AuthoringAction::ReadSource,
                        target: &expected.document,
                        expected_authority_generation: expected.authority_generation,
                    },
                )
                .map_err(|e| e.to_string())?;
                let saved_path = if let Some(path) = &destination {
                    let html = crate::present::render_session_html(&self.production)?;
                    crate::present::write_presentation_file(
                        path,
                        &html,
                        self.production.limits().package_bytes,
                    )?;
                    Some(path.display().to_string())
                } else {
                    None
                };
                crate::present::session_presentation_summary(
                    &self.production,
                    saved_path.as_deref(),
                )
            })())
        })
    }
    /// Fresh replay of the SAME admitted source from exact saved bytes. Older or
    /// unrelated sources require an explicit native source adoption, not a filename.
    pub fn reopen_candidate(
        &mut self,
        expected: &NativeContext,
        name: &str,
        package_digest: [u8; 32],
        cancel: &AtomicBool,
    ) -> Result<(), String> {
        self.guard(expected)?;
        self.authority.transact(&self.session, |v| {
            transaction(
                authorize(
                    v,
                    &AuthoringRequest {
                        action: AuthoringAction::ReopenCheckpoint,
                        target: &expected.document,
                        expected_authority_generation: expected.authority_generation,
                    },
                )
                .map(|_| ())
                .map_err(|e| e.to_string()),
            )
        })?;
        let path = self
            .output_root
            .as_ref()
            .ok_or("Native output destination not configured")?
            .destination(name)?;
        let bytes =
            io::load_bundle_cancellable(&path, self.production.limits().package_bytes, cancel)?;
        let actual: [u8; 32] = Sha256::digest(&bytes).into();
        if actual != package_digest {
            return Err("Saved package digest changed".into());
        }
        let next = self.production.reopen(&bytes, cancel)?;
        let generation = reserve_generation()?;
        if next.document_digest()? != expected.document.source_digest {
            return Err("Saved source differs from the current native document".into());
        }
        let authority = self.authority.clone();
        let session = self.session.clone();
        authority.transact(&session, |v| {
            transaction((|| {
                authorize(
                    v,
                    &AuthoringRequest {
                        action: AuthoringAction::ReopenCheckpoint,
                        target: &expected.document,
                        expected_authority_generation: expected.authority_generation,
                    },
                )
                .map_err(|e| e.to_string())?;
                if cancel.load(Ordering::Acquire) {
                    return Err("Reopen cancelled before activation".into());
                }
                self.production = next;
                self.source_generation = generation;
                Ok(())
            })())
        })
    }
}
