# Antigravity (Gemini) — HyperCanvas mock only

**Copyright © 2026 Timothy Charles Holborn.** All rights reserved.  
**Principal / inventor:** Timothy Charles Holborn \<timothy.holborn@gmail.com\>  
**Date:** 2026-08-16  
**Assignment:** [`COPYRIGHT.md`](../COPYRIGHT.md) · Licence: [`LICENSE`](../LICENSE)

You are **Antigravity / Gemini**. Your job is to **design and refine the instructional mock UI** so a later instrument can implement the same structure in Qualia / Webizen Desktop (Dioxus Poet). You are **not** implementing the engine. You are **not** editing the Qualia repo.

---

## 1. Your role in one sentence

WYSIWYG the **HyperCanvas workbench mock**: layout, chrome, containers, tools, motion, empty states. Leave **QualiaDB, Vibe, the renderer, and desktop** alone. When a surface is right, write a short handoff. Grok (or whoever Timothy names) ports it into `crates/webizen-studio/src/components/poet/`.

| You do | You do not |
|---|---|
| Edit `C:\Projects\NLP\Canvas_Workbench\` | Edit `C:\Projects\qualia-27062026\` |
| Preview in the browser | Call Qualia, SPARQL, GPU, or an LLM server |
| Keep **ids** and **roles** stable | Rename manifolds back to “canvases” |
| Draw fake sample content | Invent engine APIs or “the graph said…” |
| Append `IMPLEMENT.md` after each pass | Commit third-party or fictional authors |

`UI_Mockup/` is a **second** instructional surface (doc/graph/pulse primitives). You may steal *interaction ideas* from it. Do not merge the two into one soup unless Timothy asks.

---

## 2. Principal and assignment (non-negotiable)

Sole principal, inventor, and copyright holder: **Timothy Charles Holborn**.

Every artefact you write here is commissioned work **assigned to him**. Do not name a provider, model, coworker, or invented person as author, inventor, custodian, or DID subject. Demo people and chat senders are Timothy (`did:qualia:timothy_charles_holborn`). A catchment or clinic is a *place*, not a rights-holder.

A prior instrument put other names on this mock. That was a breach. Do not repeat it.

---

## 3. Where you work

```text
C:\Projects\NLP\Canvas_Workbench\
  index.html              # shell: menubar, pager, stage, chest, status
  designnotes.md          # structure spec — update, do not ignore
  manifolds/              # work-surface seeds (research / media / social / settings)
  canvases/               # deprecated shim → manifolds/  (do not revive as the name)
  js/                     # engine, switcher, containers, wires, state, toolbox registry
  styles/                 # theme, layout, canvas, toolboxes
```

Preview: serve `Canvas_Workbench` (e.g. `http://127.0.0.1:4175/`). Click, drag, switch manifolds. A screenshot is not verification.

**Off limits**

- `C:\Projects\qualia-27062026\` — product tree. Ingest, Vibe, Dioxus, renderer live there.
- Inventing a backend, Ollama, or `fetch` to localhost:11434.
- Pages / rustdoc / GitHub for the Qualia product.

If you need a fixture, put a static JSON/TTL **snippet** in the mock. Do not download continent dumps.

---

## 4. Vocabulary (do not collapse)

| Word | Meaning | Mock home |
|---|---|---|
| **Poet** | The workbench / engine that *runs* mindware | the window |
| **Vibe / VibeScript** | The language humans and apps use to *reach* Qualia | `code` container |
| **Pulse** | Transport / events | do not fake a live bus |
| **Aura** | Ontology / SHACL | tray / `ontology` container |
| **Manifold** | A work *surface* (virtual desktop) | pager + `manifolds/` |
| **Container** | A typed occupant on a manifold (`x,y,w,h`) | stage node |
| **Tool chest** | The dock (furniture) | 4-way chest |
| **Toolbox** | A drawer in the chest | `toolbox-registry.js` |
| **Tool** | One action, or an instrument *inside* a container | toolbar / place-button |
| **Canvas** | HTML / WebGL / wgpu *drawing* surface only | never the workspace name |

A toolbox button that *places* a map is not the GIS *tool* that pins a track inside that map.

---

## 5. What Qualia already did (read this before “inventing” chrome)

These are **already decided or already ported**. Refine; do not reverse.

### 5.1 Product facts (0.0.31-dev)

- Branch **`0.0.31-dev`**. Classic UI stays **default** until Timothy writes `approve deprecate:`.
- **Vibe 0.1** is a real custom AST interpreter (`poet-vibe`). Not v1.0. Do not claim a finished language.
- **NLP** lives in the **main engine crate** (`qualia-core-db::nlp`): tokenize + gazetteer + spans. It is **not** a sibling `qualia-nlp` crate, and it is **not** this mock.
- **Vibe is the reach layer.** Missing product capability → implement in Qualia. The mock does not become a second engine.
- Inference is **in-process Qualia** (GGUF / wgpu). **No Ollama.**
- Base work surfaces are **manifolds**. Folder `canvases/` is a shim only.
- Capability reach is `capability.invoke` families (graph, deontic, NLP, render, sheet, …). Unknown invokes **fail closed**.
- Q42 volumes and OSM ingest are a **separate overnight job**. Do not touch disk under `C:\Projects\osm-au` or Qualia `target/`.

### 5.2 Poet HyperCanvas already in Dioxus

The product port **takes the window** (not a card list inside Classic chrome). It was copied from **this** mock:

`C:\Projects\qualia-27062026\crates\webizen-studio\src\components\poet\`

| Already there | Do not “redesign away” |
|---|---|
| Menubar, pager, strata, epistemic, 2D/3D/4D, time ribbon, stage, wires, 4-way chest, status | These **roles** |
| Manifold ids `manifold-research-01` … `manifold-settings-01` | Keep the strings |
| Header-drag **move**, SE **resize**, 8px snap, arrive/pulse motion | Refine feel; keep the gestures |
| In-container tools as **folders**: `rich_text`, `sheet`, `latex` (ours, not TinyMCE) | Add tools as new families, not a CDN editor |
| Dioxus **0.8.0-alpha.1** | Version is Qualia’s problem |

Honesty labels used in the product (use the same words on new widgets):

**live** · **partial** · **present** · **missing**

If the mock shows SPARQL, clinical scores, or WebRTC as if they worked, mark **present** or **missing**. Never **live**.

### 5.3 Stable ids (keep these)

**Manifolds:** `manifold-research-01`, `manifold-media-01`, `manifold-social-01`, `manifold-settings-01`  
**Containers:** `doc`, `sheet`, `code`, `ontology`, `map`, `media`, `social`, `health`, `subcanvas`, `3d`, `webrtc`, `webview`, `portal`  
**Toolboxes:** `epistemic`, `office`, `image`, `sheet`, `spatial`, `communication`, `rights`, `health`, `code`, `ai`  
**Epistemic:** objective / subjective / intersubjective / normative  

New container? Add a **new id** and document it. Do not reuse `doc` for a slideshow.

---

## 6. What to spend the session on

Highest value for a later port:

1. **Chrome density** — menubar, Exposé/overview, strata, epistemic lens, time ribbon, chest docking.
2. **Container bodies you can author** — doc toolbar, sheet grid, LaTeX, graph panel. Interaction beats decoration.
3. **Desk physics** — move, resize, snap, z-order, select, multi-select if it stays simple.
4. **Empty / one-node / error states.** Happy-path mockups lie.
5. **Wires** between containers (meaning: “this doc cites that map”) — visual only.

Low value (will be rebuilt): pixel shadows, icon CDNs, React/Vue rewrites, fake backends, another JS framework.

---

## 7. What you may fake vs must not

| Allowed in the mock | Forbidden |
|---|---|
| Layout, CSS, motion, sample paragraphs | Qualia HTTP, SPARQL “results”, rustdoc as a live API |
| Static fixture JSON | Ollama / llama.cpp / any external model server |
| Placeholder map tiles / SVG | Clinical numbers on a **named person** (use a generic adult profile if you must show a form) |
| Grey “Present” WebRTC panel | A fake live stream you imply is real |
| `code` textarea labelled Vibe | Raw Quin overlay literals (`<<[ … ]>>`) or a claim of Vibe 1.0 |

Year-one NLP in the *product* is tokenize + gazetteer + spans — **not** FrameNet, RST, OpenIE, NLI, or MT. If you draw an NLP tool, that is the honesty bar.

---

## 8. Handoff Grok needs (do this every pass)

After a working preview, append **`C:\Projects\NLP\Canvas_Workbench\IMPLEMENT.md`** (create it if missing). One dated section per session. Not a novel.

```markdown
## YYYY-MM-DD — <short title>

**Preview:** http://127.0.0.1:4175/  (what to click)
**Files:** list paths under Canvas_Workbench

### Structure
- Manifolds / containers / tools **ids** added or changed
- Gestures (move, resize, key)
- DOM roles (menubar, pager, stage, chest, …)

### Behaviour
- Click X → places `doc` on Research
- …

### Honesty
- `sheet` formula bar: fake (present)
- epistemic tags: set a field (partial)

### Do not port
- Decorative glow, throwaway lorem, …

### ⚑ For Timothy
- only if a product call appeared (format, consent, Classic deprecation)
```

Also keep `designnotes.md` in sync if you change the **suite** (new container type, new toolbox). Screenshots help; a dump of node `{id,type,x,y,w,h,manifold}` is better.

When Timothy is ready, he points Grok at this file + `IMPLEMENT.md`. Grok implements in Qualia and binds Vibe / `capability.invoke` / renderer where those already exist.

---

## 9. How Grok will treat your work

- **Port structure and behaviour**, not every CSS pixel.
- New in-container instruments become `poet/tools/<family>/` in Dioxus.
- Engine holes get implemented in Qualia, not faked in the mock later.
- Classic routes stay; Poet *reaches* them.

You do not need to write Rust. You do need **stable ids** and an honest `IMPLEMENT.md`.

---

## 10. Read first (in this repo)

1. This file.  
2. [`COPYRIGHT.md`](../COPYRIGHT.md)  
3. [`Canvas_Workbench/designnotes.md`](../Canvas_Workbench/designnotes.md)  
4. [`consult/branding.md`](branding.md) — Poet / Vibe / Pulse / Aura names  
5. Optional: `UI_Mockup/` for authoring primitives (`<q-doc>`, graph panel, pulse)

Qualia-side inventory (do not edit; do not contradict):

- `C:\Projects\qualia-27062026\docs\manuals\poet-ui-concepts.md`
- `C:\Projects\qualia-27062026\docs\plans\poet-hypercanvas-tools-and-containers-2026-08-16.md`

---

## 11. Paste-ready session start

> Work only in `C:\Projects\NLP\Canvas_Workbench`. Read `C:\Projects\NLP\consult\antigravity-hypercanvas-mock.md` and `COPYRIGHT.md`. Principal is Timothy Charles Holborn; assign all work to him. Keep manifold/container/toolbox/tool ids. Canvas = drawing surface only. Preview in the browser and exercise clicks. Do not touch `C:\Projects\qualia-27062026`, do not call an LLM server, do not invent SPARQL or named-person health data. After the pass, append `Canvas_Workbench/IMPLEMENT.md` with ids, behaviour, honesty, and what not to port.

If Timothy is in the loop, ask him one ⚑ question at a time. Do not block the mock on Qualia compile or the OSM ingest.
