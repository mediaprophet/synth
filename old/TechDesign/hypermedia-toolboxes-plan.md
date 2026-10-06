# Hypermedia Asset Toolboxes — Implementation Plan

**Copyright © 2026 Timothy Charles Holborn.** All rights reserved.
**Created:** 2026-08-17
**Status:** Planning

---

## Objective

Create manifolds, canvases, tool-chains, and toolboxes for creating, editing, and inspecting hypermedia assets across 7 domains:

1. **Image** — raster/vector editing (Photoshop-class)
2. **Audio** — production, synth, multi-track, processing, MIDI
3. **Video** — production, editing, colour correction, asset generation, transitions
4. **3D** — assets, animation, stories/narratives
5. **Hypermedia** — 2nd screen, social functionality, interactive shows, HbbTV 2.x packaging
6. **Portals** — web portals, worlds, immersive environments
7. **Productions** — public spaces, events, DMX controllers, projection mapping

---

## Architecture

Each domain becomes a **toolbox** (chest drawer) containing multiple **tool-chains** (grouped tool sets). Each toolbox has:
- A **toolbox spec** (markdown) defining containers, tool-chains, tools, and manifold seeds
- An **ontology** (N3) defining domain vocabulary, asset types, and properties
- **Manifold seeds** for each workflow within the domain
- **Containers** (content, panel, widget) placed on manifolds

All toolboxes follow the existing pattern established by `TOOLBOX_CODE_SPEC.md`, `TOOLBOX_INVESTIGATION_SPEC.md`, `TOOLBOX_RESEARCH_SPEC.md`, and `TOOLBOX_EPISTEMICS_SPEC.md`.

### File structure

```
qualia-ui/tool-chest/
  TOOLBOX_HYPERMEDIA_SPEC.md          # All 7 hypermedia toolboxes in one spec (split if >800 lines)
  TOOL_CHEST_SPEC.md                  # Updated with new toolbox entries and counts

qualia-ui/ontologies/
  hypermedia.n3                        # Core hypermedia ontology — asset types, provenance, format
  image-editing.n3                     # Image domain — layers, filters, brushes, masks
  audio-production.n3                  # Audio domain — tracks, synthesis, MIDI, effects
  video-production.n3                  # Video domain — timeline, clips, colour, transitions
  spatial-3d.n3                        # 3D domain — meshes, rigs, animation, narratives
  interactive-hypermedia.n3            # Hypermedia domain — HbbTV, 2nd screen, interactive
  portal-worlds.n3                     # Portal domain — worlds, immersive environments
  production-events.n3                 # Production domain — DMX, projection, events
```

### Nomenclature compliance

All ontologies and specs follow the agent nomenclature isolation rules (see `agent-nomenclature-rules.md`). Software tools use computational terms (process, render, enumerate, generate), not mind-dependent terms.

---

## Plan

### Phase 1: Core hypermedia ontology + spec scaffolding

- [ ] **1.1** Create `hypermedia.n3` — core ontology defining:
  - `hm:HypermediaAsset` (base class for all media assets)
  - Asset format types (raster, vector, audio, video, 3D, interactive, portal, production)
  - Provenance properties (source, derivative, licence, rights)
  - Composition properties (layer, track, scene, timeline)
  - Inspection properties (metadata, codec, resolution, duration, sample-rate)
  - Cross-domain references (e.g. video references audio track, 3D references texture)

- [ ] **1.2** Create `TOOLBOX_HYPERMEDIA_SPEC.md` — header, overview, shared containers, relationship table

- [ ] **1.3** Update `TOOL_CHEST_SPEC.md` — add new toolbox entries to registry, update counts

### Phase 2: Image toolbox (raster/vector editing)

- [ ] **2.1** Create `image-editing.n3` ontology:
  - `img:ImageAsset` (raster, vector, hybrid)
  - `img:Layer` (pixel, adjustment, vector, text, mask, smart-object)
  - `img:Filter` (blur, sharpen, colour-adjust, distort, noise, stylise)
  - `img:Brush` (pixel, vector, eraser, clone, heal, gradient)
  - `img:Selection` (rectangular, lasso, magic-wand, vector-path)
  - `img:Mask` (layer-mask, clipping-mask, vector-mask)
  - `img:ColourProfile` (sRGB, AdobeRGB, ProPhoto, DCI-P3, Rec.2020)
  - Properties: resolution, colour-depth, colour-space, dimensions, DPI

- [ ] **2.2** Define image toolbox spec:
  - **Containers:** `image-canvas` (content), `layer-panel` (panel), `brush-palette` (panel), `colour-picker` (panel), `histogram` (panel), `navigator` (panel)
  - **Tool-chains:**
    - `layers` (8 tools) — add-layer, delete-layer, reorder-layer, merge-layers, duplicate-layer, group-layers, layer-opacity, layer-blend-mode
    - `brushes` (10 tools) — brush-select, brush-size, brush-opacity, brush-hardness, brush-flow, brush-colour, eraser, clone-stamp, heal-brush, gradient-fill
    - `selection` (7 tools) — rect-select, lasso-select, magic-wand, vector-path-select, invert-selection, feather-selection, refine-edge
    - `filters` (8 tools) — gaussian-blur, sharpen, colour-balance, hue-saturation, curves, levels, distort, noise-reduction
    - `masks` (5 tools) — add-layer-mask, apply-mask, invert-mask, vector-mask, clipping-mask
    - `colour` (6 tools) — colour-picker, sample-colour, colour-profile, convert-profile, gradient-map, posterise
    - `vector` (6 tools) — pen-tool, shape-tool, path-edit, text-on-path, vector-export, svg-import
    - `inspect` (5 tools) — histogram, colour-sampler, info-probe, metadata-view, profile-inspector
  - **Manifold seed:** image-canvas (centre), layer-panel (right), brush-palette (left), colour-picker (left-bottom), histogram (bottom), navigator (top-right)

### Phase 3: Audio toolbox (production, synth, MIDI, processing)

- [ ] **3.1** Create `audio-production.n3` ontology:
  - `aud:AudioAsset` (wave, midi, sample, recording)
  - `aud:Track` (audio, midi, instrument, bus, send)
  - `aud:Clip` (audio-clip, midi-clip, automation-clip)
  - `aud:Synthesiser` (oscillator, filter, envelope, LFO, modulator)
  - `aud:Effect` (reverb, delay, compression, EQ, distortion, modulation)
  - `aud:MIDIEvent` (note-on, note-off, CC, pitch-bend, program-change, sysex)
  - `aud:TempoMap` (BPM, time-signature, tempo-change)
  - Properties: sample-rate, bit-depth, channels, duration, BPM, key

- [ ] **3.2** Define audio toolbox spec:
  - **Containers:** `audio-timeline` (content), `mixer` (panel), `synth-rack` (panel), `midi-editor` (panel), `effect-chain` (panel), `transport` (panel)
  - **Tool-chains:**
    - `transport` (6 tools) — play, stop, record, loop-region, metronome, tempo-set
    - `tracks` (8 tools) — add-audio-track, add-midi-track, add-bus, delete-track, rename-track, track-volume, track-pan, track-routing
    - `editing` (10 tools) — split-clip, join-clips, trim-clip, fade-in, fade-out, crossfade, time-stretch, pitch-shift, reverse, normalise
    - `synthesis` (8 tools) — osc-config, filter-config, envelope-config, lfo-config, mod-routing, preset-save, preset-load, wavetable-import
    - `midi` (7 tools) — note-draw, note-edit, velocity-edit, CC-edit, quantise, transpose, midi-import
    - `effects` (8 tools) — add-effect, reorder-effect, remove-effect, reverb-config, delay-config, compressor-config, eq-config, saturation-config
    - `mixing` (6 tools) — set-send, set-bus, automation-draw, automation-edit, bounce-track, bounce-mix
    - `inspect` (5 tools) — spectrum-analyser, waveform-view, phase-meter, loudness-meter, metadata-view
  - **Manifold seed:** audio-timeline (centre), mixer (right), transport (top), midi-editor (bottom), synth-rack (left), effect-chain (right-bottom)

### Phase 4: Video toolbox (production, editing, colour, transitions)

- [ ] **4.1** Create `video-production.n3` ontology:
  - `vid:VideoAsset` (clip, sequence, render, stream)
  - `vid:Timeline` (video-track, audio-track, overlay-track)
  - `vid:Transition` (cut, crossfade, wipe, dissolve, push, slide, zoom)
  - `vid:Effect` (colour-correction, stabilisation, speed-change, transform, keying, tracking)
  - `vid:ColourGrade` (LUT, primary, secondary, node-based)
  - `vid:Generator` (title, lower-third, shape, colour-matte, noise, gradient)
  - Properties: resolution, frame-rate, codec, colour-space, bit-depth, duration

- [ ] **4.2** Define video toolbox spec:
  - **Containers:** `video-timeline` (content), `preview-monitor` (content), `colour-scope` (panel), `effect-inspector` (panel), `asset-bin` (panel), `transport` (panel)
  - **Tool-chains:**
    - `transport` (6 tools) — play, stop, mark-in, mark-out, shuttle, jog
    - `editing` (10 tools) — razor, splice, overwrite, insert, ripple-delete, lift, append, split-clip, match-frame, replace
    - `transitions` (7 tools) — crossfade, wipe, dissolve, push, slide, dip-to-black, custom-transition
    - `colour` (8 tools) — lift-gamma-gain, saturation, colour-wheels, curves, hue-vs-hue, hue-vs-sat, LUT-apply, LUT-export
    - `effects` (7 tools) — stabilise, speed-change, transform, chroma-key, mask, motion-track, add-blur
    - `generators` (5 tools) — title-generator, lower-third, shape-generator, colour-matte, noise-generator
    - `audio-sync` (5 tools) — sync-from-audio, auto-align, scrub-audio, extract-audio, link-clips
    - `inspect` (5 tools) — waveform-monitor, vectorscope, histogram, frame-info, metadata-view
    - `render` (5 tools) — render-queue, render-settings, render-preview, export-preset, batch-render
  - **Manifold seed:** video-timeline (bottom), preview-monitor (centre), colour-scope (right), asset-bin (left), transport (top), effect-inspector (right-bottom)

### Phase 5: 3D toolbox (assets, animation, narratives)

- [ ] **5.1** Create `spatial-3d.n3` ontology:
  - `sp3:Scene3D` (scene, object, camera, light, material)
  - `sp3:MeshAsset` (static, rigged, morph-target, procedural)
  - `sp3:Animation` (keyframe, skeletal, morph, procedural, constraint)
  - `sp3:Material` (PBR, unlit, custom-shader, texture-set)
  - `sp3:Camera` (perspective, orthographic, panoramic, stereo)
  - `sp3:Light` (point, directional, spot, area, HDRI)
  - `sp3:Narrative` (story-board, scene-sequence, camera-path, trigger)
  - Properties: vertex-count, polygon-count, UV-maps, rig-bones, morph-targets

- [ ] **5.2** Define 3D toolbox spec:
  - **Containers:** `viewport-3d` (content), `outliner` (panel), `properties-3d` (panel), `timeline-3d` (panel), `material-editor` (panel), `asset-library` (panel)
  - **Tool-chains:**
    - `object` (8 tools) — add-mesh, import-mesh, delete-object, duplicate-object, parent-object, group-objects, snap-to-grid, transform-object
    - `modelling` (8 tools) — extrude, inset, bevel, loop-cut, subdivide, merge-vertices, knife-tool, boolean-op
    - `rigging` (6 tools) — add-bone, weight-paint, inverse-kinematics, bone-constraint, pose-mirror, rig-test
    - `animation` (7 tools) — keyframe-insert, keyframe-edit, dope-sheet, graph-editor, action-clip, animation-mixer, motion-path
    - `materials` (6 tools) — create-material, assign-material, texture-paint, UV-edit, material-node, PBR-preset
    - `camera-light` (6 tools) — add-camera, camera-frame, add-light, light-type, HDRI-setup, shadow-setup
    - `narrative` (6 tools) — add-scene, scene-sequence, camera-path, storyboard-frame, trigger-zone, narrative-export
    - `inspect` (5 tools) — mesh-statistics, material-inspector, rig-inspector, scene-graph, metadata-view
  - **Manifold seed:** viewport-3d (centre), outliner (top-right), properties-3d (right), timeline-3d (bottom), material-editor (left), asset-library (top-left)

### Phase 6: Hypermedia toolbox (2nd screen, interactive, HbbTV)

- [ ] **6.1** Create `interactive-hypermedia.n3` ontology:
  - `ihm:InteractiveAsset` (app, overlay, trigger, stream)
  - `ihm:SecondScreen` (companion-app, sync-stream, remote-control)
  - `ihm:HbbTVPackage` (DVB-stream, AIT, app-data, DRM)
  - `ihm:Trigger` (time-based, event-based, user-input, sensor)
  - `ihm:SocialLayer` (chat, poll, reaction, share, co-view)
  - `ihm:SyncModel` (timeline-sync, event-sync, wall-clock, SCTE-35)
  - Properties: target-platform, stream-protocol, DRM-scheme, bandwidth-tier

- [ ] **6.2** Define hypermedia toolbox spec:
  - **Containers:** `interactive-timeline` (content), `preview-screen` (content), `trigger-panel` (panel), `social-panel` (panel), `package-inspector` (panel), `device-emulator` (panel)
  - **Tool-chains:**
    - `interactive` (8 tools) — add-trigger, trigger-condition, trigger-action, overlay-create, overlay-edit, overlay-timeline, interactive-preview, interactive-test
    - `second-screen` (6 tools) — companion-app-config, sync-stream-setup, remote-control-map, second-screen-preview, push-notification, screen-pairing
    - `social` (6 tools) — chat-overlay, poll-create, reaction-overlay, share-button, co-view-sync, social-moderation
    - `packaging` (7 tools) — hbbtv-package, ait-config, dvb-stream-bind, drm-config, app-data-bundle, package-validate, package-export
    - `sync` (5 tools) — timeline-sync, event-sync, wall-clock-sync, scte35-marker, sync-test
    - `inspect` (5 tools) — package-inspector, stream-analyser, device-emulator, bandwidth-simulator, metadata-view
  - **Manifold seed:** interactive-timeline (bottom), preview-screen (centre), trigger-panel (right), social-panel (left), package-inspector (right-bottom), device-emulator (top-right)

### Phase 7: Portals toolbox (web portals, worlds)

- [ ] **7.1** Create `portal-worlds.n3` ontology:
  - `pw:Portal` (web-portal, world, immersive-env, social-space)
  - `pw:World` (terrain, skybox, environment, physics, spawn-point)
  - `pw:PortalLink` (warp, teleport, scene-load, iri-link)
  - `pw:Avatar` (mesh, animation, customisation, controller-map)
  - `pw:EnvironmentObject` (prop, furniture, interactive-object, portal-anchor)
  - `pw:PhysicsBody` (static, dynamic, trigger, character-controller)
  - Properties: world-iri, player-count, physics-engine, render-pipeline

- [ ] **7.2** Define portals toolbox spec:
  - **Containers:** `portal-viewport` (content), `world-outliner` (panel), `portal-properties` (panel), `asset-library` (panel), `physics-inspector` (panel), `portal-preview` (content)
  - **Tool-chains:**
    - `world-building` (8 tools) — create-world, terrain-edit, skybox-set, environment-light, spawn-point, boundary-set, world-save, world-publish
    - `objects` (7 tools) — place-prop, place-furniture, interactive-object, portal-anchor, object-transform, object-duplicate, object-delete
    - `portals` (6 tools) — create-portal, portal-link, portal-destination, portal-preview, portal-list, portal-test
    - `avatars` (5 tools) — avatar-select, avatar-customise, controller-map, avatar-preview, avatar-spawn
    - `physics` (5 tools) — add-collider, add-rigidbody, add-trigger, physics-material, physics-bake
    - `inspect` (5 tools) — world-statistics, physics-inspector, portal-graph, player-list, metadata-view
  - **Manifold seed:** portal-viewport (centre), world-outliner (top-right), portal-properties (right), asset-library (left), physics-inspector (bottom), portal-preview (tabbed with viewport)

### Phase 8: Productions toolbox (events, DMX, projection mapping)

- [ ] **8.1** Create `production-events.n3` ontology:
  - `pe:Production` (event, show, installation, broadcast)
  - `pe:DMXUniverse` (universe, channel, fixture, scene, cue)
  - `pe:Fixture` (light, moving-head, LED-par, laser, strobe, haze)
  - `pe:ProjectionSurface` (flat, curved, architectural, volumetric)
  - `pe:ProjectionMap` (geometry-map, UV-map, edge-blend, calibration)
  - `pe:CueStack` (cue, sequence, timeline, trigger, fade)
  - `pe:Timeline` (show-control, SMPTE, MIDI-timecode, Art-Net)
  - Properties: fixture-count, DMX-universes, projection-resolution, surface-count

- [ ] **8.2** Define productions toolbox spec:
  - **Containers:** `production-timeline` (content), `dmx-controller` (content), `projection-canvas` (content), `fixture-patch` (panel), `cue-stack` (panel), `preview-monitor` (content)
  - **Tool-chains:**
    - `dmx` (8 tools) — add-universe, patch-fixture, unpatch-fixture, fixture-profile, channel-assign, DMX-monitor, DMX-scene, DMX-cue
    - `fixtures` (6 tools) — add-moving-head, add-led-par, add-laser, add-strobe, add-hazer, fixture-position
    - `lighting` (6 tools) — colour-mix, beam-shape, gobo-select, position-set, intensity-set, effect-run
    - `projection` (7 tools) — add-surface, surface-map, edge-blend, geometry-correct, calibration-point, projection-content, projection-preview
    - `cue-stack` (6 tools) — add-cue, edit-cue, cue-sequence, cue-trigger, cue-fade, cue-go
    - `show-control` (5 tools) — smpte-sync, midi-timecode, art-net-config, OSC-config, timeline-trigger
    - `inspect` (5 tools) — DMX-monitor, fixture-report, projection-analyser, power-calculator, metadata-view
  - **Manifold seed:** production-timeline (bottom), dmx-controller (left), projection-canvas (centre), fixture-patch (top-right), cue-stack (right), preview-monitor (top-centre)

### Phase 9: Cross-domain integration + poet-ui-concepts update

- [ ] **9.1** Define cross-domain container references in `hypermedia.n3`:
  - Video references audio tracks (sync)
  - 3D references image textures (material)
  - Hypermedia references video/audio streams (interactive overlay)
  - Productions reference DMX + projection + audio (show control)
  - Portals reference 3D scenes + audio (immersive)

- [ ] **9.2** Update `poet-ui-concepts.md`:
  - Add §15-21 sections for each hypermedia toolbox
  - Add manifold seeds
  - Update implementation home table with new ontology files

- [ ] **9.3** Update `TOOL_CHEST_SPEC.md`:
  - Add all 7 new toolboxes to the toolbox registry
  - Add new containers to the container table
  - Add new manifold seeds
  - Update tool counts
  - Add new ontology files to relationship table

- [ ] **9.4** Update `container.n3`:
  - Add new container types for each domain

### Phase 10: Verification

- [ ] **10.1** Verify all files under 800 lines
- [ ] **10.2** Verify nomenclature compliance (no mind-dependent terms for software agents)
- [ ] **10.3** Verify all tool-chains have ≥2 tools
- [ ] **10.4** Verify all manifold seeds have container + dock + notes
- [ ] **10.5** Verify cross-references between specs are consistent

---

## Tool count summary (planned)

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

---

## Ordering note

Phases 1-8 can be partially parallelised (each domain ontology is independent). Phase 9 depends on all prior phases. Phase 10 is final verification.

The spec file `TOOLBOX_HYPERMEDIA_SPEC.md` may exceed 800 lines — if so, split into:
- `TOOLBOX_HYPERMEDIA_SPEC.md` (image + audio + video)
- `TOOLBOX_HYPERMEDIA_SPEC_2.md` (3D + hypermedia + portals + productions)

Or split by domain if needed.
