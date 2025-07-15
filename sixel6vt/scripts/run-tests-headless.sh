#!/bin/bash
# Headless Test Runner for PTY Terminal
# Runs tests in a headless environment suitable for CI/containers

set -euo pipefail

# Configuration
DISPLAY="${DISPLAY:-:99}"
XVFB_RESOLUTION="${XVFB_RESOLUTION:-1920x1080x24}"
TEST_TIMEOUT="${TEST_TIMEOUT:-300}"
RUST_LOG="${RUST_LOG:-info}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

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

# Check if running in headless environment
is_headless() {
    if [ -n "${CI:-}" ] || [ -n "${GITHUB_ACTIONS:-}" ] || [ -z "${DISPLAY:-}" ]; then
        return 0
    fi
    return 1
}

# Setup virtual display
setup_virtual_display() {
    if ! is_headless; then
        log_info "Display available, skipping virtual display setup"
        return 0
    fi
    
    log_info "Setting up virtual display for headless testing"
    
    # Check if Xvfb is available
    if ! command -v Xvfb &> /dev/null; then
        log_error "Xvfb not found. Install with: apt-get install xvfb"
        exit 1
    fi
    
    # Start Xvfb
    log_info "Starting Xvfb on display $DISPLAY with resolution $XVFB_RESOLUTION"
    Xvfb "$DISPLAY" -screen 0 "$XVFB_RESOLUTION" -ac +extension GLX +render -noreset &
    local xvfb_pid=$!
    
    # Wait for Xvfb to start
    sleep 2
    
    # Check if Xvfb is running
    if ! kill -0 $xvfb_pid 2>/dev/null; then
        log_error "Failed to start Xvfb"
        exit 1
    fi
    
    export DISPLAY
    export XVFB_PID=$xvfb_pid
    
    log_success "Virtual display ready on $DISPLAY"
}

# Cleanup virtual display
cleanup_virtual_display() {
    if [ -n "${XVFB_PID:-}" ]; then
        log_info "Cleaning up virtual display"
        kill "$XVFB_PID" 2>/dev/null || true
        wait "$XVFB_PID" 2>/dev/null || true
    fi
}

# Setup test environment
setup_test_environment() {
    log_info "Setting up test environment"
    
    # Set environment variables for testing
    export RUST_LOG="$RUST_LOG"
    export RUST_BACKTRACE=1
    export CARGO_TERM_COLOR=always
    
    # Test-specific environment
    export TEST_MODE=1
    export HEADLESS_MODE=1
    export NO_GPU=1  # Disable GPU acceleration for headless tests
    
    # Timeout for tests
    export TEST_TIMEOUT="$TEST_TIMEOUT"
    
    log_success "Test environment configured"
}

# Run unit tests
run_unit_tests() {
    log_info "Running unit tests"
    
    local test_args=(
        "test"
        "--lib"
        "--bins"
        "--verbose"
        "--"
        "--test-threads=1"
        "--nocapture"
    )
    
    if timeout "$TEST_TIMEOUT" cargo "${test_args[@]}"; then
        log_success "Unit tests passed"
        return 0
    else
        local exit_code=$?
        if [ $exit_code -eq 124 ]; then
            log_error "Unit tests timed out after $TEST_TIMEOUT seconds"
        else
            log_error "Unit tests failed with exit code $exit_code"
        fi
        return $exit_code
    fi
}

# Run integration tests
run_integration_tests() {
    log_info "Running integration tests"
    
    local test_args=(
        "test"
        "--test"
        "*"
        "--verbose"
        "--"
        "--test-threads=1"
        "--nocapture"
    )
    
    if timeout "$TEST_TIMEOUT" cargo "${test_args[@]}"; then
        log_success "Integration tests passed"
        return 0
    else
        local exit_code=$?
        if [ $exit_code -eq 124 ]; then
            log_error "Integration tests timed out after $TEST_TIMEOUT seconds"
        else
            log_error "Integration tests failed with exit code $exit_code"
        fi
        return $exit_code
    fi
}

# Run doctests
run_doctests() {
    log_info "Running documentation tests"
    
    if timeout "$TEST_TIMEOUT" cargo test --doc --verbose; then
        log_success "Documentation tests passed"
        return 0
    else
        local exit_code=$?
        if [ $exit_code -eq 124 ]; then
            log_error "Documentation tests timed out after $TEST_TIMEOUT seconds"
        else
            log_error "Documentation tests failed with exit code $exit_code"
        fi
        return $exit_code
    fi
}

# Run GUI tests (if any)
run_gui_tests() {
    log_info "Checking for GUI tests"
    
    # Look for GUI-specific test files
    local gui_tests=()
    while IFS= read -r -d '' file; do
        gui_tests+=("$file")
    done < <(find tests -name "*gui*" -o -name "*ui*" -o -name "*visual*" 2>/dev/null | head -10)
    
    if [ ${#gui_tests[@]} -eq 0 ]; then
        log_info "No GUI tests found, skipping"
        return 0
    fi
    
    log_info "Found ${#gui_tests[@]} GUI test file(s)"
    
    # Run GUI tests with virtual display
    local test_args=(
        "test"
        "--verbose"
        "--"
        "--test-threads=1"
        "--nocapture"
    )
    
    # Add specific GUI test patterns
    for test_file in "${gui_tests[@]}"; do
        test_args+=("$(basename "$test_file" .rs)")
    done
    
    if timeout "$TEST_TIMEOUT" cargo "${test_args[@]}"; then
        log_success "GUI tests passed"
        return 0
    else
        local exit_code=$?
        if [ $exit_code -eq 124 ]; then
            log_error "GUI tests timed out after $TEST_TIMEOUT seconds"
        else
            log_error "GUI tests failed with exit code $exit_code"
        fi
        return $exit_code
    fi
}

# Run specific test category
run_test_category() {
    local category="$1"
    
    case "$category" in
        unit)
            run_unit_tests
            ;;
        integration)
            run_integration_tests
            ;;
        doc)
            run_doctests
            ;;
        gui)
            run_gui_tests
            ;;
        all)
            run_unit_tests && \
            run_integration_tests && \
            run_doctests && \
            run_gui_tests
            ;;
        *)
            log_error "Unknown test category: $category"
            exit 1
            ;;
    esac
}

# Generate test report
generate_test_report() {
    log_info "Generating test report"
    
    local report_dir="test-reports"
    mkdir -p "$report_dir"
    
    local report_file="$report_dir/headless-test-report.txt"
    
    {
        echo "PTY Terminal Headless Test Report"
        echo "================================="
        echo ""
        echo "Test Date: $(date)"
        echo "Display: $DISPLAY"
        echo "Resolution: $XVFB_RESOLUTION"
        echo "Timeout: $TEST_TIMEOUT seconds"
        echo "Environment: Headless"
        echo ""
        echo "Test Results:"
        echo "- Unit Tests: $([ $unit_result -eq 0 ] && echo "PASSED" || echo "FAILED")"
        echo "- Integration Tests: $([ $integration_result -eq 0 ] && echo "PASSED" || echo "FAILED")"
        echo "- Documentation Tests: $([ $doc_result -eq 0 ] && echo "PASSED" || echo "FAILED")"
        echo "- GUI Tests: $([ $gui_result -eq 0 ] && echo "PASSED" || echo "FAILED")"
        echo ""
        echo "Overall Status: $([ $overall_result -eq 0 ] && echo "PASSED" || echo "FAILED")"
    } > "$report_file"
    
    log_success "Test report generated: $report_file"
}

# Main execution
main() {
    local test_category="${1:-all}"
    
    log_info "Starting headless test runner for PTY Terminal"
    log_info "Test category: $test_category"
    log_info "Display: $DISPLAY"
    log_info "Timeout: $TEST_TIMEOUT seconds"
    
    # Trap cleanup
    trap cleanup_virtual_display EXIT
    
    # Setup
    setup_virtual_display
    setup_test_environment
    
    # Run tests
    local exit_code=0
    
    if [ "$test_category" = "all" ]; then
        # Run all test categories and track results
        local unit_result=0
        local integration_result=0
        local doc_result=0
        local gui_result=0
        local overall_result=0
        
        run_unit_tests || unit_result=$?
        run_integration_tests || integration_result=$?
        run_doctests || doc_result=$?
        run_gui_tests || gui_result=$?
        
        # Calculate overall result
        if [ $unit_result -ne 0 ] || [ $integration_result -ne 0 ] || [ $doc_result -ne 0 ] || [ $gui_result -ne 0 ]; then
            overall_result=1
        fi
        
        # Generate report
        generate_test_report
        
        exit_code=$overall_result
    else
        # Run specific category
        run_test_category "$test_category"
        exit_code=$?
    fi
    
    if [ $exit_code -eq 0 ]; then
        log_success "All tests completed successfully!"
    else
        log_error "Some tests failed"
    fi
    
    exit $exit_code
}

# Usage information
usage() {
    cat << EOF
Headless Test Runner for PTY Terminal

Usage: $0 [CATEGORY]

CATEGORIES:
    unit         Run unit tests only
    integration  Run integration tests only
    doc          Run documentation tests only
    gui          Run GUI tests only
    all          Run all tests (default)

ENVIRONMENT VARIABLES:
    DISPLAY              X11 display (default: :99)
    XVFB_RESOLUTION     Virtual display resolution (default: 1920x1080x24)
    TEST_TIMEOUT        Test timeout in seconds (default: 300)
    RUST_LOG            Rust logging level (default: info)

EXAMPLES:
    $0                  # Run all tests
    $0 unit            # Run only unit tests
    $0 gui             # Run only GUI tests
    
    # With custom settings:
    DISPLAY=:1 TEST_TIMEOUT=600 $0 all

EOF
}

# Handle help
if [ "${1:-}" = "--help" ] || [ "${1:-}" = "-h" ]; then
    usage
    exit 0
fi

# Run main function
main "$@"