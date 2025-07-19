# Platform-Specific Signing Setup for Rio Ext Test

This directory contains platform-specific build configurations and signing scripts for the Rio Ext Test terminal emulator application.

## Overview

The signing setup supports:
- **macOS**: Code signing with Developer ID and notarization
- **Windows**: Authenticode signing with MSI installer creation
- **Linux**: GPG signing with DEB, RPM, and TAR.XZ package creation

## Files Structure

```
signing/
├── README.md                 # This documentation
├── signing-config.toml       # Configuration file for all platforms
├── build-and-sign.sh        # Master build and signing script
├── macos-sign.sh            # macOS code signing script
├── windows-sign.ps1         # Windows code signing script
└── linux-package.sh         # Linux package creation and signing
```

## Quick Start

1. **Build for all platforms:**
   ```bash
   ./signing/build-and-sign.sh
   ```

2. **Build for specific platform:**
   ```bash
   ./signing/build-and-sign.sh macos
   ./signing/build-and-sign.sh windows
   ./signing/build-and-sign.sh linux
   ```

3. **Clean build:**
   ```bash
   ./signing/build-and-sign.sh --clean
   ```

## Environment Variables

### macOS Signing
Set these environment variables for macOS code signing:
```bash
export MACOS_DEVELOPER_ID="Developer ID Application: Your Name (TEAM_ID)"
export MACOS_KEYCHAIN_PROFILE="your-notarization-profile"
export MACOS_TEAM_ID="YOUR_TEAM_ID"
```

### Windows Signing
Set these environment variables for Windows code signing:
```bash
export WINDOWS_CERT_PATH="/path/to/certificate.p12"
export WINDOWS_CERT_PASSWORD="certificate_password"
```

### Linux Signing
Set these environment variables for Linux package signing:
```bash
export LINUX_MAINTAINER="Your Name <email@example.com>"
export LINUX_GPG_KEY_ID="your_gpg_key_id"
export LINUX_GPG_PASSPHRASE="gpg_passphrase"  # Optional
```

## Platform-Specific Details

### macOS

**Prerequisites:**
- Xcode command line tools
- Valid Developer ID Application certificate
- Configured notarization profile (for release builds)

**Process:**
1. Signs the binary with Developer ID
2. Submits for notarization (if configured)
3. Creates a DMG installer

**Manual Usage:**
```bash
./signing/macos-sign.sh
```

### Windows

**Prerequisites:**
- Windows SDK (for signtool.exe)
- Valid code signing certificate (.p12 or .pfx)
- WiX toolset (optional, for MSI creation)

**Process:**
1. Signs the binary with Authenticode
2. Creates MSI installer (if WiX available) or ZIP package
3. Signs the installer

**Manual Usage:**
```powershell
./signing/windows-sign.ps1
```

### Linux

**Prerequisites:**
- dpkg-deb (for DEB packages)
- rpmbuild (for RPM packages)
- GPG (for signing)

**Process:**
1. Creates DEB package for Debian/Ubuntu
2. Creates RPM package for RedHat/Fedora
3. Creates TAR.XZ archive for universal distribution
4. Signs all packages with GPG

**Manual Usage:**
```bash
./signing/linux-package.sh
```

## Build Configuration

### Cargo Configuration
The `.cargo/config.toml` file contains platform-specific build settings:
- Linker configurations
- Target-specific flags
- Static linking options

### Build Script
The `build.rs` script handles:
- Platform detection
- Architecture-specific optimizations
- Library linking
- Code signing preparation

## Cross-Platform Builds

### Supported Targets
- `x86_64-apple-darwin` (macOS Intel)
- `aarch64-apple-darwin` (macOS Apple Silicon)
- `x86_64-pc-windows-msvc` (Windows)
- `x86_64-unknown-linux-gnu` (Linux Intel)
- `aarch64-unknown-linux-gnu` (Linux ARM64)

### Target Installation
```bash
# Add Windows target
rustup target add x86_64-pc-windows-msvc

# Add Linux ARM64 target  
rustup target add aarch64-unknown-linux-gnu
```

## Advanced Usage

### Custom Build Options
```bash
# Debug build with verbose output
./signing/build-and-sign.sh --debug --verbose

# Skip signing step
./signing/build-and-sign.sh --skip-signing

# Build specific targets only
./signing/build-and-sign.sh macos-x86_64 windows

# Skip package creation
./signing/build-and-sign.sh --no-package linux
```

### Development Workflow
For development builds without signing:
```bash
# Quick build for current platform
cargo build --release

# Cross-platform build without signing
./signing/build-and-sign.sh --skip-signing
```

## Distribution

After successful builds, distribution packages are created in:
- `target/dist/` - Final distribution packages
- `target/dist/checksums.sha256` - SHA256 checksums

### Package Types
- **macOS**: `.dmg` installer
- **Windows**: `.msi` installer or `.zip` archive
- **Linux**: `.deb`, `.rpm`, and `.tar.xz` packages

## Troubleshooting

### Common Issues

1. **Missing signing certificates:**
   - Ensure environment variables are set correctly
   - Verify certificate validity and permissions

2. **Cross-compilation failures:**
   - Install required targets with `rustup target add`
   - Check platform-specific toolchain requirements

3. **Package creation failures:**
   - Install platform-specific packaging tools
   - Verify GPG configuration for Linux signing

### Debug Information
Enable verbose output for detailed build information:
```bash
./signing/build-and-sign.sh --verbose
```

## Security Considerations

- Store certificates and keys securely
- Use environment variables for sensitive data
- Verify signatures after creation
- Keep signing tools and certificates up to date

## Automation

This setup is designed for both manual use and CI/CD integration:
- Environment variables for configuration
- Return codes for script success/failure
- Comprehensive logging for debugging
- Atomic operations with cleanup on failure