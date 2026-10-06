# Poet HyperCanvas — containers, tools, and Qualia surfaces

**Date:** 2026-08-16  
**Branch:** `0.0.31-dev`  
**Principal:** Timothy Charles Holborn  
**Sources:** `C:\Projects\NLP\Canvas_Workbench` (structure), `UI_Mockup` (authoring primitives), `CAPABILITY_DESCRIPTORS` + `ids::ALL_BOUND` (engine), Classic `webizen-studio` routes (what was poorly presented).

You should **not** have to spoon-feed the inventory. This file is that list. What only you decide is **priority and honesty** on a few product calls (marked ⚑). Everything else is derived from work already in the tree.

---

## 0. Vocabulary (do not collapse)

| Word | What it is | Lives |
|---|---|---|
| **Manifold** | Work *surface* (virtual desktop) | pager + `manifolds/` seeds |
| **Container** | Occupant on a manifold (has x/y/w/h, strata, epistemic) | stage node |
| **Tool chest** | The dock | `chest.rs` |
| **Toolbox** | A drawer in the chest (places containers, tags, …) | palette |
| **Tool** | An instrument *inside* a container (toolbar, formula bar) | `poet/tools/<family>/` |
| **Canvas** | HTML/GPU drawing surface only | `gpu_frame`, `/gpu-viewport` |

A toolbox button that *places* a map is not the same object as the GIS *tool* that pins a track inside that map.

---

## 1. How much instruction you still need to give

**You do not need to list containers or tools.** Those are below.

**You do need to decide these ⚑ items when we hit them (not now):**

1. First rich-text persistence format: `yaml-ld-q42` / HCF vs a transitional markdown-with-`<q-entity>` (today).  
2. Whether WebRTC is in-process (`webrtc` crate already in desktop) or stays Present until a consent gate is designed.  
3. Which Classic routes may be *linked from* a Poet container vs which stay Classic-only until you write `approve deprecate: <route>`.  
4. Acceptable first spreadsheet: range stats via Vibe (now) vs CML cell→N3 (later).

Everything else: implement from this list, fail closed, label honesty.

---

## 2. Containers (complete set)

From HyperCanvas §4 + UI_Mockup primitives + Qualia surfaces that had no honest desk occupant.

| Id | Honesty now | Body / tool folder | Qualia it should reach |
|---|---|---|---|
| `doc` | live | `tools/rich_text` | `nlp.analyze`, gazetteer, `Document.ingest`, later HCF / `<q-doc>` |
| `code` | live | `tools/vibe` (to extract from console) | `poet_eval`, diagnose, `capability.invoke` |
| `sheet` | live (range) | `tools/sheet` | `Sheet.sum_range`, `Sheet.stats`, Statistics, P64 latents |
| `ontology` | partial | `tools/aura` | `SHACL.validate`, `SHACL.extensions`, catalog TTL |
| `map` | live (scene) | `tools/gis` | `Render.scene`, hull, `Manifold.project`, later GeoSPARQL |
| `media` | live (scene) | `tools/media` | kinematics, vision ahash, grapheme, later Forge |
| `social` | live (LWW) | `tools/social` | `Social.lww`, chat graph, later Pulse |
| `health` | live (Framingham ref. profile) | `tools/health` | ClinicalRisk, anatomy renderer, Wellfair (consent) |
| `subcanvas` | partial | — | switch manifold; enter-zoom later |
| `3d` | present | `tools/mesh` | `/gpu-viewport`, `webizen-render`, `.d10` / vocal tract |
| `webrtc` | present | `tools/rtc` | desktop `webrtc` crate; no fake stream |
| `webview` | present | `tools/webframe` | `<q-web-frame>`, capability-gated fetch |
| `portal` | present | `tools/portal` | wormhole IRI; later multi-tenant |
| `latex` | **missing** (add) | `tools/latex` | SymbolicAlgebra, CAS wasm |
| `graph` | **missing** (add) | `tools/graph` | SPARQL, quin.statement, RDF 1.2 (UI_Mockup graph-panel) |
| `triad` | **missing** (add) | `tools/triad` | q42↔p64↔d10, `qualia-audio` |
| `pulse` | **missing** (add) | `tools/pulse` | `pulse.publish` allowlist (not `pulse.broadcast`) |
| `rights` | **missing** (add) | `tools/rights` | deontic, DID, fiduciary sign |
| `wallet` | **missing** (add) | `tools/wallet` | identity / key vault (Classic Identity route) |
| `anatomy` | **missing** (add) | `tools/anatomy` | Wellfair anatomy `RenderScene` |
| `listen` | **missing** (add) | `tools/listen` | `qualia-audio`, EnCodec |
| `vision` | **missing** (add) | `tools/vision` | `ComputerVision.ahash`, `qualia-vision` |
| `finance` | **missing** (add) | `tools/econ` | Black–Scholes, portfolio (collab projects) |
| `slide` | **missing** (add) | `tools/slide` | office toolbox “presentation” |

Do **not** invent a second HTML mock for these. Place a container, put the tool in `poet/tools/<id>/`, bind Vibe/engine, label honesty.

---

## 3. Toolboxes (chest drawers)

From `toolbox-registry.js`. Keep these ids.

| Toolbox | Places / actions | First real tools |
|---|---|---|
| `epistemic` | tag objective / subjective / intersubjective / normative | set node.epistemic (no alert) |
| `office` | +doc, +ontology, +slide | `tools/rich_text` |
| `image` | +media, marker, heatmap | `tools/media`, vision |
| `sheet` | +sheet, import, resonance | `tools/sheet` |
| `spatial` | +map, +3d, +portal, pin, track | `tools/gis` |
| `communication` | +social, +webrtc, +webview | `tools/social`, rtc |
| `rights` | authors group, fiduciary, DID sign | `tools/rights` |
| `health` | +health, pathology, 10D anatomy | `tools/health`, anatomy |
| `code` | +vibe cell, requires[], quin.statement | `tools/vibe` — **never** `<<[` overlay |
| `ai` | co-author, extractor, sentinel, triad | local `AgentRuntime` only (no Ollama) |

---

## 4. In-container tool families (`poet/tools/`)

Each family is a folder. Toolbar spec is **data** so you can customise without rewriting the editor.

| Folder | Status | Analog | Customise by |
|---|---|---|---|
| `rich_text/` | started | TinyMCE-class, but ours | `ToolbarSpec` groups |
| `sheet/` | toolbar stub | Excel formula bar | command list |
| `latex/` | toolbar stub | Overleaf snippet bar | wrap + CAS invoke |
| `vibe/` | still in `vibe_console.rs` | IDE + REPL | extract next |
| `aura/` | not yet | SHACL inspector | shape list |
| `gis/` | not yet | QGIS-lite pin/track | — |
| `graph/` | not yet | UI_Mockup graph-panel | — |
| `triad/` | not yet | articulatory inspector | — |
| `pulse/` | not yet | UI_Mockup pulse-panel | — |

**Rich text decision:** we **make** it. TinyMCE (or CKEditor) is a third-party JS product with its own persistence and no `<q-entity>` / gazetteer / HCF path. A `ToolbarSpec` + mark wrap + gazetteer is the 0.1 door. Later: selection API / contenteditable + yaml-ld-q42.

---

## 5. Qualia / Classic surfaces that were poorly presented

These already exist as routes or engine families. They should become **containers or tools**, not another item in the Classic sidebar.

| Classic / engine surface | Poet home |
|---|---|
| `/library` Lived Memory | `doc` + `graph` on Research |
| `/talk` Relations | `social` |
| `/wellfair` Care | `health` + `anatomy` (consent) |
| `/browser` World | `webview` |
| `/work` Practice | `sheet` + `finance` + `rights` |
| `/listen` | `listen` / triad |
| `/vision` | `vision` / media |
| `/anatomy` | `anatomy` + `Render.scene` |
| `/identity` `/agency` `/sanctuary` | `wallet` + `rights` |
| `/chora` `/10d-browser` `/gpu-viewport` | `3d` / `map` / native canvas |
| `/qapp-studio` | `code` + portal |
| Nexus / SPARQL / SHACL panes | `graph` + `ontology` |
| Logic modalities (deontic, epistemic, LTL, ASP) | `code` invoke + `rights` |
| Math / stats / geometry / science / clinical / econ | `sheet`, `latex`, `map`, `health`, `finance` |
| Crypto / DID / ML-DSA | `rights` / `wallet` |

Classic **stays default**. Poet *reaches* these; it does not delete the routes.

---

Performance ∩ capability (do these before a native chrome rewrite): [`native-presentation-and-vibe-beyond-webview-2026-08-16.md`](native-presentation-and-vibe-beyond-webview-2026-08-16.md) **§1.3**.

## 6. Implementation order (no spoon-feeding required)

1. **Desk physics** — move, resize, snap, motion (this session).  
2. **Authoring trio** — `rich_text` customise, `sheet` formula bar live, `latex` wrap + CAS.  
3. **Epistemic/strata as tools** — palette sets fields on the selected node (replace alerts).  
4. **Graph + Aura** — SPARQL / SHACL inside containers.  
5. **Spatial** — pin/track on map; GPU frame already there.  
6. **Social / Pulse** — LWW is live; Pulse allowlist; WebRTC stays Present until ⚑2.  
7. **Extract `vibe/` tool** from the console.  
8. **Health / anatomy / listen / vision** — bind existing engines, consent where required.  
9. **Rights / wallet** — deontic + DID.  
10. Classic link-through only where you approve.

---

## 7. Dioxus version (so we do not thrash the studio)

| Pin | Role |
|---|---|
| **0.7.10** | Latest **stable** on crates.io (`max_stable_version`) |
| **0.8.0-alpha.0** | Previous pin |
| **0.8.0-alpha.1** | Current pin (`webizen-studio`, `qualia-mobile-harness`) — 2026-07-31 |

0.7.10 is **older** than the 0.8 alpha line. Studio checked green on `0.8.0-alpha.1`. The installed `dx` CLI may still be `0.8.0-alpha.0`; update it separately if wasm hot-patch misbehaves.

---

## 8. Honesty rules for every new tool

- No `alert()` as a finished action.  
- No fake WebRTC / chat streams.  
- No TinyMCE/CKEditor/Monaco unless you explicitly ask to vendor one.  
- No `<<[ s p o g prov ]>>`. Quin construction is `quin.statement`.  
- No Ollama.  
- One folder per tool family. Files stay small.
