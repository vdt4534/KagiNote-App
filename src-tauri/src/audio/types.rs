//! Audio-related type definitions
//! 
//! Common types used throughout the audio processing pipeline.

use serde::{Deserialize, Serialize};
use std::time::SystemTime;
use thiserror::Error;

/// Audio data structure containing samples and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioData {
    pub samples: Vec<f32>,
    pub sample_rate: u32,
    pub channels: u8,
    pub timestamp: SystemTime,
    pub source_channel: AudioSource,
    pub duration_seconds: f32,
}

/// Audio source types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AudioSource {
    Microphone,
    System,
    File,
}

/// Audio device information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioDevice {
    pub id: String,
    pub name: String,
    pub is_input_device: bool,
    pub is_default: bool,
    pub sample_rates: Vec<u32>,
    pub channels: u8,
}

/// Audio processing errors
#[derive(Debug, Error)]
pub enum AudioError {
    #[error("Invalid sample rate: {0}")]
    InvalidSampleRate(u32),
    
    #[error("Audio permission denied")]
    PermissionDenied { device: String },
    
    #[error("Audio device disconnected: {device}")]
    DeviceDisconnected { device: String },
    
    #[error("No audio capture method available. Tried: {attempted_methods:?}")]
    NoAudioMethodAvailable { attempted_methods: Vec<String> },
    
    #[error("No fallback device available")]
    NoFallbackDevice,
    
    #[error("Audio system initialization failed: {source}")]
    InitializationFailed { source: Box<dyn std::error::Error + Send + Sync> },
    
    #[error("Audio buffer overflow")]
    BufferOverflow,
    
    #[error("Audio processing failed: {message}")]
    ProcessingFailed { message: String },
}

/// VAD-specific errors
#[derive(Debug, Error)]
pub enum VADError {
    #[error("Invalid threshold: {0} (must be between 0.0 and 1.0)")]
    InvalidThreshold(f32),
    
    #[error("VAD model not found at path: {path}")]
    ModelNotFound { path: String },
    
    #[error("Empty audio provided")]
    EmptyAudio,
    
    #[error("Unsupported sample rate: {0} (expected 16000)")]
    UnsupportedSampleRate(u32),
    
    #[error("Clipped audio detected: {clipped_samples} samples > threshold")]
    ClippedAudio { clipped_samples: usize },
    
    #[error("VAD processing failed: {message}")]
    ProcessingFailed { message: String },
}

/// VAD result structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VADResult {
    pub has_speech: bool,
    pub confidence: f32,
    pub speech_segments: Vec<SpeechSegment>,
    pub adapted_threshold: Option<f32>,
    pub estimated_snr: Option<f32>,
    pub has_clipping_warning: bool,
}

/// Speech segment within audio
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeechSegment {
    pub start_time: f32,
    pub end_time: f32,
    pub confidence: f32,
}

/// VAD configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VADConfig {
    pub threshold: f32,
    pub min_speech_duration_ms: u32,
    pub max_speech_duration_ms: u32,
    pub padding_before_ms: u32,
    pub padding_after_ms: u32,
    pub adaptive_threshold: bool,
    pub context_frames: usize,
    pub model_path: Option<std::path::PathBuf>,
}

/// Model information for VAD
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VADModelInfo {
    pub version: String,
    pub sample_rate: u32,
    pub supports_streaming: bool,
}

impl Default for VADConfig {
    fn default() -> Self {
        Self {
            threshold: 0.5,
            min_speech_duration_ms: 500,
            max_speech_duration_ms: 30000,
            padding_before_ms: 200,
            padding_after_ms: 200,
            adaptive_threshold: false,
            context_frames: 16,
            model_path: None,
        }
    }
}