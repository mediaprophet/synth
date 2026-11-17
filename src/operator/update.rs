//! Signed GitHub-release updates.
//!
//! QualiaDB Ed25519 `sign_with_context` binds a 512-byte buffer, so the signed
//! message is the SHA-256 of the artifact, not the file body. Context label:
//! `synth.release.sha256.v1`. The 32-byte seed stays in gitignored
//! `.synth/update-seed.hex` / `SYNTH_UPDATE_SEED`; the public key is compiled in.
use qualia_core_db::crypto::network::ed25519::{
    public_from_seed, sign_with_context, verify_with_context,
};
use qualia_core_db::crypto::network::entropy::fill_os;
use serde::Deserialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Path, PathBuf},
};

pub const RELEASE_SIGN_CONTEXT: &[u8] = b"synth.release.sha256.v1";
pub const RELEASE_PUBLIC_KEY_HEX: &str = include_str!("../../.synth/update-public.hex");
pub const RELEASES_URL: &str = "https://github.com/mediaprophet/synth/releases";
pub const GITHUB_SECRET_NAME: &str = "SYNTH_UPDATE_SEED";
const SEED_FILE_NAME: &str = "update-seed.hex";

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default, deny_unknown_fields)]
pub struct UpdateArg {
    pub action: Option<String>,
    pub artifact: Option<String>,
    pub sig: Option<String>,
    pub dest: Option<String>,
}

pub fn signing_info() -> Value {
    json!({
        "algorithm": "ed25519",
        "context": std::str::from_utf8(RELEASE_SIGN_CONTEXT).unwrap_or("synth.release.sha256.v1"),
        "public_key_hex": RELEASE_PUBLIC_KEY_HEX.trim(),
        "binds": "SHA-256 of the artifact bytes (QualiaDB sign_with_context)",
        "github_secret": GITHUB_SECRET_NAME,
        "releases_url": RELEASES_URL
    })
}

pub fn release_public_key() -> Result<[u8; 32], String> {
    parse_hex32(RELEASE_PUBLIC_KEY_HEX)
}

pub fn to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

pub fn parse_hex32(raw: &str) -> Result<[u8; 32], String> {
    let s: String = raw.chars().filter(|c| !c.is_whitespace()).collect();
    if s.len() != 64 {
        return Err(format!("expected 64 hex chars, got {}", s.len()));
    }
    let mut out = [0u8; 32];
    for i in 0..32 {
        out[i] = u8::from_str_radix(&s[i * 2..i * 2 + 2], 16)
            .map_err(|_| "invalid hex in 32-byte value".to_string())?;
    }
    Ok(out)
}

fn parse_hex64(raw: &str) -> Result<[u8; 64], String> {
    let s: String = raw.chars().filter(|c| !c.is_whitespace()).collect();
    if s.len() != 128 {
        return Err(format!("expected 128 hex chars for signature, got {}", s.len()));
    }
    let mut out = [0u8; 64];
    for i in 0..64 {
        out[i] = u8::from_str_radix(&s[i * 2..i * 2 + 2], 16)
            .map_err(|_| "invalid hex in signature".to_string())?;
    }
    Ok(out)
}

pub fn sha256_file(path: &Path) -> Result<[u8; 32], String> {
    let bytes = fs::read(path).map_err(|e| format!("read {}: {e}", path.display()))?;
    Ok(Sha256::digest(&bytes).into())
}

pub fn sign_digest(seed: &[u8; 32], digest: &[u8; 32]) -> Result<[u8; 64], String> {
    let mut sig = [0u8; 64];
    sign_with_context(seed, digest, RELEASE_SIGN_CONTEXT, &mut sig)
        .map_err(|e| format!("ed25519 sign failed: {e:?}"))?;
    Ok(sig)
}

pub fn verify_digest(digest: &[u8; 32], signature: &[u8; 64]) -> Result<(), String> {
    let pk = release_public_key()?;
    verify_with_context(&pk, digest, RELEASE_SIGN_CONTEXT, signature)
        .map_err(|_| "signature does not match the embedded Synth release public key".to_string())
}

fn default_key_dir() -> PathBuf {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".synth");
    if manifest.parent().map(|p| p.is_dir()).unwrap_or(false) {
        manifest
    } else {
        PathBuf::from(".synth")
    }
}

fn seed_path(dir: &Path) -> PathBuf {
    dir.join(SEED_FILE_NAME)
}

pub fn load_seed(explicit: Option<&Path>) -> Result<[u8; 32], String> {
    if let Ok(env) = std::env::var("SYNTH_UPDATE_SEED") {
        let trimmed = env.trim();
        if !trimmed.is_empty() {
            return parse_hex32(trimmed);
        }
    }
    if let Some(path) = explicit {
        let raw = fs::read_to_string(path).map_err(|e| format!("read seed {}: {e}", path.display()))?;
        return parse_hex32(&raw);
    }
    let default = seed_path(&default_key_dir());
    if default.is_file() {
        let raw = fs::read_to_string(&default)
            .map_err(|e| format!("read seed {}: {e}", default.display()))?;
        return parse_hex32(&raw);
    }
    Err(format!(
        "No update seed. Set {GITHUB_SECRET_NAME} or pass --seed-file. Generate with `synth update keygen`."
    ))
}

/// Write a new 32-byte seed (gitignored) and matching public-key sidecar.
pub fn keygen(out_dir: Option<&Path>, force: bool) -> Result<Value, String> {
    let dir = out_dir.map(Path::to_path_buf).unwrap_or_else(default_key_dir);
    fs::create_dir_all(&dir).map_err(|e| format!("create {}: {e}", dir.display()))?;
    let seed_file = seed_path(&dir);
    if seed_file.exists() && !force {
        return Err(format!(
            "seed already exists at {} (pass --force to rotate; then re-embed the public key)",
            seed_file.display()
        ));
    }
    let mut seed = [0u8; 32];
    fill_os(&mut seed).map_err(|e| format!("OS CSPRNG failed: {e:?}"))?;
    let pk = public_from_seed(&seed);
    fs::write(&seed_file, to_hex(&seed)).map_err(|e| format!("write seed: {e}"))?;
    let pk_file = dir.join("update-public.hex");
    fs::write(&pk_file, format!("{}\n", to_hex(&pk))).map_err(|e| format!("write public: {e}"))?;
    Ok(json!({
        "schema": "synth.update-keygen/v1",
        "seed_path": seed_file.display().to_string(),
        "public_key_path": pk_file.display().to_string(),
        "public_key_hex": to_hex(&pk),
        "github_secret": GITHUB_SECRET_NAME,
        "note": "Paste the 64 hex chars from the seed file into the GitHub Actions secret. Do not commit the seed."
    }))
}

pub fn sign_artifact(artifact: &Path, seed_file: Option<&Path>) -> Result<Value, String> {
    if !artifact.is_file() {
        return Err(format!("artifact not found: {}", artifact.display()));
    }
    let seed = load_seed(seed_file)?;
    let pk = public_from_seed(&seed);
    let embedded = release_public_key()?;
    if pk != embedded {
        return Err(
            "seed does not match the public key compiled into this binary; re-embed update-public.hex"
                .into(),
        );
    }
    let digest = sha256_file(artifact)?;
    let sig = sign_digest(&seed, &digest)?;
    let sig_path = sig_path_for(artifact);
    let sha_path = sha_path_for(artifact);
    fs::write(&sig_path, format!("{}\n", to_hex(&sig))).map_err(|e| format!("write sig: {e}"))?;
    let name = artifact
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("artifact");
    fs::write(&sha_path, format!("{}  {}\n", to_hex(&digest), name))
        .map_err(|e| format!("write sha256: {e}"))?;
    Ok(json!({
        "schema": "synth.update-sign/v1",
        "artifact": artifact.display().to_string(),
        "sha256": to_hex(&digest),
        "signature": to_hex(&sig),
        "sig_path": sig_path.display().to_string(),
        "sha256_path": sha_path.display().to_string(),
        "public_key_hex": to_hex(&pk),
        "context": std::str::from_utf8(RELEASE_SIGN_CONTEXT).unwrap_or("")
    }))
}

fn sig_path_for(artifact: &Path) -> PathBuf {
    let mut p = artifact.as_os_str().to_os_string();
    p.push(".sig");
    PathBuf::from(p)
}

fn sha_path_for(artifact: &Path) -> PathBuf {
    let mut p = artifact.as_os_str().to_os_string();
    p.push(".sha256");
    PathBuf::from(p)
}

pub fn verify_artifact(artifact: &Path, sig: Option<&Path>) -> Result<Value, String> {
    if !artifact.is_file() {
        return Err(format!("artifact not found: {}", artifact.display()));
    }
    let sig_path = sig.map(Path::to_path_buf).unwrap_or_else(|| sig_path_for(artifact));
    let raw = fs::read_to_string(&sig_path)
        .map_err(|e| format!("read signature {}: {e}", sig_path.display()))?;
    let signature = parse_hex64(&raw)?;
    let digest = sha256_file(artifact)?;
    verify_digest(&digest, &signature)?;
    Ok(json!({
        "schema": "synth.update-verify/v1",
        "ok": true,
        "artifact": artifact.display().to_string(),
        "sha256": to_hex(&digest),
        "sig_path": sig_path.display().to_string(),
        "public_key_hex": RELEASE_PUBLIC_KEY_HEX.trim()
    }))
}

fn would_overwrite_running_exe(dest: &Path) -> bool {
    let Ok(exe) = std::env::current_exe() else {
        return false;
    };
    if dest == exe {
        return true;
    }
    if let (Ok(d), Ok(e)) = (fs::canonicalize(dest), fs::canonicalize(&exe)) {
        return d == e;
    }
    dest.file_name() == exe.file_name() && dest.parent() == exe.parent()
}

fn install_dest(artifact: &Path, dest_dir: Option<&Path>) -> Result<PathBuf, String> {
    let name = artifact
        .file_name()
        .ok_or_else(|| "artifact has no file name".to_string())?;
    let dir = if let Some(dir) = dest_dir {
        dir.to_path_buf()
    } else {
        let exe = std::env::current_exe().map_err(|e| e.to_string())?;
        exe.parent()
            .ok_or_else(|| "current exe has no parent directory".to_string())?
            .to_path_buf()
    };
    fs::create_dir_all(&dir).map_err(|e| format!("create {}: {e}", dir.display()))?;
    let dest = dir.join(name);
    if would_overwrite_running_exe(&dest) {
        let stem = artifact
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("synth");
        let ext = artifact
            .extension()
            .and_then(|s| s.to_str())
            .map(|e| format!(".{e}"))
            .unwrap_or_default();
        return Ok(dir.join(format!("{stem}.new{ext}")));
    }
    Ok(dest)
}

/// Copy a verified artifact next to the running binary. Never overwrites a locked exe.
pub fn apply_artifact(
    artifact: &Path,
    sig: Option<&Path>,
    dest_dir: Option<&Path>,
) -> Result<Value, String> {
    let verified = verify_artifact(artifact, sig)?;
    let dest = install_dest(artifact, dest_dir)?;
    if would_overwrite_running_exe(&dest) {
        return Err(
            "refusing to overwrite the running synth.exe; drop the MCP child or copy the sibling .new file"
                .into(),
        );
    }
    fs::copy(artifact, &dest).map_err(|e| format!("copy to {}: {e}", dest.display()))?;
    Ok(json!({
        "schema": "synth.update-apply/v1",
        "ok": true,
        "artifact": artifact.display().to_string(),
        "installed": dest.display().to_string(),
        "sha256": verified["sha256"],
        "auto_replace": false,
        "policy": "Never overwrite a locked synth.exe. Point MCP at the new sidecar, then /mcps reload."
    }))
}

pub fn dispatch(arg: UpdateArg) -> Result<Value, String> {
    let action = arg.action.as_deref().unwrap_or("check").to_ascii_lowercase();
    match action.as_str() {
        "check" => Ok(super::update_status(true)),
        "verify" => {
            let path = arg
                .artifact
                .as_deref()
                .ok_or("verify requires artifact=PATH")?;
            verify_artifact(Path::new(path), arg.sig.as_deref().map(Path::new))
        }
        "apply" => {
            let path = arg
                .artifact
                .as_deref()
                .ok_or("apply requires artifact=PATH")?;
            apply_artifact(
                Path::new(path),
                arg.sig.as_deref().map(Path::new),
                arg.dest.as_deref().map(Path::new),
            )
        }
        "sign" | "keygen" => Err(
            "sign and keygen stay on the CLI (`synth update sign` / `synth update keygen`) so the seed is not passed through MCP"
                .into(),
        ),
        other => Err(format!(
            "unknown update action `{other}` (check, verify, apply)"
        )),
    }
}

/// `synth update check|verify|apply|sign|keygen`
pub fn run_cli(args: &[String]) -> Result<(), String> {
    let action = args.first().map(String::as_str).unwrap_or("check");
    let mut artifact: Option<PathBuf> = None;
    let mut sig: Option<PathBuf> = None;
    let mut seed_file: Option<PathBuf> = None;
    let mut dest: Option<PathBuf> = None;
    let mut out: Option<PathBuf> = None;
    let mut force = false;
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--artifact" => {
                i += 1;
                artifact = Some(PathBuf::from(
                    args.get(i).ok_or("Usage: synth update … --artifact PATH")?,
                ));
            }
            "--sig" => {
                i += 1;
                sig = Some(PathBuf::from(
                    args.get(i).ok_or("Usage: synth update … --sig PATH")?,
                ));
            }
            "--seed-file" => {
                i += 1;
                seed_file = Some(PathBuf::from(
                    args.get(i).ok_or("Usage: synth update sign --seed-file PATH")?,
                ));
            }
            "--dest" => {
                i += 1;
                dest = Some(PathBuf::from(
                    args.get(i).ok_or("Usage: synth update apply --dest DIR")?,
                ));
            }
            "--out" => {
                i += 1;
                out = Some(PathBuf::from(
                    args.get(i).ok_or("Usage: synth update keygen --out DIR")?,
                ));
            }
            "--force" => force = true,
            other => return Err(format!("Unknown `synth update` flag `{other}`")),
        }
        i += 1;
    }
    let payload = match action {
        "check" | "--help" | "help" => {
            if action != "check" {
                println!(
                    "Usage:\n  synth update check\n  synth update verify --artifact FILE [--sig FILE.sig]\n  synth update apply --artifact FILE [--sig FILE.sig] [--dest DIR]\n  synth update sign --artifact FILE [--seed-file PATH]\n  synth update keygen [--out DIR] [--force]"
                );
                return Ok(());
            }
            super::update_status(true)
        }
        "verify" => {
            let path = artifact.ok_or("Usage: synth update verify --artifact FILE")?;
            verify_artifact(&path, sig.as_deref())?
        }
        "apply" => {
            let path = artifact.ok_or("Usage: synth update apply --artifact FILE")?;
            apply_artifact(&path, sig.as_deref(), dest.as_deref())?
        }
        "sign" => {
            let path = artifact.ok_or("Usage: synth update sign --artifact FILE")?;
            sign_artifact(&path, seed_file.as_deref())?
        }
        "keygen" => keygen(out.as_deref(), force)?,
        other => {
            return Err(format!(
                "Unknown `synth update` action `{other}` (check, verify, apply, sign, keygen)"
            ))
        }
    };
    println!(
        "{}",
        serde_json::to_string_pretty(&payload).map_err(|e| e.to_string())?
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_public_key_is_32_bytes() {
        let pk = release_public_key().expect("embedded public key");
        assert_eq!(pk.len(), 32);
        assert_eq!(to_hex(&pk), RELEASE_PUBLIC_KEY_HEX.trim());
    }

    #[test]
    fn sign_verify_roundtrip_over_sha256() {
        let mut seed = [0u8; 32];
        fill_os(&mut seed).unwrap();
        let pk = public_from_seed(&seed);
        let digest: [u8; 32] = Sha256::digest(b"synth-release-fixture").into();
        let mut sig = [0u8; 64];
        sign_with_context(&seed, &digest, RELEASE_SIGN_CONTEXT, &mut sig).unwrap();
        verify_with_context(&pk, &digest, RELEASE_SIGN_CONTEXT, &sig).unwrap();
        let mut bad = digest;
        bad[0] ^= 1;
        assert!(verify_with_context(&pk, &bad, RELEASE_SIGN_CONTEXT, &sig).is_err());
    }

    #[test]
    fn apply_writes_sibling_and_refuses_to_clobber_running_exe() {
        let dir = std::env::temp_dir().join(format!(
            "synth-update-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&dir).unwrap();
        let artifact = dir.join("synth-test-artifact.bin");
        fs::write(&artifact, b"hello-synth-release").unwrap();
        let seed = load_seed(None);
        if seed.is_err() {
            let _ = fs::remove_dir_all(&dir);
            return;
        }
        let seed = seed.unwrap();
        if public_from_seed(&seed) != release_public_key().unwrap() {
            let _ = fs::remove_dir_all(&dir);
            return;
        }
        let signed = sign_artifact(&artifact, None).unwrap();
        assert_eq!(signed["schema"], "synth.update-sign/v1");
        let dest_dir = dir.join("install");
        let applied = apply_artifact(&artifact, None, Some(&dest_dir)).unwrap();
        assert_eq!(applied["ok"], true);
        assert_eq!(applied["auto_replace"], false);
        let installed = PathBuf::from(applied["installed"].as_str().unwrap());
        assert_eq!(fs::read(&installed).unwrap(), b"hello-synth-release");
        assert!(!would_overwrite_running_exe(&installed));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn local_seed_matches_embedded_public_key() {
        let Ok(seed) = load_seed(None) else {
            return;
        };
        assert_eq!(public_from_seed(&seed), release_public_key().unwrap());
    }
}
