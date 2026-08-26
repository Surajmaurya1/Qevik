# Spotlight for Windows - Release Packaging Script (Section 43 & Phase 19)
param (
    [Parameter(Mandatory=$true)]
    [string]$Version
)

$ErrorActionPreference = "Stop"

Write-Host "Packaging release v$Version..." -ForegroundColor Cyan
./scripts/build.ps1 -Release
Write-Host "Calculating SHA-256 checksums..." -ForegroundColor Yellow

$artifacts = Get-ChildItem -Path "src-tauri/target/release/bundle/msi/*.msi", "src-tauri/target/release/bundle/nsis/*.exe" -ErrorAction SilentlyContinue

foreach ($file in $artifacts) {
    $hash = Get-FileHash -Path $file.FullName -Algorithm SHA256
    Write-Host "$($file.Name): $($hash.Hash)" -ForegroundColor Green
    "$($hash.Hash)  $($file.Name)" | Out-File -FilePath "$($file.FullName).sha256" -Encoding utf8
}

Write-Host "Release v$Version artifacts ready." -ForegroundColor Green
