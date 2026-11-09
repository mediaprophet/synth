//! Local HTTP Studio & Capabilities Workbench Server (`http://127.0.0.1:8787`).
//!
//! Explicitly avoids QualiaDB / Webizen Desktop reserved ports (`8080`, `4242`, `9001`).
//! Serves the interactive 5-tab Synth Studio UI (`GET /`) alongside live JSON endpoints:
//! - `GET /api/status`
//! - `GET /api/capabilities`
//! - `POST /api/produce`
//! - `POST /api/mcp/call`

use crate::{
    present::{render_resource_card, render_session_html},
    present_capabilities::FullCapabilitiesSnapshot,
    production::{ProducerRegistry, ProductionLimits},
    tray_env::{environment_report_json, TRAY_ENVIRONMENT_SOURCE},
};
use qualia_mcp::authoring::AuthoringBackend;
use serde::Deserialize;
use serde_json::{json, Value};
use std::{
    io::{Read, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    process::Command,
    sync::atomic::AtomicBool,
    thread,
    time::Duration,
};

pub const DEFAULT_STUDIO_PORT: u16 = 8787;
pub const WEBIZEN_RESERVED_PORTS: &[u16] = &[8080, 4242, 9001];

#[derive(Deserialize)]
struct ProduceRequest {
    source: String,
    profile: Option<String>,
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
                let _ = handle_connection(stream);
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
    if open_browser {
        open_in_system_browser(&url);
    }
    for stream in listener.incoming().flatten() {
        let _ = handle_connection(stream);
    }
    Ok(())
}

fn write_http_response(
    mut stream: TcpStream,
    status: &str,
    content_type: &str,
    body: &[u8],
) -> Result<(), String> {
    let header = format!(
        "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: http://127.0.0.1\r\nConnection: close\r\n\r\n",
        body.len()
    );
    stream
        .write_all(header.as_bytes())
        .and_then(|_| stream.write_all(body))
        .and_then(|_| stream.flush())
        .map_err(|e| e.to_string())
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
    for (idx, resource) in candidate.outputs().iter().enumerate() {
        cards_html.push_str(&render_resource_card(idx + 1, resource));
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
    json!({
        "ok": true,
        "package_sha256": candidate.sha256(),
        "document_digest": doc_digest,
        "resource_count": candidate.outputs().len(),
        "cards_html": cards_html,
    })
}

fn handle_mcp_call(body: &[u8]) -> Value {
    let Ok(req) = serde_json::from_slice::<McpCallRequest>(body) else {
        return json!({ "ok": false, "error": "Invalid JSON body for /api/mcp/call" });
    };
    let mut mcp = crate::mcp_full::StandaloneSynthMcp::default();
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

/// Processes a single HTTP request over `stream` (bounded to 256 KiB).
pub fn handle_connection(mut stream: TcpStream) -> Result<(), String> {
    let _ = stream.set_read_timeout(Some(Duration::from_secs(3)));
    let _ = stream.set_write_timeout(Some(Duration::from_secs(5)));
    let mut buf = vec![0u8; 256 * 1024];
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
    let header_str = std::str::from_utf8(&buf[..body_offset]).map_err(|e| e.to_string())?;
    let first_line = header_str.lines().next().unwrap_or("");
    let mut parts = first_line.split_whitespace();
    let method = parts.next().unwrap_or("GET").to_owned();
    let path = parts.next().unwrap_or("/").to_owned();

    let content_len = header_str
        .lines()
        .find_map(|line| {
            let (k, v) = line.split_once(':')?;
            if k.trim().eq_ignore_ascii_case("content-length") {
                v.trim().parse::<usize>().ok()
            } else {
                None
            }
        })
        .unwrap_or(0);

    while read_total < body_offset.saturating_add(content_len) && read_total < buf.len() {
        match stream.read(&mut buf[read_total..]) {
            Ok(0) | Err(_) => break,
            Ok(n) => read_total += n,
        }
    }
    let body = &buf[body_offset..read_total.min(body_offset.saturating_add(content_len))];

    match (method.as_str(), path.as_str()) {
        ("GET", "/" | "/index.html") => {
            let html = render_root_studio_html()?;
            write_http_response(stream, "200 OK", "text/html; charset=utf-8", html.as_bytes())
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
        _ => write_http_response(
            stream,
            "404 Not Found",
            "application/json",
            br#"{"ok":false,"error":"Not found"}"#,
        ),
    }
}
