---
name: synth-studio
description: Author and review local VibeScript assets with Synth, Hypermedia, QualiaDB and Etalon; package deterministic HMC files and inspect native PNG, SVG, mesh and animation outputs. Supports local Codex, Grok and Antigravity clients through the same production host.
---

# Local Synth production

Read `C:\github\synth\AGENTS.md` before claiming a tool is absent or authoring a `.vibe`. Category tables live in `C:\github\synth\agent-instructions/<category>.md`. The 150-operation catalog and Hypermedia's `audit-docs/FUNCTIONS.md` are authoritative. QualiaDB owns codecs and geometry; Hypermedia is the studio; Synth is the producer; Etalon supplies render and signal providers. Reuse the existing operation that performs the requested work. Find with `synth_find`.

- **Holographic 10D v2 Primacy (`HOLO` `0x0306`):** Every sealed `.10d` carries `FIELD_KIND_HOLOGRAPHIC_PHASE_SPACE` (`0x0306`, `M64HoloRecord` + `EulerianWavefieldRecord`), scale-space harmonic LODs $k=0..K$, Hamiltonian stationary-action trajectories ($\delta S = 0$), and two-tier scene constituency (`AREF`/`IOVR`/`INST`).
- **7-Layer Hierarchy (`L0`–`L6`) & No-ASCII:** `yaml-ld-q42` v2 manifests map across `L0`–`L6` (`L6` = `L6QuantumPhysics`, encompassing quantum dynamics, quantum physics, and quantum chemistry). Use UTF-8/UTF-16 text and CBOR-LD (`CBLD`) binary sections.
- **3D Scene/Asset Conversion & Licensing (`synth_convert_3d`):** Convert external `.glb`/`.gltf`/`.obj`/`.stl` files via `synth convert-3d` / MCP `synth_convert_3d` with mandatory `source-asset-licensing.n3` provenance (`spdx_license`, `attribution`, `source_uri`), sealing `CBLD` + `PROV` + `HOLO` (`0x0306`). Validate deliverables with `synth_validate_model` and inspect packages with `synth_package_inspect`.

## Select and discover the executable

Do not overwrite or kill live MCP binaries under `target` or `target-studio`. This integration selects `C:\github\synth\target-prod\debug\synth-mcp.exe`. Check the executable identity and runtime `operations`, `mcp-tools`, and `capabilities`; counts in older documentation do not establish missing functions.

Use `synth_find`, then `synth_operations`, `synth_ontology_morphisms` and `synth_operation_contracts` to discover execution host, argument contracts, output recipes and handle lifetime. Hypermedia execution belongs to `synth_produce_present` / CLI `produce`. Generic Poet `vibe_invoke` is a different execution host and may return an explicit production routing error. Never infer an encoder from an operation namespace.

## Produce with retained inputs

Supply MCP `inputs:[{key,kind,path}]` under the configured `SYNTH_INPUT_ROOT`. Source uses those retained keys. In-memory mesh/image handles live for one source evaluation; encoded resources and HMC packages are durable. Use explicit fields such as `assembly.container_10d` or `sheet.png` from the operation contract.

MCP `synth_produce_present` accepts `source`, `inputs`, `out_hmc`, `export_dir`, `present_html`, `receipt_json`, `preview_assets`, and `quality_profile`. Output names are create-only basenames under `SYNTH_OUTPUT_ROOT`. `preview_assets` contains exported `.10d` keys and requires `export_dir`. A native review failure preserves completed exports and reports an error; inspect the separate review completion receipt.

CLI and common helper:

```powershell
$synth = 'C:\github\synth\target-prod\debug\synth.exe'
& $synth produce --source <FILE.vibe> --input reference.png png <INPUT.png> --out <NEW.hmc> --export-dir <NEW_DIR>
C:\github\synth\scripts\invoke-asset.ps1 -Source <FILE.vibe> -InputManifest <INPUTS.json> -Preview
```

`preview` is the default bounded production profile. `production16` raises explicit resource/work budgets and supports the retained RGBA16 UNORM path; it does not promise arbitrary HDR floats or unlimited raster sizes. Unsupported precision-changing operations refuse rather than silently quantize. The `png-sequence` capture profile packages exact-rational timing, colour interpretation and leaf PNG/WAV resources.

## Preserve data during revisions

Imported supported GLB normals, UVs, tangents, material assignments and textures survive seal/package/reload. Unsupported GLB features are diagnosed. Static scene hierarchy and mirrored transforms are applied. Texture payloads use the existing content-addressed HMC resource keys and must travel with the `.10d`.

Geometry edits that cannot transport imported attributes refuse by default. `attribute_policy:"discard"` explicitly creates a new geometry-only revision; rebind or bake its surface. Never silently discard attributes or reuse a textured atlas after changing UVs. Untextured unwrap retains remapped geometry and supported surface attributes. A specialist BSDF preview is not evidence that a native mesh shader implements that closure; obey the material compiler's supported binding contract.

Physical terrain records retain metre samples, spacing, origin and coordinate frame independently from relief colours. ENU Z-up terrain needs an appropriate camera; do not rotate physical data to hide a framing error. Custom animation can bind a verified retained SKL1 skeleton/weights/clips to matching mesh vertices; inspect actual posed frames.

## Inspect actual outputs

Read `{project}/.synth/annotations.json` before revisions. Inspect exported PNG/SVG and the three native views: `front-view-assumed.png`, `side-view-assumed.png`, `three-quarter-view-assumed.png`.

```powershell
& $synth preview --asset <MESH.10d> --out <NEW_REVIEW_DIR> --hmc <PACKAGE.hmc> --asset-key outputs/<MESH.10d>
& $synth preview --asset <MESH.10d> --out <NEW_SHOT_DIR> --shot <SHOT.json> --hmc <PACKAGE.hmc> --asset-key outputs/<MESH.10d>
```

Shots retain camera, lighting, output size, optional occlusion/shadow/probe controls and frame times. Supply HMC/key for textured assets. Review several animation poses, texture seams, material response and intended scientific/design relationships. A file, triangle count, HTTP 200 or passing test is not visual acceptance. Record source/input/package hashes, shot, inspected views, defects and decision in the review receipt.

The existing studio remains `http://127.0.0.1:8787/`; project workbench is `/project`; QualiaPortal shell is `/qualia/js/qualia-shell.js`. Do not replace the demo studio to show a project.

## Client adapters

- Codex: `vendor/codex/README.md` and `config.toml`; local stdio with explicit local environment and scoped roots. Managed ChatGPT Work is a separate host and does not automatically consume laptop configuration.
- Grok: `vendor/grok`; its helper delegates to `scripts/invoke-asset.ps1` with explicit output field/encoder or complete source.
- Antigravity: `vendor/antigravity-sdk-python/examples/synth_local.py`; import `McpStdioServer` from `google.antigravity.types`. Optional `<agent-embed>` is presentation only; ordinary exported files remain usable by every client.

Registration, actual client discovery, production, review and reconnection are separate checks. Configuration files alone do not prove a working client connection.
