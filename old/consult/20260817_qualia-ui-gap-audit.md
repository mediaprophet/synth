# Qualia-UI gap audit — what the Poet HyperCanvas is missing versus webizen-desktop and webizen-studio

**Audit date:** 2026-08-17
**Auditor:** Devin (GLM-5.2 High)
**Principal / copyright holder:** Timothy Charles Holborn <timothy.holborn@gmail.com>
**Snapshots audited:**
- `C:\Projects\qualia-27062026` — QualiaDB workspace, branch `0.0.30`/`0.0.31-dev`, crates `webizen-desktop` v0.0.30 and `webizen-studio` v0.0.30.
- `C:\Projects\NLP\qualia-ui` — `qualia-ui` v0.1.0 (Trunk + `wasm32-unknown-unknown`, direct DOM via `web-sys`).
- Reference plan: `docs/plans/poet-hypercanvas-tools-and-containers-2026-08-16.md` (the Poet HyperCanvas container/tool inventory).

**Recommendation status:** The `qualia-ui` crate is a clean, well-typed early-stage shell. It is missing almost everything that makes the desktop and studio crates useful as product surfaces. This report names every missing capability, groups it, and ties it back to the HyperCanvas plan so work can be sequenced without spoon-feeding.

---

## 1. Scope and method

Three crates were audited in full:

| Crate | Path | LOC / file count | Role |
|---|---|---|---|
| `webizen-desktop` | `crates/webizen-desktop` | ~70 source files, 200+ Tauri commands | Tauri native host: windowing, browser shell, daemon, render, wallet, wellfair, vision/audio, MCP server, settings server, companion gateway, updater |
| `webizen-studio` | `crates/webizen-studio` | ~450 source files, 40+ routes | Dioxus UI: canvas editor, theme engine, render subsystem, wellfair life-domain shell, 300+ academic QApps, specialized tools (clinical, scientific, quantum, knowledge, governance, AI/ML) |
| `qualia-ui` | `C:\Projects\NLP\qualia-ui` | ~30 source files, 36 N3 ontologies | Trunk/WASM browser UI: tool-chest core types, 6 manifold seeds, payload types, glassmorphism shell. **No backend integration.** |

The audit cross-referenced every public command, route, component, and capability domain in the desktop and studio crates against what `qualia-ui` currently implements or stubs. The HyperCanvas plan was used as the authoritative product direction: Classic studio routes become Poet containers/tools, the desktop host stays the native chrome, and `qualia-ui` is the WASM HyperCanvas surface that reaches the same engines through a typed capability-binding layer.

---

## 2. Executive assessment

`qualia-ui` has the right **skeleton** — typed payloads, a CBOR-LD intent bus trait, manifold seeds, a glassmorphism design system, and 36 authored N3 ontologies — but it has none of the **muscle**: no `IntentBus` implementation, no ontology loader, no toolbox registration, no backend transport, no real container bodies, no tool families, and no integration with any QualiaDB crate. Every container that should reach an engine currently renders placeholder text. The desktop and studio crates together expose roughly **200+ Tauri commands** and **40+ routed surfaces** spanning wellfair, social, wallet, vision/audio, render, SPARQL/SHACL, deontic logic, clinical risk, quantum, mesh, mail, QApp vault, and 300+ academic QApps. None of that is reachable from `qualia-ui` today.

The HyperCanvas plan (`docs/plans/poet-hypercanvas-tools-and-containers-2026-08-16.md`) defines exactly which containers and tools should subsume which Classic surfaces. This report maps the gaps to that plan so implementation can proceed in the documented order (§6 of the plan) without re-deriving the inventory.

### What should be retained in qualia-ui

- The **Trunk + `wasm-bindgen` + direct DOM** stack. It is the right choice for a browser-first HyperCanvas that must also run inside the desktop webview. Do not migrate to Dioxus just to match studio — the plan pins studio on `0.8.0-alpha.1` and Poet is a separate surface.
- The **tool-chest type system** (`IntentBus`, `VibeScriptPayload`, `Manifest`, `Registry`, `ManifoldSeed`, payload types). These are the right abstractions; they just need implementations and wiring.
- The **36 N3 ontology files** as the authoring source of truth. They need a compile-to-CBOR-LD pipeline, not runtime N3 parsing in WASM.
- The **glassmorphism design system** and the 6 manifold seeds as the starting layout vocabulary.

### What must change before qualia-ui can be called a product surface

| Priority | Blocking gap | Required artifact |
|---|---|---|
| P0 | No `IntentBus` implementation — payloads cannot be dispatched | A concrete `IntentBus` struct with a WASM transport (fetch/WebSocket to the desktop daemon or a public-web relay), capability gating, provenance stamping, and receipt handling. |
| P0 | No backend transport — `qualia-ui` cannot reach any QualiaDB engine | A typed client layer that maps `VibeScriptPayload` → daemon HTTP/WS endpoints (the same endpoints `webizen-studio::endpoints` already uses) or a public-web fallback. |
| P0 | No ontology loading — `OntologyRegistry` is always empty | A build-time N3 → CBOR-LD compiler step (Trunk hook or `build.rs`) plus a WASM `OntologyLoader` that reads pre-compiled CBOR-LD bytes embedded via `include_bytes!`. |
| P0 | No toolbox/tool registration at startup — `Registry::new()` is empty | Startup initialization that registers all 9 toolboxes, their tool-chains, and the 6 manifold seeds into a shared `Registry`. |
| P0 | 6 of the ~25 planned containers are placeholders; the other 19 do not exist yet | Implement the container bodies per the HyperCanvas plan §2, each backed by a `poet/tools/<family>/` module. |
| P0 | No tool families — `poet/tools/` does not exist | Create the 9 tool family folders from the plan §4 (`rich_text`, `sheet`, `latex`, `vibe`, `aura`, `gis`, `graph`, `triad`, `pulse`) with `ToolbarSpec`-driven toolbars. |
| P1 | No canvas editor semantics — only visual pan/zoom/drag | Undo/redo history, grid snapping, resize handles, manifold persistence (mirror `webizen-studio::canvas_editor`), and `ManifoldSeed` serialization. |
| P1 | No theme engine — single hardcoded CSS string | Port the QPrime theme system from `webizen-studio::theme_engine` (4 canonical presets, sanctuary zero-motion, WCAG contrast, Shoelace token bridging). |
| P1 | No render subsystem — canvas is 2D DOM only | A WASM `Canvas2D` renderer (studio already has one) plus a bridge to the native GPU viewport when running inside the desktop webview. |
| P1 | No capability detection — no notion of native-host vs public-web | Port `webizen-studio::endpoints` capability gating so containers can fail closed when an engine is unreachable. |
| P0 | No hypermedia library surface — the Lived Memory / HypermediaStore is entirely absent | A `library` (or `lived-memory`) container + `doc` container body reaching the 27 desktop library Tauri commands and the `qualia-semantic-library` crate (ingest, browse, faceted search, timeline/map, CML enrichment, commons, perception catalogue). See §3.8b. |
| P2 | No QApp studio surface | A `code` + `portal` container pair that can dispatch, verify, and launch QApps (mirror desktop `qapp_host`/`qapp_export` commands). |
| P2 | No wellfair life-domain shell | `health`, `anatomy`, `rights`, `wallet` containers reaching the 60+ wellfair Tauri commands. |
| P2 | No specialized scientific/clinical/quantum tools | `sheet`/`latex`/`map`/`health`/`finance` containers reaching the native_bindings, clinical, and quantum commands. |

---

## 3. webizen-desktop — capabilities the HyperCanvas must be able to reach

The desktop crate is the native host. `qualia-ui` does not need to re-implement these — it needs to **invoke** them through the intent bus. The inventory below is what the bus must be able to route to.

### 3.1 Core desktop / system (commands/system.rs, supervisor.rs, desktop_log.rs)

- `get_desktop_status`, `get_desktop_logs`, `set_desktop_debug_mode`, `report_client_error`
- `get_supervisor_state` — service and operation lifecycle snapshots
- `get_hardware_status`, `profile_energy_circumstance`
- `start_daemon`, `daemon_status`, `get_active_daemon_port`, `qualia_protocol_port`
- `run_engine_command`
- `get_config`, `save_config`
- Setup flow: `get_setup_state`, `complete_setup_step`, `update_setup_profile`, `finish_setup`
- `get_identity_plane`, `list_apparatus_devices`, `register_remote_apparatus_device`
- `resolve_job_device_placement`, `schedule_job_on_device`, `list_remote_job_outbox`, `retry_remote_job_outbox`
- `mint_person_webid_tls_cert`, `accept_fleet_job_envelope`
- `export_person_public`, `export_person_transfer_bundle`, `import_person_transfer_bundle`

**Gap in qualia-ui:** No status bar wired to real daemon state; no setup flow; no hardware/energy surface; no identity plane. The bottom status bar currently shows static `graph:manifold:<id>` text.

### 3.2 Browser shell (browser/, commands/browser.rs)

- Multi-webview chrome + content architecture
- Navigation: `open_web_url`, `browser_navigate`, `browser_navigate_content`, `browser_content_url`, `browser_focus`, `browser_reload`, `browser_go_back`, `browser_go_forward`, `browser_status`
- Trust store: `browser_trust_list`, `browser_trust_add_pem`, `browser_trust_add_did`, `browser_trust_set_enabled`, `browser_trust_remove`, `browser_trust_verdict`, `browser_trust_pin_host`, `browser_trust_list_suggested`, `browser_trust_import_suggested`
- Certificate override (Windows WebView2): `browser_cert_override_status`, `browser_cert_override_set_enabled`, `browser_cert_override_attach`
- Cookies: `browser_cookie_summary`, `browser_cookie_observe`, `browser_clear_site_data`, `browser_cookies_refresh`
- Agent: `browser_agent_ask`, `browser_agent_tls_status`
- Engine preference: `browser_engine_status`, `browser_set_engine` (OS WebView vs experimental Servo)
- QLinks: `list_qlinks`

**Gap in qualia-ui:** The `webview` container is listed in the plan as `present` but does not exist in qualia-ui. No trust store UI, no cookie observation, no cert override, no agent ask. The browser is the `World` life-domain in the plan.

### 3.3 QApp vault (commands/qapp_host.rs, qapp_export.rs, qapp_telemetry.rs)

- `list_installed_qapps`, `generate_qapp_credential`, `verify_and_install_qapp`, `launch_installed_qapp`
- `qapp_analyze`, `certify_forge_physics`, `run_forge_compute_probe`, `get_qualia_compute_profile`, `submit_record`
- `get_latest_diffusion_snapshot`, `reconfigure_diffusion`, `get_diffusion_frame_rgba`, `get_diffusion_ledger_health`
- `export_qapp_as_wasm_package`
- `wellfair_get_model_lifecycle_status`, `wellfair_force_model_lifecycle_phase`, `wellfair_get_llm_telemetry`

**Gap in qualia-ui:** No QApp surface at all. The plan maps `/qapp-studio` to `code` + `portal` containers. Neither exists.

### 3.4 Semantic logic (commands/semantic_logic.rs, semantic.rs)

- `execute_sparql_query`, `fetch_domain_ontology`, `validate_shacl_shape`, `evaluate_logic_rules`
- `generate_front_door_invite`, `mint_semantic_token`
- `fetch_wallet_portfolio`, `toggle_nym_relay`, `toggle_stark_prover`
- `update_solar_input`, `fetch_torrent_telemetry`, `fetch_remote_manifest`

**Gap in qualia-ui:** The `graph` and `ontology` containers are in the plan but not implemented. No SPARQL explorer, no SHACL validator, no deontic/epistemic/LTL/ASP logic surfaces. These are the `Nexus / SPARQL / SHACL panes` and `Logic modalities` rows in plan §5.

### 3.5 Social & communication (commands/social.rs, mail.rs)

- Social connect: `generate_connect_invite`, `accept_connect_invite`, `list_chat_contacts`, `get_user_profile`, `save_user_profile`
- Chat: `list_chat_sessions`, `load_chat_session`, `create_chat_session`, `ensure_chat_session`, `create_group_chat_session`, `add_chat_participant`, `remove_chat_participant`, `get_chat_participants`, `append_chat_message`, `get_chat_graph`, `stream_chat_inference`, `cancel_chat_inference`
- Agent roster: `agent_roster_list`, `agent_roster_get`, `agent_roster_upsert`, `agent_roster_remove`, `agent_runtime_status`, `agent_roster_add_remote`, `agent_remote_connection_test`
- Provider credentials: `provider_credential_store`, `provider_credential_remove`
- MCP tools: `mcp_list_local_tools`, `mcp_call_tool_gated`, `agent_set_allowed_mcp_tools`, `mcp_ensure_safe_tool_allowlist`
- Jobs: `ingest_chat_cml`, `schedule_agent_job`, `list_local_jobs`, `cancel_local_job`, `retry_local_job`, `clear_finished_local_jobs`, `schedule_model_download`, `schedule_model_activation`, `schedule_anatomy_asset_acquire`
- Mail: `list_mail_domains`, `add_mail_domain`, `purpose_inbox_presets`, `list_mail_addresses`, `mint_purpose_inbox`, `onboard_mail_domain`, `talk_setup_status`, `resolve_mail_delivery`, `save_mail_transport_config`, `load_mail_transport_config`, `mint_relationship_address`, `set_mail_address_enabled`, `front_door_forms`, `cf_verify_token`, `cf_list_zones`, `cf_publish_front_door`, `cf_deploy_infrastructure`, `deploy_static_site_cf_pages`, `start_qdp_server`, `parse_magic_link`, `mail_send`, `mail_fetch`, `mail_accept`, `mail_list`, `mail_get`, `mail_set_read`, `mail_delete`, `mail_dns_forms`, `mail_receiver_status`, `mail_receiver_start`, `mail_receiver_stop`
- Connections: `generate_connection_identifier`, `generate_magic_link`, `accept_connection`, `list_social_peers`, `set_social_peer_active`, `answer_connection_challenge`, `mesh_dialability`
- Cooperative projects: `list_project_collaborators`, `list_coop_projects`, `create_coop_project`, `add_project_collaborator`, `remove_project_collaborator`, `coop_share_package`, `accept_coop_share_package`, `create_project_group_chat`, `set_social_peer_endpoint`
- Agreements: `list_agreements`, `agreements_for`, `create_agreement`, `save_agreement`, `set_agreement_consent`
- Advanced: `get_qpu_settings`, `save_qpu_settings`, `enable_qpu_feature`, `disable_qpu_feature`, `activate_advanced_capabilities`, `get_advanced_activation_status`, `get_commitment_prompt`, `submit_omibox_query`
- Solid/Semantic handshake: `resolve_qdp_did`, `get_ns_records_for_did`, `sync_to_solid_pod`, `fetch_from_solid_pod`, `put_to_solid_pod`, `evaluate_data_request`, `apply_semantic_handshake`, `save_qlink`, `compute_context_hash`, `run_computational_geometry`

**Gap in qualia-ui:** The `social` container exists with chat graph rendering and connection-request cards, but it is **static** — no LWW CRDT, no chat session load/append, no agent roster, no MCP tool gating, no mail, no cooperative projects, no agreements, no Solid bridge. The `pulse` container (plan §2) does not exist. The `webrtc` container (plan §2) does not exist. The plan maps `/talk` Relations to `social` and adds `pulse` + `webrtc` containers.

### 3.6 Wellfair health system (commands/wellfair/* — 24 files, 60+ commands)

- **Health records:** list, export, companion pairing, Samsung folder import, receipts
- **Medication:** management, administration, reminders (preferences, permissions, due), OS notifications, diet, sleep, emergency contacts
- **Anatomy:** body view, comorbidity, scorecard, body assets, organ percepts
- **Clinical:** reports, attachments, file pickers, PWA publishing
- **Policy:** consent, conditions, allergies, housing safety
- **Sanctuary:** preferences, setup, lock/unlock, vault configuration, notes, decoy retention
- **Ledger:** append, verify, entries, balance
- **Library:** document ingestion, search, CML context, catalog
- **Chora:** worlds, layers, navigation, temporal, asset publishing
- **Agency:** domains, delegations, access evaluation
- **Guardianship:** proxy conditions, proposals, voting
- **Safeguard:** dead man's switch, incapacity switch, DEK recovery
- **Disclosure:** transparency CC, chain, leak tracing
- **Welfare support:** assistance needs, streams, government letters
- **Wellbeing:** observations, therapy notes, live shares
- **Work items:** add, status, board
- **Assessment:** recording, instruments
- **Credentials:** add, get, present
- **Crypto:** owner envelope, credential sealing/opening
- **Finance:** ledger entries, balance
- **Life records:** life events, welfare cases, case tasks
- **Sync:** relay, backup export/import, diagnostics
- **Consent credentials:** conduct audit trail

**Gap in qualia-ui:** The `health` and `anatomy` containers are in the plan but do not exist. None of the 60+ wellfair commands are reachable. The plan maps `/wellfair` Care to `health` + `anatomy` (consent-gated), `/work` Practice to `sheet` + `finance` + `rights`, and `/identity`/`/agency`/`/sanctuary` to `wallet` + `rights`.

### 3.7 Wallet & identity (commands/wallet.rs)

- `get_wallet_status`, `is_first_run`, `read_identity`, `save_identity`, `load_identity`
- `get_coin_balances`, `get_transaction_history`
- `generate_bip39_seed`, `derive_wallets_from_seed`, `import_external_seed`
- `get_tokens`, `add_token`, `remove_token`
- `get_tax_suite`, `save_tax_suite`, `dispatch_tax_payment`
- `build_send_xec`, `confirm_send_xec`, `send_ecash_token`
- `accept_vault_handshake`, `receive_vault_job`

**Gap in qualia-ui:** The `wallet` container is in the plan (§2, marked **missing — add**) but does not exist. No BIP39, no balances, no token management, no vault handshake.

### 3.8 Ingestion & inference (commands/ingest.rs)

- `ingest_pdf`, `ingest_literature`, `ingest_ontology`, `ingest_image`, `ingest_image_async`
- `upsert_cmld_definition`, `export_to_solid`
- `verify_graph_equivalence` — graph proof verification
- `discover_models`, `download_and_vectorize`, `download_model`, `cancel_download`
- `get_active_model`, `set_active_model`, `get_active_downloads`
- `run_agent_inference`

**Gap in qualia-ui:** The `doc` container (plan §2, `tools/rich_text`) should reach `nlp.analyze`, `Document.ingest`, and the gazetteer. None of that is wired. No model discovery/download/activation surface.

### 3.8b Hypermedia library — the Lived Memory / HypermediaStore surface (CRITICAL GAP)

This is the surface the first pass of this audit under-reported. The hypermedia library is one of the largest functional domains in the desktop + studio stack, and it has **no representation at all** in `qualia-ui` beyond the `doc` container name in the plan. It spans three layers: a dedicated Rust crate (`qualia-semantic-library`), 27 desktop Tauri commands, and two large studio UI panels (the Advanced Technical `library_panel.rs` at 2193 lines and the Naturalised `semantic_library.rs`).

#### 3.8b.1 `qualia-semantic-library` crate (the engine)

`crates/qualia-semantic-library` v0.0.30 — a Rust-native semantic library that turns a document corpus into `.hmc` hypermedia containers and feeds CML/RDF into QualiaDB. Offline dev tool; external LLM reached over HTTP behind a swappable `LlmBackend` trait, never compiled into the core runtime.

**Modules:**
| File | Purpose |
|---|---|
| `container/mod.rs` | The `.hmc` hypermedia container — a single ZIP file bundling the original source + all derived assets (canonical HTML, CML/RDF, plain text, structural chunks, embeddings, extraction metadata) behind one self-describing `HmcManifest`. BLAKE3 integrity, random access, idempotent re-runs. |
| `container/manifest.rs` | `HmcManifest`, `SourceInfo`, `AssetEntry`, `AssetKind` (Source, Text, Html, Cml, Chunks, Embeddings, ExtractionMeta), `PipelineInfo`, `HmcWriter`, `HmcContainer` |
| `ingest/mod.rs` | Ingest orchestration: file → `.hmc` container. Deterministic, no LLM. `ingest_file`, `ingest_path` (recursive with panic isolation per file), `IngestOptions`, `IngestResult` |
| `ingest/acquire.rs` | Source acquisition — hash (BLAKE3), dedup, MIME detection, `SourceInfo` |
| `ingest/extract.rs` | Structured text/HTML extraction (PDF via pdf-extract, HTML, txt, markdown, epub) |
| `ingest/chunk.rs` | Structural chunking → `chunks.jsonl` |
| `ingest/cml.rs` | CML/RDF annotation layer → `document.cml.ttl` |
| `embedding.rs` | f32 embedding matrix codec (centroid, cosine, decode_f32_matrix) |
| `library.rs` | `Library` — an index over `.hmc` containers. `scan`, `exact_duplicates` (BLAKE3), `near_duplicates` (centroid cosine), `search` (chunk-level cosine), `novelty_ranking` (1 − max cosine to known vectors) |
| `ontology.rs` | Ontology alignment |
| `reorganize.rs` | Propose/apply organised on-disk layout from container metadata + clusters |
| `llm/mod.rs` | Optional swappable LLM backend (`LlmBackend` trait) — Ollama today, cloud/native later. Used only for embeddings + structured method extraction. |
| `llm/ollama.rs` | Ollama HTTP backend |
| `bin/qsl.rs` | CLI binary `qsl` — ingest, search, dedup, reorganize |

**Container layout inside `.hmc`:**
```
manifest.json              # the index (HmcManifest)
source/<original>          # verbatim source bytes
derived/document.html      # canonical structured HTML
derived/document.txt       # plain text with [[page N]] markers
derived/document.cml.ttl   # CML / RDF annotations
derived/chunks.jsonl       # structural chunks
embeddings/vectors.f32     # row-major f32 embedding matrix
meta/extraction.json       # tool versions / timings / confidence
```

**Feature flags:** `pdf` (pdf-extract + lopdf), `llm-http` (reqwest). Default = both on.

**Gap in qualia-ui:** No `.hmc` container support. No ingest pipeline. No library index. No dedup/near-dup/search/novelty. The `qualia-semantic-library` crate is native-only (uses `std::fs`, `walkdir`, `zip`, `blake3`) and cannot be a direct WASM dependency — the HyperCanvas must reach it through desktop Tauri commands or a daemon HTTP endpoint.

#### 3.8b.2 Desktop Tauri commands — hypermedia library (commands/wellfair/library.rs)

27 commands covering the full library lifecycle:

**Ingest:**
- `wellfair_ingest_document` — ingest text document with person-authored facets (occurred_at, place_label, lat/lon, project, purpose, sensitivity, section, commons_visibility), guardianship flag → notify
- `wellfair_ingest_file_hex` — ingest binary asset (photo/audio) hex-encoded; EXIF capture-time + GPS auto-populate timeline + map
- `wellfair_ingest_legislation_text` — ingest legislation text
- `wellfair_ingest_legislation_pdf_hex` — ingest legislation PDF (hex)

**Search & query:**
- `wellfair_search_library` — free-text search
- `wellfair_search_library_time` — time-bounded search
- `wellfair_search_library_text` — text search
- `wellfair_query_library_faceted` — faceted query (topics, purposes, projects, media_type, sensitivity, section)
- `wellfair_library_facet_counts` — facet counts for filter UI
- `library_search`, `library_search_text`, `library_search_time` — non-wellfair-prefixed search variants

**Browse & list:**
- `wellfair_list_library` — list by section (all, secret, wellfair, personal, work, tools, software, commons)
- `library_list` — list library entries
- `wellfair_library_stats` / `library_stats` — library statistics (document count, ontology count, model count, graph facts)

**CML / semantic enrichment:**
- `wellfair_build_cml_context` — build CML context for a document
- `wellfair_build_cof_package` — build CoF (context-of-facts) package
- `wellfair_enrich_library_cml` — enrich library entry with CML

**Commons / sharing:**
- `wellfair_set_library_commons` — set commons visibility
- `wellfair_library_commons_share_card` — generate commons share card

**Perception / model / ontology catalogue:**
- `wellfair_seed_studio_qapps` — seed studio QApp inventory into library
- `wellfair_seed_perception_library` — seed perception (model/ontology/CV) library
- `library_seed_perception_assets` — seed perception assets (specialized-lib rows)
- `wellfair_list_qapp_catalog_categories` — list QApp catalog categories

**Management:**
- `wellfair_remove_library_entry` — remove library entry
- `wellfair_export_library_graph` — export library as graph

**Gap in qualia-ui:** None of these 27 commands are reachable. The `doc` container in the plan should be the primary ingest surface, and a `library` / `lived-memory` container (not yet in the plan §2 list — this is a gap in the plan itself) should be the browse/search/inspect surface. The plan §5 maps `/library` Lived Memory to `doc` + `graph` on Research, but the library panel in studio is far richer than a `doc` + `graph` pair — it has sections, facets, timeline/map views, perception catalogue, commons sharing, CML enrichment, and QApp catalog seeding.

#### 3.8b.3 Studio UI — Advanced Technical library panel (library_panel.rs, 2193 lines)

The existing studio library panel is the Advanced Technical workbench. It provides:

- **8 sections:** All, Secret (Sanctuary/private/classified), Wellfair (Health & welfare), Personal (Private life shelf), Work (Projects & labour), Tools (Logs/telemetry/agent output), Software (QApps/websites/packages), Commons (Peers & permissive share)
- **3 view modes:** List, Timeline, Map
- **Free-text search** + **facet search** (topics, purposes, projects, media_type, sensitivity, section)
- **Ingest:** text document, binary file (hex), legislation text, legislation PDF
- **Person-authored facets at ingest:** occurred_at, place_label, lat/lon, project, purpose, sensitivity, section, commons_visibility
- **Perception catalogue:** seed models, ontologies, computer_vision specialized-lib rows; filter perception vs computer-vision rows
- **Browser bookmarks:** purpose=bookmark filter
- **Commons sharing:** set commons visibility, generate share card
- **CML enrichment:** build CML context, build CoF package, enrich library CML
- **QApp catalog:** seed studio QApps, list catalog categories
- **Graph operations:** export library graph, view morph, view pick scene, view project library, view render memory spatial, view select/select_uri, view set observer, view set presentation level, view session
- **Honesty chips** per entry (live/partial/present/missing)
- **Sanctuary gating:** reads are vault-free (list/query/stats); sanctuary unlock only for secret-shelf UX and vault-gated writes

#### 3.8b.4 Studio UI — Naturalised Semantic Library (semantic_library.rs + views.rs)

The naturalised, human-facing surface for the HypermediaStore. Provides:

- **9 collections:** Overview, All items, Documents, Ontologies, AI models, Health records, Finance, QApps, Images & audio
- **Overview metrics:** Documents count, Ontologies count, AI models count, Graph facts count
- **Semantic pipeline visualization:** Source → Extract → Map → Validate → Use (5-step pipeline)
- **Import classification:** Ontology, Pdf, Image, Text, Tabular, Model, Unsupported (auto-classify by path)
- **Connected capability:** links to Anatomy + comorbidity engine (FHIR, LOINC, ICD)
- **Item inspector** and **EmptyCollection** states
- **Ingest** via `ingest_document` with facets
- **Faceted query** via `query_library_faceted`
- **Stats** via `library_stats`

#### 3.8b.5 Studio host client bridge (host_client/library.rs, ~700 lines)

The WASM-side Tauri invoke bridge for all library commands. Implements:
- `ingest_document` (with full `IngestFacets` struct: occurred_at, place_label, lat, lon, project, purpose, sensitivity, section, commons_visibility)
- `ingest_file_hex`, `ingest_legislation_text`
- `search_library`, `search_library_time`, `search_library_text`
- `list_library_section`, `query_library_faceted`
- `library_stats`, `library_commons_share_card`, `set_library_commons`
- `remove_library_entry`, `export_library_graph`
- `seed_perception_library`, `seed_studio_qapps`
- `view_morph`, `view_pick_scene`, `view_project_library`, `view_render_memory_spatial`, `view_select`, `view_select_uri`, `view_session`, `view_set_observer`, `view_set_presentation_level`

This is exactly the kind of host-client bridge `qualia-ui` needs to build for the `IntentBus` implementation.

#### 3.8b.6 qualia-ui hypermedia specs (tool-chest/)

Two spec documents define the hypermedia asset toolboxes that should live in the tool chest:

**`TOOLBOX_HYPERMEDIA_SPEC.md` (Part 1: image, audio, video)** — 3 toolboxes:
- `image` — 6 containers (image-canvas, layer-panel, brush-palette, colour-picker, histogram, navigator), 8 tool-chains (layers, brushes, selection, filters, masks, colour, vector, inspect). All containers marked `missing`.
- `audio` — audio production/editing toolbox (ontology: `audio-production.n3`)
- `video` — video production/editing toolbox (ontology: `video-production.n3`)

**`TOOLBOX_HYPERMEDIA_SPEC_2.md` (Part 2: 3D, interactive, portals, productions)** — 4 toolboxes:
- `3d` — 6 containers (viewport-3d, outliner, properties-3d, timeline-3d, material-editor, asset-library), tool-chains (object, modelling, rigging, animation, materials, rendering). All `missing`.
- `interactive` — HbbTV / interactive hypermedia (ontology: `interactive-hypermedia.n3`)
- `portal` — portal worlds (ontology: `portal-worlds.n3`)
- `production` — live productions: DMX, projection mapping (ontology: `production-events.n3`)

**7 domain ontologies** in `qualia-ui/ontologies/` support these toolboxes: `hypermedia.n3`, `image-editing.n3`, `audio-production.n3`, `video-production.n3`, `spatial-3d.n3`, `interactive-hypermedia.n3`, `portal-worlds.n3`, `production-document.n3`, `production-events.n3`.

**Gap in qualia-ui:** All 7 hypermedia toolboxes are specified but **none are implemented** — no containers, no tool-chains, no tools, no payloads. The specs define the full tool inventory (layers, brushes, selection, filters, masks, colour, vector, inspect for image alone) but none of it exists in `src/`. The `media` container seed in qualia-ui is a single 3D viewport placeholder with none of this surface.

#### 3.8b.7 Summary — what the HyperCanvas needs for the hypermedia library

| Capability | Engine (desktop/semantic-library) | Studio UI surface | qualia-ui state | Priority |
|---|---|---|---|---|
| `.hmc` container ingest (text, PDF, image, legislation) | `qualia-semantic-library` crate + 4 desktop commands | `library_panel.rs` + `semantic_library.rs` | **missing** — no `doc` container body, no ingest UI | P0 |
| Library browse (8 sections, 9 collections) | `wellfair_list_library`, `library_list` | `library_panel.rs` (2193 lines) | **missing** — no `library`/`lived-memory` container | P0 |
| Faceted search + free-text search | `wellfair_query_library_faceted`, `wellfair_search_library*` | Both panels | **missing** | P0 |
| Timeline + Map views | `wellfair_search_library_time` | `library_panel.rs` | **missing** | P1 |
| Near-duplicate / novelty / dedup | `qualia-semantic-library::Library` | (engine-level, not surfaced in studio) | **missing** | P2 |
| CML enrichment + CoF package | `wellfair_build_cml_context`, `wellfair_build_cof_package`, `wellfair_enrich_library_cml` | `library_panel.rs` | **missing** | P1 |
| Commons sharing | `wellfair_set_library_commons`, `wellfair_library_commons_share_card` | `library_panel.rs` | **missing** | P2 |
| Perception catalogue (models, ontologies, CV) | `wellfair_seed_perception_library`, `library_seed_perception_assets` | `library_panel.rs` | **missing** | P2 |
| QApp catalog seeding | `wellfair_seed_studio_qapps`, `wellfair_list_qapp_catalog_categories` | `library_panel.rs` | **missing** | P2 |
| Library graph export + view ops | `wellfair_export_library_graph`, `view_*` commands | `library_panel.rs` | **missing** | P1 |
| Image editing (7 tool-chains) | (not yet in desktop — spec only) | (not in studio) | **missing** — `TOOLBOX_HYPERMEDIA_SPEC.md` defines it | P2 |
| Audio production/editing | `qualia-audio` crate + `audio_*` commands | `listen_workbench.rs` | **missing** — spec defines toolbox | P2 |
| Video production/editing | (not yet in desktop — spec only) | (not in studio) | **missing** — spec defines toolbox | P2 |
| 3D editing (6 containers, 6 tool-chains) | `webizen-render` + GPU surface | `ten_d_browser.rs`, `native_gpu_viewport.rs` | **missing** — spec defines toolbox | P2 |
| Interactive hypermedia (HbbTV) | (spec only) | (not in studio) | **missing** — spec defines toolbox | P2 |
| Portal worlds | (spec only) | (not in studio) | **missing** — spec defines toolbox | P2 |
| Live productions (DMX, projection) | (spec only) | (not in studio) | **missing** — spec defines toolbox | P2 |

**This is the largest single functional gap in the audit.** The hypermedia library is the primary knowledge-management surface in the desktop/studio stack — 27 Tauri commands, a dedicated Rust crate, two large UI panels (2193 + ~430 lines), 7 hypermedia toolbox specs, and 9 supporting ontologies. `qualia-ui` has none of it. The plan §5 maps `/library` Lived Memory to `doc` + `graph` on Research, but that mapping under-represents the library surface — the library panel is not just a doc viewer + graph explorer, it is a full ingest/browse/search/facet/timeline/map/commons/perception/CML/catalog surface.

**Recommendation:** Add a dedicated `library` (or `lived-memory`) container to the plan §2 inventory, distinct from `doc`, that hosts the browse/search/facet/timeline/map/commons surface. The `doc` container is the authoring/ingest surface; the `library` container is the discovery/management surface. Both reach the same 27 desktop commands through the `IntentBus`.

### 3.9 Vision & audio (commands/vision_audio.rs, mic_capture.rs)

- Vision: `vision_run_synthetic_demo`, `vision_reject_instance`, `vision_correct_instance`, `vision_generate_image`, `vision_image_to_3d_demo`, `vision_gs_continuum`, `vision_detect_image_file`, `vision_section15_smoke`, `vision_ensure_weights`, `vision_detect_disk_weights_demo`, `vision_twin_elasticity_demo`, `vision_super_resolve`
- Audio: `audio_ears_demo`, `audio_cross_modal_demo`, `audio_section18_smoke`, `audio_import_wav`, `audio_reject_instance`, `audio_correct_instance`, `audio_ears_weighted`, `audio_sonify_hear`, `audio_speech_demo`, `audio_capture_policy_demo`, `audio_pick_wav_path`, `audio_mic_start`, `audio_mic_stop`, `audio_mic_status`, `audio_ensure_weights`, `audio_daw_history_demo`, `audio_live_aed`, `audio_speech_disk`, `audio_music_demo`, `audio_daw_fx_demo`, `audio_gen_demo`, `audio_shared_clock_demo`, `audio_mixer_default`, `audio_mixer_bounce`, `audio_capabilities`

**Gap in qualia-ui:** The `vision` and `listen` containers are in the plan (§2, both marked **missing — add**) but do not exist. The `triad` container (q42+p64+d10, `qualia-audio`) is also missing. The `media` container exists as a seed but renders a 3D viewport placeholder with no kinematics, vision ahash, or grapheme binding.

### 3.10 Render & GPU (commands/render.rs, render_pipeline.rs, native_surface.rs, runtime.rs)

- `update_render_preview`, `toggle_render_loop`, `navigate_to_node`, `select_node_at`
- Native GPU surface: `mount_gpu_surface`, `set_gpu_scene`, `set_gpu_camera`, `set_gpu_camera_mode`, `upload_gpu_mesh`, `upload_gpu_mesh_colored`, `upload_gpu_10d_mesh`, `load_gpu_10d_asset`, `unmount_gpu_surface`
- WGPU diffusion runtime: `DiffusionConfig`, `FrameHandle`, `LedgerRecord`, `SimulationKernel`, `SharedFrameBuffer`
- Custom protocols: `webizen://localhost/diffusion/frame/{slot}`, `webizen://localhost/render/preview.png`, `webizen://localhost/anatomy/body.png`, `webizen://localhost/anatomy/body.json`, `webizen://localhost/anatomy/10d/{model}/{organ_key}`

**Gap in qualia-ui:** The `3d` container (plan §2, `tools/mesh`) is `present` in the plan but does not exist in qualia-ui. No GPU frame bridge, no scene contract, no 10D mesh loading. The `map` container exists as a seed with a GIS placeholder but has no `Render.scene`, hull, or `Manifold.project` binding.

### 3.11 Telemetry (commands/telemetry.rs, telemetry_bridge.rs, telemetry_hooks.rs)

- `collapse_wavefunction`, `collapse_wavefunction_legacy`, `set_temporal_slice`, `register_browser_capabilities`
- `load_ccf_asset`, `list_ccf_assets`, `test_ccf_ipc_handshake`, `test_larynx_smoke`, `test_vasculature_stress`
- Thread-safe telemetry bridge for ambient visualization
- Zero-heap telemetry extraction hooks

**Gap in qualia-ui:** The right dock has a "Pulse stream" panel with static event entries, but it is not wired to the telemetry bridge. No wavefunction collapse, no temporal slice, no browser capability registration.

### 3.12 Native scientific bindings (commands/native_bindings.rs)

- `calculate_chemistry_properties`, `calculate_framingham_risk`, `calculate_quantum_dft`, `calculate_monte_carlo_var`

**Gap in qualia-ui:** The `health` container should reach `calculate_framingham_risk`. The `sheet`/`latex` containers should reach chemistry, quantum DFT, and Monte Carlo. None exist.

### 3.13 Mesh networking (commands/mesh.rs)

- `mesh_start`, `mesh_stop`, `mesh_status`

**Gap in qualia-ui:** No mesh surface. The plan does not list a dedicated mesh container; it is reachable via `communication` toolbox / `webrtc` container.

### 3.14 Updater (commands/updater.rs, updater_service.rs)

- `updater_check`, `updater_download_and_install`, `updater_restart`

**Gap in qualia-ui:** No updater surface. This is a desktop-host concern but the HyperCanvas should surface update availability in the status bar.

### 3.15 Companion gateway & WebRTC (companion_gateway.rs, webrtc_manager.rs)

- Phone WebSocket + loopback POST for Samsung health bundles
- WebRTC peer connection manager for PWA delivery
- **TODO in desktop:** WebRTC data channel not yet wired to a WebRTC engine

**Gap in qualia-ui:** The `webrtc` container is `present` in the plan (no fake stream — stays present until consent gate designed). Does not exist in qualia-ui.

### 3.16 MCP server (mcp_server.rs)

- TCP server on `127.0.0.1:4245` for JSON-RPC tool calls
- `mcp_list_local_tools`, `mcp_call_tool_gated`, `agent_set_allowed_mcp_tools`, `mcp_ensure_safe_tool_allowlist`

**Gap in qualia-ui:** No MCP inspector. The studio has `components/mcp_inspector.rs`; qualia-ui has nothing.

### 3.17 Settings server (settings_server.rs)

- Loopback HTTP portal on `127.0.0.1:8080`
- Studio WASM serving
- Portal documentation assets

**Gap in qualia-ui:** The `settings` manifold seed exists with preferences/capabilities/protection-policies containers, but all three are placeholders. No preference query/update, no capability grant/revoke, no configuration management.

### 3.18 GLB / 10D asset management (commands/glb_ingest.rs, browser_10d.rs, binary_registry.rs)

- `ingest_glb` with semantic extraction (FMA, SNOMED-CT)
- `browse_10d_containers`, `browse_vision_10d`, `load_vision_10d`, `scrub_vision_10d_paint`, `inspect_10d_container`, `open_10d_file_picker`
- Binary node registry for GLB/10D assets

**Gap in qualia-ui:** The `3d` and `anatomy` containers should reach these. Neither exists.

### 3.19 Poet (commands/poet.rs, poet_render.rs)

- `poet_eval`, `poet_reset`, `poet_gazetteer`, `poet_capabilities`, `poet_render_preview`

**Gap in qualia-ui:** The `vibe` manifold seed has a VibeScript console container, but it is a static placeholder. No `poet_eval`, no gazetteer, no capabilities query. The plan §4 says to extract `vibe/` from the console into a `poet/tools/vibe/` family.

### 3.20 Directory & personal directory (commands/directory.rs, personal_directory.rs)

- `get_front_doors`, `generate_front_door`, `get_directory_actors`, `add_directory_actor`, `get_delegation_rules`, `add_delegation_rule`
- `list_directory`, `list_directory_categories`, `create_directory_category`, `set_directory_entry_categories`, `search_directory`

**Gap in qualia-ui:** No directory surface. The plan does not list a dedicated directory container; it is reachable via `rights`/`wallet` containers.

---

## 4. webizen-studio — capabilities the HyperCanvas must subsume as containers/tools

The studio crate is the Classic UI. The plan §5 says Classic stays default and Poet *reaches* these surfaces; it does not delete the routes. The inventory below is what the HyperCanvas containers must eventually present.

### 4.1 Canvas / workspace editor (canvas_editor.rs, canvas_graph.rs, canvas_model.rs, studio_canvas.rs)

- Undo/redo history (32-entry bounded stack)
- Grid snapping and clamping
- Layout strategies: PointGrid, CssGrid, FlexBox, Masonry
- Presentation modes: GridBound, NodeRelational, Spatial
- Coordinate spaces: GlobalCartesian, RelativeAnchored
- Layer behaviors: Docked, FloatingOverlay, ModalOverlay, FullManifold
- Pane relationship graph derivation (anchors, data_bindings)
- Graph neighbor queries with strength scoring
- QPrime elevation CSS (motion, shadows, animations)
- Workspace manifest persistence (via daemon endpoints)
- Keyword-driven pane layout planner (`pane_generator.rs`)
- Pane registry mapping semantic type hashes to component identifiers (`pane_registry.rs`)

**Gap in qualia-ui:** qualia-ui has visual pan/zoom/drag/selection and manifold switching, but no undo/redo, no resize, no grid snapping, no layout strategies, no pane relationship graph, no manifest persistence, no pane registry, no layout planner. This is plan §6 item 1 (desk physics) and item 3 (epistemic/strata as tools).

### 4.2 Theme engine (theme_engine.rs)

- QPrime preset system (4 canonical: fiduciary-dark, commons-light, sanctuary, infosphere)
- Legacy presets (human-warmth, twilight-blue, midnight-slate)
- Theme binding with presentation levels (P0-P6)
- Spatial grammar hints (list, board, globe, room, strata, hyperspace)
- Sanctuary mode: zero motion, spatialize disabled
- WCAG contrast ratio calculation
- Shoelace design token bridging
- Motion timeline integration

**Gap in qualia-ui:** Single hardcoded 567-line CSS string in `css.rs`. No theme presets, no sanctuary mode, no WCAG enforcement, no Shoelace bridging, no motion timeline. This is a P1 gap.

### 4.3 Render subsystem (render/*.rs)

- Backend-agnostic `Renderer` trait (immediate-mode 3D)
- CPU Canvas 2D renderer for WASM
- Native wgpu integration (native-only)
- Scene graph with nodes, meshes, transforms
- Camera with projection (world → screen)
- Spring physics for motion
- Motion loop with selection pulse and mode pulse
- Semantic scene building from QualiaDB data
- Node graph visualization
- Tensor buffer utilities
- Studio preview pipeline (native-only)
- Scene to contract conversion (native-only)

**Gap in qualia-ui:** No render subsystem at all. The `media` and `3d` containers are placeholders. This is a P1 gap — port the WASM Canvas2D renderer and the scene graph.

### 4.4 Wellfair life-domain shell (components/wellfair/*.rs — 70+ files)

- **Selfhood/Identity:** personal profile, social book, consent management
- **Relations:** people, chat, reception, projects, mail, agreements
- **Memory (Lived Memory):** semantic library, hypermedia shelf, advanced/simple modes
- **Care:** health vault, anatomy 3D, clinical, wellbeing, sleep, medication, comorbidity
- **Practice:** work board (Kanban), projects, finance, credentials
- **Instruments:** vision workbench, listen workbench, tools, sync/backup, audit
- **World:** browser pane, Chora commons (spatio-temporal manifold)
- **Sanctuary:** vault lock, protected spaces, decoy retention
- **Agency:** guardianship, accountability, safeguards
- **Host Client:** Tauri command bridge for all wellfair features (20+ modules)

**Gap in qualia-ui:** None of the 7 life domains are implemented. The plan maps them to containers: `/library` → `doc` + `graph`, `/talk` → `social`, `/wellfair` → `health` + `anatomy`, `/browser` → `webview`, `/work` → `sheet` + `finance` + `rights`, `/listen` → `listen`/triad, `/vision` → `vision`/media, `/anatomy` → `anatomy` + `Render.scene`, `/identity`/`/agency`/`/sanctuary` → `wallet` + `rights`, `/chora`/`/10d-browser`/`/gpu-viewport` → `3d`/`map`/native canvas.

### 4.5 QApps studio (components/qapps.rs, qapp_dispatcher.rs, contextual_workspace.rs)

- QApps catalog with 300+ academic discipline modules
- QApp dispatcher for runtime QApp loading
- Context studio with graph visualization
- Dynamic page renderer for QApp editing
- Pane layout planner (keyword-driven)
- QApp publishing panel
- Registry pane preview

**Gap in qualia-ui:** No QApp surface. Plan maps `/qapp-studio` to `code` + `portal` containers.

### 4.6 Academic QApps (300+ modules)

- 300+ academic discipline QApps across 23 categories (Social Sciences, Humanities, Natural Sciences, Formal Sciences, Interdisciplinary, Applied, Emerging, Specialized, Pre-Professional, Language/Regional, Arts/Performance, Advanced Sub-Disciplines, Niche Sciences, Philosophy/Theory, Religious/Theological, Literary/Media, Linguistics/Semiotics, Intersectional/Applied, Historical/Textual, Critical/Cultural, Interdisciplinary Science, Design/Media/Spatial, Advanced Critical Theory)
- **Note:** Many are catalogued stubs (documented in `library_panel.rs`)

**Gap in qualia-ui:** No academic QApp inventory. These are reached via the `portal` container when implemented.

### 4.7 Specialized tools (studio components)

| Domain | Studio components | Plan container/tool |
|---|---|---|
| Clinical | `clinical_risk_scorer`, `comorbidity_analyzer`, `dicom_viewer` | `health`, `anatomy` |
| Scientific | `chemistry_modeler`, `physics_simulator`, `ode_solver`, `statistical_analysis` | `sheet`, `latex` |
| Quantum | `qpu_optimizer`, `qpu_providers`, `quantum_dft`, `qaoa_explorer`, `zk_proof_studio` | `sheet`, `latex`, `rights` |
| Data | `matrix_lab`, `portfolio_analyzer`, `risk_engine` | `sheet`, `finance` |
| Network | `p2p_dashboard`, `webtorrent_seeder`, `ebpf_filter_manager` | `webrtc`, `communication` toolbox |
| Knowledge | `n3_logic_studio`, `shacl_validator`, `sparql_explorer`, `rdf_star_editor` | `graph`, `ontology`, `code` |
| Governance | `deontic_logic_editor`, `agreements_rights`, `key_vault_manager` | `rights`, `wallet` |
| Storage | `storage_driver_config`, `q42_volume_manager`, `wal_inspector` | `sheet`, `code` |
| AI/ML | `llm_harness`, `lora_manager`, `inference_monitor`, `model_lifecycle` | `code`, `ai` toolbox |
| Extensions | `extension_bus`, `mcp_inspector`, `tool_use_card` | `code`, `ai` toolbox |
| Provenance | `provenance_graph`, `graph_proof` | `graph`, `rights` |
| Testing | `benchmark_harness`, `gbm_simulator` | `sheet`, `code` |

**Gap in qualia-ui:** None of these specialized tools exist. Each is a container body + tool family per the plan.

### 4.8 Browser integration (components/browser_panes/*.rs)

- Web browser pane (desktop webview only)
- Dialectical sidebar pane
- Shared browser utilities
- Browser unavailable fallback (public web demo)

**Gap in qualia-ui:** No `webview` container. Plan §2 lists it as `present` with `tools/webframe` and capability-gated fetch.

### 4.9 Command palette (components/command_palette.rs)

- Ctrl+K / Ctrl+P command palette
- Omnibox routing (keywords → routes)
- URL detection and browser handoff
- Draft handoff for Talk composer

**Gap in qualia-ui:** No command palette. The topbar has a menubar but no omnibox. The desktop has `submit_omnibox_query`.

### 4.10 Diagnostics & monitoring (agent_qa_panel.rs, job_center.rs, problems_pane.rs, telemetry.rs)

- Agent QA panel for debugging
- Job center for background job queue
- Problems pane for operations issues
- Desktop logs viewer with debug mode toggle
- Device telemetry (WASM local_storage / native file)
- Native error reporting (window.error, unhandledrejection, rust.panic)
- Desktop status polling

**Gap in qualia-ui:** The right dock has an "Aura tray" (SHACL validation status — static) and a "Pulse stream" (static event log). No agent QA, no job center, no problems pane, no diagnostic polling.

### 4.11 10D integration (ten_d_browser.rs, native_gpu_viewport.rs)

- 10D manifold browser
- Native GPU viewport page
- Anatomy test component

**Gap in qualia-ui:** No 10D browser, no GPU viewport. Plan maps `/chora`/`/10d-browser`/`/gpu-viewport` to `3d`/`map`/native canvas.

### 4.12 Poet / Vibe harness (poet_harness.rs, poet.rs)

- Poet harness for Vibe 0.1 interpreter
- Shell kind switcher (Poet vs Classic)
- Poet-specific chrome and routing

**Gap in qualia-ui:** The `vibe` manifold seed has a VibeScript console container (placeholder). No `poet_eval`, no shell kind switcher. Plan §4 says extract `vibe/` from console into `poet/tools/vibe/`.

### 4.13 Knowledge Nexus (nexus.rs)

- Knowledge Nexus for research, claims, and epistemic threads

**Gap in qualia-ui:** No Nexus. Plan maps `Nexus / SPARQL / SHACL panes` to `graph` + `ontology` containers.

### 4.14 Ontology tools (ontology_import_wizard.rs, personal_ontology.rs)

- Ontology import wizard with layout suggestions
- Personal ontology editor

**Gap in qualia-ui:** The `ontology` container is `partial` in the plan. qualia-ui has `OntologyRegistry` type but no loader and no editor UI. No import wizard.

### 4.15 Hardware & infrastructure (hardware_configurator.rs, health_monitor.rs, endpoints.rs)

- Hardware configurator
- Health monitoring
- Daemon endpoint management
- Capability detection (`is_native_host()`, `current_host_surface()`, `supports_browser_pane()`)

**Gap in qualia-ui:** No capability detection. This is critical — containers must fail closed when an engine is unreachable. Port `webizen-studio::endpoints` capability gating.

### 4.16 Daemon endpoints (endpoints.rs)

- `http://127.0.0.1:{port}/manifest` — workspace manifest GET/POST
- `http://127.0.0.1:{port}/manifest/history` — Quin WAL checkpoints
- `http://127.0.0.1:{port}/design-studio.html` — Portal design studio
- `http://127.0.0.1:{port}/api/assets/catalog` — Assets catalog
- `http://127.0.0.1:{port}/api/assets/enqueue` — Asset enqueue
- `http://127.0.0.1:{port}/api/jobs/{id}` — Job status
- `http://127.0.0.1:{port}/manifest/replay/{revision}` — Manifest replay
- `http://127.0.0.1:{port}/generate_pane` — Pane layout planner
- `http://127.0.0.1:{port}/manifest/undo-chain` — Undo chain
- `http://127.0.0.1:{port}/manifest/undo-frame?stack_index={n}` — Undo frame
- `http://127.0.0.1:{port}/telemetry` — Telemetry SSE stream
- `http://127.0.0.1:{port}/logs` — Logs page
- `ws://127.0.0.1:4242` — Native LLM / handshake WebSocket
- `ws://127.0.0.1:9001` — Custom web-module sandbox RPC socket

**Gap in qualia-ui:** No endpoint client. The `IntentBus` implementation should route to these endpoints when running inside the desktop webview.

---

## 5. Gap summary by HyperCanvas container

This is the authoritative mapping from the plan §2 container inventory to the current qualia-ui state and the desktop/studio capability it must reach.

| Container | Plan honesty | qualia-ui state | Desktop/studio capability to reach | Priority |
|---|---|---|---|---|
| `doc` | live | **missing** | `ingest_pdf`, `ingest_literature`, `nlp.analyze`, gazetteer, `Document.ingest`, HCF/`<q-doc>`, `wellfair_ingest_document`, `wellfair_ingest_file_hex` | P0 |
| `library` | **not in plan §2** | **missing** | 27 desktop library commands: ingest, browse (8 sections), faceted search, timeline/map, CML enrichment, commons, perception catalogue, QApp catalog, graph export. See §3.8b. **Recommend adding to plan.** | P0 |
| `code` | live | **missing** | `poet_eval`, diagnose, `capability.invoke`, `execute_sparql_query`, `evaluate_logic_rules` | P0 |
| `sheet` | live (range) | **missing** | `Sheet.sum_range`, `Sheet.stats`, Statistics, P64 latents, `calculate_chemistry_properties`, `calculate_monte_carlo_var` | P0 |
| `ontology` | partial | **missing** | `fetch_domain_ontology`, `validate_shacl_shape`, catalog TTL | P1 |
| `map` | live (scene) | seed placeholder | `Render.scene`, hull, `Manifold.project`, GeoSPARQL | P1 |
| `media` | live (scene) | seed placeholder | kinematics, vision ahash, grapheme, Forge | P1 |
| `social` | live (LWW) | static rendering | `Social.lww`, chat graph, Pulse, `list_chat_sessions`, `append_chat_message`, `get_chat_graph` | P0 |
| `health` | live (Framingham) | **missing** | `calculate_framingham_risk`, anatomy renderer, Wellfair (consent), 60+ wellfair commands | P2 |
| `subcanvas` | partial | **missing** | switch manifold; enter-zoom | P1 |
| `3d` | present | **missing** | `/gpu-viewport`, `webizen-render`, `.d10`/vocal tract, `mount_gpu_surface`, `upload_gpu_10d_mesh` | P1 |
| `webrtc` | present | **missing** | desktop `webrtc` crate; no fake stream (stays present until consent gate) | P2 |
| `webview` | present | **missing** | `<q-web-frame>`, capability-gated fetch, `browser_navigate`, trust store | P1 |
| `portal` | present | **missing** | wormhole IRI; later multi-tenant; QApp dispatch | P2 |
| `latex` | **missing** (add) | **missing** | SymbolicAlgebra, CAS wasm | P1 |
| `graph` | **missing** (add) | **missing** | SPARQL, `quin.statement`, RDF 1.2, `execute_sparql_query` | P0 |
| `triad` | **missing** (add) | **missing** | q42+p64+d10, `qualia-audio` | P2 |
| `pulse` | **missing** (add) | **missing** | `pulse.publish` allowlist (not `pulse.broadcast`), telemetry SSE | P1 |
| `rights` | **missing** (add) | **missing** | deontic, DID, fiduciary sign, `list_agreements`, `create_agreement` | P2 |
| `wallet` | **missing** (add) | **missing** | identity/key vault, BIP39, balances, `accept_vault_handshake` | P2 |
| `anatomy` | **missing** (add) | **missing** | Wellfair anatomy `RenderScene`, `browse_10d_containers`, organ percepts | P2 |
| `listen` | **missing** (add) | **missing** | `qualia-audio`, EnCodec, `audio_*` commands | P2 |
| `vision` | **missing** (add) | **missing** | `ComputerVision.ahash`, `qualia-vision`, `vision_*` commands | P2 |
| `finance` | **missing** (add) | **missing** | Black-Scholes, portfolio, `fetch_wallet_portfolio` | P2 |
| `slide` | **missing** (add) | **missing** | office toolbox "presentation" | P2 |

**Current qualia-ui container count:** 4 with content (`social`, `connection-requests`, `protection-policies`, `conversations`) + 6 placeholders (`reputation`, `capabilities`, `settings`, `channels`, `presence`, generic).
**Plan container count:** 24 containers + 1 recommended addition (`library` / lived-memory, see §3.8b) = 25.
**Missing:** 21 containers entirely (including `library`); 6 are placeholders needing real bodies; 4 have static content needing engine wiring.

---

## 6. Gap summary by tool family (plan §4)

| Tool family | Plan status | qualia-ui state | Required |
|---|---|---|---|
| `rich_text/` | started | **missing** | `ToolbarSpec` groups, mark wrap, gazetteer, `<q-entity>` annotation, selection API, contenteditable, yaml-ld-q42 persistence |
| `sheet/` | toolbar stub | **missing** | Excel formula bar, command list, `Sheet.sum_range`, `Sheet.stats`, range stats via Vibe |
| `latex/` | toolbar stub | **missing** | Overleaf snippet bar, wrap + CAS invoke, SymbolicAlgebra |
| `vibe/` | still in console | placeholder container | IDE + REPL, extract from console, `poet_eval`, diagnose, `capability.invoke` |
| `aura/` | not yet | **missing** | SHACL inspector, shape list, `validate_shacl_shape` |
| `gis/` | not yet | **missing** | QGIS-lite pin/track, `Render.scene`, hull, `Manifold.project` |
| `graph/` | not yet | **missing** | UI_Mockup graph-panel, SPARQL, `quin.statement`, RDF 1.2 |
| `triad/` | not yet | **missing** | articulatory inspector, q42+p64+d10, `qualia-audio` |
| `pulse/` | not yet | **missing** | UI_Mockup pulse-panel, `pulse.publish` allowlist, telemetry SSE |

**Missing:** 8 of 9 tool families entirely; 1 (`vibe`) exists as a placeholder container.

---

## 7. Gap summary by toolbox (plan §3)

| Toolbox | Places/actions | qualia-ui state |
|---|---|---|
| `epistemic` | tag objective/subjective/intersubjective/normative | **missing** — qualia-ui has epistemic lens selector buttons but no tool that sets `node.epistemic` |
| `office` | +doc, +ontology, +slide | **missing** — no toolbox registration |
| `image` | +media, marker, heatmap | **missing** |
| `sheet` | +sheet, import, resonance | **missing** |
| `spatial` | +map, +3d, +portal, pin, track | **missing** |
| `communication` | +social, +webrtc, +webview | **missing** |
| `rights` | authors group, fiduciary, DID sign | **missing** |
| `health` | +health, pathology, 10D anatomy | **missing** |
| `code` | +vibe cell, requires[], quin.statement | **missing** |
| `ai` | co-author, extractor, sentinel, triad | **missing** |

**Current qualia-ui toolbox dock:** 7 static buttons (Social, Settings, Communications, Office, Research, Media, Rights) with click selection but no registration, no tool-chains, no tools, no container placement action.

**Missing:** All 10 toolboxes need registration with their tool-chains and tools.

---

## 8. Cross-cutting infrastructure gaps

These are not containers or tools but foundational capabilities the HyperCanvas needs to function.

### 8.1 IntentBus implementation (P0)

**Current:** `IntentBus` is a trait in `intent_bus.rs` with `dispatch()` and `cancel()` methods. No struct implements it.

**Required:** A concrete `WasmIntentBus` struct that:
- Takes a `VibeScriptPayload<P>`, stamps provenance, checks capability scope
- Routes to the desktop daemon HTTP/WS endpoints when `is_native_host()` is true
- Routes to a public-web relay or fails closed when on public web
- Returns `IntentReceipt` with status and provenance
- Supports cancellation via a request ID
- Serializes payloads as CBOR-LD (ciborium is already a dependency)

### 8.2 Backend transport (P0)

**Current:** No HTTP client, no WebSocket client, no Tauri invoke bridge.

**Required:**
- `web-sys::WebSocket` for the telemetry SSE stream (`ws://127.0.0.1:4242`, `ws://127.0.0.1:9001`)
- `web-sys::fetch` (via `wasm-bindgen-futures`) for daemon HTTP endpoints
- A Tauri `invoke` bridge when running inside the desktop webview (detect via `window.__TAURI_INTERNALS__`)
- Capability detection: `is_native_host()`, `current_host_surface()`, `supports_browser_pane()` (port from `webizen-studio::endpoints`)

### 8.3 Ontology loading (P0)

**Current:** `OntologyModule` has a `cbor_path` field but no loading logic. `ontologies/mod.rs` says "Placeholder — ontology loading tests will be added when core/ontology.rs implements CBOR-LD ontology loading." 36 N3 files exist but are not compiled or loaded.

**Required:**
- A build-time N3 → CBOR-LD compiler (Trunk hook or a `build.rs` in a separate host crate, since `build.rs` cannot easily run N3 parsing in a WASM target)
- Embedded CBOR-LD bytes via `include_bytes!` in the WASM binary
- A WASM `OntologyLoader` that populates `OntologyRegistry` from embedded bytes at startup
- Alternatively, fetch pre-compiled CBOR-LD from the daemon when native

### 8.4 Toolbox/tool registration (P0)

**Current:** `Registry::new()` creates an empty registry. No code calls `register_toolbox()`, `register_manifold()`, or registers tools at startup.

**Required:** A startup initialization function that:
- Registers all 10 toolboxes with their metadata
- Registers all tool-chains within each toolbox
- Registers all tools within each tool-chain
- Registers all 6 (expanding to 24) manifold seeds
- Loads ontologies into the registry
- Makes the registry available to the browser shell for toolbox dock rendering and container placement

### 8.5 Canvas editor semantics (P1)

**Current:** Visual pan/zoom/drag/selection and manifold switching. No undo/redo, no resize, no grid snapping, no manifest persistence.

**Required (port from `webizen-studio::canvas_editor`):**
- `WorkspaceHistory` — 32-entry bounded undo/redo stack
- Grid snapping and clamping utilities
- Resize handles on containers
- `ManifoldSeed` serialization/deserialization (CBOR-LD)
- Manifest persistence via daemon `POST /manifest`
- Pane relationship graph derivation

### 8.6 Theme engine (P1)

**Current:** Single hardcoded 567-line CSS string.

**Required (port from `webizen-studio::theme_engine`):**
- QPrime preset system (4 canonical + 3 legacy)
- Theme binding with presentation levels (P0-P6)
- Sanctuary mode (zero motion, spatialize disabled)
- WCAG contrast ratio calculation
- Shoelace design token bridging
- CSS custom property injection (replace hardcoded string)

### 8.7 Render subsystem (P1)

**Current:** None.

**Required (port WASM parts from `webizen-studio::render`):**
- `Renderer` trait (immediate-mode 3D)
- `Canvas2D` renderer (CPU, WASM-compatible)
- Scene graph with nodes, meshes, transforms
- Camera with projection
- Spring physics for motion
- Motion loop with selection pulse
- Semantic scene building from QualiaDB data
- Bridge to native GPU viewport when inside desktop webview (via Tauri `mount_gpu_surface` commands)

### 8.8 Capability detection (P1)

**Current:** None.

**Required (port from `webizen-studio::endpoints`):**
- `is_native_host()` — returns true when daemon is reachable
- `current_host_surface()` — `PublicWeb` vs `DesktopWebview`
- `supports_browser_pane()` — browser pane only on desktop
- Endpoint URL construction for manifest, assets, jobs, telemetry, logs
- Containers must call these to fail closed when an engine is unreachable

### 8.9 Command palette / omnibox (P1)

**Current:** Topbar has menubar items (File/Edit/View/Insert/Help) but no omnibox.

**Required (port from `webizen-studio::command_palette`):**
- Ctrl+K / Ctrl+P palette
- Keyword → route/container/tool routing
- URL detection and browser handoff
- Draft handoff for chat composer
- Desktop has `submit_omnibox_query` command

### 8.10 Diagnostics & monitoring (P1)

**Current:** Right dock has static Aura tray and Pulse stream.

**Required:**
- Wire Aura tray to `validate_shacl_shape` results
- Wire Pulse stream to telemetry SSE (`ws://127.0.0.1:{port}/telemetry`)
- Job center for background job queue (`list_local_jobs`, `cancel_local_job`)
- Problems pane for operations issues
- Desktop status polling (`get_desktop_status`)
- Error reporting (`report_client_error`)

### 8.11 Duplicate directory cleanup (P2)

**Current:** `tool-chest/` directory at project root duplicates `src/tool_chest/`. The root copy is not part of the build.

**Required:** Remove the root `tool-chest/` directory (or move the spec markdown files into `docs/` and delete the duplicate Rust source).

---

## 9. Implementation priority (aligned with plan §6)

The plan §6 gives the implementation order. This section annotates it with the specific gaps from this audit.

### Phase 1 — Desk physics (this session, plan §6 item 1)

- [ ] Undo/redo history (port `WorkspaceHistory` from studio)
- [ ] Grid snapping and clamping
- [ ] Resize handles on containers
- [ ] Motion (spring physics, selection pulse)
- [ ] `ManifoldSeed` CBOR-LD serialization
- [ ] Manifest persistence via daemon `POST /manifest`

### Phase 2 — Authoring trio (plan §6 item 2)

- [ ] `rich_text/` tool family — `ToolbarSpec`, mark wrap, gazetteer, `<q-entity>` annotation
- [ ] `sheet/` tool family — formula bar live, range stats via Vibe, `Sheet.sum_range`, `Sheet.stats`
- [ ] `latex/` tool family — wrap + CAS invoke, SymbolicAlgebra
- [ ] `doc`, `sheet`, `latex` containers with real bodies

### Phase 3 — Epistemic/strata as tools (plan §6 item 3)

- [ ] `epistemic` toolbox — palette sets `node.epistemic` field (replace alert placeholders)
- [ ] Strata selector wired to container metadata
- [ ] Dimension switch (2D/3D/4D) wired to render subsystem

### Phase 3b — Hypermedia library / Lived Memory (not in plan §6 — add, see §3.8b)

- [ ] `doc` container body — ingest text/PDF/image/legislation via `wellfair_ingest_document`, `wellfair_ingest_file_hex`, `wellfair_ingest_legislation_*`
- [ ] `library` container (recommended addition to plan §2) — browse 8 sections, faceted search, free-text search, timeline/map views
- [ ] Library stats (documents, ontologies, models, graph facts)
- [ ] CML enrichment (`wellfair_build_cml_context`, `wellfair_build_cof_package`, `wellfair_enrich_library_cml`)
- [ ] Commons sharing (`wellfair_set_library_commons`, `wellfair_library_commons_share_card`)
- [ ] Perception catalogue (`wellfair_seed_perception_library`, `library_seed_perception_assets`)
- [ ] QApp catalog seeding (`wellfair_seed_studio_qapps`, `wellfair_list_qapp_catalog_categories`)
- [ ] Library graph export + view ops (`wellfair_export_library_graph`, `view_*` commands)
- [ ] Person-authored facets at ingest (occurred_at, place_label, lat/lon, project, purpose, sensitivity, section, commons_visibility)
- [ ] Sanctuary gating (reads vault-free; writes vault-gated)

### Phase 4 — Graph + Aura (plan §6 item 4)

- [ ] `graph/` tool family — SPARQL explorer, `quin.statement`, RDF 1.2
- [ ] `aura/` tool family — SHACL inspector, shape list
- [ ] `graph` and `ontology` containers with real bodies
- [ ] Wire to `execute_sparql_query`, `fetch_domain_ontology`, `validate_shacl_shape`

### Phase 5 — Spatial (plan §6 item 5)

- [ ] `gis/` tool family — pin/track on map
- [ ] `map` container — `Render.scene`, hull, `Manifold.project`
- [ ] GPU frame bridge (already exists in desktop)
- [ ] `3d` container — `/gpu-viewport`, `webizen-render`, `.d10`

### Phase 6 — Social / Pulse (plan §6 item 6)

- [ ] `social` container — LWW CRDT live, chat session load/append, chat graph
- [ ] `pulse/` tool family — `pulse.publish` allowlist, telemetry SSE wiring
- [ ] `pulse` container
- [ ] `webrtc` container — stays `present` until consent gate (plan §1 decision 2)

### Phase 7 — Extract vibe/ tool (plan §6 item 7)

- [ ] `vibe/` tool family — extract from console, IDE + REPL
- [ ] `code` container — `poet_eval`, diagnose, `capability.invoke`
- [ ] Wire to `poet_eval`, `poet_gazetteer`, `poet_capabilities`

### Phase 8 — Health / anatomy / listen / vision (plan §6 item 8)

- [ ] `health` container — `calculate_framingham_risk`, clinical risk, Wellfair (consent-gated)
- [ ] `anatomy` container — Wellfair anatomy `RenderScene`, organ percepts, 10D
- [ ] `listen` container — `qualia-audio`, EnCodec, `audio_*` commands
- [ ] `vision` container — `ComputerVision.ahash`, `qualia-vision`, `vision_*` commands
- [ ] `triad` container — q42+p64+d10, articulatory inspector
- [ ] `media` container — kinematics, vision ahash, grapheme, Forge

### Phase 9 — Rights / wallet (plan §6 item 9)

- [ ] `rights` container — deontic logic, DID, fiduciary sign, agreements
- [ ] `wallet` container — identity/key vault, BIP39, balances, vault handshake
- [ ] `finance` container — Black-Scholes, portfolio
- [ ] Wire to `list_agreements`, `create_agreement`, wallet commands, `fetch_wallet_portfolio`

### Phase 10 — Classic link-through (plan §6 item 10)

- [ ] `webview` container — `<q-web-frame>`, capability-gated fetch
- [ ] `portal` container — QApp dispatch, wormhole IRI
- [ ] `slide` container — office presentation
- [ ] Command palette / omnibox
- [ ] Classic route link-through where approved

### Foundational (must precede or run alongside Phase 1)

- [ ] **IntentBus implementation** (P0) — without this, no container can reach any engine
- [ ] **Backend transport** (P0) — WebSocket + fetch + Tauri invoke bridge
- [ ] **Capability detection** (P1) — port `endpoints.rs` so containers can fail closed
- [ ] **Ontology loading** (P0) — build-time N3 → CBOR-LD + WASM loader
- [ ] **Toolbox/tool registration** (P0) — startup initialization of `Registry`
- [ ] **Theme engine** (P1) — port QPrime presets, sanctuary mode, WCAG
- [ ] **Render subsystem** (P1) — port WASM Canvas2D renderer + scene graph
- [ ] **Diagnostics wiring** (P1) — Aura tray, Pulse stream, job center, status polling

---

## 10. Honesty labels (per plan §8)

Every new container and tool must label its honesty. Current qualia-ui placeholders say "awaiting backend wiring" — this is honest but vague. Use the plan's vocabulary:

| Label | Meaning |
|---|---|
| `live` | Reaches a real engine, returns real data, no fake streams |
| `partial` | Some functionality works, some is stubbed — name what is stubbed |
| `present` | UI exists but engine is not wired (e.g. WebRTC until consent gate) |
| `missing` | Not yet implemented — render nothing or a labeled empty state |

**Do not:**
- Use `alert()` as a finished action
- Fake WebRTC/chat streams
- Vendor TinyMCE/CKEditor/Monaco unless explicitly approved
- Use `<<[ s p o g prov ]>>` — Quin construction is `quin.statement`
- Use Ollama
- Create a second HTML mock for a container that should be a real container

---

## 11. Dependency additions required

Current `Cargo.toml` has: `serde`, `serde_json`, `ciborium`, `wasm-bindgen`, `wasm-bindgen-futures`, `js-sys`, `web-sys` (limited features), `wasm-bindgen-test`.

**Required additions:**

| Dependency | Purpose | Notes |
|---|---|---|
| `web-sys` features expansion | `WebSocket`, `WebSocket`, `CanvasRenderingContext2d`, `Clipboard`, `DragEvent`, `MouseEvent`, `WheelEvent`, `KeyboardEvent`, `InputEvent`, `HtmlTextAreaElement`, `HtmlCanvasElement`, `SvgElement`, `MutationObserver`, `Performance`, `Storage` (localStorage) | Many features needed for canvas, transport, telemetry, theme |
| `console_error_panic_hook` | Panic handling in WASM | Standard for WASM apps |
| `gloo-timers` | Async timers for polling | Already used by studio WASM |
| `getrandom` (wasm_js) | Random number generation | For IDs, nonces |
| `uuid` (v4, js) | UUID generation | For intent receipts, job IDs |
| `base64` | Base64 encoding | For asset protocols |
| `chrono` (wasmbind) | Timestamp handling | For provenance, receipts |
| `futures-util` | Async utilities | For stream handling |
| `serde-wasm-bindgen` | Serde-WASM bridge | For Tauri invoke bridge |

**Do NOT add:**
- `qualia-core-db` as a direct dependency (it is native-only; the WASM HyperCanvas reaches it through the daemon, not in-process)
- `webizen-render` as a direct dependency (native-only; reach through Tauri GPU surface commands)
- Any crate that cannot compile to `wasm32-unknown-unknown`

---

## 12. Test coverage gaps

**Current:** 8 test modules covering serialization roundtrips (registry, tool, manifest, ontology, intent bus, social/settings/communications payloads). No tests for:
- Browser shell rendering
- Canvas interactions
- Manifold switching
- IntentBus dispatch (no implementation to test)
- Ontology loading (no loader to test)
- Capability detection (not implemented)

**Required:**
- `wasm-bindgen-test` tests for DOM rendering of each container type
- Tests for manifold switching state
- Tests for IntentBus dispatch routing (once implemented)
- Tests for capability detection fallback (public web vs desktop)
- Tests for ontology loader (once implemented)
- Tests for canvas editor semantics (undo/redo, snapping, resize)
- Tests for theme engine contrast compliance

---

## 13. Settings, configuration, and setup surfaces — what the HyperCanvas settings environment must contain

This section is the exhaustive inventory of settings, configuration, preferences, admin, and setup surfaces across the QualiaDB workspace. It is organised by the settings section structure the HyperCanvas needs to build, with each setting annotated: name, type, default, persistence location, the command(s) that read/write it, and whether a UI surface exists today (and where).

The studio already has a 10-section settings shell (`SettingsSection` enum in `webizen-studio/src/components/settings/types.rs`): Health, Data, Models, People, Privacy, Appearance, Device, Backup, Services, Technical. The desktop has 26 system commands plus 60+ wellfair commands that carry settings. The HyperCanvas needs to subsume all of these into a unified settings environment that works in both the desktop webview and the public-web fallback.

### 13.1 Setup / first-run onboarding

**Source:** `qualia-client-core/src/setup.rs`, `webizen-studio/src/components/onboarding.rs`, desktop commands `get_setup_state`, `complete_setup_step`, `update_setup_profile`, `finish_setup`.

**8 required steps** (must complete before first-run gate opens):

| Step | What it collects | Config fields | qualia-ui state |
|---|---|---|---|
| `welcome` | Greeting, language selection | — | **missing** — no onboarding flow |
| `storage` | Data home directory path, storage quota | `AgentConfig.storage_path`, `AgentConfig.storage_quota_gb` | **missing** |
| `control` | Control & recovery — key vault init, recovery material | `key_vault.json` (encrypted) | **missing** |
| `device` | Machine context — ownership, fleet, user scope, shared setting | `DeviceContext.ownership/machine_fleet/user_scope/shared_setting/notes` | **missing** |
| `inference` | AI instruments — model selection, backend choice | `AgentConfig.inference_backend`, active model | **missing** |
| `relations` | How you're known — preferred name, locale, timezone, interests, preferred ontologies | `SetupProfile.preferred_name/locale/timezone/interests/preferred_ontologies` | **missing** |
| `care` | How it should feel — care priorities, accessibility needs, qapp goals | `SetupProfile.care_priorities/accessibility_needs/qapp_goals` | **missing** |
| `ready` | Completion checkpoint | `SetupState.completed = true` | **missing** |

**6 progressive paths** (never block first-run; surface later in Setup Health / Relations):

| Path | What it collects | qualia-ui state |
|---|---|---|
| `reachability` | Private / mesh / public posture | **missing** |
| `assurance` | Backup destination + verified restore | **missing** |
| `people_connections` | Invites, contacts, groups | **missing** |
| `domains_mail` | Front door, DNS, purpose mailboxes | **missing** |
| `care_records` | Provenance-backed health material | **missing** |
| `peer_agreements` | Multi-party norms once peers exist | **missing** |

**Locale choices** (BCP-47, from onboarding.rs): en-AU, en-GB, en-US, en-NZ, en-CA, en, es, fr, de, it, pt-BR, pt-PT, zh-Hans, zh-Hant, ja, ko, ar, hi, + "Prefer not to say".

**Care priority options:** medications, pathology, sleep, mobility, mental, carers, none_yet.

**Interest options:** health, family, research, local_history, work, civic, creative, environment, technology, justice.

**Gap:** qualia-ui has no onboarding flow at all. The first-run gate, step progression, profile collection, and progressive path surfacing all need to be built.

### 13.2 Core application config (AgentConfig)

**Source:** `qualia-client-core/src/state.rs:64-75`, desktop commands `get_config` / `save_config`.

| Field | Type | Default | Persistence | UI surface | qualia-ui state |
|---|---|---|---|---|---|
| `storage_path` | String | OS default | `config.json` | Settings → Technical | **missing** |
| `storage_quota_gb` | u64 | 10 | `config.json` | Settings → Technical | **missing** |
| `base_connectivity_cost_ilp` | u64 | 5000 | `config.json` | Settings → Technical | **missing** |
| `daemon_host` | String | "127.0.0.1" | `config.json` | Settings → Technical | **missing** |
| `daemon_port` | u16 | 4242 | `config.json` | Settings → Technical | **missing** |
| `inference_backend` | String | "local" | `config.json` | Settings → AI instruments | **missing** |
| `settings_port` | u16 | 8080 | `config.json` | Settings → Technical | **missing** |

### 13.3 User profile & sharing policy

**Source:** `qualia-client-core/src/user_profile.rs`, desktop commands `get_user_profile` / `save_user_profile`.

**UserProfile fields:**

| Field | Type | Default | Persistence | qualia-ui state |
|---|---|---|---|---|
| `display_name` | String | "" | `profile.json` | **missing** |
| `bio` | Option<String> | None | `profile.json` | **missing** |
| `public_did` | String | "" | `profile.json` | **missing** |
| `active_front_door_id` | Option<String> | None | `profile.json` | **missing** |
| `relay_base_url` | Option<String> | None | `profile.json` | **missing** |
| `sharing` | SharingPolicy | defaults below | `profile.json` | **missing** |
| `updated_at` | u64 | 0 | `profile.json` | **missing** |

**SharingPolicy fields** (10 privacy toggles):

| Field | Type | Default | Meaning | qualia-ui state |
|---|---|---|---|---|
| `share_display_name` | bool | true | Show display name to peers | **missing** |
| `share_public_did` | bool | true | Show public DID | **missing** |
| `share_active_model` | bool | false | Show which AI model is active | **missing** |
| `share_llm_outcomes` | bool | false | Share LLM outputs | **missing** |
| `share_ontology_scope` | bool | false | Share ontology scope | **missing** |
| `share_installed_qapps` | bool | false | Share installed QApps list | **missing** |
| `share_daemon_status` | bool | false | Share daemon status | **missing** |
| `allow_group_chat_invites` | bool | true | Accept group chat invites | **missing** |
| `allow_directory_lookup` | bool | true | Allow directory lookup | **missing** |
| `allow_email_invites` | bool | true | Accept email invites | **missing** |

### 13.4 Accessibility preferences

**Source:** `qualia-client-core/src/wellfair/accessibility_prefs.rs` (or `host_state.rs:44-49`), persistence `wellfair/accessibility.json`.

| Field | Type | Default | qualia-ui state |
|---|---|---|---|
| `high_contrast` | bool | false | **missing** |
| `reduced_motion` | bool | false | **missing** — critical for sanctuary mode |
| `text_scale_percent` | u8 | 100 | **missing** |
| `screen_reader_hints` | bool | true | **missing** |

### 13.5 Appearance & theme

**Source:** `webizen-studio/src/theme_engine.rs`, `webizen-studio/src/components/settings_technical.rs`.

**Theme system (QPrime):**

| Setting | Type | Options | qualia-ui state |
|---|---|---|---|
| Theme preset | enum | fiduciary-dark, commons-light, sanctuary, infosphere, human-warmth, twilight-blue, midnight-slate | **missing** — qualia-ui has hardcoded CSS |
| Presentation level | enum P0-P6 | P0 (flat) → P6 (full spatial) | **missing** |
| Spatial grammar | enum | list, board, globe, room, strata, hyperspace | **missing** |
| Spatialize allowed | bool | true/false | **missing** |
| Sanctuary mode (zero motion) | bool | derived from accessibility + theme | **missing** |
| WCAG contrast enforcement | computed | AA minimum | **missing** |
| Shoelace token bridging | computed | CSS custom properties | **missing** |
| Motion timeline | computed | spring physics, selection pulse | **missing** |

### 13.6 AI instruments / model settings

**Source:** `webizen-studio/src/components/settings/model_setup.rs`, `webizen-studio/src/components/settings_technical.rs`, desktop commands `discover_models`, `download_model`, `cancel_download`, `get_active_model`, `set_active_model`, `get_active_downloads`, `run_agent_inference`.

| Setting | Type | Commands | qualia-ui state |
|---|---|---|---|
| Inference backend | String | `get_config`/`save_config` | **missing** |
| Active model | Option<String> | `get_active_model`/`set_active_model` | **missing** |
| Model discovery list | Vec<ModelInfo> | `discover_models` | **missing** |
| Model download URL | String | `download_model` | **missing** |
| Download progress | Vec<ProgressPayload> | `get_active_downloads` | **missing** |
| Download cancellation | — | `cancel_download` | **missing** |
| Model probe (test active model) | AgentQaModelProbe | `agent_qa_test_active_model` | **missing** |
| Ollama status | — | `check_ollama_status` | **missing** |

**Advanced inference settings (NO UI anywhere — CLI/lab only):**

| Setting | Type | Default | Source | qualia-ui state |
|---|---|---|---|---|
| `SamplerConfig.temperature` | f32 | 0.0 (greedy) | `qualia-core-db/src/inference/sampler.rs` | **no UI** |
| `SamplerConfig.top_k` | u32 | 0 | same | **no UI** |
| `SamplerConfig.top_p` | f32 | 1.0 | same | **no UI** |
| `SamplerConfig.repeat_penalty` | f32 | 1.0 | same | **no UI** |
| `SamplerConfig.freq_penalty` | f32 | 0.0 | same | **no UI** |
| `SamplerConfig.presence_penalty` | f32 | 0.0 | same | **no UI** |
| `SamplerConfig.penalty_window` | u32 | 64 | same | **no UI** |
| `SamplerConfig.seed` | u64 | 0 | same | **no UI** |
| `ExecutionToggles.kv_int8` | bool | — | `execution_profile.rs` | **no UI** |
| `ExecutionToggles.resident_decode` | bool | — | same | **no UI** |
| `ExecutionToggles.coop_gemv` | bool | — | same | **no UI** |
| `ExecutionToggles.ffn_fusion` | bool | — | same | **no UI** |
| `ExecutionToggles.ffn_f16` | bool | — | same | **no UI** |
| `ExecutionToggles.ternary_ffn` | bool | — | same | **no UI** |
| LoRA adapter path | PathBuf | — | `qualia-core-db/src/lora/webgpu_lora.rs` | **no UI** |
| Diffusion config (steps, guidance) | — | — | `webizen-runtime` | **no UI** |
| Thermal governor state | Cool/Warm/Critical | — | `orchestrator.rs` | **no UI** (automatic) |

### 13.7 Privacy, sanctuary & security

**Source:** `qualia-client-core/src/wellfair/sanctuary_vault.rs`, desktop commands `wellfair_sanctuary_*`, `wellfair_setup_sanctuary_vault*`, `wellfair_*_decoy_*`.

**Sanctuary vault settings:**

| Setting | Type | Commands | qualia-ui state |
|---|---|---|---|
| Vault lifecycle state | enum (Unconfigured/Locked/Unlocked) | `wellfair_sanctuary_vault_configured`, `wellfair_lock_sanctuary`, `wellfair_unlock_sanctuary` | **missing** |
| Vault setup (PIN, KDF) | — | `wellfair_setup_sanctuary`, `wellfair_setup_sanctuary_vault`, `wellfair_setup_sanctuary_vault_wrapped` | **missing** |
| Keychain wrapping | bool | `wellfair_sanctuary_vault_is_keychain_wrapped` | **missing** |
| Recovery unlock | — | `wellfair_sanctuary_vault_unlock_with_recovery` | **missing** |
| Vault notes | Vec<Note> | `wellfair_sanctuary_vault_add_note`, `wellfair_sanctuary_vault_add_note_in_session`, `wellfair_sanctuary_vault_list_notes` | **missing** |
| Decoy lane (duress PIN) | — | `wellfair_curate_decoy_note`, `wellfair_review_decoy_activity` | **missing** |
| Decoy retention mode | enum (AutoArchive/Manual) | `wellfair_get_decoy_retention_mode`, `wellfair_set_decoy_retention_mode` | **missing** |
| Sanctuary prefs | — | `wellfair_sanctuary_prefs` | **missing** |

**Consent & conduct:**

| Setting | Type | Commands | qualia-ui state |
|---|---|---|---|
| Consent credentials | Vec<ConsentCredential> | `wellfair_*_consent_creds` | **missing** |
| Conduct audit trail | Vec<ConductRecord> | `wellfair_conduct_audit` | **missing** |
| Disclosure chain | — | `wellfair_disclosure_*` | **missing** |
| Data request evaluation | — | `evaluate_data_request` | **missing** |
| Semantic handshake | — | `apply_semantic_handshake` | **missing** |

**Key vault & identity:**

| Setting | Type | Commands | qualia-ui state |
|---|---|---|---|
| Key vault (encrypted) | — | automatic (recovery material) | **no UI** (intentional) |
| Identity plane | IdentityPlaneSnapshot | `get_identity_plane` | **missing** |
| Apparatus devices | Vec<DeviceRecord> | `list_apparatus_devices`, `register_remote_apparatus_device` | **missing** |
| Person public export | PersonPublic | `export_person_public` | **missing** |
| Person transfer bundle | — | `export_person_transfer_bundle`, `import_person_transfer_bundle` | **missing** |
| WebID TLS cert | — | `mint_person_webid_tls_cert` | **missing** |
| Fleet job envelope | — | `accept_fleet_job_envelope` | **missing** |

**Trust store (browser):**

| Setting | Type | Commands | qualia-ui state |
|---|---|---|---|
| Trust anchors (PEM/DID) | Vec<TrustAnchor> | `browser_trust_list`, `browser_trust_add_pem`, `browser_trust_add_did`, `browser_trust_remove`, `browser_trust_set_enabled` | **missing** |
| Trust verdict | — | `browser_trust_verdict` | **missing** |
| Host pinning | — | `browser_trust_pin_host` | **missing** |
| Suggested trust | — | `browser_trust_list_suggested`, `browser_trust_import_suggested` | **missing** |
| Cert override (Windows) | — | `browser_cert_override_*` | **missing** |

**Delegation rules:**

| Setting | Type | Commands | qualia-ui state |
|---|---|---|---|
| Delegation rules | Vec<DelegationRule> | `get_delegation_rules`, `add_delegation_rule` | **missing** |
| Directory actors | Vec<Actor> | `get_directory_actors`, `add_directory_actor` | **missing** |
| Front doors | Vec<FrontDoor> | `get_front_doors`, `generate_front_door` | **missing** |

### 13.8 People, reachability & network

**Source:** `qualia-client-core/src/mail_transport.rs`, `cloudflare.rs`, `qdp_server.rs`, `social_mesh.rs`, desktop commands in `mail.rs`, `social.rs`, `semantic.rs`.

**Mail transport:**

| Setting | Type | Default | Persistence | qualia-ui state |
|---|---|---|---|---|
| SMTP host | String | "" | `mail-transport.json` | **missing** |
| SMTP port | u16 | 587 | `mail-transport.json` | **missing** |
| SMTP username | String | "" | `mail-transport.json` | **missing** |
| SMTP password | String | "" | `mail-transport.json` | **missing** |
| IMAP host | String | "" | `mail-transport.json` | **missing** |
| IMAP port | u16 | 993 | `mail-transport.json` | **missing** |
| IMAP username | String | "" | `mail-transport.json` | **missing** |
| IMAP password | String | "" | `mail-transport.json` | **missing** |
| Mail receiver running | bool | false | runtime | **missing** |
| Talk setup status | — | — | runtime | **missing** |

**Mail domains & purpose inboxes:**

| Setting | Commands | qualia-ui state |
|---|---|---|
| Mail domains list | `list_mail_domains`, `add_mail_domain` | **missing** |
| Mail addresses | `list_mail_addresses`, `mint_purpose_inbox`, `set_mail_address_enabled` | **missing** |
| Purpose inbox presets | `purpose_inbox_presets` | **missing** |
| Relationship addresses | `mint_relationship_address` | **missing** |
| Mail send/fetch/list | `mail_send`, `mail_fetch`, `mail_list`, `mail_get`, `mail_set_read`, `mail_delete` | **missing** |
| Mail DNS forms | `mail_dns_forms` | **missing** |

**Cloudflare:**

| Setting | Type | Persistence | qualia-ui state |
|---|---|---|---|
| CF API token | String | NOT persisted (session-only) | **missing** |
| CF zone ID | String | NOT persisted | **missing** |
| CF zones list | — | runtime | **missing** |
| CF verify token | — | runtime | **missing** |
| CF publish front door | — | runtime | **missing** |
| CF deploy infrastructure | — | runtime | **missing** |
| CF Pages static site | — | runtime | **missing** |

**QDP / front door:**

| Setting | Commands | qualia-ui state |
|---|---|---|
| QDP server start | `start_qdp_server` | **missing** |
| QDP DID resolution | `resolve_qdp_did` | **missing** |
| NS records for DID | `get_ns_records_for_did` | **missing** |
| Front door forms | `front_door_forms` | **missing** |
| Front door invite | `generate_front_door_invite` | **missing** |

**Mesh networking:**

| Setting | Commands | qualia-ui state |
|---|---|---|
| Mesh start/stop/status | `mesh_start`, `mesh_stop`, `mesh_status` | **missing** |
| Mesh dialability | `mesh_dialability` | **missing** |
| Social peer endpoint | `set_social_peer_endpoint` | **missing** |
| Social peers list | `list_social_peers`, `set_social_peer_active` | **missing** |

**Advanced network (NO UI — CLI/automatic):**

| Setting | Type | qualia-ui state |
|---|---|---|
| Nym relay active | AtomicBool | **no UI** (automatic) |
| Stark prover active | AtomicBool | **no UI** (automatic) |
| Simulated solar watts | AtomicU32 | **no UI** (testing only) |
| WebRTC ICE servers | RTCConfiguration | **no UI** (hardcoded) |
| Companion gateway port | — | **no UI** (automatic) |
| eBPF filter/firewall | — | **no UI** (engine-internal) |
| Acoustic BLE mesh | — | **no UI** (experimental) |
| Sonic token | — | **no UI** (experimental) |

**Solid pod:**

| Setting | Type | Persistence | qualia-ui state |
|---|---|---|---|
| Solid listen address | SocketAddr | env vars only | **no UI** |
| Solid data root | PathBuf | env vars only | **no UI** |
| Solid public base | String | env vars only | **no UI** |
| Solid demo OIDC | bool | env vars only | **no UI** |
| Sync to/from/put to pod | — | runtime | **missing** |

### 13.9 QPU / quantum oracle settings

**Source:** `qualia-client-core/src/qpu_oracle.rs`, desktop commands `get_qpu_settings`, `save_qpu_settings`, `enable_qpu_feature`, `disable_qpu_feature`, `activate_advanced_capabilities`, `get_advanced_activation_status`, `get_commitment_prompt`.

| Setting | Type | Default | qualia-ui state |
|---|---|---|---|
| Feature unlocked | bool | false (requires human rights commitment) | **missing** |
| Commitment affirmed at | Option<String> | None | **missing** |
| IBM token (encrypted) | String | "" | **missing** |
| D-Wave token (encrypted) | String | "" | **missing** |
| IonQ token (encrypted) | String | "" | **missing** |
| Rigetti token (encrypted) | String | "" | **missing** |
| Azure credentials (encrypted) | String | "" | **missing** |
| Braket credentials (encrypted) | String | "" | **missing** |
| Google token (encrypted) | String | "" | **missing** |
| Quantinuum token (encrypted) | String | "" | **missing** |
| Max shots per task | u32 | — | **missing** |
| Fallback to classical | bool | — | **missing** |
| Enable QUBO routing | bool | — | **missing** |
| Enable DFT ground state | bool | — | **missing** |
| Enable defeasible resolution | bool | — | **missing** |
| GSR enabled | bool | — | **missing** |
| GSR solver preference | String | — | **missing** |
| GSR timeout seconds | u32 | — | **missing** |
| GSR max parallel jobs | u32 | — | **missing** |
| Per-provider minutes used | u32 each | 0 | **missing** |

### 13.10 Services, daemon & updates

**Source:** `webizen-desktop/src/supervisor.rs`, `updater_service.rs`, `settings_server.rs`, `mcp_server.rs`, desktop commands `start_daemon`, `daemon_status`, `get_active_daemon_port`, `qualia_protocol_port`, `run_engine_command`, `get_supervisor_state`, `updater_*`.

| Setting | Type | Commands | qualia-ui state |
|---|---|---|---|
| Daemon running | bool | `daemon_status`, `start_daemon` | **missing** |
| Daemon port | u16 | `get_active_daemon_port` | **missing** |
| Qualia protocol port | u16 | `qualia_protocol_port` | **missing** |
| Engine command | String | `run_engine_command` | **missing** |
| Supervisor service snapshots | Vec<ServiceSnapshot> | `get_supervisor_state` | **missing** |
| Supervisor operation snapshots | Vec<OperationSnapshot> | `get_supervisor_state` | **missing** |
| Settings server port | u16 (8080) | `AgentConfig.settings_port` | **missing** |
| MCP server port | 4245 | runtime | **no UI** |
| Update check | — | `updater_check` | **missing** |
| Update download & install | — | `updater_download_and_install` | **missing** |
| Update restart | — | `updater_restart` | **missing** |

### 13.11 Data, storage & memory

**Source:** `webizen-studio/src/components/q42_volume_manager.rs`, `wal_inspector.rs`, `storage_driver_config.rs`, desktop commands `list_q42_volumes`, `inspect_q42_volume`, `verify_q42_volume`, `magnet_q42_volume`, `compact_q42_volume`.

| Setting | Type | Commands | qualia-ui state |
|---|---|---|---|
| Storage path | String | `get_config`/`save_config` | **missing** |
| Storage quota | u64 GB | `get_config`/`save_config` | **missing** |
| Q42 volume list | Vec<Volume> | `list_q42_volumes` | **missing** |
| Q42 volume inspect | — | `inspect_q42_volume` | **missing** |
| Q42 volume verify | — | `verify_q42_volume` | **missing** |
| Q42 volume magnet | — | `magnet_q42_volume` | **missing** |
| Q42 volume compact | — | `compact_q42_volume` | **missing** |
| WAL inspector | — | (studio component) | **missing** |
| Storage driver config | — | (studio component) | **missing** |

### 13.12 Hardware & device

**Source:** `webizen-desktop/src/commands/system.rs`, `webizen-studio/src/components/hardware_configurator.rs`, `health_monitor.rs`.

| Setting | Type | Commands | qualia-ui state |
|---|---|---|---|
| Hardware status | HardwareStatus | `get_hardware_status` | **missing** |
| Energy circumstance | String | `profile_energy_circumstance` | **missing** |
| GPU device selection | — | runtime | **missing** |
| GPU memory budget | — | runtime | **missing** |
| GPU feature flags | — | runtime | **missing** |
| CPU utilization | f64 | telemetry | **missing** |
| RAM usage | f64 MB | telemetry | **missing** |
| Network congestion | — | telemetry | **missing** |
| Apparatus device list | Vec<DeviceRecord> | `list_apparatus_devices` | **missing** |
| Remote device registration | — | `register_remote_apparatus_device` | **missing** |
| Local control base URL | String | `set_local_control_base_url` | **missing** |
| Remote job outbox | — | `list_remote_job_outbox`, `retry_remote_job_outbox` | **missing** |
| Job device placement | — | `resolve_job_device_placement`, `schedule_job_on_device` | **missing** |

### 13.13 Medication & health preferences

**Source:** `webizen-desktop/src/commands/wellfair/med_reminder.rs`, `medication.rs`.

| Setting | Type | Commands | qualia-ui state |
|---|---|---|---|
| Med reminder prefs | — | `wellfair_med_reminder_prefs` | **missing** |
| Med reminder permission | — | `wellfair_grant_med_reminder_permission` | **missing** |
| Med reminders enabled | bool | `wellfair_set_med_reminders_enabled` | **missing** |
| Due med reminders | Vec<Reminder> | `wellfair_list_due_med_reminders` | **missing** |
| Graph coverage query | — | `wellfair_query_graph_coverage` | **missing** |
| Medication management | — | `wellfair_*_medication_*` | **missing** |
| Diet tracking | — | `wellfair_*_diet_*` | **missing** |
| Sleep tracking | — | `wellfair_*_sleep_*` | **missing** |
| Emergency contacts | — | `wellfair_*_emergency_contacts` | **missing** |

### 13.14 Browser engine & trust

**Source:** `webizen-desktop/src/browser/engine.rs`, `cert_override.rs`, `cookies.rs`.

| Setting | Type | Commands | qualia-ui state |
|---|---|---|---|
| Browser engine preference | enum (OS WebView / Servo) | `browser_engine_status`, `browser_set_engine` | **missing** |
| Cert override enabled | bool | `browser_cert_override_set_enabled` | **missing** |
| Cert override attached | — | `browser_cert_override_attach` | **missing** |
| Cookie observation | — | `browser_cookie_observe`, `browser_cookie_summary`, `browser_cookies_refresh` | **missing** |
| Clear site data | — | `browser_clear_site_data` | **missing** |
| Browser agent TLS | — | `browser_agent_tls_status` | **missing** |

### 13.15 Vision, audio & render settings

**Source:** `qualia-vision`, `qualia-audio`, `webizen-render`, `webizen-runtime`, desktop commands `vision_*`, `audio_*`, `audio_capabilities`.

| Setting | Type | Commands | qualia-ui state |
|---|---|---|---|
| Audio capabilities | — | `audio_capabilities` | **missing** |
| Mic capture status | — | `audio_mic_start`, `audio_mic_stop`, `audio_mic_status` | **missing** |
| Mic capture policy | CapturePurpose | runtime | **no UI** (automatic) |
| Vision weights path | — | `vision_ensure_weights` | **missing** |
| Audio weights path | — | `audio_ensure_weights` | **missing** |
| WAV import path | — | `audio_pick_wav_path`, `audio_import_wav` | **missing** |
| Render loop toggle | bool | `toggle_render_loop` | **missing** |
| Render preview | — | `update_render_preview` | **missing** |
| GPU surface mount | — | `mount_gpu_surface`, `unmount_gpu_surface` | **missing** |
| GPU scene | — | `set_gpu_scene` | **missing** |
| GPU camera | — | `set_gpu_camera`, `set_gpu_camera_mode` | **missing** |
| GPU mesh upload | — | `upload_gpu_mesh`, `upload_gpu_mesh_colored`, `upload_gpu_10d_mesh` | **missing** |
| 10D asset load | — | `load_gpu_10d_asset` | **missing** |
| Diffusion config | DiffusionConfig | `reconfigure_diffusion` | **missing** |
| Diffusion snapshot | — | `get_latest_diffusion_snapshot` | **missing** |
| Diffusion frame RGBA | — | `get_diffusion_frame_rgba` | **missing** |
| Diffusion ledger health | — | `get_diffusion_ledger_health` | **missing** |

### 13.16 Telemetry & diagnostics

**Source:** `webizen-desktop/src/telemetry_bridge.rs`, `telemetry_hooks.rs`, `desktop_log.rs`, desktop commands `collapse_wavefunction`, `set_temporal_slice`, `register_browser_capabilities`, `get_desktop_logs`, `set_desktop_debug_mode`, `report_client_error`, `agent_qa_snapshot`.

| Setting | Type | Commands | qualia-ui state |
|---|---|---|---|
| Desktop logs | Vec<LogEntry> | `get_desktop_logs` | **missing** |
| Debug mode | bool | `set_desktop_debug_mode` | **missing** |
| Agent QA snapshot | AgentQaSnapshot | `agent_qa_snapshot` | **missing** |
| Wavefunction collapse | — | `collapse_wavefunction` | **missing** |
| Temporal slice | — | `set_temporal_slice` | **missing** |
| Browser capabilities | — | `register_browser_capabilities` | **missing** |
| Client error reporting | — | `report_client_error` | **missing** |
| CCF assets | — | `list_ccf_assets`, `load_ccf_asset` | **missing** |
| Smoke tests | — | `test_ccf_ipc_handshake`, `test_larynx_smoke`, `test_vasculature_stress` | **missing** |

### 13.17 Wallet & finance

**Source:** `webizen-desktop/src/commands/wallet.rs`, `wellfair/finance.rs`, `ledger.rs`.

| Setting | Type | Commands | qualia-ui state |
|---|---|---|---|
| Wallet status | — | `get_wallet_status` | **missing** |
| First run | bool | `is_first_run` | **missing** |
| Identity read/save | — | `read_identity`, `save_identity`, `load_identity` | **missing** |
| BIP39 seed generation | — | `generate_bip39_seed` | **missing** |
| Wallet derivation | — | `derive_wallets_from_seed` | **missing** |
| External seed import | — | `import_external_seed` | **missing** |
| Coin balances | — | `get_coin_balances` | **missing** |
| Transaction history | — | `get_transaction_history` | **missing** |
| Token management | — | `get_tokens`, `add_token`, `remove_token` | **missing** |
| Tax suite | TaxRecipientSuite | `get_tax_suite`, `save_tax_suite`, `dispatch_tax_payment` | **missing** |
| Send XEC | — | `build_send_xec`, `confirm_send_xec`, `send_ecash_token` | **missing** |
| Vault handshake | — | `accept_vault_handshake`, `receive_vault_job` | **missing** |
| Ledger entries | — | `wellfair_ledger_*` | **missing** |
| Finance balance | — | `wellfair_finance_*` | **missing** |
| Wallet portfolio | — | `fetch_wallet_portfolio` | **missing** |
| Semantic token mint | — | `mint_semantic_token` | **missing** |

### 13.18 Agreements, cooperative & governance

**Source:** `qualia-cooperative-core`, desktop commands `list_agreements`, `create_agreement`, `save_agreement`, `set_agreement_consent`, `list_coop_projects`, `create_coop_project`, `add_project_collaborator`, `coop_share_package`.

| Setting | Type | Commands | qualia-ui state |
|---|---|---|---|
| Agreements list | Vec<Agreement> | `list_agreements`, `agreements_for` | **missing** |
| Create agreement | — | `create_agreement`, `save_agreement` | **missing** |
| Agreement consent | — | `set_agreement_consent` | **missing** |
| Cooperative projects | Vec<Project> | `list_coop_projects`, `create_coop_project` | **missing** |
| Project collaborators | — | `list_project_collaborators`, `add_project_collaborator`, `remove_project_collaborator` | **missing** |
| Coop share package | — | `coop_share_package`, `accept_coop_share_package` | **missing** |
| Project group chat | — | `create_project_group_chat` | **missing** |
| Guardianship proposals | — | `wellfair_*_guardianship_*` | **missing** |
| Safeguard (dead man's switch) | — | `wellfair_*_safeguard_*` | **missing** |

### 13.19 Settings with NO UI anywhere in the workspace

These settings exist in code but have no UI surface in studio, desktop, or qualia-ui. They are candidates for a new "Advanced / Developer" settings section.

| Setting | Source | Why it lacks UI |
|---|---|---|
| `SamplerConfig` (temperature, top_k, top_p, penalties, seed) | `qualia-core-db/src/inference/sampler.rs` | CLI/API only |
| `ExecutionProfile` (toggles, metrics, representation, objectives) | `qualia-core-db/src/q42/execution_profile.rs` | Inference lab only |
| `ExecutionToggles` (kv_int8, resident_decode, coop_gemv, ffn_fusion, ffn_f16, ternary_ffn) | `execution_profile.rs` | Inference lab only |
| `BridgeConfig` (Solid listen, data_root, public_base, demo_oidc) | `qualia-solid-bridge/src/solid_proxy.rs` | Environment variables only |
| `RopeConfig` (rotary embedding) | `qualia-core-db/src/wgsl_forge/graph_ops/stencil.rs` | Engine-internal |
| `ReliabilityConfig` (Monte Carlo) | `engineering_analysis/reliability.rs` | Tests only |
| `CampaignConfig` (optimization campaign) | `inference/lab/campaign.rs` | Lab only |
| `AutoImproveConfig` (auto-tuning) | `inference/lab/auto_improve.rs` | Lab only |
| `RawDecodeConfig` (benchmark) | `inference/inference_bench/raw_decode/config.rs` | CLI only |
| `BoundaryLayerConfig` (CFD) | `physics_simulation/discretization.rs` | Engine-internal |
| `VerificationAlgorithmConfig` (crypto) | `cryptographic_library/signing.rs` | Engine-internal |
| `SimulationConfig` (chemistry) | `chemistry_modeling/simulation.rs` | Engine-internal |
| `ConfigurationManager` (BLE mesh) | `net/acoustic_ble_mesh/ble.rs` | Engine-internal |
| `SignalProcessingConfig` (acoustic) | `net/acoustic_ble_mesh/acoustic.rs` | Engine-internal |
| `SolverConfig` (CFD solver) | `engineering_analysis/cfd.rs` | Engine-internal |
| `CanvasWorldConfig` (canvas projection) | `qualia-client-core/src/canvas_world.rs` | Engine-internal |
| `ChatEnvironmentConfig` (chat session) | `qualia-client-core/src/context_binding.rs` | Runtime |
| `ParticipantAgentConfig` (sub-agent binding) | `qualia-client-core/src/chat_agents.rs` | Runtime |
| `QappWasmConfig`, `QappDaemonConfig`, `QappChatIntegrationConfig`, `QappPinnModelConfig` | `qualia-client-core/src/qapp_registry.rs` | qApp manifests only |
| Thermal governor state | `orchestrator.rs` | Automatic |
| MCP server tool registration | `mcp_server.rs` | API only |
| WebRTC ICE servers | `webrtc_manager.rs` | Hardcoded |
| Stark prover state | `state.rs` | Automatic |
| Nym relay state | `state.rs` | Automatic |
| Simulated solar watts | `state.rs` | Testing only |
| Key vault internal keys | `key_vault.rs` | Recovery material only |
| eBPF filter/firewall | `net/ebpf.rs` | Engine-internal |
| Acoustic BLE mesh | `net/acoustic_ble_mesh/` | Experimental |
| Sonic token | `net/sonic_token.rs` | Experimental |
| Deontic circuit | `crypto/deontic_circuit.rs` | Experimental (zk-culling) |
| Post-quantum KEM shim | `crypto/pq_kem_shim.rs` | Experimental (pq-kem) |
| Sanctuary crypto | `crypto/sanctuary_crypto.rs` | Experimental (sanctuary-crypto) |
| LoRA adapter path | `lora/webgpu_lora.rs` | CLI only |
| Diffusion config (steps, guidance) | `webizen-runtime` | CLI only |

### 13.20 Recommended settings section structure for qualia-ui

Based on the inventory above, the HyperCanvas settings environment should have these sections (mapping to the studio's 10 sections + additions for gaps):

| Section | What it contains | Studio equivalent | New for qualia-ui |
|---|---|---|---|
| **Setup health** | First-run gate status, progressive paths, setup completion | Settings → Health | Onboarding wizard + setup health dashboard |
| **Data & memory** | Storage path, quota, Q42 volumes, WAL inspector, storage drivers | Settings → Data | + Q42 volume browser (no UI today) |
| **AI instruments** | Inference backend, active model, model discovery/download, sampler config, execution toggles, LoRA, diffusion | Settings → Models | + Advanced inference settings (no UI today) |
| **People & reachability** | User profile, sharing policy, mail transport, domains, Cloudflare, QDP, mesh, social peers, Solid pod | Settings → People | + Solid pod config (env-only today) |
| **Privacy & sanctuary** | Sanctuary vault, consent credentials, conduct audit, disclosure, decoy retention, delegation rules, trust store, cert override | Settings → Privacy | + Consent management UI (planned) |
| **Appearance & access** | Theme presets, presentation level, spatial grammar, accessibility, WCAG, motion, Shoelace tokens | Settings → Appearance | Full theme engine port |
| **Person & devices** | Identity plane, apparatus devices, fleet jobs, transfer bundles, WebID TLS cert, hardware status | Settings → Device | + Hardware configurator |
| **Backup & recovery** | Backup destination, verified restore, sync relay, export/import | Settings → Backup | + Sync diagnostics |
| **Services & updates** | Daemon control, supervisor state, MCP server, settings server, updater, logs, debug mode | Settings → Services | + MCP inspector (no UI today) |
| **Technical** | Raw config (AgentConfig), QPU oracle, browser engine, telemetry, diagnostics, agent QA | Settings → Technical | + Advanced/developer settings (33 settings with no UI today — see §13.19) |
| **Wallet & finance** *(new)* | BIP39 wallet, balances, tokens, tax suite, XEC sending, vault handshake, ledger, portfolio | (scattered across studio) | Unified wallet settings |
| **Agreements & governance** *(new)* | Agreements, cooperative projects, collaborators, guardianship, safeguards, deontic norms | (scattered across studio) | Unified governance settings |
| **Health preferences** *(new)* | Medication reminders, permissions, diet, sleep, emergency contacts, care priorities | (scattered across wellfair) | Unified health preferences |
| **Browser & trust** *(new)* | Browser engine, trust store, cert override, cookies, site data, agent TLS | (scattered across browser) | Unified browser settings |

---

## 14. Dark capabilities — engine features without UI support

This section catalogues engine capabilities that exist in the QualiaDB workspace but have **no UI surface** in studio, desktop, or qualia-ui. Many are intentionally internal (engine kernels, platform bridges, build-time tools); others are gaps where a UI surface would be valuable.

### 14.1 Summary statistics

| Category | Total features | Dark (no UI) | Dark % |
|---|---|---|---|
| qualia-core-db public API | 200+ | 80+ | 40% |
| Tauri commands | 500+ | 200+ | 40% |
| CLI commands | 50+ | 50+ | 100% |
| Extensions | 5 | 5 | 100% |
| Inference kernel | 1 | 1 | 100% |
| Q-Forge | 2 | 2 | 100% |
| Solid bridge | 6 | 6 | 100% |
| Mobile harness | 3 | 3 | 100% |
| Render | 10 | 4 | 40% |
| Audio | 40+ | 35+ | 87% |
| Vision | 50+ | 45+ | 90% |
| Wellfair core | 25 | 15 | 60% |
| Cooperative core | 8 | 5 | 62% |
| Poet vibe | 14 | 13 | 93% |
| Semantic library | 7 | 7 | 100% |
| Webizen web | 2 | 2 | 100% |
| Webizen runtime | 15 | 15 | 100% |
| **Total** | ~900+ | ~500+ | ~55% |

### 14.2 Capability descriptors with no UI

From `CAPABILITY_DESCRIPTORS` in `qualia-core-db/src/lib.rs:552-1050`:

| Capability | Surfaces | Gap |
|---|---|---|
| SymbolicAlgebra | native, mcp | No desktop/studio UI |
| NumericalCalculus | native, library, webizen | No direct panel |
| Optimization | native, library | No UI |
| NumberTheory | native, library | No UI |
| SpecialFunctionsAndTransforms | native, library | No UI |
| MachineLearning | native, library | Marked "fail-closed" — no UI |
| EngineeringAnalysis | native, mcp | No studio UI |
| CausalFuzzyAndControl | native, library, webizen | Limited desktop UI |

### 14.3 qualia-core-db dark modules (CLI/engine-internal)

| Module | Features | Why dark |
|---|---|---|
| `chat_relay_daemon` | Chat relay | CLI-only daemon |
| `daemon_swarm` | Swarm daemon | CLI-only |
| `ilp_dispatcher` | ILP dispatch | Engine-internal |
| `webtorrent_routes` / `webtorrent_seeder` | WebTorrent | CLI-only |
| `comorbidity_eval` | Comorbidity evaluation | CLI-only |
| `dicom` / `dicom_ingest` | DICOM processing | CLI-only — no desktop DICOM UI |
| `device_benchmark` | Device benchmarking | CLI-only |
| `hardware_passport` | Hardware passport | CLI-only |
| `jni_bridge` | JNI bridge | Android-only |
| `npu_ffi` / `tee_ffi` | NPU/TEE FFI | Engine-internal |
| `platform_scheduler` | Platform scheduler | Engine-internal |
| `ambient_orchestration` | Ambient orchestration | Engine-internal |
| `cuda_lane::*` | CUDA operations | CLI/lab-only |
| `directml_bridge` | DirectML | Windows-specific, no UI |
| `inference_bench` | Benchmarking | CLI-only |
| `inference_gpu_profiler` | GPU profiling | CLI/lab-only |
| `inference_path_selector` | Path selection | CLI-only |
| `kv_capture` | KV capture | Engine-internal |
| `lab::*` | Lab/experimentation | CLI-only |
| `metal_bridge` | Metal | macOS-specific, no UI |
| `residency_planner` | Residency planning | Engine-internal |
| `ternary_gpu` / `topk_gpu` | Ternary/Top-K GPU | Experimental, no UI |
| `thermal_telemetry` | Thermal telemetry | CLI-only |
| `key_vault` | Key vault | CLI-only (recovery material) |
| `profiles` | Identity profiles | Engine-internal |
| `topology_draft` | Topology drafting | Engine-internal |
| `acoustic_ble_mesh` | Acoustic BLE mesh | Experimental |
| `ebpf_filter` / `ebpf_firewall` | eBPF | Engine-internal |
| `nym_adapter` | Nym adapter | Experimental |
| `sonic_token` | Sonic token | Experimental |
| `web_civics` | Web civics | CLI-only |
| `webizen_sync` | Webizen sync | Engine-internal |
| `deontic_circuit` | Deontic circuits | Experimental (zk-culling) |
| `pq_kem_shim` | Post-quantum KEM | Experimental (pq-kem) |
| `sanctuary_crypto` | Sanctuary crypto | Experimental (sanctuary-crypto) |

### 14.4 Tauri commands with no studio component caller

**System / fleet** (9 commands): `list_apparatus_devices`, `register_remote_apparatus_device`, `resolve_job_device_placement`, `schedule_job_on_device`, `set_local_control_base_url`, `list_remote_job_outbox`, `retry_remote_job_outbox`, `mint_person_webid_tls_cert`, `accept_fleet_job_envelope`.

**Q42 volumes** (5 commands): `list_q42_volumes`, `inspect_q42_volume`, `verify_q42_volume`, `magnet_q42_volume`, `compact_q42_volume`.

**Semantic / advanced** (8 commands): `generate_front_door_invite`, `mint_semantic_token`, `fetch_wallet_portfolio`, `toggle_nym_relay`, `toggle_stark_prover`, `update_solar_input`, `fetch_torrent_telemetry`, `fetch_remote_manifest`.

**Directory / delegation** (6 commands): `get_front_doors`, `generate_front_door`, `get_directory_actors`, `add_directory_actor`, `get_delegation_rules`, `add_delegation_rule`.

**Mail / cooperative / agreements** (21 commands): `mail_dns_forms`, `mail_receiver_status`, `mail_receiver_start`, `mail_receiver_stop`, `generate_connection_identifier`, `answer_connection_challenge`, `mesh_dialability`, `list_project_collaborators`, `list_coop_projects`, `create_coop_project`, `add_project_collaborator`, `remove_project_collaborator`, `coop_share_package`, `accept_coop_share_package`, `create_project_group_chat`, `set_social_peer_endpoint`, `list_agreements`, `agreements_for`, `create_agreement`, `save_agreement`, `set_agreement_consent`.

**Telemetry / testing** (9 commands): `collapse_wavefunction`, `collapse_wavefunction_legacy`, `set_temporal_slice`, `register_browser_capabilities`, `load_ccf_asset`, `list_ccf_assets`, `test_ccf_ipc_handshake`, `test_larynx_smoke`, `test_vasculature_stress`.

**Native scientific bindings** (4 commands): `calculate_chemistry_properties`, `calculate_framingham_risk`, `calculate_quantum_dft`, `calculate_monte_carlo_var`.

**Browser 10D** (6 commands): `browse_10d_containers`, `browse_vision_10d`, `load_vision_10d`, `scrub_vision_10d_paint`, `inspect_10d_container`, `open_10d_file_picker`.

**Vision / audio** (37 commands): All `vision_*` and `audio_*` demo/test commands — see §3.9 of this report for the full list.

**QApp host / export / telemetry** (8 commands): `certify_forge_physics`, `run_forge_compute_probe`, `get_qualia_compute_profile`, `export_qapp_as_wasm_package`, `wellfair_get_model_lifecycle_status`, `wellfair_force_model_lifecycle_phase`, `wellfair_get_llm_telemetry`, `submit_record`.

**Render** (3 commands): `toggle_render_loop`, `navigate_to_node`, `select_node_at`.

**Mesh** (3 commands): `mesh_start`, `mesh_stop`, `mesh_status`.

**Personal directory** (5 commands): `list_directory`, `list_directory_categories`, `create_directory_category`, `set_directory_entry_categories`, `search_directory`.

**Entity view API** (16 commands): `view_session`, `view_set_observer`, `view_set_presentation_level`, `view_project_library`, `view_project_web_locus`, `view_morph`, `view_pick_scene`, `view_select`, `view_select_uri`, `view_clear_selection`, `view_bifurcate_package`, `view_capability_report`, `view_render_memory_spatial`, `view_remote_controller_info`, `view_set_circumstance`, `shell_navigate`.

**Agent QA** (2 commands): `agent_qa_snapshot`, `agent_qa_test_active_model`.

**Poet** (5 commands): `poet_eval`, `poet_reset`, `poet_gazetteer`, `poet_capabilities`, `poet_render_preview`.

**Semantic logic** (4 commands): `execute_sparql_query`, `fetch_domain_ontology`, `validate_shacl_shape`, `evaluate_logic_rules`.

**Total dark Tauri commands: ~145** (out of 500+).

### 14.5 CLI-only features (qualia-cli, 50+ commands)

`q42_comparative_bench`, `shader` (WGSL compilation/certification), `MeshProbe`, `Shacl`, `Vault`, `Migrate`, `Mem`, `Inspect`, `Dump`, `ExportSolid`, `Solid`, `Ingest`, `VerifyIntegrity`, `VerifyGraph`, `Import`, `IngestJob`, `Query`, `Compress`, `Resources`, `Profile`, `Benchmark`, `Bench`, `Qpu`, `Extension`, `Evaluate`, `Solve`, `Science`, `Governance`, `Compile`.

### 14.6 Extension / harvester / inference kernel / Q-Forge / Solid / mobile dark features

| Crate | Dark features | Why |
|---|---|---|
| `qualia-extensions` | `ExtensionManager`, `pinn_extension`, `qpu_extension`, `snn_extension`, `webgpu_extension` | FFI bridge only, no direct UI |
| `webizen-component-harvester` | Shoelace generator, Dioxus component generation | Build-time only |
| `qualia-inference-kernel` | `paged_attention` | Low-level contract |
| `qualia-q-forge` | `qasm` (QASM parsing), `sim` (quantum simulation) | Experimental |
| `qualia-solid-bridge` | `consumer::*`, `ldp_translator`, `oidc_micro_idp`, `pod_store`, `solid_proxy` | CLI-only |
| `qualia-mobile-harness` | Companion pairing, Samsung Health CSV, WebSocket health sync | Mobile-only |

### 14.7 Audio dark features (35+ of 40+)

`aed_weights`, `aqa` (audio QA), `capture::*`, `convert::*`, `cross_modal::*`, `events::*`, `features::*`, `fx::*`, `generation::*`, `midi`, `music::*`, `pipeline::*`, `production::*`, `semantic::*`, `session_history`, `sonify`, `speech::*`.

### 14.8 Vision dark features (45+ of 50+)

`bio::*`, `biosense::*`, `classifier::*`, `cv::*`, `detector::*`, `embeddings::*`, `generator::*`, `gpu::*`, `metrics::*`, `ops::*`, `overlay::*`, `preprocess::*`, `recipes::*`, `semantic::*`, `spatial::*`, `sr::*` (super-resolution), `synthetic::*`, `tracker::*`, `weights::*`, taxonomy modules (ADE20K, CIFAR, Cityscapes, COCO, KITTI, OpenImages, Pascal VOC, VGGFace2).

### 14.9 Wellfair core dark features (15 of 25)

`authority_attestation`, `companion_pairing`, `companion_sync`, `conditions`, `live_share`, `models`, `n3_rules`, `parser`, `personal_records`, `qualia_bindings`, `rdf`, `shapes`, `sleep_analytics`, `store`, `webizen`.

### 14.10 Cooperative core dark features (5 of 8)

`agency_domain`, `authority_type`, `provenance`, `qapp_package`, `taxonomy`, `trigger`.

### 14.11 Highest-value UI gaps (recommended for surfacing)

These are the dark capabilities where a UI surface would deliver the most user value:

| Priority | Capability | Current surface | Recommended UI |
|---|---|---|---|
| P0 | Q42 volume management | CLI-only | Settings → Data & memory → Q42 volume browser |
| P0 | Scientific calculations (chemistry, Framingham, DFT, Monte Carlo) | CLI-only | `sheet`/`latex` containers with native_bindings invoke |
| P0 | SPARQL / SHACL / logic rules | CLI-only | `graph` + `ontology` + `aura` containers |
| P0 | Poet eval / gazetteer / capabilities | CLI-only | `vibe` tool family in `code` container |
| P1 | Q42 comparative benchmark | CLI-only | Settings → Technical → Benchmark panel |
| P1 | Model lifecycle telemetry | CLI-only | Settings → AI instruments → Lifecycle panel |
| P1 | Forge physics certification | CLI-only | `sheet`/`code` container → Forge probe |
| P1 | Entity view API (morph, pick, bifurcate, capability report) | Partial | Canvas editor integration |
| P1 | Cooperative projects / agreements | CLI-only | `rights` container + cooperative toolbox |
| P1 | Personal directory / categories | CLI-only | `rights`/`wallet` container |
| P2 | Advanced vision operations (classifier, detector, tracker, SR) | Engine-internal | `vision` container + `image` toolbox |
| P2 | Advanced audio operations (fx, generation, music, speech, MIDI) | Engine-internal | `listen` container + `audio` toolbox |
| P2 | Extension manager | FFI only | Settings → Technical → Extensions |
| P2 | Solid pod operations | CLI-only | Settings → People → Solid pod |
| P2 | Mesh networking control | CLI-only | Settings → People → Reachability |
| P2 | Nym relay / Stark prover toggles | Automatic | Settings → Technical → Network |
| P2 | Thermal governor / GPU profiler | CLI-only | Settings → Technical → Performance |
| P2 | eBPF filter/firewall | Engine-internal | Settings → Technical → Network (advanced) |
| P3 | DICOM processing/ingest | CLI-only | `health`/`anatomy` container |
| P3 | WebTorrent seeding | CLI-only | `media` container |
| P3 | Acoustic BLE mesh | Experimental | Settings → Technical → Experimental |
| P3 | Post-quantum KEM / deontic circuit / sanctuary crypto | Experimental | Settings → Technical → Experimental |

### 14.12 Intentionally dark (keep CLI/engine-internal)

These should NOT get a UI surface — they are intentionally internal:

- Low-level kernels: `webizen-runtime` (diffusion kernel, frame buffer, ledger sink), `qualia-inference-kernel` (paged attention)
- Platform bridges: `cuda_lane`, `metal_bridge`, `directml_bridge`, `jni_bridge`, `npu_ffi`, `tee_ffi`
- Build-time tools: `webizen-component-harvester`, `qualia-semantic-library` (offline dev tool)
- Engine internals: `ilp_dispatcher`, `platform_scheduler`, `residency_planner`, `kv_capture`, `ambient_orchestration`, `topology_draft`, `resource_catalog`
- Research modules: most `qualia-vision` and `qualia-audio` sub-modules (engine-internal abstractions)
- Experimental crypto: `deontic_circuit`, `pq_kem_shim`, `sanctuary_crypto` (until stable)

---

## 15. Summary

`qualia-ui` is a clean skeleton with the right abstractions but no engine wiring. The desktop and studio crates together expose **500+ Tauri commands** (including 27 for the hypermedia library alone), **40+ routed surfaces**, **70+ wellfair modules**, **300+ academic QApps**, **30+ specialized tools**, a dedicated `qualia-semantic-library` crate (`.hmc` container ingest, embedding manifold, dedup/search/novelty), and **45+ config structs** across **10+ persistence files**. None of that is reachable from `qualia-ui` today.

The HyperCanvas plan defines exactly **24 containers** (plus a recommended `library`/lived-memory addition, see §3.8b), **10 toolboxes**, and **9 tool families** that should subsume the Classic surfaces. `qualia-ui` currently has **4 containers with static content**, **6 placeholders**, and **0 tool families**.

The settings environment must cover **14 settings sections** (§13.20) spanning **45+ config structs**, **8 required setup steps**, **6 progressive setup paths**, **10 sharing/privacy toggles**, **4 accessibility preferences**, **7 theme presets**, **20+ QPU oracle settings**, **8 mail transport fields**, **17+ sanctuary/consent settings**, **15+ wallet/finance settings**, **8+ agreement/governance settings**, **16+ health preferences**, **12+ browser/trust settings**, **17+ vision/audio/render settings**, **10+ telemetry/diagnostic settings**, and **33 settings with no UI anywhere** (§13.19).

The dark-capabilities audit identifies **~500+ features** (§14) across 18 crates that have no UI surface. Of these, **~145 Tauri commands** have no studio component caller, **50+ CLI commands** have no desktop equivalent, and **33 config structs** have no settings panel. The highest-value UI gaps are Q42 volume management, scientific calculations, SPARQL/SHACL, Poet eval, and cooperative/agreements management (§14.11).

The critical path is:
1. **IntentBus + backend transport + capability detection** — without these, nothing reaches an engine.
2. **Ontology loading + toolbox registration** — without these, the registry is empty and the toolbox dock is decorative.
3. **Canvas editor semantics (undo/redo, resize, snapping, persistence)** — plan §6 item 1.
4. **Authoring trio (`rich_text`, `sheet`, `latex`)** — plan §6 item 2.
5. **Graph + Aura (`graph`, `ontology`, `aura`)** — plan §6 item 4.
6. **Settings environment** (§13) — onboarding, config, preferences, privacy, sanctuary, services.
7. **Hypermedia library** (§3.8b) — `doc` + `library` containers reaching 27 desktop commands.
8. Then proceed through plan §6 items 5-10 in order.

The desktop host stays the native chrome. The studio Classic routes stay default. `qualia-ui` is the Poet HyperCanvas that *reaches* the same engines through a typed, CBOR-LD, capability-gated intent bus — and it is the surface that eventually subsumes the Classic routes as containers, per the plan §5 mapping.

---

*Generated with [Devin](https://devin.ai) for Timothy Charles Holborn. Audit performed 2026-08-17 against QualiaDB branch 0.0.30/0.0.31-dev and qualia-ui v0.1.0. Settings and dark-capabilities audit added 2026-08-17.*
