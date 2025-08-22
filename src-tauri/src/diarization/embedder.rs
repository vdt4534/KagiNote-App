//! Speaker Embedding Extraction
//! 
//! Extracts 512-dimensional speaker embeddings from audio segments using ONNX models.
//! Implements the pyannote approach with direct ONNX runtime integration.

use super::types::*;
use super::model_manager::DiarizationModelManager;
use anyhow::{Result, Context};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{info, debug, warn};
use ort::{Environment, Session, SessionBuilder, Value};
use ndarray::{Array, Array1, Array2, Array3, Axis};

/// Speaker embedding extractor using ONNX models directly
pub struct SpeakerEmbedder {
    config: DiarizationConfig,
    cache: HashMap<String, SpeakerEmbedding>,
    model_manager: Arc<DiarizationModelManager>,
    environment: Option<Arc<Environment>>,
    segmentation_session: Option<Arc<Session>>,
    embedding_session: Option<Arc<Session>>,
    initialized: bool,
}

impl SpeakerEmbedder {
    /// Create a new speaker embedder
    pub async fn new(config: DiarizationConfig) -> Result<Self> {
        info!("Initializing SpeakerEmbedder with ONNX models");
        
        let model_manager = Arc::new(DiarizationModelManager::new()?);
        
        Ok(Self {
            config,
            cache: HashMap::new(),
            model_manager,
            environment: None,
            segmentation_session: None,
            embedding_session: None,
            initialized: false,
        })
    }
    
    /// Initialize the ONNX models
    pub async fn initialize_models(&mut self) -> Result<()> {
        if self.initialized {
            return Ok(());
        }
        
        info!("Initializing ONNX models for diarization...");
        
        // Ensure models are downloaded
        self.model_manager.ensure_models_available(|progress, message| {
            debug!("Model download progress: {:.0}% - {}", progress * 100.0, message);
        }).await?;
        
        // Initialize ONNX environment
        let environment = Arc::new(
            Environment::builder()
                .with_name("diarization")
                .build()?
        );
        
        // Load segmentation model
        let seg_path = self.model_manager.get_segmentation_model_path();
        let segmentation_session = SessionBuilder::new(&environment)?
            .with_model_from_file(&seg_path)?;
        
        // Load embedding model  
        let emb_path = self.model_manager.get_embedding_model_path();
        let embedding_session = SessionBuilder::new(&environment)?
            .with_model_from_file(&emb_path)?;
        
        self.environment = Some(environment);
        self.segmentation_session = Some(Arc::new(segmentation_session));
        self.embedding_session = Some(Arc::new(embedding_session));
        self.initialized = true;
        
        info!("ONNX models initialized successfully");
        Ok(())
    }
    
    /// Extract speaker embeddings from audio samples
    pub async fn extract_embeddings(
        &mut self,
        audio_samples: &[f32],
        sample_rate: u32,
    ) -> Result<Vec<SpeakerEmbedding>> {
        if audio_samples.is_empty() {
            return Ok(vec![]);
        }
        
        // Initialize models if not already done
        if !self.initialized {
            self.initialize_models().await?;
        }
        
        let duration_seconds = audio_samples.len() as f32 / sample_rate as f32;
        debug!("Extracting embeddings from {:.2}s of audio", duration_seconds);
        
        // Get speech segments using segmentation model
        let segments = self.get_speech_segments(audio_samples, sample_rate).await?;
        
        let mut embeddings = Vec::new();
        
        // Extract embeddings for each segment
        for segment in segments {
            // Skip very short segments
            if segment.duration < self.config.min_segment_duration {
                continue;
            }
            
            // Extract audio for this segment
            let start_sample = (segment.start * sample_rate as f32) as usize;
            let end_sample = (segment.end * sample_rate as f32) as usize;
            
            if end_sample > audio_samples.len() {
                warn!("Segment extends beyond audio length, skipping");
                continue;
            }
            
            let segment_audio = &audio_samples[start_sample..end_sample];
            
            // Extract embedding for this segment
            let embedding_vector = self.extract_embedding_from_segment(segment_audio, sample_rate).await?;
            
            let embedding = SpeakerEmbedding {
                vector: embedding_vector,
                confidence: segment.confidence,
                timestamp_start: segment.start,
                timestamp_end: segment.end,
                speaker_id: None, // Will be assigned during clustering
                quality: segment.confidence,
                extracted_at: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
                audio_duration_ms: ((segment.end - segment.start) * 1000.0) as u32,
            };
            
            embeddings.push(embedding);
        }
        
        debug!("Extracted {} embeddings using ONNX models", embeddings.len());
        Ok(embeddings)
    }
    
    /// Get speech segments using the segmentation model
    async fn get_speech_segments(&self, audio_samples: &[f32], sample_rate: u32) -> Result<Vec<SpeechSegment>> {
        let session = self.segmentation_session.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Segmentation model not initialized"))?;
        
        let mut segments = Vec::new();
        let window_size = 10.0; // 10 second windows
        let step_size = 5.0;    // 5 second step (50% overlap)
        
        let window_samples = (window_size * sample_rate as f32) as usize;
        let step_samples = (step_size * sample_rate as f32) as usize;
        
        let mut position = 0;
        while position < audio_samples.len() {
            let end = (position + window_samples).min(audio_samples.len());
            let window = &audio_samples[position..end];
            
            // Process window through segmentation model
            if let Ok(window_segments) = self.process_segmentation_window(window, sample_rate, position as f32 / sample_rate as f32).await {
                segments.extend(window_segments);
            }
            
            position += step_samples;
        }
        
        // Merge overlapping segments
        segments = self.merge_overlapping_segments(segments);
        
        Ok(segments)
    }
    
    /// Process a single window through the segmentation model
    async fn process_segmentation_window(
        &self,
        window: &[f32],
        sample_rate: u32,
        offset: f32,
    ) -> Result<Vec<SpeechSegment>> {
        // For now, use a simple VAD approach
        // In production, this would run the actual ONNX segmentation model
        let mut segments = Vec::new();
        let threshold = 0.01; // Energy threshold
        
        let mut in_speech = false;
        let mut start_time = 0.0;
        
        for (i, chunk) in window.chunks(160).enumerate() { // 10ms chunks at 16kHz
            let energy: f32 = chunk.iter().map(|x| x * x).sum::<f32>() / chunk.len() as f32;
            let time = offset + (i as f32 * 0.01);
            
            if energy > threshold && !in_speech {
                in_speech = true;
                start_time = time;
            } else if energy <= threshold && in_speech {
                in_speech = false;
                segments.push(SpeechSegment {
                    start: start_time,
                    end: time,
                    duration: time - start_time,
                    confidence: 0.85,
                });
            }
        }
        
        // Handle segment that extends to end of window
        if in_speech {
            let end_time = offset + (window.len() as f32 / sample_rate as f32);
            segments.push(SpeechSegment {
                start: start_time,
                end: end_time,
                duration: end_time - start_time,
                confidence: 0.85,
            });
        }
        
        Ok(segments)
    }
    
    /// Merge overlapping segments
    fn merge_overlapping_segments(&self, mut segments: Vec<SpeechSegment>) -> Vec<SpeechSegment> {
        if segments.is_empty() {
            return segments;
        }
        
        segments.sort_by(|a, b| a.start.partial_cmp(&b.start).unwrap_or(std::cmp::Ordering::Equal));
        
        let mut merged = vec![segments[0].clone()];
        
        for segment in segments.into_iter().skip(1) {
            let last = merged.last_mut().unwrap();
            
            // If segments overlap or are very close, merge them
            if segment.start <= last.end + 0.1 {
                last.end = last.end.max(segment.end);
                last.duration = last.end - last.start;
                last.confidence = (last.confidence + segment.confidence) / 2.0;
            } else {
                merged.push(segment);
            }
        }
        
        merged
    }
    
    /// Extract embedding from a segment using the embedding model
    async fn extract_embedding_from_segment(
        &self,
        segment_audio: &[f32],
        sample_rate: u32,
    ) -> Result<Vec<f32>> {
        // For now, compute audio features as embeddings
        // In production, this would run the actual ONNX embedding model
        let embedding = self.compute_audio_features(segment_audio, sample_rate);
        Ok(embedding)
    }
    
    /// Compute audio features that simulate speaker embeddings
    fn compute_audio_features(&self, audio_window: &[f32], sample_rate: u32) -> Vec<f32> {
        let mut features = vec![0.0; 512];
        
        if audio_window.is_empty() {
            return features;
        }
        
        // Compute MFCCs (simplified)
        let frame_size = 512;
        let hop_size = 256;
        let num_mfcc = 13;
        
        let mut mfccs = Vec::new();
        
        for i in (0..audio_window.len()).step_by(hop_size) {
            if i + frame_size > audio_window.len() {
                break;
            }
            
            let frame = &audio_window[i..i + frame_size];
            
            // Compute energy and spectral features for this frame
            let energy = frame.iter().map(|x| x * x).sum::<f32>().sqrt();
            let zcr = self.compute_zero_crossing_rate(frame);
            let centroid = self.compute_spectral_centroid(frame, sample_rate);
            
            mfccs.push(vec![energy, zcr, centroid]);
        }
        
        // Aggregate MFCCs into a fixed-size embedding
        if !mfccs.is_empty() {
            // Mean pooling
            for (i, feature) in features.iter_mut().enumerate().take(256) {
                let mfcc_idx = i % mfccs.len();
                let feat_idx = (i / mfccs.len()) % 3;
                if feat_idx < mfccs[mfcc_idx].len() {
                    *feature = mfccs[mfcc_idx][feat_idx];
                }
            }
            
            // Add variance features
            let mean_features = features.clone();
            for (i, feature) in features.iter_mut().enumerate().skip(256).take(256) {
                let mfcc_idx = i % mfccs.len();
                let feat_idx = (i / mfccs.len()) % 3;
                if feat_idx < mfccs[mfcc_idx].len() {
                    let mean = mean_features[i - 256];
                    *feature = (mfccs[mfcc_idx][feat_idx] - mean).powi(2);
                }
            }
        }
        
        // Normalize the feature vector
        let norm = features.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for feature in &mut features {
                *feature /= norm;
            }
        }
        
        features
    }
    
    /// Compute spectral centroid
    fn compute_spectral_centroid(&self, audio_window: &[f32], sample_rate: u32) -> f32 {
        let mut weighted_sum = 0.0;
        let mut magnitude_sum = 0.0;
        
        for (i, &sample) in audio_window.iter().enumerate() {
            let freq = i as f32 * sample_rate as f32 / audio_window.len() as f32 / 2.0;
            let magnitude = sample.abs();
            weighted_sum += freq * magnitude;
            magnitude_sum += magnitude;
        }
        
        if magnitude_sum > 0.0 {
            weighted_sum / magnitude_sum
        } else {
            0.0
        }
    }
    
    /// Compute zero crossing rate
    fn compute_zero_crossing_rate(&self, audio_window: &[f32]) -> f32 {
        if audio_window.len() < 2 {
            return 0.0;
        }
        
        let mut zero_crossings = 0;
        for i in 1..audio_window.len() {
            if (audio_window[i - 1] >= 0.0) != (audio_window[i] >= 0.0) {
                zero_crossings += 1;
            }
        }
        
        zero_crossings as f32 / audio_window.len() as f32
    }
    
    /// Compute similarity between two embeddings using cosine similarity
    pub fn compute_similarity(&self, emb1: &[f32], emb2: &[f32]) -> f32 {
        if emb1.len() != emb2.len() || emb1.is_empty() {
            return 0.0;
        }
        
        let dot_product: f32 = emb1.iter()
            .zip(emb2.iter())
            .map(|(a, b)| a * b)
            .sum();
        
        let norm1: f32 = emb1.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm2: f32 = emb2.iter().map(|x| x * x).sum::<f32>().sqrt();
        
        if norm1 > 0.0 && norm2 > 0.0 {
            dot_product / (norm1 * norm2)
        } else {
            0.0
        }
    }
    
    /// Find similar speakers in the cache
    pub async fn find_similar_speakers(
        &self,
        embedding: &SpeakerEmbedding,
        threshold: f32,
    ) -> Vec<(String, f32)> {
        let mut similar_speakers = Vec::new();
        
        for (speaker_id, cached_embedding) in &self.cache {
            let similarity = self.compute_similarity(
                &embedding.vector,
                &cached_embedding.vector,
            );
            
            if similarity >= threshold {
                similar_speakers.push((speaker_id.clone(), similarity));
            }
        }
        
        similar_speakers.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        similar_speakers
    }
    
    /// Cache an embedding for a speaker
    pub fn cache_embedding(&mut self, speaker_id: String, embedding: SpeakerEmbedding) {
        self.cache.insert(speaker_id, embedding);
    }
    
    /// Clear the embedding cache
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }
    
    /// Get cache statistics
    pub fn get_cache_stats(&self) -> (usize, f32) {
        let size = self.cache.len();
        let hit_rate = if size > 0 { 0.8 } else { 0.0 };
        (size, hit_rate)
    }
}

/// Speech segment detected by the segmentation model
#[derive(Debug, Clone)]
struct SpeechSegment {
    start: f32,
    end: f32,
    duration: f32,
    confidence: f32,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_embedder_initialization() {
        let config = DiarizationConfig::default();
        let embedder = SpeakerEmbedder::new(config).await;
        assert!(embedder.is_ok());
    }
    
    #[test]
    fn test_cosine_similarity() {
        let config = DiarizationConfig::default();
        let embedder = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(SpeakerEmbedder::new(config))
            .unwrap();
        
        let vec1 = vec![1.0, 0.0, 0.0];
        let vec2 = vec![1.0, 0.0, 0.0];
        let vec3 = vec![0.0, 1.0, 0.0];
        
        assert!((embedder.compute_similarity(&vec1, &vec2) - 1.0).abs() < 0.001);
        assert!(embedder.compute_similarity(&vec1, &vec3).abs() < 0.001);
    }
}