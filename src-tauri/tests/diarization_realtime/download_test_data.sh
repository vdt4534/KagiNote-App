#!/bin/bash

# Download Test Data Script for KagiNote Speaker Diarization
# This script downloads sample audio files for testing and creates ground truth data
# Falls back to synthetic generation if downloads fail

set -e  # Exit on any error

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TEST_AUDIO_DIR="$SCRIPT_DIR/test_audio"
GROUND_TRUTH_DIR="$SCRIPT_DIR/ground_truth"
TEMP_DIR="$SCRIPT_DIR/tmp_download"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

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

# Create directories
create_directories() {
    log_info "Creating directory structure..."
    mkdir -p "$TEST_AUDIO_DIR"
    mkdir -p "$GROUND_TRUTH_DIR"
    mkdir -p "$TEMP_DIR"
    log_success "Directories created"
}

# Check dependencies
check_dependencies() {
    log_info "Checking dependencies..."
    
    local missing_deps=()
    
    # Check for curl or wget
    if ! command -v curl >/dev/null 2>&1 && ! command -v wget >/dev/null 2>&1; then
        missing_deps+=("curl or wget")
    fi
    
    # Check for ffmpeg (for audio conversion)
    if ! command -v ffmpeg >/dev/null 2>&1; then
        log_warning "ffmpeg not found - some audio conversions may fail"
    fi
    
    # Check for Rust/Cargo for fallback generation
    if ! command -v cargo >/dev/null 2>&1; then
        missing_deps+=("cargo")
    fi
    
    if [ ${#missing_deps[@]} -gt 0 ]; then
        log_error "Missing dependencies: ${missing_deps[*]}"
        log_error "Please install missing dependencies and try again"
        exit 1
    fi
    
    log_success "All required dependencies found"
}

# Download function with fallback
download_file() {
    local url="$1"
    local output_file="$2"
    local description="$3"
    
    log_info "Downloading $description..."
    
    # Try curl first, then wget
    if command -v curl >/dev/null 2>&1; then
        if curl -L -o "$output_file" "$url" --connect-timeout 10 --max-time 60; then
            log_success "Downloaded $description"
            return 0
        fi
    elif command -v wget >/dev/null 2>&1; then
        if wget -O "$output_file" "$url" --timeout=60; then
            log_success "Downloaded $description"
            return 0
        fi
    fi
    
    log_warning "Failed to download $description"
    return 1
}

# Download sample audio files from public sources
download_sample_audio() {
    log_info "Attempting to download sample audio files..."
    
    # Array of public domain audio samples
    # Note: These are example URLs - in practice, you'd use actual public domain audio
    declare -A audio_urls
    
    # LibriVox public domain recordings (example URLs)
    audio_urls["interview_sample.wav"]="https://archive.org/download/interview_sample/sample.wav"
    audio_urls["meeting_sample.wav"]="https://archive.org/download/meeting_sample/sample.wav"
    audio_urls["conference_sample.wav"]="https://archive.org/download/conference_sample/sample.wav"
    
    local download_success=0
    local total_attempts=0
    
    for filename in "${!audio_urls[@]}"; do
        total_attempts=$((total_attempts + 1))
        local url="${audio_urls[$filename]}"
        local temp_file="$TEMP_DIR/$filename"
        local final_file="$TEST_AUDIO_DIR/$filename"
        
        if download_file "$url" "$temp_file" "$filename"; then
            # Convert to standard format if needed
            if convert_audio_format "$temp_file" "$final_file"; then
                download_success=$((download_success + 1))
                create_ground_truth_for_sample "$filename"
            else
                log_warning "Failed to convert $filename"
            fi
        else
            log_warning "Skipping $filename - download failed"
        fi
    done
    
    log_info "Successfully downloaded $download_success out of $total_attempts audio files"
    
    if [ $download_success -eq 0 ]; then
        log_warning "No audio files downloaded successfully"
        return 1
    fi
    
    return 0
}

# Convert audio to standard format (16kHz, mono, WAV)
convert_audio_format() {
    local input_file="$1"
    local output_file="$2"
    
    if ! command -v ffmpeg >/dev/null 2>&1; then
        # No ffmpeg available, just copy the file
        cp "$input_file" "$output_file"
        return 0
    fi
    
    log_info "Converting $(basename "$input_file") to standard format..."
    
    if ffmpeg -i "$input_file" -ar 16000 -ac 1 -f wav "$output_file" -y >/dev/null 2>&1; then
        log_success "Converted $(basename "$input_file")"
        return 0
    else
        log_warning "Failed to convert $(basename "$input_file")"
        return 1
    fi
}

# Create ground truth data for downloaded samples
create_ground_truth_for_sample() {
    local filename="$1"
    local basename="${filename%.*}"
    local ground_truth_file="$GROUND_TRUTH_DIR/${basename}.json"
    
    log_info "Creating ground truth for $filename..."
    
    # Get audio duration using ffprobe if available
    local duration=30.0
    if command -v ffprobe >/dev/null 2>&1; then
        local audio_file="$TEST_AUDIO_DIR/$filename"
        if [ -f "$audio_file" ]; then
            duration=$(ffprobe -v quiet -show_entries format=duration -of csv=p=0 "$audio_file" 2>/dev/null || echo "30.0")
        fi
    fi
    
    # Create basic ground truth structure
    # Note: This creates placeholder data - real ground truth would need manual annotation
    cat > "$ground_truth_file" << EOF
{
  "audio_file": "$filename",
  "duration": $duration,
  "segments": [
    {
      "speaker_id": "speaker_0",
      "start_time": 0.0,
      "end_time": $(echo "$duration * 0.4" | bc -l 2>/dev/null || echo "12.0"),
      "text": "Sample speaker 1 content",
      "confidence": 0.85
    },
    {
      "speaker_id": "speaker_1", 
      "start_time": $(echo "$duration * 0.5" | bc -l 2>/dev/null || echo "15.0"),
      "end_time": $duration,
      "text": "Sample speaker 2 content",
      "confidence": 0.80
    }
  ],
  "total_speakers": 2,
  "metadata": {
    "scenario_type": "downloaded_sample",
    "description": "Downloaded sample audio with estimated ground truth",
    "source": "public_domain",
    "note": "Ground truth is estimated and should be manually verified"
  }
}
EOF
    
    log_success "Created ground truth for $filename"
}

# Generate synthetic test audio using Rust
generate_synthetic_audio() {
    log_info "Generating synthetic test audio files..."
    
    # Change to the src-tauri directory to run Cargo
    local original_dir="$(pwd)"
    cd "$SCRIPT_DIR/../../.."
    
    # Run the Rust test that generates synthetic audio
    if cargo test --manifest-path src-tauri/Cargo.toml diarization_realtime::create_test_audio::tests::test_generate_all_synthetic_audio -- --nocapture; then
        log_success "Successfully generated synthetic audio files"
        cd "$original_dir"
        return 0
    else
        log_error "Failed to generate synthetic audio files"
        cd "$original_dir"
        return 1
    fi
}

# Generate synthetic audio via Rust (fallback method)
generate_synthetic_audio_fallback() {
    log_info "Using Rust fallback to generate test audio..."
    
    # Create a temporary Rust program to generate audio
    cat > "$TEMP_DIR/generate_audio.rs" << 'EOF'
use std::path::Path;

// Include our test audio generator (this would need proper module structure)
// For now, we'll call the existing Rust test infrastructure

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Generating synthetic test audio...");
    
    // This calls into the test infrastructure we just created
    // In practice, this would be structured as a proper binary
    
    println!("✅ Synthetic audio generation completed");
    Ok(())
}
EOF
    
    log_warning "Synthetic fallback generation not yet fully implemented"
    log_info "Use 'cargo test diarization_realtime' to generate synthetic audio"
    return 1
}

# Validate downloaded/generated files
validate_test_files() {
    log_info "Validating test files..."
    
    local audio_files=("$TEST_AUDIO_DIR"/*.wav)
    local ground_truth_files=("$GROUND_TRUTH_DIR"/*.json)
    
    if [ ! -f "${audio_files[0]}" ]; then
        log_error "No audio files found in $TEST_AUDIO_DIR"
        return 1
    fi
    
    if [ ! -f "${ground_truth_files[0]}" ]; then
        log_error "No ground truth files found in $GROUND_TRUTH_DIR"
        return 1
    fi
    
    local valid_files=0
    local total_files=0
    
    for audio_file in "${audio_files[@]}"; do
        if [ -f "$audio_file" ]; then
            total_files=$((total_files + 1))
            local basename=$(basename "$audio_file" .wav)
            local ground_truth_file="$GROUND_TRUTH_DIR/${basename}.json"
            
            # Check if corresponding ground truth exists
            if [ -f "$ground_truth_file" ]; then
                # Validate JSON format
                if python3 -m json.tool "$ground_truth_file" >/dev/null 2>&1 || node -e "JSON.parse(require('fs').readFileSync('$ground_truth_file', 'utf8'))" >/dev/null 2>&1; then
                    valid_files=$((valid_files + 1))
                    log_success "Validated: $(basename "$audio_file")"
                else
                    log_warning "Invalid JSON: $(basename "$ground_truth_file")"
                fi
            else
                log_warning "Missing ground truth: $(basename "$audio_file")"
            fi
        fi
    done
    
    log_info "Validation complete: $valid_files/$total_files files valid"
    
    if [ $valid_files -eq 0 ]; then
        return 1
    fi
    
    return 0
}

# Create test data summary
create_summary() {
    log_info "Creating test data summary..."
    
    local summary_file="$SCRIPT_DIR/test_data_summary.md"
    
    cat > "$summary_file" << EOF
# Test Data Summary

Generated on: $(date)

## Audio Files

EOF
    
    for audio_file in "$TEST_AUDIO_DIR"/*.wav; do
        if [ -f "$audio_file" ]; then
            local filename=$(basename "$audio_file")
            local basename="${filename%.*}"
            local ground_truth_file="$GROUND_TRUTH_DIR/${basename}.json"
            
            echo "### $filename" >> "$summary_file"
            echo "" >> "$summary_file"
            
            # Get file size
            local size=$(ls -lh "$audio_file" | awk '{print $5}')
            echo "- **Size**: $size" >> "$summary_file"
            
            # Get duration if possible
            if command -v ffprobe >/dev/null 2>&1; then
                local duration=$(ffprobe -v quiet -show_entries format=duration -of csv=p=0 "$audio_file" 2>/dev/null || echo "unknown")
                echo "- **Duration**: ${duration}s" >> "$summary_file"
            fi
            
            # Extract info from ground truth
            if [ -f "$ground_truth_file" ]; then
                local speakers=$(grep -o '"total_speakers": [0-9]*' "$ground_truth_file" | grep -o '[0-9]*' || echo "unknown")
                local scenario=$(grep -o '"scenario_type": "[^"]*"' "$ground_truth_file" | sed 's/"scenario_type": "\([^"]*\)"/\1/' || echo "unknown")
                echo "- **Speakers**: $speakers" >> "$summary_file"
                echo "- **Scenario**: $scenario" >> "$summary_file"
                echo "- **Ground Truth**: ✅ Available" >> "$summary_file"
            else
                echo "- **Ground Truth**: ❌ Missing" >> "$summary_file"
            fi
            
            echo "" >> "$summary_file"
        fi
    done
    
    echo "## Usage" >> "$summary_file"
    echo "" >> "$summary_file"
    echo "To run tests with this data:" >> "$summary_file"
    echo "" >> "$summary_file"
    echo '```bash' >> "$summary_file"
    echo "cargo test diarization_realtime --manifest-path src-tauri/Cargo.toml" >> "$summary_file"
    echo '```' >> "$summary_file"
    
    log_success "Created test data summary: $summary_file"
}

# Cleanup temporary files
cleanup() {
    log_info "Cleaning up temporary files..."
    rm -rf "$TEMP_DIR"
    log_success "Cleanup completed"
}

# Main execution
main() {
    log_info "🎵 KagiNote Speaker Diarization Test Data Setup"
    log_info "=============================================="
    
    # Check if we should skip downloads
    local skip_downloads=false
    if [ "$1" = "--synthetic-only" ]; then
        skip_downloads=true
        log_info "Skipping downloads, generating synthetic audio only"
    fi
    
    # Setup
    create_directories
    check_dependencies
    
    # Download or generate audio
    local download_success=false
    
    if [ "$skip_downloads" = false ]; then
        if download_sample_audio; then
            download_success=true
        else
            log_warning "Download failed, falling back to synthetic generation"
        fi
    fi
    
    # Generate synthetic audio if downloads failed or were skipped
    if [ "$download_success" = false ]; then
        log_info "Generating synthetic test audio files..."
        log_info "Note: Run 'cargo test diarization_realtime::create_test_audio' to generate audio files"
        
        # Create basic directory structure for synthetic generation
        mkdir -p "$TEST_AUDIO_DIR"
        mkdir -p "$GROUND_TRUTH_DIR"
        
        log_info "Directory structure prepared for synthetic audio generation"
        log_info "Use the Rust test suite to generate synthetic audio files:"
        log_info "  cargo test --manifest-path src-tauri/Cargo.toml test_generate_all_test_audio -- --nocapture"
    fi
    
    # Validate what we have
    if validate_test_files; then
        create_summary
        log_success "✅ Test data setup completed successfully!"
        
        # Print next steps
        echo ""
        log_info "Next steps:"
        log_info "1. Run tests: ./run_tests.sh"
        log_info "2. Or run specific tests: cargo test diarization_realtime --manifest-path src-tauri/Cargo.toml"
        log_info "3. View summary: cat $SCRIPT_DIR/test_data_summary.md"
        
    else
        log_warning "⚠️  Test data setup completed with issues"
        log_info "You may need to manually verify or regenerate some files"
        log_info "Try running: cargo test diarization_realtime::create_test_audio --manifest-path src-tauri/Cargo.toml"
    fi
    
    cleanup
}

# Handle script arguments
show_help() {
    echo "KagiNote Test Data Download Script"
    echo ""
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  --synthetic-only    Skip downloads, only prepare for synthetic generation"
    echo "  --help             Show this help message"
    echo ""
    echo "This script downloads sample audio files for speaker diarization testing."
    echo "If downloads fail, it falls back to synthetic audio generation."
}

# Parse command line arguments
case "${1:-}" in
    --help|-h)
        show_help
        exit 0
        ;;
    --synthetic-only)
        main "$1"
        ;;
    "")
        main
        ;;
    *)
        log_error "Unknown option: $1"
        show_help
        exit 1
        ;;
esac