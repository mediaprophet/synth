# Packet P03 — agent report

Agent: p03-decode. Date: 16 November 2026.
Workspace: `C:\github\synth` on the updated root (`discover_worktrees()` parents resolve to `C:\github`).
Status: harness remaining work executed; packet not accepted; campaign not qualified.

This is a source-and-Python-harness report. It is not P03/P22 acceptance.

## Objective coverage

1. Independent sealed `.10d` decode harness is present, callable, and executed on existing evidence `.10d` files. Decoder is pure Python QuantizedMesh from the in-tree layout (`qualiaDB/docs/manuals/standards/10d-container-format.md` and `container_10d/{header,section,mesh_section}.rs`). It does not trust `geometry-receipt.json` for counts or positions. Sidecar `decoded-vertices.json` / `decoded-triangles.json` are a cross-check only; mismatch would be labelled, not repaired. Seven existing evidence `simplified.10d` files decoded; sidecar mismatch count 0.
2. Collapse provenance inspect helper hashes a snapshot when present. All inspected evidence directories had no snapshot; status is `not_supplied` with `sha256=null`, not 0.0.
3. B0 identity ablation helper is inspect/CPU only. Five published identity fixtures compared; `identity_preserved=true`; GPU not invoked; `qualified=false`; slot `error`/`cost` remain null.
4. `known_bad`, `holdout`, and `maximum_size_descriptor` remain in `SKIP_INVOCATION_FAMILIES` and are not invoked.
5. Unit tests run from `C:\github\synth\scripts\production_core`. No cargo. No synth rebuild. No GPU.
6. `INCOMPLETE_CORE_CHECKLIST` updated only to honest states. `final_10d_sealing_independently_decoded=executed_not_qualified`. `collapse_provenance_qualification=inspected_not_supplied`. Neither item is qualified. `p01_p03_accepted` remains `open`.

## Paths written (create-only evidence; harness/tests under lease)

| Path | Role |
|---|---|
| `C:\github\synth\scripts\production_core\decode_10d_harness.py` | Independent QuantizedMesh decoder, provenance inspect, B0 CPU ablation, evidence CLI |
| `C:\github\synth\scripts\production_core\test_decode_10d_harness.py` | Unit tests for decoder, provenance, B0, skip families, live evidence decode |
| `C:\github\synth\scripts\production_core\run_qualification_campaign.py` | Wire independent decode, provenance inspect, B0 CPU ablation; keep skip families |
| `C:\github\synth\scripts\production_core\test_qualification_campaign.py` | Campaign tests for B0 CPU inspect, provenance `not_supplied`, skip families |
| `C:\github\synth\scripts\production_core\production_core_artifacts.py` | Honest checklist states only |
| `C:\github\synth\docs\production-core\evidence\integration-wave-p03\decode-receipt.json` | Create-only decode receipt |
| `C:\github\synth\docs\production-core\evidence\integration-wave-p03\agent-report.md` | This report |

Existing evidence directories were read-only inputs. No copies of `.10d` files were written. No Rust, Cargo.*, lease, board, or prior evidence files were edited.

## Independent decode execution

Decoder: `pure_python_quantized_mesh_section`. CRC-32C of the QuantizedMesh payload is verified (RFC 3720 check value `123456789 → 0xE3069283`). Dequantization follows the documented endpoint bit-for-bit / interior f64→f32 contract. `geometry-receipt.json` is hashed as `present_not_trusted` only.

| Evidence directory | simplified.10d sha256 | V | F | sidecar | CRC |
|---|---|---:|---:|---|---|
| `joined-tracer-01/geometry-1` | `13e7d5c2…cfd72d` | 26 | 24 | match, 0 bit mismatches | match |
| `joined-tracer-01/geometry-2` | `13e7d5c2…cfd72d` | 26 | 24 | match | match |
| `joined-tracer-01/geometry-3` | `13e7d5c2…cfd72d` | 26 | 24 | match | match |
| `qualification-campaign-run-02/geometry-identity-two-subpart-seam` | `7a0ad9a8…ca91f09` | 6 | 2 | match | match |
| `qualification-campaign-run-02/geometry-analytic-plane-scale-1` | `c0c7f34f…279efa2` | 4 | 2 | match | match |
| `geometry-corpus-identity-seam-01` | `7a0ad9a8…ca91f09` | 6 | 2 | match | match |
| `admitted-geometry-run-01` | `13e7d5c2…cfd72d` | 26 | 24 | match | match |

Capability: present / callable / executed. Not qualified. GPU: false. Invented passing geometry receipt: false.

## Tests actually run

Working directory: `C:\github\synth\scripts\production_core` (required for imports).

```
python -m unittest test_decode_10d_harness test_qualification_campaign test_production_core_artifacts
```

Result: 50 tests, 0 failed, 0 skipped, exit 0. Re-run after checklist update: 50 OK.

New tests cover CRC-32C, endpoint dequantization, synthetic roundtrip, refusal to trust receipt counts, sidecar mismatch labelling, missing-file honesty, provenance `not_supplied` vs hashed present, B0 CPU identity pass/mismatch/empty, and skip of known-bad/holdout/maximum-size. Live tests decoded `joined-tracer-01/geometry-1` and campaign identity `simplified.10d`. Campaign inspect-only now records B0 as CPU-executed and provenance as `not_supplied`. Fake invoke still does not invent a `.10d` pass (`independent_decode=missing_file`).

Not run: cargo, geometry exe, Forge GPU probe, synth produce, maximum-size materializer.

## Remaining gaps

- P03 is not accepted. Fixture publication and this harness are not production qualification.
- Independent decode does not certify UV/material/normal sidecars, whole-surface enclosing bounds, or GPU-derived `.10d` (`export-*/gpu-derived.10d` was not decoded in this receipt).
- Collapse provenance snapshot is `not_supplied`. P07 hashable lineage export was not found next to evidence `.10d` files. Qualification remains unexecuted.
- B0 compared published identity candidates on CPU. Production no-reduction path was not invoked. B1/B3/B4/B5 remain `not_executed` with null error/cost, not zeros.
- Live campaign re-invoke of `production_core_geometry.exe` was not performed this packet. Campaign wiring for independent decode is unit-tested; existing evidence `.10d` files were decoded read-only.
- `p01_p03_accepted` remains `open`. `operator_or_physical_qualification` remains `not_executed`. `whole_surface_bound` remains `not_measured`.
- Do not treat sidecar JSON agreement as a substitute for a future independent whole-surface certificate.
