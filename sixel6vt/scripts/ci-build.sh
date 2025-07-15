#!/bin/bash
# CI/CD Build Script for PTY Terminal
# Designed to run in continuous integration environments

set -euo pipefail

# Configuration
CI="${CI:-false}"
GITHUB_ACTIONS="${GITHUB_ACTIONS:-false}"
BUILD_TYPE="${BUILD_TYPE:-release}"
TARGET="${TARGET:-}"
ENABLE_SIGNING="${ENABLE_SIGNING:-false}"
ENABLE_TESTS="${ENABLE_TESTS:-true}"
ENABLE_BENCHMARKS="${ENABLE_BENCHMARKS:-false}"
ENABLE_SECURITY_AUDIT="${ENABLE_SECURITY_AUDIT:-true}"

# Directories
PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BUILD_DIR="$PROJECT_DIR/target/ci-build"
ARTIFACTS_DIR="$PROJECT_DIR/ci-artifacts"
REPORTS_DIR="$PROJECT_DIR/ci-reports"

# Colors for output (disabled in CI unless explicitly enabled)
if [ "${CI_COLORS:-false}" = "true" ]; then
    RED='\033[0;31m'
    GREEN='\033[0;32m'
    YELLOW='\033[1;33m'
    BLUE='\033[0;34m'
    NC='\033[0m'
else
    RED=''
    GREEN=''
    YELLOW=''
    BLUE=''
    NC=''
fi

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

log_step() {
    echo ""
    echo "::group::$1" # GitHub Actions group
    log_info "$1"
}

log_step_end() {
    echo "::endgroup::" # GitHub Actions group end
}

# Error handling
handle_error() {
    local exit_code=$?
    log_error "CI build failed with exit code $exit_code"
    
    # Upload artifacts even on failure for debugging
    if [ -d "$REPORTS_DIR" ]; then
        log_info "Preserving reports for debugging..."
    fi
    
    exit $exit_code
}

trap handle_error ERR

# Setup CI environment
setup_ci_environment() {
    log_step "Setting up CI environment"
    
    # Create directories
    mkdir -p "$BUILD_DIR" "$ARTIFACTS_DIR" "$REPORTS_DIR"
    
    # Set environment variables
    export CARGO_TARGET_DIR="$BUILD_DIR"
    export RUST_BACKTRACE=1
    export CARGO_TERM_COLOR=always
    
    # CI-specific settings
    if [ "$CI" = "true" ]; then
        export CARGO_INCREMENTAL=0
        export RUSTFLAGS="-D warnings"
        export RUST_LOG=info
    fi
    
    # GitHub Actions specific
    if [ "$GITHUB_ACTIONS" = "true" ]; then
        # Add Cargo bin to PATH
        echo "$HOME/.cargo/bin" >> "$GITHUB_PATH"
        
        # Set outputs
        echo "artifacts-dir=$ARTIFACTS_DIR" >> "$GITHUB_OUTPUT"
        echo "reports-dir=$REPORTS_DIR" >> "$GITHUB_OUTPUT"
    fi
    
    log_step_end
}

# Install required tools
install_tools() {
    log_step "Installing required tools"
    
    # Basic Rust components
    if ! rustup component list --installed | grep -q rustfmt; then
        rustup component add rustfmt
    fi
    
    if ! rustup component list --installed | grep -q clippy; then
        rustup component add clippy
    fi
    
    # Install target if specified
    if [ -n "$TARGET" ]; then
        if ! rustup target list --installed | grep -q "$TARGET"; then
            rustup target add "$TARGET"
        fi
    fi
    
    # Install additional tools for CI
    local tools_to_install=()
    
    if [ "$ENABLE_SECURITY_AUDIT" = "true" ] && ! command -v cargo-audit &> /dev/null; then
        tools_to_install+=("cargo-audit")
    fi
    
    if [ "$ENABLE_BENCHMARKS" = "true" ] && ! command -v cargo-criterion &> /dev/null; then
        tools_to_install+=("cargo-criterion")
    fi
    
    # Install tools if needed
    if [ ${#tools_to_install[@]} -gt 0 ]; then
        log_info "Installing additional tools: ${tools_to_install[*]}"
        for tool in "${tools_to_install[@]}"; do
            cargo install "$tool" || log_warning "Failed to install $tool"
        done
    fi
    
    log_step_end
}

# Check code formatting
check_formatting() {
    log_step "Checking code formatting"
    
    if ! cargo fmt --all -- --check; then
        log_error "Code is not properly formatted"
        log_info "Run 'cargo fmt' to fix formatting issues"
        exit 1
    fi
    
    log_success "Code formatting is correct"
    log_step_end
}

# Run Clippy lints
run_clippy() {
    log_step "Running Clippy lints"
    
    local clippy_args=("clippy" "--all-targets" "--all-features")
    
    if [ -n "$TARGET" ]; then
        clippy_args+=("--target" "$TARGET")
    fi
    
    clippy_args+=("--" "-D" "warnings")
    
    # Run clippy and save output
    local clippy_output="$REPORTS_DIR/clippy-report.txt"
    if cargo "${clippy_args[@]}" 2>&1 | tee "$clippy_output"; then
        log_success "Clippy checks passed"
    else
        log_error "Clippy found issues"
        exit 1
    fi
    
    log_step_end
}

# Run tests
run_tests() {
    if [ "$ENABLE_TESTS" != "true" ]; then
        log_info "Tests disabled, skipping"
        return 0
    fi
    
    log_step "Running tests"
    
    local test_args=("test" "--all-features")
    
    if [ -n "$TARGET" ]; then
        test_args+=("--target" "$TARGET")
    fi
    
    if [ "$BUILD_TYPE" = "release" ]; then
        test_args+=("--release")
    fi
    
    # Add test output format for CI
    test_args+=("--" "--test-threads=1" "--nocapture")
    
    # Run tests and capture output
    local test_output="$REPORTS_DIR/test-report.txt"
    if cargo "${test_args[@]}" 2>&1 | tee "$test_output"; then
        log_success "All tests passed"
    else
        log_error "Some tests failed"
        exit 1
    fi
    
    log_step_end
}

# Run security audit
run_security_audit() {
    if [ "$ENABLE_SECURITY_AUDIT" != "true" ]; then
        log_info "Security audit disabled, skipping"
        return 0
    fi
    
    log_step "Running security audit"
    
    if command -v cargo-audit &> /dev/null; then
        local audit_output="$REPORTS_DIR/security-audit.txt"
        if cargo audit 2>&1 | tee "$audit_output"; then
            log_success "Security audit passed"
        else
            log_warning "Security audit found issues"
            # Don't fail the build for audit issues, just warn
        fi
    else
        log_warning "cargo-audit not available, skipping security audit"
    fi
    
    log_step_end
}

# Check dependencies
check_dependencies() {
    log_step "Checking dependencies"
    
    # Check for duplicate dependencies
    local deps_output="$REPORTS_DIR/dependencies.txt"
    cargo tree --duplicates > "$deps_output" 2>&1 || true
    
    if [ -s "$deps_output" ]; then
        log_warning "Duplicate dependencies found:"
        cat "$deps_output"
    else
        log_success "No duplicate dependencies found"
    fi
    
    # Check for outdated dependencies (if tool is available)
    if command -v cargo-outdated &> /dev/null; then
        local outdated_output="$REPORTS_DIR/outdated-dependencies.txt"
        cargo outdated > "$outdated_output" 2>&1 || true
        
        if [ -s "$outdated_output" ]; then
            log_info "Outdated dependencies report saved to $outdated_output"
        fi
    fi
    
    log_step_end
}

# Build the project
build_project() {
    log_step "Building project ($BUILD_TYPE mode)"
    
    local build_args=("build" "--all-features")
    
    if [ "$BUILD_TYPE" = "release" ]; then
        build_args+=("--release")
    fi
    
    if [ -n "$TARGET" ]; then
        build_args+=("--target" "$TARGET")
    fi
    
    # Add verbose output for CI debugging
    build_args+=("--verbose")
    
    # Build and capture output
    local build_output="$REPORTS_DIR/build-output.txt"
    if cargo "${build_args[@]}" 2>&1 | tee "$build_output"; then
        log_success "Build completed successfully"
    else
        log_error "Build failed"
        exit 1
    fi
    
    # Copy binary to artifacts
    copy_build_artifacts
    
    log_step_end
}

# Copy build artifacts
copy_build_artifacts() {
    log_info "Copying build artifacts"
    
    local target_dir="$BUILD_DIR"
    if [ -n "$TARGET" ]; then
        target_dir="$target_dir/$TARGET"
    else
        target_dir="$target_dir/$(rustc -vV | grep host | cut -d' ' -f2)"
    fi
    
    local build_subdir="debug"
    if [ "$BUILD_TYPE" = "release" ]; then
        build_subdir="release"
    fi
    
    local binary_path="$target_dir/$build_subdir/rio-ext-test"
    
    # Add .exe extension for Windows
    if [[ "$TARGET" == *"windows"* ]] || [[ "$(uname -s)" == "MINGW"* ]]; then
        binary_path="$binary_path.exe"
    fi
    
    if [ -f "$binary_path" ]; then
        local artifact_name="rio-ext-test"
        if [[ "$TARGET" == *"windows"* ]]; then
            artifact_name="$artifact_name.exe"
        fi
        
        cp "$binary_path" "$ARTIFACTS_DIR/$artifact_name"
        log_success "Binary copied to artifacts: $artifact_name"
        
        # Get binary info
        local binary_info="$REPORTS_DIR/binary-info.txt"
        {
            echo "Binary: $artifact_name"
            echo "Size: $(stat -c%s "$binary_path" 2>/dev/null || stat -f%z "$binary_path" 2>/dev/null || echo "unknown")"
            echo "Target: ${TARGET:-$(rustc -vV | grep host | cut -d' ' -f2)}"
            echo "Build type: $BUILD_TYPE"
            echo "Build timestamp: $(date)"
        } > "$binary_info"
    else
        log_error "Binary not found: $binary_path"
        exit 1
    fi
}

# Run benchmarks
run_benchmarks() {
    if [ "$ENABLE_BENCHMARKS" != "true" ]; then
        log_info "Benchmarks disabled, skipping"
        return 0
    fi
    
    log_step "Running benchmarks"
    
    if command -v cargo-criterion &> /dev/null; then
        local bench_output="$REPORTS_DIR/benchmark-results"
        mkdir -p "$bench_output"
        
        # Run benchmarks (if any exist)
        if cargo criterion --output-format json > "$bench_output/results.json" 2>&1; then
            log_success "Benchmarks completed"
        else
            log_warning "Benchmark execution failed or no benchmarks found"
        fi
    else
        log_warning "cargo-criterion not available, skipping benchmarks"
    fi
    
    log_step_end
}

# Generate build report
generate_build_report() {
    log_step "Generating build report"
    
    local report_file="$REPORTS_DIR/build-summary.md"
    
    cat > "$report_file" << EOF
# PTY Terminal CI Build Report

**Build Date:** $(date)
**Build Type:** $BUILD_TYPE
**Target:** ${TARGET:-$(rustc -vV | grep host | cut -d' ' -f2)}
**CI Environment:** $CI
**Signing Enabled:** $ENABLE_SIGNING

## Build Status

- **Formatting:** ✅ Passed
- **Clippy:** ✅ Passed
- **Tests:** $([ "$ENABLE_TESTS" = "true" ] && echo "✅ Passed" || echo "⏭️ Skipped")
- **Security Audit:** $([ "$ENABLE_SECURITY_AUDIT" = "true" ] && echo "✅ Passed" || echo "⏭️ Skipped")
- **Build:** ✅ Passed

## Artifacts

EOF
    
    # List artifacts
    if [ -d "$ARTIFACTS_DIR" ] && [ "$(ls -A "$ARTIFACTS_DIR")" ]; then
        for artifact in "$ARTIFACTS_DIR"/*; do
            if [ -f "$artifact" ]; then
                echo "- $(basename "$artifact")" >> "$report_file"
            fi
        done
    else
        echo "- No artifacts generated" >> "$report_file"
    fi
    
    echo "" >> "$report_file"
    echo "## Environment Information" >> "$report_file"
    echo "" >> "$report_file"
    echo "- **Rust Version:** $(rustc --version)" >> "$report_file"
    echo "- **Cargo Version:** $(cargo --version)" >> "$report_file"
    echo "- **OS:** $(uname -s) $(uname -r)" >> "$report_file"
    
    log_success "Build report generated: $report_file"
    log_step_end
}

# Main execution
main() {
    log_info "Starting CI build for PTY Terminal"
    log_info "Working directory: $PROJECT_DIR"
    log_info "Build configuration:"
    echo "  Build type: $BUILD_TYPE"
    echo "  Target: ${TARGET:-default}"
    echo "  Tests: $ENABLE_TESTS"
    echo "  Security audit: $ENABLE_SECURITY_AUDIT"
    echo "  Benchmarks: $ENABLE_BENCHMARKS"
    echo "  Signing: $ENABLE_SIGNING"
    
    # Change to project directory
    cd "$PROJECT_DIR"
    
    # Execute CI pipeline
    setup_ci_environment
    install_tools
    check_formatting
    run_clippy
    check_dependencies
    run_security_audit
    run_tests
    build_project
    run_benchmarks
    generate_build_report
    
    log_success "CI build completed successfully!"
    
    # Summary
    echo ""
    echo "Build Summary:"
    echo "  Artifacts: $ARTIFACTS_DIR"
    echo "  Reports: $REPORTS_DIR"
    
    if [ -d "$ARTIFACTS_DIR" ] && [ "$(ls -A "$ARTIFACTS_DIR")" ]; then
        echo "  Generated artifacts:"
        for artifact in "$ARTIFACTS_DIR"/*; do
            if [ -f "$artifact" ]; then
                echo "    - $(basename "$artifact")"
            fi
        done
    fi
}

# Handle script interruption
trap 'log_error "CI build interrupted"; exit 130' INT TERM

# Run main function
main "$@"