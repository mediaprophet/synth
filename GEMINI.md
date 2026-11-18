# Webizen AI Synth -- Antigravity digest

Read `AGENTS.md` in this directory before claiming a tool is missing or writing a `.vibe`. This file is the always-on digest so Antigravity stays inside its rules budget.

**Morphism tables live in `agent-instructions/<category>.md`.** Open only the file you need. Index: `agent-instructions/README.md`. Do not load every category file.

**Webizen AI Synth 0.0.41.12 is the development tool** (produce, preview, compile). QualiaPortal plus the world host is the visitor runtime. Do not mint a per-scene Rust file. Do not treat Synth as the player.

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
| Operator | `synth_operator_status` / `GET /operator` |

Present HTML is Antigravity-ready (`<agent-embed>`, Auxiliary Pane). Author logic as **N3**. Add scene assets as external HMC / `.10d` / `.q42`. House rules: `AGENTS.md`. Category files: `agent-instructions/<category>.md`.
