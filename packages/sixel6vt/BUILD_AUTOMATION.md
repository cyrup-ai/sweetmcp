# PTY Terminal Build Automation System

This document describes the comprehensive build automation system for PTY Terminal, including multi-platform builds, continuous integration, and installer creation.

## Overview

The build automation system provides:

- **Multi-platform builds** for macOS, Windows, and Linux
- **Automated installer creation** (DMG, MSI, AppImage, DEB packages)
- **Code signing and notarization** for distribution
- **Continuous integration** with GitHub Actions
- **Local development workflows** with convenient scripts
- **Docker-based builds** for consistent environments
- **Quality assurance** with automated testing and linting

## Quick Start

### Local Development

```bash
# Quick development build
make dev

# Release build with installer
make all

# Build for specific platform
./scripts/build-local.sh --release --platform linux-x86_64

# Clean build
make clean
```

### Windows (PowerShell)

```powershell
# Quick development build
.\scripts\build-local.ps1

# Release build with installer
.\scripts\build-local.ps1 -Release -Installer

# Build for specific target
.\scripts\build-local.ps1 -Release -Target aarch64-pc-windows-msvc
```

## Build System Components

### 1. Core Build Scripts

#### `build-installers.rs`
- Rust-based multi-platform build orchestrator
- Handles cross-compilation, signing, and installer creation
- Supports all target platforms with automatic toolchain management

```bash
# Build for current platform
cargo run --bin build-installers

# Build for all platforms
cargo run --bin build-installers -- --all-platforms

# Build for specific platform
cargo run --bin build-installers -- --platform macos-aarch64
```

#### `scripts/build-local.sh` (Unix)
```bash
# Development workflow
./scripts/build-local.sh --release --installer --sign

# Cross-compilation
./scripts/build-local.sh --target aarch64-apple-darwin --release

# Clean build
./scripts/build-local.sh --clean --release
```

#### `scripts/build-local.ps1` (Windows)
```powershell
# Development workflow
.\scripts\build-local.ps1 -Release -Installer -Sign

# Cross-compilation
.\scripts\build-local.ps1 -Target aarch64-pc-windows-msvc -Release

# Clean build
.\scripts\build-local.ps1 -Clean -Release
```

### 2. Makefile Targets

The Makefile provides convenient targets for common operations:

```bash
# Development
make dev          # Format, lint, test, build
make build        # Debug build
make release      # Release build
make test         # Run tests
make clean        # Clean artifacts

# Code quality
make fmt          # Format code
make clippy       # Run lints
make audit        # Security audit

# Packaging
make installer    # Create installer
make package      # Full release package
make all          # Release + installer

# Cross-compilation
make build-macos    # All macOS architectures
make build-linux    # All Linux architectures  
make build-windows  # All Windows architectures
make build-all      # All platforms
```

### 3. Docker Support

#### Multi-stage Dockerfile
- **linux-builder**: Cross-compilation environment
- **development**: Full development environment with tools
- **runtime**: Minimal production runtime
- **ci**: Continuous integration environment

```bash
# Build for Linux in Docker
docker build --target linux-builder -t pty-terminal-builder .

# Development environment
docker build --target development -t pty-terminal-dev .
docker run -it --rm -v $(pwd):/app pty-terminal-dev

# CI build
docker build --target ci -t pty-terminal-ci .
docker run --rm -v $(pwd)/artifacts:/app/dist pty-terminal-ci
```

### 4. GitHub Actions Workflow

The CI/CD pipeline in `.github/workflows/build-pty-terminal.yml` provides:

- **Multi-platform builds** on every push/PR
- **Automated testing** with security audits
- **Code signing** for release builds
- **Draft releases** with build artifacts
- **Universal macOS binaries** combining Intel and Apple Silicon
- **Security scanning** and dependency audits

#### Workflow Triggers
- Push to `main` or `develop` branches
- Pull requests to `main`
- Release publication
- Manual workflow dispatch with platform selection

#### Build Matrix
- macOS: Intel (x86_64) and Apple Silicon (aarch64)
- Windows: x64 and ARM64
- Linux: x64 and ARM64

## Configuration

### Build Configuration (`build-config.toml`)

Centralized configuration for build settings:

```toml
[package]
name = "PTY Terminal"
binary_name = "rio-ext-test"
vendor = "SweetMCP"

[build]
default_profile = "release"
enable_lto = true
strip_symbols = true

[targets]
supported = [
    "x86_64-apple-darwin",
    "aarch64-apple-darwin",
    "x86_64-pc-windows-msvc",
    "aarch64-pc-windows-msvc",
    "x86_64-unknown-linux-gnu",
    "aarch64-unknown-linux-gnu",
]

[signing]
enable_signing = false
sign_release_builds_only = true

[installers]
create_installers = true
create_universal_macos = true
```

## Code Signing & Distribution

### macOS Code Signing

1. **Setup Developer Certificate**:
   ```bash
   # Import certificate to keychain
   security import certificate.p12 -k ~/Library/Keychains/login.keychain
   ```

2. **Configure Signing**:
   ```bash
   export ENABLE_SIGNING=1
   export SIGNING_IDENTITY="Developer ID Application: Your Name"
   ./scripts/build-local.sh --release --sign
   ```

3. **Notarization** (for distribution):
   ```bash
   export MACOS_NOTARIZATION_APPLE_ID="your-apple-id"
   export MACOS_NOTARIZATION_PWD="app-specific-password"
   ```

### Windows Code Signing

1. **Setup Certificate**:
   - Obtain code signing certificate (.pfx file)
   - Install WiX Toolset for MSI creation

2. **Configure Signing**:
   ```powershell
   $env:ENABLE_SIGNING = "1"
   $env:WINDOWS_CERTIFICATE_PATH = "path\to\certificate.pfx"
   .\scripts\build-local.ps1 -Release -Sign
   ```

### Linux Package Signing

1. **Setup GPG Key**:
   ```bash
   gpg --generate-key
   export GPG_KEY_ID="your-key-id"
   ```

2. **Configure Repository Signing**:
   ```bash
   export ENABLE_PACKAGE_SIGNING=1
   ./scripts/build-local.sh --release --sign
   ```

## Installer Creation

### macOS (.dmg)
- App bundle with proper Info.plist
- DMG with custom background and layout
- Code signing and notarization ready

### Windows (.msi)
- WiX-based MSI installer
- Proper registry entries and shortcuts
- Authenticode signing support

### Linux (AppImage, .deb)
- Portable AppImage for universal compatibility
- Debian packages for apt repositories
- Desktop integration files

## Testing

### Local Testing
```bash
# Run all tests
make test

# Headless testing (for CI)
./scripts/run-tests-headless.sh

# Specific test categories
./scripts/run-tests-headless.sh unit
./scripts/run-tests-headless.sh gui
```

### CI Testing
The CI pipeline runs:
- Code formatting checks
- Clippy lints
- Unit and integration tests
- Security audits
- Cross-platform builds

## Development Workflow

### Daily Development
1. **Code Changes**: Make your changes
2. **Quality Check**: `make dev` (format, lint, test, build)
3. **Commit**: Commit with descriptive message
4. **Push**: Push to feature branch

### Release Preparation
1. **Version Bump**: Update version in `Cargo.toml`
2. **Full Build**: `make build-all` or use GitHub Actions
3. **Test Installers**: Test generated installers on target platforms
4. **Create Release**: Use GitHub release with generated artifacts

### Debug Build Issues

#### Enable Verbose Output
```bash
export VERBOSE=1
./scripts/build-local.sh --release --verbose
```

#### Check Dependencies
```bash
make check-deps
cargo tree --duplicates
```

#### Platform-Specific Issues
- **macOS**: Check Xcode command line tools
- **Windows**: Verify Visual Studio Build Tools
- **Linux**: Install development packages

## Environment Variables

### Build Configuration
- `CARGO_TARGET_DIR`: Override build directory
- `RUST_LOG`: Set logging level
- `VERBOSE`: Enable verbose output
- `CLEAN_BUILD`: Force clean build

### Signing Configuration
- `ENABLE_SIGNING`: Enable code signing
- `SIGNING_IDENTITY`: macOS signing identity
- `WINDOWS_CERTIFICATE_PATH`: Windows certificate path
- `GPG_KEY_ID`: Linux GPG key for package signing

### CI Configuration
- `CI`: Indicates CI environment
- `GITHUB_ACTIONS`: GitHub Actions specific settings
- `BUILD_TYPE`: Build type (debug/release)
- `TARGET`: Specific target triple

## Troubleshooting

### Common Issues

#### Cross-compilation Failures
- Install target: `rustup target add <target-triple>`
- Install cross: `cargo install cross`
- Check system dependencies

#### Signing Failures
- Verify certificate installation
- Check signing identity
- Ensure proper entitlements (macOS)

#### Installer Creation Failures
- Install platform-specific tools (WiX, create-dmg, etc.)
- Check file permissions
- Verify asset files exist

#### CI/CD Issues
- Check GitHub Actions logs
- Verify secrets are configured
- Ensure proper permissions

### Getting Help

1. **Check Logs**: Look in `ci-reports/` and build output
2. **Verbose Mode**: Use `--verbose` flags for detailed output
3. **Clean Build**: Try `make clean` followed by rebuild
4. **Platform Tools**: Verify platform-specific tools are installed

## Future Enhancements

- **Automated performance benchmarks**
- **Integration with package managers** (Homebrew, Chocolatey, Snap)
- **Delta updates** for efficient distribution
- **Crash reporting** integration
- **Automated testing** on real devices

This build automation system provides a solid foundation for professional software distribution across all major platforms.