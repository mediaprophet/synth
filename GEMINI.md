# Webizen AI Synth -- Antigravity digest

Read `AGENTS.md` in this directory before claiming a tool is missing or writing a `.vibe`. This file is the always-on digest so Antigravity stays inside its rules budget.

**Morphism tables live in `agent-instructions/<category>.md`.** Open only the file you need. Index: `agent-instructions/README.md`. Do not load every category file.

**Webizen AI Synth 0.0.41.13 is the development tool** (produce, preview, compile). QualiaPortal plus the world host is the visitor runtime. Do not mint a per-scene Rust file. Do not treat Synth as the player.

## First session

1. Call MCP `synth_find` with `q=` for the work (`xpbd`, `nla`, `ik`, `navmesh`, `camera`, `input`, `gis`, `osm`, `n3`, `sparql`, `hmc`).
2. Open `agent-instructions/<category>.md` for the returned id. House rules and DCC mapping stay in `AGENTS.md`.
3. Invoke morphisms with `capability.invoke` **inside** `synth_produce_present` source. `vibe_invoke` is a different Poet snapshot.
4. Inspect returned PNG / SVG / native review files. A file existing is not a visual review.

CLI: `synth find xpbd`. Catalog tools are paged (default 40 summaries; pass `category`, `id`, `q`, `offset`, `limit`, `detail=ids|summary|full`). Count is **150**. Trust `AGENTS.md` over a stale MCP binary that still says 114 or 148.

## MCP (Antigravity names are unprefixed)

Workspace config: `.agents/mcp_config.json`. Plugin: `.agents/plugins/synth-qualia/`. SDK adapter: `vendor/antigravity/synth_local.py`. Global: `~/.gemini/config/mcp_config.json`.

Spawn `C:\github\synth\target-prod\debug\synth-mcp.exe mcp` with `SYNTH_INPUT_ROOT=C:\github` and `SYNTH_OUTPUT_ROOT=C:\github\synth\production`. Do not rebuild `target` or `target-studio`. Do not kill live MCP children.

| Job | Tool |
|---|---|
| Find | `synth_find` |
| Produce HMC / `.10d` | `synth_produce_present` (`quality_profile=production16`) |
| N3 -> Q42 | `synth_compile_q42` then `query_sparql` `vault_path` |
| SPARQL / RDF | `query_sparql`, `query_graph`, `ingest_ontology`, `parse_rdf` |
| Bookmark GIS/N3 | `synth_bookmarks` (`kind=gis\|rdf\|sparql`) |
| Fetch | `synth_jobs` (`kind=fetch\|ingest\|compile`) |
| Contracts | `synth_operation_contracts` |
| Validate Model (10D v2 / L0-L6) | `synth_validate_model` |
| Inspect `.hmc` / `.10d` / `.q42` | `synth_package_inspect` |
| Convert 3D (`.glb`/`.obj`/`.stl` -> `.10d` v2 + license `.q42`) | `synth_convert_3d` |
| Audit Registry | `synth_audit_index` |
| Operator | `synth_operator_status` / `GET /operator` |

Present HTML is Antigravity-ready (`<agent-embed>`, Auxiliary Pane). Author logic as **N3**. Add scene assets as external HMC / `.10d` / `.q42`.
- **Holographic 10D v2 Primacy:** Every `.10d` carries `HOLO` (`0x0306`, `M64HoloRecord` + `EulerianWavefieldRecord`), scale-space harmonic LODs $k=0..K$, Hamiltonian stationary-action trajectories ($\delta S = 0$), and two-tier constituency (`AREF`/`IOVR`/`INST`).
- **7-Layer Hierarchy & No-ASCII:** `yaml-ld-q42` v2 manifests declare `L0`–`L6` (`L6` = `L6QuantumPhysics` covering quantum dynamics, quantum physics, and quantum chemistry). Use UTF-8/UTF-16 text and CBOR-LD (`CBLD`) binary sections.
- **Orgs, Human Rights & Licensing:** Made-world things live under `C:\github\poiesis\<thing-slug>\` (`qualia-poiesis`); biosphere taxa live under `C:\github\eco-intent\<species-slug>\` (`intenteco`). External 3D conversions must validate against `source-asset-licensing.n3` via `synth_convert_3d`. Qualia repositories are strictly `CC BY-NC-ND 4.0` protecting the moral and material interests of the author (UDHR Art. 27 / ICESCR Art. 15). Never use the word `"sovereign"` or `"self-sovereign"`; use human-rights agreement vocabulary (**human rights**, **human dignity**, **human agency**, **natural person**, **moral rights**). House rules: `AGENTS.md`. Category files: `agent-instructions/<category>.md`.
