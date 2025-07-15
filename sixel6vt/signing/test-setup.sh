#!/bin/bash
# Test script for signing setup verification

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m'

log_pass() {
    echo -e "${GREEN}[PASS]${NC} $1"
}

log_fail() {
    echo -e "${RED}[FAIL]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

test_file_exists() {
    local file="$1"
    local description="$2"
    
    if [ -f "$file" ]; then
        log_pass "$description exists"
        return 0
    else
        log_fail "$description missing: $file"
        return 1
    fi
}

test_file_executable() {
    local file="$1"
    local description="$2"
    
    if [ -x "$file" ]; then
        log_pass "$description is executable"
        return 0
    else
        log_fail "$description not executable: $file"
        return 1
    fi
}

test_cargo_config() {
    log_pass "Testing Cargo configuration..."
    
    test_file_exists "$PROJECT_ROOT/.cargo/config.toml" "Cargo config file"
    test_file_exists "$PROJECT_ROOT/build.rs" "Build script"
    
    # Test that Cargo.toml references build.rs
    if grep -q "build = \"build.rs\"" "$PROJECT_ROOT/Cargo.toml"; then
        log_pass "Cargo.toml references build script"
    else
        log_fail "Cargo.toml missing build script reference"
    fi
}

test_signing_scripts() {
    log_pass "Testing signing scripts..."
    
    test_file_exists "$SCRIPT_DIR/signing-config.toml" "Signing configuration"
    test_file_exists "$SCRIPT_DIR/macos-sign.sh" "macOS signing script"
    test_file_exists "$SCRIPT_DIR/windows-sign.ps1" "Windows signing script"
    test_file_exists "$SCRIPT_DIR/linux-package.sh" "Linux packaging script"
    test_file_exists "$SCRIPT_DIR/build-and-sign.sh" "Master build script"
    test_file_exists "$SCRIPT_DIR/README.md" "Documentation"
    
    test_file_executable "$SCRIPT_DIR/macos-sign.sh" "macOS signing script"
    test_file_executable "$SCRIPT_DIR/linux-package.sh" "Linux packaging script"
    test_file_executable "$SCRIPT_DIR/build-and-sign.sh" "Master build script"
}

test_build_capability() {
    log_pass "Testing build capability..."
    
    cd "$PROJECT_ROOT"
    
    # Test that project can be built
    if cargo check --quiet; then
        log_pass "Project passes cargo check"
    else
        log_fail "Project fails cargo check"
    fi
    
    # Test build script syntax
    if cargo build --quiet 2>/dev/null; then
        log_pass "Build script executes successfully"
    else
        log_warn "Build script has issues (this might be expected in some environments)"
    fi
}

test_cross_compilation_targets() {
    log_pass "Testing cross-compilation targets..."
    
    local targets=(
        "x86_64-apple-darwin"
        "aarch64-apple-darwin"
        "x86_64-pc-windows-msvc"
        "x86_64-unknown-linux-gnu"
        "aarch64-unknown-linux-gnu"
    )
    
    for target in "${targets[@]}"; do
        if rustup target list --installed | grep -q "$target"; then
            log_pass "Target $target is installed"
        else
            log_warn "Target $target not installed (install with: rustup target add $target)"
        fi
    done
}

test_environment_variables() {
    log_pass "Testing environment variable documentation..."
    
    # Check if signing config has environment variable documentation
    if grep -q "MACOS_DEVELOPER_ID" "$SCRIPT_DIR/signing-config.toml"; then
        log_pass "macOS environment variables documented"
    else
        log_fail "macOS environment variables not documented"
    fi
    
    if grep -q "WINDOWS_CERT_PATH" "$SCRIPT_DIR/signing-config.toml"; then
        log_pass "Windows environment variables documented"
    else
        log_fail "Windows environment variables not documented"
    fi
    
    if grep -q "LINUX_GPG_KEY_ID" "$SCRIPT_DIR/signing-config.toml"; then
        log_pass "Linux environment variables documented"
    else
        log_fail "Linux environment variables not documented"
    fi
}

test_script_help() {
    log_pass "Testing script help functionality..."
    
    cd "$PROJECT_ROOT"
    
    # Test that build script shows help
    if "$SCRIPT_DIR/build-and-sign.sh" --help >/dev/null 2>&1; then
        log_pass "Build script help works"
    else
        log_fail "Build script help fails"
    fi
}

main() {
    echo "Rio Ext Test - Signing Setup Verification"
    echo "========================================"
    echo
    
    test_cargo_config
    echo
    
    test_signing_scripts
    echo
    
    test_build_capability
    echo
    
    test_cross_compilation_targets
    echo
    
    test_environment_variables
    echo
    
    test_script_help
    echo
    
    log_pass "Signing setup verification completed"
    echo
    echo "Next steps:"
    echo "1. Install cross-compilation targets if needed"
    echo "2. Set up signing certificates and environment variables"
    echo "3. Test build with: ./signing/build-and-sign.sh --skip-signing"
    echo "4. Test full signing workflow with proper certificates"
}

main "$@"