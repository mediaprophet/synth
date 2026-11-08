param([string]$OutputDirectory, [switch]$SkipBuild)
$ErrorActionPreference = 'Stop'
$PSNativeCommandUseErrorActionPreference = $false
$root = Split-Path $PSScriptRoot -Parent
if (-not $OutputDirectory) { $OutputDirectory = Join-Path $root ('target/production-tests/general-' + [guid]::NewGuid().ToString('N')) }
$OutputDirectory = [IO.Path]::GetFullPath($OutputDirectory)
if (Test-Path $OutputDirectory) { throw 'Choose a new output directory; existing evidence is preserved.' }
& (Join-Path $PSScriptRoot 'prepare-dependencies.ps1') -Offline
if (-not $SkipBuild) {
    & cargo test --manifest-path (Join-Path $root 'Cargo.toml') --locked -p synth --lib --tests -j 1
    if ($LASTEXITCODE -ne 0) { throw 'Synth tests/build failed' }
}
New-Item -ItemType Directory -Path $OutputDirectory | Out-Null
$binary = Join-Path $root 'target/debug/synth.exe'
function Invoke-Synth([string]$Stage, [string[]]$Arguments) {
    & $binary @Arguments 1> (Join-Path $OutputDirectory "$Stage.stdout.log") 2> (Join-Path $OutputDirectory "$Stage.stderr.log")
    if ($LASTEXITCODE -ne 0) { throw "$Stage failed; see $OutputDirectory" }
    Write-Output "$Stage passed"
}
try {
    Copy-Item -LiteralPath (Join-Path $root 'provenance/current-dependencies.json') -Destination (Join-Path $OutputDirectory 'dependencies.json')
    $source = Join-Path $OutputDirectory 'assets.vibe'
    Copy-Item -LiteralPath (Join-Path $root 'examples/assets.vibe') -Destination $source
    $candidate = Join-Path $OutputDirectory 'candidate.hmc'
    $reopened = Join-Path $OutputDirectory 'reopened.hmc'
    $repeated = Join-Path $OutputDirectory 'independent.hmc'
    $export = Join-Path $OutputDirectory 'exports'
    Invoke-Synth 'capabilities' @('capabilities')
    Invoke-Synth 'profiles' @('profiles')
    Invoke-Synth 'produce' @('produce', '--source', $source, '--out', $candidate, '--export-dir', $export)
    Invoke-Synth 'reopen' @('produce', '--reopen', $candidate, '--out', $reopened)
    Invoke-Synth 'repeat' @('produce', '--source', $source, '--out', $repeated)
    $hash = (Get-FileHash $candidate).Hash
    if ($hash -ne (Get-FileHash $reopened).Hash -or $hash -ne (Get-FileHash $repeated).Hash) { throw 'Reopen or independent production changed package bytes' }
    $svg = [xml](Get-Content (Join-Path $export 'icon.svg') -Raw)
    if ($svg.DocumentElement.LocalName -ne 'svg') { throw 'SVG was not produced' }
    $palette = Get-Content (Join-Path $export 'palette.json') -Raw | ConvertFrom-Json
    if ($palette.foreground -ne '#8ce6ba') { throw 'Palette differs' }
    $signal = [IO.File]::ReadAllBytes((Join-Path $export 'signal.bin'))
    if (($signal -join ',') -ne '0,31,127,255') { throw 'Etalon-derived binary output differs' }
    $inputSource = Join-Path $OutputDirectory 'input.vibe'
    [IO.File]::WriteAllText($inputSource, 'let asset = { files: [{ key: "retained.svg", kind: "svg", input: "reference.svg" }] };')
    Invoke-Synth 'retain-input' @('produce', '--source', $inputSource, '--input', 'reference.svg', 'svg', (Join-Path $export 'icon.svg'), '--out', (Join-Path $OutputDirectory 'retained.hmc'), '--export-dir', (Join-Path $OutputDirectory 'retained-exports'))
    if ((Get-FileHash (Join-Path $export 'icon.svg')).Hash -ne (Get-FileHash (Join-Path $OutputDirectory 'retained-exports/retained.svg')).Hash) { throw 'Retained input changed' }
    & $binary produce --reopen $candidate --out $candidate 1> (Join-Path $OutputDirectory 'overwrite.stdout.log') 2> (Join-Path $OutputDirectory 'overwrite.stderr.log')
    if ($LASTEXITCODE -eq 0 -or (Get-FileHash $candidate).Hash -ne $hash) { throw 'Create-only output protection failed' }
    $damaged = Join-Path $OutputDirectory 'truncated.hmc'
    $bytes = [IO.File]::ReadAllBytes($candidate)
    [IO.File]::WriteAllBytes($damaged, $bytes[0..127])
    $rejected = Join-Path $OutputDirectory 'must-not-exist.hmc'
    & $binary produce --reopen $damaged --out $rejected 1> (Join-Path $OutputDirectory 'corrupt.stdout.log') 2> (Join-Path $OutputDirectory 'corrupt.stderr.log')
    if ($LASTEXITCODE -eq 0 -or (Test-Path $rejected)) { throw 'Damaged package was not rejected' }
    $result = [ordered]@{
        passed = $true; scope = 'General-purpose production CLI: SVG, JSON, Etalon-derived bytes and retained resources';
        candidate_sha256 = $hash.ToLowerInvariant(); binary_sha256 = (Get-FileHash $binary).Hash.ToLowerInvariant();
        cargo_lock_sha256 = (Get-FileHash (Join-Path $root 'Cargo.lock')).Hash.ToLowerInvariant();
        output_files = 3; exact_fresh_reopen = $true; independent_generation_identical = $true;
        retained_input_identical = $true; overwrite_rejected = $true; truncated_package_rejected = $true
    }
    $result | ConvertTo-Json | Set-Content (Join-Path $OutputDirectory 'result.json')
    Write-Output "General production workflow passed: $OutputDirectory"
} catch {
    @{passed = $false; error = $_.Exception.Message} | ConvertTo-Json | Set-Content (Join-Path $OutputDirectory 'result.json')
    throw
}