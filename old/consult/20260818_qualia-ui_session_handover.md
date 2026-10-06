# qualia-ui Session Handover

**Date:** 2026-08-18
**Principal / copyright holder:** Timothy Charles Holborn <timothy.holborn@gmail.com>
**Repository:** `C:\Projects\NLP\qualia-ui`
**Purpose:** Enable a new session to continue work without re-reading the full codebase (which causes OOM). Read this document first, then use targeted file reads only as needed.

---

## 1. Project overview

`qualia-ui` is a Rust/WebAssembly HyperCanvas UI for the QualiaDB/Webizen ecosystem. The goal is to complete the UI/UX independently of the backend so it can be evaluated before daemon integration.

- **Build:** `trunk build` (WASM, `wasm32-unknown-unknown`)
- **Test:** `cargo test --lib` (63 tests passing as of last session)
- **Format:** `cargo fmt`
- **Check:** `cargo check` (only pre-existing `async_fn_in_trait` warnings in `src/tool_chest/core/intent_bus.rs`)
- **No daemon dependency** — all backend-dependent features are mocked with honest labels

---

## 2. Architecture

### 2.1 Module structure

```
src/browser/
  mod.rs                  — entry point, app builder, module wiring
  css.rs                  — all CSS as a const string (append new styles at end before `"#;`)
  topbar.rs               — menubar (File/Edit/View/Insert/Help) + canvas control bar
  containers.rs           — container DOM builder (build_container)
  container_views.rs      — container body views (doc, sheet, graph, ontology, etc.)
  container_views_ext.rs  — extended container views (media, 3d, webrtc, etc.)
  interactions.rs         — drag, resize, selection, deletion, wires, zoom, set_timeout binding
  docks.rs                — toolbox dock (left), right dock, status bar
  command_palette.rs      — Ctrl+K palette with fuzzy search + command dispatch
  search_workbench.rs     — Ctrl+Shift+F modal: faceted search, SPARQL builder, manual SPARQL, saved queries
  logic_workbench.rs      — Ctrl+Shift+L modal: 9 reasoning tools (deontic, N3, SHACL, RDF-Star, etc.)
  contextual_popover.rs   — text selection → RDF entity annotation popover
  workflow_panels.rs      — workflow panel views
  instrument_panel.rs     — contextual tool panel (renamed from "ribbon")
  wire_inspector.rs       — wire label editing
  history.rs              — undo/redo history (push_current_frame)
  manifest.rs             — ManifoldSeed CBOR-LD serialization stubs
  diagnostics.rs          — Aura tray, Pulse stream, Job Center
  capabilities.rs         — capability detection stubs
  theme.rs                — 7 theme presets, sanctuary mode, WCAG contrast
  registration.rs         — builds the registry (10 toolboxes + 10 manifold seeds)
  intent_bus.rs           — IntentBus trait (async, has pre-existing warnings)

src/tool_chest/
  core/
    registry.rs           — ManifoldSeed, SeedContainer, SeedConnection, SeedPanel, ContainerKind
    toolbox.rs            — Toolbox struct
    tool_chain.rs         — ToolChain struct
    tool.rs               — Tool struct
    ontology.rs           — ontology registry (present, no UI)
    manifest.rs           — manifest serialization
    intent_bus.rs         — VibeScriptPayload, ActionType, TargetIdentifier
  manifolds/
    research.rs, media.rs, social.rs, communications.rs, knowledge.rs,
    projects.rs, rights.rs, sanctuary.rs, settings.rs, vibe.rs
  payloads/
    communications.rs, settings.rs, social.rs
```

### 2.2 Key types

- `ManifoldSeed` — id, label, icon, ontology_prefix, description, containers, connections, panels
- `SeedContainer` — container_type, kind (ContainerKind enum), title, x, y, width, height, z, honesty; has `Default` impl so literals use `..Default::default()`
- `ContainerKind` — Content, Panel, Widget; `from_type()` infers from container_type
- Honesty labels: `live`, `present`, `partial`, `missing`

### 2.3 Patterns for adding new UI surfaces

1. **Modal overlay pattern** (used by search_workbench, logic_workbench):
   - Create a `build_*()` function returning an `Element` (overlay div, hidden by default)
   - Append to `app` in `mod.rs::build_app()`
   - Wire a keyboard shortcut in `mod.rs` after other wiring
   - Add command palette entries in `command_palette.rs::build_command_list()` + dispatch in `execute_command()`
   - Add a topbar button in `topbar.rs::build_top_menubar()`
   - Add CSS in `css.rs` before the closing `"#;`

2. **Container type pattern**:
   - Add to `containers.rs::build_container()` body rendering
   - Add view in `container_views.rs` or `container_views_ext.rs`
   - Add to manifold seed literals in `tool_chest/manifolds/*.rs`

3. **Event wiring**:
   - Use `Closure::wrap(Box::new(move |e: MouseEvent| { ... }) as Box<dyn FnMut(MouseEvent)>)`
   - Always `.forget()` the closure after `add_event_listener_with_callback`
   - Use `super::interactions::set_timeout(callback, delay_ms)` for timeouts

---

## 3. Current state (as of 2026-08-18, session 3)

### 3.1 Completed features

- 10 manifold seeds (research, media, social, communications, knowledge, projects, rights, sanctuary, settings, vibe)
- ~25 container body views (all mock/placeholder)
- Container drag/resize/selection/deletion/duplication
- Wire drawing, selection, deletion, label editing
- Canvas zoom, pan, multi-select (Shift+click)
- Command palette (Ctrl+K) with fuzzy search
- Search Workbench (Ctrl+Shift+F) — 4 modes: faceted, query builder, manual SPARQL, saved queries
- Logic Workbench (Ctrl+Shift+L) — 9 tools: deontic, N3, SHACL, RDF-Star, ontology builder, evaluate modality, symbolic infer, jural, argumentation
- Contextual RDF popover (text selection → entity annotation)
- Theme engine (7 presets, sanctuary mode, WCAG)
- Undo/redo history
- Strata/epistemic filters
- Right dock (Aura tray, Pulse stream)
- Topbar menubar with dropdown menus
- Toolbox dock with family flyouts
- Instrument Panel (contextual tooling)
- 63 unit tests passing
- Trunk build clean

### 3.2 Workstream A — Phase 1 containers (completed session 3)

- **Project containers** (9): kanban, project_sheet, budget, cost_base, deliverable, review, discussion, roadmap, commons
- **Rights & Wallet containers** (2): rights (5 tabs: agreements, deontic, jural, breach, consents), wallet (4 tabs: balances, ILP, tax, compute)
- **Files created:**
  - `src/browser/project_views/` — mod.rs + 9 view modules (kanban.rs, project_sheet.rs, budget.rs, cost_base.rs, deliverable.rs, review.rs, discussion.rs, roadmap.rs, commons.rs)
  - `src/browser/rights_views/` — mod.rs, rights_tabs.rs, wallet_tabs.rs
  - `src/browser/container_inline_views.rs`
- **Files modified:**
  - `src/browser/containers.rs` — dispatch for new container types
  - `src/tool_chest/manifolds/projects.rs` — 9 new container seeds
  - `src/tool_chest/manifolds/rights.rs` — 3 new container seeds (rights, wallet, plus existing)
  - `src/browser/command_palette/commands.rs` — 20+ new command entries
- **Bug fixes:** Rust type mismatch errors in `rights_tabs.rs` and `wallet_tabs.rs` (pre-calculate color values outside loops, use explicit tuple field access with `*` deref)

### 3.3 Workstream A — Extended implementation plan (completed session 3)

**Document:** `WORKSTREAM_A_IMPLEMENTATION_PLAN.md` (~1360 lines)

Comprehensive plan covering **74 container types** across 12 sub-sections:

| Section | Topic | Containers | Priority |
|---------|-------|-----------|----------|
| §8a | Agreement framework & instruments | 1 (agreement_builder) | P0 |
| §8b | Fair value, compensation & obligation cost | 2 (compensation_model, contribution_ledger) | P0 |
| §8c | Differential licensing & obligation recovery | 2 (license_builder, obligation_tracker) | P0 |
| §8d | Awards, tokens & recognition | 2 (awards, token_mgr) | P1 |
| §8e | Disputes, complaints & corrections | 3 (disputes, complaints, corrections) | P0 |
| §8f | Onboarding & bulk admin | 2 (onboarding, bulk_import) | P0 |
| §8g | Governance meetings, minutes & resolutions | 2 (governance_meetings, governance_settings) | P0 |
| §8h | Conflict of interest | 1 (conflict_of_interest) | P0 |
| §8i | Zero-knowledge privacy controls | 0 (cross-cutting) | P0–P1 |
| §8j | Provenance studies, innovation, research & IP | 4 (provenance_studies, innovation_log, research_tools, ip_registry) | P0–P1 |
| §8k | Knowledge base & project specialist agents | 2 (knowledge_base, agent_console) | P1 |
| §8l | External data sources & evaluations | 2 (data_sources, evaluations) | P0–P1 |

**Key design decisions:**
- Per-constituent licensing (§8c.2): licenses apply to any individual constituent, not just project-wide. Project default + override + inheritance + conflict detection + mixed-licensing view
- Pre-configured data sources (§8l.4): PPP (OECD/World Bank/IMF), exchange rates (BIS/ECB/Fed), HDI (UNDP), WHO, NOAA, etc. — mocked for UI, wired when daemon available
- Agent nomenclature: software agents described as computational systems (no mind, no intent). Agent responses are "model assertions" requiring verification. Neuro-symbolic bridge: knowledge base provides symbolic substrate, LLM provides NL interface
- 24 open questions for the principal (§14)

### 3.4 Test count

- 63 tests in `cargo test --lib`
- Pre-existing doctest failure in `intent_bus.rs` (unrelated — run `--lib` only)

---

## 4. Consult documents (requirements sources)

Located in `C:\Projects\NLP\consult\`:

| Doc | Workstream | Status |
|-----|-----------|--------|
| `20260818_qualia-collaborative-ui-requirements.md` | A — Collaborative/ERP/PM | [pending] |
| `20260818_qualia-health-wellbeing-document-credentials-ui-requirements.md` | B — Health/Wellbeing | [pending] |
| `20260818_logic-modalities-audit.md` | C — Logic modalities | [in_progress] — P0 done, P1/P2 pending |
| `20260818_qualia-3d-animation-dataset-audio-ui-requirements.md` | D — 3D/Animation/Audio | [pending] |
| `20260818_qualia-ui-qapp-audit.md` | E — QApp browser | [pending] |
| `20260818_qualia-ui_handover.md` | Prior handover | reference |
| `20260817_qualia-ui-gap-audit.md` | Gap audit | reference |

All consult docs assume the QualiaDB engine is completed. qualia-ui surfaces are built as `present` (structure exists, backend not wired) with honest labels.

---

## 5. Pending workstreams (priority order)

### Workstream C (continued) — P1/P2 logic modalities
Still needed: STIT, Causal, Responsibility, Capacity, Delegation, Contract, Consensus, Meta-deontic, Value flow, Interaction governance, Identity fabric, Capability gap, LTL, CTL, ASP, Defeasible, Linear, Description, Dialectical, Abductive, Fuzzy, Probabilistic, Graph theory, Interval, Manifold 10D, Epistemic boundaries, Modal, Legal compose, Deontic compose. These can be added as new tabs in `logic_workbench.rs` or as sub-panels in the existing Evaluate Modality tab.

### Workstream A — Collaborative/ERP/PM
**Phase 1 DONE** (11 containers implemented). **Extended plan DONE** (`WORKSTREAM_A_IMPLEMENTATION_PLAN.md`, 74 containers across §8a–§8l).
**Next:** Begin implementing P0 items from the plan. Suggested order:
1. `agreement_builder` (§8a) — instrument-based agreement composition
2. `compensation_model` + `contribution_ledger` (§8b) — fair value calculator
3. `license_builder` + `obligation_tracker` (§8c) — differential licensing with per-constituent scope
4. `ip_registry` (§8j.4) — IP creation, inventor attribution
5. `data_sources` (§8l.1) — source registry, pre-configured sources
6. `disputes` + `complaints` + `corrections` (§8e)
7. `onboarding` + `bulk_import` (§8f)
8. `governance_meetings` + `conflict_of_interest` (§8g, §8h)
9. Knowledge base sensitivity-aware access + search (§8k)
10. Agent grounding + capability gating (§8k)

Each container follows the existing pattern: view module in `project_views/`, dispatch in `containers.rs`, seed in `projects.rs`, commands in `command_palette/commands.rs`.

**24 open questions** in plan §14 need principal decisions before/during implementation.

### Workstream B — Health/Wellbeing
New Health manifold with ~20 containers (Overview dashboard, Conditions, Lab Results, Medications, Mental Wellbeing, Hypotheses, Biometrics, Documents, Welfare Support, etc.). QECP credential PDFs.

### Workstream D — 3D/Animation/Audio
Real wgpu 3D surface, animation, complex dataset presentation, audio desk, spatial soundfields.

### Workstream E — QApp browser
QApp discovery/install/launch for 333 engine QApps.

---

## 6. Conventions and rules

### 6.1 Authorship
- All artefacts assigned to Timothy Charles Holborn
- Demo identity: `did:qualia:timothy_charles_holborn`
- Do not name third parties, fictional people, or the agent as author

### 6.2 Honesty
- Distinguish `live` (working), `present` (UI shell, backend pending), `partial`, `missing`
- Never claim backend execution, SPARQL execution, RDF extraction, or daemon integration unless actually implemented
- Mock results must be clearly labeled as mock

### 6.3 Code style
- `cargo fmt` before committing
- Compact code — collapse duplicate branches, avoid unnecessary nesting
- Don't add/remove comments unless asked
- Follow existing patterns (Closure::wrap + forget for events, inline CSS via set_css_text)
- Use `..Default::default()` for SeedContainer literals

### 6.4 WASM compatibility
- No platform-native C bindings or OS-specific dependencies
- All code must compile for `wasm32-unknown-unknown`
- Use `web_sys` and `js_sys` for browser APIs
- `set_timeout` is available via `super::interactions::set_timeout`

### 6.5 Verification
Always run after changes:
```powershell
cd C:\Projects\NLP\qualia-ui
cargo fmt
cargo test --lib
trunk build
```
PowerShell uses `;` not `&&` for command chaining.

---

## 7. Known issues

1. **Pre-existing `async_fn_in_trait` warnings** in `src/tool_chest/core/intent_bus.rs:375,386` — not blockers, do not fix unless asked
2. **Pre-existing doctest failure** in `intent_bus.rs` — run `cargo test --lib` only, not `cargo test`
3. **OOM risk** — the codebase has grown large; avoid reading entire files when targeted reads suffice. Use `grep` and `code_search` to find specific sections. The largest files are `logic_workbench.rs` (~1800 lines), `search_workbench.rs` (~1700 lines), `interactions.rs`, `container_views.rs`, `css.rs`

---

## 8. How to continue a new session

1. **Read this document first** — it contains everything needed to orient
2. **Check the todo list** — if the user provides one, continue from it; otherwise ask what to work on
3. **Use targeted reads** — grep for specific functions/patterns rather than reading whole files
4. **Verify after each change** — `cargo fmt && cargo test --lib && trunk build` (using `;` in PowerShell)
5. **Update `IMPLEMENTATION_TRACKER.md`** after completing each feature
6. **Keep responses concise** — long responses with full file contents contribute to OOM

### 8.1 Quick-start commands

```powershell
# Check compilation
cd C:\Projects\NLP\qualia-ui; cargo check

# Run tests
cd C:\Projects\NLP\qualia-ui; cargo test --lib

# Build WASM
cd C:\Projects\NLP\qualia-ui; trunk build

# Format
cd C:\Projects\NLP\qualia-ui; cargo fmt
```

### 8.2 Key files to know (read only if needed)

- `IMPLEMENTATION_TRACKER.md` — full feature inventory and status
- `SAVE_ARCHITECTURE.md` — save/persistence model + saved queries (§13)
- `src/browser/mod.rs` — app entry point and wiring
- `src/browser/command_palette.rs` — command list and dispatch (line ~540 for dispatch, ~620 for list)
- `src/tool_chest/core/registry.rs` — ManifoldSeed, SeedContainer definitions (line ~177)

---

## 9. Recent session work (2026-08-18)

### Session 1 (continued from prior)
- Fuzzy search in command palette
- Contextual RDF popover
- Search Workbench (4 modes + saved queries + query-as-container-source)

### Session 2
- Logic Workbench (9 reasoning tools)
- Tracker updated with 5 new consult workstreams (A-E)

### Session 3 (current)
- Workstream A Phase 1: 11 containers (9 project + 2 rights/wallet) implemented
- `project_views/` subdirectory with 9 modules
- `rights_views/` subdirectory with 3 modules (rights_tabs, wallet_tabs, mod)
- `container_inline_views.rs` for inline container views
- `containers.rs` dispatch updated for all new types
- `projects.rs` and `rights.rs` manifold seeds updated
- `command_palette/commands.rs` — 20+ new command entries
- Bug fix: Rust type mismatch in rights_tabs.rs and wallet_tabs.rs (pre-calculate color values, use `*` deref on tuple elements)
- Extended implementation plan created: `WORKSTREAM_A_IMPLEMENTATION_PLAN.md` (~1360 lines, 74 containers, §8a–§8l)
- Plan sections: agreements, compensation, licensing (with per-constituent scope), awards/tokens, disputes/complaints/corrections, onboarding/bulk admin, governance meetings, conflict of interest, ZK privacy controls, provenance studies/innovation/research/IP, knowledge base/specialist agents, external data sources/evaluations
- `IMPLEMENTATION_TRACKER.md` updated to reflect 74 containers and all sub-sections
- 24 open questions documented in plan §14

### Files created (session 3)
- `src/browser/project_views/mod.rs`
- `src/browser/project_views/kanban.rs`
- `src/browser/project_views/project_sheet.rs`
- `src/browser/project_views/budget.rs`
- `src/browser/project_views/cost_base.rs`
- `src/browser/project_views/deliverable.rs`
- `src/browser/project_views/review.rs`
- `src/browser/project_views/discussion.rs`
- `src/browser/project_views/roadmap.rs`
- `src/browser/project_views/commons.rs`
- `src/browser/rights_views/mod.rs`
- `src/browser/rights_views/rights_tabs.rs`
- `src/browser/rights_views/wallet_tabs.rs`
- `src/browser/container_inline_views.rs`
- `WORKSTREAM_A_IMPLEMENTATION_PLAN.md`

### Files modified (session 3)
- `src/browser/containers.rs` — dispatch for 11 new container types
- `src/browser/mod.rs` — module declarations for project_views, rights_views, container_inline_views
- `src/tool_chest/manifolds/projects.rs` — 9 new container seeds
- `src/tool_chest/manifolds/rights.rs` — 3 new container seeds
- `src/browser/command_palette/commands.rs` — 20+ new command entries
- `IMPLEMENTATION_TRACKER.md` — Workstream A status updated (74 containers, §8a–§8l)

### Key files for next session
- `WORKSTREAM_A_IMPLEMENTATION_PLAN.md` — the master plan, read §11 for priority sequencing
- `IMPLEMENTATION_TRACKER.md` — overall tracker, Workstream A section at line ~427
- `src/browser/project_views/` — existing pattern for new container views
- `src/browser/containers.rs` — dispatch pattern (match on container_type)
- `src/tool_chest/manifolds/projects.rs` — seed pattern (SeedContainer literals with ..Default::default())

---

_End of handover. Start here, then use targeted reads to continue._
