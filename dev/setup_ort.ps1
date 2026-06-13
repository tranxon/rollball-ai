#!/usr/bin/env pwsh
# setup_ort.ps1 - Download and configure ONNX Runtime for Windows
#
# On Windows, `download-ort` feature usually works out of the box.
# This script is for users who want to:
#   - Use a specific ORT version
#   - Avoid re-downloading on every clean build
#   - Use a custom ORT build
#
# Usage:
#   .\dev\setup_ort.ps1                       # Auto-detect and install
#   .\dev\setup_ort.ps1 -Version "1.22.0"     # Specific version
#   .\dev\setup_ort.ps1 -Reinstall            # Force re-download
#
# After running, the env file is generated and can be loaded:
#   . .\.ort_env.ps1

param(
    [string] $Version = "",
    [switch] $Reinstall
)

$ErrorActionPreference = "Stop"
$WorkspaceRoot = Split-Path -Parent $PSScriptRoot

Write-Host "============================================" -ForegroundColor Cyan
Write-Host "  ONNX Runtime Setup for RollBall.AI (Win)" -ForegroundColor Cyan
Write-Host "============================================" -ForegroundColor Cyan
Write-Host ""

# ── Check if download-ort is sufficient ─────────────────────────────────────
if ([string]::IsNullOrEmpty($Version) -and -not $Reinstall) {
    Write-Host "  On Windows, the download-ort feature usually works out of the box." -ForegroundColor Gray
    Write-Host "  You can simply run:" -ForegroundColor Gray
    Write-Host "    cargo build --release -p rollball-embed --features download-ort" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "  This script is for custom ORT installations." -ForegroundColor Gray
    Write-Host ""
    $proceed = Read-Host "  Continue with manual setup? [y/N]"
    if ($proceed -ne "y" -and $proceed -ne "Y") {
        Write-Host "  Aborted." -ForegroundColor Yellow
        exit 0
    }
    Write-Host ""
}

# ── Defaults ────────────────────────────────────────────────────────────────
$OrtVersion = if ([string]::IsNullOrEmpty($Version)) { "1.22.0" } else { $Version }
$OrtPlatform = "win"
$OrtArch = "x64"
$OrtLibName = "onnxruntime.dll"
$OrtLibStatic = "onnxruntime.lib"

# ── Paths ───────────────────────────────────────────────────────────────────
$OrtInstallDir = Join-Path $WorkspaceRoot ".ort"
$OrtArchiveName = "onnxruntime-${OrtPlatform}-${OrtArch}-${OrtVersion}"
$OrtExtractedDir = Join-Path $OrtInstallDir $OrtArchiveName
$OrtUrl = "https://github.com/microsoft/onnxruntime/releases/download/v${OrtVersion}/${OrtArchiveName}.zip"

Write-Host "  ORT Version : $OrtVersion" -ForegroundColor Cyan
Write-Host "  Platform    : $OrtPlatform ($OrtArch)" -ForegroundColor Cyan
Write-Host "  Install Dir : $OrtExtractedDir" -ForegroundColor Cyan
Write-Host ""

# ── Check if already installed ──────────────────────────────────────────────
$OrtLibPath = Join-Path $OrtExtractedDir "lib\$OrtLibStatic"
if ((Test-Path $OrtLibPath) -and -not $Reinstall) {
    Write-Host "  ORT $OrtVersion already installed." -ForegroundColor Green
    Write-Host "  Use -Reinstall to force re-download." -ForegroundColor Gray
} else {
    # ── Download ─────────────────────────────────────────────────────────────
    Write-Host "[1/4] Downloading ONNX Runtime $OrtVersion..." -ForegroundColor Yellow
    Write-Host "  URL: $OrtUrl" -ForegroundColor Gray

    $tempZip = Join-Path $env:TEMP "${OrtArchiveName}.zip"
    try {
        Invoke-WebRequest -Uri $OrtUrl -OutFile $tempZip -UseBasicParsing
    } catch {
        Write-Host "  Download failed: $_" -ForegroundColor Red
        exit 1
    }
    Write-Host "  Download complete." -ForegroundColor Green
    Write-Host ""

    # ── Extract ──────────────────────────────────────────────────────────────
    Write-Host "[2/4] Extracting..." -ForegroundColor Yellow
    if (-not (Test-Path $OrtInstallDir)) {
        New-Item -ItemType Directory -Path $OrtInstallDir -Force | Out-Null
    }
    if ($Reinstall -and (Test-Path $OrtExtractedDir)) {
        Remove-Item -Recurse -Force $OrtExtractedDir
    }

    Expand-Archive -Path $tempZip -DestinationPath $OrtInstallDir -Force
    Remove-Item -Path $tempZip -Force -ErrorAction SilentlyContinue

    if (-not (Test-Path $OrtExtractedDir)) {
        Write-Host "  Extraction failed. Expected: $OrtExtractedDir" -ForegroundColor Red
        exit 1
    }
    Write-Host "  Extracted to: $OrtExtractedDir" -ForegroundColor Green
    Write-Host ""

    # ── Verify ───────────────────────────────────────────────────────────────
    Write-Host "[3/4] Verifying installation..." -ForegroundColor Yellow
    $libDir = Join-Path $OrtExtractedDir "lib"
    if (Test-Path (Join-Path $libDir $OrtLibName)) {
        Write-Host "  DLL: $OrtLibName" -ForegroundColor Green
    }
    if (Test-Path (Join-Path $libDir $OrtLibStatic)) {
        Write-Host "  Lib: $OrtLibStatic" -ForegroundColor Green
    }
    Write-Host ""

    # ── Clean cache ──────────────────────────────────────────────────────────
    Write-Host "[4/4] Cleaning cached ORT downloads..." -ForegroundColor Yellow
    $ortCache = Join-Path $env:USERPROFILE ".cache\ort.pyke.io"
    if (Test-Path $ortCache) {
        Remove-Item -Recurse -Force $ortCache
        Write-Host "  Removed $ortCache" -ForegroundColor Green
    } else {
        Write-Host "  No cache to clean." -ForegroundColor Gray
    }
    Write-Host ""
}

# ── Set environment variables ───────────────────────────────────────────────
$OrtLibLocation = Join-Path $OrtExtractedDir "lib"
$OrtDylibPath = Join-Path $OrtLibLocation $OrtLibName
$env:ORT_LIB_LOCATION = $OrtLibLocation
$env:ORT_DYLIB_PATH = $OrtDylibPath

# ── Generate env file (PowerShell) ──────────────────────────────────────────
$envFilePs = Join-Path $WorkspaceRoot ".ort_env.ps1"
@"
# ONNX Runtime environment variables (PowerShell)
# Generated by dev/setup_ort.ps1 on $(Get-Date -Format "o")
# Load with: . .\.ort_env.ps1
#
# ORT version: $OrtVersion

`$env:ORT_LIB_LOCATION = "$OrtLibLocation"
`$env:ORT_DYLIB_PATH = "$OrtDylibPath"
"@ | Set-Content -Path $envFilePs -Encoding UTF8

# ── Generate env file (cmd/batch) ───────────────────────────────────────────
$envFileBat = Join-Path $WorkspaceRoot ".ort_env.bat"
@"
@echo off
REM ONNX Runtime environment variables
REM Generated by dev/setup_ort.ps1
REM Usage: call .ort_env.bat

set ORT_LIB_LOCATION=$OrtLibLocation
set ORT_DYLIB_PATH=$OrtDylibPath
"@ | Set-Content -Path $envFileBat -Encoding ASCII

# ── Summary ─────────────────────────────────────────────────────────────────
Write-Host "============================================" -ForegroundColor Cyan
Write-Host "  Setup Complete" -ForegroundColor Cyan
Write-Host "============================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "  ORT Version : $OrtVersion" -ForegroundColor White
Write-Host "  Install Dir : $OrtExtractedDir" -ForegroundColor White
Write-Host "  Library     : $OrtLibLocation" -ForegroundColor White
Write-Host "  Env Files   : .ort_env.ps1 / .ort_env.bat" -ForegroundColor Cyan
Write-Host ""
Write-Host "Next steps:" -ForegroundColor Yellow
Write-Host ""
Write-Host "  Option A - Build with env vars (already set in this session):" -ForegroundColor White
Write-Host "    cd core" -ForegroundColor Cyan
Write-Host "    cargo build --release -p rollball-embed" -ForegroundColor Cyan
Write-Host ""
Write-Host "  Option B - Load env in new terminal:" -ForegroundColor White
Write-Host "    . .\.ort_env.ps1" -ForegroundColor Cyan
Write-Host "    cd core; cargo build --release -p rollball-embed" -ForegroundColor Cyan
Write-Host ""
Write-Host "  Option C - Use build_core.ps1:" -ForegroundColor White
Write-Host "    . .\.ort_env.ps1; .\dev\build_core.ps1" -ForegroundColor Cyan
Write-Host ""
