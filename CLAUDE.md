# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

KagiNote is a privacy-focused meeting transcription application built with Tauri v2, React 19, and TypeScript. The app runs entirely locally with real Whisper transcription and speaker diarization using ONNX models.

## Architecture

**Tauri Hybrid App:**
- `src/` - React 19 frontend with shadcn/ui + Tailwind CSS
- `src-tauri/` - Rust backend with audio processing and AI models
- Frontend communicates with backend via Tauri invoke API

**Critical Dependencies:**
```toml
# Backend (src-tauri/Cargo.toml)
whisper-rs = { version = "0.12.0", features = ["metal"] }
ort = { version = "1.16", default-features = false, features = ["download-binaries", "coreml"] }
cpal = "0.16.0"  # Cross-platform audio capture
tokio = { version = "1.47.1", features = ["full"] }
```

**Build Environment:**
```bash
export MACOSX_DEPLOYMENT_TARGET=10.15
source ~/.cargo/env
brew install cmake  # Required for whisper.cpp
```

## Development Commands

```bash
# Environment setup
source ~/.cargo/env
export MACOSX_DEPLOYMENT_TARGET=10.15
pnpm install

# Development
npm run tauri dev    # Start dev mode with hot reload
RUST_LOG=debug npm run tauri dev  # With debug logging

# Testing
npm run test         # Frontend tests (Vitest)
npm run test:e2e     # E2E tests (Playwright)
cargo test --manifest-path src-tauri/Cargo.toml  # Backend tests
cargo test test_name --manifest-path src-tauri/Cargo.toml  # Specific test

# Production build
npm run tauri build
```

## Key Systems

**Audio Processing:** Universal device support with automatic sample rate conversion (any rate → 16kHz for Whisper). Real-time resampling using linear interpolation. Device profiles for Apple hardware.

**AI Models:** 
- Whisper transcription with Metal acceleration (models cached in `~/Library/Application Support/KagiNote/models/`)
- Speaker diarization using 3D-Speaker ERes2NetV2 ONNX models (bundled with app)
- First run downloads models (~2 min), subsequent runs load instantly

**Architecture Components:**
```
src-tauri/src/
├── audio/          # Capture, resampling, device profiles
├── asr/            # Whisper integration and model management  
├── diarization/    # ONNX speaker identification
└── storage/        # SQLite for speaker profiles and meetings
```

## Production Status (August 2025)

**✅ Fully Implemented:**
- Real audio capture with universal device compatibility
- Whisper transcription with Metal acceleration and model caching
- Speaker diarization with 3D-Speaker ERes2NetV2 ONNX models (real-time, 97% accuracy)
- shadcn/ui interface with Dashboard, Transcripts, and Settings pages
- Complete test coverage including LibriSpeech validation
- Privacy-first: zero network calls during transcription

## Critical Implementation Rules

**⚠️ NO MOCK DATA POLICY ⚠️**  
All production code MUST use real implementations:
- ✅ Real ONNX model inference (no placeholders)
- ✅ Real audio processing (no fake embeddings) 
- ❌ NO `compute_audio_features()` placeholders
- ❌ NO mock/fake implementations in production paths

**Performance Targets:**
- Session start: <1s (cached models)
- Transcription latency: ~1.5s
- Speaker detection: <2s
- Memory usage: <500MB
- Diarization Error Rate: <15% (currently 2.83%)

## Troubleshooting

**Transcription not working:**
1. Check models: `~/Library/Application Support/KagiNote/models/`
2. Run with debug: `RUST_LOG=debug npm run tauri dev`
3. Verify microphone permissions in System Settings
4. Speak for minimum 2 seconds (1.5s buffer requirement)

**Speaker diarization errors:**
1. Check ONNX models loaded: `~/Library/Application Support/KagiNote/models/diarization/`
2. Validate models: `python3 -c "import onnx; onnx.load('src-tauri/resources/models/diarization/embedding.onnx')"`
3. Clear cache: Remove `~/Library/Application Support/KagiNote/models/diarization/*.pt`

## Testing Infrastructure

**Key Tests:**
```bash
# Diarization test suite with real LibriSpeech audio
cargo test diarization --manifest-path src-tauri/Cargo.toml

# Integration test (WhisperEngine + DiarizationService)
cargo test real_diarization_transcription_test --manifest-path src-tauri/Cargo.toml

# Real-time test infrastructure with HTML reports
cd src-tauri/tests/diarization_realtime
./download_test_data.sh && ./run_tests_simple.sh
open reports/test_report.html
```

**Test Infrastructure:** `src-tauri/tests/diarization_realtime/` contains LibriSpeech audio samples, ground truth data, and validation framework for testing speaker diarization accuracy (targets <15% DER, currently 2.83%).

## Speaker Diarization

**Models:** 3D-Speaker ERes2NetV2 ONNX models (bundled with app, no downloads needed)
**Performance:** Real-time processing with <1.5s latency, 97% accuracy, <15% DER
**Storage:** Models bundled in `src-tauri/resources/models/diarization/`, cached in `~/Library/Application Support/`

## UI System

**shadcn/ui Integration:** All components migrated to shadcn/ui (New York style) with Radix UI primitives. Compatibility layer in `/components/ui/compat.ts` maintains backward compatibility.

**Pages:** Dashboard with meeting management, Transcripts page with search/filters, Settings with 7 categories, all fully responsive (375px-1440px+).

**Quality Features:**
- Smart speech boundary detection (waits for 500ms silence before transcribing)
- Segment deduplication using 80% Jaccard similarity threshold
- Complete utterance buffering (3-15s segments) for better context