# Antigravity adapter for Webizen AI Synth

Wires Google Antigravity (IDE, App, CLI, and the Python SDK) to the local Synth MCP server. Synth is the development tool (produce / preview / compile). QualiaPortal is the visitor runtime. Catalog: **150** morphisms. Canonical contract: `C:\github\synth\AGENTS.md`. Always-on digest: `C:\github\synth\GEMINI.md`. Category tables: `C:\github\synth\agent-instructions\<category>.md`.

This directory is the adapter. The SDK source is `vendor/antigravity-sdk-python` (also installable as `google-antigravity`). The modeller stays in Hypermedia; Antigravity calls it.

## What this gives Antigravity

- Workspace MCP: `.agents/mcp_config.json` (copy of `mcp_config.json` here).
- Plugin: `.agents/plugins/synth-qualia/` (mcp_config + skill + rules pointer).
- Skill: `.agents/skills/synth-studio/SKILL.md`.
- SDK helper: `synth_local.py` returns a real `google.antigravity.types.McpStdioServer`.
- Example agent: `agent_synth.py`.
- Verify: `scripts/verify-antigravity-client.py` (config + MCP transport; no model/IDE claim).

## Enable

### Antigravity IDE / App / CLI (workspace)

Copy `mcp_config.json` to the workspace:

```
C:\github\synth\.agents\mcp_config.json
```

The synth repo already commits that file. If you open `C:\github` as the workspace, also copy it to `C:\github\.agents\mcp_config.json`.

IDE path: **…** on the agent panel → **MCP Servers** → **Manage MCP Servers** → **View raw config**. Refresh after the file lands.

### Global (all Antigravity projects)

Merge the `synth` entry into `~/.gemini/config/mcp_config.json` (`C:\Users\<you>\.gemini\config\mcp_config.json`). Do not replace unrelated servers.

### Python SDK

```python
from google.antigravity import Agent, LocalAgentConfig
from vendor.antigravity.synth_local import synth_server  # or copy the function

server = synth_server()
async with Agent(LocalAgentConfig(mcp_servers=[server])) as agent:
    response = await agent.chat("Call synth_find with q=xpbd, then produce a revolve.")
```

Inspect-only (this machine, installed SDK in `.venv-antigravity`):

```powershell
.\.venv-antigravity\Scripts\python.exe scripts\verify-antigravity-client.py --inspect-only
```

That constructs `McpStdioServer` against google-antigravity 0.1.23. `--fixture-dir` from `scripts/verify-local-clients.py` runs the stdio transport. `antigravity_agent_harness_roundtrip` and `antigravity_ide_frontend_acceptance` stay false unless those surfaces are actually driven.

## Binary

Do not rebuild `target` or `target-studio`. Default command:

```
C:\github\synth\target-prod\debug\synth-mcp.exe mcp
```

Env: `SYNTH_INPUT_ROOT=C:\github`, `SYNTH_OUTPUT_ROOT=C:\github\synth\production`. Override with `SYNTH_BIN`. `synth.exe` is the produce CLI; `synth-mcp.exe` is the sidecar Grok/Antigravity spawn while `synth.exe` is locked.

Tool names are unprefixed (`synth_find`, `synth_produce_present`). Start with `synth_find`. Present HTML is Antigravity-ready (`<agent-embed>`, Auxiliary Pane).
