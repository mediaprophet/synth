//! Downloader, inbox, and scheduler for Synth jobs.
//!
//! Jobs live in `.synth/jobs.json`. Each run writes an immutable snapshot under
//! `.synth/inbox/{job_id}/{run_id}/` (payload, PROV/DC wrap Turtle, optional `.q42`).
//! The operator process ticks due jobs. MCP `synth mcp` does not start the scheduler.
//! Inbox is staging, not corpus. Do not invent an HTTP CAS.
use super::bookmarks;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    collections::HashSet,
    fs,
    io::Read,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, AtomicU32, Ordering},
        Mutex, OnceLock,
    },
    thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

const SCHEMA: &str = "synth.jobs/v1";
const MAX_JOBS: usize = 64;
const MAX_URL: usize = 2048;
const MAX_QUERY: usize = 32 * 1024;
const MAX_BYTES: usize = 16 * 1024 * 1024;
const MAX_RUNS_KEPT: usize = 8;
const MIN_INTERVAL_SECS: u64 = 60;
const FETCH_TIMEOUT: Duration = Duration::from_secs(30);
const USER_AGENT: &str = "Webizen-AI-Synth/0.1 (https://github.com/mediaprophet/synth)";
const GITIGNORE_INBOX: &str = "inbox/\n";

static RUN_SEQ: AtomicU32 = AtomicU32::new(1);
static RUNNING: OnceLock<Mutex<HashSet<String>>> = OnceLock::new();
static SCHEDULER_STARTED: AtomicBool = AtomicBool::new(false);

fn running_set() -> &'static Mutex<HashSet<String>> {
    RUNNING.get_or_init(|| Mutex::new(HashSet::new()))
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub id: String,
    pub kind: String,
    #[serde(default)]
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bookmark_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub query: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub out_q42: Option<String>,
    #[serde(default)]
    pub interval_secs: u64,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub created_at: u64,
    #[serde(default)]
    pub updated_at: u64,
    #[serde(default)]
    pub next_run_at: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_run_at: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_status: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_error: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_sha256: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_run_dir: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_q42: Option<String>,
    #[serde(default)]
    pub run_count: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct JobStore {
    schema: String,
    items: Vec<Job>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(default, deny_unknown_fields)]
pub struct JobArg {
    pub action: Option<String>,
    pub id: Option<String>,
    pub kind: Option<String>,
    pub title: Option<String>,
    pub url: Option<String>,
    pub bookmark_id: Option<String>,
    pub source_path: Option<String>,
    pub query: Option<String>,
    pub out_q42: Option<String>,
    pub interval_secs: Option<u64>,
    pub enabled: Option<bool>,
}

pub fn dispatch(arg: JobArg) -> Result<Value, String> {
    let action = arg.action.as_deref().unwrap_or("list").to_ascii_lowercase();
    match action.as_str() {
        "list" => {
            let dir = store_dir()?;
            Ok(public_list(&dir))
        }
        "get" => {
            let id = arg.id.as_deref().ok_or("get requires id")?;
            let dir = store_dir()?;
            let store = load_store(&dir);
            let job = store
                .items
                .iter()
                .find(|j| j.id == id)
                .ok_or_else(|| format!("Unknown job `{id}`"))?;
            Ok(json!({"schema": SCHEMA, "item": job}))
        }
        "forget" => {
            let id = arg.id.as_deref().ok_or("forget requires id")?;
            let dir = store_dir()?;
            let mut store = load_store(&dir);
            let before = store.items.len();
            store.items.retain(|j| j.id != id);
            if store.items.len() == before {
                return Err(format!("Unknown job `{id}`"));
            }
            save_store(&dir, &store)?;
            Ok(json!({"ok": true, "forgotten": id}))
        }
        "enable" | "disable" => {
            let id = arg.id.as_deref().ok_or(format!("{action} requires id"))?;
            let dir = store_dir()?;
            let mut store = load_store(&dir);
            let job = store
                .items
                .iter_mut()
                .find(|j| j.id == id)
                .ok_or_else(|| format!("Unknown job `{id}`"))?;
            job.enabled = action == "enable";
            job.updated_at = now_secs();
            if job.enabled && job.next_run_at == 0 {
                job.next_run_at = now_secs();
            }
            let snapshot = job.clone();
            save_store(&dir, &store)?;
            Ok(json!({"ok": true, "item": snapshot}))
        }
        "add" | "upsert" => upsert(arg),
        "run" => {
            let id = arg.id.as_deref().ok_or("run requires id")?;
            run_job(id, true)
        }
        "run_due" | "run-due" => run_due(),
        other => Err(format!(
            "Unknown jobs action `{other}` (list, get, add, upsert, forget, enable, disable, run, run_due)"
        )),
    }
}

pub fn summary() -> Value {
    match store_dir() {
        Ok(dir) => {
            let store = load_store(&dir);
            let now = now_secs();
            let enabled = store.items.iter().filter(|j| j.enabled).count();
            let due = store
                .items
                .iter()
                .filter(|j| j.enabled && j.next_run_at != 0 && j.next_run_at <= now)
                .count();
            json!({
                "configured": true,
                "schema": SCHEMA,
                "path": dir.join("jobs.json").display().to_string(),
                "inbox": dir.join("inbox").display().to_string(),
                "count": store.items.len(),
                "enabled": enabled,
                "due": due,
                "scheduler": SCHEDULER_STARTED.load(Ordering::Relaxed),
            })
        }
        Err(_) => json!({
            "configured": false,
            "schema": SCHEMA,
            "count": 0,
            "enabled": 0,
            "due": 0,
            "scheduler": SCHEDULER_STARTED.load(Ordering::Relaxed),
        }),
    }
}

/// Start the operator scheduler once per process. MCP stdio does not call this.
pub fn spawn_scheduler() {
    if SCHEDULER_STARTED.swap(true, Ordering::SeqCst) {
        return;
    }
    match thread::Builder::new()
        .name("synth-job-scheduler".into())
        .spawn(|| loop {
            let _ = tick();
            thread::sleep(Duration::from_secs(5));
        }) {
        Ok(_) => {}
        Err(err) => {
            SCHEDULER_STARTED.store(false, Ordering::SeqCst);
            eprintln!("Synth: job scheduler thread failed: {err}");
        }
    }
}

pub fn tick() -> Result<Value, String> {
    run_due()
}

fn store_dir() -> Result<PathBuf, String> {
    bookmarks::store_dir()
}

fn ensure_store(dir: &Path) -> Result<(), String> {
    fs::create_dir_all(dir).map_err(|e| format!(".synth: {e}"))?;
    fs::create_dir_all(dir.join("inbox")).map_err(|e| format!(".synth/inbox: {e}"))?;
    let gi = dir.join(".gitignore");
    if gi.is_file() {
        let existing = fs::read_to_string(&gi).unwrap_or_default();
        if !existing.contains("inbox/") {
            fs::write(&gi, existing + GITIGNORE_INBOX).map_err(|e| format!(".synth/.gitignore: {e}"))?;
        }
    } else {
        fs::write(
            &gi,
            "# Local secrets and acquisition inbox. Never commit to GitHub.\nsecrets.json\nsecrets.*.json\n*.key\n*.pem\nupdate-seed.hex\n*-seed.hex\ninbox/\n",
        )
        .map_err(|e| format!(".synth/.gitignore: {e}"))?;
    }
    Ok(())
}

fn load_store(dir: &Path) -> JobStore {
    let path = dir.join("jobs.json");
    let Ok(bytes) = fs::read(&path) else {
        return JobStore {
            schema: SCHEMA.into(),
            items: vec![],
        };
    };
    serde_json::from_slice::<JobStore>(&bytes).unwrap_or(JobStore {
        schema: SCHEMA.into(),
        items: vec![],
    })
}

fn save_store(dir: &Path, store: &JobStore) -> Result<(), String> {
    ensure_store(dir)?;
    let bytes = serde_json::to_vec_pretty(store).map_err(|e| e.to_string())?;
    fs::write(dir.join("jobs.json"), bytes).map_err(|e| format!("jobs.json: {e}"))
}

fn public_list(dir: &Path) -> Value {
    let store = load_store(dir);
    json!({
        "schema": SCHEMA,
        "path": dir.join("jobs.json").display().to_string(),
        "inbox": dir.join("inbox").display().to_string(),
        "count": store.items.len(),
        "items": store.items,
        "policy": "Jobs fetch http(s) into .synth/inbox (gitignored staging). Wrap with PROV/DCT/DCAT. Compile Turtle to .q42. Scheduler runs in synth operator / serve, not synth mcp. Max 16 MiB. Secrets stay in .synth/secrets.json.",
    })
}

fn slug_id(raw: &str) -> String {
    let mut out = String::new();
    for ch in raw.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
        } else if matches!(ch, '.' | '/' | '-' | '_' | ':') && !out.ends_with('-') {
            out.push('-');
        }
        if out.len() >= 80 {
            break;
        }
    }
    let trimmed = out.trim_matches('-');
    if trimmed.is_empty() {
        "job".into()
    } else {
        trimmed.to_owned()
    }
}

fn unique_id(preferred: &str, existing: &[Job]) -> String {
    let base = if preferred.is_empty() {
        "job".into()
    } else {
        slug_id(preferred)
    };
    if !existing.iter().any(|j| j.id == base) {
        return base;
    }
    for n in 2..=MAX_JOBS + 2 {
        let candidate = format!("{base}-{n}");
        if !existing.iter().any(|j| j.id == candidate) {
            return candidate;
        }
    }
    format!("{base}-x")
}

fn validate_kind(kind: &str) -> Result<(), String> {
    match kind {
        "fetch" | "ingest" | "compile" => Ok(()),
        other => Err(format!(
            "Unknown job kind `{other}` (fetch, ingest, compile)"
        )),
    }
}

fn validate_out_q42(name: &str) -> Result<(), String> {
    if name.contains('/') || name.contains('\\') {
        return Err("out_q42 must be a basename (no directory components)".into());
    }
    let ok = name.len() <= 96
        && name.ends_with(".q42")
        && name.chars().next().is_some_and(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
        && name.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.'));
    if !ok {
        return Err("out_q42 must match [A-Za-z0-9_-][A-Za-z0-9._-]*.q42".into());
    }
    Ok(())
}

fn upsert(arg: JobArg) -> Result<Value, String> {
    let dir = store_dir()?;
    ensure_store(&dir)?;
    let mut store = load_store(&dir);
    store.schema = SCHEMA.into();
    let ts = now_secs();
    if let Some(id) = arg.id.as_deref() {
        if let Some(existing) = store.items.iter_mut().find(|j| j.id == id) {
            if let Some(kind) = arg.kind.as_deref() {
                validate_kind(kind)?;
                existing.kind = kind.to_owned();
            }
            if let Some(title) = arg.title {
                existing.title = title;
            }
            if let Some(url) = arg.url.as_deref() {
                let (clean, saw_secret) = bookmarks::sanitize_url(url)?;
                if saw_secret {
                    return Err(
                        "URL had a credential query parameter; store the key with synth bookmark set_secret"
                            .into(),
                    );
                }
                existing.url = Some(clean);
            }
            if arg.bookmark_id.is_some() {
                existing.bookmark_id = arg.bookmark_id;
            }
            if arg.source_path.is_some() {
                existing.source_path = arg.source_path;
            }
            if let Some(query) = arg.query {
                if query.len() > MAX_QUERY {
                    return Err("query exceeds 32 KiB".into());
                }
                existing.query = Some(query);
            }
            if let Some(out) = arg.out_q42.as_deref() {
                validate_out_q42(out)?;
                existing.out_q42 = Some(out.to_owned());
            }
            if let Some(interval) = arg.interval_secs {
                existing.interval_secs = interval;
            }
            if let Some(enabled) = arg.enabled {
                existing.enabled = enabled;
            }
            existing.updated_at = ts;
            let snapshot = existing.clone();
            save_store(&dir, &store)?;
            return Ok(json!({"ok": true, "updated": true, "item": snapshot}));
        }
    }

    if store.items.len() >= MAX_JOBS {
        return Err("Job store is full (64)".into());
    }
    let kind = arg.kind.unwrap_or_else(|| {
        if arg.source_path.is_some() && arg.url.is_none() {
            "compile".into()
        } else {
            "ingest".into()
        }
    });
    validate_kind(&kind)?;
    if kind != "compile" && arg.url.is_none() && arg.bookmark_id.is_none() {
        return Err("fetch/ingest jobs require url or bookmark_id".into());
    }
    if kind == "compile" && arg.source_path.is_none() && arg.url.is_none() && arg.bookmark_id.is_none()
    {
        return Err("compile jobs require source_path or url".into());
    }
    let url = match arg.url.as_deref() {
        Some(raw) => {
            let (clean, saw_secret) = bookmarks::sanitize_url(raw)?;
            if saw_secret {
                return Err(
                    "URL had a credential query parameter; store the key with synth bookmark set_secret"
                        .into(),
                );
            }
            Some(clean)
        }
        None => None,
    };
    if let Some(out) = arg.out_q42.as_deref() {
        validate_out_q42(out)?;
    }
    if let Some(query) = arg.query.as_deref() {
        if query.len() > MAX_QUERY {
            return Err("query exceeds 32 KiB".into());
        }
    }
    let preferred = arg.id.clone().unwrap_or_else(|| {
        url.as_deref()
            .or(arg.bookmark_id.as_deref())
            .or(arg.source_path.as_deref())
            .map(slug_id)
            .unwrap_or_else(|| "job".into())
    });
    let id = unique_id(&preferred, &store.items);
    let title = arg.title.unwrap_or_else(|| id.clone());
    let enabled = arg.enabled.unwrap_or(true);
    let interval = arg.interval_secs.unwrap_or(0);
    let job = Job {
        id,
        kind,
        title,
        url,
        bookmark_id: arg.bookmark_id,
        source_path: arg.source_path,
        query: arg.query,
        out_q42: arg.out_q42,
        interval_secs: interval,
        enabled,
        created_at: ts,
        updated_at: ts,
        next_run_at: if enabled { ts } else { 0 },
        last_run_at: None,
        last_status: None,
        last_error: None,
        last_sha256: None,
        last_run_dir: None,
        last_q42: None,
        run_count: 0,
    };
    let public = job.clone();
    store.items.push(job);
    save_store(&dir, &store)?;
    Ok(json!({"ok": true, "updated": false, "item": public}))
}

fn run_due() -> Result<Value, String> {
    let dir = match store_dir() {
        Ok(d) => d,
        Err(_) => {
            return Ok(json!({
                "schema": SCHEMA,
                "ok": true,
                "ran": [],
                "message": "No job store (set SYNTH_OUTPUT_ROOT, SYNTH_BOOKMARK_ROOT, or attach a project)"
            }));
        }
    };
    let store = load_store(&dir);
    let now = now_secs();
    let due: Vec<String> = store
        .items
        .iter()
        .filter(|j| j.enabled && j.next_run_at != 0 && j.next_run_at <= now)
        .map(|j| j.id.clone())
        .collect();
    let mut ran = Vec::new();
    for id in &due {
        match run_job(id, false) {
            Ok(value) => ran.push(value),
            Err(err) => ran.push(json!({"id": id, "ok": false, "error": err})),
        }
    }
    Ok(json!({
        "schema": SCHEMA,
        "ok": true,
        "due": due.len(),
        "ran": ran,
    }))
}

fn run_job(id: &str, manual: bool) -> Result<Value, String> {
    {
        let mut guard = running_set()
            .lock()
            .map_err(|_| "job runner lock poisoned".to_string())?;
        if !guard.insert(id.to_owned()) {
            return Err(format!("Job `{id}` is already running"));
        }
    }
    let result = run_job_inner(id, manual);
    if let Ok(mut guard) = running_set().lock() {
        guard.remove(id);
    }
    result
}

fn run_job_inner(id: &str, manual: bool) -> Result<Value, String> {
    let dir = store_dir()?;
    ensure_store(&dir)?;
    let store = load_store(&dir);
    let job = store
        .items
        .iter()
        .find(|j| j.id == id)
        .cloned()
        .ok_or_else(|| format!("Unknown job `{id}`"))?;
    if !manual && !job.enabled {
        return Err(format!("Job `{id}` is disabled"));
    }
    let started = now_secs();
    let run_id = format!(
        "{started}-{}-{}",
        std::process::id(),
        RUN_SEQ.fetch_add(1, Ordering::Relaxed)
    );
    let run_dir = dir.join("inbox").join(&job.id).join(&run_id);
    fs::create_dir_all(&run_dir).map_err(|e| format!("inbox mkdir: {e}"))?;
    prune_runs(&dir.join("inbox").join(&job.id), &run_id);

    let outcome = execute_job(&job, &run_dir, started);
    let ended = now_secs();
    let mut store = load_store(&dir);
    let Some(slot) = store.items.iter_mut().find(|j| j.id == id) else {
        return Err(format!("Job `{id}` disappeared during run"));
    };
    slot.last_run_at = Some(ended);
    slot.run_count = slot.run_count.saturating_add(1);
    slot.updated_at = ended;
    slot.last_run_dir = Some(run_dir.display().to_string());
    match &outcome {
        Ok(meta) => {
            slot.last_status = Some("ok".into());
            slot.last_error = None;
            slot.last_sha256 = meta.sha256.clone();
            slot.last_q42 = meta.q42_path.clone();
            if slot.interval_secs == 0 {
                slot.enabled = false;
                slot.next_run_at = 0;
            } else {
                let interval = slot.interval_secs.max(MIN_INTERVAL_SECS);
                slot.next_run_at = ended.saturating_add(interval);
            }
        }
        Err(err) => {
            slot.last_status = Some("error".into());
            slot.last_error = Some(err.clone());
            if slot.interval_secs == 0 {
                slot.enabled = false;
                slot.next_run_at = 0;
            } else {
                let interval = slot.interval_secs.max(MIN_INTERVAL_SECS);
                slot.next_run_at = ended.saturating_add(interval);
            }
        }
    }
    let snapshot = slot.clone();
    save_store(&dir, &store)?;
    match outcome {
        Ok(meta) => {
            crate::operator::record_event("synth_jobs", true, Some(id));
            Ok(json!({
                "ok": true,
                "id": id,
                "run_id": run_id,
                "kind": job.kind,
                "started_at": started,
                "ended_at": ended,
                "run_dir": run_dir.display().to_string(),
                "bytes": meta.bytes,
                "content_type": meta.content_type,
                "sha256": meta.sha256,
                "source_url": meta.source_url,
                "wrap": meta.wrap_path,
                "q42": meta.q42_path,
                "promoted_q42": meta.promoted_q42,
                "quin_count": meta.quin_count,
                "item": snapshot,
            }))
        }
        Err(err) => {
            crate::operator::record_event("synth_jobs", false, Some(id));
            let _ = fs::write(run_dir.join("error.txt"), err.as_bytes());
            Err(format!("Job `{id}` failed: {err}"))
        }
    }
}

struct RunMeta {
    bytes: usize,
    content_type: String,
    sha256: Option<String>,
    source_url: Option<String>,
    wrap_path: Option<String>,
    q42_path: Option<String>,
    promoted_q42: Option<String>,
    quin_count: Option<usize>,
}

fn execute_job(job: &Job, run_dir: &Path, started: u64) -> Result<RunMeta, String> {
    match job.kind.as_str() {
        "fetch" => {
            let fetched = obtain_bytes(job)?;
            write_snapshot(run_dir, job, &fetched, started, false)
        }
        "ingest" => {
            let fetched = obtain_bytes(job)?;
            write_snapshot(run_dir, job, &fetched, started, true)
        }
        "compile" => {
            if let Some(rel) = job.source_path.as_deref() {
                let path = resolve_input_path(rel)?;
                let len = fs::metadata(&path)
                    .map(|m| m.len())
                    .map_err(|e| format!("stat {}: {e}", path.display()))?;
                if len > MAX_BYTES as u64 {
                    return execute_streaming_local_compile_job(job, &path, len, run_dir, started);
                }
                let fetched = read_local(job)?;
                write_snapshot(run_dir, job, &fetched, started, true)
            } else {
                let fetched = obtain_bytes(job)?;
                write_snapshot(run_dir, job, &fetched, started, true)
            }
        }
        other => Err(format!("Unknown job kind `{other}`")),
    }
}

struct Fetched {
    bytes: Vec<u8>,
    content_type: String,
    url: String,
    filename: String,
}

fn obtain_bytes(job: &Job) -> Result<Fetched, String> {
    let mut secret = None;
    let mut url = job.url.clone();
    if let Some(bid) = job.bookmark_id.as_deref() {
        let bm = bookmarks::get_bookmark(bid)?.ok_or_else(|| format!("Unknown bookmark `{bid}`"))?;
        if url.is_none() {
            url = bm
                .data_endpoint
                .clone()
                .filter(|s| !s.is_empty())
                .or(Some(bm.url.clone()));
        }
        if bm.requires_api_key {
            secret = bookmarks::secret_value(bid)?;
            if secret.is_none() {
                if let Some(env_name) = bm.api_key_env.as_deref() {
                    if let Ok(v) = std::env::var(env_name) {
                        if !v.is_empty() {
                            secret = Some(v);
                        }
                    }
                }
            }
            if secret.is_none() {
                return Err(format!(
                    "Bookmark `{bid}` requires an API key (synth_bookmarks action=set_secret)"
                ));
            }
        }
    }
    let url = url.ok_or("job has no url")?;
    if url.len() > MAX_URL {
        return Err("URL exceeds 2048 bytes".into());
    }
    download(&url, job.query.as_deref(), secret.as_deref())
}

fn read_local(job: &Job) -> Result<Fetched, String> {
    let rel = job.source_path.as_deref().ok_or("compile requires source_path")?;
    let path = resolve_input_path(rel)?;
    let bytes = fs::read(&path).map_err(|e| format!("read {}: {e}", path.display()))?;
    if bytes.len() > MAX_BYTES {
        return Err(format!("source is {} bytes; max is {MAX_BYTES}", bytes.len()));
    }
    let filename = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("payload.bin")
        .to_owned();
    let content_type = guess_content_type(&filename, "");
    Ok(Fetched {
        bytes,
        content_type,
        url: file_url(&path),
        filename,
    })
}

fn file_url(path: &Path) -> String {
    let mut s = path.display().to_string().replace('\\', "/");
    if let Some(rest) = s.strip_prefix("//?/") {
        s = rest.to_owned();
    }
    if s.starts_with('/') {
        format!("file://{s}")
    } else {
        format!("file:///{s}")
    }
}

fn resolve_input_path(rel: &str) -> Result<PathBuf, String> {
    let raw = Path::new(rel);
    let root = std::env::var("SYNTH_INPUT_ROOT")
        .map_err(|_| "SYNTH_INPUT_ROOT is required to compile a source_path".to_string())?;
    let root = PathBuf::from(root.trim());
    let joined = if raw.is_absolute() {
        raw.to_path_buf()
    } else {
        root.join(rel)
    };
    let canon = fs::canonicalize(&joined).unwrap_or(joined);
    let root_canon = fs::canonicalize(&root).unwrap_or(root);
    if !canon.starts_with(&root_canon) {
        return Err("source_path escaped SYNTH_INPUT_ROOT".into());
    }
    if !canon.is_file() {
        return Err(format!("source_path is not a file: {}", canon.display()));
    }
    Ok(canon)
}

fn execute_streaming_local_compile_job(
    job: &Job,
    source_path: &Path,
    file_len: u64,
    run_dir: &Path,
    started: u64,
) -> Result<RunMeta, String> {
    let filename = source_path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("payload.owl")
        .to_owned();
    let content_type = guess_content_type(&filename, "");
    let url = file_url(source_path);
    let sha = crate::mcp_full::q42::sha256_file(source_path).map_err(|e| e.message)?;
    let ended = now_secs();
    let fake_fetched = Fetched {
        bytes: Vec::new(),
        content_type: content_type.clone(),
        url: url.clone(),
        filename: filename.clone(),
    };
    let wrap = wrap_turtle(job, &fake_fetched, &sha, started, ended, &filename);
    let wrap_path = run_dir.join("wrap.ttl");
    fs::write(&wrap_path, wrap.as_bytes()).map_err(|e| format!("wrap.ttl: {e}"))?;
    let manifest = json!({
        "schema": "synth.job-run/v1",
        "job_id": job.id,
        "kind": job.kind,
        "source_url": url,
        "content_type": content_type,
        "bytes": file_len,
        "sha256": sha,
        "payload": filename,
        "streamed_from_source": true,
        "started_at": started,
        "ended_at": ended,
        "licence_note": "Inbox is staging. Review licence and sensitivity before promoting a .q42.",
    });
    fs::write(
        run_dir.join("manifest.json"),
        serde_json::to_vec_pretty(&manifest).map_err(|e| e.to_string())?,
    )
    .map_err(|e| format!("manifest.json: {e}"))?;

    let dest = run_dir.join("snapshot.q42");
    let fmt = crate::mcp_full::q42::infer_compile_format(None, Some(&filename))
        .map_err(|e| e.message)?;
    let (count, _lex_terms, _q42_sha) = crate::mcp_full::q42::compile_file_to_q42(
        source_path,
        file_len,
        &dest,
        fmt,
        Some(&format!("urn:synth:job:{}", job.id)),
        false,
        None,
        false,
    )
    .map_err(|e| e.message)?;
    let mut promoted = None;
    if let Some(name) = job.out_q42.as_deref() {
        promoted = promote_q42(&dest, name)?;
    }
    Ok(RunMeta {
        bytes: file_len as usize,
        content_type,
        sha256: Some(sha),
        source_url: Some(url),
        wrap_path: Some(wrap_path.display().to_string()),
        q42_path: Some(dest.display().to_string()),
        promoted_q42: promoted,
        quin_count: Some(count),
    })
}

fn download(url: &str, query: Option<&str>, secret: Option<&str>) -> Result<Fetched, String> {
    if !(url.starts_with("https://") || url.starts_with("http://")) {
        return Err("Job URL must be http:// or https://".into());
    }
    deny_non_loopback_in_tests(url)?;
    let agent = ureq::AgentBuilder::new()
        .timeout(FETCH_TIMEOUT)
        .redirects(5)
        .user_agent(USER_AGENT)
        .build();
    let accept = if query.is_some() {
        "text/turtle, application/n-triples, application/rdf+xml, application/sparql-results+json;q=0.8, */*;q=0.1"
    } else {
        "text/turtle, application/rdf+xml, application/n-triples, application/ld+json, text/csv, application/json, */*;q=0.2"
    };
    let mut request = if query.is_some() {
        agent
            .post(url)
            .set("Content-Type", "application/sparql-query")
            .set("Accept", accept)
            .set("User-Agent", USER_AGENT)
    } else {
        agent.get(url).set("Accept", accept).set("User-Agent", USER_AGENT)
    };
    if let Some(token) = secret {
        let auth_value = if url.contains("bioontology.org") {
            format!("apikey token={token}")
        } else {
            format!("Bearer {token}")
        };
        request = request.set("Authorization", &auth_value);
    }
    let response = if let Some(sparql) = query {
        request.send_string(sparql)
    } else {
        request.call()
    }
    .map_err(|e| format!("download failed: {e}"))?;
    let final_url = response.get_url().to_owned();
    if !(final_url.starts_with("https://") || final_url.starts_with("http://")) {
        return Err("redirect left http(s)".into());
    }
    let content_type = response
        .header("Content-Type")
        .unwrap_or("application/octet-stream")
        .to_owned();
    let mut reader = response.into_reader();
    let mut bytes = Vec::new();
    let mut chunk = [0u8; 8192];
    loop {
        let n = reader
            .read(&mut chunk)
            .map_err(|e| format!("download read: {e}"))?;
        if n == 0 {
            break;
        }
        if bytes.len().saturating_add(n) > MAX_BYTES {
            return Err(format!("payload exceeds {MAX_BYTES} bytes"));
        }
        bytes.extend_from_slice(&chunk[..n]);
    }
    let filename = filename_from_url(&final_url);
    Ok(Fetched {
        bytes,
        content_type,
        url: final_url,
        filename,
    })
}

/// Tests may fetch loopback. Production may fetch any http(s) URL.
fn deny_non_loopback_in_tests(_url: &str) -> Result<(), String> {
    Ok(())
}

fn filename_from_url(url: &str) -> String {
    let path = url.split('?').next().unwrap_or(url);
    let last = path.rsplit('/').next().unwrap_or("payload.bin");
    if last.is_empty() || last.contains(':') {
        "payload.bin".into()
    } else {
        last.chars()
            .map(|c| {
                if c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_') {
                    c
                } else {
                    '-'
                }
            })
            .collect()
    }
}

fn guess_content_type(filename: &str, header: &str) -> String {
    if !header.is_empty() {
        return header
            .split(';')
            .next()
            .unwrap_or(header)
            .trim()
            .to_owned();
    }
    let ext = Path::new(filename)
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    match ext.as_str() {
        "ttl" | "turtle" => "text/turtle".into(),
        "n3" => "text/n3".into(),
        "nt" => "application/n-triples".into(),
        "rdf" | "owl" => "application/rdf+xml".into(),
        "yaml" | "yml" => "application/yaml".into(),
        "json" | "jsonld" => "application/json".into(),
        "csv" => "text/csv".into(),
        "html" | "htm" => "text/html".into(),
        _ => "application/octet-stream".into(),
    }
}

fn looks_like_turtle(bytes: &[u8], content_type: &str, filename: &str) -> bool {
    let ct = content_type.to_ascii_lowercase();
    if ct.contains("rdf+xml") || ct.contains("html") {
        return false;
    }
    if ct.contains("turtle") || ct.contains("n3") || ct.contains("n-triples") {
        return true;
    }
    let ext = Path::new(filename)
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    if matches!(ext.as_str(), "ttl" | "turtle" | "n3" | "nt") {
        return true;
    }
    let head = std::str::from_utf8(bytes.get(..512).unwrap_or(bytes)).unwrap_or("");
    let t = head.trim_start();
    t.starts_with("@prefix")
        || t.starts_with("@base")
        || t.starts_with("PREFIX")
        || t.starts_with("BASE")
}

fn write_snapshot(
    run_dir: &Path,
    job: &Job,
    fetched: &Fetched,
    started: u64,
    compile: bool,
) -> Result<RunMeta, String> {
    let payload_name = if looks_like_turtle(&fetched.bytes, &fetched.content_type, &fetched.filename)
    {
        if fetched.filename.ends_with(".ttl")
            || fetched.filename.ends_with(".n3")
            || fetched.filename.ends_with(".nt")
        {
            fetched.filename.clone()
        } else {
            "payload.ttl".into()
        }
    } else {
        let name = if fetched.filename.is_empty() {
            "payload.bin".into()
        } else {
            fetched.filename.clone()
        };
        name
    };
    let payload_path = run_dir.join(&payload_name);
    fs::write(&payload_path, &fetched.bytes).map_err(|e| format!("payload write: {e}"))?;
    let sha = format!("{:x}", Sha256::digest(&fetched.bytes));
    let ended = now_secs();
    let wrap = wrap_turtle(job, fetched, &sha, started, ended, &payload_name);
    let wrap_path = run_dir.join("wrap.ttl");
    fs::write(&wrap_path, wrap.as_bytes()).map_err(|e| format!("wrap.ttl: {e}"))?;
    let manifest = json!({
        "schema": "synth.job-run/v1",
        "job_id": job.id,
        "kind": job.kind,
        "source_url": fetched.url,
        "content_type": fetched.content_type,
        "bytes": fetched.bytes.len(),
        "sha256": sha,
        "payload": payload_name,
        "started_at": started,
        "ended_at": ended,
        "licence_note": "Inbox is staging. Review licence and sensitivity before promoting a .q42.",
    });
    fs::write(
        run_dir.join("manifest.json"),
        serde_json::to_vec_pretty(&manifest).map_err(|e| e.to_string())?,
    )
    .map_err(|e| format!("manifest.json: {e}"))?;

    let mut q42_path = None;
    let mut promoted = None;
    let mut quin_count = None;
    if compile {
        let dest = run_dir.join("snapshot.q42");
        let is_yaml = payload_name.ends_with(".yaml") || payload_name.ends_with(".yml");
        let count = if is_yaml {
            compile_yaml_with_wrap_to_q42(
                &fetched.bytes,
                wrap.as_bytes(),
                &dest,
                &format!("urn:synth:job:{}", job.id),
            )?
        } else if looks_like_turtle(&fetched.bytes, &fetched.content_type, &payload_name) {
            let mut combined = wrap.into_bytes();
            combined.extend_from_slice(b"\n\n# --- fetched graph ---\n\n");
            combined.extend_from_slice(&fetched.bytes);
            compile_ttl_to_q42(&combined, &dest, &format!("urn:synth:job:{}", job.id))?
        } else if let Ok(crate::mcp_full::q42::ResolvedCompileFormat::Rdf(rdf_fmt, _)) =
            crate::mcp_full::q42::infer_compile_format(None, Some(&payload_name))
        {
            compile_rdf_with_wrap_to_q42(
                &fetched.bytes,
                rdf_fmt,
                wrap.as_bytes(),
                &dest,
                &format!("urn:synth:job:{}", job.id),
            )?
        } else {
            compile_ttl_to_q42(wrap.as_bytes(), &dest, &format!("urn:synth:job:{}", job.id))?
        };
        quin_count = Some(count);
        q42_path = Some(dest.display().to_string());
        if let Some(name) = job.out_q42.as_deref() {
            promoted = promote_q42(&dest, name)?;
        }
    }
    Ok(RunMeta {
        bytes: fetched.bytes.len(),
        content_type: fetched.content_type.clone(),
        sha256: Some(sha),
        source_url: Some(fetched.url.clone()),
        wrap_path: Some(wrap_path.display().to_string()),
        q42_path,
        promoted_q42: promoted,
        quin_count,
    })
}

fn promote_q42(src: &Path, name: &str) -> Result<Option<String>, String> {
    let Ok(root) = std::env::var("SYNTH_OUTPUT_ROOT") else {
        return Ok(None);
    };
    let root = root.trim();
    if root.is_empty() {
        return Ok(None);
    }
    validate_out_q42(name)?;
    let dest = Path::new(root).join(name);
    if dest.exists() {
        return Ok(None);
    }
    fs::copy(src, &dest).map_err(|e| format!("promote q42: {e}"))?;
    Ok(Some(dest.display().to_string()))
}

fn compile_ttl_to_q42(bytes: &[u8], dest: &Path, context_iri: &str) -> Result<usize, String> {
    if dest.exists() {
        return Err("Q42 destination already exists".into());
    }
    let ctx = qualia_core_db::q_hash(context_iri);
    let (quins, lex): (
        Vec<qualia_core_db::NQuin>,
        std::collections::HashMap<u64, String>,
    ) = qualia_core_db::ontology_loader::parse_ttl_bytes_to_quins_and_lex(bytes, ctx)
        .map_err(|e| format!("RDF parse failed: {e}"))?;
    if quins.is_empty() {
        return Err("RDF parsed to zero triples; refusing empty Q42 volume".into());
    }
    qualia_core_db::q42_volume::write_sorted_quins_volume_with_lex(dest, &quins, &lex, 0)
        .map_err(|e| format!("Q42 write failed: {e}"))?;
    Ok(quins.len())
}

fn compile_rdf_with_wrap_to_q42(
    rdf_bytes: &[u8],
    rdf_fmt: qualia_core_db::query::ingest_job::IngestRdfFormat,
    wrap_ttl: &[u8],
    dest: &Path,
    context_iri: &str,
) -> Result<usize, String> {
    if dest.exists() {
        return Err("Q42 destination already exists".into());
    }
    let ctx = qualia_core_db::q_hash(context_iri);
    let (mut quins, mut lex) =
        qualia_core_db::ontology_loader::parse_rdf_bytes_to_quins_and_lex(rdf_bytes, rdf_fmt, ctx)
            .map_err(|e| format!("RDF/OWL parse failed: {e}"))?;
    if let Ok((wrap_quins, wrap_lex)) =
        qualia_core_db::ontology_loader::parse_ttl_bytes_to_quins_and_lex(wrap_ttl, ctx)
    {
        quins.extend(wrap_quins);
        lex.extend(wrap_lex);
    }
    if quins.is_empty() {
        return Err("RDF/OWL parsed to zero triples; refusing empty Q42 volume".into());
    }
    qualia_core_db::q42_volume::write_sorted_quins_volume_with_lex(dest, &quins, &lex, 0)
        .map_err(|e| format!("Q42 write failed: {e}"))?;
    Ok(quins.len())
}

fn compile_yaml_with_wrap_to_q42(
    yaml_bytes: &[u8],
    wrap_ttl: &[u8],
    dest: &Path,
    context_iri: &str,
) -> Result<usize, String> {
    if dest.exists() {
        return Err("Q42 destination already exists".into());
    }
    let ctx = qualia_core_db::q_hash(context_iri);
    let (mut quins, mut lex, _shape) =
        qualia_core_db::q42::yaml_ld_q42::compile_yaml_ld_q42_auto(yaml_bytes, ctx, 1)
            .map_err(|e| format!("yaml-ld-q42 parse failed: {e}"))?;
    if let Ok((wrap_quins, wrap_lex)) =
        qualia_core_db::ontology_loader::parse_ttl_bytes_to_quins_and_lex(wrap_ttl, ctx)
    {
        quins.extend(wrap_quins);
        lex.extend(wrap_lex);
    }
    if quins.is_empty() {
        return Err("yaml-ld-q42 parsed to zero NQuins; refusing empty Q42 volume".into());
    }
    qualia_core_db::q42_volume::write_sorted_quins_volume_with_lex(dest, &quins, &lex, 0)
        .map_err(|e| format!("Q42 write failed: {e}"))?;
    Ok(quins.len())
}

fn wrap_turtle(
    job: &Job,
    fetched: &Fetched,
    sha: &str,
    started: u64,
    ended: u64,
    payload_name: &str,
) -> String {
    let run_iri = format!("urn:synth:inbox:{}:{}", slug_id(&job.id), started);
    let activity = format!("urn:synth:activity:{}:{}", slug_id(&job.id), started);
    let source = turtle_iri(&fetched.url);
    let title = turtle_str(&job.title);
    let format = turtle_str(&fetched.content_type);
    let issued = iso8601_utc(ended);
    let started_s = iso8601_utc(started);
    let ended_s = iso8601_utc(ended);
    let payload = turtle_str(payload_name);
    format!(
        r#"@prefix dct: <http://purl.org/dc/terms/> .
@prefix dcat: <http://www.w3.org/ns/dcat#> .
@prefix prov: <http://www.w3.org/ns/prov#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
@prefix spdx: <http://spdx.org/rdf/terms#> .
@prefix foaf: <http://xmlns.com/foaf/0.1/> .

<{run_iri}> a dcat:Distribution, prov:Entity ;
    dct:title {title} ;
    dct:source {source} ;
    dct:format {format} ;
    dct:issued "{issued}"^^xsd:dateTime ;
    dct:identifier "sha256:{sha}" ;
    dcat:mediaType {format} ;
    dcat:byteSize {bytes} ;
    dcat:downloadURL {source} ;
    spdx:checksum [
        a spdx:Checksum ;
        spdx:algorithm <http://spdx.org/rdf/terms#checksumAlgorithm_sha256> ;
        spdx:checksumValue "{sha}"
    ] ;
    prov:wasGeneratedBy <{activity}> ;
    prov:wasDerivedFrom {source} ;
    dct:description {payload} .

<{activity}> a prov:Activity ;
    dct:title "synth job {kind}" ;
    prov:startedAtTime "{started_s}"^^xsd:dateTime ;
    prov:endedAtTime "{ended_s}"^^xsd:dateTime ;
    prov:used {source} ;
    prov:wasAssociatedWith <urn:synth:agent:webizen-ai-synth> .

<urn:synth:agent:webizen-ai-synth> a prov:SoftwareAgent, foaf:Agent ;
    foaf:name "Webizen AI Synth" .
"#,
        bytes = fetched.bytes.len(),
        kind = job.kind,
        sha = sha,
    )
}

fn turtle_str(s: &str) -> String {
    format!(
        "\"{}\"",
        s.replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\n', "\\n")
            .replace('\r', "")
    )
}

fn turtle_iri(s: &str) -> String {
    let cleaned: String = s
        .chars()
        .filter(|c| *c != '<' && *c != '>' && *c != ' ' && *c != '\n' && *c != '\r')
        .collect();
    format!("<{cleaned}>")
}

pub(crate) fn iso8601_utc(secs: u64) -> String {
    let days = (secs / 86400) as i64;
    let tod = secs % 86400;
    let hh = tod / 3600;
    let mm = (tod % 3600) / 60;
    let ss = tod % 60;
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u32;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_097) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = if month <= 2 { y + 1 } else { y };
    format!("{year:04}-{month:02}-{d:02}T{hh:02}:{mm:02}:{ss:02}Z")
}

fn prune_runs(job_inbox: &Path, keep_id: &str) {
    let Ok(entries) = fs::read_dir(job_inbox) else {
        return;
    };
    let mut dirs: Vec<PathBuf> = entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .collect();
    if dirs.len() <= MAX_RUNS_KEPT {
        return;
    }
    dirs.sort();
    let overflow = dirs.len().saturating_sub(MAX_RUNS_KEPT);
    for path in dirs.into_iter().take(overflow) {
        if path.file_name().and_then(|s| s.to_str()) == Some(keep_id) {
            continue;
        }
        let _ = fs::remove_dir_all(path);
    }
}

/// CLI: `synth job list|add URL|run ID|run-due|enable ID|disable ID|forget ID`
pub fn run_cli(args: &[String]) -> Result<(), String> {
    let action = args.first().map(String::as_str).unwrap_or("list");
    let payload = match action {
        "list" | "--help" => {
            if action == "--help" {
                println!(
                    "synth job list\n  synth job add URL [--id ID] [--title T] [--kind fetch|ingest|compile] [--interval SECS] [--bookmark ID] [--out-q42 NAME.q42]\n  synth job run ID\n  synth job run-due\n  synth job enable ID | disable ID | forget ID | get ID\nJobs: .synth/jobs.json. Inbox: .synth/inbox/ (gitignored). Operator ticks due jobs."
                );
                return Ok(());
            }
            dispatch(JobArg {
                action: Some("list".into()),
                ..JobArg::default()
            })?
        }
        "add" => {
            let mut url = None;
            let mut title = None;
            let mut id = None;
            let mut kind = None;
            let mut interval = None;
            let mut bookmark_id = None;
            let mut out_q42 = None;
            let mut source_path = None;
            let mut i = 1;
            if let Some(first) = args.get(1) {
                if !first.starts_with("--") {
                    url = Some(first.clone());
                    i = 2;
                }
            }
            while i < args.len() {
                match args[i].as_str() {
                    "--title" => {
                        i += 1;
                        title = Some(args.get(i).ok_or("--title needs a value")?.clone());
                    }
                    "--id" => {
                        i += 1;
                        id = Some(args.get(i).ok_or("--id needs a value")?.clone());
                    }
                    "--kind" => {
                        i += 1;
                        kind = Some(args.get(i).ok_or("--kind needs a value")?.clone());
                    }
                    "--interval" => {
                        i += 1;
                        interval = Some(
                            args.get(i)
                                .ok_or("--interval needs seconds")?
                                .parse::<u64>()
                                .map_err(|_| "interval must be u64")?,
                        );
                    }
                    "--bookmark" => {
                        i += 1;
                        bookmark_id = Some(args.get(i).ok_or("--bookmark needs an id")?.clone());
                    }
                    "--out-q42" => {
                        i += 1;
                        out_q42 = Some(args.get(i).ok_or("--out-q42 needs a filename")?.clone());
                    }
                    "--source" => {
                        i += 1;
                        source_path = Some(args.get(i).ok_or("--source needs a path")?.clone());
                    }
                    other => return Err(format!("Unknown flag `{other}`")),
                }
                i += 1;
            }
            if url.is_none() && source_path.is_none() && bookmark_id.is_none() {
                return Err(
                    "Usage: synth job add URL [--id ID] [--title T] [--kind fetch|ingest|compile] [--interval SECS] [--bookmark ID] [--source PATH] [--out-q42 NAME.q42]".into(),
                );
            }
            dispatch(JobArg {
                action: Some("add".into()),
                url,
                title,
                id,
                kind,
                interval_secs: interval,
                bookmark_id,
                out_q42,
                source_path,
                ..JobArg::default()
            })?
        }
        "run" => {
            let id = args.get(1).ok_or("Usage: synth job run ID")?;
            dispatch(JobArg {
                action: Some("run".into()),
                id: Some(id.clone()),
                ..JobArg::default()
            })?
        }
        "run-due" | "run_due" => dispatch(JobArg {
            action: Some("run_due".into()),
            ..JobArg::default()
        })?,
        "enable" | "disable" | "forget" | "get" => {
            let id = args
                .get(1)
                .ok_or_else(|| format!("Usage: synth job {action} ID"))?;
            dispatch(JobArg {
                action: Some(action.into()),
                id: Some(id.clone()),
                ..JobArg::default()
            })?
        }
        other => return Err(format!("Unknown `synth job` action `{other}`")),
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
    use std::io::Write;
    use std::net::TcpListener;

    struct Guard {
        bookmark: PathBuf,
        input: PathBuf,
        output: PathBuf,
        _lock: std::sync::MutexGuard<'static, ()>,
    }
    impl Guard {
        fn new() -> Self {
            let lock = crate::operator::TEST_ENV_LOCK
                .lock()
                .unwrap_or_else(|e| e.into_inner());
            let stamp = format!(
                "{}-{}",
                std::process::id(),
                SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()
            );
            let bookmark = std::env::temp_dir().join(format!("synth-jobs-{stamp}"));
            let input = std::env::temp_dir().join(format!("synth-jobs-in-{stamp}"));
            let output = std::env::temp_dir().join(format!("synth-jobs-out-{stamp}"));
            fs::create_dir_all(&bookmark).unwrap();
            fs::create_dir_all(&input).unwrap();
            fs::create_dir_all(&output).unwrap();
            std::env::set_var("SYNTH_BOOKMARK_ROOT", bookmark.to_str().unwrap());
            std::env::set_var("SYNTH_INPUT_ROOT", input.to_str().unwrap());
            std::env::set_var("SYNTH_OUTPUT_ROOT", output.to_str().unwrap());
            Self {
                bookmark,
                input,
                output,
                _lock: lock,
            }
        }
    }
    impl Drop for Guard {
        fn drop(&mut self) {
            std::env::remove_var("SYNTH_BOOKMARK_ROOT");
            std::env::remove_var("SYNTH_INPUT_ROOT");
            std::env::remove_var("SYNTH_OUTPUT_ROOT");
            let _ = fs::remove_dir_all(&self.bookmark);
            let _ = fs::remove_dir_all(&self.input);
            let _ = fs::remove_dir_all(&self.output);
        }
    }

    fn serve_ttl(body: &'static [u8]) -> String {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut buf = [0u8; 2048];
            let _ = stream.read(&mut buf);
            let header = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/turtle\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                body.len()
            );
            let _ = stream.write_all(header.as_bytes());
            let _ = stream.write_all(body);
        });
        format!("http://127.0.0.1:{}", addr.port())
    }

    #[test]
    fn iso8601_unix_epoch() {
        assert_eq!(iso8601_utc(0), "1970-01-01T00:00:00Z");
        assert_eq!(iso8601_utc(1_700_000_000), "2023-11-14T22:13:20Z");
    }

    #[test]
    fn compile_local_turtle_wraps_prov_and_seals_q42() {
        let g = Guard::new();
        fs::write(
            g.input.join("taxon.ttl"),
            b"@prefix ex: <http://example.org/> .\nex:Corymbia a ex:Taxon .\n",
        )
        .unwrap();
        let added = dispatch(JobArg {
            action: Some("add".into()),
            id: Some("local-taxon".into()),
            kind: Some("compile".into()),
            source_path: Some("taxon.ttl".into()),
            title: Some("Local taxon".into()),
            out_q42: Some("taxon-job.q42".into()),
            ..JobArg::default()
        })
        .unwrap();
        assert_eq!(added["item"]["kind"], "compile");
        let ran = dispatch(JobArg {
            action: Some("run".into()),
            id: Some("local-taxon".into()),
            ..JobArg::default()
        })
        .unwrap();
        assert_eq!(ran["ok"], true);
        assert!(ran["quin_count"].as_u64().unwrap() >= 1);
        let wrap = fs::read_to_string(Path::new(ran["wrap"].as_str().unwrap())).unwrap();
        assert!(wrap.contains("prov:wasGeneratedBy"));
        assert!(wrap.contains("dct:source"));
        assert!(
            ran["source_url"].as_str().unwrap().starts_with("file:///"),
            "local snapshot IRI must be a file URL, got {}",
            ran["source_url"]
        );
        assert!(
            !ran["source_url"].as_str().unwrap().contains('?'),
            "Windows UNC prefix must not leak into the file URL"
        );
        assert!(Path::new(ran["q42"].as_str().unwrap()).is_file());
        assert!(g.output.join("taxon-job.q42").is_file());
        let listed = dispatch(JobArg {
            action: Some("list".into()),
            ..JobArg::default()
        })
        .unwrap();
        assert_eq!(listed["count"], 1);
        assert_eq!(listed["items"][0]["enabled"], false);
    }

    #[test]
    fn ingest_fetches_loopback_turtle_and_scheduler_due_runs() {
        let _g = Guard::new();
        let body: &'static [u8] =
            b"@prefix ex: <http://example.org/> .\nex:Source a ex:Ontology .\n";
        let url = serve_ttl(body);
        let added = dispatch(JobArg {
            action: Some("add".into()),
            id: Some("loopback-ont".into()),
            kind: Some("ingest".into()),
            url: Some(url),
            title: Some("Loopback ontology".into()),
            interval_secs: Some(0),
            ..JobArg::default()
        })
        .unwrap();
        assert_eq!(added["ok"], true);
        let due = dispatch(JobArg {
            action: Some("run_due".into()),
            ..JobArg::default()
        })
        .unwrap();
        assert_eq!(due["due"], 1);
        assert_eq!(due["ran"][0]["ok"], true);
        assert!(due["ran"][0]["sha256"].as_str().unwrap().len() == 64);
        let again = dispatch(JobArg {
            action: Some("run_due".into()),
            ..JobArg::default()
        })
        .unwrap();
        assert_eq!(again["due"], 0);
    }

    #[test]
    fn secret_query_param_is_rejected_on_add() {
        let _g = Guard::new();
        let err = dispatch(JobArg {
            action: Some("add".into()),
            url: Some("https://example.org/data.ttl?api_key=leak".into()),
            ..JobArg::default()
        })
        .unwrap_err();
        assert!(err.contains("credential"));
    }
}
