//! Operator shell: production rules, git-tag update check, activity log, browser page.
//!
//! The Webizen AI Synth system tray (Windows, macOS, Linux) lives behind `native-ui`.
//! Aether (Dioxus fork) is the desktop toolkit; `aether_tray` defines this app.
//! HTTP `/operator` and MCP tools work on every native Synth build.
use serde::Deserialize;
use serde_json::{json, Value};
use std::{
    fs,
    io::{BufRead, Write},
    path::{Path, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

const EMBEDDED_RULES: &str = include_str!("production-rules.json");
const MAX_EVENT_BYTES: u64 = 256 * 1024;
const MAX_EVENTS: usize = 120;

#[cfg(feature = "native-ui")]
mod aether_tray;
pub mod bookmarks;
pub mod branding;
pub mod jobs;
pub mod update;
pub use branding::APP_DISPLAY_NAME;

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default, deny_unknown_fields)]
pub struct RuleRequest {
    pub domain: Option<String>,
    pub taxon: Option<String>,
    pub fiction: Option<bool>,
    pub action: Option<String>,
    pub evidence_q42: Option<String>,
}

#[derive(Debug, Clone)]
pub struct RuleDecision {
    pub allowed: bool,
    pub rule_id: Option<String>,
    pub code: &'static str,
    pub message: String,
    pub require: Vec<String>,
}

impl RuleDecision {
    pub fn to_json(&self) -> Value {
        json!({
            "schema": "synth.production-rule-decision/v1",
            "allowed": self.allowed,
            "rule_id": self.rule_id,
            "code": self.code,
            "message": self.message,
            "require": self.require,
            "encyclopedia": "ecopunk",
        })
    }
}

fn rules_document() -> Value {
    if let Ok(root) = std::env::var("SYNTH_INPUT_ROOT") {
        let path = Path::new(root.trim()).join("ecopunk/docs/production-rules.json");
        if let Ok(bytes) = fs::read(&path) {
            if let Ok(value) = serde_json::from_slice::<Value>(&bytes) {
                return value;
            }
        }
    }
    serde_json::from_str(EMBEDDED_RULES).unwrap_or_else(|_| json!({ "rules": [] }))
}

fn is_biosphere_nonfiction(req: &RuleRequest) -> bool {
    let domain = req.domain.as_deref().unwrap_or("").to_ascii_lowercase();
    domain == "biosphere" && req.fiction != Some(true)
}

fn evidence_present(req: &RuleRequest) -> bool {
    if let Some(path) = req.evidence_q42.as_deref() {
        if Path::new(path).is_file() {
            return true;
        }
        if let Ok(root) = std::env::var("SYNTH_INPUT_ROOT") {
            if Path::new(root.trim()).join(path).is_file() {
                return true;
            }
        }
    }
    if let Some(taxon) = req.taxon.as_deref() {
        if taxon.is_empty() {
            return false;
        }
        if let Ok(root) = std::env::var("SYNTH_INPUT_ROOT") {
            let dir = Path::new(root.trim()).join("ecopunk/entities/taxa").join(taxon);
            if dir.is_dir() {
                if let Ok(entries) = fs::read_dir(&dir) {
                    for entry in entries.flatten() {
                        if entry.path().extension().and_then(|s| s.to_str()) == Some("q42") {
                            return true;
                        }
                    }
                }
            }
        }
    }
    false
}

/// Evaluate encyclopedia production rules. `compile_q42` / graph queries are the evidence path.
pub fn evaluate_rules(req: &RuleRequest) -> RuleDecision {
    let action = req.action.as_deref().unwrap_or("model").to_ascii_lowercase();
    let doc = rules_document();
    let Some(rules) = doc["rules"].as_array() else {
        return RuleDecision {
            allowed: true,
            rule_id: None,
            code: "ok",
            message: "No production rules loaded".into(),
            require: vec![],
        };
    };
    for rule in rules {
        let when = &rule["when"];
        if when["domain"].as_str() != Some("biosphere") {
            continue;
        }
        if !is_biosphere_nonfiction(req) {
            continue;
        }
        let allowed_without: Vec<&str> = rule["allowed_without_evidence"]
            .as_array()
            .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect())
            .unwrap_or_default();
        if allowed_without.iter().any(|a| *a == action) {
            return RuleDecision {
                allowed: true,
                rule_id: rule["id"].as_str().map(str::to_owned),
                code: "ok",
                message: format!("Action `{action}` is the evidence path for biosphere work"),
                require: vec![],
            };
        }
        if evidence_present(req) {
            return RuleDecision {
                allowed: true,
                rule_id: rule["id"].as_str().map(str::to_owned),
                code: "ok",
                message: "Biosphere evidence (.q42) is present".into(),
                require: vec![],
            };
        }
        let require: Vec<String> = rule["require"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(str::to_owned))
                    .collect()
            })
            .unwrap_or_default();
        return RuleDecision {
            allowed: false,
            rule_id: rule["id"].as_str().map(str::to_owned),
            code: "production_rule_blocked",
            message: format!(
                "Biosphere non-fiction taxon{} requires sourced evidence before `{action}`",
                req.taxon
                    .as_deref()
                    .map(|t| format!(" `{t}`"))
                    .unwrap_or_default()
            ),
            require,
        };
    }
    RuleDecision {
        allowed: true,
        rule_id: None,
        code: "ok",
        message: "No matching production rule".into(),
        require: vec![],
    }
}

pub fn enforce_produce(req: &RuleRequest) -> Result<(), String> {
    let decision = evaluate_rules(req);
    if decision.allowed {
        return Ok(());
    }
    let steps = decision.require.join(" ");
    Err(format!("{} — {}", decision.message, steps))
}

fn events_path() -> Option<PathBuf> {
    let root = std::env::var("SYNTH_OUTPUT_ROOT").ok()?;
    let trimmed = root.trim();
    if trimmed.is_empty() {
        return None;
    }
    Some(Path::new(trimmed).join(".synth").join("operator-events.jsonl"))
}

pub fn record_event(tool: &str, ok: bool, detail: Option<&str>) {
    let Some(path) = events_path() else {
        return;
    };
    let _ = fs::create_dir_all(path.parent().unwrap_or(Path::new(".")));
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let line = json!({
        "ts": ts,
        "tool": tool,
        "ok": ok,
        "detail": detail,
    });
    if let Ok(mut file) = fs::OpenOptions::new().create(true).append(true).open(&path) {
        let _ = writeln!(file, "{line}");
    }
    if let Ok(meta) = fs::metadata(&path) {
        if meta.len() > MAX_EVENT_BYTES {
            if let Ok(bytes) = fs::read(&path) {
                let text = String::from_utf8_lossy(&bytes);
                let kept: Vec<&str> = text.lines().rev().take(MAX_EVENTS).collect();
                let mut kept = kept;
                kept.reverse();
                let _ = fs::write(&path, kept.join("\n") + "\n");
            }
        }
    }
}

pub fn recent_events() -> Vec<Value> {
    let Some(path) = events_path() else {
        return vec![];
    };
    let Ok(file) = fs::File::open(path) else {
        return vec![];
    };
    let lines: Vec<String> = std::io::BufReader::new(file)
        .lines()
        .map_while(Result::ok)
        .collect();
    lines
        .iter()
        .rev()
        .take(40)
        .rev()
        .filter_map(|line| serde_json::from_str(line).ok())
        .collect()
}

fn git(repo: &Path, args: &[&str]) -> Option<String> {
    let out = Command::new("git")
        .args(args)
        .current_dir(repo)
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&out.stdout).trim().to_owned();
    if text.is_empty() {
        None
    } else {
        Some(text)
    }
}

fn synth_repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Local git describe plus optional `git ls-remote --tags`. Never replaces a running exe.
pub fn update_status(fetch_remote: bool) -> Value {
    let repo = synth_repo();
    let describe = git(&repo, &["describe", "--tags", "--always", "--dirty"]).unwrap_or_else(|| "unknown".into());
    let head = git(&repo, &["rev-parse", "--short", "HEAD"]);
    let local_tags = git(&repo, &["tag", "--list", "--sort=-creatordate"])
        .unwrap_or_default()
        .lines()
        .take(8)
        .map(str::to_owned)
        .collect::<Vec<_>>();
    let mut remote_latest = Value::Null;
    if fetch_remote {
        if let Some(raw) = git(&repo, &["ls-remote", "--tags", "--refs", "origin"]) {
            let last = raw.lines().last().unwrap_or("");
            let tag = last.split('/').next_back().unwrap_or("");
            if !tag.is_empty() {
                remote_latest = json!(tag);
            }
        }
    }
    json!({
        "schema": "synth.operator-update/v1",
        "repo": repo.display().to_string(),
        "describe": describe,
        "head": head,
        "local_tags": local_tags,
        "remote_latest_tag": remote_latest,
        "policy": "Never overwrite a locked synth.exe. Drop the MCP stdio child (/mcps reload), rebuild into target-prod, copy sidecar synth-mcp.exe. Webizen AI Synth tray uses --features native-ui. Signed GitHub Releases: Upgrade button, synth update upgrade, or verify / apply.",
        "auto_replace": false,
        "releases_url": update::RELEASES_URL,
        "signing": update::signing_info(),
        "github": update::github_release_status(fetch_remote)
    })
}

pub fn snapshot() -> Value {
    snapshot_with_fetch(false)
}

pub fn snapshot_with_fetch(fetch_remote: bool) -> Value {
    json!({
        "schema": "synth.operator-status/v1",
        "binary": std::env::current_exe().ok().map(|p| p.display().to_string()),
        "version": crate::agent_index::RELEASE_VERSION,
        "role": crate::agent_index::ROLE,
        "runtime": crate::agent_index::RUNTIME,
        "morphisms": crate::production::assets::OPERATIONS.len(),
        "synth_tools": crate::mcp_full::StandaloneSynthMcp::synth_tools().len(),
        "input_root": std::env::var("SYNTH_INPUT_ROOT").ok(),
        "output_root": std::env::var("SYNTH_OUTPUT_ROOT").ok(),
        "app": branding::APP_DISPLAY_NAME,
        "pid": std::process::id(),
        "encyclopedia": "ecopunk",
        "rules": rules_document(),
        "update": update_status(fetch_remote),
        "events": recent_events(),
        "bookmarks": bookmarks::summary(),
        "jobs": jobs::summary(),
        "platforms": ["windows", "macos", "linux"],
        "tray": if cfg!(feature = "native-ui") {
            "Webizen AI Synth system tray (Windows, macOS, Linux). Close hides the window; left-click restores. Right-click for Show / Operator / Studio / Upgrade / Exit."
        } else {
            "Build with --features native-ui for the Webizen AI Synth system tray. Browser /operator is the headless shell."
        }
    })
}

pub fn rules_json() -> Value {
    rules_document()
}

pub fn page_html() -> String {
    r#"<!DOCTYPE html>
<html lang="en"><head>
<meta charset="utf-8"/>
<title>Webizen AI Synth</title>
<link rel="icon" href="/operator/icon.png" type="image/png"/>
<link rel="apple-touch-icon" href="/operator/icon-256.png"/>
<style>
body{font:14px/1.45 system-ui,sans-serif;background:#0b1412;color:#d7eee6;margin:0;padding:24px;}
a{color:#7dd3fc} pre{background:#12211e;padding:12px;border-radius:8px;overflow:auto;max-height:50vh}
button{background:#142523;color:#e5f1ea;border:1px solid #2b4c46;border-radius:6px;padding:7px 12px;margin:0 8px 8px 0;cursor:pointer}
.brand{display:flex;align-items:center;gap:14px;margin-bottom:12px;}
.brand img.emblem{width:48px;height:48px}
.brand img.banner{width:280px;height:98px;border-radius:10px;border:1px solid #1e3330}
</style>
</head><body>
<div class="brand">
<img class="emblem" src="/operator/emblem.svg" alt="Webizen AI Synth"/>
<div>
<h1 style="margin:0">Webizen AI Synth</h1>
<p style="margin:4px 0 0 0;color:#8fb5a7">Operator workbench</p>
</div>
<img class="banner" src="/operator/banner.png" alt="Webizen AI Synth banner"/>
</div>
<p>Webizen AI Synth system tray (Windows / macOS / Linux) plus this page. Close hides to the emerald tray mark; right-click that icon for Show / Operator / Studio / Upgrade / Exit. Biosphere non-fiction requires sourced .q42 evidence before modelling. Encyclopedia: Ecopunk. The operator ticks due jobs every few seconds.</p>
<div id="live" style="display:grid;grid-template-columns:repeat(auto-fit,minmax(180px,1fr));gap:10px;margin:16px 0;"></div>
<p>
<button onclick="location.href='/'">Studio</button>
<button id="upgrade-btn" onclick="upgrade()" style="background:#134e4a;color:#99f6e4;border:1px solid #2dd4bf;">Upgrade</button>
<button onclick="refresh(true)">Check remote tags</button>
<button onclick="shutdown()" style="background:#2c1618;color:#fca5a5;border:1px solid #ef4444;">Exit</button>
<a href="https://github.com/mediaprophet/synth/releases">GitHub Releases</a>
</p>
<h2>Install / update</h2>
<p>Upgrade downloads the latest signed portable from GitHub Releases, verifies Ed25519 (<code>synth.release.sha256.v1</code>), and copies it next to this process as <code>synth.new.exe</code> (or the platform equivalent). It never overwrites a locked <code>synth.exe</code>. Prefer the signed installer for a first install. CLI: <code>synth update upgrade</code>, or <code>synth update verify --artifact FILE</code> then <code>synth update apply --artifact FILE</code>.</p>
<pre id="update">Ready.</pre>
<h2>Semantic bookmarks</h2>
<p>Paste an RDF dump, SPARQL endpoint, or landing-page URL. Keys go to <code>.synth/secrets.json</code> (gitignored), never GitHub.</p>
<p>
<input id="bm-url" type="url" placeholder="https://query.wikidata.org/sparql" style="width:min(520px,90%);padding:7px;background:#12211e;color:#d7eee6;border:1px solid #2b4c46;border-radius:6px"/>
<button onclick="addBookmark()">Bookmark URL</button>
</p>
<pre id="bookmarks">loading…</pre>
<h2>Jobs</h2>
<p>Download into <code>.synth/inbox</code> (staging), wrap PROV/DCT/DCAT, compile Turtle to <code>.q42</code>. One-shot unless you set an interval. Scheduler runs in this operator process.</p>
<p>
<input id="job-url" type="url" placeholder="https://w3id.org/tern/ontologies/tern.ttl" style="width:min(520px,90%);padding:7px;background:#12211e;color:#d7eee6;border:1px solid #2b4c46;border-radius:6px"/>
<button onclick="addJob()">Add ingest job</button>
<button onclick="runDue()">Run due jobs</button>
</p>
<pre id="jobs">loading…</pre>
<pre id="status">loading…</pre>
<script>
function card(label, value){
  return '<div style="background:#12211e;border:1px solid #233e38;border-radius:8px;padding:10px 14px;"><p style="margin:0;font-size:11px;color:#8fb5a7;">'+label+'</p><p style="margin:4px 0 0 0;font-size:15px;">'+value+'</p></div>';
}
async function refresh(fetchRemote){
  const q = fetchRemote ? '/api/operator?fetch=1' : '/api/operator';
  const r = await fetch(q);
  const j = await r.json();
  document.getElementById('status').textContent = JSON.stringify(j,null,2);
  const live = document.getElementById('live');
  if (live) {
    const events = Array.isArray(j.events) ? j.events.length : 0;
    live.innerHTML = [
      card('APP', j.app || 'Webizen AI Synth'),
      card('PID', j.pid || ''),
      card('VERSION', j.version || ''),
      card('MORPHISMS', j.morphisms || ''),
      card('TRAY', j.tray || ''),
      card('EVENTS', events),
      card('JOBS', (j.jobs && j.jobs.count) || 0),
      card('JOBS DUE', (j.jobs && j.jobs.due) || 0),
      card('RELEASE', (j.update && j.update.github && (j.update.github.tag || j.update.github.error)) || (j.update && j.update.remote_latest_tag) || '')
    ].join('');
  }
}
async function upgrade(){
  const btn = document.getElementById('upgrade-btn');
  const out = document.getElementById('update');
  if (btn) btn.disabled = true;
  if (out) out.textContent = 'Downloading the signed GitHub Release… this can take a few minutes. The running synth.exe is never overwritten.';
  try {
    const r = await fetch('/api/update', {method:'POST', headers:{'content-type':'application/json'}, body: JSON.stringify({action:'upgrade'})});
    const j = await r.json();
    if (out) out.textContent = JSON.stringify(j,null,2);
    refresh(true);
  } catch (e) {
    if (out) out.textContent = String(e);
  }
  if (btn) btn.disabled = false;
}
async function shutdown(){
  await fetch('/api/operator/shutdown', {method:'POST'});
}
async function refreshBookmarks(){
  const r = await fetch('/api/bookmarks');
  const j = await r.json();
  document.getElementById('bookmarks').textContent = JSON.stringify(j,null,2);
}
async function addBookmark(){
  const url = document.getElementById('bm-url').value.trim();
  if(!url) return;
  const r = await fetch('/api/bookmarks', {method:'POST', headers:{'content-type':'application/json'}, body: JSON.stringify({action:'upsert', url})});
  const j = await r.json();
  document.getElementById('bm-url').value = '';
  document.getElementById('bookmarks').textContent = JSON.stringify(j,null,2);
  refreshBookmarks();
}
async function refreshJobs(){
  const r = await fetch('/api/jobs');
  const j = await r.json();
  document.getElementById('jobs').textContent = JSON.stringify(j,null,2);
}
async function addJob(){
  const url = document.getElementById('job-url').value.trim();
  if(!url) return;
  const r = await fetch('/api/jobs', {method:'POST', headers:{'content-type':'application/json'}, body: JSON.stringify({action:'add', url, kind:'ingest'})});
  const j = await r.json();
  document.getElementById('job-url').value = '';
  document.getElementById('jobs').textContent = JSON.stringify(j,null,2);
  refreshJobs();
}
async function runDue(){
  const r = await fetch('/api/jobs', {method:'POST', headers:{'content-type':'application/json'}, body: JSON.stringify({action:'run_due'})});
  const j = await r.json();
  document.getElementById('jobs').textContent = JSON.stringify(j,null,2);
  refreshJobs();
}
refresh(false);
refreshBookmarks();
refreshJobs();
setInterval(()=>{refresh(false); refreshBookmarks(); refreshJobs();}, 2500);
if (location.hash === '#upgrade') upgrade();
</script>
</body></html>"#
        .to_owned()
}

/// `synth operator [--port N] [--no-browser] [--headless]`
pub fn run_cli(args: &[String]) -> Result<(), String> {
    let mut port = None;
    let mut open_browser = true;
    let mut headless = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--port" => {
                i += 1;
                let raw = args.get(i).ok_or("Usage: synth operator [--port PORT] [--no-browser] [--headless]")?;
                port = Some(raw.parse::<u16>().map_err(|_| "port must be u16")?);
            }
            "--no-browser" => open_browser = false,
            "--headless" => headless = true,
            other => return Err(format!("Unknown `synth operator` flag `{other}`")),
        }
        i += 1;
    }
    let url = crate::web_server::spawn_background_server(port)?;
    jobs::spawn_scheduler();
    let operator_url = format!("{url}/operator");
    println!("{} workbench {operator_url}", branding::APP_DISPLAY_NAME);
    println!("Studio {url}  (Webizen AI Synth tray on Windows/macOS/Linux with --features native-ui)");
    if open_browser {
        crate::web_server::open_in_system_browser(&operator_url);
    }
    #[cfg(feature = "native-ui")]
    if !headless {
        aether_tray::launch(url);
        return Ok(());
    }
    #[cfg(not(feature = "native-ui"))]
    let _ = headless;
    println!("Headless operator running. Ctrl+C exits. Open {operator_url}");
    loop {
        std::thread::sleep(std::time::Duration::from_secs(60));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn biosphere_model_without_evidence_is_blocked() {
        let blocked = evaluate_rules(&RuleRequest {
            domain: Some("biosphere".into()),
            taxon: Some("corymbia-aparrerinja".into()),
            fiction: Some(false),
            action: Some("model".into()),
            evidence_q42: None,
        });
        assert!(!blocked.allowed);
        assert_eq!(blocked.code, "production_rule_blocked");
        assert!(blocked.require.iter().any(|s| s.contains("synth_compile_q42")));
    }

    #[test]
    fn compile_q42_is_the_evidence_path() {
        let ok = evaluate_rules(&RuleRequest {
            domain: Some("biosphere".into()),
            action: Some("compile_q42".into()),
            ..RuleRequest::default()
        });
        assert!(ok.allowed);
        let ingest = evaluate_rules(&RuleRequest {
            domain: Some("biosphere".into()),
            action: Some("ingest".into()),
            ..RuleRequest::default()
        });
        assert!(ingest.allowed);
    }

    #[test]
    fn fiction_is_not_gated() {
        let ok = evaluate_rules(&RuleRequest {
            domain: Some("biosphere".into()),
            fiction: Some(true),
            action: Some("produce".into()),
            ..RuleRequest::default()
        });
        assert!(ok.allowed);
    }

    #[test]
    fn branding_assets_are_nonempty() {
        assert_eq!(branding::APP_DISPLAY_NAME, "Webizen AI Synth");
        assert_eq!(branding::APP_BUNDLE_ID, "au.webizen.synth");
        assert!(!branding::ICON_32_PNG.is_empty());
        assert!(!branding::ICON_PNG.is_empty());
        assert!(!branding::EMBLEM_SVG.is_empty());
        assert!(branding::asset("icon-32.png").is_some());
    }

    #[test]
    fn toolkit_identity_names_the_app() {
        branding::claim_toolkit_identity();
        assert_eq!(std::env::var("AETHER_APP_TITLE").unwrap(), "Webizen AI Synth");
        assert_eq!(
            std::env::var("AETHER_PRODUCT_NAME").unwrap(),
            "Webizen AI Synth"
        );
    }

    #[test]
    fn snapshot_names_ecopunk() {
        let snap = snapshot();
        assert_eq!(snap["app"], "Webizen AI Synth");
        assert_eq!(snap["encyclopedia"], "ecopunk");
        assert_eq!(snap["schema"], "synth.operator-status/v1");
        assert_eq!(snap["update"]["auto_replace"], false);
        assert!(snap["update"]["github"].is_null());
        assert_eq!(
            snap["update"]["signing"]["public_key_hex"],
            update::RELEASE_PUBLIC_KEY_HEX.trim()
        );
        assert!(snap["jobs"].is_object());
        assert_eq!(snap["jobs"]["schema"], "synth.jobs/v1");
    }
}
