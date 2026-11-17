# Integration wave 0.0.41.3 — shared subagent constraints

Copy this block into every packet prompt. Packets are self-contained.

- Roots: `C:\github\{qualiaDB,etalon,hypermedia-production,synth,webizen-website}`. Isolation `none`. Edit github roots. Do not create a git worktree.
- Do not commit, push, or force-push.
- Do not kill live Synth MCP. Do not rebuild `synth/target` or `synth/target-studio`.
- One rustc at a time. Source packets must not run cargo. Coordinator cargo uses `target-production-core` (wasm: `target-production-core-wasm`).
- P00 is the only accepted production-core packet. Do not mark this packet, P22, or P23 qualified.
- Do not checkout etalon `origin/0.0.41.2` (`057545fc`).
- Do not bump QualiaDB / Hypermedia / Synth etalon git `rev` (packet I0 / coordinator).
- Policy: WebGPU is preferred. WebGL2 is the required fallback when WebGPU is absent or unusable. `?webgl2=1` is force-GL, not the default.
- Do not edit EMF/QAC pipelines except through `effective_readonly_storage_binding_size` already landed.
- No `cargo fix` across a workspace.
- Write only leased paths. Create-only `agent-report.md` under the packet evidence directory. Do not overwrite other evidence dirs.
- After two failed bounded repairs, stop and report.
- Return: packet id, paths written, what changed, commands actually run with exit codes, remaining gaps.
