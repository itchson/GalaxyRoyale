param(
    [string]$Python = 'python',
    [string]$OutputDirectory = 'work/shooting-star-native',
    [switch]$SkipDependencies
)
$ErrorActionPreference = 'Stop'
$starRepo = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
Push-Location $starRepo
try {
    $starOutput = [System.IO.Path]::GetFullPath((Join-Path $starRepo $OutputDirectory))
    New-Item -ItemType Directory -Path $starOutput -Force | Out-Null
    if (!$SkipDependencies) {
        cargo build --locked -p game_web
        if ($LASTEXITCODE -ne 0) { throw 'Dependency build failed' }
    }
    $starDeps = Join-Path $starRepo 'target/debug/deps'
    $starExternals = @()
    foreach ($starName in @('engine_core', 'serde', 'serde_json')) {
        $starLibrary = Get-ChildItem -LiteralPath $starDeps -Filter "lib$starName-*.rlib" |
            Sort-Object LastWriteTimeUtc -Descending | Select-Object -First 1
        if (!$starLibrary) { throw "Missing dependency $starName" }
        $starExternals += @('--extern', "$starName=$($starLibrary.FullName)")
    }
    $starCommon = @('--edition=2024', '--crate-name', 'shooting_star_review',
        'crates/game_core/src/shooting_star.rs', '-L', "dependency=$starDeps") + $starExternals
    $starTestExe = Join-Path $starOutput 'native-tests.exe'
    rustc @starCommon --test -o $starTestExe
    if ($LASTEXITCODE -ne 0) { throw 'Native test compile failed' }
    & $starTestExe --nocapture | Tee-Object -FilePath (Join-Path $starOutput 'native-tests.log')
    if ($LASTEXITCODE -ne 0) { throw 'Native tests failed' }
    $starArtLibrary = Join-Path $starOutput 'libshooting_star_review.rlib'
    clippy-driver @starCommon --crate-type rlib -D warnings -o $starArtLibrary
    if ($LASTEXITCODE -ne 0) { throw 'Native library Clippy failed' }
    $starProofExe = Join-Path $starOutput 'native-proof.exe'
    rustc --edition=2024 scripts/asset-shooting-star-proof.rs --extern "shooting_star_review=$starArtLibrary" `
        @starExternals -L "dependency=$starDeps" -o $starProofExe
    if ($LASTEXITCODE -ne 0) { throw 'Native proof compile failed' }
    & $starProofExe (Join-Path $starOutput 'native-packets.json') |
        Tee-Object -FilePath (Join-Path $starOutput 'native-proof.log')
    if ($LASTEXITCODE -ne 0) { throw 'Native proof failed' }
    & $Python scripts/asset-shooting-star-render.py $starOutput
    if ($LASTEXITCODE -ne 0) { throw 'Native packet preview failed' }
    Write-Output "Native proof saved to $starOutput; posed software review only"
} finally { Pop-Location }
