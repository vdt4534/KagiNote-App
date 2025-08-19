//! Performance benchmarks for KagiNote audio processing pipeline
//!
//! These tests validate that the implementation meets the performance targets
//! specified in the requirements.

use kaginote_lib::audio::capture::{AudioCaptureService, AudioConfig};
use kaginote_lib::audio::vad::{SileroVAD, VADProcessor};
use kaginote_lib::audio::types::{AudioData, AudioSource, VADConfig};
use kaginote_lib::asr::whisper::{WhisperEngine, WhisperConfig};
use kaginote_lib::asr::types::{TranscriptionContext, ModelTier};
use std::time::{SystemTime, Instant};

#[tokio::test]
async fn benchmark_audio_capture_initialization() {
    // ARRANGE
    let config = AudioConfig::default();
    
    // ACT
    let start = Instant::now();
    let result = AudioCaptureService::new(config).await;
    let init_time = start.elapsed();
    
    // ASSERT - Should initialize within 100ms (target from test requirements)
    println!("Audio capture initialization time: {}ms", init_time.as_millis());
    
    // Allow failure if no audio devices available (CI/headless environments)
    if let Ok(capture) = result {
        assert!(init_time.as_millis() < 100, 
                "Audio capture initialization took {}ms, should be <100ms", 
                init_time.as_millis());
        assert!(capture.is_ready());
    } else {
        println!("Audio capture initialization failed (expected in CI/headless environments)");
    }
}

#[tokio::test]
async fn benchmark_vad_processing_latency() {
    // ARRANGE
    let config = VADConfig::default();
    let vad = SileroVAD::new(config).await.unwrap();
    
    // Create test audio chunk (100ms)
    let test_audio = create_test_audio_chunk(0.1);
    
    // ACT - Process multiple chunks to get average latency
    let mut latencies = Vec::new();
    for _ in 0..10 {
        let start = Instant::now();
        let _result = vad.detect_speech(&test_audio).await.unwrap();
        latencies.push(start.elapsed().as_micros());
    }
    
    let avg_latency = latencies.iter().sum::<u128>() / latencies.len() as u128;
    
    // ASSERT - Average latency should be <50ms for VAD processing (target from requirements)
    println!("VAD processing average latency: {}μs", avg_latency);
    assert!(avg_latency < 50_000, 
            "VAD processing latency: {}μs, should be <50,000μs", avg_latency);
}

#[tokio::test]
async fn benchmark_asr_real_time_factor() {
    // ARRANGE - Test all model tiers
    let test_cases = vec![
        (ModelTier::Standard, 1.0),      // RTF ≤1.0
        (ModelTier::HighAccuracy, 2.0),  // RTF ≤2.0  
        (ModelTier::Turbo, 0.8),         // RTF ≤0.8
    ];
    
    for (tier, max_rtf) in test_cases {
        let config = WhisperConfig {
            model_tier: tier,
            ..Default::default()
        };
        let engine = WhisperEngine::new(config).await.unwrap();
        
        // Create 5-second test audio
        let test_audio = create_test_audio_chunk(5.0);
        let context = TranscriptionContext::default();
        
        // ACT
        let start = Instant::now();
        let result = engine.transcribe(&test_audio, &context).await.unwrap();
        let processing_time = start.elapsed();
        
        // ASSERT - Real-time factor should meet tier requirements
        let rtf = processing_time.as_secs_f32() / test_audio.duration_seconds;
        println!("Model tier: {:?}, RTF: {:.3}, Processing time: {}ms", 
                 tier, rtf, processing_time.as_millis());
        
        assert!(rtf <= max_rtf, 
                "RTF for {:?} should be ≤{:.1}, got {:.2}", tier, max_rtf, rtf);
        assert!(!result.text.is_empty());
        assert!(result.confidence > 0.0);
    }
}

#[tokio::test]
async fn benchmark_memory_usage() {
    // ARRANGE
    let vad_config = VADConfig::default();
    let whisper_config = WhisperConfig::default();
    
    // ACT - Initialize systems and measure memory
    let initial_memory = get_memory_usage_mb();
    
    let _vad = SileroVAD::new(vad_config).await.unwrap();
    let vad_memory = get_memory_usage_mb();
    
    let _engine = WhisperEngine::new(whisper_config).await.unwrap();
    let total_memory = get_memory_usage_mb();
    
    let vad_overhead = vad_memory - initial_memory;
    let asr_overhead = total_memory - vad_memory;
    let total_overhead = total_memory - initial_memory;
    
    // ASSERT - Memory usage should be reasonable
    println!("Memory usage - VAD: {}MB, ASR: {}MB, Total: {}MB", 
             vad_overhead, asr_overhead, total_overhead);
    
    // These are generous limits for a transcription system
    assert!(vad_overhead < 200, "VAD memory overhead should be <200MB, got {}MB", vad_overhead);
    assert!(asr_overhead < 3000, "ASR memory overhead should be <3GB, got {}MB", asr_overhead);
    assert!(total_overhead < 8000, "Total memory overhead should be <8GB, got {}MB", total_overhead);
}

#[tokio::test]
async fn benchmark_end_to_end_pipeline() {
    // ARRANGE
    let vad_config = VADConfig::default();
    let whisper_config = WhisperConfig::default();
    
    // Initialize components
    let vad = SileroVAD::new(vad_config).await.unwrap();
    let asr_engine = WhisperEngine::new(whisper_config).await.unwrap();
    
    // Create realistic test audio (30 seconds)
    let test_audio = create_test_audio_chunk(30.0);
    
    // ACT - Run complete pipeline
    let pipeline_start = Instant::now();
    
    // VAD processing
    let vad_start = Instant::now();
    let vad_result = vad.detect_speech(&test_audio).await.unwrap();
    let vad_time = vad_start.elapsed();
    
    // ASR processing (only if speech detected)
    let asr_time = if vad_result.has_speech {
        let asr_start = Instant::now();
        let context = TranscriptionContext::default();
        let transcription = asr_engine.transcribe(&test_audio, &context).await.unwrap();
        let asr_elapsed = asr_start.elapsed();
        
        // Validate transcription quality
        assert!(!transcription.text.is_empty());
        assert!(transcription.confidence > 0.5);
        
        asr_elapsed
    } else {
        std::time::Duration::from_millis(0)
    };
    
    let total_time = pipeline_start.elapsed();
    
    // ASSERT - End-to-end performance targets
    let vad_rtf = vad_time.as_secs_f32() / test_audio.duration_seconds;
    let asr_rtf = asr_time.as_secs_f32() / test_audio.duration_seconds;
    let total_rtf = total_time.as_secs_f32() / test_audio.duration_seconds;
    
    println!("End-to-end pipeline performance:");
    println!("  VAD: {}ms (RTF: {:.3})", vad_time.as_millis(), vad_rtf);
    println!("  ASR: {}ms (RTF: {:.3})", asr_time.as_millis(), asr_rtf);
    println!("  Total: {}ms (RTF: {:.3})", total_time.as_millis(), total_rtf);
    
    assert!(vad_rtf < 0.1, "VAD RTF should be <0.1, got {:.3}", vad_rtf);
    assert!(total_rtf < 1.5, "Total pipeline RTF should be <1.5x for real-time processing, got {:.3}", total_rtf);
}

// Helper functions

fn create_test_audio_chunk(duration_seconds: f32) -> AudioData {
    let sample_rate = 16000;
    let num_samples = (duration_seconds * sample_rate as f32) as usize;
    
    // Generate speech-like test signal
    let samples = kaginote_lib::asr::whisper::generate_test_signal(440.0, duration_seconds, sample_rate);
    
    AudioData {
        samples,
        sample_rate: sample_rate as u32,
        channels: 1,
        timestamp: SystemTime::now(),
        source_channel: AudioSource::Microphone,
        duration_seconds,
    }
}

fn get_memory_usage_mb() -> usize {
    // Simplified memory usage measurement
    // In a real implementation, this would use more accurate system APIs
    use sysinfo::{System, RefreshKind, MemoryRefreshKind};
    
    let mut sys = System::new_with_specifics(RefreshKind::new().with_memory(MemoryRefreshKind::new().with_ram()));
    sys.refresh_memory();
    
    let used_memory = sys.used_memory() as f64 / (1024.0 * 1024.0);
    used_memory as usize
}