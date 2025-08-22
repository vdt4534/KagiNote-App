#!/bin/bash

# Comprehensive Test Execution Script for KagiNote Speaker Diarization
# This script runs all diarization tests and generates detailed reports

set -e  # Exit on any error

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$SCRIPT_DIR/../../.."
REPORTS_DIR="$SCRIPT_DIR/reports"
TEST_AUDIO_DIR="$SCRIPT_DIR/test_audio"
GROUND_TRUTH_DIR="$SCRIPT_DIR/ground_truth"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Test configuration
DEFAULT_TIMEOUT=300  # 5 minutes per test category
VERBOSE=${VERBOSE:-false}
GENERATE_REPORTS=${GENERATE_REPORTS:-true}
RUN_BENCHMARKS=${RUN_BENCHMARKS:-false}
RUN_STRESS_TESTS=${RUN_STRESS_TESTS:-false}
MAX_PARALLEL_JOBS=${MAX_PARALLEL_JOBS:-4}

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
    echo -e "${CYAN}[TEST]${NC} $1"
}

# Test result tracking (using bash 3.x compatible arrays)
# We'll use indexed arrays and track keys separately
test_categories=()
test_results=()
test_durations=()
test_details=()

# Initialize test environment
init_test_environment() {
    log_header "Initializing Test Environment"
    
    # Create reports directory
    mkdir -p "$REPORTS_DIR"
    
    # Store test run metadata
    local timestamp=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
    cat > "$REPORTS_DIR/test_run_metadata.json" << EOF
{
  "timestamp": "$timestamp",
  "script_version": "1.0.0",
  "project_root": "$PROJECT_ROOT",
  "test_configuration": {
    "timeout": $DEFAULT_TIMEOUT,
    "verbose": $VERBOSE,
    "generate_reports": $GENERATE_REPORTS,
    "run_benchmarks": $RUN_BENCHMARKS,
    "run_stress_tests": $RUN_STRESS_TESTS,
    "max_parallel_jobs": $MAX_PARALLEL_JOBS
  },
  "environment": {
    "os": "$(uname -s)",
    "arch": "$(uname -m)",
    "rust_version": "$(rustc --version 2>/dev/null || echo 'unknown')",
    "cargo_version": "$(cargo --version 2>/dev/null || echo 'unknown')"
  }
}
EOF
    
    log_success "Test environment initialized"
    log_info "Reports will be saved to: $REPORTS_DIR"
}

# Check system dependencies
check_dependencies() {
    log_header "Checking Dependencies"
    
    local missing_deps=()
    
    # Check Rust/Cargo
    if ! command -v cargo >/dev/null 2>&1; then
        missing_deps+=("cargo")
    fi
    
    if ! command -v rustc >/dev/null 2>&1; then
        missing_deps+=("rustc")
    fi
    
    # Check optional tools
    if ! command -v jq >/dev/null 2>&1; then
        log_warning "jq not found - JSON report processing will be limited"
    fi
    
    if ! command -v bc >/dev/null 2>&1; then
        log_warning "bc not found - some calculations may be approximate"
    fi
    
    if [ ${#missing_deps[@]} -gt 0 ]; then
        log_error "Missing required dependencies: ${missing_deps[*]}"
        log_error "Please install missing dependencies and try again"
        exit 1
    fi
    
    # Check project structure
    if [ ! -f "$PROJECT_ROOT/src-tauri/Cargo.toml" ]; then
        log_error "Cannot find Cargo.toml at $PROJECT_ROOT/src-tauri/"
        log_error "Please run this script from the correct directory"
        exit 1
    fi
    
    log_success "All dependencies verified"
}

# Prepare test data
prepare_test_data() {
    log_header "Preparing Test Data"
    
    # Check if test data exists
    if [ ! -d "$TEST_AUDIO_DIR" ] || [ -z "$(ls -A "$TEST_AUDIO_DIR" 2>/dev/null)" ]; then
        log_info "Test audio not found, attempting to generate..."
        
        # Try to run the download script
        if [ -f "$SCRIPT_DIR/download_test_data.sh" ]; then
            log_info "Running download_test_data.sh..."
            if "$SCRIPT_DIR/download_test_data.sh" --synthetic-only; then
                log_success "Test data preparation initiated"
            else
                log_warning "Download script failed, continuing with synthetic generation"
            fi
        fi
        
        # Generate synthetic audio via Rust tests
        log_info "Generating synthetic test audio..."
        cd "$PROJECT_ROOT"
        
        if timeout $DEFAULT_TIMEOUT cargo test --manifest-path src-tauri/Cargo.toml test_generate_all_test_audio -- --nocapture; then
            log_success "Synthetic test audio generated successfully"
        else
            log_warning "Synthetic audio generation failed or timed out"
            log_info "Some tests may fail due to missing test data"
        fi
    else
        log_success "Test data already available"
        local audio_count=$(find "$TEST_AUDIO_DIR" -name "*.wav" | wc -l)
        local ground_truth_count=$(find "$GROUND_TRUTH_DIR" -name "*.json" | wc -l)
        log_info "Found $audio_count audio files and $ground_truth_count ground truth files"
    fi
}

# Run a single test category with timeout and result tracking
run_test_category() {
    local category="$1"
    local description="$2"
    local cargo_filter="$3"
    local timeout="${4:-$DEFAULT_TIMEOUT}"
    
    log_test "Running $description"
    
    local start_time=$(date +%s)
    local output_file="$REPORTS_DIR/${category}_output.log"
    local result_file="$REPORTS_DIR/${category}_result.json"
    
    cd "$PROJECT_ROOT"
    
    # Prepare cargo command
    local cargo_cmd="cargo test --manifest-path src-tauri/Cargo.toml $cargo_filter"
    if [ "$VERBOSE" = true ]; then
        cargo_cmd="$cargo_cmd -- --nocapture"
    fi
    
    # Run test with timeout
    local exit_code=0
    if timeout "$timeout" bash -c "$cargo_cmd" > "$output_file" 2>&1; then
        test_results["$category"]="PASSED"
        log_success "$description completed successfully"
    else
        exit_code=$?
        if [ $exit_code -eq 124 ]; then
            test_results["$category"]="TIMEOUT"
            log_error "$description timed out after ${timeout}s"
        else
            test_results["$category"]="FAILED"
            log_error "$description failed with exit code $exit_code"
        fi
    fi
    
    local end_time=$(date +%s)
    local duration=$((end_time - start_time))
    test_durations["$category"]=$duration
    
    # Extract test details from output
    local passed_count=0
    local failed_count=0
    local ignored_count=0
    
    if [ -f "$output_file" ]; then
        # Parse cargo test output
        passed_count=$(grep -o "test result: ok\. [0-9]* passed" "$output_file" | grep -o "[0-9]*" | head -1 || echo "0")
        failed_count=$(grep -o "[0-9]* failed" "$output_file" | grep -o "^[0-9]*" | head -1 || echo "0")
        ignored_count=$(grep -o "[0-9]* ignored" "$output_file" | grep -o "^[0-9]*" | head -1 || echo "0")
    fi
    
    # Store detailed results
    cat > "$result_file" << EOF
{
  "category": "$category",
  "description": "$description",
  "status": "${test_results[$category]}",
  "duration_seconds": $duration,
  "test_counts": {
    "passed": $passed_count,
    "failed": $failed_count,
    "ignored": $ignored_count,
    "total": $((passed_count + failed_count + ignored_count))
  },
  "timestamp": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")",
  "timeout_seconds": $timeout,
  "exit_code": $exit_code
}
EOF
    
    test_details["$category"]="$passed_count passed, $failed_count failed, $ignored_count ignored in ${duration}s"
    
    # Show summary for this category
    log_info "  Result: ${test_results[$category]} ($passed_count passed, $failed_count failed, ${duration}s)"
}

# Run all test categories
run_all_tests() {
    log_header "Running All Test Categories"
    
    # Core functionality tests
    run_test_category "unit" "Unit Tests" "diarization_realtime::mod"
    run_test_category "integration" "Integration Tests" "diarization_realtime::integration_tests"
    run_test_category "accuracy" "Accuracy Tests" "diarization_realtime::accuracy_tests"
    run_test_category "performance" "Performance Tests" "diarization_realtime::performance_tests"
    run_test_category "memory" "Memory Tests" "diarization_realtime::memory_tests"
    
    # Audio generation and validation tests
    run_test_category "audio_generation" "Audio Generation Tests" "create_test_audio"
    run_test_category "validation" "Validation Framework Tests" "diarization_realtime::validation"
    
    # Conditional tests
    if [ "$RUN_BENCHMARKS" = true ]; then
        run_test_category "benchmarks" "Benchmark Tests" "diarization_realtime::benchmark" 600
    fi
    
    if [ "$RUN_STRESS_TESTS" = true ]; then
        run_test_category "stress" "Stress Tests" "diarization_realtime::stress_tests" 900
    fi
    
    # Model and pipeline tests
    run_test_category "models" "Model Tests" "diarization_model_tests"
    run_test_category "pipeline" "Pipeline Tests" "speaker_diarization_pipeline"
}

# Generate comprehensive HTML report
generate_html_report() {
    log_header "Generating Comprehensive Test Report"
    
    local report_file="$REPORTS_DIR/comprehensive_test_report.html"
    local timestamp=$(date)
    
    # Calculate summary statistics
    local total_tests=0
    local passed_tests=0
    local failed_tests=0
    local timeout_tests=0
    local total_duration=0
    
    for category in "${!test_results[@]}"; do
        total_tests=$((total_tests + 1))
        case "${test_results[$category]}" in
            "PASSED") passed_tests=$((passed_tests + 1)) ;;
            "FAILED") failed_tests=$((failed_tests + 1)) ;;
            "TIMEOUT") timeout_tests=$((timeout_tests + 1)) ;;
        esac
        total_duration=$((total_duration + ${test_durations[$category]}))
    done
    
    local success_rate=0
    if [ $total_tests -gt 0 ]; then
        success_rate=$(echo "scale=1; $passed_tests * 100 / $total_tests" | bc -l 2>/dev/null || echo "0")
    fi
    
    # Generate HTML report
    cat > "$report_file" << EOF
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>KagiNote Diarization Test Report</title>
    <style>
        body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; margin: 40px; background: #f8f9fa; }
        .container { max-width: 1200px; margin: 0 auto; background: white; padding: 40px; border-radius: 8px; box-shadow: 0 2px 10px rgba(0,0,0,0.1); }
        .header { text-align: center; margin-bottom: 40px; }
        .header h1 { color: #2563eb; margin-bottom: 10px; }
        .summary { display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 20px; margin-bottom: 40px; }
        .summary-card { background: #f8f9fa; padding: 20px; border-radius: 6px; text-align: center; }
        .summary-card h3 { margin: 0 0 10px 0; color: #6b7280; font-size: 14px; text-transform: uppercase; }
        .summary-card .value { font-size: 24px; font-weight: bold; margin: 0; }
        .passed { color: #059669; }
        .failed { color: #dc2626; }
        .timeout { color: #d97706; }
        .test-results { margin-bottom: 40px; }
        .test-category { background: #f8f9fa; margin-bottom: 20px; border-radius: 6px; overflow: hidden; }
        .test-category-header { background: #e5e7eb; padding: 15px; font-weight: bold; }
        .test-category-content { padding: 20px; }
        .status-badge { padding: 4px 8px; border-radius: 4px; font-size: 12px; font-weight: bold; text-transform: uppercase; }
        .status-passed { background: #dcfce7; color: #166534; }
        .status-failed { background: #fee2e2; color: #991b1b; }
        .status-timeout { background: #fef3c7; color: #92400e; }
        .details-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(150px, 1fr)); gap: 15px; margin-top: 15px; }
        .detail-item { background: white; padding: 10px; border-radius: 4px; }
        .detail-item strong { display: block; color: #6b7280; font-size: 12px; margin-bottom: 5px; }
        .footer { text-align: center; color: #6b7280; font-size: 14px; margin-top: 40px; }
        .progress-bar { background: #e5e7eb; height: 8px; border-radius: 4px; overflow: hidden; margin: 10px 0; }
        .progress-fill { height: 100%; background: linear-gradient(90deg, #059669, #34d399); transition: width 0.3s ease; }
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>🎵 KagiNote Speaker Diarization Test Report</h1>
            <p>Generated on $timestamp</p>
            <div class="progress-bar">
                <div class="progress-fill" style="width: ${success_rate}%"></div>
            </div>
            <p><strong>${success_rate}% Success Rate</strong></p>
        </div>
        
        <div class="summary">
            <div class="summary-card">
                <h3>Total Tests</h3>
                <p class="value">$total_tests</p>
            </div>
            <div class="summary-card">
                <h3>Passed</h3>
                <p class="value passed">$passed_tests</p>
            </div>
            <div class="summary-card">
                <h3>Failed</h3>
                <p class="value failed">$failed_tests</p>
            </div>
            <div class="summary-card">
                <h3>Timeouts</h3>
                <p class="value timeout">$timeout_tests</p>
            </div>
            <div class="summary-card">
                <h3>Total Duration</h3>
                <p class="value">${total_duration}s</p>
            </div>
        </div>
        
        <div class="test-results">
            <h2>Test Category Results</h2>
EOF
    
    # Add each test category to the report
    for category in $(echo "${!test_results[@]}" | tr ' ' '\n' | sort); do
        local status="${test_results[$category]}"
        local duration="${test_durations[$category]}"
        local details="${test_details[$category]}"
        
        local status_class=""
        case "$status" in
            "PASSED") status_class="status-passed" ;;
            "FAILED") status_class="status-failed" ;;
            "TIMEOUT") status_class="status-timeout" ;;
        esac
        
        cat >> "$report_file" << EOF
            <div class="test-category">
                <div class="test-category-header">
                    $category
                    <span class="status-badge $status_class">$status</span>
                </div>
                <div class="test-category-content">
                    <p><strong>Details:</strong> $details</p>
                    <div class="details-grid">
                        <div class="detail-item">
                            <strong>Duration</strong>
                            ${duration}s
                        </div>
                        <div class="detail-item">
                            <strong>Status</strong>
                            $status
                        </div>
                    </div>
                </div>
            </div>
EOF
    done
    
    cat >> "$report_file" << EOF
        </div>
        
        <div class="footer">
            <p>Report generated by KagiNote test infrastructure</p>
            <p>For detailed logs, check the individual files in the reports directory</p>
        </div>
    </div>
</body>
</html>
EOF
    
    log_success "HTML report generated: $report_file"
}

# Generate CI/CD compatible report
generate_ci_report() {
    local ci_report="$REPORTS_DIR/ci_report.json"
    local junit_report="$REPORTS_DIR/junit_report.xml"
    
    # JSON report for CI systems
    cat > "$ci_report" << EOF
{
  "summary": {
    "total": ${#test_results[@]},
    "passed": $(echo "${test_results[@]}" | tr ' ' '\n' | grep -c "PASSED" || echo 0),
    "failed": $(echo "${test_results[@]}" | tr ' ' '\n' | grep -c "FAILED" || echo 0),
    "timeout": $(echo "${test_results[@]}" | tr ' ' '\n' | grep -c "TIMEOUT" || echo 0),
    "success_rate": $(echo "scale=2; $(echo "${test_results[@]}" | tr ' ' '\n' | grep -c "PASSED" || echo 0) * 100 / ${#test_results[@]}" | bc -l 2>/dev/null || echo "0")
  },
  "categories": {
EOF
    
    local first=true
    for category in "${!test_results[@]}"; do
        if [ "$first" = false ]; then
            echo "," >> "$ci_report"
        fi
        first=false
        
        cat >> "$ci_report" << EOF
    "$category": {
      "status": "${test_results[$category]}",
      "duration": ${test_durations[$category]}
    }EOF
    done
    
    echo -e "\n  }\n}" >> "$ci_report"
    
    # JUnit XML report
    cat > "$junit_report" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<testsuites name="KagiNote Diarization Tests" tests="${#test_results[@]}" failures="$(echo "${test_results[@]}" | tr ' ' '\n' | grep -c "FAILED" || echo 0)" time="$total_duration">
EOF
    
    for category in "${!test_results[@]}"; do
        local status="${test_results[$category]}"
        local duration="${test_durations[$category]}"
        
        cat >> "$junit_report" << EOF
  <testsuite name="$category" tests="1" failures="$([ "$status" != "PASSED" ] && echo 1 || echo 0)" time="$duration">
    <testcase name="$category" time="$duration">
EOF
        
        if [ "$status" != "PASSED" ]; then
            cat >> "$junit_report" << EOF
      <failure message="Test $status">$status after ${duration}s</failure>
EOF
        fi
        
        cat >> "$junit_report" << EOF
    </testcase>
  </testsuite>
EOF
    done
    
    echo "</testsuites>" >> "$junit_report"
    
    log_success "CI reports generated: $ci_report, $junit_report"
}

# Clean up temporary files and optimize reports
cleanup_and_optimize() {
    log_header "Cleanup and Optimization"
    
    # Compress large log files
    for log_file in "$REPORTS_DIR"/*.log; do
        if [ -f "$log_file" ] && [ $(stat -f%z "$log_file" 2>/dev/null || stat -c%s "$log_file" 2>/dev/null || echo 0) -gt 1048576 ]; then
            log_info "Compressing large log file: $(basename "$log_file")"
            gzip "$log_file" 2>/dev/null || true
        fi
    done
    
    # Create archive of all reports
    local archive_name="diarization_test_results_$(date +%Y%m%d_%H%M%S).tar.gz"
    cd "$SCRIPT_DIR"
    tar -czf "$archive_name" reports/ 2>/dev/null || true
    
    if [ -f "$archive_name" ]; then
        log_success "Test results archived: $archive_name"
    fi
    
    log_success "Cleanup completed"
}

# Display final summary
display_summary() {
    log_header "Test Execution Summary"
    
    local total_tests=${#test_results[@]}
    local passed_tests=0
    local failed_tests=0
    local timeout_tests=0
    
    for category in "${!test_results[@]}"; do
        case "${test_results[$category]}" in
            "PASSED") passed_tests=$((passed_tests + 1)) ;;
            "FAILED") failed_tests=$((failed_tests + 1)) ;;
            "TIMEOUT") timeout_tests=$((timeout_tests + 1)) ;;
        esac
    done
    
    echo "📊 Overall Results:"
    echo "  Total Categories: $total_tests"
    echo "  ✅ Passed: $passed_tests"
    echo "  ❌ Failed: $failed_tests"
    echo "  ⏱️  Timeout: $timeout_tests"
    
    if [ $total_tests -gt 0 ]; then
        local success_rate=$(echo "scale=1; $passed_tests * 100 / $total_tests" | bc -l 2>/dev/null || echo "0")
        echo "  📈 Success Rate: ${success_rate}%"
    fi
    
    echo ""
    echo "📝 Detailed Results:"
    for category in $(echo "${!test_results[@]}" | tr ' ' '\n' | sort); do
        local status="${test_results[$category]}"
        local details="${test_details[$category]}"
        local status_icon=""
        
        case "$status" in
            "PASSED") status_icon="✅" ;;
            "FAILED") status_icon="❌" ;;
            "TIMEOUT") status_icon="⏱️" ;;
        esac
        
        echo "  $status_icon $category: $details"
    done
    
    echo ""
    echo "📁 Reports Location: $REPORTS_DIR"
    echo "🌐 HTML Report: $REPORTS_DIR/comprehensive_test_report.html"
    
    # Final recommendations
    if [ $failed_tests -gt 0 ] || [ $timeout_tests -gt 0 ]; then
        echo ""
        log_warning "Some tests failed or timed out. Recommendations:"
        echo "  1. Check individual log files in $REPORTS_DIR"
        echo "  2. Ensure test data is properly generated"
        echo "  3. Verify system resources (memory, CPU)"
        echo "  4. Run with --verbose for more details"
    else
        echo ""
        log_success "🎉 All tests passed successfully!"
        echo "  Your speaker diarization implementation is working correctly."
    fi
}

# Help function
show_help() {
    echo "KagiNote Speaker Diarization Test Runner"
    echo ""
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  --verbose              Enable verbose output"
    echo "  --no-reports           Skip report generation"
    echo "  --benchmarks           Include benchmark tests (slower)"
    echo "  --stress-tests         Include stress tests (much slower)"
    echo "  --timeout SECONDS      Set timeout for each test category (default: $DEFAULT_TIMEOUT)"
    echo "  --parallel-jobs NUM    Set max parallel jobs (default: $MAX_PARALLEL_JOBS)"
    echo "  --help                 Show this help message"
    echo ""
    echo "Environment Variables:"
    echo "  VERBOSE=true           Enable verbose output"
    echo "  GENERATE_REPORTS=false Disable report generation"
    echo "  RUN_BENCHMARKS=true    Include benchmark tests"
    echo "  RUN_STRESS_TESTS=true  Include stress tests"
    echo ""
    echo "Examples:"
    echo "  $0                           # Run standard tests"
    echo "  $0 --verbose --benchmarks    # Run with benchmarks and verbose output"
    echo "  $0 --timeout 600            # Use 10-minute timeout per category"
    echo "  VERBOSE=true $0              # Use environment variable"
}

# Parse command line arguments
parse_arguments() {
    while [[ $# -gt 0 ]]; do
        case $1 in
            --verbose)
                VERBOSE=true
                shift
                ;;
            --no-reports)
                GENERATE_REPORTS=false
                shift
                ;;
            --benchmarks)
                RUN_BENCHMARKS=true
                shift
                ;;
            --stress-tests)
                RUN_STRESS_TESTS=true
                shift
                ;;
            --timeout)
                DEFAULT_TIMEOUT="$2"
                shift 2
                ;;
            --parallel-jobs)
                MAX_PARALLEL_JOBS="$2"
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
}

# Main execution function
main() {
    # Parse arguments
    parse_arguments "$@"
    
    # Start test execution
    log_header "🎵 KagiNote Speaker Diarization Test Suite"
    log_info "Starting comprehensive test execution..."
    log_info "Configuration: verbose=$VERBOSE, reports=$GENERATE_REPORTS, benchmarks=$RUN_BENCHMARKS, stress=$RUN_STRESS_TESTS"
    
    # Initialize and prepare
    init_test_environment
    check_dependencies
    prepare_test_data
    
    # Run all tests
    local start_time=$(date +%s)
    run_all_tests
    local end_time=$(date +%s)
    local total_execution_time=$((end_time - start_time))
    
    # Generate reports
    if [ "$GENERATE_REPORTS" = true ]; then
        generate_html_report
        generate_ci_report
    fi
    
    # Cleanup and summary
    cleanup_and_optimize
    display_summary
    
    log_success "Total execution time: ${total_execution_time}s"
    
    # Exit with appropriate code
    local failed_count=0
    for status in "${test_results[@]}"; do
        if [ "$status" != "PASSED" ]; then
            failed_count=$((failed_count + 1))
        fi
    done
    
    if [ $failed_count -eq 0 ]; then
        log_success "🎉 All tests completed successfully!"
        exit 0
    else
        log_error "❌ $failed_count test categories failed"
        exit 1
    fi
}

# Execute main function with all arguments
main "$@"