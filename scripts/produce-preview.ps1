param(
    [Parameter(Mandatory = $true)]
    [string]$Source,
    [string]$Name = (Get-Date -Format "yyyyMMdd-HHmmss")
)

$ErrorActionPreference = "Stop"
$synth = "C:\github\synth\target-studio\debug\synth.exe"
if (-not (Test-Path $synth)) {
    throw "Build the preview CLI first: `$env:CARGO_TARGET_DIR='C:\github\synth\target-studio'; cargo build --bin synth --features preview"
}
if (-not (Test-Path $Source)) {
    throw "Missing vibe: $Source"
}

$runs = "C:\github\synth\target-studio\runs"
New-Item -ItemType Directory -Force -Path $runs | Out-Null
$export = Join-Path $runs $Name
$hmc = Join-Path $runs "$Name.hmc"
$html = Join-Path $runs "$Name.html"
if ((Test-Path $export) -or (Test-Path $hmc) -or (Test-Path $html)) {
    throw "Run name already exists: $Name"
}

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
