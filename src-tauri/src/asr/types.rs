//! ASR-related type definitions
//! 
//! Common types used throughout the speech recognition pipeline.

// Removed unused import
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// ASR result containing transcription and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ASRResult {
    pub text: String,
    pub confidence: f32,
    pub language: String,
    pub language_confidence: f32,
    pub words: Vec<WordResult>,
    pub estimated_snr: Option<f32>,
    pub speaker_consistency_score: Option<f32>,
    pub language_segments: Option<Vec<LanguageSegment>>,
}

/// Individual word result with timing and confidence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WordResult {
    pub word: String,
    pub start_time: f32,
    pub end_time: f32,
    pub confidence: f32,
}

/// Language segment for mixed-language content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageSegment {
    pub language: String,
    pub start_time: f32,
    pub end_time: f32,
    pub confidence: f32,
}

/// Language detection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageDetectionResult {
    pub detected_language: String,
    pub confidence: f32,
    pub alternatives: Vec<LanguageAlternative>,
}

/// Alternative language possibility
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageAlternative {
    pub language: String,
    pub confidence: f32,
}

/// Transcription context for improved accuracy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionContext {
    pub previous_segments: Vec<String>,
    pub speaker_context: Option<String>,
    pub domain_context: Option<String>,
    pub speaker_embedding: Option<Vec<f32>>,
    pub speaking_rate: Option<f32>,
    pub accent_profile: Option<String>,
    pub overlap_buffer: Option<String>,
    pub overlap_threshold: f32,
}

impl Default for TranscriptionContext {
    fn default() -> Self {
        Self {
            previous_segments: Vec::new(),
            speaker_context: None,
            domain_context: None,
            speaker_embedding: None,
            speaking_rate: None,
            accent_profile: None,
            overlap_buffer: None,
            overlap_threshold: 0.8,
        }
    }
}

/// Model information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub version: String,
    pub checksum: String,
    pub is_verified: bool,
    pub memory_requirements_gb: f32,
}

/// Device capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceCapabilities {
    pub memory_gb: f32,
    pub compute_capability: Option<String>,
    pub supports_fp16: bool,
    pub max_batch_size: usize,
}

/// Performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub real_time_factor: f32,
    pub processing_time_ms: u64,
    pub memory_usage_mb: usize,
    pub cpu_usage_percent: f32,
}

/// ASR processing errors
#[derive(Debug, Error)]
pub enum ASRError {
    #[error("Insufficient memory: required {required}GB, available {available}GB")]
    InsufficientMemory { required: f32, available: f32 },
    
    #[error("Model not found at path: {path}")]
    ModelNotFound { path: String },
    
    #[error("Model loading failed: {message}")]
    ModelLoadFailed { message: String },
    
    #[error("Transcription failed: {message}")]
    TranscriptionFailed { message: String },
    
    #[error("Language detection failed: {message}")]
    LanguageDetectionFailed { message: String },
    
    #[error("Invalid audio format: {message}")]
    InvalidAudioFormat { message: String },
    
    #[error("Processing timeout after {seconds}s")]
    ProcessingTimeout { seconds: u64 },
    
    #[error("Device not available: {device}")]
    DeviceNotAvailable { device: String },
    
    #[error("Out of memory during processing")]
    OutOfMemory,
    
    #[error("Model verification failed: expected {expected}, got {actual}")]
    ModelVerificationFailed { expected: String, actual: String },
    
    #[error("Unsupported language: {language}")]
    UnsupportedLanguage { language: String },
}

/// Model tier selection
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ModelTier {
    Standard,       // Whisper Medium - balanced performance
    HighAccuracy,   // Whisper Large-v3 - best accuracy
    Turbo,         // Whisper Large-v3-Turbo - fastest processing
}

/// Processing device selection
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Device {
    CPU,
    CUDA,
    Metal,
    Auto,
}

/// Task type for Whisper
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Task {
    Transcribe,
    Translate,
}

/// Optimization level
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum OptimizationLevel {
    Speed,
    Balanced,
    Accuracy,
}