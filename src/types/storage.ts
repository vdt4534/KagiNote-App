// TypeScript types for speaker storage and database operations
// These types match the Rust models for consistent data handling

export interface SpeakerProfile {
  id: string;
  name: string;
  description?: string;
  color: string;
  voice_characteristics: VoiceCharacteristics;
  created_at: string; // ISO 8601 timestamp
  updated_at: string; // ISO 8601 timestamp
  identification_count: number;
  confidence_threshold: number;
  is_active: boolean;
}

export interface VoiceCharacteristics {
  pitch_range: [number, number]; // [min, max] in Hz
  pitch_mean: number; // Average fundamental frequency in Hz
  speaking_rate?: number; // Words per minute
  quality_features: Record<string, number>; // Various audio quality metrics
  gender?: string; // Gender classification if detected
  age_range?: [number, number]; // [min, max] age estimate
  language_markers: string[]; // Language/accent indicators
}

export interface VoiceEmbedding {
  id: string;
  speaker_id: string;
  vector: number[]; // Embedding vector (typically 256-512 dimensions)
  dimensions: number;
  model_name: string; // Model used to generate embedding
  quality_score: number; // 0.0-1.0 quality of source audio
  duration_seconds: number; // Length of audio segment
  created_at: string; // ISO 8601 timestamp
}

export interface MeetingSpeaker {
  id: string;
  meeting_id: string;
  speaker_id: string;
  display_name: string;
  speaking_time_seconds: number;
  segment_count: number;
  average_confidence: number;
  first_spoken_at: string; // ISO 8601 timestamp
  last_spoken_at: string; // ISO 8601 timestamp
  is_verified: boolean; // User-verified identification
  notes?: string;
}

export interface SpeakerStats {
  speaker_id: string;
  speaker_name: string;
  total_meetings: number;
  total_speaking_time_seconds: number;
  total_segments: number;
  average_confidence: number;
  first_meeting_date: string; // ISO 8601 timestamp
  last_meeting_date: string; // ISO 8601 timestamp
}

export interface MeetingStats {
  meeting_id: string;
  total_duration_seconds: number;
  total_speakers: number;
  speakers: MeetingSpeakerStats[];
  created_at: string; // ISO 8601 timestamp
}

export interface MeetingSpeakerStats {
  speaker_id: string;
  display_name: string;
  speaking_time_seconds: number;
  speaking_percentage: number; // 0-100 percentage of meeting
  segment_count: number;
  average_confidence: number;
  is_verified: boolean;
}

export interface SimilarSpeaker {
  speaker: SpeakerProfile;
  similarity_score: number; // 0.0-1.0 cosine similarity
  matching_embeddings: number; // Number of embeddings that matched
}

export interface SpeakerIdentification {
  speaker?: SpeakerProfile; // Identified speaker (if any)
  confidence: number; // 0.0-1.0 identification confidence
  is_new_speaker: boolean; // Whether this appears to be a new speaker
  alternatives: Array<{
    speaker: SpeakerProfile;
    confidence: number;
  }>; // Alternative speaker candidates
}

// Request/Response types for API operations

export interface CreateSpeakerProfileRequest {
  name: string;
  description?: string;
  color?: string; // Hex color code
  confidence_threshold?: number; // 0.0-1.0, defaults to 0.7
}

export interface UpdateSpeakerProfileRequest {
  name?: string;
  description?: string;
  color?: string;
  confidence_threshold?: number;
  is_active?: boolean;
}

export interface UpsertMeetingSpeakerRequest {
  meeting_id: string;
  speaker_id: string;
  display_name?: string;
  speaking_time_seconds?: number;
  segment_count?: number;
  confidence_score?: number;
  is_verified?: boolean;
  notes?: string;
}

export interface SpeakerParticipation {
  speaker: SpeakerProfile;
  meeting_speaker: MeetingSpeaker;
  participation_percentage: number;
}

// Frontend-specific storage types

export interface SpeakerStorageConfig {
  max_profiles: number; // Maximum number of profiles to store
  max_embeddings_per_speaker: number; // Limit embeddings per speaker
  auto_cleanup_days: number; // Days before cleaning up unused profiles
  export_format: 'json' | 'csv'; // Default export format
}

export interface LocalSpeakerData {
  profiles: SpeakerProfile[];
  embeddings: VoiceEmbedding[];
  meetings: MeetingSpeaker[];
  config: SpeakerStorageConfig;
  last_sync: string; // ISO 8601 timestamp of last backend sync
  version: number; // Data format version for migrations
}

export interface ExportData {
  metadata: {
    exported_at: string; // ISO 8601 timestamp
    app_version: string;
    total_profiles: number;
    total_meetings: number;
    format_version: number;
  };
  profiles: SpeakerProfile[];
  embeddings?: VoiceEmbedding[]; // Optional, may be excluded for privacy
  meeting_speakers: MeetingSpeaker[];
  statistics: {
    [speaker_id: string]: SpeakerStats;
  };
}

export interface ImportResult {
  success: boolean;
  imported_profiles: number;
  imported_embeddings: number;
  imported_meetings: number;
  errors: string[];
  warnings: string[];
}

// Database connection and migration types

export interface DatabaseConfig {
  path: string;
  enable_wal: boolean; // Write-ahead logging
  cache_size_mb: number;
  sync_mode: 'full' | 'normal' | 'off';
}

export interface MigrationInfo {
  version: number;
  name: string;
  applied_at: string; // ISO 8601 timestamp
  checksum: string;
}

export interface DatabaseStats {
  total_profiles: number;
  total_embeddings: number;
  total_meeting_speakers: number;
  database_size_mb: number;
  last_vacuum: string; // ISO 8601 timestamp
  performance_stats: {
    avg_query_time_ms: number;
    cache_hit_ratio: number;
    total_queries: number;
  };
}

// Search and filtering types

export interface SpeakerSearchFilters {
  name_query?: string; // Partial name match
  created_after?: string; // ISO 8601 timestamp
  created_before?: string; // ISO 8601 timestamp
  min_confidence?: number;
  has_embeddings?: boolean;
  is_active?: boolean;
  color?: string;
  min_identification_count?: number;
}

export interface SpeakerSearchResults {
  profiles: SpeakerProfile[];
  total_count: number;
  page: number;
  page_size: number;
  has_more: boolean;
  query_time_ms: number;
}

export interface EmbeddingSearchOptions {
  threshold: number; // Similarity threshold (0.0-1.0)
  max_results: number;
  include_inactive?: boolean; // Include inactive speaker profiles
  model_filter?: string; // Filter by embedding model
  min_quality?: number; // Minimum quality score
}

// Validation schemas (for runtime type checking)

export const SpeakerProfileSchema = {
  id: 'string',
  name: 'string',
  description: 'string?',
  color: 'string',
  voice_characteristics: 'object',
  created_at: 'string',
  updated_at: 'string',
  identification_count: 'number',
  confidence_threshold: 'number',
  is_active: 'boolean',
} as const;

export const VoiceEmbeddingSchema = {
  id: 'string',
  speaker_id: 'string',
  vector: 'array<number>',
  dimensions: 'number',
  model_name: 'string',
  quality_score: 'number',
  duration_seconds: 'number',
  created_at: 'string',
} as const;

// Utility type guards

export function isSpeakerProfile(obj: any): obj is SpeakerProfile {
  return (
    obj &&
    typeof obj.id === 'string' &&
    typeof obj.name === 'string' &&
    typeof obj.color === 'string' &&
    typeof obj.created_at === 'string' &&
    typeof obj.updated_at === 'string' &&
    typeof obj.identification_count === 'number' &&
    typeof obj.confidence_threshold === 'number' &&
    typeof obj.is_active === 'boolean' &&
    obj.voice_characteristics &&
    typeof obj.voice_characteristics === 'object'
  );
}

export function isVoiceEmbedding(obj: any): obj is VoiceEmbedding {
  return (
    obj &&
    typeof obj.id === 'string' &&
    typeof obj.speaker_id === 'string' &&
    Array.isArray(obj.vector) &&
    typeof obj.dimensions === 'number' &&
    typeof obj.model_name === 'string' &&
    typeof obj.quality_score === 'number' &&
    typeof obj.duration_seconds === 'number' &&
    typeof obj.created_at === 'string'
  );
}

export function isMeetingSpeaker(obj: any): obj is MeetingSpeaker {
  return (
    obj &&
    typeof obj.id === 'string' &&
    typeof obj.meeting_id === 'string' &&
    typeof obj.speaker_id === 'string' &&
    typeof obj.display_name === 'string' &&
    typeof obj.speaking_time_seconds === 'number' &&
    typeof obj.segment_count === 'number' &&
    typeof obj.average_confidence === 'number' &&
    typeof obj.first_spoken_at === 'string' &&
    typeof obj.last_spoken_at === 'string' &&
    typeof obj.is_verified === 'boolean'
  );
}

// Constants for default values

export const DEFAULT_CONFIDENCE_THRESHOLD = 0.7;
export const DEFAULT_MAX_PROFILES = 1000;
export const DEFAULT_MAX_EMBEDDINGS_PER_SPEAKER = 50;
export const DEFAULT_AUTO_CLEANUP_DAYS = 365;

export const SPEAKER_COLORS = [
  '#3B82F6', '#EF4444', '#10B981', '#F59E0B', 
  '#8B5CF6', '#EC4899', '#06B6D4', '#84CC16',
  '#F97316', '#6366F1', '#14B8A6', '#F472B6'
] as const;

export type SpeakerColor = typeof SPEAKER_COLORS[number];