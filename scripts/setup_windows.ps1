# EmuForge - Windows Environment Checker

Write-Host "🔧 EmuForge Dependency Checker" -ForegroundColor Cyan

# Check for Rust
if (Get-Command cargo -ErrorAction SilentlyContinue) {
    Write-Host "✅ Rust (cargo) is installed." -ForegroundColor Green
} else {
    Write-Host "❌ Rust is missing. Please install it from https://rustup.rs" -ForegroundColor Red
}

# Check for Node.js
if (Get-Command npm -ErrorAction SilentlyContinue) {
    Write-Host "✅ Node.js (npm) is installed." -ForegroundColor Green
} else {
    Write-Host "❌ Node.js is missing. Please install it from https://nodejs.org" -ForegroundColor Red
}

# Check for WebView2
$webviewKey = "HKLM:\SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}"
$webviewKeySystem = "HKLM:\SOFTWARE\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}"

if ((Test-Path $webviewKey) -or (Test-Path $webviewKeySystem)) {
    Write-Host "✅ WebView2 Runtime is detected." -ForegroundColor Green
} else {
    Write-Host "⚠️ WebView2 Runtime might be missing." -ForegroundColor Yellow
    Write-Host "If the app fails to launch, install the 'Evergreen Bootstrapper' from Microsoft."
}

# Install Node dependencies
if (Test-Path "ui") {
    Write-Host "Installing UI dependencies..." -ForegroundColor Cyan
    npm install --prefix ui
}

Write-Host "`nSetup Check Complete. You can now run: npm run tauri dev --prefix ui"

