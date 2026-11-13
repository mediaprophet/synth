# Etalon local GPU test prerequisites - 13 November 2026

Read Etalon AGENTS.md, docs/testing.md, dev-docs/TODO_INDEX.md, applicable xtask source and CI installation action before editing. No nested AGENTS or function index exists in the inspected Etalon paths. Root owns all Cargo/build/test jobs; this worker ran no Cargo command.

## Source change

Only `C:/github/etalon/xtask/src/test.rs` changed executable code: WARP output now begins at `CARGO_TARGET_DIR` when supplied, then appends the existing coverage/profile directories. Absent the variable, original target/debug and target/llvm-cov-target/debug behavior remains. Pure regression covers isolated debug, release, custom coverage profile and defaults. No backend/filter/SDK requirement change. CHANGELOG.md records the developer-visible correction. This repair specifically follows the environment variable; use the commands below rather than a forwarded --target-dir flag, which the separate GPU discovery phase does not receive.

## Prepared dependencies

All archives retained at `C:/github/synth/target-etalon-gpu-prerequisites`. No global install/configuration changed. Restricted DNS lookup failed; approved elevated downloads from official GitHub/NuGet succeeded. Exact package identity/version checked from NuGet manifests before staging. The nextest archive hash matches the official immutable release asset digest. Windows Authenticode verification returned Valid for nextest (SignPath Foundation), WARP, D3D12Core and d3d12SDKLayers (Microsoft Corporation).

| Dependency | Archive SHA256 | Staged path |
|---|---|---|
| cargo-nextest 0.9.131 Windows x64 | 06cf3ac1f2ccf8af955e3efd0adece4b3b0b85fe51c57234731a98a228808288 | C:/github/synth/target-etalon-gpu-prerequisites/nextest-0.9.131/bin/cargo-nextest.exe |
| Microsoft.Direct3D.WARP 1.0.20 | e5fe5de661ce98b58ef9cfb736e73c0a7a2623d3bbf5f14839b2d55566d87e40 | C:/github/synth/target-etalon-gpu-review/debug/d3d10warp.dll and deps/d3d10warp.dll, plus warp.txt |
| Microsoft.Direct3D.D3D12 1.619.0 | d5edab9a0c4d1b78ba6fe55b425eeef9fefba7d2a101e889d70fd21d481e6cb1 | C:/github/etalon/target/agility-sdk/build/native/bin/x64, plus version.txt |

Evidence files: setup-receipt.json (every staged file hash), signature-evidence.json, nextest-version.txt and nextest-release-0.9.131.json. The local offline prepare_offline.py refuses replacing different pre-existing artifacts. Only runner --version executed: cargo-nextest 0.9.131 (af40d2e87 2026-03-18), host x86_64-pc-windows-msvc. No test/Cargo/provider process launched.

Sources: [official nextest 0.9.131 release](https://github.com/nextest-rs/nextest/releases/tag/cargo-nextest-0.9.131), [documented release URLs](https://nexte.st/docs/installation/release-urls/), [official WARP NuGet archive](https://api.nuget.org/v3-flatcontainer/microsoft.direct3d.warp/1.0.20/microsoft.direct3d.warp.1.0.20.nupkg), [official Agility NuGet archive](https://api.nuget.org/v3-flatcontainer/microsoft.direct3d.d3d12/1.619.0/microsoft.direct3d.d3d12.1.619.0.nupkg). Stable pinned nextest 0.9.131 was selected rather than a floating latest dependency.

VulkanSDK1.4.363.0 is already installed and Bin is on PATH; dxc.exe and dxcompiler.dll exist. System vulkaninfo.exe/curl.exe/tar.exe exist. dxil.dll was absent in that Bin; actual test execution must determine whether any selected shader backend needs it. No claim that every Cargo dependency or GPU driver is qualified yet.

## Root-owned commands

Use process-scoped environment only, working directory Etalon:

```powershell
$env:PATH = 'C:/github/synth/target-etalon-gpu-prerequisites/nextest-0.9.131/bin;' + $env:PATH
$env:CARGO_TARGET_DIR = 'C:/github/synth/target-etalon-gpu-review'
$env:CARGO_NET_OFFLINE = 'true'
$env:CARGO_BUILD_JOBS = '2'
Set-Location C:/github/etalon
cargo test -p etalon-xtask warp_follows_isolated_cargo_target_and_profile
cargo test -p etalon-render --lib shadow_math::receiver::tests
cargo xtask test --test-threads 2
```

Full xtask suite generates real .gpuconfig using etalon-info, then nextest --benches --tests --all-features over workspace default members. No package/backend filter is added. Repository nextest policy excludes OOM tests by default; fail-fast=false, retries=0. WARP and Agility receipts now satisfy the existing installer checks, preventing their HTTP downloads for this exact target/profile. Cargo offline refuses uncached Cargo dependencies rather than downloading. Agility remains an explicit repo-local SDK cache; test build/WARP artifacts use the isolated Synth target. This is native suite coverage, not WebGPU CTS acceptance.

Rust source frozen after the routing repair; root must execute and review targeted/full results. Actual native repaired-shadow image acceptance and final installed SDK transport still require a fresh frozen Synth candidate.

## Full-feature static DXC prerequisite (13 November final run)

The all-features link additionally needs Microsoft ATL atls.lib; it was absent from the installed MSVC tree. Staged official Microsoft.VC.14.44.17.14.ATL.X64.base package14.44.35208 from the installed Visual Studio catalog, verified SHA2566dd0b7f8079c654db2b8fdaebc74900e8418b5204914a77bde93a5b01ac0998c. No system installation/settings change. Package/catalog payload and extracted library hashes: target-etalon-gpu-prerequisites/atl-14.44.35208/{manifest.json,verification.json}.

Before the full suite, add only to its process environment:

```powershell
$env:LIB = 'C:/github/synth/target-etalon-gpu-prerequisites/atl-14.44.35208/lib;' + $env:LIB
```

The first full all-feature compile failed on missing atls.lib in static DXC, not a renderer test assertion. Preserve tests-final-etalon-full-j2.log; rerun evidence tests-final-etalon-atl.log.
