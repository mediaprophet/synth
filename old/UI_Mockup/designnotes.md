# Poet Mindware Workbench — UI Design & Architecture Notes

> *"Poet: Define your mindware, with VibeScript"*

**Copyright © 2026 Timothy Charles Holborn.** All rights reserved.  
**Principal / inventor:** Timothy Charles Holborn &lt;timothy.holborn@gmail.com&gt;  
Assignment: [`COPYRIGHT.md`](../COPYRIGHT.md) · Licence: [`LICENSE`](../LICENSE)

This document outlines the UX philosophy, visual design system, domain primitives, interaction patterns, and evolutionary architecture for the **Poet Mindware Workbench** interactive mock-up.

---

## 1. Vision & Core Philosophy

**Poet** is an innovative hypermedia authoring and mindware workbench where human authors, co-authors, and autonomous mindware agents collaborate across structured knowledge, 3D kinematic assets, neural tensor latents, and dynamic scripts.

### Key Tenets
1. **Pure Linked Data:** Native support for W3C RDF 1.2 triple terms (`<<( s p o )>>`) and Qualia 48-byte Nquins (`<<[ s p o g prov ]>>`).
2. **Zero Ambient Authority (Fiduciary Security):** Capabilities (`graph`, `pulse`, `motor`, `codecs`) are explicitly declared and gated via the Sentinel VM.
3. **Zero Node.js Dependency:** 100% browser-native web standards (HTML5, CSS Custom Properties, ES Modules, and Custom Elements) requiring zero build steps or bundlers.
4. **Universal Linguistic Grounding:** Ontological Graphemes represent unencoded and specialized phonetic systems as semantic nodes in the knowledge graph.
5. **Auditory Intelligence Triad:** Generative articulatory audio pipeline across `q42` $\leftrightarrow$ `p64` $\leftrightarrow$ `d10`.

---

## 2. Visual Hierarchy & Design System

The workbench interface adopts a **cyber-semantic slate theme** designed for clarity when working with linked data, 3D models, neural audio tensors, and reactive code.

### Core Color & Domain Role Tokens
- **Backgrounds:**
  - Base Shell: `--bg-darkest` (`#07090e`) / `--bg-base` (`#0c1017`)
  - Surfaces: `--bg-surface` (`#131822`) / Elevated: `--bg-surface-elevated` (`#1a2230`)
- **Semantic Domain Roles:**
  - **Doc / Context Text (`--role-doc`):** Electric Cyan (`#00d2ff`)
  - **Ontological Entities (`--role-entity`):** Emerald (`#00f2a9`)
  - **Semantic Relations & Auras (`--role-relation`):** Violet (`#a855f7`)
  - **Reactive Cells (`--role-cell`):** Amber (`#ffb834`)
  - **Telemetry Pulses (`--role-event`):** Crimson/Rose (`#ff4d6d`)
  - **Ontological Graphemes (`--role-grapheme`):** Sky Blue (`#38bdf8`)
  - **Articulatory Triad (`--role-triad`):** Magenta/Pink (`#ec4899`)

---

## 3. Interactive Web Component Primitives

The workbench encapsulates all domain concepts inside standard Custom Elements:

| Custom Element | Purpose & Behavior |
| :--- | :--- |
| `<q-doc>` | Hypermedia document root container authored in `yaml-ld-q42` without XML/HTML closing-tag noise. |
| `<q-entity>` | Inline Ontological Entity marker (`data-id`, `data-type`, `data-label`). Click/hover triggers entity popover and inspector focus. |
| `<q-relation>` | Linked semantic relation bridge between nodes with visual relation indicator. |
| `<q-grapheme>` | Universal Linguistic Ontological Grapheme node with IPA approximation, visual stroke geometry (`.d10`), and phonological grounding. |
| `<q-cell>` | Dynamic reactive VibeScript calculation cell with gas metering and live re-evaluation triggers. |
| `<q-web-frame>` | Sandboxed legacy `www/` web partition with capability-checked IPC event bridge logging. |

---

## 4. Multi-View Workbench Layout

```
┌─────────────────────────────────────────────────────────────────────────────┐
│ POET Mindware Workbench | [Doc Selector] | [Fiduciary Level 3 Badge] | [📁][🔍]│
├────┬──────────────────────┬──────────────────────────────────┬──────────────┤
│ 📄 │ Hypermedia Explorer  │ Viewport Tabs:                   │ Aura Tray    │
│ ⚡ │ - .hcf Documents     │ [HyperDoc] [VibeScript] [Triad]  │ (SHACL Valid)│
│ 🎙️ │ - .vibe Agents       │ [Q42 Graph] [Sandboxed www/]     │              │
│ 🌐 │ - .q42 Graphs        ├──────────────────────────────────┼──────────────┤
│ 🛡️ │ - Active Ontologies  │ Active View Canvas / Editor /    │ Pulse Stream │
│    │                      │ Synthesizer / Graph DAG          │ (CRDT Sync)  │
├────┴──────────────────────┴──────────────────────────────────┴──────────────┤
│ Status: Graph: catchment_sites | Merkle Root: 0x8f...a42 | Gas: 984,500/1M    │
└─────────────────────────────────────────────────────────────────────────────┘
```

### View Descriptions
1. **HyperDoc Authoring (`<poet-hyperdoc-editor>`):** Rich reading and context markup annotation environment with inline entity cards and reactive cells.
2. **VibeScript IDE & REPL (`<poet-vibescript-editor>`):** Typed DSL code editor, homoiconic CBOR-LD AST tree visualizer, and gas-metered execution console.
3. **Triad Audio & 3D Kinematics (`<poet-triad-inspector>`):** Interactive articulatory speech synthesizer with real-time Web Audio API formant synthesis, animated oscilloscope, and 3D vocal tract profile parameters.
4. **Q42 Knowledge Graph Canvas (`<poet-graph-panel>`):** Interactive HTML5 Canvas RDF 1.2 / Nquin DAG visualizer with draggable nodes, predicate labels, and reification inspection.
5. **Sandboxed Legacy Web Partition (`<poet-webframe-sandbox>`):** Isolated `www/` portal container with capability-gated IPC message logging.

---

## 5. The Generative Articulatory Audio Triad (`q42` $\leftrightarrow$ `p64` $\leftrightarrow$ `d10`)

QualiaDB replaces flat audio recording playback with a generative articulatory pipeline:
- **`q42` (Phonological Graph):** Stores phonetic intent, phonemic nquins (`<<[ grapheme:042 ipa "ŋal" ]>>`), and relational prosodic cadence.
- **`p64` (Neural Latent Tensors):** 64-aligned columnar tables and continuous EnCodec neural latents processed via Forge GPU kernels.
- **`d10` (Articulatory Kinematics & 10D EMF):** Models the virtual vocal tract geometry (velum, tongue, lip constriction) to project physical acoustic waves and achieve **mathematically exact 3D lip-sync** and **speaker identity portability**.

---

## 6. Evolutionary Roadmap to Production

1. **Phase 1 (Interactive UI Mock-up — Current):** Dial in the UX for context markup, entity tagging, split-pane navigation, and hypermedia asset embedding using HTML/CSS/JS.
2. **Phase 2 (Protocol & API Alignment):** Connect `api.js` to live QualiaDB endpoints (`webizen_server`, `solid_ldp`, WebSocket/SSE) to validate data contracts (RDF 1.2, CBOR-LD, Solid LDP).
3. **Phase 3 (Rust / WASM Core Integration):** Compile the Poet Engine (VibeScript AST evaluator, SHACL-AF validator, and phonological parser) to WebAssembly (`wasm32-unknown-unknown`), keeping the UI layer lightweight, responsive, and decoupled.
