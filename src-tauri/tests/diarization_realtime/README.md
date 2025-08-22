# Real-Time Speaker Diarization Test Infrastructure

This test infrastructure provides comprehensive testing for KagiNote's real-time speaker diarization functionality, ensuring accuracy, performance, and reliability in production environments.

## Purpose

The real-time diarization testing framework validates:

- **Real-time Performance**: Ensures diarization processes audio faster than real-time (target: 1.2x real-time factor)
- **Accuracy Metrics**: Validates speaker identification accuracy and Diarization Error Rate (DER < 15%)
- **Memory Management**: Monitors memory usage and prevents leaks during long sessions
- **Edge Case Handling**: Tests challenging scenarios like overlapping speech, noisy environments, and many speakers
- **Integration Testing**: Validates end-to-end pipeline from audio input to speaker identification
- **Stress Testing**: Ensures system stability under load and resource constraints

## Test Structure

```
diarization_realtime/
├── mod.rs                  # Main test module with utilities and types
├── test_audio/            # Audio test files (excluded from git)
├── ground_truth/          # Expected results in JSON format
├── reports/               # Test execution reports (excluded from git)
├── performance_tests.rs   # Performance and latency tests
├── accuracy_tests.rs      # Speaker identification accuracy tests
├── integration_tests.rs   # End-to-end pipeline tests
├── memory_tests.rs        # Memory usage and leak detection
├── stress_tests.rs        # Load and concurrent session tests
└── README.md              # This documentation
```

## Running Tests

### Quick Start

The easiest way to run all tests with comprehensive reporting:

```bash
# Run complete test suite (requires bash 4.0+ - use on Linux/CI)
./src-tauri/tests/diarization_realtime/run_tests.sh

# Run simple test suite (compatible with macOS default bash)
./src-tauri/tests/diarization_realtime/run_tests_simple.sh

# Run with verbose output
./src-tauri/tests/diarization_realtime/run_tests_simple.sh --verbose

# Generate test audio only
./src-tauri/tests/diarization_realtime/download_test_data.sh --synthetic-only
```

### Test Audio Generation

Before running tests, you need to generate test audio files:

```bash
# Generate synthetic test audio files
cargo test test_generate_all_test_audio --manifest-path src-tauri/Cargo.toml -- --nocapture

# Or use the download script (creates directory structure and calls Rust)
./src-tauri/tests/diarization_realtime/download_test_data.sh

# Generate specific test scenarios
cargo test test_standard_test_audio_generation --manifest-path src-tauri/Cargo.toml -- --nocapture
```

### Basic Test Commands

```bash
# Run all diarization real-time tests
cargo test diarization_realtime --manifest-path src-tauri/Cargo.toml

# Run specific test categories
cargo test diarization_realtime::performance_tests --manifest-path src-tauri/Cargo.toml
cargo test diarization_realtime::accuracy_tests --manifest-path src-tauri/Cargo.toml
cargo test diarization_realtime::integration_tests --manifest-path src-tauri/Cargo.toml

# Run with debug output
RUST_LOG=debug cargo test diarization_realtime --manifest-path src-tauri/Cargo.toml -- --nocapture

# Run memory tests with detailed output
cargo test diarization_realtime::memory_tests --manifest-path src-tauri/Cargo.toml -- --nocapture

# Test audio generation specifically
cargo test create_test_audio --manifest-path src-tauri/Cargo.toml -- --nocapture
```

### Performance Benchmarking

```bash
# Run performance benchmarks
cargo test diarization_realtime::performance_tests::benchmark_ --manifest-path src-tauri/Cargo.toml

# Generate performance reports
cargo test diarization_realtime::performance_tests::generate_performance_report --manifest-path src-tauri/Cargo.toml

# Stress test with concurrent sessions
cargo test diarization_realtime::stress_tests::concurrent_sessions --manifest-path src-tauri/Cargo.toml
```

### Custom Test Configuration

Tests can be configured via environment variables:

```bash
# Set maximum test duration (seconds)
export DIARIZATION_TEST_DURATION=60

# Set number of speakers for stress tests
export DIARIZATION_MAX_SPEAKERS=10

# Enable hardware acceleration
export DIARIZATION_HARDWARE_ACCEL=true

# Set memory limit for tests (MB)
export DIARIZATION_MEMORY_LIMIT=512
```

## Test Scenarios

### Standard Test Scenarios

1. **Two Speaker Interview** (`two_speaker_interview.wav`)
   - Clean audio with 2 speakers
   - Expected DER: < 10%
   - Target latency: < 1.5s

2. **Four Speaker Meeting** (`four_speaker_meeting.wav`)
   - Business meeting with some overlapping speech
   - Expected DER: < 15%
   - Target latency: < 2.0s

3. **Eight Speaker Conference** (`eight_speaker_conference.wav`)
   - Large conference call (challenging scenario)
   - Expected DER: < 20%
   - Target latency: < 2.5s

4. **Noisy Environment** (`noisy_environment.wav`)
   - 2 speakers with background noise
   - Expected DER: < 25%
   - Target latency: < 2.0s

5. **Overlapping Speech** (`overlapping_speech.wav`)
   - 3 speakers with frequent overlaps
   - Expected DER: < 18%
   - Target latency: < 2.2s

### Synthetic Test Generation

The framework can generate synthetic test audio for consistent testing:

```rust
// Generate 30-second audio with 4 speakers
let audio = DiarizationTestUtils::generate_synthetic_audio(30.0, 4, 16000);
```

## Expected Performance Metrics

### Real-Time Processing Targets

| Metric | Target | Acceptable Range |
|--------|--------|------------------|
| Real-time Factor | 1.2x | 1.0x - 1.5x |
| Processing Latency | < 2.0s | < 3.0s |
| Memory Usage | < 500MB | < 750MB |
| DER (Overall) | < 15% | < 20% |
| Speaker Accuracy | > 85% | > 80% |

### Per-Scenario Targets

| Scenario | DER Target | Latency Target | Memory Target |
|----------|------------|----------------|---------------|
| 2 Speakers (Clean) | < 10% | < 1.5s | < 300MB |
| 4 Speakers (Meeting) | < 15% | < 2.0s | < 450MB |
| 8 Speakers (Conference) | < 20% | < 2.5s | < 500MB |
| Noisy Environment | < 25% | < 2.0s | < 350MB |
| Overlapping Speech | < 18% | < 2.2s | < 400MB |

## Test Data Format

### Ground Truth JSON Format

```json
{
  "metadata": {
    "duration": "30.0",
    "sample_rate": "16000",
    "num_speakers": "3"
  },
  "segments": [
    {
      "start": 0.0,
      "end": 3.5,
      "speaker": "speaker_1",
      "text": "Hello, welcome to our meeting today."
    },
    {
      "start": 3.5,
      "end": 7.2,
      "speaker": "speaker_2", 
      "text": "Thank you for having me."
    }
  ],
  "total_speakers": 3,
  "total_duration": 30.0
}
```

### Test Results Format

```json
{
  "scenario": "four_speaker_meeting",
  "timestamp": "2025-08-22T10:30:00Z",
  "latency_ms": 1850,
  "der": 0.14,
  "memory_mb": 420.5,
  "accuracy": 0.87,
  "real_time_factor": 1.15,
  "detected_speakers": 4,
  "expected_speakers": 4,
  "passed": true,
  "details": {
    "model_load_time_ms": "450",
    "avg_segment_processing_ms": "320",
    "peak_memory_mb": "485.2"
  }
}
```

## Generated Test Audio Files

The test infrastructure automatically generates the following synthetic audio files:

### Standard Scenarios
1. **2speaker_conversation.wav** - Simple turn-taking between 2 speakers (30s)
2. **3speaker_meeting.wav** - Business meeting with 3 speakers and interruptions (45s)
3. **overlapping_speech.wav** - Multiple speakers talking simultaneously (25s)
4. **rapid_switching.wav** - Very fast speaker changes every 1-2 seconds (20s)
5. **long_silences.wav** - Speakers with long pauses between utterances (40s)
6. **single_speaker.wav** - Single speaker monologue (30s)

### Challenging Edge Cases
7. **noisy_environment.wav** - High background noise scenario (30s)
8. **many_speakers.wav** - 8 speakers in conference call (60s)
9. **whisper_speech.wav** - Very quiet whisper-like speech (20s)
10. **mixed_gender.wav** - Mixed male and female speakers (35s)

### Audio Characteristics
- **Sample Rate**: 16kHz (optimized for Whisper)
- **Format**: Mono WAV files
- **Voice Synthesis**: Multi-frequency harmonics with realistic voice characteristics
- **Background Noise**: Subtle room tone and electrical hum for realism
- **Confidence Levels**: Varied based on scenario difficulty

## Interpreting Test Results

### HTML Report Structure

The comprehensive test runner generates detailed HTML reports:

```
reports/
├── comprehensive_test_report.html  # Main visual report
├── ci_report.json                 # CI/CD compatible JSON
├── junit_report.xml               # JUnit format for CI systems
├── test_run_metadata.json         # Test execution metadata
└── [category]_output.log          # Detailed logs per test category
```

### Success Metrics

| Metric | Interpretation |
|--------|----------------|
| **Success Rate** | Percentage of test categories that passed |
| **Individual Category Status** | PASSED/FAILED/TIMEOUT for each test type |
| **Duration** | Execution time per category (watch for timeouts) |
| **Test Counts** | Passed/Failed/Ignored per category |

### Understanding Test Categories

1. **Unit Tests** - Basic component functionality
2. **Integration Tests** - End-to-end pipeline testing
3. **Accuracy Tests** - Speaker identification precision
4. **Performance Tests** - Speed and latency measurements
5. **Memory Tests** - Resource usage and leak detection
6. **Audio Generation** - Test file creation and validation
7. **Validation Framework** - Ground truth comparison tools

## Troubleshooting Guide

### Test Audio Generation Issues

**Problem**: "Audio generation failed"
```bash
# Solutions:
1. Check directory permissions
   mkdir -p src-tauri/tests/diarization_realtime/test_audio
   
2. Verify Rust dependencies
   cargo check --manifest-path src-tauri/Cargo.toml
   
3. Run audio generation separately
   cargo test test_generate_all_test_audio --manifest-path src-tauri/Cargo.toml -- --nocapture
```

**Problem**: "Missing ground truth files"
```bash
# Solutions:
1. Regenerate all test data
   ./src-tauri/tests/diarization_realtime/download_test_data.sh --synthetic-only
   
2. Check ground truth directory
   ls -la src-tauri/tests/diarization_realtime/ground_truth/
```

### Test Execution Issues

**Problem**: Tests timeout
```bash
# Solutions:
1. Increase timeout
   ./run_tests.sh --timeout 600
   
2. Run specific categories
   cargo test diarization_realtime::accuracy_tests --manifest-path src-tauri/Cargo.toml
   
3. Check system resources
   top -o cpu
```

**Problem**: High memory usage
```bash
# Solutions:
1. Reduce parallel jobs
   ./run_tests.sh --parallel-jobs 2
   
2. Monitor memory during tests
   cargo test diarization_realtime::memory_tests --manifest-path src-tauri/Cargo.toml -- --nocapture
```

**Problem**: Missing dependencies
```bash
# Install required tools:
brew install bc jq ffmpeg  # macOS
apt-get install bc jq ffmpeg  # Ubuntu
```

### Diarization Performance Issues

**Problem**: Low accuracy (< 70%)
- Check audio quality and ground truth validity
- Verify speaker similarity thresholds
- Review embedding model performance

**Problem**: High latency (> 3s)
- Check system CPU/memory resources
- Verify hardware acceleration settings
- Review buffer sizes and processing windows

**Problem**: Memory leaks during long tests
- Enable detailed logging: `RUST_LOG=trace`
- Run memory profiling tests
- Check for proper cleanup in error paths

### CI/CD Integration Issues

**Problem**: Tests fail in CI but pass locally
```bash
# Generate CI-compatible reports
./run_tests.sh --no-reports
cat reports/ci_report.json

# Use containerized testing
docker run --rm -v $(pwd):/workspace rust:latest bash -c "cd /workspace && ./run_tests.sh"
```

## Adding New Test Cases

### 1. Add Audio File
Place your test audio file in `test_audio/`:
```bash
cp your_test_audio.wav src-tauri/tests/diarization_realtime/test_audio/
```

### 2. Create Ground Truth
Create corresponding ground truth in `ground_truth/`:
```json
{
  "audio_file": "your_test_audio.wav",
  "duration": 45.0,
  "segments": [
    {
      "speaker_id": "speaker_0",
      "start_time": 0.0,
      "end_time": 15.0,
      "text": "Sample speaker content",
      "confidence": 0.9
    }
  ],
  "total_speakers": 2,
  "metadata": {
    "scenario_type": "custom_test",
    "description": "Your custom test description"
  }
}
```

### 3. Add Custom Scenario to Generator
Edit `src-tauri/tests/diarization_realtime/create_test_audio.rs`:
```rust
pub fn your_custom_scenario() -> Result<GeneratedAudioFile, Box<dyn std::error::Error>> {
    let mut ground_truth = GroundTruthData::new("your_test.wav".to_string(), 45.0);
    
    // Add your segments
    ground_truth.add_segment(GroundTruthSegment::new(
        "speaker_0".to_string(), 0.0, 15.0, 
        Some("Your test content".to_string()), 0.9
    ));
    
    // Generate audio and save
    let audio_path = self.generate_scenario_audio(&ground_truth, "your_test")?;
    let ground_truth_path = self.save_ground_truth(&ground_truth, "your_test")?;
    
    Ok(GeneratedAudioFile { /* ... */ })
}
```

## Debugging Test Failures

### Common Issues

1. **High DER (> 20%)**
   - Check ground truth accuracy
   - Verify speaker similarity threshold
   - Review audio quality

2. **High Latency (> 3s)**
   - Check system resources
   - Verify hardware acceleration
   - Review buffer sizes

3. **Memory Leaks**
   - Run with memory debugging: `RUST_LOG=trace`
   - Check for proper cleanup in error paths
   - Monitor long-running sessions

### Debug Commands

```bash
# Enable detailed logging
RUST_LOG=kaginote::diarization=trace cargo test diarization_realtime

# Run single test with output
cargo test specific_test_name --manifest-path src-tauri/Cargo.toml -- --nocapture

# Profile memory usage
valgrind --tool=memcheck cargo test diarization_realtime::memory_tests

# Generate test report
cargo test diarization_realtime::generate_full_report --manifest-path src-tauri/Cargo.toml
```

## Continuous Integration

These tests are designed to run in CI environments:

```yaml
# Example GitHub Actions integration
- name: Run Diarization Tests
  run: |
    cargo test diarization_realtime --manifest-path src-tauri/Cargo.toml
    
- name: Generate Test Report
  run: |
    cargo test diarization_realtime::generate_ci_report --manifest-path src-tauri/Cargo.toml
```

## Contributing

When adding new diarization functionality:

1. **Add corresponding tests** in appropriate test module
2. **Update performance targets** if needed
3. **Add new test scenarios** for edge cases
4. **Ensure backwards compatibility** with existing tests
5. **Document any new configuration options**

## Performance Optimization

Use these tests to guide optimization efforts:

1. **Identify bottlenecks** via performance tests
2. **Set realistic targets** based on hardware capabilities  
3. **Monitor memory usage** patterns during development
4. **Validate optimizations** don't reduce accuracy
5. **Test edge cases** thoroughly after changes

For questions about the test infrastructure, see the main project documentation or the diarization module source code.