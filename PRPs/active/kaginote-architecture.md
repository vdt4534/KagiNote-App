# KagiNote Technical Architecture & Test Specifications

**Project:** KagiNote Privacy-First Meeting Transcription System  
**Architecture Version:** 1.0  
**Date:** January 19, 2025  
**Purpose:** Complete technical design enabling parallel TDD implementation

---

## 1. System Architecture

### 1.1 Component Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                    React Frontend (TypeScript)                  │
├─────────────────────────────────────────────────────────────────┤
│  UI Components │ State Management │ Audio Visualization │ Export │
│     (Radix)    │   (Zustand)      │   (WaveSurfer.js)   │ Mgmt   │
├─────────────────────────────────────────────────────────────────┤
│                    Tauri Bridge Layer                           │
├─────────────────────────────────────────────────────────────────┤
│                     Rust Backend Core                           │
├─────────────────────────────────────────────────────────────────┤
│ Audio    │  VAD      │ ASR Engine │ Diarization │ Export  │ System│
│ Capture  │ (Silero)  │ Manager    │ Engine      │ Manager │ Monitor│
├─────────────────────────────────────────────────────────────────┤
│ Platform Audio APIs │ ONNX Runtime │ Model Management │ File I/O │
│ WASAPI/CoreAudio     │ GPU/CPU      │ CTranslate2      │          │
└─────────────────────────────────────────────────────────────────┘
```

### 1.2 Technology Stack

**Frontend Stack:**
- React 19 with functional components and hooks
- TypeScript for type safety
- Radix UI for accessible components
- Zustand for state management
- WaveSurfer.js for audio visualization
- Vite for development and building

**Backend Stack:**
- Rust with Tauri v2 framework
- cpal for cross-platform audio capture
- hound for WAV file operations
- tokio for async runtime
- onnxruntime for AI model inference
- CTranslate2 for Whisper optimization
- serde for serialization

**AI Models:**
- Whisper (Medium/Large-v3/Turbo) for ASR
- ReazonSpeech k2-v2 for Japanese optimization
- Silero-VAD v5 for voice activity detection
- sherpa-onnx for CPU diarization
- pyannote 3.1 for GPU diarization

---

## 2. API Specification

### 2.1 Tauri Command Interface

#### Core Transcription Commands

```rust
#[tauri::command]
pub async fn start_transcription(
    config: TranscriptionConfig,
    state: tauri::State<'_, AppState>
) -> Result<TranscriptionSessionId, TranscriptionError>;
```

**Request Schema:**
```typescript
interface TranscriptionConfig {
  qualityTier: 'standard' | 'high-accuracy' | 'turbo';
  languages: string[]; // ISO 639-1 codes, e.g., ['en', 'ja']
  enableSpeakerDiarization: boolean;
  enableTwoPassRefinement: boolean;
  audioSources: {
    microphone: boolean;
    systemAudio: boolean;
  };
  vadThreshold: number; // 0.0 - 1.0
  customVocabulary?: string[];
}

type TranscriptionSessionId = string; // UUID
```

**Response Codes:**
- 200: Session started successfully
- 400: Invalid configuration
- 503: Hardware insufficient for requested quality tier
- 500: Internal audio capture or model initialization error

**Test Scenarios:**
1. **Valid configuration with standard tier** → Returns session ID within 2 seconds
2. **Invalid language code** → Returns 400 with specific error message
3. **Hardware insufficient for high-accuracy** → Returns 503 with alternative suggestions
4. **Audio capture blocked** → Returns 500 with fallback options
5. **Multiple sessions attempt** → Returns 400 (only one session allowed)

---

```rust
#[tauri::command]
pub async fn stop_transcription(
    session_id: TranscriptionSessionId,
    state: tauri::State<'_, AppState>
) -> Result<FinalTranscriptionResult, TranscriptionError>;
```

**Response Schema:**
```typescript
interface FinalTranscriptionResult {
  sessionId: string;
  totalDuration: number; // seconds
  segments: TranscriptionSegment[];
  speakers: SpeakerProfile[];
  qualityMetrics: TranscriptionQualityMetrics;
  processingTimeMs: number;
}
```

**Test Scenarios:**
1. **Normal session stop** → Returns complete results within 5 seconds
2. **Stop non-existent session** → Returns 404 error
3. **Stop already stopped session** → Returns 409 conflict
4. **Stop during processing** → Gracefully completes current segment
5. **Emergency stop during high CPU** → Stops immediately with partial results

---

```rust
#[tauri::command]
pub async fn get_real_time_results(
    session_id: TranscriptionSessionId,
    since_timestamp: Option<f64>,
    state: tauri::State<'_, AppState>
) -> Result<Vec<TranscriptionSegment>, TranscriptionError>;
```

**Test Scenarios:**
1. **Get latest segments** → Returns new segments since timestamp
2. **Get all segments** → Returns complete session segments
3. **No new segments** → Returns empty array
4. **Invalid session ID** → Returns 404 error
5. **Concurrent requests** → Handles multiple simultaneous calls

#### Export Commands

```rust
#[tauri::command]
pub async fn export_transcription(
    session_id: TranscriptionSessionId,
    format: ExportFormat,
    options: ExportOptions,
    state: tauri::State<'_, AppState>
) -> Result<ExportResult, ExportError>;
```

**Request Schema:**
```typescript
interface ExportOptions {
  includeSpeakerLabels: boolean;
  includeTimestamps: boolean;
  includeConfidenceScores: boolean;
  separateFilePerSpeaker?: boolean;
  customTemplate?: string; // For advanced formatting
}

enum ExportFormat {
  TXT = 'txt',
  SRT = 'srt',
  VTT = 'vtt',
  JSON = 'json',
  TTML = 'ttml',
  CSV = 'csv'
}

interface ExportResult {
  filePaths: string[]; // Absolute paths to exported files
  format: ExportFormat;
  totalSizeBytes: number;
  exportTimeMs: number;
}
```

**Test Scenarios:**
1. **Export TXT with speakers** → Creates formatted text file
2. **Export SRT with timing** → Creates valid SubRip subtitle file
3. **Export JSON with metadata** → Creates complete JSON with quality metrics
4. **Separate speaker files** → Creates multiple files, one per speaker
5. **Invalid session for export** → Returns 404 with clear error message

### 2.2 Real-Time Event System

#### Event Definitions

```typescript
// Real-time transcription updates
interface TranscriptionUpdateEvent {
  sessionId: string;
  segment: TranscriptionSegment;
  updateType: 'new' | 'refined' | 'speaker_update';
  processingPass: 1 | 2;
}

// System performance updates
interface SystemStatusEvent {
  thermalStatus: {
    temperature: number;
    riskLevel: 'low' | 'medium' | 'high' | 'critical';
  };
  memoryUsage: {
    used: number;
    available: number;
    percentage: number;
  };
  processingMetrics: {
    realTimeFactor: number;
    averageLatency: number;
    queuedSegments: number;
  };
}

// Error and warning events
interface ErrorEvent {
  sessionId?: string;
  errorType: 'audio_capture' | 'model_inference' | 'thermal_throttle' | 'memory_pressure';
  severity: 'warning' | 'error' | 'critical';
  message: string;
  suggestedAction?: string;
  timestamp: number;
}
```

**Event Listening Pattern:**
```typescript
import { listen } from '@tauri-apps/api/event';

// Listen for real-time transcription updates
const unsubscribe = await listen<TranscriptionUpdateEvent>(
  'transcription-update',
  (event) => {
    updateTranscriptionDisplay(event.payload);
  }
);
```

---

## 3. Data Models

### 3.1 Core TypeScript Interfaces

#### Audio Processing Models

```typescript
interface AudioData {
  sampleRate: number; // Always 16000
  channels: 1 | 2;
  samples: Float32Array;
  timestamp: number; // Unix timestamp in milliseconds
  sourceChannel: 'microphone' | 'system' | 'mixed' | 'unknown';
  durationSeconds: number;
}

interface SpeechSegment {
  startTime: number; // seconds from session start
  endTime: number;
  audioData: AudioData;
  vadConfidence: number; // 0.0 - 1.0
  preprocessed: boolean;
}
```

#### Transcription Models

```typescript
interface TranscriptionSegment {
  id: string; // UUID
  startTime: number; // seconds from session start
  endTime: number;
  text: string;
  speakerId?: string;
  language: string; // ISO 639-1
  confidence: number; // 0.0 - 1.0
  words: WordTiming[];
  processingPass: 1 | 2;
  createdAt: number; // Unix timestamp
  updatedAt?: number;
}

interface WordTiming {
  word: string;
  startTime: number;
  endTime: number;
  confidence: number;
}

interface SpeakerProfile {
  id: string; // UUID
  name?: string; // User-assigned
  embedding: number[]; // 512-dimensional ECAPA-TDNN vector
  languagePreference: string;
  totalSpeechTime: number; // seconds
  segmentCount: number;
  createdAt: number;
  lastActive: number;
  isPersistent: boolean; // Save across sessions
}
```

#### Configuration Models

```typescript
interface SessionSettings {
  qualityTier: QualityTier;
  languages: string[];
  enableSpeakerDiarization: boolean;
  enableTwoPassRefinement: boolean;
  vadSettings: {
    threshold: number;
    minSpeechDuration: number; // milliseconds
    maxSpeechDuration: number;
    paddingDuration: number;
  };
  thermalManagement: {
    enabled: boolean;
    throttleTemperature: number; // Celsius
    shutdownTemperature: number;
  };
}

enum QualityTier {
  STANDARD = 'standard',        // Whisper Medium INT8
  HIGH_ACCURACY = 'high-accuracy', // Whisper Large-v3
  TURBO = 'turbo'              // Whisper Large-v3-Turbo
}

interface TranscriptionSession {
  id: string;
  startTime: Date;
  endTime?: Date;
  settings: SessionSettings;
  segments: TranscriptionSegment[];
  speakers: SpeakerProfile[];
  qualityMetrics: TranscriptionQualityMetrics;
  metadata: SessionMetadata;
}
```

#### Quality Metrics Models

```typescript
interface TranscriptionQualityMetrics {
  overallConfidence: number; // Average confidence across all segments
  speakerConsistency: number; // Percentage of stable speaker assignments
  languageDetectionAccuracy: number;
  realTimeFactor: number; // Processing time / audio duration
  memoryUsagePeak: number; // Bytes
  cpuUsageAverage: number; // Percentage
  thermalEvents: ThermalEvent[];
  processingLatency: {
    pass1Average: number; // milliseconds
    pass2Average: number;
    p95: number;
    p99: number;
  };
}

interface ThermalEvent {
  timestamp: number;
  temperature: number;
  action: 'throttle' | 'model_downgrade' | 'emergency_stop';
  durationMs: number;
}
```

### 3.2 Rust Data Structures

#### Core Audio Types

```rust
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioData {
    pub sample_rate: u32,           // Always 16000
    pub channels: u8,               // 1 or 2
    pub samples: Vec<f32>,          // Normalized [-1.0, 1.0]
    pub timestamp: u64,             // Unix timestamp millis
    pub source_channel: AudioSource,
    pub duration_seconds: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AudioSource {
    Microphone,
    SystemAudio,
    Mixed,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeechSegment {
    pub start_time: f64,
    pub end_time: f64,
    pub audio_data: AudioData,
    pub vad_confidence: f32,
    pub preprocessed: bool,
}
```

#### Transcription Types

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionSegment {
    pub id: Uuid,
    pub start_time: f64,
    pub end_time: f64,
    pub text: String,
    pub speaker_id: Option<Uuid>,
    pub language: String,
    pub confidence: f32,
    pub words: Vec<WordTiming>,
    pub processing_pass: PassType,
    pub created_at: SystemTime,
    pub updated_at: Option<SystemTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WordTiming {
    pub word: String,
    pub start_time: f64,
    pub end_time: f64,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PassType {
    Pass1RealTime,
    Pass2Refined,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeakerProfile {
    pub id: Uuid,
    pub name: Option<String>,
    pub embedding: Vec<f32>,        // 512-dimensional ECAPA-TDNN
    pub language_preference: String,
    pub total_speech_time: f64,
    pub segment_count: u32,
    pub created_at: SystemTime,
    pub last_active: SystemTime,
    pub is_persistent: bool,
}
```

#### Configuration Types

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionConfig {
    pub quality_tier: QualityTier,
    pub languages: Vec<String>,
    pub enable_speaker_diarization: bool,
    pub enable_two_pass_refinement: bool,
    pub audio_sources: AudioSourceConfig,
    pub vad_threshold: f32,
    pub custom_vocabulary: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QualityTier {
    Standard,       // Whisper Medium INT8
    HighAccuracy,   // Whisper Large-v3
    Turbo,         // Whisper Large-v3-Turbo
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioSourceConfig {
    pub microphone: bool,
    pub system_audio: bool,
}
```

### 3.3 Data Validation with Zod (Frontend)

```typescript
import { z } from 'zod';

const TranscriptionConfigSchema = z.object({
  qualityTier: z.enum(['standard', 'high-accuracy', 'turbo']),
  languages: z.array(z.string().length(2)), // ISO 639-1 codes
  enableSpeakerDiarization: z.boolean(),
  enableTwoPassRefinement: z.boolean(),
  audioSources: z.object({
    microphone: z.boolean(),
    systemAudio: z.boolean()
  }).refine(sources => sources.microphone || sources.systemAudio, {
    message: "At least one audio source must be enabled"
  }),
  vadThreshold: z.number().min(0).max(1),
  customVocabulary: z.array(z.string()).optional()
});

const TranscriptionSegmentSchema = z.object({
  id: z.string().uuid(),
  startTime: z.number().min(0),
  endTime: z.number(),
  text: z.string().min(1),
  speakerId: z.string().uuid().optional(),
  language: z.string().length(2),
  confidence: z.number().min(0).max(1),
  words: z.array(z.object({
    word: z.string(),
    startTime: z.number(),
    endTime: z.number(),
    confidence: z.number().min(0).max(1)
  })),
  processingPass: z.union([z.literal(1), z.literal(2)]),
  createdAt: z.number(),
  updatedAt: z.number().optional()
}).refine(segment => segment.endTime > segment.startTime, {
  message: "End time must be greater than start time"
});

// Type inference from Zod schemas
export type TranscriptionConfig = z.infer<typeof TranscriptionConfigSchema>;
export type TranscriptionSegment = z.infer<typeof TranscriptionSegmentSchema>;
```

---

## 4. Test Specifications for QA Engineer

### 4.1 Unit Test Requirements

#### 4.1.1 Audio Processing Tests

**File: `tests/unit/audio/audio_capture.test.ts`**

```typescript
describe('Audio Capture System', () => {
  test('should initialize audio capture with correct parameters', async () => {
    const config = {
      sampleRate: 16000,
      channels: 1,
      bufferSize: 1024
    };
    
    const capture = await AudioCapture.initialize(config);
    
    expect(capture.getSampleRate()).toBe(16000);
    expect(capture.getChannels()).toBe(1);
    expect(capture.isReady()).toBe(true);
  });

  test('should handle audio permission denial gracefully', async () => {
    // Mock permission denial
    mockAudioPermissions(false);
    
    await expect(AudioCapture.initialize({}))
      .rejects
      .toThrow('Audio permissions denied');
  });

  test('should maintain audio quality within acceptable bounds', async () => {
    const capture = await AudioCapture.initialize({ sampleRate: 16000 });
    const testSignal = generateTestSignal(440, 1.0); // 440Hz sine wave
    
    const captured = await capture.processTestSignal(testSignal);
    const snr = calculateSNR(testSignal, captured);
    
    expect(snr).toBeGreaterThan(40); // >40dB SNR
  });

  test('should detect audio input device changes', async () => {
    const capture = await AudioCapture.initialize({});
    const changeHandler = jest.fn();
    
    capture.onDeviceChange(changeHandler);
    simulateDeviceChange();
    
    await waitFor(() => {
      expect(changeHandler).toHaveBeenCalledWith(
        expect.objectContaining({ deviceId: expect.any(String) })
      );
    });
  });

  test('should fallback to alternative capture method when primary fails', async () => {
    // Simulate WASAPI failure on Windows
    mockCaptureFail('wasapi');
    
    const capture = await AudioCapture.initialize({});
    
    expect(capture.getCurrentMethod()).toBe('wdm'); // Fallback method
    expect(capture.isReady()).toBe(true);
  });
});
```

**Expected Test Data:**
```typescript
export const AudioTestFactories = {
  createCleanSpeech: (duration: number) => ({
    samples: generateSineWave(440, duration, 16000),
    sampleRate: 16000,
    channels: 1,
    timestamp: Date.now(),
    sourceChannel: 'microphone' as const
  }),
  
  createNoisyConferenceCall: () => ({
    samples: mixAudio(
      generateSpeech("Hello everyone, let's start the meeting"),
      generateBackgroundNoise(-25) // -25dB background noise
    ),
    sampleRate: 16000,
    channels: 2,
    timestamp: Date.now(),
    sourceChannel: 'mixed' as const
  })
};
```

#### 4.1.2 VAD Processing Tests

**File: `tests/unit/vad/silero_vad.test.ts`**

```typescript
describe('Silero VAD Processing', () => {
  test('should detect speech with high accuracy', async () => {
    const vad = new SileroVAD({ threshold: 0.5 });
    const speechAudio = AudioTestFactories.createCleanSpeech(5.0);
    
    const result = await vad.detectSpeech(speechAudio);
    
    expect(result.hasSpeech).toBe(true);
    expect(result.confidence).toBeGreaterThan(0.8);
    expect(result.speechSegments.length).toBeGreaterThan(0);
  });

  test('should reject silence and background noise', async () => {
    const vad = new SileroVAD({ threshold: 0.5 });
    const silenceAudio = AudioTestFactories.createSilence(3.0);
    
    const result = await vad.detectSpeech(silenceAudio);
    
    expect(result.hasSpeech).toBe(false);
    expect(result.confidence).toBeLessThan(0.3);
  });

  test('should adapt threshold dynamically to noise levels', async () => {
    const vad = new SileroVAD({ adaptiveThreshold: true });
    
    // Test with increasing noise levels
    for (const noiseLevel of [-40, -30, -20, -15]) {
      const noisyAudio = AudioTestFactories.createSpeechWithNoise(noiseLevel);
      const result = await vad.detectSpeech(noisyAudio);
      
      expect(result.hasSpeech).toBe(true); // Should still detect speech
      expect(result.adaptedThreshold).toBeGreaterThan(0.3);
    }
  });

  test('should process real-time streams with low latency', async () => {
    const vad = new SileroVAD({ threshold: 0.5 });
    const streamProcessor = new AudioStreamProcessor(vad);
    
    const startTime = performance.now();
    const chunks = AudioTestFactories.createStreamingChunks(10); // 10 chunks
    
    for (const chunk of chunks) {
      await streamProcessor.processChunk(chunk);
    }
    
    const processingTime = performance.now() - startTime;
    const audioTime = chunks.length * 0.5 * 1000; // 0.5s per chunk
    const realTimeFactor = processingTime / audioTime;
    
    expect(realTimeFactor).toBeLessThan(0.1); // <0.1x real-time
  });

  test('should handle edge cases without crashing', async () => {
    const vad = new SileroVAD({ threshold: 0.5 });
    
    // Test empty audio
    await expect(vad.detectSpeech({ samples: [], sampleRate: 16000, channels: 1 }))
      .rejects.toThrow('Empty audio data');
    
    // Test clipped audio
    const clippedAudio = { samples: [1.5, -1.5, 2.0], sampleRate: 16000, channels: 1 };
    await expect(vad.detectSpeech(clippedAudio))
      .rejects.toThrow('Clipped audio detected');
    
    // Test invalid sample rate
    const invalidRateAudio = { samples: [0.1, 0.2, 0.3], sampleRate: 8000, channels: 1 };
    await expect(vad.detectSpeech(invalidRateAudio))
      .rejects.toThrow('Invalid sample rate');
  });
});
```

#### 4.1.3 ASR Engine Tests

**File: `tests/unit/asr/whisper_engine.test.ts`**

```typescript
describe('Whisper ASR Engine', () => {
  test('should achieve target WER for clean English speech', async () => {
    const engine = new WhisperEngine({ model: 'medium', language: 'en' });
    const testAudio = await loadGroundTruthAudio('clean_english_business.wav');
    const expectedText = "Good morning everyone, let's begin today's quarterly review meeting.";
    
    const result = await engine.transcribe(testAudio);
    const wer = calculateWordErrorRate(result.text, expectedText);
    
    expect(wer).toBeLessThan(0.12); // <12% WER requirement
    expect(result.confidence).toBeGreaterThan(0.85);
    expect(result.processingTimeMs).toBeLessThan(testAudio.durationSeconds * 1000); // Real-time
  });

  test('should handle multilingual input correctly', async () => {
    const engine = new WhisperEngine({ model: 'medium', language: 'auto' });
    const mixedAudio = AudioTestFactories.createMultilingualMeeting(['en', 'ja', 'es']);
    
    const result = await engine.transcribe(mixedAudio);
    
    expect(result.detectedLanguages).toContain('en');
    expect(result.detectedLanguages).toContain('ja');
    expect(result.languageSegments.length).toBeGreaterThan(1);
    
    // Verify each segment has correct language detection
    result.languageSegments.forEach(segment => {
      expect(segment.language).toMatch(/^(en|ja|es)$/);
      expect(segment.confidence).toBeGreaterThan(0.7);
    });
  });

  test('should optimize performance for different quality tiers', async () => {
    const testAudio = AudioTestFactories.createCleanSpeech(60); // 1 minute
    
    const standardEngine = new WhisperEngine({ model: 'medium-int8', tier: 'standard' });
    const turboEngine = new WhisperEngine({ model: 'large-v3-turbo', tier: 'turbo' });
    
    const [standardResult, turboResult] = await Promise.all([
      standardEngine.transcribe(testAudio),
      turboEngine.transcribe(testAudio)
    ]);
    
    // Turbo should be significantly faster
    expect(turboResult.processingTimeMs).toBeLessThan(standardResult.processingTimeMs * 0.5);
    
    // Both should meet accuracy requirements
    expect(standardResult.confidence).toBeGreaterThan(0.8);
    expect(turboResult.confidence).toBeGreaterThan(0.8);
  });

  test('should handle context-aware processing', async () => {
    const engine = new WhisperEngine({ model: 'medium', enableContext: true });
    const contextSegments = [
      { text: "We're discussing machine learning algorithms today.", timestamp: 0 },
      { text: "The neural network architecture is crucial.", timestamp: 30 }
    ];
    
    const testAudio = AudioTestFactories.createContextualSpeech(
      "The algorithms we mentioned earlier are performing well."
    );
    
    const result = await engine.transcribeWithContext(testAudio, contextSegments);
    
    expect(result.text).toContain("algorithms");
    expect(result.contextImprovementScore).toBeGreaterThan(0.1); // >10% improvement
  });

  test('should gracefully handle model loading failures', async () => {
    // Test with non-existent model path
    const engine = new WhisperEngine({ 
      model: 'medium', 
      modelPath: '/non/existent/path' 
    });
    
    await expect(engine.initialize())
      .rejects
      .toThrow('Model file not found');
    
    // Test with corrupted model file
    const corruptedEngine = new WhisperEngine({ 
      model: 'medium',
      modelPath: './tests/fixtures/corrupted_model.bin'
    });
    
    await expect(corruptedEngine.initialize())
      .rejects
      .toThrow('Model checksum validation failed');
  });
});
```

### 4.2 Integration Test Requirements

#### 4.2.1 Full Transcription Pipeline Tests

**File: `tests/integration/transcription_pipeline.test.ts`**

```typescript
describe('Complete Transcription Pipeline Integration', () => {
  test('should process complete meeting workflow end-to-end', async () => {
    const config: TranscriptionConfig = {
      qualityTier: 'standard',
      languages: ['en'],
      enableSpeakerDiarization: true,
      enableTwoPassRefinement: true,
      audioSources: { microphone: true, systemAudio: false },
      vadThreshold: 0.5
    };
    
    // Start transcription session
    const sessionId = await invoke<string>('start_transcription', { config });
    expect(sessionId).toMatch(/^[0-9a-f-]{36}$/); // UUID format
    
    // Simulate 30-minute meeting audio
    const meetingAudio = AudioTestFactories.createBusinessMeeting(1800); // 30 minutes
    const eventHandler = new TranscriptionEventHandler();
    
    // Listen for real-time updates
    const unsubscribe = await listen<TranscriptionUpdateEvent>(
      'transcription-update',
      eventHandler.handleUpdate
    );
    
    // Process meeting audio chunks
    for (const chunk of meetingAudio.chunks) {
      await simulateAudioInput(chunk);
      await wait(chunk.duration * 1000); // Simulate real-time
    }
    
    // Stop transcription
    const finalResult = await invoke<FinalTranscriptionResult>(
      'stop_transcription',
      { sessionId }
    );
    
    // Verify results quality
    expect(finalResult.segments.length).toBeGreaterThan(50); // Reasonable segmentation
    expect(finalResult.speakers.length).toBe(3); // Expected speaker count
    expect(finalResult.qualityMetrics.overallConfidence).toBeGreaterThan(0.8);
    expect(finalResult.qualityMetrics.realTimeFactor).toBeLessThan(1.0);
    
    // Verify two-pass processing
    const pass1Segments = finalResult.segments.filter(s => s.processingPass === 1);
    const pass2Segments = finalResult.segments.filter(s => s.processingPass === 2);
    expect(pass2Segments.length).toBeGreaterThan(0);
    
    // Verify speaker consistency
    const speakerSwitches = countSpeakerSwitches(finalResult.segments);
    expect(speakerSwitches).toBeLessThan(finalResult.segments.length * 0.1); // <10% switch rate
    
    unsubscribe();
  });

  test('should handle system resource pressure gracefully', async () => {
    const config: TranscriptionConfig = {
      qualityTier: 'high-accuracy',
      languages: ['en', 'ja'],
      enableSpeakerDiarization: true,
      enableTwoPassRefinement: true,
      audioSources: { microphone: true, systemAudio: true },
      vadThreshold: 0.5
    };
    
    // Monitor system resources
    const resourceMonitor = new SystemResourceMonitor();
    resourceMonitor.startMonitoring();
    
    const sessionId = await invoke<string>('start_transcription', { config });
    
    // Simulate high-load meeting with complex audio
    const complexAudio = AudioTestFactories.createHighLoadScenario({
      speakers: 8,
      languages: ['en', 'ja'],
      backgroundNoise: -20, // Significant noise
      crossTalk: true,
      duration: 3600 // 1 hour
    });
    
    let resourcePressureHandled = false;
    const errorHandler = await listen<ErrorEvent>('transcription-error', (event) => {
      if (event.payload.errorType === 'memory_pressure') {
        resourcePressureHandled = true;
        expect(event.payload.suggestedAction).toContain('reduce quality');
      }
    });
    
    // Process complex audio
    await processAudioScenario(complexAudio);
    
    const finalResult = await invoke<FinalTranscriptionResult>(
      'stop_transcription',
      { sessionId }
    );
    
    // System should have handled pressure without crashing
    expect(finalResult).toBeDefined();
    expect(resourceMonitor.peakMemoryUsage).toBeLessThan(8 * 1024 * 1024 * 1024); // <8GB
    
    if (resourcePressureHandled) {
      // Should have automatically reduced quality
      expect(finalResult.qualityMetrics.thermalEvents.length).toBeGreaterThan(0);
    }
    
    resourceMonitor.stop();
    errorHandler();
  });

  test('should maintain accuracy across different audio conditions', async () => {
    const testConditions = [
      { name: 'clean_office', noise: -40, speakers: 2 },
      { name: 'noisy_cafe', noise: -20, speakers: 3 },
      { name: 'phone_conference', noise: -25, speakers: 4, compressed: true },
      { name: 'large_room', noise: -30, speakers: 6, reverb: true }
    ];
    
    const results: Array<{ condition: string; wer: number; der: number }> = [];
    
    for (const condition of testConditions) {
      const config: TranscriptionConfig = {
        qualityTier: 'standard',
        languages: ['en'],
        enableSpeakerDiarization: true,
        enableTwoPassRefinement: true,
        audioSources: { microphone: true, systemAudio: false },
        vadThreshold: 0.5
      };
      
      const sessionId = await invoke<string>('start_transcription', { config });
      
      const testAudio = AudioTestFactories.createTestCondition(condition);
      await processAudioScenario(testAudio);
      
      const finalResult = await invoke<FinalTranscriptionResult>(
        'stop_transcription',
        { sessionId }
      );
      
      const wer = calculateWordErrorRate(finalResult, testAudio.groundTruth.text);
      const der = calculateDiarizationErrorRate(finalResult, testAudio.groundTruth.speakers);
      
      results.push({ condition: condition.name, wer, der });
      
      // Each condition should meet minimum quality standards
      expect(wer).toBeLessThan(0.15); // <15% WER
      expect(der).toBeLessThan(0.2);  // <20% DER
    }
    
    // Log results for analysis
    console.table(results);
  });
});
```

#### 4.2.2 Real-Time Performance Tests

**File: `tests/integration/real_time_performance.test.ts`**

```typescript
describe('Real-Time Performance Integration', () => {
  test('should maintain real-time processing under normal load', async () => {
    const performanceTracker = new PerformanceTracker();
    
    const config: TranscriptionConfig = {
      qualityTier: 'standard',
      languages: ['en'],
      enableSpeakerDiarization: true,
      enableTwoPassRefinement: true,
      audioSources: { microphone: true, systemAudio: false },
      vadThreshold: 0.5
    };
    
    const sessionId = await invoke<string>('start_transcription', { config });
    
    // Track real-time metrics
    const metricsHandler = await listen<SystemStatusEvent>('system-status', 
      (event) => performanceTracker.recordMetrics(event.payload));
    
    // Simulate 2-hour continuous meeting
    const continuousAudio = AudioTestFactories.createContinuousMeeting(7200); // 2 hours
    
    let segmentCount = 0;
    const updateHandler = await listen<TranscriptionUpdateEvent>('transcription-update',
      (event) => {
        segmentCount++;
        
        // Track latency for each segment
        const latency = Date.now() - event.payload.segment.createdAt;
        performanceTracker.recordLatency(latency);
        
        // Verify real-time constraint
        expect(latency).toBeLessThan(2000); // <2s latency
      });
    
    // Process continuous audio
    await processAudioScenario(continuousAudio);
    
    const finalResult = await invoke<FinalTranscriptionResult>(
      'stop_transcription',
      { sessionId }
    );
    
    // Verify real-time performance
    const metrics = performanceTracker.getMetrics();
    expect(metrics.averageRTF).toBeLessThan(0.8); // <0.8x real-time
    expect(metrics.p95Latency).toBeLessThan(2500); // p95 <2.5s
    expect(metrics.p99Latency).toBeLessThan(4000); // p99 <4s
    
    // Verify no processing backlogs
    expect(metrics.maxQueuedSegments).toBeLessThan(10);
    
    // Verify system stability
    expect(segmentCount).toBeGreaterThan(1000); // Should have processed many segments
    expect(finalResult.qualityMetrics.overallConfidence).toBeGreaterThan(0.8);
    
    metricsHandler();
    updateHandler();
  });

  test('should handle concurrent sessions without degradation', async () => {
    // Test system limits - should reject concurrent sessions
    const config: TranscriptionConfig = {
      qualityTier: 'standard',
      languages: ['en'],
      enableSpeakerDiarization: false, // Lighter load
      enableTwoPassRefinement: false,
      audioSources: { microphone: true, systemAudio: false },
      vadThreshold: 0.5
    };
    
    // Start first session
    const session1Id = await invoke<string>('start_transcription', { config });
    expect(session1Id).toBeDefined();
    
    // Attempt second session - should be rejected
    await expect(invoke<string>('start_transcription', { config }))
      .rejects
      .toThrow('Only one active transcription session allowed');
    
    // Stop first session
    await invoke<FinalTranscriptionResult>('stop_transcription', { sessionId: session1Id });
    
    // Now second session should be allowed
    const session2Id = await invoke<string>('start_transcription', { config });
    expect(session2Id).toBeDefined();
    
    await invoke<FinalTranscriptionResult>('stop_transcription', { sessionId: session2Id });
  });
});
```

### 4.3 End-to-End Test Requirements

#### 4.3.1 User Workflow Tests

**File: `tests/e2e/user_workflows.e2e.test.ts`**

```typescript
import { test, expect } from '@playwright/test';

test.describe('Complete User Workflows', () => {
  test('new user setup and first transcription', async ({ page }) => {
    // Launch application
    await page.goto('http://localhost:1420');
    
    // Verify initial load
    await expect(page.getByText('KagiNote')).toBeVisible();
    
    // System requirements check should pass
    await expect(page.getByTestId('system-check-passed')).toBeVisible({ timeout: 10000 });
    
    // Hardware detection should complete
    await expect(page.getByTestId('hardware-detected')).toBeVisible();
    const recommendedTier = await page.getByTestId('recommended-tier').textContent();
    expect(['standard', 'high-accuracy', 'turbo']).toContain(recommendedTier);
    
    // Audio setup
    await page.getByTestId('setup-audio').click();
    
    // Grant audio permissions (automated in test)
    await mockAudioPermissions(page, true);
    
    // Audio test should pass
    await expect(page.getByTestId('audio-test-passed')).toBeVisible({ timeout: 5000 });
    
    // Start transcription with default settings
    await page.getByTestId('start-recording').click();
    
    // Should see recording indicator
    await expect(page.getByTestId('recording-active')).toBeVisible();
    
    // Simulate test audio input
    await simulateAudioInput(page, AudioTestFactories.createShortSpeech());
    
    // Should see transcription text appear
    await expect(page.getByTestId('transcription-text')).toContainText(
      /hello|test|meeting/i, 
      { timeout: 3000 }
    );
    
    // Stop recording
    await page.getByTestId('stop-recording').click();
    
    // Should show completion status
    await expect(page.getByTestId('transcription-complete')).toBeVisible();
    
    // Export functionality
    await page.getByTestId('export-dropdown').click();
    await page.getByTestId('export-txt').click();
    
    // Should show export success
    await expect(page.getByTestId('export-success')).toBeVisible({ timeout: 30000 });
    
    // Verify exported file exists
    const exportPath = await page.getByTestId('export-path').textContent();
    expect(exportPath).toMatch(/\.txt$/);
  });

  test('power user multilingual meeting scenario', async ({ page }) => {
    await page.goto('http://localhost:1420');
    
    // Skip setup (assume returning user)
    await page.getByTestId('skip-setup').click();
    
    // Configure advanced settings
    await page.getByTestId('advanced-settings').click();
    
    // Set quality tier
    await page.getByTestId('quality-tier').selectOption('high-accuracy');
    
    // Enable multilingual support
    await page.getByTestId('language-en').check();
    await page.getByTestId('language-ja').check();
    
    // Enable speaker diarization
    await page.getByTestId('enable-diarization').check();
    
    // Enable two-pass refinement
    await page.getByTestId('enable-refinement').check();
    
    // Add custom vocabulary
    await page.getByTestId('custom-vocab-input').fill('Kubernetes,microservices,DevOps');
    await page.getByTestId('add-vocab').click();
    
    // Start advanced transcription
    await page.getByTestId('start-recording').click();
    
    // Simulate multilingual meeting
    const multilingualAudio = AudioTestFactories.createMultilingualBusiness();
    await simulateAudioInput(page, multilingualAudio);
    
    // Should detect language switches
    await expect(page.getByTestId('language-indicator')).toContainText('EN', { timeout: 5000 });
    await expect(page.getByTestId('language-indicator')).toContainText('JA', { timeout: 10000 });
    
    // Should show multiple speakers
    await expect(page.getByTestId('speaker-count')).toContainText(/[2-4] speakers/);
    
    // Should show two-pass refinement
    await expect(page.getByTestId('pass-1-indicator')).toBeVisible();
    await expect(page.getByTestId('pass-2-indicator')).toBeVisible({ timeout: 15000 });
    
    // Custom vocabulary should be recognized
    await expect(page.getByTestId('transcription-text')).toContainText('Kubernetes');
    
    // Stop recording
    await page.getByTestId('stop-recording').click();
    
    // Review final results
    await expect(page.getByTestId('final-confidence')).toContainText(/[8-9][0-9]%/); // >80%
    await expect(page.getByTestId('processing-time')).toContainText(/RTF: 0\.[0-9]/); // <1.0 RTF
    
    // Export in multiple formats
    await page.getByTestId('export-json').click();
    await expect(page.getByTestId('json-export-success')).toBeVisible();
    
    await page.getByTestId('export-srt').click();
    await expect(page.getByTestId('srt-export-success')).toBeVisible();
  });

  test('system resource management under stress', async ({ page }) => {
    await page.goto('http://localhost:1420');
    
    // Monitor system resources
    const resourceMonitor = await page.evaluate(() => {
      return new Promise((resolve) => {
        window.startResourceMonitoring(resolve);
      });
    });
    
    // Configure high-load scenario
    await page.getByTestId('advanced-settings').click();
    await page.getByTestId('quality-tier').selectOption('high-accuracy');
    await page.getByTestId('enable-diarization').check();
    await page.getByTestId('max-speakers').fill('8');
    
    // Start transcription
    await page.getByTestId('start-recording').click();
    
    // Simulate high-load audio (many speakers, noise, long duration)
    const highLoadAudio = AudioTestFactories.createStressTestScenario();
    await simulateAudioInput(page, highLoadAudio);
    
    // Should show system monitoring
    await expect(page.getByTestId('system-monitor')).toBeVisible();
    
    // Monitor for thermal management
    const thermalAlert = page.getByTestId('thermal-alert');
    if (await thermalAlert.isVisible()) {
      // Should show quality reduction notification
      await expect(page.getByTestId('quality-reduced')).toBeVisible();
      
      // Should continue processing at reduced quality
      await expect(page.getByTestId('transcription-text')).toBeVisible();
    }
    
    // Monitor for memory pressure
    const memoryAlert = page.getByTestId('memory-alert');
    if (await memoryAlert.isVisible()) {
      // Should show memory optimization notification
      await expect(page.getByTestId('memory-optimized')).toBeVisible();
      
      // Should continue processing
      await expect(page.getByTestId('transcription-active')).toBeVisible();
    }
    
    // Complete transcription despite stress
    await page.getByTestId('stop-recording').click();
    await expect(page.getByTestId('transcription-complete')).toBeVisible({ timeout: 60000 });
    
    // Verify system recovered
    await expect(page.getByTestId('system-status')).toContainText('Normal');
    
    // Verify results quality despite stress
    const finalConfidence = await page.getByTestId('final-confidence').textContent();
    expect(parseFloat(finalConfidence!)).toBeGreaterThan(0.7); // >70% under stress
  });
});
```

### 4.4 Performance Benchmark Tests

#### 4.4.1 Model Performance Benchmarks

**File: `tests/benchmarks/model_performance.bench.ts`**

```typescript
import { performance } from 'perf_hooks';

describe('Model Performance Benchmarks', () => {
  const benchmarkAudio = AudioTestFactories.createBenchmarkSuite();
  
  benchmark('Whisper Medium CPU performance', async () => {
    const engine = new WhisperEngine({
      model: 'medium',
      device: 'cpu',
      numThreads: 4
    });
    
    const results = [];
    
    for (const testCase of benchmarkAudio.standard) {
      const startTime = performance.now();
      const result = await engine.transcribe(testCase.audio);
      const endTime = performance.now();
      
      const processingTime = (endTime - startTime) / 1000; // seconds
      const rtf = processingTime / testCase.audio.durationSeconds;
      const wer = calculateWordErrorRate(result.text, testCase.groundTruth);
      
      results.push({
        duration: testCase.audio.durationSeconds,
        processingTime,
        rtf,
        wer,
        confidence: result.confidence
      });
      
      // Performance requirements
      expect(rtf).toBeLessThan(1.0); // Real-time requirement
      expect(wer).toBeLessThan(0.12); // <12% WER requirement
    }
    
    // Aggregate performance
    const avgRTF = results.reduce((sum, r) => sum + r.rtf, 0) / results.length;
    const avgWER = results.reduce((sum, r) => sum + r.wer, 0) / results.length;
    
    console.table(results);
    console.log(`Average RTF: ${avgRTF.toFixed(3)}`);
    console.log(`Average WER: ${(avgWER * 100).toFixed(1)}%`);
    
    // Benchmark assertions
    expect(avgRTF).toBeLessThan(0.8); // Target <0.8x real-time
    expect(avgWER).toBeLessThan(0.10); // Target <10% WER
  });

  benchmark('GPU vs CPU performance comparison', async () => {
    if (!hasGPUSupport()) {
      console.log('GPU not available, skipping GPU benchmarks');
      return;
    }
    
    const cpuEngine = new WhisperEngine({ model: 'medium', device: 'cpu' });
    const gpuEngine = new WhisperEngine({ model: 'large-v3-turbo', device: 'gpu' });
    
    const testAudio = AudioTestFactories.createStandardBenchmark(300); // 5 minutes
    
    // CPU benchmark
    const cpuStart = performance.now();
    const cpuResult = await cpuEngine.transcribe(testAudio);
    const cpuTime = performance.now() - cpuStart;
    
    // GPU benchmark
    const gpuStart = performance.now();
    const gpuResult = await gpuEngine.transcribe(testAudio);
    const gpuTime = performance.now() - gpuStart;
    
    const speedup = cpuTime / gpuTime;
    const cpuRTF = cpuTime / 1000 / testAudio.durationSeconds;
    const gpuRTF = gpuTime / 1000 / testAudio.durationSeconds;
    
    console.log(`CPU RTF: ${cpuRTF.toFixed(3)}, GPU RTF: ${gpuRTF.toFixed(3)}`);
    console.log(`GPU Speedup: ${speedup.toFixed(2)}x`);
    
    // GPU should be significantly faster
    expect(speedup).toBeGreaterThan(3.0); // >3x speedup expected
    expect(gpuRTF).toBeLessThan(0.3); // Very fast processing
    
    // Both should maintain accuracy
    const cpuWER = calculateWordErrorRate(cpuResult.text, testAudio.groundTruth);
    const gpuWER = calculateWordErrorRate(gpuResult.text, testAudio.groundTruth);
    
    expect(cpuWER).toBeLessThan(0.12);
    expect(gpuWER).toBeLessThan(0.12);
  });

  benchmark('Memory usage profiling', async () => {
    const memoryProfiler = new MemoryProfiler();
    
    const engine = new WhisperEngine({ model: 'medium' });
    
    // Baseline memory
    const baselineMemory = memoryProfiler.getCurrentUsage();
    
    // Load model
    await engine.initialize();
    const modelLoadedMemory = memoryProfiler.getCurrentUsage();
    const modelMemory = modelLoadedMemory - baselineMemory;
    
    // Process various audio lengths
    const audioLengths = [30, 60, 300, 1800, 3600]; // 30s to 1 hour
    const memoryUsages = [];
    
    for (const length of audioLengths) {
      const testAudio = AudioTestFactories.createCleanSpeech(length);
      
      const beforeProcessing = memoryProfiler.getCurrentUsage();
      await engine.transcribe(testAudio);
      const afterProcessing = memoryProfiler.getCurrentUsage();
      
      const processingMemory = afterProcessing - beforeProcessing;
      memoryUsages.push({ length, memory: processingMemory });
      
      // Force garbage collection
      global.gc && global.gc();
      await new Promise(resolve => setTimeout(resolve, 1000));
    }
    
    console.log(`Model Memory: ${formatBytes(modelMemory)}`);
    console.table(memoryUsages);
    
    // Memory requirements
    expect(modelMemory).toBeLessThan(2 * 1024 * 1024 * 1024); // <2GB for model
    
    // Processing memory should scale reasonably
    const maxProcessingMemory = Math.max(...memoryUsages.map(u => u.memory));
    expect(maxProcessingMemory).toBeLessThan(1 * 1024 * 1024 * 1024); // <1GB for processing
  });
});
```

---

## 5. Mock Data Factories

### 5.1 Audio Test Data Factory

```typescript
export class AudioTestFactory {
  static createCleanSpeech(durationSeconds: number): AudioData {
    const sampleRate = 16000;
    const samples = this.generateCleanSpeechSamples(durationSeconds, sampleRate);
    
    return {
      sampleRate,
      channels: 1,
      samples,
      timestamp: Date.now(),
      sourceChannel: 'microphone',
      durationSeconds
    };
  }
  
  static createBusinessMeeting(durationSeconds: number): ComplexAudioScenario {
    const speakers = [
      { name: 'Manager', voiceProfile: 'male-us-business', speechRatio: 0.4 },
      { name: 'Developer', voiceProfile: 'female-us-technical', speechRatio: 0.35 },
      { name: 'Designer', voiceProfile: 'male-uk-creative', speechRatio: 0.25 }
    ];
    
    const segments = this.generateMeetingSegments(durationSeconds, speakers);
    
    return {
      audio: this.mixSegmentsToAudio(segments),
      groundTruth: {
        text: segments.map(s => s.text).join(' '),
        speakers: segments.map(s => ({
          startTime: s.startTime,
          endTime: s.endTime,
          speakerId: s.speakerId,
          text: s.text
        })),
        languages: ['en']
      },
      segments,
      speakers
    };
  }
  
  static createMultilingualMeeting(): ComplexAudioScenario {
    const segments = [
      { text: "Good morning everyone, let's start our quarterly review.", language: 'en', speaker: 'speaker1', startTime: 0, endTime: 4 },
      { text: "おはようございます。今四半期の業績について話し合いましょう。", language: 'ja', speaker: 'speaker2', startTime: 4, endTime: 9 },
      { text: "Thank you. Our revenue this quarter exceeded expectations.", language: 'en', speaker: 'speaker1', startTime: 9, endTime: 13 },
      { text: "素晴らしいニュースです。具体的な数字を教えていただけますか？", language: 'ja', speaker: 'speaker2', startTime: 13, endTime: 18 },
    ];
    
    return this.createScenarioFromSegments(segments);
  }
  
  static createNoisyConferenceCall(): AudioData {
    const cleanSpeech = this.createCleanSpeech(60);
    const backgroundNoise = this.generateBackgroundNoise(-25, 60); // -25dB
    const mixedSamples = this.mixAudio(cleanSpeech.samples, backgroundNoise);
    
    return {
      ...cleanSpeech,
      samples: mixedSamples,
      sourceChannel: 'system'
    };
  }
  
  static createStressTestScenario(): ComplexAudioScenario {
    return {
      audio: this.createHighLoadAudio({
        speakers: 8,
        languages: ['en', 'ja', 'es'],
        backgroundNoise: -20,
        crossTalk: true,
        duration: 3600, // 1 hour
        technicalTerms: true
      }),
      groundTruth: this.generateStressTestGroundTruth(),
      complexity: 'extreme'
    };
  }
  
  private static generateCleanSpeechSamples(duration: number, sampleRate: number): Float32Array {
    // Generate synthetic speech-like audio with formant frequencies
    const samples = new Float32Array(duration * sampleRate);
    const fundamental = 150; // Hz
    const formants = [800, 1200, 2400]; // Hz
    
    for (let i = 0; i < samples.length; i++) {
      const t = i / sampleRate;
      let sample = 0;
      
      // Fundamental frequency with formant emphasis
      for (const formant of formants) {
        sample += 0.3 * Math.sin(2 * Math.PI * formant * t) * Math.exp(-t * 0.1);
      }
      
      // Add speech envelope
      const envelope = this.speechEnvelope(t, duration);
      samples[i] = sample * envelope * 0.3;
    }
    
    return samples;
  }
  
  private static speechEnvelope(t: number, duration: number): number {
    // Simulate natural speech patterns with pauses
    const speechRate = 2.5; // syllables per second
    const syllableTime = t * speechRate;
    const syllablePhase = syllableTime % 1;
    
    // Speech vs silence pattern
    const isSpeech = syllablePhase < 0.6 && (Math.floor(t * 0.5) % 3) < 2;
    
    return isSpeech ? Math.sin(Math.PI * syllablePhase / 0.6) : 0;
  }
  
  private static generateBackgroundNoise(dbLevel: number, duration: number): Float32Array {
    const sampleRate = 16000;
    const samples = new Float32Array(duration * sampleRate);
    const amplitude = Math.pow(10, dbLevel / 20);
    
    for (let i = 0; i < samples.length; i++) {
      samples[i] = amplitude * (Math.random() * 2 - 1);
    }
    
    return samples;
  }
  
  private static mixAudio(signal1: Float32Array, signal2: Float32Array): Float32Array {
    const length = Math.min(signal1.length, signal2.length);
    const mixed = new Float32Array(length);
    
    for (let i = 0; i < length; i++) {
      mixed[i] = Math.tanh(signal1[i] + signal2[i]); // Soft clipping
    }
    
    return mixed;
  }
}

export interface ComplexAudioScenario {
  audio: AudioData;
  groundTruth: {
    text: string;
    speakers: Array<{
      startTime: number;
      endTime: number;
      speakerId: string;
      text: string;
    }>;
    languages: string[];
  };
  segments?: any[];
  speakers?: any[];
  complexity?: 'low' | 'medium' | 'high' | 'extreme';
}
```

### 5.2 Transcription Test Data Factory

```typescript
export class TranscriptionTestFactory {
  static createBasicTranscriptionSegments(): TranscriptionSegment[] {
    return [
      {
        id: '550e8400-e29b-41d4-a716-446655440001',
        startTime: 0.0,
        endTime: 3.5,
        text: "Good morning everyone, let's begin today's meeting.",
        speakerId: '550e8400-e29b-41d4-a716-446655440010',
        language: 'en',
        confidence: 0.95,
        words: this.createWordTimings("Good morning everyone, let's begin today's meeting."),
        processingPass: 2,
        createdAt: Date.now(),
        updatedAt: Date.now() + 1000
      },
      {
        id: '550e8400-e29b-41d4-a716-446655440002',
        startTime: 4.0,
        endTime: 7.2,
        text: "Thank you for joining us today.",
        speakerId: '550e8400-e29b-41d4-a716-446655440011',
        language: 'en',
        confidence: 0.92,
        words: this.createWordTimings("Thank you for joining us today."),
        processingPass: 1,
        createdAt: Date.now() + 4000
      }
    ];
  }
  
  static createMultilingualSegments(): TranscriptionSegment[] {
    return [
      {
        id: '550e8400-e29b-41d4-a716-446655440003',
        startTime: 0.0,
        endTime: 4.0,
        text: "Welcome to our international conference call.",
        speakerId: '550e8400-e29b-41d4-a716-446655440010',
        language: 'en',
        confidence: 0.93,
        words: this.createWordTimings("Welcome to our international conference call."),
        processingPass: 2,
        createdAt: Date.now()
      },
      {
        id: '550e8400-e29b-41d4-a716-446655440004',
        startTime: 4.5,
        endTime: 9.0,
        text: "こんにちは、今日はお忙しい中お時間をいただきありがとうございます。",
        speakerId: '550e8400-e29b-41d4-a716-446655440011',
        language: 'ja',
        confidence: 0.89,
        words: this.createWordTimings("こんにちは、今日はお忙しい中お時間をいただきありがとうございます。"),
        processingPass: 2,
        createdAt: Date.now() + 4500
      }
    ];
  }
  
  static createTechnicalPresentationSegments(): TranscriptionSegment[] {
    const technicalTerms = [
      "machine learning algorithms",
      "neural network architecture",
      "Docker containerization",
      "Kubernetes orchestration",
      "microservices deployment",
      "API gateway configuration"
    ];
    
    return technicalTerms.map((term, index) => ({
      id: `550e8400-e29b-41d4-a716-44665544${index.toString().padStart(4, '0')}`,
      startTime: index * 8.0,
      endTime: (index + 1) * 8.0 - 0.5,
      text: `Let me explain our approach to ${term} in this project.`,
      speakerId: '550e8400-e29b-41d4-a716-446655440010',
      language: 'en',
      confidence: 0.91,
      words: this.createWordTimings(`Let me explain our approach to ${term} in this project.`),
      processingPass: 2,
      createdAt: Date.now() + (index * 8000)
    }));
  }
  
  static createSpeakerProfiles(): SpeakerProfile[] {
    return [
      {
        id: '550e8400-e29b-41d4-a716-446655440010',
        name: 'Project Manager',
        embedding: this.generateSpeakerEmbedding('male-us-business'),
        languagePreference: 'en',
        totalSpeechTime: 450.5, // seconds
        segmentCount: 25,
        createdAt: Date.now() - 86400000, // 1 day ago
        lastActive: Date.now(),
        isPersistent: true
      },
      {
        id: '550e8400-e29b-41d4-a716-446655440011',
        name: 'Lead Developer',
        embedding: this.generateSpeakerEmbedding('female-us-technical'),
        languagePreference: 'en',
        totalSpeechTime: 380.2,
        segmentCount: 32,
        createdAt: Date.now() - 86400000,
        lastActive: Date.now(),
        isPersistent: true
      },
      {
        id: '550e8400-e29b-41d4-a716-446655440012',
        name: 'Tokyo Team Lead',
        embedding: this.generateSpeakerEmbedding('male-ja-business'),
        languagePreference: 'ja',
        totalSpeechTime: 295.8,
        segmentCount: 18,
        createdAt: Date.now() - 86400000,
        lastActive: Date.now(),
        isPersistent: true
      }
    ];
  }
  
  static createQualityMetrics(): TranscriptionQualityMetrics {
    return {
      overallConfidence: 0.91,
      speakerConsistency: 0.94,
      languageDetectionAccuracy: 0.96,
      realTimeFactor: 0.76,
      memoryUsagePeak: 3.2 * 1024 * 1024 * 1024, // 3.2GB
      cpuUsageAverage: 68.5, // 68.5%
      thermalEvents: [
        {
          timestamp: Date.now() - 1200000,
          temperature: 82.0,
          action: 'throttle',
          durationMs: 45000
        }
      ],
      processingLatency: {
        pass1Average: 1250, // milliseconds
        pass2Average: 3800,
        p95: 2100,
        p99: 4200
      }
    };
  }
  
  private static createWordTimings(text: string): WordTiming[] {
    const words = text.split(/\s+/);
    const wordsPerSecond = 2.5; // Average speaking rate
    let currentTime = 0;
    
    return words.map(word => {
      const duration = (word.length / 5) * (1 / wordsPerSecond); // Rough duration
      const wordTiming: WordTiming = {
        word: word.replace(/[.,!?]$/, ''), // Remove trailing punctuation
        startTime: currentTime,
        endTime: currentTime + duration,
        confidence: 0.85 + Math.random() * 0.13 // 0.85-0.98 confidence
      };
      
      currentTime += duration + 0.1; // Small pause between words
      return wordTiming;
    });
  }
  
  private static generateSpeakerEmbedding(profile: string): number[] {
    // Generate realistic 512-dimensional ECAPA-TDNN speaker embedding
    const embedding = new Array(512);
    const seed = this.hashString(profile);
    
    for (let i = 0; i < 512; i++) {
      // Use deterministic random based on profile for consistent embeddings
      const pseudoRandom = Math.sin(seed + i * 1.618) * 10000;
      embedding[i] = (pseudoRandom - Math.floor(pseudoRandom)) * 2 - 1; // [-1, 1]
    }
    
    // Normalize to unit vector (as ECAPA-TDNN embeddings are)
    const magnitude = Math.sqrt(embedding.reduce((sum, val) => sum + val * val, 0));
    return embedding.map(val => val / magnitude);
  }
  
  private static hashString(str: string): number {
    let hash = 0;
    for (let i = 0; i < str.length; i++) {
      const char = str.charCodeAt(i);
      hash = ((hash << 5) - hash) + char;
      hash = hash & hash; // Convert to 32-bit integer
    }
    return Math.abs(hash);
  }
}
```

---

## 6. Performance Benchmarks

### 6.1 Hardware Performance Targets

| Hardware Configuration | Target RTF | Max Memory | Expected WER | Min Speakers |
|------------------------|------------|------------|--------------|--------------|
| 6-core CPU, 8GB RAM   | ≤1.0       | 6GB        | ≤12% (EN)    | 2            |
| 8-core CPU, 16GB RAM  | ≤0.8       | 12GB       | ≤10% (EN)    | 4            |
| GPU (4GB VRAM)         | ≤0.3       | 8GB        | ≤8% (EN)     | 8            |
| High-end GPU (8GB+)    | ≤0.2       | 16GB       | ≤6% (EN)     | 10+          |

### 6.2 Quality Benchmarks

| Language | Standard Tier WER | High Accuracy WER | Turbo Tier WER | CER (Japanese) |
|----------|-------------------|-------------------|----------------|----------------|
| English  | ≤12%              | ≤8%               | ≤10%           | N/A            |
| Japanese | ≤15%              | ≤10%              | ≤12%           | ≤8%            |
| Spanish  | ≤14%              | ≤9%               | ≤11%           | N/A            |
| French   | ≤14%              | ≤9%               | ≤11%           | N/A            |
| German   | ≤13%              | ≤9%               | ≤11%           | N/A            |

### 6.3 Performance Test Scenarios

```typescript
export const PerformanceTestScenarios = {
  // Latency benchmarks
  realTimeLatency: {
    description: 'First word appears within 1.5s of speech start',
    target: 1500, // milliseconds
    testAudio: () => AudioTestFactory.createCleanSpeech(10),
    measure: 'time_to_first_word'
  },
  
  // Throughput benchmarks  
  continuousProcessing: {
    description: 'Process 4-hour meeting without degradation',
    target: { rtf: 1.0, memoryLeak: 0 },
    testAudio: () => AudioTestFactory.createLongMeeting(14400), // 4 hours
    measure: ['average_rtf', 'memory_stability']
  },
  
  // Accuracy benchmarks
  cleanSpeechAccuracy: {
    description: 'Clean English business meeting accuracy',
    target: { wer: 0.10, confidence: 0.90 },
    testAudio: () => AudioTestFactory.createBusinessMeeting(1800), // 30 min
    measure: ['word_error_rate', 'average_confidence']
  },
  
  // Stress test benchmarks
  thermalStability: {
    description: 'Maintain performance under thermal pressure',
    target: { gracefulDegradation: true, noDataLoss: true },
    testAudio: () => AudioTestFactory.createStressTestScenario(),
    measure: ['thermal_response', 'data_integrity']
  }
};
```

---

## 7. Security Considerations

### 7.1 Privacy Architecture

```rust
// Privacy-first data handling
pub struct PrivacyManager {
    memory_cleaner: MemoryCleaner,
    file_encryptor: FileEncryptor,
    network_isolator: NetworkIsolator,
}

impl PrivacyManager {
    pub fn secure_audio_buffer(&self, buffer: &mut AudioBuffer) {
        // Encrypt audio data in memory
        self.file_encryptor.encrypt_in_place(&mut buffer.samples);
        buffer.is_encrypted = true;
    }
    
    pub fn clear_sensitive_data(&self) {
        // Securely wipe memory containing audio or transcription data
        self.memory_cleaner.zero_memory();
        self.memory_cleaner.random_fill_memory(); // Defense against memory recovery
    }
    
    pub fn verify_network_isolation(&self) -> Result<(), SecurityError> {
        // Ensure no network calls are made during transcription
        if self.network_isolator.has_active_connections() {
            return Err(SecurityError::NetworkLeakDetected);
        }
        Ok(())
    }
}
```

### 7.2 Model Integrity Verification

```rust
pub struct ModelSecurityManager {
    checksum_validator: ChecksumValidator,
    signature_verifier: SignatureVerifier,
}

impl ModelSecurityManager {
    pub fn verify_model_integrity(&self, model_path: &Path) -> Result<(), SecurityError> {
        // Verify model file checksum
        let expected_checksum = self.get_expected_checksum(model_path)?;
        let actual_checksum = self.checksum_validator.calculate_sha256(model_path)?;
        
        if expected_checksum != actual_checksum {
            return Err(SecurityError::ModelTampering);
        }
        
        // Verify digital signature if available
        if let Some(signature_path) = self.find_signature_file(model_path) {
            self.signature_verifier.verify_signature(model_path, signature_path)?;
        }
        
        Ok(())
    }
}
```

### 7.3 Security Test Requirements

```typescript
describe('Security and Privacy Tests', () => {
  test('should encrypt all audio data at rest', async () => {
    const audioData = AudioTestFactory.createCleanSpeech(60);
    
    const encryptionManager = new EncryptionManager();
    const encryptedData = await encryptionManager.encryptAudio(audioData);
    
    // Verify data is encrypted
    expect(encryptedData.isEncrypted).toBe(true);
    expect(encryptedData.samples).not.toEqual(audioData.samples);
    
    // Verify decryption works
    const decryptedData = await encryptionManager.decryptAudio(encryptedData);
    expect(decryptedData.samples).toEqual(audioData.samples);
  });

  test('should securely wipe memory after processing', async () => {
    const memoryMonitor = new MemoryMonitor();
    const processor = new AudioProcessor();
    
    const sensitiveAudio = AudioTestFactory.createCleanSpeech(30);
    
    // Process audio
    await processor.processAudio(sensitiveAudio);
    
    // Check memory before cleanup
    const beforeCleanup = memoryMonitor.scanForAudioData();
    expect(beforeCleanup.foundSamples).toBeGreaterThan(0);
    
    // Trigger cleanup
    await processor.cleanup();
    
    // Verify memory is wiped
    const afterCleanup = memoryMonitor.scanForAudioData();
    expect(afterCleanup.foundSamples).toBe(0);
  });

  test('should detect and prevent network calls during transcription', async () => {
    const networkMonitor = new NetworkTrafficMonitor();
    networkMonitor.startMonitoring();
    
    const config = { qualityTier: 'standard', languages: ['en'] };
    const sessionId = await invoke('start_transcription', { config });
    
    // Simulate transcription
    const testAudio = AudioTestFactory.createBusinessMeeting(300);
    await simulateAudioInput(testAudio);
    
    await invoke('stop_transcription', { sessionId });
    
    const networkActivity = networkMonitor.getActivity();
    
    // Should have no outbound connections during transcription
    const transcriptionConnections = networkActivity.filter(
      activity => activity.timestamp > sessionId.startTime &&
                  activity.timestamp < sessionId.endTime &&
                  activity.type === 'outbound'
    );
    
    expect(transcriptionConnections).toHaveLength(0);
  });
});
```

---

## 8. Implementation Phases

### 8.1 Phase 0: Foundation (Weeks 1-6)

#### Parallel Development Groups

**Group A: Backend Core (backend-developer)**
- Audio capture system implementation
- VAD integration (Silero-VAD v5)
- Basic Tauri command structure
- Error handling and logging

**Group B: Frontend Foundation (frontend-developer)**
- React UI component library setup
- State management with Zustand
- Tauri API integration
- Basic audio visualization

**Group C: Test Infrastructure (qa-engineer)**
- Test framework setup (Jest, Playwright)
- Mock data factories
- CI/CD pipeline configuration
- Performance benchmarking tools

### 8.2 Phase 1: Core Features (Weeks 7-14)

#### Parallel Development Groups

**Group A: ASR Engine (backend-developer)**
- Whisper model integration (all tiers)
- Language detection and routing
- Two-pass processing pipeline
- Performance optimization

**Group B: UI/UX (frontend-developer)**
- Real-time transcription display
- Settings configuration interface
- Export functionality
- System monitoring dashboard

**Group C: Speaker Diarization (database-engineer)**
- sherpa-onnx integration (CPU)
- pyannote 3.1 integration (GPU)
- Speaker profile management
- ECAPA-TDNN embedding system

### 8.3 Phase 2: Advanced Features (Weeks 15-22)

#### Parallel Development Groups

**Group A: Optimization (backend-developer)**
- Thermal management system
- Memory optimization
- Model quantization
- Hardware detection

**Group B: Export & Integration (frontend-developer)**
- Multiple export formats
- Quality metrics display
- User preferences
- Accessibility features

**Group C: Quality Assurance (qa-engineer)**
- Comprehensive test coverage
- Performance validation
- Security testing
- Cross-platform verification

---

## 9. Component Interfaces

### 9.1 Backend Service Contracts

```rust
// Audio processing service interface
pub trait AudioProcessor: Send + Sync {
    async fn start_capture(&mut self, config: AudioConfig) -> Result<(), AudioError>;
    async fn stop_capture(&mut self) -> Result<(), AudioError>;
    async fn get_audio_chunk(&mut self) -> Result<Option<AudioChunk>, AudioError>;
    fn is_capturing(&self) -> bool;
    fn get_current_level(&self) -> f32;
}

// ASR engine service interface
pub trait ASREngine: Send + Sync {
    async fn initialize(&mut self, model_config: ModelConfig) -> Result<(), ASRError>;
    async fn transcribe(&self, audio: &AudioData, context: &TranscriptionContext) 
        -> Result<TranscriptionResult, ASRError>;
    fn get_supported_languages(&self) -> Vec<String>;
    fn get_performance_metrics(&self) -> PerformanceMetrics;
    async fn cleanup(&mut self) -> Result<(), ASRError>;
}

// Speaker diarization service interface  
pub trait SpeakerDiarizer: Send + Sync {
    async fn initialize(&mut self, config: DiarizationConfig) -> Result<(), DiarizationError>;
    async fn process_segment(&self, audio: &AudioData, existing_speakers: &[SpeakerProfile]) 
        -> Result<DiarizationResult, DiarizationError>;
    async fn update_speaker_profiles(&mut self, segments: &[TranscriptionSegment]) 
        -> Result<(), DiarizationError>;
    fn get_max_speakers(&self) -> usize;
}
```

### 9.2 Frontend Component Interfaces

```typescript
// Main transcription component interface
export interface TranscriptionControllerProps {
  onSessionStart: (sessionId: string) => void;
  onSessionEnd: (result: FinalTranscriptionResult) => void;
  onError: (error: TranscriptionError) => void;
  initialConfig?: Partial<TranscriptionConfig>;
}

// Real-time display component interface
export interface TranscriptionDisplayProps {
  segments: TranscriptionSegment[];
  currentSegment?: TranscriptionSegment;
  showSpeakerLabels: boolean;
  showConfidenceScores: boolean;
  highlightThreshold: number;
  onSegmentClick: (segment: TranscriptionSegment) => void;
}

// Audio visualization component interface
export interface AudioVisualizerProps {
  audioLevel: number;
  isRecording: boolean;
  vadActivity: boolean;
  showWaveform: boolean;
  height?: number;
  width?: number;
}

// Export manager component interface
export interface ExportManagerProps {
  sessionResult: FinalTranscriptionResult;
  availableFormats: ExportFormat[];
  onExportStart: (format: ExportFormat, options: ExportOptions) => void;
  onExportComplete: (result: ExportResult) => void;
  onExportError: (error: ExportError) => void;
}
```

---

## 10. Files to Create/Modify

### 10.1 New Backend Files

#### Core Audio Processing
- `src-tauri/src/audio/mod.rs` - Audio module entry point
- `src-tauri/src/audio/capture.rs` - Cross-platform audio capture
- `src-tauri/src/audio/preprocessor.rs` - Audio preprocessing pipeline
- `src-tauri/src/audio/vad.rs` - Silero-VAD integration
- `src-tauri/src/audio/types.rs` - Audio data types

#### ASR Engine System
- `src-tauri/src/asr/mod.rs` - ASR module entry point
- `src-tauri/src/asr/whisper.rs` - Whisper engine implementation
- `src-tauri/src/asr/reasonspeech.rs` - ReazonSpeech integration
- `src-tauri/src/asr/language_detector.rs` - Language detection
- `src-tauri/src/asr/manager.rs` - ASR engine manager

#### Speaker Diarization
- `src-tauri/src/diarization/mod.rs` - Diarization module entry
- `src-tauri/src/diarization/sherpa.rs` - sherpa-onnx implementation
- `src-tauri/src/diarization/pyannote.rs` - pyannote integration
- `src-tauri/src/diarization/speaker_profiles.rs` - Speaker management

#### System Management
- `src-tauri/src/system/mod.rs` - System monitoring entry
- `src-tauri/src/system/thermal.rs` - Thermal management
- `src-tauri/src/system/resource_monitor.rs` - Resource monitoring
- `src-tauri/src/system/hardware_detector.rs` - Hardware capability detection

#### Transcription Pipeline
- `src-tauri/src/transcription/mod.rs` - Transcription module entry
- `src-tauri/src/transcription/session.rs` - Session management
- `src-tauri/src/transcription/two_pass.rs` - Two-pass processing
- `src-tauri/src/transcription/context_manager.rs` - Context preservation

#### Export System
- `src-tauri/src/export/mod.rs` - Export module entry
- `src-tauri/src/export/formats.rs` - Export format implementations
- `src-tauri/src/export/manager.rs` - Export coordination

#### Security & Privacy
- `src-tauri/src/security/mod.rs` - Security module entry
- `src-tauri/src/security/encryption.rs` - Data encryption
- `src-tauri/src/security/memory_manager.rs` - Secure memory handling
- `src-tauri/src/security/model_verifier.rs` - Model integrity verification

### 10.2 New Frontend Files

#### Core Components
- `src/components/TranscriptionController.tsx` - Main transcription logic
- `src/components/AudioVisualizer.tsx` - Real-time audio visualization
- `src/components/TranscriptionDisplay.tsx` - Real-time text display
- `src/components/SpeakerPanel.tsx` - Speaker identification display

#### Configuration & Settings
- `src/components/SettingsPanel.tsx` - User configuration interface
- `src/components/QualityTierSelector.tsx` - Model tier selection
- `src/components/LanguageSelector.tsx` - Language configuration
- `src/components/AudioSourceConfig.tsx` - Audio source selection

#### Export & Results
- `src/components/ExportManager.tsx` - Export functionality
- `src/components/ResultsViewer.tsx` - Final results display
- `src/components/QualityMetrics.tsx` - Quality metrics dashboard
- `src/components/SessionHistory.tsx` - Past session management

#### System Monitoring
- `src/components/SystemMonitor.tsx` - Real-time system status
- `src/components/PerformanceIndicator.tsx` - Performance metrics
- `src/components/ThermalIndicator.tsx` - Thermal status display

#### State Management
- `src/stores/transcriptionStore.ts` - Main transcription state
- `src/stores/settingsStore.ts` - User preferences state
- `src/stores/systemStore.ts` - System monitoring state
- `src/stores/exportStore.ts` - Export management state

#### Utilities & Hooks
- `src/hooks/useAudioCapture.ts` - Audio capture React hook
- `src/hooks/useTranscription.ts` - Transcription management hook
- `src/hooks/useSystemMonitor.ts` - System monitoring hook
- `src/utils/audioUtils.ts` - Audio processing utilities
- `src/utils/formatUtils.ts` - Text formatting utilities
- `src/utils/validationUtils.ts` - Input validation utilities

### 10.3 Test Files

#### Backend Tests
- `tests/unit/audio/capture.test.rs` - Audio capture unit tests
- `tests/unit/asr/whisper.test.rs` - Whisper engine tests
- `tests/unit/diarization/speaker_profiles.test.rs` - Speaker management tests
- `tests/integration/transcription_pipeline.test.rs` - Pipeline integration tests
- `tests/benchmarks/performance.bench.rs` - Performance benchmarks

#### Frontend Tests
- `tests/unit/components/TranscriptionController.test.tsx` - Controller tests
- `tests/unit/hooks/useTranscription.test.ts` - Hook tests
- `tests/integration/user_workflows.test.tsx` - Integration tests
- `tests/e2e/complete_workflows.e2e.test.ts` - End-to-end tests

### 10.4 Configuration Files

#### Updated Files
- `src-tauri/Cargo.toml` - Add new Rust dependencies
- `package.json` - Add new frontend dependencies
- `src-tauri/tauri.conf.json` - Add new Tauri commands and permissions

#### New Configuration Files
- `src-tauri/models/model_configs.json` - Model configuration definitions
- `src/config/defaultSettings.json` - Default application settings
- `tests/fixtures/test_audio_metadata.json` - Test audio configurations
- `.github/workflows/ci.yml` - GitHub Actions CI configuration

### 10.5 Dependency Updates

#### Rust Dependencies (Cargo.toml)
```toml
[dependencies]
# Existing dependencies
tauri = { version = "2", features = ["macos-private-api"] }
tauri-plugin-opener = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
cpal = "0.16.0"
hound = "3.5.1"
tokio = { version = "1.47.1", features = ["full"] }

# New AI/ML dependencies
onnxruntime = { version = "0.0.14", features = ["cuda"] }
candle = { version = "0.6.0", features = ["cuda", "mkl"] }
tch = { version = "0.16.0", optional = true }

# Audio processing
dasp = "0.11.0"
rustfft = "6.0.0"
spectrum-analyzer = "1.5.0"

# Utility dependencies
uuid = { version = "1.0", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
anyhow = "1.0"
thiserror = "1.0"
tracing = "0.1"
tracing-subscriber = "0.3"

# System monitoring
sysinfo = "0.30.0"
dirs = "5.0.0"

# Security
aes-gcm = "0.10.0"
sha2 = "0.10.0"
```

#### Frontend Dependencies (package.json)
```json
{
  "dependencies": {
    "@radix-ui/themes": "^3.2.1",
    "@tauri-apps/api": "^2.8.0",
    "@tauri-apps/plugin-opener": "^2",
    "react": "^19.1.0",
    "react-dom": "^19.1.0",
    "wavesurfer.js": "^7.10.1",
    "zustand": "^5.0.2",
    "zod": "^3.22.4",
    "date-fns": "^3.6.0",
    "react-hotkeys-hook": "^4.6.1",
    "react-use": "^17.6.0"
  },
  "devDependencies": {
    "@tauri-apps/cli": "^2",
    "@types/react": "^19.1.8",
    "@types/react-dom": "^19.1.6",
    "@vitejs/plugin-react": "^4.6.0",
    "typescript": "~5.8.3",
    "vite": "^7.0.4",
    "@testing-library/react": "^16.1.0",
    "@testing-library/jest-dom": "^6.8.1",
    "@testing-library/user-event": "^14.8.1",
    "@playwright/test": "^1.50.1",
    "jest": "^29.8.1",
    "@types/jest": "^29.8.1",
    "ts-jest": "^29.8.1"
  }
}
```

---

## 11. Success Criteria

### 11.1 Technical Success Criteria

#### Performance Requirements
- [ ] Real-Time Factor ≤0.8 on minimum hardware (6-core CPU, 8GB RAM)
- [ ] First word latency ≤1.5 seconds for real-time transcription
- [ ] Memory usage ≤8GB peak during operation
- [ ] CPU usage ≤75% average during active transcription
- [ ] No crashes during 4-hour continuous operation

#### Accuracy Requirements
- [ ] Word Error Rate ≤12% for clean English speech (Standard tier)
- [ ] Word Error Rate ≤8% for clean English speech (High Accuracy tier)
- [ ] Character Error Rate ≤8% for Japanese speech with ReazonSpeech
- [ ] Speaker Diarization Error Rate ≤15% for 2-4 speaker meetings
- [ ] Language detection accuracy ≥90% for supported languages

#### Privacy & Security Requirements
- [ ] Zero network calls during transcription processing (verifiable via network monitoring)
- [ ] All audio data encrypted at rest with AES-256
- [ ] Secure memory wiping within 5 seconds of session end
- [ ] Model integrity verification via SHA-256 checksums
- [ ] No persistent audio data after application closure

### 11.2 User Experience Success Criteria

#### Usability Requirements
- [ ] New user setup completes within 5 minutes
- [ ] Application startup time ≤10 seconds
- [ ] Export completes within 30 seconds for 1-hour meetings
- [ ] All features accessible via keyboard navigation
- [ ] WCAG 2.1 AA compliance for screen readers

#### Feature Completeness
- [ ] Support for minimum 3 quality tiers (Standard, High Accuracy, Turbo)
- [ ] Real-time transcription with background refinement
- [ ] Speaker diarization for 2-8 speakers
- [ ] Export in 5+ formats (TXT, SRT, VTT, JSON, CSV)
- [ ] Multilingual support for 10+ languages

### 11.3 Test Coverage Success Criteria

#### Automated Test Coverage
- [ ] Unit test coverage ≥90% for critical audio processing components
- [ ] Unit test coverage ≥85% for ASR engine implementations
- [ ] Integration test coverage for all user workflows
- [ ] End-to-end test coverage for complete transcription scenarios
- [ ] Performance benchmark tests for all quality tiers

#### Test Quality Requirements
- [ ] All test data factories provide realistic test scenarios
- [ ] Mock implementations accurately simulate real model behavior
- [ ] Error scenarios comprehensively covered in test suites
- [ ] Cross-platform test execution (Windows and macOS)
- [ ] Automated CI/CD pipeline with test results reporting

---

This technical architecture provides comprehensive specifications that enable immediate test-driven development. QA engineers can write failing tests for all specified interfaces and behaviors, while backend and frontend developers can implement against clear contracts. The detailed test specifications, mock data factories, and performance benchmarks ensure that implementation can proceed in parallel with confidence that all components will integrate correctly.

The architecture maintains the core privacy-first principles while providing the scalability and performance needed for real-world usage. All specifications are measurable and testable, enabling a true TDD approach where tests drive implementation rather than being an afterthought.