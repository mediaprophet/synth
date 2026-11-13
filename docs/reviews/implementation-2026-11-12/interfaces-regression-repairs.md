# Resumed interface regression repairs

Read PAUSE_HANDOFF.md and interfaces-pause-handoff.md and inspected current source/diffs before edits. Root confirmed no active overlap. No Cargo/build or test suite was run by this worker; root owns scheduling. Existing EMF/M64/streaming/nquins source was left intact.

## Five repairs written; root tests pending

1. Synth production16 fixture now emits a valid seven-byte note and verifies output plus resource/aggregate/package/work limits and invalid-profile refusal.
2. Synth recipe regression resolves declared resource_key rather than assuming sorted output[0] is the manifest. Recipe metadata now declares keys/kinds. The sequence assertion still requires exactly sequence.json plus both PNG leaves and checks their kinds/digests.
3. Species and sweep both declared StableFuncId(2,24). Inventory of every manifold descriptor established 1..26 occupied and 27 free. Species now uses (2,27); sweep and every other existing identity remain intact. The 148-way uniqueness test remains strict; no operation ID/catalog count changed.
4. png 0.18.1 decoder stores raw declared gAMA/cHRM in gama_chunk/chrm_chunk; source_* aliases were absent in the failures. graphics.rs reads raw gamma with source_gamma fallback and checks either chromaticity field. Unsupported ICC/custom gamma/primaries/HDR metadata still refuses. The gamma fixture now first verifies an actual gAMA=0.7 declaration.
5. Gradient regression verifies the explicit raw gAMA remains 1.0, retains >900 levels and exact alpha, reloads through the production importer as Linear transfer, and requires contradictory sRGB source metadata to refuse. No precision/colour assertion was removed.

Changed source: Synth src/mcp_full/{tests,contracts}.rs and scripts/verify-local-clients.py; Hypermedia asset-production-vibe src/graphics.rs, src/radiance_precision.rs, src/ontology/morphisms_09.rs. Metadata slots were inventoried directly from all morphisms_*.rs declarations before repair.

## Original CLI/MCP discrepancy: causal confirmation

Fixture: production/client-parity-20261112-123529-f536dfcc.

CLI source is 884 bytes with 9 CRLF endings, SHA 8c03ac2de2d18eabfba9e03dda3a9299a53c2ab0d52589ae427bb689a20b0a11.
MCP source is 875 bytes with 9 LF endings, SHA 6a367b83a54b591b2199859c50887a8a3459e987f0f091007dba9503bac80455, matching its production receipt.
Source content differs only by these newlines. Both HMC manifests are identical except source strings; all four exported resources and both input digests match exactly.

A CLI run using the exact saved MCP source bytes and original inputs on the original frozen executable produced a package byte-identical to original MCP HMC: f0729d78eba42291c304006136a45552e4d3e863d1305cc38d5ae215b099e2c5. Frozen executable SHA was checked against the original receipt: 46962750e1d6d87cc01e18d007a0772defcc97b624097b632c515b9e083aed99.

Evidence: production/client-parity-20261112-123529-f536dfcc/newline-parity-evidence.json; proof source/log/package under newline-proof-52912310 in the same directory. Original artifacts are preserved. This is OLD-runtime GLB/PNG/UI evidence only, not qualification of current EMF/M64 changes.

Harness now writes source.encode('utf-8') with write_bytes and asserts exact equality to the bytes encoded for MCP. Producer source/replay semantics and HMC equality checks are unchanged. Root should rerun the full harness against a newly frozen assembled executable after repairs.

## SDK/process state

SDK dependency/network setup remains deferred. Existing .venv-antigravity is preserved; no successful new SDK install was observed and no aborted install was retried. Repair/proof exec session 78402 completed exit 0; no worker-owned live process remains. Host tools were delayed, so no redundant test/build processes were started.
