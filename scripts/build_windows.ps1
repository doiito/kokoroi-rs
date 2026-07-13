# Build script for Kokoro TTS on Windows
$ErrorActionPreference = "Stop"
$ProjectDir = Split-Path -Parent $PSScriptRoot

Write-Host "Building Kokoro TTS for Windows..." -ForegroundColor Green
Set-Location $ProjectDir

cargo build --release -p koko -p kokoros-server
Write-Host "Build complete!" -ForegroundColor Green
Get-ChildItem target/release/koko.exe, target/release/kokoros-server.exe
