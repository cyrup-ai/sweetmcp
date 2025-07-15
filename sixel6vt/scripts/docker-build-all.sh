#!/bin/bash
# Docker Multi-Platform Build Script
# Builds PTY Terminal for all supported platforms within Docker

set -euo pipefail

# Configuration
BUILD_OUTPUT_DIR="${BUILD_OUTPUT_DIR:-/app/dist}"
VERBOSE="${VERBOSE:-false}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Supported platforms in Docker environment
PLATFORMS=(
    "linux-x86_64"
    "linux-aarch64"
)

# Note: Windows and macOS builds require additional setup
EXPERIMENTAL_PLATFORMS=(
    "windows-x86_64"
    "windows-aarch64"
)

main() {
    log_info "Starting Docker multi-platform build..."
    log_info "Output directory: $BUILD_OUTPUT_DIR"
    
    # Ensure output directory exists
    mkdir -p "$BUILD_OUTPUT_DIR"
    
    # Build for Linux platforms (supported)
    local success_count=0
    local total_count=0
    
    for platform in "${PLATFORMS[@]}"; do
        log_info "Building for platform: $platform"
        total_count=$((total_count + 1))
        
        if build_platform "$platform"; then
            log_success "Successfully built $platform"
            success_count=$((success_count + 1))
        else
            log_error "Failed to build $platform"
        fi
    done
    
    # Experimental builds (may fail)
    for platform in "${EXPERIMENTAL_PLATFORMS[@]}"; do
        log_warning "Attempting experimental build for: $platform"
        total_count=$((total_count + 1))
        
        if build_platform_experimental "$platform"; then
            log_success "Successfully built $platform (experimental)"
            success_count=$((success_count + 1))
        else
            log_warning "Failed to build $platform (experimental - this is expected)"
        fi
    done
    
    # Summary
    log_info "Build Summary:"
    echo "  Successful builds: $success_count"
    echo "  Total attempts: $total_count"
    echo "  Success rate: $((success_count * 100 / total_count))%"
    
    # List artifacts
    log_info "Available artifacts:"
    find "$BUILD_OUTPUT_DIR" -type f -name "*" | while read -r file; do
        echo "  $(basename "$file")"
    done
    
    if [ $success_count -gt 0 ]; then
        log_success "Docker multi-platform build completed with $success_count successful builds"
        exit 0
    else
        log_error "All builds failed"
        exit 1
    fi
}

build_platform() {
    local platform="$1"
    
    case "$platform" in
        linux-x86_64)
            build_linux_x86_64
            ;;
        linux-aarch64)
            build_linux_aarch64
            ;;
        *)
            log_error "Unknown platform: $platform"
            return 1
            ;;
    esac
}

build_platform_experimental() {
    local platform="$1"
    
    case "$platform" in
        windows-x86_64)
            build_windows_x86_64_experimental
            ;;
        windows-aarch64)
            build_windows_aarch64_experimental
            ;;
        *)
            log_error "Unknown experimental platform: $platform"
            return 1
            ;;
    esac
}

build_linux_x86_64() {
    log_info "Building for Linux x86_64..."
    
    # Use cross for reliable cross-compilation
    if command -v cross &> /dev/null; then
        cross build --release --target x86_64-unknown-linux-gnu
    else
        cargo build --release --target x86_64-unknown-linux-gnu
    fi
    
    local binary_path="target/x86_64-unknown-linux-gnu/release/rio-ext-test"
    if [ -f "$binary_path" ]; then
        cp "$binary_path" "$BUILD_OUTPUT_DIR/pty-terminal-linux-x86_64"
        
        # Create AppImage if possible
        if create_appimage "x86_64" "$binary_path"; then
            log_success "AppImage created for Linux x86_64"
        else
            log_warning "AppImage creation failed for Linux x86_64"
        fi
        
        return 0
    else
        log_error "Binary not found: $binary_path"
        return 1
    fi
}

build_linux_aarch64() {
    log_info "Building for Linux aarch64..."
    
    # Install cross-compilation toolchain if not present
    if ! rustup target list --installed | grep -q aarch64-unknown-linux-gnu; then
        rustup target add aarch64-unknown-linux-gnu
    fi
    
    # Use cross for reliable cross-compilation
    if command -v cross &> /dev/null; then
        cross build --release --target aarch64-unknown-linux-gnu
    else
        # Fallback to cargo (may require additional setup)
        cargo build --release --target aarch64-unknown-linux-gnu
    fi
    
    local binary_path="target/aarch64-unknown-linux-gnu/release/rio-ext-test"
    if [ -f "$binary_path" ]; then
        cp "$binary_path" "$BUILD_OUTPUT_DIR/pty-terminal-linux-aarch64"
        
        # Create AppImage if possible
        if create_appimage "aarch64" "$binary_path"; then
            log_success "AppImage created for Linux aarch64"
        else
            log_warning "AppImage creation failed for Linux aarch64"
        fi
        
        return 0
    else
        log_error "Binary not found: $binary_path"
        return 1
    fi
}

build_windows_x86_64_experimental() {
    log_info "Attempting experimental Windows x86_64 build..."
    
    # This requires MinGW-w64 or Wine + MSVC
    if ! rustup target list --installed | grep -q x86_64-pc-windows-gnu; then
        rustup target add x86_64-pc-windows-gnu
    fi
    
    # Try MinGW build
    if command -v x86_64-w64-mingw32-gcc &> /dev/null; then
        export CC_x86_64_pc_windows_gnu=x86_64-w64-mingw32-gcc
        export AR_x86_64_pc_windows_gnu=x86_64-w64-mingw32-ar
        
        if cargo build --release --target x86_64-pc-windows-gnu; then
            local binary_path="target/x86_64-pc-windows-gnu/release/rio-ext-test.exe"
            if [ -f "$binary_path" ]; then
                cp "$binary_path" "$BUILD_OUTPUT_DIR/pty-terminal-windows-x86_64.exe"
                return 0
            fi
        fi
    fi
    
    log_warning "Windows x86_64 experimental build failed (requires MinGW-w64 or MSVC)"
    return 1
}

build_windows_aarch64_experimental() {
    log_info "Attempting experimental Windows aarch64 build..."
    
    # This is very experimental and likely to fail
    if ! rustup target list --installed | grep -q aarch64-pc-windows-msvc; then
        rustup target add aarch64-pc-windows-msvc
    fi
    
    # This will likely fail without proper Windows SDK
    if cargo build --release --target aarch64-pc-windows-msvc; then
        local binary_path="target/aarch64-pc-windows-msvc/release/rio-ext-test.exe"
        if [ -f "$binary_path" ]; then
            cp "$binary_path" "$BUILD_OUTPUT_DIR/pty-terminal-windows-aarch64.exe"
            return 0
        fi
    fi
    
    log_warning "Windows aarch64 experimental build failed (requires Windows SDK)"
    return 1
}

create_appimage() {
    local arch="$1"
    local binary_path="$2"
    
    # Download appimagetool if not present
    local appimagetool="/tmp/appimagetool-$arch.AppImage"
    
    if [ ! -f "$appimagetool" ]; then
        local download_url
        case "$arch" in
            x86_64)
                download_url="https://github.com/AppImage/AppImageKit/releases/download/continuous/appimagetool-x86_64.AppImage"
                ;;
            aarch64)
                download_url="https://github.com/AppImage/AppImageKit/releases/download/continuous/appimagetool-aarch64.AppImage"
                ;;
            *)
                log_warning "Unknown architecture for AppImage: $arch"
                return 1
                ;;
        esac
        
        if command -v wget &> /dev/null; then
            wget -O "$appimagetool" "$download_url"
        elif command -v curl &> /dev/null; then
            curl -L -o "$appimagetool" "$download_url"
        else
            log_warning "Neither wget nor curl available for downloading appimagetool"
            return 1
        fi
        
        chmod +x "$appimagetool"
    fi
    
    # Create AppDir structure
    local appdir="/tmp/PTY_Terminal.AppDir"
    rm -rf "$appdir"
    mkdir -p "$appdir"
    
    # Copy binary as AppRun
    cp "$binary_path" "$appdir/AppRun"
    chmod +x "$appdir/AppRun"
    
    # Create .desktop file
    cat > "$appdir/pty-terminal.desktop" << 'EOF'
[Desktop Entry]
Name=PTY Terminal
Comment=Advanced terminal emulator with sixel support
Exec=AppRun
Icon=pty-terminal
Terminal=false
Type=Application
Categories=System;TerminalEmulator;
EOF
    
    # Copy icon if available
    if [ -f "assets/icon/icon-512x512.png" ]; then
        cp "assets/icon/icon-512x512.png" "$appdir/pty-terminal.png"
    elif [ -f "assets/icon/icon.png" ]; then
        cp "assets/icon/icon.png" "$appdir/pty-terminal.png"
    fi
    
    # Create AppImage
    local appimage_path="$BUILD_OUTPUT_DIR/PTY_Terminal-$arch.AppImage"
    
    if "$appimagetool" "$appdir" "$appimage_path"; then
        chmod +x "$appimage_path"
        return 0
    else
        return 1
    fi
}

# Handle script interruption
trap 'log_error "Build interrupted"; exit 130' INT TERM

# Run main function
main "$@"