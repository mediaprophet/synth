"""Local Synth MCP configuration for the Google Antigravity SDK.

Used by scripts/verify-antigravity-client.py and vendor/antigravity/agent_synth.py.
Default binary is the isolated target-prod sidecar (400 morphisms = 150 base + 250 expansion). Override with
SYNTH_BIN / SYNTH_INPUT_ROOT / SYNTH_OUTPUT_ROOT. Do not point at target or
target-studio; those binaries stay locked.
"""
import os
from google.antigravity.types import McpStdioServer

_DEFAULT_BIN = r"C:\github\synth\target-prod\debug\synth-mcp.exe"
_DEFAULT_INPUT = r"C:\github"
_DEFAULT_OUTPUT = r"C:\github\synth\production"


def synth_server():
    return McpStdioServer(
        name="synth",
        command=os.environ.get("SYNTH_BIN", _DEFAULT_BIN),
        args=["mcp"],
        env={
            "SYNTH_INPUT_ROOT": os.environ.get("SYNTH_INPUT_ROOT", _DEFAULT_INPUT),
            "SYNTH_OUTPUT_ROOT": os.environ.get("SYNTH_OUTPUT_ROOT", _DEFAULT_OUTPUT),
        },
    )


def colab_mcp_server():
    """Google Colab MCP Server (googlecolab/colab-mcp) for driving Colab Ultra A100/L4 runtimes."""
    return McpStdioServer(
        name="colab-mcp",
        command=os.environ.get("UVX_BIN", "uvx"),
        args=["git+https://github.com/googlecolab/colab-mcp"],
    )
