> Current status: see [COMMIT_HANDOFF.md](COMMIT_HANDOFF.md). Further tests stopped at user request; committing the implementation checkpoint.

# Paused for other Synth work — 13 November 2026

User explicitly requested this pause so another Synth thread can finish before asset work resumes. Do not resume builds, tests, implementation, rendering or dependency setup until the user asks. Everything is local and uncommitted; not all green. Preserve other chats' edits and processes.

## What is complete and verified this resume

- Supervisor P2 source-limit fix: cumulative source history is checked before prepare_edit clones inputs or replays. Shared check also protects submit. Counting-producer regression passed; cancelled/refused edits preserve previous state, accepted edit replays. Source-budget tests passed in tests-integrated-resume-20261113.log. MCP source/source_path exactly-one schema and runtime reviewed.
- Actual 92,574-byte heart source passes CLI/repeat/reopen/scoped MCP, identical HMC, receipts and native three views: production/source-budget-20261113-092925-c4669b46. Root viewed all three PNGs. This proves larger authored assets, not film-quality or medical/anatomical validation.
- Rebuilt integration: Hypermedia223/223, profile10/10, Synth51 passed/11 failed initially (10 localhost permission failures plus invalid MCP test handshake). All13 web tests and authenticated integration1/1 subsequently passed with authorized localhost access. General production2/2 and bin1/1 passed. Asset integration16/17; remaining real Boolean regression described below.
- CLI/MCP/Grok retained asset parity, strict replay and invalid input/host refusals: production/client-parity-20261113-092758-e1bf1a21.
- Actual installed Antigravity0.1.23 SDK-configured MCP transport passed all148catalog, production, strict replay, durable receipt, reconnect, wrong-host/unknown-key refusal and CLI HMC parity: production/antigravity-final-20261113-092959-099c8a9a. No model provider calls; not IDE/Agent-harness acceptance.
- Controlled shadow proof passes numerical gates: production/shadow-final-20261113-093345-3de3c46b. Cosmos receiver p95=0 and >4levels0%; terrain p95=0 and >4levels0.223% (old4.04%); actual box caster3886pixels>=12levels contrast,max114. Worker visually reviewed cosmos/caster; sparse terrain flecks remain unresolved, see below.
- Previous71 actual codec/lifecycle tests remain the latest codec evidence: tests-resume-formats-portal-control-updated.log. Preserve scope; later shared edits may require rerun.

## Runtime and actual Codex connection

Fresh FROZEN runtime: C:/github/synth/production/review-runtime-20261113-092726/synth.exe
SHA256: 0e3e4afc32fa2c23cef068bb947be7a2429f1709059d321d241b2465b1885df2
Repository templates vendor/codex/{config.toml,synth-qualia.mcp_config.json,README.md,synth-studio.SKILL.md} and supported CLI global synth-local registration now point here. Inputroot C:/github; outputroot C:/github/synth/production. This runtime includes source budgeting,4MiB MCP response, knee/foliage and shadow fixes; it predates the last cap-triangulation fix and Portal queue.present fix.

Synth tools are NOW available in this actual chat. Small actual produce call succeeded with strict replay, but receipt identified the OLD a892... frozen executable. Full catalog call on that connection reproduced its known response-size error. Evidence production/codex-resume-20261112221908011.{hmc,json}. This is old-client-call proof, not updated connection acceptance.

Supported proxy reload still fails before initialize with Windows OS10050 at app-server-control.sock. Evidence production/codex-client-resume-20261113-0931/client-stderr.log and failure.json. User was asked to refresh synth-local via client Settings, but replied requesting pause instead. Fresh connection remains unverified. Do not kill a live MCP, restart the app unexpectedly, or automate ChatGPT desktop UI (computer-use rules prohibit that). No new chat/model turn/provider call was created.

## Outstanding native failures and saved fixes

1. MCP wire regression had missing notifications/initialized. Supervisor fixed harness in src/mcp_full/tests.rs and asserts initialization+two replies. Full148 dual content/structured catalogue and exact2MiB refusal coverage preserved. Runtime handshake was correct. Corrected test not yet executed.
2. Original cylinder/pyramid union fails exact RegistryCollision in tests/asset_operations_suites/waves_4_to_7.rs. Genuine near-coincident exact points arose from rounded extrusion cap centroid. Narrow H asset-production-vibe/src/geometry.rs repair uses existing Core polygon triangulator and perimeter vertices; preserves planar cap and exact collision refusal. CW/CCW closure regression plus isolated unchanged-geometry union/strict-replay regression added. Supervisor source reviewed; not yet runtime qualified. No epsilon welding or test weakening.
3. Subsequent no-run/all-targets attempts intermittently failed linker LNK1104 opening output test executables (asset_production_vibe, synth lib, authenticated_host, then synth bin test), including elevated and serial retry. No persistent test process was established by Get-Process; free disk~444GB. Output file sometimes absent after failure. Do not assert cause or success. Relevant logs build-followup-20261113.log, build-followup-20261113-retry.log, tests-followup-allowed-20261113.log, tests-followup-serial-20261113.log. On resume diagnose narrowly; use fresh output artifacts while reusing caches if appropriate, never live target/target-studio.

## Portal WASM, local provider and GPU suite

Isolated minimal build driver C:/github/synth/target-gap-review-portal-driver/Cargo.toml depends on real Q with features portal, copied local Synth patch tables and isolated lock. Q declares cdylib+rlib; consume its actual qualia_core_db.wasm, not a driver ABI. This avoids changing shared Q lock and avoids pinned git Etalon instead of local provider. Native Synth --lib is not a portable WASM driver. -p external Q --features portal is rejected by Cargo.

Command used RUSTUP_TOOLCHAIN=1.96.1, RUSTFLAGS=-C target-feature=+simd128 -C link-arg=-zstack-size=8388608 -C link-arg=--max-memory=1073741824 --cfg getrandom_backend="wasm_js", cargo build --offline --manifest-path <driver>/Cargo.toml --lib --target wasm32-unknown-unknown --release --target-dir C:/github/synth/target-gap-review-wasm -j2. Required locked dependencies downloaded with authorization; driver lock is separate. Build FAILED eight errors in build-portal-local-driver-20261113.log:
- Seven cryptographic_library WASM cfg/import errors in current Q handshake/COSE, untouched. Read exact log and coordinate with other streaming/crypto owner; do not reintroduce obsolete modules.
- EMF frame.present() mismatch: saved narrow fix Q src/render/gpu/emf_present.rs uses self.queue.present(frame), matching existing portal_gpu/render_frame.rs. Not built/tested.

Verified official wasm-bindgen0.2.130 executable at target-gap-review-wasm/tools/wasm-bindgen-0.2.130/wasm-bindgen-0.2.130-x86_64-pc-windows-msvc/wasm-bindgen.exe; archive hash844d23f1b56097549b936dd2e0d561fadebc1e8c9e985d5068b6b919654104ad verified against official checksum and release digest. verification.json retained. Bind actual Q wasm --target web --out-name qualia --out-dir <new staging>/pkg; preserve snippets/types. No new served package promoted. Actual browser metadata/PCM/corruption/lifecycle/canvas pixel verification remains pending using scripts/prepare-portal-emf-review.py and verify-local-browser.py. Earlier browser crashhelper issue may need retest after user's AppControl change; no bypass of permission/security UI.

Etalon root AGENTS requires full cargo xtask test after GPU change. Worker repaired xtask/src/test.rs WARP target directory to honor CARGO_TARGET_DIR and added pure routing regression; supervisor source reviewed. Pinned nextest0.9.131, WARP1.0.20, Agility1.619.0 verified/staged locally (Authenticode/checksums). Exact instructions/hashes: interfaces-gpu-prerequisites.md. Targeted shadow Cargo build was STOPPED on user pause while compiling dependencies; no targeted or full GPU test result. Log tests-etalon-shadow-20261113.log. Runner path target-etalon-gpu-prerequisites/nextest-0.9.131/bin, isolated target target-etalon-gpu-review; CARGO_NET_OFFLINE=true, CARGO_BUILD_JOBS=2. Root owns all Cargo jobs.

## Actual visual work still outstanding

Full9-domain render review production/quality-20261113-093448-f1b95a0a completed exit0: all37steps and all9 deterministic replays passed. Worker viewed all spatial standard3views/hero via review sheets and UI raster. This is not blanket visual acceptance; see concrete findings and interfaces-pause-handoff-2026-11-13.md. Concrete findings: flora hero clips leaf tip (distance*0.82); tissue detail low contrast; flora-branch canopy has dark stippled/banded patches needing paired shadow diagnosis; detached flower petal may be intended exploded stage, inspect source before change; fauna dark limbs lose detail on navy background. Biology/cell/heart shown clean in limited stylized scope. Root viewed larger heart all three but no anatomical/film acceptance.

Residual terrain: supervisor independently viewed on/off pair, sparse diagonal/angular flecks only with shadows. Current mesh4096verts/7938tris has seven faces slightly backfacing to sun, min normal dot light -0.0097148, so genuine selfshadow cannot be ruled out. Cross-triangle bilinear taps are plausible, not proven. Need world/camera correlation before further provider change. See shadow-pause-review-20261113.md. Do not disable shadows to pass.

Fresh precision/EMF-recording reruns, final affected visuals and final independent acceptance remain. Earlier precision/recording evidence and original plan are in PAUSE_HANDOFF_before_resume_20261113.md and owner notes. M64 current/E64legacy; preserve full-BLAKE3 and concurrent streaming work. Read project annotations before changing models.

## Process and continuation state

Root closed/stopped its sessions31502,86571,63311,41235,75777,11001; largeheart42949 and clientparity29819 finished exit0. Native/WASM attempts mostly already failed; targeted Etalon session86571 interrupted. No live MCP/target/target-studio executable replaced or killed. All three workers confirmed paused with no owned pending processes/calls; full quality70048 completed exit0 before pause. Other user-owned chats may continue; do not stop their processes.

On explicit resume: read this handoff plus latest worker notes/TODO; inspect concurrent updates first. Diagnose saved native/WASM issues, build/test, freeze unique runtime, complete actual Codex refresh and adapter parity, then render/fix/review remaining assets. Ask no repeated permission for previously authorized local web tests or dependency downloads. No claim that quota remains or resets: future available quota is controlled by the client/account.
