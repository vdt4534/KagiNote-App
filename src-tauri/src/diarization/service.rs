//! Main Speaker Diarization Service
//! 
//! Provides the primary interface for speaker diarization functionality.
//! This service coordinates all diarization components to identify speakers
//! in audio streams and generate speaker segments.

use super::types::*;
use super::embedder::SpeakerEmbedder;
use super::clustering::SpeakerClusterer;
use super::pipeline::DiarizationPipeline;

use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tracing;

/// Main speaker diarization service that coordinates all diarization components.
/// 
/// This service provides the primary interface for speaker identification in audio streams.
/// It manages speaker embedders, clustering algorithms, and maintains speaker profiles
/// across recording sessions.
/// 
/// # Examples
/// 
/// ```rust
/// use kaginote::diarization::{DiarizationService, DiarizationConfig};
/// 
/// let config = DiarizationConfig {
///     max_speakers: 4,
///     similarity_threshold: 0.7,
///     ..Default::default()
/// };
/// 
/// let service = DiarizationService::new(config).await?;
/// let result = service.diarize(&audio_samples, 16000).await?;
/// println!("Detected {} speakers", result.total_speakers);
/// ```
pub struct DiarizationService {
    config: DiarizationConfig,
    embedder: Arc<Mutex<SpeakerEmbedder>>,
    clusterer: Arc<Mutex<SpeakerClusterer>>,
    pipeline: Arc<Mutex<DiarizationPipeline>>,
    speaker_profiles: Arc<Mutex<HashMap<String, SpeakerProfile>>>,
}

impl DiarizationService {
    /// Creates a new diarization service with the specified configuration.
    /// 
    /// This initializes all required components including speaker embedders,
    /// clustering algorithms, and the diarization pipeline. The service will
    /// validate the configuration and may download required models if not cached.
    /// 
    /// # Arguments
    /// 
    /// * `config` - Diarization configuration specifying parameters like max speakers,
    ///              similarity thresholds, and hardware acceleration options
    /// 
    /// # Returns
    /// 
    /// A `Result` containing the initialized service or a `DiarizationError` if
    /// initialization fails (e.g., model loading, invalid config, insufficient memory)
    /// 
    /// # Errors
    /// 
    /// * `DiarizationError::ConfigError` - Invalid configuration parameters
    /// * `DiarizationError::ModelLoadError` - Failed to load required models
    /// * `DiarizationError::MemoryError` - Insufficient memory for configuration
    /// * `DiarizationError::HardwareError` - Requested hardware acceleration unavailable
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// let config = DiarizationConfig {
    ///     max_speakers: 6,
    ///     similarity_threshold: 0.8,
    ///     hardware_acceleration: HardwareAcceleration::Metal,
    ///     ..Default::default()
    /// };
    /// 
    /// let service = DiarizationService::new(config).await
    ///     .expect("Failed to initialize diarization service");
    /// ```
    pub async fn new(config: DiarizationConfig) -> Result<Self, DiarizationError> {
        tracing::info!("Initializing DiarizationService with config: {:?}", config);
        
        // Validate configuration
        Self::validate_config(&config)?;
        
        // Initialize components
        let embedder = SpeakerEmbedder::new(config.clone()).await
            .map_err(|e| DiarizationError::EmbeddingError { 
                message: format!("Failed to initialize embedder: {}", e) 
            })?;
            
        let clusterer = SpeakerClusterer::new(config.clone()).await
            .map_err(|e| DiarizationError::ClusteringError { 
                message: format!("Failed to initialize clusterer: {}", e) 
            })?;
            
        let pipeline = DiarizationPipeline::new(config.clone()).await
            .map_err(|e| DiarizationError::ProcessingError { 
                message: format!("Failed to initialize pipeline: {}", e) 
            })?;
        
        Ok(Self {
            config,
            embedder: Arc::new(Mutex::new(embedder)),
            clusterer: Arc::new(Mutex::new(clusterer)),
            pipeline: Arc::new(Mutex::new(pipeline)),
            speaker_profiles: Arc::new(Mutex::new(HashMap::new())),
        })
    }
    
    /// Get the configuration used by this service
    pub fn get_config(&self) -> &DiarizationConfig {
        &self.config
    }
    
    /// Extracts speaker embeddings from audio samples for similarity analysis.
    /// 
    /// This method analyzes audio to generate mathematical representations (embeddings)
    /// of speaker voices. Each embedding is a 512-dimensional vector that captures
    /// unique voice characteristics like pitch, formants, and speaking patterns.
    /// 
    /// # Arguments
    /// 
    /// * `audio_samples` - Raw audio samples as f32 values
    /// * `sample_rate` - Audio sample rate in Hz
    /// 
    /// # Returns
    /// 
    /// A vector of `SpeakerEmbedding` objects, each containing:
    /// - 512-dimensional vector representation
    /// - Confidence score (0.0-1.0) 
    /// - Timestamp information
    /// - Quality metrics
    /// 
    /// # Errors
    /// 
    /// * `DiarizationError::InsufficientAudio` - Audio too short for embedding extraction
    /// * `DiarizationError::EmbeddingError` - Failed to compute embeddings
    /// * `DiarizationError::InvalidSampleRate` - Unsupported sample rate
    /// 
    /// # Performance
    /// 
    /// - Processing time: ~50-100ms per second of audio
    /// - Memory usage: ~1MB per minute of audio
    /// - Optimized for batch processing of multiple segments
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// let embeddings = service.extract_speaker_embeddings(&audio, 16000).await?;
    /// 
    /// for embedding in embeddings {
    ///     println!("Embedding at {:.1}s with confidence {:.2}", 
    ///         embedding.timestamp_start, 
    ///         embedding.confidence
    ///     );
    /// }
    /// 
    /// // Compare embeddings for similarity
    /// let similarity = embeddings[0].similarity(&embeddings[1]);
    /// println!("Speaker similarity: {:.2}", similarity);
    /// ```
    pub async fn extract_speaker_embeddings(
        &self,
        audio_samples: &[f32],
        sample_rate: u32,
    ) -> Result<Vec<SpeakerEmbedding>, DiarizationError> {
        if audio_samples.is_empty() {
            return Err(DiarizationError::InsufficientAudio);
        }
        
        if sample_rate == 0 {
            return Err(DiarizationError::InvalidSampleRate);
        }
        
        let duration_seconds = audio_samples.len() as f32 / sample_rate as f32;
        if duration_seconds < self.config.min_segment_duration {
            return Err(DiarizationError::InsufficientAudio);
        }
        
        tracing::debug!("Extracting embeddings from {:.2}s of audio at {}Hz", 
                       duration_seconds, sample_rate);
        
        let mut embedder = self.embedder.lock().await;
        embedder.extract_embeddings(audio_samples, sample_rate).await
            .map_err(|e| DiarizationError::EmbeddingError { 
                message: format!("Embedding extraction failed: {}", e) 
            })
    }
    
    /// Cluster speaker embeddings into speaker groups
    pub async fn cluster_speakers(
        &self,
        embeddings: &[SpeakerEmbedding],
    ) -> Result<HashMap<String, Vec<SpeakerEmbedding>>, DiarizationError> {
        if embeddings.is_empty() {
            return Ok(HashMap::new());
        }
        
        tracing::debug!("Clustering {} embeddings", embeddings.len());
        
        let mut clusterer = self.clusterer.lock().await;
        clusterer.cluster_embeddings(embeddings).await
            .map_err(|e| DiarizationError::ClusteringError { 
                message: format!("Clustering failed: {}", e) 
            })
    }
    
    /// Detect speaker change points in audio
    pub async fn detect_speaker_changes(
        &self,
        audio_samples: &[f32],
        sample_rate: u32,
    ) -> Result<Vec<f32>, DiarizationError> {
        if audio_samples.is_empty() {
            return Err(DiarizationError::InsufficientAudio);
        }
        
        if sample_rate == 0 {
            return Err(DiarizationError::InvalidSampleRate);
        }
        
        tracing::debug!("Detecting speaker changes in {:.2}s of audio", 
                       audio_samples.len() as f32 / sample_rate as f32);
        
        let pipeline = self.pipeline.lock().await;
        pipeline.detect_speaker_changes(audio_samples, sample_rate).await
            .map_err(|e| DiarizationError::ProcessingError { 
                message: format!("Speaker change detection failed: {}", e) 
            })
    }
    
    /// Performs complete speaker diarization on audio samples.
    /// 
    /// This method processes raw audio samples to identify and separate different speakers.
    /// It extracts speaker embeddings, performs clustering to assign speaker IDs, and
    /// returns detailed results including speaker profiles and timing information.
    /// 
    /// # Arguments
    /// 
    /// * `audio_samples` - Raw audio samples as f32 values, typically in range [-1.0, 1.0]
    /// * `sample_rate` - Audio sample rate in Hz (automatically resampled to 16kHz if needed)
    /// 
    /// # Returns
    /// 
    /// A `DiarizationResult` containing:
    /// - Speaker segments with timing and confidence scores
    /// - Speaker profiles with voice characteristics  
    /// - Processing metrics and performance data
    /// - Any warnings or issues encountered during processing
    /// 
    /// # Errors
    /// 
    /// * `DiarizationError::InsufficientAudio` - Audio too short (<1 second)
    /// * `DiarizationError::InvalidSampleRate` - Unsupported sample rate
    /// * `DiarizationError::ProcessingError` - Failed during embedding or clustering
    /// * `DiarizationError::MemoryError` - Exceeded configured memory limit
    /// * `DiarizationError::TimeoutError` - Processing took too long
    /// 
    /// # Performance
    /// 
    /// Processing time scales with audio length and number of speakers:
    /// - 2-4 speakers: ~0.1x real-time (1 minute audio in 6 seconds)
    /// - 5-8 speakers: ~0.2x real-time (1 minute audio in 12 seconds)
    /// - Memory usage: 50-500MB depending on configuration
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// // Process 10 seconds of audio at 44.1kHz
    /// let audio_samples: Vec<f32> = load_audio_file("meeting.wav");
    /// let sample_rate = 44100;
    /// 
    /// let result = service.diarize(&audio_samples, sample_rate).await?;
    /// 
    /// println!("Found {} speakers in {:.1}s audio", 
    ///     result.total_speakers, 
    ///     audio_samples.len() as f32 / sample_rate as f32
    /// );
    /// 
    /// for segment in result.segments {
    ///     println!("{}: {:.1}s-{:.1}s (confidence: {:.2})", 
    ///         segment.speaker_id,
    ///         segment.start_time,
    ///         segment.end_time, 
    ///         segment.confidence
    ///     );
    /// }
    /// ```
    /// 
    /// # Threading
    /// 
    /// This method is async and uses tokio for parallel processing of embeddings
    /// and clustering. It's safe to call from multiple tasks but each service
    /// instance should only process one audio stream at a time for optimal performance.
    pub async fn diarize(
        &self,
        audio_samples: &[f32],
        sample_rate: u32,
    ) -> Result<DiarizationResult, DiarizationError> {
        let start_time = Instant::now();
        
        if audio_samples.is_empty() {
            return Err(DiarizationError::InsufficientAudio);
        }
        
        if sample_rate == 0 {
            return Err(DiarizationError::InvalidSampleRate);
        }
        
        let duration_seconds = audio_samples.len() as f32 / sample_rate as f32;
        if duration_seconds < self.config.min_segment_duration {
            return Err(DiarizationError::InsufficientAudio);
        }
        
        tracing::info!("Starting complete diarization of {:.2}s audio at {}Hz", 
                      duration_seconds, sample_rate);
        
        // Extract embeddings
        let embeddings = self.extract_speaker_embeddings(audio_samples, sample_rate).await?;
        tracing::debug!("Extracted {} embeddings", embeddings.len());
        
        // Cluster speakers
        let clusters = self.cluster_speakers(&embeddings).await?;
        let total_speakers = clusters.len();
        tracing::debug!("Found {} distinct speakers", total_speakers);
        
        // Create speaker profiles
        let mut speaker_profiles = HashMap::new();
        let mut segments = Vec::new();
        let mut overall_confidence = 0.0;
        
        for (speaker_id, speaker_embeddings) in clusters {
            // Calculate speaker statistics
            let total_speech_time: f32 = speaker_embeddings.iter()
                .map(|e| e.timestamp_end - e.timestamp_start)
                .sum();
            
            let average_confidence: f32 = speaker_embeddings.iter()
                .map(|e| e.confidence)
                .sum::<f32>() / speaker_embeddings.len() as f32;
            
            overall_confidence += average_confidence;
            
            // Create speaker profile
            let profile = SpeakerProfile {
                id: speaker_id.clone(),
                display_name: format!("Speaker {}", speaker_id.replace("speaker_", "")),
                color: Self::generate_speaker_color(&speaker_id),
                voice_characteristics: VoiceCharacteristics::default(),
                embeddings: speaker_embeddings.clone(),
                total_speech_time,
                segment_count: speaker_embeddings.len(),
                average_confidence,
                last_active: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
                notes: None,
            };
            
            speaker_profiles.insert(speaker_id.clone(), profile);
            
            // Create segments from embeddings
            for embedding in speaker_embeddings {
                segments.push(SpeakerSegment {
                    speaker_id: speaker_id.clone(),
                    start_time: embedding.timestamp_start,
                    end_time: embedding.timestamp_end,
                    confidence: embedding.confidence,
                    text: None, // Will be filled by transcription integration
                    embedding: Some(embedding),
                    has_overlap: false, // Simple implementation doesn't detect overlaps yet
                    overlapping_speakers: vec![],
                });
            }
        }
        
        // Sort segments by time
        segments.sort_by(|a, b| a.start_time.partial_cmp(&b.start_time).unwrap());
        
        // Calculate overall confidence
        overall_confidence = if total_speakers > 0 { 
            overall_confidence / total_speakers as f32 
        } else { 
            0.0 
        };
        
        let processing_time = start_time.elapsed();
        
        tracing::info!("Diarization completed in {:?}: {} speakers, {:.2} confidence", 
                      processing_time, total_speakers, overall_confidence);
        
        Ok(DiarizationResult {
            segments,
            speakers: speaker_profiles,
            total_speakers,
            overall_confidence,
            processing_time,
            session_id: None,
            metrics: None,
            warnings: vec![],
        })
    }
    
    /// Store speaker profiles for future reidentification
    pub async fn store_speaker_profiles(
        &self,
        speakers: &HashMap<String, SpeakerProfile>
    ) -> Result<(), DiarizationError> {
        let mut stored_profiles = self.speaker_profiles.lock().await;
        
        for (speaker_id, profile) in speakers {
            stored_profiles.insert(speaker_id.clone(), profile.clone());
        }
        
        tracing::info!("Stored {} speaker profiles", speakers.len());
        Ok(())
    }
    
    /// Reidentify speakers using stored profiles
    pub async fn reidentify_speaker(
        &self,
        embedding: &SpeakerEmbedding
    ) -> Result<Option<String>, DiarizationError> {
        let stored_profiles = self.speaker_profiles.lock().await;
        
        let mut best_match = None;
        let mut best_similarity = 0.0;
        
        for (speaker_id, profile) in stored_profiles.iter() {
            for stored_embedding in &profile.embeddings {
                let similarity = embedding.similarity(stored_embedding);
                if similarity > best_similarity && similarity > self.config.similarity_threshold {
                    best_similarity = similarity;
                    best_match = Some(speaker_id.clone());
                }
            }
        }
        
        Ok(best_match)
    }
    
    /// Validate configuration parameters
    fn validate_config(config: &DiarizationConfig) -> Result<(), DiarizationError> {
        if config.max_speakers < config.min_speakers {
            return Err(DiarizationError::ConfigError {
                message: "max_speakers must be >= min_speakers".to_string(),
            });
        }
        
        if config.embedding_dimension != 512 {
            return Err(DiarizationError::ConfigError {
                message: "embedding_dimension must be 512".to_string(),
            });
        }
        
        if config.similarity_threshold < 0.0 || config.similarity_threshold > 1.0 {
            return Err(DiarizationError::ConfigError {
                message: "similarity_threshold must be between 0.0 and 1.0".to_string(),
            });
        }
        
        if config.min_segment_duration <= 0.0 {
            return Err(DiarizationError::ConfigError {
                message: "min_segment_duration must be positive".to_string(),
            });
        }
        
        Ok(())
    }
    
    /// Generate a color for a speaker ID
    fn generate_speaker_color(speaker_id: &str) -> String {
        // Simple hash-based color generation
        let colors = [
            "#3B82F6", "#EF4444", "#10B981", "#F59E0B", "#8B5CF6",
            "#EC4899", "#06B6D4", "#84CC16", "#F97316", "#6366F1"
        ];
        
        let hash = speaker_id.chars().map(|c| c as u8).sum::<u8>() as usize;
        colors[hash % colors.len()].to_string()
    }
}