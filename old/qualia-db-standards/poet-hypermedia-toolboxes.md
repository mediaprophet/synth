# Poet UI Concepts — Hypermedia Asset Toolboxes

**Copyright © 2026 Timothy Charles Holborn.** All rights reserved.
**Parent:** [`poet-ui-concepts.md`](poet-ui-concepts.md) §15

Seven toolboxes for creating, editing, and inspecting hypermedia assets across image, audio, video, 3D, interactive, portals, and productions.

**Full toolbox spec:** [`TOOLBOX_HYPERMEDIA_SPEC.md`](../qualia-ui/tool-chest/TOOLBOX_HYPERMEDIA_SPEC.md) (Part 1: image, audio, video) · [`TOOLBOX_HYPERMEDIA_SPEC_2.md`](../qualia-ui/tool-chest/TOOLBOX_HYPERMEDIA_SPEC_2.md) (Part 2: 3D, interactive, portals, productions)
**Core ontology:** [`hypermedia.n3`](../qualia-ui/ontologies/hypermedia.n3)

---

## 15.1 Image toolbox (raster/vector editing)

8 tool-chains, 55 tools — layers, brushes, selection, filters, masks, colour management, vector tools, inspection.

**Ontology:** [`image-editing.n3`](../qualia-ui/ontologies/image-editing.n3)

**Manifold seed:** image-canvas (centre), layer-panel (right), brush-palette (left), colour-picker (left-bottom), histogram (bottom), navigator (top-right)

## 15.2 Audio toolbox (production, synth, MIDI, processing)

8 tool-chains, 58 tools — transport, tracks, clip editing, synthesis, MIDI, effects, mixing, inspection.

**Ontology:** [`audio-production.n3`](../qualia-ui/ontologies/audio-production.n3)

**Manifold seed:** audio-timeline (centre), mixer (right), transport (top), midi-editor (bottom), synth-rack (left), effect-chain (right-bottom)

## 15.3 Video toolbox (production, editing, colour, transitions)

9 tool-chains, 58 tools — transport, editing, transitions, colour grading, effects, generators, audio-sync, inspection, rendering.

**Ontology:** [`video-production.n3`](../qualia-ui/ontologies/video-production.n3)

**Manifold seed:** video-timeline (bottom), preview-monitor (centre), colour-scope (right), asset-bin (left), transport (top), effect-inspector (right-bottom)

## 15.4 3D toolbox (assets, animation, narratives)

8 tool-chains, 52 tools — object management, modelling, rigging, animation, materials, camera/light, narrative, inspection.

**Ontology:** [`spatial-3d.n3`](../qualia-ui/ontologies/spatial-3d.n3)

**Manifold seed:** viewport-3d (centre), outliner (top-right), properties-3d (right), timeline-3d (bottom), material-editor (left), asset-library (top-left)

## 15.5 Hypermedia toolbox (2nd screen, interactive, HbbTV)

6 tool-chains, 37 tools — interactive overlays, second-screen, social layer, HbbTV packaging, synchronisation, inspection.

**Ontology:** [`interactive-hypermedia.n3`](../qualia-ui/ontologies/interactive-hypermedia.n3)

**Manifold seed:** interactive-timeline (bottom), preview-screen (centre), trigger-panel (right), social-panel (left), package-inspector (right-bottom), device-emulator (top-right)

## 15.6 Portals toolbox (web portals, worlds)

6 tool-chains, 36 tools — world-building, object placement, portal links, avatars, physics, inspection.

**Ontology:** [`portal-worlds.n3`](../qualia-ui/ontologies/portal-worlds.n3)

**Manifold seed:** portal-viewport (centre), world-outliner (top-right), portal-properties (right), asset-library (left), physics-inspector (bottom), portal-preview (tabbed)

## 15.7 Productions toolbox (events, DMX, projection mapping)

7 tool-chains, 43 tools — DMX universe management, fixture placement, lighting control, projection mapping, cue stacks, show control, inspection.

**Ontology:** [`production-events.n3`](../qualia-ui/ontologies/production-events.n3)

**Manifold seed:** production-timeline (bottom), dmx-controller (left), projection-canvas (centre), fixture-patch (top-right), cue-stack (right), preview-monitor (top-centre)

---

## 15.8 Cross-domain references

Assets reference assets across domains: 3D materials reference image textures, video references audio tracks, hypermedia references video/audio streams, productions reference DMX + projection + audio, portals reference 3D scenes + audio.

---

## 15.9 Hypermedia tool count summary

| Toolbox | Tool-chains | Tools |
|:--------|:-----------|:------|
| `image` | 8 | 55 |
| `audio` | 8 | 58 |
| `video` | 9 | 58 |
| `3d` | 8 | 52 |
| `hypermedia` | 6 | 37 |
| `portals` | 6 | 36 |
| `productions` | 7 | 43 |
| **Total** | **52** | **339** |
