# Spotlight for Windows - Code Signing Script (Section 42 & Phase 17)
param (
    [Parameter(Mandatory=$true)]
    [string]$FilePath,
    [string]$CertThumbprint = $env:SIGNING_CERT_THUMBPRINT
)

$ErrorActionPreference = "Stop"

if (-not (Test-Path $FilePath)) {
    Write-Error "File not found: $FilePath"
}

Write-Host "Signing artifact: $FilePath" -ForegroundColor Cyan

if ($CertThumbprint) {
    signtool.exe sign /sha1 $CertThumbprint /tr "http://timestamp.digicert.com" /td sha256 /fd sha256 $FilePath
    signtool.exe verify /pa $FilePath
    Write-Host "Code signing verified successfully." -ForegroundColor Green
} else {
    Write-Warning "SIGNING_CERT_THUMBPRINT not set. Skipping real signature (test build mode)."
}
