# Speaker Diarization Technical Architecture

## System Architecture Overview

### Core Objectives
- **Privacy-First**: Zero network calls, all processing local
- **Real-time Performance**: Process 1 hour of audio in < 1 minute
- **Multi-Speaker Support**: Efficiently handle 2-10 speakers
- **Production Reliability**: Graceful degradation and error recovery
- **Seamless Integration**: Works alongside existing Whisper pipeline

### High-Level Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Audio Input (16kHz mono)                  │
└────────────────┬────────────────────────────────────────────┘
                 │
                 v
┌─────────────────────────────────────────────────────────────┐
│                   Audio Buffer Manager                       │
│  - Ring buffer for concurrent access                         │
│  - Shared between Whisper & Diarization                      │
│  - Lock-free SPSC queue implementation                       │
└────────────────┬──────────────────────────┬─────────────────┘
                 │                          │
       ┌─────────v────────┐        ┌───────v──────────┐
       │ Whisper Pipeline │        │ Diarization      │
       │ (Existing)       │        │ Pipeline         │
       └─────────┬────────┘        └───────┬──────────┘
                 │                          │
                 v                          v
       ┌──────────────────┐       ┌────────────────────┐
       │ Transcription    │       │ Speaker Segments   │
       │ Segments         │       │ with IDs           │
       └─────────┬────────┘       └────────┬───────────┘
                 │                          │
                 └──────────┬───────────────┘
                            v
       ┌─────────────────────────────────────────────┐
       │          Segment Merger & Aligner           │
       │  - Temporal alignment of text & speakers    │
       │  - Conflict resolution                      │
       └──────────────────┬──────────────────────────┘
                          v
       ┌─────────────────────────────────────────────┐
       │         Final Transcription Output          │
       │  - Text with speaker IDs                    │
       │  - Confidence scores                        │
       │  - Timing information                       │
       └─────────────────────────────────────────────┘
```

## Component Interactions

### 1. Audio Buffer Manager
Shared audio access for parallel processing:
```rust
pub struct SharedAudioBuffer {
    buffer: Arc<RwLock<RingBuffer<f32>>>,
    write_pos: AtomicUsize,
    read_positions: HashMap<String, AtomicUsize>,
    capacity: usize,
}
```

### 2. Diarization Pipeline
pyannote-rs based speaker identification:
```rust
pub struct DiarizationPipeline {
    model: PyannoteModel,
    embedder: SpeakerEmbedder,
    clustering: AgglomerativeClustering,
    vad: VoiceActivityDetector,
}
```

### 3. Segment Merger
Combines transcription and speaker data:
```rust
pub struct SegmentMerger {
    alignment_threshold: f32,
    overlap_strategy: OverlapStrategy,
    confidence_threshold: f32,
}
```

## API Contracts

### Tauri Commands

#### start_diarization
```typescript
interface StartDiarizationRequest {
  sessionId: string;
  config: DiarizationConfig;
}

interface DiarizationConfig {
  maxSpeakers: number;           // 2-10
  minSpeakers: number;           // 1-10
  minSegmentLength: number;      // milliseconds (default: 1500)
  embeddingWindowSize: number;   // milliseconds (default: 3000)
  clusteringThreshold: number;   // 0.0-1.0 (default: 0.7)
  enableAdaptiveClustering: boolean;
  hardwareAcceleration: 'auto' | 'cpu' | 'metal';
}

interface StartDiarizationResponse {
  sessionId: string;
  status: 'initializing' | 'ready' | 'error';
  estimatedProcessingTime: number; // milliseconds
  error?: string;
}
```

#### process_audio_for_diarization
```typescript
interface ProcessAudioRequest {
  sessionId: string;
  audioData: Float32Array;
  timestamp: number;
  isLastChunk: boolean;
}

interface ProcessAudioResponse {
  sessionId: string;
  segments: SpeakerSegment[];
  processingTime: number;
  bufferUtilization: number; // 0.0-1.0
}
```

#### update_speaker_profile
```typescript
interface UpdateSpeakerRequest {
  sessionId: string;
  speakerId: string;
  updates: {
    displayName?: string;
    color?: string;
    notes?: string;
  };
}

interface UpdateSpeakerResponse {
  success: boolean;
  speaker: SpeakerProfile;
}
```

#### get_diarization_statistics
```typescript
interface GetStatisticsRequest {
  sessionId: string;
}

interface DiarizationStatistics {
  totalSpeakers: number;
  speakerDistribution: Map<string, SpeakerStats>;
  overallConfidence: number;
  processingMetrics: {
    totalAudioProcessed: number; // seconds
    processingTime: number;      // milliseconds
    realTimeFactor: number;      // processing_time / audio_duration
  };
}

interface SpeakerStats {
  speakerId: string;
  totalSpeechTime: number;
  percentageOfTotal: number;
  segmentCount: number;
  averageSegmentLength: number;
  averageConfidence: number;
}
```

### Event Payloads

#### speaker-detected
```typescript
interface SpeakerDetectedEvent {
  sessionId: string;
  speakerId: string;
  confidence: number;
  timestamp: number;
  isNewSpeaker: boolean;
  embedding?: number[]; // Optional 512-dim vector
}
```

#### speaker-activity
```typescript
interface SpeakerActivityEvent {
  sessionId: string;
  speakerId: string;
  isActive: boolean;
  confidence: number;
  startTime: number;
  endTime?: number;
  text?: string; // If transcription available
}
```

#### diarization-error
```typescript
interface DiarizationErrorEvent {
  sessionId: string;
  error: string;
  code: 'MODEL_LOAD_ERROR' | 'PROCESSING_ERROR' | 'MEMORY_ERROR' | 'TIMEOUT';
  recoverable: boolean;
  fallbackAvailable: boolean;
}
```

## Data Models

### Core Types (Rust)
See `src-tauri/src/diarization/types.rs` for complete definitions.

### Core Types (TypeScript)
See `src/types/diarization.ts` for complete definitions.

## Performance Optimization Strategies

### 1. Parallel Processing Architecture
```rust
// Concurrent processing with shared audio buffer
let (whisper_handle, diarization_handle) = tokio::join!(
    whisper_pipeline.process_async(&audio_buffer),
    diarization_pipeline.process_async(&audio_buffer)
);
```

### 2. Adaptive Clustering
- Start with conservative threshold (0.7)
- Adjust based on speaker count and confidence
- Use online clustering for real-time updates

### 3. Memory Management
- Sliding window for embeddings (keep last 5 minutes)
- Compress older embeddings using PCA
- Lazy loading of speaker profiles

### 4. Hardware Acceleration
- CoreML backend on macOS (Metal Performance Shaders)
- ONNX Runtime for cross-platform compatibility
- Fallback to optimized CPU implementation

## Error Handling & Recovery

### Failure Modes
1. **Model Loading Failure**
   - Fallback: Continue without diarization
   - Recovery: Retry with smaller model
   - User feedback: "Speaker identification temporarily unavailable"

2. **Memory Exhaustion**
   - Fallback: Reduce embedding window size
   - Recovery: Clear old embeddings, continue with new
   - User feedback: "Optimizing memory usage..."

3. **Processing Timeout**
   - Fallback: Process in smaller chunks
   - Recovery: Increase chunk size gradually
   - User feedback: "Processing speakers..."

### Graceful Degradation Path
```
Full Diarization → Simplified (2 speakers) → Voice Activity Only → Disabled
```

## Security Considerations

### Privacy Protection
- No network calls during processing
- Embeddings stored in memory only (not persisted by default)
- Optional encryption for speaker profiles
- Secure wiping of audio buffers after processing

### Input Validation
- Audio format validation (16kHz, mono)
- Buffer size limits (max 1GB)
- Speaker count limits (max 10)
- Prevent injection attacks in speaker names

## Implementation Phases

### Phase 1: Core Infrastructure (Parallel)
**Group 1A: Rust Backend Setup**
- Create `src-tauri/src/diarization/` module structure
- Implement types and data models
- Set up pyannote-rs integration
- Create audio buffer manager

**Group 1B: TypeScript Frontend Setup**
- Create TypeScript type definitions
- Set up event listeners
- Create speaker UI components
- Implement state management

**Group 1C: Test Infrastructure**
- Write unit tests for diarization module
- Create integration test fixtures
- Set up performance benchmarks
- Implement mock diarization service

### Phase 2: Pipeline Implementation (Sequential)
**Depends on Phase 1**
- Implement VAD for diarization
- Create speaker embedding extractor
- Implement clustering algorithm
- Build segment merger

### Phase 3: Integration (Sequential)
**Depends on Phase 2**
- Integrate with Whisper pipeline
- Implement real-time event emission
- Add speaker profile management
- Create statistics aggregation

### Phase 4: Optimization (Parallel)
**Group 4A: Performance**
- Add hardware acceleration
- Implement adaptive clustering
- Optimize memory usage
- Add caching layer

**Group 4B: User Experience**
- Implement speaker renaming
- Add color customization
- Create speaker search
- Build export functionality

## Component Interfaces

### DiarizationService Interface
```rust
#[async_trait]
pub trait DiarizationService: Send + Sync {
    async fn initialize(&mut self, config: DiarizationConfig) -> Result<()>;
    async fn process_audio(&mut self, audio: &[f32]) -> Result<Vec<SpeakerSegment>>;
    async fn get_speaker_profiles(&self) -> Result<Vec<SpeakerProfile>>;
    async fn update_speaker(&mut self, id: &str, updates: SpeakerUpdates) -> Result<()>;
    async fn get_statistics(&self) -> Result<DiarizationStatistics>;
    async fn shutdown(&mut self) -> Result<()>;
}
```

### AudioBufferManager Interface
```rust
pub trait AudioBufferManager: Send + Sync {
    fn write(&self, data: &[f32]) -> Result<()>;
    fn read(&self, consumer_id: &str, count: usize) -> Result<Vec<f32>>;
    fn peek(&self, consumer_id: &str, count: usize) -> Result<Vec<f32>>;
    fn skip(&self, consumer_id: &str, count: usize) -> Result<()>;
    fn available(&self, consumer_id: &str) -> usize;
}
```

### SegmentMerger Interface
```rust
pub trait SegmentMerger: Send + Sync {
    fn merge(
        &self,
        transcription: Vec<TranscriptionSegment>,
        speakers: Vec<SpeakerSegment>,
    ) -> Result<Vec<FinalSegment>>;
    
    fn resolve_conflicts(
        &self,
        segments: Vec<FinalSegment>,
    ) -> Result<Vec<FinalSegment>>;
}
```

## Files to Create/Modify

### New Files to Create
```
src-tauri/src/diarization/
├── mod.rs                    # Module exports
├── types.rs                  # Data types and structures
├── service.rs               # Main diarization service
├── pipeline.rs              # Processing pipeline
├── embedder.rs              # Speaker embedding extraction
├── clustering.rs            # Speaker clustering
├── buffer_manager.rs        # Shared audio buffer
├── segment_merger.rs        # Merge transcription & speakers
├── hardware_accel.rs        # Hardware acceleration
└── tests.rs                 # Unit tests

src/types/
├── diarization.ts           # TypeScript types
└── events.ts                # Event type definitions

src/services/
├── diarizationService.ts    # Frontend service
└── speakerManager.ts        # Speaker state management

src/components/features/
├── SpeakerDisplay.tsx       # Speaker UI component
├── SpeakerStatistics.tsx    # Statistics display
└── SpeakerTimeline.tsx      # Timeline visualization

tests/unit/diarization/
├── embedder.test.rs         # Embedder tests
├── clustering.test.rs       # Clustering tests
└── merger.test.rs           # Merger tests

tests/integration/
└── diarization_pipeline.test.rs  # Full pipeline tests
```

### Files to Modify
```
src-tauri/src/lib.rs         # Add diarization module
src-tauri/src/commands.rs    # Add diarization commands
src-tauri/Cargo.toml         # Add pyannote-rs dependency
src/screens/RecordingScreen.tsx  # Add speaker display
src/components/features/TranscriptView.tsx  # Show speakers
package.json                 # Add any needed dependencies
```

## Success Criteria

### Functional Requirements
- ✅ Identify 2-10 speakers in real-time
- ✅ Process 1 hour of audio in < 1 minute
- ✅ Maintain >90% accuracy for distinct voices
- ✅ Handle overlapping speech gracefully
- ✅ Support speaker renaming and customization
- ✅ Export transcripts with speaker labels

### Performance Requirements
- ✅ < 100ms latency for speaker detection
- ✅ < 5% CPU overhead during recording
- ✅ < 100MB memory for 1-hour session
- ✅ Real-time factor < 0.02 (1 min processing for 1 hour audio)

### Quality Requirements
- ✅ >90% speaker identification accuracy
- ✅ >75% accuracy for similar voices
- ✅ >60% accuracy in noisy conditions
- ✅ 100% consistency for speaker re-identification

### Integration Requirements
- ✅ Zero interference with Whisper pipeline
- ✅ Seamless fallback when diarization fails
- ✅ Compatible with existing UI components
- ✅ Maintains privacy-first architecture

## Monitoring & Diagnostics

### Performance Metrics to Track
```rust
pub struct DiarizationMetrics {
    pub segments_processed: u64,
    pub avg_processing_time_ms: f64,
    pub speaker_changes_detected: u64,
    pub embedding_cache_hits: u64,
    pub clustering_iterations: u64,
    pub memory_usage_mb: f32,
    pub cpu_usage_percent: f32,
}
```

### Debug Logging Points
- Model initialization status
- Audio buffer utilization
- Embedding extraction timing
- Clustering convergence
- Segment merger conflicts
- Memory pressure events
- Hardware acceleration status

## Future Enhancements

### Phase 5: Advanced Features
- Speaker voice authentication
- Emotion detection per speaker
- Meeting dynamics analysis
- Automatic speaker naming suggestions
- Cross-meeting speaker recognition
- Real-time translation per speaker

### Phase 6: Cloud Integration (Optional)
- Encrypted speaker profile sync
- Collaborative speaker labeling
- Meeting analytics dashboard
- Speaker database management