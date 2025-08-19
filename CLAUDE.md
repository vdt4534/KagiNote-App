# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

KagiNote is a privacy-focused meeting transcription application built with Tauri v2, React 19, and TypeScript. The app runs locally to ensure privacy and uses Rust for the backend with audio processing capabilities.

## Architecture

**Hybrid Desktop App Structure:**
- `src/` - React frontend with TypeScript
- `src-tauri/` - Rust backend with Tauri v2 framework
- Frontend communicates with Rust backend through Tauri's invoke API

**Key Dependencies:**
- **Frontend**: React 19, Radix UI themes, WaveSurfer.js for audio visualization
- **Backend**: Tauri v2, cpal for audio capture, hound for audio file handling, tokio for async operations
- **Build Tools**: Vite for frontend bundling, TypeScript compiler

## Development Commands

**Frontend Development:**
```bash
npm run dev          # Start Vite dev server (port 1420)
npm run build        # Build frontend (tsc + vite build)
npm run preview      # Preview production build
```

**Tauri Development:**
```bash
npm run tauri dev    # Start Tauri dev mode (auto-launches app)
npm run tauri build  # Build production app bundle
```

**Package Management:**
```bash
pnpm install         # Install dependencies (uses pnpm)
```

## Audio Processing Architecture

The app is designed for audio capture and processing:
- Rust backend uses `cpal` for cross-platform audio capture
- `hound` library handles WAV file operations
- `tokio` provides async runtime for audio streaming
- WaveSurfer.js in frontend provides audio visualization

## Tauri Configuration

- App identifier: `com.david.kaginote`
- Default window: 800x600px
- Development server runs on port 1420
- CSP is disabled for development flexibility
- Supports all bundle targets (macOS, Windows, Linux)

## Project Structure Notes

- Tauri configuration in `src-tauri/tauri.conf.json`
- Rust library entry point: `src-tauri/src/lib.rs`
- Vite configured specifically for Tauri development with HMR on port 1421
- Icons are provided for all platforms in `src-tauri/icons/`

## Development Notes

- The app currently has template code (`greet` command) that should be replaced with transcription functionality
- Audio processing dependencies are already configured but not yet implemented
- Frontend uses modern React 19 with functional components and hooks
- TypeScript strict mode enabled