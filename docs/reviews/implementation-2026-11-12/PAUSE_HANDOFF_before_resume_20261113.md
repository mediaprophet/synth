# Paused for battery charging — 13 November 2026

The user explicitly requested a pause. Do not resume implementation, builds, tests, rendering, dependency setup or agent work until the user resumes. Local changes are saved and uncommitted. The task is not complete or all green.

Root stopped its native build session48474 and WASM build session10425; both returned exit1 after interruption. The codec test session7377 completed exit0. The bounded localhost review server3574 completed exit0. No live Synth MCP or target/target-studio process was killed or replaced. Workers were instructed to freeze; their final notes are below or in their owner documents. Other user-owned chats may still have their own jobs; do not stop them.

## Objective and current steering

Continue the authorized local Synth/Hypermedia asset-production repairs, Codex integration, actual-client checks and actual rendered-output review. Original plan: ../2026-11-12-asset-quality-audit/IMPLEMENTATION_PLAN.md. Read Synth AGENTS.md and Hypermedia audit-docs/FUNCTIONS.md before capability-absence claims; all148 catalog morphisms exist.

M64 is current; E64 is legacy compatibility. Preserve external full-BLAKE3 integrity repairs and concurrent streaming/inference work. EMF retains physical fields plus explicit colour/time interpretation and original independent PCM; display and listening projections are declared derivatives, not replacements. The user specifically requires asset-aware dynamic source admission, not reduced asset detail to fit a fixed64KiB cap.

The user reported turning off Windows Application Control. Ordinary builds subsequently ran and codec tests passed. Do not change Windows security settings yourself. Earlier automatic approval-service timeouts were before process creation, not determinations that the requested operations were unsafe.

## Immediate resume order

1. Read this handoff, RESUME_PROGRESS.md, TODO.md and supervisor.md. Check live files before applying any recorded pending fix; another chat edits QualiaDB concurrently.
2. Resolve/confirm the two demonstrated qwen35 GGUF dependency references (interfaces agent is checking them during pause preparation). Rebuild isolated native review target and run all relevant suites with no-fail-fast. Source-admission tests and final visual changes are not yet validated.
3. Freeze a NEW executable under production/review-runtime-<unique>/ before any rendering or client use. Never run a mutable target-gap-review executable while Cargo may overwrite it.
4. Run source-limit, CLI/MCP/Grok, actual Antigravity, precision, EMF recording, shader and visual checks listed below. Inspect actual images, not only logs or counts.
5. Complete local-provider WASM packaging and actual browser verification before promoting docs/pkg/qualia. Current served WASM is old. Complete Codex actual-client acceptance if execution is available; registration alone is not acceptance.

## Latest verified results

- All71 actual production codec/lifecycle tests passed,0failed,0ignored: tests-resume-formats-portal-control-updated.log. Includes four new retained-EMF lifecycle tests. Exact command: cargo test --offline --manifest-path C:/github/qualiaDB/tools/container-format-conformance/Cargo.toml --target-dir C:/github/synth/target-gap-review-formats-portal -j2. Concurrent new M64 tests may have changed since that run; preserve exact evidence scope.
- Last integrated test evidence remains H212pass/7fail, Synth library50pass/3fail (all13web tests passed), asset integration16pass/1fail, profile10pass. All11 failures have source repairs; final rerun pending.
- Current usable OLD frozen executable: C:/github/synth/production/review-runtime-20261112-212521-de79a6b3/synth.exe, SHA256 a8921532631d57e67198a4c29122674acaec8a24a68c0e0116d1565e072c92fd. It predates knee, foliage, Portal lifecycle/render,4MiB MCP transport, shadow and adaptive-source fixes.
- Actual CLI/MCP/Grok package parity passed at production/client-parity-20261112-212641-dcde5373. Three cube views independently inspected.
- Actual EMF recording metadata/time/originalPCM/signed-zero/replay checks passed at production/emf-recording-20261112-213942-a333ceec. Root viewed the cyan native wavelength derivative. Its exposure change left original EMF unchanged. No actual browser pixel/listening acceptance yet.
- Earlier2048²/16-bit and24000/1001 precision checks passed; rerun with final runtime.

## Dynamic source implementation — saved, not yet runtime-qualified

Files: src/production/{mod.rs,source.rs,package.rs,tests.rs}, src/asset_cli.rs, src/mcp_full.rs, src/mcp_full/{production.rs,tests.rs}. Root removed the obsolete tray-only64KiB exception in src/tray_env.rs.

Default caller source ceilings are optional usize::MAX and intersected with an effective per-asset capacity before session/read/replay. Explicit64KiB ceilings remain fixed. Capacity accounts for actual retained input bytes, package/work ceilings and a128× retained-source logical reservation. Small-source workflows retain resource headroom; source reservations reduce effective host allowance. The128× factor is a conservative policy estimate, not an allocator or process-peak guarantee. Overlapping candidates/clones/RSS are explicitly outside the claimed per-revision accounting scope. Evaluator steps and host/source logical work have separately bounded counters; automatic evaluator allowance uses an explicit u64::MAX sentinel and honors explicit100k limits. Execution and MCP receipts share for_source_usage.

MCP supports exactly one of inline source or admitted source_path (oneOf schema), scoped through existing SYNTH_INPUT_ROOT checks. Inline length is checked before cloning. CLI reads against remaining history/submission capacity. Strict package replay recalculates admission.

Tests authored: actual34,000-byte data asset from >64KiB structured source, exact replay, retained input/history/overflow, explicit caps, rejected edit preservation, six1024² RGBA workspace (>22MiB) and20MiB refusal, CLI/MCP source-file parity. Supervisor final findings are in supervisor.md; do not call this green until executed.

Actual high-detail heart qualification source:
C:/github/synth/examples/quality/fixtures/anatomy-source-budget.vibe
92,574bytes; SHA256208d8934507f2816aa98784486af137d1e90b2195eda36af2353ce57c08df0ea.
Generator generate_anatomy.py now accepts --radial-segs32..128 and --output; default32-side anatomy.vibe remains61,396bytes, SHA25616e78d7d089e047593edf5f127a144f5bc7fcc6b94c242c0f5809af10f983997. Repeated generation was checked deterministic.
Run: python scripts/verify-large-heart-source.py --synth <fresh-frozen-synth.exe>. Syntax checked only; actual CLI/repeat/reopen/scoped-MCP/HMC parity/export/native3views pending.

## Latest native build interruption and concrete errors

Command: cargo test --offline -p synth -p asset-production-vibe -p profile-authoring --features synth/preview --all-targets --no-run --target-dir C:/github/synth/target-gap-review -j2.
Log: build-integrated-resume-5-control-updated.log. Root interrupted at pause, after compiler reported two unresolved etalon_inference_kernel references:
- Q crates/qualia-core-db/src/gguf_bridge/qwen35/gguf/linear.rs:7
- Q crates/qualia-core-db/src/gguf_bridge/qwen35/gguf/session.rs:212
Core directly depends on qualia-inference-kernel, not etalon-inference-kernel. Interfaces completed both namespace substitutions to qualia_inference_kernel::recurrent after verifying that the wrapper reexports the identical canonical Etalon module. The fix is saved but unbuilt. Do not change inference algorithms or add speculative dependencies. Earlier external build3/4 errors (module declarations, opaque metadata helper, ephemeral array type, control-guard borrowing) are already repaired live.

## Other source fixes awaiting final integration

- Real valid M64/E64/EMF presentation fixtures; explicit production16/discard test input; EMF dispatch work/integrity assertions; analytic closed Boolean volume/replay checks.
- Authored21joint fauna bindings; knee Bishop transport fix and H integrated regression.
- Foliage welded poles/seams, normals and explicit material coverage; translucent teaching layer and flower/capsule placement.
- MCP standalone4MiB wire budget and actual stdio full-catalogue regression after installed Antigravity reproduced the old internal budget refusal.
- Retained EMF state and recovery plus explicit pending/uploaded/unavailable status; four real codec/state tests now passed.
- Portal render_emf_preview(amplitude_gain): bounded1024², explicit z=0/t=0 WebGPU phase/sigma diagnostic, display-only gain, submission-only receipt. Actual canvas pixels remain separate acceptance.
- Etalon shadow receiver-plane correction uses actual per-tap fractional bilinear support, bounded singular/grazing fallback, forward-Z sign and nonzero precision floor. Source frozen; CPU analytic cases and full WGSL/Naga validation await execution. Shader is re-exported into Qualia, not duplicated.

## Renderer checks required on resume

Targeted: cargo test --offline -p etalon-render --lib shadow_math::receiver::tests --target-dir C:/github/synth/target-gap-review -j2 (sequence after other same-target Cargo work).
Etalon AGENTS.md requires full cargo xtask test after GPU-behavior changes. Its xtask generates real.gpuconfig then nextest --benches --tests --all-features. Do not claim WebGPU CTS coverage. Read interfaces notes for prerequisites: VulkanSDK1.4.363.0+dxc present; cargo-nextest not yet found. Windows runner stages WARP and installs Agility even with CARGO_NET_OFFLINE; --no-require-agility-sdk does not skip those downloads. No full suite was launched.

Run scripts/verify-shadow-quality.py --synth <fresh-frozen> --scene-root C:/github/synth/production/quality-20261112-225513-6f8d5a17. It compares receiver on/off pairs and a real floor/box caster. Old artifact assessment correctly fails (production/shadow-quality-20261112-235209-825c868c/verification.json). Inspect the resulting PNGs independently after metrics.

## Visual evidence already inspected

Root independently inspected all3standard views of the current biological/cell/heart/fauna/terrain/cosmos examples, interfacePNG, and four paired shadow/AO controls. These are stylized illustrative assets, not film-quality or anatomical validation.
Best32-side heart: production/quality-20261112-230958-5214d6d2 — round open vessel lips, taper and solidified walls improved. Default retained unchanged while64-side variant awaits runtime.
Terrain/cosmos refinement: production/quality-20261112-225513-6f8d5a17. terrain-shot-south/frame-0000.png shows upright relief using pitch-1.05, preserving authoredZ-up geometry. Final harness has that camera.
Shadow/AO diagnostic: production/render-effects-diagnostic-20261112-9971807a3c. Shadows alone caused planet pole rings/terrain fine streaks; disabling them was diagnostic, not final acceptance.
Fauna production/quality-20261112-215619-485301d9 still predates knee repair. Fresh3views+3poses required. Flora source fixes also need fresh views/detail/turnarounds. User heart annotation was reread and remains the apex/atria/ventricular mass/great-vessel request in anatomy/.synth/annotations.json.

## WASM packaging issue discovered before pause

The Q package-qualia-wasm-full.ps1 invocation builds the Q workspace's pinned git Etalon, so it misses local renderer fixes. Root interrupted this build (session10425; build-portal-wasm-control-updated.log). No package was promoted. Do not blindly rerun and claim it tests local providers.

Preferred next investigation: build qualia-core-db's cdylib from the Synth workspace with its existing local provider patches, using cargo build --manifest-path C:/github/synth/Cargo.toml -p qualia-core-db --no-default-features --features portal --target wasm32-unknown-unknown --release, isolated target-gap-review-wasm, Rust1.96.1, SIMD128,8MiBstack/1GiBmaxmemory/getrandom_backend=wasm_js flags. Then use matching wasm-bindgen CLI to stage web bindings. This is a proposed next step, not executed.
Synth Cargo.lock binds wasm-bindgen0.2.130; Q build used0.2.129. Cached CLI C:/Users/Hp/AppData/Local/.wasm-pack/wasm-bindgen-06a9b89f25ab66ba/wasm-bindgen.exe was actually checked: version0.2.129. Need matching0.2.130 tool or another clean local-provider staging solution. Do not mutate shared Q lock/config or downgrade unrelated dependencies casually.

Browser harnesses: scripts/prepare-portal-emf-review.py (synchronized API, gain0.02, exact64bit-clock comparisons, originalPCM, corruption preserves accepted state, early-load/retry/replay and actual canvas readback); scripts/verify-local-browser.py (isolated local Chrome, bounded server, screenshot/result). Browser execution not qualified. Ordinary Chrome previously exited during crash-report helper startup; disabling optional crash reporting did not fix it before the user's system change. In-app localhost origin automatic approval timed out twice; do not bypass browser security controls or substitute an indirect route around that gate. CUA portion was closed with failure summary.

## Codex and other clients

Global synth-local registration succeeded with supported installed Codex CLI and the old frozen runtime. Repository configs under vendor/codex also point to that runtime; update only after fresh binary qualification. Existing current chat01a1cce5-d6bb-79e3-a81b-c5414d9c938b has no exposed Synth tools yet. Desktop app-server proxy failed OS10050 before initialize. Normal standalone discovery cannot write protected home state/temp paths. Elevated isolated-state discovery requests timed out before launch. scripts/verify-codex-client.py --isolated-discovery supports a new SQLite state directory and makes no new chat/model/provider turn; actual discovery/tool-call acceptance remains open.
Installed CLI: C:/Users/Hp/AppData/Local/OpenAI/Codex/bin/9e1da79431da526b/codex.exe.
Official Antigravity0.1.23 Windows wheel/dependencies are installed in .venv-antigravity, pip check passed. Real SDK initialize/tool-list worked and exposed catalogue budget defect. Run full actual SDK parity with fresh4MiB-cap binary. User previously approved local web tests and dependency downloads; do not ask for that same authorization again solely because a request timed out.

## Preservation constraints

No commits, publication or user security-setting changes requested. Preserve all concurrent edits. .git/.agents/.codex and user-home configuration are protected. Build only isolated review targets; never replace live Synth target/target-studio/MCP. Keep root studio8787 routing and /project unchanged. Frozen candidates and old failed evidence remain for comparison.

Separate formats/streaming chat id01a1cd90-fd56-73b1-8eaa-6b32b0b58ef6 (hostlocal) was read-only inspected. It remained active; latest note concerned a new M64 corruption-test borrow fix. No message was sent to it. Do not message user-owned chats without authorization.

## Final worker freeze confirmation

Asset worker: no pending calls/processes/edits. Interfaces worker: no pending calls/processes; all source frozen. Additional interfaces handoff: interfaces-pause-handoff-2026-11-13.md. The two qwen35 namespace fixes are saved; no manifest, feature or algorithm changes were made. Supervisor final review status is recorded separately when received.

## Final supervisor verdict at pause

All workers have finished writes/calls and are paused. Portal source review is complete; actual codec/lifecycle evidence is71passed/0failed. Dynamic-source review confirms explicit64KiB source and100,000step evaluator caps are preserved, execution/receipts share policy, and accounting is honestly per-revision logical rather than process-peak. One P2 remains: src/production/source.rs prepare_edit checks only submission size before cloning inputs and replaying history; add cumulative-history admission before those allocations/replays, with a regression proving rejected edits do not replay. Do this first on resume. Final MCP oneOf block was reported by the implementer but was beyond the supervisor captured read; verify it and execute source/MCP regressions. The24MiB real workspace/refusal regression is authored but unrun. No worker-owned processes or calls remain.
