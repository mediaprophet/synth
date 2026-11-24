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
    io::{Read, Write},
    path::{Path, PathBuf},
    time::Duration,
};

pub const RELEASE_SIGN_CONTEXT: &[u8] = b"synth.release.sha256.v1";
pub const RELEASE_PUBLIC_KEY_HEX: &str = include_str!("../../.synth/update-public.hex");
pub const RELEASES_URL: &str = "https://github.com/mediaprophet/synth/releases";
pub const GITHUB_API_LATEST: &str =
    "https://api.github.com/repos/mediaprophet/synth/releases/latest";
pub const GITHUB_SECRET_NAME: &str = "SYNTH_UPDATE_SEED";
const SEED_FILE_NAME: &str = "update-seed.hex";
const USER_AGENT: &str = "Webizen-AI-Synth/0.1 (https://github.com/mediaprophet/synth)";
const UPGRADE_MAX_BYTES: u64 = 512 * 1024 * 1024;
const GITHUB_TIMEOUT: Duration = Duration::from_secs(20);
const DOWNLOAD_TIMEOUT: Duration = Duration::from_secs(300);

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

pub fn platform_portable_name() -> &'static str {
    #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
    {
        "synth-windows-x86_64.exe"
    }
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    {
        "synth-macos-aarch64"
    }
    #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
    {
        "synth-macos-x86_64"
    }
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    {
        "synth-linux-x86_64"
    }
    #[cfg(not(any(
        all(target_os = "windows", target_arch = "x86_64"),
        all(target_os = "macos", target_arch = "aarch64"),
        all(target_os = "macos", target_arch = "x86_64"),
        all(target_os = "linux", target_arch = "x86_64"),
    )))]
    {
        "synth-unknown"
    }
}

fn installer_name_matches(name: &str) -> bool {
    #[cfg(target_os = "windows")]
    {
        name == "WebizenAI-Synth-Setup-windows-x86_64.exe"
    }
    #[cfg(target_os = "macos")]
    {
        name.starts_with("WebizenAI-Synth-macos-") && name.ends_with(".dmg")
    }
    #[cfg(target_os = "linux")]
    {
        name.starts_with("webizen-ai-synth_") && name.ends_with("_amd64.deb")
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        false
    }
}

fn asset_url(assets: &[Value], name: &str) -> Option<(String, u64)> {
    for asset in assets {
        if asset.get("name").and_then(Value::as_str) != Some(name) {
            continue;
        }
        let url = asset.get("browser_download_url").and_then(Value::as_str)?;
        let size = asset.get("size").and_then(Value::as_u64).unwrap_or(0);
        return Some((url.to_owned(), size));
    }
    None
}

/// Map a GitHub Releases JSON document onto this platform's signed portable.
pub fn parse_github_release(body: &Value) -> Result<Value, String> {
    let tag = body
        .get("tag_name")
        .and_then(Value::as_str)
        .ok_or("GitHub release missing tag_name")?;
    let html_url = body
        .get("html_url")
        .and_then(Value::as_str)
        .unwrap_or(RELEASES_URL);
    let assets = body
        .get("assets")
        .and_then(Value::as_array)
        .ok_or("GitHub release missing assets")?;
    let portable = platform_portable_name();
    let sig_name = format!("{portable}.sig");
    let (portable_url, portable_bytes) = asset_url(assets, portable)
        .ok_or_else(|| format!("latest release {tag} has no {portable}"))?;
    let (sig_url, _) = asset_url(assets, &sig_name)
        .ok_or_else(|| format!("latest release {tag} has no {sig_name}"))?;
    let mut installer = Value::Null;
    let mut installer_url = Value::Null;
    for asset in assets {
        let Some(name) = asset.get("name").and_then(Value::as_str) else {
            continue;
        };
        if !installer_name_matches(name) {
            continue;
        }
        if let Some(url) = asset.get("browser_download_url").and_then(Value::as_str) {
            installer = json!(name);
            installer_url = json!(url);
            break;
        }
    }
    Ok(json!({
        "tag": tag,
        "html_url": html_url,
        "portable": portable,
        "portable_url": portable_url,
        "portable_bytes": portable_bytes,
        "sig": sig_name,
        "sig_url": sig_url,
        "installer": installer,
        "installer_url": installer_url
    }))
}

fn fetch_github_latest() -> Result<Value, String> {
    let agent = ureq::AgentBuilder::new()
        .timeout(GITHUB_TIMEOUT)
        .redirects(5)
        .user_agent(USER_AGENT)
        .build();
    let response = agent
        .get(GITHUB_API_LATEST)
        .set("User-Agent", USER_AGENT)
        .set("Accept", "application/vnd.github+json")
        .set("X-GitHub-Api-Version", "2022-11-28")
        .call()
        .map_err(|e| format!("GitHub Releases API: {e}"))?;
    let mut reader = response.into_reader();
    let mut buf = Vec::new();
    let mut chunk = [0u8; 8192];
    loop {
        let n = reader
            .read(&mut chunk)
            .map_err(|e| format!("GitHub Releases API read: {e}"))?;
        if n == 0 {
            break;
        }
        if buf.len().saturating_add(n) > 1024 * 1024 {
            return Err("GitHub Releases API response exceeds 1 MiB".into());
        }
        buf.extend_from_slice(&chunk[..n]);
    }
    let body: Value =
        serde_json::from_slice(&buf).map_err(|e| format!("GitHub Releases JSON: {e}"))?;
    parse_github_release(&body)
}

/// Latest GitHub Release for this platform. `fetch_remote=false` stays Null so
/// the operator poll does not hit api.github.com every few seconds.
pub fn github_release_status(fetch_remote: bool) -> Value {
    if !fetch_remote {
        return Value::Null;
    }
    match fetch_github_latest() {
        Ok(v) => v,
        Err(e) => json!({ "error": e }),
    }
}

fn update_cache_dir() -> Result<PathBuf, String> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let dir = exe
        .parent()
        .ok_or_else(|| "current exe has no parent directory".to_string())?
        .join("updates");
    fs::create_dir_all(&dir).map_err(|e| format!("create {}: {e}", dir.display()))?;
    Ok(dir)
}

fn upgrade_install_dest() -> Result<PathBuf, String> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let dir = exe
        .parent()
        .ok_or_else(|| "current exe has no parent directory".to_string())?
        .to_path_buf();
    let stem = exe
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("synth");
    let ext = exe
        .extension()
        .and_then(|s| s.to_str())
        .map(|e| format!(".{e}"))
        .unwrap_or_default();
    Ok(dir.join(format!("{stem}.new{ext}")))
}

fn download_to_file(url: &str, dest: &Path, max_bytes: u64) -> Result<u64, String> {
    if !(url.starts_with("https://") || url.starts_with("http://")) {
        return Err("update URL must be http(s)".into());
    }
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("create {}: {e}", parent.display()))?;
    }
    let agent = ureq::AgentBuilder::new()
        .timeout(DOWNLOAD_TIMEOUT)
        .redirects(8)
        .user_agent(USER_AGENT)
        .build();
    let response = agent
        .get(url)
        .set("User-Agent", USER_AGENT)
        .set("Accept", "application/octet-stream")
        .call()
        .map_err(|e| format!("download {url}: {e}"))?;
    let final_url = response.get_url().to_owned();
    if !(final_url.starts_with("https://") || final_url.starts_with("http://")) {
        return Err("redirect left http(s)".into());
    }
    if let Some(len) = response
        .header("Content-Length")
        .and_then(|s| s.parse::<u64>().ok())
    {
        if len > max_bytes {
            return Err(format!("artifact is {len} bytes; cap is {max_bytes}"));
        }
    }
    let tmp = {
        let mut name = dest.file_name().unwrap_or_default().to_os_string();
        name.push(".part");
        dest.with_file_name(name)
    };
    let mut file = fs::File::create(&tmp).map_err(|e| format!("create {}: {e}", tmp.display()))?;
    let mut reader = response.into_reader();
    let mut written = 0u64;
    let mut buf = [0u8; 64 * 1024];
    loop {
        let n = reader.read(&mut buf).map_err(|e| {
            let _ = fs::remove_file(&tmp);
            format!("read {url}: {e}")
        })?;
        if n == 0 {
            break;
        }
        written = written.saturating_add(n as u64);
        if written > max_bytes {
            let _ = fs::remove_file(&tmp);
            return Err(format!("download exceeded {max_bytes} bytes"));
        }
        file.write_all(&buf[..n]).map_err(|e| {
            let _ = fs::remove_file(&tmp);
            format!("write {}: {e}", tmp.display())
        })?;
    }
    drop(file);
    fs::rename(&tmp, dest).map_err(|e| format!("rename {}: {e}", dest.display()))?;
    Ok(written)
}

/// Atomically or live-rename-swap a verified binary onto a target path (e.g. `synth.exe` or `synth-mcp.exe`),
/// moving any running locked binary aside to `<stem>.prev<ext>` so that Windows and Unix can place the new
/// binary at the canonical executable path immediately while keeping `<stem>.new<ext>` staged.
fn try_live_rename_swap_target(verified_artifact: &Path, target: &Path) -> Value {
    let stem = target
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("synth");
    let ext = target
        .extension()
        .and_then(|s| s.to_str())
        .map(|e| format!(".{e}"))
        .unwrap_or_default();
    let parent = target.parent().unwrap_or_else(|| Path::new("."));
    let prev_path = parent.join(format!("{stem}.prev{ext}"));

    // 1. If a direct overwrite works (unlocked file), do it directly
    if !would_overwrite_running_exe(target) {
        if fs::copy(verified_artifact, target).is_ok() {
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let _ = fs::set_permissions(target, fs::Permissions::from_mode(0o755));
            }
            return json!({
                "target": target.display().to_string(),
                "status": "replaced_direct"
            });
        }
    }

    // 2. Windows & Unix live-rename dance: clean up any old unlocked `.prev`, rename live target -> `.prev`, then copy new binary to target
    let _ = fs::remove_file(&prev_path);
    if target.exists() && fs::rename(target, &prev_path).is_ok() {
        match fs::copy(verified_artifact, target) {
            Ok(_) => {
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let _ = fs::set_permissions(target, fs::Permissions::from_mode(0o755));
                }
                return json!({
                    "target": target.display().to_string(),
                    "moved_running_to": prev_path.display().to_string(),
                    "status": "live_rename_swapped"
                });
            }
            Err(e) => {
                // Roll back rename if copy failed
                let _ = fs::rename(&prev_path, target);
                return json!({
                    "target": target.display().to_string(),
                    "status": "staged_only",
                    "detail": e.to_string()
                });
            }
        }
    }

    json!({
        "target": target.display().to_string(),
        "status": "staged_only"
    })
}

/// Download the latest signed portable, verify Ed25519, copy to a sibling
/// `synth.new.exe` (or platform equivalent), and live-rename-swap canonical
/// `synth` / `synth-mcp` binaries in `dest_dir` (moving any running locked binary
/// aside to `.prev.exe` without overwriting its mapped image in-place).
pub fn upgrade_latest() -> Result<Value, String> {
    upgrade_latest_in(None)
}

pub fn upgrade_latest_in(dest_dir: Option<&Path>) -> Result<Value, String> {
    let release = fetch_github_latest()?;
    let portable_url = release
        .get("portable_url")
        .and_then(Value::as_str)
        .ok_or("GitHub release missing portable_url")?;
    let sig_url = release
        .get("sig_url")
        .and_then(Value::as_str)
        .ok_or("GitHub release missing sig_url")?;
    let portable_name = release
        .get("portable")
        .and_then(Value::as_str)
        .unwrap_or_else(|| platform_portable_name());
    let sig_name = release
        .get("sig")
        .and_then(Value::as_str)
        .map(str::to_owned)
        .unwrap_or_else(|| format!("{portable_name}.sig"));
    let cache = update_cache_dir()?;
    let artifact = cache.join(portable_name);
    let sig = cache.join(sig_name);
    let downloaded = download_to_file(portable_url, &artifact, UPGRADE_MAX_BYTES)?;
    download_to_file(sig_url, &sig, 4096)?;
    let verified = verify_artifact(&artifact, Some(&sig))?;
    let new_sha = verified
        .get("sha256")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_owned();
    let current_sha = std::env::current_exe()
        .ok()
        .and_then(|p| sha256_file(&p).ok())
        .map(|d| to_hex(&d));
    if dest_dir.is_none() && current_sha.as_deref() == Some(new_sha.as_str()) {
        return Ok(json!({
            "schema": "synth.update-upgrade/v1",
            "ok": true,
            "already_current": true,
            "tag": release.get("tag"),
            "sha256": new_sha,
            "artifact": artifact.display().to_string(),
            "auto_replace": false,
            "releases_url": release.get("html_url").cloned().unwrap_or_else(|| json!(RELEASES_URL)),
            "policy": "Running binary already matches the latest signed portable. The exe was not replaced."
        }));
    }
    let dest = if let Some(dir) = dest_dir {
        fs::create_dir_all(dir).map_err(|e| format!("create {}: {e}", dir.display()))?;
        let ext = if cfg!(windows) { ".exe" } else { "" };
        dir.join(format!("synth.new{ext}"))
    } else {
        upgrade_install_dest()?
    };
    if would_overwrite_running_exe(&dest) {
        return Err(
            "refusing to overwrite the running synth.exe; drop the MCP child or copy the sibling .new file"
                .into(),
        );
    }
    fs::copy(&artifact, &dest).map_err(|e| format!("copy to {}: {e}", dest.display()))?;

    // Also attempt safe live-rename swap onto canonical `synth` and `synth-mcp` in the target directory
    let target_dir = dest
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));
    let ext = if cfg!(windows) { ".exe" } else { "" };
    let synth_bin = target_dir.join(format!("synth{ext}"));
    let synth_mcp_bin = target_dir.join(format!("synth-mcp{ext}"));
    let mut swapped = Vec::new();
    swapped.push(try_live_rename_swap_target(&artifact, &synth_bin));
    if synth_mcp_bin.exists() || target_dir.join("synth-mcp.exe").exists() {
        swapped.push(try_live_rename_swap_target(&artifact, &synth_mcp_bin));
    }

    Ok(json!({
        "schema": "synth.update-upgrade/v1",
        "ok": true,
        "already_current": false,
        "tag": release.get("tag"),
        "artifact": artifact.display().to_string(),
        "installed": dest.display().to_string(),
        "live_swapped_targets": swapped,
        "sha256": new_sha,
        "bytes": downloaded,
        "installer": release.get("installer"),
        "installer_url": release.get("installer_url"),
        "auto_replace": false,
        "releases_url": release.get("html_url").cloned().unwrap_or_else(|| json!(RELEASES_URL)),
        "policy": "Verified sibling written (.new) and canonical synth/synth-mcp live-rename-swapped (.prev). Reload MCP (/mcps reload) to spawn the updated binary."
    }))
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
        "upgrade" => upgrade_latest_in(arg.dest.as_deref().map(Path::new)),
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
            "unknown update action `{other}` (check, upgrade, verify, apply)"
        )),
    }
}

/// `synth update check|upgrade|verify|apply|sign|keygen`
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
                    "Usage:\n  synth update check\n  synth update upgrade\n  synth update verify --artifact FILE [--sig FILE.sig]\n  synth update apply --artifact FILE [--sig FILE.sig] [--dest DIR]\n  synth update sign --artifact FILE [--seed-file PATH]\n  synth update keygen [--out DIR] [--force]"
                );
                return Ok(());
            }
            super::update_status(true)
        }
        "upgrade" => upgrade_latest_in(dest.as_deref())?,
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
                "Unknown `synth update` action `{other}` (check, upgrade, verify, apply, sign, keygen)"
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

    #[test]
    fn parse_github_release_picks_this_platform_portable_and_sig() {
        let portable = platform_portable_name();
        let sig = format!("{portable}.sig");
        let body = json!({
            "tag_name": "v0.0.41.8",
            "html_url": "https://github.com/mediaprophet/synth/releases/tag/v0.0.41.8",
            "assets": [
                {
                    "name": "synth-windows-x86_64.exe",
                    "browser_download_url": "https://github.com/mediaprophet/synth/releases/download/v0.0.41.8/synth-windows-x86_64.exe",
                    "size": 12
                },
                {
                    "name": "synth-windows-x86_64.exe.sig",
                    "browser_download_url": "https://github.com/mediaprophet/synth/releases/download/v0.0.41.8/synth-windows-x86_64.exe.sig",
                    "size": 129
                },
                {
                    "name": "synth-macos-aarch64",
                    "browser_download_url": "https://github.com/mediaprophet/synth/releases/download/v0.0.41.8/synth-macos-aarch64",
                    "size": 11
                },
                {
                    "name": "synth-macos-aarch64.sig",
                    "browser_download_url": "https://github.com/mediaprophet/synth/releases/download/v0.0.41.8/synth-macos-aarch64.sig",
                    "size": 129
                },
                {
                    "name": "synth-macos-x86_64",
                    "browser_download_url": "https://github.com/mediaprophet/synth/releases/download/v0.0.41.8/synth-macos-x86_64",
                    "size": 11
                },
                {
                    "name": "synth-macos-x86_64.sig",
                    "browser_download_url": "https://github.com/mediaprophet/synth/releases/download/v0.0.41.8/synth-macos-x86_64.sig",
                    "size": 129
                },
                {
                    "name": "synth-linux-x86_64",
                    "browser_download_url": "https://github.com/mediaprophet/synth/releases/download/v0.0.41.8/synth-linux-x86_64",
                    "size": 10
                },
                {
                    "name": "synth-linux-x86_64.sig",
                    "browser_download_url": "https://github.com/mediaprophet/synth/releases/download/v0.0.41.8/synth-linux-x86_64.sig",
                    "size": 129
                },
                {
                    "name": "WebizenAI-Synth-Setup-windows-x86_64.exe",
                    "browser_download_url": "https://github.com/mediaprophet/synth/releases/download/v0.0.41.8/WebizenAI-Synth-Setup-windows-x86_64.exe",
                    "size": 20
                },
                {
                    "name": "WebizenAI-Synth-macos-aarch64.dmg",
                    "browser_download_url": "https://github.com/mediaprophet/synth/releases/download/v0.0.41.8/WebizenAI-Synth-macos-aarch64.dmg",
                    "size": 21
                },
                {
                    "name": "webizen-ai-synth_0.0.41.8_amd64.deb",
                    "browser_download_url": "https://github.com/mediaprophet/synth/releases/download/v0.0.41.8/webizen-ai-synth_0.0.41.8_amd64.deb",
                    "size": 22
                }
            ]
        });
        let parsed = parse_github_release(&body).expect("platform assets present");
        assert_eq!(parsed["tag"], "v0.0.41.8");
        assert_eq!(parsed["portable"], portable);
        assert_eq!(parsed["sig"], sig);
        assert!(parsed["portable_url"].as_str().unwrap().ends_with(portable));
        assert!(parsed["sig_url"].as_str().unwrap().ends_with(&sig));
        assert_eq!(parsed["auto_replace"], Value::Null);
        assert!(parsed["installer"].is_string() || parsed["installer"].is_null());
    }

    #[test]
    fn parse_github_release_fails_closed_without_this_platform_sig() {
        let body = json!({
            "tag_name": "v0.0.41.8",
            "html_url": "https://github.com/mediaprophet/synth/releases/tag/v0.0.41.8",
            "assets": []
        });
        let err = parse_github_release(&body).unwrap_err();
        assert!(err.contains(platform_portable_name()));
    }

    #[test]
    fn upgrade_install_dest_is_sibling_new_not_running_exe() {
        let dest = upgrade_install_dest().expect("exe parent");
        let exe = std::env::current_exe().unwrap();
        assert_ne!(dest, exe);
        assert!(dest
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .contains(".new"));
        assert!(!would_overwrite_running_exe(&dest));
    }
}
