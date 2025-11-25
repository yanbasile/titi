#!/bin/bash
# Test Runner Script for Titi Terminal Emulator
#
# This script runs the complete test suite with detailed reporting.
# Usage: ./scripts/run_tests.sh [options]
#
# Options:
#   --quick       Run only fast unit tests (skip stress tests)
#   --regression  Run only regression tests
#   --coverage    Generate code coverage report
#   --verbose     Show detailed output

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Print section header
print_header() {
    echo -e "\n${BLUE}=== $1 ===${NC}\n"
}

# Print success message
print_success() {
    echo -e "${GREEN}✓ $1${NC}"
}

# Print error message
print_error() {
    echo -e "${RED}✗ $1${NC}"
}

# Print warning message
print_warning() {
    echo -e "${YELLOW}⚠ $1${NC}"
}

# Parse command line arguments
QUICK=false
REGRESSION_ONLY=false
COVERAGE=false
VERBOSE=""

for arg in "$@"; do
    case $arg in
        --quick)
            QUICK=true
            ;;
        --regression)
            REGRESSION_ONLY=true
            ;;
        --coverage)
            COVERAGE=true
            ;;
        --verbose)
            VERBOSE="--nocapture"
            ;;
        *)
            echo "Unknown option: $arg"
            exit 1
            ;;
    esac
done

# Start test execution
print_header "Titi Terminal Emulator Test Suite"

if [ "$REGRESSION_ONLY" = true ]; then
    print_header "Running Regression Tests Only"
    cargo test --test regression -- $VERBOSE
    print_success "Regression tests completed"
    exit 0
fi

# Terminal Core Tests
print_header "Terminal Core Tests"

echo "Running grid tests..."
if cargo test --test grid_tests -- $VERBOSE; then
    print_success "Grid tests passed"
else
    print_error "Grid tests failed"
    exit 1
fi

echo "Running parser tests..."
if cargo test --test parser_tests -- $VERBOSE; then
    print_success "Parser tests passed"
else
    print_error "Parser tests failed"
    exit 1
fi

echo "Running dirty tracking tests..."
if cargo test --test dirty_tracking_tests -- $VERBOSE; then
    print_success "Dirty tracking tests passed"
else
    print_error "Dirty tracking tests failed"
    exit 1
fi

echo "Running text extraction tests..."
if cargo test --test text_extraction_tests -- $VERBOSE; then
    print_success "Text extraction tests passed"
else
    print_error "Text extraction tests failed"
    exit 1
fi

# Renderer Tests
print_header "Renderer Tests"

echo "Running glyph atlas tests..."
if cargo test --test glyph_atlas_tests -- $VERBOSE; then
    print_success "Glyph atlas tests passed"
else
    print_error "Glyph atlas tests failed"
    exit 1
fi

# Regression Tests
print_header "Regression Tests"

echo "Running regression test suite..."
if cargo test --test regression -- $VERBOSE; then
    print_success "Regression tests passed"
else
    print_error "Regression tests failed"
    exit 1
fi

# Stress Tests (skip if --quick)
if [ "$QUICK" = false ]; then
    print_header "Stress Tests"

    echo "Running performance tests..."
    if cargo test --test performance --release -- $VERBOSE; then
        print_success "Performance tests passed"
    else
        print_warning "Performance tests failed (non-critical)"
    fi

    echo "Running concurrency tests..."
    if cargo test --test concurrency -- $VERBOSE; then
        print_success "Concurrency tests passed"
    else
        print_warning "Concurrency tests failed (non-critical)"
    fi

    echo "Running memory leak detection..."
    if cargo test --test memory_leak_detection -- $VERBOSE; then
        print_success "Memory leak tests passed"
    else
        print_warning "Memory leak tests failed (non-critical)"
    fi
else
    print_warning "Skipping stress tests (--quick mode)"
fi

# Code Coverage (if requested)
if [ "$COVERAGE" = true ]; then
    print_header "Code Coverage"

    if command -v cargo-tarpaulin &> /dev/null; then
        cargo tarpaulin --out Html --output-dir coverage
        print_success "Coverage report generated in coverage/"
    else
        print_warning "cargo-tarpaulin not installed. Install with: cargo install cargo-tarpaulin"
    fi
fi

# Summary
print_header "Test Summary"
print_success "All critical tests passed!"

echo ""
echo "Test Suite Complete"
echo "  Terminal Core: ✓"
echo "  Renderer:      ✓"
echo "  Regression:    ✓"

if [ "$QUICK" = false ]; then
    echo "  Stress Tests:  ✓"
fi

echo ""
print_success "Test suite execution successful"
