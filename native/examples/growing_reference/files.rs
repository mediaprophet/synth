//! Bounded, explicit output paths; no overwrites or credential IO.
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{
    fs::{self, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
};

fn ordinary(path: &Path, directory: bool) -> Result<(), String> {
    let metadata = fs::symlink_metadata(path).map_err(|e| format!("{}: {e}", path.display()))?;
    if metadata.file_type().is_symlink() || metadata.is_dir() != directory {
        return Err(format!(
            "expected ordinary {}: {}",
            if directory { "directory" } else { "file" },
            path.display()
        ));
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        if metadata.file_attributes() & 0x400 != 0 {
            return Err(format!("reparse paths are refused: {}", path.display()));
        }
    }
    Ok(())
}
pub fn existing_directory(path: &Path, empty: bool) -> Result<PathBuf, String> {
    if !path.is_absolute() {
        return Err("use an explicit absolute existing directory".into());
    }
    for ancestor in path.ancestors() {
        ordinary(ancestor, true)?;
    }
    let path = fs::canonicalize(path).map_err(|e| e.to_string())?;
    if empty
        && fs::read_dir(&path)
            .map_err(|e| e.to_string())?
            .next()
            .is_some()
    {
        return Err("output directory must already exist and be empty".into());
    }
    Ok(path)
}
pub fn create_directory(path: &Path) -> Result<(), String> {
    ordinary(path.parent().ok_or("missing parent")?, true)?;
    fs::create_dir(path).map_err(|e| format!("{}: {e}", path.display()))
}
pub fn read(path: &Path, budget: usize) -> Result<Vec<u8>, String> {
    ordinary(path, false)?;
    for ancestor in path.parent().ok_or("missing parent")?.ancestors() {
        ordinary(ancestor, true)?;
    }
    let mut options = OpenOptions::new();
    options.read(true);
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        options.custom_flags(0x00200000);
    }
    let file = options.open(path).map_err(|e| e.to_string())?;
    if !file.metadata().map_err(|e| e.to_string())?.is_file() {
        return Err("not a regular input file".into());
    }
    let mut bytes = Vec::new();
    file.take(budget as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() > budget {
        return Err(format!("input exceeds {budget} bytes: {}", path.display()));
    }
    Ok(bytes)
}
pub fn write(path: &Path, bytes: &[u8]) -> Result<(), String> {
    for ancestor in path.parent().ok_or("missing parent")?.ancestors() {
        ordinary(ancestor, true)?;
    }
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|e| format!("{}: {e}", path.display()))?;
    file.write_all(bytes)
        .and_then(|_| file.sync_all())
        .map_err(|e| e.to_string())
}
pub fn json(path: &Path, value: &Value) -> Result<(), String> {
    write(
        path,
        &serde_json::to_vec_pretty(value).map_err(|e| e.to_string())?,
    )
}
pub fn hash(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}
pub fn check_hash(path: &Path, expected: &str, budget: usize) -> Result<Vec<u8>, String> {
    let bytes = read(path, budget)?;
    if hash(&bytes) != expected {
        return Err(format!("SHA256 changed: {}", path.display()));
    }
    Ok(bytes)
}
