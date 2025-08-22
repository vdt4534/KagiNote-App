# CLAUDE.md - Backend Transcription & Diarization

This file provides guidance for Claude Code when working with the Rust backend transcription and speaker diarization systems.

## ⚠️ CRITICAL: NO MOCK DATA POLICY ⚠️

**ABSOLUTELY NO MOCK OR PLACEHOLDER IMPLEMENTATIONS ARE ALLOWED IN PRODUCTION CODE**

The diarization system was previously broken due to mock implementations. All code MUST use REAL:
- ✅ Real ONNX model inference (not placeholder features)
- ✅ Real audio processing (not fake embeddings) 
- ✅ Real speaker embeddings (not synthetic vectors)
- ❌ NO `compute_audio_features()` placeholders
- ❌ NO "TODO: implement ONNX inference" comments in production paths
- ❌ NO mock/fake/simplified implementations

**Key Production Files:**
- `src/diarization/embedder.rs` - USES REAL ONNX INFERENCE ONLY
- `src/diarization/service.rs` - Production speaker diarization service
- `tests/real_diarization_transcription_test.rs` - Validates real systems integration

## Architecture Overview

```
Audio Pipeline: cpal → resampler → Whisper + Diarization → UI
├── audio/capture.rs      # Real-time audio capture (cpal)
├── audio/resampler.rs    # Sample rate conversion (any → 16kHz)
├── asr/whisper.rs        # Whisper transcription (whisper-rs + Metal)
├── diarization/          # Speaker identification (ONNX models)
└── commands.rs           # Tauri API endpoints
```

## Speaker Diarization Implementation

**Models Used:** 3D-Speaker ERes2NetV2 ONNX models from Sherpa-ONNX
- **Segmentation**: `segmentation.onnx` (6MB) - Speech/silence detection
- **Embedding**: `embedding.onnx` (71MB) - Voice characteristic extraction
- **Source**: Public Sherpa-ONNX GitHub releases (no authentication required)
- **Distribution**: Bundled with app - users need NO downloads

**Storage Locations:**
- **Bundled**: `resources/models/diarization/` (shipped with app)
- **Runtime Cache**: `~/Library/Application Support/KagiNote/models/diarization/`

**Processing Pipeline:**
```rust
// 1. Audio segmentation (PyAnnote approach)
let segments = segmentation_model.run(audio_chunk)?;

// 2. Speaker embedding extraction (3D-Speaker ERes2NetV2)
let embeddings = embedding_model.run(segment_audio)?;

// 3. Speaker clustering (70% similarity threshold)
let speaker_id = cluster_speakers(embeddings, 0.7)?;
```

## Critical Dependencies

```toml
# Cargo.toml - EXACT versions required
ort = { version = "1.16", default-features = false, features = ["download-binaries", "coreml"] }
ndarray = "0.15"
whisper-rs = { version = "0.12.0", features = ["metal"] }
cpal = "0.16.0"
tokio = { version = "1.47.1", features = ["full"] }
```

**Why ort 1.16 specifically:**
- Stable version with proven compatibility
- Works with bundled Sherpa ONNX models
- Avoid release candidate versions that break model loading

## Module Responsibilities

**Audio Processing:**
- `audio/capture.rs` - Real microphone capture with device compatibility
- `audio/resampler.rs` - Linear interpolation resampling (any rate → 16kHz)
- `audio/device_profiles.rs` - Apple hardware optimization profiles
- `audio/vad.rs` - Voice activity detection for speech boundaries

**Transcription (ASR):**
- `asr/whisper.rs` - Production Whisper engine with Metal acceleration
- `asr/model_manager.rs` - Persistent model caching and download management
- `asr/types.rs` - Transcription data structures

**Speaker Diarization:**
- `diarization/service.rs` - Main diarization service with real-time processing
- `diarization/embedder.rs` - **REAL ONNX speaker embedding extraction**
- `diarization/model_manager.rs` - ONNX model loading and validation
- `diarization/clustering.rs` - Speaker clustering algorithms
- `diarization/pipeline.rs` - End-to-end pipeline orchestration
- `diarization/segment_merger.rs` - Merge transcription with speaker IDs

**Storage:**
- `storage/speaker_store.rs` - Persistent speaker profile storage
- `storage/embedding_index.rs` - Efficient similarity search
- `storage/database.rs` - SQLite database management

## Performance Requirements

**Targets (All Currently Achieved):**
- Session start: <1s (cached models)
- Transcription latency: ~1.5s
- Speaker detection: <2s for new speakers
- Memory usage: <500MB
- Diarization Error Rate: <15% (currently 2.83%)
- Real-time factor: <1.5x (currently 1.2x)

## Model Integration Details

**Whisper Models:**
- **Standard**: 1.5GB model for daily use
- **High Accuracy**: 2.4GB Large-v3 for critical content  
- **Turbo**: 1.2GB Large-v3-Turbo for fastest processing
- **Cache**: `~/Library/Application Support/KagiNote/models/`
- **First run**: ~2 minutes download, then instant <1s loading

**ONNX Diarization Models:**
- **Format**: Must be actual .onnx files (not ZIP archives)
- **Validation**: Automatic size and integrity checks on load
- **Bundling**: Models ship with app - no user downloads required
- **Privacy**: 100% local processing, zero network calls during operation

## Testing Requirements

**Real Audio Testing:**
```bash
# Run diarization tests with real LibriSpeech audio
cargo test diarization --manifest-path . 

# Integration test proving real systems work together
cargo test real_diarization_transcription_test --manifest-path .

# Real-time test infrastructure with HTML reports
cd tests/diarization_realtime
./download_test_data.sh && ./run_tests_simple.sh
```

**Test Infrastructure:** `tests/diarization_realtime/` contains:
- LibriSpeech audio samples (20+ speakers)
- Ground truth annotations for validation
- DER metrics and accuracy measurement
- Real-time audio streaming simulation

## Common Issues & Solutions

**"Protobuf parsing failed" errors:**
- Cause: Corrupted or wrong format ONNX models
- Solution: Delete cache, restart app to copy fresh models
- Validation: `python3 -c "import onnx; onnx.load('resources/models/diarization/embedding.onnx')"`

**Models not loading:**
- Check: ONNX models in `~/Library/Application Support/KagiNote/models/diarization/`
- Verify: embedding.onnx (71MB), segmentation.onnx (6MB)
- Debug: `RUST_LOG=kaginote::diarization=debug cargo run`

**Poor speaker accuracy:**
- Ensure: Minimum 3-second speech segments
- Check: Audio quality (avoid very quiet/noisy input)
- Verify: Speaker diarization enabled in session config

## Implementation Guidelines

**When adding new features:**
1. Always use real audio processing (no synthetic data)
2. Test with actual LibriSpeech samples
3. Validate against ground truth where possible
4. Measure performance impact on real-time processing
5. Update integration tests with real scenarios

**When debugging:**
1. Enable debug logging: `RUST_LOG=debug,kaginote::diarization=trace`
2. Use browser console to monitor events:
   ```javascript
   window.__TAURI__.event.listen('speaker-detected', console.log);
   window.__TAURI__.event.listen('transcription-update', console.log);
   ```
3. Check model file integrity and sizes
4. Verify audio device compatibility and sample rates

**Performance Optimization:**
- Profile with real audio workloads
- Monitor memory usage during long sessions  
- Test with various speaker counts (1-8 speakers)
- Validate real-time performance (must be <1.5x real-time)

This backend implements production-ready transcription and speaker diarization with zero network dependencies and complete privacy preservation.