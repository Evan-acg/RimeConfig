param(
    [switch]$NoBuild,
    [string]$Target = "C:\Tools"
)

$ProjectRoot = Split-Path -Parent $MyInvocation.MyCommand.Path

if (-not $NoBuild) {
    Write-Host "Building release..." -ForegroundColor Cyan
    Push-Location $ProjectRoot
    cargo build --release
    Pop-Location
    if ($LASTEXITCODE -ne 0) {
        Write-Error "Build failed"
        exit 1
    }
}

$Source = Join-Path $ProjectRoot "target\release\rime-addword.exe"
$Dest = Join-Path $Target "rime-addword.exe"

if (-not (Test-Path $Source)) {
    Write-Error "Not found: $Source"
    exit 1
}

if (-not (Test-Path $Target)) {
    New-Item -ItemType Directory -Path $Target -Force | Out-Null
}

Copy-Item -Path $Source -Destination $Dest -Force
Write-Host "Deployed to $Dest" -ForegroundColor Green

$userPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($userPath -notlike "*$Target*") {
    $choice = Read-Host "C:\Tools is not in PATH. Add it? (y/n)"
    if ($choice -eq 'y') {
        [Environment]::SetEnvironmentVariable("Path", "$userPath;$Target", "User")
        Write-Host "Added to PATH. Restart terminal to apply." -ForegroundColor Yellow
    }
}
