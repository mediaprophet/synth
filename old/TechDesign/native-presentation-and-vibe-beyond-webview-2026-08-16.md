# Native presentation, Vibe, and leaving the WebView — considerations for socialisation

**Date:** 2026-08-16  
**Status:** discussion paper for the principal and reviewers — **not** an implementation claim and **not** a deprecation of Classic.  
**Branch:** `0.0.31-dev`  
**Principal / inventor:** Timothy Charles Holborn  
**Copyright © 2026 Timothy Charles Holborn.** All rights reserved.

This note takes a Gemini (Antigravity) conversation on Tauri v2 vs Dioxus, Dioxus-on-WASM, a custom 10D pipeline, and a “Web VM” that eradicates V8, and **grounds it in what Qualia already is**. It adds an incremental replacement sequence that prefers **modern native technique** over compatibility with a public Webizen/Qualia release that **has not happened yet**.

**The point of that sequence is not “a nicer website.”** Qualia/Webizen is allowed to be an **advanced, optimised, re-envisaged environment that is different from the traditional WWW ecosystem.** HTML, CSS, JS, and JSON do not define it. They do not veto it. Ontologies of those parts exist so WWW can be a **named, optional exit** — not the design language.

| Leave (host / legacy WWW pair) | Arrive (product HID) |
|---|---|
| **JSON** as IPC, REST, diagnose blobs, and “JSON-LD on the wire” | **CBOR-LD 1.0** as the machine and wire layer |
| **JavaScript** (and Dioxus `rsx!` as a stand-in) as the app language | **Vibe** interpreted by Poet |
| **HTML + CSS** as the document and layout language | **HCF** (yaml-ld-q42) + **SceneContract** / 10D / theme morphology |

HCF already titles this: *“Pure Linked Data from Authoring to Wire: The End of XML, HTML, and JS Bloat.”* Authoring is **yaml-ld-q42**. Wire is **CBOR-LD**. Language is **Vibe**. Presentation is **Poet + SceneContract**, not a DOM. Resident graph is **Q42 / NQuin**. Pulse carries **CBOR**, not JSON documents. Dioxus, Tauri JSON commands, wasm-bindgen glue, and `*.json` sidecars are a **temporary host**, not the destination.

**2026-08-16 reviews:**

1. First draft specified the long WebView-exit path (§5) but under-specified **near-term work that is both faster and uses more of the engine**. That intersection is §1.3 (Q0/Q1). Do not socialise §5 without §1.3.
2. **Second draft buried JSON→CBOR-LD and JS→Vibe as background.** They are the destination pair (§0.1). WebView exit is *how chrome stops needing JS*. CBOR-LD is *how the wire stops being JSON*. Do not socialise §5 or §6 as “maybe later we might prefer Vibe.”

Classic UI stays default until Timothy writes `approve deprecate:`.

---

## 0. How to read this

| If you want… | Start here |
|---|---|
| **The destination pair (JSON→CBOR-LD, JS→Vibe)** | **§0.1** |
| **WWW ontologies are exits; HID is not a website** | **§0.2** |
| **WASM package family is the bridge** | **§0.3** |
| What the product already does | §1 |
| **Quick wins: faster *and* more Qualia used** | **§1.3** |
| Tauri vs Dioxus **in this repo** (not generic Medium) | §2 |
| Can Dioxus be a WASM app? | §3 |
| How big is a custom Qualia presentation library? | §4 |
| Biggest wins, in order (WebView *and* wire) | §5 |
| Vibe instead of JavaScript (why 0.1 is not a JS clone) | §6 |
| Optional later “WWW VM” (not a design constraint) | §7 |
| Open questions for feedback | §8 |

### 0.1 The destination pair — this is the point

Two replacements, one RDF model. They are not optional styling of a web app.

```
Human authors          Machine / wire           Resident / compute
─────────────          ──────────────           ──────────────────
yaml-ld-q42  ──encode──►  CBOR-LD 1.0  ──map──►  Q42 / NQuin / P64 / D10
.vibe source ──parse───►  CBOR-LD AST  ──eval──►  Poet + capability.invoke
                          Pulse frames
```

| Layer | Destination | Temporary host (honest) | Must not become the product |
|---|---|---|---|
| **Document / QApp** | yaml-ld-q42 source, HCF persist | Studio `rsx!` + HTML mock | HTML, XML, TinyMCE |
| **Wire / IPC / sync** | CBOR-LD 1.0 (HCF §3; vibe essay §5.A) | Tauri `serde` JSON commands; `--progress json`; ingest `*.q42.source.json`; agent diagnose JSON | JSON-LD as the *wire*; REST-of-JSON Quins |
| **Language** | Vibe (`poet-vibe`, no JIT) | Dioxus Rust chrome; wasm-bindgen JS *glue* | JavaScript as the app language; Boa as a Vibe interpreter |
| **Events** | Pulse (CBOR envelopes already exist in `chat_mesh`) | DOM events + Tauri events | `addEventListener` as the HID bus |
| **Graph** | Q42 / Super-Quin | JSON clones of Quins into WASM | “the app state is a JS object” |

**Why CBOR-LD rather than “just binary” or “just JSON-LD”:**

- Same RDF 1.2 model as yaml-ld-q42 and Turtle. Compression is *term dictionaries*, not a private schema.
- Shared context → zero-copy-friendly on WASM / mobile / IoT (HCF §3). Magic `0xd9 0xd9 0xf7` is already a `detect_format` probe in this tree.
- Q42 ingest already speaks of CBOR-LD + LZ4 SuperBlocks. HMC stores HCF as a **CBOR-LD binary AST**.
- vibe-core artifact table: Canonical AST is “versioned nodes (**later RDF/CBOR-LD**)”. The architectural essay already calls this homoiconic: code is a graph, loaded without re-lexing strings.

**Why Vibe rather than “Rust in the WebView” or “a better JS”:**

- Humans and agents author *capabilities on a graph*, not DOM mutations.
- 0.1 is closed, typed, effect-bounded, iOS-safe (no JIT). It **reaches** Qualia; it does not reimplement Chromium.
- The consult line “do not try to replace JS” meant: **0.1 is not a JS clone and does not host React.** It did **not** mean the product keeps JavaScript. Replacing JS *as Qualia’s application language* is the HID destination. That is a later numbered profile (`vibe-hid-0.2` suggested) plus this wire, not a 0.1 grammar fork.

**Honesty (do not claim done):** desktop Poet still crosses Tauri as JSON-shaped values; Vibe still evaluates a Rust AST (not yet a CBOR-LD AST on disk); ingest attestation sidecars are JSON; Classic chrome is Dioxus-web. Those are **debt**, scheduled in §1.3 / §5, not architecture.

### 0.2 WWW ontologies are optional exits — they do not confine the HID

The interesting fact is **not** that HTML or CSS can be emitted. It is that Qualia **does not have to be designed as HTML+CSS+JS**. Presentation morphology, SceneContract, 10D manifolds, HCF, and Vibe are allowed to be a *different* environment. A traditional WWW stack is one possible *view* of that environment, never its ceiling.

Having ontologies of WWW parts (HTML/RDFa, CSS properties, HTTP, format IRIs, …) is how you keep a **typed door** to the old world without living in it:

```
                    ┌── projector ──► HTML+RDFa / CSS-as-RDF / JSON-LD / PDF
HCF / CML / Q42 ────┤
 SceneContract      └── native paint ► Poet stage / wgpu / taffy / cosmic-text
```

The left-to-native path is the product. The left-to-WWW path is interop, print, agent windows, or Browse quarantine (§7). **Do not invert those.**

A projector is still three things (ontology ≠ bytes):

1. **Vocabulary** — typed classes/properties for the *target* (e.g. `cof:Section`, `css:marginTop`, XHTML/RDFa `role`).
2. **Mapping** — HCF/CML/SceneContract → those classes, with an explicit **loss policy**.
3. **Serializer** — emit bytes that conform to a **profile IRI**.

**Vocabularies that exist (WWW-side, for exits):**

| Piece | What it is | Where | In Qualia bundle? |
|---|---|---|---|
| **COF** | CML → constrained HTML5+RDFa (agent windows) | `tools/legislation-etl/cof.n3`, `cml_context/cof_html.rs` | yes (code) |
| RDFa + XHTML vocab | prefixes, `role`, `license` | `bundled/ontologies/w3c/rdfa.ttl` | yes |
| Format identity | `formats:RDFa`, Turtle, JSON-LD, … | `bundled/ontologies/w3c-archives/formats-overview.ttl` | yes |
| HTTP terms | status / methods as RDF | `bundled/ontologies/w3c/http.ttl` | yes |
| **CSS properties as RDF** | `http://www.w3.org/ns/css#` (`css:marginTop`, …) — camelCase as in the CSSOM | `C:\Projects\ontologies-2023\w3c archives\ns-main\w3c-ns\css.ttl` (~19 KiB) | **not bundled yet** |
| HCF block types | `Section` / `Paragraph` / `ReactiveCell` | HCF §3 | yes (spec); no HCF→HTML serializer |

`css.ttl` is Timothy’s evidence that “etc.” is real: CSS can be *described* in RDF the same way HTML structure can. It is **not** Qualia’s style language. Theme tokens, presentation levels, and SceneContract are. Bundling `css.ttl` is a catalog chore, not a reason to paint chrome with CSS.

**What those ontologies do *not* do:**

- They do not require the HID to implement a CSS cascade, a DOM, or a WHATWG content model.
- A complete element-level HTML ontology is a *larger Browse projector*, not a prerequisite for Poet.
- **Lossy both ways** on WWW exits. HCF → HTML/CSS cannot carry a live Vibe cell, a `.d10` mesh, an unencoded grapheme, or a deontic Aura except as RDFa hooks or degradation. HTML/CSS → HCF cannot promote layout and scripts into semantics.
- JSON-LD / Turtle are near-lossless *RDF views*. Markdown/PDF are print/text views.

**Rule for socialisation:** do not author or persist the HID in HTML/CSS/JS. You **may** emit those under a named profile when talking to the old world. You are **not confined** to them when inventing the new one. Do not save an emit back as the document.

The **preferred bridge** is not that emit. It is a **proportionate WASM agent** from the package family (§0.3).

### 0.3 The bridge is a WASM package family, not HTML+RDFa

Reverting to HTML+RDFa (COF) so a foreign host or an LLM can “see” Qualia is the old-world door. The door that stays *ours* is: ship a **WASM agent sized to the application**, plus CBOR-LD / Q42 data. The host runs the agent. The agent speaks `capability.invoke`, Pulse, and the graph. It does not ask the host to understand a DOM.

That only works if the WASM artefacts are a **named family**, not “whatever `wasm-full` happens to contain.”

```
Application (HCF + Vibe + required invoke ids)
        │  select smallest covering package
        ▼
WASM family member  +  CBOR-LD / Q42 snapshot
        │
        ├── foreign browser / Pages / ns.webcivics.net
        ├── another Webizen / edge box
        └── LLM host (tools/MCP), not a tokenised HTML page
```

**Already the compile-time family** — source of truth [`wasm_capabilities.rs`](../../crates/qualia-core-db/src/wasm_capabilities.rs), manual [`wasm-capability-profiles.md`](../manuals/wasm-capability-profiles.md):

| Member | Purpose | Includes (characteristic) | Excludes (must stay out) | Size honesty |
|---|---|---|---|---|
| **ontology-mcp** (`webizen-lite-wasm`) | Read-only ontology / ns sites; agent MCP | N3, bounded Quin query, SHACL, deontic/epistemic/LTL/DL/ASP | Portal, WebGPU, science, LLM, net, fs | 512 KiB raw / 200 KiB gzip gate |
| **portal** | Spatial / phenomenal pages | CBOR-LD ingest, Tensor10D, WebGPU viewport, AcousticPlane | Logic bridge, science, LLM | viewport product, not MCP |
| **logic** | RDF / rule demos | N3/Turtle, query, SHACL, modalities, LWW | Science, LLM | |
| **scientific** | Browser scientific playground | logic + WASM-safe bio/chem/clinical/econ/solvers | LLM | |
| **llm** | In-browser GGUF/Q42 decode | scientific + WebGPU inference | Portal | |
| **full / playground** | API explorer, local dev | union of the above | Native daemon / net / fs | **not** a shippable “app” |

Vibe already has **runtime** profiles (`native-desktop`, `wasm32`, later `native-mobile`, `edge`) that must not grow extra grammars. The WASM *family* is the **engine-capability** cut; Vibe profiles are the **language-host** cut. An application names **both**.

**Proportionate** means:

1. Collect the app’s required `capability.invoke` ids (and whether it needs WebGPU, MCP, ingest).
2. Choose the **smallest** family member whose `compiled_capabilities()` covers that set.
3. **Fail closed** if nothing covers it — do not silently promote to `wasm-full`.
4. Publish a **package manifest** (later RDF/CBOR-LD, same destination wire): member id, inclusions, exclusions, byte budget, Sentinel honesty, hashes, signature. vibe-core already reserved “Package manifest” as a later artifact.

That manifest *is* the bridge contract. A foreign host loads package + data. A native host may use the same ids in-process (no WASM). One capability table; two runtimes.

**What this replaces, and what it does not:**

| Temptation | Do this instead |
|---|---|
| COF HTML+RDFa so an LLM can read a law | ontology-mcp + `.q42` + MCP tools (already the lite-agent plan). COF remains a **text projector for token windows**, not the app bridge |
| “Dump the engine in the browser” | forbidden — 42 MB Sentinel; family exists for this reason |
| New ad-hoc `wasm-my-app` feature per QApp | select a family member; if a new member is needed, add it to `wasm_capabilities.rs` with purpose / include / exclude / size gate |
| HTML+CSS ontology as the interop language | optional §0.2 exit only |
| WASM Component / WIT as 0.1 | later packaging of the **same** family (consult: extensions = WASM components, never new keywords) |

**Honesty:** `webizen-lite-wasm` still speaks JSON-RPC at the MCP boundary (JS number safety via decimal strings). That boundary should move to **CBOR-LD** with the rest of the wire (§0.1 Q0.9). wasm-bindgen JS glue remains a *host* constraint until the host is native; it is not the agent’s language.

Normative language and grammar live in the Vibe/Poet specs. This file must not invent keywords or claim v1.0.

---

## 1. What is already in the tree (do not re-propose)

The Gemini notes describe a greenfield “Rust GUI vs Electron” fork. Qualia is already past that fork in several places.

### 1.1 Shell, chrome, language, engine

| Piece | What exists | Where |
|---|---|---|
| Desktop shell | **Tauri 2** (`tauri = 2.11.5`) — OS WebView, tray, updater, custom protocol | `crates/webizen-desktop` |
| Workbench UI | **Dioxus 0.8.0-alpha.1** with `router` + `web` — compiled for the WebView, **not** `dioxus-desktop` | `crates/webizen-studio` |
| Why not Dioxus desktop today | Studio Cargo.toml: *“dioxus is included with `router` only (not `desktop`) to avoid wry conflicts”* with Tauri’s wry | same crate |
| Poet HyperCanvas | `/poet` takes the window; manifolds, chest, move/resize | `crates/webizen-studio/src/components/poet/` |
| Instructional mock | HTML HyperCanvas (Gemini/Antigravity lane) | `C:\Projects\NLP\Canvas_Workbench` + [`consult/antigravity-hypercanvas-mock.md`](file:///C:/Projects/NLP/consult/antigravity-hypercanvas-mock.md) |
| Vibe 0.1 | Lexer / parser / check / eval, `capability.invoke`, no JIT | `crates/poet-vibe`, host in `qualia-core-db::poet_host` |
| Document NLP | Tokenize + gazetteer + spans **in the main engine crate** | `qualia-core-db::nlp` |
| 10D / volumetric draw | Shared wgpu 30 device, Tensor10D projector, mesh, bloom, caller-buffered RGBA8 | `qualia_core_db::render::gpu`, `crates/webizen-render` |
| WASM products | Explicit profiles (ontology MCP, portal, logic, scientific, LLM, full) — not one “dump the engine in the browser” | [`docs/manuals/wasm-capability-profiles.md`](../manuals/wasm-capability-profiles.md) |
| Inference | In-process GGUF / wgpu. **No Ollama.** | `llm_agent.rs`, `gguf_bridge.rs` |

So the live desktop is already a **hybrid**: Tauri owns the window and OS; Dioxus-web owns the chrome as WASM/HTML inside that WebView; wgpu owns scientific/volumetric paint on a **separate** GPU path (`/gpu-viewport`, `poet_render`). The expensive Chromium-class cost is the **WebView chrome**, not the absence of a GPU engine.

### 1.2 Normative Vibe / Poet documents (link these when socialising)

| Document | Role |
|---|---|
| [`docs/manuals/standards/vibescript-core.md`](../manuals/standards/vibescript-core.md) | **Normative** vibe-0.1 grammar, types, effects, bindings |
| [`docs/manuals/standards/vibescript-specification.md`](../manuals/standards/vibescript-specification.md) | Architectural essay only — not the parser contract |
| [`docs/manuals/ai-agent-vibescript-readiness.md`](../manuals/ai-agent-vibescript-readiness.md) | Agent overlay; core spec wins on conflict |
| [`docs/vibe/SKILL.md`](../vibe/SKILL.md) | Agent skill: diagnose, no overlay literals |
| [`docs/vibe/index.html`](../vibe/index.html) | Public hub |
| [`crates/poet-vibe/grammar/vibe-0.1.ebnf`](../../crates/poet-vibe/grammar/vibe-0.1.ebnf) | Machine grammar |
| [`docs/manuals/poet-ui-concepts.md`](../manuals/poet-ui-concepts.md) | Manifold / container / chest vocabulary |
| [`docs/plans/poet-hypercanvas-tools-and-containers-2026-08-16.md`](poet-hypercanvas-tools-and-containers-2026-08-16.md) | Inventory already ported or scheduled |
| [`docs/manuals/standards/hypermedia-content-format-hcf.md`](../manuals/standards/hypermedia-content-format-hcf.md) | Authoring format (yaml-ld-q42 / CBOR-LD) |
| [`docs/plans/n-dimensional-portal-engine-implementation-plan.md`](n-dimensional-portal-engine-implementation-plan.md) | n-D HID spine (product language: mindware, not “portal”) |
| [`docs/plans/presentation-morphology-and-multisensory-display.md`](presentation-morphology-and-multisensory-display.md) | Presentation continuum |
| [`docs/manuals/wasm-capability-profiles.md`](../manuals/wasm-capability-profiles.md) | What may enter a browser binary |

Consult inputs that already shaped 0.1 (do not treat as competing specs):

- `C:\Projects\NLP\consult\branding.md` — Poet = engine, Vibe = language, Pulse = transport, Aura = ontology  
- `C:\Projects\NLP\consult\20260815_codex.md` — closed grammar, no raw overlays  
- `C:\Projects\NLP\consult\20260815_qapp-script-gemini.md` — three-tier UI / DSL / compiled engine  

### 1.3 Quick wins — performance **and** more engine employed

§5 is the **WebView-exit** sequence (months). This subsection is the **next few sessions**: work that makes the HID **faster** *because* it stops faking capabilities Qualia already has.

The first draft of this paper under-specified that intersection. The bottleneck today is not “no GPU” and not “wrong JS framework.” It is **chrome that does not call the host**, so every desk either (a) ships **JSON** through the WebView, or (b) draws a placeholder and the kernel never runs. Cutting (a) is not “use a faster JSON.” It is **stop putting the graph on a JSON wire** — handles now, **CBOR-LD** when a value must move.

**Rule of a quick win:** it must do **both** —

1. cut a crossing (IPC, JSON serialize, second adapter, `read_to_string`, JS math), **and**
2. put a real `capability.invoke` or `poet_eval` / `nlp.analyze` / `Render.scene` on a Poet container.

If it only paints prettier HTML, it is mock work (Antigravity). If it only rewrites wgpu, it is not quick.

#### Already bound — employ these, do not reimplement

`ids::ALL_BOUND` (2026-08-16) already reaches, among others:

`CapabilityDiscovery.{list,coverage,catalog}` · `SHACL.{validate,extensions}` · `GraphDatabase.{stats,sparql}` · `Deontic.eval` · `Epistemic.eval` · `Paraconsistent.route` · `Ltl.{globally,finally}` · `DL.subsumes` · `ASP.enumerate` · `Nlp.analyze` · `Document.ingest` · `Sheet.{stats,sum_range}` · `Manifold.{distance,axes,project}` · `Geom.hull2` · `Render.scene` · `Social.lww` · `ClinicalRisk` (Framingham) · `Hash.iri` · `Crypto.sha256` · math/stats/science kernels.

Desktop doors that already exist and are **under-used by HyperCanvas**:

| Door | File | Honesty |
|---|---|---|
| `poet_eval` | `webizen-desktop/src/commands/poet.rs` | live on in-process `PoetSnapshot` (Partial vs daemon) |
| gazetteer | same + `nlp::analyze_document` | live, year-one |
| `poet_render_preview` | `commands/poet_render.rs` | live via `webizen-render` / shared GPU |

Coverage backlog (do not rustdoc-scrape): `capability.invoke("CapabilityDiscovery.coverage")` and `scripts/vibe-coverage.ps1`.

#### Q0 — this week (hours–2 days each)

| # | Win | Faster because | More Qualia because |
|---|---|---|---|
| Q0.1 | **Code container = `poet_eval`** | no JS interpreter, no IPC of a fake REPL | every bound invoke becomes reachable from a desk |
| Q0.2 | **Doc toolbar → `Nlp.analyze` / gazetteer** | spans computed natively; no regex-in-JS | document NLP actually marks the text |
| Q0.3 | **Sheet formula → `Sheet.stats` / `Sheet.sum_range`** | no JS reduce over cells | Statistics / sheet kernels run |
| Q0.4 | **Ontology / graph pane → `SHACL.validate` + `GraphDatabase.sparql`** | query stays on the native snapshot | SPARQL-star / SHACL door is the app, not a sidebar orphan |
| Q0.5 | **Map / spatial body → `Manifold.project` + `Geom.hull2` + `Render.scene`** | no SVG-as-GIS; GPU path already paid | 10D projector is visible on the Research manifold |
| Q0.6 | **Health body → Framingham on the reference profile** | one invoke, not a form theatre | clinical engine is employed (consent still ⚑) |
| Q0.7 | **Discovery strip in status / chest** | one catalog fetch, not N rustdoc pages | humans see what is bound vs MCP-only |
| Q0.8 | **Handles, not blobs, across Tauri** | preview is a texture / path / hash, not base64 RGBA | `poet_render_preview` stays on the shared device |
| Q0.9 | **`poet_eval` args/result as CBOR-LD or a handle** | no `serde_json` of Quins / AST through WebView | the *wire* starts matching HCF; Vibe stays on the native AST |

Q0.1–Q0.4 are the highest leverage for *employment*. Q0.8–Q0.9 are the highest leverage for *the destination pair*. They do not need taffy, Boa, or a new crate.

#### Q1 — this fortnight (still before native chrome)

| # | Win | Faster because | More Qualia because |
|---|---|---|---|
| Q1.1 | **Keep `PoetSnapshot` on the native side** | graph never cloned into WASM as JSON Quins | SPARQL / deontic / LTL run where the arena is |
| Q1.2 | **Epistemic / strata tools set fields** (no `alert`) | less JS; one write | Epistemic.eval can filter the same field |
| Q1.3 | **Extract `tools/vibe` from the console** | one eval path | inventory D23 |
| Q1.4 | **LaTeX wrap → `Symbolic.evaluate`** | CAS stays in Rust | SymbolicAlgebra is on a desk |
| Q1.5 | **Rights chip → `Deontic.eval`** | no second policy toy | deontic norms show on the container |
| Q1.6 | **Status: invoke id + µs + honesty + E\*** | you see cost | E300/E400 become HID, not only REPL |
| Q1.7 | **One GPU** (already D12) — refuse a second adapter in studio | no device thrash | render + inference keep the shared device |
| Q1.8 | **Ingest `--job-dir` / `--url`** (landed, not in the *running* OSM binary) | no second copy of a dump; resume | Q42 is how ontologies arrive |
| Q1.9 | **Pulse / ingest sidecar: CBOR-LD option** | `--progress json` stays for humans; machine path is CBOR-LD | Pulse already has CBOR envelopes (`chat_mesh`); sidecars stop being a second JSON dialect |
| Q1.10 | **HCF persist path: yaml-ld-q42 in, CBOR-LD out** | no “save as HTML / JSON-LD” for Poet docs | authoring/wire split in HCF §3 becomes a file, not a slogan |

#### Q2 — do **not** call these quick wins

| Tempting | Why it fails the dual test |
|---|---|
| Swap Tauri for `dioxus-desktop` | Same wry/WebView; Cargo already forbids the clash |
| Embed TinyMCE / a chart NPM | Slower *and* blocks gazetteer / HCF / Vibe cells |
| “Keep JSON-LD as the wire because tools know JSON” | Tools can transcode; the *product* wire is CBOR-LD. JSON-LD is a *view*, not the bus |
| COF HTML+RDFa as the *app* interop surface | §0.3: proportionate WASM family + CBOR-LD. COF is a token-window projector only |
| Ship `wasm-full` “so it works everywhere” | Breaks 42 MB Sentinel; pick the smallest covering member |
| Boa / QuickJS / Servo this month | WWW-VM (§7), not HID; months, not a desk. Interpreting JS is the opposite of replacing it |
| “Full engine WASM in Studio” | Breaks 42 MB Sentinel; profiles exist for a reason |
| Auto-wrap every rustdoc item as invoke | D17: coverage process, not a generator |
| Second wgpu device “for the UI” | Fights inference and `/gpu-viewport` |
| Pretty CSS in the mock as if it were perf | Zero kernel time saved |

#### How this relates to §5

```
Q0/Q1  employ what is bound; stop JSON Quins; start CBOR-LD / handles
   │
§5.B   Poet stage = SceneContract (pixels follow the invokes)
   │
§5.C   native chrome (drop WebView — and its JS glue — for Poet only)
   │
§5 wire  yaml-ld-q42 persist + CBOR-LD Pulse / AST (parallel to chrome)
```

Q0/Q1 make §5 cheaper: when the stage goes native, the **same invoke ids** keep working, and the wire is already the destination pair instead of “native pixels, JSON bus.” If we skip Q0 and jump to taffy, we will native-paint **empty** desks. If we native-paint but keep JSON IPC, we have left Chromium and kept the other half of the WWW pair.

#### Measurement (or it is not a win)

For each Q0 item, record in the Poet progress log:

- invoke id  
- p50 / p95 host time (not “feels faster”)  
- bytes across Tauri (before → after; target: handle, not payload)  
- encoding of those bytes: JSON (debt) vs CBOR-LD (destination) vs handle (best)  
- honesty label on the container (`live` only if the kernel ran)

Do not extrapolate a kernel µs to end-to-end. Do not mark `live` if the UI still stubs.

---

## 2. Tauri v2 vs Dioxus — refined for Qualia

The Gemini comparison is directionally right and **wrong about this repository** in one important place.

### 2.1 Generic distinction (keep)

| | Tauri v2 | Dioxus |
|---|---|---|
| Philosophy | Polyglot shell: any web frontend + Rust host | One language: `rsx!` virtual DOM |
| Desktop paint today (typical) | OS WebView (WebView2 / WKWebView / WebKitGTK) | Same **wry/tao** stack if you use `dioxus-desktop` |
| Web | Deploy the JS/WASM frontend | First-class `dioxus-web` → WASM |
| IPC | JS ↔ Rust commands/events | Native desktop: Rust drives the DOM without a second app language. Web: still WASM↔JS for DOM. |

Dioxus desktop is **not** “no WebView.” It is “no second UI language.” The pixels of ordinary chrome still go through a WebView unless you adopt an experimental native renderer (Blitz / similar). wgpu in Qualia is already that native path for **manifolds and volumes**, not for menus.

### 2.2 What Qualia actually compiled

```
webizen-desktop (Tauri 2, native Rust)
    └── WebView
            └── webizen-studio (Dioxus 0.8 web + router) as WASM/HTML
                    ├── Classic routes (default)
                    └── /poet HyperCanvas
    └── shared wgpu device (render::gpu / webizen-render) — not the WebView compositor
```

Consequences:

1. **We already use both.** Dropping Tauri for `dioxus-desktop` would swap one wry owner for another and re-open the wry-conflict the Cargo.toml already documents. That is not a win by itself.
2. **Dioxus-web is already a WASM app** — it is how Studio ships inside the desktop shell and how Pages can host instructional surfaces. §3 is “yes, and here is the profile discipline.”
3. The **Chromium-class tax** is WebView2 (Windows) / system WebKit (elsewhere), plus whatever JS the Dioxus renderer still emits at the WASM↔DOM boundary. It is **not** “we bundled Electron.”
4. The **10D tax** is already paid on the wgpu side. The remaining gap is **chrome and document layout**, not “we have no GPU.”

### 2.3 When to keep the hybrid (now)

Keep Tauri + Dioxus-web while:

- Classic must remain the default HID;
- HyperCanvas is still being designed in the NLP mock;
- ingest, Q42, and Vibe 0.1 host work consume the machine;
- a native text/layout stack (taffy / cosmic-text / vello) is not yet a product surface.

That is an **increment**, not a loyalty to Chromium.

---

## 3. Yes — Dioxus can be a WASM app (and already is)

Two Dioxus web shapes, mapped to Qualia products:

| Shape | Meaning here | Honest use |
|---|---|---|
| **dioxus-web SPA** | Entire chrome in WASM, static host | Studio-in-WebView; optional Pages **instructional** builds. Do **not** point public Pages at `0.0.31-dev` wholesale. |
| **dioxus-fullstack** | Same crate, server functions + WASM client | Not the Qualia model. Graph, Sentinel, and keys stay in-process or on the principal’s machine. Do not invent a “Vibe cloud.” |

Workspace rule that already exists and must stay: **WASM is a product profile**, not a leftover of the native graph. `wasm_capabilities.rs` and the ontology MCP size gate (raw / gzip) are the pattern. A “full engine in the browser” is how you ship a 42 MB Sentinel you cannot honour.

Dioxus WASM still **cannot touch the DOM without JS**. Diffs stay in Rust; patches cross `wasm-bindgen`. That is fine for chrome. It is the wrong bus for 10D tensors — those already go `NQuin` / `Tensor10D` → wgpu.

---

## 4. How large is a custom Qualia presentation library?

Gemini’s 3–6 / 9–18 month bands assume a **blank** GPU + window + text stack. Qualia is not blank.

### 4.1 Already composed (do not rebuild)

- **wgpu 30** shared device, capability-intersected f16 / subgroup / timing  
- **Tensor10D** projector, depth, bloom, mesh, picking, RGBA8 readback  
- **WGSL Forge** — typed kernels, Naga, CPU oracle, adapter-keyed cache  
- **P64 / GGUF** weight path  
- **Zero-heap Tier-1** contract and 42 MB Sentinel  
- **Vibe 0.1** interpreter and `capability.invoke` families  

A “custom library for Qualia-DB” that **reimplements** those is waste. The job is a **presentation crate** that consumes them.

### 4.2 The remaining job (honest split)

| Layer | Job | Suggested primitives | Size if we compose |
|---|---|---|---|
| Window / events | Already Tauri | keep until a native chrome exists | done |
| Chrome layout | Flex/grid for menubar, chest, pager | taffy | weeks–months, not years |
| Text | Documents, LaTeX later, CJK/unencoded graphemes | cosmic-text; HCF grapheme path is **our** problem | months (linguistic honesty > fonts) |
| 2D vectors | Tool chrome, wires, sheets | vello or femtovg on the **same** wgpu device | months |
| 10D / volume | Exists | extend projector; do not add a second adapter | ongoing |
| Event routing / a11y / IME | The hard UI toolkit | only as Poet needs it | the 9–18 month piece if you insist on “full toolkit” |

**Recommendation:** do **not** schedule “a Qualia UI toolkit.” Schedule **Poet chrome leaves the WebView first**, then **document bodies**, then **optional WWW VM**. Each leave is a crate with a SceneContract, not a monorepo rewrite.

### 4.3 Improvements that are real (and already half-paid)

These are not marketing. They are consequences of the two-tier zero-heap model and the existing GPU path.

1. **Zero-copy (or one-copy) manifold → VRAM.** `Tensor10D` / Super-Quin slices are `repr(C)`. A WebView forces JSON/IPC or a canvas putImageData. Native already skips that for `/gpu-viewport`.
2. **Compute for projection, not for CSS.** SO(n) / chart maps / geodesic colouring belong in WGSL Forge kernels with a CPU oracle — the Forge already forbids “repair the shader text in a loop.”
3. **Deterministic frames.** No V8 GC in the paint path once chrome is native. WebView chrome will always jitter; scientific panes should not wait on it.
4. **RAM.** WebView2 idle is hundreds of MB. Native wgpu + taffy chrome can sit in tens of MB. Measure; do not quote blog numbers as ours until we measure this tree.
5. **Startup.** Tauri + WASM hydrate is “app-like.” A native Poet window can be “tool-like.” Both can coexist (Classic in WebView, Poet native) during the strangler.

What a custom pipeline does **not** automatically give: accessibility, IME, print, “it runs the New York Times.” Those are WWW-VM features (§7), not HID features.

---

## 5. Incremental replacement (biggest wins first)

No public Qualia/Webizen release exists that we must not break. That is permission to be **modern**, not permission to land a ten-year browser in one PR.

**Rule:** each phase ships a product the principal can dogfood. Classic remains. Poet may switch presentation backends behind the same manifold/container ids. **Chrome exit and wire exit run in parallel.** Leaving the WebView while still speaking JSON is only half the replacement.

```
Phase A  Dual-path paint (now → next)
         WebView chrome + wgpu scene   ← we are here
         Wire: JSON IPC (debt) + handles on GPU path
              │
Phase B  Native scene contract for Poet desks
         Same ids; stage pixels from wgpu/vello, not HTML
         Wire: poet_eval / Pulse as CBOR-LD or handle (Q0.9 / Q1.9)
              │
Phase C  Native chrome (menubar, chest, pager)
         taffy + cosmic-text; WebView — and wasm-bindgen JS — shrink to Classic
              │
Phase D  Document bodies leave the DOM
         HCF: yaml-ld-q42 authoring, CBOR-LD persist, cosmic-text + entity spans
         Vibe cells evaluate; no contenteditable / no JS editor
              │
Phase E  Optional WWW compatibility VM
         Only if Timothy wants “run a React app inside Webizen”
         That VM speaks JS *in quarantine*. The HID never adopts it as Vibe.
```

### Phase A — Dual path (current)

**Win:** 10D / mesh / bloom already off the DOM.  
**Do:** keep `/gpu-viewport` and `Render.scene` as the only honest 3D/4D path. Stop drawing fake WebGL cubes in the mock as if they were the engine.  
**Don’t:** start a second wgpu instance from Dioxus.

### Phase B — Poet stage is a SceneContract

**Win:** container `x,y,w,h`, wires, and spatial bodies share the renderer the engine already certifies.  
**Do:** one `SceneContract` per manifold (already the n-D HID plan). Dioxus keeps being the **editor of the contract** (or Vibe does), not the rasterizer.  
**Effort:** weeks if we reuse `webizen-render`; months if we insist every toolbox widget is a GPU node on day one.

### Phase C — Native chrome

**Win:** drop WebView **for Poet**. Classic can stay on Tauri WebView until deprecated.  
**Do:** taffy layout of menubar / pager / chest / status; input via winit or Tauri’s window without a page.  
**Don’t:** rewrite Classic in taffy “while we’re here.”

### Phase D — Documents (HCF, not HTML)

**Win:** gazetteer spans, unencoded graphemes, Vibe cells — none of this wants `contenteditable`.  
**Do:** cosmic-text + our span Quins (`text_span`). Persist **yaml-ld-q42** (author) and **CBOR-LD** (wire/HMC). Rich-text `ToolbarSpec` already exists as data.  
**Don’t:** embed TinyMCE or a JS editor “temporarily.” Don’t save Poet documents as HTML or JSON-LD and call that HCF.

### Phase E — WWW VM (optional, last)

Only if the product must **host other people’s websites**. That is a browser. Qualia is allowed to be a **mindware HID that can open a website**, not a website that pretends to be a HID. See §7.

---

## 6. Vibe instead of JavaScript

**Yes — replacing JS with Vibe is part of the point** (§0.1). This section is the *how*, not a maybe.

Gemini asked for paradigm, concurrency, native types, and engine. Those answers already exist for **0.1**. What 0.1 refused is becoming a **JS clone** (DOM, `eval`, event loop, JIT). What the product still owes is **Vibe as the language apps are written in**, with a **CBOR-LD AST** on the wire (essay §5.A; vibe-core artifact table: Canonical AST → later RDF/CBOR-LD).

A “JS killer” that hosts React is the Browse VM (§7). A HID that authors Vibe and speaks CBOR-LD is the product.

### 6.1 What 0.1 already decided (do not reopen casually)

From [`vibescript-core.md`](../manuals/standards/vibescript-core.md):

| Question | 0.1 answer |
|---|---|
| Paradigm | Typed, capability-bounded, mostly functional cells + staged modules. Not OOP, not “a better JS.” |
| Concurrency | Host-dispatched `on pulse` / `on ui`. No V8 event loop. No JIT (iOS-safe). |
| Native types | Graph + RDF 1.2 terms + `capability.invoke`. **No** raw `<<[ … ]>>` overlays. 10D / tensors arrive as **bindings**, not grammar. |
| Engine | Pure Rust AST interpreter (`poet-vibe`). Same crate on native and wasm32. |
| Effects | Pulse / Aura / graph write are explicit. Cells (`= …`) are Pure. |
| Interchange | Source is UTF-8 `.vibe`. Canonical AST is versioned nodes, **later RDF/CBOR-LD**. |

The consult Gemini note that said “qapp-script should not try to replace JS” was right **as a 0.1 scope cut**: do not implement `document`, promises-as-the-bus, or a compatibility grammar. It was **wrong if read as product architecture**. Timothy’s aim — a native environment **not compromised by old-web compatibility** — is the HID destination: **apps are Vibe; the wire is CBOR-LD.** A later numbered profile (suggested name: `vibe-ui-0.2` or `vibe-hid-0.2`) **adds bindings and host hooks**, not a fork of the grammar, and does not reintroduce JavaScript as the authoring language.

### 6.2 What a HID profile should add (suggestions)

These are the “significant improvements” that actually match the engine:

1. **Scene as a value.** `Render.scene` / manifold axes already invoke. A UI profile should treat a `SceneContract` as a first-class Vibe value (handle, or CBOR-LD, **not** a JSON blob).
2. **Caller-buffered numeric types.** `vec4`, `mat4`, later `tensor10` as **capability types** that map to `repr(C)` slices — never `Vec<f64>` in a hot loop.
3. **Budgets as language.** 0.1 already has E400. Surface them in the Poet status bar (gas, Sentinel, 42 MB).
4. **No DOM in the standard library.** `document.createElement` is a WWW-VM polyfill (§7), not Vibe.
5. **One invoke table.** New paint ops are `capability.invoke("Render.*")` / `Manifold.*`. Do not add keywords per widget.

### 6.3 What Vibe must not become

- A second Webizen VM opcode set  
- A Chromium content script  
- A claim of v1.0 before parser-backed conformance (0.1 is already that bar)  
- A reason to stop implementing engine families that `capability.invoke` already names  

Vibe is **how humans and apps reach Qualia** (the language that replaces JS here). CBOR-LD is **how those programs and documents move** (the wire that replaces JSON here). The presentation stack is **how Qualia reaches the principal’s eyes and hands**. They meet at `capability.invoke`, Pulse, and `SceneContract`.

---

## 7. Optional WWW compatibility VM (not a constraint)

Timothy’s aim: eradicate V8 **and** still run traditional WWW apps somehow.

Those are **two products**:

| Product | Constraint | Stack |
|---|---|---|
| **Webizen HID / Poet** | Modern, ours, no old-web veto | Vibe + CBOR-LD + SceneContract + wgpu + (later) taffy |
| **Webizen Browse** | Other people’s HTML/JS | Today: system WebView. Later: Boa/QuickJS + taffy + vello + fetch/canvas polyfills — **or** keep the OS WebView as a **container type** (`webview`) |

The Gemini “strangler fig” (Boa → taffy → vello → polyfill `document`) is a reasonable **Browse** roadmap. It is a **poor** HID roadmap: you would spend the next year recreating the DOM so React can run, while Poet still cannot paint a manifold.

**Biggest win that still allows WWW:** keep `webview` / `portal` containers as **hosted WebViews** (or a future light VM) **inside** a native Poet stage. The stage is native; the alien page is quarantined. That matches Permissive Commons vs Sanctuary thinking: untrusted markup does not own the process.

Boa (pure Rust, ~1.0-class in 2026) is the right **experiment** if we ever interpret untrusted JS. It is the wrong interpreter for Vibe.

---

## 8. Questions worth circulating (⚑ principal)

1. Is **Poet-native chrome** (Phase C) more urgent than **document bodies** (Phase D), or the reverse?  
2. Must **Classic** stay on WebView forever, or only until a written deprecation?  
3. Is **WWW-in-a-container** enough, or is a full Browse VM a product?  
4. First persistence for rich text: HCF / yaml-ld-q42 + CBOR-LD vs transitional markdown (already on the tools inventory).  
5. May a `vibe-hid-0.2` profile add scene/tensor **types** without new keywords?  
6. First CBOR-LD cut: `poet_eval` IPC, Pulse frames, or HCF persist? (Recommendation: IPC + Pulse before persist — Q0.9 / Q1.9.)  
7. May agent **diagnose** stay JSON as a *view* while the canonical AST is CBOR-LD?  
8. Is the present WASM family (ontology-mcp / portal / logic / scientific / llm / full) enough, or is a **Poet-HID** member needed (SceneContract + Vibe, no LLM, no science)?

---

## 9. Suggested next concrete steps (when the machine is free)

None of these require stopping OSM ingest or rewriting Tauri this week.

1. Keep Gemini on `Canvas_Workbench` + `IMPLEMENT.md` ([Antigravity brief](file:///C:/Projects/NLP/consult/antigravity-hypercanvas-mock.md)).  
2. Keep Grok porting **ids and behaviour** into `poet/`, binding `capability.invoke`.  
3. When ready to spend a phase: **SceneContract for the Poet stage** (Phase B) — highest leverage, reuses wgpu.  
4. In the same window: **`poet_eval` / Pulse as CBOR-LD or a handle** (Q0.9 / Q1.9) so the wire matches HCF instead of growing more JSON.  
5. Do **not** schedule Boa, Servo, or “replace WebView2” until B is dogfoodable. Do **not** add a JS interpreter to “help” Vibe.  
6. Measure WebView vs a trivial native window RAM/startup on this machine before quoting §4.3 numbers as ours.

---

## 10. One-paragraph summary for reviewers

Qualia’s desktop is already Tauri 2 + Dioxus-web (WASM in a system WebView) + a real wgpu 10D/volumetric engine. That stack is a **temporary host**. The destination pair is **CBOR-LD on the wire** and **Vibe as the language** (HCF: end of XML, HTML, and JS bloat; authoring remains yaml-ld-q42). The HID is **not confined** to HTML/CSS. The **bridge** to a foreign host or agent is a **proportionate WASM family member** plus CBOR-LD / Q42 (§0.3), not HTML+RDFa. Dioxus `rsx!`, Tauri JSON commands, and wasm-bindgen glue are not the product language or the product wire. The **quick** wins are Poet containers calling `poet_eval` / `capability.invoke` / `Nlp.analyze` / `Render.scene` that are **already bound**, with graphs and frames as **handles** (then CBOR-LD, never JSON Quins). Leaving Chromium-class chrome (§5) and leaving JSON IPC are the same replacement, done incrementally, *after* those desks are live (§1.3). Vibe 0.1 reaches the engine without being a JS clone; a later HID profile owns scene/tensor values still without becoming JavaScript. A WWW VM is an optional Browse product that may interpret *other people’s* JS in quarantine. Classic stays until explicitly deprecated.
