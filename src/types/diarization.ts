/**
 * Speaker Diarization Type Definitions
 * 
 * TypeScript interfaces for speaker identification and diarization
 * These types mirror the Rust definitions for seamless integration
 */

/**
 * Configuration for speaker diarization
 */
export interface DiarizationConfig {
  /** Maximum number of speakers to detect (2-10) */
  maxSpeakers: number;
  
  /** Minimum number of speakers to detect (1-10) */
  minSpeakers: number;
  
  /** Minimum segment length in milliseconds */
  minSegmentLength: number;
  
  /** Window size for speaker embedding extraction (ms) */
  embeddingWindowSize: number;
  
  /** Clustering threshold for speaker separation (0.0-1.0) */
  clusteringThreshold: number;
  
  /** Enable adaptive clustering based on confidence */
  enableAdaptiveClustering: boolean;
  
  /** Hardware acceleration mode */
  hardwareAcceleration: 'auto' | 'cpu' | 'metal' | 'cuda';
  
  /** VAD threshold for speech detection (0.0-1.0) */
  vadThreshold: number;
  
  /** Enable overlapping speech detection */
  detectOverlaps: boolean;
  
  /** Maximum memory usage in MB */
  maxMemoryMb: number;
}

/**
 * Speaker segment with timing and confidence
 */
export interface SpeakerSegment {
  /** Unique speaker identifier */
  speakerId: string;
  
  /** Start time in seconds */
  startTime: number;
  
  /** End time in seconds */
  endTime: number;
  
  /** Confidence score (0.0-1.0) */
  confidence: number;
  
  /** Optional transcribed text for this segment */
  text?: string;
  
  /** Speaker embedding for this segment */
  embedding?: SpeakerEmbedding;
  
  /** Whether this segment overlaps with another speaker */
  hasOverlap: boolean;
  
  /** Overlapping speaker IDs if any */
  overlappingSpeakers: string[];
}

/**
 * Speaker embedding representation
 */
export interface SpeakerEmbedding {
  /** 512-dimensional embedding vector */
  vector: number[];
  
  /** Quality score of the embedding (0.0-1.0) */
  quality: number;
  
  /** Timestamp when extracted */
  extractedAt: number;
  
  /** Duration of audio used for extraction (ms) */
  audioDurationMs: number;
}

/**
 * Complete diarization result
 */
export interface DiarizationResult {
  /** Session identifier */
  sessionId: string;
  
  /** All detected speaker segments */
  segments: SpeakerSegment[];
  
  /** Speaker profiles */
  speakers: Map<string, SpeakerProfile>;
  
  /** Total number of unique speakers */
  totalSpeakers: number;
  
  /** Overall confidence score */
  overallConfidence: number;
  
  /** Processing metrics */
  metrics: ProcessingMetrics;
  
  /** Any warnings during processing */
  warnings: string[];
}

/**
 * Speaker profile information
 */
export interface SpeakerProfile {
  /** Unique speaker identifier */
  id: string;
  
  /** Display name (can be customized by user) */
  displayName: string;
  
  /** Color for UI display (hex format) */
  color: string;
  
  /** Voice characteristics */
  voiceCharacteristics: VoiceCharacteristics;
  
  /** All embeddings for this speaker */
  embeddings: SpeakerEmbedding[];
  
  /** Total speech time in seconds */
  totalSpeechTime: number;
  
  /** Number of segments */
  segmentCount: number;
  
  /** Average confidence score */
  averageConfidence: number;
  
  /** Last active timestamp */
  lastActive: number;
  
  /** User notes (optional) */
  notes?: string;
}

/**
 * Voice characteristics for a speaker
 */
export interface VoiceCharacteristics {
  /** Average pitch in Hz */
  pitch?: number;
  
  /** First formant frequency (F1) in Hz */
  formantF1?: number;
  
  /** Second formant frequency (F2) in Hz */
  formantF2?: number;
  
  /** Speaking rate in words per minute */
  speakingRate?: number;
  
  /** Voice energy level (0.0-1.0) */
  energyLevel?: number;
}

/**
 * Processing metrics for performance monitoring
 */
export interface ProcessingMetrics {
  /** Total audio processed in seconds */
  totalAudioSeconds: number;
  
  /** Processing time in milliseconds */
  processingTimeMs: number;
  
  /** Real-time factor (processing_time / audio_duration) */
  realTimeFactor: number;
  
  /** Memory usage in MB */
  memoryUsageMb: number;
  
  /** CPU usage percentage */
  cpuUsagePercent: number;
  
  /** Number of embeddings extracted */
  embeddingsExtracted: number;
  
  /** Number of clustering iterations */
  clusteringIterations: number;
  
  /** Cache hit rate for embeddings */
  cacheHitRate: number;
}

/**
 * Speaker update request
 */
export interface SpeakerUpdates {
  displayName?: string;
  color?: string;
  notes?: string;
}

/**
 * Diarization statistics
 */
export interface DiarizationStatistics {
  /** Total number of speakers */
  totalSpeakers: number;
  
  /** Speaker distribution */
  speakerDistribution: Map<string, SpeakerStats>;
  
  /** Overall confidence */
  overallConfidence: number;
  
  /** Processing metrics */
  processingMetrics: ProcessingMetrics;
  
  /** Session duration in seconds */
  sessionDuration: number;
  
  /** Number of speaker changes */
  speakerChanges: number;
  
  /** Average segment length in seconds */
  avgSegmentLength: number;
}

/**
 * Individual speaker statistics
 */
export interface SpeakerStats {
  /** Speaker identifier */
  speakerId: string;
  
  /** Total speech time in seconds */
  totalSpeechTime: number;
  
  /** Percentage of total speaking time */
  percentageOfTotal: number;
  
  /** Number of segments */
  segmentCount: number;
  
  /** Average segment length in seconds */
  averageSegmentLength: number;
  
  /** Average confidence score */
  averageConfidence: number;
  
  /** Words per minute (if transcription available) */
  wordsPerMinute?: number;
}

/**
 * Diarization event types for real-time updates
 */
export type DiarizationEvent = 
  | SpeakerDetectedEvent
  | SpeakerActivityEvent
  | ProcessingProgressEvent
  | DiarizationErrorEvent
  | DiarizationCompleteEvent;

/**
 * New speaker detected event
 */
export interface SpeakerDetectedEvent {
  type: 'speaker-detected';
  payload: {
    sessionId: string;
    speakerId: string;
    confidence: number;
    timestamp: number;
    isNewSpeaker: boolean;
    embedding?: number[];
  };
}

/**
 * Speaker activity change event
 */
export interface SpeakerActivityEvent {
  type: 'speaker-activity';
  payload: {
    sessionId: string;
    speakerId: string;
    isActive: boolean;
    confidence: number;
    startTime: number;
    endTime?: number;
    text?: string;
  };
}

/**
 * Processing progress update event
 */
export interface ProcessingProgressEvent {
  type: 'processing-progress';
  payload: {
    sessionId: string;
    processedSeconds: number;
    totalSeconds: number;
    speakersFound: number;
  };
}

/**
 * Diarization error event
 */
export interface DiarizationErrorEvent {
  type: 'diarization-error';
  payload: {
    sessionId: string;
    error: string;
    code: 'MODEL_LOAD_ERROR' | 'PROCESSING_ERROR' | 'MEMORY_ERROR' | 'TIMEOUT';
    recoverable: boolean;
    fallbackAvailable: boolean;
  };
}

/**
 * Diarization complete event
 */
export interface DiarizationCompleteEvent {
  type: 'diarization-complete';
  payload: {
    sessionId: string;
    totalSpeakers: number;
    processingTimeMs: number;
  };
}

/**
 * Audio buffer state for shared access
 */
export interface BufferState {
  /** Current write position */
  writePosition: number;
  
  /** Read positions for each consumer */
  readPositions: Map<string, number>;
  
  /** Buffer capacity */
  capacity: number;
  
  /** Utilization percentage */
  utilization: number;
  
  /** Sample rate */
  sampleRate: number;
}

/**
 * Final merged segment with transcription and speaker
 */
export interface FinalSegment {
  /** Start time in seconds */
  startTime: number;
  
  /** End time in seconds */
  endTime: number;
  
  /** Speaker identifier */
  speakerId: string;
  
  /** Transcribed text */
  text: string;
  
  /** Transcription confidence */
  transcriptionConfidence: number;
  
  /** Speaker confidence */
  speakerConfidence: number;
  
  /** Combined confidence */
  overallConfidence: number;
  
  /** Whether this segment was modified during merging */
  wasMerged: boolean;
}

/**
 * Request/Response types for Tauri commands
 */

export interface StartDiarizationRequest {
  sessionId: string;
  config: DiarizationConfig;
}

export interface StartDiarizationResponse {
  sessionId: string;
  status: 'initializing' | 'ready' | 'error';
  estimatedProcessingTime: number;
  error?: string;
}

export interface ProcessAudioRequest {
  sessionId: string;
  audioData: Float32Array;
  timestamp: number;
  isLastChunk: boolean;
}

export interface ProcessAudioResponse {
  sessionId: string;
  segments: SpeakerSegment[];
  processingTime: number;
  bufferUtilization: number;
}

export interface UpdateSpeakerRequest {
  sessionId: string;
  speakerId: string;
  updates: SpeakerUpdates;
}

export interface UpdateSpeakerResponse {
  success: boolean;
  speaker: SpeakerProfile;
}

export interface GetStatisticsRequest {
  sessionId: string;
}

export interface ExportStatisticsRequest {
  sessionId: string;
  format: 'json' | 'csv';
  includeEmbeddings: boolean;
}

export interface ExportStatisticsResponse {
  data: string;
  filename: string;
  mimeType: string;
}

/**
 * UI State Management Types
 */

export interface SpeakerUIState {
  /** Map of speaker IDs to their UI state */
  speakers: Map<string, SpeakerUIInfo>;
  
  /** Currently active speaker */
  activeSpeaker?: string;
  
  /** Selected speakers for filtering */
  selectedSpeakers: Set<string>;
  
  /** Show speaker statistics panel */
  showStatistics: boolean;
  
  /** Search query for speakers */
  searchQuery: string;
  
  /** Sort order for speakers */
  sortBy: 'name' | 'time' | 'confidence';
  
  /** Error message if any */
  error?: string;
  
  /** Loading state */
  isLoading: boolean;
}

export interface SpeakerUIInfo {
  /** Speaker profile */
  profile: SpeakerProfile;
  
  /** UI-specific properties */
  isHovered: boolean;
  isSelected: boolean;
  isEditing: boolean;
  
  /** Temporary name while editing */
  tempName?: string;
  
  /** Animation state */
  animationState: 'idle' | 'speaking' | 'fading';
}

/**
 * Color palette for speakers
 */
export const SPEAKER_COLORS = [
  '#2563EB', // Blue
  '#10B981', // Green
  '#F59E0B', // Amber
  '#EF4444', // Red
  '#8B5CF6', // Purple
  '#EC4899', // Pink
  '#14B8A6', // Teal
  '#F97316', // Orange
  '#6366F1', // Indigo
  '#84CC16', // Lime
] as const;

/**
 * Default diarization configuration
 */
export const DEFAULT_DIARIZATION_CONFIG: DiarizationConfig = {
  maxSpeakers: 8,
  minSpeakers: 2,
  minSegmentLength: 1500,
  embeddingWindowSize: 3000,
  clusteringThreshold: 0.7,
  enableAdaptiveClustering: true,
  hardwareAcceleration: 'auto',
  vadThreshold: 0.5,
  detectOverlaps: true,
  maxMemoryMb: 500,
};

/**
 * Helper function to get speaker color
 */
export function getSpeakerColor(index: number): string {
  return SPEAKER_COLORS[index % SPEAKER_COLORS.length];
}

/**
 * Helper function to format speaker time
 */
export function formatSpeakerTime(seconds: number): string {
  const minutes = Math.floor(seconds / 60);
  const remainingSeconds = Math.floor(seconds % 60);
  return `${minutes}m ${remainingSeconds}s`;
}

/**
 * Helper function to calculate speaker similarity
 */
export function calculateSpeakerSimilarity(
  embedding1: SpeakerEmbedding,
  embedding2: SpeakerEmbedding
): number {
  if (embedding1.vector.length !== embedding2.vector.length) {
    return 0;
  }
  
  let dotProduct = 0;
  let norm1 = 0;
  let norm2 = 0;
  
  for (let i = 0; i < embedding1.vector.length; i++) {
    dotProduct += embedding1.vector[i] * embedding2.vector[i];
    norm1 += embedding1.vector[i] * embedding1.vector[i];
    norm2 += embedding2.vector[i] * embedding2.vector[i];
  }
  
  norm1 = Math.sqrt(norm1);
  norm2 = Math.sqrt(norm2);
  
  if (norm1 === 0 || norm2 === 0) {
    return 0;
  }
  
  return dotProduct / (norm1 * norm2);
}