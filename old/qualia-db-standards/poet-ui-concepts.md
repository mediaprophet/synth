# Poet UI concepts — manifolds, tool chest, containers

**Copyright © 2026 Timothy Charles Holborn.** All rights reserved.  
**Status:** product vocabulary for Poet chrome. Classic UI stays default (D7).  
**Instructional mock:** `C:\Projects\NLP\Canvas_Workbench\designnotes.md`  
**Container ontology:** [`qualia-ui/ontologies/container.n3`](../qualia-ui/ontologies/container.n3) (N3 authoring → CBOR-LD runtime)

This is how the workbench is named. HTML `<canvas>` / wgpu / Canvas2D stay **canvas** (drawing surfaces).

---

## 1. Five kinds of thing (many types of each)

| Kind | What it is | Many of… |
|---|---|---|
| **Manifold** | A work *surface* (virtual desktop or canvas). Same word as the engine 10D manifold. Can contain sub-manifolds. | Research, Media, Social, Settings, Vibe, … |
| **Container** | A typed *occupant* placed on a manifold. Every container has a **ContainerKind**: `content`, `panel`, or `widget`. | doc, sheet, code, map, inspector, aura-tray, mini-map, … |
| **Tool chest** | The *furniture* that holds toolboxes. One chest per workbench, dockable. | (usually one) |
| **Toolbox** | A *themed drawer* inside the chest. | Vibe, Documents, Sheets, Spatial, Social, Rights, Health, Epistemic |
| **Tool** | One *action* in a toolbox (often "place this container"). | + Document, Run cell, Gazetteer, … |

A **tool chest** is not a toolbox. The chest is the rack; a toolbox is a drawer; a tool is what you pick up.

A **manifold** is not a container. A manifold is the surface; containers are what sit on it. A manifold may contain another manifold (sub-manifold), but most occupants are containers.

Legacy mock folder `toolboxes/` is the chest's contents. Do not rename HTML/GPU canvas modules here (D14).

---

## 2. Manifolds

Switchable surfaces, like a pager of virtual desktops. A container on one manifold can target another (**sub-manifold**). Manifolds can nest — a sub-manifold is a manifold placed inside another manifold.

| Id | Role | Default seed |
|---|---|---|
| `research` | GIS, clinical, rights alignment | map + graph panel + aura tray (right dock) |
| `media` | `.10d` kinematics, grapheme, acoustic | 3D mesh + media + inspector (right dock) |
| `social` | chat graphs, live peers, field notes | social + pulse panel (bottom dock) |
| `settings` | capabilities, fiduciary VM, sub-manifolds | property sheet + capability badge |
| `vibe` | VibeScript console + diagnose (human door into Qualia) | code + inspector (right dock) + status bar (bottom) |

Pager: Overview + numbered surfaces + `+`. Shortcuts later: Alt+1…

---

## 3. Containers

Placed by tools. Every container has a **ContainerKind** that discriminates its role:

| ContainerKind | Role | Examples |
|:---|:---|:---|
| **content** | Holds media (documents, sheets, maps, code, 3D, audio…). Media = collective of medium. | doc, sheet, code, map, media, graph, social, health, … |
| **panel** | Holds UI chrome — inspectors, property sheets, outlines, tool palettes. Panels **observe** content containers. | inspector, property-sheet, outline, tool-palette, aura-tray, pulse-panel, graph-panel |
| **widget** | Small UI elements — lightweight, usually read-only or single-action. | mini-map, status-bar, breadcrumb, progress-indicator, capability-badge |

### 3.1 Content containers

| Type | Honesty today | Notes |
|---|---|---|
| `code` | live | Vibe eval / diagnose / `Render.scene` |
| `doc` | live (gazetteer) | year-one NLP, not FrameNet |
| `sheet` | live (range stats via invoke) | CML cell→N3 later |
| `ontology` | partial | `SHACL.extensions` live; no shape-IRI registry yet |
| `map` | live | hull + `Manifold.project` + `webizen-render` |
| `media` | live | kinematics poses → `Render.scene`; swapchain on `/gpu-viewport` |
| `social` | live | LWW + peer hash; ring is the renderer contract (no fake chat) |
| `health` | live | Framingham on a **reference adult profile**, not a named person |
| `graph` | missing | SPARQL, `quin.statement`, RDF 1.2 |
| `latex` | missing | SymbolicAlgebra, CAS wasm |
| `triad` | missing | q42↔p64↔d10, `qualia-audio` |
| `pulse` | missing | `pulse.publish` allowlist |
| `finance` | missing | Black–Scholes, portfolio |
| `slide` | missing | office toolbox "presentation" |
| `anatomy` | missing | Wellfair anatomy `RenderScene` (consent) |
| `listen` | missing | `qualia-audio`, EnCodec |
| `vision` | missing | `ComputerVision.ahash`, `qualia-vision` |
| `submanifold` | partial | nested `Manifold.axes` + same scene contract |
| `webview` `webrtc` `portal` | present | not placed yet; do not fake streams |

### 3.2 Panel containers

| Type | Role | Observes |
|---|---|---|
| `inspector` | Inspects the active content container's semantic structure | active content container |
| `property-sheet` | Edits properties of the selected element | active content container |
| `outline` | Structural overview (document outline, sheet tree, scene graph) | active content container |
| `tool-palette` | Floating/docked quick-access tools from the active toolbox | (none — palette is independent) |
| `aura-tray` | Ontological shape & schema inspector (SHACL, validation results) | active content container |
| `pulse-panel` | Collaborative stream & telemetry monitor | (none — subscribes to Pulse) |
| `graph-panel` | Interactive RDF / knowledge graph preview (read-only) | active graph or content container |

Panels **dock** to a manifold edge (left, right, top, bottom) or float.

### 3.3 Widget containers

| Type | Role | Attached to |
|---|---|---|
| `mini-map` | Zoomed-out overview of manifold layout | the manifold itself |
| `status-bar` | Budget, gas, Sentinel state, connection status | (manifold-level) |
| `breadcrumb` | Navigation path through manifold hierarchy | (manifold-level) |
| `progress-indicator` | Long-running operation progress | active operation |
| `capability-badge` | Visual Sentinel indicator for active container/tool scope | active container or tool |

Widgets are lightweight — they do not hold media and do not have their own tool-chains.

---

## 4. Tool chest

Dock: left / right / top / bottom (left first). Each toolbox icon opens that drawer. Tools **place containers** or **run Vibe** — they do not `alert()`.

Honesty labels on tools that are not wired (Present / Partial / Missing).

---

## 5. Vibe's place

Vibe is how humans and apps *reach* Qualia from a **code** container (and the Vibe manifold). It is not a manifold type of its own forever — it is a container kind that can sit on any manifold.

NLP is a **doc** tool, not a Vibe keyword.

---

## 6. Container ontology

The container ontology is authored in **N3** and compiled to **CBOR-LD** for runtime. See:

- **Authoring source:** [`qualia-ui/ontologies/container.n3`](../qualia-ui/ontologies/container.n3)
- **Compiled runtime:** `qualia-ui/ontologies/container.cbor` (produced by `n3-to-cbor` build tool)

The ontology defines:

- `manifold:Manifold` class with `contains`, `subManifoldOf`, `activeContainer` properties
- `container:Container` base class with `kind`, `title`, `honesty` properties
- `container:ContentContainer`, `container:PanelContainer`, `container:WidgetContainer` subclasses
- Specific container types (Document, Sheet, Code, Map, Inspector, AuraTray, MiniMap, …)
- Relationships: `observes` (panel → content), `attachedTo` (widget → container), `dockPosition`
- Manifold seeds (initial layouts for research, media, social, settings, vibe)

---

## 7. Presentation

Containers are rendered through a **presentation context** — the negotiated result of matching what a container *wants* against what the machine *can do* and what the user *needs*. This is analogous to responsive design, but multi-axis and ontology-driven.

**Ontology:** [`qualia-ui/ontologies/presentation.n3`](../qualia-ui/ontologies/presentation.n3) (N3 authoring → CBOR-LD runtime)

### 7.1 Five axes

| Axis | What it describes | Examples |
|:-----|:------------------|:---------|
| **Display modality** | How content is rendered | 2D, 3D, 4D (time), AR, VR, audio-only, tactile, agent-readable |
| **Form factor** | Physical display class | mobile, desktop/laptop, large-format (TV/projector), head-mounted, embedded |
| **Accessibility** | Access modalities for humans and agents | screen-reader, high-contrast, reduced-motion, font-scaling, switch-control, voice-control, captioning, audio-description, colour-blind-safe, seizure-safe, agent-accessible |
| **Internationalisation** | Locale & cultural settings | language tag (BCP 47), text direction (LTR/RTL/TTB), calendar, number/date/currency format, timezone |
| **Hardware capacity** | Machine rendering tier | minimal, low, mid, high, enthusiast, agent — gated by GPU, VRAM, RAM, CPU, network, max texture size |

### 7.2 Negotiation flow

```
Container Presentation Hints        Machine Profile
  (what it wants)                     (what it can do)
       │                                   │
       └──────────┬────────────────────────┘
                  ▼
         Presentation Context
          (negotiated result)
```

1. Start with the container's **preferred modality**.
2. If the machine's `supportedModalities` doesn't include it, walk the **fallback chain** (e.g. 4D → 3D → 2D → audio-only → agent-readable).
3. Check **minimum hardware tier** — if the machine is below, use the fallback chain.
4. Apply **accessibility features**: if the user's profile requires a feature the container doesn't support, warn but degrade gracefully.
5. Select **locale**: match the user's language tag against the container's supported locales. If no match, use the container's default locale.
6. Produce a **PresentationContext** with the negotiated modality, form factor, accessibility profile, locale, and hardware tier.

### 7.3 Modality degradation chain

```
4D → 3D → 2D → audio-only → agent-readable
VR → 3D
AR → 3D
```

Every machine supports `agent-readable` — it is the floor. No container is ever unrenderable; it degrades to a CBOR-LD data stream that an agent (or a very minimal UI) can consume.

### 7.4 Agent accessibility

Software agents are a first-class access modality, not an afterthought. `ModalityAgentReadable` means the container's content is serialised as CBOR-LD for programmatic consumption. This is the same wire format used for HCF persistence and Pulse events — no separate "API view" is needed.

---

## 8. Composition & Provenance

Manifolds and containers are **two states of the same thing**: a **Work Artifact**. A document, a film, a software project, a research corpus — each is a Work Artifact that can be *open* (a manifold, editable, composed of sub-parts) or *flat* (a container, published derivative, self-contained).

**Ontologies:**
- [`qualia-ui/ontologies/container.n3`](../qualia-ui/ontologies/container.n3) §8–10 — WorkArtifact, state transitions, composition graph
- [`qualia-ui/ontologies/provenance.n3`](../qualia-ui/ontologies/provenance.n3) — contributors, sources, constituencies, derivative chains, credits

### 8.1 State duality

```
     ┌─────────────┐                    ┌─────────────┐
     │   Manifold   │  ── flatten ──→   │  Container   │
     │   (open)     │  ←─ expand ───    │  (flat)      │
     │  editable    │                    │  published   │
     │  composed    │                    │  derivative  │
     │  of parts    │                    │  self-contained │
     └─────────────┘                    └─────────────┘
            │                                  │
            │           sourceManifold          │
            └──────────────────────────────────┘
                  (link back for re-expansion)
```

- **Flatten** — a manifold is composed into a single self-contained artifact (a derivative). The derivative format may be PDF, MP4, XLS, DOC, website, HCF, CBOR-LD, or Q42 DAG.
- **Expand** — a flat container is decomposed back into a manifold for editing. The `sourceManifold` link allows re-expansion without losing the composition structure.
- The artifact identity persists across transitions. Flattening and re-expanding does not create a new artifact — it changes the state of the same one.

### 8.2 Composition graph

Work artifacts form a **DAG** (directed acyclic graph), not a tree:

- A manifold may contain sub-manifolds (recursive nesting).
- A container may `embed` other containers (composition within a flat artifact — e.g. a document embedding a sheet).
- A container may `reference` a manifold it is not derived from (a live link, not a copy — e.g. a published report that embeds a live data manifold).
- An artifact may be `partOf` multiple parents (shared components — e.g. a shared ontology used in two documents).
- Cycles are forbidden — the graph is acyclic.

### 8.3 Provenance graph

Every work artifact carries its own **provenance graph** — a record of all contributors, data sources, transformations, and derivative relationships over time. This is like film credits, a software dependency tree, or a git log, but per-artifact and machine-readable.

**Actor types:**

| Actor | Examples |
|:------|:---------|
| **Human** | Authors, editors, reviewers, directors — identified by DID |
| **Agent** | NLP pipelines, LLMs, symbolic extractors, gazetteers — identified by type + version |
| **Organisation** | Clinics, research groups, standards bodies |
| **Data source** | SNOMED CT, GeoNames, sensor feeds, corpora — identified by IRI + license |

**Contribution roles** (modelled on film credits + software contribution):

- **Authoring:** author, co-author, editor, contributor
- **Extraction (agents):** extractor, annotator, normalizer, validator
- **Production:** director, producer, composer
- **Source/supply:** dataSource, libraryProvider, infrastructureProvider
- **Rights/review:** reviewer, rightsHolder, fiduciary

Each contribution entry records: actor, role, timestamp, byte span (for text-level contributions), confidence (for agent-extracted content), and content hash.

### 8.4 Constituencies

A constituency is a group of actors who have a stake in an artifact — not necessarily contributors, but affected parties:

| Constituency | Example |
|:-------------|:--------|
| **Data subject** | Patients whose data appears in a clinical document |
| **Rights holder** | Parties who hold rights over embedded content |
| **Stakeholder** | Parties affected by the artifact's content or use |
| **Audience** | Intended audience for the artifact |
| **Community** | A community represented or referenced in the artifact |

Constituencies may require **consent** before an artifact can be published or shared. The fiduciary Sentinel gates this.

### 8.5 Derivative chains

When a manifold is flattened into a container, and that container is later transformed (translated, rendered, re-published), each step forms a **derivative chain** — a DAG of transformations from the original source to the current artifact. Each step records the input artifact, output artifact, transform type, and actor.

### 8.6 Credits

A **Credits** summary is generated from the provenance graph — not authored independently. Like film credits or a colophon, it lists the actors and their roles in display order. This is what appears on a published derivative (e.g. the last page of a PDF, the credits screen of an MP4, the footer of a website).

---

## 9. Agency — Entities, Actors, Contracts, Claims

Provenance records **what happened**. Agency records **whether it was authorised**. Both are needed — provenance is honest (records everything), agency is normative (evaluates validity).

**Ontology:** [`qualia-ui/ontologies/agency.n3`](../qualia-ui/ontologies/agency.n3) (N3 authoring → CBOR-LD runtime)

### 9.1 Entity vs Actor

An **entity** is the thing that exists. An **actor** is an entity performing actions in a specific circumstance. The same entity can be multiple actors — a person can be an employee of one company and a contractor for another, each with different scope.

| Entity type | Examples | Identified by |
|:------------|:---------|:--------------|
| **Natural person** | Authors, editors, reviewers, fiduciaries | DID |
| **Legal person** | Companies, institutions, organisations | Registration IRI (ABN, ACN, ORCID, ROR) |
| **Software entity** | NLP pipelines, LLMs, symbolic extractors | Type + version + instance IRI |
| **Collective entity** | Communities, groups, unincorporated associations | IRI |

### 9.2 Circumstance

Every actor operates within a **circumstance** — a bounding context that scopes when the actor's agency is valid:

- **Temporal scope** — valid from / valid until
- **Topical scope** — the domain (e.g. clinical-research, financial-audit, content-authoring)
- **Subject scope** — the specific matter (e.g. a patient ID, a project ID, a document IRI)
- **Relational scope** — the relationship (e.g. employee-of, contractor-for, delegate-of)

An agency claim is only valid within its circumstance.

### 9.3 Agency

An **agency** relationship exists when an actor (the agent) acts on behalf of a principal entity, under a contract that defines permitted purposes and scope.

```
  Agent (Actor)  ─── acts on behalf of ───→  Principal (Entity)
       │                                          │
       └──────── under Contract ──────────────────┘
                    within Circumstance
```

### 9.4 Contract

A contract defines what the agent is authorised to do on behalf of the principal:

| Contract type | Example |
|:--------------|:--------|
| **Explicit** | Signed employment contract, service agreement, NDA |
| **Implicit** | Employment law, fiduciary duty, terms of use |
| **Statutory** | Privacy law, professional licensing, mandatory reporting |
| **Delegation** | Sub-delegation from an existing agency |
| **Machine policy** | Capability scope, access control, Sentinel rules |

Each contract specifies:
- **Permitted purposes** — what the agent is authorised to do
- **Prohibited purposes** — what the agent is explicitly not authorised to do, even if it might seem within scope
- **Contract rules** — constraints the agent must follow (e.g. "must not share data with third parties", "fiduciary duty of care")

### 9.5 Claim & Validity

A **claim** is an assertion of agency: "I act on behalf of X under contract Y for purpose Z". Claims are **evaluated**, not assumed.

| Validity | Meaning |
|:---------|:--------|
| **valid** | Claim is consistent with contract, circumstance, and principal's rules |
| **invalid** | Claim is not consistent — the actor is not authorised |
| **unverified** | Claim has not yet been evaluated |
| **contested** | Claim has been challenged — authority is disputed |
| **expired** | Claim was once valid but the circumstance has ended |
| **ultraVires** | Actor is acting beyond the scope of their authority — the action is not permitted by the contract or principal, even if the agency relationship itself is valid |

The **ultraVires** case is the one you described: a worker doing something not permitted by their organisation. The agency relationship may be real (they are an employee), but the specific action is outside the scope of what they're authorised to do.

### 9.6 Delegation chains

Agency can be delegated recursively. Each link in the chain is its own agency with its own contract and circumstance:

```
  Software Agent  ← contracted by ←  Company  ← used by ←  Human Agent of Company
       │                    │                      │
    Contract A          Contract B            Contract C
   (machine policy)    (explicit)           (employment)
```

- `delegatedBy` links each agency to its parent agency
- `delegationDepth` records how many levels deep
- `originalPrincipal` identifies the root entity at the top of the chain

A delegation chain is only valid if every link is valid. If any link is broken (expired contract, ultraVires action), the entire chain is compromised.

### 9.7 Linking provenance to agency

When a contribution is recorded in the provenance graph, the actor references an `agency:Actor`. That actor has an `agency:Agency` relationship to a principal entity, governed by a contract. When the contribution is made, a **Claim** is evaluated:

1. Does the claimed contract exist and is it active?
2. Is the claimed purpose in the contract's permitted purposes?
3. Is the claimed purpose not in the prohibited purposes?
4. Does the actor's circumstance match (topic, subject, relation)?
5. Is the delegation chain intact?

If the claim is invalid or ultraVires, the contribution is **still recorded** (provenance is honest — it records what happened, not what should have happened), but the Sentinel may flag it, gate it, or require remediation.

---

## 10. Hypermedia Documents

The `office` toolbox is the primary toolbox for authoring, editing, and publishing hypermedia documents. It is the native equivalent of Word/Pages/Docs — but built on Vibe, CBOR-LD, and the context graph.

**Ontology:** [`qualia-ui/ontologies/document.n3`](../qualia-ui/ontologies/document.n3) (N3 authoring → CBOR-LD runtime)
**Full toolbox spec:** [`TOOL_CHEST_SPEC.md` §13](../qualia-ui/tool-chest/TOOL_CHEST_SPEC.md)

### 10.1 Document types

Documents are classified by type, which determines required datasource bindings, citation requirements, and temporal status support:

| Type | Datasources | Citations | Temporal | Example |
|:-----|:------------|:----------|:---------|:--------|
| Research | research-corpus, gazetteer, ontology | required | yes | Academic research with checked sources |
| News article | public-stats, news-wire | required | yes | Article relying on stats that may change |
| Legal | legislation, case-law (jurisdiction-scoped) | required | yes | Legal document in a specific jurisdiction |
| Academic paper | academic-corpus | required | no | Formal paper with peer-review |
| General | (none) | optional | no | Free-form document |
| Form | (varies by field) | no | no | Interactive form with submission |

### 10.2 Templates

Templates define structure, styling, and datasource bindings. They can be:
- **Supplied** — system-provided, shipped with the toolbox
- **Custom** — user-created from an existing document
- **Community** — shared via a template registry

A template is itself a WorkArtifact — it can be versioned, provenanced, and shared.

### 10.3 Context markup

Context markup is the core differentiator from traditional word processors. Specific text spans can be semantically linked to sources:

- **Terms** → ontology entries
- **Entities** → knowledge graph nodes
- **Claimed facts** → supporting/refuting sources
- **Statements** → provenance and assertion context
- **Statistics** → datasource with temporal status (created-at vs present)
- **Citations** → academic or legal sources
- **Definitions** → ontology or glossary
- **Quotes** → source utterance

These markup nodes form the document's **context graph** — a knowledge graph embedded in the document. The graph can be appended to by audiences (3rd parties, contributors) and may be updated over time (e.g. refreshing statistics).

### 10.4 Temporal status

For datasource-linked content (especially statistics), the document tracks two snapshots:
- **Created-at status** — frozen at document creation time (what the author saw)
- **Present status** — live refresh of the datasource (what the current state is)

This allows a reader to see both what the author referenced when writing and whether the underlying data has changed since.

### 10.5 Publishing

A document manifold can be flattened to multiple derivative formats:

| Target | Media | Live links | Context graph | Interactive |
|:-------|:------|:-----------|:--------------|:------------|
| Print (PDF) | yes | no | no | no |
| Webpage | yes | yes | yes | yes |
| Ebook (EPUB) | yes | no | no | no |
| HCF Bundle | yes | yes | yes | yes |
| Markdown | no | no | no | no |
| Plain text | no | no | no | no |

HCF Bundle is the canonical publishing format — fully interactive, includes all hypermedia assets, live datasource links, context graph, and provenance.

### 10.6 Forms

Forms are interactive document elements with typed input fields, validation rules, and submission workflows. Fields can bind to datasources (e.g. entity-select from a knowledge graph) and ontologies (constraining valid values). Submission may require agency claims and constituency consent.

### 10.7 Toolchains

The `office` toolbox has 9 tool-chains (78 tools total):

- **formatting** (15) — bold, italic, headings, lists, alignment, fonts, colours
- **structure** (9) — sections, tables, footnotes, citations, TOC, hyperlinks
- **context-markup** (12) — tag terms/entities/facts/statistics, link markup, append, temporal refresh
- **media-embed** (9) — embed images, audio, video, sheets, graphs, maps, 3D, forms, manifolds
- **template** (7) — select type, apply/create templates, add sections/styles/bindings, validate
- **datasource** (6) — bind/unbind, query, refresh, list, set jurisdiction
- **publish** (7) — print, webpage, ebook, HCF, markdown, plain, view credits
- **review** (7) — track changes, comments, compare versions, request review
- **forms** (6) — insert form, add fields, set validation, bind datasource, set workflow, submit

---

## 11. Code, AI & Spatial Toolboxes

Three toolboxes serve QualiaDB / Webizen / human-centric application (QApp) development. They are **loosely coupled** to containers — the same tool works across different container and manifold types.

**Ontology:** [`qualia-ui/ontologies/code.n3`](../qualia-ui/ontologies/code.n3) (N3 authoring → CBOR-LD runtime)
**Full toolbox spec:** [`TOOLBOX_CODE_SPEC.md`](../qualia-ui/tool-chest/TOOLBOX_CODE_SPEC.md)

### 11.1 `code` toolbox — VibeScript & QApp development

The native IDE for Vibe. 6 tool-chains, 37 tools:

- **vibe** (6) — console, eval, diagnose, syntax-check, format, outline
- **quin** (5) — statement, inspect, ref, reify, validate (RDF-Star graph statements)
- **project** (8) — new, add/remove module, build, test, package, deploy, inspect manifest
- **hcf** (5) — author, compile, validate, inspect, extract (Hypermedia Content Format)
- **n3-ontology** (7) — author, compile, validate, import, prefix-check, load, query
- **capability** (6) — declare, inspect sentinel, test, grant, revoke, audit

QApp types: Document, Spatial, Agent, Social, Library. Build targets: WASM, Native, HCF, CBOR-LD.

### 11.2 `ai` toolbox — Symbolic + Neural + Neuro-Symbolic

The full AI stack — not just LLMs. 4 tool-chains, 31 tools across 4 layers:

- **symbolic** (10) — gazetteer, FST, coref sieve, frame semantics, temporal/geo/quantity parsers, relation extractor, build gazetteer/FST
- **neural** (7) — embedder, transformer, classifier, reranker, constrained decoder, load/unload model
- **bridge** (6) — substrate extractor, graph indexer, GraphRAG, grounding verifier, feedback loop, inspect substrate
- **agent** (8) — plan, execute, verify, trace, load golden corpus, evaluate, sentinel inspect/gate

The neuro-symbolic bridge is the core innovation: symbolic AI produces verified substrates, neural models consume them for grounded inference, and outputs feed back into the symbolic graph for verification.

### 11.3 `spatial` toolbox — 10D Browser & 3D Design

Scene graph construction, mesh manipulation, rendering, and 10D manifold navigation. 5 tool-chains, 41 tools:

- **scene** (10) — create, add/remove node, set tensor/mesh/material, add light/camera, link semantic, duplicate
- **mesh** (8) — import, export, edit vertex, subdivide, decimate, compute normals, add rig, add blend space
- **render** (6) — render scene, set viewport, clear colour, post-process, capture frame, set render budget
- **manifold-nav** (10) — navigate spatial/temporal/quantum/topology/manifold/spectral, focus node, orbit, set manifold camera, set spectral perception
- **design** (7) — place UI element, snap to grid/surface, measure, align, group, set spatial layout

The "10D" refers to the Q42 10D volumetric tensor `[q, v, w, x, y, z, t, α, μ, σ]` — a 40-byte, zero-heap, stack-allocated structure. Dimensions are: structural/quantum (q epistemic context, v topological class, w manifold domain), spacetime (x, y, z, t), and spectral-logical (α amplitude, μ modulation, σ EMF spectral signature). The spectral dimensions use the **EMF spectrum**, not colour space alone — agents that perceive more spectrum (UV, IR, radio, X-ray) access σ directly without human-range clamping. Normative reference: `q42-10d-tensor-standard.md` v1.2.

### 11.4 Loose coupling

Tools are not hard-coded to containers. The registry resolves availability based on container kind and toolbox capability scope. For example, `vibe-eval` works in a `code` container (primary), a `doc` container (embedded code block), a `scene` container (scene script), or a `graph` container (query script).

---

## 12. Investigation & Forecast Toolboxes

Two toolboxes serve investigation and forecasting work — forensic reconstruction, evidence-based hypothesis testing, and forward-looking scenario modelling. They are **loosely coupled** to containers.

**Ontology:** [`qualia-ui/ontologies/investigation.n3`](../qualia-ui/ontologies/investigation.n3) (N3 authoring → CBOR-LD runtime)
**Full toolbox spec:** [`TOOLBOX_INVESTIGATION_SPEC.md`](../qualia-ui/tool-chest/TOOLBOX_INVESTIGATION_SPEC.md)

### 12.1 Investigation modes

- **Forensic** — backward-looking. Reconstructs what happened from evidence. Chain of custody, verification, timeline reconstruction, causal analysis.
- **Forecasting** — forward-looking. Projects what may happen based on patterns and models. Scenario generation, probability estimation, risk assessment.
- **Hybrid** — both. Reconstructs past events AND projects future outcomes. Example: fraud investigation that reconstructs the fraud and forecasts future risk exposure.

### 12.2 `investigate` toolbox — Forensic Investigation

7 tool-chains, 70 tools:

- **case** (11) — new investigation, set mode/status/jurisdiction, add subject/topic/event/case/constituency, query, export findings
- **evidence** (8) — collect, set reliability, verify, link source, tag, query, redact, compare
- **custody** (5) — init, transfer, record transform, verify, history
- **hypothesis** (7) — propose, set status, evaluate evidence, set mutually exclusive, query, summary, rank
- **hypothesis-graph** (25) — create graph, add to graph, contribute evaluation/confidence revision, bridge dark link, reframe/merge/split hypothesis, flag/close gap, merge/reject contribution, adjudicate conflict, create/diff/rollback revision, query graph/gaps/contributions/bridges, compute confidence, rank, visualise, subscribe/unsubscribe updates, export
- **timeline** (7) — create, add entry, set time precision/track, query, detect sequences, reconcile
- **link-analysis** (7) — add link, link evidence, query, find path, detect clusters, compute centrality, visualise

Evidence types: documentary, testimonial, physical, digital, sensor, analytical, circumstantial. Reliability ratings: confirmed, likely, possible, doubtful, disputed, fabricated. Chain of custody with transfer records and transformations.

The **hypothesis-graph** tool-chain is the living, versioned, multi-agent core. Multiple agents contribute evaluations, confidence revisions, and dark link bridges over time. Each contribution is provenance-tracked and may be merged, conflicting, or superseded. The graph dynamically updates as new information arrives — confidence scores shift, gaps close, and dark links are confirmed or refuted.

### 12.3 `forecast` toolbox — Scenario Modelling & Prediction

4 tool-chains, 27 tools:

- **scenario** (9) — create, set probability, add driving factor/outcome, set base, branch, query, compare, merge
- **trend** (7) — detect trends, fit model, forecast, detect anomalies, seasonal decompose, correlate, import timeseries
- **risk** (6) — assess, build matrix, identify tail risks, set threshold, monitor, report
- **causal** (5) — build model, query path, intervene (do-calculus), counterfactual, validate

Scenario types: best case, worst case, most likely, black swan, branching. Projected outcomes with metrics, values, units, and confidence ranges.

### 12.4 Investigation manifold seed

| Container | Dock | Toolbox |
|:----------|:-----|:--------|
| `case-board` | centre | investigate |
| `evidence-board` | left | investigate |
| `hypothesis-tree` | right | investigate |
| `timeline` | bottom | investigate |
| `link-graph` | centre (tab) | investigate |
| `scenario-board` | centre (tab) | forecast (hybrid) |
| `forecast-chart` | bottom (tab) | forecast (hybrid) |
| `constituency-panel` | right (panel) | both |
| `custody-log` | left (panel) | investigate |

### 12.5 Hybrid workflow

Hybrid investigations use both toolboxes in sequence: reconstruct (investigate) → understand (investigate) → project (forecast) → assess (forecast) → report (both).

---

## 13. Research Toolbox

The `research` toolbox serves structured enquiry — defining scope and constraints, building corpora, studying social/socio-economic/spatio-temporal dynamics, inferring dark links, drawing conclusions through inference chains, and synthesising findings. It is **loosely coupled** to containers and **loosely related** to the investigation toolboxes.

**Ontology:** [`qualia-ui/ontologies/research.n3`](../qualia-ui/ontologies/research.n3) (N3 authoring → CBOR-LD runtime)
**Full toolbox spec:** [`TOOLBOX_RESEARCH_SPEC.md`](../qualia-ui/tool-chest/TOOLBOX_RESEARCH_SPEC.md)

### 13.1 Scope & constraints

Research projects define their own scope (temporal, spatial, social, topical, methodological) with an ontology that constrains valid vocabulary. Constraints (ethical, legal, resource, methodological, access, temporal) bound the research process.

### 13.2 Purpose types

8 purpose types: exploratory, explanatory, evaluative, generative, confirmatory, integrative, critical, applied. Each purpose may bind to an ontology that constrains its vocabulary and valid claims.

### 13.3 `research` toolbox — 8 tool-chains, 75 tools

- **enquiry** (10) — new research, set purpose, define scope, add constraint, add/link questions, set status, link investigation, query
- **corpus** (10) — add item, import literature/dataset/web, set confidence, tag, query, deduplicate, extract NLP, annotate
- **dynamics** (10) — define social/economic/spatio-temporal dynamics, link dynamics, query, analyse network/inequality/diffusion, overlay, detect cross-dynamic patterns
- **dark-links** (9) — infer, detect provenance gaps/concealment patterns, link evidence, set confidence, query, confirm, refute, trace provenance
- **inference** (7) — make, chain, set confidence, query, validate, trace chain, compare
- **synthesis** (9) — create finding, set confidence, mark contested, link to question, create synthesis, add finding, query findings/syntheses, export
- **provenance** (5) — trace, verify, detect breaks, compare, export report
- **bootstrap** (15) — assess deficit, identify frontier, generate/refine/confirm/reframe/disprove/supersede/promote bootstrap hypotheses, record enrichment, query enrichment history/capabilities, suggest next enrichment, query frontier, identify unknown-unknowns

### 13.4 Dynamics — social, socio-economic, spatio-temporal

Three dynamic families, each with sub-types:

- **Social:** networks, structures, power relations, cultural norms
- **Socio-economic:** markets, inequality, capital flows, access patterns
- **Spatio-temporal:** migration, diffusion, environmental change, urban dynamics

Cross-dynamic links may use a **different ontology** than either dynamic they connect — e.g. a social-economic ontology for links between social networks and market structures.

### 13.5 Dark links

4 dark link types: **hidden** (deliberately concealed), **latent** (not yet explicit), **suppressed** (once visible, now obscured), **inferred** (hypothesised from patterns). Dark links use inference methods (network analysis, provenance gap detection, pattern matching, counterfactual reasoning, NLP extraction, temporal/spatial correlation, narrative inconsistency). Each dark link may use a different ontology than visible links.

### 13.6 Research manifold seed

| Container | Dock | Notes |
|:----------|:-----|:------|
| `research-board` | centre | Primary workspace |
| `corpus-browser` | left | Corpus management |
| `inference-graph` | right | Inference chains |
| `question-tree` | right (panel) | Research questions |
| `dynamic-map` | centre (tab) | Spatio-temporal dynamics |
| `dark-link-explorer` | centre (tab) | Dark link inference |
| `synthesis-view` | centre (tab) | Synthesis |
| `literature-review` | left (tab) | Literature |
| `data-canvas` | left (tab) | Dataset exploration |
| `provenance-trace` | bottom (panel) | Provenance chains |
| `constraint-panel` | bottom (panel) | Research constraints |

### 13.7 Relationship to investigation

Research is exploratory/generative; investigation is reconstructive/predictive. Research may inform investigation hypotheses; investigation evidence may enrich research corpora. The `relatedInvestigation` property links them loosely — each can exist independently.

---

## 14. Epistemics Toolbox

The `epistemics` toolbox evaluates the epistemic status of claims, observations, behaviours, and expressions — whether they are grounded in objective reality, subjective perspective, intersubjective consensus, or fictional construction. It is **agent-centric**: any entity (human, software agent, sensor, organisation, AI model) can be an epistemic agent with a perspective. It considers spatio-temporal and social relations as context that shapes what is subjective vs objective.

**Ontology:** [`qualia-ui/ontologies/epistemics.n3`](../qualia-ui/ontologies/epistemics.n3) (N3 authoring → CBOR-LD runtime)
**Split modules:** [`agent-nomenclature.n3`](../qualia-ui/ontologies/agent-nomenclature.n3), [`ungrounded-generation.n3`](../qualia-ui/ontologies/ungrounded-generation.n3)
**Nomenclature rules:** [`qualia-db-standards/agent-nomenclature-rules.md`](agent-nomenclature-rules.md)
**Full toolbox spec:** [`TOOLBOX_EPISTEMICS_SPEC.md`](../qualia-ui/tool-chest/TOOLBOX_EPISTEMICS_SPEC.md)

### 14.0a Agent nomenclature isolation

The epistemics toolbox follows strict agent nomenclature isolation (see [`agent-nomenclature-rules.md`](agent-nomenclature-rules.md)). Three agent types have distinct terminology:

- **Natural agent** (human, animal): mind-dependent terms — cognition, perception, belief, emotion, instinct, habit, hallucination, deliberate, coerced, negligent, reckless, mistaken, fabrication (if deliberate)
- **Legal person** (company, institution): institutional terms — decision-making process, observation, institutional position, routine, procedural error, misrepresentation (if deliberate through agents)
- **Software agent** (AI model, sensor, program): computational terms — computation, signal acquisition, model assertion, default behaviour, context allocation, enumeration failure, ungrounded generation, overridden

Mind-dependent terms must never be used for software agents. Intent-dependent terms (fabricated, deceptive) must never be used for software agents. Records are appendable — any agent may clarify the nature of an axiom at a future datetime.

### 14.1 Epistemic modes

6 modes: **objective** (verifiable, perspective-independent), **subjective** (agent-dependent, experiential), **intersubjective** (shared among agents through convention or agreement), **contested** (agents disagree on status), **ambiguous** (insufficient evidence to classify), **fictional** (not asserted as real).

### 14.2 Reality classification — fiction vs non-fiction

Three top-level categories with sub-types:

- **Non-fiction** (9 sub-types): empirical, testimonial, documentary, analytical, theoretical, procedural, normative, journalistic, administrative
- **Fiction** (12 sub-types): speculative, historical-fiction, realistic, fantasy, science-fiction, allegorical, satirical, hypothetical, counterfactual, simulated, roleplay, deceptive
- **Blended** (4 sub-types): mythological, legendary, propagandistic, marketing

Classification is agent-relative — what one agent treats as fiction another may treat as non-fiction (e.g. religious texts, myths). The `realityCategoryDisputed` flag marks contested classifications.

### 14.3 Intentionality — intentional vs unintentional

10 intentionality types: deliberate, intentional, negligent, reckless, accidental, mistaken, systematic, emergent, coerced, habitual.

9 mistake types: false-belief, skill-deficit, attention-failure, perception-error, judgement-error, system-design, deliberate-deviation, ungrounded-generation, simulation-artefact.

This is critical for evaluating whether 'mistakes' are intentional (deception, deliberate deviation) or unintentional (accidental, mistaken, systematic). The intentionality × mistake cross-reference distinguishes, for example, a deliberate lie (deliberate + deliberate-deviation) from an honest error (mistaken + false-belief) from an AI model's ungrounded generation (mistaken + ungrounded-generation).

### 14.4 Behaviour grounding

14 grounding types: observation, hypothesis, interpretation, belief, assumption, inference, instruction, emotion, instinct, habit, simulation, fiction, norm, values.

An agent acting on a **hypothesis** is different from one acting on a **belief**, which is different from one acting on a **fictional premise**. The `groundingVerifiable` flag indicates whether the grounding can be checked by other agents.

### 14.5 Sentiment — evaluation and analysis

Sentiment is assessed across multiple dimensions:

- **Type:** positive, negative, neutral, mixed, ambivalent
- **Intensity:** mild, moderate, strong, extreme
- **Authenticity:** genuine, performed, manipulated, simulated
- **Dimensions:** valence, arousal, dominance, certainty, moral, aesthetic, irony, sarcasm

Sentiment is linked to reality classification — sentiment towards fiction (e.g. emotional response to a novel) is different from sentiment towards non-fiction (e.g. anger at a policy). The `detect-sentiment-reality-mismatch` tool flags disproportionate sentiment.

### 14.6 Agent perspective

6 perspective types: human, software-agent, sensor, organisation, ai-model, collective.

4 bias families: cognitive (confirmation, availability, anchoring, framing, selection, recency), algorithmic (training-data, model-architecture, optimisation, ungrounded-generation), institutional (procedural, structural, cultural), sensor (calibration-drift, environmental-interference, resolution-limit).

### 14.7 Spatio-temporal context

6 mediation types: direct (co-located, contemporaneous), recorded (photo, video, sensor log), reported (through another agent), inferred (from indirect evidence), simulated (model output), absent (no access).

What is objective in one spatio-temporal frame may be subjective in another — a temperature reading is objective at the moment of measurement but becomes historical (and potentially contested) once time passes.

### 14.8 Social relation context

7 relation types: hierarchical, peer, collaborative, adversarial, dependent, community, stranger.

Contextual factors: power asymmetry (0.0–1.0), trust level (0.0–1.0), community consensus (0.0–1.0), incentive to misrepresent (0.0–1.0), social pressure (0.0–1.0).

Power asymmetries may make a dominant agent's subjective view appear objective. Adversarial relations increase the burden of evidence. Social pressure may make expressed sentiment performed rather than genuine.

### 14.9 `epistemics` toolbox — 7 tool-chains, 58 tools

- **evaluate** (12) — create assessment, set epistemic mode, set reality category, mark disputed, set spatio-temporal/social context, query, compare, recursive assessment, export, link to investigation/research
- **perspective** (7) — register perspective, add bias, query, compare, detect conflict, assess coverage, reconcile
- **intentionality** (6) — assess intentionality, classify mistake, query intentionality/mistakes, detect patterns, compare
- **grounding** (5) — assess grounding, query, detect ungrounded, verify, compare
- **fiction** (6) — classify reality, query, detect deceptive fiction, detect blended content, compare, trace fiction-to-reality
- **sentiment** (10) — assess sentiment, set dimension, link to reality, query, analyse trends, detect manipulation, detect performed sentiment, map network, compare, detect reality mismatch
- **ungrounded-generation** (12) — create instance, set cause (18 types), set consequence (12 types), set detection, set mitigation, set calibration, query instances/causes/consequences, detect patterns, compare, query cause-consequence matrix

### 14.10 Ungrounded generation — causes and consequences

A transformer inference system generates outputs through probabilistic token prediction. The output distribution is shaped by multiple influences — not simply the model's training dataset. These include the input prompt, retrieved context (RAG), attention weights, sampling parameters (temperature, top-k, top-p), decoding strategy, fine-tuning/RLHF alignment, tokenization, context window limits, and system prompts. Each can independently or jointly cause ungrounded generation.

**18 causes** across 7 influence families:

- **Training-influenced** (4): training-data-gap, training-data-conflict, training-data-noise, distribution-shift
- **Context-influenced** (4): context-window-overflow, context-contamination, prompt-ambiguity, system-prompt-conflict
- **Attention-influenced** (2): attention-misallocation, attention-dilution
- **Sampling-influenced** (2): sampling-artefact, decoding-loop
- **Retrieval-influenced** (2): retrieval-failure, retrieval-contamination
- **Alignment-influenced** (3): alignment-tax, sycophancy, fine-tuning-drift
- **Inference-environment** (3): tokenization-artefact, quantisation-artefact, numerical-instability

**12 consequences** (all software-agent-scoped, no intent implied):

enumerated-false-fact, enumerated-false-citation, enumerated-false-entity, enumerated-false-relationship, enumerated-false-quote, enumerated-false-data, context-drift, self-reinforcing-enumeration, confident-ungrounded-output, plausible-ungrounded-output, omission-by-ungrounded-output, style-mimicry

Each instance records: model id, sampling parameters, context length, retrieval used, detection method, mitigation, and confidence calibration gap (difference between expressed confidence and actual grounding quality). A cause-consequence matrix documents known pairings with likelihood and recommended mitigation.

### 14.11 Recursive epistemics

Assessments can themselves be assessed. When agents disagree on epistemic status, a third agent can create a recursive assessment evaluating the disagreement. This prevents epistemic gridlock — disagreements can be adjudicated, and the adjudication can itself be evaluated.

### 14.12 Epistemics manifold seed

| Container | Dock | Notes |
|:----------|:-----|:------|
| `reality-board` | centre | Primary workspace |
| `perspective-view` | left | Agent perspectives |
| `sentiment-map` | right | Sentiment visualisation |
| `intentionality-panel` | bottom (panel) | Intentionality & mistakes |
| `grounding-panel` | bottom (panel) | Behaviour grounding |
| `fiction-classifier` | right (panel) | Fiction/non-fiction classification |
| `social-context-panel` | left (panel) | Social relation context |

---

## 15. Hypermedia Asset Toolboxes

Seven toolboxes (52 tool-chains, 339 tools) for image, audio, video, 3D, interactive hypermedia (HbbTV), portals, and productions. Full details: [`poet-hypermedia-toolboxes.md`](poet-hypermedia-toolboxes.md). Specs: [`TOOLBOX_HYPERMEDIA_SPEC.md`](../qualia-ui/tool-chest/TOOLBOX_HYPERMEDIA_SPEC.md) · [`TOOLBOX_HYPERMEDIA_SPEC_2.md`](../qualia-ui/tool-chest/TOOLBOX_HYPERMEDIA_SPEC_2.md). Core ontology: [`hypermedia.n3`](../qualia-ui/ontologies/hypermedia.n3).

---

## 16. Implementation home

Chrome is the **HyperCanvas** from `C:\Projects\NLP\Canvas_Workbench` (menubar, pager, strata, epistemic lens, 2D/3D/4D, time ribbon, spatial stage, wires, 4-way tool chest, status). `/poet` is **not** inside Classic studio chrome.

| Concept | Path |
|---|---|
| Instructional mock | `C:\Projects\NLP\Canvas_Workbench/` |
| Container ontology (N3) | `qualia-ui/ontologies/container.n3` |
| Presentation ontology (N3) | `qualia-ui/ontologies/presentation.n3` |
| Provenance ontology (N3) | `qualia-ui/ontologies/provenance.n3` |
| Agency ontology (N3) | `qualia-ui/ontologies/agency.n3` |
| Document ontology (N3) | `qualia-ui/ontologies/document.n3` |
| Code ontology (N3) | `qualia-ui/ontologies/code.n3` |
| Investigation ontology (N3) | `qualia-ui/ontologies/investigation.n3` |
| Research ontology (N3) | `qualia-ui/ontologies/research.n3` |
| Epistemics ontology (N3) | `qualia-ui/ontologies/epistemics.n3` |
| Agent nomenclature ontology (N3) | `qualia-ui/ontologies/agent-nomenclature.n3` |
| Ungrounded generation ontology (N3) | `qualia-ui/ontologies/ungrounded-generation.n3` |
| Agent nomenclature rules | `qualia-db-standards/agent-nomenclature-rules.md` |
| Hypermedia toolboxes | `qualia-db-standards/poet-hypermedia-toolboxes.md` |
| Selfhood ontology (N3) | `qualia-ui/ontologies/selfhood.n3` |
| Personhood ontology (N3) | `qualia-ui/ontologies/personhood.n3` |
| Duty of care ontology (N3) | `qualia-ui/ontologies/duty-of-care.n3` |
| Care scope ontology (N3) | `qualia-ui/ontologies/care-scope.n3` |
| Guardianship ontology (N3) | `qualia-ui/ontologies/guardianship.n3` |
| Adversarial conduct ontology (N3) | `qualia-ui/ontologies/adversarial-conduct.n3` |
| Production document ontology (N3) | `qualia-ui/ontologies/production-document.n3` |
| Game design ontology (N3) | `qualia-ui/ontologies/game-design.n3` |
| Game world ontology (N3) | `qualia-ui/ontologies/game-world.n3` |
| Learning core ontology (N3) | `qualia-ui/ontologies/learning-core.n3` |
| Learning experience ontology (N3) | `qualia-ui/ontologies/learning-experience.n3` |
| Learning experience modality ontology (N3) | `qualia-ui/ontologies/learning-experience-modality.n3` |
| Faith systems ontology (N3) | `qualia-ui/ontologies/faith-systems.n3` |
| Values ontology (N3) | `qualia-ui/ontologies/values.n3` |
| Obligations ontology (N3) | `qualia-ui/ontologies/obligations.n3` |
| Adversarial relational methods ontology (N3) | `qualia-ui/ontologies/adversarial-relational.n3` |
| Adversarial relational scenarios ontology (N3) | `qualia-ui/ontologies/adversarial-relational-scenarios.n3` |
| Settings ontology (N3) | `qualia-ui/ontologies/settings.n3` |
| Communications manifold ontology (N3) | `qualia-ui/ontologies/communications.n3` |
| Social manifold ontology (N3) | `qualia-ui/ontologies/social.n3` |
| Social connections ontology (N3) | `qualia-ui/ontologies/social-connections.n3` |
| CSS (copied) | `crates/webizen-studio/src/components/poet/css/` |
| Registry | `…/poet/kinds.rs` |
| Manifold seeds | `…/poet/manifolds.rs` |
| Shell & bodies | `…/poet/workbench.rs` + `chrome.rs` + `stage.rs` + `chest.rs` + `bodies/` |
| Vibe console & GPU frame | `…/poet/vibe_console.rs` + `gpu_frame.rs` |
