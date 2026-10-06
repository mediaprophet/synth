# qualia-ui Collaborative / Cooperative ERP & PM UI Requirements

**Date:** 2026-08-18
**Principal / copyright holder:** Timothy Charles Holborn <timothy.holborn@gmail.com>
**Target UI:** `C:\Projects\NLP\qualia-ui` (WASM HyperCanvas — the primary UI target; webizen-studio Dioxus desktop is the prior work that qualia-ui supersedes and will eventually be ported once qualia-ui passes UAT)
**Companion document:** `C:\Projects\qualia-27062026\docs\plans\qualia-collaborative-erp-pm-todo-2026-08-18.md` (qualiaDB codebase to-do — **assumed completed** for the purposes of this requirements doc)

---

## 1. Purpose and scope

This document specifies the qualia-ui functionality required to deliver the **complete collaborative / cooperative ERP & project-management solution** — work-as-projects: personal, social, professional, cooperative/collaborative — using the full QualiaDB / Webizen capability stack.

It assumes the qualiaDB codebase to-do (companion doc) is completed: every engine record, command, ontology, and modality bridge referenced here is treated as available in the engine and exposed via the desktop Tauri command bus, the MCP tool surface, and the WASM runtime exports.

### 1.1 The six test cases the UI must support

| # | Use case | Project type |
|---|----------|--------------|
| 1 | Homeless person coordinating welfare/health/financial/legal support across agencies and professionals | Welfare-support |
| 2 | Family building a house with architect, builder, town planning | Professional |
| 3 | Long-term human-rights software project with historical costs; others join and add resource costs | Professional / open |
| 4 | Civic idea made public; others join; project funded; innovator paid | Civic / open |
| 5 | Research project on a topic/problem | Research |
| 6 | Permissive Commons humanitarian ICT (languages, biosphere, flora, fauna, funga, medicinal/organic chemistry); obligation costs paid down by defined consumers | Humanitarian ICT / commons |

### 1.2 Honesty labels

- **live** — fully implemented and functional in qualia-ui today
- **present** — UI structure exists but backend not wired
- **missing** — no UI surface at all
- **planned** — referenced in specs/manifold seeds but not implemented

---

## 2. qualia-ui current state (relevant to collaboration/ERP/PM)

### 2.1 What exists

| Surface | File | Status |
|---------|------|--------|
| 10 manifold seeds (research, media, social, communications, knowledge, projects, rights, sanctuary, settings, vibe) | `tool_chest/manifolds/*.rs` | **present** — seeds define layouts only; container bodies are placeholders |
| Projects manifold (sheet, graph, finance, portal, pulse containers + wires) | `tool_chest/manifolds/projects.rs` | **present** — `finance` and `portal` containers marked `honesty: "missing"` |
| Rights & Wallet manifold (rights, wallet containers) | `tool_chest/manifolds/rights.rs` | **present** |
| ~25 container body views (social, GIS, media, VibeScript, doc, sheet, graph, ontology, pulse, rights, wallet, library, aura, latex, health, anatomy, webview, webrtc, finance, vision, listen, triad, portal, slide, 3d, subcanvas) | `browser/containers.rs`, `browser/container_views.rs`, `browser/container_views_ext.rs` | **present** — mock views, no real engine wiring |
| Theme engine (7 presets, presentation levels, sanctuary mode, WCAG) | `browser/theme.rs` | **live** |
| Command palette (Ctrl+K) | `browser/command_palette.rs` | **live** |
| Container drag/resize/selection, canvas pan/zoom, wire rendering | `browser/interactions.rs` | **live** |
| Diagnostics (Aura tray, Pulse stream, Job Center) | `browser/diagnostics.rs` | **live** |
| ManifoldSeed CBOR-LD serialization stubs | `tool_chest/core/manifest.rs` | **present** — stubs |
| Ontology registry infrastructure | `tool_chest/core/ontology.rs` | **present** — no UI |
| IntentBus trait (no implementation) | `tool_chest/core/intent_bus.rs` | **missing** |
| Capability detection stubs | `browser/capabilities.rs` | **present** |
| Undo/redo history stubs | `browser/history.rs` | **present** |
| Toolbox/tool registration specs | `tool_chest/core/registry.rs`, `tool_chest/core/toolbox.rs`, `tool_chest/core/tool_chain.rs` | **present** — registry exists, no tools registered |

### 2.2 What is missing (the gaps this doc fills)

- No `IntentBus` implementation (P0 blocker)
- No backend transport (P0 blocker)
- No ontology loading at runtime (P0 blocker)
- No toolbox/tool registration at startup (P0 blocker)
- No tool families (`poet/tools/`)
- No dynamic clippable toolchain deck
- No real container body wiring to engine capabilities
- No project/work-item/Kanban/contribution/obligation surfaces
- No budget/finance/cost-base/funding/royalty surfaces
- No agency/delegation/credential/consent surfaces for projects
- No permissive commons publication/obligation/TSL surfaces
- No logic-modality editors (deontic, jural, N3, SHACL, SPARQL, etc.)
- No manifold/container sharing surfaces

---

## 3. Architectural requirements (P0 — must exist before any project surface)

These are the foundational pieces every project surface depends on. They map to INF-1..INF-4 in the companion to-do.

### 3.1 IntentBus (INF-1)

**Requirement:** A concrete `IntentBus` struct with:
- Typed intent payloads (`VibeScriptPayload`, graph mutations, project commands, logic evaluations)
- WASM transport (fetch for HTTP, WebSocket for streaming) + Tauri command bus for desktop
- Capability gating: every intent checked against the active QApp's `CompiledCapability` before dispatch
- Provenance stamping: every intent carries author DID, timestamp, manifold context
- Receipt handling: every dispatched intent produces a receipt (success/failure + result) written to the Pulse stream

**UI surface:** No direct UI; consumed by every container/tool. Diagnostics Aura tray shows intent traffic.

### 3.2 Backend transport (INF-2)

**Requirement:** A typed client layer mapping `VibeScriptPayload` → daemon endpoints:
- Desktop: Tauri `invoke()` for `wellfair_*`, `evaluate_*`, `execute_*` commands
- WASM-only: HTTP fetch to `127.0.0.1:{daemon_port}` + WebSocket for streaming
- Capability detection at startup: native-host vs public-web, available daemon, available models

**UI surface:** `browser/capabilities.rs` becomes real — shows a capability badge in the topbar (Native / Web / Limited).

### 3.3 Ontology loader (INF-3)

**Requirement:** Build-time N3 → CBOR-LD compiler (Trunk hook or `build.rs`) + WASM `OntologyLoader` with `include_bytes!` so the 36 ontologies in `qualia-ui/ontologies/` plus the new cooperative ontology (INF-15) load at runtime without network.

**UI surface:** Ontology browser in the Knowledge manifold; ontology selector in container headers.

### 3.4 Toolbox registration (INF-4)

**Requirement:** At `start()`, register all toolboxes, tool-chains, and manifold seeds into the shared `Registry`. Each toolbox declares its containers, tools, capability requirements, and ontology bindings.

**UI surface:** Toolbox dock populated; manifold switcher populated; command palette populated.

---

## 4. Project & ERP surfaces (the core of this document)

### 4.1 Projects manifold (enhance existing seed)

**Current:** `projects.rs` seed has sheet, graph, finance (missing), portal (missing), pulse containers.

**Required containers:**

| Container | Type | Purpose | Engine binding | Priority |
|-----------|------|---------|----------------|----------|
| Project Sheet | `sheet` | Project metadata, type, members, roles, licensing, values anchor | `wellfair_add_project`, `wellfair_add_project_membership`, COP-P1/P2 type templates | P0 |
| Work Items Kanban | `kanban` (new container type) | 7-column board (Proposed→Todo→InProgress→Blocked→InReview→Done→Cancelled) with cards | `wellfair_work_item_board`, `wellfair_add_work_item`, `wellfair_add_work_item_status` | P0 |
| Project Graph | `graph` | Work-item dependency graph, phase grouping, contributor network | COP-P4 dependencies, COP-P3 phases | P1 |
| Budget & Finance | `finance` | Budget line items, planned vs actual, ledger entries, funding flows, royalty distributions | COP-P6 Budget, `wellfair_add_ledger_entry`, `wellfair_ledger_balance`, COP-C5 FundingFlow, COP-C8 RoyaltyDistribution | P0 |
| Cost Base & Obligation | `cost_base` (new) | Cost-base entries by kind/contributor/phase, obligation state, TSL state (State A/B + shift indicator) | COP-C1 CostBaseEntry, COP-C2 value_flow, COP-C3 TSL, LOG-4 | P0 |
| Deliverables & Artifacts | `deliverable` (new) | Deliverable list with artifact hash, kind, acceptance status; artifact attachments | COP-P8 Deliverable, COP-X5 ArtifactAttachment | P1 |
| Reviews & Decisions | `review` (new) | Review assignments + decisions; project DecisionLog with M-of-N consensus indicator | COP-A4 ReviewAssignment, COP-X2 DecisionLog, LOG-5 consensus | P1 |
| Discussion | `discussion` (new) | Threaded comments on work items / projects | COP-X1 Comment/DiscussionThread | P1 |
| Roadmap & Phases | `roadmap` (new) | Timeline of phases + milestones with target dates; LTL invariant indicators | COP-P3 Phase, COP-P5 Roadmap, LOG-8 LTL | P1 |
| Activity Pulse | `pulse` | Project event stream (new work item, status change, contribution, review, obligation shift, funding) | COP-X4 Notification routing | P0 |
| Portal | `portal` | Project-scoped `qualia://` deep links, public project page (for civic/open types), collaborator invite | PRT-5 deep links, COP-A5 ConsentGrant | P1 |

### 4.2 Project Sheet container — detailed requirements

**Header:** Project name, type badge (from COP-P1 taxonomy), sensitivity class badge (COP-R1), values anchor (COP-R2), licensing profile (COP-R3).

**Tabs:**

| Tab | Fields / actions | Engine commands |
|-----|------------------|-----------------|
| Overview | Name, description, type selector (taxonomy), created_at, licensing_ontologies, values_anchor, sensitivity class | `wellfair_add_project` (create), `wellfair_get_project` (read) |
| Members | Member list with DID, role (COP-A3 taxonomy), credential status (verified/expired/revoked — COP-A2), joined_at; add/remove member; assign role | `wellfair_add_project_membership`, COP-A2 credential verify |
| Roles & Permissions | Per-type role taxonomy editor (Architect/Builder/TownPlanner for house-build; SocialWorker/HealthProvider/FinancialCounselor for welfare-support); role→Hohfeldian position mapping (LOG-2); role→deontic norm preview (LOG-1) | COP-A3, LOG-2, LOG-1 |
| Licensing | Licensing profile editor: obligation terms, share-alike toggle, consumer-class restrictions (COP-C4), TSL parameters (base fair value, risk multiplier, temporal compound) | COP-R3, COP-C4, COP-C3 |
| Agency & Delegation | Project-scoped delegations list (principal, agents, domain, trigger, validity, consent, revocation); create/revoke delegation; trigger builder UI (VerifiableEvent/TemporalWindow/DeadmanSwitch/HumanConsensus m-of-n/All/Any/Not) | COP-A1, `wellfair/agency.rs` commands, trigger.rs |
| Consent | Project-participation consent grants (principal, grantee, scope, validity, revocable); grant/revoke | COP-A5 |

### 4.3 Work Items Kanban container — detailed requirements

**Board:** 7 columns in canonical order (Proposed, Todo, InProgress, Blocked, InReview, Done, Cancelled). Cards show title, type icon (Task/Issue/Milestone), priority, assignee DID, estimate.

**Card interactions:**
- Click card → detail panel (description, estimate, assignee, phase, dependencies, comments, review assignments, time entries, linked deliverable)
- Drag card between columns → emits `WorkItemStatusEvent` via IntentBus
- Card context menu: edit, delete (tombstone), convert to milestone, link dependency, assign reviewer

**Filters:** by assignee, by phase, by type, by priority, by dependency-depth.

**Bulk actions:** multi-select → change status, assign, move to phase.

**Honesty:** Status is a derived projection — the UI must never show a "status" field on the card edit form; status changes only via column drag or explicit status event action. This matches the replay-safe derivation in `work_item.rs`.

### 4.4 Budget & Finance container — detailed requirements

**Tabs:**

| Tab | Content | Engine |
|-----|---------|--------|
| Budget | Line items (category, planned_cents, currency, phase); add/edit/delete line item; total per phase; total project | COP-P6 |
| Ledger | Ledger entries (description, amount, currency, category, counterparty, project_id); add entry; running balance per currency | `wellfair_add_ledger_entry`, `wellfair_ledger_balance` |
| Variance | Planned vs actual per category per phase; over/under budget indicators | COP-P6 BudgetVariance derivation |
| Funding | Funding flows (funder, amount, kind [grant/donation/revenue/sponsorship], date); add flow; "apply to obligation" action | COP-C5 |
| Royalties | Royalty distributions per contributor (amount, based on roi_multiplier + funding); replay-safe indicator | COP-C8 |
| Tax Router | TaxRecipientSuite editor (jurisdiction, recipients with share_percent summing to 100, use_nym); dispatch plan preview; execute (ILP/Lightning/XEC) | `route_tax_payment`, PRT-1/2/3 |

### 4.5 Cost Base & Obligation container — detailed requirements

**Tabs:**

| Tab | Content | Engine |
|-----|---------|--------|
| Cost Base | CostBaseEntry list (contributor, kind [effort/capital/material/overhead], amount, currency, date, evidence_ref); add entry; per-contributor per-kind totals; per-phase roll-up | COP-C1 |
| Obligation | Per-contributor obligation (total_effort, total_capital, resolved_obligation_score, count); project total obligation; outstanding indicator | COP-C2, `wellfair_project_obligations` |
| TSL State | Current state (State A Commercial / State B Commons), shifted_at, obligation met %, shift button (evaluates `evaluate_tsl_shift`), WAL audit trail of shift | COP-C3 |
| Consumer Classes | ConsumerClass bindings (consumer_class_term_id, paydown_weight); add/remove; ontology selector | COP-C4 |
| Commons Publication | "Publish to Commons" action: classify artefacts, split Selfhood/Personhood, emit magnets for Permissive Commons volumes, attach licensing profile + cost base | COP-M1 |
| Commons Artefacts | Registry of published artefacts (hash, kind, obligation_state, consumer_classes, shifted_at) | COP-M3 |

### 4.6 Reviews & Decisions container — detailed requirements

**Tabs:**

| Tab | Content | Engine |
|-----|---------|--------|
| Review Assignments | Open assignments (work_item, reviewer, required_credential, deadline); assign reviewer; credential badge | COP-A4 |
| Review Decisions | Decisions log (verdict [Approve/Reject/ChangesRequested], signed_by, decided_at, provenance); sign decision | COP-A4 |
| Decision Log | Project decisions (decided_by[], kind, summary, evidence_ref, date); M-of-N consensus indicator; add decision | COP-X2, LOG-5 |
| Argumentation | Argumentation framework view over reviews+comments (attack/defense graph, grounded extension highlight) | LOG-7 |

### 4.7 Roadmap & Phases container — detailed requirements

**View:** Timeline (horizontal) with phases as segments, milestones as markers with target dates. LTL invariant indicators (G φ / F φ) shown as badges.

**Interactions:** add phase, add milestone to roadmap, drag milestone to set target date, link work items to milestones.

**Engine:** COP-P3 Phase, COP-P5 Roadmap, LOG-8 LTL.

### 4.8 Discussion container — detailed requirements

**View:** Threaded comments (parent_id tree), author DID, timestamp, body (rendered from body_hash + content). Per work item or per project.

**Interactions:** add comment, reply, edit (creates new entry with predecessor_id — never mutate), link to work item.

**Engine:** COP-X1.

---

## 5. Per-project-type UI variations

The UI must adapt to the project type (COP-P1). The Project Sheet container's type selector drives manifold-level variations:

| Project type | Manifold variation | Default sensitivity | Required containers | Optional containers |
|--------------|--------------------|--------------------|---------------------|---------------------|
| Welfare-support | + Health, Rights, Sanctuary containers; SocialWebNet channel indicator | Restricted (Selfhood-adjacent for health/legal/financial) | Project Sheet, Kanban, Budget, Cost Base, Discussion, Pulse | Health vitals, Clinical risk, Anatomy, Rights & Agreements, Agency & Delegation |
| Professional (house-build) | + 3D model container for house plans; GIS for site | Restricted | Project Sheet, Kanban, Budget, Deliverables, Reviews, Roadmap, Pulse | 3D model, GIS, Finance, Portal |
| Professional / open (software) | + VibeScript code container, Git integration | Permissive | Project Sheet, Kanban, Cost Base, Obligation, Discussion, Pulse | VibeScript, Graph, Portal, Reviews |
| Civic / open | + Portal (public page), Funding container prominent | Public | Project Sheet, Kanban, Funding, Royalties, Portal, Pulse | Discussion, Reviews |
| Research | + Library, Latex, Citation containers | Permissive | Project Sheet, Kanban, Deliverables, Reviews, Roadmap, Discussion, Pulse | Library, Latex, Graph, Portal |
| Humanitarian ICT / commons | + Commons Publication, Ontology Builder, Chemistry/Bio containers | Permissive (Commons-flagged) | Project Sheet, Kanban, Cost Base, Obligation, TSL State, Commons Publication, Commons Artefacts, Pulse | Ontology Builder, Chemistry, Bioinformatics, Library |

---

## 6. Rights, agency, and wallet surfaces (Rights & Wallet manifold — enhance)

### 6.1 Rights & Agreements container

**Tabs:**

| Tab | Content | Engine |
|-----|---------|--------|
| Agreements | Agreement list (parties, scope, M-of-N threshold, status); create agreement; compile to `AgreementDID` Quins | `webizen.rs::AgreementDID`, INF-9 |
| Deontic Norms | Project deontic norms (OBLIGATE/PERMIT/FORBID); compile from licensing profile + values anchor + roles; evaluate against live graph | COP-R4, LOG-1, `evaluate_deontic_contract` |
| Jural Relations | 8 Hohfeldian positions per role; correlative/opposite highlighting; find unmet correlatives | LOG-2, `jural.rs` |
| Breach Log | Meta-deontic breach records (norm, status, opcode, defeat_kind); WAL audit trail | LOG-6, `meta_deontic.rs` |
| Consents | Policy consents (grant/revoke/list) + project-participation consents (COP-A5) | `wellfair_grant_consent`/`revoke`/`list`, COP-A5 |

### 6.2 Wallet container

**Tabs:**

| Tab | Content | Engine |
|-----|---------|--------|
| Balances | Per-currency balance (from `derived_balance`); project-scoped balance filter | `wellfair_ledger_balance` |
| ILP / Lightning / XEC | Payment pointer management; send/receive micropayment; ILP execution status | PRT-1/2/3 |
| Tax Suite | TaxRecipientSuite editor (shared with Budget & Finance tab) | `route_tax_payment` |
| Compute Costs | ComputeCostReceipt history (telemetry → sats) | `ComputeCostReceipt::generate` |

---

## 7. Logic modality editors (Knowledge manifold — enhance)

These are the reasoning tools that project governance surfaces depend on. The full inventory is in `consult/20260818_logic-modalities-audit.md`; the project-relevant subset:

### 7.1 Core reasoning tools (P0 for project governance)

| Surface | Container type | Engine | qualia-ui status |
|---------|---------------|--------|-----------------|
| Deontic rule editor | `ontology` (specialised) | `deontic_logic.rs`, `evaluate_deontic_contract` | missing |
| N3 Logic Studio | `ontology` (specialised) | `n3_parser.rs`, `n3_compiler.rs`, `evaluate_logic_rules` | missing |
| SHACL Validator | `ontology` (specialised) | `shacl_compiler.rs`, `validate_shacl_shape` | missing |
| SPARQL Explorer | `graph` (specialised) | `execute_sparql_query` | missing |
| RDF-Star Editor | `graph` (specialised) | `resolver.rs`, `lexicon.rs` | missing |
| Ontology Builder | `ontology` | `fetch_domain_ontology` | missing |
| Evaluate Modality panel | `aura` (specialised) | MCP `evaluate_modality` (LTL/ASP/deontic/epistemic/DL/paraconsistent) | missing |
| Symbolic logic infer | `aura` (specialised) | MCP `symbolic_logic_infer` | missing |

### 7.2 Legal/governance tools (P1 for project governance)

| Surface | Engine | Project binding |
|---------|--------|-----------------|
| Jural relations editor | `jural.rs` | LOG-2 (project roles) |
| STIT agency inspector | `stit.rs` | COP-A1 (project delegations) |
| Causal liability analyzer | `causal.rs` | LOG-3 (deliverable attribution) |
| Responsibility/meta-guard | `responsibility.rs` | LOG-6 (breach accountability) |
| Capacity evaluator | `capacity.rs` | LOG-10 (membership capacity) |
| Delegation tracker | `delegation.rs` | COP-A1 (project delegations) |
| Contract formation | `contract.rs` | COP-R3 (licensing as contract) |
| Consensus/partition | `consensus.rs` | LOG-5 (M-of-N decisions) |
| Meta-deontic breach log | `meta_deontic.rs` | LOG-6 |
| Value flow / commons | `value_flow.rs` | LOG-4 (cost base + obligation) |
| Interaction governance | `interaction_governance.rs` | project policy mode |
| Identity fabric | `identity_fabric.rs` | project member identity |
| Capability gap analyzer | `capability_gap.rs` | project role requirements |

### 7.3 Advanced logic tools (P1/P2)

Epistemic, paraconsistent, LTL, CTL, ASP, defeasible, linear, description, dialectical, argumentation, abductive, fuzzy, probabilistic, graph theory, interval, manifold 10D, epistemic boundaries, modal — see `consult/20260818_logic-modalities-audit.md` §8.2 Tier 3. Project-relevant subset: LTL (roadmap milestones), defeasible (project policies), argumentation (review), capacity (membership).

---

## 8. Permissive Commons publication surfaces (use case 6)

### 8.1 Commons Publication container (in Cost Base & Obligation container, §4.5)

**Flow UI:**
1. Select project → list artefacts (deliverables, ontologies, datasets, models, manifolds)
2. Classify each artefact: Selfhood / Personhood / Unmarked / Permissive Commons
3. Mixed volumes warning (Selfhood + Commons in one volume → denied until split)
4. Set consumer classes (COP-C4) + obligation terms + TSL parameters
5. Publish → emit magnets for Permissive Commons volumes only; Selfhood volumes stay SocialWebNet/local
6. Show publication receipts (magnet links, web-seed URLs, TrustGroup gating status)

### 8.2 Commons Artefact registry container

**View:** Table of published artefacts (hash, kind, project, obligation_state, consumer_classes, shifted_at). Filter by project, by state (A/B), by consumer class. Click → detail (provenance, seeders, download count, obligation paydown progress).

### 8.3 Share-alike seed obligation indicator

Post-shift (State B) artefacts show a "share-alike: seed required" badge; the billing gate enforces it (COP-M2). The UI shows the user's seeding status per artefact.

---

## 9. Manifold and container sharing surfaces

### 9.1 ManifoldSeed CBOR-LD serialization (COP-S1)

**Requirement:** `ManifoldSeed` round-trips through CBOR-LD; the manifest is hashable and publishable. The `tool_chest/core/manifest.rs` stubs must be completed.

**UI surface:** "Save manifold" action in the topbar → writes CBOR-LD manifest to the project's artifact store; "Load manifold" → reads from artifact store or commons.

### 9.2 Publish manifold to commons (COP-S2)

**UI surface:** "Publish manifold to Commons" action → classifies the manifold (containers must be Permissive Commons types or already-published templates), emits magnet, registers in CommonsArtefact registry.

### 9.3 ContainerTemplate registry (COP-S3)

**UI surface:** Container template browser in the toolbox dock; drag a template onto the canvas to instantiate a pre-configured container (e.g. "House-build Budget Template", "Welfare-support Health Vitals Template", "Research Citation Graph Template").

---

## 10. Tool families (`poet/tools/` — P0)

The 9–12 tool families from the handover doc, with project-relevant tools highlighted:

| Family | Tools | Project relevance | Priority |
|--------|-------|-------------------|----------|
| `rich_text` | WYSIWYG, Semantic Spans, NLP Gazetteer | Discussion, DecisionLog, deliverable docs | P0 |
| `sheet` | Spreadsheets, Formulas, Columnar Tensors | Project Sheet, Budget, Cost Base | P0 |
| `rdf_graph` | Aura / SHACL Validation, N3 Logic, SPARQL | Project Graph, logic editors | P0 |
| `hypermedia` | Hypermedia Library, HCF/CBOR Archives | Deliverables, Commons Artefacts, manifold sharing | P0 |
| `media_studio` | Photo, Vector Art, Video | House-build plans, research figures | P1 |
| `ten_d_assets` | 10D Asset Container, Kinematics, Animations | House-build 3D, anatomy | P1 |
| `audio_speech` | Spectral Tomography, Waveforms, Q42/D10 Triads | Accessibility, oral tradition (use case 6) | P2 |
| `communication` | Multi-Agent Chat Graph, WebRTC, DID Auth | Discussion, meetings, collaborator invite | P0 |
| `spatial_gis` | Topographic GIS Maps, 4D Tracks | House-build site, welfare-support location | P1 |
| `vibescript_code` | VibeScript 0.1 AST Console, Capabilities, Gas | Software project (use case 3), automation | P0 |
| `rights_fiduciary` | Fiduciary Duty Scopes, L3 Gates, Merkle Audit | Rights & Agreements, agency, breach log | P0 |
| `health_clinical` | Pathology Lab, Bio-Telemetry, 10D Anatomy | Welfare-support (use case 1) | P1 |

### 10.1 Dynamic clippable toolchain deck (P1)

**Requirement:** A context ribbon receptor that switches the active toolchain based on the focused container. Focusing the Kanban shows board tools; focusing the Budget shows sheet/finance tools; focusing the Rights container shows rights/fiduciary tools.

---

## 11. Cross-cutting UI requirements

### 11.1 Honesty labels everywhere

Every container and tool shows an honesty badge (live/present/missing/planned) so the user never mistakes a placeholder for a working feature. The Projects manifold seed already uses this pattern (`honesty: "missing"` on finance/portal); extend it to all containers.

### 11.2 Provenance and audit visibility

Every record displayed in the UI shows its provenance: author DID, asserted time, valid time, predecessor_id (for the author chain), blob_hash, epistemic_status, evidence_type, sensitivity class. The Pulse stream shows the live audit trail. This is non-negotiable for the welfare-support and human-rights-software use cases where accountability is the point.

### 11.3 Replay-safe indicators

Where a derivation is replay-safe (Contribution merge, Ledger merge, WorkItemStatus merge, CostBaseEntry merge), show a "replay-safe" badge so users understand duplicate/reordered sync frames cannot corrupt the data.

### 11.4 Sensitivity class badges

Every record shows its sensitivity class (Public / Restricted / Classified) per the Quin context bits 56-63. Selfhood-adjacent records (health, legal, financial, reproductive/biometric/genetic) show a Selfhood badge. The UI must never allow a Selfhood record to be dragged into a Permissive Commons publication flow.

### 11.5 Capability gating in the UI

Every tool/container action that requires a capability (daemon, WASM, chat, health, query, etc.) checks the capability at call time and shows a disabled state with an explanation if the capability is unavailable. This matches the QApp capability model (`verify_execution_scope`).

### 11.6 Multi-language support

Per the Permissive Commons doc §2.3, the UI must support all first languages including all languages of prayer. The theme engine already has presentation levels; the i18n layer must cover every visible string. This is load-bearing for use case 6 (humanitarian ICT) and for the welfare-support use case with non-English-speaking principals.

### 11.7 Accessibility (WCAG)

The theme engine already has WCAG contrast; extend to all project surfaces. The welfare-support use case specifically requires accessible UI for people in crisis.

### 11.8 Sanctuary mode

The theme engine has Sanctuary mode; the project surfaces must respect it — in Sanctuary mode, Selfhood-adjacent records are hidden by default, commons publication is disabled, and the UI shows only what is safe to show.

---

## 12. Use-case UI walkthroughs (validation)

### 12.1 Use case 1 — Homeless welfare-support coordination

1. Principal opens qualia-ui → Sanctuary mode (default for welfare-support projects)
2. Creates welfare-support project (type selector → welfare_support template → Restricted sensitivity, UN-HR values anchor)
3. Adds members: self (Principal), social worker (SocialWorker role + credential), health provider (HealthProvider role + credential), financial counselor (FinancialCounselor role + credential)
4. Creates project-scoped delegations: social worker → personal_welfare domain; health provider → medical domain; financial counselor → financial domain. Each delegation has a trigger (e.g. HumanConsensus 1-of-1 signed for routine, 2-of-N for crisis)
5. Kanban: work items for "Find housing", "Apply for benefits", "Medical appointment", "Legal aid application". Each assigned, with dependencies
6. Health vitals container (existing wellfair health commands) shows biometric records (Selfhood — never publishable)
7. Rights & Agreements container shows the delegations + consents
8. DecisionLog records each case-management decision with M-of-N consensus
9. Pulse stream shows all activity; notifications routed to relevant members
10. SocialWebNet channel indicator shows the pairwise WireGuard tunnel is active for restricted collaboration

### 12.2 Use case 2 — House-building

1. Family creates professional project (type → professional → house-build template)
2. Adds members: Owner (Principal), Architect (Architect role + license credential), Builder (Builder role + registration credential), Town Planner (TownPlanner role + credential)
3. Roadmap: phases (Design, Planning Approval, Construction, Inspection, Handover); milestones per phase with target dates
4. Kanban: work items per phase (e.g. "Draft plans", "Submit DA", "Pour slab", "Frame", "Lock-up", "Fixing", "Practical completion")
5. Budget: line items per phase (materials, labor, permits, contingencies); variance tracked against ledger entries
6. Deliverables: plans (artifact hash), DA approval (artifact hash), inspection certificates; each accepted by the relevant role with a ReviewDecision
7. 3D model container shows the house plans; GIS container shows the site
8. Reviews: architect signs off plans, town planner signs off DA, builder signs off construction milestones
9. Portal: deep links to specific work items for each collaborator

### 12.3 Use case 3 — Long-term human-rights software

1. Principal creates professional/open project (type → professional/open → software template → Permissive sensitivity, specific human-rights values anchor)
2. Adds historical CostBaseEntry records (effort + capital from years of work) with evidence_ref
3. Others join as Contributors; each adds their own CostBaseEntry records (effort, capital) — author chain via predecessor_id
4. Obligation derivation shows per-contributor obligation growing
5. VibeScript code container for the actual software; Git integration for commits
6. Discussion container for design decisions; DecisionLog for governance decisions
7. Deontic norms compiled from the values anchor + licensing profile (e.g. "FORBID enclosure of fundamental knowledge axioms")
8. If the project publishes ontologies/rulesets to the Permissive Commons: Commons Publication flow with consumer classes + TSL

### 12.4 Use case 4 — Civic idea → funding → paid innovator

1. Principal creates civic/open project (type → civic → Public sensitivity)
2. Publishes the idea via the Portal container (public project page)
3. Others join as Contributors; Kanban tracks the work to develop the idea
4. FundingFlow records: grants, donations, sponsorship; each "applied to obligation" reduces outstanding
5. RoyaltyDistribution shows each contributor's share (innovator + contributors per roi_multiplier)
6. Tax Router splits incoming funding 12% → TaxRecipientSuite, 88% → principal/contributors
7. ILP executor (PRT-1) executes the micropayment streams
8. Notifications notify contributors when funding is received and distributed

### 12.5 Use case 5 — Research project

1. PI creates research project (type → research → Permissive sensitivity)
2. Adds members: PI, Researchers, Reviewers, Funder
3. Roadmap: phases (Literature review, Methodology, Data collection, Analysis, Writing, Peer review, Publication)
4. Kanban: work items per phase; ReviewAssignments for the peer-review phase
5. Library container for references; Citation graph in the Project Graph
6. Deliverables: datasets (artifact hash), paper drafts (artifact hash), final paper
7. Reviews: ReviewDecisions with provenance; argumentation framework view over reviewer comments
8. LTL invariants on the Roadmap (G "data integrity holds", F "publication submitted")
9. DecisionLog records research decisions with evidence_ref

### 12.6 Use case 6 — Permissive Commons humanitarian ICT

1. Contributors create humanitarian_ict/commons project (type → humanitarian_ict → Permissive Commons sensitivity)
2. Kanban: work items for "Document language X", "Formalize medicinal ontology Y", "Chemistry profile of compound Z"
3. Ontology Builder container authors the ontologies; Chemistry/Bioinformatics containers do the science
4. CostBaseEntry records accumulate the cost of producing the commons goods (effort + capital)
5. ConsumerClass bindings: e.g. "corporation" pays down, "indigenous_knowledge_holder" does not
6. TSL parameters set (base fair value, risk multiplier, temporal compound)
7. Commons Publication flow: classify artefacts (Personhood — publishable), set consumer classes + obligation terms, emit magnets
8. CommonsArtefact registry shows published artefacts with obligation_state
9. As consumers pay down (FundingFlow with kind=revenue from defined consumers), the obligation reduces; TSL State A→B shift indicator approaches 100%
10. On shift: State B badge, share-alike seed obligation enforced, WAL audit Quin emitted
11. WebTorrent project-channel seeds the artefacts with TrustGroup gating

---

## 13. Implementation priority for qualia-ui

### Phase 0 — Foundation (P0, must precede everything)

- INF-1 IntentBus implementation
- INF-2 Backend transport (Tauri + HTTP/WS)
- INF-3 Ontology loader (build-time N3 → CBOR-LD + WASM loader)
- INF-4 Toolbox registration at startup
- Real capability detection (`browser/capabilities.rs`)

### Phase 1 — Core project surfaces (P0)

- Projects manifold enhancement: Project Sheet, Work Items Kanban, Activity Pulse containers wired
- Budget & Finance container (Budget, Ledger, Variance tabs)
- Cost Base & Obligation container (Cost Base, Obligation, TSL State, Consumer Classes tabs)
- Rights & Wallet manifold enhancement: Rights & Agreements (Agreements, Deontic Norms, Consents tabs), Wallet (Balances, Compute Costs tabs)
- Tool families: `rich_text`, `sheet`, `rdf_graph`, `hypermedia`, `communication`, `vibescript_code`, `rights_fiduciary` (P0 subset)
- Logic editors: Deontic rule editor, N3 Logic Studio, SHACL Validator, SPARQL Explorer, RDF-Star Editor, Ontology Builder, Evaluate Modality panel, Symbolic logic infer

### Phase 2 — Extended project surfaces (P1)

- Project Graph (dependencies, phases, contributor network)
- Deliverables & Artifacts container
- Reviews & Decisions container
- Discussion container
- Roadmap & Phases container
- Portal container (deep links, public page, collaborator invite)
- Commons Publication + Commons Artefacts containers
- Tax Router + ILP/Lightning/XEC execution in Wallet
- Tool families: `media_studio`, `ten_d_assets`, `spatial_gis`, `health_clinical`
- Logic editors: Jural, STIT, Causal, Responsibility, Capacity, Delegation, Contract, Consensus, Meta-deontic, Value flow, Interaction governance, Identity fabric, Capability gap
- Dynamic clippable toolchain deck
- ManifoldSeed CBOR-LD serialization (COP-S1)

### Phase 3 — Polish (P2)

- Sprint/Iteration container
- Meeting container
- Publish manifold to commons (COP-S2)
- ContainerTemplate registry (COP-S3)
- Tool families: `audio_speech`
- Logic editors: LTL, CTL, ASP, Defeasible, Linear, Description, Dialectical, Argumentation, Abductive, Fuzzy, Probabilistic, Graph theory, Interval, Manifold 10D, Epistemic boundaries, Modal
- Lightning/XEC/Nym bridges in Wallet

---

## 14. File structure (proposed additions to qualia-ui)

```
qualia-ui/src/
├── browser/
│   ├── containers.rs              (extend: kanban, cost_base, deliverable, review, discussion, roadmap, portal container types)
│   ├── container_views.rs         (extend: project sheet, kanban, budget, cost base views)
│   ├── container_views_ext.rs     (extend: rights, wallet, commons publication views)
│   ├── capabilities.rs            (real capability detection)
│   └── ...
├── tool_chest/
│   ├── core/
│   │   ├── intent_bus.rs          (INF-1: real implementation)
│   │   ├── transport.rs           (INF-2: new — Tauri + HTTP/WS transport)
│   │   ├── ontology.rs            (INF-3: real loader)
│   │   ├── registry.rs            (INF-4: real registration)
│   │   └── manifest.rs            (COP-S1: real CBOR-LD serialization)
│   ├── manifolds/
│   │   ├── projects.rs            (enhance: add new containers)
│   │   └── ...
│   └── tools/                     (new — tool families)
│       ├── rich_text/
│       ├── sheet/
│       ├── rdf_graph/
│       ├── hypermedia/
│       ├── communication/
│       ├── vibescript_code/
│       ├── rights_fiduciary/
│       ├── media_studio/
│       ├── ten_d_assets/
│       ├── spatial_gis/
│       ├── health_clinical/
│       └── audio_speech/
└── ontologies/
    └── cooperative-projects.ttl   (INF-15: new cooperative ontology)
```

---

## 15. Open questions for the principal

1. **Kanban as a new container type vs sheet variant**: should the Kanban be a first-class container type (with its own drag/column semantics) or a specialised sheet view? First-class seems right given the derived-status invariant, but it's more work.
2. **Tool family location**: `tool_chest/tools/` (proposed) vs `poet/tools/` (handover doc). The handover doc says `poet/tools/`; the existing code uses `tool_chest/`. Which convention wins?
3. **Per-project-type manifold variation**: should each project type get its own manifold seed (welfare_support, house_build, software, civic, research, humanitarian_ict) or should the single Projects manifold adapt its containers based on the project type? The latter is less code but harder to make feel native.
4. **Commons Publication flow placement**: should it live in the Cost Base & Obligation container (proposed) or be a standalone manifold (a "Commons" manifold alongside Projects/Rights/Knowledge)?
5. **Multi-language i18n priority**: is i18n P0 (required for use case 6 humanitarian ICT and use case 1 welfare-support with non-English principals) or P1 (English first, other languages after UAT)?

---

## 16. References

| Document | Path |
|----------|------|
| qualiaDB to-do (companion) | `C:\Projects\qualia-27062026\docs\plans\qualia-collaborative-erp-pm-todo-2026-08-18.md` |
| Logic modalities audit | `C:\Projects\NLP\consult\20260818_logic-modalities-audit.md` |
| QApp audit | `C:\Projects\NLP\consult\20260818_qualia-ui-qapp-audit.md` |
| qualia-ui handover | `C:\Projects\NLP\consult\20260818_qualia-ui_handover.md` |
| Permissive Commons spec | `C:\Projects\qualia-27062026\docs\manuals\webizen_permissive_commons.md` |
| Cooperative-core lib | `C:\Projects\qualia-27062026\crates\qualia-cooperative-core\src\lib.rs` |
| Wellfare-core projects | `C:\Projects\qualia-27062026\crates\wellfare-core\src\projects.rs` |
| Wellfare-core finance | `C:\Projects\qualia-27062026\crates\wellfare-core\src\finance.rs` |
| Agency delegation | `C:\Projects\qualia-27062026\crates\qualia-cooperative-core\src\agency_delegation.rs` |
| Trigger algebra | `C:\Projects\qualia-27062026\crates\qualia-cooperative-core\src\trigger.rs` |
| ILP Tax Router | `C:\Projects\qualia-27062026\crates\qualia-core-db\src\services\rpc.rs` |
| Desktop wellfair commands | `C:\Projects\qualia-27062026\crates\webizen-desktop\src\commands\wellfair\` |

---

_End of UI requirements. Companion engine to-do: `C:\Projects\qualia-27062026\docs\plans\qualia-collaborative-erp-pm-todo-2026-08-18.md`._
