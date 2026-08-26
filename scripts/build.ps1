# Spotlight for Windows - Production Build Script
param (
    [switch]$Release = $true
)

$ErrorActionPreference = "Stop"
Write-Host "==========================================" -ForegroundColor Cyan
Write-Host "Building Spotlight for Windows (Production)" -ForegroundColor Cyan
Write-Host "==========================================" -ForegroundColor Cyan

# 1. Typecheck and lint
Write-Host "`n[1/4] Running frontend typecheck and lint..." -ForegroundColor Yellow
npm run typecheck
npm run lint

# 2. Build React production bundle
Write-Host "`n[2/4] Building Vite production bundle..." -ForegroundColor Yellow
npm run build

# 3. Build Tauri binary
Write-Host "`n[3/4] Building Tauri Windows application..." -ForegroundColor Yellow
if ($Release) {
    npm run tauri build
} else {
    npm run tauri build -- --debug
}

Write-Host "`n[4/4] Build complete!" -ForegroundColor Green
