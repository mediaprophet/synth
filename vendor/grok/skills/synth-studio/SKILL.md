---
name: synth-studio
description: Produce Synth / Hypermedia assets from Grok. Use when making 3D models, images, SVG, audio, anatomy, kinetics (XPBD, NLA, IK, navmesh), GIS, N3/Q42/SPARQL, or any .vibe / .hmc / .10d package. Synth is the development tool; QualiaPortal is the visitor runtime. Search with synth_find.
---

# Synth Studio for Grok

Webizen AI Synth 0.0.41.11 is the development tool (produce / preview / compile). QualiaPortal plus the world host is the visitor runtime. Hypermedia is the studio (locus, manifold, radiance, surface, kinetics, geodesy). A `.vibe` names the work. Grok does not add a Rust compiler per species or organ.

**Find work with `synth_find`** (`q=xpbd`, `nla`, `ik`, `navmesh`, `camera`, `input`, `gis`, `osm`, `n3`, `sparql`, `hmc`). CLI: `synth find xpbd`. Then open `C:\github\synth\agent-instructions\<category>.md` (index `agent-instructions/README.md`; house rules `AGENTS.md`). Function-level index: `C:\github\hypermedia-production\audit-docs\FUNCTIONS.md`. Do not re-audit the kernel in place of calling a morphism that is already listed.

Counts must match: `OPERATIONS.len() == ontology::MORPHISMS.len() == 150`. Trust `AGENTS.md` and `operations_catalog.rs` over a connected MCP binary whose tool description still says 114 or 148.

Author logic as N3 (N3Logic + SHACL). Add scene assets as external HMC / `.10d` / `.q42` files; the host does not rebuild.

## MCP

When MCP is connected, start with `synth__synth_find`. Also: `synth__synth_ontology_morphisms`, `synth__synth_operations`, `synth__synth_operation_contracts` (paged: `category`, `id`, `q`, `offset`, `limit`, `detail=ids|summary|full`; default 40 summaries so Grok does not overflow), `synth__synth_produce_present`, `synth__synth_compile_q42`, `synth__synth_operator_status`, `synth__synth_production_rules`, `synth__synth_bookmarks`, `synth__synth_jobs`, `synth__synth_capabilities`, `synth__vibe_invoke`, `synth__vibe_catalog`, and QualiaDB graph tools (`query_sparql`, `query_graph`, `ingest_ontology`, `parse_rdf`). Search with `search_tool` for `synth`. `synth_operation_contracts` is the typed output recipe; do not guess encoders from a namespace. Seal sourced N3 to `.q42` with `synth_compile_q42` (`source_path` under `SYNTH_INPUT_ROOT` or inline `rdf_data`, create-only `out_q42` under `SYNTH_OUTPUT_ROOT`), then `query_sparql` with `vault_path`. Fetch http(s) with `synth_jobs` (`kind=fetch|ingest|compile`) into gitignored `.synth/inbox`; the operator ticks due jobs. Bookmark N3/RDF/SPARQL/GIS URLs with `synth_bookmarks` (`kind=gis` for OSM/DEM/STAC later); if the user pasted a docs page, add `data_endpoint`; if it needs a key, set `requires_api_key` and `action=set_secret` (writes gitignored `.synth/secrets.json`). Biosphere encyclopedia work belongs in Ecopunk. Non-fiction biosphere taxa need that evidence `.q42` before modelling (`synth_production_rules`; produce `domain=biosphere`). Operator shell: `synth operator` (Aether desktop tray on Windows/macOS/Linux with `native-ui`) and Studio `GET /operator`. Signed GitHub Releases: `synth update verify|apply` (`synth_update`). If no MCP tools exist, use the CLI below. Do not write `~/.grok/config.toml`. Project config lives in `vendor/grok/config.toml`. After a `target-prod` rebuild, `/mcps` reload.

Asset morphisms run through `capability.invoke` inside `synth_produce_present` (production AssetHost). `vibe_invoke` / `vibe_catalog` use a separate generic Poet snapshot.

## CLI (this machine)

Do not replace or kill `C:\github\synth\target-studio\debug\synth.exe` (live Studio MCP). Do not rebuild into `target` or `target-studio`. Produce and Grok-spawned MCP use the isolated preview binary:

```powershell
$synth = "C:\github\synth\target-prod\debug\synth.exe"
$stamp = "$(Get-Date -Format 'yyyyMMdd-HHmmss-ffffff')-$PID-$([guid]::NewGuid().ToString('N').Substring(0,8))"
$run = "C:\github\synth\target-prod\runs\$stamp"
New-Item -ItemType Directory -Force -Path (Split-Path $run) | Out-Null
& $synth produce --profile files --quality-profile production16 --source <FILE.vibe> --out "$run.hmc" --export-dir $run --present "$run.html"
Get-ChildItem $run -Filter *.10d | ForEach-Object {
  & $synth preview --asset $_.FullName --out "$run-preview-$($_.BaseName)"
}
```

`--out`, `--export-dir`, `--present`, and `preview --out` are create-only.

One morphism as JSON (typed encode; never infer from a namespace):

```powershell
C:\github\synth\vendor\grok\scripts\invoke.ps1 -Morphism hypermedia.manifold.revolve -ArgsJson '{"points":[[14,0],[22,12],[16,30]]}' -Encode mesh -Name revolve-demo
```

## Modelling tools to actually call

The four-line subset that used to live here is **not** the catalog. The catalog is **150** morphisms. Call `synth_find` then open `C:\github\synth\agent-instructions\<category>.md`. Grep `C:\github\hypermedia-production\audit-docs\FUNCTIONS.md` before saying a function does not exist.

A named organ (heart, lung, eye) is a vibe that lofts/revolves/booleans those parts. Do not call `hypermedia.manifold.compile_vertebrate_anatomy_10d` for a recognisable organ. That morphism currently emits an ellipsoid.

## Review before claiming the shape is right

1. Read exported `.png` / `.svg`.
2. Read `front-view-assumed.png`, `side-view-assumed.png`, `three-quarter-view-assumed.png` from each preview dir.
3. Serve the project workbench: `http://127.0.0.1:8787/project` (`$synth serve --port 8787 --project C:\github\anatomy`). Keep `http://127.0.0.1:8787/` as the QualiaDB demo studio.
4. Read `{project}/.synth/annotations.json` (tags, comments, ratings, reference URLs) before changing a model.
5. A file existing, HTTP 200, or a triangle count is not a visual review.
