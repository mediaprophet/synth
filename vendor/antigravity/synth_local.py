"""Local Synth server configuration for the existing Antigravity SDK."""
import os
from google.antigravity.types import McpStdioServer

def synth_server():
    return McpStdioServer(
        name="synth-local",
        command=os.environ.get("SYNTH_BIN", r"C:\github\synth\target-gap-review\debug\synth.exe"),
        args=["mcp"],
        env={
            "SYNTH_INPUT_ROOT": os.environ.get("SYNTH_INPUT_ROOT", r"C:\github"),
            "SYNTH_OUTPUT_ROOT": os.environ.get("SYNTH_OUTPUT_ROOT", r"C:\github\synth\production"),
        },
    )
