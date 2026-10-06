# QualiaDB Standards & Manuals Synchronization Log

**Copyright © 2026 Timothy Charles Holborn.** All rights reserved.  
**Principal / inventor:** Timothy Charles Holborn &lt;timothy.holborn@gmail.com&gt;

**Date:** 2026-08-15  
**Target Repository:** `C:\Projects\qualia-27062026`  

---

## 1. Deployed Standards & Specifications

| Target File | Scope & Capabilities |
| :--- | :--- |
| [`docs/manuals/standards/vibescript-core.md`](file:///C:/Projects/qualia-27062026/docs/manuals/standards/vibescript-core.md) | **VibeScript Core `vibe-0.1` (normative editor’s draft):** implementable grammar, types, effects, Quin constructor (no raw overlay), RDF 1.2 Turtle spellings, QApp/Pure-cell boundary, 0.1 bindings. Consult: Codex, Gemini, branding, extensible-runtime PDF. |
| [`docs/manuals/standards/vibescript-specification.md`](file:///C:/Projects/qualia-27062026/docs/manuals/standards/vibescript-specification.md) | Architectural essay only — not the parser contract. |
| [`docs/manuals/standards/hypermedia-content-format-hcf.md`](file:///C:/Projects/qualia-27062026/docs/manuals/standards/hypermedia-content-format-hcf.md) | **Hypermedia Content Format (HCF) & Linguistic Specification (v1.0):** RDF-native authoring in `yaml-ld-q42` and binary transport in `CBOR-LD 1.0` (zero HTML/XML/JS bloat), **Universal Linguistic Architecture (Ontological Graphemes)** for non-Unicode/unencoded mother tongues, **Generative Articulatory Speech Pipeline** across the `q42` $\leftrightarrow$ `p64` $\leftrightarrow$ `d10` triad, and Bao verified streaming for `.hmc` archives. |
| [`docs/manuals/ai-agent-vibescript-readiness.md`](file:///C:/Projects/qualia-27062026/docs/manuals/ai-agent-vibescript-readiness.md) | **AI Agent Readiness & Reference Knowledge Graph:** RDF/OWL & SHACL ontology (`vibe:`, `pulse:`, `aura:`, `poet:`), grammar-guided constrained decoding (GBNF / JSON-Schema), golden reference corpus, and Rust/WASM diagnostic feedback compiler loop. |
| [`docs/manuals/poet-mindware-workbench-ui.md`](file:///C:/Projects/qualia-27062026/docs/manuals/poet-mindware-workbench-ui.md) | **Poet Mindware Workbench UI Strategy:** Zero-build native Web Components (`<q-doc>`, `<q-entity>`, `<q-relation>`, `<q-cell>`), Aura schema tray, and real-time Pulse collaborative monitor. |

---

## 2. Updated Directory Indices & HTML Portals

- [`docs/manuals/standards/DIRECTORY_INDEX.md`](file:///C:/Projects/qualia-27062026/docs/manuals/standards/DIRECTORY_INDEX.md) — Updated with VibeScript and HCF format entries.
- [`docs/manuals/DIRECTORY_INDEX.md`](file:///C:/Projects/qualia-27062026/docs/manuals/DIRECTORY_INDEX.md) — Updated with AI Agent Readiness and Poet Workbench entries.
- [`docs/manuals/standards/standards-backlog.md`](file:///C:/Projects/qualia-27062026/docs/manuals/standards/standards-backlog.md) — Marked VibeScript and HCF specifications as completed standards deliverables.
- [`docs/manuals/standards/index.html`](file:///C:/Projects/qualia-27062026/docs/manuals/standards/index.html) — Added *Languages & Hypermedia* section with links to the live specs.
- [`docs/manuals/index.html`](file:///C:/Projects/qualia-27062026/docs/manuals/index.html) — Added navigation links for VibeScript, HCF, AI Agent Readiness, and Poet Workbench.

---

## 3. Real core-db / desktop gaps (P7.D — do not invent more)

Logged from the 0.0.31-dev Poet / NLP landing. These are actual holes, not essays.

| Gap | What exists | What is missing |
|---|---|---|
| Poet graph host | `PoetSnapshot` is an in-process `Vec<NQuin>` | No daemon-graph / port 4242 snapshot attach (`P5.7`) |
| Aura | `aura_validate` returns `true` | Not wired to the SHACL subset already in core-db |
| Pulse | Topic strings appended on the snapshot | Not the telemetry / event bus |
| Span-Quin | `text_span::annotation_quin` seals parity on the host | No Lived Memory document store; harness pastes a snippet |
| Gazetteer lexicon | Closed list in `qualia-nlp::lexicon` | Not the engine lexicon / ontology workbench IDs |
| Classic UI | Default chrome; `/poet` is a harness | No principal `approve deprecate:` for any Classic route |

Year-one NLP is tokenize + gazetteer + span plans. Not FrameNet, RST, OpenIE, NLI, or MT.
