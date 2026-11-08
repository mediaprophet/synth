param([string]$OutputDirectory, [switch]$SkipBuild)
$ErrorActionPreference = 'Stop'
$PSNativeCommandUseErrorActionPreference = $false
$root = Split-Path $PSScriptRoot -Parent
if (-not $OutputDirectory) { $OutputDirectory = Join-Path $root ('target/production-tests/assets-' + [guid]::NewGuid().ToString('N')) }
$OutputDirectory = [IO.Path]::GetFullPath($OutputDirectory)
if (Test-Path -LiteralPath $OutputDirectory) { throw 'Choose a new output directory; existing evidence is preserved.' }
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
    $source = Join-Path $OutputDirectory 'production.vibe'
    Copy-Item -LiteralPath (Join-Path $root 'examples/production.vibe') -Destination $source
    $candidate = Join-Path $OutputDirectory 'candidate.hmc'
    $reopened = Join-Path $OutputDirectory 'reopened.hmc'
    $repeated = Join-Path $OutputDirectory 'independent.hmc'
    $export = Join-Path $OutputDirectory 'exports'
    Invoke-Synth 'operations' @('operations')
    Invoke-Synth 'capabilities' @('capabilities')
    Invoke-Synth 'produce' @('produce', '--source', $source, '--out', $candidate, '--export-dir', $export)
    Invoke-Synth 'reopen' @('produce', '--reopen', $candidate, '--out', $reopened)
    Invoke-Synth 'repeat' @('produce', '--source', $source, '--out', $repeated)
    $hash = (Get-FileHash -LiteralPath $candidate).Hash
    if ($hash -ne (Get-FileHash -LiteralPath $reopened).Hash -or $hash -ne (Get-FileHash -LiteralPath $repeated).Hash) { throw 'Package bytes changed on replay or independent generation' }
    $svg = [xml](Get-Content -LiteralPath (Join-Path $export 'profile.svg') -Raw)
    if ($svg.DocumentElement.LocalName -ne 'svg' -or -not $svg.svg.polygon) { throw 'Missing generated SVG geometry' }
    $inputSource = Join-Path $OutputDirectory 'input.vibe'
    [IO.File]::WriteAllText($inputSource, @'
requires [capability("capability.invoke")];
let image = capability.invoke("synth.image.import_png", {input:"reference.png"});
let output = capability.invoke("synth.image.png", {image:image});
let asset = {files:[{key:"roundtrip.png",resource:output}]};
'@)
    $retained = Join-Path $OutputDirectory 'retained.hmc'
    Invoke-Synth 'import-png' @('produce','--source',$inputSource,'--input','reference.png','png',(Join-Path $export 'frames/0000.png'),'--out',$retained,'--export-dir',(Join-Path $OutputDirectory 'import-export'))
    Invoke-Synth 'reopen-import' @('produce','--reopen',$retained,'--out',(Join-Path $OutputDirectory 'retained-reopened.hmc'))
    if ((Get-FileHash -LiteralPath (Join-Path $export 'frames/0000.png')).Hash -ne (Get-FileHash -LiteralPath (Join-Path $OutputDirectory 'import-export/roundtrip.png')).Hash) { throw 'PNG encode/decode roundtrip differs' }
    $files = @(Get-ChildItem -LiteralPath $export -Recurse -File | Sort-Object FullName | ForEach-Object {
        [ordered]@{key = [IO.Path]::GetRelativePath($export,$_.FullName).Replace('\','/'); bytes=$_.Length; sha256=(Get-FileHash -LiteralPath $_.FullName).Hash.ToLowerInvariant()}
    })
    if ($files.Count -ne 9) { throw 'Expected nine generated assets' }
    $result = [ordered]@{
        passed=$true; scope='Shared-library VibeScript production: SVG, meshes, raster frames/effects/mask, PCM audio, retained PNG';
        candidate_sha256=$hash.ToLowerInvariant(); binary_sha256=(Get-FileHash -LiteralPath $binary).Hash.ToLowerInvariant();
        cargo_lock_sha256=(Get-FileHash -LiteralPath (Join-Path $root 'Cargo.lock')).Hash.ToLowerInvariant();
        exact_fresh_reopen=$true; independent_generation_identical=$true; png_import_replay=$true; outputs=$files
    }
    $result | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $OutputDirectory 'result.json')
    Write-Output "Asset workflow passed: $OutputDirectory"
} catch {
    @{passed=$false;error=$_.Exception.Message} | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $OutputDirectory 'result.json')
    throw
}
