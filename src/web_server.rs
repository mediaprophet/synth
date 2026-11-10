//! Local HTTP Studio & Capabilities Workbench Server (`http://127.0.0.1:8787`).
//!
//! Explicitly avoids QualiaDB / Webizen Desktop reserved ports (`8080`, `4242`, `9001`).
//! Serves the interactive 5-tab Synth Studio UI (`GET /`) for QualiaDB demo pages,
//! the project workbench (`GET /project`), and live JSON endpoints:
//! - `GET /api/status`
//! - `GET /api/capabilities`
//! - `POST /api/produce`
//! - `POST /api/mcp/call`
//! - `GET /api/project`
//! - `GET|POST /api/project/annotations`

use crate::{
    present::{render_resource_card, render_session_html},
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
        let _ = handle_connection(stream);
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
    match range_header.map(str::trim).filter(|s| !s.is_empty()) {
        None => write_http_response_with(
            stream,
            "200 OK",
            content_type,
            body,
            &[("Accept-Ranges", "bytes")],
        ),
        Some(header) => match parse_single_byte_range(header, total) {
            Some((start, end)) => {
                let content_range = format!("bytes {start}-{end}/{total}");
                write_http_response_with(
                    stream,
                    "206 Partial Content",
                    content_type,
                    &body[start..=end],
                    &[
                        ("Accept-Ranges", "bytes"),
                        ("Content-Range", &content_range),
                    ],
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
        ("GET", "/project" | "/project/") => {
            let html = render_project_html()?;
            write_http_response(stream, "200 OK", "text/html; charset=utf-8", html.as_bytes())
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
    }
}
