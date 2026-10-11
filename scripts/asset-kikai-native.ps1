param(
    [string]$Python = 'python',
    [string]$OutputDirectory = 'work/kikai-native',
    [switch]$SkipDependencies
)
$ErrorActionPreference = 'Stop'
$kikaiRepo = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
Push-Location $kikaiRepo
try {
    $kikaiOutput = [System.IO.Path]::GetFullPath((Join-Path $kikaiRepo $OutputDirectory))
    New-Item -ItemType Directory -Path $kikaiOutput -Force | Out-Null
    if (!$SkipDependencies) {
        cargo build --locked -p game_web
        if ($LASTEXITCODE -ne 0) { throw 'Dependency build failed' }
    }
    $kikaiDeps = Join-Path $kikaiRepo 'target/debug/deps'
    $kikaiExternals = @()
    foreach ($kikaiName in @('engine_core', 'serde', 'serde_json')) {
        $kikaiLibrary = Get-ChildItem -LiteralPath $kikaiDeps -Filter "lib$kikaiName-*.rlib" |
            Sort-Object LastWriteTimeUtc -Descending | Select-Object -First 1
        if (!$kikaiLibrary) { throw "Missing dependency $kikaiName" }
        $kikaiExternals += @('--extern', "$kikaiName=$($kikaiLibrary.FullName)")
    }
    $kikaiCommon = @('--edition=2024', '--crate-name', 'kikai_art_review',
        'crates/game_web/src/kikai_art.rs', '-L', "dependency=$kikaiDeps") + $kikaiExternals
    $kikaiTestExe = Join-Path $kikaiOutput 'native-tests.exe'
    rustc @kikaiCommon --test -o $kikaiTestExe
    if ($LASTEXITCODE -ne 0) { throw 'Native test compile failed' }
    & $kikaiTestExe --nocapture | Tee-Object -FilePath (Join-Path $kikaiOutput 'native-tests.log')
    if ($LASTEXITCODE -ne 0) { throw 'Native tests failed' }
    $kikaiArtLibrary = Join-Path $kikaiOutput 'libkikai_art_review.rlib'
    clippy-driver @kikaiCommon --crate-type rlib -D warnings -o $kikaiArtLibrary
    if ($LASTEXITCODE -ne 0) { throw 'Native library Clippy failed' }
    $kikaiProofExe = Join-Path $kikaiOutput 'native-proof.exe'
    rustc --edition=2024 scripts/asset-kikai-proof.rs --extern "kikai_art_review=$kikaiArtLibrary" `
        @kikaiExternals -L "dependency=$kikaiDeps" -o $kikaiProofExe
    if ($LASTEXITCODE -ne 0) { throw 'Native proof compile failed' }
    & $kikaiProofExe (Join-Path $kikaiOutput 'native-packets.json') |
        Tee-Object -FilePath (Join-Path $kikaiOutput 'native-proof.log')
    if ($LASTEXITCODE -ne 0) { throw 'Native proof failed' }
    & $Python scripts/asset-kikai-render.py $kikaiOutput
    if ($LASTEXITCODE -ne 0) { throw 'Native packet preview failed' }
    Write-Output "Native proof saved to $kikaiOutput; posed software review only"
} finally { Pop-Location }
