# SweetMCP One-Line Installer for Windows
# Usage: iex (iwr -UseBasicParsing https://get.cyrup.ai/sweetmcp.ps1).Content

param(
    [switch]$DryRun = $false
)

# Enable strict mode
Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

# Color output functions
function Write-Info($Message) {
    Write-Host "[INFO] $Message" -ForegroundColor Blue
}

function Write-Warn($Message) {
    Write-Host "[WARN] $Message" -ForegroundColor Yellow
}

function Write-Error($Message) {
    Write-Host "[ERROR] $Message" -ForegroundColor Red
}

function Write-Success($Message) {
    Write-Host "[SUCCESS] $Message" -ForegroundColor Green
}

# Cleanup function
function Cleanup {
    if ($global:TempDir -and (Test-Path $global:TempDir)) {
        Write-Info "Cleaning up temporary directory: $global:TempDir"
        Remove-Item -Path $global:TempDir -Recurse -Force -ErrorAction SilentlyContinue
    }
}

# Platform detection
function Detect-Platform {
    $arch = if ([Environment]::Is64BitOperatingSystem) { "x86_64" } else { "i686" }
    $global:Platform = "$arch-pc-windows-msvc"
    Write-Info "Detected platform: $global:Platform"
}

# Check system requirements
function Check-Requirements {
    Write-Info "Checking system requirements..."
    
    # Check for required commands
    $requiredCommands = @("git", "curl")
    foreach ($cmd in $requiredCommands) {
        if (-not (Get-Command $cmd -ErrorAction SilentlyContinue)) {
            Write-Error "Required command not found: $cmd"
            switch ($cmd) {
                "git" { Write-Info "Install git: https://git-scm.com/downloads" }
                "curl" { Write-Info "curl should be available on Windows 10+ or install from: https://curl.se/windows/" }
            }
            exit 1
        }
    }
    
    # Check for admin privileges
    $currentUser = [Security.Principal.WindowsIdentity]::GetCurrent()
    $principal = New-Object Security.Principal.WindowsPrincipal($currentUser)
    if (-not $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) {
        Write-Error "This script requires administrator privileges. Please run as Administrator."
        exit 1
    }
    
    # Check for Rust toolchain
    if (-not (Get-Command rustc -ErrorAction SilentlyContinue)) {
        Write-Warn "Rust toolchain not found. Installing..."
        Install-Rust
    } else {
        Write-Info "Rust toolchain found: $(rustc --version)"
    }
    
    Write-Success "System requirements satisfied"
}

# Install Rust toolchain
function Install-Rust {
    Write-Info "Installing Rust toolchain..."
    
    # Download and run rustup-init
    $rustupUrl = "https://win.rustup.rs/x86_64"
    $rustupPath = "$env:TEMP\rustup-init.exe"
    
    Invoke-WebRequest -Uri $rustupUrl -OutFile $rustupPath
    & $rustupPath -y --default-toolchain stable
    
    # Add cargo to PATH for current session
    $cargoPath = "$env:USERPROFILE\.cargo\bin"
    $env:PATH = "$cargoPath;$env:PATH"
    
    # Verify installation
    if (Get-Command rustc -ErrorAction SilentlyContinue) {
        Write-Success "Rust installed: $(rustc --version)"
    } else {
        Write-Error "Failed to install Rust toolchain"
        exit 1
    }
}

# Clone the repository
function Clone-Repository {
    Write-Info "Cloning SweetMCP repository..."
    
    $global:TempDir = Join-Path $env:TEMP "sweetmcp-$(Get-Random)"
    New-Item -ItemType Directory -Path $global:TempDir | Out-Null
    Set-Location $global:TempDir
    
    # Try SSH first, fallback to HTTPS
    try {
        git clone git@github.com:cyrup-ai/sweetmcp.git 2>$null
        Write-Info "Cloned via SSH"
    } catch {
        try {
            git clone https://github.com/cyrup-ai/sweetmcp.git 2>$null
            Write-Info "Cloned via HTTPS"
        } catch {
            Write-Error "Failed to clone repository"
            exit 1
        }
    }
    
    Set-Location sweetmcp
    Write-Success "Repository cloned successfully"
}

# Build the project
function Build-Project {
    Write-Info "Building SweetMCP..."
    
    # Build in release mode for performance
    $buildResult = cargo build --release --package sweetmcp-daemon
    if ($LASTEXITCODE -eq 0) {
        Write-Success "Build completed successfully"
    } else {
        Write-Error "Build failed"
        exit 1
    }
}

# Install the daemon
function Install-Daemon {
    Write-Info "Installing SweetMCP daemon..."
    
    # Run the daemon installer
    $installResult = & ".\target\release\sweetmcp-daemon.exe" install
    if ($LASTEXITCODE -eq 0) {
        Write-Success "SweetMCP daemon installed successfully"
    } else {
        Write-Error "Daemon installation failed"
        exit 1
    }
}

# Verify installation
function Verify-Installation {
    Write-Info "Verifying installation..."
    
    # Check if daemon is installed
    if (Get-Command cyrupd -ErrorAction SilentlyContinue) {
        Write-Success "Daemon binary installed: $(Get-Command cyrupd | Select-Object -ExpandProperty Source)"
    } else {
        Write-Warn "Daemon binary not found in PATH"
    }
    
    # Check certificate
    $certPath = "$env:APPDATA\sweetmcp\wildcard.cyrup.pem"
    if (Test-Path $certPath) {
        Write-Success "Wildcard certificate installed: $certPath"
    } else {
        Write-Warn "Wildcard certificate not found at: $certPath"
    }
    
    # Test host entries
    $testDomains = @("sweetmcp.cyrup.dev", "sweetmcp.cyrup.ai", "sweetmcp.cyrup.cloud", "sweetmcp.cyrup.pro")
    $hostsWorking = $true
    
    foreach ($domain in $testDomains) {
        try {
            $result = Test-NetConnection -ComputerName $domain -Port 80 -WarningAction SilentlyContinue
            if ($result.TcpTestSucceeded -or $result.RemoteAddress -eq "127.0.0.1") {
                Write-Success "Host entry working: $domain"
            } else {
                Write-Warn "Host entry not working: $domain"
                $hostsWorking = $false
            }
        } catch {
            Write-Warn "Could not test host entry: $domain"
            $hostsWorking = $false
        }
    }
    
    if ($hostsWorking) {
        Write-Success "All host entries are working"
    } else {
        Write-Warn "Some host entries may need manual verification"
    }
}

# Main installation function
function Main {
    Write-Info "Starting SweetMCP installation..."
    Write-Info "============================================"
    
    try {
        Detect-Platform
        Check-Requirements
        Clone-Repository
        Build-Project
        Install-Daemon
        Verify-Installation
        
        Write-Info "============================================"
        Write-Success "SweetMCP installation completed!"
        Write-Info ""
        Write-Info "Next steps:"
        Write-Info "  1. Start the service: sc start cyrupd"
        Write-Info "  2. Enable auto-start: sc config cyrupd start= auto"
        Write-Info "  3. Check status: sc query cyrupd"
        Write-Info "  4. View logs: Get-WinEvent -LogName Application | Where-Object {$_.ProviderName -eq 'cyrupd'}"
        Write-Info ""
        Write-Info "Configuration:"
        Write-Info "  - Config file: $env:APPDATA\cyrupd\cyrupd.toml"
        Write-Info "  - Certificate: $env:APPDATA\sweetmcp\wildcard.cyrup.pem"
        Write-Info "  - Host entries: C:\Windows\System32\drivers\etc\hosts"
        Write-Info ""
        Write-Info "Domains available:"
        Write-Info "  - https://sweetmcp.cyrup.dev:8443"
        Write-Info "  - https://sweetmcp.cyrup.ai:8443"
        Write-Info "  - https://sweetmcp.cyrup.cloud:8443"
        Write-Info "  - https://sweetmcp.cyrup.pro:8443"
        Write-Info ""
        Write-Success "Welcome to SweetMCP! 🍯"
    } finally {
        Cleanup
    }
}

# Run main function
Main