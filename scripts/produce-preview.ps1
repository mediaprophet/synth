param(
    [Parameter(Mandatory = $true)]
    [string]$Source,
    [string]$Name
)

$ErrorActionPreference = "Stop"

function New-UniqueRunSuffix {
    $stamp = Get-Date -Format "yyyyMMdd-HHmmss-ffffff"
    $rand = [guid]::NewGuid().ToString("N").Substring(0, 8)
    return "$stamp-$PID-$rand"
}

function Get-UniqueRunName {
    param([string]$Prefix)
    $suffix = New-UniqueRunSuffix
    if ([string]::IsNullOrWhiteSpace($Prefix)) {
        return $suffix
    }
    return "$Prefix-$suffix"
}

function Test-RunNameExists {
    param([string]$RunsDir, [string]$RunName)
    $exportPath = Join-Path $RunsDir $RunName
    $hmcPath = Join-Path $RunsDir "$RunName.hmc"
    $htmlPath = Join-Path $RunsDir "$RunName.html"
    return (Test-Path $exportPath) -or (Test-Path $hmcPath) -or (Test-Path $htmlPath)
}

$synthProd = "C:\github\synth\target-prod\debug\synth.exe"
$synthStudio = "C:\github\synth\target-studio\debug\synth.exe"
if (Test-Path -LiteralPath $synthProd) {
    $synth = $synthProd
} elseif (Test-Path -LiteralPath $synthStudio) {
    $synth = $synthStudio
} else {
    throw "Build the preview CLI first: `$env:CARGO_TARGET_DIR='C:\github\synth\target-prod'; cargo build --bin synth --features preview"
}
if (-not (Test-Path $Source)) {
    throw "Missing vibe: $Source"
}

$runs = Join-Path (Split-Path (Split-Path $synth)) "runs"
New-Item -ItemType Directory -Force -Path $runs | Out-Null
$prefix = $Name
$Name = Get-UniqueRunName -Prefix $prefix
if (Test-RunNameExists -RunsDir $runs -RunName $Name) {
    $Name = Get-UniqueRunName -Prefix $prefix
    if (Test-RunNameExists -RunsDir $runs -RunName $Name) {
        throw "Run name already exists: $Name"
    }
}
$export = Join-Path $runs $Name
$hmc = Join-Path $runs "$Name.hmc"
$html = Join-Path $runs "$Name.html"

& $synth produce --profile files --source $Source --out $hmc --export-dir $export --present $html
if ($LASTEXITCODE -ne 0) { throw "produce failed" }

Get-ChildItem $export -Filter *.10d -Recurse | ForEach-Object {
    $preview = Join-Path $runs "$Name-preview-$($_.BaseName)"
    & $synth preview --asset $_.FullName --out $preview
    if ($LASTEXITCODE -ne 0) { throw "preview failed for $($_.Name)" }
}

Write-Host "HMC $hmc"
Write-Host "Export $export"
Write-Host "HTML $html"
Write-Host "Review the PNG files under $runs before calling the shape done."
