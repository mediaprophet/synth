# Interfaces implementation — 12 November 2026

## Implemented source (integration tests deferred by supervisor)

- P01: production vs generic Poet discovery explicitly names each execution host and handle lifetime. Canonical Hypermedia/species calls on the generic host return `wrong_execution_host`, pointing at `synth_operations` and `synth_produce_present`, without suggesting absent implementation.
- MCP initialize exposes backend instructions. Authoring protocol sets `isError` for returned error/failed status and non-null error objects, and enforces advertised closed top-level argument records. Qualia dispatch failures retain distinct argument/availability/permission/execution codes and bounded diagnostics.
- `cosmic_coordinates` rejects ignored/unknown per-operation keys (including `total_z`) and invalid types; HEALPix uses integer `z`, signed integer x/y, power-of-two positive nside <=2^29. Its schema matches these names. Computational geometry advertises the canonical geometry JSON arguments, including vertex_count/triangles.
- Metadata ABI descriptor enumeration now includes the existing Qualia species constructor at its declared manifold sub-id 2. Added a 148-way bijection regression; provider calls remain metadata schema-hash calls.
- `synth_operation_contracts` returns all 148 IDs with explicit host/lifetime/domain/codomain, selected typed resource/record recipes and two executable PNG composition examples. Unknown output types remain explicitly operation-specific; no namespace guesses.
- P02 MCP accepts `inputs:[{key,kind,path}]` under native `SYNTH_INPUT_ROOT`, with canonical confinement, root identity, regular file, no link/junction traversal, per-resource and aggregate limits. Output requests remain create-only names under `SYNTH_OUTPUT_ROOT`.
- MCP accepts `export_dir`, `receipt_json`, and `preview_assets:[exported .10d keys]`. Exports retain `.synth/source.vibe`, a production receipt and a separate review completion record; each native review has its own renderer receipt. Native previews run the selected executable without a shell, drain bounded diagnostics and enforce a 180-second bound. Failed reviews preserve exports and return a structured error.
- Production receipts include producer/version, executable path/version/SHA-256, catalog count/hash, source/input/package/resource hashes, absolute saved paths and inspection status. Durable files remain available after client/session restart.
- P03 integration bridge: files/v2 automatically traverses selected compiled resource `textures` dependencies, verifies SHA-256, MIME and encoded kind, de-duplicates equal dependencies and refuses conflicts. Canonical texture resources are global HMC `textures/sha256/<digest>` with MIME kinds, matching existing Qualia resolver. Exports preserve the same tree. Original files/v1 interpretation remains available for strict reopen, without falsely declaring v1 packages upgraded.

## Files owned

Synth: `src/mcp_full.rs`, new `src/mcp_full/{production,contracts,tests}.rs`; `src/production/{files,mod,package,tests}.rs`.
Qualia: `crates/qualia-mcp/src/authoring/{types,protocol}.rs`, `src/registry.rs`, `src/tools/{vibe,living_cosmic}.rs`.
Hypermedia: `crates/asset-production-vibe/src/ontology/vibe_descriptors.rs` only.

## Regression coverage written

- MCP input/source recipe produces byte-identical HMC to direct producer; exported bytes, durable receipt/source and absolute references verified.
- Unconfigured/outside/traversing inputs and unknown arguments refused before output creation; invalid preview usage refused.
- Actual AuthoringServer initialize/wrong-host tool calls exercise instruction publication and MCP isError.
- Declared PNG recipes execute and inventory covers 148 entries.
- Original files/v1 package reopens byte-exactly under legacy semantics.
- Selected mesh texture fixture transports shared encoded bytes once, resolves via the actual Qualia HMC resolver and rejects tampered digest.
- HEALPix/schema and computational-geometry/schema argument regressions.
- Metadata descriptors are bijective with the 148-entry ontology.

Run after supervisor integration gate: Synth library tests; qualia-mcp tests with authoring+engine-tools; asset-production-vibe ontology tests. Use the isolated review target selected by root. No build, test, live-process restart or live-target write has been executed by this workstream.

## Bounds and remaining validation

Native GPU preview subprocess and actual client reconnection need assembled integration runs. A created configuration is not live client acceptance. Most morphology argument prose is still represented as domain-specific contracts rather than a generated exhaustive JSON schema; typed MCP input/output delivery and explicitly declared export recipes are enforced. Stored review PNGs remain pending human/reference acceptance. Receipt output installs are create-only, with provenance saved before native review; an OS failure can leave completed partial outputs and is reported without claiming rollback.

## Independent UI/client source review

Validated Antigravity example against the vendored `types.McpStdioServer` fields. Found and repaired quoted JSON keys injected as Vibe records (the real parser requires identifier keys), replacing this with bounded recursive literal serialization and invariant numbers. Corrected stale Grok skill counts, isolated executable path and explicit mesh encoder example. Retained-font glyphs without supported vector outlines now refuse explicitly instead of silently disappearing; whitespace/joiners remain valid blanks. The project plugin `.agents/plugins/synth-qualia/mcp_config.json` still points at the old live target and lacks roots; that read-only surface requires a trusted configuration update rather than an incidental rewrite. Client acceptance remains deferred.

Final source review follow-ups: HMC preview flags now forward the actual package and canonical outputs/<key>, with a private candidate HMC retained when necessary. The CLI preview helper forwards identical flags. Legacy global texture-key promotion is explicitly gated on files/v2. Native texture transport preserves the existing image/webp case. PNG-sequence encoded manifest leaf dependencies are admitted, bounded, conflict-checked and packaged automatically. The metadata contract includes mesh_to_sdf_brick's preserve/discard policy. Source diff checks are clean; compilation, all unit/integration tests and actual GPU/client runs remain supervisor-gated.


Resumed regression repairs and causal newline-parity proof: see interfaces-regression-repairs.md. Source repairs are written; root test/build results remain pending.


EMF v3 implementation and coordinated test handoff: see interfaces-emf-v3.md.
