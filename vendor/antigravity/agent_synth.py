"""Example Antigravity SDK agent wired to local Synth MCP.

Uses vendor/antigravity/synth_local.py (target-prod synth-mcp.exe, 400 morphisms).
Do not point at target or target-studio. Do not kill live MCP children.

This script constructs a real Agent with McpStdioServer. Running it issues a
model turn; prefer scripts/verify-antigravity-client.py --inspect-only for
config-only checks.
"""
from __future__ import annotations

import asyncio
import sys
from pathlib import Path

_HERE = Path(__file__).resolve().parent
if str(_HERE) not in sys.path:
    sys.path.insert(0, str(_HERE))

from synth_local import synth_server  # noqa: E402


async def main() -> None:
    from google.antigravity import Agent, LocalAgentConfig

    server = synth_server()
    prompt = (
        "You are producing with Webizen AI Synth 0.0.41.13, the development tool. "
        "Call synth_find with q=xpbd. Then open agent-instructions/kinetics.md "
        "(index agent-instructions/README.md; house rules AGENTS.md). "
        "Invoke morphisms with capability.invoke inside synth_produce_present. "
        "Do not use vibe_invoke for Hypermedia ids."
    )
    async with Agent(LocalAgentConfig(mcp_servers=[server])) as agent:
        response = await agent.chat(prompt)
        print(response)


if __name__ == "__main__":
    asyncio.run(main())
