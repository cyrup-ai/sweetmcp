#!/bin/bash
# Master Build and Signing Script for rio-ext-test
# Orchestrates cross-platform builds and signing

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
APP_NAME="rio-ext-test"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

log_debug() {
    echo -e "${BLUE}[DEBUG]${NC} $1"
}

log_header() {
    echo -e "${CYAN}==== $1 ====${NC}"
}

show_usage() {
    cat << EOF
Usage: $0 [OPTIONS] [TARGETS...]

Build and sign rio-ext-test for multiple platforms.

OPTIONS:
    -h, --help          Show this help message
    -c, --clean         Clean build artifacts before building
    -r, --release       Build in release mode (default)
    -d, --debug         Build in debug mode
    -s, --skip-signing  Skip signing step
    -v, --verbose       Enable verbose output
    --no-package        Skip package creation

TARGETS:
    all                 Build for all supported platforms (default)
    macos               Build for macOS (both x86_64 and ARM64)
    macos-x86_64        Build for macOS x86_64
    macos-arm64         Build for macOS ARM64
    windows             Build for Windows x86_64
    linux               Build for Linux x86_64
    linux-arm64         Build for Linux ARM64

ENVIRONMENT VARIABLES:
    See signing-config.toml for required environment variables for signing.

EXAMPLES:
    $0                          # Build all targets in release mode
    $0 --clean macos windows    # Clean build for macOS and Windows only
    $0 --skip-signing linux     # Build Linux without signing
    $0 --debug --verbose        # Debug build with verbose output

EOF
}

parse_arguments() {
    CLEAN=false
    RELEASE=true
    SKIP_SIGNING=false
    VERBOSE=false
    NO_PACKAGE=false
    TARGETS=()
    
    while [[ $# -gt 0 ]]; do
        case $1 in
            -h|--help)
                show_usage
                exit 0
                ;;
            -c|--clean)
                CLEAN=true
                shift
                ;;
            -r|--release)
                RELEASE=true
                shift
                ;;
            -d|--debug)
                RELEASE=false
                shift
                ;;
            -s|--skip-signing)
                SKIP_SIGNING=true
                shift
                ;;
            -v|--verbose)
                VERBOSE=true
                shift
                ;;
            --no-package)
                NO_PACKAGE=true
                shift
                ;;
            all|macos|macos-x86_64|macos-arm64|windows|linux|linux-arm64)
                TARGETS+=("$1")
                shift
                ;;
            *)
                log_error "Unknown option: $1"
                show_usage
                exit 1
                ;;
        esac
    done
    
    # Default to all targets if none specified
    if [ ${#TARGETS[@]} -eq 0 ]; then
        TARGETS=("all")
    fi
    
    # Expand 'all' target
    if [[ " ${TARGETS[*]} " =~ " all " ]]; then
        TARGETS=("macos-x86_64" "macos-arm64" "windows" "linux" "linux-arm64")
    fi
    
    # Expand 'macos' target
    if [[ " ${TARGETS[*]} " =~ " macos " ]]; then
        # Remove 'macos' and add specific targets
        TARGETS=("${TARGETS[@]/macos}")
        TARGETS+=("macos-x86_64" "macos-arm64")
    fi
}

check_prerequisites() {
    log_info "Checking prerequisites..."
    
    # Check Rust installation
    if ! command -v cargo &> /dev/null; then
        log_error "Cargo not found. Please install Rust."
        exit 1
    fi
    
    # Check if we're in the right directory
    if [ ! -f "$PROJECT_ROOT/Cargo.toml" ]; then
        log_error "Cargo.toml not found. Please run this script from the project root."
        exit 1
    fi
    
    # Check for cross-compilation tools if needed
    for target in "${TARGETS[@]}"; do
        case $target in
            windows)
                if ! rustup target list --installed | grep -q "x86_64-pc-windows-msvc"; then
                    log_warn "Windows target not installed, installing..."
                    rustup target add x86_64-pc-windows-msvc
                fi
                ;;
            linux-arm64)
                if ! rustup target list --installed | grep -q "aarch64-unknown-linux-gnu"; then
                    log_warn "Linux ARM64 target not installed, installing..."
                    rustup target add aarch64-unknown-linux-gnu
                fi
                ;;
        esac
    done
    
    log_info "Prerequisites check completed"
}

clean_artifacts() {
    if [ "$CLEAN" = true ]; then
        log_info "Cleaning build artifacts..."
        cd "$PROJECT_ROOT"
        cargo clean
        rm -rf target/packages
        log_info "Clean completed"
    fi
}

build_target() {
    local target=$1
    local rust_target=""
    local binary_name="$APP_NAME"
    
    case $target in
        macos-x86_64)
            rust_target="x86_64-apple-darwin"
            ;;
        macos-arm64)
            rust_target="aarch64-apple-darwin"
            ;;
        windows)
            rust_target="x86_64-pc-windows-msvc"
            binary_name="$APP_NAME.exe"
            ;;
        linux)
            rust_target="x86_64-unknown-linux-gnu"
            ;;
        linux-arm64)
            rust_target="aarch64-unknown-linux-gnu"
            ;;
        *)
            log_error "Unknown target: $target"
            return 1
            ;;
    esac
    
    log_header "Building $target ($rust_target)"
    
    cd "$PROJECT_ROOT"
    
    local build_cmd="cargo build --target $rust_target"
    if [ "$RELEASE" = true ]; then
        build_cmd="$build_cmd --release"
    fi
    
    if [ "$VERBOSE" = true ]; then
        build_cmd="$build_cmd --verbose"
    fi
    
    log_info "Executing: $build_cmd"
    eval "$build_cmd"
    
    # Verify binary exists
    local profile_dir="debug"
    if [ "$RELEASE" = true ]; then
        profile_dir="release"
    fi
    
    local binary_path="target/$rust_target/$profile_dir/$binary_name"
    if [ ! -f "$binary_path" ]; then
        log_error "Build failed: binary not found at $binary_path"
        return 1
    fi
    
    log_info "Build completed: $binary_path"
    return 0
}

sign_target() {
    local target=$1
    
    if [ "$SKIP_SIGNING" = true ]; then
        log_warn "Skipping signing for $target"
        return 0
    fi
    
    log_header "Signing $target"
    
    cd "$PROJECT_ROOT"
    
    case $target in
        macos-*)
            if [ -x "$SCRIPT_DIR/macos-sign.sh" ]; then
                "$SCRIPT_DIR/macos-sign.sh"
            else
                log_warn "macOS signing script not found or not executable"
            fi
            ;;
        windows)
            if [ -f "$SCRIPT_DIR/windows-sign.ps1" ]; then
                if command -v pwsh &> /dev/null; then
                    pwsh -File "$SCRIPT_DIR/windows-sign.ps1"
                elif command -v powershell &> /dev/null; then
                    powershell -File "$SCRIPT_DIR/windows-sign.ps1"
                else
                    log_warn "PowerShell not found, skipping Windows signing"
                fi
            else
                log_warn "Windows signing script not found"
            fi
            ;;
        linux*)
            if [ -x "$SCRIPT_DIR/linux-package.sh" ] && [ "$NO_PACKAGE" = false ]; then
                "$SCRIPT_DIR/linux-package.sh"
            else
                log_warn "Linux packaging script not found, not executable, or packaging disabled"
            fi
            ;;
    esac
}

create_distribution() {
    log_header "Creating Distribution Packages"
    
    local dist_dir="$PROJECT_ROOT/target/dist"
    mkdir -p "$dist_dir"
    
    # Copy all built artifacts to distribution directory
    for target in "${TARGETS[@]}"; do
        local profile_dir="debug"
        if [ "$RELEASE" = true ]; then
            profile_dir="release"
        fi
        
        case $target in
            macos-*)
                # Copy signed DMG if it exists
                local dmg_file="$PROJECT_ROOT/target/$profile_dir/$APP_NAME.dmg"
                if [ -f "$dmg_file" ]; then
                    cp "$dmg_file" "$dist_dir/${APP_NAME}-${target}.dmg"
                    log_info "Copied DMG: ${APP_NAME}-${target}.dmg"
                fi
                ;;
            windows)
                # Copy signed installer or ZIP
                local msi_file="$PROJECT_ROOT/target/$profile_dir/$APP_NAME-installer.msi"
                local zip_file="$PROJECT_ROOT/target/$profile_dir/$APP_NAME.zip"
                if [ -f "$msi_file" ]; then
                    cp "$msi_file" "$dist_dir/${APP_NAME}-${target}.msi"
                    log_info "Copied MSI: ${APP_NAME}-${target}.msi"
                elif [ -f "$zip_file" ]; then
                    cp "$zip_file" "$dist_dir/${APP_NAME}-${target}.zip"
                    log_info "Copied ZIP: ${APP_NAME}-${target}.zip"
                fi
                ;;
            linux*)
                # Copy Linux packages
                local packages_dir="$PROJECT_ROOT/target/packages"
                if [ -d "$packages_dir" ]; then
                    find "$packages_dir" -type f \( -name "*.deb" -o -name "*.rpm" -o -name "*.tar.xz" \) -exec cp {} "$dist_dir/" \;
                    log_info "Copied Linux packages to distribution directory"
                fi
                ;;
        esac
    done
    
    # Create checksums
    cd "$dist_dir"
    if [ -n "$(ls -A 2>/dev/null)" ]; then
        sha256sum * > checksums.sha256
        log_info "Created checksums.sha256"
        
        # List distribution contents
        log_info "Distribution contents:"
        ls -la "$dist_dir"
    else
        log_warn "No distribution files found"
    fi
}

main() {
    log_header "Rio Ext Test - Build and Sign"
    
    parse_arguments "$@"
    
    if [ "$VERBOSE" = true ]; then
        log_debug "Configuration:"
        log_debug "  Targets: ${TARGETS[*]}"
        log_debug "  Release: $RELEASE"
        log_debug "  Clean: $CLEAN"
        log_debug "  Skip Signing: $SKIP_SIGNING"
        log_debug "  No Package: $NO_PACKAGE"
    fi
    
    check_prerequisites
    clean_artifacts
    
    # Build and sign each target
    local failed_targets=()
    for target in "${TARGETS[@]}"; do
        if build_target "$target"; then
            sign_target "$target"
        else
            failed_targets+=("$target")
        fi
    done
    
    # Report results
    if [ ${#failed_targets[@]} -eq 0 ]; then
        log_info "All targets built successfully"
        create_distribution
        log_header "Build and Sign Completed Successfully"
    else
        log_error "Failed targets: ${failed_targets[*]}"
        exit 1
    fi
}

# Run main function with all arguments
main "$@"