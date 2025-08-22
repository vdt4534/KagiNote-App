#!/bin/bash

# Simple Test Execution Script for KagiNote Speaker Diarization
# Compatible with older bash versions (macOS default)

set -e  # Exit on any error

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$SCRIPT_DIR/../../.."
REPORTS_DIR="$SCRIPT_DIR/reports"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
NC='\033[0m' # No Color

# Default configuration
VERBOSE=false
TIMEOUT=300

# Logging functions
log_header() {
    echo -e "\n${PURPLE}================================${NC}"
    echo -e "${PURPLE}$1${NC}"
    echo -e "${PURPLE}================================${NC}\n"
}

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

log_test() {
    echo -e "${BLUE}[TEST]${NC} $1"
}

# Check dependencies
check_dependencies() {
    log_header "Checking Dependencies"
    
    if ! command -v cargo >/dev/null 2>&1; then
        log_error "cargo not found - please install Rust"
        exit 1
    fi
    
    if [ ! -f "$PROJECT_ROOT/src-tauri/Cargo.toml" ]; then
        log_error "Cannot find Cargo.toml at $PROJECT_ROOT/src-tauri/"
        exit 1
    fi
    
    log_success "Dependencies verified"
}

# Generate test audio
generate_test_audio() {
    log_header "Generating Test Audio"
    
    log_info "Creating test directories..."
    mkdir -p "$SCRIPT_DIR/test_audio"
    mkdir -p "$SCRIPT_DIR/ground_truth"
    mkdir -p "$REPORTS_DIR"
    
    cd "$PROJECT_ROOT"
    
    log_info "Running audio generation test..."
    if timeout $TIMEOUT cargo test test_generate_all_test_audio --manifest-path src-tauri/Cargo.toml -- --nocapture > "$REPORTS_DIR/audio_generation.log" 2>&1; then
        log_success "Test audio generation completed"
        return 0
    else
        log_warning "Audio generation test failed or timed out"
        log_info "Check $REPORTS_DIR/audio_generation.log for details"
        return 1
    fi
}

# Run a single test category
run_test_category() {
    local category="$1"
    local description="$2"
    local test_filter="$3"
    
    log_test "Running $description"
    
    cd "$PROJECT_ROOT"
    local output_file="$REPORTS_DIR/${category}_output.log"
    
    local start_time=$(date +%s)
    local exit_code=0
    
    if [ "$VERBOSE" = true ]; then
        cargo_cmd="cargo test $test_filter --manifest-path src-tauri/Cargo.toml -- --nocapture"
    else
        cargo_cmd="cargo test $test_filter --manifest-path src-tauri/Cargo.toml"
    fi
    
    if timeout $TIMEOUT $cargo_cmd > "$output_file" 2>&1; then
        log_success "$description completed"
    else
        exit_code=$?
        if [ $exit_code -eq 124 ]; then
            log_error "$description timed out after ${TIMEOUT}s"
        else
            log_error "$description failed with exit code $exit_code"
        fi
    fi
    
    local end_time=$(date +%s)
    local duration=$((end_time - start_time))
    
    # Extract test counts
    local passed_count=0
    local failed_count=0
    
    if [ -f "$output_file" ]; then
        passed_count=$(grep -o "test result: ok\. [0-9]* passed" "$output_file" | grep -o "[0-9]*" | head -1 || echo "0")
        failed_count=$(grep -o "[0-9]* failed" "$output_file" | grep -o "^[0-9]*" | head -1 || echo "0")
    fi
    
    log_info "  Result: $passed_count passed, $failed_count failed (${duration}s)"
    
    # Return 0 if tests passed, 1 if they failed
    if [ $failed_count -eq 0 ] && [ $exit_code -eq 0 ]; then
        return 0
    else
        return 1
    fi
}

# Run core diarization tests
run_core_tests() {
    log_header "Running Core Diarization Tests"
    
    local total_tests=0
    local passed_tests=0
    
    # Test categories to run
    local test_configs=(
        "audio_generation:Audio Generation Tests:test_comprehensive_audio_scenarios"
        "scenario_validation:Scenario Validation:test_scenarios"
        "basic_functionality:Basic Functionality:test_diarization_test_runner"
    )
    
    for config in "${test_configs[@]}"; do
        IFS=':' read -r category description filter <<< "$config"
        total_tests=$((total_tests + 1))
        
        if run_test_category "$category" "$description" "$filter"; then
            passed_tests=$((passed_tests + 1))
        fi
    done
    
    log_info "Core tests completed: $passed_tests/$total_tests passed"
    
    if [ $passed_tests -eq $total_tests ]; then
        return 0
    else
        return 1
    fi
}

# Generate simple report
generate_report() {
    log_header "Generating Test Report"
    
    local report_file="$REPORTS_DIR/simple_test_report.html"
    local timestamp=$(date)
    
    cat > "$report_file" << EOF
<!DOCTYPE html>
<html>
<head>
    <title>KagiNote Diarization Test Report</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 40px; }
        .header { text-align: center; margin-bottom: 40px; }
        .log-section { margin: 20px 0; }
        .log-content { background: #f5f5f5; padding: 15px; border-radius: 5px; font-family: monospace; white-space: pre-wrap; }
    </style>
</head>
<body>
    <div class="header">
        <h1>KagiNote Diarization Test Report</h1>
        <p>Generated on $timestamp</p>
    </div>
    
    <div class="log-section">
        <h2>Test Execution Logs</h2>
EOF
    
    for log_file in "$REPORTS_DIR"/*.log; do
        if [ -f "$log_file" ]; then
            local log_name=$(basename "$log_file" .log)
            cat >> "$report_file" << EOF
        <h3>$log_name</h3>
        <div class="log-content">$(cat "$log_file" | head -100)</div>
EOF
        fi
    done
    
    cat >> "$report_file" << EOF
    </div>
</body>
</html>
EOF
    
    log_success "Report generated: $report_file"
}

# Show help
show_help() {
    echo "KagiNote Speaker Diarization Test Runner (Simple)"
    echo ""
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  --verbose              Enable verbose output"
    echo "  --timeout SECONDS      Set timeout per test (default: $TIMEOUT)"
    echo "  --help                 Show this help message"
    echo ""
    echo "This script runs basic diarization tests and generates simple reports."
    echo "It's compatible with older bash versions (macOS default)."
}

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --verbose)
            VERBOSE=true
            shift
            ;;
        --timeout)
            TIMEOUT="$2"
            shift 2
            ;;
        --help|-h)
            show_help
            exit 0
            ;;
        *)
            log_error "Unknown option: $1"
            show_help
            exit 1
            ;;
    esac
done

# Main execution
main() {
    log_header "🎵 KagiNote Speaker Diarization Test Suite (Simple)"
    log_info "Starting test execution with timeout=${TIMEOUT}s, verbose=${VERBOSE}"
    
    # Check environment
    check_dependencies
    
    # Generate test audio
    local audio_success=true
    if ! generate_test_audio; then
        audio_success=false
        log_warning "Audio generation failed, some tests may not work"
    fi
    
    # Run core tests
    local tests_success=true
    if ! run_core_tests; then
        tests_success=false
    fi
    
    # Generate report
    generate_report
    
    # Summary
    log_header "Test Execution Summary"
    
    if [ "$audio_success" = true ]; then
        echo "✅ Audio Generation: SUCCESS"
    else
        echo "❌ Audio Generation: FAILED"
    fi
    
    if [ "$tests_success" = true ]; then
        echo "✅ Core Tests: SUCCESS"
    else
        echo "❌ Core Tests: FAILED"
    fi
    
    echo ""
    echo "📁 Reports: $REPORTS_DIR"
    echo "🌐 HTML Report: $REPORTS_DIR/simple_test_report.html"
    
    # Exit with appropriate code
    if [ "$tests_success" = true ]; then
        log_success "🎉 All tests completed successfully!"
        exit 0
    else
        log_error "❌ Some tests failed"
        exit 1
    fi
}

# Execute main function
main "$@"