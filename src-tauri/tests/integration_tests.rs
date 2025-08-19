//! Integration tests for KagiNote audio processing pipeline
//! 
//! Tests the complete audio capture -> VAD -> ASR pipeline

use kaginote_lib::audio::capture::{AudioCaptureService, AudioConfig};
use kaginote_lib::audio::vad::{SileroVAD, VADProcessor};
use kaginote_lib::audio::types::{AudioData, AudioSource, VADConfig};
use kaginote_lib::asr::whisper::{WhisperEngine, WhisperConfig};
use kaginote_lib::asr::types::{TranscriptionContext, ModelTier};
use std::time::SystemTime;
use tokio;

#[tokio::test]
async fn test_complete_transcription_pipeline() -> anyhow::Result<()> {
    // ARRANGE
    let audio_config = AudioConfig::default();
    let vad_config = VADConfig::default();
    let whisper_config = WhisperConfig::default();
    
    // Create test audio data
    let test_audio = AudioData {
        samples: kaginote_lib::asr::whisper::generate_test_signal(440.0, 2.0, 16000),
        sample_rate: 16000,
        channels: 1,
        timestamp: SystemTime::now(),
        source_channel: AudioSource::Microphone,
        duration_seconds: 2.0,
    };
    
    // ACT
    // Initialize VAD
    let vad = SileroVAD::new(vad_config).await?;
    
    // Process audio through VAD
    let vad_result = vad.detect_speech(&test_audio).await?;
    
    // Initialize ASR engine
    let asr_engine = WhisperEngine::new(whisper_config).await?;
    
    // Transcribe if speech is detected
    if vad_result.has_speech {
        let context = TranscriptionContext::default();
        let transcription = asr_engine.transcribe(&test_audio, &context).await?;
        
        // ASSERT
        assert!(!transcription.text.is_empty());
        assert!(transcription.confidence > 0.0);
        println!("Transcription: {}", transcription.text);
    }
    
    Ok(())
}

#[tokio::test]
async fn test_audio_capture_initialization() -> anyhow::Result<()> {
    // ARRANGE
    let config = AudioConfig::default();
    
    // ACT
    let capture_result = AudioCaptureService::new(config).await;
    
    // ASSERT
    match capture_result {
        Ok(capture) => {
            assert!(capture.is_ready());
            assert_eq!(capture.get_sample_rate(), 16000);
        }
        Err(e) => {
            // This is acceptable if no audio devices are available in CI
            println!("Audio capture initialization failed (expected in CI): {}", e);
        }
    }
    
    Ok(())
}

#[tokio::test]
async fn test_vad_initialization() -> anyhow::Result<()> {
    // ARRANGE
    let config = VADConfig::default();
    
    // ACT
    let vad_result = SileroVAD::new(config).await;
    
    // ASSERT
    assert!(vad_result.is_ok());
    let vad = vad_result?;
    assert!(vad.is_initialized());
    assert_eq!(vad.get_threshold(), 0.5);
    
    Ok(())
}

#[tokio::test]
async fn test_whisper_engine_initialization() -> anyhow::Result<()> {
    // ARRANGE
    let config = WhisperConfig::default();
    
    // ACT
    let engine_result = WhisperEngine::new(config).await;
    
    // ASSERT
    assert!(engine_result.is_ok());
    let engine = engine_result?;
    assert!(engine.is_loaded());
    assert_eq!(engine.get_model_tier(), ModelTier::Standard);
    assert_eq!(engine.get_supported_languages().len(), 99);
    
    Ok(())
}