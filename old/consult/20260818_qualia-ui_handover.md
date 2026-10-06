# Qualia-UI Implementation Hand-Over

**Date:** 2026-08-18  
**Principal / copyright holder:** Timothy Charles Holborn <timothy.holborn@gmail.com>  
**Session focus:** WASM runtime debugging → app loads → next implementation phases

---

## 1. What was accomplished this session

### 1.1 WASM runtime fix (root cause found and fixed)

**Problem:** The `qualia-ui` WASM app was not loading — the `#loading` div persisted indefinitely, no UI rendered, and no console logs appeared.

**Root cause:** `build_gis_map_view()` in `c:\Projects\NLP\qualia-ui\src\browser\containers.rs` was calling `.set_class_name()` on SVG elements (`<svg>`, `<g>`). SVG elements expose `className` as a **read-only** property (per SVG spec); only `setAttribute("class", ...)` works. This threw a `TypeError: Cannot set property className of #<SVGElement> which has only a getter`, which propagated as a JS exception through the WASM call stack, killing the entire `start()` function before any UI could render.

**Fix applied:** Changed 3 calls in `containers.rs` from `set_class_name(...)` to `set_attribute("class", ...)`:
- Line 564: `svg.set_attribute("class", "gis-map-svg")`
- Line 595: `pin1.set_attribute("class", "agent-pin-marker")`
- Line 605: `pin2.set_attribute("class", "agent-pin-marker")`

### 1.2 Error handling infrastructure added

The following diagnostic/error-display infrastructure was added to `src/browser/mod.rs` and `index.html`:

- `console_error_panic_hook::set_once()` installed at start of `start()`
- `try_start()` wraps app initialization in `Result<(), String>` with error propagation
- `show_fatal_error()` displays errors in the DOM (replaces loading indicator with red error text)
- `index.html` has early `error` and `unhandledrejection` listeners that catch and display WASM init failures
- `index.html` has a 10-second timeout fallback that shows "WASM did not initialize" if no `.app` element appears
- `#[wasm_bindgen(start)]` was **removed** from `start()`; `main()` calls `start()` directly, and `index.html` manually calls `window.wasmBindings.start()` after the `TrunkApplicationStarted` event

### 1.3 Cleanup needed

- The `start()` function still has a green "WASM loaded OK — building app..." debug message injected into the loading div. This should be removed once confident the app loads reliably.
- The `index.html` has extensive debug/error-display JS that can be simplified once the app is stable.
- `#[wasm_bindgen(start)]` was removed — consider re-adding it once the double-init issue is properly understood. The current approach (manual call from HTML) works but is non-standard.
- `panic = "unwind"` was removed from `Cargo.toml` — `catch_unwind` was removed from `build_app_safe`, so this is fine. If `catch_unwind` is needed again, `panic = "unwind"` must be re-added.
- `build_app_safe()` is now a trivial wrapper around `build_app()` — it can be inlined.

---

## 2. Current project state

### 2.1 What works

- WASM app loads and renders in the browser via `trunk serve --port 8080`
- 6 manifold seeds render with containers, wires, and container bodies
- Theme engine (7 presets, presentation levels, sanctuary mode, WCAG)
- Command palette (Ctrl+K)
- Container drag/resize/selection, canvas pan/zoom
- Diagnostics module (Aura tray, Pulse stream, Job Center)
- Manifest module (CBOR-LD serialization stubs)
- 36 N3 ontology files in `qualia-ui/ontologies/`

### 2.2 File structure

```
qualia-ui/
├── Cargo.toml          -- crate-type = ["cdylib", "rlib"], wasm32 target
├── Trunk.toml          -- port 8080, dist dir
├── index.html          -- Trunk entry, error handlers, manual start() call
├── src/
│   ├── lib.rs          -- library crate root
│   ├── main.rs         -- binary entry, calls browser::start()
│   ├── browser/        -- 16 modules:
│   │   ├── mod.rs            -- start(), try_start(), build_app(), build_canvas(), manifold switching
│   │   ├── css.rs            -- full glassmorphism CSS string
│   │   ├── topbar.rs         -- menubar, canvas control bar
│   │   ├── containers.rs     -- container node builder + all container body views (social, GIS, media, VibeScript)
│   │   ├── container_views.rs     -- doc, sheet, graph, ontology, pulse, rights, wallet views
│   │   ├── container_views_ext.rs -- library, aura, latex, health, anatomy, webview, webrtc, finance, vision, listen, triad, portal, slide, 3d, subcanvas views
│   │   ├── docks.rs          -- toolbox dock, right dock (diagnostics), bottom statusbar
│   │   ├── interactions.rs   -- drag, resize, pan/zoom, selection, wire rendering
│   │   ├── command_palette.rs -- Ctrl+K palette
│   │   ├── theme.rs          -- 7 theme presets, presentation levels, sanctuary, WCAG
│   │   ├── diagnostics.rs    -- Aura tray, Pulse stream, Job Center
│   │   ├── manifest.rs       -- ManifoldSeed CBOR-LD serialization stubs
│   │   ├── capabilities.rs   -- capability detection stubs
│   │   ├── intent_bus.rs     -- IntentBus trait (no implementation)
│   │   ├── registration.rs   -- toolbox/tool registration specs
│   │   └── history.rs        -- undo/redo history stubs
│   └── tool_chest/
│       ├── core/             -- registry, intent_bus trait, payload types
│       ├── manifolds/        -- 6 manifold seed definitions
│       └── payloads/         -- VibeScriptPayload, graph mutations, etc.
└── ontologies/               -- 36 N3 ontology files
```

### 2.3 Build & run

```powershell
$env:CARGO_TARGET_DIR="$env:TEMP\qualia-ui-target"
$env:RUSTFLAGS="--cfg web_sys_unstable_apis"
trunk serve --port 8080
```

Browser: `http://127.0.0.1:8080`

---

## 3. What remains to be implemented

### 3.1 From the gap audit (`consult/20260817_qualia-ui-gap-audit.md`)

**P0 — Blocking gaps:**

| Gap | Status | Notes |
|-----|--------|-------|
| No `IntentBus` implementation | Not started | Need concrete struct with WASM transport (fetch/WebSocket), capability gating, provenance, receipts |
| No backend transport | Not started | Typed client layer mapping `VibeScriptPayload` → daemon HTTP/WS endpoints |
| No ontology loading | Not started | Build-time N3 → CBOR-LD compiler + WASM `OntologyLoader` with `include_bytes!` |
| No toolbox/tool registration at startup | Not started | Register all 9 toolboxes, tool-chains, 6 manifold seeds into shared `Registry` |
| Container bodies are placeholders | Partial | ~25 container types exist as mock views; need real engine wiring |
| No tool families (`poet/tools/`) | Not started | 9 tool family folders: rich_text, sheet, latex, vibe, aura, gis, graph, triad, pulse |
| No hypermedia library surface | Not started | `library` container + `doc` container reaching `qualia-semantic-library` crate |

**P1 — Important:**

| Gap | Status | Notes |
|-----|--------|-------|
| No canvas editor semantics | Stub only | Undo/redo history exists in `history.rs` but is not wired. Need grid snapping, manifold persistence |
| No theme engine | **Done** | `theme.rs` has 7 presets, presentation levels, sanctuary mode, WCAG contrast |
| No render subsystem | Not started | WASM Canvas2D renderer + native GPU bridge |
| No capability detection | Stub only | `capabilities.rs` has stubs; need real native-host vs public-web detection |

**P2 — Future:**

| Gap | Status | Notes |
|-----|--------|-------|
| No QApp studio surface | Not started | `code` + `portal` container pair |
| No wellfair life-domain shell | Partial | `health`, `anatomy`, `rights`, `wallet` containers exist as mocks |
| No specialized scientific tools | Partial | `sheet`/`latex`/`map`/`health`/`finance` containers exist as mocks |

### 3.2 From the tracking todo (`TechDesign/poet_hypercanvas_tracking_todo.md`)

**Phase 2: Dioxus Webizen-Studio Port** (adapted for qualia-ui):
- [ ] Create modular tool components under `src/browser/tools/` (or similar)
- [ ] Implement Dynamic Context Ribbon Receptor (clippable toolchain deck)
- [ ] Rename Canvases → Manifolds menu in TopMenubar
- [ ] Implement UI Modality & A11y state in store
- [ ] Wire 8px desk physics and SVG connection wires (partially done)

**Phase 3: Tauri IPC Integration:**
- [ ] Connect Vibe console to `poet_eval` Tauri command
- [ ] Connect RichText entity button to `poet_gazetteer` NLP command
- [ ] Connect Companion Gateway (WebRTC / Tablet Remote)
- [ ] Connect Media/3D tools to `poet_render` WGPU native surface
- [ ] Verify telemetry bridge and MCP server integration

### 3.3 Tool families to implement (from plan §3)

```
tools/
├── rich_text/          # WYSIWYG, Semantic Spans & NLP Gazetteer
├── sheet/              # Spreadsheets, Formulas, Columnar Tensors
├── rdf_graph/          # Aura / SHACL Validation, N3 Logic, SPARQL
├── hypermedia/         # Hypermedia Library, HCF/CBOR Archives
├── media_studio/       # Photo, Vector Art, Video
├── ten_d_assets/       # 10D Asset Container, Kinematics, Animations
├── audio_speech/       # Spectral Tomography, Waveforms, Q42/D10 Triads
├── communication/      # Multi-Agent Chat Graph, WebRTC, DID Auth
├── spatial_gis/        # Topographic GIS Maps, 4D Tracks
├── vibescript_code/    # VibeScript 0.1 AST Console, Capabilities, Gas
├── rights_fiduciary/   # Fiduciary Duty Scopes, L3 Gates, Merkle Audit
└── health_clinical/    # Pathology Lab, Bio-Telemetry, 10D Anatomy
```

---

## 4. Key references for the next session

| Document | Path | Purpose |
|----------|------|---------|
| Gap audit | `consult/20260817_qualia-ui-gap-audit.md` | Comprehensive gap analysis vs desktop/studio crates |
| Tracking todo | `TechDesign/poet_hypercanvas_tracking_todo.md` | Phase plan, tool family specs, implementation checklist |
| HyperCanvas plan | `TechDesign/poet-hypercanvas-tools-and-containers-2026-08-16.md` | Original product direction document |
| Agent rules | `AGENTS.md` | Project rules, WASM constraints, nomenclature isolation |
| Nomenclature | `qualia-db-standards/agent-nomenclature-rules.md` | Agent type term isolation rules |
| VibeScript spec | `qualia-db-standards/vibescript-specification.md` | VibeScript 0.1 language specification |
| File format | `TechDesign/FileFormat.md` | HCF/CBOR-LD file format specification |
| Scripting lang | `TechDesign/scriptingLang.md` | VibeScript & Poet Engine spec |
| Agent requirements | `TechDesign/agent_vibescript_requirements.md` | AI Agent VibeScript requirements |

---

## 5. Technical notes for the next session

### 5.1 WASM gotchas discovered

1. **SVG elements:** Use `set_attribute("class", ...)` NOT `set_class_name()`. SVG `className` is read-only.
2. **`panic = "unwind"` on wasm32:** Works but changes the WASM binary. `catch_unwind` requires it. Without it, `catch_unwind` is a no-op.
3. **`#[wasm_bindgen(start)]` vs `main()`:** When both exist, `__wbindgen_start` auto-calls `start()` during WASM instantiation. If `main()` also calls `start()`, double-init occurs. Current solution: removed `#[wasm_bindgen(start)]`, `main()` calls `start()`, and `index.html` manually calls `window.wasmBindings.start()` after `TrunkApplicationStarted`. Consider re-adding `#[wasm_bindgen(start)]` with a proper guard.
4. **`thread_local!` with `Cell<bool>`:** Was used as double-init guard. May have caused issues on wasm32 — was replaced with `static mut bool` then removed entirely. If re-adding a guard, test carefully.
5. **`web_sys::console::log_1`:** Works fine on wasm32. Takes `&JsValue` — use `&"string".into()`.

### 5.2 File size limits

Project rule: **No file may exceed 800 lines.** Several files are approaching this:
- `container_views_ext.rs` — ~29KB, likely 700+ lines. May need splitting.
- `containers.rs` — ~26KB, 637 lines. Close to limit.
- `css.rs` — ~30KB but it's a single CSS string constant, so line count may be fine.
- `registration.rs` — ~21KB, check line count.

### 5.3 Trunk build

- `Trunk.toml` configures port 8080
- `Cargo.toml` has `crate-type = ["cdylib", "rlib"]`
- `RUSTFLAGS="--cfg web_sys_unstable_apis"` is required for some web-sys features
- Target dir is set to `$env:TEMP\qualia-ui-target` to avoid polluting the project tree

### 5.4 Suggested next steps (priority order)

1. **Clean up debug infrastructure** — remove green "WASM loaded OK" message, simplify `index.html` error handlers, consider re-adding `#[wasm_bindgen(start)]` with proper guard
2. **Implement `IntentBus`** — concrete struct with WASM transport, capability gating, provenance stamping, receipt handling (P0)
3. **Implement toolbox registration** — register all 9 toolboxes and tool-chains at startup (P0)
4. **Create `poet/tools/` directory structure** — 9 tool family modules with `ToolbarSpec`-driven toolbars (P0)
5. **Implement dynamic clippable toolchain deck** — context ribbon receptor that switches based on focused container (P1)
6. **Wire container bodies to real engines** — replace placeholder views with actual capability calls (P0, ongoing)
7. **Build-time N3 → CBOR-LD compiler** — Trunk hook or `build.rs` to compile ontologies (P0)
8. **Canvas editor semantics** — undo/redo, grid snapping, manifold persistence (P1)
