//! Deterministic, self-contained HTML presentation for Antigravity IDE,
//! Antigravity 2.0 Auxiliary Pane / `<agent-embed>`, and Browser Preview.
use crate::production::{ProductionSession, Resource};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{fmt::Write as _, fs::OpenOptions, io::Write, path::Path};

pub const PRESENTATION_SCHEMA: &str = "synth.presentation-html/v1";

fn hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        out.push(DIGITS[(b >> 4) as usize] as char);
        out.push(DIGITS[(b & 15) as usize] as char);
    }
    out
}

pub(crate) fn base64_encode(bytes: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = chunk.get(1).copied().unwrap_or(0) as u32;
        let b2 = chunk.get(2).copied().unwrap_or(0) as u32;
        let triple = (b0 << 16) | (b1 << 8) | b2;
        out.push(TABLE[((triple >> 18) & 0x3f) as usize] as char);
        out.push(TABLE[((triple >> 12) & 0x3f) as usize] as char);
        out.push(if chunk.len() > 1 { TABLE[((triple >> 6) & 0x3f) as usize] as char } else { '=' });
        out.push(if chunk.len() > 2 { TABLE[(triple & 0x3f) as usize] as char } else { '=' });
    }
    out
}

pub(crate) fn escape_html(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for ch in text.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(ch),
        }
    }
    out
}

fn classify_modality(kind: &str, key: &str) -> &'static str {
    let ext = key.rsplit('.').next().unwrap_or("").to_ascii_lowercase();
    match (kind.to_ascii_lowercase().as_str(), ext.as_str()) {
        ("svg", _) | (_, "svg") => "vector",
        ("png" | "ppm" | "jpg" | "jpeg" | "webp", _) | (_, "png" | "ppm") => "raster",
        ("10d" | "m64" | "e64" | "emf" | "q42" | "p64" | "dxf", _)
        | (_, "10d" | "m64" | "e64" | "emf" | "q42" | "p64" | "dxf") => "spatial",
        ("wav" | "aac" | "qac", _) | (_, "wav" | "aac" | "qac") => "audio",
        ("mp4", _) | (_, "mp4") => "video",
        _ => "data",
    }
}

pub fn classify_macro_domain(key: &str) -> (&'static str, &'static str, &'static str) {
    let lower = key.to_ascii_lowercase();
    if ["world_of_god/", "wallaby/", "fauna/", "flora/", "biosphere/", "cosmos/"].iter().any(|p| lower.starts_with(p)) {
        ("world-of-god", "World of God · Flora, Fauna, Biosphere & Space", "border-emerald-500/50 bg-emerald-950/60 text-emerald-300")
    } else if ["world_of_man/", "vehicle/", "street/", "house/", "bim/"].iter().any(|p| lower.starts_with(p)) {
        ("world-of-man", "World of Man · Cars, Streets, Houses & BIM/CAD", "border-amber-500/50 bg-amber-950/60 text-amber-300")
    } else if ["world_of_mind/software/", "tray/", "studio/", "audio/", "environment/"].iter().any(|p| lower.starts_with(p)) {
        ("world-of-mind-software", "World of Mind ↳ Software & Systems (schema:SoftwareApplication)", "border-sky-500/50 bg-sky-950/60 text-sky-300")
    } else {
        ("world-of-mind-creative", "World of Mind ↳ Creative Works (schema:CreativeWork)", "border-purple-500/50 bg-purple-950/60 text-purple-300")
    }
}

fn hex_preview(bytes: &[u8], limit: usize) -> String {
    let mut out = String::new();
    for (row_idx, chunk) in bytes.iter().take(limit).collect::<Vec<_>>().chunks(16).enumerate() {
        let _ = write!(&mut out, "{:04x}: ", row_idx * 16);
        for b in chunk { let _ = write!(&mut out, "{b:02x} "); }
        out.push('\n');
    }
    if bytes.len() > limit {
        let _ = writeln!(&mut out, "... ({} additional bytes)", bytes.len() - limit);
    }
    out
}

fn portal_pngs_for(key: &str, all: &[Resource]) -> Vec<(String, String)> {
    all.iter()
        .filter(|r| {
            (r.kind.eq_ignore_ascii_case("png") || r.key.ends_with(".png"))
                && crate::present_portal::related_png(key, &r.key)
                && r.bytes.len() <= 8 * 1024 * 1024
        })
        .map(|r| (r.key.clone(), format!("data:image/png;base64,{}", base64_encode(&r.bytes))))
        .collect()
}

/// Render a single resource card HTML fragment.
pub fn render_resource_card(index: usize, resource: &Resource) -> String {
    render_resource_card_in_set(index, resource, std::slice::from_ref(resource))
}

/// Render a resource card, attaching sibling PNG contact-sheet bytes when `all` is the output set.
pub fn render_resource_card_in_set(index: usize, resource: &Resource, all: &[Resource]) -> String {
    let sha = hex(&Sha256::digest(&resource.bytes));
    let modality = classify_modality(&resource.kind, &resource.key);
    let safe_key = escape_html(&resource.key);
    let safe_kind = escape_html(&resource.kind);
    let preview_body = match modality {
        "vector" => {
            if let Ok(svg_text) = std::str::from_utf8(&resource.bytes) {
                let data_uri = format!("data:image/svg+xml;base64,{}", base64_encode(&resource.bytes));
                let escaped_svg = escape_html(svg_text);
                format!(
                    r#"<div class="space-y-3"><div class="flex items-center justify-center rounded-lg border border-[var(--border)] p-4 min-h-[180px]" style="background-image:radial-gradient(rgba(140,230,186,0.16) 1px, transparent 1px);background-size:16px 16px;"><img src="{data_uri}" alt="{safe_key}" class="max-h-[260px] w-auto object-contain" /></div><details class="text-xs"><summary class="cursor-pointer text-[var(--muted-foreground)] hover:text-[var(--foreground)] select-none">Inspect SVG markup ({len} bytes)</summary><pre class="mt-2 p-3 rounded-lg bg-[var(--background)] border border-[var(--border)] overflow-x-auto font-mono text-xs leading-relaxed">{escaped_svg}</pre></details></div>"#,
                    len = resource.bytes.len()
                )
            } else {
                format!(r#"<pre class="p-3 rounded-lg bg-[var(--background)] border border-[var(--border)] overflow-x-auto font-mono text-xs">{}</pre>"#, escape_html(&hex_preview(&resource.bytes, 256)))
            }
        }
        "raster" if resource.kind.eq_ignore_ascii_case("png") || resource.key.ends_with(".png") => {
            let data_uri = format!("data:image/png;base64,{}", base64_encode(&resource.bytes));
            format!(
                r#"<div class="flex items-center justify-center rounded-lg border border-[var(--border)] p-4 min-h-[180px]" style="background-image:linear-gradient(45deg, rgba(128,128,128,0.12) 25%, transparent 25%), linear-gradient(-45deg, rgba(128,128,128,0.12) 25%, transparent 25%), linear-gradient(45deg, transparent 75%, rgba(128,128,128,0.12) 75%), linear-gradient(-45deg, transparent 75%, rgba(128,128,128,0.12) 75%);background-size:16px 16px;background-position:0 0, 0 8px, 8px -8px, -8px 0;"><img src="{data_uri}" alt="{safe_key}" class="max-h-[260px] w-auto object-contain rounded" /></div>"#
            )
        }
        "audio" if resource.kind.eq_ignore_ascii_case("wav") || resource.key.ends_with(".wav") => {
            let data_uri = format!("data:audio/wav;base64,{}", base64_encode(&resource.bytes));
            format!(
                r#"<div class="p-4 rounded-lg bg-[var(--background)] border border-[var(--border)] space-y-2"><div class="text-xs text-[var(--muted-foreground)]">PCM / WAV Audio Stream · {len} bytes</div><audio controls src="{data_uri}" class="w-full"></audio></div>"#,
                len = resource.bytes.len()
            )
        }
        "audio" if resource.kind.eq_ignore_ascii_case("qac") || resource.key.ends_with(".qac") => {
            crate::present_qac::render_session_qac_card(index, &safe_key, &resource.bytes)
        }
        "video" if resource.kind.eq_ignore_ascii_case("mp4") || resource.key.ends_with(".mp4") => {
            let data_uri = format!("data:video/mp4;base64,{}", base64_encode(&resource.bytes));
            format!(
                r#"<div class="p-3.5 rounded-lg bg-[var(--background)] border border-emerald-500/40 space-y-2.5"><div class="flex flex-wrap items-center justify-between gap-2 text-xs font-mono"><span class="text-emerald-300 font-semibold">Deterministic Cinema Stream · H.264 / ISO-BMFF (.mp4)</span><span class="text-[var(--muted-foreground)]">{len} bytes · YUV420p Macroblock-Aligned</span></div><div class="flex items-center justify-center rounded-lg border border-[var(--border)] bg-[#070d0c] p-3"><video controls autoplay loop muted playsinline src="{data_uri}" class="max-h-[260px] w-auto rounded border border-emerald-500/30"></video></div></div>"#,
                len = resource.bytes.len()
            )
        }
        "spatial" => {
            let dump = escape_html(&hex_preview(&resource.bytes, 128));
            if resource.kind.eq_ignore_ascii_case("10d") || resource.key.ends_with(".10d") {
                let png_owned = portal_pngs_for(&resource.key, all);
                let pngs: Vec<crate::present_portal::PortalPng<'_>> = png_owned
                    .iter()
                    .map(|(rel, src)| crate::present_portal::PortalPng { rel, src })
                    .collect();
                let mesh_url = resource.key.clone();
                let emf_url = crate::present_portal::sibling_with_ext(&mesh_url, "emf");
                let m64_url = crate::present_portal::sibling_with_ext(&mesh_url, "m64");
                let qac_sibling = crate::present_portal::sibling_with_ext(&mesh_url, "qac");
                let emf_res = all.iter().find(|r| r.key.eq_ignore_ascii_case(&emf_url));
                let m64_bytes = all
                    .iter()
                    .find(|r| r.key.eq_ignore_ascii_case(&m64_url))
                    .map(|r| r.bytes.as_slice());
                let qac_res = all.iter().find(|r| r.key.eq_ignore_ascii_case(&qac_sibling));
                let mesh_b64 = base64_encode(&resource.bytes);
                let emf_bytes = emf_res.map(|r| r.bytes.as_slice());
                let emf_b64 = emf_res.map(|r| base64_encode(&r.bytes)).unwrap_or_default();
                let qac_url = if qac_res.is_some() {
                    qac_sibling.as_str()
                } else {
                    ""
                };
                let qac_b64 = qac_res.map(|r| base64_encode(&r.bytes)).unwrap_or_default();
                crate::present_3d::render_3d_host(
                    index,
                    &safe_key,
                    &resource.bytes,
                    m64_bytes,
                    emf_bytes,
                    &mesh_url,
                    &emf_url,
                    qac_url,
                    &mesh_b64,
                    &emf_b64,
                    &qac_b64,
                    &pngs,
                    &dump,
                )
            } else if ["m64", "e64", "p64", "emf", "q42"]
                .iter()
                .any(|ext| resource.kind.eq_ignore_ascii_case(ext) || resource.key.ends_with(&format!(".{ext}")))
            {
                crate::present_3d::render_m64_or_emf_card_body(&safe_key, &safe_kind, &resource.bytes, &dump)
            } else {
                format!(
                    r#"<div class="p-3 rounded-lg bg-[var(--background)] border border-[var(--border)] space-y-2"><div class="flex items-center justify-between text-xs"><span class="font-medium text-[var(--foreground)]">Native Spatial / Manifold Container (`.{safe_kind}`)</span><span class="text-[var(--muted-foreground)]">Use `synth preview --asset {safe_key} --out DIR` for 9-angle 2048px render</span></div><pre class="overflow-x-auto font-mono text-xs text-[var(--muted-foreground)] leading-relaxed">{dump}</pre></div>"#
                )
            }
        }
        _ => {
            if let Ok(text) = std::str::from_utf8(&resource.bytes) {
                let snippet: String = text.chars().take(4096).collect();
                format!(r#"<pre class="p-3 rounded-lg bg-[var(--background)] border border-[var(--border)] overflow-x-auto font-mono text-xs leading-relaxed">{}</pre>"#, escape_html(&snippet))
            } else {
                format!(r#"<pre class="p-3 rounded-lg bg-[var(--background)] border border-[var(--border)] overflow-x-auto font-mono text-xs leading-relaxed">{}</pre>"#, escape_html(&hex_preview(&resource.bytes, 256)))
            }
        }
    };

    let (domain_id, domain_label, domain_badge_cls) = classify_macro_domain(&resource.key);
    let (stage_id, stage_label, stage_badge_cls, stage_note) =
        crate::present_style::classify_lifecycle_stage(&resource.key);

    format!(
        r#"<article data-modality="{modality}" data-domain="{domain_id}" data-stage="{stage_id}" class="resource-card bg-[var(--card)] text-[var(--foreground)] border border-[var(--border)] rounded-xl p-4 shadow-sm space-y-3"><div class="space-y-2 border-b border-[var(--border)] pb-3"><div class="flex flex-wrap items-center justify-between gap-2"><div class="flex flex-wrap items-center gap-2"><span class="px-2 py-0.5 rounded text-xs font-mono border border-[var(--border)] bg-[var(--background)]">#{index}</span><h3 class="font-semibold text-sm font-mono">{safe_key}</h3></div><div class="flex items-center gap-2 text-xs text-[var(--muted-foreground)] font-mono"><span class="px-2 py-0.5 rounded bg-[var(--background)] border border-[var(--border)] uppercase">{safe_kind}</span><span>{bytes} B</span><span title="SHA-256: {sha}">sha256:{short_sha}…</span></div></div><div class="flex flex-wrap items-center justify-between gap-2"><div class="flex flex-wrap items-center gap-1.5"><span class="px-2 py-0.5 rounded text-[11px] font-mono border {domain_badge_cls}">{domain_label}</span><span class="px-2 py-0.5 rounded text-[11px] font-mono {stage_badge_cls}">{stage_label}</span></div><span class="text-[11px] text-[var(--muted-foreground)]">{stage_note}</span></div></div>{preview_body}</article>"#,
        bytes = resource.bytes.len(),
        short_sha = &sha[..12],
    )
}

/// Render a self-contained HTML presentation for an admitted `ProductionSession`.
pub fn render_session_html(session: &ProductionSession) -> Result<String, String> {
    let candidate = session.candidate().ok_or("No candidate available to present")?;
    let doc_digest = hex(&session.document_digest()?);
    let pkg_sha = candidate.sha256();
    let (producer_id, producer_ver) = session.profile();
    let outputs = candidate.outputs();
    let total_bytes: usize = outputs.iter().map(|r| r.bytes.len()).sum();

    let mut cards = String::new();
    for (idx, resource) in outputs.iter().enumerate() {
        cards.push_str(&render_resource_card_in_set(idx + 1, resource, outputs));
        cards.push('\n');
    }

    let mut source_cells = String::new();
    for (idx, src) in session.sources().iter().enumerate() {
        let _ = write!(
            &mut source_cells,
            r#"<div class="border border-[var(--border)] rounded-lg p-3 bg-[var(--background)] space-y-2"><div class="flex items-center justify-between text-xs text-[var(--muted-foreground)] font-mono"><span>Source Cell #{idx}</span><span>{len} bytes</span></div><pre class="overflow-x-auto font-mono text-xs leading-relaxed text-[var(--foreground)]">{escaped}</pre></div>"#,
            len = src.len(),
            escaped = escape_html(src)
        );
    }

    let snap = crate::present_capabilities::FullCapabilitiesSnapshot::collect();
    let capabilities_html = crate::present_capabilities::render_capabilities_panels_html(&snap);
    let morph_count = snap.morphisms["morphisms"].as_array().map_or(0, Vec::len);
    let mcp_count = snap.mcp_tools.len();
    let sdk_count = snap.vibe_sdk["providers"].as_array().map_or(0, Vec::len);
    let pin_count = snap.living_world["pins"].as_array().map_or(0, Vec::len);
    let model_count = snap.discovered_models.len();
    let default_vibe = escape_html(
        session.sources().last().map(String::as_str).unwrap_or(crate::tray_env::TRAY_ENVIRONMENT_SOURCE),
    );

    Ok(format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head><meta charset="utf-8" /><meta name="viewport" content="width=device-width, initial-scale=1" /><title>Synth Studio &amp; Capabilities Workbench · {producer_id}@{producer_ver} · rev {rev}</title><style>{studio_css}</style></head>
<body>
  <main class="max-w-7xl mx-auto space-y-5">
    <header class="bg-[var(--card)] border border-[var(--border)] rounded-xl p-5 shadow-sm space-y-4">
      <div class="flex flex-wrap items-center justify-between gap-3"><div><div class="text-xs font-mono uppercase tracking-wider text-emerald-400">{schema} · Human-Rights Asset, Ontology &amp; MCP Studio</div><h1 class="text-xl font-semibold mt-0.5">Synth Studio &amp; Substantive Capabilities Workbench — <span class="font-mono text-emerald-300">{producer_id}@{producer_ver}</span></h1></div><div class="flex flex-wrap items-center gap-2 text-xs font-mono"><a class="px-2.5 py-1 rounded-md bg-amber-950/50 border border-amber-500/40 text-amber-200" href="/project">Project workbench</a><span class="px-2.5 py-1 rounded-md bg-emerald-950/60 border border-emerald-500/50 text-emerald-300">{mode_label}</span><span class="px-2.5 py-1 rounded-md bg-[var(--background)] border border-[var(--border)]">Revision {rev}</span><span class="px-2.5 py-1 rounded-md bg-[var(--background)] border border-[var(--border)]">{count} Resources ({total_bytes} B)</span></div></div>
      <div class="text-[11px] font-mono text-[var(--muted-foreground)]">QualiaPortal WASM: <span class="text-emerald-300">/qualia/js/qualia-shell.js</span> on the synth http origin (COOP/COEP). file:// keeps the PNG contact sheet. 2D canvas is degraded-only.</div>
      {webgpu_snapshot}
      <div class="grid grid-cols-2 md:grid-cols-6 gap-2 pt-2 border-t border-[var(--border)] text-xs font-mono">
        <div class="bg-[var(--background)] border border-[var(--border)] rounded-lg p-2.5"><div class="text-[var(--muted-foreground)] text-[10px] uppercase">Produced Assets</div><div class="text-sm font-bold text-emerald-300">{count} files · rev {rev}</div></div>
        <div class="bg-[var(--background)] border border-[var(--border)] rounded-lg p-2.5"><div class="text-[var(--muted-foreground)] text-[10px] uppercase">Categorical Ontology</div><div class="text-sm font-bold text-sky-300">12 cats · {morph_count} morphisms</div></div>
        <div class="bg-[var(--background)] border border-[var(--border)] rounded-lg p-2.5"><div class="text-[var(--muted-foreground)] text-[10px] uppercase">Unified MCP Suite</div><div class="text-sm font-bold text-amber-300">{mcp_count} tools (0.0.41)</div></div>
        <div class="bg-[var(--background)] border border-[var(--border)] rounded-lg p-2.5"><div class="text-[var(--muted-foreground)] text-[10px] uppercase">Etalon-Vibe SDK</div><div class="text-sm font-bold text-emerald-300">{sdk_count} providers · Parity OK</div></div>
        <div class="bg-[var(--background)] border border-[var(--border)] rounded-lg p-2.5"><div class="text-[var(--muted-foreground)] text-[10px] uppercase">Living-World &amp; Cosmic</div><div class="text-sm font-bold text-sky-300">{pin_count} pins · OCS v2.2.0</div></div>
        <div class="bg-[var(--background)] border border-[var(--border)] rounded-lg p-2.5"><div class="text-[var(--muted-foreground)] text-[10px] uppercase">Local LLM Vault</div><div class="text-sm font-bold text-amber-300">{model_count} models (C:\LLM_files)</div></div>
      </div>
      <div class="flex flex-wrap items-center gap-2 pt-2 border-t border-[var(--border)]" role="tablist" aria-label="Studio Navigation">
        <button onclick="switchStudioTab('tab-assets', this)" class="studio-nav-btn px-3.5 py-1.5 rounded-lg text-xs font-mono font-semibold border border-emerald-500/70 bg-emerald-950/50 text-emerald-200 filter-pill-active">1. Asset Workbench ({count})</button>
        <button onclick="switchStudioTab('tab-ontology', this)" class="studio-nav-btn px-3.5 py-1.5 rounded-lg text-xs font-mono border border-[var(--border)] bg-[var(--background)]">2. Categorical Ontology (12 · {morph_count})</button>
        <button onclick="switchStudioTab('tab-mcp', this)" class="studio-nav-btn px-3.5 py-1.5 rounded-lg text-xs font-mono border border-[var(--border)] bg-[var(--background)]">3. QualiaDB + Synth MCP ({mcp_count})</button>
        <button onclick="switchStudioTab('tab-sdk-living', this)" class="studio-nav-btn px-3.5 py-1.5 rounded-lg text-xs font-mono border border-[var(--border)] bg-[var(--background)]">4. Vibe SDK ({sdk_count}) &amp; Living-World ({pin_count})</button>
        <button onclick="switchStudioTab('tab-llm-vault', this)" class="studio-nav-btn px-3.5 py-1.5 rounded-lg text-xs font-mono border border-[var(--border)] bg-[var(--background)]">5. Local LLM Vault ({model_count})</button>
      </div>
    </header>
    <section id="tab-assets" class="studio-tab space-y-5">
      <div class="bg-[var(--card)] border border-[var(--border)] rounded-xl p-4 shadow-sm space-y-3">
        <div class="lifecycle-pipeline-banner space-y-2">
          <div class="flex flex-wrap items-center justify-between gap-2 text-xs font-mono"><span class="font-semibold text-emerald-300">Creative-to-Realized Ontological Pipeline (World of Mind → Parametric CAD / BIM → Realized World of Man &amp; World of God)</span><span id="visible-asset-counter" class="text-[var(--muted-foreground)]">Showing {count} of {count} resources</span></div>
          <div class="flex flex-wrap items-center gap-2 text-xs"><div class="lifecycle-step"><span class="px-2 py-0.5 rounded font-mono text-[11px] badge-stage-concept">Stage 1</span><div><div class="font-semibold">Concept &amp; Creative Works (World of Mind)</div><div class="text-[11px] text-[var(--muted-foreground)]">schema:CreativeWork · Concept 3D models, music, fiction &amp; prototypes conceived before physical realization</div></div></div><span class="lifecycle-arrow">→</span><div class="lifecycle-step"><span class="px-2 py-0.5 rounded font-mono text-[11px] badge-stage-spec">Stage 2</span><div><div class="font-semibold">Parametric CAD, IFC4 BIM &amp; Anatomical Scaffolds</div><div class="text-[11px] text-[var(--muted-foreground)]">Mind → Reality Bridge · Orthogonal blueprints, STEP/IFC4 BIM, and portable .json/.vibe species scaffolds</div></div></div><span class="lifecycle-arrow">→</span><div class="lifecycle-step"><span class="px-2 py-0.5 rounded font-mono text-[11px] badge-stage-realized">Stage 3</span><div><div class="font-semibold">Realized Twins &amp; Production Artifacts</div><div class="text-[11px] text-[var(--muted-foreground)]">World of Man (Vehicles, Streets, Houses) · World of God (Flora, Fauna, Biosphere, Cosmos) · Active Runtime</div></div></div></div>
        </div>
        <div class="flex flex-wrap items-center justify-between gap-2 pt-2 border-t border-[var(--border)]"><div class="text-xs font-mono text-[var(--muted-foreground)]">1. Filter by Macro-Domain:</div><div class="flex flex-wrap items-center gap-1.5" role="tablist" aria-label="Macro-Domain Filter"><button onclick="filterDomain('all', this)" class="domain-btn px-2.5 py-1 rounded text-xs font-mono border border-emerald-500/70 bg-emerald-950/50 text-emerald-200 filter-pill-active" data-domain="all">All Domains ({count})</button><button onclick="filterDomain('world-of-god', this)" class="domain-btn px-2.5 py-1 rounded text-xs font-mono border border-emerald-500/40 bg-emerald-950/30 text-emerald-300" data-domain="world-of-god">World of God (Flora · Fauna · Biosphere · Space)</button><button onclick="filterDomain('world-of-man', this)" class="domain-btn px-2.5 py-1 rounded text-xs font-mono border border-amber-500/40 bg-amber-950/30 text-amber-300" data-domain="world-of-man">World of Man (Cars · Streets · Houses · BIM)</button><button onclick="filterDomain('world-of-mind', this)" class="domain-btn px-2.5 py-1 rounded text-xs font-mono border border-purple-500/40 bg-purple-950/30 text-purple-300" data-domain="world-of-mind">World of Mind (All)</button><button onclick="filterDomain('world-of-mind-creative', this)" class="domain-btn px-2.5 py-1 rounded text-xs font-mono border border-fuchsia-500/40 bg-fuchsia-950/30 text-fuchsia-300" data-domain="world-of-mind-creative">↳ Creative Works (schema:CreativeWork)</button><button onclick="filterDomain('world-of-mind-software', this)" class="domain-btn px-2.5 py-1 rounded text-xs font-mono border border-sky-500/40 bg-sky-950/30 text-sky-300" data-domain="world-of-mind-software">↳ Software &amp; Systems (schema:SoftwareApplication)</button></div></div>
        <div class="flex flex-wrap items-center justify-between gap-2 pt-2 border-t border-[var(--border)]"><div class="text-xs font-mono text-[var(--muted-foreground)]">2. Filter by Realization Lifecycle (Mind → Reality):</div><div class="flex flex-wrap items-center gap-1.5" role="tablist" aria-label="Lifecycle Stage Filter"><button onclick="filterStage('all', this)" class="stage-btn px-2.5 py-1 rounded text-xs font-mono border border-[var(--border)] bg-[var(--background)] filter-pill-active" data-stage="all">All Lifecycle Stages ({count})</button><button onclick="filterStage('concept-creative', this)" class="stage-btn px-2.5 py-1 rounded text-xs font-mono badge-stage-concept" data-stage="concept-creative">Stage 1 · Concept &amp; Creative Models (Mind)</button><button onclick="filterStage('cad-spec', this)" class="stage-btn px-2.5 py-1 rounded text-xs font-mono badge-stage-spec" data-stage="cad-spec">Stage 2 · CAD / BIM / Anatomical Specs (Mind → Reality Bridge)</button><button onclick="filterStage('realized-twin', this)" class="stage-btn px-2.5 py-1 rounded text-xs font-mono badge-stage-realized" data-stage="realized-twin">Stage 3 · Realized Twins &amp; Production (Man · God · Runtime)</button></div></div>
        <div class="flex flex-wrap items-center justify-between gap-2 pt-2 border-t border-[var(--border)]"><div class="text-xs font-mono text-[var(--muted-foreground)]">3. Filter by Modality:</div><div class="flex flex-wrap items-center gap-1.5" role="tablist" aria-label="Modality Filter"><button onclick="filterCards('all', this)" class="filter-btn px-2.5 py-1 rounded text-xs font-mono border border-[var(--border)] bg-[var(--background)] filter-pill-active" data-filter="all">All Modalities ({count})</button><button onclick="filterCards('spatial', this)" class="filter-btn px-2.5 py-1 rounded text-xs font-mono border border-[var(--border)] bg-[var(--background)]" data-filter="spatial">Spatial (.10d · .m64 · .e64 · .emf · .p64 · .q42)</button><button onclick="filterCards('video', this)" class="filter-btn px-2.5 py-1 rounded text-xs font-mono border border-[var(--border)] bg-[var(--background)]" data-filter="video">Cinema Video (.mp4)</button><button onclick="filterCards('raster', this)" class="filter-btn px-2.5 py-1 rounded text-xs font-mono border border-[var(--border)] bg-[var(--background)]" data-filter="raster">Raster / PNG</button><button onclick="filterCards('vector', this)" class="filter-btn px-2.5 py-1 rounded text-xs font-mono border border-[var(--border)] bg-[var(--background)]" data-filter="vector">Vector / SVG</button><button onclick="filterCards('audio', this)" class="filter-btn px-2.5 py-1 rounded text-xs font-mono border border-[var(--border)] bg-[var(--background)]" data-filter="audio">Audio / WAV / .qac</button><button onclick="filterCards('data', this)" class="filter-btn px-2.5 py-1 rounded text-xs font-mono border border-[var(--border)] bg-[var(--background)]" data-filter="data">Data / BIM / MaterialX / LUT</button></div></div>
        <div class="flex flex-wrap items-center justify-between gap-2 pt-2 border-t border-[var(--border)] text-xs font-mono text-[var(--muted-foreground)]"><div>HMC SHA-256: <span id="hmc-sha-display" class="text-[var(--foreground)]">{pkg_sha}</span> · Document: <span id="doc-sha-display" class="text-[var(--foreground)]">{doc_digest}</span></div></div>
        <details class="pt-2 border-t border-[var(--border)]"><summary class="cursor-pointer text-xs font-mono text-emerald-300 font-semibold">Live VibeScript Authoring &amp; Job Runner (executes via local Synth Studio Server)</summary><div class="mt-3 space-y-2"><textarea id="live-vibe-source" rows="8" class="w-full p-3 rounded-lg bg-[var(--background)] border border-[var(--border)] font-mono text-xs text-[var(--foreground)]">{default_vibe}</textarea><div class="flex items-center gap-3"><button onclick="runLiveVibeProduction()" class="px-4 py-1.5 rounded-lg text-xs font-mono font-semibold bg-emerald-600/30 border border-emerald-500 text-emerald-200">Compile &amp; Produce Candidate</button><span id="live-vibe-status" class="text-xs font-mono text-[var(--muted-foreground)]"></span></div></div></details>
      </div>
      <div id="resource-grid" class="grid grid-cols-1 lg:grid-cols-2 gap-4">{cards}</div>
      <div class="bg-[var(--card)] border border-[var(--border)] rounded-xl p-5 shadow-sm space-y-3"><h2 class="text-sm font-semibold">VibeScript Source History ({src_count} cells)</h2><div class="space-y-3">{source_cells}</div></div>
    </section>
    {capabilities_html}
  </main>
  <script>
    let activeMorphCat = 'all', activeMcpDom = 'all', activeModality = 'all', activeDomain = 'all', activeStage = 'all';
    function switchStudioTab(tabId, btn) {{ document.querySelectorAll('.studio-tab').forEach(el => el.classList.add('hidden')); const target = document.getElementById(tabId); if (target) target.classList.remove('hidden'); document.querySelectorAll('.studio-nav-btn').forEach(b => {{ b.className = 'studio-nav-btn px-3.5 py-1.5 rounded-lg text-xs font-mono border border-[var(--border)] bg-[var(--background)]'; }}); if (btn) btn.className = 'studio-nav-btn px-3.5 py-1.5 rounded-lg text-xs font-mono font-semibold border border-emerald-500/70 bg-emerald-950/50 text-emerald-200 filter-pill-active'; }}
    function applyAssetCardFilters() {{ const cards = document.querySelectorAll('.resource-card'); let shown = 0; cards.forEach(card => {{ const modOk = activeModality === 'all' || card.dataset.modality === activeModality; const dom = card.dataset.domain || ''; const domOk = activeDomain === 'all' || dom === activeDomain || (activeDomain === 'world-of-mind' && (dom === 'world-of-mind' || dom === 'world-of-mind-creative' || dom === 'world-of-mind-software')); const stgOk = activeStage === 'all' || (card.dataset.stage || '') === activeStage; const vis = modOk && domOk && stgOk; card.style.display = vis ? '' : 'none'; if (vis) shown++; }}); const counter = document.getElementById('visible-asset-counter'); if (counter) counter.textContent = 'Showing ' + shown + ' of ' + cards.length + ' resources'; }}
    function filterCards(modality, btn) {{ activeModality = modality; document.querySelectorAll('.filter-btn').forEach(b => b.classList.remove('filter-pill-active')); if (btn) btn.classList.add('filter-pill-active'); applyAssetCardFilters(); }}
    function filterDomain(domain, btn) {{ activeDomain = domain; document.querySelectorAll('.domain-btn').forEach(b => b.classList.remove('filter-pill-active')); if (btn) btn.classList.add('filter-pill-active'); applyAssetCardFilters(); }}
    function filterStage(stage, btn) {{ activeStage = stage; document.querySelectorAll('.stage-btn').forEach(b => b.classList.remove('filter-pill-active')); if (btn) btn.classList.add('filter-pill-active'); applyAssetCardFilters(); }}
    function setMorphCategory(cat) {{ activeMorphCat = cat; filterMorphisms(); }}
    function filterMorphisms() {{ const q = (document.getElementById('morph-search')?.value || '').toLowerCase(); document.querySelectorAll('.morph-card').forEach(card => {{ const catOk = activeMorphCat === 'all' || card.dataset.cat === activeMorphCat; const textOk = !q || card.textContent.toLowerCase().includes(q); card.style.display = (catOk && textOk) ? '' : 'none'; }}); }}
    function setMcpDomain(dom) {{ activeMcpDom = dom; filterMcpTools(); }}
    function filterMcpTools() {{ const q = (document.getElementById('mcp-search')?.value || '').toLowerCase(); document.querySelectorAll('.mcp-card').forEach(card => {{ const domOk = activeMcpDom === 'all' || card.dataset.dom === activeMcpDom; const textOk = !q || card.textContent.toLowerCase().includes(q); card.style.display = (domOk && textOk) ? '' : 'none'; }}); }}
    function selectMcpTool(name) {{ const el = document.getElementById('mcp-exec-name'); if (el) el.value = name; }}
    async function invokeMcpToolLive() {{ const name = document.getElementById('mcp-exec-name')?.value || ''; const rawArgs = document.getElementById('mcp-exec-args')?.value || '{{}}'; const out = document.getElementById('mcp-exec-output'); if (!out) return; out.classList.remove('hidden'); out.textContent = 'Executing ' + name + '...'; try {{ const res = await fetch('/api/mcp/call', {{ method: 'POST', headers: {{ 'Content-Type': 'application/json' }}, body: JSON.stringify({{ name, arguments: JSON.parse(rawArgs) }}) }}); out.textContent = JSON.stringify(await res.json(), null, 2); }} catch (err) {{ out.textContent = 'Local server required (run `synth serve` or `synth tray`): ' + err; }} }}
    async function runLiveVibeProduction() {{ const source = document.getElementById('live-vibe-source')?.value || ''; const status = document.getElementById('live-vibe-status'); if (status) status.textContent = 'Compiling & producing candidate...'; try {{ const res = await fetch('/api/produce', {{ method: 'POST', headers: {{ 'Content-Type': 'application/json' }}, body: JSON.stringify({{ source }}) }}); const data = await res.json(); if (!data.ok) {{ if (status) status.textContent = 'Error: ' + data.error; return; }} const grid = document.getElementById('resource-grid'); if (grid && data.cards_html) {{ grid.innerHTML = data.cards_html; grid.querySelectorAll('script').forEach(oldScript => {{ const s = document.createElement('script'); s.textContent = oldScript.textContent; oldScript.replaceWith(s); }}); }} const hmcEl = document.getElementById('hmc-sha-display'); if (hmcEl && data.package_sha256) hmcEl.textContent = data.package_sha256; const docEl = document.getElementById('doc-sha-display'); if (docEl && data.document_digest) docEl.textContent = data.document_digest; if (status) status.textContent = 'Produced ' + data.resource_count + ' resources · strict replay verified (SHA-256: ' + data.package_sha256.slice(0, 12) + '…)'; if (typeof window.bootSynthPortals === 'function') window.bootSynthPortals(); if (typeof window.bootSynthQacCards === 'function') window.bootSynthQacCards(); }} catch (err) {{ if (status) status.textContent = 'Local server required (run `synth serve` or `synth tray`): ' + err; }} }}
    {viewer_3d_js}
    {portal_boot_js}
    {qac_viewer_js}
  </script>
</body>
</html>
"#,
        schema = PRESENTATION_SCHEMA,
        producer_id = escape_html(producer_id),
        producer_ver = escape_html(producer_ver),
        mode_label = escape_html(&snap.mode_label),
        rev = candidate.revision(),
        count = outputs.len(),
        src_count = session.sources().len(),
        studio_css = crate::present_style::SYNTH_STUDIO_CSS,
        viewer_3d_js = crate::present_3d::SYNTH_3D_VIEWER_RUNTIME_JS,
        portal_boot_js = crate::present_portal::PORTAL_BOOT_JS,
        qac_viewer_js = crate::present_qac::QAC_VIEWER_JS,
        webgpu_snapshot = crate::present_portal::webgpu_snapshot_html(),
    ))
}

/// Render an interactive 9-angle `.10d` turntable HTML presentation for `synth preview`.
pub fn render_10d_preview_html(receipt: &Value, views: &[(String, Vec<u8>)]) -> Result<String, String> {
    let source_path = receipt["source"]["path"].as_str().unwrap_or(".10d");
    let source_sha = receipt["source"]["sha256"].as_str().unwrap_or("");
    let source_bytes = receipt["source"]["bytes"].as_u64().unwrap_or(0);
    let mut buttons = String::new();
    let mut panels = String::new();
    for (idx, (name, png)) in views.iter().enumerate() {
        let data_uri = format!("data:image/png;base64,{}", base64_encode(png));
        let safe_name = escape_html(name);
        let active_style = if idx == 0 { "" } else { "display:none;" };
        let _ = write!(&mut buttons, r#"<button onclick="selectView({idx})" class="px-3 py-1 rounded-md text-xs font-mono border border-[var(--border)] bg-[var(--background)]">{safe_name}</button>"#);
        let _ = write!(
            &mut panels,
            r#"<div id="view-{idx}" class="view-panel space-y-2" style="{active_style}"><div class="text-xs font-mono text-[var(--muted-foreground)]">Camera Angle: <span class="text-[var(--foreground)]">{safe_name}</span> ({bytes} B)</div><div class="flex items-center justify-center rounded-lg border border-[var(--border)] bg-[var(--background)] p-3"><img src="{data_uri}" alt="{safe_name}" class="max-h-[420px] w-auto object-contain rounded" /></div></div>"#,
            bytes = png.len()
        );
    }
    let receipt_pretty = escape_html(&serde_json::to_string_pretty(receipt).map_err(|e| e.to_string())?);
    Ok(format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head><meta charset="utf-8" /><meta name="viewport" content="width=device-width, initial-scale=1" /><title>Synth .10d Preview · {path}</title><style>{studio_css}</style></head>
<body>
  <main class="max-w-5xl mx-auto space-y-4">
    <div class="bg-[var(--card)] border border-[var(--border)] rounded-xl p-5 shadow-sm space-y-3">
      <div class="text-xs font-mono uppercase tracking-wider text-[var(--muted-foreground)]">synth.native-10d-preview/v1 · Volumetric Turntable</div>
      <h1 class="text-lg font-semibold font-mono truncate">{path}</h1>
      <div class="text-xs font-mono text-[var(--muted-foreground)]">SHA-256: {sha} ({source_bytes} B)</div>
      <div class="flex flex-wrap items-center gap-3 pt-2 border-t border-[var(--border)]">
        <button id="motion-play-btn" onclick="toggleMotionPlay()" class="px-3 py-1 rounded-md text-xs font-mono font-semibold border border-[var(--border)] bg-[var(--primary)] text-[var(--primary-foreground)]">Play Motion</button>
        <input id="motion-scrubber" type="range" min="0" max="{max_idx}" value="0" oninput="selectView(parseInt(this.value,10))" class="w-48" />
        <label class="text-xs font-mono text-[var(--muted-foreground)]">FPS:
          <select id="motion-fps" class="ml-1 px-2 py-0.5 rounded bg-[var(--background)] border border-[var(--border)] text-xs font-mono">
            <option value="4">4</option>
            <option value="8" selected>8</option>
            <option value="12">12</option>
            <option value="24">24</option>
          </select>
        </label>
        <span id="motion-frame-label" class="text-xs font-mono text-[var(--muted-foreground)]">Frame 1 / {total_views}</span>
      </div>
      <div class="flex flex-wrap gap-2 pt-2">{buttons}</div>
      <div class="pt-2">{panels}</div>
      <details class="text-xs pt-2"><summary class="cursor-pointer text-[var(--muted-foreground)] font-mono">Annotate Viewport Frame / Pin Feedback for Agent</summary>
        <div class="mt-2 p-3 rounded-lg bg-[var(--background)] border border-[var(--border)] space-y-2">
          <div class="flex gap-2">
            <input id="ann-comment" type="text" placeholder="Describe shape/motion/material feedback at this frame..." class="flex-1 px-2 py-1 rounded border border-[var(--border)] bg-[var(--card)] text-xs font-mono" />
            <button onclick="submitViewportAnnotation()" class="px-3 py-1 rounded border border-[var(--border)] bg-[var(--card)] text-xs font-mono">Save Annotation</button>
          </div>
          <div id="ann-status" class="text-xs font-mono text-[var(--muted-foreground)]"></div>
        </div>
      </details>
      <details class="text-xs pt-2"><summary class="cursor-pointer text-[var(--muted-foreground)] font-mono">Inspect review-receipt.json</summary><pre class="mt-2 p-3 rounded-lg bg-[var(--background)] border border-[var(--border)] overflow-x-auto font-mono text-xs">{receipt_pretty}</pre></details>
    </div>
  </main>
  <script>
    var currentViewIndex = 0;
    var totalViews = {total_views};
    var motionTimer = null;
    function selectView(index) {{
      currentViewIndex = index;
      document.querySelectorAll('.view-panel').forEach((el, idx) => {{ el.style.display = idx === index ? '' : 'none'; }});
      var scrub = document.getElementById('motion-scrubber');
      if (scrub) scrub.value = index;
      var lbl = document.getElementById('motion-frame-label');
      if (lbl) lbl.textContent = 'Frame ' + (index + 1) + ' / ' + totalViews;
    }}
    function toggleMotionPlay() {{
      var btn = document.getElementById('motion-play-btn');
      if (motionTimer) {{
        clearInterval(motionTimer);
        motionTimer = null;
        if (btn) btn.textContent = 'Play Motion';
      }} else {{
        var fpsEl = document.getElementById('motion-fps');
        var fps = fpsEl ? parseInt(fpsEl.value, 10) || 8 : 8;
        if (btn) btn.textContent = 'Pause';
        motionTimer = setInterval(function() {{
          selectView((currentViewIndex + 1) % Math.max(1, totalViews));
        }}, Math.max(30, Math.round(1000 / fps)));
      }}
    }}
    function submitViewportAnnotation() {{
      var comment = (document.getElementById('ann-comment') || {{}}).value || '';
      var status = document.getElementById('ann-status');
      var payload = {{ file: '{path}', lane: 'working', frame_index: currentViewIndex, comment: comment, rating: 3 }};
      fetch('http://127.0.0.1:8787/api/project/annotations', {{
        method: 'POST',
        headers: {{ 'Content-Type': 'application/json' }},
        body: JSON.stringify(payload)
      }}).then(function(r) {{ return r.json(); }}).then(function() {{
        if (status) status.textContent = 'Saved annotation at frame ' + currentViewIndex + ' to .synth/annotations.json';
      }}).catch(function() {{
        if (status) status.textContent = 'Offline annotation JSON: ' + JSON.stringify(payload);
      }});
    }}
  </script>
</body>
</html>
"#,
        path = escape_html(source_path),
        sha = escape_html(source_sha),
        max_idx = views.len().saturating_sub(1),
        total_views = views.len(),
        studio_css = crate::present_style::SYNTH_STUDIO_CSS,
    ))
}

/// Write a create-only `.html` presentation file within budget.
pub fn write_presentation_file(path: &Path, html: &str, max_bytes: usize) -> Result<(), String> {
    if path.extension().and_then(|e| e.to_str()) != Some("html") {
        return Err("Presentation output must use .html".into());
    }
    if html.len() > max_bytes {
        return Err("Presentation HTML exceeds package byte budget".into());
    }
    let parent = path.parent().filter(|p| !p.as_os_str().is_empty()).unwrap_or(Path::new("."));
    if !parent.is_dir() || path.file_name().is_none() {
        return Err("Presentation output parent must exist".into());
    }
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|e| format!("create-only presentation {}: {e}", path.display()))?;
    file.write_all(html.as_bytes()).map_err(|e| e.to_string())?;
    file.sync_all().map_err(|e| e.to_string())
}

/// Windows `canonicalize` prefixes `\\?\` / `\\.\`; those must not become `file:///?/C:`.
fn file_url_from_fs_path(path: &str) -> String {
    let normalized = path.replace('\\', "/");
    let without_verbatim = normalized
        .strip_prefix("//?/")
        .or_else(|| normalized.strip_prefix("//./"))
        .unwrap_or(&normalized);
    format!("file:///{}", without_verbatim.trim_start_matches('/'))
}

/// Build a structured JSON summary for MCP agents presenting the current candidate.
pub fn session_presentation_summary(
    session: &ProductionSession,
    saved_html_path: Option<&str>,
) -> Result<Value, String> {
    let candidate = session.candidate().ok_or("No candidate available")?;
    let (producer, version) = session.profile();
    let resources: Vec<Value> = candidate
        .outputs()
        .iter()
        .map(|r| {
            let is_qac = r.kind.eq_ignore_ascii_case("qac") || r.key.ends_with(".qac");
            let mut item = json!({
                "key": r.key,
                "kind": r.kind,
                "modality": classify_modality(&r.kind, &r.key),
                "bytes": r.bytes.len(),
                "sha256": hex(&Sha256::digest(&r.bytes)),
            });
            if is_qac {
                item["qac_inspection"] = crate::present_qac::qac_summary_json(&r.bytes);
            }
            item
        })
        .collect();
    Ok(json!({
        "schema": PRESENTATION_SCHEMA,
        "producer": producer,
        "producer_version": version,
        "revision": candidate.revision(),
        "package_sha256": candidate.sha256(),
        "document_digest": hex(&session.document_digest()?),
        "presentation_html": saved_html_path,
        "antigravity_embed_hint": saved_html_path.map(|p| format!("<agent-embed src=\"{}\"></agent-embed>", file_url_from_fs_path(p))),
        "resources": resources,
    }))
}

#[cfg(test)]
mod tests {
    use super::file_url_from_fs_path;
    use crate::production::Resource;

    #[test]
    fn session_presentation_embed_hint_strips_windows_verbatim_prefix() {
        assert_eq!(file_url_from_fs_path(r"\\?\C:\Temp\note.html"), "file:///C:/Temp/note.html");
        assert_eq!(file_url_from_fs_path(r"\\.\C:\Temp\note.html"), "file:///C:/Temp/note.html");
        assert_eq!(file_url_from_fs_path("//?/C:/Temp/note.html"), "file:///C:/Temp/note.html");
        assert_eq!(file_url_from_fs_path("/tmp/note.html"), "file:///tmp/note.html");
        for path in [r"\\?\C:\Temp\note.html", r"\\.\C:\Temp\note.html", "//?/C:/Temp/note.html"] {
            let url = file_url_from_fs_path(path);
            assert!(url.starts_with("file:///C:"), "got {url}");
            assert!(!url.contains("?/C:"), "verbatim prefix leaked: {url}");
        }
    }

    fn sample_valid_10d() -> Vec<u8> {
        use qualia_core_db::render::assets::Mesh;
        use qualia_core_db::render::compile_10d::compile_mesh_to_10d;
        let mesh = Mesh {
            positions: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            triangles: vec![[0, 1, 2]],
            min: [0.0, 0.0, 0.0],
            max: [1.0, 1.0, 0.0],
        };
        compile_mesh_to_10d(&mesh).unwrap()
    }

    fn sample_valid_emf_v4() -> Vec<u8> {
        use qualia_core_db::q42::field_envelope::{EntryInput, EnvelopeInput, Family, Limits};
        let payload = vec![0u8; 128];
        let entry = EntryInput {
            mode: 0,
            revision: 1,
            flags: 0,
            stream_id: 0,
            frame_id: 1,
            start_tick: 0,
            time_count: 0,
            spatial_key: 0,
            record_count: 1,
            stride: 128,
            codec: 0,
            descriptor_id: 1,
            payload: &payload,
        };
        let input = EnvelopeInput {
            family: Family::EmfV4Candidate,
            flags: 0,
            metadata: &[],
            entries: &[entry],
        };
        let mut out = vec![0u8; input.encoded_len(Limits::default()).unwrap()];
        input.encode_into(&mut out, Limits::default()).unwrap();
        out
    }

    #[test]
    fn ten_d_card_html_contains_inline_mesh_b64_and_pairs_sibling_qac() {
        let bytes = sample_valid_10d();
        let emf_bytes = sample_valid_emf_v4();
        let qac_bytes = b"qac\0\0\0\0\0sample-qac-envelope";
        let mesh = Resource { key: "heart.10d".into(), kind: "10d".into(), bytes: bytes.clone() };
        let emf = Resource { key: "heart.emf".into(), kind: "emf".into(), bytes: emf_bytes.clone() };
        let qac = Resource { key: "heart.qac".into(), kind: "qac".into(), bytes: qac_bytes.to_vec() };
        let html = super::render_resource_card_in_set(1, &mesh, &[mesh.clone(), emf, qac]);
        let mesh_b64 = super::base64_encode(&bytes);
        let emf_b64 = super::base64_encode(&emf_bytes);
        let qac_b64 = super::base64_encode(qac_bytes);
        assert!(html.contains(&format!("data-mesh-b64=\"{mesh_b64}\"")));
        assert!(html.contains(&format!("data-emf-b64=\"{emf_b64}\"")));
        assert!(html.contains("data-qac-url=\"heart.qac\""));
        assert!(html.contains(&format!("data-qac-b64=\"{qac_b64}\"")));
        assert!(html.contains("data-mesh-url=\"heart.10d\""));
    }

    #[test]
    fn ten_d_card_fails_closed_when_sibling_emf_is_corrupt() {
        let bytes = sample_valid_10d();
        let corrupt_emf = b"Q42EMF64\x00fake-field".to_vec();
        let mesh = Resource { key: "heart.10d".into(), kind: "10d".into(), bytes };
        let emf = Resource { key: "heart.emf".into(), kind: "emf".into(), bytes: corrupt_emf };
        let html = super::render_resource_card_in_set(1, &mesh, &[mesh.clone(), emf]);
        assert!(html.contains("Container verification refused"));
        assert!(!html.contains("data-synth-portal=\"1\""));
        assert!(!html.contains("data-emf-b64="));
    }
}
