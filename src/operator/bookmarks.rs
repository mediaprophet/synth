//! Semantic bookmarks for RDF / SPARQL / similar datasources.
//!
//! Public metadata lives in `.synth/bookmarks.json`. API keys and tokens live in
//! `.synth/secrets.json`, which is gitignored. Fetching is a Synth job
//! (`synth job` / `synth_jobs`); bookmarks stay the public register.
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

const SCHEMA: &str = "synth.semantic-bookmarks/v1";
const SECRETS_SCHEMA: &str = "synth.semantic-bookmark-secrets/v1";
const MAX_BOOKMARKS: usize = 256;
const MAX_URL: usize = 2048;
const MAX_NOTES: usize = 8192;
const MAX_SECRET: usize = 4096;
const GITIGNORE: &str = "# Local secrets and acquisition inbox. Never commit to GitHub.\nsecrets.json\nsecrets.*.json\n*.key\n*.pem\nupdate-seed.hex\n*-seed.hex\ninbox/\n";

const SECRET_QUERY_KEYS: &[&str] = &[
    "api_key",
    "apikey",
    "access_token",
    "token",
    "key",
    "password",
    "secret",
    "auth",
    "authorization",
    "jwt",
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bookmark {
    pub id: String,
    pub url: String,
    #[serde(default)]
    pub kind: String,
    #[serde(default)]
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data_endpoint: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub formats: Vec<String>,
    #[serde(default)]
    pub requires_api_key: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_key_env: Option<String>,
    #[serde(default)]
    pub auth: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub licence: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_notes: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub domain: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    #[serde(default)]
    pub created_at: u64,
    #[serde(default)]
    pub updated_at: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct BookmarkStore {
    schema: String,
    items: Vec<Bookmark>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct SecretStore {
    #[serde(default)]
    schema: String,
    #[serde(default)]
    keys: BTreeMap<String, SecretEntry>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct SecretEntry {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    env: Option<String>,
    #[serde(default)]
    value: String,
}

#[derive(Debug, Deserialize, Default)]
#[serde(default, deny_unknown_fields)]
pub struct BookmarkArg {
    pub action: Option<String>,
    pub id: Option<String>,
    pub url: Option<String>,
    pub title: Option<String>,
    pub kind: Option<String>,
    pub data_endpoint: Option<String>,
    pub formats: Option<Vec<String>>,
    pub requires_api_key: Option<bool>,
    pub api_key_env: Option<String>,
    pub auth: Option<String>,
    pub licence: Option<String>,
    pub notes: Option<String>,
    pub agent_notes: Option<String>,
    pub domain: Option<String>,
    pub tags: Option<Vec<String>>,
    pub secret: Option<String>,
}

pub fn store_dir() -> Result<PathBuf, String> {
    if let Ok(raw) = std::env::var("SYNTH_BOOKMARK_ROOT") {
        let trimmed = raw.trim();
        if !trimmed.is_empty() {
            return Ok(PathBuf::from(trimmed).join(".synth"));
        }
    }
    if let Some(project) = crate::project::current() {
        return Ok(project.root.join(".synth"));
    }
    if let Ok(raw) = std::env::var("SYNTH_OUTPUT_ROOT") {
        let trimmed = raw.trim();
        if !trimmed.is_empty() {
            return Ok(PathBuf::from(trimmed).join(".synth"));
        }
    }
    Err(
        "Set SYNTH_OUTPUT_ROOT, SYNTH_BOOKMARK_ROOT, or attach a project so bookmarks can live in .synth/"
            .into(),
    )
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn ensure_store(dir: &Path) -> Result<(), String> {
    fs::create_dir_all(dir).map_err(|e| format!(".synth: {e}"))?;
    let gi = dir.join(".gitignore");
    if !gi.is_file() {
        fs::write(&gi, GITIGNORE).map_err(|e| format!(".synth/.gitignore: {e}"))?;
    }
    Ok(())
}

fn load_store(dir: &Path) -> BookmarkStore {
    let path = dir.join("bookmarks.json");
    let Ok(bytes) = fs::read(&path) else {
        return BookmarkStore {
            schema: SCHEMA.into(),
            items: vec![],
        };
    };
    serde_json::from_slice::<BookmarkStore>(&bytes).unwrap_or(BookmarkStore {
        schema: SCHEMA.into(),
        items: vec![],
    })
}

fn save_store(dir: &Path, store: &BookmarkStore) -> Result<(), String> {
    ensure_store(dir)?;
    let bytes = serde_json::to_vec_pretty(store).map_err(|e| e.to_string())?;
    fs::write(dir.join("bookmarks.json"), bytes).map_err(|e| format!("bookmarks.json: {e}"))
}

fn load_secrets(dir: &Path) -> SecretStore {
    let Ok(bytes) = fs::read(dir.join("secrets.json")) else {
        return SecretStore {
            schema: SECRETS_SCHEMA.into(),
            keys: BTreeMap::new(),
        };
    };
    serde_json::from_slice::<SecretStore>(&bytes).unwrap_or(SecretStore {
        schema: SECRETS_SCHEMA.into(),
        keys: BTreeMap::new(),
    })
}

fn save_secrets(dir: &Path, store: &SecretStore) -> Result<(), String> {
    ensure_store(dir)?;
    let bytes = serde_json::to_vec_pretty(store).map_err(|e| e.to_string())?;
    fs::write(dir.join("secrets.json"), bytes).map_err(|e| format!("secrets.json: {e}"))
}

fn public_item(bm: &Bookmark, secret_present: bool) -> Value {
    json!({
        "id": bm.id,
        "url": bm.url,
        "kind": bm.kind,
        "title": bm.title,
        "data_endpoint": bm.data_endpoint,
        "formats": bm.formats,
        "requires_api_key": bm.requires_api_key,
        "api_key_env": bm.api_key_env,
        "auth": bm.auth,
        "licence": bm.licence,
        "notes": bm.notes,
        "agent_notes": bm.agent_notes,
        "domain": bm.domain,
        "tags": bm.tags,
        "created_at": bm.created_at,
        "updated_at": bm.updated_at,
        "secret_present": secret_present,
        "needs_endpoint": bm.data_endpoint.as_deref().unwrap_or("").is_empty()
            && bm.kind == "page",
        "needs_secret": bm.requires_api_key && !secret_present,
    })
}

fn slug_id(url: &str) -> String {
    let rest = url
        .trim()
        .trim_start_matches("https://")
        .trim_start_matches("http://");
    let mut out = String::new();
    for ch in rest.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
        } else if matches!(ch, '.' | '/' | '-' | '_' | ':') {
            if !out.ends_with('-') {
                out.push('-');
            }
        }
        if out.len() >= 80 {
            break;
        }
    }
    let trimmed = out.trim_matches('-');
    if trimmed.is_empty() {
        "bookmark".into()
    } else {
        trimmed.to_owned()
    }
}

fn unique_id(preferred: &str, existing: &[Bookmark], skip: Option<&str>) -> String {
    let base = if preferred.is_empty() {
        "bookmark".into()
    } else {
        preferred
            .chars()
            .map(|c| {
                if c.is_ascii_alphanumeric() {
                    c.to_ascii_lowercase()
                } else {
                    '-'
                }
            })
            .collect::<String>()
            .trim_matches('-')
            .to_owned()
    };
    let taken = |id: &str| {
        existing
            .iter()
            .any(|b| b.id == id && skip != Some(b.id.as_str()))
    };
    if !taken(&base) {
        return base;
    }
    for n in 2..=MAX_BOOKMARKS + 2 {
        let candidate = format!("{base}-{n}");
        if !taken(&candidate) {
            return candidate;
        }
    }
    format!("{base}-x")
}

/// Strip credential query parameters. Returns (cleaned url, saw_secret_param).
pub fn sanitize_url(raw: &str) -> Result<(String, bool), String> {
    let trimmed = raw.trim();
    if trimmed.len() > MAX_URL {
        return Err("URL exceeds 2048 bytes".into());
    }
    if !(trimmed.starts_with("https://") || trimmed.starts_with("http://")) {
        return Err("Bookmark URL must be http:// or https://".into());
    }
    if trimmed.contains([' ', '\n', '\r', '\t']) {
        return Err("Bookmark URL must not contain whitespace".into());
    }
    let (base, query) = match trimmed.split_once('?') {
        Some((b, q)) => (b, Some(q)),
        None => (trimmed, None),
    };
    let Some(query) = query else {
        return Ok((base.to_owned(), false));
    };
    let mut kept = Vec::new();
    let mut saw_secret = false;
    for pair in query.split('&') {
        if pair.is_empty() {
            continue;
        }
        let key = pair.split('=').next().unwrap_or(pair).to_ascii_lowercase();
        if SECRET_QUERY_KEYS.iter().any(|k| *k == key) {
            saw_secret = true;
            continue;
        }
        kept.push(pair);
    }
    if kept.is_empty() {
        Ok((base.to_owned(), saw_secret))
    } else {
        Ok((format!("{base}?{}", kept.join("&")), saw_secret))
    }
}

/// Classify a URL from its shape. Does not fetch.
pub fn classify(url: &str) -> (String, Vec<String>, Option<String>, bool) {
    let (without_query, _) = url.split_once('?').unwrap_or((url, ""));
    let path = without_query
        .split("://")
        .nth(1)
        .and_then(|rest| rest.split_once('/').map(|(_, p)| p))
        .unwrap_or("");
    let lower = without_query.to_ascii_lowercase();
    let ext = Path::new(path)
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    if lower.contains("/sparql") || path.eq_ignore_ascii_case("sparql") || lower.ends_with("/sparql")
    {
        return (
            "sparql".into(),
            vec!["sparql".into()],
            Some(without_query.to_owned()),
            false,
        );
    }
    if lower.contains("openstreetmap.org")
        || lower.contains("://osm.org")
        || lower.contains("www.osm.org")
        || lower.contains("/planet.osm")
        || ext == "osm"
        || ext == "pbf"
        || lower.contains("opentopography.org")
        || lower.contains("/stac/")
        || lower.contains("stac.cloud")
    {
        return (
            "gis".into(),
            vec!["osm".into(), "geotiff".into(), "stac".into()],
            Some(without_query.to_owned()),
            false,
        );
    }
    match ext.as_str() {
        "ttl" | "turtle" => (
            "rdf".into(),
            vec!["ttl".into()],
            Some(without_query.to_owned()),
            false,
        ),
        "n3" => (
            "rdf".into(),
            vec!["n3".into()],
            Some(without_query.to_owned()),
            false,
        ),
        "nt" | "nq" => (
            "rdf".into(),
            vec![ext],
            Some(without_query.to_owned()),
            false,
        ),
        "rdf" | "owl" | "xml" => (
            "rdf".into(),
            vec!["rdfxml".into()],
            Some(without_query.to_owned()),
            false,
        ),
        "jsonld" | "json-ld" => (
            "rdf".into(),
            vec!["jsonld".into()],
            Some(without_query.to_owned()),
            false,
        ),
        "csv" | "tsv" => (
            "csv".into(),
            vec![ext],
            Some(without_query.to_owned()),
            false,
        ),
        _ => ("page".into(), vec![], None, true),
    }
}

fn bounded_text(label: &str, value: Option<String>, max: usize) -> Result<Option<String>, String> {
    match value {
        None => Ok(None),
        Some(s) if s.len() <= max => Ok(Some(s)),
        Some(_) => Err(format!("{label} exceeds {max} bytes")),
    }
}

fn public_list(dir: &Path) -> Value {
    let store = load_store(dir);
    let secrets = load_secrets(dir);
    let items: Vec<Value> = store
        .items
        .iter()
        .map(|bm| public_item(bm, secrets.keys.contains_key(&bm.id)))
        .collect();
    json!({
        "schema": SCHEMA,
        "path": dir.join("bookmarks.json").display().to_string(),
        "secrets_path": dir.join("secrets.json").display().to_string(),
        "count": items.len(),
        "items": items,
        "policy": "Secrets stay in .synth/secrets.json (gitignored). Never commit API keys. Fetch with synth job / synth_jobs into .synth/inbox, then compile via synth_compile_q42.",
    })
}

pub fn summary() -> Value {
    match store_dir() {
        Ok(dir) => {
            let store = load_store(&dir);
            let secrets = load_secrets(&dir);
            let mut needs_endpoint = Vec::new();
            let mut needs_secret = Vec::new();
            for bm in &store.items {
                if bm.data_endpoint.as_deref().unwrap_or("").is_empty() && bm.kind == "page" {
                    needs_endpoint.push(bm.id.clone());
                }
                if bm.requires_api_key && !secrets.keys.contains_key(&bm.id) {
                    needs_secret.push(bm.id.clone());
                }
            }
            json!({
                "configured": true,
                "path": dir.join("bookmarks.json").display().to_string(),
                "count": store.items.len(),
                "needs_endpoint": needs_endpoint,
                "needs_secret": needs_secret,
            })
        }
        Err(_) => json!({
            "configured": false,
            "count": 0,
            "needs_endpoint": [],
            "needs_secret": [],
        }),
    }
}

pub fn get_bookmark(id: &str) -> Result<Option<Bookmark>, String> {
    let dir = store_dir()?;
    let store = load_store(&dir);
    Ok(store.items.into_iter().find(|b| b.id == id))
}

pub fn secret_value(id: &str) -> Result<Option<String>, String> {
    let dir = store_dir()?;
    let secrets = load_secrets(&dir);
    Ok(secrets.keys.get(id).and_then(|e| {
        if e.value.is_empty() {
            None
        } else {
            Some(e.value.clone())
        }
    }))
}

pub fn dispatch(arg: BookmarkArg) -> Result<Value, String> {
    let action = arg.action.as_deref().unwrap_or("list").to_ascii_lowercase();
    if action != "set_secret" && arg.secret.is_some() {
        return Err("Pass secrets with action=set_secret so they write .synth/secrets.json, never bookmarks.json".into());
    }
    match action.as_str() {
        "list" => {
            let dir = store_dir()?;
            Ok(public_list(&dir))
        }
        "get" => {
            let id = arg.id.as_deref().ok_or("get requires id")?;
            let dir = store_dir()?;
            let store = load_store(&dir);
            let secrets = load_secrets(&dir);
            let bm = store
                .items
                .iter()
                .find(|b| b.id == id)
                .ok_or_else(|| format!("Unknown bookmark `{id}`"))?;
            Ok(public_item(bm, secrets.keys.contains_key(&bm.id)))
        }
        "forget" => {
            let id = arg.id.as_deref().ok_or("forget requires id")?;
            let dir = store_dir()?;
            let mut store = load_store(&dir);
            let before = store.items.len();
            store.items.retain(|b| b.id != id);
            if store.items.len() == before {
                return Err(format!("Unknown bookmark `{id}`"));
            }
            save_store(&dir, &store)?;
            let mut secrets = load_secrets(&dir);
            secrets.keys.remove(id);
            let _ = save_secrets(&dir, &secrets);
            Ok(json!({"ok": true, "forgotten": id}))
        }
        "set_secret" => {
            let id = arg.id.as_deref().ok_or("set_secret requires id")?;
            let secret = arg.secret.as_deref().ok_or("set_secret requires secret")?;
            if secret.len() > MAX_SECRET {
                return Err("secret exceeds 4096 bytes".into());
            }
            let dir = store_dir()?;
            let mut store = load_store(&dir);
            if !store.items.iter().any(|b| b.id == id) {
                if id == "bioportal" {
                    let ts = now_secs();
                    store.schema = SCHEMA.into();
                    store.items.push(Bookmark {
                        id: "bioportal".into(),
                        url: "https://data.bioontology.org".into(),
                        kind: "rest".into(),
                        title: "NCBO BioPortal Biomedical Ontology Repository".into(),
                        data_endpoint: Some("https://data.bioontology.org".into()),
                        formats: vec!["rdfxml".into(), "ttl".into(), "jsonld".into()],
                        requires_api_key: true,
                        api_key_env: Some(
                            arg.api_key_env
                                .clone()
                                .unwrap_or_else(|| "BIOPORTAL_API_KEY".into()),
                        ),
                        auth: "apikey_token".into(),
                        licence: Some("Per-ontology license (see BioPortal submission metadata)".into()),
                        notes: Some("BioPortal REST API for biomedical ontologies, class trees, and cross-ontology SKOS/LOOM mappings.".into()),
                        agent_notes: Some("Authenticate via header Authorization: apikey token=<key>.".into()),
                        domain: Some("biomedical".into()),
                        tags: vec!["biomedical".into(), "ontology".into(), "bioportal".into()],
                        created_at: ts,
                        updated_at: ts,
                    });
                } else {
                    return Err(format!("Unknown bookmark `{id}` — upsert the URL first"));
                }
            }
            let mut secrets = load_secrets(&dir);
            secrets.schema = SECRETS_SCHEMA.into();
            secrets.keys.insert(
                id.to_owned(),
                SecretEntry {
                    env: arg.api_key_env.clone(),
                    value: secret.to_owned(),
                },
            );
            save_secrets(&dir, &secrets)?;
            if let Some(bm) = store.items.iter_mut().find(|b| b.id == id) {
                bm.requires_api_key = true;
                if bm.api_key_env.is_none() {
                    bm.api_key_env = arg.api_key_env.clone();
                }
                bm.updated_at = now_secs();
                if bm.auth == "none" || bm.auth.is_empty() {
                    bm.auth = "api_key".into();
                }
            }
            save_store(&dir, &store)?;
            Ok(json!({
                "ok": true,
                "id": id,
                "secret_present": true,
                "stored": dir.join("secrets.json").display().to_string(),
                "gitignored": true,
            }))
        }
        "upsert" => upsert(arg),
        other => Err(format!(
            "Unknown bookmarks action `{other}` (list, get, upsert, forget, set_secret)"
        )),
    }
}

fn upsert(arg: BookmarkArg) -> Result<Value, String> {
    let dir = store_dir()?;
    ensure_store(&dir)?;
    let mut store = load_store(&dir);
    store.schema = SCHEMA.into();
    let ts = now_secs();
    let notes = bounded_text("notes", arg.notes, MAX_NOTES)?;
    let agent_notes = bounded_text("agent_notes", arg.agent_notes, MAX_NOTES)?;
    let title = bounded_text("title", arg.title, 256)?.unwrap_or_default();
    let licence = bounded_text("licence", arg.licence, 256)?;
    let domain = bounded_text("domain", arg.domain, 64)?;
    let api_key_env = bounded_text("api_key_env", arg.api_key_env, 128)?;
    let data_endpoint = match arg.data_endpoint {
        Some(ep) => {
            let (clean, secret_in_ep) = sanitize_url(&ep)?;
            if secret_in_ep {
                return Err("data_endpoint had a credential query parameter; store the key with set_secret".into());
            }
            Some(clean)
        }
        None => None,
    };

    if let Some(id) = arg.id.as_deref() {
        if let Some(existing) = store.items.iter_mut().find(|b| b.id == id) {
            if let Some(url) = arg.url.as_deref() {
                let (clean, saw_secret) = sanitize_url(url)?;
                existing.url = clean;
                if saw_secret {
                    existing.requires_api_key = true;
                }
            }
            if !title.is_empty() {
                existing.title = title;
            }
            if let Some(kind) = arg.kind {
                existing.kind = kind;
            }
            if data_endpoint.is_some() {
                existing.data_endpoint = data_endpoint;
            }
            if let Some(formats) = arg.formats {
                existing.formats = formats;
            }
            if let Some(flag) = arg.requires_api_key {
                existing.requires_api_key = flag;
            }
            if api_key_env.is_some() {
                existing.api_key_env = api_key_env;
            }
            if let Some(auth) = arg.auth {
                existing.auth = auth;
            }
            if licence.is_some() {
                existing.licence = licence;
            }
            if notes.is_some() {
                existing.notes = notes;
            }
            if agent_notes.is_some() {
                existing.agent_notes = agent_notes;
            }
            if domain.is_some() {
                existing.domain = domain;
            }
            if let Some(tags) = arg.tags {
                existing.tags = tags;
            }
            existing.updated_at = ts;
            let snapshot = existing.clone();
            save_store(&dir, &store)?;
            let secrets = load_secrets(&dir);
            let present = secrets.keys.contains_key(&snapshot.id);
            return Ok(json!({"ok": true, "updated": true, "item": public_item(&snapshot, present)}));
        }
    }

    let url = arg.url.as_deref().ok_or("upsert of a new bookmark requires url")?;
    let (clean, saw_secret) = sanitize_url(url)?;
    let (guess_kind, guess_formats, guess_endpoint, _needs) = classify(&clean);
    let preferred = arg.id.clone().unwrap_or_else(|| slug_id(&clean));
    let id = unique_id(&preferred, &store.items, None);
    if store.items.len() >= MAX_BOOKMARKS {
        return Err("Bookmark store is full (256)".into());
    }
    let kind = arg.kind.unwrap_or(guess_kind);
    let formats = arg.formats.unwrap_or(guess_formats);
    let endpoint = data_endpoint.or(guess_endpoint);
    let requires = arg.requires_api_key.unwrap_or(saw_secret);
    let auth = arg.auth.unwrap_or_else(|| {
        if requires {
            "api_key".into()
        } else {
            "none".into()
        }
    });
    let title = if title.is_empty() {
        id.clone()
    } else {
        title
    };
    let bm = Bookmark {
        id,
        url: clean,
        kind,
        title,
        data_endpoint: endpoint,
        formats,
        requires_api_key: requires,
        api_key_env,
        auth,
        licence,
        notes,
        agent_notes,
        domain,
        tags: arg.tags.unwrap_or_default(),
        created_at: ts,
        updated_at: ts,
    };
    let public = public_item(&bm, false);
    store.items.push(bm);
    save_store(&dir, &store)?;
    Ok(json!({"ok": true, "updated": false, "item": public}))
}

/// CLI: `synth bookmark add URL` / `list` / `get ID`
pub fn run_cli(args: &[String]) -> Result<(), String> {
    let action = args.first().map(String::as_str).unwrap_or("list");
    let payload = match action {
        "list" | "--help" => {
            if action == "--help" {
                println!(
                    "synth bookmark list\n  synth bookmark add URL [--title T] [--id ID]\n  synth bookmark get ID\nPublic metadata: .synth/bookmarks.json. Secrets: .synth/secrets.json (gitignored)."
                );
                return Ok(());
            }
            dispatch(BookmarkArg {
                action: Some("list".into()),
                ..BookmarkArg::default()
            })?
        }
        "add" => {
            let url = args.get(1).ok_or("Usage: synth bookmark add URL [--title T] [--id ID]")?;
            let mut title = None;
            let mut id = None;
            let mut i = 2;
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
                    other => return Err(format!("Unknown flag `{other}`")),
                }
                i += 1;
            }
            dispatch(BookmarkArg {
                action: Some("upsert".into()),
                url: Some(url.clone()),
                title,
                id,
                ..BookmarkArg::default()
            })?
        }
        "get" => {
            let id = args.get(1).ok_or("Usage: synth bookmark get ID")?;
            dispatch(BookmarkArg {
                action: Some("get".into()),
                id: Some(id.clone()),
                ..BookmarkArg::default()
            })?
        }
        "set-secret" | "set_secret" => {
            let id = args
                .get(1)
                .ok_or("Usage: synth bookmark set-secret ID SECRET [--env ENV_VAR]")?;
            let secret = args
                .get(2)
                .ok_or("Usage: synth bookmark set-secret ID SECRET [--env ENV_VAR]")?;
            let mut api_key_env = None;
            let mut i = 3;
            while i < args.len() {
                match args[i].as_str() {
                    "--env" => {
                        i += 1;
                        api_key_env = Some(args.get(i).ok_or("--env needs a value")?.clone());
                    }
                    other => return Err(format!("Unknown flag `{other}`")),
                }
                i += 1;
            }
            dispatch(BookmarkArg {
                action: Some("set_secret".into()),
                id: Some(id.clone()),
                secret: Some(secret.clone()),
                api_key_env,
                ..BookmarkArg::default()
            })?
        }
        other => return Err(format!("Unknown `synth bookmark` action `{other}`")),
    };
    println!("{}", serde_json::to_string_pretty(&payload).map_err(|e| e.to_string())?);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Guard(PathBuf);
    impl Guard {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!(
                "synth-bookmarks-{}-{}",
                std::process::id(),
                now_secs()
            ));
            fs::create_dir_all(&path).unwrap();
            std::env::set_var("SYNTH_BOOKMARK_ROOT", path.to_str().unwrap());
            Self(path)
        }
    }
    impl Drop for Guard {
        fn drop(&mut self) {
            std::env::remove_var("SYNTH_BOOKMARK_ROOT");
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn classifies_sparql_rdf_and_landing_page() {
        let (kind, formats, ep, needs) = classify("https://query.wikidata.org/sparql");
        assert_eq!(kind, "sparql");
        assert_eq!(formats, vec!["sparql"]);
        assert_eq!(ep.as_deref(), Some("https://query.wikidata.org/sparql"));
        assert!(!needs);
        let (kind, formats, ep, needs) = classify("https://example.org/ontologies/taxa.ttl");
        assert_eq!(kind, "rdf");
        assert_eq!(formats, vec!["ttl"]);
        assert!(ep.is_some());
        assert!(!needs);
        let (kind, _, ep, needs) = classify("https://ala.org.au/");
        assert_eq!(kind, "page");
        assert!(ep.is_none());
        assert!(needs);
        let (kind, formats, ep, needs) = classify("https://www.openstreetmap.org/export");
        assert_eq!(kind, "gis");
        assert!(formats.contains(&"osm".to_string()));
        assert!(ep.is_some());
        assert!(!needs);
    }

    #[test]
    fn strips_api_key_query_and_flags_auth() {
        let (clean, saw) =
            sanitize_url("https://api.example.org/v1?q=banksia&api_key=super-secret").unwrap();
        assert_eq!(clean, "https://api.example.org/v1?q=banksia");
        assert!(saw);
        assert!(!clean.contains("super-secret"));
    }

    #[test]
    fn upsert_and_agent_enrichment_keep_secrets_out_of_bookmarks() {
        let _g = Guard::new();
        let created = dispatch(BookmarkArg {
            action: Some("upsert".into()),
            url: Some("https://ala.org.au/".into()),
            title: Some("Atlas of Living Australia".into()),
            domain: Some("biosphere".into()),
            ..BookmarkArg::default()
        })
        .unwrap();
        assert_eq!(created["item"]["kind"], "page");
        assert_eq!(created["item"]["needs_endpoint"], true);
        let id = created["item"]["id"].as_str().unwrap().to_owned();

        let enriched = dispatch(BookmarkArg {
            action: Some("upsert".into()),
            id: Some(id.clone()),
            data_endpoint: Some("https://biocache-ws.ala.org.au/ws".into()),
            kind: Some("rest".into()),
            requires_api_key: Some(true),
            api_key_env: Some("ALA_JWT".into()),
            agent_notes: Some("Protected APIs need JWT; public occurrence search does not.".into()),
            ..BookmarkArg::default()
        })
        .unwrap();
        assert_eq!(enriched["item"]["data_endpoint"], "https://biocache-ws.ala.org.au/ws");
        assert_eq!(enriched["item"]["needs_endpoint"], false);
        assert_eq!(enriched["item"]["needs_secret"], true);

        let err = dispatch(BookmarkArg {
            action: Some("upsert".into()),
            url: Some("https://example.org/data.ttl".into()),
            secret: Some("should-not-land-here".into()),
            ..BookmarkArg::default()
        })
        .unwrap_err();
        assert!(err.contains("set_secret"));

        dispatch(BookmarkArg {
            action: Some("set_secret".into()),
            id: Some(id.clone()),
            secret: Some("jwt-value".into()),
            api_key_env: Some("ALA_JWT".into()),
            ..BookmarkArg::default()
        })
        .unwrap();
        let listed = dispatch(BookmarkArg {
            action: Some("list".into()),
            ..BookmarkArg::default()
        })
        .unwrap();
        let text = listed.to_string();
        assert!(!text.contains("jwt-value"));
        assert_eq!(listed["items"][0]["secret_present"], true);
        let bookmarks = fs::read_to_string(
            Path::new(std::env::var("SYNTH_BOOKMARK_ROOT").unwrap().as_str())
                .join(".synth/bookmarks.json"),
        )
        .unwrap();
        assert!(!bookmarks.contains("jwt-value"));
        let gi = fs::read_to_string(
            Path::new(std::env::var("SYNTH_BOOKMARK_ROOT").unwrap().as_str())
                .join(".synth/.gitignore"),
        )
        .unwrap();
        assert!(gi.contains("secrets.json"));
    }
}
