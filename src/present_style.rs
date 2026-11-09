//! Self-contained, zero-external-dependency Studio CSS Design System & Ontological Lifecycle
//! Classifier (`present_style.rs`).
//!
//! Eliminates external CDN dependencies and cyclic CSS custom property bugs so Synth Studio
//! renders with identical high-contrast architectural precision in system browsers
//! (`http://127.0.0.1:8787`), standalone `.html` artifacts, and `<agent-embed>` panes.

/// Classifies a resource key into its Realization Lifecycle Stage:
/// - `("concept-creative", "Stage 1 · Concept & Creative Work (World of Mind)", ...)`
/// - `("cad-spec", "Stage 2 · Parametric CAD / BIM / Scaffold Spec (Mind → Reality)", ...)`
/// - `("realized-twin", "Stage 3 · Realized Twin & Production Artifact", ...)`
pub fn classify_lifecycle_stage(
    key: &str,
) -> (&'static str, &'static str, &'static str, &'static str) {
    let lower = key.to_ascii_lowercase();
    if lower.contains("creative_works/")
        || lower.contains("fictional_")
        || lower.contains("music_")
        || lower.contains("airship")
        || lower.contains("chronicle_folio")
    {
        (
            "concept-creative",
            "Stage 1 · Creative / Concept Model",
            "badge-stage-concept",
            "Conceived in World of Mind (schema:CreativeWork) · Ready for imaginative presentation or CAD realization into World of Man",
        )
    } else if lower.ends_with(".svg")
        || lower.ends_with(".ifc")
        || lower.contains("_blueprint")
        || lower.contains("_ecorche")
        || lower.contains("_skeleton")
        || lower.contains("manifest.json")
        || lower.contains("telemetry")
    {
        let bridge_note = if lower.starts_with("world_of_man/") {
            "Cross-Domain Bridge (Mind → World of Man): Parametric CAD / IFC4 BIM engineering specification for physical manufacture"
        } else if lower.starts_with("world_of_god/") {
            "Cross-Domain Bridge (Mind → World of God): Portable anatomical & osteological scaffold (.json/.vibe) modeling a natural species"
        } else {
            "Formal System & Pipeline Specification (Mind → Compute Runtime)"
        };
        (
            "cad-spec",
            "Stage 2 · CAD / BIM / Anatomical Spec",
            "badge-stage-spec",
            bridge_note,
        )
    } else {
        let realized_note = if lower.starts_with("world_of_man/") {
            "Realized in World of Man: Manufacturable / built physical assembly produced from conceptual & CAD modeling"
        } else if lower.starts_with("world_of_god/") {
            "Realized in World of God: Empirical biological, geomorphic, or planetary digital twin (.10d / DEM)"
        } else {
            "Realized in World of Mind (Software): Active self-hosted Studio & Tray runtime artifact"
        };
        (
            "realized-twin",
            "Stage 3 · Realized Twin / Production",
            "badge-stage-realized",
            realized_note,
        )
    }
}

/// Complete, zero-external-dependency CSS stylesheet for Synth Studio & Tray Workbench.
pub const SYNTH_STUDIO_CSS: &str = r#"
:root {
  --background: #0a1014;
  --surface: #0f171d;
  --card: #131d24;
  --card-hover: #18252e;
  --foreground: #eaf2f0;
  --muted-foreground: #8ca39e;
  --border: #22343e;
  --border-strong: #2f4956;
  --emerald: #10b981;
  --emerald-light: #6ee7b7;
  --emerald-bg: rgba(6, 78, 59, 0.45);
  --amber: #f59e0b;
  --amber-light: #fcd34d;
  --amber-bg: rgba(120, 53, 15, 0.45);
  --purple: #a855f7;
  --purple-light: #d8b4fe;
  --purple-bg: rgba(88, 28, 135, 0.45);
  --fuchsia: #d946ef;
  --fuchsia-light: #f0abfc;
  --fuchsia-bg: rgba(112, 26, 117, 0.45);
  --sky: #0ea5e9;
  --sky-light: #7dd3fc;
  --sky-bg: rgba(12, 74, 110, 0.45);
  --font-sans: 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, sans-serif;
  --font-mono: 'JetBrains Mono', 'Cascadia Code', 'Fira Code', ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
}

*, *::before, *::after {
  box-sizing: border-box;
}

html, body {
  margin: 0;
  padding: 0;
  background-color: var(--background);
  background-image:
    radial-gradient(circle at 12% 0%, rgba(16, 185, 129, 0.10) 0%, transparent 42%),
    radial-gradient(circle at 88% 4%, rgba(14, 165, 233, 0.10) 0%, transparent 42%),
    radial-gradient(circle at 50% 100%, rgba(168, 85, 247, 0.07) 0%, transparent 50%);
  background-attachment: fixed;
  color: var(--foreground);
  font-family: var(--font-sans);
  font-size: 14px;
  line-height: 1.5;
  -webkit-font-smoothing: antialiased;
}

body {
  padding: 24px 28px 48px;
}

h1, h2, h3, h4, p, pre {
  margin: 0;
}

button, input, textarea, select {
  font-family: inherit;
  color: inherit;
}

button {
  cursor: pointer;
  transition: all 0.15s ease;
}

button:hover {
  filter: brightness(1.12);
}

/* Layout Containers */
.max-w-7xl { max-width: 1380px; }
.max-w-5xl { max-width: 1080px; }
.mx-auto { margin-left: auto; margin-right: auto; }

.space-y-1 > * + * { margin-top: 4px; }
.space-y-1\.5 > * + * { margin-top: 6px; }
.space-y-2 > * + * { margin-top: 8px; }
.space-y-2\.5 > * + * { margin-top: 10px; }
.space-y-3 > * + * { margin-top: 12px; }
.space-y-4 > * + * { margin-top: 16px; }
.space-y-5 > * + * { margin-top: 20px; }

/* Grid & Flexbox */
.grid { display: grid; }
.grid-cols-1 { grid-template-columns: repeat(1, minmax(0, 1fr)); }
.grid-cols-2 { grid-template-columns: repeat(2, minmax(0, 1fr)); }

@media (min-width: 768px) {
  .md\:grid-cols-2 { grid-template-columns: repeat(2, minmax(0, 1fr)); }
  .md\:grid-cols-3 { grid-template-columns: repeat(3, minmax(0, 1fr)); }
  .md\:grid-cols-6 { grid-template-columns: repeat(6, minmax(0, 1fr)); }
}

@media (min-width: 1080px) {
  .lg\:grid-cols-2 { grid-template-columns: repeat(2, minmax(0, 1fr)); }
  .lg\:grid-cols-3 { grid-template-columns: repeat(3, minmax(0, 1fr)); }
}

.flex { display: flex; }
.flex-wrap { flex-wrap: wrap; }
.items-center { align-items: center; }
.items-end { align-items: flex-end; }
.justify-between { justify-content: space-between; }
.justify-center { justify-content: center; }

.gap-1 { gap: 4px; }
.gap-1\.5 { gap: 6px; }
.gap-2 { gap: 8px; }
.gap-3 { gap: 12px; }
.gap-4 { gap: 16px; }

/* Spacing & Sizing */
.p-2 { padding: 8px; }
.p-2\.5 { padding: 10px; }
.p-3 { padding: 12px; }
.p-3\.5 { padding: 14px; }
.p-4 { padding: 16px; }
.p-5 { padding: 20px; }
.px-1\.5 { padding-left: 6px; padding-right: 6px; }
.px-2 { padding-left: 8px; padding-right: 8px; }
.px-2\.5 { padding-left: 10px; padding-right: 10px; }
.px-3 { padding-left: 12px; padding-right: 12px; }
.px-3\.5 { padding-left: 14px; padding-right: 14px; }
.px-4 { padding-left: 16px; padding-right: 16px; }
.py-0\.5 { padding-top: 2px; padding-bottom: 2px; }
.py-1 { padding-top: 5px; padding-bottom: 5px; }
.py-1\.5 { padding-top: 7px; padding-bottom: 7px; }
.pt-1 { padding-top: 4px; }
.pt-2 { padding-top: 10px; }
.pb-3 { padding-bottom: 12px; }
.pr-1 { padding-right: 4px; }
.mt-0\.5 { margin-top: 2px; }
.mt-2 { margin-top: 8px; }
.mt-3 { margin-top: 12px; }
.mb-1 { margin-bottom: 4px; }

.w-full { width: 100%; }
.w-auto { width: auto; }
.w-64 { width: 256px; }
.w-72 { width: 288px; }
.min-h-\[180px\] { min-height: 200px; }
.max-h-\[260px\] { max-height: 260px; }
.max-h-\[420px\] { max-height: 420px; }
.max-h-\[430px\] { max-height: 430px; }
.max-h-\[520px\] { max-height: 520px; }
.max-h-56 { max-height: 224px; }
.h-\[320px\] { height: 320px; }

/* Typography */
.font-mono { font-family: var(--font-mono); }
.font-sans { font-family: var(--font-sans); }
.font-normal { font-weight: 400; }
.font-medium { font-weight: 500; }
.font-semibold { font-weight: 600; }
.font-bold { font-weight: 700; }
.uppercase { text-transform: uppercase; }
.tracking-wider { letter-spacing: 0.06em; }
.leading-relaxed { line-height: 1.6; }
.truncate { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.select-none { user-select: none; }

.text-\[10px\] { font-size: 10.5px; }
.text-\[11px\] { font-size: 11.5px; }
.text-xs { font-size: 12px; }
.text-sm { font-size: 13.5px; }
.text-base { font-size: 15.5px; }
.text-lg { font-size: 18px; }
.text-xl { font-size: 21px; }

/* Colors & Surfaces */
.bg-\[var\(--background\)\] { background-color: var(--surface); }
.bg-\[\#0a1210\] { background-color: #091214; }
.bg-\[var\(--card\)\] {
  background: linear-gradient(180deg, #142027 0%, #101920 100%);
}

.text-\[var\(--foreground\)\] { color: var(--foreground); }
.text-\[var\(--muted-foreground\)\] { color: var(--muted-foreground); }
.text-emerald-200 { color: #a7f3d0; }
.text-emerald-300 { color: var(--emerald-light); }
.text-emerald-400 { color: var(--emerald); }
.text-amber-300 { color: var(--amber-light); }
.text-sky-200 { color: #bae6fd; }
.text-sky-300 { color: var(--sky-light); }
.text-sky-400 { color: var(--sky); }
.text-purple-300 { color: var(--purple-light); }
.text-fuchsia-300 { color: var(--fuchsia-light); }

/* Borders & Corners */
.border { border: 1px solid var(--border); }
.border-t { border-top: 1px solid var(--border); }
.border-b { border-bottom: 1px solid var(--border); }
.border-\[var\(--border\)\] { border-color: var(--border); }

.rounded { border-radius: 6px; }
.rounded-md { border-radius: 8px; }
.rounded-lg { border-radius: 10px; }
.rounded-xl { border-radius: 14px; }
.shadow-sm { box-shadow: 0 8px 24px -6px rgba(0, 0, 0, 0.45), 0 0 0 1px rgba(255, 255, 255, 0.02) inset; }

/* Semantic Accent Badges & Buttons */
.border-emerald-500\/40, .border-emerald-500\/50, .border-emerald-500\/60, .border-emerald-500\/70, .border-emerald-500 {
  border-color: rgba(16, 185, 129, 0.55);
}
.bg-emerald-950\/30, .bg-emerald-950\/40, .bg-emerald-950\/50, .bg-emerald-950\/60, .bg-emerald-950\/70, .bg-emerald-600\/30 {
  background-color: var(--emerald-bg);
}

.border-amber-500\/40, .border-amber-500\/50 {
  border-color: rgba(245, 158, 11, 0.55);
}
.bg-amber-950\/30, .bg-amber-950\/60 {
  background-color: var(--amber-bg);
}

.border-purple-500\/40, .border-purple-500\/50 {
  border-color: rgba(168, 85, 247, 0.55);
}
.bg-purple-950\/30, .bg-purple-950\/60 {
  background-color: var(--purple-bg);
}

.border-fuchsia-500\/40, .border-fuchsia-500\/50 {
  border-color: rgba(217, 70, 239, 0.55);
}
.bg-fuchsia-950\/30, .bg-fuchsia-950\/60 {
  background-color: var(--fuchsia-bg);
}

.border-sky-500\/40, .border-sky-500\/50 {
  border-color: rgba(14, 165, 233, 0.55);
}
.bg-sky-950\/30, .bg-sky-950\/50, .bg-sky-950\/60 {
  background-color: var(--sky-bg);
}

/* Studio Cards & Interactive Polish */
.resource-card {
  display: flex;
  flex-direction: column;
  justify-content: space-between;
  transition: transform 0.16s ease, border-color 0.16s ease, box-shadow 0.16s ease;
}

.resource-card:hover {
  border-color: var(--border-strong);
  box-shadow: 0 12px 30px -8px rgba(0, 0, 0, 0.6), 0 0 0 1px rgba(110, 231, 183, 0.12) inset;
}

/* Lifecycle Pipeline Banner & Stage Badges */
.lifecycle-pipeline-banner {
  background: linear-gradient(90deg, rgba(88, 28, 135, 0.24) 0%, rgba(12, 74, 110, 0.26) 50%, rgba(6, 78, 59, 0.26) 100%);
  border: 1px solid var(--border-strong);
  border-radius: 12px;
  padding: 12px 16px;
}

.lifecycle-step {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 12px;
  border-radius: 9px;
  background: rgba(10, 16, 20, 0.65);
  border: 1px solid var(--border);
  flex: 1 1 240px;
}

.lifecycle-arrow {
  color: var(--muted-foreground);
  font-family: var(--font-mono);
  font-size: 16px;
  font-weight: 700;
  padding: 0 2px;
}

.badge-stage-concept {
  background: rgba(112, 26, 117, 0.42);
  border: 1px solid rgba(217, 70, 239, 0.55);
  color: #f5d0fe;
}

.badge-stage-spec {
  background: rgba(12, 74, 110, 0.45);
  border: 1px solid rgba(56, 189, 248, 0.55);
  color: #bae6fd;
}

.badge-stage-realized {
  background: rgba(6, 78, 59, 0.45);
  border: 1px solid rgba(16, 185, 129, 0.55);
  color: #a7f3d0;
}

.filter-pill-active {
  box-shadow: 0 0 0 1px rgba(110, 231, 183, 0.65), 0 0 14px -2px rgba(16, 185, 129, 0.35);
  font-weight: 600;
}

/* Misc utilities */
.relative { position: relative; }
.absolute { position: absolute; }
.bottom-2 { bottom: 8px; }
.left-2\.5 { left: 10px; }
.block { display: block; }
.hidden { display: none !important; }
.overflow-hidden { overflow: hidden; }
.overflow-auto { overflow: auto; }
.overflow-x-auto { overflow-x: auto; }
.overflow-y-auto { overflow-y: auto; }
.object-contain { object-fit: contain; }
.pointer-events-none { pointer-events: none; }
.cursor-pointer { cursor: pointer; }
.cursor-grab { cursor: grab; }
.active\:cursor-grabbing:active { cursor: grabbing; }

pre {
  white-space: pre-wrap;
  word-break: break-word;
}

input[type="text"], input[type="search"], textarea {
  background-color: #0b1217;
  border: 1px solid var(--border);
  color: var(--foreground);
  outline: none;
}

input[type="text"]:focus, input[type="search"]:focus, textarea:focus {
  border-color: var(--emerald);
  box-shadow: 0 0 0 2px rgba(16, 185, 129, 0.2);
}
"#;
