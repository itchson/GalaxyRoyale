param([string]$OutputDirectory = 'work/haven-topdown-repro')
$ErrorActionPreference = 'Stop'
$havenWorkspace = (Resolve-Path (Join-Path $PSScriptRoot '../..')).Path
Push-Location $havenWorkspace
try {
    cargo build --locked -p game_core
    if ($LASTEXITCODE -ne 0) { throw "Native build failed: $LASTEXITCODE" }
    $havenDeps = Join-Path $havenWorkspace 'target/debug/deps'
    $havenCore = Get-ChildItem -LiteralPath $havenDeps -Filter 'libgame_core-*.rlib' | Sort-Object LastWriteTimeUtc -Descending | Select-Object -First 1
    $havenEngine = Get-ChildItem -LiteralPath $havenDeps -Filter 'libengine_core-*.rlib' | Sort-Object LastWriteTimeUtc -Descending | Select-Object -First 1
    $havenJson = Get-ChildItem -LiteralPath $havenDeps -Filter 'libserde_json-*.rlib' | Sort-Object LastWriteTimeUtc -Descending | Select-Object -First 1
    if (!$havenCore -or !$havenEngine -or !$havenJson) { throw 'Expected production dependency libraries are absent' }
    $havenOutput = Join-Path $havenWorkspace $OutputDirectory
    New-Item -ItemType Directory -Force -Path $havenOutput | Out-Null
    $havenExe = Join-Path $havenOutput 'proof.exe'
    rustc --edition=2024 (Join-Path $PSScriptRoot 'proof.rs') --extern "game_core=$($havenCore.FullName)" --extern "engine_core=$($havenEngine.FullName)" --extern "serde_json=$($havenJson.FullName)" -L "dependency=$havenDeps" -o $havenExe
    if ($LASTEXITCODE -ne 0) { throw "Proof compile failed: $LASTEXITCODE" }
    & $havenExe (Join-Path $havenOutput 'native-extraction.json')
    if ($LASTEXITCODE -ne 0) { throw "Proof execution failed: $LASTEXITCODE" }
} finally {
    Pop-Location
}
