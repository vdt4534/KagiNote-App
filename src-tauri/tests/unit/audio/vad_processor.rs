//! Voice Activity Detection (VAD) Unit Tests
//! 
//! These tests are written BEFORE implementation exists (TDD).
//! Tests define the contract for Silero-VAD v5 integration.
//! All tests should FAIL initially - this is correct TDD behavior.

use rstest::*;
use mockall::predicate::*;
use tokio_test;
use anyhow::Result;
use std::time::Instant;
use rand::Rng;

// Import from the main crate
use kaginote_lib::audio::vad::{VADProcessor, SileroVAD};
use kaginote_lib::audio::types::{AudioData, AudioSource, VADError, VADResult, VADConfig};

/// Test fixtures for VAD testing
#[fixture]
fn vad_config() -> VADConfig {
    VADConfig {
        threshold: 0.5,
        min_speech_duration_ms: 500,
        max_speech_duration_ms: 30000,
        padding_before_ms: 200,
        padding_after_ms: 200,
        adaptive_threshold: false,
        context_frames: 16,
    }
}

#[fixture]
fn speech_audio() -> AudioData {
    AudioData {
        sample_rate: 16000,
        channels: 1,
        samples: generate_speech_samples(5.0), // 5 seconds of speech
        timestamp: std::time::SystemTime::now(),
        source_channel: AudioSource::Microphone,
        duration_seconds: 5.0,
    }
}

#[fixture] 
fn silence_audio() -> AudioData {
    AudioData {
        sample_rate: 16000,
        channels: 1,
        samples: generate_silence_samples(3.0), // 3 seconds of silence
        timestamp: std::time::SystemTime::now(),
        source_channel: AudioSource::Microphone,
        duration_seconds: 3.0,
    }
}

#[fixture]
fn noisy_speech_audio() -> AudioData {
    AudioData {
        sample_rate: 16000,
        channels: 1,
        samples: generate_noisy_speech_samples(5.0, -20.0), // Speech with -20dB noise
        timestamp: std::time::SystemTime::now(),
        source_channel: AudioSource::System,
        duration_seconds: 5.0,
    }
}

/// Core VAD Functionality Tests
mod vad_initialization {
    use super::*;

    #[tokio::test]
    async fn should_initialize_silero_vad_with_correct_parameters() {
        // ARRANGE
        let config = vad_config();

        // ACT - This WILL FAIL because SileroVAD doesn't exist
        let result = SileroVAD::new(config).await;

        // ASSERT - Define what implementation must provide
        assert!(result.is_ok());
        let vad = result.unwrap();
        assert_eq!(vad.get_threshold(), 0.5);
        assert_eq!(vad.get_min_speech_duration_ms(), 500);
        assert!(vad.is_initialized());
    }

    #[tokio::test]
    async fn should_fail_with_invalid_threshold() {
        // ARRANGE
        let mut invalid_config = vad_config();
        invalid_config.threshold = 1.5; // Invalid threshold > 1.0

        // ACT
        let result = SileroVAD::new(invalid_config).await;

        // ASSERT - Implementation must validate parameters
        assert!(result.is_err());
        match result.unwrap_err() {
            VADError::InvalidThreshold(threshold) => assert_eq!(threshold, 1.5),
            _ => panic!("Expected InvalidThreshold error"),
        }
    }

    #[tokio::test] 
    async fn should_load_silero_model_successfully() {
        // ARRANGE
        let config = vad_config();

        // ACT
        let vad = SileroVAD::new(config).await.unwrap();

        // ASSERT - Model should be loaded and ready
        assert!(vad.is_model_loaded());
        
        let model_info = vad.get_model_info();
        assert_eq!(model_info.version, "v5.0");
        assert_eq!(model_info.sample_rate, 16000);
        assert!(model_info.supports_streaming);
    }

    #[tokio::test]
    async fn should_handle_missing_model_file_gracefully() {
        // ARRANGE
        let config = VADConfig {
            model_path: Some("/non/existent/model.onnx".into()),
            ..vad_config()
        };

        // ACT
        let result = SileroVAD::new(config).await;

        // ASSERT
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), VADError::ModelNotFound { .. }));
    }
}

mod vad_speech_detection {
    use super::*;

    #[tokio::test]
    async fn should_detect_speech_with_high_accuracy() {
        // ARRANGE
        let config = vad_config();
        let vad = SileroVAD::new(config).await.unwrap();
        let speech_audio = speech_audio();

        // ACT - This WILL FAIL because detect_speech doesn't exist
        let result = vad.detect_speech(&speech_audio).await;

        // ASSERT - Define expected behavior
        assert!(result.is_ok());
        let vad_result = result.unwrap();
        
        assert!(vad_result.has_speech);
        assert!(vad_result.confidence > 0.8);
        assert!(!vad_result.speech_segments.is_empty());
        
        // Should detect continuous speech in our test audio
        let total_speech_duration: f32 = vad_result.speech_segments
            .iter()
            .map(|seg| seg.end_time - seg.start_time)
            .sum();
        assert!(total_speech_duration > 4.0); // Most of the 5s should be speech
    }

    #[tokio::test]
    async fn should_reject_silence_and_background_noise() {
        // ARRANGE
        let config = vad_config();
        let vad = SileroVAD::new(config).await.unwrap();
        let silence_audio = silence_audio();

        // ACT
        let result = vad.detect_speech(&silence_audio).await.unwrap();

        // ASSERT - Should not detect speech in silence
        assert!(!result.has_speech);
        assert!(result.confidence < 0.3);
        assert!(result.speech_segments.is_empty());
    }

    #[tokio::test]
    async fn should_handle_noisy_speech_correctly() {
        // ARRANGE
        let config = vad_config();
        let vad = SileroVAD::new(config).await.unwrap();
        let noisy_audio = noisy_speech_audio();

        // ACT
        let result = vad.detect_speech(&noisy_audio).await.unwrap();

        // ASSERT - Should still detect speech despite noise
        assert!(result.has_speech);
        assert!(result.confidence > 0.6); // Lower confidence due to noise
        assert!(!result.speech_segments.is_empty());
        
        // Should provide noise level estimation
        assert!(result.estimated_snr.is_some());
        let snr = result.estimated_snr.unwrap();
        assert!(snr > 0.0 && snr < 30.0); // Reasonable SNR range
    }

    #[tokio::test]
    async fn should_respect_minimum_speech_duration() {
        // ARRANGE
        let mut config = vad_config();
        config.min_speech_duration_ms = 2000; // 2 seconds minimum
        let vad = SileroVAD::new(config).await.unwrap();
        
        // Create audio with short speech bursts
        let short_burst_audio = create_short_speech_bursts(vec![0.5, 0.3, 0.8]); // All < 2s

        // ACT
        let result = vad.detect_speech(&short_burst_audio).await.unwrap();

        // ASSERT - Should filter out short bursts
        assert!(result.speech_segments.is_empty() || 
                result.speech_segments.iter().all(|seg| 
                    (seg.end_time - seg.start_time) >= 2.0
                ));
    }

    #[tokio::test] 
    async fn should_split_long_speech_segments() {
        // ARRANGE
        let mut config = vad_config();
        config.max_speech_duration_ms = 10000; // 10 seconds maximum
        let vad = SileroVAD::new(config).await.unwrap();
        
        // Create 20 seconds of continuous speech
        let long_speech = create_continuous_speech(20.0);

        // ACT
        let result = vad.detect_speech(&long_speech).await.unwrap();

        // ASSERT - Should split into segments ≤10 seconds
        assert!(result.has_speech);
        assert!(result.speech_segments.len() >= 2);
        
        for segment in result.speech_segments {
            let duration = segment.end_time - segment.start_time;
            assert!(duration <= 10.5); // Allow small tolerance
        }
    }
}

mod vad_adaptive_threshold {
    use super::*;

    #[tokio::test]
    async fn should_adapt_threshold_to_noise_levels() {
        // ARRANGE
        let mut config = vad_config();
        config.adaptive_threshold = true;
        let vad = SileroVAD::new(config).await.unwrap();

        // Test with increasing noise levels
        let noise_levels = vec![-40.0, -30.0, -20.0, -15.0]; // dB levels

        // ACT & ASSERT
        for noise_level in noise_levels {
            let noisy_audio = generate_speech_with_noise(5.0, noise_level);
            let result = vad.detect_speech(&noisy_audio).await.unwrap();
            
            // Should still detect speech and show adapted threshold
            assert!(result.has_speech);
            assert!(result.adapted_threshold.is_some());
            
            let adapted_threshold = result.adapted_threshold.unwrap();
            // Higher noise should result in higher threshold
            if noise_level > -25.0 {
                assert!(adapted_threshold > 0.5); // Above base threshold
            }
        }
    }

    #[tokio::test]
    async fn should_maintain_consistent_detection_across_conditions() {
        // ARRANGE
        let mut config = vad_config();
        config.adaptive_threshold = true;
        let vad = SileroVAD::new(config).await.unwrap();

        // Test same speech content with different noise levels
        let base_speech = generate_speech_samples(5.0);
        let clean_audio = AudioData {
            samples: base_speech.clone(),
            sample_rate: 16000,
            channels: 1,
            timestamp: std::time::SystemTime::now(),
            source_channel: AudioSource::Microphone,
            duration_seconds: 5.0,
        };

        let noisy_audio = AudioData {
            samples: add_noise_to_samples(&base_speech, -20.0),
            ..clean_audio
        };

        // ACT
        let clean_result = vad.detect_speech(&clean_audio).await.unwrap();
        let noisy_result = vad.detect_speech(&noisy_audio).await.unwrap();

        // ASSERT - Should detect speech in both cases
        assert!(clean_result.has_speech);
        assert!(noisy_result.has_speech);
        
        // Speech segments should be similar (within 20% duration)
        let clean_duration: f32 = clean_result.speech_segments
            .iter()
            .map(|seg| seg.end_time - seg.start_time)
            .sum();
        let noisy_duration: f32 = noisy_result.speech_segments
            .iter()
            .map(|seg| seg.end_time - seg.start_time)
            .sum();
            
        let duration_diff = (clean_duration - noisy_duration).abs() / clean_duration;
        assert!(duration_diff < 0.2); // Within 20% difference
    }
}

mod vad_streaming_processing {
    use super::*;

    #[tokio::test]
    async fn should_process_real_time_audio_chunks() {
        // ARRANGE
        let config = vad_config();
        let mut vad = SileroVAD::new(config).await.unwrap();
        
        // Create streaming chunks (100ms each)
        let chunk_duration = 0.1;
        let chunks = create_streaming_chunks(10, chunk_duration);

        // ACT
        let mut results = Vec::new();
        for chunk in chunks {
            let chunk_result = vad.process_chunk(&chunk).await.unwrap();
            results.push(chunk_result);
        }

        // ASSERT - Should process all chunks successfully
        assert_eq!(results.len(), 10);
        
        // Should maintain context across chunks
        for (i, result) in results.iter().enumerate() {
            // Each result should have timing relative to stream start
            if result.has_speech {
                for segment in &result.speech_segments {
                    let expected_start_range = i as f32 * chunk_duration;
                    assert!(segment.start_time >= expected_start_range);
                }
            }
        }
    }

    #[tokio::test]
    async fn should_maintain_low_latency_for_streaming() {
        // ARRANGE
        let config = vad_config();
        let vad = SileroVAD::new(config).await.unwrap();
        
        let test_chunk = create_single_chunk(0.1); // 100ms chunk

        // ACT - Measure processing time
        let start = Instant::now();
        let _result = vad.process_chunk(&test_chunk).await.unwrap();
        let processing_time = start.elapsed();

        // ASSERT - Should process within 10ms for real-time performance
        assert!(
            processing_time.as_millis() < 10,
            "VAD processing took {}ms, should be <10ms for real-time",
            processing_time.as_millis()
        );
    }

    #[tokio::test]
    async fn should_preserve_context_across_chunk_boundaries() {
        // ARRANGE
        let config = vad_config();
        let mut vad = SileroVAD::new(config).await.unwrap();

        // Create audio that spans chunk boundaries
        let speech_chunks = create_speech_spanning_chunks();

        // ACT
        let mut all_segments = Vec::new();
        for chunk in speech_chunks {
            let result = vad.process_chunk(&chunk).await.unwrap();
            all_segments.extend(result.speech_segments);
        }

        // ASSERT - Should not have artificial gaps at chunk boundaries
        for segments in all_segments.windows(2) {
            let gap = segments[1].start_time - segments[0].end_time;
            assert!(gap < 0.2); // No gaps >200ms between continuous speech
        }
    }
}

mod vad_error_handling {
    use super::*;

    #[tokio::test]
    async fn should_handle_empty_audio_gracefully() {
        // ARRANGE
        let config = vad_config();
        let vad = SileroVAD::new(config).await.unwrap();
        let empty_audio = AudioData {
            samples: vec![],
            sample_rate: 16000,
            channels: 1,
            timestamp: std::time::SystemTime::now(),
            source_channel: AudioSource::Microphone,
            duration_seconds: 0.0,
        };

        // ACT
        let result = vad.detect_speech(&empty_audio).await;

        // ASSERT
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), VADError::EmptyAudio));
    }

    #[tokio::test]
    async fn should_handle_invalid_sample_rate() {
        // ARRANGE
        let config = vad_config();
        let vad = SileroVAD::new(config).await.unwrap();
        let invalid_audio = AudioData {
            samples: vec![0.1, 0.2, 0.3],
            sample_rate: 8000, // Invalid for Silero VAD
            channels: 1,
            timestamp: std::time::SystemTime::now(),
            source_channel: AudioSource::Microphone,
            duration_seconds: 0.1,
        };

        // ACT
        let result = vad.detect_speech(&invalid_audio).await;

        // ASSERT
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), VADError::UnsupportedSampleRate(8000)));
    }

    #[tokio::test]
    async fn should_handle_clipped_audio() {
        // ARRANGE
        let config = vad_config();
        let vad = SileroVAD::new(config).await.unwrap();
        let clipped_audio = AudioData {
            samples: vec![1.5, -1.5, 2.0, -2.0], // Values outside [-1, 1] range
            sample_rate: 16000,
            channels: 1,
            timestamp: std::time::SystemTime::now(),
            source_channel: AudioSource::Microphone,
            duration_seconds: 0.001,
        };

        // ACT
        let result = vad.detect_speech(&clipped_audio).await;

        // ASSERT - Should either handle gracefully or provide clear error
        if result.is_err() {
            assert!(matches!(result.unwrap_err(), VADError::ClippedAudio { .. }));
        } else {
            // If handled gracefully, should indicate clipping in result
            let vad_result = result.unwrap();
            assert!(vad_result.has_clipping_warning);
        }
    }
}

// Helper functions for test data generation

fn generate_speech_samples(duration_seconds: f32) -> Vec<f32> {
    let sample_rate = 16000;
    let num_samples = (duration_seconds * sample_rate as f32) as usize;
    let mut samples = Vec::with_capacity(num_samples);
    
    // Generate speech-like signal with multiple formants
    for i in 0..num_samples {
        let t = i as f32 / sample_rate as f32;
        
        // Fundamental + formants
        let f0 = 150.0; // Fundamental frequency
        let formants = [800.0, 1200.0, 2400.0]; // Formant frequencies
        
        let mut sample = 0.0;
        sample += 0.3 * (2.0 * std::f32::consts::PI * f0 * t).sin();
        for &formant in &formants {
            sample += 0.2 * (2.0 * std::f32::consts::PI * formant * t).sin();
        }
        
        // Apply speech envelope (with pauses)
        let envelope = speech_envelope(t, duration_seconds);
        samples.push(sample * envelope * 0.3);
    }
    
    samples
}

fn generate_silence_samples(duration_seconds: f32) -> Vec<f32> {
    let sample_rate = 16000;
    let num_samples = (duration_seconds * sample_rate as f32) as usize;
    let mut samples = Vec::with_capacity(num_samples);
    
    // Generate very low-level noise (microphone noise floor)
    let mut rng = rand::thread_rng();
    for _ in 0..num_samples {
        let noise = (rng.gen::<f32>() - 0.5) * 0.001; // -60dB noise floor
        samples.push(noise);
    }
    
    samples
}

fn generate_noisy_speech_samples(duration_seconds: f32, noise_level_db: f32) -> Vec<f32> {
    let speech = generate_speech_samples(duration_seconds);
    add_noise_to_samples(&speech, noise_level_db)
}

fn add_noise_to_samples(samples: &[f32], noise_level_db: f32) -> Vec<f32> {
    let noise_amplitude = 10.0_f32.powf(noise_level_db / 20.0);
    let mut rng = rand::thread_rng();
    
    samples.iter().map(|&sample| {
        let noise = (rng.gen::<f32>() - 0.5) * noise_amplitude;
        sample + noise
    }).collect()
}

fn speech_envelope(t: f32, duration: f32) -> f32 {
    // Simulate natural speech patterns with pauses
    let speech_rate = 2.5; // syllables per second
    let syllable_time = t * speech_rate;
    let syllable_phase = syllable_time % 1.0;
    
    // 60% speech, 40% pause pattern
    let is_speech = syllable_phase < 0.6 && (t * 0.5) as i32 % 3 < 2;
    
    if is_speech {
        (std::f32::consts::PI * syllable_phase / 0.6).sin()
    } else {
        0.0
    }
}

fn create_short_speech_bursts(durations: Vec<f32>) -> AudioData {
    let mut all_samples = Vec::new();
    let mut current_time = 0.0;
    
    for duration in durations {
        // Add silence gap
        let silence_duration = 1.0;
        all_samples.extend(generate_silence_samples(silence_duration));
        current_time += silence_duration;
        
        // Add speech burst
        all_samples.extend(generate_speech_samples(duration));
        current_time += duration;
    }
    
    AudioData {
        samples: all_samples,
        sample_rate: 16000,
        channels: 1,
        timestamp: std::time::SystemTime::now(),
        source_channel: AudioSource::Microphone,
        duration_seconds: current_time,
    }
}

fn create_continuous_speech(duration_seconds: f32) -> AudioData {
    AudioData {
        samples: generate_speech_samples(duration_seconds),
        sample_rate: 16000,
        channels: 1,
        timestamp: std::time::SystemTime::now(),
        source_channel: AudioSource::Microphone,
        duration_seconds,
    }
}

fn generate_speech_with_noise(duration_seconds: f32, noise_db: f32) -> AudioData {
    AudioData {
        samples: generate_noisy_speech_samples(duration_seconds, noise_db),
        sample_rate: 16000,
        channels: 1,
        timestamp: std::time::SystemTime::now(),
        source_channel: AudioSource::System,
        duration_seconds,
    }
}

fn create_streaming_chunks(count: usize, duration_each: f32) -> Vec<AudioData> {
    (0..count).map(|i| AudioData {
        samples: generate_speech_samples(duration_each),
        sample_rate: 16000,
        channels: 1,
        timestamp: std::time::SystemTime::now(),
        source_channel: AudioSource::Microphone,
        duration_seconds: duration_each,
    }).collect()
}

fn create_single_chunk(duration: f32) -> AudioData {
    AudioData {
        samples: generate_speech_samples(duration),
        sample_rate: 16000,
        channels: 1,
        timestamp: std::time::SystemTime::now(),
        source_channel: AudioSource::Microphone,
        duration_seconds: duration,
    }
}

fn create_speech_spanning_chunks() -> Vec<AudioData> {
    // Create speech that spans multiple chunks without artificial breaks
    let full_speech = generate_speech_samples(2.0); // 2 seconds total
    let chunk_size = 16000 / 10; // 100ms chunks
    
    let mut chunks = Vec::new();
    for i in (0..full_speech.len()).step_by(chunk_size) {
        let end = std::cmp::min(i + chunk_size, full_speech.len());
        let chunk_samples = full_speech[i..end].to_vec();
        
        chunks.push(AudioData {
            samples: chunk_samples,
            sample_rate: 16000,
            channels: 1,
            timestamp: std::time::SystemTime::now(),
            source_channel: AudioSource::Microphone,
            duration_seconds: (end - i) as f32 / 16000.0,
        });
    }
    
    chunks
}

/*
IMPLEMENTATION NOTES:
===================

These tests define the complete contract for VAD processing.
Implementation must provide:

1. SileroVAD struct with:
   - new(config) -> Result<Self, VADError>
   - detect_speech(audio) -> Result<VADResult, VADError>
   - process_chunk(chunk) -> Result<VADResult, VADError>
   - is_initialized() -> bool
   - get_threshold() -> f32

2. VADConfig struct with validation
3. VADResult struct with speech segments and confidence
4. VADError enum for all error scenarios
5. Silero-VAD v5 ONNX model integration
6. Real-time streaming support with context
7. Adaptive threshold adjustment for noise
8. Performance targets: <10ms processing latency

Performance Requirements:
- >95% accuracy on clean speech
- <5% false positive rate on silence
- Real-time processing (RTF < 0.1)
- Memory usage <100MB
- CPU usage <1% during operation

All these tests should FAIL initially - this is correct TDD behavior.
*/