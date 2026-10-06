# File Format & Linguistic Specifications: QualiaDB Hypermedia Ecosystem
> *"Pure Linked Data from Authoring to Wire: The End of XML, HTML, and JS Bloat"*

**Copyright © 2026 Timothy Charles Holborn.** All rights reserved.  
**Principal / inventor:** Timothy Charles Holborn &lt;timothy.holborn@gmail.com&gt;  
Assignment: [`COPYRIGHT.md`](../COPYRIGHT.md) · Licence: [`LICENSE`](../LICENSE)

QualiaDB defines a suite of specialized, high-performance binary, graph, and human-readable container formats optimized for decentralized knowledge representation, 3D spatial assets, LLM inference tables, universal linguistic expression, and hypermedia mindware.

---

## 1. The QualiaDB Core File Format Suite

| Format | Extension | Primary Role | Underlying Architecture |
| :--- | :--- | :--- | :--- |
| **Q42** | `.q42` | Primary Knowledge & Phonological Graph | Merkle-CRDT DAG, Nquins (`<<[ s p o g prov ]>>`), Phonemic Nodes, Sonic Tokens |
| **D10** | `.d10` | 10D Projector, Spatial Mesh & Articulatory EMF | 10D Dimensional manifold (`[α, μ, σ, t, ...]`), Vocal Tract Kinematics, 3D Mesh |
| **p64** | `.p64` | Columnar Tables & Neural Acoustic Tensors | 64-aligned continuous embeddings, discrete neural audio tokens (EnCodec), Forge weights |
| **HCF** | `.hcf` | Hypermedia Content Document | RDF-Native (`yaml-ld-q42` source / `CBOR-LD` binary) with Context Markup |
| **HMC** | `.hmc` | HyperMedia Container Archive | Cryptographic archive packaging HCF, Q42, D10, and p64 assets with Bao streaming |

---

## 2. Universal Linguistic Support: The Ontological Grapheme Architecture

Traditional computing architectures rely strictly on Unicode codepoints and standard BCP 47 language tags. As a result, unencoded, historical, specialized phonetic, and gestural/signed linguistic systems cannot be natively digitized.

QualiaDB resolves this by treating language not as flat character strings, but as **first-class semantic nodes (Ontological Graphemes)** within the Q42 graph.

### A. The Semantic Grapheme Model
When a document contains text in an unencoded script, each glyph or semantic unit is represented as a structured node:

```yaml
# yaml-ld-q42 representation of an unencoded project grapheme
- "@id": "grapheme:project_glyph_042"
  "@type": "qualia:OntologicalGrapheme"
  "qualia:language":
    "@id": "lang:example_unencoded"
    "qualia:tag": "und-x-project"
    "qualia:family": "UnencodedPlaceholder"
  "qualia:visualGeometry": "asset://mesh/project_glyph_042.d10"  # 3D/2D vector geometry
  "qualia:strokeSequence": "asset://strokes/glyph_042.svg"       # Stroke order data
  "qualia:phoneticGrounding": "grapheme:project_phoneme_042"     # Linked phonological node in Q42
  "qualia:ipa": "ŋal"                                           # IPA approximation
  "qualia:puaMapping": "\\uE042"                                # Unicode Private Use Area fallback
  "qualia:semanticConcept": "concept:FreshwaterSpring"          # Direct ontological grounding
```

### B. Multi-Modal Rendering & Accessibility
1. **Visual Display:** Poet renders the character using direct GPU mesh rendering (`.d10`) or Bezier paths, entirely bypassing system font limitations.
2. **Audio Synthesis:** Non-written or oral-first languages synthesize speech directly through the generative articulatory pipeline without text-to-speech approximations.
3. **Gestural & Signed Languages:** Link directly to 3D skeletal animation kinematics in `.d10` representing spatial signs.

---

## 3. Native Auditory Intelligence: Generative Articulatory Speech Pipeline

Traditional audio files (WAV, MP3, AAC) are flat 2D air-pressure recordings over time ("acoustic exhaust"). They are immutable, difficult to manipulate semantically, and computationally disconnected from 3D visual rendering.

QualiaDB replaces flat playback with a **Generative Articulatory Speech & Auditory Pipeline** operating across the **`q42` $\leftrightarrow$ `p64` $\leftrightarrow$ `d10` Triad**:

```
┌──────────────────────────────────────────────────────────────────────────┐
│                   Generative Articulatory Audio Pipeline                 │
│                                                                          │
│  ┌────────────────────────────────────────────────────────────────────┐  │
│  │ 1. Q42 Phonological & Intent Graph                                 │  │
│  │    - Phonemic Nquins (IPA / Mother-Tongue Phone Nodes)             │  │
│  │    - Relational Prosody (Pitch, Stress, Emotion, Cadence)          │  │
│  │    - Sonic Tokens (8-byte compact event control ABI)               │  │
│  └─────────────────────────────────┬──────────────────────────────────┘  │
│                                    │ Semantic Intent & Prosody Map       │
│  ┌─────────────────────────────────▼──────────────────────────────────┐  │
│  │ 2. P64 Neural Acoustic Latents & Tensors                           │  │
│  │    - 64-aligned continuous embeddings / discrete EnCodec tokens    │  │
│  │    - Forge WGSL kernel compute graphs & residency planner          │  │
│  └─────────────────────────────────┬──────────────────────────────────┘  │
│                                    │ Continuous Acoustic Instructions    │
│  ┌─────────────────────────────────▼──────────────────────────────────┐  │
│  │ 3. D10 Articulatory Kinematics & The 10D EMF Projector             │  │
│  │    - U3 AcousticPlane (Tensor10D with shared spectral coordinate σ)│  │
│  │    - Virtual Vocal Tract (Glottal, tongue, lip kinematic manifold) │  │
│  │    - Emits physical acoustic wave & 100% synchronized 3D lip-sync  │  │
│  └────────────────────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────────────────┘
```

### A. The Three Pipeline Pillars

1. **`q42` (The Phonological Graph):** Stores the *linguistic and phonetic intent*. Words, morphemes, and dialectal pronunciations are represented as phonemic nquins. Prosodic features (pitch contours, emphasis, emotional modulation) are stored as graph relations rather than baked into audio samples.
2. **`p64` (Neural Acoustic Tokens & Embeddings):** 64-byte aligned columnar tables store dense neural acoustic representations and ML weights. The Forge compute engine executes streaming transformations converting phonological graph sequences into continuous acoustic latents without third-party tensor bloat (no Candle/Burn runtime dependency).
3. **`d10` (Articulatory Kinematics & 10D EMF Projector):** The `.d10` container models the physical geometry and electromagnetic/spatial projection of the **virtual vocal tract**. Grounded in articulatory phonetics (glottalization, velum position, lingual resonance), the host engine projects the acoustic latents through the physical tract parameters to compute the localized sound wave.

### B. Architectural Superpowers of the Triad
- **Speaker Identity Portability:** An agent’s "voice" is a `.d10` vocal tract profile. Swapping the underlying `.q42` phonological graph (e.g. from English to an example unencoded language) passes through the same `.d10` physical tract, preserving the speaker's vocal timbre and identity across all languages.
- **Mathematically Perfect 3D Lip-Sync:** Because the `.d10` articulatory parameters represent the actual physical mouth/lip/jaw states used to generate the acoustics, 3D avatar facial animation and lip-sync are mathematically exact by construction.
- **Unified Spectral Manifold ($\sigma$):** QualiaDB's U3 AcousticPlane shares the spectral coordinate $\sigma$ and timeline $t$ across both visual projection and auditory rendering, enabling unified cross-modal perception ("eyes and ears" intelligence).

---

## 3. Hypermedia Content Format (HCF) (`.hcf`)

HCF eliminates legacy HTML, XML, and JavaScript in favor of **RDF-Native serializations**:
- **Authoring Layer:** Expressed in **`yaml-ld-q42`** — clean, indentation-based semantic documents without closing tags or DOM boilerplate.
- **Machine & Wire Layer:** Serialized as **CBOR-LD 1.0** — binary compressed Linked Data with shared term-dictionaries for zero-copy memory mapping on WASM, Mobile, and IoT.

### A. Document Structure in `yaml-ld-q42`
```yaml
# ===========================================================================
# Clinical Case Study: Comorbidity & Spatiotemporal Telemetry
# Format: Hypermedia Content Format (HCF) v1.0 (yaml-ld-q42)
# ===========================================================================
"@context":
  "@vocab": "https://qualiadb.org/schema/hcf#"
  "snomed": "http://snomed.info/id/"
  "geo": "http://www.opengis.net/ont/geosparql#"
  "vibe": "https://qualiadb.org/schema/vibe#"
  "pulse": "https://qualiadb.org/schema/pulse#"
  "aura": "https://qualiadb.org/schema/aura#"

"@id": "doc:case_study_2026_01"
"@type": "HypermediaDocument"
"title": "Pediatric Comorbidity Analysis"
"ontology": "asset://graph/medical_ontology.q42"

"content":
  - "@type": "Section"
    "heading": "Patient Assessment"
    "blocks":
      - "@type": "Paragraph"
        "text": "Patient presented with acute symptoms of diabetic complication."
        "annotations":
          - "@type": "ContextAnnotation"
            "span": [24, 45]
            "target": "snomed:44054006"
            "aura:schema": "snomed:DiabetesMellitus"
            "prov:confidence": 0.98

      - "@type": "ReactiveCell"
        "@id": "cell:risk_score"
        "formula": "=COUNT(graph.match(?s, :hasCondition, snomed:44054006))"

  - "@type": "MultiModalContainer"
    "assets":
      - "@id": "asset:character_avatar"
        "@type": "SkeletalRig"
        "src": "asset://mesh/avatar_rig.d10"
      - "@id": "asset:locomotion_clip"
        "@type": "AnimationClip"
        "src": "asset://animations/locomotion.d10"
      - "@id": "asset:embeddings"
        "@type": "TensorTable"
        "src": "asset://embeddings/clinical_vectors.p64"

# Homoiconic VibeScript AST embedded directly in the document graph
"scripts":
  - "@id": "script:character_controller"
    "@type": "vibe:TickHandler"
    "vibe:onEvent": "vibe:on_tick"
    "vibe:body": |
      on_tick(dt) {
          // Real-time 60fps/120fps procedural animation and look-at IK
          let target = player.get_head_position();
          ik.look_at("mesh:avatar", [head_bone: "head", target: target, clamp_angle: 60.0]);
          anim.blend_2d("asset://animations/locomotion.d10", [forward: input.forward, strafe: input.strafe]);
      }

---

## 4. D10: 10D Projector, Skeletal Animation & Kinematic Rigging

The `.d10` format serves as QualiaDB's multi-dimensional geometric engine, supporting real-time game-engine tier animation, skeletal deformation, and physics:

### A. Skeletal Bone Hierarchies & PGA Motor Skinning
- **PGA Motor Transforms:** Bones and joints are oriented using 64-byte $\mathbb{R}_{3,0,1}$ Motors instead of quaternions and $4\times 4$ matrices, guaranteeing continuous screw-motion interpolation with zero gimbal lock.
- **Dual-Motor Vertex Skinning:** Mesh vertices store up to 8 bone weights per vertex for dual-motor skinning, eliminating "candy-wrapper" joint collapsing during extreme rotations.

### B. Blend Spaces & Morph Target Deformations
- **1D / 2D Blend Spaces:** Multi-clip parametric interpolation maps directional input vectors (`forward`, `strafe`, `speed`) to smooth locomotion cycles.
- **Articulatory Morph Targets (Visemes):** Morph target deformation deltas for facial expressions and speech synthesis, dynamically driven by the `q42` phonological graph and `.p64` acoustic latents.

### C. Keyframe Tracks & AnimNotify Event Descriptors
- **Hermite/Bezier Keyframe Curves:** High-precision temporal tracks controlling bone transforms, blend weights, and spatial parameters.
- **Embedded AnimNotifies:** Keyframe markers that emit structured `on_anim_event` hooks into VibeScript (e.g. triggering spatial sound pulses, physics impulses, or particle emitters).
```

---

## 5. The `www/` Legacy Web Partition & Sandboxed Bridge

While Poet, VibeScript, and HCF standardize on pure Linked Data (`yaml-ld-q42`, CBOR-LD, `.q42`), real-world applications frequently require **legacy HTML/JS/CSS assets**, micro-frontends, or third-party web visualization libraries (D3.js, Three.js, Chart.js, PDF.js).

To accommodate this without polluting the core semantic graph, `.hmc` containers and QApp projects support a dedicated **`www/` webroot partition**:

```text
my_hypermedia_project.hmc/
├── document.hcf             # Core RDF-Native hypermedia document (yaml-ld-q42)
├── mindware.vibe            # VibeScript reactive formulas, Pulse & Aura hooks
├── ontologies/
│   └── domain.q42           # Primary Q42 knowledge graph (Merkle-CRDT DAG)
├── assets/
│   ├── mesh.d10             # 3D spatial models, skeletal rigs & blend spaces
│   └── tensors.p64          # Neural acoustic latents & 64-aligned ML embeddings
└── www/                     # Legacy Web Compatibility Partition (HTML/CSS/JS)
    ├── index.html           # Legacy web canvas / custom widget UI
    ├── styles.css           # Vanilla CSS styling
    ├── lib/
    │   └── d3.min.js        # Third-party visualization library
    └── adapter.js           # Lightweight bridge connecting JS <-> VibeScript / QualiaDB
```

### A. Embedding Legacy Web Assets in HCF (`yaml-ld-q42`)
```yaml
# Embedding a legacy D3 or custom JS widget inside an HCF document
- "@type": "LegacyWebFrame"
  "@id": "frame:custom_chart"
  "src": "www://index.html"
  "permissions":
    - "pulse:subscribe:topic:telemetry"
    - "graph:read:concept:ActiveData"
  "sandbox": "allow-scripts allow-same-origin"
```

### B. Fiduciary Sentinel Sandboxing
Legacy JavaScript running in the `www/` partition is strictly sandboxed by the Sentinel VM:
- **Zero Direct Host Access:** JavaScript cannot directly read the host filesystem or access raw DID private keys.
- **Controlled Event Bridge (`window.qualia`):** Communication with the parent VibeScript engine occurs exclusively over a typed, capability-gated postMessage event bus.

---

## 6. Dual Runtime Profiles: Sovereign Native vs. WASM Web Compatibility

The QualiaDB ecosystem defines two distinct execution tiers, optimizing for raw 64-bit performance on native hardware while maintaining universal fallback in standard browsers:

| Feature & Capability | Profile A: Sovereign Native (Webizen Desktop & Mobile) | Profile B: WASM Web Compatibility (Chrome/Safari/Edge) |
| :--- | :--- | :--- |
| **Host Environment** | Dedicated Native Webizen Shell (Rust AOT) | Mainstream Browser (`wasm32-unknown-unknown`) |
| **Architecture & Memory** | **Full 64-bit (`p64`) Architecture**, unlimited virtual memory | 32-bit WASM linear memory sandbox (2–4 GB limit) |
| **GPU & 3D Acceleration** | **Direct WGPU 29 (Vulkan / Metal / DirectX 12)** with shared device buffers & WGSL Forge compute | WebGPU / WebGL2 through browser security wrappers and origin isolation |
| **Physics & PGA Motors** | Hardware SIMD (AVX-512 / NEON) evaluated directly on GPU/CPU | WebAssembly SIMD128 in-memory emulation |
| **Graph Storage & I/O** | Direct NVMe/SSD memory-mapped `.q42` Merkle DAG | Origin Private File System (OPFS) / IndexedDB VFS |
| **Networking & P2P** | Direct raw POSIX/WinSock TCP/UDP, QUIC, and libp2p | WebSockets / WebRTC DataChannels (CORS-scoped) |
| **Audio & Sensors** | Low-latency hardware capture/playback (WASAPI/CoreAudio/ALSA) | `AudioWorklet` SharedArrayBuffer ring bridge (`Q3AS`) |

---

## 7. HyperMedia Container (HMC) (`.hmc`)

The `.hmc` container packages all multi-modal assets into an immutable, cryptographically verifiable archive:

```
┌──────────────────────────────────────────────────────────────────────────┐
│                     HyperMedia Container Archive (.hmc)                  │
│                                                                          │
│  ┌────────────────────────────────────────────────────────────────────┐  │
│  │ 1. Container Manifest & Bao Outboard Hash Tree (BLAKE3 1-KiB)      │  │
│  │    - Header CID, DID Signatures, Permissive Commons Rights Scope   │  │
│  └─────────────────────────────────┬──────────────────────────────────┘  │
│                                    │                                     │
│  ┌─────────────────────────────────┼──────────────────────────────────┐  │
│  │                                 │                                  │  │
│  ▼                                 ▼                                  ▼  │
│┌──────────────────────┐ ┌──────────────────────┐ ┌──────────────────────┐│
││ HCF Document (.hcf)  │ │ Q42 Graph (.q42)     │ │ Spatial, Audio & www ││
││ (CBOR-LD binary AST) │ │ (Merkle-CRDT DAG)    │ │ (.d10, .p64, www/)   ││
│└──────────────────────┘ └──────────────────────┘ └──────────────────────┘│
└──────────────────────────────────────────────────────────────────────────┘
```

---

## 8. Streaming & Distributed Synchronization Protocols

### A. Bao Verified Streaming (BLAKE3 Chunk Trees)
To support seamless, real-time access to large multi-modal assets (`.p64` embedding tensors, `.d10` 3D meshes, and `www/` resources) packaged within `.hmc` archives:
- **Zero Monolithic Downloads:** The HMC container stores an interleaved Bao outboard tree containing BLAKE3 parent hashes over 1-KiB leaf chunks.
- **Random-Access Verification:** Webizen, mobile, and IoT clients can seek to arbitrary byte offsets (e.g. streaming slice `[102400..204800]`), verifying each chunk against the container's root hash in $O(\log N)$ time before executing or rendering.

### B. Merkle-CRDT State Compaction & Tombstone Pruning
For `.q42` knowledge graphs subject to frequent edits and retractions across offline agents:
- **Tombstone Pruning:** Operational deletion records are periodically consolidated via state-based delta snapshots.
- **Epoch Hashing:** `qualia_core_db` computes a new convergent Merkle root for the active state, pruning historical operational tombstones while maintaining cryptographic provenance of the active graph.
