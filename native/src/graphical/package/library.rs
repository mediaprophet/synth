use std::{
    io::Read,
    path::{Path, PathBuf},
};

/// Cold, bounded file ingress. Root and budget come only from native embedding.
pub struct ResourceLibrary {
    configured: PathBuf,
    canonical: PathBuf,
    max_bytes: usize,
}
impl ResourceLibrary {
    pub fn new(root: &Path, max_bytes: usize) -> Result<Self, String> {
        let canonical = root.canonicalize().map_err(|e| e.to_string())?;
        if max_bytes == 0 || !canonical.is_dir() {
            return Err(
                "Resource library requires an existing directory and explicit nonzero byte budget"
                    .into(),
            );
        }
        Ok(Self {
            configured: root.into(),
            canonical,
            max_bytes,
        })
    }
    pub fn read(&self, name: &str) -> Result<Vec<u8>, String> {
        self.read_with_budget(name, self.max_bytes)
    }
    pub(crate) fn byte_budget(&self) -> usize {
        self.max_bytes
    }
    pub(crate) fn validate_filename(name: &str) -> Result<(), String> {
        if name.is_empty()
            || name.len() > 240
            || name.starts_with('.')
            || !name
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.'))
        {
            return Err("Choose one plain filename from the native resource library".into());
        }
        let stem = name.split('.').next().unwrap_or("").to_ascii_uppercase();
        if matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL")
            || (stem.len() == 4
                && (stem.starts_with("COM") || stem.starts_with("LPT"))
                && matches!(stem.as_bytes()[3], b'1'..=b'9'))
        {
            return Err("Resource filename is an OS device name".into());
        }
        Ok(())
    }
    pub(crate) fn read_with_budget(&self, name: &str, remaining: usize) -> Result<Vec<u8>, String> {
        Self::validate_filename(name)?;
        let cap = remaining.min(self.max_bytes);
        if self.configured.canonicalize().map_err(|e| e.to_string())? != self.canonical
            || self.canonical.canonicalize().map_err(|e| e.to_string())? != self.canonical
        {
            return Err("Native resource library directory changed".into());
        }
        let path = self.canonical.join(name);
        let meta = std::fs::symlink_metadata(&path).map_err(|e| e.to_string())?;
        if !meta.is_file() || meta.file_type().is_symlink() || meta.len() > cap as u64 {
            return Err("Resource must be a bounded regular file, not a link/directory".into());
        }
        if path.canonicalize().map_err(|e| e.to_string())? != path {
            return Err("Resource resolution changed".into());
        }
        let file = std::fs::File::open(&path).map_err(|e| e.to_string())?;
        let opened = file.metadata().map_err(|e| e.to_string())?;
        if !opened.is_file() || opened.len() > cap as u64 {
            return Err("Opened resource exceeds remaining aggregate import budget".into());
        }
        let mut bytes = Vec::new();
        file.take((cap as u64).saturating_add(1))
            .read_to_end(&mut bytes)
            .map_err(|e| e.to_string())?;
        if bytes.len() > cap {
            return Err("Resource grew beyond import byte budget".into());
        }
        Ok(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn selected_library_is_bounded_and_paths_are_not_authority() {
        let root = tempfile::tempdir().unwrap();
        std::fs::write(root.path().join("master.bin"), [1, 2, 3, 4]).unwrap();
        let library = ResourceLibrary::new(root.path(), 4).unwrap();
        assert_eq!(library.read("master.bin").unwrap(), [1, 2, 3, 4]);
        for name in [
            "../master.bin",
            "C:/secret",
            "master.bin:stream",
            "CON.bin",
            ".hidden",
        ] {
            assert!(library.read(name).is_err(), "{name}")
        }
        assert!(ResourceLibrary::new(root.path(), 3)
            .unwrap()
            .read("master.bin")
            .is_err());
        assert!(ResourceLibrary::new(root.path(), 0).is_err());
    }
}
