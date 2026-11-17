---
name: synth-studio
description: Produce Synth / Hypermedia assets from Grok. Use when making 3D models, images, SVG, audio, anatomy, or any .vibe / .hmc / .10d package. Drive locus, manifold, radiance, and surface morphisms, then review PNG previews.
---

# Synth Studio for Grok

Synth is the producer. Hypermedia is the studio (Illustrator + Photoshop + Blender + CAD + terrain). A `.vibe` names the work. Grok does not add a Rust compiler per species or organ.

**Full catalog (150 morphisms) and Hypermedia capabilities:** read `C:\github\synth\AGENTS.md` before claiming a tool is missing. Function-level index: `C:\github\hypermedia-production\audit-docs\FUNCTIONS.md` (generated function registry; inspect its timestamp). Do not re-audit the kernel in place of calling a morphism that is already listed.

Counts must match: `OPERATIONS.len() == ontology::MORPHISMS.len() == 150`. Trust `AGENTS.md` and `operations_catalog.rs` over a connected MCP binary whose tool description still says 114 or 148.

## MCP

When MCP is connected, tools include `synth__synth_ontology_morphisms`, `synth__synth_operations`, `synth__synth_operation_contracts`, `synth__synth_produce_present`, `synth__synth_capabilities`, `synth__vibe_invoke`, `synth__vibe_catalog`. Search with `search_tool` for `synth`. `synth_operation_contracts` is the typed output recipe; do not guess encoders from a namespace. If no MCP tools exist, use the CLI below. Do not write `~/.grok/config.toml`. Project config lives in `vendor/grok/config.toml`.

Asset morphisms run through `capability.invoke` inside `synth_produce_present` (production AssetHost). `vibe_invoke` / `vibe_catalog` use a separate generic Poet snapshot.

## CLI (this machine)

Do not replace or kill `C:\github\synth\target-studio\debug\synth.exe` (live Studio MCP). Do not rebuild into `target` or `target-studio`. Produce and Grok-spawned MCP use the isolated preview binary:

```powershell
$synth = "C:\github\synth\target-prod\debug\synth.exe"
$stamp = "$(Get-Date -Format 'yyyyMMdd-HHmmss-ffffff')-$PID-$([guid]::NewGuid().ToString('N').Substring(0,8))"
$run = "C:\github\synth\target-prod\runs\$stamp"
New-Item -ItemType Directory -Force -Path (Split-Path $run) | Out-Null
& $synth produce --profile files --source <FILE.vibe> --out "$run.hmc" --export-dir $run --present "$run.html"
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

The four-line subset that used to live here is **not** the catalog. The catalog is **150** morphisms. Open `C:\github\synth\AGENTS.md` (Blender/CAD mapping table + full ID/args tables). Grep `C:\github\hypermedia-production\audit-docs\FUNCTIONS.md` before saying a function does not exist.

A named organ (heart, lung, eye) is a vibe that lofts/revolves/booleans those parts. Do not call `hypermedia.manifold.compile_vertebrate_anatomy_10d` for a recognisable organ. That morphism currently emits an ellipsoid.

## Review before claiming the shape is right

1. Read exported `.png` / `.svg`.
2. Read `front-view-assumed.png`, `side-view-assumed.png`, `three-quarter-view-assumed.png` from each preview dir.
3. Serve the project workbench: `http://127.0.0.1:8787/project` (`$synth serve --port 8787 --project C:\github\anatomy`). Keep `http://127.0.0.1:8787/` as the QualiaDB demo studio.
4. Read `{project}/.synth/annotations.json` (tags, comments, ratings, reference URLs) before changing a model.
5. A file existing, HTTP 200, or a triangle count is not a visual review.
