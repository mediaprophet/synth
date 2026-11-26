# Shared typed adapter with named forwarding to the common implementation.
param(
    [string]$Source, [string]$Morphism, [string]$ArgsJson = '{}', [string]$ResultField,
    [ValidateSet('none','mesh','rgba','rgb','audio','profile')][string]$Encode = 'none',
    [string]$Key = 'result.10d', [string]$InputManifest, [string]$Name = 'asset',
    [string]$Synth = $env:SYNTH_BIN, [string]$RunsDir, [switch]$Preview,
    [ValidateSet('preview','production16')][string]$QualityProfile = 'production16'
)
& (Join-Path $PSScriptRoot '..\..\..\scripts\invoke-asset.ps1') @PSBoundParameters
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
