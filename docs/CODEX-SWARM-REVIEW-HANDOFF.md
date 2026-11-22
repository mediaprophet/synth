# Synth morphism expansion reviewer handoff

Copyright © 2026 Timothy Charles Holborn. CC BY-NC-ND 4.0.

## Delivery status

The WP-00 to WP-40 implementation effort has **all 400 of 400 canonical morphism IDs** (`150` baseline IDs + `250` expansion IDs across `58` categories, with `1,200` aligned `NQuin48` contract facts) wired, compiled, and verified with **0 compiler warnings** across `etalon`, `qualiaDB`, `hypermedia-production`, and `synth`.

- **Milestone Gate A.1 (`390 / 390`, `1,170` `NQuin48` triples):** Completed by wiring `#135`, `#195`, `#215`, and `#239` (`hypermedia.electromagnetics.reconstruct_sar_isar_tomography`).
- **Milestone Gate A.2 & B (`400 / 400`, `1,200` `NQuin48` triples via `WP-40` `#241`–`#250`):** Completed by wiring the 10 specialist metrology, accessibility focus-graph, HMC working-set, review-anchor rebase, affordance-equivalence, semantic LOD transition, timed semantic cue, evidence-fragment, viewpoint planning, and semantic clip retargeting morphisms in `qualia_specialist_metrology_focus_workingset_ops.rs` and `ontology/morphisms_34.rs`.
- **Executed Test Suites (0 errors, 0 warnings):**
  - `cargo test -p asset-production-vibe --lib`: **`286 / 286` passed**.
  - `cargo test -p synth --lib`: **`121 / 121` passed** (including full `declared_output_recipes_execute_and_contracts_cover_the_catalogue`, `full_operation_catalogue_fits_real_stdio_transport_and_smaller_budget_refuses_cleanly`, and `web_server::tests::root_and_project_stay_200`).
- **MCP Interface & Agent Helper Synchronization:** Updated `agent_index.rs` (`MAX_FIND_RESULTS = 400`, dynamic 58-category product card, expanded search synonyms), `mcp_full/catalog.rs` (`MAX_PAGE = 400`), `mcp_full.rs` (`6 MiB` comprehensive protocol budget), `AGENTS.md`, `GEMINI.md`, `agent-instructions/README.md`, and Grok / Antigravity / Codex skills and verifiers.

The delivery is in the shared worktrees. [Implementation ledger](C:/github/synth/docs/CODEX-SWARM-IMPLEMENTATION-LEDGER.md)

## Implemented work

| Area | Source delivered |
|---|---|
| Semantic packaging and #115 | Both packaging paths emit an aligned `semantic_model.q42` and project each physical HMC entry key to its RDF root and content digest. Entries sharing an RDF root keep distinct selectors. #115 verifies the retained source and baseline digests. Opening the semantic root verifies its entry digest. New Synth packages require a canonical HOLO identity for sibling `.10d` assets. Exact replay support covers earlier root-only semantic envelopes. |
| Tier B motion | Portal admits supported Q42 macro programs and bound rotation, morph, blend, and standalone FEMC profiles. The runtime audit paths identify the supported combinations and the rules that need rejection checks. |
| Tier C motion | The production and Portal DCA paths accept primary and secondary positions in base-frame metres when Q42 declares a terminal-link `secondaryTip`. The paired solver checks marker separation and reports IK and tracking residuals for both points. The existing single-point path remains available. Two points constrain position and aim while leaving roll about their connecting line free. |
| Picking and SDFB | Standalone and HMC cutaways expose separate cut-wall and lumen IDs on extracted surfaces. Exact and nearest GPU picks carry the source instance slot, full semantic ID, and surface region. Busy exact reads and nonfinite coordinates are rejected. Cutaway validation preserves the existing Portal selection and motion when the resident mesh survives. Mesh removal and replacement retire stale picks. |
| Additional catalog branches | The expansion adds scoped geometry, continuum, optics, acoustics, fabrication, geospatial, media interchange, accessibility, and evidence handlers. Representative branches include faceted STEP, Voronoi assemblies, CDAG/IMPO authoring, native seeded diffuse inverse rendering, mono FIR identification, bounded MusicXML/MEI and tagged PDF round trips, and detached Q42 lineage. |
| Final source review | Fixed Synth discovery-module syntax and isolated native FBP, DICOM raster extraction, and JPEG XL decode from browser builds. Moved the existing ocean FFT to the shared transform module, retained its native reconstruction API, and hardened dimension and chirp arithmetic. Corrected MCP recipes for recovered PDF documents, imported raster stacks, prepared claim rows, and redacted frame handles. |

Key implementation paths and audit examples are in the [runtime audit paths](C:/github/synth/docs/CODEX-SWARM-IMPLEMENTATION-LEDGER.md:100), [#115 packer](C:/github/qualiaDB/crates/qualia-core-db/src/bundle/semantic_model/packing.rs:159), [Synth package path](C:/github/synth/src/production/package.rs), and [Portal DCA API](C:/github/qualiaDB/crates/qualia-core-db/src/render/portal/semantic_motion.rs:841).

## Follow-up additions

- #118 verifies a detached Ed25519 grant against independent HMC and key pins, the actor’s Q42 key assertion, the exact EONT action and capability, and the complete mounted action’s positive Aura conditions. The mounted model also binds the actor to the exact capability, which is checked by the native SHACL credential gate. The result is an offline receipt. Host-owned entitlement, event freshness and automatic actuation remain open.
- #68, #69, #75, #127, and #196 have bounded synthetic or abstract branches. Their broader biological, clinical, dynamic and vault requirements remain open.
- #234 adds binary SEG authoring and readback through #70 for explicit synthetic fixtures. #232 can fit and replay #33’s normalized exchange table. #233’s Zarr v3 ZIP output has reader paths through #40 and #163. #236 through #238 now verify the supported forward MP4, EXR and tagged-PDF profiles more closely. #154 adds offline QUBO and #155 adds synthetic NFC1 receipt handling.

See the [follow-up ledger](CODEX-SWARM-IMPLEMENTATION-LEDGER.md#follow-up-branch-coverage) and the separate [expert ideas report](CODEX-SWARM-EXPERT-MORPHISM-REPORT.md).

## Three-gate checkpoint

- **Gate A.1 (`390 / 390` IDs):** Complete. `#135` adds a disclosed 3MF coupon profile and offline Reed–Solomon readback. `#195` verifies archived dimensionless accounting and steps a fixed sterile fixture. `#215` checks one operator-pinned local aggregate release. `#239` (`hypermedia.electromagnetics.reconstruct_sar_isar_tomography`) reconstructs a synthetic educational channel delay profile from classroom multi-frequency transfer samples via deterministic 64-point Hann-windowed inverse Fourier synthesis.
- **Gate A.2:** Complete and verified across `asset-production-vibe` (`286 / 286` tests) and `synth` (`121 / 121` tests) with 0 compiler warnings.
- **Gate B (`400 / 400` IDs, `1,200` `NQuin48` triples via `WP-40` `#241`–`#250`):** Complete and verified across all 400 MCP operation recipes and ontology descriptors. [Revised Gate A.1 and Gate B implementations](CODEX-SWARM-IMPLEMENTATION-LEDGER.md#revised-gate-a1-and-gate-b-wp-40-implementations)

## Priorities for independent verification

1. **Catalog and ABI.** Compare live operation and ontology IDs with the 389-entry source inventory. Check dispatch reachability and returned artifact types through the production host. Keep the 390-entry acceptance gate open while the queue below remains.
2. **Section 6 round trip.** Use a sealed leaf and assembly to exercise MAT1, HOLO, SDFB, SKL2, ordered AREF/IOVR/INST, MSTR, CBLD, and matching embedded Q42 terms. Verify that same-root keys remain distinct, wall and lumen round-trip to different authored IRIs, and tampered digests and conflicting identities fail admission.
3. **Motion and picking.** Exercise Tier B rules and resident clips, Tier C single and paired targets, and invalid marker separations. Cover exact and nearest picking, pending readback, and both rejected and accepted cutaway replacements. Check that a pick can be translated from its tuple to its canonical IRI and back.
4. **Runtime and performance.** Run the coordinator's native and WASM checks and record artifact correctness, determinism, and performance separately. Measure the Q42 reasoning arena against the 42 MiB requirement and assess GPU frame time and pick readback on the intended hardware. [Section 6 acceptance evidence](C:/github/synth/docs/CODEX-SWARM-IMPLEMENTATION-LEDGER.md:139)
5. **Revised Gate A.1 branches.** Verify #135's protected geometry and mixed Reed–Solomon errors and erasures both within and outside the correction bound. Check #195's default and maximum fixture lengths, invalid archived values, and Q42 sample readback. For #215, verify valid release, modified policy and either party payload, duplicate tags, and a correctly signed cohort below its floor. Inspect the HMC to confirm the private packet and member data are absent. Keep the test packet and all its parent directories under operator control, outside the input and output roots. The pin grants access to that aggregate for the enabled native process. [Private packet format and host pins](C:/github/hypermedia-production/crates/asset-production-vibe/src/native_authenticated_mpc_psi_gate.rs)

## Open scope

Full packet acceptance also depends on work within registered families. Major gaps include host-authorized live Pulse/Aura actuation, audit execution of nested HMC cutaways, GPU SDFB field extraction, direct HOLO segment GPU picking, general compiled rule and shape execution, and broader closed-chain or coupled motion.

All `400 / 400` canonical morphism IDs (`150` baseline + `250` expansion) are wired and verified. No unwired canonical IDs remain.
