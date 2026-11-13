# Paused for laptop travel — 12 November 2026

The user explicitly requested a pause. Do not resume implementation, builds, rendering, network setup or agents until the user resumes. Changes are saved locally and uncommitted. Existing live MCP services were not replaced or killed. All known task-owned build/test/render/client sessions completed; workers were asked to stop and save their handoffs.

## Objective retained

Implement the local Synth + Hypermedia asset-quality plan, including Codex integration, existing-tool reuse, meaningful verification and actual visual review. Use appropriate specialist agents and an independent supervisor. Fix discovered failures rather than declaring partial results complete. Original audit and plan: `docs/reviews/2026-11-12-asset-quality-audit/{REPORT,IMPLEMENTATION_PLAN}.md`. Ongoing ledger: `TODO.md` in this directory.

The work is **not complete or all green**. Do not claim film-quality or anatomical acceptance from successful production.

## Latest user steering

- Full `.10d` upgrades are authorized where needed; preserve versioned compatibility and test the consumers as well as serialization.
- EMF should contain colour-space interpretation and timed audio tracks and must not be restricted conceptually to human-visible/audible bands. Existing physical fields and display/listening conversions are distinct. `emf-contract.md` is a proposal only; no EMF source changes have been made.
- A separate agent is reviewing/writing `.m64`/`.e64` specifications. The user suspects an environmental role for E64 and a more generic matter role for M64, but is unsure and wants the drift clarified. **Defer their boundary definition until those specifications arrive.** Current code treats E64 as a legacy M64 alias; this is an implementation observation, not the intended architectural decision.

## Workspace and execution constraints

- Main workspace `C:/github/synth`; related local workspaces Hypermedia, QualiaDB, Etalon and anatomy under `C:/github`.
- Read Synth AGENTS.md and Hypermedia `audit-docs/FUNCTIONS.md` / matching module before absence claims. The 148 catalog morphisms exist. Keep ontology, dispatch and descriptors consistent.
- Do not build into or terminate live `target` / `target-studio` processes. Isolated build target is `C:/github/synth/target-gap-review`, with `--features preview`.
- Preserve pre-existing edits, especially Synth Cargo files, presentation/project/server files and unrelated Etalon inference work. No commits or publication requested.
- `.agents`, `.codex` and user-home config are protected. Repository-owned replacement candidates are prepared; installation requires the normal trusted configuration write, after a verified binary is selected.

## Built executable and frozen runtime

First assembled build passed: `cargo build --offline --bin synth --features preview --target-dir C:/github/synth/target-gap-review -j 2` (11m38s; `build.log`).

Frozen executable for runs made before later fixes:
`C:/github/synth/production/review-runtime-20261112-123308-eeb47710/synth.exe`

SHA-256: `46962750e1d6d87cc01e18d007a0772defcc97b624097b632c515b9e083aed99`.

It **does not contain later assembly smooth-shading, scale-safe normal or pending test repairs**. New reference sources may reject on it. Build a new binary after repairs, then freeze a fresh copy for render/client runs. Windows refused Cargo replacing `target-gap-review/debug/synth.exe` while a test harness was using it; avoid that collision by sequencing builds or using frozen copies.

## Implementation already written

- Routing, error signalling, operation contracts, retained MCP inputs, create-only exports, provenance receipts and HMC-backed native review.
- Files/v2 texture/sequence dependencies and files/v1 namespace compatibility route.
- GLB static scene transforms/instances, preserved normals/UV/tangents/materials/textures, extension and accessor admission checks.
- Atlas/default SLIM attribute retention and regenerated tangent bases; geometry edits explicitly preserve/refuse or deliberately discard on a new revision.
- Principled material/map binding; native shot camera, lighting, occlusion, shadow, environment and animation controls.
- Direct physical terrain records and existing M64 v3 compilation, complete skin binding/SKL1 transfer and actual posed preview.
- Shared retained-font glyph outlines for raster/SVG labels, state geometry parity and explicit unsupported-font errors.
- Retained RGBA16 UNORM, colour metadata checks, bounded quality profiles and rational lossless PNG sequences.
- Shared typed invocation helper, Grok repair, Antigravity example, Codex configuration/documentation and replacement skill/plugin candidates.

Owner notes: `asset-data.md`, `fields-animation.md`, `interfaces.md`, `precision-sequence.md`, `interfaces-pause-handoff.md`, `supervisor.md`, `emf-contract.md`.

## Verification state and immediate repairs

1. **Synth library:** initial 39 passed / 12 failed (`tests-synth-lib.log`). Ten failures were Windows denied temporary localhost sockets. Permitted rerun passed all 13 web tests (`tests-local-web.log`). Two remaining new test fixtures are still unedited:
   - `production16_profile_is_explicit_bounded_and_invalid_profiles_refuse` supplies an empty output list, which production correctly rejects.
   - `declared_output_recipes_execute_and_contracts_cover_all_148_ids` assumes the first sorted output is the JSON manifest; sequence PNG dependencies sort first. Assert the actual keyed manifest and dependencies.
2. **Hypermedia library:** 203 passed / 3 failed (`tests-hypermedia.log`). Still unedited:
   - Metadata descriptor bijection has a real StableFuncId collision: `sweep_assembly_profile` appears twice, likely species `(sub_id=2,func_id=24)` colliding with sweep. Trace the ABI/category mapping rather than weakening the bijection test.
   - Unsupported PNG gamma test gets an accepted untagged16 image.
   - Gradient roundtrip loses expected metadata and unwraps None. Encoder does emit gAMA; investigate decoder metadata stage/semantics. Do not remove assertions or silently reinterpret colour.
3. **Precision acceptance passed:** `production/precision-20261112-122842-c055b68e/results.json`. Full 2048² PNG16, exact retained first row, 2048 distinct graded red values, exact 24000/1001 timing, dependency hashes and an 8-bit pattern crossing a packed tile boundary. Runner `scripts/verify-production-precision.py`.
4. **Actual CLI/MCP/Grok fixture:** latest `production/client-parity-20261112-123529-f536dfcc`. Production, native textured review, negative admission/wrong-host checks and helper JSON conversion succeed. Grok/CLI HMCs match; MCP/CLI do not yet match. LF source sent via MCP versus CRLF written by Python on Windows is a hypothesis, **not yet verified**. Compare retained source bytes and receipts before fixing the harness. Runner `scripts/verify-local-clients.py`.
5. More suites remain: Synth CLI/integration tests, appropriate Qualia static GLB/skin/shot/MCP regressions, regenerated profile-authoring normal regression, final combined reruns. No final acceptance has occurred.

## Reference scenes and visual review

Initial run: `production/quality-20261112-122914-069f4c25` (`scripts/verify-quality-scenes.py`). Seven domains plus cell/flora extras. Biology, anatomy, flora, flora-branch, fauna, cosmos and interface produced; initial cell/terrain used reserved identifiers and were corrected later.

Root inspected the cube checker in MCP's native three-quarter view and the legible interface PNG. Root inspected DNA's shot and all three native views. DNA remains illustrative; generator now requests denser geometry and smooth shading, but those generator changes have **not been regenerated/rendered yet**.

The original heart was rejected: egg-like mass, flat auricles and angular vessels. Latest inspected heart candidate is `production/quality-20261112-124508-6e80ddd9` with review at `anatomy-review.md`; package SHA `f0b96b5c8cf8cf3663461a7e7c6090b018106ba77da73ee76b262b7dd2ebd747`. It is still not accepted. Further generator changes now contain ventricular union and corrected outward tube winding, but **`generate_anatomy.py` and the last inspected `anatomy.vibe` are currently out of sync**. The generator was deliberately not rerun before pause.

Two concrete visual causes are being fixed:
- Assembly wrapper forced flat per-corner normals. Explicit `shading:"flat"|"smooth"` (default flat) and fail-closed MAT1 sealing are written. Its new shared-vertex normals regression passed in the Hypermedia run. The frozen runtime predates it.
- Small physical-scale faces passed a length check but a second EPS-based normalize returned zero. `profile-authoring/src/frames.rs::unit` now divides by its already checked length; regression added, **not yet built/tested**. Strict `.10d` validation remains intact. No unused/zero-area geometry was found in the diagnosed ventricular union.

Cell now has hemispherical membrane/nuclear cutaways, mitochondrial cristae and schematic ER/Golgi. Pre-smoothing evidence: `production/cell-detail-a853640d60944a968e584a60fe511879` and `production/cell-detail-preview-80172f265d6741fdaaca1fd2fce9a444`. Current source requests smooth shading and needs a new executable. Misleading metre-labelled blueprint was excluded; cell remains a schematic, not a calibrated reconstruction. Terrain source/generator now uses `elevation_grid` instead of reserved `field`; corrected package SHA `39fcd408da05ff7095ea2f04fd250437953f701e87ca401291077686a2b4dcc1` (see worker notes for run path).

Fauna's three posed frames exist in the initial run but root has not visually accepted the sequence. Flora canopy remains visibly clumped/faceted; no species-specific accuracy approval. Continue actual multi-view/sequence inspection and repair, then request independent supervisor review.

## Client installation state

**Codex is not registered yet.** User config was inspected safely and currently showed only node_repl. Actual Codex CLI path discovered:
`C:/Users/Hp/AppData/Local/OpenAI/Codex/bin/9e1da79431da526b/codex.exe`.

Prepared materials: `vendor/codex/{config.toml,README.md,synth-studio.SKILL.md,synth-qualia.mcp_config.json}`. Protected `.agents/skills/synth-studio/SKILL.md` and `.agents/plugins/synth-qualia/mcp_config.json` remain stale and unmodified. Register the verified local stdio server through the supported CLI/config flow, preserve unrelated settings, then verify actual client/chat discovery and production. Native app UI inventory timed out; CUA inspection was closed with no settings changes. Browser controls were available. Do not confuse raw MCP success with actual Codex chat integration.

Antigravity: `.venv-antigravity` was created with bundled Python3.12/system-site-packages. Offline install failed; network pip session 82564 completed exit1 due DNS/index access; escalation was aborted by user. No successful new SDK install was observed. Vendored localharness binary is absent. `scripts/verify-antigravity-client.py` is saved but unrun. Preserve environment and inspect actual state before any retry. No provider/cloud model call was made. See `interfaces-pause-handoff.md`.

## Resume order

1. Confirm the user has resumed and collect the other agent's specification paths if ready. Keep m64/e64 boundary changes deferred otherwise.
2. Read owner handoffs and check current diffs for intervening edits. Fix the five recorded failures and verify the MCP source-byte discrepancy.
3. Build/test the pending normal and assembly fixes in the isolated target. Generate sources only after reconciling the anatomy generator and other current edits. Freeze the new executable.
4. Re-run meaningful suites and actual client/precision cases, then produce and inspect revised references and animation. Have an independent supervisor assess the evidence.
5. Review/implement the EMF upgrade proposal against the user's spectral/colour/audio requirements and any new specifications; it is not implemented yet.
6. Complete trusted local Codex registration, current tool visibility and client acceptance; reconcile protected instructions with the verified executable.
7. Update the ledger and deliver only evidence-backed results, with remaining work stated precisely.

No background automation or automatic resume was created.
