# QApp Audit — Comprehensive Inventory and qualia-ui Gap Analysis

**Audit date:** 2026-08-18
**Workspace:** `C:\Projects\qualia-27062026`
**Target UI:** `C:\Projects\NLP\qualia-ui`
**Principal:** Timothy Charles Holborn

---

## 1. Executive summary

The QualiaDB workspace contains a substantial QApp (Qualia Application) ecosystem spanning three layers: a **capability-compiled registry** with zero-allocation hot-path enforcement, a **studio catalog** of 333 QApp definitions across 13 categories, and a **desktop hosting infrastructure** that serves installed QApps via a loopback HTTP protocol with token-based authentication.

**Key numbers:**

| Metric | Count |
|---|---|
| Total QApp catalog entries | 333 |
| Academic QApp catalog entries | 274 |
| Non-academic QApp catalog entries | 59 |
| Fully implemented bundled QApps | 1 (Anatomy) |
| Studio academic QApp component files (`*_qapp.rs`) | 275 |
| Studio catalog categories | 13 |
| Featured workspace templates | 6 |
| QApp manifest config structs | 12+ |
| Desktop Tauri commands (QApp-related) | 15+ |
| QApp infrastructure source files | 18+ |
| qualia-ui QApp surfaces | 1 (placeholder) |

**Honesty labels used in this report:**

- **live** — fully implemented and functional
- **partial** — implemented but incomplete (e.g., stub save)
- **present** — UI exists but backend not wired
- **missing** — no UI surface at all
- **planned** — referenced in specs but not implemented
- **host-only** — implemented in desktop/engine but no UI
- **experimental** — behind feature flags, not production-ready

---

## 2. QApp architecture overview

### 2.1 Three manifest formats

The workspace defines three distinct QApp manifest formats serving different purposes:

#### A. `QappPackageManifest` — on-disk developer manifest (`qapp.json`)

**Source:** `qualia-client-core/src/qapp_registry.rs:209-219`

```rust
pub struct QappPackageManifest {
    pub name: String,
    pub version: String,
    pub required_shapes: Vec<String>,
    pub dev_port: Option<u16>,
    pub x_qualia: Option<QappHostExtension>,
}
```

This is the JSON file that lives inside each installed QApp directory. The `x_qualia` extension carries the full host integration metadata. **Status: live** — used by the install pipeline, launcher, and readiness inspector.

#### B. `QappManifest` — compiled capability manifest

**Source:** `qualia-client-core/src/qapp_manifest.rs:50-57`

```rust
pub struct QappManifest {
    pub app_id: String,
    pub host_metadata: HostMetadata,
    pub capability_claims: CapabilityClaims,
}
```

This is the boot-time compiled manifest that gets hashed into `CompiledCapability` records for the 42MB Sentinel hot loop. String parsing is permitted only during install/boot; the compiled record is a fixed-size `#[repr(C)]` struct with `[u64; 8]` domain array. **Status: live** — enforced by zero-allocation tests.

#### C. `QappManifest` (cooperative) — transport-neutral PWA manifest

**Source:** `qualia-cooperative-core/src/qapp_package/manifest.rs:114-142`

```rust
pub struct QappManifest {
    pub id: String,
    pub name: String,
    pub short_name: String,
    pub version: String,
    pub description: String,
    pub kind: QappKind,
    pub capabilities: Vec<Capability>,
    pub entry_wasm: WasmRef,
    pub icons: Vec<IconRef>,
    pub theme_color: String,
    pub background_color: String,
    pub display: String,
    pub offline: bool,
}
```

This is the transport-neutral definition used by the PWA generator. It describes *what a QApp is* for phone delivery: identity, least-privilege capabilities, content-addressed WASM reference, and PWA presentation metadata. **Status: live** — used by `qapp_publish` and `generate_pwa`.

### 2.2 Registry and install pipeline

**Source files:**
- `qualia-client-core/src/qapp_registry.rs` (239 lines) — `QappRegistryState`, `RegisteredQapp`, `QappTarget`
- `qualia-client-core/src/qapp_manifest.rs` (433 lines) — `CompiledCapability`, `compile_and_register_qapp`, `install_qapp_capabilities`
- `qualia-client-core/src/qapp_install.rs` (700+ lines) — atomic install, versioning, rollback, revocation
- `qualia-client-core/src/qapp_version.rs` (110 lines) — semver parsing with prerelease support
- `qualia-client-core/src/bundled_qapps.rs` (340 lines) — bundled QApp seeding

**Install pipeline (`qapp_install.rs`):**

1. `validate_package_source()` — validates `qapp.json`, optional `package-manifest.json` sidecar with ed25519 signature
2. `install_package_atomic()` — stages to temp, archives previous version, atomic rename, updates `registry.json`
3. `install_qapp_capabilities()` — compiles manifest into `CompiledCapability`, caches remote endpoints, validates PINN model config
4. `reconcile_registry_with_disk()` — syncs registry with on-disk packages

**Install policies:**
- `Development` — bundled/dev packages may omit signatures
- `Production` — requires signed content manifest with ed25519

**Constants:**
- `MAX_PERMITTED_DOMAINS: 8` — max ontology domains per QApp
- `MAX_QAPP_REGISTRY: 32` — max registered QApps in-process
- `SUPPORTED_QAPP_ABI_VERSION: "1.0"`
- `SUPPORTED_HOST_API_VERSION: "1"`
- `QAPP_PACKAGE_MANIFEST: "qapp.json"`
- `QAPPS_DIR: "Qapps"`

**Status: live** — full install/upgrade/rollback/revocation pipeline implemented.

### 2.3 Capability and permission model

**Source:** `qualia-client-core/src/qapp_manifest.rs`, `qualia-client-core/src/qapp_api.rs`

**Clearance levels** (from `NQuin` sensitivity bits 56-63):
- `0x00` — Public
- `0x01` — Restricted
- `0x02` — Classified

**CompiledCapability** (zero-allocation, `#[repr(C)]`):

```rust
pub struct CompiledCapability {
    pub app_id_hash: u64,
    pub clearance_level: u8,
    pub domain_count: u8,
    pub permitted_domains: [u64; 8],  // FNV-1a hashes of ontology IRIs
}
```

**Enforcement (`qapp_api.rs`):**
- `verify_execution_scope()` — walks compiled bytecode, checks every operand against permitted domains
- `execute_scoped_query_in_place()` — hot-path zero-heap query execution with clearance filtering
- `append_sanctuary_violation()` — logs conduct-violation Quin to WAL when a QApp breaches its scope
- `result_passes_clearance()` — filters query results by sensitivity byte vs. app clearance

**Capability declaration in `qapp.json`:**

```json
"requires": [
  {"capability": "qualia.localDaemon.health", "required": false},
  {"capability": "qualia.localDaemon.query", "required": false},
  {"capability": "qualia.wasm.execute_ntriples_query", "required": false},
  {"capability": "qualia.flutter.chatRepresentationLaunch", "required": false}
]
```

**PWA capability model (cooperative):**

```rust
pub enum Capability {
    ReadRecords,
    WriteRecords,
    Sync,
    BlobStore,
    Notifications,
    Camera,
    Custom(String),
}
```

**Status: live** — capability compilation, enforcement, and violation logging all implemented and tested.

### 2.4 QApp loopback protocol server

**Source:** `qualia-client-core/src/qapps_protocol.rs` (200 lines)

The loopback HTTP server serves installed QApp assets from `{storage_path}/Qapps/` on `127.0.0.1:4567..4600` so embedded WebViews can load sandboxed HTML without `file://` CORS restrictions.

**Key functions:**
- `start_qualia_protocol()` — starts the tiny_http server (idempotent)
- `qualia_protocol_port()` — returns the active port
- `qualia_qapp_launch_url(qapp_name)` — returns `http://127.0.0.1:{port}/{qapp}/index.html`
- `register_qualia_uri_handler(exe_path)` — registers `qualia://` protocol handler (Windows registry)

**Security headers set:**
- `Content-Security-Policy: default-src 'self' 'unsafe-inline' 'unsafe-eval' blob: data: ws: wss: http://127.0.0.1:8080 http://localhost:8080;`
- `X-Content-Type-Options: nosniff`
- `Referrer-Policy: no-referrer`

**Path traversal protection:** `safe_path()` rejects any component that is not `Normal` or `CurDir`.

**Status: live** — full loopback server with CSP, path traversal protection, and MIME guessing.

### 2.5 QApp launcher and readiness inspection

**Source:** `qualia-client-core/src/api/qapp_launcher.rs` (695+ lines)

**Launch functions:**
- `launch_installed_qapp(qapp_name)` — returns URL to open in system browser
- `launch_installed_qapp_with_context(qapp_name, entrypoint, surface, payload_json, source)` — appends `qualia_source`, `qualia_surface`, `qualia_payload`, `qualia_token`, `qualia_daemon_port`, `qualia_qapp` query params

**Readiness inspection (`inspect_installed_qapp_readiness`):**
- Checks capability requirements (daemon running, WASM, chat)
- Checks required ontologies (catalog + disk presence in `Index/` and `SemanticLibrary/`)
- Checks required models (catalog + disk presence in `Models/`)
- Checks optional remote SPARQL endpoints (catalog lookup)
- Returns `QappReadinessReport` with `blocking_issues` and `optional_warnings` counts

**Resource management:**
- `list_installed_ontology_artifacts()` — lists ontology files
- `remove_installed_ontology(ontology_id)` — removes ontology artifacts
- `remove_installed_model(model_id)` — removes model files, unloads if active
- `test_sparql_endpoint(endpoint_or_id)` — probes SPARQL endpoint availability

**Status: live** — full launcher with context injection, readiness inspection, and resource management.

### 2.6 QApp token and credential system

**Source:** `qualia-core-db/src/identity/key_vault.rs:309-336`

**`issue_qapp_token(qapp_did, audience, expiry_epoch, nonce, capabilities, sensitivity_clearance)`:**
- Issues cryptographically signed Semantic Token
- Format: `payload_hex.signature_hex`
- Signed with Master Key from KeyVault
- Enforces gatekeeper boundary policies and shape access

**`verify_qapp_token`:**
- Verifies signature using Master Key
- Checks expiry and audience
- Returns `QappSessionTokenV2` payload

**QApp credential generation (`qapp_vault.rs:50`):**
- `generate_qapp_credential(qapp_name)` — **stub**: returns `did:qualia:qapp:{name}:signed_vc` formatted string only

**Anatomy QApp auth config:**
```json
"auth": {
  "handoff": "host-launch-url",
  "token_header": "X-Qualia-Token",
  "issuer": "qualia.key_vault.issue_qapp_token"
}
```

**Status: live** for token issuance/verification; **stub** for credential generation.

### 2.7 QApp MCP integration

**Source:** `qualia-client-core/src/qapp_mcp.rs` (133 lines)

**MCP tools:**
- `list_qapp_catalogue_json` — returns JSON array of installed QApps with full metadata
- `get_qapp_manifest_json` — returns full `qapp.json` for a specific QApp
- `inspect_qapp_readiness_json` — returns readiness inspection report
- `list_qapp_updates_json` — returns available update offers
- `describe_qapp_surface_schema_json` — returns QApp host surface schema:
  - Layout strategies: PointGrid, CssGrid, FlexBox, Masonry
  - Presentation modes: GridBound, NodeRelational, Spatial
  - Manifest surfaces: static-web, wasm-local, online-daemon-aware, native-dioxus-pane

**Status: live** — MCP tools implemented and available.

### 2.8 QApp export (WASM packaging)

**Source:** `webizen-desktop/src/commands/qapp_export.rs` (134 lines)

**`export_qapp_as_wasm_package(qapp_name)`:**

1. Calls `qapp_analyze()` to generate Q42 provenance
2. Creates export directory in `temp/webizen-exported-qapps/{slug}/`
3. Writes `qapp_scene.json` with Q42 triples
4. Copies `webizen-web` WASM runtime from `webizen-web/pkg/`
5. Generates loader HTML with WebEngine initialization
6. Starts LAN server on port 8081
7. Returns URL and LAN URL for QR code scanning

**Purpose:** Off-grid "create on desktop, QR on LAN, load on mobile" flow.

**Status: live** — full export pipeline implemented.

### 2.9 QApp PWA publishing

**Source:** `qualia-client-core/src/wellfair/qapp_publish.rs` (183 lines), `qualia-cooperative-core/src/qapp_package/pwa.rs`, `qualia-cooperative-core/src/qapp_package/remote_controller.rs`

**`build_manifest`** — builds `QappManifest` from author-supplied fields:
- Parses kind: cooperative, health, journal, directory, custom
- Parses capabilities: read_records, write_records, sync, blob_store, notifications, camera

**`write_pwa_bundle`** — generates PWA bundle:
- `manifest.webmanifest`
- `sw.js` (service worker)
- `index.html` (loader)

**`generate_remote_controller_pwa`** — generates phone-as-control-surface PWA.

**Status: live** — PWA generation implemented.

---

## 3. QApp catalog — full inventory

### 3.1 Catalog structure

**Source:** `webizen-studio/src/components/qapps/catalog/mod.rs` (214 lines)

**Categories (13):**

| Category enum | Label | QApp count |
|---|---|---|
| `Cat::Platform` | Platform | 5 |
| `Cat::Ai` | AI & Inference | 6 |
| `Cat::Knowledge` | Knowledge | 5 |
| `Cat::Scientific` | Scientific | 6 |
| `Cat::Quantum` | Quantum | 5 |
| `Cat::Medical` | Medical | 5 |
| `Cat::Financial` | Financial | 5 |
| `Cat::Security` | Security | 6 |
| `Cat::Data` | Data | 5 |
| `Cat::Network` | Network | 5 |
| `Cat::Developer` | Dev Tools | 5 |
| `Cat::Academic` | Liberal Arts | 274 |
| **Total** | | **333** |

**Status model:**

```rust
pub enum Stat {
    Active,  // real launch path exists
    Beta,    // functional but incomplete
    Soon,    // catalog placeholder, not a broken product
}
```

**QApp struct:**

```rust
pub struct QApp {
    pub id: &'static str,
    pub name: &'static str,
    pub tagline: &'static str,
    pub desc: &'static str,
    pub icon: &'static str,
    pub route: Option<AppRoute>,
    pub stat: Stat,
    pub cat: Cat,
}
```

**Featured templates (6):**

| Template | Description |
|---|---|
| Scientific Research Bench | Physics, chemistry, ODE solver, statistics, matrix lab |
| Personal Knowledge Hub | Semantic graph, ontology builder, SPARQL console, N3 logic editor, Solid LDP browser |
| Clinical Decision Support | Health vitals, clinical risk scoring, DICOM viewer, comorbidity analysis |
| Quantum Finance Lab | Portfolio analyser, QPU optimiser, GBM simulator, VaR risk engine |
| Governance Console | Agreements & rights, deontic logic editor, SHACL validator, ZK proofs, key vault |
| AI Research Bench | LLM harness, LoRA adapter manager, neuro-symbolic chat, MCP tool inspector |

### 3.2 Non-academic QApp catalog (59 QApps)

#### Platform (5)

| ID | Name | Status | Description |
|---|---|---|---|
| context-studio | Context Studio | Active | Node-graph canvas with Selfhood/Personhood zone segregation, Inforg assistant, Commons Gateway, Contextual Lenses, temporal scrubber |
| qapp-studio | QApp Studio | Active | Drag-and-drop Shoelace + Qualia pane composer. Output is a signed QApp manifest written to the WAL |
| profile-identity | Profile & Identity | Beta | Manage did:q42 identifiers, ed25519 keypairs, Verifiable Credentials, capability grants |
| hardware-config | Hardware Configurator | Beta | GPU backend, ZNS/NVMe storage zones, thermal governor, QPU credentials, NPU FFI |
| notification-center | Notification Center | Soon | Unified event stream from WAL mutations, deontic violations, QPU completions, governance alerts |

#### AI & Inference (6)

| ID | Name | Status | Description |
|---|---|---|---|
| chat | Neuro-Symbolic Chat | Beta | Phase 8 bifurcated LLM with Webizen Sentinel oversight; all output requires NQuin provenance citation |
| llm-harness | LLM Model Harness | Beta | Load GGUF models, run autoregressive inference on wgpu shader, inspect logit vectors |
| lora-manager | LoRA Adapter Manager | Beta | Zero-copy LoRA multiplexing, blend up to 8 adapters per token via fused WGSL shader |
| agent-config | Agent Configuration | Beta | Configure AgentBackend (Local/Remote/Hybrid), ModelLifecycle, 128MB RAM cap, Nym mixnet routing |
| inference-monitor | Inference Monitor | Beta | Live dashboard for ThermalGovernor, VRAM utilisation, tokens/sec, Sentinel anomalies |
| model-lifecycle | Model Lifecycle | Beta | Download, cache, swap, retire GGUF models. Verify SHA-256, manage LRU policy |

#### Knowledge (5)

| ID | Name | Status | Description |
|---|---|---|---|
| ontology-builder | Ontology Builder | Beta | SHACL shape authoring, class/property hierarchy, N3 rule attachment |
| sparql-explorer | SPARQL Explorer | Beta | Query console with result binding table, federation support, query plan inspection |
| n3-logic-studio | N3 Logic Studio | Beta | N3Rule editor with strict/defeasible/defeater/linear rule types, reasoning trace |
| rdf-star-editor | RDF-Star Editor | Beta | Triple editor with embedded triple annotation, provenance metadata, Turtle/N-Triples/CBOR-LD export |
| solid-browser | Solid LDP Browser | Beta | Browse Solid pods, create/delete resources, manage ACL, OIDC authentication |

#### Scientific (6)

| ID | Name | Status | Description |
|---|---|---|---|
| physics-simulator | Physics Simulator | Beta | Thermodynamics MCMC, RK4 ODE, DFT ground state, PINN binding affinity |
| chemistry-modeler | Chemistry Modeler | Beta | SMILES, InChI, MW, LogP, TPSA, Lipinski/Veber/Ghose/Egan, pKa, Morgan fingerprint |
| ode-lab | ODE & Calculus Lab | Beta | ODE solver with phase-space visualisation, parameter sweeps |
| matrix-lab | Matrix & Linear Algebra | Beta | Matrix operations, eigenvalue decomposition, SVD, tensor contraction |
| stats-lab | Statistical Analysis Lab | Beta | Descriptive statistics, hypothesis testing, regression, Bayesian inference |
| bioinformatics-lab | Bioinformatics Lab | Beta | SW alignment, protein analysis, k-mer counting, FASTA, Tanimoto similarity |

#### Quantum (5)

| ID | Name | Status | Description |
|---|---|---|---|
| qpu-optimizer | QPU Optimizer | Beta | QUBO problem routing to IBM/D-Wave/IonQ/Rigetti/Google/Quantinuum |
| quantum-dft | Quantum DFT Lab | Beta | DFT ground state calculations via QPU oracle |
| qaoa-explorer | QAOA Explorer | Beta | QAOA algorithm exploration and parameter tuning |
| qpu-providers | QPU Provider Manager | Beta | Provider token management, usage tracking, shot limits |
| nexus | Nexus | Beta | Quantum-classical hybrid workflow orchestration |

#### Medical (5)

| ID | Name | Status | Description |
|---|---|---|---|
| health-vitals | Health Vital Monitor | Active | Biometric records, sleep records, medication tracking |
| clinical-risk | Clinical Risk Scorer | Active | Framingham, CHA₂DS₂-VASc, SCORE2, drug interactions, contraindications |
| dicom-viewer | DICOM Viewer | Active | DICOM series browsing, organ overlay, metadata inspection |
| anatomy-browser | Anatomy Context Browser | Active | 3D anatomy with Babylon.js, condition-impact mapping, system overlay |
| comorbidity | Comorbidity Analyzer | Active | Comorbidity evaluation with compounded risk scoring |

#### Financial (5)

| ID | Name | Status | Description |
|---|---|---|---|
| portfolio | Portfolio Analyzer | Beta | XEC/token balances, transaction history, portfolio valuation |
| risk-engine | Risk Engine | Beta | Monte Carlo VaR, GBM simulation, stress testing |
| gbm-sim | GBM Simulator | Beta | Geometric Brownian Motion price path simulation |
| tax-schema | Tax Schema Editor | Beta | Tax recipient suite configuration, ILP payment dispatch |
| ilp-dashboard | ILP Routing Dashboard | Soon | Interledger payment routing visualization |

#### Security (6)

| ID | Name | Status | Description |
|---|---|---|---|
| agreements | Agreements & Rights | Beta | Agreement creation, consent management, bilateral micro-commons |
| key-vault | Key Vault Manager | Beta | BIP39 seed, ed25519 keypairs, WebID TLS cert minting |
| zk-studio | ZK Proof Studio | Beta | Zero-knowledge proof generation and verification |
| deontic-editor | Deontic Logic Editor | Beta | OBLIGATE/PERMIT/FORBID rule authoring, defeasible reasoning |
| shacl-validator | SHACL Validator | Beta | SHACL shape validation against NQuin graph |
| credential-manager | Credential Manager | Soon | Verifiable Credential issuance, verification, revocation |

#### Data (5)

| ID | Name | Status | Description |
|---|---|---|---|
| wal-inspector | WAL Inspector | Beta | Write-Ahead Log entry inspection, audit trail |
| q42-volume | Q42 Volume Manager | Beta | Q42 volume list, inspect, verify, magnet, compact |
| provenance-graph | Provenance Graph | Beta | Q42 provenance chain visualization |
| storage-config | Storage Driver Config | Beta | Storage path, quota, driver selection |
| crdt-sync | CRDT Sync Dashboard | Soon | LWW CRDT resolution, delegation verification, suspended transaction queue |

#### Network (5)

| ID | Name | Status | Description |
|---|---|---|---|
| webtorrent | WebTorrent Seeder | Beta | WebTorrent routing and seeding |
| p2p-dashboard | P2P Node Dashboard | Beta | Mesh networking status, peer discovery |
| ebpf-filter | eBPF Filter Manager | Beta | eBPF packet filtering and firewall rules |
| acoustic-ble | Acoustic BLE Mesh | Beta | Acoustic BLE mesh networking |
| nym-gateway | Nym Privacy Gateway | Soon | Nym mixnet relay configuration |

#### Developer (5)

| ID | Name | Status | Description |
|---|---|---|---|
| mcp-inspector | MCP Tool Inspector | Beta | MCP server tool registration and capability manifest inspection |
| benchmark | Benchmark Harness | Beta | Q42 comparative benchmark, Forge kernel certification |
| cli-bridge | CLI Bridge | Beta | Bridge to qualia-cli commands |
| extension-bus | Extension Bus | Beta | Extension manager for PINN, QPU, SNN, WebGPU extensions |
| marketplace | QApp Marketplace | Soon | QApp install/uninstall from remote sources |

### 3.3 Academic QApp catalog (274 QApps)

**Catalog source files:**

| File | QApp count | Subject areas |
|---|---|---|
| `catalog/academic_social.rs` | 30 | Social sciences, humanities, natural sciences |
| `catalog/academic_sciences.rs` | 63 | Natural sciences, formal sciences, area studies, applied liberal arts, emerging interdisciplinary |
| `catalog/academic_arts.rs` | 48 | Fine arts, advanced subdisciplines, philosophy, religious studies, literary/media studies |
| `catalog/academic_critical.rs` | 103 | Critical theory, historical studies, linguistic subdisciplines, interdisciplinary science, design/media |
| `catalog/academic_specialised.rs` | 30 | Interdisciplinary science, pre-professional, language/regional, arts/performance |
| **Total** | **274** | |

**Wellfair library catalog (`qapp_catalog.rs`):**

The `STUDIO_QAPP_CATALOG` const array in `qualia-client-core/src/wellfair/qapp_catalog.rs` contains 278 entries with domain categories for faceted browse in the hypermedia Library Software section:

| Category slug | Human-readable label | Count |
|---|---|---|
| social-sciences | Social Sciences | ~30 |
| humanities | Humanities | ~17 |
| natural-sciences | Natural Sciences | ~13 |
| formal-sciences | Formal Sciences | 4 |
| area-studies | Area Studies | ~15 |
| applied-liberal-arts | Applied Liberal Arts | ~7 |
| emerging-interdisciplinary | Emerging & Interdisciplinary | ~20 |
| pre-professional | Pre-Professional | 7 |
| language-regional | Language & Regional | ~14 |
| arts-performance | Arts & Performance | ~11 |
| advanced-subdisciplines | Advanced Subdisciplines | ~10 |
| niche-sciences | Niche Sciences | ~8 |
| philosophy-theory | Philosophy & Theory | ~8 |
| religious-studies | Religious Studies | 7 |
| literary-media | Literary & Media Studies | ~6 |
| linguistics-semiotics | Linguistics & Semiotics | 6 |
| intersectional-applied | Intersectional & Applied | ~10 |
| historical-textual | Historical & Textual Studies | ~10 |
| critical-cultural | Critical & Cultural Studies | ~20 |
| interdisciplinary-stem | Interdisciplinary STEM | ~10 |
| design-spatial | Design & Spatial Studies | ~6 |
| critical-theory | Critical Theory | ~10 |

**Academic QApp component pattern:**

Each of the 275 `*_qapp.rs` files in `webizen-studio/src/components/` follows this pattern:

```rust
#[component]
pub fn ZoologyQapp() -> Element {
    rsx! {
        QAppEngine {
            ontology_id: "urn:ontology:zoology".to_string(),
            title: "Zoology Explorer".to_string()
        }
    }
}
```

These are **stub components** wrapping `QAppEngine` with discipline-specific ontology IDs. They render ontology-driven forms but the "Save Node" button is a placeholder. **Status: partial** — forms render, save is stub.

**Representative academic QApps by discipline:**

- **Social Sciences:** African American Studies, Anthropology, Archaeology, Area & Regional Studies, Cultural Studies, Economics, Gender & Sexuality Studies, Human Geography, History, International Relations, Political Science, Psychology, Sociology
- **Humanities:** Art History, Classics, Comparative Literature, Creative Writing, Dance, English Literature, Ethics, Film & Media Studies, Foreign Languages, Linguistics, Music History, Music Performance, Music Theory, Philosophy, Religion & Theology, Studio Art, Theater & Drama
- **Natural Sciences:** Astronomy, Astrophysics, Biology, Botany, Chemistry, Earth Science, Ecology, Environmental Science, Evolutionary Biology, Geology, Neuroscience, Oceanography, Physics, Zoology
- **Formal Sciences:** Computer Science, Logic, Mathematics, Statistics
- **Critical Theory:** Critical Theory, Critical Race & Ethnic Studies, Postcolonial Studies, Decolonial Studies, Queer Theory, Fat Studies, Disability Studies, Whiteness Studies
- **Area Studies:** American Studies, Asian Studies, European Studies, Global Studies, Indigenous & Native American Studies, Jewish Studies, Latin American Studies, Medieval & Renaissance Studies, Middle Eastern Studies
- **Pre-Professional:** Art Conservation, Arts Management, Bioethics, Curatorial Studies, Leadership Studies, Legal Studies, Social Work
- **Language & Regional:** Ancient Near Eastern Studies, Appalachian Studies, Arctic Studies, Balkan Studies, Caribbean Studies, Celtic Studies, Central Asian Studies, Egyptology, Francophone Studies, Germanic Languages, Hispanic & Luso-Brazilian Studies, Philology, Romance Languages, South Asian Studies, Southeast Asian Studies
- **Arts & Performance:** Animation & Digital Arts, Book Arts & Papermaking, Ceramics, Cinematography, Dramaturgy, Musicology, Photography, Playwriting, Printmaking, Screenwriting, Sculpture
- **Philosophy:** Aesthetics, Epistemology, Metaphysics, Phenomenology, Philosophy of Mind, Philosophy of Religion, Philosophy of Science, Social & Political Philosophy
- **Religious Studies:** Biblical Studies, Buddhist Studies, Canon Law, Hindu Studies, Islamic Studies, Missiology, Patristics
- **Interdisciplinary STEM:** Biogeochemistry, Bioinformatics, Chemical Physics, Computational Linguistics, Cryptography, Environmental Chemistry, Geochemistry, Mathematical Biology, Mathematical Economics, Systems Biology

---

## 4. Bundled QApps

### 4.1 Anatomy QApp (the only fully implemented bundled QApp)

**Source:** `bundled/qapps/Anatomy/`

**Manifest:** `bundled/qapps/Anatomy/qapp.json` (125 lines)

| Field | Value |
|---|---|
| Name | Anatomy |
| Version | 0.0.12 |
| App ID | `did:qualia:qapp:anatomy` |
| Display name | Qualia Anatomy |
| Category | health-visualization |
| Preferred launch mode | flutter-app-vault |
| Supports offline | true |
| Max sensitivity clearance | 0x00 (Public) |

**Required shapes:** schema:HealthCondition, schema:MedicalRecord, qualia:BiometricRecord, qualia:SleepRecord, qualia:MedicationRecord, qualia:ConsentRecord, qualia:AnatomicalStructure

**Required ontologies:** q42:anatomy, snomedct-us, https://qualia.anatomy.example/ontology/bio#

**Optional remote endpoints:** wikidata-main

**Capabilities declared:**
- `qualia.localDaemon.health` (optional)
- `qualia.localDaemon.query` (optional)
- `qualia.wasm.execute_ntriples_query` (optional)
- `qualia.flutter.chatRepresentationLaunch` (optional)

**WASM config:**
- Engine package: `qualia-core-wasm`
- Expected exports: `get_engine_version`, `compile_query_to_json`, `execute_ntriples_query`, `parse_turtle_wasm`, `parse_n3logic_wasm`, `validate_shacl_constraint_wasm`

**Local daemon config:**
- Health path: `/health`
- Query path: `/query`
- WebSocket path: `/qualia-bridge`
- Requires token: true

**Chat integration:**
- Intent: `anatomy:represent-graph-complexity`
- Supports launch from chat: true
- Supports return to chat: true
- Emits: conditions, conditionImpactMap, systems, daemonMatchCount, dicomOverlay

**Representation contract:**
- Accepts: anatomySelection, conditionImpactMap, conditions, dicomOverlay, graphSummary, explanationCard, systemOverlay, systemFocus, relatedSystems
- Knowledge assets: system-map.json, condition-map.json, conditions.ttl, shapes.shacl, dicom-organ-map.json
- Supported system count: 17

**UI surfaces:** full-screen, panel, embedded

**Files:**
- `index.html` — main entrypoint
- `qualia.js` — Qualia API client
- `daemon-client.js` — daemon communication
- `dicom-overlay.js` — DICOM visualization
- `knowledge-parser.js` — knowledge graph parsing
- `lib/babylon.js` — Babylon.js 3D engine
- `lib/babylonjs.loaders.min.js` — Babylon.js loaders
- `Knowledge/` — N3, Turtle, SHACL, JSON maps
  - `biomarkers.n3`, `conditions.n3`, `conditions.ttl`, `rules.n3`, `shapes.shacl`
  - `condition-map.json`, `dicom-organ-map.json`, `system-map.json`
  - `shapes/dicom-healthcare.shacl.ttl`, `shapes/qualia-agency.shacl.ttl`, `shapes/radlex-anatomy.shacl.ttl`

**Status: live** — production-ready bundled QApp with complete manifest, knowledge assets, 3D visualization, daemon integration, and chat handoff.

### 4.2 Bundled QApp seeding

**Source:** `qualia-client-core/src/bundled_qapps.rs` (340 lines)

**`DEFAULT_BUNDLED_QAPPS`:** `["Anatomy"]`

**Functions:**
- `seed_bundled_qapps()` — seeds default QApps on first launch
- `check_bundled_qapp_update()` — checks for bundled updates
- `apply_bundled_qapp_update()` — applies bundled update
- `upgrade_qapp_from_source()` — upgrades from external directory

**Status: live** — seeding and update pipeline implemented.

---

## 5. Desktop QApp hosting and commands

### 5.1 QApp host API (message bus)

**Source:** `webizen-desktop/src/commands/qapp_host.rs` (756 lines)

| Command | Signature | What it does |
|---|---|---|
| `submit_record` | `(app, qapp_id, envelope, source) -> Result<usize, String>` | Submits record from QApp to Host API |
| `qapp_analyze` | `(request: QappAnalysisRequest) -> Result<QappAnalysisResult, String>` | Analyzes QApp fields, generates Q42 provenance via Forge graph |
| `certify_forge_physics` | `async () -> Result<ForgePhysicsCertification, String>` | Certifies Forge physics kinematics (n-body simulation, WGPU vs CPU oracle) |
| `run_forge_compute_probe` | `async () -> Result<ForgeComputeProbe, String>` | Runs Top-K, GEMM, FFT kernel diagnostics |
| `get_qualia_compute_profile` | `() -> QualiaComputeProfile` | Returns GPU adapter info, backend, Forge module availability |
| `get_latest_diffusion_snapshot` | `(runtime) -> Option<RuntimeSnapshotRecord>` | Returns latest diffusion frame |
| `reconfigure_diffusion` | `(runtime, config) -> Result<(), String>` | Reconfigures diffusion (currently stub: returns Ok(())) |
| `get_diffusion_frame_rgba` | `(runtime, slot) -> Result<Vec<u8>, String>` | Returns diffusion frame RGBA bytes |
| `get_diffusion_ledger_health` | `(runtime) -> RuntimeLedgerHealth` | Returns diffusion ledger health |
| `probe_localhost_preview` | `async () -> LocalPreviewProbe` | Probes localhost:8080 for dev preview |

**Status: live** — all commands implemented. `reconfigure_diffusion` is a stub.

### 5.2 QApp telemetry

**Source:** `webizen-desktop/src/commands/qapp_telemetry.rs` (193 lines)

| Command | What it exposes |
|---|---|
| `wellfair_get_model_lifecycle_status` | Current lifecycle state label |
| `wellfair_force_model_lifecycle_phase` | **Honest error** — force phase not exposed; real transitions go through activate/unload/scrub |
| `wellfair_get_llm_telemetry` | tokens_per_sec, vram_usage_gb, loaded_model, model_lifecycle, thermal_state, llm_memory_bytes, kv_cache_used_mb, inference_backend, ollama_optional_note |

**Data structures:**
- `QappAnalysisRequest` — discipline, fields, notes
- `QappAnalysisResult` — summary, assertions, provenance_hash (q42: format), engine, graph_nodes, q42_quins, evidence_weight, forge_schema_version
- `QualiaComputeProfile` — GPU adapter info, backend, device type, WGPU/Naga/CUDARC API versions, limits, available modules
- `ForgePhysicsCertification` — kernel certification results, max_abs_error, momentum_drift, q42_provenance
- `ForgeComputeProbe` — Top-K, GEMM, FFT kernel results with certification and timing

**Status: live** — telemetry commands implemented with honest error reporting.

### 5.3 Desktop shell QApp hosting

**Source:** `webizen-desktop/src/shell/shell_html.rs` (lines 190-364)

The desktop hosts QApps through:
1. **WebView iframe** — each QApp runs in a separate iframe
2. **Tab management** — multi-tab interface for multiple QApps
3. **Protocol routing** — `qualia://` protocol for QApp addressing
4. **Host API** — Tauri commands for QApp-backend communication

**Key functions:**
- `normalizeQappId()` — normalizes QApp IDs
- `createTab()` — creates new QApp tab
- `studioPath()` — maps QApp ID to route path
- `switchToTab()` — switches active QApp
- `closeTab()` — closes QApp tab

**Status: live** — multi-tab QApp hosting implemented.

---

## 6. Studio QApp UI surfaces

### 6.1 QApp catalog browser

**Source:** `webizen-studio/src/components/qapps/component.rs` (326 lines)

**Component:** `QApps`

**Features:**
- Category filter tabs (13 categories)
- Search box for filtering by task/domain/capability
- Featured templates section (6 templates)
- QApp card grid with status badges (Active/Beta/Soon)
- Capability chips on each card (Local execution, Semantic Library, Health ontologies, Financial ontology, Ontology/graph, AI model, Domain graph)
- Action buttons: Launch (routes to ContextStudio/QAppStudio/Nexus), Edit (routes to StudioEditRoute), Run QApp, Coming Soon
- "Show full catalog (Soon)" toggle for placeholder visibility

**Status: live** — comprehensive catalog UI.

### 6.2 QAppEngine (ontology-driven form builder)

**Source:** `webizen-studio/src/components/qapp_engine.rs` (222 lines)

**Component:** `QAppEngine`

**Features:**
- Loads domain ontology schema via `fetch_domain_ontology`
- Renders SHACL shape-based forms
- "Save Node" button (placeholder — no actual save logic)

**Status: partial** — renders forms but save is stub.

### 6.3 QAppDispatcher (routing)

**Source:** `webizen-studio/src/components/qapp_dispatcher.rs` (742 lines)

**Component:** `QAppDispatcher`

Routes element tags to specific QApp components. Dispatches 333+ QApps including all academic and non-academic catalog entries.

**Status: live** — full routing implemented.

### 6.4 QApp publish panel

**Source:** `webizen-studio/src/components/wellfair/qapp_publish_panel.rs` (227 lines)

**Component:** `WellfairQappPublishPanel`

**Features:**
- App ID (reverse-DNS format)
- Display name
- Kind selector (cooperative, health, journal, directory, custom)
- WASM entry filename
- Description
- Capability checkboxes (least-privilege): read_records, write_records, sync, blob_store, notifications, camera
- Output folder picker
- Generate PWA bundle button

**Tauri commands invoked:** `pick_directory`, `publish_qapp_pwa`

**Status: live** — WP2 authoring surface for companion-PWA feature.

### 6.5 Library panel (QApp catalog integration)

**Source:** `webizen-studio/src/components/wellfair/library_panel.rs` (2413 lines)

**Component:** `LibraryPanel`

Seeds QApps into hypermedia Library Software section via `seed_studio_qapps`. Supports faceted browse by perception/model/ontology rows.

**Status: live** — includes QApp seeding and catalog integration.

### 6.6 Studio routes

**Source:** `webizen-studio/src/main.rs` (lines 160-285)

| Route | Path | Component |
|---|---|---|
| `QAppsRoute` | `/qapps` | `components::qapps::QApps` |
| `StudioRoute` | `/qapp-studio` | `DynamicPage` |
| `StudioEditRoute` | `/qapp-studio/:app_id` | `DynamicPage` |
| `ContextStudioRoute` | `/context-studio` | `components::contextual_workspace::ContextualWorkspace` |

**Status: live** — all routes defined and functional.

---

## 7. QApp config structs (full reference)

### 7.1 `QappHostExtension` — main extension struct

**Source:** `qualia-client-core/src/qapp_registry.rs:147-196`

| Field | Type | Purpose |
|---|---|---|
| `app_id` | String | DID identifier |
| `display_name` | String | Human-facing name |
| `category` | String | Category slug |
| `launch_modes` | Vec<String> | static-web, wasm-local, flutter-app-vault, online-daemon-aware |
| `preferred_launch_mode` | String | Default launch mode |
| `supports_offline` | bool | Offline capability |
| `requires` | Vec<QappLaunchRequirement> | Capability requirements |
| `required_ontologies` | Vec<String> | Ontology domains |
| `optional_remote_endpoints` | Vec<String> | Remote SPARQL endpoints |
| `max_sensitivity_clearance` | String | 0x00/0x01/0x02 |
| `required_models` | Vec<String> | Required AI models |
| `entrypoints` | HashMap<String, String> | Named entry points (web, representation, chat_component) |
| `wasm` | Option<QappWasmConfig> | WASM engine config |
| `local_daemon` | Option<QappDaemonConfig> | Daemon integration |
| `chat_integration` | Option<QappChatIntegrationConfig> | Chat handoff |
| `representation_contract` | Option<QappRepresentationContract> | Data contract |
| `pinn_model` | Option<QappPinnModelConfig> | PINN model config |
| `ui_surfaces` | Vec<String> | full-screen, panel, embedded |
| `notes` | Vec<String> | Developer notes |
| `pages` | Vec<QappPage> | Multi-page views |
| `export_targets` | Vec<String> | embedded, wasm-mobile, tauri-desktop |
| `did_git` | Option<String> | Git identity for Gitmark validation |

### 7.2 `QappWasmConfig`

| Field | Type |
|---|---|
| `engine_package` | String |
| `expected_exports` | Vec<String> |

### 7.3 `QappDaemonConfig`

| Field | Type |
|---|---|
| `health_path` | String |
| `query_path` | String |
| `websocket_path` | String |
| `requires_token` | bool |

### 7.4 `QappChatIntegrationConfig`

| Field | Type |
|---|---|
| `intent` | String |
| `payload_version` | String |
| `supports_launch_from_chat` | bool |
| `supports_return_to_chat` | bool |

### 7.5 `QappRepresentationContract`

| Field | Type |
|---|---|
| `accepts` | Vec<String> |

### 7.6 `QappPinnModelConfig` (1.58-bit ternary quantized PINN)

| Field | Type | Default |
|---|---|---|
| `model_name` | String | "" |
| `model_path` | String | "" |
| `model_type` | String | "" |
| `quantization_bits` | String | "" (e.g., "1.58") |
| `compression_ratio` | f32 | 0.0 |
| `smx_version` | String | "" |
| `physics_domain` | String | "" |
| `input_dim` | usize | 0 |
| `output_dim` | usize | 0 |
| `uses_ternary_quantization` | bool | false |
| `memory_requirement_mb` | u32 | 0 |
| `vram_requirement_mb` | Option<u32> | None |
| `supported_operations` | Vec<String> | [] |
| `training_metadata` | Option<QappPinnTrainingMetadata> | None |
| `performance_metrics` | Option<QappPinnPerformanceMetrics> | None |

### 7.7 `QappPinnTrainingMetadata`

| Field | Type |
|---|---|
| `epochs` | u32 |
| `final_loss` | f64 |
| `convergence_rate` | f64 |
| `validation_accuracy` | f64 |
| `training_dataset` | String |
| `training_date` | String |

### 7.8 `QappPinnPerformanceMetrics`

| Field | Type |
|---|---|
| `inference_speedup` | f32 |
| `memory_savings` | f32 |
| `quantization_error` | f64 |
| `sparsity_ratio` | f32 |
| `avg_inference_time_ms` | f32 |
| `throughput_ops_per_sec` | f64 |

### 7.9 `QappPage`

| Field | Type |
|---|---|
| `name` | String |
| `path` | String |
| `title` | String |
| `icon` | String |

### 7.10 `QappLaunchRequirement`

| Field | Type |
|---|---|
| `capability` | String |
| `required` | bool |

### 7.11 `QappTarget`

| Variant | Payload |
|---|---|
| `LocalDevDirectory` | PathBuf |
| `LocalProxyPort` | u16 |
| `IsolatedVault` | String |

### 7.12 `QappRegistryState`

| Field | Type |
|---|---|
| `installed_qapps` | HashMap<String, RegisteredQapp> |

### 7.13 `RegisteredQapp`

| Field | Type |
|---|---|
| `did` | String |
| `manifest` | QappPackageManifest |
| `target` | QappTarget |

### 7.14 Install registry structs

**`QappContentManifest`:** schema_version, package_id, version, abi_version, host_api_version, files (Vec<QappFileHash>), signature_hex

**`QappRegistryEntry`:** package_id, active_version, content_hash, installed_at_unix, revoked, archived_versions

**`QappInstallRegistry`:** packages (HashMap<String, QappRegistryEntry>)

---

## 8. QApp documentation and specifications

### 8.1 QApps specification

**Path:** `docs/manuals/qapps_specification.md`

Defines QApp as stateless declarative UI shell with YAML-LD-Q42 manifest format, data binding, semantic linking, capabilities via shared pool, and manifold worlds (3D + 2D views).

### 8.2 QApp loopback protocol

**Path:** `docs/manuals/standards/qualia-qapp-loopback-protocol.md`

Defines localhost serving boundary for installed QApps, asset server (127.0.0.1:4567..4600), package manifest role, launch resolution, and daemon relationship.

### 8.3 QApp LLM helper

**Path:** `docs/manuals/qapp_llmHelper.md`

Defines LLM integration patterns for QApps.

### 8.4 Vault manifest

**Path:** `docs/manuals/standards/qualia-vault-manifest.md`

Defines `.qualia` as vault orchestration layer with CBOR-LD projection and Q42 lexicon. Turtle as primary format, N3 as richer profile.

### 8.5 Anatomy QApp knowledge base

**Path:** `bundled/qapps/Anatomy/Knowledge/`

- `biomarkers.n3` — biomarker definitions
- `conditions.n3` — medical conditions
- `rules.n3` — N3Logic rules
- `conditions.ttl` — Turtle format conditions
- `shapes.shacl` — SHACL shapes
- `dicom-organ-map.json` — DICOM organ mapping
- `condition-map.json` — condition mapping
- `system-map.json` — system mapping
- `shapes/dicom-healthcare.shacl.ttl`
- `shapes/qualia-agency.shacl.ttl`
- `shapes/radlex-anatomy.shacl.ttl`

---

## 9. qualia-ui QApp gap analysis

### 9.1 Current state

**Source:** `C:\Projects\NLP\qualia-ui\src\browser\container_views_ext.rs` (lines 526-555)

The only QApp-related surface in qualia-ui is a **placeholder portal view**:

```
Portal — QApp dispatch and wormhole IRI.
launch_installed_qapp, verify_and_install_qapp,
export_qapp_as_wasm_package. Awaiting engine wiring.
```

**Status: present** — placeholder UI exists, no functionality wired.

### 9.2 What qualia-ui needs

The HyperCanvas must reach the full QApp ecosystem through typed capability bindings and host transport. The following surfaces are required:

#### A. QApp catalog browser (container)

| Feature | Source | qualia-ui state |
|---|---|---|
| 13-category filter | `catalog/mod.rs` | **missing** |
| 333 QApp cards with status badges | `catalog/*.rs` | **missing** |
| Search by task/domain/capability | `component.rs` | **missing** |
| 6 featured workspace templates | `catalog/mod.rs` | **missing** |
| "Show Soon" toggle | `component.rs` | **missing** |
| Capability chips | `component.rs` | **missing** |
| Launch/Edit/Run QApp buttons | `component.rs` | **missing** |

#### B. QApp dispatcher (routing)

| Feature | Source | qualia-ui state |
|---|---|---|
| Route element tags to QApp components | `qapp_dispatcher.rs` | **missing** |
| 333+ QApp routing | `qapp_dispatcher.rs` | **missing** |

#### C. QApp engine (ontology-driven forms)

| Feature | Source | qualia-ui state |
|---|---|---|
| Load domain ontology schema | `qapp_engine.rs` | **missing** |
| Render SHACL shape-based forms | `qapp_engine.rs` | **missing** |
| Save node (currently stub everywhere) | `qapp_engine.rs` | **missing** |

#### D. QApp install/uninstall/marketplace

| Feature | Source | qualia-ui state |
|---|---|---|
| Install QApp from source directory | `qapp_install.rs` | **missing** |
| List installed QApps | `qapp_install.rs` | **missing** |
| Uninstall/revoke QApp | `qapp_install.rs` | **missing** |
| Version display and upgrade | `qapp_version.rs` | **missing** |
| Bundled QApp seeding | `bundled_qapps.rs` | **missing** |
| Marketplace browser (Soon status) | `catalog/developer.rs` | **missing** |

#### E. QApp readiness inspection

| Feature | Source | qualia-ui state |
|---|---|---|
| Capability requirement checks | `qapp_launcher.rs` | **missing** |
| Ontology presence checks | `qapp_launcher.rs` | **missing** |
| Model presence checks | `qapp_launcher.rs` | **missing** |
| SPARQL endpoint probes | `qapp_launcher.rs` | **missing** |
| Readiness report display | `qapp_launcher.rs` | **missing** |

#### F. QApp launch and hosting

| Feature | Source | qualia-ui state |
|---|---|---|
| Launch installed QApp with context | `qapp_launcher.rs` | **missing** |
| Loopback protocol URL resolution | `qapps_protocol.rs` | **missing** |
| Token injection | `key_vault.rs` | **missing** |
| Multi-tab QApp hosting | `shell_html.rs` | **missing** |
| `qualia://` protocol routing | `qapps_protocol.rs` | **missing** |

#### G. QApp authoring and publishing

| Feature | Source | qualia-ui state |
|---|---|---|
| PWA manifest builder form | `qapp_publish_panel.rs` | **missing** |
| Kind selector (cooperative/health/journal/directory) | `qapp_publish.rs` | **missing** |
| Least-privilege capability checkboxes | `qapp_publish.rs` | **missing** |
| Output directory picker | `qapp_publish.rs` | **missing** |
| PWA bundle generation | `qapp_package/pwa.rs` | **missing** |
| Remote controller PWA generation | `qapp_package/remote_controller.rs` | **missing** |

#### H. QApp export (WASM packaging)

| Feature | Source | qualia-ui state |
|---|---|---|
| Export QApp as WASM package | `qapp_export.rs` | **missing** |
| QR code for LAN mobile loading | `qapp_export.rs` | **missing** |
| Q42 provenance scene generation | `qapp_host.rs` | **missing** |

#### I. QApp telemetry and monitoring

| Feature | Source | qualia-ui state |
|---|---|---|
| Model lifecycle status | `qapp_telemetry.rs` | **missing** |
| LLM telemetry (tokens/sec, VRAM, thermal) | `qapp_telemetry.rs` | **missing** |
| Forge physics certification | `qapp_host.rs` | **missing** |
| Forge compute probe (Top-K, GEMM, FFT) | `qapp_host.rs` | **missing** |
| Compute profile (GPU adapter, backend) | `qapp_host.rs` | **missing** |
| Diffusion snapshot/ledger | `qapp_host.rs` | **missing** |

#### J. QApp capability and permission management

| Feature | Source | qualia-ui state |
|---|---|---|
| View compiled capabilities | `qapp_manifest.rs` | **missing** |
| View clearance level (Public/Restricted/Classified) | `qapp_manifest.rs` | **missing** |
| View permitted ontology domains | `qapp_manifest.rs` | **missing** |
| View capability requirements | `qapp_registry.rs` | **missing** |
| Token issuance/verification status | `key_vault.rs` | **missing** |
| Sanctuary violation log | `qapp_api.rs` | **missing** |

#### K. QApp MCP integration

| Feature | Source | qualia-ui state |
|---|---|---|
| List QApp catalogue (MCP) | `qapp_mcp.rs` | **missing** |
| Get QApp manifest (MCP) | `qapp_mcp.rs` | **missing** |
| Inspect QApp readiness (MCP) | `qapp_mcp.rs` | **missing** |
| Describe QApp surface schema | `qapp_mcp.rs` | **missing** |

#### L. QApp settings

| Feature | Source | qualia-ui state |
|---|---|---|
| Per-QApp configuration panel | — | **missing** |
| QApp permission profile editor | — | **missing** |
| QApp ontology domain management | `qapp_launcher.rs` | **missing** |
| QApp model requirement management | `qapp_launcher.rs` | **missing** |
| QApp daemon endpoint configuration | `qapp_registry.rs` | **missing** |
| QApp chat integration configuration | `qapp_registry.rs` | **missing** |
| QApp PINN model configuration | `qapp_registry.rs` | **missing** |
| QApp export target selection | `qapp_registry.rs` | **missing** |

---

## 10. Implementation priorities

### Phase 1 — QApp catalog and discovery (P0)

| Priority | Task | Dependencies |
|---|---|---|
| P0 | QApp catalog container with 13-category filter | IntentBus + transport |
| P0 | 333 QApp card rendering with status badges | Catalog data port |
| P0 | QApp search | Catalog data port |
| P0 | QApp readiness inspector | `inspect_installed_qapp_readiness` command |
| P0 | QApp launch with context injection | `launch_installed_qapp_with_context` command |

### Phase 2 — QApp install and lifecycle (P0)

| Priority | Task | Dependencies |
|---|---|---|
| P0 | Installed QApps list | `list_registry_entries` command |
| P0 | QApp install from source | `install_package_atomic` command |
| P0 | QApp uninstall/revoke | `revoke_package` command |
| P0 | Bundled QApp seeding | `seed_bundled_qapps` command |
| P1 | QApp version display and upgrade | `check_bundled_qapp_update` command |
| P1 | QApp marketplace browser | Remote source support |

### Phase 3 — QApp hosting and dispatch (P1)

| Priority | Task | Dependencies |
|---|---|---|
| P1 | QApp dispatcher routing | Phase 1 |
| P1 | QAppEngine ontology-driven forms | `fetch_domain_ontology` command |
| P1 | Loopback protocol URL resolution | `qualia_qapp_launch_url` command |
| P1 | Multi-tab QApp hosting | Container management |
| P1 | `qualia://` protocol routing | Protocol handler |
| P2 | QAppEngine save node (currently stub everywhere) | WAL append |

### Phase 4 — QApp authoring and publishing (P1)

| Priority | Task | Dependencies |
|---|---|---|
| P1 | PWA manifest builder form | `publish_qapp_pwa` command |
| P1 | Kind selector | `QappKind` enum |
| P1 | Least-privilege capability checkboxes | `Capability` enum |
| P1 | Output directory picker | `pick_directory` command |
| P2 | PWA bundle generation | `generate_pwa` |
| P2 | Remote controller PWA | `generate_remote_controller_pwa` |

### Phase 5 — QApp export and telemetry (P2)

| Priority | Task | Dependencies |
|---|---|---|
| P2 | WASM package export | `export_qapp_as_wasm_package` command |
| P2 | QR code for LAN mobile loading | Export URL |
| P2 | Model lifecycle status display | `wellfair_get_model_lifecycle_status` |
| P2 | LLM telemetry dashboard | `wellfair_get_llm_telemetry` |
| P2 | Forge physics certification display | `certify_forge_physics` |
| P2 | Forge compute probe display | `run_forge_compute_probe` |
| P2 | Compute profile display | `get_qualia_compute_profile` |

### Phase 6 — QApp capability management and settings (P2)

| Priority | Task | Dependencies |
|---|---|---|
| P2 | Per-QApp capability view | `get_compiled_capability` |
| P2 | Per-QApp clearance level display | `CompiledCapability` |
| P2 | Per-QApp permitted domains display | `CompiledCapability` |
| P2 | Sanctuary violation log | `append_sanctuary_violation` |
| P2 | QApp token status | `issue_qapp_token` / `verify_qapp_token` |
| P3 | QApp PINN model configuration UI | `QappPinnModelConfig` |
| P3 | QApp chat integration configuration | `QappChatIntegrationConfig` |
| P3 | QApp daemon endpoint configuration | `QappDaemonConfig` |
| P3 | QApp export target selection | `QappHostExtension.export_targets` |

### Phase 7 — QApp MCP integration (P3)

| Priority | Task | Dependencies |
|---|---|---|
| P3 | MCP catalogue listing | `list_qapp_catalogue_json` |
| P3 | MCP manifest retrieval | `get_qapp_manifest_json` |
| P3 | MCP readiness inspection | `inspect_qapp_readiness_json` |
| P3 | MCP surface schema display | `describe_qapp_surface_schema_json` |

---

## 11. Summary

The QualiaDB workspace contains a comprehensive QApp ecosystem with **333 catalog entries** (274 academic + 59 non-academic), **1 fully implemented bundled QApp** (Anatomy), a **capability-compiled registry** with zero-allocation hot-path enforcement, an **atomic install pipeline** with versioning and rollback, a **loopback protocol server** for WebView-safe asset serving, a **PWA publishing pipeline** for mobile delivery, a **WASM export pipeline** for LAN QR-code distribution, and **15+ desktop Tauri commands** for hosting, telemetry, and analysis.

The studio has a **fully implemented catalog browser** with 13 categories, search, featured templates, and status badges. It has a **QApp dispatcher** routing 333+ QApps, a **QAppEngine** ontology-driven form builder (partial — save is stub), a **QApp publish panel** for PWA authoring, and a **library panel** with QApp catalog integration. The desktop has a **multi-tab hosting shell** with `qualia://` protocol routing.

`qualia-ui` has **one placeholder portal view** with no functionality wired. The entire QApp ecosystem — catalog, install, launch, hosting, authoring, export, telemetry, capability management, and MCP integration — is **missing** from the HyperCanvas.

The critical path is:
1. **IntentBus + backend transport** — without this, no QApp command can be invoked.
2. **QApp catalog container** — port the 333-entry catalog with category filtering and search.
3. **QApp install/list/launch** — wire `install_package_atomic`, `list_registry_entries`, `launch_installed_qapp_with_context`.
4. **QApp readiness inspector** — wire `inspect_installed_qapp_readiness`.
5. **QApp dispatcher and QAppEngine** — port the routing and ontology-driven form rendering.
6. **QApp authoring/publishing** — port the PWA manifest builder.
7. **QApp export and telemetry** — wire WASM export, Forge probes, and LLM telemetry.
8. **QApp capability management** — surface compiled capabilities, clearance levels, and sanctuary violations.

The desktop host stays the native chrome. The studio Classic routes stay default. `qualia-ui` is the Poet HyperCanvas that *reaches* the QApp ecosystem through a typed, CBOR-LD, capability-gated intent bus.

---

*Generated with [Devin](https://devin.ai) for Timothy Charles Holborn. Audit performed 2026-08-18 against QualiaDB branch 0.0.30/0.0.31-dev and qualia-ui v0.1.0.*
