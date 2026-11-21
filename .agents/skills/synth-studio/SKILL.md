---
name: synth-studio
description: Produce Synth / Hypermedia assets from Antigravity. Use when making 3D models, images, SVG, audio, anatomy, kinetics (XPBD, NLA, IK, navmesh), GIS, N3/Q42/SPARQL, or any .vibe / .hmc / .10d package. Synth is the development tool; QualiaPortal is the visitor runtime. Search with synth_find, then open agent-instructions/<category>.md.
---

# Synth Studio for Antigravity

Webizen AI Synth 0.0.41.14 is the development tool (produce / preview / compile). QualiaPortal plus the world host is the visitor runtime. Hypermedia is the studio. A `.vibe` names the work. Do not add a Rust compiler per species or organ.

Always-on digest: `C:\github\synth\GEMINI.md`. House rules: `C:\github\synth\AGENTS.md`. **Category tables:** `C:\github\synth\agent-instructions/<category>.md` (index `agent-instructions/README.md`). Open only the category you are producing.

**Find work with `synth_find`** (`q=xpbd`, `nla`, `ik`, `navmesh`, `camera`, `input`, `gis`, `osm`, `n3`, `sparql`, `hmc`). CLI: `synth find xpbd`. Then open the matching category file. Function-level index: `C:\github\hypermedia-production\audit-docs\FUNCTIONS.md`.

Counts must match: `OPERATIONS.len() == ontology::MORPHISMS.len() == 150`. Trust `AGENTS.md` over a connected MCP binary whose tool description still says 114 or 148.

Author logic as N3 (N3Logic + SHACL). Add scene assets as external HMC / `.10d` / `.q42` files; the host does not rebuild.
- **Holographic 10D v2 Primacy (`HOLO` `0x0306`):** Every sealed `.10d` carries `FIELD_KIND_HOLOGRAPHIC_PHASE_SPACE` (`0x0306`, `M64HoloRecord` + `EulerianWavefieldRecord`), scale-space harmonic LODs $k=0..K$, Hamiltonian stationary-action trajectories ($\delta S = 0$), and two-tier constituency (`AREF`/`IOVR`/`INST`). Analytic `.10d` containers without explicit triangle soup synthesize a 12-triangle proxy lens on decode for Etalon's `< 32`-step raymarcher.
- **7-Layer Computational Hierarchy (`L0`–`L6`) & No-ASCII:** `yaml-ld-q42` v2 manifests map across `L0`–`L6` (`L6` = `L6QuantumPhysics`, encompassing quantum dynamics, quantum physics, and quantum chemistry). Use UTF-8/UTF-16 text and CBOR-LD (`CBLD`) binary sections.
- **3D Conversion & Licensing (`synth_convert_3d`):** Convert `.glb`/`.gltf`/`.obj`/`.stl` assets or scenes via `synth convert-3d` / MCP `synth_convert_3d` with mandatory `source-asset-licensing.n3` metadata (`spdx_license`, `attribution`, `source_uri`), sealing `CBLD` + `PROV` + `HOLO` (`0x0306`). Validate deliverables with `synth_validate_model` and inspect containers with `synth_package_inspect`.
- **Google Colab Ultra Burst (`synth_colab` + `colab-mcp`):** For heavy 24GB+ VRAM / 83GB High-RAM jobs (`trellis2_image_to_3d`, `trellis2_multiview_to_3d` for `microsoft/TRELLIS.2-4B` O-Voxel + 4K PBR, `hunyuan3d_shape_and_pbr` for `tencent/Hunyuan3D-2.1` DiT shape & delighted PBR re-texturing, `partcrafter_decompose_3d` for compositional multi-part `AREF`/`INST` decomposition, `unirig_auto_skeleton` for `SKL1`/`SKL2` auto-rigging, or `ontology_compile_q42` for online/BioPortal OWL/RDF `.q42` compilation via the Linux `synth` CLI), use `synth_colab` (`action=status|configure|routines|submit|poll|ingest|colab_mcp_cells|export_notebook`) paired with `colab-mcp` (`uvx git+https://github.com/googlecolab/colab-mcp`). Call `synth_colab action=routines` for the model selection decision matrix. `synth_colab action=ingest` verifies SHA-256 receipts and seals Holographic `.10d` v2 (`HOLO` `0x0306` + `PRV1` + `.ovoxel.json` `M64HoloRecord`/`VOX0`) and `.q42` volumes locally.
- **Anatomical & Pathological Topology (`synth_validate_model` / `synth_bioportal`, [`agent-instructions/species.md`](../../agent-instructions/species.md)):** Enforce cavity SDF containment ($\phi_{\text{cavity}} \le -\epsilon$ + serous `suspended_by_id`), intrinsic radial tissue depth $d \in [0, 1]$ (`periosteum_capsule` $[0,0.08] \to$ `dermis_epidermis` $[0.92,1.0]$), barycentric UV entheses + polyaxial retinacular pulleys, 3-stage `ShEx` (`shex_nodes` cyclic loops & streaming `.hmc` `LoadedHmcPayload|WireframeStub`) $\to$ `N3Logic` $\to$ `SHACL` (`topological_layers`) validation, and 4-regime pathology (adipose $V_{\text{cell}}/\rho_{\text{cell}}$ with `retinacula cutis` septa, `"displace_only"` cysts, `DOID`/`HP`-authorized invasive neoplasms, and 0-mesh 2D reaction-diffusion rashes).
- **Orgs:** Made-world things live in `C:\github\poiesis\<thing-slug>\` (`https://github.com/qualia-poiesis`); biosphere species live in `C:\github\eco-intent\<species-slug>\` (`https://github.com/intenteco`).

## MCP (unprefixed names)

Workspace config: `.agents/mcp_config.json`. Plugin: `.agents/plugins/synth-qualia/`. SDK: `vendor/antigravity/synth_local.py`. Global: `~/.gemini/config/mcp_config.json`.

Start with `synth_find`. Also: `synth_ontology_morphisms`, `synth_operations`, `synth_operation_contracts` (paged: `category`, `id`, `q`, `offset`, `limit`, `detail=ids|summary|full`; default 40 summaries), `synth_produce_present`, `synth_compile_q42`, `synth_operator_status`, `synth_production_rules`, `synth_bookmarks`, `synth_jobs`, `synth_capabilities`, `vibe_invoke`, `vibe_catalog`, and QualiaDB graph tools (`query_sparql`, `query_graph`, `ingest_ontology`, `parse_rdf`).

Asset morphisms run through `capability.invoke` inside `synth_produce_present` (production AssetHost). `vibe_invoke` / `vibe_catalog` use a generic Poet snapshot.

Seal sourced N3 to `.q42` with `synth_compile_q42` (`source_path` under `SYNTH_INPUT_ROOT` or inline `rdf_data`, create-only `out_q42` under `SYNTH_OUTPUT_ROOT`), then `query_sparql` with `vault_path`. Bookmark GIS/N3 with `synth_bookmarks` (`kind=gis`). Non-fiction biosphere taxa need evidence `.q42` before modelling (`synth_production_rules`).

Present HTML uses Antigravity allowlisted Tailwind and `<agent-embed>` for the Auxiliary Pane.

## CLI (this machine)

Do not replace or kill `C:\github\synth\target-studio\debug\synth.exe`. Do not rebuild into `target` or `target-studio`. Produce and spawned MCP use:

```powershell
$synth = "C:\github\synth\target-prod\debug\synth.exe"
$stamp = "$(Get-Date -Format 'yyyyMMdd-HHmmss-ffffff')-$PID-$([guid]::NewGuid().ToString('N').Substring(0,8))"
$run = "C:\github\synth\target-prod\runs\$stamp"
New-Item -ItemType Directory -Force -Path (Split-Path $run) | Out-Null
& $synth produce --profile files --quality-profile production16 --source <FILE.vibe> --out "$run.hmc" --export-dir $run --present "$run.html"
```

`--out`, `--export-dir`, `--present`, and `preview --out` are create-only.

## Modelling

The catalog is **150** morphisms. Call `synth_find`, then open `agent-instructions/<category>.md`. Grep `audit-docs/FUNCTIONS.md` before saying a function does not exist.

A named organ (heart, lung, eye) is a vibe that lofts/revolves/booleans those parts. Do not call `hypermedia.manifold.compile_vertebrate_anatomy_10d` for a recognisable organ. That morphism currently emits an ellipsoid.

## Review before claiming the shape is right

1. Read exported `.png` / `.svg`.
2. Read `front-view-assumed.png`, `side-view-assumed.png`, `three-quarter-view-assumed.png` from each preview dir.
3. Serve the project workbench: `http://127.0.0.1:8787/project`.
4. Read `{project}/.synth/annotations.json` before changing a model.
5. A file existing, HTTP 200, or a triangle count is not a visual review.
