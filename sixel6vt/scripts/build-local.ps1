# PTY Terminal Local Build Script for Windows
# This script provides a simplified interface for local development builds on Windows

param(
    [switch]$Release,
    [string]$Target = "",
    [switch]$Clean,
    [switch]$Verbose,
    [switch]$Installer,
    [switch]$Sign,
    [switch]$Help
)

# Configuration
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$ProjectDir = Split-Path -Parent $ScriptDir
$BuildDir = Join-Path $ProjectDir "target\local-build"
$DistDir = Join-Path $ProjectDir "dist"

# Default values
$BuildType = if ($Release) { "release" } else { "debug" }

function Show-Usage {
    @"
PTY Terminal Local Build Script for Windows

Usage: .\build-local.ps1 [OPTIONS]

OPTIONS:
    -Release            Build in release mode
    -Target <target>    Specify target triple (e.g., x86_64-pc-windows-msvc)
    -Clean              Clean build directory before building
    -Verbose            Enable verbose output
    -Installer          Create installer after building
    -Sign               Enable code signing (requires setup)
    -Help               Show this help message

EXAMPLES:
    .\build-local.ps1                    # Debug build for current platform
    .\build-local.ps1 -Release           # Release build for current platform  
    .\build-local.ps1 -Release -Installer # Release build with installer
    .\build-local.ps1 -Target aarch64-pc-windows-msvc -Release  # Cross-compile for ARM64

ENVIRONMENT VARIABLES:
    CARGO_TARGET_DIR    Override cargo target directory
    RUST_LOG           Set logging level (debug, info, warn, error)
    SIGNING_CERT_PATH  Path to code signing certificate
    
"@
}

function Write-Info {
    param([string]$Message)
    Write-Host "[INFO] $Message" -ForegroundColor Blue
}

function Write-Success {
    param([string]$Message)
    Write-Host "[SUCCESS] $Message" -ForegroundColor Green
}

function Write-Warning {
    param([string]$Message)
    Write-Host "[WARNING] $Message" -ForegroundColor Yellow
}

function Write-Error {
    param([string]$Message)
    Write-Host "[ERROR] $Message" -ForegroundColor Red
}

function Get-CurrentPlatform {
    $arch = $env:PROCESSOR_ARCHITECTURE
    switch ($arch) {
        "AMD64" { return "windows-x86_64" }
        "ARM64" { return "windows-aarch64" }
        default { return "windows-unknown" }
    }
}

function Test-Dependencies {
    Write-Info "Checking dependencies..."
    
    # Check Rust/Cargo
    if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
        Write-Error "Cargo not found. Please install Rust: https://rustup.rs/"
        exit 1
    }
    
    # Check target if specified
    if ($Target) {
        $installedTargets = rustup target list --installed
        if ($installedTargets -notcontains $Target) {
            Write-Info "Installing target: $Target"
            rustup target add $Target
        }
    }
    
    # Check for Visual Studio Build Tools
    $vsWhere = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe"
    if (Test-Path $vsWhere) {
        $vsInstalls = & $vsWhere -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -format json | ConvertFrom-Json
        if (-not $vsInstalls) {
            Write-Warning "Visual Studio Build Tools not found. Please install Visual Studio or Build Tools."
        }
    }
    
    # Check for WiX if installer is requested
    if ($Installer) {
        if (-not (Get-Command candle.exe -ErrorAction SilentlyContinue)) {
            Write-Warning "WiX Toolset not found. Installer creation will be skipped."
            Write-Warning "Install WiX from: https://github.com/wixtoolset/wix3/releases"
        }
    }
}

function Initialize-BuildEnvironment {
    Write-Info "Setting up build environment..."
    
    # Create directories
    if (-not (Test-Path $BuildDir)) {
        New-Item -ItemType Directory -Path $BuildDir -Force | Out-Null
    }
    if (-not (Test-Path $DistDir)) {
        New-Item -ItemType Directory -Path $DistDir -Force | Out-Null
    }
    
    # Set environment variables
    if (-not $env:CARGO_TARGET_DIR) {
        $env:CARGO_TARGET_DIR = $BuildDir
    }
    
    if ($Verbose) {
        if (-not $env:RUST_LOG) {
            $env:RUST_LOG = "debug"
        }
        $env:RUST_BACKTRACE = "1"
    }
    
    if ($Sign) {
        $env:ENABLE_AUTHENTICODE_SIGNING = "1"
    }
}

function Invoke-CleanBuild {
    if ($Clean) {
        Write-Info "Cleaning build directory..."
        if (Test-Path $BuildDir) {
            Remove-Item -Recurse -Force $BuildDir
        }
        if (Test-Path $DistDir) {
            Remove-Item -Recurse -Force $DistDir
        }
        cargo clean
    }
}

function Test-CodeQuality {
    Write-Info "Checking code formatting and linting..."
    
    # Check formatting
    $formatResult = cargo fmt -- --check
    if ($LASTEXITCODE -ne 0) {
        Write-Warning "Code is not properly formatted. Run 'cargo fmt' to fix."
    }
    
    # Run clippy
    $clippyArgs = @("clippy")
    if ($Target) {
        $clippyArgs += @("--target", $Target)
    }
    if ($Release) {
        $clippyArgs += @("--release")
    }
    $clippyArgs += @("--", "-D", "warnings")
    
    & cargo @clippyArgs
    if ($LASTEXITCODE -ne 0) {
        Write-Error "Clippy found issues. Please fix them before building."
        exit 1
    }
}

function Invoke-Tests {
    Write-Info "Running tests..."
    
    $testArgs = @("test")
    if ($Target) {
        $testArgs += @("--target", $Target)
    }
    if ($Release) {
        $testArgs += @("--release")
    }
    if ($Verbose) {
        $testArgs += @("--verbose")
    }
    
    & cargo @testArgs
    if ($LASTEXITCODE -ne 0) {
        Write-Error "Tests failed"
        exit 1
    }
}

function Invoke-Build {
    Write-Info "Building PTY Terminal ($BuildType mode)..."
    
    $buildArgs = @("build")
    if ($Release) {
        $buildArgs += @("--release")
    }
    if ($Target) {
        $buildArgs += @("--target", $Target)
    }
    if ($Verbose) {
        $buildArgs += @("--verbose")
    }
    
    # Use cross for cross-compilation if available
    $buildCmd = "cargo"
    if ($Target) {
        $hostTarget = (rustc -vV | Select-String "host" | ForEach-Object { $_.ToString().Split(' ')[1] })
        if ($Target -ne $hostTarget) {
            if (Get-Command cross -ErrorAction SilentlyContinue) {
                $buildCmd = "cross"
                Write-Info "Using cross for cross-compilation"
            } else {
                Write-Warning "Cross-compilation target specified but 'cross' not found. Using cargo."
            }
        }
    }
    
    & $buildCmd @buildArgs
    if ($LASTEXITCODE -ne 0) {
        Write-Error "Build failed"
        exit 1
    }
    
    Write-Success "Build completed successfully"
}

function Copy-Artifacts {
    Write-Info "Copying build artifacts..."
    
    $targetDir = $BuildDir
    if ($Target) {
        $targetDir = Join-Path $targetDir $Target
    } else {
        $hostTarget = (rustc -vV | Select-String "host" | ForEach-Object { $_.ToString().Split(' ')[1] })
        $targetDir = Join-Path $targetDir $hostTarget
    }
    
    $buildSubdir = if ($Release) { "release" } else { "debug" }
    $binaryPath = Join-Path $targetDir "$buildSubdir\rio-ext-test.exe"
    
    if (-not (Test-Path $binaryPath)) {
        Write-Error "Binary not found at: $binaryPath"
        exit 1
    }
    
    # Copy to dist directory
    $distBinary = Join-Path $DistDir "rio-ext-test.exe"
    Copy-Item $binaryPath $distBinary -Force
    
    Write-Success "Binary copied to: $distBinary"
}

function New-Installer {
    if ($Installer) {
        Write-Info "Creating installer..."
        
        $platform = Get-CurrentPlatform
        $platformArg = "--platform $platform"
        
        if ($platformArg) {
            cargo run --bin build-installers -- $platformArg.Split(' ')
            if ($LASTEXITCODE -eq 0) {
                Write-Success "Installer created successfully"
            } else {
                Write-Error "Installer creation failed"
                exit 1
            }
        } else {
            Write-Warning "Installer creation not supported for platform: $platform"
        }
    }
}

function Show-Summary {
    Write-Info "Build Summary:"
    Write-Host "  Platform: $(Get-CurrentPlatform)"
    Write-Host "  Build Type: $BuildType"
    
    $displayTarget = if ($Target) { $Target } else { 
        (rustc -vV | Select-String "host" | ForEach-Object { $_.ToString().Split(' ')[1] })
    }
    Write-Host "  Target: $displayTarget"
    
    Write-Host "  Signing: $(if ($Sign) { 'enabled' } else { 'disabled' })"
    Write-Host "  Installer: $(if ($Installer) { 'created' } else { 'skipped' })"
    Write-Host "  Binary: $(Join-Path $DistDir 'rio-ext-test.exe')"
    
    if ($Installer) {
        Write-Host "  Installer files:"
        Get-ChildItem -Path $DistDir -Filter "*.msi" -ErrorAction SilentlyContinue | ForEach-Object {
            Write-Host "    $($_.FullName)"
        }
    }
}

function Main {
    if ($Help) {
        Show-Usage
        return
    }
    
    # Change to project directory
    Set-Location $ProjectDir
    
    Write-Info "Starting PTY Terminal build process..."
    Write-Info "Project directory: $ProjectDir"
    
    try {
        Test-Dependencies
        Initialize-BuildEnvironment
        Invoke-CleanBuild
        Test-CodeQuality
        Invoke-Tests
        Invoke-Build
        Copy-Artifacts
        New-Installer
        Show-Summary
        
        Write-Success "Build process completed successfully!"
    }
    catch {
        Write-Error "Build failed: $_"
        exit 1
    }
}

# Handle script interruption
trap {
    Write-Error "Build interrupted"
    exit 130
}

# Run main function
Main