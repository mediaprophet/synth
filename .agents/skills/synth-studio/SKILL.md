---
name: synth-studio
description: Author VibeScript assets, query the 12-category / 114-morphism ontology, invoke the comprehensive 63-tool QualiaDB 0.0.41 MCP suite, package deterministic .hmc containers, and present interactive HTML showcases in Antigravity IDE, Antigravity 2.0 Auxiliary Pane (<agent-embed>), and Browser Preview.
---

# Synth Studio — Agent Authoring, QualiaDB MCP & Presentation Skill

Synth is a deterministic VibeScript asset-production engine and application over QualiaDB (`0.0.41`), Etalon, Hypermedia, and Aether. Use this skill whenever authoring, inspecting, packaging, or visually presenting `.vibe` assets and `.hmc` containers in Antigravity IDE, Antigravity 2.0, or via the `vendor/antigravity-sdk-python` harness.

## 1. Inspecting Capabilities, Ontology & the Comprehensive QualiaDB MCP Suite

Before authoring a `.vibe` script, inspect the admitted runtime providers, categorical morphisms, and MCP tools:

### CLI Inspection & MCP Server
```powershell
./target/debug/synth.exe capabilities
./target/debug/synth.exe ontology
./target/debug/synth.exe ontology radiance
./target/debug/synth.exe operations
./target/debug/synth.exe mcp-tools   # Prints all 94 unified QualiaDB + Synth MCP tools as JSON
./target/debug/synth.exe mcp         # Runs the comprehensive JSON-RPC 2.0 MCP server over stdio
./target/debug/synth.exe env         # Bootstraps & inspects Synth's self-produced tray & studio environment
./target/debug/synth.exe serve       # Runs the local Synth Studio HTTP workbench on http://127.0.0.1:8787 (non-conflicting with Webizen 8080/4242/9001)
./target/debug/synth.exe tray        # Launches the resident Aether Studio & Tray environment + local HTTP server (--features native-ui)
```

### Connecting via `vendor/antigravity-sdk-python` (`McpStdioServer`)
All Synth and QualiaDB MCP tools use FastMCP-compatible Python-identifier-safe names (`^[a-zA-Z_][a-zA-Z0-9_]*$`):
```python
from google.antigravity import Agent, AgentConfig
from google.antigravity.mcp import McpStdioServer

server = McpStdioServer(
    name="synth-qualia",
    command="c:/github/synth/target/debug/synth.exe",
    args=["mcp"],
)
```

### Synth Authoring & Presentation Tools
Both dotted (`synth.*`) and FastMCP Python-identifier-safe (`synth_*`) tool names are supported:
- `synth_inspect` / `synth.inspect`: Returns the active session context token, producer (`files@1`), current resources, and admitted operations.
- `synth_ontology_categories` / `synth.ontology.categories`: Lists all 12 canonical categories (`locus`, `manifold`, `surface`, `radiance`, `glyph`, `layout`, `kinetics`, `acoustics`, `electromagnetics`, `geodesy`, `discourse`, `chronicle`) and their `NQuin48` triple counts.
- `synth_ontology_morphisms` / `synth.ontology.morphisms`: Lists Domain -> Codomain signatures, `AgenticUtility` teleology, and parameter schemas (optionally filtered by `{"category": "<name>"}`).
- `synth_produce_present`: Standalone MCP tool that compiles a `.vibe` script, optionally writes `.hmc` / export files, and generates a self-contained HTML presentation (`synth.presentation-html/v1`).
- `synth_environment_bootstrap` / `synth.environment.bootstrap`: Bootstraps Synth's self-produced Aether Tray & Studio environment (`examples/tray_environment.vibe`), probes Webizen Desktop presence (`127.0.0.1:8080` / `ws://127.0.0.1:4242`), and scans the local LLM model vault (`QUALIA_LLM_VAULT` / `C:\LLM_files` / `C:\LLM_Models`) for `Qwen` and `Gemma 4` `.p64`/`.gguf` models.
- `synth_package_present` / `synth.package.present`: Authenticated host MCP tool that renders a self-contained HTML showcase in the native output directory.

### Integrated QualiaDB `0.0.41` Engine Tools (63 Tools)
All 63 tools from `qualia_mcp::stable_mcp_tools()` are bridged into `synth::mcp_full` and `SynthAgentBackend::with_qualia_engine_tools()`, including:
- **VibeScript & Etalon-Vibe SDK**: `vibe_check`, `vibe_compile`, `vibe_format`, `vibe_invoke`, `vibe_eval`, `vibe_catalog`, `etalon_vibe_sdk`, `wgsl_forge`, `poet_authoring`, `poet_studio_domain`, `poet_artifact_lifecycle`
- **Living-World Anatomy & Omniversal Cosmic Coordinates (`0.0.41`)**: `living_world_anatomy` (biped/quadruped/avian/aquatic/arthropod/flora rigs, callout pins, wing/fin/gait cycles, species morph), `cosmic_coordinates` (5-tier log-scale coordinate hierarchy from sub-millimetre to Gpc, Keplerian orbit propagation, cosmological redshift, 60-bit `NQuin` packing)
- **Semantic Graph, Modalities & SHACL**: `query_sparql`, `codebase_audit`, `evaluate_modality`, `evaluate_logic_rules`, `symbolic_logic_infer`, `validate_shacl`, `shacl_route`
- **Simulation, Geospatial, Vision, Audio & Science**: `sim_soil_compaction`, `sim_trench_excavation`, `sim_space_weather_eval`, `sim_fire_ecology_eval`, `geospatial_transform`, `geospatial_hilbert`, `geospatial_strata`, `computer_vision`, `vision_rppg`, `audio_features`, `audio_spatial_hrtf`, `geometry_compute`, `ode_solve`, `cas_evaluate`, `quantum_circuit_sim`, `quantum_chemistry_eval`

## 2. Producing & Packaging Deterministic `.hmc` Assets

```powershell
./target/debug/synth.exe produce `
  --source ./examples/production.vibe `
  --out ./target/production.hmc `
  --export-dir ./target/production-export `
  --present ./target/production-showcase.html
```

- Every output path (`--out`, `--export-dir`, `--present`) is strictly **create-only** (`ExpectedFile::Absent`). Never overwrite existing packages or presentation files.
- Strict replay is verified automatically before saving.

## 3. Presenting Work in Antigravity IDE, Antigravity 2.0 & Browser Preview

Synth natively generates self-contained HTML showcases (`synth.presentation-html/v1`) styled with Antigravity's allowlisted Tailwind runtime (`https://www.gstatic.com/antigravity/web/dev/tailwindcss.min.js`) and semantic theme variables (`--background`, `--card`, `--foreground`, `--muted-foreground`, `--border`).

### A. Generate a Standalone Presentation HTML from any `.hmc` or `.vibe`
```powershell
# From an existing .hmc package:
./target/debug/synth.exe present --reopen ./target/production.hmc --out ./target/showcase.html

# Or directly from VibeScript source:
./target/debug/synth.exe present --source ./examples/production.vibe --out ./target/showcase.html
```

### B. Generate a 9-Angle `.10d` Volumetric Turntable Preview
When built with `--features preview`:
```powershell
./target/debug/synth.exe preview --asset ./target/production-export/extrude.10d --out ./target/extrude-preview
```
This writes the 9 offscreen 2048x2048 PNG views, `review-receipt.json`, and an interactive `index.html` camera-orbit turntable viewer inside `./target/extrude-preview/index.html`.

### C. Surfacing in Antigravity Auxiliary Pane / Chat / Browser
1. **Side Pane Artifact (`UserFacing: true`)**: Copy or write the generated `.html` showcase into `<appDataDir>/brain/<conversation-id>/showcase.html` with `ArtifactMetadata { UserFacing: true }` so the user can open and interact with the multi-modal gallery (SVG, PNG, WAV audio player, `.10d` telemetry, and VibeScript source cells) in the Antigravity Auxiliary Pane.
2. **Inline Chat Widget**: Emit `<agent-embed src="file:///<artifact_path>/showcase.html"></agent-embed>` when presenting compact visual results inline.
3. **Browser Preview**: Open the generated `.html` file directly in the Antigravity IDE Browser Preview.
