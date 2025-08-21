# Speaker Diarization Troubleshooting Guide

**Comprehensive troubleshooting guide for KagiNote's speaker diarization system**

## Quick Diagnostics

### System Status Check

Before troubleshooting specific issues, verify the system status:

```bash
# Check diarization service status
RUST_LOG=info npm run tauri dev

# Look for these initialization messages:
# INFO kaginote::diarization: Service initialized successfully
# INFO kaginote::storage: Speaker database ready
# INFO kaginote::diarization::embedder: Embedding model loaded
```

### Basic Health Check

```typescript
// In browser console
window.__TAURI__.invoke('get_diarization_stats', { sessionId: 'test' })
  .then(stats => console.log('Service healthy:', stats))
  .catch(err => console.log('Service issue:', err));
```

## Common Issues and Solutions

### Speaker Detection Problems

#### Speakers Not Detected At All

**Symptoms:**
- All audio attributed to single speaker
- No speaker changes detected
- "Speaker 1" for entire recording

**Causes and Solutions:**

1. **Insufficient Audio Duration**
   ```
   Problem: Speakers must talk for >1 second to be detected
   Solution: Ensure minimum speaking time per person
   Test: Check min_segment_duration setting (default: 1.0s)
   ```

2. **VAD Threshold Too High**
   ```bash
   # Check current VAD threshold
   RUST_LOG=debug,kaginote::diarization::vad=trace npm run tauri dev
   
   # Look for: "VAD threshold: 0.5, detected speech: false"
   # If speech not detected, lower threshold:
   config.vad_threshold = 0.3;  // More sensitive
   ```

3. **Audio Quality Issues**
   ```
   Check for:
   - Low volume levels
   - Background noise interference
   - Echo or reverberation
   - Compressed audio formats
   
   Solution: Improve audio setup or enable noise filtering
   ```

4. **Service Not Initialized**
   ```typescript
   // Manually initialize service
   await invoke('initialize_diarization_service', {
     config: {
       maxSpeakers: 8,
       minSpeakers: 2,
       vadThreshold: 0.4,
       // ... other settings
     }
   });
   ```

---

#### Too Many Speakers Detected

**Symptoms:**
- Single speaker split into multiple IDs
- Excessive speaker changes
- "Speaker 4, Speaker 5, etc." for same person

**Solutions:**

1. **Increase Similarity Threshold**
   ```rust
   // Make clustering more aggressive
   config.similarity_threshold = 0.8;  // Higher = fewer speakers
   config.enable_adaptive_clustering = true;
   ```

2. **Merge Incorrectly Split Speakers**
   ```typescript
   await invoke('merge_speaker_profiles', {
     primarySpeakerId: "speaker_001",
     secondarySpeakerId: "speaker_002"
   });
   ```

3. **Check Voice Consistency**
   ```bash
   # Debug clustering decisions
   RUST_LOG=kaginote::diarization::clustering=trace npm run tauri dev
   
   # Look for similarity scores:
   # TRACE: Speaker similarity: 0.65 (below threshold 0.7)
   ```

---

#### Poor Speaker Separation

**Symptoms:**
- Wrong speaker attributed to segments
- Speakers confused with each other
- Low confidence scores (<0.6)

**Performance Tuning:**

1. **Increase Embedding Window Size**
   ```rust
   config.embedding_window_size = 5000; // 5 seconds (default: 3000ms)
   ```

2. **Enable Advanced Features**
   ```rust
   config.detect_overlaps = true;
   config.enable_adaptive_clustering = true;
   ```

3. **Voice Characteristics Analysis**
   ```bash
   # Check voice characteristic extraction
   RUST_LOG=kaginote::diarization::embedder=debug npm run tauri dev
   
   # Look for pitch/formant differences:
   # DEBUG: Voice characteristics - Pitch: 180Hz, F1: 730Hz, F2: 1090Hz
   ```

---

### Performance Issues

#### High Memory Usage

**Symptoms:**
- System slowdown during diarization
- "Memory limit exceeded" errors
- High swap usage

**Solutions:**

1. **Reduce Memory Configuration**
   ```rust
   let config = DiarizationConfig {
       max_memory_mb: 256,      // Reduce from 500MB
       max_speakers: 4,         // Reduce from 8
       embedding_window_size: 2000, // Reduce from 3000ms
       ..Default::default()
   };
   ```

2. **Clear Old Sessions**
   ```bash
   # Test memory usage
   cargo test memory_usage_test --manifest-path src-tauri/Cargo.toml -- --nocapture
   ```

3. **Monitor Real-time Usage**
   ```typescript
   listen('processing-progress', (event) => {
     console.log('Memory usage:', event.payload.memoryUsageMb);
     if (event.payload.memoryUsageMb > 400) {
       console.warn('High memory usage detected');
     }
   });
   ```

---

#### Slow Processing Speed

**Symptoms:**
- Long delays before speaker identification
- Real-time factor >2.0x
- UI becomes unresponsive

**Optimization Steps:**

1. **Hardware Acceleration**
   ```rust
   config.hardware_acceleration = HardwareAcceleration::Metal; // macOS
   // or
   config.hardware_acceleration = HardwareAcceleration::CUDA;  // NVIDIA
   ```

2. **Reduce Processing Load**
   ```rust
   config.detect_overlaps = false;          // Disable if not needed
   config.min_segment_duration = 2.0;       // Increase minimum segment
   config.enable_adaptive_clustering = false; // Disable if consistent speakers
   ```

3. **System Resource Check**
   ```bash
   # Monitor CPU and memory during processing
   top -pid $(pgrep kaginote)
   
   # Check available memory
   vm_stat | grep "Pages free"
   ```

---

### Audio Processing Errors

#### Invalid Sample Rate

**Error:** `Invalid sample rate: expected 16000, got 44100`

**Solutions:**
1. **Automatic Resampling (Recommended)**
   ```rust
   // This should be handled automatically by the audio resampler
   // Check if resampler is working:
   RUST_LOG=kaginote::audio::resampler=debug npm run tauri dev
   ```

2. **Manual Sample Rate Conversion**
   ```typescript
   // If automatic resampling fails, convert before sending
   const resampledAudio = await resampleAudio(audioData, 44100, 16000);
   ```

---

#### Audio Format Not Supported

**Error:** `AudioFormatError: Unsupported audio format`

**Solutions:**
```rust
// Supported formats:
- Sample rates: 8kHz - 96kHz (automatically resampled to 16kHz)
- Bit depths: 16-bit, 32-bit float
- Channels: Mono (preferred), Stereo (mixed to mono)

// Check current format:
RUST_LOG=kaginote::audio::capture=debug npm run tauri dev
```

---

### Database and Storage Issues

#### Speaker Database Corruption

**Error:** `Database error: table speakers does not exist`

**Recovery Steps:**
```bash
# 1. Clear corrupted database
cargo test clear_all_speaker_data --manifest-path src-tauri/Cargo.toml

# 2. Reinitialize database
cargo test initialize_speaker_storage --manifest-path src-tauri/Cargo.toml

# 3. Verify database structure
sqlite3 ~/Library/Application\ Support/KagiNote/speakers.db ".schema"
```

---

#### Embedding Index Corruption

**Error:** `EmbeddingError: Index file corrupted`

**Recovery:**
```bash
# Rebuild embedding index
cargo test rebuild_embedding_index --manifest-path src-tauri/Cargo.toml

# Check index integrity
cargo test verify_embedding_index --manifest-path src-tauri/Cargo.toml
```

---

### Model Loading Issues

#### Model Files Missing

**Error:** `ModelLoadError: Could not find diarization model`

**Solutions:**
```bash
# Check model files exist
ls -la ~/Library/Application\ Support/KagiNote/models/diarization/

# Expected files:
# - speaker_embedder.onnx
# - voice_activity_detector.onnx
# - clustering_model.bin

# If missing, models will be downloaded automatically on next start
```

---

#### Model Version Mismatch

**Error:** `ModelLoadError: Incompatible model version`

**Recovery:**
```bash
# Clear old models
rm -rf ~/Library/Application\ Support/KagiNote/models/diarization/

# Restart app to download latest models
npm run tauri dev
```

## Debug Logging

### Enable Comprehensive Logging

```bash
# Full debug logging
RUST_LOG=debug,kaginote=trace npm run tauri dev

# Specific component logging
RUST_LOG=kaginote::diarization=trace npm run tauri dev
RUST_LOG=kaginote::audio::capture=debug npm run tauri dev
RUST_LOG=kaginote::storage::speaker_store=debug npm run tauri dev
```

### Frontend Event Monitoring

```typescript
// Monitor all diarization events
const events = [
  'speaker-detected',
  'speaker-activity', 
  'processing-progress',
  'diarization-error',
  'diarization-complete'
];

events.forEach(eventName => {
  listen(eventName, (event) => {
    console.log(`[${eventName}]`, event.payload);
  });
});
```

### Performance Profiling

```bash
# Run performance benchmarks
cargo bench diarization_performance --manifest-path src-tauri/Cargo.toml

# Profile memory usage
cargo test --release memory_profile_test --manifest-path src-tauri/Cargo.toml -- --nocapture

# Check processing latency
cargo test --release latency_test --manifest-path src-tauri/Cargo.toml -- --nocapture
```

## Configuration Tuning

### Optimal Settings by Use Case

#### Small Meetings (2-4 people)
```rust
DiarizationConfig {
    max_speakers: 4,
    min_speakers: 2,
    similarity_threshold: 0.8,
    min_segment_duration: 1.5,
    embedding_window_size: 2000,
    detect_overlaps: false,
    max_memory_mb: 200,
    ..Default::default()
}
```

#### Large Meetings (5-8 people)
```rust
DiarizationConfig {
    max_speakers: 8,
    min_speakers: 3,
    similarity_threshold: 0.6,
    min_segment_duration: 1.0,
    embedding_window_size: 3000,
    detect_overlaps: true,
    max_memory_mb: 500,
    enable_adaptive_clustering: true,
    ..Default::default()
}
```

#### High-Quality Audio
```rust
DiarizationConfig {
    vad_threshold: 0.3,           // Lower threshold for clean audio
    similarity_threshold: 0.75,   // Can be more precise
    embedding_window_size: 4000,  // Longer windows for better accuracy
    ..Default::default()
}
```

#### Poor Audio Quality
```rust
DiarizationConfig {
    vad_threshold: 0.6,           // Higher threshold to avoid noise
    similarity_threshold: 0.6,    // More lenient clustering
    min_segment_duration: 2.0,    // Longer segments for stability
    detect_overlaps: false,       // Disable complex processing
    ..Default::default()
}
```

## FAQ

### General Questions

**Q: How accurate is speaker diarization?**
A: Accuracy ranges from 85-95% depending on audio quality, number of speakers, and voice distinctiveness. Clean audio with 2-4 distinct speakers typically achieves >90% accuracy.

**Q: Can it work with accents or different languages?**
A: Yes, speaker diarization is language and accent agnostic as it analyzes voice characteristics, not speech content.

**Q: How much memory does diarization use?**
A: Default configuration uses ~500MB RAM. Memory usage scales with max_speakers, embedding_window_size, and session length.

### Privacy Questions

**Q: What voice data is stored?**
A: Only mathematical embeddings (512-dimensional vectors) are stored, never raw audio or voice samples. Embeddings cannot be reverse-engineered to recreate original audio.

**Q: Can speaker profiles be shared?**
A: Yes, profiles can be exported/imported, but they only contain mathematical representations and user-assigned names/colors.

### Technical Questions

**Q: Why does speaker detection take a few seconds?**
A: The system needs sufficient audio (1-2 seconds minimum) to extract reliable voice characteristics and perform clustering.

**Q: Can multiple people with similar voices be distinguished?**
A: Yes, but accuracy may be lower. The system analyzes pitch, formants, speaking patterns, and other voice characteristics beyond just the sound.

**Q: What happens if someone joins the meeting late?**
A: New speakers are automatically detected when they first speak and assigned new speaker IDs and colors.

### Performance Questions

**Q: Does diarization slow down transcription?**
A: Minimal impact. Diarization runs in parallel with transcription and adds ~10-15% processing overhead.

**Q: Can it handle overlapping speech?**
A: Yes, when detect_overlaps is enabled. Overlapping segments are marked and can be attributed to multiple speakers.

**Q: How many speakers can be detected simultaneously?**
A: Up to 8 speakers by default, configurable down to 2 or up to 10. More speakers require more memory and processing power.

## Getting Help

### Before Requesting Support

1. **Check Debug Logs:**
   ```bash
   RUST_LOG=debug npm run tauri dev 2>&1 | grep -E "(ERROR|WARN|diarization)"
   ```

2. **Test with Known Audio:**
   ```bash
   # Use test audio files to isolate issues
   cargo test integration_test_with_known_speakers --manifest-path src-tauri/Cargo.toml
   ```

3. **Verify System Requirements:**
   - 8GB+ RAM (16GB recommended for 6+ speakers)
   - Modern CPU with AVX2 support
   - macOS 10.15+ for Metal acceleration

### Support Channels

- **GitHub Issues**: For bug reports with logs and reproduction steps
- **Documentation**: Check [SPEAKER-DIARIZATION-GUIDE.md](SPEAKER-DIARIZATION-GUIDE.md) for user guidance
- **API Reference**: See [DIARIZATION-API.md](DIARIZATION-API.md) for integration help

### Reporting Bugs

Include this information:
- Operating system and version
- KagiNote version
- Full debug logs (with sensitive info redacted)
- Steps to reproduce the issue
- Expected vs actual behavior
- Audio setup details (microphone, sample rate, etc.)

### Performance Issues

Provide benchmarking data:
```bash
# Generate performance report
cargo bench --manifest-path src-tauri/Cargo.toml > performance_report.txt

# Include system specs
system_profiler SPHardwareDataType > system_specs.txt
```

For urgent issues, include sample audio (with no personal/confidential content) that demonstrates the problem.