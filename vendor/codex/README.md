# Local Synth in Codex

This adapter connects the existing Synth stdio server to Codex. Production files stay in the selected local workspace. It does not add an asset engine or require an API key.

`config.toml` is the local configuration template. Its executable must be built and validated before registration. The input root admits explicit local retained-input requests; the output root admits create-only production/export/review files. Keep the live `target` and `target-studio` binaries untouched.

Registration uses Codex's supported MCP command:

```powershell
codex mcp add synth-local --env SYNTH_INPUT_ROOT=C:\github --env SYNTH_OUTPUT_ROOT=C:\github\synth\production -- C:\github\synth\production\review-runtime-20261113-122833\synth.exe mcp
```

The supervisor performs registration after the implementation build has passed its checks. Inspect the registered command with `codex mcp get synth-local`. The template's explicit local environment and timeouts can be merged through the client's configuration controls. Registration must be followed by discovery inside the actual client; a CLI listing alone is not end-to-end acceptance.

Start with `synth_operations`, `synth_operation_contracts` and `synth_ontology_morphisms`. Run Hypermedia recipes through `synth_produce_present`; generic `vibe_invoke` belongs to a different host. Use the actual tools advertised by your server, not names inferred from old prose.

Provide `inputs:[{key,kind,path}]` for external assets, and unique `out_hmc`, `export_dir` and `receipt_json` names. Request native previews for exported `.10d` keys. Inspect PNG/SVG outputs and all requested native views, and read project annotations before revising a model. Resource paths and hashes in a receipt are delivery evidence, not proof of visual quality.

The same workflow is available from `scripts/invoke-asset.ps1`. For multi-operation work, use `-Source FILE.vibe -InputManifest FILE.json -Preview`. One-operation calls select `-ResultField` and `-Encode` explicitly. Example: a `sweep_assembly_profile` result uses `-ResultField container_10d -Encode none`; an RGBA result uses `-Encode rgba`. Do not guess by namespace.

Codex's app, CLI and IDE share agent configuration. Managed ChatGPT Work environments do not automatically read this laptop's configuration. This adapter does not create a public tunnel or switch execution to a cloud environment. Official references: [MCP configuration](https://learn.chatgpt.com/docs/extend/mcp), [developer settings](https://learn.chatgpt.com/docs/developer-settings).

For true 2k/high-bit image work, select MCP `quality_profile:"production16"` before production. It admits 64/256/320 MiB resource/aggregate/package budgets and 500 million bounded work units. For lossless final sequences, call the existing `capture_deterministic_movie` with `profile:"png-sequence"`, rational `fps:{numerator:24000,denominator:1001}`, and congruent retained RGBA8/16 frames. Export `sequence.manifest_resource`; files/v2 automatically includes its PNG/WAV leaves. This delivers a usable exact-timing image sequence, not a claimed compressed final movie.

For an installed and registered server, `scripts/verify-codex-client.py --codex <codex.exe> --proxy --thread <existing-chat-id> --out <new-evidence-directory>` checks the actual 148 catalog IDs through that client's app-server. For fixture parity, additionally supply `--fixture-source <file.vibe> --input-manifest <bindings.json> --synth <verified-synth.exe> --output-root <configured-SYNTH_OUTPUT_ROOT> --preview-key <exported.10d>`. Input paths in the manifest must be absolute. The tool checks exact source/input digests, an invalid-argument refusal, create-only repeated production, HMC equality with CLI, binary identity, receipt and three native PNGs. This does not create a model turn or establish visual acceptance; inspect the generated images and record that separately. Registration, the actual desktop thread and reconnect behavior must be verified independently.
