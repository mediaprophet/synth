param([switch]$Offline)
$ErrorActionPreference = 'Stop'
$root = Split-Path $PSScriptRoot -Parent
$pins = Get-Content (Join-Path $root 'provenance/current-dependencies.json') -Raw | ConvertFrom-Json
foreach ($pin in $pins.repositories | Where-Object { $_.path -like '../*' }) {
    $repository = [IO.Path]::GetFullPath((Join-Path $root $pin.path)).Replace('\', '/')
    $actualPin = & git -c "safe.directory=$repository" -C $repository rev-parse HEAD
    if ($LASTEXITCODE -ne 0 -or $actualPin -ne $pin.commit) {
        throw "$($pin.name) must match recorded revision $($pin.commit); found $actualPin"
    }
}
Write-Output 'Recorded SDK/runtime dependencies verified. No project-specific source is required.'