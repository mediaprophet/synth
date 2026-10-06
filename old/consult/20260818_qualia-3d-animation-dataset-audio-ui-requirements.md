# qualia-ui 3D / Animation / Complex-Dataset Presentation / Audio UI Requirements

**Date:** 2026-08-18
**Principal / copyright holder:** Timothy Charles Holborn <timothy.holborn@gmail.com>
**Target UI:** `C:\Projects\NLP\qualia-ui` (WASM HyperCanvas — the primary UI target)
**Companion document:** `C:\Projects\qualia-27062026\docs\plans\qualia-3d-animation-dataset-audio-todo-2026-08-18.md` (qualiaDB engine to-do — **assumed completed** for the purposes of this requirements doc)

---

## 1. Purpose and scope

This document specifies the qualia-ui functionality required to deliver **3D modelling and rendering, animation, curation/presentation/representation of complex datasets and data-sources, and audio** — in two parts: (A) an **audio desk** for multi-track production / mixing / routing, and (B) **immersive audio** for manifold-scoped spatial soundfields.

It assumes the engine to-do (companion doc) is completed: every engine record, command, ontology, and capability referenced here is treated as available via the desktop Tauri command bus, the MCP tool surface, and the WASM runtime exports.

### 1.1 Honesty labels

- **live** — fully implemented and functional in qualia-ui today
- **present** — UI structure exists but backend not wired
- **missing** — no UI surface at all
- **planned** — referenced in specs/manifold seeds but not implemented

### 1.2 Current qualia-ui state (relevant surfaces)

| Surface | File | Status |
|---------|------|--------|
| `3d` container view | `browser/container_views.rs` / `container_views_ext.rs` | **present** — placeholder, no real wgpu surface |
| `subcanvas` container view | `browser/containers.rs` | **present** — placeholder |
| `vision` container view | `browser/container_views_ext.rs` | **present** — placeholder |
| `listen` container view | `browser/container_views_ext.rs` | **present** — placeholder |
| `triad` container view (Q42/D10 audio triad) | `browser/container_views_ext.rs` | **present** — placeholder |
| `media` container view | `browser/container_views.rs` | **present** — placeholder |
| `webrtc` container view | `browser/container_views_ext.rs` | **present** — placeholder |
| `webview` container view | `browser/container_views_ext.rs` | **present** — placeholder |
| Media manifold seed | `tool_chest/manifolds/media.rs` | **present** — seed only |
| Vibe manifold seed | `tool_chest/manifolds/vibe.rs` | **present** — seed only |
| Research manifold seed | `tool_chest/manifolds/research.rs` | **present** — seed only |
| Knowledge manifold seed | `tool_chest/manifolds/knowledge.rs` | **present** — seed only |
| Tool family `ten_d_assets` (10D Asset Container, Kinematics, Animations) | `tool_chest/tools/` (proposed) | **missing** |
| Tool family `media_studio` (Photo, Vector Art, Video) | `tool_chest/tools/` (proposed) | **missing** |
| Tool family `audio_speech` (Spectral Tomography, Waveforms, Q42/D10 Triads) | `tool_chest/tools/` (proposed) | **missing** |
| Tool family `spatial_gis` (Topographic GIS Maps, 4D Tracks) | `tool_chest/tools/` (proposed) | **missing** |

---

## 2. 3D surfaces

### 2.1 3D container (enhance existing placeholder)

**Current:** `3d` container view is a placeholder.

**Required: a real wgpu surface mounted in the WASM canvas, wired to PortalGpu.**

| Feature | Engine binding | Priority |
|---------|----------------|----------|
| Render Tensor10D projected nodes (ambient particle field) | `PortalGpu::upload_tensor`, `set_ambient_enabled` | P0 |
| Render imported mesh (OBJ/STL/GLB) with per-mesh colour | `render/assets.rs` import → `PortalGpu::set_mesh` | P0 |
| Orbit / pan / zoom camera | `render/navigation.rs`, ICP `OP_SET_CAMERA_DELTA` | P0 |
| Click-to-pick tensor node or mesh face | `PortalGpu` picking texture + `pending_pick` readback | P1 |
| Multi-artefact scene (bounded scene graph) | R3D-6 scene graph | P1 |
| Directional + point lighting | R3D-7 lighting | P1 |
| PBR / Phong material editor | R3D-8 material system | P1 |
| Shadow mapping | R3D-9 shadows | P2 |
| Volumetric rendering (density fields, CT/MRI) | `webizen-render/volumetric.rs` | P1 |
| Isosurface extraction (scalar field → mesh) | `computational_geometry/isosurface.rs` | P1 |
| Point cloud with per-node colour (epistemic_q / sensitivity) | PortalGpu ambient + colour mapping | P1 |
| 3D graph (RDF-Star node-link, force-directed or PGA-anchored) | `graph_index` + `render/pga.rs` + PortalGpu | P1 |
| GIS terrain mesh + map tiles | R3D-14..16 GIS | P1/P2 |

**Container header:** artefact name, LOD badge, sensitivity badge, capability badge (Native/Web/Limited), view-mode selector (3D / 2D degraded / wireframe / solid / point-cloud / volumetric).

**Container toolbar:** import mesh, upload tensor, add light, add artefact, snapshot to .10d, export GLB/OBJ/STL, toggle picking, toggle bloom, toggle shadows.

### 2.2 3D manifold (new manifold seed: "Studio" or enhance "Media")

A dedicated 3D-authoring manifold with containers:

| Container | Type | Purpose | Priority |
|-----------|------|---------|----------|
| Scene View | `3d` | The main wgpu viewport | P0 |
| Scene Graph | `tree` | Hierarchical list of artefacts + lights + cameras | P1 |
| Material Editor | `sheet` | Per-mesh material (colour, roughness, metalness, emission) | P1 |
| Lighting Editor | `sheet` | Per-light params (colour, intensity, direction, falloff) | P1 |
| Tensor Inspector | `sheet` | Tensor10D field editor (σ/α/μ/x/y/z/v/w/q/t) with live preview | P1 |
| LOD Chain | `tree` | Per-LOD level (decimation %, error, .10d size) | P2 |
| Animation Timeline | `timeline` (new) | See §3 | P0 |
| Asset Library | `library` | Imported meshes + .10d containers + tensors | P1 |

### 2.3 Tool family: `ten_d_assets`

| Tool | Purpose | Engine | Priority |
|------|---------|--------|----------|
| 10D Asset Container | Browse / load / save .10d containers (mesh + LOD + animation + tensor) | `render/compile_10d.rs` | P0 |
| Kinematics | Edit artefact joints (slide/spin axis, range, speed) | `render/physics/joint.rs` | P1 |
| Animations | Edit keyframes / clips / tracks (see §3) | `render/animation/` | P0 |
| PGA Motor Editor | Edit motor algebra (so3 exp/log, quaternion slerp) for camera/artefact transforms | `render/pga.rs` | P1 |

---

## 3. Animation surfaces

### 3.1 Animation Timeline container (new container type: `timeline`)

**View:** Horizontal timeline with tracks stacked vertically. Each track targets an artefact + property (position / rotation / scale / colour / opacity / custom). Keyframes are dots on the track; the playhead is a vertical scrubber.

**Interactions:**
- Add keyframe at playhead (records current artefact state)
- Drag keyframe to move in time
- Edit keyframe value + interpolation (Step/Linear/Hermite/CubicBezier) + ease in/out
- Add clip (bounded keyframe group with loop mode + speed)
- Drag clip on track to resequence
- Play / pause / scrub / set speed (ICP commands to AnimationPlayer)
- Loop mode selector (Once/Loop/PingPong)
- Track blend mode (Replace/Add/Override) + weight slider

**Engine:** ANI-1..ANI-6 (keyframe/clip/track/mixer/player), `render/control.rs` ICP commands.

### 3.2 Animation tool family (`ten_d_assets/animations`)

| Tool | Purpose | Engine | Priority |
|------|---------|--------|----------|
| Keyframe Editor | Add/edit/delete keyframes; interpolation curves | ANI-1 | P0 |
| Clip Editor | Compose keyframes into clips; loop mode; speed | ANI-2 | P0 |
| Track Sequencer | Sequence clips on artefacts; blend modes | ANI-3 | P0 |
| Procedural Nodes | Sine/noise/spring/IK procedural animation | ANI-7 | P1 |
| Motion Path | RRT/roadmap planned path → animation clip | ANI-8 + `motion_planning.rs` | P1 |
| Particle Editor | Emitter / lifetime / velocity / colour ramp | ANI-10 + `render/gpu/particles.rs` | P1 |
| Ragdoll/Skin | Skeleton + skin weights + physics joints | ANI-9 | P2 |

### 3.3 Time-scrub integration

**Requirement:** The timeline playhead drives `time_scrub.rs` — scrubbing materialises the scene frame at that time (onset/decay alpha ramps + keyframe sample). The 3D viewport updates live as the playhead moves.

**Engine:** ANI-11..ANI-12.

### 3.4 Animation export

**Requirement:** Export animation to `.10d` animation section (ANI-13) or glTF 2.0 animation (ANI-14). The export action lives in the timeline container toolbar.

---

## 4. Complex-dataset presentation / curation surfaces

### 4.1 Dataset manifold (new manifold seed: "Datasets" or enhance "Knowledge")

A dedicated dataset-curation manifold with containers:

| Container | Type | Purpose | Engine | Priority |
|-----------|------|---------|--------|----------|
| Dataset Registry | `table` | List of imported datasets (name, kind, source, row/node count, sensitivity, licensing) | DAT-1..DAT-2 | P0 |
| Dataset Importer | `form` | Import CSV/JSON/Parquet/RDF/N3/JSON-LD/.10d/tensor → dataset descriptor | DAT-3 | P0 |
| Presentation Editor | `sheet` | Compose datasets + view specs into a presentation; layout editor | DAT-5..DAT-6 | P0 |
| View Canvas | `subcanvas` | The multi-view presentation canvas (renders the ViewSpecs) | DAT-6..DAT-7 | P0 |
| Annotation Panel | `list` | Annotations on the selected dataset (target, annotator, body, sensitivity) | DAT-26 | P1 |
| Lineage Graph | `graph` | Derived dataset lineage (parent → transform → derived) | DAT-27 | P1 |
| Publish | `form` | Publish presentation to Permissive Commons / bilateral micro-commons | DAT-28 | P2 |

### 4.2 View kinds (rendered inside the View Canvas / subcanvas)

Each ViewSpec renders as a sub-view inside the presentation canvas. The view kind determines the renderer:

| View kind | Renderer | Engine binding | Priority |
|-----------|----------|----------------|----------|
| Table | HTML/SVG table with sort/filter/pagination | DAT-9 | P1 |
| Graph (2D node-link) | SVG/canvas force-directed layout | DAT-10 | P1 |
| Graph (3D node-link) | PortalGpu 3D + picking | DAT-11 | P1 |
| 3D mesh | PortalGpu 3D container (§2.1) | DAT-12 | P1 |
| 3D point cloud | PortalGpu ambient particles | DAT-13 | P1 |
| 3D volume | `webizen-render/volumetric.rs` | DAT-14 | P1 |
| 3D isosurface | marching cubes → mesh → PortalGpu | DAT-15 | P1 |
| Chart (bar/line/scatter/area) | New `render/charts.rs` (SVG or wgpu) | DAT-16 | P1 |
| Map (2D/3D GIS) | `render/gis.rs` + GeoSPARQL | DAT-17 | P2 |
| Tensor heatmap | Colour-mapped grid | DAT-18 | P1 |
| Tree / hierarchy | Tree layout (SVG or canvas) | DAT-19 | P1 |
| Timeline | `render/time_scrub.rs` + provenance bounds | DAT-20 | P1 |
| Spectrogram | STFT → colour-mapped time-frequency | DAT-21 | P1 |
| Image | `computer_vision` decode + display | DAT-22 | P1 |
| Video | `computer_vision/cv/video` frame sequence | DAT-23 | P2 |
| Audio waveform/spectrogram | `qualia-audio` features + `render/acoustic.rs` | DAT-24 | P1 |

### 4.3 View Spec editor

**Requirement:** When a user adds a view to a presentation, a View Spec editor opens:

| Field | Options | Engine |
|-------|---------|--------|
| Dataset | Selector from Dataset Registry | DAT-1 |
| View kind | Selector from §4.2 table | DAT-6 |
| Filter | SPARQL query builder / SHACL shape selector / sensitivity filter / range filter | DAT-7 |
| Colour mapping | Field selector → colour ramp / spectral σ → CIE XYZ | DAT-8 |
| Projection params | 3D camera, graph layout algorithm, chart axes, tensor slice | DAT-6 |
| Governance | Sensitivity class, attestation required, rights-bounded refusal | `render/authoring.rs` |

### 4.4 Curation workflow

**Requirement:** The curation workflow is:

1. Import datasets (Dataset Importer)
2. Create presentation (Presentation Editor)
3. Add views (View Spec editor → subcanvas)
4. Annotate (Annotation Panel)
5. Derive new datasets (filter/aggregate/join/mesh/tensor-slice → DerivedDataset)
6. Publish (Publish container → Permissive Commons or bilateral)

**Engine:** DAT-25 (CurationSession), DAT-26 (Annotation), DAT-27 (DerivedDataset), DAT-28 (PresentationPublish).

### 4.5 CV-assisted curation tools

| Tool | Purpose | Engine | Priority |
|------|---------|--------|----------|
| Image → Mesh | Photo → twin bridge → mesh IR → 3D view | DAT-29 + `computer_vision/spatial` | P1 |
| Image → Embedding | Photo → perceptual hash / colour hist → similarity search | DAT-30 + `computer_vision/embeddings` | P1 |
| Super-Resolve | Enhance image dataset (classical/CNN/ESRGAN/Swin) | DAT-31 + `computer_vision/sr` | P2 |
| Medical Imaging | DICOM / histopathology / radiomics → 3D volume / slice view | DAT-32 + `computer_vision/bio` | P1 |

### 4.6 Geometry-assisted curation tools

| Tool | Purpose | Engine | Priority |
|------|---------|--------|----------|
| Point Cloud → Mesh | Alpha shape / screened Poisson / advancing front → mesh | DAT-33 + `computational_geometry` | P1 |
| Mesh Analysis | Mesh quality / TDA barcode / curvature field / Laplacian spectrum | DAT-34 + `computational_geometry` | P1 |
| CAD → Mesh | NURBS / Bézier / loft / revolve → tessellated mesh | DAT-35 + `parametric_cad.rs` | P2 |

---

## 5. Audio desk surfaces (Part A)

### 5.1 Audio Desk manifold (new manifold seed: "Studio" or enhance "Media")

A dedicated audio-production manifold with containers:

| Container | Type | Purpose | Engine | Priority |
|-----------|------|---------|--------|----------|
| Desk Surface | `desk` (new) | The mixing console: channel strips + master bus + aux buses | AUD-1..AUD-5 | P0 |
| Channel Strip | `strip` (new, opens per-channel) | EQ + comp + gate + delay/reverb + gain/pan/mute/solo + send levels | AUD-6..AUD-10 | P1 |
| Transport | `transport` (new) | Play/pause/stop/record/scrub/time-display | AUD-5 + `capture.rs` | P0 |
| Meter Bridge | `meters` (new) | Per-channel + per-bus peak/RMS/LUFS/true-peak meters + spectrum analyzer + phase correlation | AUD-13..AUD-15 | P1 |
| Routing Matrix | `matrix` (new) | Channel → bus → master routing graph editor; sends; VCA groups | AUD-2..AUD-4 | P0 |
| Automation Lanes | `timeline` (reuses §3.1) | Per-channel parameter automation (gain/pan/EQ/comp/send) | AUD-11..AUD-12 | P1 |
| Scene Binding | `form` | Bind desk output → manifold position (immersive audio, §6) | AUD-18 | P1 |
| Patch Library | `library` | Saved desk patches (mix templates) | AUD-19 | P2 |

### 5.2 Desk Surface container — detailed requirements

**Layout:** Horizontal channel strips, each with:

| Section | Controls | Engine |
|---------|----------|--------|
| Header | Channel name, colour, source indicator (track/bus/sidechain) | AUD-1 |
| Input | Gain trim, phase invert, input meter | AUD-1 |
| EQ | Multi-band parametric (freq/gain/Q per band, bypass per band) | AUD-6 |
| Dynamics | Compressor (threshold/ratio/attack/release/knee), gate/expander, sidechain input | AUD-7..AUD-8 |
| Sends | Aux send level per bus (pre/post fader) | AUD-3 |
| Fader | Channel gain fader (vertical slider), pan knob, mute, solo | AUD-1 |
| Output | Bus assignment, direct out, channel meter | AUD-2 |

**Master section:** Master bus fader + meter + output routing (manifold / PCM sink / file).

**Interactions:**
- Drag channel strip to reorder
- Drag send level to adjust
- Click EQ band to edit (opens curve editor overlay)
- Click comp to edit (opens transfer function overlay)
- Solo/mute affects bus routing in real time
- All changes emit DeskPatch mutations via IntentBus (replay-safe)

### 5.3 Routing Matrix container

**View:** Grid of channels (rows) × buses (columns). Cell = send level (0..1). Master bus column highlighted. VCA group assignments shown as coloured overlays.

**Interactions:** Drag cell to set send level; drag channel to VCA group; add/remove bus; add/remove channel.

### 5.4 Tool family: `audio_speech`

| Tool | Purpose | Engine | Priority |
|------|---------|--------|----------|
| Spectral Tomography | STFT/CQT spectrogram view + edit (tf_surface) | `audio/stft.rs`, `audio/cqt_bake.rs`, `audio/tf_surface.rs` | P1 |
| Waveforms | PCM waveform view + edit | `qualia-audio/wav.rs` | P1 |
| Q42/D10 Triads | Tensor10D → acoustic params → AudioWorklet synth (generative mode) | `audio/acoustic_plane.rs`, `render/acoustic.rs` | P1 |
| Pitch / Music | f0 / onset / tempo / chroma / structure detection | `qualia-audio/music.rs` | P1 |
| Speech | Speech detection / VAD / language annotation | `qualia-audio/speech.rs`, `language.rs` | P1 |
| MIDI | MIDI import/export, audio-to-MIDI | `qualia-audio/midi/` | P2 |
| FX Rack | Delay / reverb / saturation / custom FX chain | `qualia-audio/fx/`, `audio/acoustic_sab.rs` | P1 |

---

## 6. Immersive audio surfaces (Part B)

### 6.1 Immersive Audio container (new container type: `spatial_audio`)

**Purpose:** Place sound sources in a 3D manifold space, configure room acoustics, set listener position/orientation, and render binaural / Ambisonic / multi-channel output.

**View:** A 3D viewport (reuses the §2.1 3D container) with overlaid audio source markers. Each marker is a SpatialAudioSource positioned at (x,y,z). The listener is shown as a head icon with orientation arrow.

**Toolbar:**
- Add source (point / directional / ambient / ambisonic)
- Set listener position / orientation
- Set room model (Sabine / ray-traced / analytic)
- Render mode selector (Binaural / Ambisonic / MultiChannel)
- Play / pause / scrub
- Sonify tensor nodes (toggle)

### 6.2 Spatial Audio Scene editor

| Field | Options | Engine |
|-------|---------|--------|
| Source kind | Point / Directional / Ambient / Ambisonic | IMM-1 |
| Source position | (x,y,z) — drag in 3D viewport or numeric entry | IMM-1 |
| Audio sheet | Generative (spectral params) or PCM (file ref) | `audio_contract.rs` |
| Directivity | Omni / cardioid / figure-8 / custom | IMM-1 |
| Gain / mute | Per-source gain + mute | IMM-1 |
| Room model | Sabine (dimensions + absorption) / Ray-traced (mesh import) / Analytic | IMM-3 |
| Render mode | Binaural (HRTF) / Ambisonic (1st order HOA) / MultiChannel (5.1/7.1/22.2) | IMM-2..IMM-4, IMM-8..IMM-10 |

### 6.3 Room editor

| Field | Options | Engine |
|-------|---------|--------|
| Room kind | Sabine / RayTraced / Analytic | IMM-3 |
| Dimensions | Width × length × height (m) | IMM-3 |
| Absorption coefficients | Per-octave-band absorption (table) | IMM-3 |
| Reverb time | RT60 (s) — derived or manual | IMM-3 |
| Early reflections | On/off + reflection count | IMM-11 |
| Late reverb | Convolution (RIR) or Sabine | IMM-12 |
| Geometry import | Import mesh (room geometry) for ray-traced acoustics | IMM-13 |

### 6.4 Listener editor

| Field | Options | Engine |
|-------|---------|--------|
| Position | (x,y,z) in manifold space | IMM-2 |
| Orientation | Yaw / pitch / roll (or drag head icon in 3D viewport) | IMM-2 |
| Head tracking | On/off (uses device orientation sensor or manual) | IMM-5 |
| HRTF set | Default (MIT KEMAR) or personalized (uploaded) | IMM-6 |
| Output config | Stereo / 5.1 / 7.1 / 22.2 / Dolby Atmos / Ambisonic AmbiX/FuMa | IMM-10 |

### 6.5 Manifold-aware audio

**Requirement:** The spatial audio scene is bound to a manifold. When the user switches manifolds, the audio scene changes (different room acoustics, different sources). When the listener moves between manifolds, a crossfade / portal transition plays.

| Feature | Engine | Priority |
|---------|--------|----------|
| Manifold acoustic properties per manifold_index | IMM-15 | P1 |
| Tensor-node sonification (σ→pitch, α→amplitude, q→timbre) | IMM-17 | P1 |
| Manifold transition audio (crossfade/portal) | IMM-16 | P2 |
| Synchronized with visual time-scrub | IMM-20 | P1 |
| Synchronized with animation timeline | IMM-19 | P1 |
| Synchronized with presentation views | IMM-18 | P1 |

### 6.6 Immersive audio tool family (`audio_speech` extended)

| Tool | Purpose | Engine | Priority |
|------|---------|--------|----------|
| Spatial Source Editor | Add/edit/move spatial sources in 3D | IMM-1 | P0 |
| Room Designer | Room model editor (Sabine/ray-traced) | IMM-3 | P0 |
| HRTF Config | HRTF set selection + head tracking | IMM-5..IMM-6 | P1 |
| Ambisonic Decoder | Speaker array config + decode | IMM-8..IMM-10 | P1 |
| Sonify Tensor | Tensor10D nodes → spatial sources (phenomenal σ→Hz) | IMM-17 + `render/acoustic.rs` | P1 |
| Audio Scrub | Time-scrub the spatial audio scene | IMM-20 | P1 |

---

## 7. Cross-cutting UI requirements

### 7.1 Honesty labels

Every 3D / animation / dataset / audio container and tool shows an honesty badge (live/present/missing/planned). The existing manifold seed pattern (`honesty: "missing"`) extends to all new containers.

### 7.2 Provenance and audit

Every 3D artefact, animation keyframe, dataset, presentation, annotation, desk channel, and spatial source shows its provenance: author DID, timestamp, predecessor_id, blob_hash, epistemic_status, sensitivity class. The Pulse stream shows the live audit trail.

### 7.3 Sensitivity class badges

Every record shows its sensitivity class (Public / Restricted / Classified). Medical imaging datasets (DAT-32) default to Restricted. Selfhood-adjacent audio (voice biometrics) defaults to Selfhood. The UI must never allow a Selfhood record into a Permissive Commons publication flow.

### 7.4 Capability gating

Every 3D / audio / dataset operation that requires a capability (GPU, WASM, daemon, HRTF, Ambisonic, ray-traced acoustics, super-resolution) checks the capability at call time and shows a disabled state with an explanation if unavailable. The 3D viewport degrades to 2D (per `render/authoring.rs` `Collapsed2D`) on constrained devices.

### 7.5 Performance and thermal awareness

3D rendering, HRTF convolution, ray-traced reflections, and super-resolution are thermal-governed. The UI shows the thermal state (Cool/Warm/Critical) and throttles accordingly. On Warm, optional effects (bloom, shadows, ray-traced reverb) are disabled. On Critical, 3D degrades to 2D.

### 7.6 Accessibility

- 3D viewport must be keyboard-navigable (orbit/pan/zoom via arrow keys + modifiers)
- Audio desk must be screen-reader accessible (channel names, levels, mute/solo state announced)
- Spectrogram / waveform views must have sonification alternatives for visually impaired users
- All colour mappings must have accessible alternatives (colour-blind safe palettes, patterns, labels)
- Animation timeline must be keyboard-navigable (play/pause/scrub via spacebar + arrows)

### 7.7 Sanctuary mode

In Sanctuary mode:
- 3D rendering of Selfhood-adjacent artefacts (anatomy, biometric) is hidden
- Audio capture (microphone) is disabled
- Medical imaging datasets are hidden
- Commons publication is disabled
- The UI shows only what is safe to show

### 7.8 Multi-language

All 3D / animation / dataset / audio UI strings must be translatable. The audio speech/language tools specifically support oral-tradition languages and non-western phonetics (per `qualia-audio/language.rs` and `lexicon.rs` multi-modal tokenisation).

### 7.9 File size constraints

Per the qualia-ui handover, no UI file should exceed 800 lines. The new container views (3D, timeline, desk, spatial audio, dataset registry, presentation editor) must be split into focused submodules when they approach the limit.

---

## 8. Use-case UI walkthroughs

### 8.1 House-build — 3D model + animation

1. User opens the Studio manifold → 3D container
2. Imports house plans (GLB) via Asset Library → mesh renders in PortalGpu
3. Adds terrain (GIS import → DEM heightmap mesh)
4. Places the house model on the terrain (PGA motor transform)
5. Adds lighting (directional sun + ambient sky)
6. Animates a walk-through: keyframes on camera position over time (Timeline container) → plays back as a fly-through
7. Exports the animation as glTF for sharing with the architect

### 8.2 Research project — complex-dataset presentation

1. User opens the Datasets manifold → Dataset Registry
2. Imports a CSV (experimental data) + an RDF graph (citations) + a tensor (simulation output)
3. Creates a Presentation: 4 views — Table (CSV), Graph (RDF citations, 2D node-link), Tensor Heatmap (simulation slice), Timeline (provenance)
4. Adds filters (SPARQL query on the graph; range filter on the CSV; sensitivity filter)
5. Annotates key findings (Annotation Panel)
6. Derives a new dataset (aggregate CSV by column → DerivedDataset)
7. Publishes the presentation to Permissive Commons

### 8.3 Humanitarian ICT — audio desk + immersive audio

1. User opens the Studio manifold → Audio Desk container
2. Imports field recordings (wav) → channels in the desk
3. Mixes: EQ + comp per channel, sends to reverb bus, master fader
4. Opens the Spatial Audio container → places sources in a 3D manifold (the field recording locations on a map)
5. Sets room model (Sabine, outdoor absorption coefficients)
6. Renders binaural output (HRTF) for headphone playback
7. Sonifies tensor nodes (language data → σ→pitch mapping) for an immersive soundscape
8. Exports the desk patch + spatial scene as a shareable artefact

### 8.4 Welfare-support — medical imaging + dataset

1. User opens the Datasets manifold → Dataset Importer
2. Imports a DICOM scan (computer_vision/bio/dicom_lite) → DatasetDescriptor (medical kind, Restricted sensitivity)
3. Views as 3D volume (volumetric renderer) in a 3D container
4. Extracts an isosurface (marching cubes) → mesh → 3D view
5. Annotates regions of interest (Annotation Panel, Restricted sensitivity)
6. Shares with the health provider via bilateral micro-commons (not Permissive Commons — medical data)
7. The presentation is attestation-gated (health provider must attest before viewing)

### 8.5 Civic project — 3D visualization of a public idea

1. User opens the Studio manifold → 3D container
2. Builds a 3D model of the civic idea (e.g. a proposed community garden) using the authoring tools
3. Animates a walk-through (Timeline)
4. Creates a Presentation with the 3D view + a Table (budget) + a Graph (supporters)
5. Publishes to Permissive Commons (Public sensitivity)
6. The public project page (Portal container from the collaborative UI doc) embeds the 3D presentation

### 8.6 Long-term software project — dataset + audio

1. User opens the Datasets manifold → imports the project's contribution graph (RDF-Star)
2. Views as 3D graph (node-link, force-directed) in a 3D container
3. Creates a Presentation: 3D graph + Timeline (contribution history) + Table (cost base)
4. Sonifies the contribution graph (tensor nodes → σ→pitch, α→amplitude) for an audio overview
5. Publishes the presentation to Permissive Commons

---

## 9. Implementation priority for qualia-ui

### Phase 0 — Foundation (P0, must precede everything)

- Reuse the collaborative doc's Phase 0 (IntentBus, transport, ontology loader, toolbox registration)
- Real capability detection for GPU / audio / HRTF / Ambisonic

### Phase 1 — Core 3D + animation + dataset + audio desk (P0)

- 3D container: real wgpu surface, tensor upload, mesh upload, orbit/pan/zoom (R3D-1..R3D-3)
- Animation Timeline container: keyframe/clip/track/mixer/player (ANI-1..ANI-6)
- Dataset Registry + Importer + Presentation Editor + View Canvas (DAT-1..DAT-7)
- Audio Desk container: channel strips + routing + transport (AUD-1..AUD-5)
- Spatial Audio container: source/scene/room/renderer (IMM-1..IMM-4)
- Tool families: `ten_d_assets` (10D Asset Container, Animations), `audio_speech` (Spectral Tomography, Waveforms, Q42/D10 Triads)
- Ontologies + SHACL + desktop commands (X-5..X-7)

### Phase 2 — Extended surfaces (P1)

- 3D: picking, ICP, scene graph, lighting, material, volumetric, isosurface, point cloud, 3D graph, GIS
- Animation: procedural nodes, motion path, particles, time-scrub integration
- Dataset: export, colour mapping, all view kinds, curation workflows, CV/geometry-assisted curation
- Audio desk: channel strip plugins (EQ/comp/gate/delay/reverb), automation, metering, spectrum, I/O + manifold binding
- Immersive: HRTF multi-source, Ambisonic, room acoustics, manifold-aware, sonification, presentation/animation sync
- Tool families: `ten_d_assets` (Kinematics, PGA Motor Editor), `audio_speech` (Pitch/Music, Speech, FX Rack), `spatial_gis` (Topographic GIS Maps, 4D Tracks), `media_studio` (Photo, Vector Art)
- Dynamic clippable toolchain deck (reuses collaborative doc Phase 2)

### Phase 3 — Polish (P2)

- 3D: shadows, map tiles
- Animation: ragdoll/skin, export
- Dataset: video view, presentation publish, super-res curation, CAD curation
- Audio desk: saturation, phase meter, desk persistence/sharing
- Immersive: HRTF personalization, near-field, Ambisonic speaker decoder, geometry-driven acoustics, manifold transition
- Tool family: `media_studio` (Video), `audio_speech` (MIDI)

---

## 10. File structure (proposed additions to qualia-ui)

```
qualia-ui/src/
├── browser/
│   ├── containers.rs              (extend: timeline, desk, spatial_audio, dataset_registry, presentation_editor container types)
│   ├── container_views.rs         (extend: 3d real wgpu, timeline, desk, spatial_audio views)
│   ├── container_views_ext.rs     (extend: dataset, presentation, audio views)
│   └── ...
├── tool_chest/
│   ├── manifolds/
│   │   ├── studio.rs              (new — 3D + audio desk + immersive audio manifold)
│   │   ├── datasets.rs            (new — dataset curation manifold)
│   │   └── ...
│   └── tools/
│       ├── ten_d_assets/          (new — 10D Asset Container, Kinematics, Animations, PGA Motor Editor)
│       ├── audio_speech/          (new — Spectral Tomography, Waveforms, Q42/D10 Triads, Pitch/Music, Speech, FX Rack, MIDI, Spatial Source, Room Designer, HRTF, Ambisonic, Sonify, Audio Scrub)
│       ├── spatial_gis/           (new — Topographic GIS Maps, 4D Tracks)
│       ├── media_studio/          (new — Photo, Vector Art, Video)
│       └── ...
└── ontologies/
    ├── visual.ttl                 (new — 3D scene, artefact, animation)
    ├── dataset.ttl                (new — descriptor, presentation, view_spec, annotation)
    └── audio.ttl                  (new — desk channel/bus/send, spatial source, room model)
```

---

## 11. Open questions for the principal

1. **3D mounting path**: should qualia-ui use the WASM portal path (browser wgpu, portable) or delegate to a native webizen-render surface via Tauri (higher performance, native-only)? WASM portal is more aligned with the qualia-ui WASM-first goal; native surface is faster but breaks the browser target.
2. **Animation system location**: should the timeline UI be a first-class container (proposed) or a tool-family panel that docks into any 3D container? First-class container is more discoverable; docked panel is more flexible.
3. **Dataset manifold vs Knowledge manifold**: should datasets be a new manifold or an extension of the existing Knowledge manifold? New manifold is cleaner; extension avoids manifold proliferation.
4. **Audio desk scope**: full DAW (recording + editing + mixing) or mixing/routing console only? The engine supports both (capture + production + desk); the UI scope question is about how much to expose.
5. **Immersive audio default**: binaural (headphones, most accessible) or Ambisonic (speaker arrays, professional) as the default render mode? Auto-detect from output config is the middle ground.
6. **Presentation vs QApp**: should a "Presentation" be a new first-class record or a QApp with view specs as its surface declarations? (Same question as the engine doc — the UI answer should match the engine answer.)

---

## 12. References

| Document | Path |
|----------|------|
| qualiaDB engine to-do (companion) | `C:\Projects\qualia-27062026\docs\plans\qualia-3d-animation-dataset-audio-todo-2026-08-18.md` |
| Collaborative ERP/PM engine to-do | `C:\Projects\qualia-27062026\docs\plans\qualia-collaborative-erp-pm-todo-2026-08-18.md` |
| Collaborative UI requirements | `C:\Projects\NLP\consult\20260818_qualia-collaborative-ui-requirements.md` |
| Native visual intelligence & generative 3D | `C:\Projects\qualia-27062026\docs\plans\native-visual-intelligence-and-generative-3d.md` |
| Anatomy 3D handover | `C:\Projects\qualia-27062026\docs\plans\HANDOVER-anatomy-3d-qualia-mixer.md` |
| Audio algorithms catalogue delivery plan | `C:\Projects\qualia-27062026\docs\plans\audio-algorithms-catalogue-delivery-plan-2026.md` |
| Audio algorithms catalogue gap plan | `C:\Projects\qualia-27062026\docs\plans\audio-algorithms-catalogue-gap-plan-2026.md` |
| Renderer implementation | `C:\Projects\qualia-27062026\crates\qualia-core-db\src\render\` |
| Computational geometry | `C:\Projects\qualia-27062026\crates\qualia-core-db\src\specialized_libs\computational_geometry\` |
| Computer vision | `C:\Projects\qualia-27062026\crates\qualia-core-db\src\specialized_libs\computer_vision\` |
| qualia-audio crate | `C:\Projects\qualia-27062026\crates\qualia-audio\src\` |
| Audio (core-db) | `C:\Projects\qualia-27062026\crates\qualia-core-db\src\audio\` |
| Audio contract | `C:\Projects\qualia-27062026\crates\webizen-render\src\audio_contract.rs` |
| Acoustic (phenomenal σ→Hz) | `C:\Projects\qualia-27062026\crates\qualia-core-db\src\render\acoustic.rs` |

---

_End of UI requirements. Companion engine to-do: `C:\Projects\qualia-27062026\docs\plans\qualia-3d-animation-dataset-audio-todo-2026-08-18.md`._
