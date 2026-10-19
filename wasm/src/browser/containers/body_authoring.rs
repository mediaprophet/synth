//! Copyright (c) 2026 Timothy Charles Holborn. All rights reserved.
//! Native authoring studio container bodies harmonized into Poet's UI.

use crate::tool_chest::core::registry::SeedContainer;
use wasm_bindgen::JsCast;
use web_sys::{Document, Element, HtmlElement};

pub(super) fn try_fill(document: &Document, container: &SeedContainer, body: &Element) -> bool {
    let view = match container.container_type.as_str() {
        "grid_studio" => build_grid_studio_view(document),
        "word_studio" => build_word_studio_view(document),
        "deck_studio" => build_deck_studio_view(document),
        "pdf_studio" => build_pdf_studio_view(document),
        "vector_studio" => build_vector_studio_view(document),
        "photo_studio" => build_photo_studio_view(document),
        "light_studio" => build_light_studio_view(document),
        "cad_studio" => build_cad_studio_view(document),
        "sound_studio" => build_sound_studio_view(document),
        "film_studio" => build_film_studio_view(document),
        "art_studio" => build_art_studio_view(document),
        _ => return false,
    };
    body.append_child(&view).unwrap();
    true
}

fn studio_shell(document: &Document, studio_class: &str, title: &str, kicker: &str) -> (Element, Element) {
    let root = document.create_element("div").unwrap();
    root.set_class_name(&format!("studio-container-body {}", studio_class));
    let r_el: HtmlElement = root.clone().dyn_into().unwrap();
    r_el.style().set_css_text(
        "display: flex; flex-direction: column; width: 100%; height: 100%; \
         background: var(--surface-bg, #0d1117); color: var(--text-primary, #e6edf3); \
         font-family: var(--font-sans, system-ui); overflow: hidden; position: relative;",
    );

    let header = document.create_element("div").unwrap();
    header.set_class_name("studio-sub-header");
    let h_el: HtmlElement = header.clone().dyn_into().unwrap();
    h_el.style().set_css_text(
        "display: flex; align-items: center; justify-content: space-between; padding: 4px 8px; \
         background: var(--surface-panel-elevated, #161b22); border-bottom: 1px solid var(--border-subtle, #30363d); \
         font-size: 11px;",
    );
    header.set_inner_html(&format!(
        "<div style=\"display:flex; align-items:center; gap:6px;\">\
           <span style=\"font-weight:700; color:var(--accent-cyan, #58a6ff); font-family:var(--font-mono);\">{}</span>\
           <span style=\"color:var(--text-muted, #8b949e); font-size:10px;\">{}</span>\
         </div>\
         <div style=\"display:flex; gap:4px; font-size:10px;\">\
           <span style=\"padding:1px 5px; border-radius:3px; background:rgba(88,166,255,0.15); color:var(--accent-cyan, #58a6ff);\">MIT Harms-Free</span>\
           <span style=\"padding:1px 5px; border-radius:3px; background:rgba(35,134,54,0.15); color:#3fb950;\">Poet Native</span>\
         </div>",
        title, kicker
    ));
    root.append_child(&header).unwrap();

    let content = document.create_element("div").unwrap();
    content.set_class_name("studio-workspace-content");
    let c_el: HtmlElement = content.clone().dyn_into().unwrap();
    c_el.style().set_css_text("flex: 1; display: flex; flex-direction: column; overflow: hidden; position: relative;");
    root.append_child(&content).unwrap();

    (root, content)
}

fn build_grid_studio_view(document: &Document) -> Element {
    let (root, content) = studio_shell(document, "grid-studio-view", "Grid Studio", "Formula & High-Perf Workbook");
    content.set_inner_html(r##"
      <div style="display:flex; align-items:center; gap:6px; padding:4px 8px; background:var(--surface-panel); border-bottom:1px solid var(--border-subtle); font-size:11px;">
        <span style="font-weight:600; color:var(--text-muted); font-family:var(--font-mono);">A1</span>
        <span style="color:var(--accent-cyan); font-weight:700;">fx</span>
        <input type="text" value="=SUM(B2:B12) * 1.15" style="flex:1; background:var(--surface-bg); border:1px solid var(--border-subtle); border-radius:3px; padding:2px 6px; color:var(--text-primary); font-family:var(--font-mono); font-size:11px;" />
      </div>
      <div style="flex:1; overflow:auto; background:var(--surface-bg);">
        <table style="width:100%; border-collapse:collapse; font-family:var(--font-mono); font-size:11px; text-align:left;">
          <thead>
            <tr style="background:var(--surface-panel); color:var(--text-muted); border-bottom:1px solid var(--border-subtle);">
              <th style="width:36px; padding:3px; border-right:1px solid var(--border-subtle); text-align:center;">#</th>
              <th style="padding:3px 8px; border-right:1px solid var(--border-subtle);">A</th>
              <th style="padding:3px 8px; border-right:1px solid var(--border-subtle);">B</th>
              <th style="padding:3px 8px; border-right:1px solid var(--border-subtle);">C</th>
              <th style="padding:3px 8px;">D</th>
            </tr>
          </thead>
          <tbody>
            <tr style="border-bottom:1px solid var(--border-subtle);"><td style="padding:3px; text-align:center; color:var(--text-muted); background:var(--surface-panel);">1</td><td style="padding:3px 8px; font-weight:600;">Metric</td><td style="padding:3px 8px;">Q1 Value</td><td style="padding:3px 8px;">Q2 Value</td><td style="padding:3px 8px;">Delta</td></tr>
            <tr style="border-bottom:1px solid var(--border-subtle);"><td style="padding:3px; text-align:center; color:var(--text-muted); background:var(--surface-panel);">2</td><td style="padding:3px 8px;">Revenue ($k)</td><td style="padding:3px 8px;">1,240.50</td><td style="padding:3px 8px;">1,426.80</td><td style="padding:3px 8px; color:#3fb950;">+15.0%</td></tr>
            <tr style="border-bottom:1px solid var(--border-subtle);"><td style="padding:3px; text-align:center; color:var(--text-muted); background:var(--surface-panel);">3</td><td style="padding:3px 8px;">Operational Cost</td><td style="padding:3px 8px;">812.30</td><td style="padding:3px 8px;">820.10</td><td style="padding:3px 8px; color:#f85149;">+0.9%</td></tr>
            <tr style="border-bottom:1px solid var(--border-subtle);"><td style="padding:3px; text-align:center; color:var(--text-muted); background:var(--surface-panel);">4</td><td style="padding:3px 8px;">Net Margin</td><td style="padding:3px 8px;">428.20</td><td style="padding:3px 8px;">606.70</td><td style="padding:3px 8px; color:#3fb950;">+41.7%</td></tr>
          </tbody>
        </table>
      </div>
      <div style="display:flex; justify-content:space-between; padding:3px 8px; background:var(--surface-panel); border-top:1px solid var(--border-subtle); font-size:10px; color:var(--text-muted);">
        <span>Sheet1 | 4 rows × 4 cols</span><span>Calc engine: Ready (Qualia zero-alloc)</span>
      </div>
    "##);
    root
}

fn build_word_studio_view(document: &Document) -> Element {
    let (root, content) = studio_shell(document, "word-studio-view", "Document Studio", "Paginated Flow Document & CML");
    content.set_inner_html(r##"
      <div style="flex:1; overflow:auto; padding:16px; display:flex; justify-content:center; background:rgba(0,0,0,0.2);">
        <div style="width:100%; max-width:540px; min-height:360px; background:var(--surface-panel); border:1px solid var(--border-subtle); border-radius:4px; padding:24px; box-shadow:0 4px 12px rgba(0,0,0,0.3);">
          <div style="font-size:9px; text-transform:uppercase; letter-spacing:1px; color:var(--accent-cyan); margin-bottom:8px;">Structured Document &middot; CML Semantic Anchor</div>
          <h1 style="font-size:18px; margin:0 0 12px 0; color:var(--text-primary); font-weight:700;">Living Document Specification</h1>
          <p style="font-size:12px; line-height:1.6; color:var(--text-secondary); margin-bottom:12px;">This document unifies high-assurance document processing with live RDF ontological anchoring. Paragraph revisions maintain deterministic Lamport timestamp ties.</p>
          <div style="padding:8px 12px; background:rgba(88,166,255,0.08); border-left:3px solid var(--accent-cyan); border-radius:2px; font-size:11px; color:var(--text-primary); margin-bottom:12px;">
            <strong>CML Anchor:</strong> <code>did:q42:doc#sec-04</code> &mdash; Verified against QualiaDB Knowledge Graph.
          </div>
          <p style="font-size:12px; line-height:1.6; color:var(--text-secondary);">Paginated flow recalculates margins and footnote citations automatically through the background typography layout engine.</p>
        </div>
      </div>
      <div style="display:flex; justify-content:space-between; padding:3px 8px; background:var(--surface-panel); border-top:1px solid var(--border-subtle); font-size:10px; color:var(--text-muted);">
        <span>Page 1 of 1 &middot; 78 words</span><span>Encoding: UTF-8 &middot; DOCX Sync</span>
      </div>
    "##);
    root
}

fn build_deck_studio_view(document: &Document) -> Element {
    let (root, content) = studio_shell(document, "deck-studio-view", "Presentation Studio", "Slide Deck & Presenter View");
    content.set_inner_html(r##"
      <div style="flex:1; display:flex; overflow:hidden;">
        <div style="width:90px; border-right:1px solid var(--border-subtle); background:var(--surface-panel); padding:8px 4px; display:flex; flex-direction:column; gap:6px;">
          <div style="height:50px; background:var(--surface-bg); border:2px solid var(--accent-cyan); border-radius:3px; display:flex; align-items:center; justify-content:center; font-size:10px; font-weight:700;">1</div>
          <div style="height:50px; background:var(--surface-bg); border:1px solid var(--border-subtle); border-radius:3px; display:flex; align-items:center; justify-content:center; font-size:10px; color:var(--text-muted);">2</div>
          <div style="height:50px; background:var(--surface-bg); border:1px solid var(--border-subtle); border-radius:3px; display:flex; align-items:center; justify-content:center; font-size:10px; color:var(--text-muted);">3</div>
        </div>
        <div style="flex:1; display:flex; align-items:center; justify-content:center; padding:16px; background:rgba(0,0,0,0.3);">
          <div style="aspect-ratio:16/9; width:90%; max-width:440px; background:linear-gradient(135deg, #161b22, #0d1117); border:1px solid var(--border-subtle); border-radius:6px; padding:20px; display:flex; flex-direction:column; justify-content:center;">
            <div style="font-size:10px; color:var(--accent-cyan); text-transform:uppercase; letter-spacing:1px; margin-bottom:6px;">Executive Briefing</div>
            <div style="font-size:16px; font-weight:700; color:#fff; margin-bottom:8px;">Decentralized Creative Suite</div>
            <div style="font-size:11px; color:var(--text-secondary); line-height:1.4;">Zero-heap hot paths &middot; Sovereign agent supervision &middot; Open standards</div>
          </div>
        </div>
      </div>
      <div style="display:flex; justify-content:space-between; padding:3px 8px; background:var(--surface-panel); border-top:1px solid var(--border-subtle); font-size:10px; color:var(--text-muted);">
        <span>Slide 1 of 3 (16:9)</span><span>Presenter Notes: Active &middot; PPTX roundtrip</span>
      </div>
    "##);
    root
}

fn build_pdf_studio_view(document: &Document) -> Element {
    let (root, content) = studio_shell(document, "pdf-studio-view", "PDF Studio", "Document Intelligence & Forms");
    content.set_inner_html(r##"
      <div style="flex:1; display:flex; flex-direction:column; overflow:auto; padding:16px; align-items:center; background:rgba(0,0,0,0.2);">
        <div style="width:100%; max-width:420px; min-height:280px; background:var(--surface-panel); border:1px solid var(--border-subtle); border-radius:4px; padding:20px; position:relative;">
          <div style="display:flex; justify-content:space-between; border-bottom:1px solid var(--border-subtle); padding-bottom:8px; margin-bottom:12px;">
            <span style="font-weight:700; font-size:13px;">NON-DISCLOSURE AGREEMENT</span>
            <span style="font-size:10px; color:var(--text-muted);">REV-2026-B</span>
          </div>
          <div style="font-size:11px; color:var(--text-secondary); line-height:1.5; margin-bottom:16px;">This covenant binds the undersigned parties in mutual confidence under topological smart contract terms.</div>
          <div style="border:1px dashed var(--accent-cyan); border-radius:4px; padding:10px; background:rgba(88,166,255,0.05); display:flex; justify-content:space-between; align-items:center;">
            <div>
              <div style="font-size:10px; font-weight:600; color:var(--accent-cyan);">Ed25519 Cryptographic Signature</div>
              <div style="font-size:9px; color:var(--text-muted); font-family:var(--font-mono);">q42:sig:8f4c2e01a9bd...</div>
            </div>
            <span style="color:#3fb950; font-size:12px;">&#10003; Valid</span>
          </div>
        </div>
      </div>
      <div style="display:flex; justify-content:space-between; padding:3px 8px; background:var(--surface-panel); border-top:1px solid var(--border-subtle); font-size:10px; color:var(--text-muted);">
        <span>Page 1/1 &middot; AcroForms 2.0</span><span>Redaction Engine: Armored</span>
      </div>
    "##);
    root
}

fn build_vector_studio_view(document: &Document) -> Element {
    let (root, content) = studio_shell(document, "vector-studio-view", "Vector Studio", "2D Vector & Boolean Engine");
    content.set_inner_html(r##"
      <div style="flex:1; position:relative; overflow:hidden; background:radial-gradient(circle, #1a202c 10%, #0d1117 90%); display:flex; align-items:center; justify-content:center;">
        <svg width="240" height="180" viewBox="0 0 240 180" style="border:1px dashed var(--border-subtle); background:rgba(0,0,0,0.2);">
          <circle cx="90" cy="90" r="50" fill="rgba(88,166,255,0.4)" stroke="#58a6ff" stroke-width="2" />
          <rect x="90" y="50" width="80" height="80" rx="6" fill="rgba(63,185,80,0.3)" stroke="#3fb950" stroke-width="2" />
          <path d="M 90 90 L 170 50" stroke="#f0883e" stroke-width="1.5" stroke-dasharray="3,3" />
        </svg>
        <div style="position:absolute; bottom:8px; left:8px; display:flex; gap:4px;">
          <span style="font-size:10px; padding:2px 6px; background:var(--surface-panel); border:1px solid var(--border-subtle); border-radius:3px;">Paths: 2</span>
          <span style="font-size:10px; padding:2px 6px; background:var(--surface-panel); border:1px solid var(--border-subtle); border-radius:3px;">Nodes: 8</span>
        </div>
      </div>
      <div style="display:flex; justify-content:space-between; padding:3px 8px; background:var(--surface-panel); border-top:1px solid var(--border-subtle); font-size:10px; color:var(--text-muted);">
        <span>SVG / PDF Vector Core</span><span>Boolean Mode: Union &middot; Snap: On</span>
      </div>
    "##);
    root
}

fn build_photo_studio_view(document: &Document) -> Element {
    let (root, content) = studio_shell(document, "photo-studio-view", "Photo Studio", "Tiled Multi-Channel Raster");
    content.set_inner_html(r##"
      <div style="flex:1; display:flex; overflow:hidden;">
        <div style="flex:1; display:flex; align-items:center; justify-content:center; background:#080b0f; position:relative;">
          <div style="width:200px; height:150px; background:linear-gradient(45deg, #1f2937, #374151); border:1px solid var(--border-subtle); display:flex; align-items:center; justify-content:center; color:var(--text-muted); font-size:11px;">
            16-Bit Tiled Viewport
          </div>
        </div>
        <div style="width:110px; border-left:1px solid var(--border-subtle); background:var(--surface-panel); padding:6px; display:flex; flex-direction:column; gap:6px;">
          <div style="font-size:10px; font-weight:700; color:var(--text-muted); text-transform:uppercase;">Layers</div>
          <div style="padding:4px; background:rgba(88,166,255,0.15); border:1px solid var(--accent-cyan); border-radius:3px; font-size:10px;">Layer 1 (Normal)</div>
          <div style="padding:4px; background:var(--surface-bg); border:1px solid var(--border-subtle); border-radius:3px; font-size:10px; color:var(--text-muted);">Adjust: Levels</div>
          <div style="padding:4px; background:var(--surface-bg); border:1px solid var(--border-subtle); border-radius:3px; font-size:10px; color:var(--text-muted);">Background</div>
        </div>
      </div>
      <div style="display:flex; justify-content:space-between; padding:3px 8px; background:var(--surface-panel); border-top:1px solid var(--border-subtle); font-size:10px; color:var(--text-muted);">
        <span>RGBA 16-Bit &middot; 3840 &times; 2160</span><span>PSD / PSB Non-destructive</span>
      </div>
    "##);
    root
}

fn build_light_studio_view(document: &Document) -> Element {
    let (root, content) = studio_shell(document, "light-studio-view", "Light Studio", "Non-Destructive RAW Photography");
    content.set_inner_html(r##"
      <div style="flex:1; display:flex; overflow:hidden;">
        <div style="flex:1; display:flex; align-items:center; justify-content:center; background:#05070a;">
          <div style="width:220px; height:140px; background:#111827; border:1px solid var(--border-subtle); display:flex; flex-direction:column; align-items:center; justify-content:center;">
            <span style="font-size:11px; color:var(--accent-cyan);">&#9650; RAW Sensor Frame</span>
            <span style="font-size:9px; color:var(--text-muted); font-family:var(--font-mono);">ISO 100 &middot; 50mm &middot; f/1.8 &middot; 1/250s</span>
          </div>
        </div>
        <div style="width:120px; border-left:1px solid var(--border-subtle); background:var(--surface-panel); padding:6px; display:flex; flex-direction:column; gap:6px; font-size:10px;">
          <div style="font-weight:700; color:var(--text-muted); text-transform:uppercase;">Develop</div>
          <div><span>Exposure</span><div style="height:3px; background:var(--accent-cyan); border-radius:2px; margin-top:2px;"></div></div>
          <div><span>Temperature</span><div style="height:3px; background:#f0883e; border-radius:2px; margin-top:2px;"></div></div>
          <div><span>Highlights</span><div style="height:3px; background:var(--border-subtle); border-radius:2px; margin-top:2px;"></div></div>
          <div><span>Shadows</span><div style="height:3px; background:var(--border-subtle); border-radius:2px; margin-top:2px;"></div></div>
        </div>
      </div>
      <div style="display:flex; justify-content:space-between; padding:3px 8px; background:var(--surface-panel); border-top:1px solid var(--border-subtle); font-size:10px; color:var(--text-muted);">
        <span>14-Bit Bayer Pattern &middot; DNG/CR3</span><span>Profile: ProPhoto RGB</span>
      </div>
    "##);
    root
}

fn build_cad_studio_view(document: &Document) -> Element {
    let (root, content) = studio_shell(document, "cad-studio-view", "CAD Studio", "Precision 2D Engineering CAD");
    content.set_inner_html(r##"
      <div style="flex:1; position:relative; overflow:hidden; background:#0d1117; display:flex; align-items:center; justify-content:center;">
        <svg width="260" height="180" viewBox="0 0 260 180" style="border:1px solid var(--border-subtle);">
          <defs><pattern id="grid" width="20" height="20" patternUnits="userSpaceOnUse"><path d="M 20 0 L 0 0 0 20" fill="none" stroke="rgba(255,255,255,0.05)" stroke-width="1"/></pattern></defs>
          <rect width="260" height="180" fill="url(#grid)" />
          <line x1="40" y1="120" x2="200" y2="120" stroke="#58a6ff" stroke-width="2" />
          <line x1="40" y1="120" x2="40" y2="60" stroke="#58a6ff" stroke-width="2" />
          <circle cx="200" cy="120" r="3" fill="#3fb950" />
          <circle cx="40" cy="60" r="3" fill="#3fb950" />
          <text x="100" y="112" fill="#58a6ff" font-size="10" font-family="monospace">160.00 mm</text>
        </svg>
        <div style="position:absolute; top:6px; left:6px; font-family:var(--font-mono); font-size:10px; color:var(--text-muted);">
          X: 142.500 mm &middot; Y: 88.125 mm
        </div>
      </div>
      <div style="display:flex; justify-content:space-between; padding:3px 8px; background:var(--surface-panel); border-top:1px solid var(--border-subtle); font-size:10px; color:var(--text-muted);">
        <span>Snap: Endpoint/Midpoint &middot; f64 precision</span><span>DXF / DWG Interop</span>
      </div>
    "##);
    root
}

fn build_sound_studio_view(document: &Document) -> Element {
    let (root, content) = studio_shell(document, "sound-studio-view", "Audio Studio", "Multitrack DAW & DSP");
    content.set_inner_html(r##"
      <div style="flex:1; display:flex; flex-direction:column; overflow:hidden; background:#0a0d12;">
        <div style="display:flex; align-items:center; gap:8px; padding:4px 8px; background:var(--surface-panel); border-bottom:1px solid var(--border-subtle); font-size:11px;">
          <span style="color:#3fb950; font-weight:700;">&#9654; Play</span>
          <span style="color:#f85149; font-weight:700;">&#9679; Rec</span>
          <span style="font-family:var(--font-mono); color:var(--accent-cyan); margin-left:12px;">120.00 BPM</span>
          <span style="font-family:var(--font-mono); color:var(--text-muted);">00:01:24.12</span>
        </div>
        <div style="flex:1; overflow:auto; padding:6px; display:flex; flex-direction:column; gap:4px;">
          <div style="height:38px; background:var(--surface-panel); border:1px solid var(--border-subtle); border-radius:3px; display:flex; align-items:center; padding:0 8px; gap:8px;">
            <span style="font-size:10px; width:50px; font-weight:600;">Track 1</span>
            <div style="flex:1; height:20px; background:rgba(88,166,255,0.2); border-radius:2px; display:flex; align-items:center; padding:0 4px; font-size:9px; color:var(--accent-cyan);">~ ~ ~ Waveform Audio ~ ~ ~</div>
          </div>
          <div style="height:38px; background:var(--surface-panel); border:1px solid var(--border-subtle); border-radius:3px; display:flex; align-items:center; padding:0 8px; gap:8px;">
            <span style="font-size:10px; width:50px; font-weight:600;">Track 2</span>
            <div style="flex:1; height:20px; background:rgba(63,185,80,0.2); border-radius:2px; display:flex; align-items:center; padding:0 4px; font-size:9px; color:#3fb950;">~ ~ ~ Bassline Synth ~ ~ ~</div>
          </div>
        </div>
      </div>
      <div style="display:flex; justify-content:space-between; padding:3px 8px; background:var(--surface-panel); border-top:1px solid var(--border-subtle); font-size:10px; color:var(--text-muted);">
        <span>48 kHz &middot; 32-bit Float Audio</span><span>DSP Engine: Native SIMD</span>
      </div>
    "##);
    root
}

fn build_film_studio_view(document: &Document) -> Element {
    let (root, content) = studio_shell(document, "film-studio-view", "Film Studio", "Non-Linear Video Editor & Grade");
    content.set_inner_html(r##"
      <div style="flex:1; display:flex; flex-direction:column; overflow:hidden; background:#080b0f;">
        <div style="flex:1; display:flex; align-items:center; justify-content:center; position:relative;">
          <div style="width:240px; height:135px; background:#161b22; border:1px solid var(--border-subtle); border-radius:4px; display:flex; align-items:center; justify-content:center; color:var(--text-muted); font-size:11px;">
            4K Video Monitor (Rec.709)
          </div>
        </div>
        <div style="height:60px; border-top:1px solid var(--border-subtle); background:var(--surface-panel); padding:4px 8px; display:flex; flex-direction:column; gap:4px;">
          <div style="display:flex; justify-content:space-between; font-size:10px; font-family:var(--font-mono); color:var(--text-muted);">
            <span>00:00:14:08</span><span>24.000 fps</span>
          </div>
          <div style="height:14px; background:rgba(88,166,255,0.25); border-radius:2px; position:relative;">
            <div style="position:absolute; left:35%; top:0; bottom:0; width:2px; background:#f85149;"></div>
          </div>
        </div>
      </div>
      <div style="display:flex; justify-content:space-between; padding:3px 8px; background:var(--surface-panel); border-top:1px solid var(--border-subtle); font-size:10px; color:var(--text-muted);">
        <span>ProRes / WebCodecs NLE</span><span>Grade: ACEScc Pipeline</span>
      </div>
    "##);
    root
}

fn build_art_studio_view(document: &Document) -> Element {
    let (root, content) = studio_shell(document, "art-studio-view", "Spatial Studio", "Spatial 3D Studio & Diffusion");
    content.set_inner_html(r##"
      <div style="flex:1; position:relative; overflow:hidden; background:radial-gradient(circle, #1a2333, #0b0f17); display:flex; align-items:center; justify-content:center;">
        <div style="width:200px; height:150px; border:1px dashed var(--accent-cyan); border-radius:6px; display:flex; flex-direction:column; align-items:center; justify-content:center; gap:6px;">
          <span style="font-size:24px;">&#129485;</span>
          <span style="font-size:10px; color:var(--accent-cyan); font-weight:600;">3D Mannequin Pose Rig</span>
          <span style="font-size:9px; color:var(--text-muted);">Focal Length: 50mm &middot; Angle: 15&deg;</span>
        </div>
        <div style="position:absolute; bottom:8px; left:8px; right:8px; display:flex; gap:6px;">
          <input type="text" placeholder="Generative prompt modifier..." style="flex:1; background:var(--surface-panel); border:1px solid var(--border-subtle); border-radius:3px; padding:3px 6px; font-size:10px; color:var(--text-primary);" />
          <button style="background:var(--accent-cyan); color:#000; border:none; border-radius:3px; padding:3px 8px; font-size:10px; font-weight:700; cursor:pointer;">Generate</button>
        </div>
      </div>
      <div style="display:flex; justify-content:space-between; padding:3px 8px; background:var(--surface-panel); border-top:1px solid var(--border-subtle); font-size:10px; color:var(--text-muted);">
        <span>Spatial Scene &middot; Inpainting Mask</span><span>Diffusion Model: P64 Bridge</span>
      </div>
    "##);
    root
}
