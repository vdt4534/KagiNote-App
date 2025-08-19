# KagiNote

**Privacy-focused meeting transcription that runs entirely on your device.**

KagiNote is a desktop application built with Tauri v2 that provides real-time meeting transcription with complete privacy - no audio data ever leaves your machine. Supports 100+ languages with optimized performance for English and Japanese.

## Features

- **🔒 Privacy First**: 100% local processing, zero network calls during transcription
- **🌐 Multilingual**: Supports 100+ languages with specialized optimization for English/Japanese
- **⚡ Real-time**: Sub-second latency with streaming transcription display
- **👥 Speaker Detection**: Automatic speaker diarization for multi-participant meetings
- **🎛️ Quality Tiers**: Choose between Standard, High Accuracy, and Turbo modes
- **📤 Multiple Exports**: TXT, SRT, VTT, JSON formats for maximum compatibility
- **🖥️ Cross-platform**: Native desktop app for Windows, macOS, and Linux

## Quick Start

### Prerequisites
- Node.js 18+ and pnpm
- Rust 1.75+ 
- 16GB+ RAM recommended for optimal performance

### Installation
```bash
git clone <repository-url>
cd KagiNote
pnpm install
```

### Development
```bash
# Start development server with hot reload
npm run tauri dev

# Build for production
npm run tauri build
```

## Architecture

**Backend (Rust)**
- Real-time audio capture with `cpal`
- Voice Activity Detection using Silero-VAD v5
- Multi-tier Whisper ASR engines (Medium/Large-v3/Turbo)
- Speaker diarization and language detection

**Frontend (React 19)**
- Real-time transcription display with WaveSurfer.js visualization
- Settings panel for model selection and configuration
- Export functionality with progress tracking
- Accessibility-compliant interface with Radix UI

## Performance

| Model Tier | RTF Target | Memory Usage | Languages | Use Case |
|------------|------------|--------------|-----------|----------|
| Standard | ≤1.0× | ~4GB | 99 | Daily meetings |
| High Accuracy | ≤2.0× | ~6GB | 100 | Critical content |
| Turbo | ≤0.8× | ~3GB | 100 | GPU-optimized |

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
- 6-core CPU (Intel 10th gen / AMD Ryzen 3000+)
- 16GB RAM
- 8GB storage for models

**Recommended**
- 8-core CPU with AVX2
- 24GB RAM
- GPU with 6GB+ VRAM (RTX 3060 or better)

## License

[License details]

## Support

For issues and feature requests, please use the GitHub issue tracker.
