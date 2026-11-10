//! Full-capability snapshot and HTML studio renderer for Synth's 6 substantive pillars:
//! 1. Asset Production Workbench (`.vibe` -> `.hmc` / SVG / PNG / WAV / `.10d`)
//! 2. Categorical Ontology (12 Categories, 148 Morphisms, 444 `NQuin48` Triples)
//! 3. Unified QualiaDB 0.0.41 + Synth MCP Suite (94 Tools)
//! 4. Admitted Etalon-Vibe SDK Providers (11 Providers, 37 Functions, 5-Field Parity Probe)
//! 5. Living-World Biological/Anatomical Engine (33 Canonical Pins) & Cosmic Coordinates (OCS v2.2.0)
//! 6. Local LLM Model Vault (`C:\LLM_files` — `Qwen3.5-4B`, `Gemma-4-E4B`, `NVFP4`, `.p64`/`.gguf`) & Aether Tray Status.

use crate::tray_env::{discover_local_models, probe_webizen_desktop, DiscoveredModel};
use qualia_mcp::authoring::{AuthoringBackend, AuthoringTool};
use serde_json::{json, Value};
use std::fmt::Write as _;

#[derive(Clone, Debug)]
pub struct FullCapabilitiesSnapshot {
    pub mode_label: String,
    pub webizen_http_online: bool,
    pub webizen_ws_online: bool,
    pub vault_root: String,
    pub discovered_models: Vec<DiscoveredModel>,
    pub categories: Value,
    pub morphisms: Value,
    pub mcp_tools: Vec<AuthoringTool>,
    pub vibe_sdk: Value,
    pub living_world: Value,
    pub cosmic_earth: Value,
    pub cosmic_mars: Value,
}

impl FullCapabilitiesSnapshot {
    pub fn collect() -> Self {
        let (mode, webizen_http_online, webizen_ws_online) = probe_webizen_desktop();
        let (vault_path, discovered_models) = discover_local_models();
        let categories = crate::production::assets::ontology::categories_json();
        let morphisms = crate::production::assets::ontology::morphisms_json(None);
        let mut mcp = crate::mcp_full::StandaloneSynthMcp::default();
        let mcp_tools = mcp.tools();
        let vibe_sdk = crate::vibe_runtime::inspect().unwrap_or_else(|_| json!({}));
        let living_world = mcp
            .call("living_world_anatomy", json!({ "op": "catalog" }))
            .unwrap_or_else(|_| json!({}));
        let cosmic_earth = mcp
            .call(
                "cosmic_coordinates",
                json!({ "op": "celestial_profile", "body": "earth" }),
            )
            .unwrap_or_else(|_| json!({}));
        let cosmic_mars = mcp
            .call(
                "cosmic_coordinates",
                json!({ "op": "celestial_profile", "body": "mars" }),
            )
            .unwrap_or_else(|_| json!({}));

        Self {
            mode_label: mode.label().to_owned(),
            webizen_http_online,
            webizen_ws_online,
            vault_root: vault_path.to_string_lossy().into_owned(),
            discovered_models,
            categories,
            morphisms,
            mcp_tools,
            vibe_sdk,
            living_world,
            cosmic_earth,
            cosmic_mars,
        }
    }
}

fn esc(text: &str) -> String {
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

pub fn classify_mcp_domain(name: &str) -> &'static str {
    if name.starts_with("synth_") || name.starts_with("poet_") {
        "synth-authoring"
    } else if name.starts_with("vibe_") || name == "etalon_vibe_sdk" || name == "wgsl_forge" {
        "vibe-sdk"
    } else if name == "living_world_anatomy" || name == "cosmic_coordinates" {
        "living-cosmic"
    } else if name.contains("sparql")
        || name.contains("graph")
        || name.contains("ontology")
        || name.contains("shacl")
        || name.contains("modality")
        || name.contains("logic")
        || name == "codebase_audit"
        || name == "get_did_info"
    {
        "semantic-logic"
    } else {
        "science-sim"
    }
}

pub fn format_bytes_human(bytes: u64) -> String {
    const GIB: f64 = 1024.0 * 1024.0 * 1024.0;
    const MIB: f64 = 1024.0 * 1024.0;
    if bytes as f64 >= GIB {
        format!("{:.2} GiB", bytes as f64 / GIB)
    } else if bytes as f64 >= MIB {
        format!("{:.1} MiB", bytes as f64 / MIB)
    } else {
        format!("{bytes} B")
    }
}

/// Renders the HTML sections for Ontology (12 categories / 148 morphisms),
/// MCP Tool Suite (94 tools), Etalon-Vibe SDK (11 providers), Living-World & Cosmic,
/// and Local LLM Vault (`C:\LLM_files`).
pub fn render_capabilities_panels_html(snap: &FullCapabilitiesSnapshot) -> String {
    let mut out = String::with_capacity(64 * 1024);

    // 1. Ontology Panel
    let cat_list = snap.categories["categories"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    let morph_list = snap.morphisms["morphisms"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    let nquin_count = crate::production::assets::ontology::ONTOLOGY_NQUINS.len();

    let _ = write!(
        &mut out,
        r#"<section id="tab-ontology" class="studio-tab hidden space-y-4">
  <div class="bg-[var(--card)] border border-[var(--border)] rounded-xl p-4 space-y-3">
    <div class="flex flex-wrap items-center justify-between gap-2">
      <div>
        <div class="text-xs font-mono uppercase text-[var(--muted-foreground)]">hypermedia.categorical-ontology/v1 · {nquin_count} NQuin48 Triples</div>
        <h2 class="text-base font-semibold">12 Canonical Categories &amp; {morph_count} Admitted Morphisms</h2>
      </div>
      <input id="morph-search" oninput="filterMorphisms()" type="search" placeholder="Search morphisms, domains, codomains, IRIs..." class="px-3 py-1.5 rounded-lg text-xs font-mono bg-[var(--background)] border border-[var(--border)] text-[var(--foreground)] w-72" />
    </div>
    <div class="flex flex-wrap gap-1.5 pt-1">
      <button onclick="setMorphCategory('all')" class="morph-cat-btn px-2.5 py-1 rounded text-xs font-mono border border-emerald-500/60 bg-emerald-950/40" data-cat="all">All ({morph_count})</button>"#,
        morph_count = morph_list.len(),
        nquin_count = nquin_count
    );
    for cat in &cat_list {
        let cid = esc(cat["id"].as_str().unwrap_or(""));
        let cnt = cat["admitted_morphism_count"].as_u64().unwrap_or(0);
        let _ = write!(
            &mut out,
            r#"<button onclick="setMorphCategory('{cid}')" class="morph-cat-btn px-2.5 py-1 rounded text-xs font-mono border border-[var(--border)] bg-[var(--background)]" data-cat="{cid}">{cid} ({cnt})</button>"#
        );
    }
    out.push_str("</div></div><div class=\"grid grid-cols-1 md:grid-cols-3 gap-3\">");
    for cat in &cat_list {
        let cid = esc(cat["id"].as_str().unwrap_or(""));
        let obj = esc(cat["object_space"].as_str().unwrap_or(""));
        let purpose = esc(cat["morphism_purpose"].as_str().unwrap_or(""));
        let cnt = cat["admitted_morphism_count"].as_u64().unwrap_or(0);
        let _ = write!(
            &mut out,
            r#"<div class="bg-[var(--card)] border border-[var(--border)] rounded-xl p-3.5 space-y-1.5">
  <div class="flex items-center justify-between">
    <span class="font-mono text-xs font-bold uppercase text-emerald-400">{cid}</span>
    <span class="text-xs font-mono text-[var(--muted-foreground)]">{cnt} morphisms</span>
  </div>
  <div class="text-xs font-mono text-sky-300">{obj}</div>
  <p class="text-xs text-[var(--muted-foreground)] leading-relaxed">{purpose}</p>
</div>"#
        );
    }
    out.push_str("</div><div id=\"morphism-list\" class=\"grid grid-cols-1 lg:grid-cols-2 gap-3\">");
    for m in &morph_list {
        let id = esc(m["id"].as_str().unwrap_or(""));
        let cat = esc(m["category"].as_str().unwrap_or(""));
        let summary = esc(m["summary"].as_str().unwrap_or(""));
        let args = esc(m["arguments"].as_str().unwrap_or(""));
        let domain = esc(m["agentic_utility"]["domain"].as_str().unwrap_or(""));
        let codomain = esc(m["agentic_utility"]["codomain"].as_str().unwrap_or(""));
        let teleology = esc(m["agentic_utility"]["teleology"].as_str().unwrap_or(""));
        let kind = esc(m["agentic_utility"]["morphism_kind"].as_str().unwrap_or(""));
        let digest = esc(
            m["ontological_contract"]["contract_digest_sha256"]
                .as_str()
                .unwrap_or(""),
        );
        let short_digest = if digest.len() >= 16 { &digest[..16] } else { &digest };
        let _ = write!(
            &mut out,
            r#"<article data-cat="{cat}" class="morph-card bg-[var(--card)] border border-[var(--border)] rounded-xl p-3.5 space-y-2">
  <div class="flex flex-wrap items-center justify-between gap-2">
    <span class="font-mono text-xs font-semibold text-emerald-300">{id}</span>
    <div class="flex items-center gap-1.5 text-[11px] font-mono">
      <span class="px-2 py-0.5 rounded bg-[var(--background)] border border-[var(--border)]">{cat}</span>
      <span class="px-2 py-0.5 rounded bg-[var(--background)] border border-[var(--border)] text-sky-300">{kind}</span>
    </div>
  </div>
  <div class="text-xs font-medium text-[var(--foreground)]">{summary}</div>
  <div class="text-[11px] font-mono bg-[var(--background)] border border-[var(--border)] rounded p-2 space-y-1">
    <div><span class="text-[var(--muted-foreground)]">Domain:</span> <span class="text-amber-300">{domain}</span></div>
    <div><span class="text-[var(--muted-foreground)]">Codomain:</span> <span class="text-emerald-300">{codomain}</span></div>
    <div><span class="text-[var(--muted-foreground)]">Args:</span> <span>{args}</span></div>
  </div>
  <p class="text-xs text-[var(--muted-foreground)]">{teleology}</p>
  <div class="text-[10px] font-mono text-[var(--muted-foreground)]">NQuin48 Contract: {short_digest}… · BitExact · PURE_COMPUTE</div>
</article>"#
        );
    }
    out.push_str("</div></section>");

    // 2. MCP Suite Panel (94 tools + Live Executor)
    let _ = write!(
        &mut out,
        r#"<section id="tab-mcp" class="studio-tab hidden space-y-4">
  <div class="bg-[var(--card)] border border-[var(--border)] rounded-xl p-4 space-y-3">
    <div class="flex flex-wrap items-center justify-between gap-2">
      <div>
        <div class="text-xs font-mono uppercase text-[var(--muted-foreground)]">synth.comprehensive-mcp/v1 · JSON-RPC 2.0 (2025-11-25) · FastMCP Identifier-Safe</div>
        <h2 class="text-base font-semibold">{tool_count} Unified QualiaDB 0.0.41 + Synth MCP Tools</h2>
      </div>
      <input id="mcp-search" oninput="filterMcpTools()" type="search" placeholder="Search 94 MCP tools..." class="px-3 py-1.5 rounded-lg text-xs font-mono bg-[var(--background)] border border-[var(--border)] text-[var(--foreground)] w-64" />
    </div>
    <div class="grid grid-cols-1 lg:grid-cols-3 gap-2 pt-2 border-t border-[var(--border)]">
      <div>
        <label class="block text-[11px] font-mono text-[var(--muted-foreground)] mb-1">Tool Name (click any card below to populate)</label>
        <input id="mcp-exec-name" type="text" value="etalon_vibe_sdk" class="w-full px-2.5 py-1.5 rounded text-xs font-mono bg-[var(--background)] border border-[var(--border)] text-emerald-300" />
      </div>
      <div>
        <label class="block text-[11px] font-mono text-[var(--muted-foreground)] mb-1">JSON Arguments</label>
        <input id="mcp-exec-args" type="text" value="{{}}" class="w-full px-2.5 py-1.5 rounded text-xs font-mono bg-[var(--background)] border border-[var(--border)] text-[var(--foreground)]" />
      </div>
      <div class="flex items-end gap-2">
        <button onclick="invokeMcpToolLive()" class="px-3.5 py-1.5 rounded text-xs font-mono font-semibold bg-emerald-600/30 border border-emerald-500 text-emerald-200 hover:bg-emerald-600/50">Invoke Live MCP Tool</button>
      </div>
    </div>
    <pre id="mcp-exec-output" class="hidden max-h-56 overflow-auto p-3 rounded-lg bg-[var(--background)] border border-[var(--border)] text-xs font-mono text-emerald-300"></pre>
    <div class="flex flex-wrap gap-1.5">
      <button onclick="setMcpDomain('all')" class="mcp-dom-btn px-2.5 py-1 rounded text-xs font-mono border border-emerald-500/60 bg-emerald-950/40" data-dom="all">All ({tool_count})</button>
      <button onclick="setMcpDomain('synth-authoring')" class="mcp-dom-btn px-2.5 py-1 rounded text-xs font-mono border border-[var(--border)] bg-[var(--background)]" data-dom="synth-authoring">Synth &amp; Studio Authoring</button>
      <button onclick="setMcpDomain('vibe-sdk')" class="mcp-dom-btn px-2.5 py-1 rounded text-xs font-mono border border-[var(--border)] bg-[var(--background)]" data-dom="vibe-sdk">VibeScript &amp; WGSL Forge</button>
      <button onclick="setMcpDomain('living-cosmic')" class="mcp-dom-btn px-2.5 py-1 rounded text-xs font-mono border border-[var(--border)] bg-[var(--background)]" data-dom="living-cosmic">Living-World &amp; Cosmic (0.0.41)</button>
      <button onclick="setMcpDomain('semantic-logic')" class="mcp-dom-btn px-2.5 py-1 rounded text-xs font-mono border border-[var(--border)] bg-[var(--background)]" data-dom="semantic-logic">SPARQL, SHACL &amp; Modalities</button>
      <button onclick="setMcpDomain('science-sim')" class="mcp-dom-btn px-2.5 py-1 rounded text-xs font-mono border border-[var(--border)] bg-[var(--background)]" data-dom="science-sim">Sim, Geo, Vision, Audio &amp; Quantum</button>
    </div>
  </div>
  <div id="mcp-tool-list" class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-3">"#,
        tool_count = snap.mcp_tools.len()
    );
    for tool in &snap.mcp_tools {
        let dom = classify_mcp_domain(&tool.name);
        let tname = esc(&tool.name);
        let tdesc = esc(&tool.description);
        let props: Vec<String> = tool
            .input_schema
            .get("properties")
            .and_then(Value::as_object)
            .map(|m| m.keys().map(|k| esc(k)).collect())
            .unwrap_or_default();
        let props_str = if props.is_empty() {
            "none".to_owned()
        } else {
            props.join(", ")
        };
        let _ = write!(
            &mut out,
            r#"<div data-dom="{dom}" onclick="selectMcpTool('{tname}')" class="mcp-card cursor-pointer hover:border-emerald-500/60 bg-[var(--card)] border border-[var(--border)] rounded-xl p-3.5 space-y-2">
  <div class="flex items-center justify-between gap-2">
    <span class="font-mono text-xs font-bold text-emerald-300">{tname}</span>
    <span class="px-2 py-0.5 rounded text-[10px] font-mono bg-[var(--background)] border border-[var(--border)] text-sky-300">{dom}</span>
  </div>
  <p class="text-xs text-[var(--muted-foreground)] leading-relaxed">{tdesc}</p>
  <div class="text-[11px] font-mono text-[var(--muted-foreground)] bg-[var(--background)] border border-[var(--border)] rounded px-2 py-1">params: <span class="text-[var(--foreground)]">{props_str}</span></div>
</div>"#
        );
    }
    out.push_str("</div></section>");

    // 3. Etalon-Vibe SDK & Living-World / Cosmic Panel
    let providers = snap.vibe_sdk["providers"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    let pins = snap.living_world["pins"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    let _ = write!(
        &mut out,
        r#"<section id="tab-sdk-living" class="studio-tab hidden space-y-4">
  <div class="grid grid-cols-1 lg:grid-cols-2 gap-4">
    <div class="bg-[var(--card)] border border-[var(--border)] rounded-xl p-4 space-y-3">
      <div class="text-xs font-mono uppercase text-emerald-400">Etalon-Vibe SDK · 5-Field NQuin48 Parity Verified</div>
      <h2 class="text-base font-semibold">{prov_count} Admitted Native Extension Providers</h2>
      <div class="space-y-2 max-h-[520px] overflow-y-auto pr-1">"#,
        prov_count = providers.len()
    );
    for p in &providers {
        let alias = esc(p["alias"].as_str().unwrap_or(""));
        let iri = esc(p["iri"].as_str().unwrap_or(""));
        let quins = p["ontology_records"].as_u64().unwrap_or(0);
        let fns = p["functions"].as_array().cloned().unwrap_or_default();
        let fn_names: Vec<String> = fns
            .iter()
            .filter_map(|f| f["path"].as_str().map(esc))
            .collect();
        let _ = write!(
            &mut out,
            r#"<div class="bg-[var(--background)] border border-[var(--border)] rounded-lg p-3 space-y-1">
  <div class="flex items-center justify-between text-xs font-mono">
    <span class="font-bold text-emerald-300">vibe:0.3/{alias}</span>
    <span class="text-[var(--muted-foreground)]">{quins} NQuin48 · {fn_len} fns</span>
  </div>
  <div class="text-[11px] font-mono text-sky-300 truncate">{iri}</div>
  <div class="text-[11px] font-mono text-[var(--muted-foreground)]">{joined}</div>
</div>"#,
            fn_len = fn_names.len(),
            joined = fn_names.join(" · ")
        );
    }
    let _ = write!(
        &mut out,
        r#"</div></div>
    <div class="bg-[var(--card)] border border-[var(--border)] rounded-xl p-4 space-y-3">
      <div class="text-xs font-mono uppercase text-sky-400">Living-World Biological/Anatomical Engine &amp; OCS v2.2.0</div>
      <h2 class="text-base font-semibold">{pin_count} Canonical Anatomical &amp; Botanical Callout Pins</h2>
      <div class="grid grid-cols-2 gap-2 text-xs font-mono bg-[var(--background)] border border-[var(--border)] rounded-lg p-2.5">
        <div>Golden Angle: <span class="text-emerald-300">2.399963 rad</span></div>
        <div>Eucalyptus Pipe Exp: <span class="text-emerald-300">2.49</span></div>
        <div>Vertebrate Max Joints: <span class="text-sky-300">64 (Cyclic RMF)</span></div>
        <div>Celestial Frames: <span class="text-amber-300">Earth &amp; Mars (J2000)</span></div>
      </div>
      <div class="space-y-2 max-h-[430px] overflow-y-auto pr-1">"#,
        pin_count = pins.len()
    );
    for pin in &pins {
        let pid = esc(pin["pin_id"].as_str().unwrap_or(""));
        let curie = esc(pin["curie"].as_str().unwrap_or(""));
        let term = esc(pin["scientific_term"].as_str().unwrap_or(""));
        let scale = pin["scale"].as_u64().unwrap_or(0);
        let _ = write!(
            &mut out,
            r#"<div class="bg-[var(--background)] border border-[var(--border)] rounded-lg p-2.5 flex items-center justify-between gap-2 text-xs font-mono">
  <div>
    <div class="text-emerald-300 font-semibold">{pid} <span class="text-[var(--foreground)] font-sans font-normal">— {term}</span></div>
    <div class="text-[11px] text-[var(--muted-foreground)]">CURIE: {curie}</div>
  </div>
  <span class="px-2 py-0.5 rounded bg-[var(--card)] border border-[var(--border)] text-sky-300">Scale {scale}</span>
</div>"#
        );
    }
    out.push_str("</div></div></div></section>");

    // 4. Local LLM Vault & Resident Aether Tray Panel
    let _ = write!(
        &mut out,
        r#"<section id="tab-llm-vault" class="studio-tab hidden space-y-4">
  <div class="bg-[var(--card)] border border-[var(--border)] rounded-xl p-4 space-y-3">
    <div class="flex flex-wrap items-center justify-between gap-2">
      <div>
        <div class="text-xs font-mono uppercase text-[var(--muted-foreground)]">Aether Resident Host &amp; Local Model Lifecycle</div>
        <h2 class="text-base font-semibold">Vault Root: <span class="font-mono text-emerald-300">{vault}</span> ({model_count} Discovered Artifacts)</h2>
      </div>
      <span class="px-3 py-1 rounded-md text-xs font-mono bg-emerald-950/50 border border-emerald-500/50 text-emerald-300">{mode}</span>
    </div>
    <div class="grid grid-cols-1 md:grid-cols-2 gap-3 pt-2">"#,
        vault = esc(&snap.vault_root),
        model_count = snap.discovered_models.len(),
        mode = esc(&snap.mode_label)
    );
    for m in &snap.discovered_models {
        let mname = esc(&m.name);
        let mfam = esc(&m.family);
        let mfmt = esc(&m.format);
        let mpath = esc(&m.path);
        let msize = format_bytes_human(m.size_bytes);
        let _ = write!(
            &mut out,
            r#"<div class="bg-[var(--background)] border border-[var(--border)] rounded-xl p-3.5 space-y-1.5">
  <div class="flex items-center justify-between">
    <span class="font-mono text-xs font-bold text-emerald-300">{mname}</span>
    <span class="text-xs font-mono text-amber-300">{msize}</span>
  </div>
  <div class="flex items-center gap-2 text-[11px] font-mono">
    <span class="px-2 py-0.5 rounded bg-[var(--card)] border border-[var(--border)] uppercase">{mfam}</span>
    <span class="px-2 py-0.5 rounded bg-[var(--card)] border border-[var(--border)] uppercase text-sky-300">{mfmt}</span>
  </div>
  <div class="text-[11px] font-mono text-[var(--muted-foreground)] truncate" title="{mpath}">{mpath}</div>
</div>"#
        );
    }
    out.push_str("</div></div></section>");
    out
}
