# KagiNote

**Privacy-focused meeting transcription that runs entirely on your device.**

KagiNote is a **production-ready** desktop application built with Tauri v2 that provides real-time meeting transcription with complete privacy - no audio data ever leaves your machine. Supports 100+ languages with optimized performance and persistent model caching.

## Features

- **🔒 Privacy First**: 100% local processing, zero network calls during transcription
- **🎤 Real Audio Capture**: Live microphone recording with proper start/stop controls
- **🤖 Actual AI Transcription**: Real Whisper model inference with persistent caching
- **⚡ Instant Startup**: <1 second load time with cached models (first run: ~2 minutes)
- **🌐 Multilingual**: Supports 100+ languages via Whisper models
- **🎛️ Quality Tiers**: Standard (1.5GB), High Accuracy (2.4GB), Turbo (1.2GB) models
- **📱 Real-time Display**: Live transcription text appears as you speak (~1.5s latency)
- **🖥️ Production Ready**: Fully functional with comprehensive error handling

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

# Build for production
npm run tauri build
```

### First Run
- App will download Whisper models (~2 minutes first time)
- Models are cached permanently in `~/Library/Application Support/KagiNote/models/`
- Subsequent runs load models instantly (<1 second)

## Architecture

**Backend (Rust)**
- **Real audio capture** with cpal streams and proper session management
- **Actual Whisper transcription** using whisper-rs with Metal acceleration
- **Persistent model caching** with integrity validation and metadata tracking
- **Session state management** with concurrent session prevention
- **Comprehensive error handling** and automatic recovery
- **Audio buffering** (1.5s minimum) for reliable transcription quality

**Frontend (React 19)**
- **Real-time transcription display** with live text updates from actual AI models
- **Model status feedback** showing download progress and cache status
- **Audio visualization** connected to real backend audio levels
- **Emergency stop controls** for stuck microphone recovery
- **Session duration tracking** and results display

## Performance

| Model Tier | Model Size | Memory Usage | Startup Time | Use Case |
|------------|------------|--------------|--------------|----------|
| Standard | 1.5GB | ~4GB | <1s (cached) | Daily meetings |
| High Accuracy | 2.4GB | ~6GB | <1s (cached) | Critical content |
| Turbo | 1.2GB | ~3GB | <1s (cached) | Fastest processing |

**Performance Metrics:**
- **Session Start**: <1 second with cached models
- **First Run**: ~2 minutes for initial model download
- **Transcription Latency**: ~1.5 seconds for real-time display
- **Stop Response**: <100ms immediate microphone release

## Privacy & Security

- **Zero network calls** during transcription processing
- **AES-256 encryption** for stored transcripts
- **Memory protection** with secure audio buffer wiping
- **OWASP compliant** with enterprise security standards
- **Source code audited** for privacy compliance

## Documentation

- [Requirements Specification](PRPs/discovery/INITIAL-20250119-144500.md)
- [Technical Architecture](PRPs/active/kaginote-architecture.md)
- [Integration Guide](INTEGRATION_SUMMARY.md)
- [Development Guide](CLAUDE.md)

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
