//! Integration test for the Audio Playback Simulator
//!
//! This test validates the audio playback simulator functionality in isolation,
//! without dependencies on other test modules that may have compilation issues.

use kaginote_lib::audio::types::AudioSource;
use std::io::Write;
use std::time::Duration;
use tempfile::NamedTempFile;
use tokio;

// Import our simulator module by including it directly
#[path = "diarization_realtime/audio_playback_simulator.rs"]
mod audio_playback_simulator;

use audio_playback_simulator::*;

/// Create a simple test WAV file for testing
async fn create_test_wav_file() -> Result<NamedTempFile, Box<dyn std::error::Error>> {
    let mut temp_file = NamedTempFile::new()?;
    
    // Create a simple WAV file with a sine wave
    let sample_rate = 44100u32;
    let duration_seconds = 2.0;
    let frequency = 440.0; // A4 note
    let samples_count = (sample_rate as f64 * duration_seconds) as usize;

    // WAV header
    temp_file.write_all(b"RIFF")?;
    temp_file.write_all(&(36 + samples_count * 2).to_le_bytes())?; // File size - 8
    temp_file.write_all(b"WAVE")?;
    temp_file.write_all(b"fmt ")?;
    temp_file.write_all(&16u32.to_le_bytes())?; // PCM header size
    temp_file.write_all(&1u16.to_le_bytes())?; // PCM format
    temp_file.write_all(&1u16.to_le_bytes())?; // Mono
    temp_file.write_all(&sample_rate.to_le_bytes())?; // Sample rate
    temp_file.write_all(&(sample_rate * 2).to_le_bytes())?; // Byte rate
    temp_file.write_all(&2u16.to_le_bytes())?; // Block align
    temp_file.write_all(&16u16.to_le_bytes())?; // Bits per sample
    temp_file.write_all(b"data")?;
    temp_file.write_all(&(samples_count * 2).to_le_bytes())?; // Data size

    // Generate sine wave
    for i in 0..samples_count {
        let t = i as f64 / sample_rate as f64;
        let sample = (2.0 * std::f64::consts::PI * frequency * t).sin();
        let sample_i16 = (sample * i16::MAX as f64) as i16;
        temp_file.write_all(&sample_i16.to_le_bytes())?;
    }

    temp_file.flush()?;
    // Ensure the file is fully written by syncing
    temp_file.as_file().sync_all()?;
    Ok(temp_file)
}

#[tokio::test]
async fn test_audio_simulator_creation() {
    let config = AudioPlaybackConfig::default();
    let simulator = AudioPlaybackSimulator::new(config);
    
    // Check initial state
    assert_eq!(simulator.get_state(), PlaybackState::Stopped);
    assert!(!simulator.is_ready());
    assert_eq!(simulator.get_position_seconds(), 0.0);
    assert_eq!(simulator.get_total_duration_seconds(), 0.0);
}

#[tokio::test]
async fn test_load_audio_file() {
    let temp_wav = create_test_wav_file().await.expect("Failed to create test WAV");
    
    let config = AudioPlaybackConfig::default();
    let mut simulator = AudioPlaybackSimulator::new(config);
    
    let result = simulator.load_audio_file(temp_wav.path()).await;
    assert!(result.is_ok(), "Failed to load audio file: {:?}", result);
    assert!(simulator.is_ready());
    
    let total_duration = simulator.get_total_duration_seconds();
    assert!(total_duration > 1.8 && total_duration < 2.2, 
            "Expected ~2 seconds duration, got {}", total_duration);
    
    let (sample_rate, channels, _) = simulator.get_audio_info();
    assert_eq!(sample_rate, 16000, "Should resample to 16kHz");
    assert_eq!(channels, 1, "Should convert to mono");
}

#[tokio::test]
async fn test_playback_control_states() {
    let temp_wav = create_test_wav_file().await.expect("Failed to create test WAV");
    
    let config = AudioPlaybackConfig::default();
    let mut simulator = AudioPlaybackSimulator::new(config);
    
    simulator.load_audio_file(temp_wav.path()).await.unwrap();
    let _receiver = simulator.create_audio_channel();
    
    // Test initial state
    assert_eq!(simulator.get_state(), PlaybackState::Stopped);
    
    // Start playback
    simulator.start_playback().await.unwrap();
    
    // Wait a bit for the stream to start
    tokio::time::sleep(Duration::from_millis(50)).await;
    assert_eq!(simulator.get_state(), PlaybackState::Playing);
    
    // Pause
    simulator.pause().await.unwrap();
    assert_eq!(simulator.get_state(), PlaybackState::Paused);
    
    // Resume
    simulator.resume().await.unwrap();
    assert_eq!(simulator.get_state(), PlaybackState::Playing);
    
    // Stop
    simulator.stop().await.unwrap();
    assert_eq!(simulator.get_state(), PlaybackState::Stopped);
    assert_eq!(simulator.get_position_seconds(), 0.0);
}

#[tokio::test]
async fn test_audio_streaming() {
    let temp_wav = create_test_wav_file().await.expect("Failed to create test WAV");
    
    let config = AudioPlaybackConfig {
        chunk_duration_ms: 50, // 50ms chunks for faster testing
        ..Default::default()
    };
    let mut simulator = AudioPlaybackSimulator::new(config);
    
    simulator.load_audio_file(temp_wav.path()).await.unwrap();
    let mut receiver = simulator.create_audio_channel();
    
    simulator.start_playback().await.unwrap();
    
    // Receive a few chunks
    let mut chunks_received = 0;
    let max_chunks = 10;
    
    while chunks_received < max_chunks {
        match tokio::time::timeout(Duration::from_millis(200), receiver.recv()).await {
            Ok(Some(audio_data)) => {
                assert_eq!(audio_data.sample_rate, 16000);
                assert_eq!(audio_data.channels, 1);
                assert!(!audio_data.samples.is_empty());
                assert_eq!(audio_data.source_channel, AudioSource::File);
                chunks_received += 1;
                println!("Received chunk {} with {} samples", chunks_received, audio_data.samples.len());
            }
            Ok(None) => {
                println!("Audio channel closed");
                break;
            }
            Err(_) => {
                println!("Timeout waiting for audio chunk after receiving {} chunks", chunks_received);
                break;
            }
        }
    }
    
    simulator.stop().await.unwrap();
    assert!(chunks_received > 0, "Should have received at least one chunk, got {}", chunks_received);
}

#[tokio::test]
async fn test_metrics_collection() {
    let temp_wav = create_test_wav_file().await.expect("Failed to create test WAV");
    
    let config = AudioPlaybackConfig {
        chunk_duration_ms: 50,
        enable_metrics: true,
        ..Default::default()
    };
    let mut simulator = AudioPlaybackSimulator::new(config);
    
    simulator.load_audio_file(temp_wav.path()).await.unwrap();
    let _receiver = simulator.create_audio_channel();
    
    simulator.start_playback().await.unwrap();
    
    // Let it play for a bit
    tokio::time::sleep(Duration::from_millis(300)).await;
    
    let metrics = simulator.get_metrics();
    assert!(metrics.chunks_sent > 0);
    assert!(metrics.total_duration_processed > 0.0);
    assert!(metrics.real_time_factor() > 0.0);
    
    println!("Metrics: chunks={}, duration={:.2}s, RTF={:.2}", 
             metrics.chunks_sent, 
             metrics.total_duration_processed, 
             metrics.real_time_factor());
    
    simulator.stop().await.unwrap();
}

#[tokio::test]
async fn test_seek_functionality() {
    let temp_wav = create_test_wav_file().await.expect("Failed to create test WAV");
    
    let config = AudioPlaybackConfig::default();
    let mut simulator = AudioPlaybackSimulator::new(config);
    
    simulator.load_audio_file(temp_wav.path()).await.unwrap();
    
    // Test seeking to middle
    simulator.seek_to_position(1.0).await.unwrap();
    let position = simulator.get_position_seconds();
    assert!((position - 1.0).abs() < 0.1, "Seek position incorrect: {}", position);
    
    // Test invalid seek positions
    assert!(simulator.seek_to_position(-1.0).await.is_err());
    assert!(simulator.seek_to_position(10.0).await.is_err());
}

#[tokio::test]
async fn test_config_variations() {
    let temp_wav = create_test_wav_file().await.expect("Failed to create test WAV");
    
    // Test different configurations
    let configs = vec![
        AudioPlaybackConfig {
            chunk_duration_ms: 25, // Very small chunks
            target_sample_rate: 8000, // Lower sample rate
            ..Default::default()
        },
        AudioPlaybackConfig {
            chunk_duration_ms: 200, // Larger chunks
            target_channels: 2, // Stereo output
            ..Default::default()
        },
        AudioPlaybackConfig {
            speed_multiplier: 2.0, // 2x speed
            loop_playback: true,
            silence_between_loops_ms: 100,
            ..Default::default()
        },
    ];
    
    for (i, config) in configs.into_iter().enumerate() {
        let mut simulator = AudioPlaybackSimulator::new(config.clone());
        let result = simulator.load_audio_file(temp_wav.path()).await;
        assert!(result.is_ok(), "Config {} failed to load audio: {:?}", i, result);
        
        let (sample_rate, channels, _) = simulator.get_audio_info();
        assert_eq!(sample_rate, config.target_sample_rate);
        assert_eq!(channels, config.target_channels);
        
        println!("Config {}: {}Hz, {} ch, {}ms chunks", i, sample_rate, channels, config.chunk_duration_ms);
    }
}

#[tokio::test]
async fn test_error_conditions() {
    let config = AudioPlaybackConfig::default();
    let mut simulator = AudioPlaybackSimulator::new(config);
    
    // Test loading non-existent file
    let result = simulator.load_audio_file("/non/existent/file.wav").await;
    assert!(result.is_err(), "Should fail to load non-existent file");
    
    // Test starting playback without loading audio
    let result = simulator.start_playback().await;
    assert!(result.is_err(), "Should fail to start playback without audio");
    
    // Test operations without creating channel
    simulator.load_audio_file("dummy_path").await.ok(); // This will fail but that's expected
    let result = simulator.start_playback().await;
    assert!(result.is_err(), "Should fail without audio channel");
}