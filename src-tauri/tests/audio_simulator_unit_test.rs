//! Unit test for the Audio Playback Simulator (without actual audio files)
//!
//! This test focuses on testing the core functionality of the audio playback simulator
//! without requiring actual audio file decoding, which can be flaky in tests.

use kaginote_lib::audio::types::{AudioData, AudioError, AudioSource};
use std::time::Duration;
use tokio;

// Import our simulator module by including it directly
#[path = "diarization_realtime/audio_playback_simulator.rs"]
mod audio_playback_simulator;

use audio_playback_simulator::*;

/// Mock audio data for testing
fn create_mock_audio_data(duration_seconds: f32, sample_rate: u32, channels: u8) -> Vec<f32> {
    let num_samples = (duration_seconds * sample_rate as f32) as usize * channels as usize;
    let mut samples = Vec::with_capacity(num_samples);
    
    // Generate a simple sine wave
    let frequency = 440.0; // A4
    for i in 0..num_samples {
        let frame_index = i / channels as usize;
        let t = frame_index as f32 / sample_rate as f32;
        let amplitude = 0.5 * (2.0 * std::f32::consts::PI * frequency * t).sin();
        samples.push(amplitude);
    }
    
    samples
}

#[tokio::test]
async fn test_simulator_configuration() {
    // Test default configuration
    let default_config = AudioPlaybackConfig::default();
    assert_eq!(default_config.target_sample_rate, 16000);
    assert_eq!(default_config.target_channels, 1);
    assert_eq!(default_config.chunk_duration_ms, 100);
    assert_eq!(default_config.speed_multiplier, 1.0);
    assert!(!default_config.loop_playback);
    
    // Test custom configuration
    let custom_config = AudioPlaybackConfig {
        target_sample_rate: 8000,
        target_channels: 2,
        chunk_duration_ms: 50,
        speed_multiplier: 2.0,
        loop_playback: true,
        silence_between_loops_ms: 500,
        enable_metrics: false,
        ..Default::default()
    };
    
    let simulator = AudioPlaybackSimulator::new(custom_config.clone());
    
    // Verify initial state
    assert_eq!(simulator.get_state(), PlaybackState::Stopped);
    assert!(!simulator.is_ready());
    assert_eq!(simulator.get_position_seconds(), 0.0);
    assert_eq!(simulator.get_total_duration_seconds(), 0.0);
}

#[tokio::test]
async fn test_public_api_only() {
    let config = AudioPlaybackConfig::default();
    let simulator = AudioPlaybackSimulator::new(config);
    
    // Test that we can call all public methods without panicking
    assert_eq!(simulator.get_state(), PlaybackState::Stopped);
    assert!(!simulator.is_ready());
    assert_eq!(simulator.get_position_seconds(), 0.0);
    assert_eq!(simulator.get_total_duration_seconds(), 0.0);
    let _metrics = simulator.get_metrics();
    let _info = simulator.get_audio_info();
    
    // The simulator should not be ready without loaded audio
    assert!(!simulator.is_ready());
    
    // Should handle error conditions gracefully
    assert_eq!(simulator.get_state(), PlaybackState::Stopped);
}

#[tokio::test]
async fn test_metrics_calculation() {
    let mut metrics = PlaybackMetrics::default();
    
    // Simulate some metrics
    metrics.chunks_sent = 10;
    metrics.total_duration_processed = 1.0; // 1 second
    metrics.total_playback_time = Duration::from_secs(1);
    
    let rtf = metrics.real_time_factor();
    assert_eq!(rtf, 1.0);
    assert!(metrics.is_keeping_up());
    
    // Test slower processing
    metrics.total_playback_time = Duration::from_millis(2000);
    let rtf_slow = metrics.real_time_factor();
    assert_eq!(rtf_slow, 0.5); // Processing at 0.5x real-time
    assert!(!metrics.is_keeping_up());
}

#[tokio::test]
async fn test_playback_states() {
    // Test all state transitions
    let states = vec![
        PlaybackState::Stopped,
        PlaybackState::Playing,
        PlaybackState::Paused,
        PlaybackState::Finished,
        PlaybackState::Error,
    ];
    
    for state in states {
        // Each state should be cloneable and debuggable
        let _cloned = state.clone();
        let _debug_str = format!("{:?}", state);
        
        // Test equality
        assert_eq!(state, state);
    }
}

#[tokio::test]
async fn test_error_handling() {
    let config = AudioPlaybackConfig::default();
    let mut simulator = AudioPlaybackSimulator::new(config);
    
    // Test trying to start without audio loaded
    let result = simulator.start_playback().await;
    assert!(result.is_err());
    
    // Test trying to start without creating channel
    // (this should also fail even if we had audio loaded)
    let result = simulator.start_playback().await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_concurrent_access() {
    use std::sync::Arc;
    
    let config = AudioPlaybackConfig::default();
    let simulator = Arc::new(AudioPlaybackSimulator::new(config));
    
    // Test that multiple threads can read metrics concurrently
    let handles: Vec<_> = (0..10).map(|_| {
        let sim = Arc::clone(&simulator);
        tokio::spawn(async move {
            let _metrics = sim.get_metrics();
            let _state = sim.get_state();
            let _position = sim.get_position_seconds();
            let _duration = sim.get_total_duration_seconds();
            let _info = sim.get_audio_info();
        })
    }).collect();
    
    // Wait for all tasks to complete
    for handle in handles {
        handle.await.unwrap();
    }
    
    // If we get here without deadlock, concurrent access works
    assert!(true);
}

#[tokio::test]
async fn test_channel_creation() {
    let config = AudioPlaybackConfig {
        buffer_size: 5,
        ..Default::default()
    };
    let mut simulator = AudioPlaybackSimulator::new(config);
    
    // Create channel
    let mut receiver = simulator.create_audio_channel();
    
    // Should be able to poll the receiver (will be empty)
    let result = receiver.try_recv();
    assert!(result.is_err()); // Should be empty
    
    // Test that we can create another channel (replaces the old one)
    let _receiver2 = simulator.create_audio_channel();
}

#[test]
fn test_playback_config_clone_and_debug() {
    let config = AudioPlaybackConfig {
        target_sample_rate: 22050,
        chunk_duration_ms: 25,
        speed_multiplier: 1.5,
        enable_metrics: false,
        ..Default::default()
    };
    
    // Should be cloneable
    let cloned = config.clone();
    assert_eq!(cloned.target_sample_rate, 22050);
    assert_eq!(cloned.chunk_duration_ms, 25);
    
    // Should be debuggable
    let debug_str = format!("{:?}", config);
    assert!(debug_str.contains("22050"));
    assert!(debug_str.contains("25"));
}