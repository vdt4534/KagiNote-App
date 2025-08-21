//! Speaker diarization type definitions
//! 
//! Core types for speaker identification and diarization pipeline

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use thiserror::Error;

/// Configuration for speaker diarization system.
/// 
/// This structure contains all parameters needed to configure the diarization
/// system for optimal performance in different scenarios. The configuration
/// affects speaker detection sensitivity, processing speed, memory usage,
/// and accuracy trade-offs.
/// 
/// # Performance Tuning
/// 
/// Different use cases benefit from different configuration approaches:
/// 
/// - **Small meetings (2-4 people)**: Higher similarity threshold (0.8),
///   shorter embedding windows (2000ms), disabled overlap detection
/// - **Large meetings (5-8 people)**: Lower similarity threshold (0.6),
///   longer embedding windows (3000ms), enabled adaptive clustering  
/// - **Poor audio quality**: Higher VAD threshold (0.6), longer minimum
///   segment duration (2.0s), disabled overlap detection
/// - **High accuracy needs**: Maximum embedding window size (4000ms),
///   Metal/CUDA acceleration, adaptive clustering enabled
/// 
/// # Examples
/// 
/// ```rust
/// // Configuration for small, high-quality meetings
/// let small_meeting_config = DiarizationConfig {
///     max_speakers: 4,
///     similarity_threshold: 0.8,
///     min_segment_duration: 1.0,
///     detect_overlaps: false,
///     max_memory_mb: 200,
///     ..Default::default()
/// };
/// 
/// // Configuration for large, complex meetings
/// let large_meeting_config = DiarizationConfig {
///     max_speakers: 8, 
///     similarity_threshold: 0.6,
///     min_segment_duration: 1.0,
///     detect_overlaps: true,
///     enable_adaptive_clustering: true,
///     max_memory_mb: 500,
///     ..Default::default()
/// };
/// 
/// // Configuration for poor audio quality
/// let poor_audio_config = DiarizationConfig {
///     vad_threshold: 0.6,
///     similarity_threshold: 0.65,
///     min_segment_duration: 2.0,
///     detect_overlaps: false,
///     max_memory_mb: 300,
///     ..Default::default()
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiarizationConfig {
    /// Maximum number of speakers to detect (2-10)
    pub max_speakers: u8,
    
    /// Minimum number of speakers to detect (1-10)
    pub min_speakers: u8,
    
    /// Embedding dimension (512 for compatibility)
    pub embedding_dimension: usize,
    
    /// Similarity threshold for clustering (0.0-1.0)
    pub similarity_threshold: f32,
    
    /// Minimum segment duration in seconds
    pub min_segment_duration: f32,
    
    /// Speaker change detection threshold (0.0-1.0)
    pub speaker_change_detection_threshold: f32,
    
    /// Minimum segment length in milliseconds (legacy)
    pub min_segment_length: u32,
    
    /// Window size for speaker embedding extraction (ms)
    pub embedding_window_size: u32,
    
    /// Clustering threshold for speaker separation (0.0-1.0) (legacy)
    pub clustering_threshold: f32,
    
    /// Enable adaptive clustering based on confidence
    pub enable_adaptive_clustering: bool,
    
    /// Hardware acceleration mode
    pub hardware_acceleration: HardwareAcceleration,
    
    /// VAD threshold for speech detection (0.0-1.0)
    pub vad_threshold: f32,
    
    /// Enable overlapping speech detection
    pub detect_overlaps: bool,
    
    /// Maximum memory usage in MB
    pub max_memory_mb: usize,
}

impl Default for DiarizationConfig {
    fn default() -> Self {
        Self {
            max_speakers: 8,
            min_speakers: 2,
            embedding_dimension: 512,
            similarity_threshold: 0.7,
            min_segment_duration: 1.0,
            speaker_change_detection_threshold: 0.6,
            min_segment_length: 1500,
            embedding_window_size: 3000,
            clustering_threshold: 0.7,
            enable_adaptive_clustering: true,
            hardware_acceleration: HardwareAcceleration::Auto,
            vad_threshold: 0.5,
            detect_overlaps: true,
            max_memory_mb: 500,
        }
    }
}

/// Hardware acceleration options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HardwareAcceleration {
    Auto,
    CPU,
    Metal,
    CUDA,
}

/// Speaker segment with timing and confidence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeakerSegment {
    /// Unique speaker identifier
    pub speaker_id: String,
    
    /// Start time in seconds
    pub start_time: f32,
    
    /// End time in seconds
    pub end_time: f32,
    
    /// Confidence score (0.0-1.0)
    pub confidence: f32,
    
    /// Optional transcribed text for this segment
    pub text: Option<String>,
    
    /// Speaker embedding for this segment
    pub embedding: Option<SpeakerEmbedding>,
    
    /// Whether this segment overlaps with another speaker
    pub has_overlap: bool,
    
    /// Overlapping speaker IDs if any
    pub overlapping_speakers: Vec<String>,
}

/// Mathematical representation of a speaker's voice characteristics.
/// 
/// A speaker embedding is a 512-dimensional vector that captures unique
/// voice characteristics such as pitch, formant frequencies, speaking patterns,
/// and other acoustic features. These embeddings enable speaker identification
/// and similarity comparison without storing actual audio data.
/// 
/// # Privacy
/// 
/// Embeddings are mathematical representations that cannot be reverse-engineered
/// to recreate original audio. They preserve speaker identity for recognition
/// while maintaining complete privacy of speech content.
/// 
/// # Similarity Calculation
/// 
/// Embeddings use cosine similarity to measure voice similarity:
/// - 1.0 = Identical voices (same person)
/// - 0.8-0.9 = Very similar voices (likely same person)  
/// - 0.6-0.8 = Somewhat similar voices (possible same person)
/// - 0.4-0.6 = Different but related voices (siblings, similar age/gender)
/// - 0.0-0.4 = Very different voices (different people)
/// 
/// # Examples
/// 
/// ```rust
/// // Compare two embeddings
/// let similarity = embedding1.similarity(&embedding2);
/// if similarity > 0.7 {
///     println!("Likely the same speaker (similarity: {:.2})", similarity);
/// } else {
///     println!("Different speakers (similarity: {:.2})", similarity);  
/// }
/// 
/// // Check embedding quality
/// if embedding.confidence > 0.8 {
///     println!("High-quality embedding suitable for identification");
/// } else {
///     println!("Low-quality embedding, may need more audio");
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeakerEmbedding {
    /// 512-dimensional embedding vector
    pub vector: Vec<f32>,
    
    /// Confidence score of the embedding (0.0-1.0)
    pub confidence: f32,
    
    /// Start timestamp in seconds
    pub timestamp_start: f32,
    
    /// End timestamp in seconds
    pub timestamp_end: f32,
    
    /// Optional speaker ID (assigned during clustering)
    pub speaker_id: Option<String>,
    
    /// Quality score of the embedding (0.0-1.0) (legacy)
    pub quality: f32,
    
    /// Timestamp when extracted (legacy)
    pub extracted_at: u64,
    
    /// Duration of audio used for extraction (ms) (legacy)
    pub audio_duration_ms: u32,
}

impl SpeakerEmbedding {
    /// Calculates cosine similarity between two speaker embeddings.
    /// 
    /// This method computes the cosine similarity between two embedding vectors,
    /// which measures how similar two speakers' voices are. The result ranges
    /// from 0.0 (completely different) to 1.0 (identical).
    /// 
    /// # Algorithm
    /// 
    /// Cosine similarity is calculated as:
    /// ```text
    /// similarity = (A · B) / (||A|| × ||B||)
    /// ```
    /// Where A and B are the embedding vectors, · is dot product, and ||·|| is magnitude.
    /// 
    /// # Arguments
    /// 
    /// * `other` - Another speaker embedding to compare with
    /// 
    /// # Returns
    /// 
    /// A similarity score between 0.0 and 1.0:
    /// - **0.9-1.0**: Almost certainly the same speaker
    /// - **0.7-0.9**: Very likely the same speaker  
    /// - **0.5-0.7**: Possibly the same speaker (needs review)
    /// - **0.3-0.5**: Probably different speakers
    /// - **0.0-0.3**: Definitely different speakers
    /// 
    /// Returns 0.0 if embeddings have different dimensions or zero magnitude.
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// let embedding1 = SpeakerEmbedding { /* ... */ };
    /// let embedding2 = SpeakerEmbedding { /* ... */ };
    /// 
    /// let similarity = embedding1.similarity(&embedding2);
    /// 
    /// match similarity {
    ///     s if s > 0.8 => println!("Same speaker (confidence: high)"),
    ///     s if s > 0.6 => println!("Likely same speaker (confidence: medium)"),
    ///     s if s > 0.4 => println!("Uncertain (confidence: low)"),
    ///     _ => println!("Different speakers"),
    /// }
    /// ```
    /// 
    /// # Performance
    /// 
    /// - Time complexity: O(n) where n is embedding dimension (512)
    /// - Typical execution time: <1µs
    /// - No memory allocation required
    pub fn similarity(&self, other: &SpeakerEmbedding) -> f32 {
        if self.vector.len() != other.vector.len() {
            return 0.0;
        }
        
        let dot_product: f32 = self.vector.iter()
            .zip(other.vector.iter())
            .map(|(a, b)| a * b)
            .sum();
        
        let norm_a: f32 = self.vector.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = other.vector.iter().map(|x| x * x).sum::<f32>().sqrt();
        
        if norm_a == 0.0 || norm_b == 0.0 {
            return 0.0;
        }
        
        dot_product / (norm_a * norm_b)
    }
}

/// Complete diarization result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiarizationResult {
    /// All detected speaker segments
    pub segments: Vec<SpeakerSegment>,
    
    /// Speaker profiles
    pub speakers: HashMap<String, SpeakerProfile>,
    
    /// Total number of unique speakers
    pub total_speakers: usize,
    
    /// Overall confidence score
    pub overall_confidence: f32,
    
    /// Processing time duration
    pub processing_time: Duration,
    
    /// Session identifier (optional)
    pub session_id: Option<String>,
    
    /// Processing metrics (optional)
    pub metrics: Option<ProcessingMetrics>,
    
    /// Any warnings during processing
    pub warnings: Vec<String>,
}

/// Speaker profile information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeakerProfile {
    /// Unique speaker identifier
    pub id: String,
    
    /// Display name (can be customized by user)
    pub display_name: String,
    
    /// Color for UI display (hex format)
    pub color: String,
    
    /// Voice characteristics
    pub voice_characteristics: VoiceCharacteristics,
    
    /// All embeddings for this speaker  
    pub embeddings: Vec<SpeakerEmbedding>,
    
    /// Total speech time in seconds
    pub total_speech_time: f32,
    
    /// Number of segments
    pub segment_count: usize,
    
    /// Average confidence score
    pub average_confidence: f32,
    
    /// Last active timestamp
    pub last_active: u64,
    
    /// User notes (optional)
    pub notes: Option<String>,
}

/// Voice characteristics for a speaker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceCharacteristics {
    /// Average pitch in Hz
    pub pitch: Option<f32>,
    
    /// First formant frequency (F1) in Hz
    pub formant_f1: Option<f32>,
    
    /// Second formant frequency (F2) in Hz
    pub formant_f2: Option<f32>,
    
    /// Speaking rate in words per minute
    pub speaking_rate: Option<f32>,
    
    /// Voice energy level (0.0-1.0)
    pub energy_level: Option<f32>,
}

impl Default for VoiceCharacteristics {
    fn default() -> Self {
        Self {
            pitch: None,
            formant_f1: None,
            formant_f2: None,
            speaking_rate: None,
            energy_level: None,
        }
    }
}

/// Processing metrics for performance monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingMetrics {
    /// Total audio processed in seconds
    pub total_audio_seconds: f32,
    
    /// Processing time in milliseconds
    pub processing_time_ms: u64,
    
    /// Real-time factor (processing_time / audio_duration)
    pub real_time_factor: f32,
    
    /// Memory usage in MB
    pub memory_usage_mb: f32,
    
    /// CPU usage percentage
    pub cpu_usage_percent: f32,
    
    /// Number of embeddings extracted
    pub embeddings_extracted: usize,
    
    /// Number of clustering iterations
    pub clustering_iterations: usize,
    
    /// Cache hit rate for embeddings
    pub cache_hit_rate: f32,
}

/// Speaker update request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeakerUpdates {
    pub display_name: Option<String>,
    pub color: Option<String>,
    pub notes: Option<String>,
}

/// Diarization statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiarizationStatistics {
    /// Total number of speakers
    pub total_speakers: usize,
    
    /// Speaker distribution
    pub speaker_distribution: HashMap<String, SpeakerStats>,
    
    /// Overall confidence
    pub overall_confidence: f32,
    
    /// Processing metrics
    pub processing_metrics: ProcessingMetrics,
    
    /// Session duration in seconds
    pub session_duration: f32,
    
    /// Number of speaker changes
    pub speaker_changes: usize,
    
    /// Average segment length in seconds
    pub avg_segment_length: f32,
}

/// Individual speaker statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeakerStats {
    /// Speaker identifier
    pub speaker_id: String,
    
    /// Total speech time in seconds
    pub total_speech_time: f32,
    
    /// Percentage of total speaking time
    pub percentage_of_total: f32,
    
    /// Number of segments
    pub segment_count: usize,
    
    /// Average segment length in seconds
    pub average_segment_length: f32,
    
    /// Average confidence score
    pub average_confidence: f32,
    
    /// Words per minute (if transcription available)
    pub words_per_minute: Option<f32>,
}

/// Overlap resolution strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OverlapStrategy {
    /// Keep both speakers in overlap
    KeepBoth,
    
    /// Assign to higher confidence speaker
    HighestConfidence,
    
    /// Split overlap equally
    SplitEqually,
    
    /// Use voice activity detection
    UseVAD,
}

/// Clustering algorithm choice
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClusteringAlgorithm {
    /// Agglomerative hierarchical clustering
    Agglomerative,
    
    /// Spectral clustering
    Spectral,
    
    /// DBSCAN clustering
    DBSCAN,
    
    /// Online clustering for real-time
    Online,
}

/// Diarization errors
#[derive(Debug, Error)]
pub enum DiarizationError {
    #[error("Model loading failed: {message}")]
    ModelLoadError { message: String },
    
    #[error("Processing error: {message}")]
    ProcessingError { message: String },
    
    #[error("Memory limit exceeded: used {used}MB, limit {limit}MB")]
    MemoryError { used: usize, limit: usize },
    
    #[error("Processing timeout after {seconds}s")]
    TimeoutError { seconds: u64 },
    
    #[error("Invalid configuration: {message}")]
    ConfigError { message: String },
    
    #[error("Audio format error: {message}")]
    AudioFormatError { message: String },
    
    #[error("Hardware acceleration not available: {device}")]
    HardwareError { device: String },
    
    #[error("Clustering failed: {message}")]
    ClusteringError { message: String },
    
    #[error("Embedding extraction failed: {message}")]
    EmbeddingError { message: String },
    
    #[error("Session not found: {session_id}")]
    SessionNotFound { session_id: String },
    
    // Test-expected error variants
    #[error("Insufficient audio for processing")]
    InsufficientAudio,
    
    #[error("Invalid sample rate")]
    InvalidSampleRate,
}

/// Diarization event types for real-time updates
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum DiarizationEvent {
    /// New speaker detected
    SpeakerDetected {
        session_id: String,
        speaker_id: String,
        confidence: f32,
        timestamp: f32,
        is_new_speaker: bool,
    },
    
    /// Speaker activity change
    SpeakerActivity {
        session_id: String,
        speaker_id: String,
        is_active: bool,
        confidence: f32,
        start_time: f32,
        end_time: Option<f32>,
    },
    
    /// Processing progress update
    ProcessingProgress {
        session_id: String,
        processed_seconds: f32,
        total_seconds: f32,
        speakers_found: usize,
    },
    
    /// Diarization error
    Error {
        session_id: String,
        error: String,
        code: String,
        recoverable: bool,
    },
    
    /// Diarization complete
    Complete {
        session_id: String,
        total_speakers: usize,
        processing_time_ms: u64,
    },
}

/// Audio buffer state for shared access
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BufferState {
    /// Current write position
    pub write_position: usize,
    
    /// Read positions for each consumer
    pub read_positions: HashMap<String, usize>,
    
    /// Buffer capacity
    pub capacity: usize,
    
    /// Utilization percentage
    pub utilization: f32,
    
    /// Sample rate
    pub sample_rate: u32,
}

/// Final merged segment with transcription and speaker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinalSegment {
    /// Start time in seconds
    pub start_time: f32,
    
    /// End time in seconds
    pub end_time: f32,
    
    /// Speaker identifier
    pub speaker_id: String,
    
    /// Transcribed text
    pub text: String,
    
    /// Transcription confidence
    pub transcription_confidence: f32,
    
    /// Speaker confidence
    pub speaker_confidence: f32,
    
    /// Combined confidence
    pub overall_confidence: f32,
    
    /// Whether this segment was modified during merging
    pub was_merged: bool,
}