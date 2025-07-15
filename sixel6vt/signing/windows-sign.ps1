# Windows Code Signing Script for rio-ext-test
param(
    [string]$CertificatePath = $env:WINDOWS_CERT_PATH,
    [string]$CertificatePassword = $env:WINDOWS_CERT_PASSWORD,
    [string]$TimestampUrl = "http://timestamp.digicert.com",
    [switch]$SkipSigning = $false
)

# Configuration
$AppName = "rio-ext-test"
$BinaryPath = "target\release\$AppName.exe"
$SignedBinaryPath = "target\release\$AppName-signed.exe"
$InstallerPath = "target\release\$AppName-installer.msi"

# Colors for output
$Green = "Green"
$Yellow = "Yellow"
$Red = "Red"

function Write-Info {
    param($Message)
    Write-Host "[INFO] $Message" -ForegroundColor $Green
}

function Write-Warn {
    param($Message)
    Write-Host "[WARN] $Message" -ForegroundColor $Yellow
}

function Write-Error {
    param($Message)
    Write-Host "[ERROR] $Message" -ForegroundColor $Red
}

function Test-Prerequisites {
    Write-Info "Checking prerequisites..."
    
    if (!(Test-Path $BinaryPath)) {
        Write-Error "Binary not found at $BinaryPath"
        Write-Error "Please run 'cargo build --release' first"
        return $false
    }
    
    if ($SkipSigning) {
        Write-Warn "Skipping code signing as requested"
        return $false
    }
    
    if ([string]::IsNullOrEmpty($CertificatePath)) {
        Write-Warn "WINDOWS_CERT_PATH not set, skipping code signing"
        return $false
    }
    
    if (!(Test-Path $CertificatePath)) {
        Write-Error "Certificate not found at $CertificatePath"
        return $false
    }
    
    # Check for signtool
    $signtool = Get-Command signtool.exe -ErrorAction SilentlyContinue
    if (!$signtool) {
        Write-Error "signtool.exe not found. Please install Windows SDK"
        return $false
    }
    
    return $true
}

function Invoke-BinarySigning {
    Write-Info "Signing binary with certificate: $CertificatePath"
    
    # Copy binary for signing
    Copy-Item $BinaryPath $SignedBinaryPath -Force
    
    # Prepare signtool arguments
    $signtoolArgs = @(
        "sign"
        "/f", $CertificatePath
        "/tr", $TimestampUrl
        "/td", "sha256"
        "/fd", "sha256"
        "/v"
    )
    
    # Add password if provided
    if (![string]::IsNullOrEmpty($CertificatePassword)) {
        $signtoolArgs += "/p", $CertificatePassword
    }
    
    $signtoolArgs += $SignedBinaryPath
    
    # Sign the binary
    $result = & signtool.exe $signtoolArgs
    if ($LASTEXITCODE -ne 0) {
        Write-Error "Failed to sign binary: $result"
        return $false
    }
    
    # Verify the signature
    $verifyResult = & signtool.exe verify /pa /v $SignedBinaryPath
    if ($LASTEXITCODE -ne 0) {
        Write-Error "Failed to verify signature: $verifyResult"
        return $false
    }
    
    Write-Info "Binary signed successfully"
    return $true
}

function New-WindowsInstaller {
    Write-Info "Creating Windows installer..."
    
    # Check for WiX toolset
    $candle = Get-Command candle.exe -ErrorAction SilentlyContinue
    $light = Get-Command light.exe -ErrorAction SilentlyContinue
    
    if (!$candle -or !$light) {
        Write-Warn "WiX toolset not found, creating simple ZIP package instead"
        New-ZipPackage
        return
    }
    
    # Create WiX source file
    $wixSource = @"
<?xml version="1.0" encoding="UTF-8"?>
<Wix xmlns="http://schemas.microsoft.com/wix/2006/wi">
    <Product Id="*" Name="$AppName" Language="1033" Version="1.0.0.0" Manufacturer="SweetMCP" UpgradeCode="12345678-1234-1234-1234-123456789012">
        <Package InstallerVersion="200" Compressed="yes" InstallScope="perMachine" />
        <MajorUpgrade DowngradeErrorMessage="A newer version of [$AppName] is already installed." />
        <MediaTemplate EmbedCab="yes" />
        
        <Feature Id="ProductFeature" Title="$AppName" Level="1">
            <ComponentGroupRef Id="ProductComponents" />
        </Feature>
    </Product>
    
    <Fragment>
        <Directory Id="TARGETDIR" Name="SourceDir">
            <Directory Id="ProgramFilesFolder">
                <Directory Id="INSTALLFOLDER" Name="$AppName" />
            </Directory>
        </Directory>
    </Fragment>
    
    <Fragment>
        <ComponentGroup Id="ProductComponents" Directory="INSTALLFOLDER">
            <Component Id="MainExecutable">
                <File Id="MainExe" Source="$SignedBinaryPath" />
            </Component>
        </ComponentGroup>
    </Fragment>
</Wix>
"@
    
    $wixFile = "target\release\$AppName.wxs"
    $wixSource | Out-File -FilePath $wixFile -Encoding utf8
    
    # Compile and link installer
    & candle.exe -out "target\release\$AppName.wixobj" $wixFile
    & light.exe -out $InstallerPath "target\release\$AppName.wixobj"
    
    if (Test-Path $InstallerPath) {
        Write-Info "Installer created at $InstallerPath"
        
        # Sign the installer if we have signing capability
        if (Test-Prerequisites) {
            Write-Info "Signing installer..."
            $signtoolArgs = @(
                "sign", "/f", $CertificatePath, "/tr", $TimestampUrl,
                "/td", "sha256", "/fd", "sha256", "/v"
            )
            if (![string]::IsNullOrEmpty($CertificatePassword)) {
                $signtoolArgs += "/p", $CertificatePassword
            }
            $signtoolArgs += $InstallerPath
            & signtool.exe $signtoolArgs
        }
    } else {
        Write-Error "Failed to create installer"
        New-ZipPackage
    }
}

function New-ZipPackage {
    Write-Info "Creating ZIP package..."
    
    $zipPath = "target\release\$AppName.zip"
    $sourceFile = if (Test-Path $SignedBinaryPath) { $SignedBinaryPath } else { $BinaryPath }
    
    Compress-Archive -Path $sourceFile -DestinationPath $zipPath -Force
    Write-Info "ZIP package created at $zipPath"
}

function Main {
    Write-Info "Starting Windows signing process for $AppName"
    
    if (Test-Prerequisites) {
        if (Invoke-BinarySigning) {
            New-WindowsInstaller
            Write-Info "Windows signing process completed successfully"
        } else {
            Write-Error "Binary signing failed"
            exit 1
        }
    } else {
        Write-Warn "Skipping signing due to missing prerequisites"
        # Create unsigned package for development
        if (Test-Path $BinaryPath) {
            Copy-Item $BinaryPath $SignedBinaryPath -Force
            New-ZipPackage
            Write-Info "Created unsigned package for development"
        }
    }
}

# Run main function
Main