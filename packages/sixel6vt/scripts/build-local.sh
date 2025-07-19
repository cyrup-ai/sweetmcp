#!/bin/bash
# Local Build Script for PTY Terminal
# This script provides a simplified interface for local development builds

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
BUILD_DIR="$PROJECT_DIR/target/local-build"
DIST_DIR="$PROJECT_DIR/dist"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Default values
BUILD_TYPE="debug"
TARGET=""
CLEAN=false
VERBOSE=false
INSTALLER=false
SIGN=false

# Print usage information
usage() {
    cat << EOF
PTY Terminal Local Build Script

Usage: $0 [OPTIONS]

OPTIONS:
    -r, --release           Build in release mode
    -t, --target TARGET     Specify target triple (e.g., x86_64-apple-darwin)
    -c, --clean             Clean build directory before building
    -v, --verbose           Enable verbose output
    -i, --installer         Create installer after building
    -s, --sign              Enable code signing (requires setup)
    -h, --help              Show this help message

EXAMPLES:
    $0                      # Debug build for current platform
    $0 -r                   # Release build for current platform  
    $0 -r -i                # Release build with installer
    $0 -t aarch64-apple-darwin -r  # Cross-compile for Apple Silicon

ENVIRONMENT VARIABLES:
    CARGO_TARGET_DIR        Override cargo target directory
    RUST_LOG               Set logging level (debug, info, warn, error)
    SIGNING_IDENTITY       macOS signing identity
    
EOF
}

# Logging functions
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

# Parse command line arguments
parse_args() {
    while [[ $# -gt 0 ]]; do
        case $1 in
            -r|--release)
                BUILD_TYPE="release"
                shift
                ;;
            -t|--target)
                TARGET="$2"
                shift 2
                ;;
            -c|--clean)
                CLEAN=true
                shift
                ;;
            -v|--verbose)
                VERBOSE=true
                shift
                ;;
            -i|--installer)
                INSTALLER=true
                shift
                ;;
            -s|--sign)
                SIGN=true
                shift
                ;;
            -h|--help)
                usage
                exit 0
                ;;
            *)
                log_error "Unknown option: $1"
                usage
                exit 1
                ;;
        esac
    done
}

# Detect current platform
detect_platform() {
    local os="$(uname -s)"
    local arch="$(uname -m)"
    
    case "$os" in
        Darwin)
            case "$arch" in
                x86_64) echo "macos-x86_64" ;;
                arm64) echo "macos-aarch64" ;;
                *) echo "macos-unknown" ;;
            esac
            ;;
        Linux)
            case "$arch" in
                x86_64) echo "linux-x86_64" ;;
                aarch64) echo "linux-aarch64" ;;
                *) echo "linux-unknown" ;;
            esac
            ;;
        MINGW*|MSYS*|CYGWIN*)
            case "$arch" in
                x86_64) echo "windows-x86_64" ;;
                *) echo "windows-unknown" ;;
            esac
            ;;
        *)
            echo "unknown"
            ;;
    esac
}

# Check dependencies
check_dependencies() {
    log_info "Checking dependencies..."
    
    # Check Rust
    if ! command -v cargo &> /dev/null; then
        log_error "Cargo not found. Please install Rust: https://rustup.rs/"
        exit 1
    fi
    
    # Check target if specified
    if [[ -n "$TARGET" ]]; then
        if ! rustup target list --installed | grep -q "$TARGET"; then
            log_info "Installing target: $TARGET"
            rustup target add "$TARGET"
        fi
    fi
    
    # Platform-specific dependencies
    local platform="$(detect_platform)"
    case "$platform" in
        macos-*)
            # Check for Xcode command line tools
            if ! xcode-select -p &> /dev/null; then
                log_warning "Xcode command line tools not found. Install with: xcode-select --install"
            fi
            ;;
        linux-*)
            # Check for essential build tools
            if ! command -v gcc &> /dev/null; then
                log_warning "GCC not found. Install build-essential package."
            fi
            ;;
        windows-*)
            # Check for Visual Studio Build Tools
            if ! command -v cl.exe &> /dev/null; then
                log_warning "MSVC not found. Install Visual Studio Build Tools."
            fi
            ;;
    esac
}

# Setup build environment
setup_build_env() {
    log_info "Setting up build environment..."
    
    # Create directories
    mkdir -p "$BUILD_DIR"
    mkdir -p "$DIST_DIR"
    
    # Set environment variables
    export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$BUILD_DIR}"
    
    if [[ "$VERBOSE" == true ]]; then
        export RUST_LOG="${RUST_LOG:-debug}"
        export RUST_BACKTRACE=1
    fi
    
    # Platform-specific setup
    local platform="$(detect_platform)"
    case "$platform" in
        macos-*)
            export MACOSX_DEPLOYMENT_TARGET="10.15"
            if [[ "$SIGN" == true ]]; then
                export ENABLE_SIGNING=1
            fi
            ;;
        windows-*)
            if [[ "$SIGN" == true ]]; then
                export ENABLE_AUTHENTICODE_SIGNING=1
            fi
            ;;
        linux-*)
            if [[ "$SIGN" == true ]]; then
                export ENABLE_PACKAGE_SIGNING=1
            fi
            ;;
    esac
}

# Clean build directory
clean_build() {
    if [[ "$CLEAN" == true ]]; then
        log_info "Cleaning build directory..."
        rm -rf "$BUILD_DIR"
        rm -rf "$DIST_DIR"
        cargo clean
    fi
}

# Format and lint code
check_code() {
    log_info "Checking code formatting and linting..."
    
    # Check formatting
    if ! cargo fmt -- --check; then
        log_warning "Code is not properly formatted. Run 'cargo fmt' to fix."
    fi
    
    # Run clippy
    local clippy_args=("clippy")
    if [[ -n "$TARGET" ]]; then
        clippy_args+=("--target" "$TARGET")
    fi
    
    if [[ "$BUILD_TYPE" == "release" ]]; then
        clippy_args+=("--release")
    fi
    
    clippy_args+=("--" "-D" "warnings")
    
    if ! cargo "${clippy_args[@]}"; then
        log_error "Clippy found issues. Please fix them before building."
        exit 1
    fi
}

# Run tests
run_tests() {
    log_info "Running tests..."
    
    local test_args=("test")
    if [[ -n "$TARGET" ]]; then
        test_args+=("--target" "$TARGET")
    fi
    
    if [[ "$BUILD_TYPE" == "release" ]]; then
        test_args+=("--release")
    fi
    
    if [[ "$VERBOSE" == true ]]; then
        test_args+=("--verbose")
    fi
    
    if ! cargo "${test_args[@]}"; then
        log_error "Tests failed"
        exit 1
    fi
}

# Build the project
build_project() {
    log_info "Building PTY Terminal ($BUILD_TYPE mode)..."
    
    local build_args=("build")
    if [[ "$BUILD_TYPE" == "release" ]]; then
        build_args+=("--release")
    fi
    
    if [[ -n "$TARGET" ]]; then
        build_args+=("--target" "$TARGET")
    fi
    
    if [[ "$VERBOSE" == true ]]; then
        build_args+=("--verbose")
    fi
    
    # Use cross for cross-compilation if available
    local build_cmd="cargo"
    if [[ -n "$TARGET" ]] && [[ "$TARGET" != "$(rustc -vV | grep host | cut -d' ' -f2)" ]]; then
        if command -v cross &> /dev/null; then
            build_cmd="cross"
            log_info "Using cross for cross-compilation"
        else
            log_warning "Cross-compilation target specified but 'cross' not found. Using cargo."
        fi
    fi
    
    if ! $build_cmd "${build_args[@]}"; then
        log_error "Build failed"
        exit 1
    fi
    
    log_success "Build completed successfully"
}

# Copy build artifacts
copy_artifacts() {
    log_info "Copying build artifacts..."
    
    local target_dir="$BUILD_DIR"
    if [[ -n "$TARGET" ]]; then
        target_dir="$target_dir/$TARGET"
    else
        target_dir="$target_dir/$(rustc -vV | grep host | cut -d' ' -f2)"
    fi
    
    local build_subdir="debug"
    if [[ "$BUILD_TYPE" == "release" ]]; then
        build_subdir="release"
    fi
    
    local binary_path="$target_dir/$build_subdir/rio-ext-test"
    local platform="$(detect_platform)"
    
    # Add extension for Windows
    if [[ "$platform" == windows-* ]]; then
        binary_path="$binary_path.exe"
    fi
    
    if [[ ! -f "$binary_path" ]]; then
        log_error "Binary not found at: $binary_path"
        exit 1
    fi
    
    # Copy to dist directory
    local dist_binary="$DIST_DIR/rio-ext-test"
    if [[ "$platform" == windows-* ]]; then
        dist_binary="$dist_binary.exe"
    fi
    
    cp "$binary_path" "$dist_binary"
    
    # Make executable (Unix-like systems)
    if [[ "$platform" != windows-* ]]; then
        chmod +x "$dist_binary"
    fi
    
    log_success "Binary copied to: $dist_binary"
}

# Create installer
create_installer() {
    if [[ "$INSTALLER" == true ]]; then
        log_info "Creating installer..."
        
        local platform="$(detect_platform)"
        local platform_arg=""
        
        case "$platform" in
            macos-*) platform_arg="--platform $platform" ;;
            linux-*) platform_arg="--platform $platform" ;;
            windows-*) platform_arg="--platform $platform" ;;
        esac
        
        if [[ -n "$platform_arg" ]]; then
            if cargo run --bin build-installers -- $platform_arg; then
                log_success "Installer created successfully"
            else
                log_error "Installer creation failed"
                exit 1
            fi
        else
            log_warning "Installer creation not supported for platform: $platform"
        fi
    fi
}

# Print build summary
print_summary() {
    log_info "Build Summary:"
    echo "  Platform: $(detect_platform)"
    echo "  Build Type: $BUILD_TYPE"
    echo "  Target: ${TARGET:-$(rustc -vV | grep host | cut -d' ' -f2)}"
    echo "  Signing: $(if [[ "$SIGN" == true ]]; then echo "enabled"; else echo "disabled"; fi)"
    echo "  Installer: $(if [[ "$INSTALLER" == true ]]; then echo "created"; else echo "skipped"; fi)"
    echo "  Binary: $DIST_DIR/rio-ext-test$(if [[ "$(detect_platform)" == windows-* ]]; then echo ".exe"; fi)"
    
    if [[ "$INSTALLER" == true ]]; then
        echo "  Installer: $DIST_DIR/"
        ls -la "$DIST_DIR"/*.{dmg,msi,deb,AppImage} 2>/dev/null || true
    fi
}

# Main execution
main() {
    parse_args "$@"
    
    # Change to project directory
    cd "$PROJECT_DIR"
    
    log_info "Starting PTY Terminal build process..."
    log_info "Project directory: $PROJECT_DIR"
    
    check_dependencies
    setup_build_env
    clean_build
    check_code
    run_tests
    build_project
    copy_artifacts
    create_installer
    print_summary
    
    log_success "Build process completed successfully!"
}

# Handle script interruption
trap 'log_error "Build interrupted"; exit 130' INT TERM

# Run main function
main "$@"