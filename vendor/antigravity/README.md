# Antigravity local adapter

`synth_local.py` configures the installed Google Antigravity SDK to launch the local Synth MCP server. Set `SYNTH_BIN`, `SYNTH_INPUT_ROOT` and `SYNTH_OUTPUT_ROOT` for your local installation. The SDK itself is a separate dependency and is not vendored here. `scripts/verify-antigravity-client.py` exercises its MCP transport without model-provider calls.
