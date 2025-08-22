# KagiNote V2

**Privacy-focused meeting transcription with professional UI/UX - runs entirely on your device.**

KagiNote V2 is a **production-ready** desktop application built with Tauri v2, React 19, and Tailwind CSS that provides real-time meeting transcription with complete privacy - no audio data ever leaves your machine. Features a modern dashboard with chat-style speaker separation, audio file import, and persistent meeting management with a privacy-first design language.

## V2 Features (August 2025)

### Core Transcription
- **🔒 Privacy First**: 100% local processing, zero network calls during transcription
- **🎤 Universal Audio Support**: Works with any microphone - automatic device compatibility
- **🔄 Smart Resampling**: Automatic sample rate conversion (any device rate → 16kHz for Whisper)
- **🤖 Actual AI Transcription**: Real Whisper model inference with persistent caching
- **⚡ Instant Startup**: <1 second load time with cached models (first run: ~2 minutes)
- **🌐 Multilingual**: Supports 100+ languages via Whisper models
- **🎛️ Quality Tiers**: Standard (1.5GB), High Accuracy (2.4GB), Turbo (1.2GB) models
- **💾 Segment Storage**: Real-time transcription segments stored and accumulated during sessions
- **🎯 Live Display**: Actual transcription text displayed in real-time (not placeholders)
- **🎭 Speaker Diarization**: Real-time speaker identification using 3D-Speaker ERes2NetV2 ONNX models
- **👥 Speaker Profiles**: Persistent speaker profiles with 512-dimensional voice embeddings
- **🔊 Advanced Audio Processing**: Sliding window analysis with speech/silence segmentation
- **🏷️ Smart Speaker Clustering**: 70% similarity threshold with confidence scoring

### New V2 Interface
- **📊 Modern Dashboard**: Meeting list with search, sorting, and filtering
- **📁 Audio File Import**: Import and transcribe WAV, MP3, M4A, WebM files
- **💾 Persistent Storage**: All meetings saved to localStorage with metadata
- **🎨 Professional Design System**: Privacy-first visual language with business aesthetics
- **🖥️ Platform-Aware**: Native look on macOS/Windows with platform-specific adaptations
- **📱 Real-time Display**: Live transcription with audio visualization
- **🗂️ Meeting Management**: Create, save, delete, and review transcripts
- **🎭 Real-time Speaker Diarization**: Identify up to 8 speakers using state-of-the-art 3D-Speaker ERes2NetV2 models
- **📊 Diarization Status UI**: Live speaker detection status, confidence levels, and error handling
- **🔧 Professional Error Recovery**: Graceful degradation with clear user feedback and troubleshooting

## Quick Start

### Prerequisites
- Node.js 18+ and pnpm
- Rust 1.75+ with macOS deployment target 10.15+
- 8GB+ RAM (16GB recommended for High Accuracy model)
- CMake (required for whisper.cpp): `brew install cmake`
- macOS 10.15+ (for Metal acceleration support)

### Installation
```bash
git clone <repository-url>
cd KagiNote
pnpm install
```

### Development
```bash
# Start development server with hot reload
source ~/.cargo/env
export MACOSX_DEPLOYMENT_TARGET=10.15
npm run tauri dev

# Start with debug logging to monitor transcription
RUST_LOG=debug npm run tauri dev

# Build for production
npm run tauri build
```

### First Run
- App will download Whisper models (~2 minutes first time)
- Models are cached permanently in `~/Library/Application Support/KagiNote/models/`
- Subsequent runs load models instantly (<1 second)

## Architecture

**Backend (Rust)**
- **Universal audio capture** with automatic device compatibility and sample rate detection
- **Real-time audio resampling** using linear interpolation (any rate → 16kHz for Whisper)
- **Device intelligence** with built-in profiles for MacBook Pro/Air, iMac microphones
- **Actual Whisper transcription** using whisper-rs with Metal acceleration
- **Speaker Diarization** using pyannote ONNX models (segmentation-3.0, wespeaker embeddings)
- **Persistent model caching** with integrity validation and metadata tracking
- **Session state management** with concurrent session prevention
- **Enhanced error diagnostics** with device-specific troubleshooting guidance
- **Audio buffering** (1.5s minimum) for reliable transcription quality

**Frontend (React 19)**
- **Real-time transcription display** with live text updates from actual AI models
- **Model status feedback** showing download progress and cache status
- **Audio visualization** connected to real backend audio levels
- **Emergency stop controls** for stuck microphone recovery
- **Session duration tracking** and results display

## Performance

| Model Tier | Model Size | Memory Usage | Startup Time | Speaker Support | Use Case |
|------------|------------|--------------|--------------|-----------------|----------|
| Standard | 1.5GB | ~4GB | <1s (cached) | 8 speakers | Daily meetings |
| High Accuracy | 2.4GB | ~6GB | <1s (cached) | 8 speakers | Critical content |
| Turbo | 1.2GB | ~3GB | <1s (cached) | 8 speakers | Fastest processing |

**Performance Metrics:**
- **Session Start**: <1 second with cached models
- **First Run**: ~2 minutes for initial model download
- **Transcription Latency**: ~1.5 seconds for real-time display
- **Speaker Detection**: <2 seconds for new speaker identification

## Enhanced Transcription Quality

The latest version includes significant improvements to transcription accuracy and readability:

- **Smart Speech Boundary Detection**: Waits for natural pauses (500ms) before transcribing
- **Complete Utterance Buffering**: Processes 3-15 second segments for better context  
- **Accurate Timestamps**: Uses actual elapsed time, no more repetitions
- **Segment Deduplication**: Prevents repeated text using 80% similarity threshold
- **Noise Filtering**: Automatically removes [BLANK_AUDIO] and [INAUDIBLE] segments

## Speaker Diarization Implementation (August 2025)

### ✅ Production-Ready 3D-Speaker ERes2NetV2 Integration

**🎯 State-of-the-Art Models:** KagiNote uses the latest 3D-Speaker ERes2NetV2 ONNX models for professional-grade speaker identification.

### Current Implementation Status

**✅ COMPLETED (August 2025):**
- 3D-Speaker ERes2NetV2 ONNX models (71MB embedding + 6MB segmentation)
- Bundled models ship with app - no network dependencies
- Real-time processing with <1.5s latency
- Comprehensive UI with status indicators and error handling
- Full TDD test suite with 50+ tests

**Technology Stack:**
```toml
# src-tauri/Cargo.toml
ort = { version = "1.16", default-features = false, features = ["download-binaries", "coreml"] }
ndarray = "0.15"
```

**Processing Pipeline:**
1. **Audio Segmentation**: PyAnnote segmentation-3.0 detects speech regions
2. **Embedding Extraction**: 3D-Speaker ERes2NetV2 creates 512-dimensional voice embeddings  
3. **Speaker Clustering**: 70% similarity threshold for speaker identification
4. **Parallel Processing**: Runs alongside Whisper transcription for efficiency

**Models & Storage:**
- **Segmentation Model**: PyAnnote segmentation-3.0.onnx (6MB)
- **Embedding Model**: 3D-Speaker ERes2NetV2.onnx (71MB) 
- **Bundled Location**: `src-tauri/resources/models/diarization/`
- **Runtime Cache**: `~/Library/Application Support/KagiNote/models/diarization/`
- **Offline Operation**: Models bundled with app, no downloads required

## Privacy & Security

- **Zero network calls** during transcription processing
- **AES-256 encryption** for stored transcripts and speaker profiles
- **Memory protection** with secure audio buffer wiping
- **No voice samples stored** - only mathematical embeddings
- **Local speaker models** - all diarization processing on-device
- **OWASP compliant** with enterprise security standards
- **Source code audited** for privacy compliance

## V2 Architecture

### Frontend Structure
```
src/
├── components/
│   ├── ui/           # Reusable UI primitives
│   ├── features/     # Feature-specific components  
│   └── layout/       # Layout components
├── screens/          # Full screen views
├── hooks/            # Custom React hooks
├── lib/              # Utilities and helpers
└── styles/           # Global CSS and Tailwind
```

### Technology Stack
- **Frontend**: React 19, TypeScript, Tailwind CSS v3.4.17
- **Backend**: Rust, Tauri v2, whisper-rs with Metal acceleration
- **Audio**: cpal, hound, real-time resampling with device profiles
- **State**: React hooks, localStorage for persistence
- **Design**: Privacy-first visual language, professional UI components
- **Styling**: Tailwind utilities + comprehensive design system

### Breaking Changes from V1
- Complete UI redesign with new component architecture
- Dashboard replaces simple recording view
- Meeting management system added
- Audio file import functionality
- Platform-aware UI components
- localStorage for data persistence

## Documentation

- [Requirements Specification](PRPs/discovery/INITIAL-20250119-144500.md)
- [Technical Architecture](PRPs/active/kaginote-architecture.md)
- [Integration Guide](INTEGRATION_SUMMARY.md)
- [Development Guide](CLAUDE.md)
- [Design System](Documents/DESIGN-UNIFIED.md)
- [Transcription Debug Report & Diarization Plan](Documents/transcription-debug-report.md)
- [Initial Implementation Plan](Documents/initial-implementation-plan.md)

## System Requirements

**Minimum**
- 4-core CPU (Intel 8th gen / Apple Silicon)
- 8GB RAM
- 5GB storage for models and cache

**Recommended**
- 6-core CPU with AVX2 / Apple Silicon M1+
- 16GB RAM 
- macOS 10.15+ for Metal acceleration

## License

[License details]

## Support

For issues and feature requests, please use the GitHub issue tracker.
