//! Local HTTP Studio & Capabilities Workbench Server (`http://127.0.0.1:8787`).
//!
//! Explicitly avoids QualiaDB / Webizen Desktop reserved ports (`8080`, `4242`, `9001`).
//! Serves the interactive 5-tab Synth Studio UI (`GET /`) for QualiaDB demo pages,
//! the project workbench (`GET /project`), QualiaPortal WASM (`GET /qualia/` from
//! `C:\\github\\qualiaDB\\docs`), and live JSON endpoints:
//! - `GET /api/status`
//! - `GET /operator` (rules, events, git tags, bookmarks; the Webizen AI Synth tray uses this page)
//! - `GET /api/operator` (`?fetch=1` checks remote git tags; never auto-replaces the exe)
//! - `GET|POST /api/update` (signed GitHub release check / upgrade / verify / apply; never overwrites locked exe)
//! - `GET|POST /api/bookmarks` (semantic RDF/SPARQL bookmarks in `.synth/`; secrets never returned)
//! - `GET|POST /api/jobs` (download / wrap / compile jobs; operator scheduler ticks due jobs)
//! - `GET /api/capabilities`
//! - `POST /api/produce`
//! - `POST /api/mcp/call`
//! - `POST /api/radiance/compose`
//! - `POST /api/hmc/bridge`
//! - `GET /synth/wasm-lite.js`
//! - `GET /qualia` (QualiaPortal docs index; files under `/qualia/`)
//! - `GET /api/project`
//! - `GET|POST /api/project/annotations`

use crate::{
    present::render_session_html,
    present_capabilities::FullCapabilitiesSnapshot,
    present_project::render_project_html,
    production::{ProducerRegistry, ProductionLimits},
    project::{self, Annotation, AnnotationLink},
    tray_env::{environment_report_json, TRAY_ENVIRONMENT_SOURCE},
};
use qualia_mcp::authoring::AuthoringBackend;
use serde::Deserialize;
use serde_json::{json, Value};
use std::{
    io::{Read, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    path::{Component, Path, PathBuf},
    process::Command,
    sync::atomic::AtomicBool,
    thread,
    time::Duration,
};

mod website;
mod production_response;
pub use website::configure_website_root;

pub const DEFAULT_STUDIO_PORT: u16 = 8787;
pub const WEBIZEN_RESERVED_PORTS: &[u16] = &[8080, 4242, 9001];

#[derive(Deserialize)]
struct ProduceRequest {
    source: String,
    profile: Option<String>,
    include_html: Option<bool>,
    include_resources: Option<bool>,
    include_package: Option<bool>,
}

#[derive(Deserialize)]
struct McpCallRequest {
    name: String,
    #[serde(default)]
    arguments: Value,
}

pub fn is_reserved_webizen_port(port: u16) -> bool {
    WEBIZEN_RESERVED_PORTS.contains(&port)
}

pub fn resolve_preferred_port(explicit: Option<u16>) -> Result<u16, String> {
    if let Some(p) = explicit {
        if is_reserved_webizen_port(p) {
            return Err(format!(
                "Port {p} is reserved for QualiaDB / Webizen Desktop (8080, 4242, 9001); use 8787+"
            ));
        }
        return Ok(p);
    }
    if let Ok(raw) = std::env::var("SYNTH_STUDIO_PORT") {
        if let Ok(p) = raw.trim().parse::<u16>() {
            if !is_reserved_webizen_port(p) && p > 0 {
                return Ok(p);
            }
        }
    }
    Ok(DEFAULT_STUDIO_PORT)
}

pub fn bind_studio_listener(preferred: Option<u16>) -> Result<(TcpListener, u16), String> {
    let base = resolve_preferred_port(preferred)?;
    if preferred.is_some() {
        let addr = SocketAddr::from(([127, 0, 0, 1], base));
        let listener = TcpListener::bind(addr)
            .map_err(|e| format!("Failed to bind 127.0.0.1:{base}: {e}"))?;
        let actual = listener.local_addr().map(|a| a.port()).unwrap_or(base);
        return Ok((listener, actual));
    }
    for candidate in base..=base.saturating_add(15) {
        if is_reserved_webizen_port(candidate) {
            continue;
        }
        let addr = SocketAddr::from(([127, 0, 0, 1], candidate));
        if let Ok(listener) = TcpListener::bind(addr) {
            let actual = listener.local_addr().map(|a| a.port()).unwrap_or(candidate);
            return Ok((listener, actual));
        }
    }
    Err(format!(
        "Could not bind any local Synth Studio port in {base}..={}",
        base.saturating_add(15)
    ))
}

pub fn open_in_system_browser(url: &str) {
    #[cfg(target_os = "windows")]
    {
        let _ = Command::new("cmd").args(["/C", "start", "", url]).spawn();
    }
    #[cfg(target_os = "macos")]
    {
        let _ = Command::new("open").arg(url).spawn();
    }
    #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
    {
        let _ = Command::new("xdg-open").arg(url).spawn();
    }
}

/// Spawns the local Synth Studio HTTP server on a background thread (`http://127.0.0.1:8787`)
/// and returns its bound localhost URL.
pub fn spawn_background_server(preferred_port: Option<u16>) -> Result<String, String> {
    let (listener, port) = bind_studio_listener(preferred_port)?;
    let url = format!("http://127.0.0.1:{port}");
    thread::Builder::new()
        .name("synth-studio-http".into())
        .spawn(move || {
            for stream in listener.incoming().flatten() {
                let _ = thread::Builder::new()
                    .name("synth-http-worker".into())
                    .stack_size(16 * 1024 * 1024)
                    .spawn(move || {
                        let _ = handle_connection(stream);
                    });
            }
        })
        .map_err(|e| e.to_string())?;
    Ok(url)
}

/// Runs the local Synth Studio HTTP server in the foreground (`synth serve`).
pub fn serve_blocking(preferred_port: Option<u16>, open_browser: bool) -> Result<(), String> {
    let (listener, port) = bind_studio_listener(preferred_port)?;
    let url = format!("http://127.0.0.1:{port}");
    println!("Synth Studio & Capabilities Workbench listening on {url}");
    println!("(Non-conflicting with Webizen Desktop ports 8080 / 4242 / 9001)");
    #[cfg(not(target_arch = "wasm32"))]
    crate::operator::jobs::spawn_scheduler();
    if let Some(project) = project::current() {
        println!(
            "Project workbench: {url}/project  ({})",
            project.name
        );
        if open_browser {
            open_in_system_browser(&format!("{url}/project"));
        }
    } else if open_browser {
        open_in_system_browser(&url);
    }
    for stream in listener.incoming().flatten() {
        let _ = thread::Builder::new()
            .name("synth-http-worker".into())
            .stack_size(16 * 1024 * 1024)
            .spawn(move || {
                let _ = handle_connection(stream);
            });
    }
    Ok(())
}

fn write_http_response(
    stream: TcpStream,
    status: &str,
    content_type: &str,
    body: &[u8],
) -> Result<(), String> {
    write_http_response_with(stream, status, content_type, body, &[])
}

fn write_http_response_with(
    mut stream: TcpStream,
    status: &str,
    content_type: &str,
    body: &[u8],
    extra_headers: &[(&str, &str)],
) -> Result<(), String> {
    let mut header = format!(
        "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: http://127.0.0.1\r\nConnection: close\r\n",
        body.len()
    );
    for (name, value) in extra_headers {
        header.push_str(name);
        header.push_str(": ");
        header.push_str(value);
        header.push_str("\r\n");
    }
    header.push_str("\r\n");
    stream
        .write_all(header.as_bytes())
        .and_then(|_| stream.write_all(body))
        .and_then(|_| stream.flush())
        .map_err(|e| e.to_string())
}

/// Single RFC 7233 `bytes=` range (inclusive start/end). Anything else is unsatisfiable.
fn parse_single_byte_range(header: &str, total: usize) -> Option<(usize, usize)> {
    let spec = header.trim();
    let eq = spec.find('=')?;
    if !spec[..eq].trim().eq_ignore_ascii_case("bytes") {
        return None;
    }
    let rest = spec[eq + 1..].trim();
    if rest.is_empty() || rest.contains(',') {
        return None;
    }
    let (start_s, end_s) = rest.split_once('-')?;
    let start_s = start_s.trim();
    let end_s = end_s.trim();
    if total == 0 {
        return None;
    }
    if start_s.is_empty() {
        let n: usize = end_s.parse().ok()?;
        if n == 0 {
            return None;
        }
        let start = total.saturating_sub(n);
        return Some((start, total - 1));
    }
    let start: usize = start_s.parse().ok()?;
    if start >= total {
        return None;
    }
    let end = if end_s.is_empty() {
        total - 1
    } else {
        let end: usize = end_s.parse().ok()?;
        if end < start {
            return None;
        }
        end.min(total - 1)
    };
    Some((start, end))
}

fn write_ranged_bytes(
    stream: TcpStream,
    content_type: &str,
    body: &[u8],
    range_header: Option<&str>,
) -> Result<(), String> {
    let total = body.len();
    let coi = ["text/html", "application/javascript", "text/javascript", "application/wasm"]
        .iter().any(|mime| content_type.starts_with(mime));
    // Present Range (including empty/whitespace) is fail-closed 416; only a missing header is 200.
    match range_header {
        None => {
            let mut extra = vec![
                ("Accept-Ranges", "bytes"),
                ("Cross-Origin-Resource-Policy", "same-origin"),
            ];
            if coi {
                extra.push(("Cross-Origin-Opener-Policy", "same-origin"));
                extra.push(("Cross-Origin-Embedder-Policy", "require-corp"));
            }
            write_http_response_with(stream, "200 OK", content_type, body, &extra)
        }
        Some(header) => match parse_single_byte_range(header, total) {
            Some((start, end)) => {
                let content_range = format!("bytes {start}-{end}/{total}");
                let mut extra = vec![
                    ("Accept-Ranges", "bytes"),
                    ("Content-Range", content_range.as_str()),
                    ("Cross-Origin-Resource-Policy", "same-origin"),
                ];
                if coi {
                    extra.push(("Cross-Origin-Opener-Policy", "same-origin"));
                    extra.push(("Cross-Origin-Embedder-Policy", "require-corp"));
                }
                write_http_response_with(
                    stream,
                    "206 Partial Content",
                    content_type,
                    &body[start..=end],
                    &extra,
                )
            }
            None => {
                let content_range = format!("bytes */{total}");
                write_http_response_with(
                    stream,
                    "416 Range Not Satisfiable",
                    "application/octet-stream",
                    b"",
                    &[
                        ("Accept-Ranges", "bytes"),
                        ("Content-Range", &content_range),
                    ],
                )
            }
        },
    }
}

const HTML_COI_HEADERS: &[(&str, &str)] = &[
    ("Cross-Origin-Opener-Policy", "same-origin"),
    ("Cross-Origin-Embedder-Policy", "require-corp"),
    ("Cross-Origin-Resource-Policy", "same-origin"),
];

fn write_html_coi(stream: TcpStream, body: &[u8]) -> Result<(), String> {
    write_http_response_with(
        stream,
        "200 OK",
        "text/html; charset=utf-8",
        body,
        HTML_COI_HEADERS,
    )
}

/// QualiaPortal static files: `C:\github\qualiaDB\docs` served at `/qualia/` (alias `/vendor/qualia/`).
pub fn qualia_docs_root() -> Option<PathBuf> {
    if let Ok(raw) = std::env::var("SYNTH_QUALIA_DOCS") {
        let p = PathBuf::from(raw.trim());
        if let Ok(canon) = p.canonicalize() {
            if canon.is_dir() {
                return Some(canon);
            }
        }
    }
    let candidates = [
        PathBuf::from(r"C:\github\qualiaDB\docs"),
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("qualiaDB")
            .join("docs"),
    ];
    for c in candidates {
        if let Ok(canon) = c.canonicalize() {
            if canon.is_dir() {
                return Some(canon);
            }
        }
    }
    None
}

/// Path-confine a `/qualia/` relative path. Any `..` (including percent-encoded) is rejected.
pub fn confine_qualia_rel(rest: &str) -> Option<PathBuf> {
    let decoded = percent_decode(rest);
    if decoded.is_empty()
        || decoded.contains('\0')
        || decoded.contains('\\')
        || decoded.starts_with('/')
        || decoded.starts_with("..")
    {
        return None;
    }
    let mut clean = PathBuf::new();
    for comp in Path::new(&decoded).components() {
        match comp {
            Component::Normal(s) => {
                let s = s.to_str()?;
                if s == ".." || s.contains(':') {
                    return None;
                }
                clean.push(s);
            }
            Component::CurDir => {}
            _ => return None,
        }
    }
    if clean.as_os_str().is_empty() {
        None
    } else {
        Some(clean)
    }
}

fn mime_for_qualia(rel: &str) -> &'static str {
    let lower = rel.to_ascii_lowercase();
    if lower.ends_with(".js") || lower.ends_with(".mjs") {
        "text/javascript; charset=utf-8"
    } else if lower.ends_with(".wasm") {
        "application/wasm"
    } else if lower.ends_with(".json") {
        "application/json"
    } else if lower.ends_with(".html") || lower.ends_with(".htm") {
        "text/html; charset=utf-8"
    } else if lower.ends_with(".css") {
        "text/css; charset=utf-8"
    } else if lower.ends_with(".svg") {
        "image/svg+xml"
    } else if lower.ends_with(".png") {
        "image/png"
    } else if lower.ends_with(".jpg") || lower.ends_with(".jpeg") {
        "image/jpeg"
    } else if lower.ends_with(".webp") {
        "image/webp"
    } else if lower.ends_with(".map") || lower.ends_with(".txt") || lower.ends_with(".md") {
        "text/plain; charset=utf-8"
    } else if lower.ends_with(".glsl") || lower.ends_with(".wgsl") {
        "text/plain; charset=utf-8"
    } else if lower.ends_with(".ttl") || lower.ends_with(".n3") {
        "text/turtle; charset=utf-8"
    } else if lower.ends_with(".nt") {
        "application/n-triples"
    } else if lower.ends_with(".nq") {
        "application/n-quads"
    } else {
        "application/octet-stream"
    }
}

fn extra_static_headers(mime: &str) -> Vec<(&'static str, &'static str)> {
    let mut h = vec![("Cross-Origin-Resource-Policy", "same-origin")];
    let lower = mime.to_ascii_lowercase();
    if lower.starts_with("text/html") || lower.contains("javascript") || lower.contains("wasm") {
        h.push(("Cross-Origin-Opener-Policy", "same-origin"));
        h.push(("Cross-Origin-Embedder-Policy", "require-corp"));
    }
    h
}

fn serve_qualia_static(stream: TcpStream, path: &str) -> Result<(), String> {
    let Some(rest) = path
        .strip_prefix("/qualia/")
        .or_else(|| path.strip_prefix("/vendor/qualia/"))
    else {
        return write_http_response(
            stream,
            "404 Not Found",
            "application/json",
            br#"{"ok":false,"error":"Not found"}"#,
        );
    };
    let Some(rel) = confine_qualia_rel(rest) else {
        return write_http_response(
            stream,
            "404 Not Found",
            "application/json",
            br#"{"ok":false,"error":"Not found"}"#,
        );
    };
    let Some(root) = qualia_docs_root() else {
        return write_http_response(
            stream,
            "404 Not Found",
            "application/json",
            br#"{"ok":false,"error":"QualiaDB docs not found"}"#,
        );
    };
    let joined = root.join(&rel);
    let Ok(canon) = joined.canonicalize() else {
        return write_http_response(
            stream,
            "404 Not Found",
            "application/json",
            br#"{"ok":false,"error":"Not found"}"#,
        );
    };
    if !canon.starts_with(&root) || !canon.is_file() {
        return write_http_response(
            stream,
            "404 Not Found",
            "application/json",
            br#"{"ok":false,"error":"Not found"}"#,
        );
    }
    let bytes = std::fs::read(&canon).map_err(|e| e.to_string())?;
    let rel_str = rel.to_string_lossy();
    let mime = mime_for_qualia(&rel_str);
    let extra = extra_static_headers(mime);
    write_http_response_with(stream, "200 OK", mime, &bytes, &extra)
}

fn list_qualia_docs(root: &Path, max: usize) -> Vec<String> {
    let mut files = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let name = entry.file_name();
            let Some(name) = name.to_str() else {
                continue;
            };
            if name == "." || name == ".." || name.contains("..") {
                continue;
            }
            let path = entry.path();
            let Ok(meta) = entry.metadata() else {
                continue;
            };
            if meta.is_dir() {
                stack.push(path);
                continue;
            }
            if !meta.is_file() {
                continue;
            }
            let Ok(rel) = path.strip_prefix(root) else {
                continue;
            };
            let rel_str = rel.to_string_lossy().replace('\\', "/");
            if rel_str.is_empty()
                || rel_str.split('/').any(|c| c.is_empty() || c == "." || c == "..")
            {
                continue;
            }
            files.push(rel_str);
            if files.len() >= max {
                files.sort();
                return files;
            }
        }
    }
    files.sort();
    files
}

fn serve_qualia_index(stream: TcpStream) -> Result<(), String> {
    let Some(root) = qualia_docs_root() else {
        return write_http_response(
            stream,
            "404 Not Found",
            "application/json",
            br#"{"ok":false,"error":"QualiaDB docs not found"}"#,
        );
    };
    let files = list_qualia_docs(&root, 200);
    let mut links = String::new();
    for rel in &files {
        let href = format!("/qualia/{rel}");
        links.push_str(&format!(
            "<li><a href=\"{safe_href}\">{safe_rel}</a></li>\n",
            safe_href = crate::present::escape_html(&href),
            safe_rel = crate::present::escape_html(rel),
        ));
    }
    let html = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8" />
  <title>QualiaPortal docs</title>
</head>
<body>
  <h1>QualiaPortal docs</h1>
  <p>Official painter: <code>load_10d</code> + sibling <code>load_emf</code>. Shell: <code>/qualia/js/qualia-shell.js</code>. WASM package: <code>pkg/qualia/qualia.js</code>.</p>
  <ul>
{links}
  </ul>
</body>
</html>
"#
    );
    write_html_coi(stream, html.as_bytes())
}

fn render_root_studio_html() -> Result<String, String> {
    let registry = ProducerRegistry::with_builtins();
    let cancel = AtomicBool::new(false);
    let mut session = registry.session("files", vec![], ProductionLimits::default())?;
    session.submit(TRAY_ENVIRONMENT_SOURCE, &cancel)?;
    render_session_html(&session)
}

fn handle_produce(body: &[u8]) -> Value {
    let Ok(req) = serde_json::from_slice::<ProduceRequest>(body) else {
        return json!({ "ok": false, "error": "Invalid JSON body for /api/produce" });
    };
    let limits = ProductionLimits::default();
    let cancel = AtomicBool::new(false);
    let registry = ProducerRegistry::with_builtins();
    let mut session = match registry.session(
        req.profile.as_deref().unwrap_or("files"),
        vec![],
        limits.clone(),
    ) {
        Ok(s) => s,
        Err(e) => return json!({ "ok": false, "error": e }),
    };
    if let Err(e) = session.submit(&req.source, &cancel) {
        return json!({ "ok": false, "error": e });
    }
    let Some(candidate) = session.candidate() else {
        return json!({ "ok": false, "error": "No candidate produced" });
    };
    if let Err(e) = registry.reopen(candidate.bundle(), limits, &cancel) {
        return json!({ "ok": false, "error": format!("Strict replay check failed: {e}") });
    }
    let mut cards_html = String::new();
    for (idx, resource) in candidate.outputs().iter().enumerate().filter(|_| req.include_html.unwrap_or(true)) {
        cards_html.push_str(&crate::present::render_resource_card_in_set(
            idx + 1,
            resource,
            candidate.outputs(),
        ));
        cards_html.push('\n');
    }
    let doc_digest = session
        .document_digest()
        .map(|d| {
            d.iter()
                .map(|b| format!("{b:02x}"))
                .collect::<String>()
        })
        .unwrap_or_default();
    let mut response = json!({
        "ok": true,
        "package_sha256": candidate.sha256(),
        "package_byte_length": candidate.bundle().len(),
        "strict_replay_verified": true,
        "document_digest": doc_digest,
        "resource_count": candidate.outputs().len(),
    });
    if req.include_html.unwrap_or(true) { response["cards_html"] = json!(cards_html); }
    if req.include_resources.unwrap_or(false) {
        response["resources"] = production_response::resources(candidate.outputs());
    }
    if req.include_package.unwrap_or(false) {
        response["package_base64"] = json!(crate::present::base64_encode(candidate.bundle()));
    }
    response
}

fn handle_mcp_call(body: &[u8]) -> Value {
    let Ok(req) = serde_json::from_slice::<McpCallRequest>(body) else {
        return json!({ "ok": false, "error": "Invalid JSON body for /api/mcp/call" });
    };
    let mut mcp = crate::mcp_full::StandaloneSynthMcp::from_process_env().unwrap_or_default();
    let args = if req.arguments.is_object() {
        req.arguments
    } else {
        json!({})
    };
    match mcp.call(&req.name, args) {
        Ok(res) => json!({ "ok": true, "tool": req.name, "result": res }),
        Err(err) => json!({ "ok": false, "tool": req.name, "error": err }),
    }
}

fn parse_query_params(raw_query: &str) -> std::collections::HashMap<String, String> {
    let mut out = std::collections::HashMap::new();
    for pair in raw_query.split('&') {
        if pair.is_empty() {
            continue;
        }
        let (k, v) = pair.split_once('=').unwrap_or((pair, ""));
        let k_dec = percent_decode(&k.replace('+', " "));
        let v_dec = percent_decode(&v.replace('+', " "));
        out.insert(k_dec, v_dec);
    }
    out
}

#[cfg(not(target_arch = "wasm32"))]
fn handle_sparql_http(
    method: &str,
    raw_path: &str,
    content_type: Option<&str>,
    body: &[u8],
) -> Value {
    let query_str = raw_path.split_once('?').map(|(_, q)| q).unwrap_or("");
    let mut params = parse_query_params(query_str);
    let ct = content_type.unwrap_or("").to_ascii_lowercase();

    let mut args = if method == "POST" && ct.contains("application/json") {
        serde_json::from_slice::<Value>(body).unwrap_or_else(|_| json!({}))
    } else {
        if method == "POST" {
            if ct.contains("application/x-www-form-urlencoded") {
                if let Ok(form) = std::str::from_utf8(body) {
                    params.extend(parse_query_params(form));
                }
            } else if let Ok(raw_q) = std::str::from_utf8(body) {
                if !raw_q.trim().is_empty() && !params.contains_key("query") {
                    params.insert("query".into(), raw_q.to_owned());
                }
            }
        }
        json!({})
    };

    if let Some(obj) = args.as_object_mut() {
        for (k, v) in &params {
            match k.as_str() {
                "query" => {
                    obj.entry("query").or_insert_with(|| Value::String(v.clone()));
                }
                "vault" | "vault_path" | "q42_path" | "default-graph-uri" => {
                    obj.entry("vault_path").or_insert_with(|| Value::String(v.clone()));
                }
                "endpoint" => {
                    obj.entry("endpoint").or_insert_with(|| Value::String(v.clone()));
                }
                "bookmark" | "bookmark_id" => {
                    obj.entry("bookmark_id").or_insert_with(|| Value::String(v.clone()));
                }
                "format" => {
                    obj.entry("format").or_insert_with(|| Value::String(v.clone()));
                }
                "limit" => {
                    if let Ok(lim) = v.parse::<u64>() {
                        obj.entry("limit").or_insert_with(|| json!(lim));
                    }
                }
                _ => {}
            }
        }
    }

    let mcp = crate::mcp_full::StandaloneSynthMcp::from_process_env().ok();
    match crate::mcp_full::q42::execute_sparql_tool(mcp.as_ref(), args) {
        Ok(res) => json!({ "ok": true, "result": res }),
        Err(err) => json!({ "ok": false, "error": err }),
    }
}

/// Processes a single HTTP request over `stream` (256 KiB; 16 MiB for wasm-lite compose/bridge).
pub fn handle_connection(mut stream: TcpStream) -> Result<(), String> {
    let _ = stream.set_read_timeout(Some(Duration::from_secs(3)));
    let _ = stream.set_write_timeout(Some(Duration::from_secs(5)));
    const SMALL_MAX: usize = 256 * 1024;
    let mut buf = vec![0u8; SMALL_MAX];
    let mut read_total = 0usize;
    let mut header_end = None;
    while read_total < buf.len() {
        match stream.read(&mut buf[read_total..]) {
            Ok(0) => break,
            Ok(n) => {
                read_total += n;
                if let Some(pos) = buf[..read_total]
                    .windows(4)
                    .position(|w| w == b"\r\n\r\n")
                {
                    header_end = Some(pos + 4);
                    break;
                }
            }
            Err(_) => break,
        }
    }
    let Some(body_offset) = header_end else {
        return Err("Incomplete HTTP header".into());
    };
    let header_str = std::str::from_utf8(&buf[..body_offset])
        .map_err(|e| e.to_string())?
        .to_owned();
    let first_line = header_str.lines().next().unwrap_or("");
    let mut parts = first_line.split_whitespace();
    let method = parts.next().unwrap_or("GET").to_owned();
    let raw_path = parts.next().unwrap_or("/").to_owned();
    let path = raw_path
        .split('?')
        .next()
        .unwrap_or(raw_path.as_str())
        .to_owned();

    let header_value = |name: &str| {
        header_str.lines().find_map(|line| {
            let (k, v) = line.split_once(':')?;
            if k.trim().eq_ignore_ascii_case(name) {
                Some(v.trim().to_string())
            } else {
                None
            }
        })
    };
    let content_len = header_value("content-length")
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(0);
    let range_header = header_value("range");

    let body_cap = match path.as_str() {
        "/api/radiance/compose" | "/api/hmc/bridge" if method == "POST" => {
            crate::wasm_lite::MAX_BODY_BYTES
        }
        _ => SMALL_MAX.saturating_sub(body_offset),
    };
    if content_len > body_cap {
        return write_http_response(
            stream,
            "200 OK",
            "application/json",
            br#"{"ok":false,"error":"Request body exceeds limit"}"#,
        );
    }
    let needed = body_offset.saturating_add(content_len);
    if needed > buf.len() {
        buf.resize(needed, 0);
    }
    while read_total < needed {
        match stream.read(&mut buf[read_total..needed]) {
            Ok(0) | Err(_) => break,
            Ok(n) => read_total += n,
        }
    }
    let body = &buf[body_offset..read_total.min(needed)];

    if method == "GET" && website::handles(&path) {
        return website::serve(stream, &path, range_header.as_deref());
    }
    #[cfg(not(target_arch = "wasm32"))]
    if method == "GET" && (path == "/operator" || path == "/operator/") {
        return write_html_coi(stream, crate::operator::page_html().as_bytes());
    }
    #[cfg(not(target_arch = "wasm32"))]
    if method == "GET" {
        if let Some(name) = path.strip_prefix("/operator/") {
            if let Some((mime, bytes)) = crate::operator::branding::asset(name) {
                return write_http_response(stream, "200 OK", mime, bytes);
            }
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    if method == "POST" && path == "/api/operator/shutdown" {
        std::thread::spawn(|| {
            std::thread::sleep(std::time::Duration::from_millis(150));
            std::process::exit(0);
        });
        return write_http_response(
            stream,
            "200 OK",
            "application/json",
            br#"{"ok":true,"stopping":true}"#,
        );
    }
    #[cfg(not(target_arch = "wasm32"))]
    if method == "GET" && path == "/api/gpu" {
        let payload = crate::session_admission::detect_runtime_and_gpu_profile(None);
        let bytes = serde_json::to_vec_pretty(&payload).map_err(|e| e.to_string())?;
        return write_http_response(stream, "200 OK", "application/json", &bytes);
    }
    #[cfg(not(target_arch = "wasm32"))]
    if method == "POST" && path == "/api/gpu" {
        let mut cfg = crate::session_admission::load_gpu_selection_config();
        let payload = match serde_json::from_slice::<Value>(body) {
            Ok(val) => {
                if let Some(p) = val.get("preference").and_then(Value::as_str) {
                    cfg.preference = p.to_owned();
                }
                if let Some(b) = val.get("backend").and_then(Value::as_str) {
                    cfg.backend = b.to_owned();
                }
                if let Some(r) = val.get("reject_igpu").and_then(Value::as_bool) {
                    cfg.reject_igpu = r;
                }
                if val.get("adapter_name_filter").is_some() {
                    cfg.adapter_name_filter = val
                        .get("adapter_name_filter")
                        .and_then(Value::as_str)
                        .map(str::trim)
                        .filter(|s| !s.is_empty())
                        .map(str::to_owned);
                }
                crate::session_admission::save_and_apply_gpu_selection_config(cfg)
                    .unwrap_or_else(|error| json!({ "ok": false, "error": error }))
            }
            Err(err) => json!({ "ok": false, "error": format!("Invalid GPU config JSON: {err}") }),
        };
        let bytes = serde_json::to_vec_pretty(&payload).map_err(|e| e.to_string())?;
        return write_http_response(stream, "200 OK", "application/json", &bytes);
    }
    #[cfg(not(target_arch = "wasm32"))]
    if method == "GET" && path == "/api/operator" {
        let fetch = raw_path.contains("fetch=1") || raw_path.contains("fetch=true");
        let payload = crate::operator::snapshot_with_fetch(fetch);
        let bytes = serde_json::to_vec_pretty(&payload).map_err(|e| e.to_string())?;
        return write_http_response(stream, "200 OK", "application/json", &bytes);
    }
    #[cfg(not(target_arch = "wasm32"))]
    if method == "GET" && path == "/api/update" {
        let fetch = raw_path.contains("fetch=1") || raw_path.contains("fetch=true");
        let payload = crate::operator::update_status(fetch);
        let bytes = serde_json::to_vec_pretty(&payload).map_err(|e| e.to_string())?;
        return write_http_response(stream, "200 OK", "application/json", &bytes);
    }
    #[cfg(not(target_arch = "wasm32"))]
    if method == "POST" && path == "/api/update" {
        let payload = match serde_json::from_slice::<crate::operator::update::UpdateArg>(body) {
            Ok(arg) => crate::operator::update::dispatch(arg)
                .unwrap_or_else(|error| json!({ "ok": false, "error": error })),
            Err(err) => json!({ "ok": false, "error": format!("Invalid update JSON: {err}") }),
        };
        let bytes = serde_json::to_vec(&payload).map_err(|e| e.to_string())?;
        return write_http_response(stream, "200 OK", "application/json", &bytes);
    }
    #[cfg(not(target_arch = "wasm32"))]
    if method == "GET" && path == "/api/bookmarks" {
        let payload = crate::operator::bookmarks::dispatch(
            crate::operator::bookmarks::BookmarkArg {
                action: Some("list".into()),
                ..crate::operator::bookmarks::BookmarkArg::default()
            },
        )
        .unwrap_or_else(|error| json!({ "ok": false, "error": error }));
        let bytes = serde_json::to_vec_pretty(&payload).map_err(|e| e.to_string())?;
        return write_http_response(stream, "200 OK", "application/json", &bytes);
    }
    #[cfg(not(target_arch = "wasm32"))]
    if method == "POST" && path == "/api/bookmarks" {
        let payload = match serde_json::from_slice::<crate::operator::bookmarks::BookmarkArg>(body) {
            Ok(arg) => crate::operator::bookmarks::dispatch(arg)
                .unwrap_or_else(|error| json!({ "ok": false, "error": error })),
            Err(err) => json!({ "ok": false, "error": format!("Invalid bookmark JSON: {err}") }),
        };
        let bytes = serde_json::to_vec(&payload).map_err(|e| e.to_string())?;
        return write_http_response(stream, "200 OK", "application/json", &bytes);
    }
    #[cfg(not(target_arch = "wasm32"))]
    if method == "GET" && path == "/api/jobs" {
        let payload = crate::operator::jobs::dispatch(
            crate::operator::jobs::JobArg {
                action: Some("list".into()),
                ..crate::operator::jobs::JobArg::default()
            },
        )
        .unwrap_or_else(|error| json!({ "ok": false, "error": error }));
        let bytes = serde_json::to_vec_pretty(&payload).map_err(|e| e.to_string())?;
        return write_http_response(stream, "200 OK", "application/json", &bytes);
    }
    #[cfg(not(target_arch = "wasm32"))]
    if (method == "GET" || method == "POST") && (path == "/sparql" || path == "/api/sparql") {
        let ct = header_value("content-type");
        let payload = handle_sparql_http(&method, &raw_path, ct.as_deref(), body);
        let bytes = serde_json::to_vec_pretty(&payload).map_err(|e| e.to_string())?;
        return write_http_response(
            stream,
            "200 OK",
            "application/sparql-results+json; charset=utf-8",
            &bytes,
        );
    }
    #[cfg(not(target_arch = "wasm32"))]
    if method == "POST" && path == "/api/jobs" {
        let payload = match serde_json::from_slice::<crate::operator::jobs::JobArg>(body) {
            Ok(arg) => crate::operator::jobs::dispatch(arg)
                .unwrap_or_else(|error| json!({ "ok": false, "error": error })),
            Err(err) => json!({ "ok": false, "error": format!("Invalid job JSON: {err}") }),
        };
        let bytes = serde_json::to_vec(&payload).map_err(|e| e.to_string())?;
        return write_http_response(stream, "200 OK", "application/json", &bytes);
    }
    match (method.as_str(), path.as_str()) {
        ("GET", "/" | "/index.html") => {
            let html = render_root_studio_html()?;
            write_html_coi(stream, html.as_bytes())
        }
        ("GET", "/api/status") => {
            let report = environment_report_json()?;
            let bytes = serde_json::to_vec_pretty(&report).map_err(|e| e.to_string())?;
            write_http_response(stream, "200 OK", "application/json", &bytes)
        }
        ("GET", "/api/capabilities") => {
            let snap = FullCapabilitiesSnapshot::collect();
            let payload = json!({
                "mode": snap.mode_label,
                "vault_root": snap.vault_root,
                "discovered_models": snap.discovered_models,
                "categories": snap.categories,
                "morphisms": snap.morphisms,
                "mcp_tool_count": snap.mcp_tools.len(),
                "vibe_sdk": snap.vibe_sdk,
                "living_world": snap.living_world,
                "cosmic_earth": snap.cosmic_earth,
            });
            let bytes = serde_json::to_vec_pretty(&payload).map_err(|e| e.to_string())?;
            write_http_response(stream, "200 OK", "application/json", &bytes)
        }
        ("POST", "/api/produce") => {
            let res = handle_produce(body);
            let bytes = serde_json::to_vec(&res).map_err(|e| e.to_string())?;
            write_http_response(stream, "200 OK", "application/json", &bytes)
        }
        ("POST", "/api/mcp/call") => {
            let res = handle_mcp_call(body);
            let bytes = serde_json::to_vec(&res).map_err(|e| e.to_string())?;
            write_http_response(stream, "200 OK", "application/json", &bytes)
        }
        ("GET", "/synth/wasm-lite.js") => {
            let mime = "text/javascript; charset=utf-8";
            let extra = extra_static_headers(mime);
            write_http_response_with(
                stream,
                "200 OK",
                mime,
                crate::wasm_lite::WASM_LITE_JS.as_bytes(),
                &extra,
            )
        }
        ("POST", "/api/radiance/compose") => match crate::wasm_lite::compose_png(body) {
            Ok(png) => write_http_response(stream, "200 OK", "image/png", &png),
            Err(err) => {
                let bytes = serde_json::to_vec(&json!({ "ok": false, "error": err }))
                    .map_err(|e| e.to_string())?;
                write_http_response(stream, "200 OK", "application/json", &bytes)
            }
        },
        ("POST", "/api/hmc/bridge") => {
            let res = crate::wasm_lite::bridge_hmc(body);
            let bytes = serde_json::to_vec(&res).map_err(|e| e.to_string())?;
            write_http_response(stream, "200 OK", "application/json", &bytes)
        }
        ("GET", "/project" | "/project/") => {
            let html = render_project_html()?;
            write_html_coi(stream, html.as_bytes())
        }
        ("GET", "/api/project") => {
            let payload = match project::current() {
                Some(p) => p.status_json()?,
                None => json!({ "ok": false, "error": "No project attached. Use synth serve --project DIR" }),
            };
            let bytes = serde_json::to_vec_pretty(&payload).map_err(|e| e.to_string())?;
            write_http_response(stream, "200 OK", "application/json", &bytes)
        }
        ("GET", "/api/project/annotations") => {
            let payload = match project::current() {
                Some(p) => json!({ "ok": true, "items": p.load_annotations()? }),
                None => json!({ "ok": false, "error": "No project attached" }),
            };
            let bytes = serde_json::to_vec_pretty(&payload).map_err(|e| e.to_string())?;
            write_http_response(stream, "200 OK", "application/json", &bytes)
        }
        ("POST", "/api/project/annotations") => {
            let res = handle_annotation(body);
            let bytes = serde_json::to_vec(&res).map_err(|e| e.to_string())?;
            write_http_response(stream, "200 OK", "application/json", &bytes)
        }
        _ if method == "GET" && path.starts_with("/project/file/") => {
            serve_project_file(stream, &path, range_header.as_deref())
        }
        ("GET", "/qualia" | "/qualia/" | "/vendor/qualia" | "/vendor/qualia/") => {
            serve_qualia_index(stream)
        }
        _ if method == "GET"
            && (path.starts_with("/qualia/") || path.starts_with("/vendor/qualia/")) =>
        {
            serve_qualia_static(stream, &path)
        }
        _ => write_http_response(
            stream,
            "404 Not Found",
            "application/json",
            br#"{"ok":false,"error":"Not found"}"#,
        ),
    }
}

fn handle_annotation(body: &[u8]) -> Value {
    let Ok(req) = serde_json::from_slice::<AnnotationRequest>(body) else {
        return json!({ "ok": false, "error": "Invalid JSON body for annotation" });
    };
    let Some(project) = project::current() else {
        return json!({ "ok": false, "error": "No project attached" });
    };
    let item = Annotation {
        id: String::new(),
        created_at: String::new(),
        lane: req.lane.unwrap_or_else(|| "working".into()),
        file: req.file,
        aspect: req.aspect.unwrap_or_default(),
        tags: req.tags.unwrap_or_default(),
        rating: req.rating.unwrap_or(0),
        comment: req.comment.unwrap_or_default(),
        links: req.links.unwrap_or_default(),
        camera_yaw: req.camera_yaw,
        camera_pitch: req.camera_pitch,
        camera_distance: req.camera_distance,
        time_seconds: req.time_seconds,
        frame_index: req.frame_index,
        screen_pin_uv: req.screen_pin_uv,
        addressed: false,
    };
    match project.add_annotation(item) {
        Ok(saved) => json!({ "ok": true, "item": saved }),
        Err(err) => json!({ "ok": false, "error": err }),
    }
}

#[derive(serde::Deserialize)]
struct AnnotationRequest {
    lane: Option<String>,
    file: String,
    aspect: Option<String>,
    tags: Option<Vec<String>>,
    rating: Option<u8>,
    comment: Option<String>,
    links: Option<Vec<AnnotationLink>>,
    camera_yaw: Option<f32>,
    camera_pitch: Option<f32>,
    camera_distance: Option<f32>,
    time_seconds: Option<f32>,
    frame_index: Option<u32>,
    screen_pin_uv: Option<[f32; 2]>,
}

fn serve_project_file(
    stream: TcpStream,
    path: &str,
    range_header: Option<&str>,
) -> Result<(), String> {
    let Some(project) = project::current() else {
        return write_http_response(
            stream,
            "404 Not Found",
            "application/json",
            br#"{"ok":false,"error":"No project attached"}"#,
        );
    };
    let rest = path.trim_start_matches("/project/file/");
    let Some((lane, rel_enc)) = rest.split_once('/') else {
        return write_http_response(
            stream,
            "400 Bad Request",
            "application/json",
            br#"{"ok":false,"error":"Path must be /project/file/{working|delivered}/..."}"#,
        );
    };
    let rel = percent_decode(rel_enc);
    match project.resolve_file(lane, &rel) {
        Ok(file) => {
            let bytes = std::fs::read(&file).map_err(|e| e.to_string())?;
            let kind = project::file_kind(&rel);
            write_ranged_bytes(stream, project::mime_for(kind), &bytes, range_header)
        }
        Err(err) => {
            let body = serde_json::to_vec(&json!({ "ok": false, "error": err }))
                .map_err(|e| e.to_string())?;
            write_http_response(stream, "404 Not Found", "application/json", &body)
        }
    }
}

fn percent_decode(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Ok(hex) = std::str::from_utf8(&bytes[i + 1..i + 3]) {
                if let Ok(value) = u8::from_str_radix(hex, 16) {
                    out.push(value);
                    i += 3;
                    continue;
                }
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::production::{ProducerRegistry, ProductionLimits};
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::sync::OnceLock;
    use std::thread;

    fn exchange(request: &[u8], body: Vec<u8>, mime: &'static str) -> Vec<u8> {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let want = request.len();
        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut buf = vec![0u8; want.max(1)];
            let mut read = 0usize;
            while read < want {
                match stream.read(&mut buf[read..]) {
                    Ok(0) | Err(_) => break,
                    Ok(n) => read += n,
                }
            }
            let header = std::str::from_utf8(&buf[..read]).unwrap_or("");
            let range = header.lines().find_map(|line| {
                let (k, v) = line.split_once(':')?;
                if k.trim().eq_ignore_ascii_case("range") {
                    Some(v.trim().to_string())
                } else {
                    None
                }
            });
            write_ranged_bytes(stream, mime, &body, range.as_deref()).unwrap();
        });
        let mut client = TcpStream::connect(addr).unwrap();
        client.write_all(request).unwrap();
        let mut resp = Vec::new();
        client.read_to_end(&mut resp).unwrap();
        server.join().unwrap();
        resp
    }

    fn http_split(resp: &[u8]) -> (&str, &[u8]) {
        let pos = resp.windows(4).position(|w| w == b"\r\n\r\n").expect("headers");
        let header = std::str::from_utf8(&resp[..pos]).unwrap();
        (header, &resp[pos + 4..])
    }

    #[test]
    fn ranged_executable_assets_preserve_cross_origin_isolation() {
        for mime in ["text/html; charset=utf-8", "application/javascript", "application/wasm"] {
            for request in [b"GET /asset HTTP/1.1\r\n\r\n".as_slice(), b"GET /asset HTTP/1.1\r\nRange: bytes=0-3\r\n\r\n".as_slice()] {
                let response = exchange(request, b"abcdefgh".to_vec(), mime);
                let (headers, _) = http_split(&response);
                assert!(headers.contains("Cross-Origin-Opener-Policy: same-origin"));
                assert!(headers.contains("Cross-Origin-Embedder-Policy: require-corp"));
                assert!(headers.contains("Cross-Origin-Resource-Policy: same-origin"));
            }
        }
    }

    fn produce_hmc() -> (Vec<u8>, Vec<u8>) {
        let registry = ProducerRegistry::with_builtins();
        let cancel = AtomicBool::new(false);
        let mut session = registry
            .session("files", vec![], ProductionLimits::default())
            .unwrap();
        session
            .submit(
                r#"let mut asset = { files: [{ key: "slice.txt", kind: "txt", text: "ABCDEFGHIJKLMNOPQRSTUVWXYZ" }] };"#,
                &cancel,
            )
            .unwrap();
        let hmc = session.candidate().unwrap().bundle().to_vec();
        let reader = qualia_core_db::bundle::BundleReader::parse(&hmc).expect("HMC parses");
        let member = reader
            .get("outputs/slice.txt")
            .expect("HMC bag member")
            .to_vec();
        (hmc, member)
    }

    #[test]
    fn range_request_on_produced_hmc_and_bag_member_returns_206_slice() {
        let (hmc, member) = produce_hmc();
        assert!(hmc.len() > 16, "produced HMC must be large enough to slice");
        assert_eq!(member, b"ABCDEFGHIJKLMNOPQRSTUVWXYZ");

        let resp = exchange(
            b"GET /project/file/working/pack.hmc HTTP/1.1\r\nHost: 127.0.0.1\r\nRange: bytes=0-15\r\nConnection: close\r\n\r\n",
            hmc.clone(),
            "application/octet-stream",
        );
        let (header, body) = http_split(&resp);
        assert!(
            header.starts_with("HTTP/1.1 206 Partial Content"),
            "produced HMC Range must be 206, got {header}"
        );
        assert!(header.to_ascii_lowercase().contains("accept-ranges: bytes"));
        assert!(header.contains("Content-Range: bytes 0-15/"));
        assert_eq!(body, &hmc[..16]);

        let resp = exchange(
            b"GET /project/file/working/slice.txt HTTP/1.1\r\nHost: 127.0.0.1\r\nRange: bytes=4-9\r\nConnection: close\r\n\r\n",
            member.clone(),
            "application/octet-stream",
        );
        let (header, body) = http_split(&resp);
        assert!(header.starts_with("HTTP/1.1 206 Partial Content"));
        assert!(header.contains("Content-Range: bytes 4-9/26"));
        assert_eq!(body, b"EFGHIJ");
    }

    #[test]
    fn invalid_range_on_produced_hmc_is_not_200() {
        let (hmc, _) = produce_hmc();
        let total = hmc.len();
        let resp = exchange(
            format!(
                "GET /project/file/working/pack.hmc HTTP/1.1\r\nHost: 127.0.0.1\r\nRange: bytes={}-{}\r\nConnection: close\r\n\r\n",
                total + 8,
                total + 32
            )
            .as_bytes(),
            hmc,
            "application/octet-stream",
        );
        let (header, body) = http_split(&resp);
        assert!(
            !header.starts_with("HTTP/1.1 200"),
            "unsatisfiable Range must not 200, got {header}"
        );
        assert!(header.starts_with("HTTP/1.1 416 Range Not Satisfiable"));
        assert!(header.contains(&format!("Content-Range: bytes */{total}")));
        assert!(header.to_ascii_lowercase().contains("accept-ranges: bytes"));
        assert!(body.is_empty());
    }

    #[test]
    fn malformed_and_suffix_ranges_fail_closed_or_slice() {
        let body = b"0123456789".to_vec();
        let resp = exchange(
            b"GET /project/file/working/file.bin HTTP/1.1\r\nRange: bytes=7-3\r\nConnection: close\r\n\r\n",
            body.clone(),
            "application/octet-stream",
        );
        assert!(http_split(&resp).0.starts_with("HTTP/1.1 416"));

        let resp = exchange(
            b"GET /project/file/working/file.bin HTTP/1.1\r\nRange: bytes=0-1,4-5\r\nConnection: close\r\n\r\n",
            body.clone(),
            "application/octet-stream",
        );
        assert!(http_split(&resp).0.starts_with("HTTP/1.1 416"));

        let resp = exchange(
            b"GET /project/file/working/file.bin HTTP/1.1\r\nRange: bytes=-4\r\nConnection: close\r\n\r\n",
            body,
            "application/octet-stream",
        );
        let (header, slice) = http_split(&resp);
        assert!(header.starts_with("HTTP/1.1 206 Partial Content"));
        assert_eq!(slice, b"6789");

        let resp = exchange(
            b"GET /project/file/working/file.bin HTTP/1.1\r\nRange:    \r\nConnection: close\r\n\r\n",
            b"0123456789".to_vec(),
            "application/octet-stream",
        );
        let (header, slice) = http_split(&resp);
        assert!(
            header.starts_with("HTTP/1.1 416 Range Not Satisfiable"),
            "whitespace-only Range must be 416, got {header}"
        );
        assert!(header.contains("Content-Range: bytes */10"));
        assert!(slice.is_empty());
    }

    fn range_http_working_dir() -> PathBuf {
        static INIT: OnceLock<PathBuf> = OnceLock::new();
        INIT.get_or_init(|| {
            if crate::project::current().is_none() {
                let path = std::env::temp_dir().join(format!(
                    "synth-range-http-{}",
                    std::process::id()
                ));
                std::fs::create_dir_all(&path).unwrap();
                crate::project::attach(Some(path)).unwrap();
            }
            crate::project::current()
                .expect("project attached for Range HTTP tests")
                .working
                .clone()
        })
        .clone()
    }

    fn unique_range_file(ext: &str) -> String {
        static N: AtomicU64 = AtomicU64::new(0);
        format!(
            "range-{}-{}.{ext}",
            std::process::id(),
            N.fetch_add(1, Ordering::Relaxed)
        )
    }

    fn exchange_handle_connection(file_name: &str, bytes: &[u8], extra_headers: &str) -> Vec<u8> {
        let working = range_http_working_dir();
        std::fs::write(working.join(file_name), bytes).unwrap();
        let request = format!(
            "GET /project/file/working/{file_name} HTTP/1.1\r\nHost: 127.0.0.1\r\n{extra_headers}Connection: close\r\n\r\n"
        );
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let server = thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            handle_connection(stream).unwrap();
        });
        let mut client = TcpStream::connect(addr).unwrap();
        client.write_all(request.as_bytes()).unwrap();
        let mut resp = Vec::new();
        client.read_to_end(&mut resp).unwrap();
        server.join().unwrap();
        resp
    }

    #[test]
    fn handle_connection_range_on_project_file_is_206_missing_is_200_blank_is_416() {
        let (hmc, _) = produce_hmc();
        let total = hmc.len();
        assert!(total > 16, "produced HMC must be large enough to slice");

        let ranged = exchange_handle_connection(
            &unique_range_file("hmc"),
            &hmc,
            "Range: bytes=0-15\r\n",
        );
        let (header, body) = http_split(&ranged);
        assert!(
            header.starts_with("HTTP/1.1 206 Partial Content"),
            "handle_connection Range must be 206, got {header}"
        );
        assert!(header.to_ascii_lowercase().contains("accept-ranges: bytes"));
        assert!(header.contains(&format!("Content-Range: bytes 0-15/{total}")));
        assert_eq!(body, &hmc[..16]);

        let full = exchange_handle_connection(&unique_range_file("hmc"), &hmc, "");
        let (header, body) = http_split(&full);
        assert!(
            header.starts_with("HTTP/1.1 200 OK"),
            "missing Range through handle_connection must be 200, got {header}"
        );
        assert!(header.to_ascii_lowercase().contains("accept-ranges: bytes"));
        assert_eq!(body, hmc.as_slice());

        let blank = exchange_handle_connection(&unique_range_file("hmc"), &hmc, "Range:    \r\n");
        let (header, body) = http_split(&blank);
        assert!(
            header.starts_with("HTTP/1.1 416 Range Not Satisfiable"),
            "whitespace-only Range through handle_connection must be 416, got {header}"
        );
        assert!(header.contains(&format!("Content-Range: bytes */{total}")));
        assert!(body.is_empty());
    }

    fn exchange_raw(request: &[u8]) -> Vec<u8> {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let server = thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            handle_connection(stream).unwrap();
        });
        let mut client = TcpStream::connect(addr).unwrap();
        client.write_all(request).unwrap();
        let mut resp = Vec::new();
        client.read_to_end(&mut resp).unwrap();
        server.join().unwrap();
        resp
    }

    #[test]
    fn confine_qualia_rel_rejects_parent_and_encoded_dotdot() {
        assert!(confine_qualia_rel("js/qualia-shell.js").is_some());
        assert!(confine_qualia_rel("../Cargo.toml").is_none());
        assert!(confine_qualia_rel("js/../../src/lib.rs").is_none());
        assert!(confine_qualia_rel("%2e%2e/Cargo.toml").is_none());
        assert!(confine_qualia_rel("js/%2e%2e/%2e%2e/src/web_server.rs").is_none());
        assert!(confine_qualia_rel("js\\qualia-shell.js").is_none());
    }

    #[test]
    fn get_qualia_shell_js_is_200_js_and_path_escape_is_404() {
        assert!(
            qualia_docs_root().is_some(),
            "QualiaDB docs must exist at C:\\github\\qualiaDB\\docs (or SYNTH_QUALIA_DOCS)"
        );
        let resp = exchange_raw(
            b"GET /qualia/js/qualia-shell.js HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n",
        );
        let (header, body) = http_split(&resp);
        assert!(
            header.starts_with("HTTP/1.1 200 OK"),
            "GET /qualia/js/qualia-shell.js must be 200, got {header}"
        );
        let header_l = header.to_ascii_lowercase();
        assert!(
            header_l.contains("content-type: text/javascript"),
            "qualia-shell.js MIME must be javascript, got {header}"
        );
        let text = std::str::from_utf8(body).unwrap_or("");
        assert!(
            text.contains("loadQualiaPortal"),
            "qualia-shell.js body must export loadQualiaPortal"
        );
        assert!(header_l.contains("cross-origin-resource-policy: same-origin"));
        assert!(
            header_l.contains("cross-origin-opener-policy: same-origin"),
            "JS must send COOP for SharedArrayBuffer, got {header}"
        );
        assert!(
            header_l.contains("cross-origin-embedder-policy: require-corp"),
            "JS must send COEP for SharedArrayBuffer, got {header}"
        );

        for escape in [
            "GET /qualia/../Cargo.toml HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n",
            "GET /qualia/js/../../Cargo.toml HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n",
            "GET /qualia/%2e%2e/Cargo.toml HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n",
            "GET /qualia/js/%2e%2e/%2e%2e/src/lib.rs HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n",
        ] {
            let resp = exchange_raw(escape.as_bytes());
            let (header, _) = http_split(&resp);
            assert!(
                header.starts_with("HTTP/1.1 404"),
                "path escape must be 404, got {header} for {escape}"
            );
        }
    }

    #[test]
    fn root_and_project_stay_200() {
        let root = exchange_raw(b"GET / HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n");
        let (header, body) = http_split(&root);
        assert!(
            header.starts_with("HTTP/1.1 200 OK"),
            "/ must stay 200, got {header}"
        );
        let header_l = header.to_ascii_lowercase();
        assert!(header_l.contains("content-type: text/html"));
        assert!(header_l.contains("cross-origin-opener-policy: same-origin"));
        assert!(header_l.contains("cross-origin-embedder-policy: require-corp"));
        let html = std::str::from_utf8(body).unwrap_or("");
        assert!(html.contains("/qualia/js/qualia-shell.js"));

        let project =
            exchange_raw(b"GET /project HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n");
        let (header, body) = http_split(&project);
        assert!(
            header.starts_with("HTTP/1.1 200 OK"),
            "/project must stay 200, got {header}"
        );
        let header_l = header.to_ascii_lowercase();
        assert!(header_l.contains("content-type: text/html"));
        assert!(header_l.contains("cross-origin-opener-policy: same-origin"));
        assert!(header_l.contains("cross-origin-embedder-policy: require-corp"));
        let html = std::str::from_utf8(body).unwrap_or("");
        assert!(html.contains("/qualia/js/qualia-shell.js"));
        assert!(html.contains("webgpu-snapshot"));
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn operator_page_and_api_are_200() {
        let page =
            exchange_raw(b"GET /operator HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n");
        let (header, body) = http_split(&page);
        assert!(
            header.starts_with("HTTP/1.1 200 OK"),
            "/operator must stay 200, got {header}"
        );
        let html = std::str::from_utf8(body).unwrap_or("");
        assert!(html.contains("Webizen AI Synth"));
        assert!(html.contains("/operator/emblem.svg"));
        assert!(html.contains("/operator/icon.png"));
        assert!(html.contains("/api/operator/shutdown"));
        assert!(html.contains("system tray"));
        assert!(html.contains("/api/operator"));
        assert!(html.contains("/api/bookmarks"));
        assert!(html.contains("Semantic bookmarks"));
        assert!(html.contains("/api/jobs"));
        assert!(html.contains("Jobs"));
        assert!(html.contains("GitHub Releases"));
        assert!(html.contains("synth update verify"));
        assert!(html.contains("id=\"upgrade-btn\""));
        assert!(html.contains("Upgrade"));
        assert!(html.contains("/api/update"));
        assert!(html.contains("action:'upgrade'"));

        let api = exchange_raw(
            b"GET /api/operator HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n",
        );
        let (header, body) = http_split(&api);
        assert!(
            header.starts_with("HTTP/1.1 200 OK"),
            "/api/operator must stay 200, got {header}"
        );
        let json: serde_json::Value = serde_json::from_slice(body).expect("operator JSON");
        assert_eq!(json["schema"], "synth.operator-status/v1");
        assert_eq!(json["encyclopedia"], "ecopunk");
        assert_eq!(json["update"]["auto_replace"], false);
        assert_eq!(
            json["update"]["signing"]["public_key_hex"],
            crate::operator::update::RELEASE_PUBLIC_KEY_HEX.trim()
        );
        assert!(json["bookmarks"].is_object());
        assert!(json["jobs"].is_object());
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn bookmarks_http_upserts_sparql_url_without_storing_query_secrets() {
        let _lock = crate::operator::TEST_ENV_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let root = std::env::temp_dir().join(format!(
            "synth-http-bookmarks-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&root).unwrap();
        std::env::set_var("SYNTH_BOOKMARK_ROOT", root.to_str().unwrap());
        let body = br#"{"action":"upsert","url":"https://query.wikidata.org/sparql?token=nope"}"#;
        let req = format!(
            "POST /api/bookmarks HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body.len(),
            std::str::from_utf8(body).unwrap()
        );
        let resp = exchange_raw(req.as_bytes());
        let (header, resp_body) = http_split(&resp);
        assert!(
            header.starts_with("HTTP/1.1 200 OK"),
            "/api/bookmarks POST must stay 200, got {header}"
        );
        let json: serde_json::Value = serde_json::from_slice(resp_body).expect("bookmark JSON");
        assert_eq!(json["ok"], true);
        assert_eq!(json["item"]["kind"], "sparql");
        assert_eq!(json["item"]["url"], "https://query.wikidata.org/sparql");
        assert_eq!(json["item"]["requires_api_key"], true);
        let listed = exchange_raw(
            b"GET /api/bookmarks HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n",
        );
        let (_, listed_body) = http_split(&listed);
        let listed_json: serde_json::Value = serde_json::from_slice(listed_body).unwrap();
        let text = listed_json.to_string();
        assert!(!text.contains("nope"));
        assert_eq!(listed_json["count"], 1);
        std::env::remove_var("SYNTH_BOOKMARK_ROOT");
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn mime_for_qualia_webp_is_image_webp_not_jpeg() {
        assert_eq!(mime_for_qualia("shot.webp"), "image/webp");
        assert_eq!(mime_for_qualia("shot.WEBP"), "image/webp");
        assert_eq!(mime_for_qualia("shot.jpg"), "image/jpeg");
        assert_eq!(mime_for_qualia("shot.jpeg"), "image/jpeg");
        assert_eq!(mime_for_qualia("shell.js"), "text/javascript; charset=utf-8");
        assert_eq!(mime_for_qualia("portal.wasm"), "application/wasm");
        assert_eq!(mime_for_qualia("rdf-holograph.vert.glsl"), "text/plain; charset=utf-8");
        assert_eq!(mime_for_qualia("graph.ttl"), "text/turtle; charset=utf-8");
        assert_eq!(mime_for_qualia("graph.nt"), "application/n-triples");
    }

    #[test]
    fn extra_static_headers_coep_on_js_wasm_and_html() {
        for mime in [
            "text/html; charset=utf-8",
            "text/javascript; charset=utf-8",
            "application/javascript",
            "application/wasm",
        ] {
            let h = extra_static_headers(mime);
            let joined: Vec<_> = h.iter().map(|(k, v)| format!("{k}: {v}")).collect();
            let blob = joined.join("\n").to_ascii_lowercase();
            assert!(blob.contains("cross-origin-resource-policy: same-origin"), "{mime}");
            assert!(blob.contains("cross-origin-opener-policy: same-origin"), "{mime}");
            assert!(blob.contains("cross-origin-embedder-policy: require-corp"), "{mime}");
        }
        let png = extra_static_headers("image/png");
        let blob = png
            .iter()
            .map(|(k, v)| format!("{k}: {v}"))
            .collect::<Vec<_>>()
            .join("\n")
            .to_ascii_lowercase();
        assert!(blob.contains("cross-origin-resource-policy: same-origin"));
        assert!(!blob.contains("cross-origin-embedder-policy"));
    }

    #[test]
    fn get_qualia_index_is_200_html_with_coep() {
        assert!(
            qualia_docs_root().is_some(),
            "QualiaDB docs must exist at C:\\github\\qualiaDB\\docs (or SYNTH_QUALIA_DOCS)"
        );
        let resp =
            exchange_raw(b"GET /qualia/ HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n");
        let (header, body) = http_split(&resp);
        assert!(
            header.starts_with("HTTP/1.1 200 OK"),
            "GET /qualia/ must be 200, got {header}"
        );
        let header_l = header.to_ascii_lowercase();
        assert!(header_l.contains("content-type: text/html"));
        assert!(header_l.contains("cross-origin-embedder-policy: require-corp"));
        assert!(header_l.contains("cross-origin-opener-policy: same-origin"));
        let html = std::str::from_utf8(body).unwrap_or("");
        assert!(
            html.contains("pkg/qualia") || html.contains("qualia.js"),
            "index must mention pkg/qualia or qualia.js"
        );
        assert!(html.contains("QualiaPortal docs"));
        assert!(html.contains("load_10d"));
        assert!(html.contains("load_emf"));

        let bare =
            exchange_raw(b"GET /qualia HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n");
        let (header, _) = http_split(&bare);
        assert!(
            header.starts_with("HTTP/1.1 200 OK"),
            "GET /qualia must be 200, got {header}"
        );
    }

    #[test]
    fn get_wasm_lite_js_is_200_javascript_with_coep() {
        let resp = exchange_raw(
            b"GET /synth/wasm-lite.js HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n",
        );
        let (header, body) = http_split(&resp);
        assert!(
            header.starts_with("HTTP/1.1 200 OK"),
            "GET /synth/wasm-lite.js must be 200, got {header}"
        );
        let header_l = header.to_ascii_lowercase();
        assert!(
            header_l.contains("content-type: text/javascript"),
            "wasm-lite.js MIME must be javascript, got {header}"
        );
        assert!(header_l.contains("cross-origin-embedder-policy: require-corp"));
        assert!(header_l.contains("cross-origin-opener-policy: same-origin"));
        let text = std::str::from_utf8(body).unwrap_or("");
        assert!(text.contains("composeToOpfs"));
        assert!(text.contains("getDirectory"));
        assert!(text.contains("ingestHmc"));
    }

    fn exchange_post(path: &str, body: &[u8]) -> Vec<u8> {
        let req = format!(
            "POST {path} HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            body.len()
        );
        let mut bytes = req.into_bytes();
        bytes.extend_from_slice(body);
        exchange_raw(&bytes)
    }

    #[test]
    fn post_radiance_compose_invalid_json_is_ok_false() {
        let resp = exchange_post("/api/radiance/compose", b"not-json");
        let (header, body) = http_split(&resp);
        assert!(header.starts_with("HTTP/1.1 200 OK"), "got {header}");
        let v: serde_json::Value = serde_json::from_slice(body).expect("json");
        assert_eq!(v["ok"], false);
        assert!(v.get("error").is_some());
    }

    #[test]
    fn post_hmc_bridge_empty_files_is_error() {
        let resp = exchange_post("/api/hmc/bridge", br#"{"files":[]}"#);
        let (header, body) = http_split(&resp);
        assert!(header.starts_with("HTTP/1.1 200 OK"), "got {header}");
        let v: serde_json::Value = serde_json::from_slice(body).expect("json");
        assert_eq!(v["ok"], false);
        let err = v["error"].as_str().unwrap_or("");
        assert!(!err.is_empty(), "empty files must return an error");
    }
}
