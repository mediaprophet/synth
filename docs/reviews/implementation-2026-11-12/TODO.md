# Local implementation ledger — 12 November 2026

Supervisor: root. Work is accepted only after integration, independent review and actual output inspection. Tests are deferred until implementation is assembled, as requested. Existing working-tree changes and live servers are preserved.

## Assigned implementation

- [ ] P01 routing, errors, schemas and descriptor parity — interfaces agent.
- [ ] P02 retained inputs, exports and MCP review access — interfaces agent.
- [ ] P02 local Codex registration, client parity, Grok/Antigravity adapter repairs — root (delegate next wave).
- [ ] P03 lossless import, seam topology, full tangents — asset_data agent.
- [ ] P04 material bindings and accurate shader capability claims — asset_data agent.
- [ ] P04 durable shot/camera/light/render controls — root (delegate next wave).
- [ ] P05 retained physical heightfields and coordinate metadata — fields_animation agent.
- [ ] P06 complete skin binding, sealing and pose evaluation — fields_animation agent.
- [ ] P07 real raster labels and reproducible glyph rendering — root.
- [ ] P08 precision, bounded production profiles and sequence timing — next wave.
- [ ] P09 seven-domain reference assets and reviewed visual receipts — next wave.

## Integration and acceptance

- [ ] Assemble changes; inspect diffs for compatibility and preserved data.
- [ ] Independent supervisor review; add all findings below and repair.
- [ ] Build separate preview-enabled target without replacing live servers.
- [ ] Run unit, integration, MCP/CLI/client parity and deterministic replay checks.
- [ ] Reproduce original failures and demonstrate corrections.
- [ ] Render and inspect reference outputs; repair visual defects.
- [ ] Validate actual Codex registration and tool visibility, distinguishing current-session refresh limitations if any.
- [ ] Review all receipts and remaining limitations before accepting work.

## Discovered work

- [ ] D01: legacy instructions still name stale binaries/counts and nine preview views; generate or correct client documentation with current contracts.
- [ ] D02: client adapters must support typed record outputs and encoding, not infer output type from a morphism namespace.
- [ ] D03: imported texture resources must be automatically packaged at the existing content-addressed HMC keys and resolved by native preview.
- [ ] D04: support normal static GLB node transforms and instancing using mesh ranges, inverse-transpose normals and tangent handedness.
- [ ] D05: existing Antigravity documentation imports a non-existent module; use the actual `google.antigravity.types.McpStdioServer` API.
- [ ] D06: font inputs must be retained, bounded and independent of installed system fonts; raster and SVG must share shaped glyph geometry.
- [ ] D07: preserve old files/v1 package reopening while introducing the texture-aware files/v2 producer.
- [ ] D08: geometry edits must preserve imported surface attributes or explicitly refuse; deliberate discard creates a separate geometry revision.
- [ ] D09: changed UV charts invalidate existing textured appearances; refuse implicit reuse and retain untextured normals/material mappings.
- [ ] D10: reject unsupported GLB extensions, duplicate documents and allocation claims before import; retain supported static scene transforms.
- [ ] D11: align raster/SVG disabled, hover, focus and corner geometry; refuse fonts without usable outlines instead of dropping labels.
- [ ] D12: carry HMC and asset keys into MCP native review so referenced textures actually reach the renderer.
- [ ] D13: retain full 16-bit pixels and colour metadata; deliver bounded lossless sequences with exact rational timing and packaged dependencies.
- [ ] D14: expose coordinate frames in terrain review receipts and select cameras appropriate for ENU Z-up geometry.
- [ ] D15: explicit smooth assembly shading must use existing authored normal transport; keep flat shading selectable and refuse failed material sealing.
- [ ] D16: repair reference recipes that use reserved Vibe identifiers or invalid theme scope; retain exact source bytes across CLI/MCP parity checks.
- [ ] D17: repair scale-dependent normal normalization for physically small geometry; preserve strict .10d normal validation.
- [ ] D18: correct tube profile winding and ventricular geometry, then inspect all native views again.
- [ ] D19: keep a frozen executable for rendering/client checks so Cargo tests do not collide with Windows executable locks.
- [ ] D20: extend EMF's explicit colour interpretation and timed audio-track contract beyond human-visible/audible-only assumptions; inspect existing codec/consumer owners first.

## User format steering (12 November, during verification)

The user authorizes full .10d format upgrades where required. Preserve versioned compatibility and test readers, writers and consumers together; format changes alone do not fix authoring or renderer wiring defects.

The user requires EMF to contain colour-space interpretation and audio tracks, with spectral scope broader than human vision/hearing. Existing E/B field values remain authoritative; display/listening conversions must be declared rather than substituted for source data.

Resumed direction: M64 is current; E64 is legacy compatibility. Follow the updated QualiaDB standards. The external full-BLAKE3 integrity repair is authoritative; preserve its explicit legacy admission and leave the separately owned streaming work intact.

## Resumed verification and newly reproduced defects

- [ ] D21: expose actual integrity/version in verification and presentation; remove unconditional Bao claims. EMFv2 and M64 codec3 use full BLAKE3; raw/GRM3 profiles remain distinct.
- [ ] D22: rebuild and qualify the served QualiaPortal WASM, which predates the integrity repair.
- [ ] D23: repair portable fauna animation using authored joint landmarks; generic pivots disagree with the authored mesh and separate limbs in the reviewed frames.
- [ ] D24: honor hidden anatomical-layer alpha in the native preview; inspect the complete skin recipe and its sealed SRD1 data.
- [ ] D25: qualify closed boolean output with the existing bounded conforming implementation; the ventricular union has 462 boundary edges and is not accepted as a closed solid.
- [ ] D26: verify installed Codex discovery and a real MCP call in this existing chat using the local client protocol. The first unprivileged proxy check could not open the local control socket.
- [ ] D27: admit combined live memory before EMF copies, field allocation and preview work, for both default v2 and opt-in v3. Supervisor reproduced omissions in verify/slice/raymarch and default-v2 compilation; add early-refusal regressions.
- [ ] D28: preserve authored fauna landmarks through regeneration. The shared wallaby source still held the old 18-joint recipe; reconcile it before regenerating reference assets.

The five pre-pause regression defects have source fixes; final integrated tests remain pending. CLI/MCP package parity is proven when both clients receive identical UTF-8 source bytes (the former mismatch was CRLF versus LF). The user explicitly approved localhost web tests after automatic permission review timed out twice. Current preflight excludes web tests; run the permitted web suite after the shared build completes.

## First verification evidence

- Isolated preview build passed (11m38s), recorded in `build.log`. Snapshot binary `production/review-runtime-20261112-123308-eeb47710/synth.exe`, SHA-256 `46962750e1d6d87cc01e18d007a0772defcc97b624097b632c515b9e083aed99`.
- Full 2048×2048 retained 16-bit gradient, grade, exact 24000/1001 sequence, dependency hashes and seam-crossing tile pack passed. Graded first row retains 2048 distinct red values. Evidence: `production/precision-20261112-122842-c055b68e/results.json`.
- Synth library initial run: 39 pass, 12 fail. Ten failures were localhost socket permission; permitted local web rerun passed all 13 web tests (`tests-local-web.log`). Two new fixture assertions are being repaired; full rerun still required.
- Hypermedia initial unit run: 203 pass, three failures in descriptor/precision regressions; assigned to interfaces worker. Evidence: `tests-hypermedia.log`.
- CLI/MCP/Grok actual retained textured cube production and review run succeeds after theme fix; Grok/CLI packages match. MCP byte parity discrepancy is under source-byte investigation (`production/client-parity-20261112-123529-f536dfcc`). Cube checker placement and UI raster text inspected; this is not blanket asset-quality acceptance.
- Initial seven-domain run is `production/quality-20261112-122914-069f4c25`. Reserved identifiers in cell/terrain were corrected in later revisions. First heart rejected visually; revised geometry, tube winding, smooth shading and scale normalization remain under repair.
- Native Codex app inspection timed out on app/window inventory; no UI settings changed. Codex registration and actual chat tool visibility remain open. Repository-owned installation candidates are under `vendor/codex`.

## Implementation progress (not acceptance)

The interfaces worker has written routing/schema/error repairs, input/export receipts and files/v2 texture dependency transport. The asset worker has added imported-attribute retention, atlas seams/topology and tangent transport; static scene transforms are being completed. The fields worker has added direct f32 terrain records and SKL1 binding and is now extending shot rendering and posed preview. Root has implemented shared UI glyph outlines with retained OpenType shaping, a typed invocation helper and local Codex/Antigravity adapter material. All are pending integrated build, tests, independent review and visual acceptance.

No item is complete merely because code was written or a test passes.

## Continued local verification — 12 November, evening

- [ ] D29: repair the authored knee loft frame flip using existing Bishop transport; Q source fix and an integrated H regression are written, fresh visual verification pending.
- [ ] D30: remove foliage pole/seam defects and separate translucent leaf teaching layers from opaque lamina; source reviewed, integrated tests and new renders pending.
- [ ] D31: retain one bounded accepted EMF across early GPU initialization and device recovery; honest pending/uploaded/unavailable state is implemented, failure transitions and actual WASM pixels remain to qualify.
- [ ] D32: admit the actual 148-operation MCP catalogue in the standalone transport. The real installed Antigravity SDK handshake/list succeeded, then reproduced the internal response-budget refusal. The bounded 4 MiB server repair and wire-level regressions are written.
- [ ] D33: repair obsolete test fixtures without weakening integrity, explicit attribute-loss policy or memory admission. Valid sealed presentation fixtures, explicit production16/discard fixture, dispatch-work assertions and analytic/replay Boolean checks are written.
- [ ] D34: improve open heart vessel tessellation and terrain view/shading; inspect all views after each meaningful revision.

Latest completed integrated evidence: build-integrated-resume-2.log passed; Hypermedia 212 passed/7 failed, Synth library 50 passed/3 failed (all 13 web tests passed), asset integration 16 passed/1 failed, profile 10 passed. Remaining failures are addressed in source but are not yet green. Actual codec harness passed 67. See compiled-tests-results.json and tests-integrated-resume.log.

Frozen candidate a8921532631d57e67198a4c29122674acaec8a24a68c0e0116d1565e072c92fd at production/review-runtime-20261112-212521-de79a6b3 predates D29–D32. Fresh CLI/MCP/Grok parity passed at production/client-parity-20261112-212641-dcde5373. Exact original PCM, metadata and deterministic EMF recording production passed at production/emf-recording-20261112-213942-a333ceec; root inspected the brighter declared derivative. Display exposure changes left original EMF bytes unchanged. No listening or browser pixel acceptance yet.

Codex global synth-local registration succeeded using the supported installed CLI and frozen candidate. Desktop proxy still fails OS10050 before initialization. Normal-permission standalone discovery cannot open the protected desktop state database; elevated discovery approval review timed out twice. This is not client-call acceptance. The installed official Antigravity 0.1.23 Windows wheel and dependencies are local; user-authorized downloads completed and pip check passed. No model-provider calls were made.

Root inspected all three terrain/cosmos native views plus shots, all three current fauna/heart native views, biology and cell views, and the interface PNG. Terrain was faceted and poorly framed; planet, fauna and heart remain stylized illustrations. None is accepted as film-quality or anatomical validation. Updated source and browser/runtime verification remain underway.

- [ ] D35: restore the opaque .10d reader metadata-overlap check after the concurrent strict-validator refactor; preserve unknown tags/reserved fields for explicit opaque retention. Build3 failed before tests on this stale call and partially saved crypto module declarations; the external owner has since wired the crypto modules. Do not duplicate those changes.
- [ ] D36: connect the existing GPU EMF slice pipeline to an actual bounded Portal surface preview. Normal Portal frames currently do not draw the uploaded field; actual browser pixels remain mandatory.
- [ ] D37: diagnose residual planet pole banding and terrain fine stripes with paired shadow/AO controls, then repair the demonstrated cause. Disabling an effect is diagnostic evidence, not final defect acceptance.

- [ ] D38: replace fixed 64 KiB source/history admission with asset-aware bounded memory/work accounting shared by CLI, scoped MCP source files, session submission and strict replay. Preserve source bytes and explicit admission receipts; do not reduce authored detail to fit a constant. User requested this during visual review.
- [ ] D39: local real-browser launch approval review timed out before execution twice. Ordinary-permission isolated WebGPU probe is pending; no browser pixel success is claimed.

Root independently reviewed the paired shadow/AO images: disabling shadows alone removes planet pole rings and terrain streaks. The provider receiver-plane comparison repair is underway. Root reviewed all three 32-sided solidified heart views (`production/quality-20261112-230958-5214d6d2`): smoother round vessel lips and retained taper are improved, but the external shape remains a simplified illustration. The south-side terrain view (`quality-20261112-225513-6f8d5a17/terrain-shot-south`) preserves authored Z-up geometry and makes the relief upright. Build4 failed on two concurrently introduced streaming types; both are now narrowly corrected in source, tests still pending.

- [ ] D40 (supervisor P2, paused): prepare_edit must check cumulative source history before cloning inputs/replaying. Add a producer-call-count regression proving refused oversized edits do not replay. All workers paused; implement only after user resumes.

## Second pause, 13 November 2026 — supersedes older statuses

- [x] D40: cumulative history check before cloning/replay, independent source review and counting-producer regression passed.
- [x] D38 core source-budget qualification: real92,574-byte heart CLI/replay/MCP parity and three views reviewed; logical allowance limitations preserved.
- [x] Actual installed Antigravity SDK transport full148/production/replay/reconnect/refusals and CLI parity passed; no provider call.
- [ ] D41: run corrected MCP initialized-notification regression.
- [ ] D42: qualify boundary-only extrusion cap repair against original cylinder/pyramid exact Boolean collision, closure/replay tests.
- [ ] D43: diagnose repeated LNK1104 output-executable failures without touching live MCP; finish integrated rerun.
- [ ] D44: fresh current-chat Codex connection refresh/call acceptance; registration/templates updated but live connection still old.
- [ ] D45: fix Q current WASM crypto cfg imports; qualify saved queue.present glue; build real local-provider Q cdylib, actual browser checks before package promotion.
- [ ] D46: run targeted shadow tests, xtask path regression and required full Etalon GPU suite with verified local prerequisites.
- [ ] D47: resolve/report residual terrain flecks by correlating geometry and actual shadow taps, not numeric threshold alone.
- [ ] D48: flora clipped tip/lowcontrast, canopy darkpatches, flower exploded-stage intent, fauna darklimbs; source-aware visual repair and final rerender.

Paused at user request; see PAUSE_HANDOFF.md and latest worker notes.

## Implementation-only scope, latest verification 13 November

User explicitly deferred creative asset refinements except test fixtures. D34/D48 and unproven residual visual diagnostics are deferred, not accepted as film-quality.

- [x] D41 corrected actual MCP handshake regression: Synth62/62.
- [x] D42 original cylinder/pyramid Boolean + strict replay: asset integration18/18 and H224/224.
- [x] D44 actual refreshed Codex chat: full148catalog, retained production, exact CLI/repeat parity, invalid dual-source refusal and3native views. Final4d2 runtime additionally passes adapter/SDK/source/precision/EMF regressions; registration finalization pending end of builds.
- [x] D45 current Portal WASM builds with local Etalon provider; actual Chrome/WebGPU metadata/PCM/lifecycle/corruption/pixels passed. Verified served package promoted with local backup and repeated browser check passed.
- [x] Profile-authoring10/10 from actual current source/deps linked to fresh unique output after Windows LNK1104. No claim underlying OS access issue was repaired.
- [x] Current source codec/lifecycle71/71.
- [x] Renderer shadow3/3 and isolated-target runner1/1.
- [ ] D46 full Etalon suite compiling; result pending.
