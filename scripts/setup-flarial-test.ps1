param(
    [switch]$Build
)

$ErrorActionPreference = "Stop"

$launcher = Split-Path -Parent $PSScriptRoot
$projects = Split-Path -Parent $launcher
$oss = Join-Path $projects "Flarial.Client.Oss"
$clientDir = Join-Path $env:LOCALAPPDATA "Lumina\Client"
$destination = Join-Path $clientDir "Flarial.Client.dll"

if (-not (Test-Path $oss)) {
    throw "Flarial.Client.Oss não encontrado em $oss"
}

if ($Build) {
    Write-Host "Compilando Flarial.Client.Oss (Release)..."
    Push-Location $oss
    try {
        cmd /c "build.bat R < NUL"
    }
    finally {
        Pop-Location
    }
}

$candidates = @(
    (Join-Path $oss "cmake-build-release-ninja\Flarial.dll"),
    (Join-Path $oss "cmake-build-debug-ninja\Flarial.dll")
)

$source = $candidates | Where-Object { Test-Path $_ } | Select-Object -First 1

if (-not $source) {
    throw "Flarial.dll não encontrada. Rode com -Build ou compile manualmente (build.bat R)."
}

New-Item -ItemType Directory -Force -Path $clientDir | Out-Null
Copy-Item -Force $source $destination

Write-Host "DLL copiada:"
Write-Host "  origem:  $source"
Write-Host "  destino: $destination"
