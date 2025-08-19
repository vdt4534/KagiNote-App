//! Tauri commands for frontend integration
//! 
//! These commands provide the interface between the React frontend
//! and the Rust backend audio/ASR processing systems.

use serde::{Deserialize, Serialize};
use crate::audio::capture::{AudioCaptureService, AudioConfig};
use crate::audio::types::{AudioData, AudioDevice};
use crate::asr::whisper::{WhisperEngine, WhisperConfig};
use crate::asr::types::{ASRResult, TranscriptionContext};
use uuid::Uuid;
use tauri::Emitter;

#[derive(Debug, Serialize, Deserialize)]
pub struct StartCaptureRequest {
    pub sample_rate: u32,
    pub channels: u8,
    pub buffer_size_ms: u32,
    pub device_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TranscribeRequest {
    pub audio_data: Vec<f32>,
    pub sample_rate: u32,
    pub language: Option<String>,
    pub context: Option<TranscriptionContext>,
}

// High-level integration types
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TranscriptionConfig {
    #[serde(rename = "qualityTier")]
    pub quality_tier: String,
    pub languages: Vec<String>,
    #[serde(rename = "enableSpeakerDiarization")]
    pub enable_speaker_diarization: bool,
    #[serde(rename = "enableTwoPassRefinement")]
    pub enable_two_pass_refinement: bool,
    #[serde(rename = "audioSources")]
    pub audio_sources: AudioSourceConfig,
    #[serde(rename = "vadThreshold")]
    pub vad_threshold: f32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AudioSourceConfig {
    pub microphone: bool,
    #[serde(rename = "systemAudio")]
    pub system_audio: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SystemCapabilities {
    #[serde(rename = "recommendedTier")]
    pub recommended_tier: String,
    #[serde(rename = "availableMemoryGB")]
    pub available_memory_gb: f32,
    #[serde(rename = "hasGPU")]
    pub has_gpu: bool,
    #[serde(rename = "cpuCores")]
    pub cpu_cores: u32,
    pub warnings: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TranscriptionSession {
    #[serde(rename = "sessionId")]
    pub session_id: String,
    pub config: TranscriptionConfig,
    #[serde(rename = "startTime")]
    pub start_time: u64,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FinalTranscriptionResult {
    #[serde(rename = "sessionId")]
    pub session_id: String,
    #[serde(rename = "totalDuration")]
    pub total_duration: f32,
    pub segments: Vec<serde_json::Value>,
    pub speakers: Option<Vec<serde_json::Value>>,
    #[serde(rename = "qualityMetrics")]
    pub quality_metrics: serde_json::Value,
    #[serde(rename = "processingTimeMs")]
    pub processing_time_ms: u64,
}

// Legacy greeting command for compatibility
#[tauri::command]
pub fn greet(name: &str) -> String {
    format!("Hello, {}! KagiNote is ready for transcription.", name)
}

#[tauri::command]
pub async fn get_audio_devices() -> Result<Vec<AudioDevice>, String> {
    AudioCaptureService::list_audio_devices()
        .await
        .map_err(|e| format!("Failed to list audio devices: {}", e))
}

#[tauri::command] 
pub async fn start_audio_capture(request: StartCaptureRequest) -> Result<String, String> {
    let config = AudioConfig {
        sample_rate: request.sample_rate,
        channels: request.channels,
        buffer_size_ms: request.buffer_size_ms,
        device_id: request.device_id,
    };
    
    // TODO: Store capture service in app state
    let mut capture_service = AudioCaptureService::new(config)
        .await
        .map_err(|e| format!("Failed to create audio capture service: {}", e))?;
        
    capture_service.start_capture()
        .await
        .map_err(|e| format!("Failed to start audio capture: {}", e))?;
        
    Ok("Audio capture started successfully".to_string())
}

#[tauri::command]
pub async fn stop_audio_capture() -> Result<String, String> {
    // TODO: Retrieve capture service from app state and stop it
    Ok("Audio capture stopped successfully".to_string())
}

#[tauri::command]
pub async fn transcribe_audio(request: TranscribeRequest) -> Result<ASRResult, String> {
    let duration_seconds = request.audio_data.len() as f32 / request.sample_rate as f32;
    let audio_data = AudioData {
        samples: request.audio_data,
        sample_rate: request.sample_rate,
        channels: 1, // Assume mono for now
        timestamp: std::time::SystemTime::now(),
        source_channel: crate::audio::types::AudioSource::Microphone,
        duration_seconds,
    };
    
    // TODO: Use configured Whisper engine from app state
    let config = WhisperConfig::default();
    let engine = WhisperEngine::new(config)
        .await
        .map_err(|e| format!("Failed to initialize Whisper engine: {}", e))?;
        
    let context = request.context.unwrap_or_default();
    engine.transcribe(&audio_data, &context)
        .await
        .map_err(|e| format!("Failed to transcribe audio: {}", e))
}

// High-level integration commands

#[tauri::command]
pub async fn get_system_info() -> Result<SystemCapabilities, String> {
    // Get system information
    let mut sys = sysinfo::System::new_all();
    sys.refresh_all();
    
    let cpu_count = sys.cpus().len() as u32;
    let total_memory = sys.total_memory() as f32 / (1024.0 * 1024.0 * 1024.0); // Convert to GB
    
    // Check for GPU availability (simplified)
    let has_gpu = cfg!(feature = "gpu") || std::env::var("CUDA_VISIBLE_DEVICES").is_ok();
    
    // Determine recommended tier based on system capabilities
    let recommended_tier = if total_memory >= 16.0 && cpu_count >= 8 {
        "high-accuracy"
    } else if total_memory >= 8.0 && cpu_count >= 4 {
        "standard"
    } else {
        "turbo"
    };
    
    let mut warnings = Vec::new();
    if total_memory < 4.0 {
        warnings.push("low_memory".to_string());
    }
    if cpu_count < 4 {
        warnings.push("limited_cpu_cores".to_string());
    }
    
    Ok(SystemCapabilities {
        recommended_tier: recommended_tier.to_string(),
        available_memory_gb: total_memory,
        has_gpu,
        cpu_cores: cpu_count,
        warnings: if warnings.is_empty() { None } else { Some(warnings) },
    })
}

#[tauri::command]
pub async fn start_transcription(
    config: TranscriptionConfig,
    app_handle: tauri::AppHandle,
) -> Result<String, String> {
    let session_id = Uuid::new_v4().to_string();
    
    // Convert frontend config to backend config
    let audio_config = AudioConfig {
        sample_rate: match config.quality_tier.as_str() {
            "high-accuracy" => 48000,
            "standard" => 16000,
            "turbo" => 16000,
            _ => 16000,
        },
        channels: 1,
        buffer_size_ms: 100,
        device_id: None,
    };
    
    // Start audio capture
    let mut capture_service = AudioCaptureService::new(audio_config)
        .await
        .map_err(|e| format!("Failed to initialize audio capture: {}", e))?;
        
    capture_service.start_capture()
        .await
        .map_err(|e| format!("Failed to start audio capture: {}", e))?;
    
    // Initialize ASR engine
    let whisper_config = WhisperConfig {
        model_tier: match config.quality_tier.as_str() {
            "high-accuracy" => crate::asr::types::ModelTier::HighAccuracy,
            "standard" => crate::asr::types::ModelTier::Standard,
            "turbo" => crate::asr::types::ModelTier::Turbo,
            _ => crate::asr::types::ModelTier::Standard,
        },
        model_path: None,
        device: crate::asr::types::Device::Auto,
        num_threads: 4,
        beam_size: match config.quality_tier.as_str() {
            "high-accuracy" => 5,
            "standard" => 3,
            "turbo" => 1,
            _ => 3,
        },
        temperature: 0.0,
        language: config.languages.first().cloned(),
        task: crate::asr::types::Task::Transcribe,
        enable_vad: true,
        enable_word_timestamps: config.enable_two_pass_refinement,
        context_size: 50,
        custom_vocabulary: None,
        optimization_level: Some(crate::asr::types::OptimizationLevel::Balanced),
    };
    
    let _engine = WhisperEngine::new(whisper_config)
        .await
        .map_err(|e| format!("Failed to initialize ASR engine: {}", e))?;
    
    // TODO: Store the session state in global app state
    // For now, just return the session ID
    
    // Simulate starting transcription process
    let session_id_clone = session_id.clone();
    tokio::spawn(async move {
        // This would be the actual transcription loop
        // For integration testing, we'll emit some test events
        let _ = app_handle.emit("transcription-update", serde_json::json!({
            "sessionId": session_id_clone,
            "segment": {
                "text": "Transcription started successfully",
                "startTime": 0.0,
                "endTime": 2.0,
                "confidence": 0.95
            },
            "updateType": "new",
            "processingPass": 1
        }));
    });
    
    Ok(session_id)
}

#[tauri::command]
pub async fn stop_transcription(session_id: String) -> Result<FinalTranscriptionResult, String> {
    // TODO: Retrieve session from global state and stop it
    // For now, return a mock result
    
    let result = FinalTranscriptionResult {
        session_id: session_id.clone(),
        total_duration: 60.0,
        segments: vec![
            serde_json::json!({
                "text": "This is a test transcription segment",
                "startTime": 0.0,
                "endTime": 5.0,
                "confidence": 0.95,
                "speaker": "speaker_1"
            })
        ],
        speakers: Some(vec![
            serde_json::json!({
                "id": "speaker_1",
                "name": "Speaker 1",
                "segments": 1
            })
        ]),
        quality_metrics: serde_json::json!({
            "averageConfidence": 0.95,
            "wordErrorRate": 0.05,
            "realTimeFactor": 0.8
        }),
        processing_time_ms: 1500,
    };
    
    Ok(result)
}