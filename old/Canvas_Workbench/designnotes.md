# HyperCanvas Mindware Workbench — Design & Architecture Notes

> **Antigravity / Gemini:** mock UI structure only. Read
> [`consult/antigravity-hypercanvas-mock.md`](../consult/antigravity-hypercanvas-mock.md)
> before changing this tree. Do not edit `C:\Projects\qualia-27062026`.
> After each pass append [`IMPLEMENT.md`](IMPLEMENT.md).

> *"The Multi-Dimensional Canvas for Interconnected Mindware, Knowledge Graphs, Social Chat Graphs, WebRTC, Health Telemetry, Webviews, Fiduciary Rights, Epistemic Modalities, and Virtual Desktops with Recursive Canvas-in-Canvas Components"*

**Copyright © 2026 Timothy Charles Holborn.** All rights reserved.  
**Principal / inventor:** Timothy Charles Holborn &lt;timothy.holborn@gmail.com&gt;  
Assignment: [`COPYRIGHT.md`](../COPYRIGHT.md) · Licence: [`LICENSE`](../LICENSE)

This document specifies the technical and interaction design for the **HyperCanvas Workbench**, an open, graph-addressable, multi-dimensional spatial workbench where documents, code, 3D assets, spreadsheets, ontology mappings, geospatial case maps, social AI sub-agent chat graphs, WebRTC live streams, clinical health telemetry, and webviews coexist as interconnected reactive nodes across modular, switchable virtual workspaces.

---

## 1. Virtual Desktops / Multi-Manifold Workspace Architecture

Work surfaces are **manifolds** (same word as the engine’s 10D manifold).  
HTML `<canvas>` / WebGL / wgpu surfaces stay named **canvas** — they are drawing surfaces, not workspaces.

Similar to virtual desktops in Linux (e.g. GNOME / KDE / i3 workspaces), the workbench supports multiple dedicated, specialized manifolds that users can switch between seamlessly:

```text
Canvas_Workbench/
├── manifolds/                          # Specialized activity manifold registry
│   ├── index.js                        # Unified manifold registry
│   ├── research-manifold.js            # 🔬 Research, GIS catchment & Clinical Telemetry
│   ├── media-production-manifold.js    # 🎨 3D Kinematics, Grapheme Art & Acoustic Tensors
│   ├── social-governance-manifold.js   # 👥 Multi-Agent Chat Graphs & Live WebRTC
│   └── system-settings-manifold.js     # ⚙️ Fiduciary VM, Merkle DAG & Sub-Manifolds
├── canvases/                           # Deprecated re-export shim → manifolds/
├── containers/                         # Modular Container Components
│   └── subcanvas-container.js         # Manifold-in-manifold recursive components
├── toolboxes/                          # Toolboxes that live in the tool chest
│   └── registry.js                     # 4-Way dockable toolbox palettes
│                                       # Tool chest = the dock; toolbox = a drawer; tool = one action
├── js/
│   ├── canvas-switcher.js              # Virtual Desktop Pager & Exposé Overview
│   ├── container-manager.js            # Multi-type container rendering & event binding
│   ├── canvas-engine.js                # 2D/3D/4D spatial transform of the HTML viewport
│   └── state.js                        # Central reactive store
```

### Predefined specialized activity manifolds

1. **🔬 Research & Epistemological Mindware Manifold (`manifold-research-01`):**
   - Focus: Geospatial GIS catchment tracking, clinical health pathology/imaging, and cross-ontology legal rights alignment.
2. **🎨 Media Production & Creative 3D Studio (`manifold-media-01`):**
   - Focus: 3D virtual vocal tract kinematics (.10d), visual grapheme art, and EnCodec P64 neural acoustic latents.
3. **👥 Social Governance & Multi-Agent Collaboration (`manifold-social-01`):**
   - Focus: Project-team & AI sub-agent chat graphs, encrypted WebRTC live streams, and field-note records.
4. **⚙️ Cybernetic Settings & Sub-Manifold Hub (`manifold-settings-01`):**
   - Focus: VibeScript fiduciary VM, capability management, and recursive sub-manifold embedding.

### Virtual Desktop Pager & Exposé Overview
- **Desktop Pager Bar:** `[🗂️ Overview] [1: 🔬 Research] [2: 🎨 Media] [3: 👥 Social] [4: ⚙️ Settings] [+]`
- **Exposé / Mission Control Grid:** Pressing `Alt+O` or clicking **🗂️ Overview** displays a 2x2 thumbnail grid overview of all active virtual desktops for rapid visual navigation.
- **Keyboard Shortcuts:** `Alt+1`, `Alt+2`, `Alt+3`, `Alt+4` for instant desktop switching.

---

## 2. Manifold-in-Manifold (sub-manifold as a component)

The workbench supports **fractal / recursive manifold containment**:
- A container node on any manifold can have `type: 'subcanvas'` (legacy type id) or target another manifold id.
- It renders an embedded sub-manifold viewport displaying child container nodes and live graph telemetry from the target workspace (e.g. embedding the *Research Manifold* inside the *Settings Hub*).
- Clicking **"Enter Sub-Manifold Viewport ➔"** seamlessly zooms and transitions the workbench camera into that sub-manifold workspace.

---

## 3. Epistemic Modalities Architecture (Objective vs. Subjective Qualia)

| Modality | Badge Icon | Semantic IRI | Description |
| :--- | :---: | :--- | :--- |
| **Objective** | 🔬 | `qualia:ObjectiveObservation` | Third-person verifiable empirical telemetry, physical coordinates, sensor logs, cryptographic hashes. |
| **Subjective** | 🧠 | `qualia:PhenomenologicalQualia` | First-person lived experience, qualia, oral history reflections, emotional resonance. |
| **Intersubjective** | 🌊 | `qualia:IntersubjectiveConsensus` | Second-person shared cultural meaning, communal agreements, etc. |
| **Normative** | ⚖️ | `qualia:NormativeStatute` | Statutory obligations, water-rights fiduciary mandates, DID governance rights. |

---

## 4. Complete Container Suite Overview

| Container Type | Icon | Honesty Level | Primary Capabilities |
| :--- | :---: | :---: | :--- |
| **`subcanvas`** | 🌌 | `partial` | Recursive embedded sub-manifold component with live preview & transition |
| **`social`** | 👥 | `partial` | AI Sub-Agent collaborative threads & Chat Graph Nquin generation |
| **`webrtc`** | 🎥 | `present` | Encrypted live audio/video streaming & peer telemetry (mock channel) |
| **`health`** | 🩺 | `present` | Generic adult bio-telemetry, spectral tomography slice, 10D vocal tract mesh |
| **`webview`** | 🌐 | `present` | Sandboxed browser navigation, web clipping, RDF extraction |
| **`map`** | 🗺️ | `partial` | Topographic GIS vector map, multi-agent tracks, incident pins |
| **`doc`** | 📄 | `partial` | In-container WYSIWYG authoring, markdown/triples views, `<q-entity>` spans |
| **`ontology`**| 🏷️ | `partial` | Class/property declarations, Aura/SHACL alignment matrix, interactive row insert |
| **`code`** | ⚡ | `partial` | VibeScript 0.1 editor, homoiconic AST inspector, gas accounting |
| **`sheet`** | 📊 | `present` | Columnar tables, formula bar (`fx`), EnCodec P64 neural latents |
| **`3d`** | 🧊 | `present` | 3D kinematics motor articulation & virtual vocal tract geometry |
| **`portal`** | 🌀 | `present` | Sub-manifold wormhole navigation & multi-tenant resource bridge |
| **`media`** | 🎨 | `partial` | Generative visual asset cards & vector grapheme art |

---

## 4-A. Tool Chest and Toolboxes

The dock is a **tool chest** (furniture). It holds themed **toolboxes** (drawers). Each toolbox holds **tools** (actions, usually placing typed containers or calibrating modalities):

1. **`epistemic`**: Objective, Subjective, Intersubjective, and Normative grounding tags & qualia affect radars.
2. **`office`**: Document records, ontology alignment nodes, semantic context markup (`<q-entity>`).
3. **`image`**: Visual asset cards, generative prompt tools, spectral filters.
4. **`sheet`**: Columnar tensor sheets, P64 latent embeddings, resonance math.
5. **`spatial`**: Geospatial case maps, 3D mesh nodes, wormhole portals, incident pins.
6. **`communication`**: Multi-agent collaborative chat graphs, WebRTC stream panels, webviews.
7. **`rights`**: Fiduciary duty-of-care scopes, DID signing (`did:qualia:timothy_charles_holborn`), audit logs.
8. **`health`**: Pathology lab profiles, spectral acoustic tomography, 10D anatomy resonators.
9. **`code`**: VibeScript 0.1 DSL containers, capability reach declarations (`graph`, `pulse`).
10. **`ai`**: Semantic co-authors, structured graph extractors, sentinel goal planners.

---

## 5. Honesty Labels Framework

To keep instructional mocks strictly honest for downstream implementation in Qualia Desktop (Dioxus Poet):

- **`live`**: Fully functional local browser state / math (pan, zoom, orbit, event bus).
- **`partial`**: Local mock AST inspector, RDF spans, matrix row insertion, GIS track scrubbing.
- **`present`**: Structural UI present with mock inputs (WebRTC stream, clinical numbers, spreadsheet formulas).
- **`missing`**: Remote services or ungrounded APIs (not faked as working).

---

## 6. Desk Physics

- **Move & Resize**: 8px grid snapping (`Math.round(val / 8) * 8`).
- **Dynamic Layering**: Focused containers dynamically elevate to the highest z-index.
- **Bounds**: Resizing respects minimum dimensions (280px × 180px).
