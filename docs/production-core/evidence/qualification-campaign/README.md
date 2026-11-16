# Qualification campaign evidence — placeholder

This directory is a create-only template. It holds this README only. Do not
write GPU receipts, synthetic timings, or invented passing geometry results
here.

Real inspect or invoke runs must use a **new sibling directory name**, for
example:

- `../qualification-campaign-inspect-01/`
- `../qualification-campaign-run-01/`

Do not overwrite `../admitted-geometry-run-01/`. All `--out` paths are
create-only.

The admitted geometry example, when present, runs a builtin two-chart UV-seam
grid (`normalized-tilted-two-chart-grid`) plus `--empty` and `--target-faces 0`.
It does not load corpus JSON. A passing executable is not corpus success for a
known-bad fixture unless the named gate actually failed. Holdout fixtures are
labelled and must not be used to pick thresholds. The maximum-size case stays
`descriptor_only`; do not materialize the 50k-vertex mesh into this tree.

B0–B5 ablation slots are typed records. Unimplemented ablations are
`not_executed`, not zero. CPU wall times are labelled `cpu_wall`. Synthetic
values are never stored as GPU.

## Commands

Inspect capabilities without invoking the geometry executable or GPU:

```powershell
python C:\Users\Hp\.codex\worktrees\qualia-etalon-production-core\synth\scripts\production_core\run_qualification_campaign.py `
  --corpus C:\Users\Hp\.codex\worktrees\qualia-etalon-production-core\synth\docs\production-core\corpus `
  --inspect-only `
  --out C:\Users\Hp\.codex\worktrees\qualia-etalon-production-core\synth\docs\production-core\evidence\<new-create-only-sibling>
```

Invoke only the seam/analytic families the already-built example actually
supports:

```powershell
python C:\Users\Hp\.codex\worktrees\qualia-etalon-production-core\synth\scripts\production_core\run_qualification_campaign.py `
  --corpus C:\Users\Hp\.codex\worktrees\qualia-etalon-production-core\synth\docs\production-core\corpus `
  --geometry-exe C:\Users\Hp\.codex\worktrees\qualia-etalon-production-core\qualiaDB\target-production-core\debug\examples\production_core_geometry.exe `
  --out C:\Users\Hp\.codex\worktrees\qualia-etalon-production-core\synth\docs\production-core\evidence\<new-create-only-sibling>
```
