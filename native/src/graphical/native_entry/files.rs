//! Bounded native host configuration and controller locators, never authority.
use crate::graphical::cooperation::Scope;
use qualia_cooperative_core::authoring::PrincipalKind;
use qualia_core_db::poet_host::native_authority::TrustedProject;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    fs::{self, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
};
pub(super) const CONTROLLER_LABEL: &str = "controller";
pub(super) const AUTHORITY_LABEL: &str = "project-authority";
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct PublicProject {
    pub version: u32,
    pub name: String,
    pub project_id: String,
    pub document_id: String,
    pub source_id: String,
    pub domain: String,
    pub issuer_identifier: String,
    pub authority_public_key: [u8; 32],
    pub filename: String,
}
impl PublicProject {
    pub fn scope(&self) -> Scope {
        Scope {
            project_id: self.project_id.clone(),
            document_id: self.document_id.clone(),
            source_id: self.source_id.clone(),
            domain: self.domain.clone(),
        }
    }
    pub fn anchor(&self) -> TrustedProject {
        TrustedProject {
            project_id: self.project_id.clone(),
            issuer_identifier: self.issuer_identifier.clone(),
            public_key: self.authority_public_key,
        }
    }
    pub fn fingerprint(&self) -> String {
        format!(
            "sha256:{}",
            hex_key(&Sha256::digest(self.authority_public_key).into())
        )
    }
    fn validate(&self) -> Result<(), String> {
        if self.version != 1 || self.domain != "Geometry" || self.authority_public_key == [0; 32] {
            return Err("Unsupported native project configuration".into());
        }
        for text in [
            &self.name,
            &self.project_id,
            &self.document_id,
            &self.source_id,
            &self.issuer_identifier,
        ] {
            bounded_text(text, 4096)?;
        }
        filename(&self.filename)
    }
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Profile {
    pub version: u32,
    pub label: String,
    pub requested_kind: PrincipalKind,
    pub controller_public_key: [u8; 32],
}
pub(super) fn hex_key(key: &[u8; 32]) -> String {
    key.iter().map(|b| format!("{b:02x}")).collect()
}
pub(super) fn subject(key: &[u8; 32]) -> String {
    format!("urn:qualia:native-controller:{}", hex_key(key))
}
pub(super) fn bounded_text(value: &str, max: usize) -> Result<(), String> {
    if value.trim().is_empty() || value.len() > max || value.chars().any(|c| c == '\0' || c == '\r')
    {
        Err("Enter bounded, nonempty native operator text".into())
    } else {
        Ok(())
    }
}
pub(super) fn profile_name(value: &str) -> Result<(), String> {
    if value.is_empty()
        || value.len() > 64
        || !value
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || matches!(b, b'-' | b'_'))
        || matches!(value, "con" | "prn" | "aux" | "nul")
        || (value.len() == 4
            && (value.starts_with("com") || value.starts_with("lpt"))
            && matches!(value.as_bytes()[3], b'1'..=b'9'))
    {
        Err(
            "Choose a lowercase profile name using letters, numbers, - or _; no paths/device names"
                .into(),
        )
    } else {
        Ok(())
    }
}
pub(super) fn filename(value: &str) -> Result<(), String> {
    if value.len() < 5
        || value.len() > 128
        || value.starts_with('.')
        || !value.to_ascii_lowercase().ends_with(".hmc")
        || !value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.'))
    {
        return Err("Choose one plain .hmc filename in packages".into());
    }
    let stem = value.split('.').next().unwrap_or("").to_ascii_lowercase();
    profile_name(&stem)
}
fn regular(meta: &fs::Metadata, directory: bool) -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        if meta.file_attributes() & 0x400 != 0 {
            return false;
        }
    }
    !meta.file_type().is_symlink()
        && if directory {
            meta.is_dir()
        } else {
            meta.is_file()
        }
}
pub(super) struct HostFiles {
    configured: PathBuf,
    pub root: PathBuf,
}
impl HostFiles {
    pub fn open(root: &Path) -> Result<Self, String> {
        if !regular(
            &fs::symlink_metadata(root).map_err(|e| e.to_string())?,
            true,
        ) {
            return Err("Native project root must be an existing non-link directory".into());
        }
        Ok(Self {
            configured: root.into(),
            root: root.canonicalize().map_err(|e| e.to_string())?,
        })
    }
    pub fn check(&self) -> Result<(), String> {
        if !regular(
            &fs::symlink_metadata(&self.configured).map_err(|e| e.to_string())?,
            true,
        ) || self.configured.canonicalize().map_err(|e| e.to_string())? != self.root
        {
            Err("Native project root changed; reopen explicitly".into())
        } else {
            Ok(())
        }
    }
    pub fn directory(&self, name: &str, create: bool) -> Result<PathBuf, String> {
        self.check()?;
        let path = self.root.join(name);
        if create && !path.exists() {
            fs::create_dir(&path).map_err(|e| e.to_string())?;
        }
        let meta =
            fs::symlink_metadata(&path).map_err(|e| format!("Native {name} directory: {e}"))?;
        if !regular(&meta, true) {
            return Err("Native directories cannot be links or reparse points".into());
        }
        let canonical = path.canonicalize().map_err(|e| e.to_string())?;
        if canonical.parent() != Some(self.root.as_path()) {
            return Err("Native directory escaped project root".into());
        }
        Ok(canonical)
    }
    pub fn project(&self) -> Result<Option<PublicProject>, String> {
        self.check()?;
        let path = self.root.join("project.json");
        match fs::symlink_metadata(&path) {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(e) => return Err(e.to_string()),
            Ok(_) => (),
        }
        let project: PublicProject = read_json(&path, 32768)?;
        project.validate()?;
        Ok(Some(project))
    }
    pub fn same_project(&self, project: &PublicProject) -> Result<(), String> {
        let actual = self
            .project()?
            .ok_or("Public project configuration disappeared")?;
        if serde_json::to_vec(&actual).map_err(|e| e.to_string())?
            != serde_json::to_vec(project).map_err(|e| e.to_string())?
        {
            return Err(
                "Public project configuration changed; explicit reopen and adoption required"
                    .into(),
            );
        }
        Ok(())
    }
    pub fn save_project(&self, project: &PublicProject) -> Result<(), String> {
        self.check()?;
        project.validate()?;
        write_new(&self.root.join("project.json"), project, 32768)
    }
    pub fn profile_path(&self, name: &str, create_parent: bool) -> Result<PathBuf, String> {
        profile_name(name)?;
        Ok(self.directory("profiles", create_parent)?.join(name))
    }
    pub fn profile(&self, name: &str) -> Result<Profile, String> {
        let path = self.profile_path(name, false)?;
        if !regular(
            &fs::symlink_metadata(&path).map_err(|e| e.to_string())?,
            true,
        ) {
            return Err("Profile must be a non-link directory".into());
        }
        let profile: Profile = read_json(&path.join("profile.json"), 8192)?;
        if profile.version != 1 || profile.label != name || profile.controller_public_key == [0; 32]
        {
            return Err("Invalid native controller locator".into());
        }
        Ok(profile)
    }
    pub fn save_profile(&self, profile: &Profile) -> Result<(), String> {
        let path = self.profile_path(&profile.label, true)?;
        if !regular(
            &fs::symlink_metadata(&path).map_err(|e| e.to_string())?,
            true,
        ) {
            return Err("Profile path changed".into());
        }
        write_new(&path.join("profile.json"), profile, 8192)
    }
}
fn read_json<T: for<'a> Deserialize<'a>>(path: &Path, budget: usize) -> Result<T, String> {
    let meta = fs::symlink_metadata(path).map_err(|e| e.to_string())?;
    if !regular(&meta, false) || meta.len() > budget as u64 {
        return Err("Native metadata violates file/byte bounds".into());
    }
    let mut options = OpenOptions::new();
    options.read(true);
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        options.custom_flags(0x00200000);
    }
    let file = options.open(path).map_err(|e| e.to_string())?;
    if !regular(&file.metadata().map_err(|e| e.to_string())?, false) {
        return Err("Native metadata changed during open".into());
    }
    let mut bytes = Vec::new();
    file.take(budget as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() > budget {
        return Err("Native metadata grew beyond byte budget".into());
    }
    serde_json::from_slice(&bytes).map_err(|e| e.to_string())
}
fn write_new<T: Serialize>(path: &Path, value: &T, budget: usize) -> Result<(), String> {
    let bytes = serde_json::to_vec_pretty(value).map_err(|e| e.to_string())?;
    if bytes.len() > budget {
        return Err("Native metadata byte budget exceeded".into());
    }
    let mut file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(path)
        .map_err(|e| e.to_string())?;
    file.write_all(&bytes)
        .and_then(|_| file.sync_all())
        .map_err(|e| e.to_string())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn locator_paths_are_bounded_and_cannot_select_authority() {
        for value in ["../me", "con", "COM1", "me/name", "Alice", "", "lpt1"] {
            assert!(profile_name(value).is_err());
        }
        for value in ["../master.hmc", "C:/master.hmc", ".hidden.hmc", "con.hmc"] {
            assert!(filename(value).is_err());
        }
        assert!(profile_name("artist-1").is_ok());
        assert!(filename("working.hmc").is_ok());
    }
    #[test]
    fn public_config_and_empty_runtime_never_open_os_keys_or_hmc() {
        let temp = tempfile::tempdir().unwrap();
        let files = HostFiles::open(temp.path()).unwrap();
        assert!(files.project().unwrap().is_none());
        let public = PublicProject {
            version: 1,
            name: "Public fixture".into(),
            project_id: "p".into(),
            document_id: "d".into(),
            source_id: "s".into(),
            domain: "Geometry".into(),
            issuer_identifier: "issuer".into(),
            authority_public_key: [9; 32],
            filename: "working.hmc".into(),
        };
        files.save_project(&public).unwrap();
        assert!(files.save_project(&public).is_err());
        let mut runtime = super::super::runtime::Runtime::open(temp.path()).unwrap();
        let view = runtime.view().unwrap();
        assert!(
            !view.adopted
                && view.current_subject.is_none()
                && view.studio.is_none()
                && view.pending.is_empty()
                && view.members.is_empty()
        );
        assert_eq!(view.authority_fingerprint, Some(public.fingerprint()));
    }
}
