# SweetMCP One-Line Installer for Windows - DOES IT ALL!
# Usage: iex (iwr -UseBasicParsing https://get.cyrup.ai/sweetmcp.ps1).Content

param([switch]$DryRun = $false)

# Strict mode
Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

# Colors
function Write-Blue($msg) { Write-Host $msg -ForegroundColor Blue }
function Write-Red($msg) { Write-Host $msg -ForegroundColor Red }
function Write-Yellow($msg) { Write-Host $msg -ForegroundColor Yellow }
function Write-Green($msg) { Write-Host $msg -ForegroundColor Green }

# Admin check
$currentUser = [Security.Principal.WindowsIdentity]::GetCurrent()
$principal = New-Object Security.Principal.WindowsPrincipal($currentUser)
if (-not $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) {
    Write-Red "This installer requires administrator privileges."
    Write-Red "Please run PowerShell as Administrator."
    exit 1
}

Write-Blue "🍯 SweetMCP Installer - We Do It All!"

# Get config directory
$ConfigHome = if ($env:XDG_CONFIG_HOME) { $env:XDG_CONFIG_HOME } else { "$env:APPDATA" }
$SweetMCPHome = "$ConfigHome\sweetmcp"

# Install missing tools via winget or direct download
function Install-Git {
    Write-Yellow "Installing Git..."
    
    # Try winget first (available on Windows 10 1709+ and Windows 11)
    if (Get-Command winget -ErrorAction SilentlyContinue) {
        winget install --id Git.Git -e --silent --accept-package-agreements --accept-source-agreements
    } else {
        # Direct download fallback
        $gitInstaller = "$env:TEMP\git-installer.exe"
        Write-Yellow "Downloading Git installer..."
        Invoke-WebRequest -Uri "https://github.com/git-for-windows/git/releases/download/v2.43.0.windows.1/Git-2.43.0-64-bit.exe" -OutFile $gitInstaller
        Start-Process -FilePath $gitInstaller -ArgumentList "/VERYSILENT", "/NORESTART" -Wait
        Remove-Item $gitInstaller -Force
    }
    
    # Refresh PATH
    $env:PATH = [System.Environment]::GetEnvironmentVariable("PATH", "Machine") + ";" + [System.Environment]::GetEnvironmentVariable("PATH", "User")
}

function Install-BuildTools {
    Write-Yellow "Installing Visual Studio Build Tools..."
    
    # Try winget first
    if (Get-Command winget -ErrorAction SilentlyContinue) {
        winget install --id Microsoft.VisualStudio.2022.BuildTools -e --silent --accept-package-agreements --accept-source-agreements
    } else {
        # Direct download
        $vsInstaller = "$env:TEMP\vs_buildtools.exe"
        Write-Yellow "Downloading Visual Studio Build Tools..."
        Invoke-WebRequest -Uri "https://aka.ms/vs/17/release/vs_buildtools.exe" -OutFile $vsInstaller
        
        # Install with C++ workload
        Start-Process -FilePath $vsInstaller -ArgumentList `
            "--quiet", "--wait", "--norestart", `
            "--add", "Microsoft.VisualStudio.Workload.VCTools", `
            "--add", "Microsoft.VisualStudio.Component.Windows10SDK.19041" `
            -Wait
        Remove-Item $vsInstaller -Force
    }
}

function Install-Curl {
    # curl is built into Windows 10+ but let's ensure it's available
    if (-not (Get-Command curl -ErrorAction SilentlyContinue)) {
        Write-Yellow "Installing curl..."
        
        if (Get-Command winget -ErrorAction SilentlyContinue) {
            winget install --id cURL.cURL -e --silent --accept-package-agreements --accept-source-agreements
        } else {
            # curl should be in System32 on Windows 10+
            if (-not (Test-Path "$env:SystemRoot\System32\curl.exe")) {
                Write-Red "curl not found and cannot auto-install on this Windows version"
                Write-Red "Please upgrade to Windows 10 or later"
                exit 1
            }
        }
    }
}

# Install dependencies
Write-Blue "Installing dependencies..."

# Git
if (-not (Get-Command git -ErrorAction SilentlyContinue)) {
    Install-Git
}

# Build tools (check for cl.exe compiler)
$vsWhere = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe"
if (-not (Test-Path $vsWhere) -and -not (Get-Command cl -ErrorAction SilentlyContinue)) {
    Install-BuildTools
}

# Curl
Install-Curl

Write-Green "Dependencies installed!"

# Create SweetMCP home directory
Write-Blue "Creating SweetMCP home directory..."
New-Item -ItemType Directory -Path $SweetMCPHome -Force | Out-Null
Set-Location $SweetMCPHome

# Install Rust if needed
if (-not (Get-Command rustc -ErrorAction SilentlyContinue)) {
    Write-Yellow "Installing Rust..."
    $rustupUrl = "https://win.rustup.rs/x86_64"
    $rustupPath = "$env:TEMP\rustup-init.exe"
    
    Invoke-WebRequest -Uri $rustupUrl -OutFile $rustupPath
    & $rustupPath -y --default-toolchain stable --profile default
    
    # Add to PATH
    $env:PATH = "$env:USERPROFILE\.cargo\bin;$env:PATH"
    Remove-Item $rustupPath -Force
}

# Clone repository (remove old clone if exists)
Write-Blue "Cloning SweetMCP..."
if (Test-Path "sweetmcp") {
    Remove-Item -Path "sweetmcp" -Recurse -Force
}

try {
    git clone --depth 1 https://github.com/cyrup-ai/sweetmcp.git 2>$null
} catch {
    Write-Red "Failed to clone repository"
    exit 1
}
Set-Location sweetmcp

# Build
Write-Blue "Building SweetMCP (this may take a few minutes)..."
$buildProcess = Start-Process -FilePath "cargo" `
    -ArgumentList @("build", "--release", "--package", "sweetmcp-daemon") `
    -Wait -PassThru -NoNewWindow

if ($buildProcess.ExitCode -ne 0) {
    Write-Red "Build failed"
    exit 1
}

# Install
Write-Blue "Installing SweetMCP daemon..."
$installProcess = Start-Process -FilePath ".\target\release\sweetmcp-daemon.exe" `
    -ArgumentList "install" `
    -Wait -PassThru -NoNewWindow

if ($installProcess.ExitCode -ne 0) {
    Write-Red "Installation failed"
    exit 1
}

# Success!
Write-Green @"

╔══════════════════════════════════════════════════════════════╗
║           SweetMCP Installation Completed! 🍯                ║
╚══════════════════════════════════════════════════════════════╝

Installed in: $SweetMCPHome

Available at:
  • https://sweetmcp.cyrup.dev:8443
  • https://sweetmcp.cyrup.ai:8443
  • https://sweetmcp.cyrup.cloud:8443
  • https://sweetmcp.cyrup.pro:8443

Go have fun!
"@