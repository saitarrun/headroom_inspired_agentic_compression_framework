# Headroom Compression MCP Installer for Windows
# Usage: powershell -ExecutionPolicy Bypass -File install.ps1

param(
    [string]$InstallDir = "$env:LOCALAPPDATA\bin",
    [switch]$Help
)

if ($Help) {
    Write-Host "Headroom Compression MCP Installer for Windows"
    Write-Host ""
    Write-Host "Usage:"
    Write-Host "  powershell -ExecutionPolicy Bypass -File install.ps1 [-InstallDir <path>]"
    Write-Host ""
    Write-Host "Options:"
    Write-Host "  -InstallDir     Installation directory (default: $env:LOCALAPPDATA\bin)"
    Write-Host "  -Help           Show this help message"
    exit 0
}

$ErrorActionPreference = "Stop"

# Configuration
$REPO = "saitarrun/agentic_context_compression_framework"
$BINARY_NAME = "compression-mcp"
$GITHUB_API = "https://api.github.com/repos/$REPO"

# Colors
function Write-Success {
    Write-Host "✓ $args" -ForegroundColor Green
}

function Write-Error-Custom {
    Write-Host "✗ $args" -ForegroundColor Red
}

function Write-Warning-Custom {
    Write-Host "⚠ $args" -ForegroundColor Yellow
}

function Write-Info {
    Write-Host "  $args" -ForegroundColor Cyan
}

# Detect OS and architecture
function Detect-Platform {
    $os = [System.Environment]::OSVersion.Platform

    if ($os -ne [System.PlatformID]::Win32NT) {
        Write-Error-Custom "This script is only for Windows"
        return $null
    }

    # Detect architecture
    if ([System.Environment]::Is64BitProcess) {
        return "x86_64-pc-windows-msvc"
    } else {
        return "unsupported"
    }
}

# Get latest release
function Get-LatestRelease {
    try {
        $response = Invoke-RestMethod -Uri "$GITHUB_API/releases/latest" -ErrorAction Stop
        return $response.tag_name
    } catch {
        return $null
    }
}

# Download binary
function Download-Binary {
    param(
        [string]$Release,
        [string]$Target
    )

    $binaryFile = "$BINARY_NAME-$Target.exe"
    $downloadUrl = "https://github.com/$REPO/releases/download/$Release/$binaryFile"
    $outputPath = "$BINARY_NAME.exe"

    Write-Host "Downloading $BINARY_NAME $Release for $Target..." -ForegroundColor Yellow

    try {
        $ProgressPreference = 'SilentlyContinue'
        Invoke-WebRequest -Uri $downloadUrl -OutFile $outputPath -ErrorAction Stop
        Write-Success "Downloaded"
        return $true
    } catch {
        Write-Error-Custom "Failed to download binary: $_"
        return $false
    }
}

# Verify checksum
function Verify-Checksum {
    param(
        [string]$Release
    )

    Write-Host "Verifying checksum..." -ForegroundColor Yellow

    try {
        $checksumUrl = "https://github.com/$REPO/releases/download/$Release/SHA256SUMS"
        $checksumContent = (Invoke-WebRequest -Uri $checksumUrl -ErrorAction Stop).Content

        # Parse checksum file to find our binary
        $lines = $checksumContent -split "`n"
        $expectedLine = $lines | Where-Object { $_ -match "compression-mcp-.*\.exe" }

        if (-not $expectedLine) {
            Write-Warning-Custom "Checksum for this binary not found, skipping verification"
            return $true
        }

        # Compute hash of downloaded file
        $fileHash = (Get-FileHash -Path "$BINARY_NAME.exe" -Algorithm SHA256).Hash.ToLower()
        $expectedHash = ($expectedLine -split " ")[0].ToLower()

        if ($fileHash -eq $expectedHash) {
            Write-Success "Checksum verified"
            return $true
        } else {
            Write-Error-Custom "Checksum verification failed"
            return $false
        }
    } catch {
        Write-Warning-Custom "Could not verify checksum: $_"
        return $true  # Don't fail on checksum errors
    }
}

# Install binary
function Install-Binary {
    Write-Host "Installing $BINARY_NAME..." -ForegroundColor Yellow

    if (-not (Test-Path $InstallDir)) {
        New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
    }

    $installPath = Join-Path $InstallDir "$BINARY_NAME.exe"
    Move-Item -Path "$BINARY_NAME.exe" -Destination $installPath -Force

    Write-Success "Installed to $installPath"

    # Add to PATH if not already there
    $currentPath = [System.Environment]::GetEnvironmentVariable("PATH", "User")
    if (-not $currentPath.Contains($InstallDir)) {
        Write-Warning-Custom "$InstallDir is not in your PATH"
        Write-Host "To add it, run this in PowerShell:" -ForegroundColor Yellow
        Write-Info "[Environment]::SetEnvironmentVariable('PATH', `$env:PATH + ';$InstallDir', 'User')"
        Write-Host "Then close and reopen PowerShell." -ForegroundColor Yellow
    }
}

# Configure Claude Code
function Configure-ClaudeCode {
    Write-Host ""
    $response = Read-Host "Configure Claude Code settings? (y/n)"

    if ($response -ne "y" -and $response -ne "Y") {
        return
    }

    $configDir = Join-Path $env:USERPROFILE ".claude"
    $configFile = Join-Path $configDir "settings.json"
    $installPath = Join-Path $InstallDir "$BINARY_NAME.exe"

    # Ensure directory exists
    if (-not (Test-Path $configDir)) {
        New-Item -ItemType Directory -Path $configDir -Force | Out-Null
    }

    # Read or create config
    $config = @{}
    if (Test-Path $configFile) {
        $config = Get-Content $configFile | ConvertFrom-Json -AsHashtable
    }

    if (-not $config.Contains("mcpServers")) {
        $config["mcpServers"] = @{}
    }

    # Add MCP server configuration
    $config["mcpServers"]["headroom-compression"] = @{
        "command" = $installPath
        "disabled" = $false
        "alwaysAllow" = @("headroom_compress", "headroom_retrieve", "headroom_stats")
    }

    # Write config
    $config | ConvertTo-Json -Depth 10 | Set-Content $configFile
    Write-Success "Claude Code configured"
}

# Main installation flow
function Main {
    Write-Host ""
    Write-Host "╔════════════════════════════════════════╗" -ForegroundColor Green
    Write-Host "║  Headroom Compression MCP Installer    ║" -ForegroundColor Green
    Write-Host "║           for Windows                  ║" -ForegroundColor Green
    Write-Host "╚════════════════════════════════════════╝" -ForegroundColor Green
    Write-Host ""

    # Detect platform
    Write-Host "Detecting platform..." -ForegroundColor Yellow
    $Target = Detect-Platform
    if (-not $Target -or $Target -eq "unsupported") {
        Write-Error-Custom "Unsupported platform"
        exit 1
    }
    Write-Success "Detected: $Target"
    Write-Host ""

    # Get latest release
    Write-Host "Fetching latest release..." -ForegroundColor Yellow
    $Release = Get-LatestRelease
    if (-not $Release) {
        Write-Error-Custom "Failed to fetch latest release"
        Write-Host "Manual download: https://github.com/$REPO/releases" -ForegroundColor Yellow
        exit 1
    }
    Write-Success "Latest release: $Release"
    Write-Host ""

    # Create temp directory
    $tempDir = New-Item -ItemType Directory -Path (Join-Path $env:TEMP "mcp-install-$(Get-Random)") -Force
    Push-Location $tempDir

    try {
        # Download
        if (-not (Download-Binary $Release $Target)) {
            exit 1
        }

        # Verify
        if (-not (Verify-Checksum $Release)) {
            exit 1
        }

        # Install
        Install-Binary
        Write-Host ""

        # Configure Claude Code
        Configure-ClaudeCode
        Write-Host ""

        Write-Success "Installation complete!" -ForegroundColor Green
        Write-Host ""
        Write-Host "Next steps:" -ForegroundColor Yellow
        Write-Host "1. Restart Claude Code for MCP changes to take effect"
        Write-Host "2. Verify installation by running: $BINARY_NAME"
        Write-Host ""
        Write-Host "For more info: https://github.com/$REPO" -ForegroundColor Cyan
    } finally {
        Pop-Location
        Remove-Item -Path $tempDir -Recurse -Force -ErrorAction SilentlyContinue
    }
}

Main
