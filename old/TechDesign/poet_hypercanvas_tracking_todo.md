# Poet HyperCanvas Mindware Workbench — Implementation Tracking & To-Do List

**Copyright © 2026 Timothy Charles Holborn.** All rights reserved.  
**Principal / Inventor:** Timothy Charles Holborn <timothy.holborn@gmail.com>  
**Assignment:** [`COPYRIGHT.md`](../COPYRIGHT.md) · Licence: [`LICENSE`](../LICENSE)

---

## 1. Executive Overview & Architecture Bridge

This document tracks the end-to-end implementation and alignment between the **Instructional Mock UI** (`C:\Projects\NLP\Canvas_Workbench`), the **Dioxus Studio Frontend** (`qualia-27062026/crates/webizen-studio/src/components/poet`), and the **Native Tauri Desktop Host** (`qualia-27062026/crates/webizen-desktop`).

```text
┌─────────────────────────────────────────────────────────────────────────────────────────────┐
│                                    CANVAS WORKBENCH (MOCK)                                   │
│  - Vanilla ES Modules / CSS Glassmorphism / Dynamic Clipable Toolbars                       │
│  - Interaction patterns, UX density, desk physics, gesture fidelity, honesty boundaries     │
└──────────────────────────────────────────────┬──────────────────────────────────────────────┘
                                               │ (Port UI Architecture & Patterns)
                                               ▼
┌─────────────────────────────────────────────────────────────────────────────────────────────┐
│                              WEBIZEN-STUDIO (DIOXUS RUST WASM/UI)                           │
│  - TopMenubar, ControlBar, Dynamic Clipable Context Ribbon, ToolChest, Stage, TechSidebar  │
│  - 13 Container Nodes, Modular In-Container Tool Suites (tools/* directories)               │
└──────────────────────────────────────────────┬──────────────────────────────────────────────┘
                                               │ (Tauri IPC / Native WGPU Surfaces)
                                               ▼
┌─────────────────────────────────────────────────────────────────────────────────────────────┐
│                              WEBIZEN-DESKTOP (TAURI V2 NATIVE HOST)                         │
│  - `poet_eval`: Real VibeScript 0.1 AST Interpreter (`poet_vibe`) & Gas Metering           │
│  - `poet_gazetteer`: NLP Document Tokenization & Entity Span Sealing (`qualia_core_db::nlp`)│
│  - Native WGPU GPU Surface & Trajectory Engine (`commands/poet_render.rs`)                  │
└─────────────────────────────────────────────────────────────────────────────────────────────┘
```

---

### 1.3 Manifold Ontology & First-Class Graph Specification

In HyperCanvas, a **Manifold** is not a generic pixel canvas; it is a **First-Class Semantic Graph / Ontological Resource (`qualia:Manifold`)** with formal W3C RDF 1.2 quad properties:

```turtle
@prefix qualia: <https://qualia.io/ns/core#> .
@prefix dcterms: <http://purl.org/dc/terms/> .
@prefix skos: <http://www.w3.org/2004/02/skos/core#> .
@prefix fiduciary: <https://qualia.io/ns/fiduciary#> .

<graph:manifold:research_epistemology_01> a qualia:Manifold ;
    qualia:uuid "urn:uuid:7f3a8b21-4d1e-4a6c-9e28-1b5e8c3a9d01"^^qualia:UUID ;
    dcterms:title "Research & Epistemological Mindware Manifold"@en ;
    qualia:category qualia:ResearchAndEpistemology ;
    skos:topic <topic:CatchmentHydrologyAndBioTelemetry> ;
    qualia:strataSpectrum ( "environmental" "social" "legal" "technical" ) ;
    dcterms:rightsHolder <did:qualia:timothy_charles_holborn> ;
    qualia:fiduciaryDutyLevel fiduciary:GateLevel3 ;
    qualia:permissionCircle <circle:ProjectAuthors> ;
    qualia:merkleRootHash "0x8F3A29B4E1C2D3A4"^^qualia:Blake3Hash ;
    dcterms:created "2026-08-15T00:00:00Z"^^xsd:dateTime ;
    dcterms:modified "2026-08-16T15:35:00Z"^^xsd:dateTime ;
    dcterms:license <https://qualia.io/licences/fiduciary-duty-of-care-v1> .
```

---

### 1.4 Recursive Nested Manifolds as Containers in Other Manifolds

**Underlying Principle:** Homoiconic, fractal graph containment. Any manifold can be embedded as a first-class container node (`type: 'manifold'`) on another manifold.

- [x] **In-Container Nested Graph Visualizer**:
  - Encapsulated container renders a live mini-graph preview of the nested child manifold's internal occupants and causal wires.
  - Displays ontological metadata badges: UUID, Category, Topic, and Fiduciary Duty Gate Level 3.
  - Live metric counter: Occupants count, Internal wires count, Merkle root hash.
- [x] **Interactive Portal Traversal**:
  - Action button (`Traverse into Manifold ➔`) seamlessly switches the active workspace viewport into the child manifold while preserving state and zoom transforms.
- [x] **Inter-Manifold Causal Wires**:
  - Connection wires can bridge between parent containers and the nested manifold node (e.g. `wire-r3`: `container-map-01` ➔ `container-nested-media-manifold` via `qualia:embedsSubGraph`).

---

### 1.5 Sanctuary Manifold — Selfhood & Personhood Architecture

The Sanctuary Manifold is split into two ontologically distinct layers:

- **Selfhood** (`selfhood.n3`): The deeply personal, interior aspects of a natural agent — dreams, reflections, qualia memories, private keys, biometric baselines, sensory pacing, personal values. Selfhood is NOT social, NOT legal, and NOT subject to external claim. No organisation has a right to access selfhood data. Scoped exclusively to `NaturalAgent`.
- **Personhood** (`personhood.n3`): The socially situated, legally governed aspects — social relations, legal status, rights/obligations, fiduciary relationships, causality consequences/benefits. Law applies to personhood. Applicable to both `NaturalPerson` and `LegalPerson`.

The boundary: when selfhood data is shared (e.g. a dream shared with a therapist, a private key used to sign a contract), the shared instance becomes personhood data. The original selfhood record remains protected.

1. **The Inforg (Informationally Embodied Organism)**:
   - Represents the sovereign human being as an informational entity possessing strict agency, qualitative qualia boundaries, cryptographic keyrings, and biological/cognitive pacing needs.
   - Grounded in sovereign DID `did:qualia:timothy_charles_holborn` with Fiduciary Gate Level 3 (Zero Ambient Authority).
   - **Air-Gap Cryptographic Shield**: 100% Local Encrypted BLAKE3 CRDT storage guarantees that selfhood data never leaks to external cloud servers.

2. **The Infosphere & Concentric Spheres of Autonomy**:
   - Rather than treating all network connections identically, the Infosphere is organized into **Concentric Rings of Autonomy**:
     - **Inner Core (Selfhood)**: Personal values, qualia memories, raw private keys, and neuro-sensory pacing baselines. Access requires direct biometric signature. No social relations, no legal standing. Protected from surveillance, organisational access, and compelled disclosure.
     - **Middle Sphere (Personhood — Fiduciary)**: Statutory powers-of-attorney, water/land rights trustees, and multi-sig emergency recovery delegates (3-of-5 threshold). Socially situated, legally governed.
     - **Outer Sphere (Personhood — Ambient)**: Autonomous mindware agents (Sentinel, Linguist, Audio Synth) and external WebRTC nodes, strictly gated via the VibeScript Sentinel capability VM.

3. **Specialized Manifold Layout Topologies (`layout: 'sanctuary'` vs `'workbench'`)**:
   - Establishes HyperCanvas's capability for **diverse spatial layout paradigms**:
     - `workbench`: Free-floating 8px desk physics nodes with pan/zoom.
     - `sanctuary`: Centered, concentric sovereign enclave with structured confidential configuration decks and ambient breathing visual feedback.

4. **Ontology files**:
   - `qualia-ui/ontologies/selfhood.n3` — selfhood classes, properties, vault types, protection scopes
   - `qualia-ui/ontologies/personhood.n3` — social relations, legal status, rights/obligations, fiduciary relationships, causality consequences/benefits
   - `qualia-ui/ontologies/duty-of-care.n3` — umbrella for all protective relations (guardianship, custodial, professional, stewardship, safety, public service, family, principal-agent)
   - `qualia-ui/ontologies/care-scope.n3` — agreements, legal domains, care contexts, scoped task assignments, software agent delegation
   - `qualia-ui/ontologies/guardianship.n3` — formal legal guardianship, ward vulnerability, family roles, intervention/challenge/replacement
   - `qualia-ui/ontologies/adversarial-conduct.n3` — adversarial strategies, methods, targets, harm scope, coordinated patterns, evidence, vulnerabilities, consequences
   - `qualia-ui/ontologies/production-document.n3` — scripts, functional specs, style guides, book publications, research reports, production briefs, document sections, entities, agent processing directives, layouts, knowledge queries
   - `qualia-ui/ontologies/game-design.n3` — game types, mechanics, objectives, characters, AI behaviour, dialogue trees, quests, items/inventory, progression, game states
   - `qualia-ui/ontologies/game-world.n3` — real-world grounding (historical, geographic, cultural, contemporary, future, fictional, hybrid), game worlds, regions, points of interest, game design documents (GDDs)
   - `qualia-ui/ontologies/learning-core.n3` — knowledge domains, skills, proficiency levels, skill assessments, skill profiles, skill gaps, recognition of prior learning (RPL), qualifications, learning events, assurance levels
   - `qualia-ui/ontologies/learning-experience.n3` — learning paths, learning activities, adaptive learning engines, tutorial environments, incidental learning trackers, elearning toolbox components, learning manifolds
   - `qualia-ui/ontologies/learning-experience-modality.n3` — experience modes (direct, lived, simulated, observational, interpersonal, educational, mediated), knowledge types (propositional, practical, experiential, interpersonal), experience records, skill development trajectories, experiential transfer, life activities (learning as byproduct), upbringing factors (background circumstances that confer knowledge), knowledge conferral (immersion-based acquisition)
   - `qualia-ui/ontologies/faith-systems.n3` — faith systems (text-based, oral, practice-based, place-based, philosophical, syncretic, ancient, personal), sacred texts, oral traditions (custodianship, protocols), sacred places (access restrictions), faith communities (institutions, congregations, peoples), practices and rituals, moral frameworks, cosmology, indigenous knowledge protocols (access levels, consent, attribution), faith adherence
   - `qualia-ui/ontologies/values.n3` — values (10 categories, conflicts, priority), projected values (statements, codes, charters, pledges), enumerated values (behaviour-derived, evidence-based), values gap (aspiration, misrepresentation, transition, contextual, hypocrisy/institutional inconsistency), values instruments (human rights, professional codes, religious frameworks, cultural customary, community charters), values agreements (relationship-grounded, content-persistent, instrument-referencing, credential-issuing), values credentials (authority-issued, community-affirmed, agreement-issued, behavioural-enumeration, self-declared; assurance levels, revocable), values networks
   - `qualia-ui/ontologies/obligations.n3` — obligations (confidentiality, secrecy, privilege, dignity, moral, community, safety, fiduciary, professional, statutory, contractual; strength: absolute/strong/standard/weak; conflicts), confidentiality instruments (NDA, deed of secrecy, suppression order, sealed record, professional code, national security, cultural protocol), human dignity (bodily integrity, identity privacy, trauma history, vulnerability, cultural-spiritual, autonomy, reputation, freedom from degradation), zero-knowledge validation (range proof, set membership, credential verification, attribute proof, relationship proof, obligation proof), coercive circumstances (physical/psychological/economic/legal/institutional/familial coercion; threat levels; endangered parties; ZKP options), disclosure exceptions (imminent harm, child protection, court order, law enforcement, public health, public interest, consent, whistleblower, self-defence; mandatory/permissive; ZKP-first), disclosure exclusions (law enforcement, courts, social workers, doctors, mental health specialists, parliamentarians, journalists, legal professionals, clergy, defence, researchers, auditors; oversight), obligation conflict resolution (ZKP, disclose, withhold, partial, deferred, escalated; rationale)
   - `qualia-ui/ontologies/adversarial-relational.n3` — relational aggression (social exclusion, gossip/rumour, relational sabotage, friendship weaponisation, triangulation, flying monkey recruitment, competitive victimhood), psychological manipulation (gaslighting with 5 variants: interpersonal/institutional/medical/racial/collective; emotional blackmail, trauma bonding, narcissistic abuse pattern, silent treatment, infantilisation, reality distortion), sexual manipulation (sexual coercion, sexual deception, reproductive coercion, sexual content weaponisation, sexual exploitation of dependency, sexual boundary testing), false claims & fabrication (false allegations, fabricated evidence, perjury, false medical/psychiatric claims, misleading true claims, false counter-claims/DARVO), content double binds (illegal sexual content, illegal recordings, classified content, privacy-protected content, self-incriminating content; reporting paths, ZKP options, mischaracterisation risk), social harm patterns (coercive control, post-separation abuse, workplace mobbing, institutional abuse, online harassment campaigns, family system abuse, community ostracism), coercive control phases (grooming → consolidation → crisis → post-separation), proxy perpetration (willing/coerced/unwitting/partially-aware; layered chains), profession-correlated access (trades, medical, legal, political, financial, religious, educator, technology, law enforcement, social work, HR/management, carer; methods, disguises), core principle (friendly/good-faith/injury test; regardless of gender/profession/visibility/normalisation), accountability evasion (feigned incompetence, discrimination weaponisation, marginalised group weaponisation with protective bias types and intergenerational impact, provocation-response inversion, ontological invisibility), instrumentalisation factors (social bias, wealth/resource asymmetry, family network, psychiatric vulnerability, substance dependency, sexual leverage, professional authority, institutional blind spots, legal status, child dependency, isolation)
   - `qualia-ui/ontologies/adversarial-relational-scenarios.n3` — contextual scenario patterns: instrumentalisation factors (11 factors: social bias, wealth/resource, family network, psychiatric vulnerability, substance dependency, sexual leverage, professional authority, institutional blind spots, legal status, child dependency, isolation), scenario patterns (wealth-family abuse: concentrated resources against target; substance-driven exploitation: dependency as weapon, children exploited for substances; sexual network leverage: strategic relationship cultivation for premeditated attacks; adolescent relational aggression: schoolyard to suicide cascade; corporate shield weaponisation: sole directorships, phoenixing, psychiatric vulnerability as shield; adolescent online commodification: isolation → online vulnerability → grooming → commodification → trafficking pipeline; proxy harassment via protective bias: untouchable instrument, victim double bind; single-parent intergenerational transmission: primary mechanism for transmission of adversarial conduct across generations)
   - `qualia-ui/ontologies/settings.n3` — settings system: setting (base class, scope: global/profile/session/container, value types, provenance, validity), preferences (natural persons: accessibility, sensory pacing, aesthetic, workflow, language, privacy, communication), configuration (legal persons: policy, compliance, security, operational, integration), parameters (software agents: threshold, mode, allocation, timeout), capability management (name, holder, grantor, scope, status: active/suspended/revoked/expired/pending, constraints, budget), access control (capability-based policies, selfhood protection: non-delegable, non-transferable), ontology loading (modules, prefixes, load order, status: loaded/pending/failed/disabled/optional, imports), runtime parameters (profiles: sovereign-native/wasm-web/edge, max memory, cache size, processing mode: batch/streaming/interactive), fiduciary VM config (sandbox strictness: strict/permissive/locked, budget enforcement, event bridge, legacy JS), profile management (personal, work, shared-device, institutional, agent, guest; active/switchable), settings manifold (sub-manifolds: preferences, capabilities, access-control, ontologies, runtime, fiduciary-vm, profiles)
   - `qualia-ui/ontologies/communications.n3` — communications manifold: pulse events (source, channel, payload types: VibeScript/graph-mutation/notification/telemetry/agent-message/presence/sync, priority: critical/high/normal/low, provenance), channels (direct, topic, request-response, stream, group, federation; transports: WebRTC, WebSocket, MQTT, HTTP, SSE, internal, DAG sync; capability-gated, encrypted), interaction patterns (conversation, request-response, broadcast, collaboration, negotiation, delegation, notification, presence), protocol handlers (state: connected/connecting/disconnected/error/reconnecting, retry policy, encryption), routing rules (match conditions, actions: deliver/queue/drop/redirect/transform/batch, priority), agent communication (PersonCommunication: natural persons, InstitutionalTransmission: legal persons, AgentDispatch: software agents), communications manifold (sub-manifolds: conversations, channels, notifications, presence, pulse-inspector)
   - `qualia-ui/ontologies/social.n3` — social manifold: social graph (owner, edges), social edges (source, target, type, strength: strong/moderate/weak/severed, context, provenance; enumerable characteristics: duration new/recent/established/long-term/enduring, interaction frequency daily/weekly/monthly/quarterly/rare/none, reciprocity mutual/asymmetric/one-sided/transactional, power dynamics peer/hierarchical/asymmetric/dependent, disclosure level public/acquaintance/personal/intimate/selfhood, support types emotional/practical/financial/informational/social, conflict history none/minor/recurring/escalated/resolved, tags, established-via), edge types (personal: friendship, family, professional, community-member, acquaintance, adversarial; institutional: membership, partnership, sponsorship; computational: connection, federation), communities (formal, informal, place-based, interest-based, event-based, online, faith; members, leaders, rules, values), reputation (dimensions: trustworthiness, competence, reliability, integrity, conduct; levels: high/moderate/low/negative/unknown; evidence-based, context-dependent), social context (relationships, communities, norms, obligations), social protocols (initiation rules, confidentiality, conflict resolution), social manifold (sub-manifolds: social-graph, communities, reputation, social-context, protocols)
   - `qualia-ui/ontologies/social-connections.n3` — social connections companion: connection requests (ZKP-verified workflow, status: pending/verifying/guardian-review/accepted/declined/blocked/expired/withdrawn; requester, target, requested edge type, provenance), ZKP claims (age proof with age ranges under-13/13-15/16-17/18-25/25+/65+, identity uniqueness, credential, community membership, conduct clear), risk assessment (levels: none/low/moderate/high/critical; indicators: new-account, no-shared-contacts, no-community-overlap, unverified-identity, duplicate-account, conduct-history, trolling-pattern, grooming-pattern, identity-mismatch, social-engineering, phishing-vector, network-weaponisation, coercive-control-entry; evidence-based), settings recommendations (disclosure level, monitoring, guardian notification, initial period, capability scope), social-vector attack types (fake-account, trolling, grooming, phishing, social-engineering, coercive-control-entry, network-weaponisation, reputation-manipulation, isolation-tactics, radicalisation), vulnerable person categories (minor, under-guardianship, recently-separated, DV-survivor, whistleblower, public-figure, housing-insecure, cognitive-impairment, new-immigrant, substance-recovery, elderly), vulnerable person protection (mandatory for structural vulnerabilities, opt-in for situational; approval modes: always/age-based/risk-based/community-based/network-based/none; max disclosure level; monitoring: passive/active/off; alert triggers: grooming-pattern, bullying-pattern, adult-contact-minor, risky-connection, location-sharing, isolation-pattern, identity-mismatch, coercive-control-pattern, network-weaponisation, retaliation-pattern, financial-exploitation)

---

### 1.6 First-Class Theme System & Visual Token Registry

HyperCanvas enforces a dedicated **Design Token & Theme System**:

1. **`🌌 Cyber-Semantic Slate Dark` (Default & Preferred)**:
   - Deep canvas ground (`#07090e` / `#0c1017`).
   - Glowing Cartesian SVG grid with major crosshairs (`rgba(0, 210, 255, 0.08)` / `0.20`).
   - Dark frosted-glass containers (`rgba(12, 16, 23, 0.92)`).
   - High-contrast glowing accents: Cyan (`#00d2ff`), Emerald (`#00f2a9`), Amber (`#ffb834`), Violet (`#a855f7`).
2. **`⚫ OLED Pure Black` (High-Contrast Dark)**:
   - `#000000` pitch-black surfaces with crisp solid white and neon vector boundaries (> 7:1 luminance ratio).
3. **`☀️ Clinical Paper Light` (Optional Daylight Paper Mode)**:
   - Clean daylight high-contrast light mode for bright sunlight environments.
4. **Theme Access Points**:
   - Menubar: `View ➔ Theme ➔ [ Cyber-Dark | OLED Dark | Clinical Light ]`.
   - Accessibility modal: `♿ SME | a11y ➔ Section 2` + `🔄 Reset to Default Cyber-Dark Theme`.

---

### 1.7 Financial Economics, Collaborative Projects & Inter-Manifold Causal Linking Architecture

1. **💰 Financial & Economic Ledger Manifold (`qualia:FinancialEconomics`)**:
   - **Double-Entry Linked Accounting**: RDF Quad ledgers (`qualia:AccountingLedger`) asserting debit/credit triples, account reconciliations, and cryptographic SHA-256 audit digests.
   - **Multi-Project Capital Flows**: Tracks total budget pools ($142,500), committed capital, milestone-locked escrow tranches, and statutory disbursements.
   - **VibeScript VM Gas Accounting**: Real-time compute units and storage byte cost metering with automated spend-cap assertions.

2. **🚀 Collaborative Projects Nexus (`qualia:CollaborativeProjects`)**:
   - **Inter-Dependent Project Family**: Co-coordinates linked work streams (*Catchment Hydrology Phase 2*, *10D Acoustic Studio*, *Water-Rights SHACL Validation Guard*).
   - **Milestone Dependency DAG**: Visualizes deliverable progress, contributor allocations, and fiduciary gate approvals.
   - **Inter-Manifold Causal Wires**: Crossing the boundary between manifolds—completing a project milestone directly triggers an automated **$15,000 Escrow Release** on the Financial Economics manifold.

3. **Manifold Families & Clusters**:
   - Manifolds are organized into semantic clusters:
     - **Core Knowledge Family**: Research & Epistemology, Creative Media Studio, Social Governance.
     - **Economics & Collaborative Projects Family**: Financial Ledgers, Collaborative Projects Nexus.
     - **Sovereign Sanctuary Family**: Inforg Identity & Sovereign Infosphere Enclave.

---

### 1.8 Container Windowing Physics, Fullscreen Expansion & Semantic Wire Inspector Architecture

1. **Manifold-Aware Dynamic Toolboxes**:
   - The vertical/horizontal toolbox shelf dynamically highlights, filters, and prioritizes domain toolboxes matching the active manifold (e.g. `Research` emphasizes Rich Text, GIS, and RDF Graphs; `Finance` emphasizes Double-Entry Ledgers, Spreadsheets, and Rights).

2. **Container Expansion & Multi-Edge Resizing**:
   - **Double-Click Header Expansion**: Double-clicking any container header (or clicking the `⛶` expand button) transitions the container into a fixed full-viewport focus state (`.expanded-fullscreen`).
   - **Breadcrumb Navigation**: Displays a prominent `[ 🔙 Back to Manifold ]` button that restores original container position, dimensions, and desk physics.
   - **Multi-Edge 8px Grid Resizing**: Supports resizing from left, right, bottom, and corner handles with strict 8px snapping.

3. **Semantic Connector Relationship Inspector (Slide-out Lightbox)**:
   - Clicking any connection wire highlights the bezier curve with a glow pulse and opens the **Semantic Connector Inspector Panel**:
     - **Subject / Predicate / Object Triple Definition**: Displays origin and target node IRIs.
     - **Ontology Predicate Selector**: Supports `qualia:causallyPrecedes`, `qualia:embedsSubGraph`, `fin:triggersMilestoneEscrow`, `fin:reconcilesPool`, `prov:wasDerivedFrom`, `dcterms:references`, and `skos:related`.
     - **Epistemic Modality Styling**: Switches wire visual modes (`🔬 Objective Solid`, `🧠 Subjective Dashed`, `🌊 Intersubjective Dotted`, `⚖️ Normative Double`).
     - **Live RDF 1.2 Quad Preview**: Generates `<<[ :source :predicate :target <graph> ]>>`.

---

## 2. Dynamic "Clipable" Contextual Toolbars Architecture

### 2.1 Underlying Interaction Principle

In HyperCanvas, screen real estate and deep focus are paramount. Rather than cluttering every container with redundant heavyweight toolbars or forcing the user into a rigid monolithic app mode:

1. **The Dynamic Context Receptor Ribbon**:
   - The top chrome of the manifold window (directly beneath the Control Bar) contains a **Contextual Action Ribbon Receptor**.
   - When the user selects or focuses any container (e.g. Document, Spreadsheet, RDF Graph, Video/Audio Studio, Chat Graph), the specialized toolbox for that domain **dynamically clips to the top of the manifold window**.
   - As the user clicks between containers (e.g., from a Document to a Tensor Sheet), the top toolbar smoothly transitions to match the active container's domain.
   - When no container is selected or the user clicks empty canvas space, the toolbar gracefully collapses (or displays default global manifold placement tools).

2. **Clipping & Pinning Behaviors**:
   - **Auto-Context Mode (Default)**: Deck automatically switches based on the focused container or active sub-selection.
   - **Pinned Mode (📌)**: User can pin a specific toolchain deck (e.g. TinyMCE Rich Text or 10D Asset Studio) to stay persistently docked at the top.
   - **In-Container Mini-Bar**: Containers retain minimal, unobtrusive micro-controls (e.g. quick view tabs, honesty badge), delegating comprehensive editing palettes to the clipped top Toolchain Deck.

```text
┌─────────────────────────────────────────────────────────────────────────────────────────────┐
│ Top Menubar: File | Manifolds | Edit | View | Insert | Strata | Epistemic | Permissions     │
├─────────────────────────────────────────────────────────────────────────────────────────────┤
│ Control Bar: [🗂️ Overview] [1: 🔬 Research] [2: 🎨 Media] ... | Strata Deck | Epistemic Lens│
├─────────────────────────────────────────────────────────────────────────────────────────────┤
│ 📌 DYNAMIC CLIPPABLE TOOLCHAIN DECK (Physical Workshop "Socket-Sets" & 3D Prism):          │
│    [When DOC focused]   -> [ 📝 Typography (8) ▾ ] [ 🏷️ Grounding (4) ▾ ] [ 🔍 Math/NLP (2) ▾ ]│
│    [When 10D focused]   -> [ 📐 Kinematics (4) ▾ ] [ ⏱️ Animation (3) ▾ ] [ ⚡ Events (3) ▾ ]  │
│    [When SHEET focused] -> [ 🧮 Formulas (3) ▾ ] [ 📐 Tensors (3) ▾ ] [ 📥 Interop (2) ▾ ]   │
│    [When RDF focused]   -> [ 🛡️ SHACL (2) ▾ ] [ 🔍 SPARQL/N3 (3) ▾ ] [ ⚡ Merkle DAG (2) ▾ ] │
├─────────────────────────────────────────────────────────────────────────────────────────────┤
│                                                                                             │
│ [Tool Chest Dock]        SPATIAL MANIFOLD STAGE (2D/3D/4D)                                  │
│ ┌──────────────┐                                                                            │
│ │ 🎭 Epistemic │         ┌────────────────────────┐       ┌──────────────────────┐          │
│ │ 📝 Document  │         │ Selected: Document     │───────│ Sheet: Tensor Table  │          │
│ │ 📊 Sheet     │         │ (Toolchain Deck Active)│       │                      │          │
│ │ 🏷️ RDF Graph │         └────────────────────────┘       └──────────────────────┘          │
│ │ 🎨 Media     │                                                                            │
│ └──────────────┘                                                                            │
└─────────────────────────────────────────────────────────────────────────────────────────────┘
```

### 2.3 Toolchain Deck Presentation Modes & Zero-Overflow Architecture

To eliminate horizontal tool clutter and overflow while providing intuitive access, the Toolchain Deck implements four presentation modes:

1. **"Socket-Case" Drop-Tray Mode (Zero Overflow Default)**:
   - At top-level, only the active Domain Badge + compact **Toolchain Socket-Case Pills** are displayed (e.g. `[ 📐 Kinematics (4) ▾ ]`, `[ ⏱️ Animation (3) ▾ ]`).
   - Clicking a pill opens an ultra-sleek, frosted glass **Socket Case Drop-Tray** directly beneath that pill displaying all its individual tool actions with numeric shortcut tags (`1`, `2`, `3`...).
   - Guarantees **zero horizontal overflow** regardless of screen size or tool count.

2. **3D Cylindrical Rotary Prism Drum Mode**:
   - The active toolset is rendered on the front face of a **3D rotating prism drum**.
   - Scrolling the mouse wheel or clicking stepper buttons rolls the prism barrel along the X-axis in 3D (`transform: rotateX(60deg) preserve-3d`), cycling between different toolchains with realistic spatial depth and lighting.

3. **Tear-off Floating Toolchain Pods**:
   - Each socket case includes a **📌 Tear-off** button that pops that specific toolchain out into a compact, draggable floating mini-pod placed directly onto the canvas stage next to the user's active work.

4. **⚙️ Deck Preferences Drawer**:
   - Persistent preferences modal allowing users to toggle between Socket-Case Drop-Tray mode and 3D Rotary Prism Drum mode with instant live preview and `localStorage` persistence.

---

## 3. Modular Toolbox Directories & "Toolchain" (Socket Set) Architecture

### 3.1 The Tool Furniture Hierarchy

To ensure scalable organization across thousands of capabilities, the workbench adopts a physical workshop furniture metaphor:

```text
┌─────────────────────────────────────────────────────────────────────────────────────────────┐
│ 1. TOOL CHEST (The Station / Dock)                                                         │
│    - The physical dockable station (Left / Top / Bottom / Right). Holds all themed drawers. │
├─────────────────────────────────────────────────────────────────────────────────────────────┤
│ 2. TOOLBOX (The Drawer)                                                                     │
│    - A dedicated domain drawer (e.g. `media_studio`, `rdf_graph`, `rich_text`, `sheet`).    │
├─────────────────────────────────────────────────────────────────────────────────────────────┤
│ 3. TOOLCHAIN (The Socket Set / Bit Holder)                                                  │
│    - A cohesive, specialized sub-assembly or workflow suite of interoperable tools bundled │
│      together as a unit within a toolbox (e.g. "10D Kinematics & Event Toolchain").        │
├─────────────────────────────────────────────────────────────────────────────────────────────┤
│ 4. TOOL (The Instrument / Action)                                                           │
│    - An individual actionable operation, widget, or button (a single socket bit).           │
└─────────────────────────────────────────────────────────────────────────────────────────────┘
```

```text
tools/
├── rich_text/          # 📝 TinyMCE-style WYSIWYG, Semantic Spans & NLP Gazetteer
├── sheet/              # 📊 Spreadsheets, Formulas, Columnar Tensors & P64 Latents
├── rdf_graph/          # 🏷️ Aura / SHACL Validation, N3 Logic, SPARQL & Nquins
├── hypermedia/         # 🗄️ Hypermedia Library, HCF/CBOR Archives & Ingestion
├── media_studio/       # 🎨 Pro Media: Photo, Vector Art, Video & Adobe-style Creative
├── ten_d_assets/       # 🧊 Native 10D Asset Container (.10d), Kinematics, Animations & Events
├── audio_speech/       # 🎙️ Spectral Tomography, Waveforms & Q42/D10 Triads
├── communication/      # 📞 Multi-Agent Chat Graph, WebRTC Streams & DID Auth
├── spatial_gis/        # 🗺️ Topographic GIS Maps, 4D Tracks & Incident Pins
├── vibescript_code/    # ⚡ VibeScript 0.1 AST Console, Capabilities & Gas Meter
├── rights_fiduciary/   # 🔒 Fiduciary Duty Scopes, L3 Gates & Merkle Audit Logs
└── health_clinical/    # 🩺 Pathology Lab, Bio-Telemetry & 10D Anatomy Resonators
```

---

### 3.1 `tools/rich_text/` — Document & Knowledge Authoring (TinyMCE-Style)

**Underlying Principle:** High-fidelity WYSIWYG document authoring with native first-class semantic entity grounding and NLP span sealing into the Merkle-CRDT DAG.

**Toolchains (Socket Sets):**
- **Typography & Block Toolchain**: Bold (`**`), Italic (`*`), Underline, Strikethrough, Subscript, Superscript, Inline Code (`<code>`), Paragraph, Heading 1-6, Blockquote, Lists, Callout alerts (`NOTE`, `TIP`, `IMPORTANT`, `WARNING`).
- **Semantic Grounding Toolchain**: Wrap text in `<q-entity data-id="concept:IRI">` with Aura autocomplete, Subjective Qualia (`🧠`), Objective Fact (`🔬`), Normative Statute (`⚖️`).
- **Math & Multi-View Toolchain**: LaTeX Math Formula inline renderer (`$$ \int e^x dx $$`), Markdown source editor, W3C RDF 1.2 Triples graph view.
- **NLP Gazetteer Toolchain (`poet_gazetteer` IPC)**: Document tokenization, sentence analysis, gazetteer term matching, and quin sealing to Merkle DAG.

---

### 3.2 `tools/sheet/` — Spreadsheets & Columnar Tensors

**Underlying Principle:** Continuous tabular computation merging classic spreadsheet calculation with multi-dimensional EnCodec neural latents and acoustic resonance math.

**Toolchains (Socket Sets):**
- **Formula Engine Toolchain**: Dynamic `fx` formula input bar, coordinate tracker (e.g. `B2`), standard functions (`SUM`, `AVG`, `MIN`, `MAX`, `MEDIAN`, `STDEV`, `IF`, `VLOOKUP`).
- **Resonance & Hydrology Toolchain**: Continuous domain math (`RESONANCE(flow, freq)`, `PURITY_INDEX(tds, do)`).
- **Neural Tensor Toolchain**: Dedicated columnar channels for continuous EnCodec 64-byte neural latents, real-time tensor alignment.
- **Tabular Interop Toolchain**: Ingestion of CSV, TSV, CBOR-LD, and export to W3C CSVW Tabular Triples.

---

### 3.3 `tools/rdf_graph/` — Semantic Graphs, Aura & SHACL Validator

**Underlying Principle:** Direct manipulation and validation of W3C RDF 1.2 graphs, Aura ontology catalogs, and Merkle-CRDT DAG compaction.

**Toolchains (Socket Sets):**
- **Aura / SHACL Validator Toolchain**: Shape constraint checker (`minCount`, `maxCount`, `datatype`, `pattern`), real-time conformance reporting (`100% Conforming`).
- **Cross-Ontology Alignment Toolchain**: Visual mapping matrix (`owl:equivalentClass`, `rdfs:subPropertyOf`, `skos:exactMatch`, `skos:closeMatch`), inline `+ Add Alignment Row` rule creator.
- **SPARQL & N3 Logic Toolchain**: Interactive SPARQL 1.1/1.2 query console, N3 forward-chaining deduction engine.
- **Cryptographic Nquin Toolchain**: 48-byte visual Nquin generator (`<<[ Subject Predicate Object Graph DID ]>>`), Merkle-CRDT DAG compaction.

---

### 3.4 `tools/hypermedia/` — Hypermedia Library & Archive Manager

**Underlying Principle:** Sovereign asset indexing, cryptographic provenance tracking, and multi-tenant archive packaging.

**Toolchains (Socket Sets):**
- **Content-Addressed Catalog Toolchain**: Asset indexing for photos, audio, video, 3D models with BLAKE3 / SHA-256 cryptographic hashes.
- **HCF / HMC Archive Toolchain**: Packaging and unpacking of HyperCanvas Format (`.hcf`) and HyperMedia Container (`.hmc`) archives.
- **Codec & Stream Telemetry Toolchain**: Real-time telemetry inspector for AV1 video, Opus audio, EnCodec neural tensors, and glTF/GLB binary streams.

---

### 3.5 `tools/media_studio/` — Creative Suite (Photo, Vector & Video)

**Underlying Principle:** Pro-grade multi-modal creative production (similar to Adobe Photoshop / Premiere / Illustrator) grounded in semantic layers.

**Toolchains (Socket Sets):**
- **Photo & Layer Toolchain (Photoshop-style)**: Non-destructive layer stack, blending modes (Multiply, Screen, Overlay), masks, color curves, levels, HSL grading.
- **Generative Prompt Toolchain**: Multi-modal vision synthesis with seed locking and variant exploration.
- **Vector Grapheme Toolchain (Illustrator-style)**: Visual phonological glyph editor, SVG bezier vector path design.
- **Video Editing Toolchain (Premiere-style)**: Multi-track timeline (Video, Audio, Overlay, Subtitle), clip splicing, keyframing, transitions.

---

### 3.6 `tools/ten_d_assets/` — Native 10D Asset Container Format (`.10d`), Kinematics, Animations & Events

**Underlying Principle:** Sovereign binary 3D container format extending glTF / GLB with multi-joint kinematic chains, timeline animations, embedded reactive event emitters, and tensor nodes.

**Specification of `.10d` Container Format:**
- **Quantized Mesh Section**: Zero-copy geometry with packed vertex positions, normals, UVs, and material PBR shaders.
- **10D Kinematic Tensor Nodes Section**: Multi-joint articulated skeleton chains (e.g. 10D vocal tract pharyngeal articulators, robotics, anatomical resonators) with angular limits and degrees of freedom.
- **Timeline Animation Section**: Non-destructive keyframed motion tracks, interpolation curves, and scrubbing controls.
- **Reactive Event Emitters Section**: Embedded event triggers (e.g. `on_closure`, `on_collision`, `on_angle_threshold`) that emit reactive pulses directly to the Merkle-CRDT bus and trigger VibeScript callbacks.
- **BVH Spatial Index & Topology Section**: Hierarchical bounding volume hierarchy for fast zero-copy raycasting.
- **DID Provenance Sidecar Section**: Embedded cryptographic signature (`did:qualia:timothy_charles_holborn`), licence, and Merkle content hash.

**Toolchains (Socket Sets):**
- **10D Kinematics Motor Toolchain**: Interactive joint angle sliders (Jaw, Tongue Body, Velum, Pharynx), inverse kinematics solvers.
- **Animation & Timeline Toolchain**: 60fps animation playback, scrub bar, loop mode, speed multiplier.
- **Reactive Event Wiring Toolchain**: Visual trigger hook mapper connecting 3D joint thresholds to VibeScript pulses (`pulse::emit("vibe:closure_reached")`).
- **WGPU GPU Viewport Toolchain**: Native WGPU hardware-accelerated 3D viewport, CAD trackball orbit gizmo, wireframe/shaded rendering (`poet_render`).

**Underlying Principle:** Deep acoustic signal processing connecting continuous speech synthesis (Q42 phonetics) to 10D physical resonators.

- [ ] **Spectral Acoustic Tomography**:
  - Multi-slice spectrogram viewer (1 to 64 depth slices).
  - Frequency response curve analysis (20 Hz - 20 kHz).
- [ ] **Speech Synthesizer Triad (`q42 ↔ d10`)**:
  - Phonological articulator grounder.
  - Formant filter controls (F1, F2, F3 frequencies & bandwidths).
  - Bio-acoustic resonance tracker.

---

### 3.7 `tools/communication/` — Social Graph & Multi-Agent Collaboration

**Underlying Principle:** Sovereign peer-to-peer collaboration uniting human co-authors and specialized AI sub-agents through CRDT chat graphs.

- [ ] **Multi-Agent Chat Graph**:
  - Sub-Agents coordinator: Sentinel Planner AI, Fiduciary Rights AI, Phonology Grounder.
  - Multi-user chat stream with sender identity attribution (`Timothy Charles Holborn`).
  - Chat Quad Generator (every message creates a signed RDF quad).
- [ ] **WebRTC Encrypted A/V Channels**:
  - Peer-to-peer audio/video streaming with cryptographic DID handshakes.
  - Screen sharing and remote canvas pointer telemetry.
- [ ] **Decentralized Protocol Bridges**:
  - Solid LDP Pod sync, Matrix event bus, and ActivityPub federation hooks.

---

### 3.8 `tools/spatial_gis/` — Spatiotemporal GIS & Catchment Case Maps

**Underlying Principle:** 4D geospatial intelligence tracking environmental phenomena, autonomous agent trajectories, and spatial legal boundaries over time.

- [ ] **Topographic GIS Layer Manager**:
  - Layer toggles: Satellite Imagery, Hydrological Flow Vectors, Sensor Pins, Autonomous UAV Tracks, Field Survey Trails.
  - Time-scrubbing trajectory synchronization (scrubber moves agent pins along paths).
- [ ] **Incident Event Pinning**:
  - Drop geolocated case incident markers with automated severity scoring.
  - Connect incident markers to legal water-rights containers via causal wires.

---

### 3.9 `tools/vibescript_code/` — VibeScript 0.1 AST Console & Capability Engine

**Underlying Principle:** Homoiconic, type-safe DSL execution for human-agent automation with deterministic gas accounting.

- [ ] **VibeScript 0.1 Editor**:
  - Real custom AST syntax highlighting (`poet-vibe`).
  - Capability reach declarations: `requires [graph, pulse, motor];`.
- [ ] **AST Inspector & Gas Metering**:
  - Visual AST hierarchy tree viewer (Program -> Requires -> FnDecl -> Stmts).
  - Gas usage telemetry meter (`Gas: 350 / 1000 Units`).
- [ ] **Template Snippet Inserters**:
  - Quick-insert buttons: `pulse::emit`, `capability.invoke`, `graph::assert`.
  - `poet_eval` Tauri IPC execution dispatcher.

---

### 3.10 `tools/rights_fiduciary/` — Fiduciary Governance & DID Key Vault

**Underlying Principle:** Cryptographically enforced duty-of-care, permission gating, and sovereign digital identity.

- [ ] **Fiduciary Permission Gates**:
  - Assign containers to permission circles: Project Authors Circle (Full Sovereign Access), Fiduciary Duty Gate Level 3.
  - Attach duty-of-care constraints and compliance tags (`#catchment-compliance`).
- [ ] **DID Cryptographic Signing**:
  - Sign container states and Merkle DAG commits using `did:qualia:timothy_charles_holborn`.
  - Export tamper-proof Merkle audit log certificate.

---

### 3.11 `tools/health_clinical/` — Bio-Telemetry & Physiology Resonators

**Underlying Principle:** Clinical telemetry and physiological modeling using generic adult baselines (strictly no named-person clinical data).

- [ ] **Pathology Lab Telemetry**:
  - Biomarker tabular readings: Aquifer Dissolved Oxygen, Salinity (TDS), Pharyngeal Resonance Index.
  - Reference range comparisons and health status flags.
- [ ] **10D Anatomy Resonator View**:
  - 10-dimensional vocal tract and physiological resonator mesh explorer.
  - Physical kinematic motor alignment.

---

---

## 4. Accessibility, UI Modalities, Multi-Device Remote & AI Agent Navigation

### 4.1 Epistemic & Audience UI Modalities (Cognitive Adaptation)
The workbench serves diverse user roles. Rather than forcing a single density/jargon level on all users, the UI dynamically adapts its presentation tier based on the active **UI Modality Persona**:

1. **`Layperson / General Public` Modality**:
   - Focus: High-level visual narratives, natural language summaries, simplified tools.
   - Hides raw RDF IRIs, gas meters, and complex matrix math.
   - Replaces technical terms with everyday concepts (e.g. "Catchment Purity Record" instead of `qualia:ObjectiveObservation`).
2. **`Subject Matter Expert (SME)` Modality** (Hydrologist, Clinician, Legal Counsel, Historian):
   - Focus: Domain-specific vocabulary (e.g. hydrological flow rates, pathology biomarker reference ranges, statutory duty-of-care clauses, EnCodec neural acoustic latents).
   - Domain-tailored tool presets and calibrated epistemic confidence meters.
3. **`Ontologist & Core Developer` Modality**:
   - Focus: Full homoiconic AST, raw W3C RDF 1.2 quad graphs, N3 logic deduction consoles, Merkle-CRDT DAG compaction, and VibeScript 0.1 gas accounting.

---

### 4.2 Sensory & Visual Accessibility (WCAG 2.2 AAA Compliance)

- [ ] **Fluid Scalable Typography**:
  - Global text scale presets: `100% (Standard)`, `125% (Comfortable)`, `150% (Large)`, `200% (Accessible)`.
  - Dyslexic-friendly font toggle (`OpenDyslexic` / high-legibility sans-serif).
  - Configurable line-height, letter-spacing, and paragraph margins.
- [ ] **High-Contrast & Color-Vision Deficiency (CVD) Modes**:
  - *Standard Glassmorphism Dark*: Deep blue-grey with translucent surfaces.
  - *High-Contrast Dark (OLED)*: Pure black `#000` with high-luminance borders (`> 7:1` contrast).
  - *High-Contrast Light (Clinical Paper)*: Crisp daylight paper palette for bright ambient environments.
  - *Color-Vision Deficiency (CVD) Filters*: Deuteranopia, Protanopia, Tritanopia, and Monochromacy color mapping.
  - *Non-Color Reliant Epistemic Cues*: Epistemic modality badges (`🔬`, `🧠`, `🌊`, `⚖️`) use distinct physical border styles (dashed, solid, double, dotted) and icons so they never rely on color alone.
- [ ] **Motor & Motion Accessibility**:
  - `prefers-reduced-motion` compliance (disables kinetic canvas panning, flow-dash wire animations, and 3D orbit momentum).
  - Oversized click targets (minimum 44px × 44px in Touch / Accessible density).
  - Full keyboard spatial navigation (`Tab`, `Shift+Tab`, Arrow navigation between containers on the stage, `Alt+1`..`Alt+4`, `Alt+O`).

---

### 4.3 Remote Control, Companion Device & Spatial AR/VR Architecture

- [ ] **Multi-Device Companion Protocol (Phone / Tablet Remote)**:
  - Mobile / tablet companion interface connected via local WebRTC / WebSocket gateway (`crates/webizen-desktop/src/companion_gateway.rs`).
  - Remote control capabilities:
    - Dedicated touch Tool Chest (phone as an external drawer controller).
    - Precision stylus drawing / handwriting input pad.
    - 3D XYZD trackball gyro controller (tilting phone tilts canvas 3D plane).
- [ ] **Spatial WebXR / AR / VR & Apple Vision Pro Presentation**:
  - Stereoscopic 3D/4D spatial manifold projection.
  - 6DoF hand-tracking gestures for grabbing, resizing, and arranging container planes in physical space.
  - Spatial audio positioning for WebRTC streams and 10D acoustic resonators.

---

### 4.4 AI Agent Programmatic Navigation (Dual-Use Semantic Tree)

**Underlying Principle:** The exact same accessible semantic tree (ARIA roles, JSON-LD, Merkle DAG, stable container IDs) created for screen readers serves as the machine-actionable grounding representation for autonomous AI sub-agents.

- [ ] **Dual-Use Accessibility & Sub-Agent Semantic Tree**:
  - Every container, wire, tool, and manifold exposes structured `aria-*` and `data-id` properties.
  - AI sub-agents navigate the manifold by reading the semantic DOM / JSON-LD tree rather than relying on brittle computer-vision pixel scraping.
  - Sub-agents can trigger tool actions via Model Context Protocol (MCP) and VibeScript commands (`capability.invoke("Tool.action", { containerId })`).

---

### 4.5 Internationalization (i18n) & Specialized Phonetic Script Support

- [ ] **Multi-Lingual UI Localization**:
  - Full string bundle localization for top menubars, control bars, toolboxes, and status messages.
  - Bidirectional text layout engine (automatic RTL for Arabic, Hebrew, and LTR for European scripts).
- [ ] **Multilingual Semantic Grounding**:
  - Ontological `<q-entity>` spans linked to multilingual SKOS labels (`skos:prefLabel@en`, `@fr`, `@zh`, `@ar`, `@ja`).
- [ ] **Phonological & Acoustic Script Grounding**:
  - Support for unencoded phonetic script attachments and phonological grounding (Q42 phonetics).
  - Authorial provenance and DID cryptographic attribution protection.

---

## 5. Implementation Checklist & Phase Plan

```text
Phase 1: Architecture & Mock Foundation (Current)
├── [x] Define modular toolbox directories in TechDesign/poet_hypercanvas_tracking_todo.md
├── [x] Implement Dynamic Clipable Toolchain Deck in Canvas_Workbench (index.html + JS)
├── [x] Implement Physical Workshop "Socket-Case" Pods & 3D Rotary Prism Drum
├── [x] Implement Zero-Overflow Top-Level Control Pods (Strata, Lens, Dimension, 4D Time)
├── [x] Implement Accessibility & UI Modalities Drawer (Layperson, SME, Dev, High-Contrast, Text Scale)
├── [x] Build modular tool suites (rich_text, sheet, rdf_graph, ten_d_assets, media_studio, etc.)
└── [x] Update IMPLEMENT.md with Toolchain Deck and Control Pod patterns

Phase 2: Dioxus Webizen-Studio Port
├── [ ] Create modular tool components under webizen-studio/src/components/poet/tools/
├── [ ] Implement Dynamic Context Ribbon Receptor in chrome.rs / ControlBar
├── [ ] Rename Canvases -> Manifolds menu in TopMenubar
├── [ ] Implement UI Modality & A11y state signals in store.rs
└── [ ] Wire 8px desk physics and SVG connection wires in stage.rs

Phase 3: Webizen-Desktop Tauri IPC Integration
├── [ ] Connect Vibe console to poet_eval Tauri command
├── [ ] Connect RichText entity button to poet_gazetteer NLP command
├── [ ] Connect Companion Gateway (WebRTC / Tablet Remote)
├── [ ] Connect Media/3D tools to poet_render WGPU native surface
└── [ ] Verify telemetry bridge and MCP server integration
```

---

## 6. Honesty Boundaries Matrix

| Domain / Tool | Fidelity Level | Engine Grounding (Qualia) | Porting Guidance |
| :--- | :---: | :--- | :--- |
| **Rich Text & Gazetteer** | `partial` | **`qualia_core_db::nlp`** | Port to `poet_gazetteer`; seals text spans to Merkle DAG. |
| **VibeScript 0.1 Console** | `partial` | **`poet_vibe` 0.1 AST** | Port to `poet_eval`; real custom AST interpreter. |
| **Spreadsheet Math** | `present` | Local Formula / Tensor | Structural UI mock until backend calc engine binds. |
| **RDF / SHACL Validation** | `partial` | **`SHACL.extensions`** | Interactive alignment matrix + dynamic row insert. |
| **Media & 3D Kinematics** | `present` / `live` | Local CSS 3D + WGPU | Port trackball orbit to stage transform & WGPU frame. |
| **WebRTC Audio/Video** | `present` | Native WebRTC crate | Encrypted monitor placeholder; never claim fake live feed. |
| **Health Telemetry** | `present` | Generic Adult Baseline | Generic adult metrics only; strictly no named-person clinical data. |
| **Dynamic Clipable Ribbon**| `live` | Local DOM / Dioxus Signal | Dynamic mounting of domain toolbar based on active node focus. |
| **Accessibility Modes** | `live` | CSS Custom Properties | Dynamic theme classes (`a11y-high-contrast`, `a11y-scale-*`). |
| **AI Agent Semantic Tree** | `live` / `partial`| DOM ARIA / Merkle DAG | Machine-actionable dual-use tree for human a11y and AI agents. |

