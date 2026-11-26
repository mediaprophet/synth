# Synth vendor adapter for Grok

Antigravity’s SDK is `vendor/antigravity-sdk-python`. This directory is the Grok equivalent: MCP wiring, a Grok skill, and a JSON invoke helper. It does not add a modeller. Hypermedia morphisms stay in Synth; Grok calls them.

Catalog: **400** morphisms (`OPERATIONS.len() == ontology::MORPHISMS.len() == 400`, 150 baseline + 250 expansion). Canonical contract: `C:\github\synth\AGENTS.md`. Public identity: **0.0.41.21**.

## What this gives Grok

- Project-scoped MCP: Grok spawns `synth mcp` over stdio and gets `synth_ontology_morphisms`, `synth_operations`, `synth_operation_contracts`, `synth_produce_present`, plus the QualiaDB tool suite.
- A skill that tells Grok to use `locus` / `manifold` / `radiance` / `surface`, then look at PNG previews.
- `scripts/invoke.ps1` for a single morphism as JSON when you want a one-shot produce without hand-writing a long vibe.

## Enable in this workspace

Copy or merge `config.toml` into the Grok project config:

```
C:\github\.grok\config.toml
C:\github\synth\.grok\config.toml
```

Merge only the Synth server entry, preserving unrelated project settings. In a Grok session rooted at `C:\github`, the server name is `synth`. Tools are `synth__synth_produce_present`, `synth__synth_ontology_morphisms`, `synth__synth_operation_contracts`, and so on.

Do not write this into `~/.grok/config.toml` unless you want Synth MCP on every Grok project.

Reload MCP (`/mcps` or a new session) after the files land so the spawned child is `target-prod` (400 morphisms, `synth-mcp.exe`). A session that still talks to `target-studio` will advertise a stale catalog.

## Binary

Grok must spawn a *new* stdio `synth mcp` child. Leave the existing Studio MCP process alone:

```
C:\github\synth\target-studio\debug\synth.exe
```

Do not rebuild into `target` or `target-studio`. Default command in `config.toml` is the isolated preview CLI, measured 2026-11-17:

```
C:\github\synth\target-prod\debug\synth-mcp.exe mcp
```

`synth operations` on that binary reports `"count": 150`. `target-gap-review` still reports 148; `target-studio` is the locked live Studio process.

Rebuild the isolated binary without touching the locked MCP exe:

```powershell
$env:CARGO_TARGET_DIR = "C:\github\synth\target-prod"
Set-Location C:\github\synth
cargo build --bin synth --features preview
```

## Review loop

1. Call morphisms from a `.vibe` via `synth_produce_present`, or `scripts/invoke.ps1`.
2. `synth preview` writes PNGs.
3. Grok reads the PNGs with `read_file`.
4. The user reviews and comments at http://127.0.0.1:8787/project
5. Grok reads `{project}/.synth/annotations.json` before changing a model.

## Heart / organs

Do not use `hypermedia.manifold.compile_vertebrate_anatomy_10d` as the modeller for a named organ. Its default comparative route emits an ellipsoid; richer spec routes are separate. Build organs with `locus`, `extrude`, `revolve`, `loft`, `boolean_3d`, `sweep_assembly_profile`, then `compile_10d`.

## Typed calls and retained inputs

The invocation helper now delegates to `scripts/invoke-asset.ps1`. It never guesses
return types from a namespace. Select `-ResultField container_10d` for an assembly,
or `-Encode rgba` for an RGBA image. Use `-Source FILE.vibe` for multiple operations
and `-InputManifest FILE.json` for an array of `{key,kind,path}` retained inputs.
`-Preview` generates native review images. Inspect the current `synth_operation_contracts`
MCP tool before choosing a recipe. Output filenames are unique and create-only.

The local MCP input/output roots are explicit in both configuration templates.
Actual client discovery and output inspection are required after registration.
