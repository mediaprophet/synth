# P09 agent report (p09-operator-tests)

Date: 16 November 2026. Packet: **P09 remaining** (true algebraic residual unit tests).
Agent: p09-operator-tests. Lease: `qualiaDB/.../ddg_production/**` only, plus this
create-only evidence file.

Source only. **Cargo was not run.** No commits. No MCP kill. This is not
qualification. P00 remains the only accepted packet.

## Touched paths

| Path | Change |
|---|---|
| `C:\github\qualiaDB\crates\qualia-core-db\src\specialized_libs\computational_geometry\ddg_production\tests.rs` | Extra tests + test helpers |
| `C:\github\synth\docs\production-core\evidence\integration-wave-p09\agent-report.md` | This file (create-only) |

Not edited: `algebraic_residual.rs`, `conventions.rs`, `mod.rs`,
`computational_geometry/mod.rs`, `production_core_geometry.rs`, `Cargo.*`.

Implementation of `algebraic_residual` (`||b − A x||`) and conventions was
already present. `DdgReport.residual` remains iterate change
`maxᵢ |x⁺ᵢ − xᵢ|`. This wave did not rename that field.

## What the extra tests assert (source, unexecuted)

1. **Constant Neumann on the unit square.** `b = 0`, `x = 1`, all free
   (Neumann). `algebraic_residual` under `ResidualNorm::EuclideanTwo` is near
   machine/scale zero: `64 u (||A||_F ||x||_2 ∨ ||b||_2 ∨ 1) n`. Independently
   reconstructed via `apply_operator` as `||b − A x||_2`. This is not iterate
   change.
2. **Zero Jacobi iterations on the same fixture.** `DdgReport.residual`
   (iterate change) stays `∞` with `iterations = 0`. `algebraic_residual` is
   still `||b − A x||_2` near machine/scale zero.
3. **Linear precision kept.** `planar_grid_has_linear_precision_on_free_dofs`
   was not changed.
4. **Rejection stays labelled.** `UnsupportedSize`, `IllConditioned`,
   `InvalidInput("empty_vertices")`, and `DimensionMismatch` remain `Err`.
   Existing `oversized_dense_problem_is_rejected` and
   `ill_conditioned_diagonal_is_rejected` were not weakened. A homogeneous
   ill-conditioned problem can have raw `||b − A x|| = 0` at `x = 0`; the
   instrumented solve must still return `IllConditioned`, never `Ok` with a
   silent `0.0` residual.

## Tests added (unrun)

Helpers in `tests.rs` (not production API):

- `euclidean_machine_scale_zero`
- `independent_euclidean_residual`
- `panic_if_silent_zero_residual`

New `#[test]` functions:

- `constant_neumann_unit_square_two_norm_is_machine_scale_zero`
- `constant_neumann_zero_iteration_keeps_iterate_change_distinct`
- `rejected_unsupported_size_is_labelled_not_silent_zero_residual`
- `rejected_ill_conditioned_is_labelled_not_silent_zero_residual`
- `empty_and_mismatched_problems_are_labelled_not_silent_zero_residual`

Kept (unchanged):

- `constant_field_is_the_neumann_nullspace`
- `planar_grid_has_linear_precision_on_free_dofs`
- `algebraic_residual_is_not_iterate_change`
- `oversized_dense_problem_is_rejected`
- `ill_conditioned_diagonal_is_rejected`
- remaining prior module tests

## Remaining gaps

- Tests were **not compiled or executed** (lease: no cargo). Coordinator
  compiles into `target-production-core`.
- Geometry example `operator_residual.status: not_measured` is out of this
  lease (coordinator: constant-Neumann residual on decoded mesh when size
  admits).
- Analytic/nullspace fixtures and heat-route stability remain unrun.
- Packet is not qualified. Source is not execution. Execution is not
  qualification. Do not accept P09.
