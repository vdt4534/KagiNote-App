# Speaker Diarization API Reference

**Complete API documentation for KagiNote's speaker diarization system**

## Overview

The KagiNote diarization API provides programmatic access to speaker identification, profile management, and real-time audio processing. All APIs use Tauri's invoke pattern for secure communication between frontend and backend.

## Core Commands

### Session Management

#### `initialize_diarization_service`
Initialize the diarization service with configuration.

**Request:**
```typescript
await invoke('initialize_diarization_service', {
  config: {
    maxSpeakers: 8,
    minSpeakers: 2,
    similarityThreshold: 0.7,
    minSegmentDuration: 1.0,
    hardwareAcceleration: 'auto',
    vadThreshold: 0.5,
    detectOverlaps: true,
    maxMemoryMb: 500
  }
});
```

**Response:**
```typescript
{
  success: true,
  message: "Diarization service initialized successfully"
}
```

**Error Codes:**
- `SERVICE_ALREADY_INITIALIZED`: Service was already running
- `INVALID_CONFIG`: Configuration validation failed
- `MODEL_LOAD_ERROR`: Failed to load diarization models
- `HARDWARE_ERROR`: Hardware acceleration not available

---

#### `diarize_audio_segment`
Process audio segment for speaker identification.

**Request:**
```typescript
await invoke('diarize_audio_segment', {
  audioSamples: Float32Array, // Raw audio samples
  sampleRate: 44100,          // Audio sample rate
  sessionId: 'session_uuid'   // Optional session identifier
});
```

**Response:**
```typescript
{
  sessionId: string,
  segments: [
    {
      speakerId: "speaker_001",
      startTime: 0.0,
      endTime: 3.2,
      confidence: 0.85,
      text: "Hello everyone, welcome to the meeting",
      hasOverlap: false,
      overlappingSpeakers: []
    }
  ],
  speakers: {
    "speaker_001": {
      id: "speaker_001",
      displayName: "Speaker 1",
      color: "#2563EB",
      voiceCharacteristics: {
        pitch: 180.5,
        formantF1: 730,
        formantF2: 1090,
        speakingRate: 145,
        energyLevel: 0.7
      },
      totalSpeechTime: 45.3,
      segmentCount: 12,
      averageConfidence: 0.82,
      lastActive: 1698765432000
    }
  },
  totalSpeakers: 3,
  overallConfidence: 0.78,
  processingTime: "150ms",
  warnings: []
}
```

---

#### `identify_speaker`
Identify speaker from audio without full diarization.

**Request:**
```typescript
await invoke('identify_speaker', {
  audioSamples: Float32Array,
  sampleRate: 16000
});
```

**Response:**
```typescript
{
  speakerId: "speaker_003" | null,
  confidence: 0.92,
  isNewSpeaker: false,
  embedding: [0.1, -0.3, 0.7, ...], // 512-dimensional vector
  processingTime: "80ms"
}
```

---

### Speaker Profile Management

#### `create_speaker_profile`
Create a new speaker profile.

**Request:**
```typescript
await invoke('create_speaker_profile', {
  request: {
    displayName: "Alice Johnson",
    color: "#10B981",
    notes: "Product manager, usually speaks first",
    initialEmbedding?: [0.1, -0.2, ...] // Optional initial embedding
  }
});
```

**Response:**
```typescript
{
  id: "uuid-4-format",
  displayName: "Alice Johnson",
  color: "#10B981",
  voiceCharacteristics: {
    pitch: null,
    formantF1: null,
    formantF2: null,
    speakingRate: null,
    energyLevel: null
  },
  embeddings: [],
  totalSpeechTime: 0,
  segmentCount: 0,
  averageConfidence: 0,
  lastActive: 1698765432000,
  notes: "Product manager, usually speaks first",
  createdAt: 1698765432000,
  updatedAt: 1698765432000,
  isActive: true
}
```

---

#### `get_speaker_profile`
Retrieve a specific speaker profile.

**Request:**
```typescript
await invoke('get_speaker_profile', {
  speakerId: "uuid-4-format"
});
```

**Response:**
```typescript
DbSpeakerProfile | null
```

---

#### `list_speaker_profiles`
List all speaker profiles.

**Request:**
```typescript
await invoke('list_speaker_profiles', {
  activeOnly: false // Include inactive profiles
});
```

**Response:**
```typescript
DbSpeakerProfile[]
```

---

#### `update_speaker_profile`
Update an existing speaker profile.

**Request:**
```typescript
await invoke('update_speaker_profile', {
  speakerId: "uuid-4-format",
  request: {
    displayName: "Alice Johnson-Smith",
    color: "#EC4899",
    notes: "Updated role: VP Product"
  }
});
```

**Response:**
```typescript
DbSpeakerProfile | null
```

---

#### `delete_speaker_profile`
Delete a speaker profile.

**Request:**
```typescript
await invoke('delete_speaker_profile', {
  speakerId: "uuid-4-format"
});
```

**Response:**
```typescript
boolean // true if deleted, false if not found
```

---

### Advanced Operations

#### `find_similar_speakers`
Find speakers similar to a given voice embedding.

**Request:**
```typescript
await invoke('find_similar_speakers', {
  queryVector: [0.1, -0.3, 0.7, ...], // 512-dimensional embedding
  threshold: 0.8,                      // Similarity threshold
  maxResults: 5                        // Maximum number of results
});
```

**Response:**
```typescript
[
  {
    speakerId: "uuid-4-format",
    similarity: 0.92,
    profile: DbSpeakerProfile
  }
]
```

---

#### `merge_speaker_profiles`
Merge two speaker profiles (combine embeddings and statistics).

**Request:**
```typescript
await invoke('merge_speaker_profiles', {
  primarySpeakerId: "uuid-primary",
  secondarySpeakerId: "uuid-secondary"
});
```

**Response:**
```typescript
{
  success: true,
  mergedProfile: DbSpeakerProfile,
  message: "Successfully merged 2 profiles with 45 total embeddings"
}
```

---

#### `update_speaker_in_session`
Update speaker information during an active session.

**Request:**
```typescript
await invoke('update_speaker_in_session', {
  sessionId: "session_uuid",
  speakerId: "speaker_001",
  displayName: "Alice",
  color: "#10B981"
});
```

**Response:**
```typescript
{
  success: true,
  message: "Speaker updated successfully"
}
```

---

### Statistics and Analytics

#### `get_diarization_stats`
Get statistics for a diarization session.

**Request:**
```typescript
await invoke('get_diarization_stats', {
  sessionId: "session_uuid"
});
```

**Response:**
```typescript
{
  sessionId: "session_uuid",
  totalSpeakers: 4,
  speakerDistribution: {
    "speaker_001": {
      speakerId: "speaker_001",
      totalSpeechTime: 145.6,
      percentageOfTotal: 42.3,
      segmentCount: 23,
      averageSegmentLength: 6.3,
      averageConfidence: 0.87,
      wordsPerMinute: 165
    }
  },
  overallConfidence: 0.81,
  processingMetrics: {
    totalAudioSeconds: 344.2,
    processingTimeMs: 2150,
    realTimeFactor: 0.16,
    memoryUsageMb: 234.5,
    cpuUsagePercent: 15.2,
    embeddingsExtracted: 687,
    clusteringIterations: 12,
    cacheHitRate: 0.73
  },
  sessionDuration: 344.2,
  speakerChanges: 89,
  avgSegmentLength: 3.9
}
```

---

### Data Management

#### `export_speaker_profiles`
Export all speaker profiles for backup or transfer.

**Request:**
```typescript
await invoke('export_speaker_profiles', {
  includeEmbeddings: false // Exclude embeddings for smaller file size
});
```

**Response:**
```typescript
{
  profiles: DbSpeakerProfile[],
  exportedAt: 1698765432000,
  version: "2.0",
  includesEmbeddings: false,
  totalProfiles: 12,
  totalEmbeddings: 0
}
```

---

#### `import_speaker_profiles`
Import previously exported speaker profiles.

**Request:**
```typescript
await invoke('import_speaker_profiles', {
  importData: {
    profiles: DbSpeakerProfile[],
    version: "2.0"
  }
});
```

**Response:**
```typescript
{
  success: true,
  imported: 8,
  updated: 3,
  skipped: 1,
  errors: [],
  message: "Successfully imported 8 new profiles and updated 3 existing ones"
}
```

---

#### `clear_all_speaker_data`
Clear all speaker data (profiles, embeddings, statistics).

**Request:**
```typescript
await invoke('clear_all_speaker_data');
```

**Response:**
```typescript
{
  success: true,
  deletedProfiles: 15,
  deletedEmbeddings: 2847,
  clearedCache: true,
  message: "All speaker data cleared successfully"
}
```

---

## Real-time Events

The diarization system emits real-time events that can be listened to using Tauri's event system.

### Event Types

#### `speaker-detected`
Emitted when a new speaker is identified.

```typescript
listen('speaker-detected', (event) => {
  const payload = event.payload as {
    sessionId: string;
    speakerId: string;
    confidence: number;
    timestamp: number;
    isNewSpeaker: boolean;
    embedding?: number[]; // Optional 512-dimensional vector
  };
});
```

---

#### `speaker-activity`
Emitted when speaker activity changes.

```typescript
listen('speaker-activity', (event) => {
  const payload = event.payload as {
    sessionId: string;
    speakerId: string;
    isActive: boolean;
    confidence: number;
    startTime: number;
    endTime?: number;
    text?: string;
  };
});
```

---

#### `processing-progress`
Emitted during long-running diarization operations.

```typescript
listen('processing-progress', (event) => {
  const payload = event.payload as {
    sessionId: string;
    processedSeconds: number;
    totalSeconds: number;
    speakersFound: number;
  };
});
```

---

#### `diarization-error`
Emitted when diarization errors occur.

```typescript
listen('diarization-error', (event) => {
  const payload = event.payload as {
    sessionId: string;
    error: string;
    code: 'MODEL_LOAD_ERROR' | 'PROCESSING_ERROR' | 'MEMORY_ERROR' | 'TIMEOUT';
    recoverable: boolean;
    fallbackAvailable: boolean;
  };
});
```

---

#### `diarization-complete`
Emitted when diarization processing completes.

```typescript
listen('diarization-complete', (event) => {
  const payload = event.payload as {
    sessionId: string;
    totalSpeakers: number;
    processingTimeMs: number;
  };
});
```

---

## Error Handling

### Common Error Codes

| Code | Description | Recovery Action |
|------|-------------|----------------|
| `SERVICE_NOT_INITIALIZED` | Diarization service not started | Call `initialize_diarization_service` |
| `INVALID_AUDIO_FORMAT` | Audio data format not supported | Check sample rate and format |
| `INSUFFICIENT_AUDIO` | Audio segment too short | Ensure >1 second of audio |
| `MEMORY_LIMIT_EXCEEDED` | Processing exceeded memory limit | Reduce max_speakers or audio length |
| `MODEL_LOAD_ERROR` | Failed to load ML models | Check model files and permissions |
| `SPEAKER_NOT_FOUND` | Speaker ID not found in database | Verify speaker ID exists |
| `EMBEDDING_EXTRACTION_FAILED` | Could not extract voice embedding | Check audio quality and format |
| `CLUSTERING_ERROR` | Speaker clustering algorithm failed | Try different similarity threshold |
| `DATABASE_ERROR` | Speaker database operation failed | Check disk space and permissions |
| `TIMEOUT_ERROR` | Processing took too long | Reduce audio length or complexity |

### Error Response Format

```typescript
interface DiarizationError {
  code: string;
  message: string;
  details?: Record<string, any>;
  recoverable: boolean;
  timestamp: number;
}
```

### Example Error Handling

```typescript
try {
  const result = await invoke('diarize_audio_segment', {
    audioSamples: audioData,
    sampleRate: 44100
  });
  // Handle success
} catch (error) {
  const diarizationError = error as DiarizationError;
  
  switch (diarizationError.code) {
    case 'INSUFFICIENT_AUDIO':
      // Show user message about minimum audio length
      break;
    case 'MEMORY_LIMIT_EXCEEDED':
      // Suggest reducing quality or splitting audio
      break;
    case 'SERVICE_NOT_INITIALIZED':
      // Automatically initialize service
      await invoke('initialize_diarization_service', defaultConfig);
      break;
    default:
      // Generic error handling
      console.error('Diarization error:', diarizationError);
  }
}
```

## Performance Considerations

### Optimization Tips

**Memory Management:**
- Set appropriate `maxMemoryMb` limits based on available RAM
- Reduce `maxSpeakers` if not all speakers are needed
- Clear old sessions periodically to free memory

**Processing Speed:**
- Use `hardwareAcceleration: 'metal'` on Apple Silicon
- Increase `minSegmentDuration` to reduce processing overhead
- Disable `detectOverlaps` if overlapping speech is rare

**Accuracy vs Performance:**
- Lower `similarityThreshold` for more speaker separation (slower)
- Increase `embeddingWindowSize` for better accuracy (slower)
- Enable `enableAdaptiveClustering` for dynamic environments

### Batching Operations

For processing multiple audio segments, batch operations for better performance:

```typescript
// Instead of multiple single calls
const segments = await Promise.all(
  audioChunks.map(chunk => 
    invoke('diarize_audio_segment', {
      audioSamples: chunk,
      sampleRate: 44100
    })
  )
);
```

## TypeScript Integration

### Type Definitions

All API types are available in `src/types/diarization.ts`:

```typescript
import {
  DiarizationConfig,
  SpeakerSegment,
  SpeakerProfile,
  DiarizationResult,
  DiarizationEvent
} from '../types/diarization';
```

### React Hook Example

```typescript
import { useEffect, useState } from 'react';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/tauri';

export function useSpeakerDiarization(sessionId: string) {
  const [speakers, setSpeakers] = useState<Map<string, SpeakerProfile>>(new Map());
  const [isProcessing, setIsProcessing] = useState(false);
  
  useEffect(() => {
    const unlistenSpeakerDetected = listen('speaker-detected', (event) => {
      const { payload } = event as { payload: SpeakerDetectedEvent['payload'] };
      if (payload.sessionId === sessionId) {
        // Handle new speaker detection
        setSpeakers(prev => new Map(prev).set(payload.speakerId, {
          id: payload.speakerId,
          displayName: `Speaker ${payload.speakerId}`,
          // ... other properties
        }));
      }
    });
    
    return () => {
      unlistenSpeakerDetected.then(fn => fn());
    };
  }, [sessionId]);
  
  const updateSpeaker = async (speakerId: string, updates: SpeakerUpdates) => {
    try {
      await invoke('update_speaker_in_session', {
        sessionId,
        speakerId,
        displayName: updates.displayName,
        color: updates.color
      });
    } catch (error) {
      console.error('Failed to update speaker:', error);
    }
  };
  
  return {
    speakers,
    isProcessing,
    updateSpeaker
  };
}
```

## Migration Guide

### From V1 to V2

If migrating from a previous version:

1. **Configuration Changes:**
   - `max_speakers` → `maxSpeakers` (camelCase)
   - New required field: `hardwareAcceleration`
   - `clustering_threshold` → `similarityThreshold`

2. **API Changes:**
   - Speaker IDs are now UUIDs instead of sequential numbers
   - Event payloads use camelCase instead of snake_case
   - New required `sessionId` parameter for most operations

3. **Database Migration:**
   ```typescript
   // Automatic migration on first run
   await invoke('initialize_speaker_storage');
   ```

For questions or issues with the API, see [DIARIZATION-TROUBLESHOOTING.md](DIARIZATION-TROUBLESHOOTING.md) or file an issue on GitHub.