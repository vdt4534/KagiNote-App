# Speaker Diarization API Specification

## Overview
This document specifies the complete API for speaker diarization functionality in KagiNote, including Tauri commands, event payloads, and internal service interfaces.

## Tauri Commands

### 1. start_diarization
Initialize speaker diarization for a session.

**Command**: `start_diarization`
**Request**:
```typescript
{
  sessionId: string;
  config: {
    maxSpeakers: number;           // 2-10
    minSpeakers: number;           // 1-10  
    minSegmentLength: number;      // ms, default: 1500
    embeddingWindowSize: number;   // ms, default: 3000
    clusteringThreshold: number;   // 0.0-1.0, default: 0.7
    enableAdaptiveClustering: boolean;
    hardwareAcceleration: 'auto' | 'cpu' | 'metal';
  }
}
```

**Response**:
```typescript
{
  sessionId: string;
  status: 'initializing' | 'ready' | 'error';
  estimatedProcessingTime: number; // milliseconds
  error?: string;
}
```

**Error Codes**:
- `MODEL_LOAD_ERROR`: Failed to load diarization model
- `CONFIG_ERROR`: Invalid configuration parameters
- `MEMORY_ERROR`: Insufficient memory
- `SESSION_EXISTS`: Session already exists

### 2. stop_diarization
Stop diarization for a session and cleanup resources.

**Command**: `stop_diarization`
**Request**:
```typescript
{
  sessionId: string;
  saveResults: boolean;
}
```

**Response**:
```typescript
{
  sessionId: string;
  finalResults?: DiarizationResult;
  success: boolean;
}
```

### 3. process_audio_for_diarization
Process audio chunk for speaker identification.

**Command**: `process_audio_for_diarization`
**Request**:
```typescript
{
  sessionId: string;
  audioData: number[]; // Float32Array as array
  timestamp: number;
  isLastChunk: boolean;
}
```

**Response**:
```typescript
{
  sessionId: string;
  segments: SpeakerSegment[];
  processingTime: number;
  bufferUtilization: number; // 0.0-1.0
}
```

### 4. update_speaker_profile
Update speaker display name, color, or notes.

**Command**: `update_speaker_profile`
**Request**:
```typescript
{
  sessionId: string;
  speakerId: string;
  updates: {
    displayName?: string;
    color?: string;      // Hex color
    notes?: string;
  }
}
```

**Response**:
```typescript
{
  success: boolean;
  speaker: SpeakerProfile;
}
```

### 5. merge_speakers
Merge two speakers into one (for correction).

**Command**: `merge_speakers`
**Request**:
```typescript
{
  sessionId: string;
  sourceSpeakerId: string;
  targetSpeakerId: string;
}
```

**Response**:
```typescript
{
  success: boolean;
  mergedSpeaker: SpeakerProfile;
  affectedSegments: number;
}
```

### 6. split_speaker
Split a speaker into two (for correction).

**Command**: `split_speaker`
**Request**:
```typescript
{
  sessionId: string;
  speakerId: string;
  splitPoint: number; // timestamp in seconds
}
```

**Response**:
```typescript
{
  success: boolean;
  speaker1: SpeakerProfile;
  speaker2: SpeakerProfile;
}
```

### 7. get_diarization_statistics
Get comprehensive statistics for the session.

**Command**: `get_diarization_statistics`
**Request**:
```typescript
{
  sessionId: string;
}
```

**Response**:
```typescript
{
  totalSpeakers: number;
  speakerDistribution: {
    [speakerId: string]: {
      speakerId: string;
      totalSpeechTime: number;
      percentageOfTotal: number;
      segmentCount: number;
      averageSegmentLength: number;
      averageConfidence: number;
      wordsPerMinute?: number;
    }
  };
  overallConfidence: number;
  processingMetrics: {
    totalAudioSeconds: number;
    processingTimeMs: number;
    realTimeFactor: number;
    memoryUsageMb: number;
    cpuUsagePercent: number;
  };
  sessionDuration: number;
  speakerChanges: number;
  avgSegmentLength: number;
}
```

### 8. export_diarization_results
Export diarization results in various formats.

**Command**: `export_diarization_results`
**Request**:
```typescript
{
  sessionId: string;
  format: 'json' | 'csv' | 'rttm' | 'textgrid';
  includeEmbeddings: boolean;
  includeStatistics: boolean;
}
```

**Response**:
```typescript
{
  data: string;
  filename: string;
  mimeType: string;
  sizeBytes: number;
}
```

### 9. import_speaker_profiles
Import saved speaker profiles for re-identification.

**Command**: `import_speaker_profiles`
**Request**:
```typescript
{
  profiles: SpeakerProfile[];
  sessionId: string;
}
```

**Response**:
```typescript
{
  imported: number;
  failed: number;
  errors?: string[];
}
```

### 10. get_speaker_embeddings
Get embeddings for a specific speaker.

**Command**: `get_speaker_embeddings`
**Request**:
```typescript
{
  sessionId: string;
  speakerId: string;
  maxCount?: number; // Limit number of embeddings
}
```

**Response**:
```typescript
{
  speakerId: string;
  embeddings: SpeakerEmbedding[];
  averageQuality: number;
}
```

## Event Specifications

### Speaker Detection Events

#### speaker-detected
Emitted when a new speaker is identified.
```typescript
{
  event: 'speaker-detected',
  payload: {
    sessionId: string;
    speakerId: string;
    confidence: number;
    timestamp: number;
    isNewSpeaker: boolean;
    embedding?: number[]; // Optional 512-dim vector
  }
}
```

#### speaker-activity
Emitted when speaker starts or stops speaking.
```typescript
{
  event: 'speaker-activity',
  payload: {
    sessionId: string;
    speakerId: string;
    isActive: boolean;
    confidence: number;
    startTime: number;
    endTime?: number;
    text?: string; // If transcription available
  }
}
```

#### speaker-merged
Emitted after merging speakers.
```typescript
{
  event: 'speaker-merged',
  payload: {
    sessionId: string;
    sourceSpeakerId: string;
    targetSpeakerId: string;
    affectedSegments: number;
  }
}
```

### Processing Events

#### diarization-progress
Periodic progress updates during processing.
```typescript
{
  event: 'diarization-progress',
  payload: {
    sessionId: string;
    processedSeconds: number;
    totalSeconds: number;
    speakersFound: number;
    currentPhase: 'vad' | 'embedding' | 'clustering' | 'merging';
  }
}
```

#### diarization-ready
Emitted when diarization is ready to process audio.
```typescript
{
  event: 'diarization-ready',
  payload: {
    sessionId: string;
    status: 'ready';
    modelInfo: {
      name: string;
      version: string;
      device: string;
    }
  }
}
```

#### diarization-complete
Emitted when processing is complete.
```typescript
{
  event: 'diarization-complete',
  payload: {
    sessionId: string;
    totalSpeakers: number;
    processingTimeMs: number;
    segments: number;
  }
}
```

### Error Events

#### diarization-error
Emitted on processing errors.
```typescript
{
  event: 'diarization-error',
  payload: {
    sessionId: string;
    error: string;
    code: 'MODEL_LOAD_ERROR' | 'PROCESSING_ERROR' | 'MEMORY_ERROR' | 'TIMEOUT';
    recoverable: boolean;
    fallbackAvailable: boolean;
    details?: any;
  }
}
```

#### diarization-warning
Non-fatal warnings during processing.
```typescript
{
  event: 'diarization-warning',
  payload: {
    sessionId: string;
    warning: string;
    code: string;
    suggestion?: string;
  }
}
```

## Internal Service APIs

### DiarizationService (Rust)
```rust
#[async_trait]
pub trait DiarizationService {
    async fn initialize(&mut self, config: DiarizationConfig) -> Result<()>;
    async fn process_audio(&mut self, audio: &[f32]) -> Result<Vec<SpeakerSegment>>;
    async fn get_speaker_profiles(&self) -> Result<Vec<SpeakerProfile>>;
    async fn update_speaker(&mut self, id: &str, updates: SpeakerUpdates) -> Result<()>;
    async fn merge_speakers(&mut self, source: &str, target: &str) -> Result<SpeakerProfile>;
    async fn split_speaker(&mut self, id: &str, split_point: f32) -> Result<(SpeakerProfile, SpeakerProfile)>;
    async fn get_statistics(&self) -> Result<DiarizationStatistics>;
    async fn export(&self, format: ExportFormat) -> Result<Vec<u8>>;
    async fn shutdown(&mut self) -> Result<()>;
}
```

### AudioBufferManager (Rust)
```rust
pub trait AudioBufferManager {
    fn write(&self, data: &[f32]) -> Result<()>;
    fn read(&self, consumer_id: &str, count: usize) -> Result<Vec<f32>>;
    fn peek(&self, consumer_id: &str, count: usize) -> Result<Vec<f32>>;
    fn skip(&self, consumer_id: &str, count: usize) -> Result<()>;
    fn available(&self, consumer_id: &str) -> usize;
    fn register_consumer(&mut self, id: String) -> Result<()>;
    fn unregister_consumer(&mut self, id: &str) -> Result<()>;
    fn get_state(&self) -> BufferState;
}
```

### Frontend Service (TypeScript)
```typescript
interface DiarizationService {
  // Lifecycle
  initialize(sessionId: string, config: DiarizationConfig): Promise<void>;
  shutdown(sessionId: string): Promise<void>;
  
  // Processing
  processAudio(sessionId: string, audio: Float32Array): Promise<SpeakerSegment[]>;
  
  // Speaker Management
  updateSpeaker(sessionId: string, speakerId: string, updates: SpeakerUpdates): Promise<SpeakerProfile>;
  mergeSpeakers(sessionId: string, source: string, target: string): Promise<SpeakerProfile>;
  splitSpeaker(sessionId: string, speakerId: string, splitPoint: number): Promise<[SpeakerProfile, SpeakerProfile]>;
  
  // Statistics & Export
  getStatistics(sessionId: string): Promise<DiarizationStatistics>;
  exportResults(sessionId: string, format: string): Promise<Blob>;
  
  // Event Subscriptions
  onSpeakerDetected(callback: (event: SpeakerDetectedEvent) => void): () => void;
  onSpeakerActivity(callback: (event: SpeakerActivityEvent) => void): () => void;
  onError(callback: (event: DiarizationErrorEvent) => void): () => void;
}
```

## WebSocket Protocol (Optional Future Enhancement)

For real-time streaming diarization:

### Connection
```
ws://localhost:3000/diarization/stream
```

### Messages

#### Client → Server
```json
{
  "type": "audio_chunk",
  "sessionId": "uuid",
  "data": [...], // Float32Array as array
  "timestamp": 1234567890,
  "sampleRate": 16000
}
```

#### Server → Client
```json
{
  "type": "speaker_update",
  "sessionId": "uuid",
  "speakers": [
    {
      "id": "speaker_1",
      "active": true,
      "confidence": 0.92
    }
  ]
}
```

## Rate Limiting

To prevent resource exhaustion:

| Endpoint | Rate Limit | Window |
|----------|------------|---------|
| process_audio_for_diarization | 100/sec | Per session |
| update_speaker_profile | 10/sec | Per session |
| get_diarization_statistics | 5/sec | Per session |
| export_diarization_results | 1/sec | Global |

## Performance Benchmarks

Expected performance targets:

| Metric | Target | Measurement |
|--------|--------|-------------|
| Model Load Time | < 3s | Time to initialize |
| Audio Processing | < 0.02 RTF | Real-time factor |
| Speaker Detection Latency | < 100ms | Time to first detection |
| Memory Usage (1hr audio) | < 100MB | Peak RSS |
| CPU Usage | < 5% | During recording |
| Embedding Extraction | < 50ms | Per 3s window |
| Clustering Time | < 200ms | For 10 speakers |

## Error Handling

### Error Response Format
```typescript
{
  success: false,
  error: {
    code: string;
    message: string;
    details?: any;
    recoverable: boolean;
    suggestion?: string;
  }
}
```

### Common Error Codes
- `MODEL_NOT_FOUND`: Diarization model not available
- `MODEL_LOAD_FAILED`: Failed to load model
- `INVALID_AUDIO_FORMAT`: Audio format not supported
- `SESSION_NOT_FOUND`: Session doesn't exist
- `SPEAKER_NOT_FOUND`: Speaker ID not found
- `MEMORY_EXCEEDED`: Memory limit exceeded
- `PROCESSING_TIMEOUT`: Processing took too long
- `INVALID_CONFIG`: Invalid configuration
- `HARDWARE_NOT_AVAILABLE`: Requested hardware not available

## Versioning

API version included in all responses:
```typescript
{
  apiVersion: "1.0.0",
  // ... rest of response
}
```

Breaking changes will increment major version.