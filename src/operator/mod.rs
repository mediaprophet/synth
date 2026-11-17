//! Operator shell: production rules, git-tag update check, activity log, browser page.
//!
//! The Aether **desktop** tray (Windows, macOS, Linux) lives behind `native-ui`.
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

const EMBEDDED_RULES: &str = include_str!("../../../ecopunk/docs/production-rules.json");
const MAX_EVENT_BYTES: u64 = 256 * 1024;
const MAX_EVENTS: usize = 120;

#[cfg(feature = "native-ui")]
mod aether_tray;
pub mod bookmarks;
pub mod update;

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
        "policy": "Never overwrite a locked synth.exe. Drop the MCP stdio child (/mcps reload), rebuild into target-prod, copy sidecar synth-mcp.exe. Aether tray uses --features native-ui. Signed GitHub Releases: synth update verify / apply.",
        "auto_replace": false,
        "releases_url": update::RELEASES_URL,
        "signing": update::signing_info()
    })
}

pub fn snapshot() -> Value {
    snapshot_with_fetch(false)
}

pub fn snapshot_with_fetch(fetch_remote: bool) -> Value {
    json!({
        "schema": "synth.operator-status/v1",
        "binary": std::env::current_exe().ok().map(|p| p.display().to_string()),
        "version": env!("CARGO_PKG_VERSION"),
        "morphisms": crate::production::assets::OPERATIONS.len(),
        "synth_tools": crate::mcp_full::StandaloneSynthMcp::synth_tools().len(),
        "input_root": std::env::var("SYNTH_INPUT_ROOT").ok(),
        "output_root": std::env::var("SYNTH_OUTPUT_ROOT").ok(),
        "encyclopedia": "ecopunk",
        "rules": rules_document(),
        "update": update_status(fetch_remote),
        "events": recent_events(),
        "bookmarks": bookmarks::summary(),
        "platforms": ["windows", "macos", "linux"],
        "tray": if cfg!(feature = "native-ui") {
            "aether-desktop tray_icon (Windows, macOS, Linux). Close hides the window; left-click restores."
        } else {
            "Build with --features native-ui for the Aether desktop tray. Browser /operator is the headless shell."
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
<title>Synth operator</title>
<style>
body{font:14px/1.45 system-ui,sans-serif;background:#0b1412;color:#d7eee6;margin:0;padding:24px;}
a{color:#7dd3fc} pre{background:#12211e;padding:12px;border-radius:8px;overflow:auto;max-height:50vh}
button{background:#142523;color:#e5f1ea;border:1px solid #2b4c46;border-radius:6px;padding:7px 12px;margin:0 8px 8px 0;cursor:pointer}
</style>
</head><body>
<h1>Synth operator</h1>
<p>Aether tray (Windows / macOS / Linux) plus this page. Biosphere non-fiction requires sourced .q42 evidence before modelling. Encyclopedia: Ecopunk.</p>
<p>
<button onclick="location.href='/'">Studio</button>
<button onclick="refresh(true)">Check remote tags</button>
<a href="https://github.com/mediaprophet/synth/releases">GitHub Releases</a>
</p>
<h2>Install / update</h2>
<p>Download a tagged binary, then <code>synth update verify --artifact FILE</code> and <code>synth update apply --artifact FILE</code>. Apply copies to a sibling path and never overwrites a locked <code>synth.exe</code>.</p>
<h2>Semantic bookmarks</h2>
<p>Paste an RDF dump, SPARQL endpoint, or landing-page URL. The agent can fill in a missing data endpoint or mark an API key. Keys go to <code>.synth/secrets.json</code> (gitignored), never GitHub.</p>
<p>
<input id="bm-url" type="url" placeholder="https://query.wikidata.org/sparql" style="width:min(520px,90%);padding:7px;background:#12211e;color:#d7eee6;border:1px solid #2b4c46;border-radius:6px"/>
<button onclick="addBookmark()">Bookmark URL</button>
</p>
<pre id="bookmarks">loading…</pre>
<pre id="status">loading…</pre>
<script>
async function refresh(fetchRemote){
  const q = fetchRemote ? '/api/operator?fetch=1' : '/api/operator';
  const r = await fetch(q);
  const j = await r.json();
  document.getElementById('status').textContent = JSON.stringify(j,null,2);
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
refresh(false);
refreshBookmarks();
setInterval(()=>{refresh(false); refreshBookmarks();}, 2500);
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
    let operator_url = format!("{url}/operator");
    println!("Synth operator workbench {operator_url}");
    println!("Studio {url}  (Aether tray on Windows/macOS/Linux with --features native-ui)");
    if open_browser {
        crate::web_server::open_in_system_browser(&operator_url);
    }
    #[cfg(feature = "native-ui")]
    if !headless {
        aether_tray::launch(url);
    }
    #[cfg(not(feature = "native-ui"))]
    let _ = headless;
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
    fn snapshot_names_ecopunk() {
        let snap = snapshot();
        assert_eq!(snap["encyclopedia"], "ecopunk");
        assert_eq!(snap["schema"], "synth.operator-status/v1");
        assert_eq!(snap["update"]["auto_replace"], false);
        assert_eq!(
            snap["update"]["signing"]["public_key_hex"],
            update::RELEASE_PUBLIC_KEY_HEX.trim()
        );
    }
}
