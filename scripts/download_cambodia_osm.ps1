# PowerShell helper script to download Cambodia OSM data
$ErrorActionPreference = "Stop"

$DataDir = Join-Path $PSScriptRoot "..\data"
if (-not (Test-Path $DataDir)) {
    New-Item -ItemType Directory -Path $DataDir | Out-Null
}

$TargetFile = Join-Path $DataDir "cambodia-latest.osm.pbf"
$DownloadUrl = "https://download.geofabrik.de/asia/cambodia-latest.osm.pbf"

Write-Host "Downloading Cambodia OSM extract from Geofabrik..." -ForegroundColor Cyan
Write-Host "Source: $DownloadUrl"
Write-Host "Target: $TargetFile"

Invoke-WebRequest -Uri $DownloadUrl -OutFile $TargetFile -UseBasicParsing

Write-Host "Download completed successfully! File size: $([math]::Round((Get-Item $TargetFile).Length / 1MB, 2)) MB" -ForegroundColor Green
Write-Host "You can now run: cargo run --release -- --data data/cambodia-latest.osm.pbf" -ForegroundColor Yellow
