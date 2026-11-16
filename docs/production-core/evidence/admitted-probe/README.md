# Admitted geometry probe — placeholder evidence

This directory is a create-only placeholder. It holds this README only. Do not
write GPU receipts, timing samples, or invented passing geometry results here.
After the QualiaDB example is rebuilt, use a **new sibling directory name**.

Capability states are distinct:

| State | Meaning |
|---|---|
| present | Library or example source exists on disk |
| callable | A binary exists that can invoke that path |
| executed | This process actually ran that binary |
| qualified | Independent acceptance evidence exists |

Source on disk is not callable. Callable is not executed. Executed is not
qualified. The admitted QualiaDB path is **present** (example wired to
`decimate_semantic_admitted_3`). It is **not callable** until the example is
rebuilt. It has **not been executed** by this probe. It is **not qualified**.

The joined tracer checking three geometry asset builds and three HMC builds is
**not** three independent GPU runs with byte-identical timing. GPU-identical
timing is not claimed.

## SHA-256 LowerHex history

The geometry example previously failed to compile:

```
error[E0277]: the trait bound `Array<u8, ...>: LowerHex` is not satisfied
format!("{:x}", Sha256::digest(bytes))
```

Log: `qualiaDB/target-production-core/geometry-build-01.log` (cargo exit 101).
Observed log span 06:52:05.969–07:32:30.054 UTC is not a full command duration.

The repair is in example source (`sha256()` encodes digest bytes as lowercase
nibbles) and **must be rebuilt later**. No passing geometry receipt is invented
from the source repair.

## Commands to run later

Do not run cargo into Synth `target` or `target-studio`. Do not reuse an
existing evidence directory name. All `--out` paths are create-only.

### 1. QualiaDB example `production_core_geometry` (admitted semantic path)

From the isolated QualiaDB worktree, after a later consolidated rebuild:

```powershell
$env:CARGO_BUILD_JOBS = '1'
$env:CARGO_INCREMENTAL = '0'
$env:RUSTFLAGS = '-C debuginfo=0'
cargo --config .cargo/production-core.toml build --offline --no-default-features -p qualia-core-db --example production_core_geometry --target-dir target-production-core
```

Then invoke the admitted driver example (this is the semantic path; it is not
GPU):

```powershell
& "C:\Users\Hp\.codex\worktrees\qualia-etalon-production-core\qualiaDB\target-production-core\debug\examples\production_core_geometry.exe" --out <new-create-only-dir>
& "C:\Users\Hp\.codex\worktrees\qualia-etalon-production-core\qualiaDB\target-production-core\debug\examples\production_core_geometry.exe" --out <new-create-only-dir> --empty
& "C:\Users\Hp\.codex\worktrees\qualia-etalon-production-core\qualiaDB\target-production-core\debug\examples\production_core_geometry.exe" --out <new-create-only-dir> --target-faces 0
```

The previous build of this example failed LowerHex formatting. Do not treat
source presence as a passing run.

### 2. Etalon `production_core_probe`

A native DX12 baseline already exists under `../forge-native-baseline/`. That
receipt is not copied here. Optional decoded-asset execution still needs a real
geometry output. GPU timestamp samples are not byte-identical across runs.

```powershell
$env:CARGO_TARGET_DIR = 'C:\Users\Hp\.codex\worktrees\qualia-etalon-production-core\etalon\target-production-core'
cargo build --locked --offline -p etalon-wgsl-forge --example production_core_probe --features gpu-native-dx12
$env:ETALON_WGPU_BACKEND = 'dx12'
& "C:\Users\Hp\.codex\worktrees\qualia-etalon-production-core\etalon\target-production-core\debug\examples\production_core_probe.exe" <new-create-only-dir> NVIDIA
```

Asset join, only after a real `simplified.10d` exists:

```powershell
& "C:\Users\Hp\.codex\worktrees\qualia-etalon-production-core\etalon\target-production-core\debug\examples\production_core_probe.exe" <new-create-only-dir> NVIDIA --vertices-json <decoded-vertices.json> --asset-sha256 <simplified.10d-sha256>
```

### 3. Existing producer GLB smoke / joined tracer

Historical packaging smoke (six vertices, two triangles; not admitted-geometry
qualification) is retained at `../attributed-handoff-smoke/`. Cached producer:

`C:\github\synth\target-prod\debug\synth.exe`

Inspect without GPU or cargo:

```powershell
python C:\Users\Hp\.codex\worktrees\qualia-etalon-production-core\synth\scripts\production_core\admitted_geometry_probe.py --out <new-create-only-dir> --inspect-only
python C:\Users\Hp\.codex\worktrees\qualia-etalon-production-core\synth\scripts\production_core\run_production_core_tracer.py --inspect-only --out <new-create-only-dir>
```

Joined run, only after the geometry executable exists:

```powershell
python C:\Users\Hp\.codex\worktrees\qualia-etalon-production-core\synth\scripts\production_core\run_production_core_tracer.py `
  --geometry-exe C:\Users\Hp\.codex\worktrees\qualia-etalon-production-core\qualiaDB\target-production-core\debug\examples\production_core_geometry.exe `
  --forge-exe C:\Users\Hp\.codex\worktrees\qualia-etalon-production-core\etalon\target-production-core\debug\examples\production_core_probe.exe `
  --synth-exe C:\github\synth\target-prod\debug\synth.exe `
  --out C:\Users\Hp\.codex\worktrees\qualia-etalon-production-core\synth\docs\production-core\evidence\<new-create-only-name>
```

The joined runner still checks three geometry builds and three HMC builds. That
is packaging determinism of one fixed Vibe/input/embedded receipt. It is not
three independent GPU runs with byte-identical timing receipts.
