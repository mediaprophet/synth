param(
    [string]$Source, [string]$Morphism, [string]$ArgsJson = '{}',
    [string]$ResultField,
    [ValidateSet('none','mesh','rgba','rgb','audio','profile')][string]$Encode = 'none',
    [string]$Key = 'result.10d', [string]$InputManifest, [string]$Name = 'asset',
    [string]$Synth = $env:SYNTH_BIN, [string]$RunsDir, [switch]$Preview,
    [ValidateSet('preview','production16')][string]$QualityProfile = 'preview'
)
$ErrorActionPreference = 'Stop'
function ConvertTo-VibeLiteral {
    param([AllowNull()]$Value, [int]$Depth = 0)
    if ($Depth -gt 64) { throw 'Argument record exceeds 64 nested levels' }
    if ($null -eq $Value) { return 'null' }
    if ($Value -is [string]) { return (ConvertTo-Json -InputObject $Value -Compress) }
    if ($Value -is [bool]) { return $Value.ToString().ToLowerInvariant() }
    if ($Value -is [pscustomobject]) {
        $fields = foreach ($property in $Value.PSObject.Properties) {
            if ($property.Name -notmatch '^[A-Za-z_][A-Za-z0-9_]*$') { throw "Vibe record key must be an identifier: $($property.Name)" }
            $property.Name + ':' + (ConvertTo-VibeLiteral -Value $property.Value -Depth ($Depth + 1))
        }
        return '{' + ($fields -join ',') + '}'
    }
    if ($Value -is [array]) {
        $items = foreach ($item in $Value) { ConvertTo-VibeLiteral -Value $item -Depth ($Depth + 1) }
        return '[' + ($items -join ',') + ']'
    }
    if ($Value -is [ValueType] -and $Value -isnot [char] -and $Value -isnot [datetime]) {
        $number = [Convert]::ToString($Value, [Globalization.CultureInfo]::InvariantCulture)
        if ($number -notmatch '^-?[0-9]+(\.[0-9]+)?([eE][+-]?[0-9]+)?$') { throw 'Argument number is not finite or representable' }
        return $number
    }
    throw 'Unsupported JSON argument value'
}
function Assert-ScopedInput {
    param([string]$Path, [string]$Root)
    $admittedRoot = (Resolve-Path -LiteralPath $Root).Path.TrimEnd('\','/')
    $resolved = (Resolve-Path -LiteralPath $Path).Path
    if (-not $resolved.StartsWith($admittedRoot + [IO.Path]::DirectorySeparatorChar, [StringComparison]::OrdinalIgnoreCase)) {
        throw 'Input file is outside SYNTH_INPUT_ROOT'
    }
    $item = Get-Item -LiteralPath $resolved -Force
    if ($item.PSIsContainer -or $item.Length -gt 16MB) { throw 'Input must be a regular file within the 16 MiB resource budget' }
    $ancestor = $item
    while ($null -ne $ancestor) {
        if (($ancestor.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) { throw 'Input paths must not traverse links or junctions' }
        if ($ancestor.FullName.TrimEnd('\','/').Equals($admittedRoot, [StringComparison]::OrdinalIgnoreCase)) { return $resolved }
        $ancestor = if ($ancestor -is [IO.DirectoryInfo]) { $ancestor.Parent } else { $ancestor.Directory }
    }
    throw 'Input path does not reach the admitted root'
}
$repoRoot = (Resolve-Path -LiteralPath (Join-Path $PSScriptRoot '..')).Path
if ([string]::IsNullOrWhiteSpace($Synth)) { $Synth = Join-Path $repoRoot 'target-gap-review\debug\synth.exe' }
if (-not (Test-Path -LiteralPath $Synth -PathType Leaf)) { throw "Build the isolated preview binary first: $Synth" }
if ([string]::IsNullOrWhiteSpace($RunsDir)) {
    $RunsDir = if ($env:SYNTH_OUTPUT_ROOT) { $env:SYNTH_OUTPUT_ROOT } else { Join-Path $repoRoot 'production' }
}
if ($env:SYNTH_OUTPUT_ROOT) {
    $admittedOutput = (Resolve-Path -LiteralPath $env:SYNTH_OUTPUT_ROOT).Path.TrimEnd('\','/')
    $requestedOutput = [IO.Path]::GetFullPath($RunsDir).TrimEnd('\','/')
    if (-not $requestedOutput.Equals($admittedOutput, [StringComparison]::OrdinalIgnoreCase)) {
        throw 'RunsDir must equal the configured SYNTH_OUTPUT_ROOT'
    }
}
if ($Name -notmatch '^[A-Za-z0-9_-]{1,80}$') { throw 'Name must contain 1..80 letters, digits, underscores or hyphens' }
if ([bool]$Source -eq [bool]$Morphism) { throw 'Supply exactly one of Source or Morphism' }
New-Item -ItemType Directory -Force -Path $RunsDir | Out-Null
$runName = "$Name-$(Get-Date -Format 'yyyyMMdd-HHmmss-ffffff')-$([guid]::NewGuid().ToString('N').Substring(0,8))"
$runBase = Join-Path (Resolve-Path -LiteralPath $RunsDir).Path $runName
if ($Morphism) {
    if ($Morphism -notmatch '^[A-Za-z_][A-Za-z0-9_.]*$') { throw 'Invalid morphism identifier' }
    if ($ResultField -and $ResultField -notmatch '^[A-Za-z_][A-Za-z0-9_]*(\.[A-Za-z_][A-Za-z0-9_]*)*$') { throw 'Invalid result field path' }
    $parsed = $ArgsJson | ConvertFrom-Json
    if ($null -eq $parsed -or $parsed -isnot [pscustomobject]) { throw 'ArgsJson must be a JSON object' }
    $argumentsLiteral = ConvertTo-VibeLiteral -Value $parsed
    $idLiteral = ConvertTo-Json -InputObject $Morphism -Compress
    $keyLiteral = ConvertTo-Json -InputObject $Key -Compress
    $value = if ($ResultField) { "produced.$ResultField" } else { 'produced' }
    $encoder = switch ($Encode) {
        'mesh' { 'let encoded = capability.invoke("hypermedia.manifold.compile_10d_pbr", {mesh: ' + $value + '});' }
        'rgba' { 'let encoded = capability.invoke("hypermedia.radiance.encode_rgba_png", {image: ' + $value + '});' }
        'rgb' { 'let encoded = capability.invoke("hypermedia.radiance.encode_rgb_png", {image: ' + $value + '});' }
        'audio' { 'let encoded = capability.invoke("hypermedia.acoustics.encode_wav", {audio: ' + $value + '});' }
        'profile' { 'let encoded = capability.invoke("hypermedia.locus.project_svg", {profile: ' + $value + '});' }
        default { 'let encoded = ' + $value + ';' }
    }
    $vibe = "requires [capability(`"capability.invoke`")];`nlet produced = capability.invoke($idLiteral, $argumentsLiteral);`n$encoder`nlet asset = {files:[{key:$keyLiteral,resource:encoded}]};`n"
    $Source = "$runBase.vibe"
    $stream = [IO.File]::Open($Source, [IO.FileMode]::CreateNew, [IO.FileAccess]::Write)
    try { $bytes = [Text.UTF8Encoding]::new($false).GetBytes($vibe); $stream.Write($bytes,0,$bytes.Length) } finally { $stream.Dispose() }
}
$produceArgs = @('produce','--profile','files','--quality-profile',$QualityProfile,'--source',(Resolve-Path -LiteralPath $Source).Path,
    '--out',"$runBase.hmc",'--export-dir',$runBase,'--present',"$runBase.html")
if ($InputManifest) {
    $inputs = Get-Content -LiteralPath $InputManifest -Raw | ConvertFrom-Json
    $inputRoot = if ($env:SYNTH_INPUT_ROOT) { $env:SYNTH_INPUT_ROOT } else { Split-Path -Parent $repoRoot }
    if (@($inputs).Count -gt 512) { throw 'Input count exceeds the 512 resource budget' }
    $totalBytes = 0L
    foreach ($inputBinding in $inputs) {
        if (-not $inputBinding.key -or -not $inputBinding.kind -or -not $inputBinding.path) { throw 'Input bindings need key, kind and path' }
        $inputPath = Assert-ScopedInput -Path $inputBinding.path -Root $inputRoot
        $totalBytes += (Get-Item -LiteralPath $inputPath).Length
        if ($totalBytes -gt 64MB) { throw 'Input bytes exceed the 64 MiB aggregate budget' }
        $produceArgs += @('--input',[string]$inputBinding.key,[string]$inputBinding.kind,$inputPath)
    }
}
& $Synth @produceArgs
if ($LASTEXITCODE -ne 0) { throw 'Production failed; inspect the structured error and operation output contract' }
if ($Preview) {
    $index = 0
    Get-ChildItem -LiteralPath $runBase -File -Recurse -Filter '*.10d' | Sort-Object FullName | ForEach-Object {
        $assetKey = 'outputs/' + [IO.Path]::GetRelativePath($runBase, $_.FullName).Replace('\','/')
        & $Synth preview --asset $_.FullName --out "$runBase-review-$index" --hmc "$runBase.hmc" --asset-key $assetKey
        if ($LASTEXITCODE -ne 0) { throw "Native review failed for $($_.FullName)" }
        $index++
    }
}
[pscustomobject]@{ source=$Source; hmc="$runBase.hmc"; export_dir=$runBase; html="$runBase.html" }
