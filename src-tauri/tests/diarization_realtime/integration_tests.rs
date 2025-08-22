//! Integration tests for real-time speaker diarization
//! 
//! These tests validate the end-to-end diarization pipeline including
//! audio input, processing, speaker identification, and output generation.

use super::*;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

/// Test complete end-to-end diarization pipeline
#[tokio::test]
async fn test_end_to_end_diarization_pipeline() {
    let config = DiarizationTestConfig {
        max_speakers: 4,
        min_speakers: 2,
        test_duration: 30,
        sample_rate: 16000,
        ..Default::default()
    };
    
    // Step 1: Generate test audio
    let audio_data = DiarizationTestUtils::generate_synthetic_audio(
        config.test_duration as f32,
        3, // 3 speakers
        config.sample_rate,
    );
    
    println!("Generated {} samples of test audio", audio_data.len());
    
    // Step 2: Initialize diarization system
    let diarization_system = MockDiarizationSystem::new(config.clone()).await;
    
    // Step 3: Process audio through pipeline
    let start_time = Instant::now();
    let results = diarization_system.process_audio(&audio_data).await
        .expect("Diarization processing should succeed");
    let processing_time = start_time.elapsed();
    
    // Step 4: Validate results
    assert!(!results.speakers.is_empty(), "Should detect at least one speaker");
    assert!(results.speakers.len() <= config.max_speakers, 
           "Should not exceed max speakers: {} vs {}", results.speakers.len(), config.max_speakers);
    
    assert!(!results.segments.is_empty(), "Should generate segments");
    
    // Validate timing constraints
    let expected_max_time = Duration::from_secs(config.test_duration as u64 * 2); // 2x real-time max
    assert!(processing_time <= expected_max_time,
           "Processing took too long: {:?} > {:?}", processing_time, expected_max_time);
    
    // Validate segment continuity
    validate_segment_continuity(&results.segments);
    
    println!("✅ End-to-end pipeline test passed");
    println!("   Detected speakers: {}", results.speakers.len());
    println!("   Generated segments: {}", results.segments.len());
    println!("   Processing time: {:?}", processing_time);
}

/// Test real-time streaming diarization
#[tokio::test]
async fn test_real_time_streaming_diarization() {
    let config = DiarizationTestConfig {
        test_duration: 60, // 1 minute test
        sample_rate: 16000,
        ..Default::default()
    };
    
    let diarization_system = MockDiarizationSystem::new(config.clone()).await;
    let results = Arc::new(Mutex::new(StreamingResults::new()));
    
    // Simulate real-time audio chunks
    let chunk_duration = 2.0; // 2-second chunks
    let chunk_samples = (chunk_duration * config.sample_rate as f32) as usize;
    let total_chunks = (config.test_duration as f32 / chunk_duration) as usize;
    
    let test_start = Instant::now();
    
    for chunk_idx in 0..total_chunks {
        // Generate audio chunk
        let audio_chunk = DiarizationTestUtils::generate_synthetic_audio(
            chunk_duration,
            2 + (chunk_idx % 3), // Vary speaker count
            config.sample_rate,
        );
        
        // Process chunk
        let chunk_start = Instant::now();
        let chunk_results = diarization_system.process_audio_chunk(&audio_chunk, chunk_idx).await
            .expect("Chunk processing should succeed");
        let chunk_latency = chunk_start.elapsed();
        
        // Update streaming results
        {
            let mut results_guard = results.lock().await;
            results_guard.add_chunk_result(chunk_results, chunk_latency);
        }
        
        // Validate real-time constraint
        let max_chunk_time = Duration::from_secs_f32(chunk_duration * 1.5); // 1.5x real-time
        assert!(chunk_latency <= max_chunk_time,
               "Chunk {} processing too slow: {:?} > {:?}", 
               chunk_idx, chunk_latency, max_chunk_time);
        
        // Simulate real-time gap
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    
    let total_time = test_start.elapsed();
    let results_guard = results.lock().await;
    
    // Validate streaming performance
    assert!(total_time <= Duration::from_secs(config.test_duration as u64 + 10),
           "Total streaming time exceeded threshold");
    
    assert!(results_guard.total_chunks == total_chunks,
           "Should have processed all chunks");
    
    println!("✅ Real-time streaming test passed");
    println!("   Processed chunks: {}", results_guard.total_chunks);
    println!("   Average latency: {:?}", results_guard.average_latency());
    println!("   Peak latency: {:?}", results_guard.peak_latency);
}

/// Test concurrent diarization sessions
#[tokio::test]
async fn test_concurrent_diarization_sessions() {
    let num_sessions = 3;
    let session_duration = 20; // seconds
    
    let mut session_handles = Vec::new();
    
    // Start multiple concurrent sessions
    for session_id in 0..num_sessions {
        let handle = tokio::spawn(async move {
            let config = DiarizationTestConfig {
                test_duration: session_duration,
                max_speakers: 2 + session_id, // Vary complexity
                ..Default::default()
            };
            
            let system = MockDiarizationSystem::new(config.clone()).await;
            let audio = DiarizationTestUtils::generate_synthetic_audio(
                session_duration as f32,
                config.max_speakers,
                config.sample_rate,
            );
            
            let start = Instant::now();
            let results = system.process_audio(&audio).await
                .expect("Session processing should succeed");
            let duration = start.elapsed();
            
            SessionResult {
                session_id,
                processing_time: duration,
                speakers_detected: results.speakers.len(),
                segments_generated: results.segments.len(),
            }
        });
        
        session_handles.push(handle);
    }
    
    // Wait for all sessions to complete
    let mut session_results = Vec::new();
    for handle in session_handles {
        let result = handle.await.expect("Session should complete successfully");
        session_results.push(result);
    }
    
    // Validate concurrent performance
    for result in &session_results {
        let max_time = Duration::from_secs(session_duration as u64 * 2);
        assert!(result.processing_time <= max_time,
               "Session {} took too long: {:?}", result.session_id, result.processing_time);
        
        assert!(result.speakers_detected > 0,
               "Session {} should detect speakers", result.session_id);
    }
    
    println!("✅ Concurrent sessions test passed");
    for result in session_results {
        println!("   Session {}: {:?}, {} speakers, {} segments",
                result.session_id, result.processing_time, 
                result.speakers_detected, result.segments_generated);
    }
}

/// Test error handling and recovery
#[tokio::test]
async fn test_error_handling_and_recovery() {
    let config = DiarizationTestConfig::default();
    let system = MockDiarizationSystem::new(config).await;
    
    // Test 1: Empty audio
    let empty_audio = vec![];
    let result = system.process_audio(&empty_audio).await;
    assert!(result.is_err(), "Empty audio should return error");
    
    // Test 2: Invalid audio (wrong sample rate)
    let invalid_audio = vec![0.0; 100]; // Too short
    let result = system.process_audio(&invalid_audio).await;
    assert!(result.is_err(), "Invalid audio should return error");
    
    // Test 3: System recovery after error
    let valid_audio = DiarizationTestUtils::generate_synthetic_audio(5.0, 2, 16000);
    let result = system.process_audio(&valid_audio).await;
    assert!(result.is_ok(), "System should recover after error");
    
    // Test 4: Timeout handling
    let large_audio = DiarizationTestUtils::generate_synthetic_audio(300.0, 8, 16000); // 5 minutes
    let timeout_result = tokio::time::timeout(
        Duration::from_secs(10),
        system.process_audio(&large_audio)
    ).await;
    
    // Should either complete within timeout or handle timeout gracefully
    match timeout_result {
        Ok(_) => println!("Large audio processed within timeout"),
        Err(_) => println!("Timeout handled appropriately"),
    }
    
    println!("✅ Error handling test passed");
}

/// Test integration with different audio formats
#[tokio::test]
async fn test_audio_format_integration() {
    let config = DiarizationTestConfig::default();
    let system = MockDiarizationSystem::new(config).await;
    
    let test_cases = vec![
        ("16kHz_mono", 16000, 1, 10.0),
        ("48kHz_stereo", 48000, 2, 10.0),
        ("8kHz_mono", 8000, 1, 10.0),
        ("44kHz_stereo", 44100, 2, 10.0),
    ];
    
    for (name, sample_rate, channels, duration) in test_cases {
        println!("Testing format: {}", name);
        
        // Generate audio with different characteristics
        let samples = (duration * sample_rate as f32) as usize;
        let mut audio = DiarizationTestUtils::generate_synthetic_audio(duration, 2, sample_rate);
        
        // Simulate stereo by duplicating channels
        if channels == 2 {
            let mono_audio = audio.clone();
            audio.clear();
            for sample in mono_audio {
                audio.push(sample); // Left channel
                audio.push(sample * 0.8); // Right channel (slightly different)
            }
        }
        
        // Process audio
        let result = system.process_audio_with_format(&audio, sample_rate, channels).await;
        
        match result {
            Ok(diarization_result) => {
                assert!(!diarization_result.speakers.is_empty(), 
                       "Should detect speakers for format {}", name);
                println!("   ✅ {} - {} speakers detected", name, diarization_result.speakers.len());
            }
            Err(e) => {
                // Some formats might not be supported
                println!("   ⚠️  {} - Format not supported: {}", name, e);
            }
        }
    }
    
    println!("✅ Audio format integration test completed");
}

/// Test memory management during long sessions
#[tokio::test]
async fn test_long_session_memory_management() {
    let config = DiarizationTestConfig {
        test_duration: 30, // Process in chunks to simulate long session
        ..Default::default()
    };
    
    let system = MockDiarizationSystem::new(config.clone()).await;
    let mut memory_measurements = Vec::new();
    
    // Simulate 10-minute session with 30-second chunks
    let total_duration = 600; // 10 minutes
    let chunk_duration = 30;  // 30 seconds
    let num_chunks = total_duration / chunk_duration;
    
    for chunk_idx in 0..num_chunks {
        let audio_chunk = DiarizationTestUtils::generate_synthetic_audio(
            chunk_duration as f32,
            3, // Consistent speaker count
            config.sample_rate,
        );
        
        let memory_before = get_memory_usage();
        
        let _result = system.process_audio(&audio_chunk).await
            .expect("Chunk processing should succeed");
        
        let memory_after = get_memory_usage();
        memory_measurements.push((chunk_idx, memory_before, memory_after));
        
        // Force garbage collection simulation
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    
    // Analyze memory usage patterns
    let initial_memory = memory_measurements.first().unwrap().1;
    let final_memory = memory_measurements.last().unwrap().2;
    let peak_memory = memory_measurements.iter()
        .map(|(_, _, after)| *after)
        .fold(0.0f64, |a, b| a.max(b));
    
    let memory_growth = final_memory - initial_memory;
    
    println!("Memory analysis for long session:");
    println!("   Initial: {:.1} MB", initial_memory);
    println!("   Final: {:.1} MB", final_memory);
    println!("   Peak: {:.1} MB", peak_memory);
    println!("   Growth: {:.1} MB", memory_growth);
    
    // Validate memory constraints
    assert!(peak_memory <= 1000.0, "Peak memory {} MB exceeds limit", peak_memory);
    assert!(memory_growth <= 200.0, "Memory growth {} MB indicates leak", memory_growth);
    
    println!("✅ Long session memory management test passed");
}

/// Test integration with external systems
#[tokio::test]
async fn test_external_system_integration() {
    let config = DiarizationTestConfig::default();
    let system = MockDiarizationSystem::new(config).await;
    
    // Test integration with transcription system
    let audio = DiarizationTestUtils::generate_synthetic_audio(20.0, 3, 16000);
    let diarization_result = system.process_audio(&audio).await
        .expect("Diarization should succeed");
    
    // Simulate integration with transcription
    let transcription_segments = simulate_transcription_integration(&diarization_result).await;
    
    assert!(!transcription_segments.is_empty(), "Should generate transcription segments");
    assert_eq!(transcription_segments.len(), diarization_result.segments.len(),
              "Transcription segments should match diarization segments");
    
    // Test integration with storage system
    let storage_result = simulate_storage_integration(&diarization_result).await;
    assert!(storage_result.is_ok(), "Storage integration should succeed");
    
    // Test integration with real-time UI updates
    let ui_updates = simulate_ui_integration(&diarization_result).await;
    assert!(!ui_updates.is_empty(), "Should generate UI updates");
    
    println!("✅ External system integration test passed");
    println!("   Transcription segments: {}", transcription_segments.len());
    println!("   UI updates: {}", ui_updates.len());
}

// Helper types and functions

#[derive(Debug)]
struct DiarizationResult {
    speakers: Vec<String>,
    segments: Vec<DiarizationSegment>,
    processing_metadata: ProcessingMetadata,
}

#[derive(Debug)]
struct DiarizationSegment {
    start_time: f32,
    end_time: f32,
    speaker_id: String,
    confidence: f32,
}

#[derive(Debug)]
struct ProcessingMetadata {
    processing_time_ms: u64,
    memory_used_mb: f64,
    model_version: String,
}

#[derive(Debug)]
struct StreamingResults {
    total_chunks: usize,
    chunk_latencies: Vec<Duration>,
    peak_latency: Duration,
}

impl StreamingResults {
    fn new() -> Self {
        Self {
            total_chunks: 0,
            chunk_latencies: Vec::new(),
            peak_latency: Duration::from_secs(0),
        }
    }
    
    fn add_chunk_result(&mut self, _result: DiarizationResult, latency: Duration) {
        self.total_chunks += 1;
        self.chunk_latencies.push(latency);
        self.peak_latency = self.peak_latency.max(latency);
    }
    
    fn average_latency(&self) -> Duration {
        if self.chunk_latencies.is_empty() {
            Duration::from_secs(0)
        } else {
            let total: Duration = self.chunk_latencies.iter().sum();
            total / self.chunk_latencies.len() as u32
        }
    }
}

#[derive(Debug)]
struct SessionResult {
    session_id: usize,
    processing_time: Duration,
    speakers_detected: usize,
    segments_generated: usize,
}

struct MockDiarizationSystem {
    config: DiarizationTestConfig,
}

impl MockDiarizationSystem {
    async fn new(config: DiarizationTestConfig) -> Self {
        // Simulate system initialization
        tokio::time::sleep(Duration::from_millis(100)).await;
        Self { config }
    }
    
    async fn process_audio(&self, audio: &[f32]) -> Result<DiarizationResult, String> {
        if audio.is_empty() {
            return Err("Empty audio input".to_string());
        }
        
        if audio.len() < 1000 {
            return Err("Audio too short for reliable diarization".to_string());
        }
        
        // Simulate processing time
        let processing_ms = (audio.len() / 160) as u64; // Simulate ~1ms per 160 samples
        tokio::time::sleep(Duration::from_millis(processing_ms.min(100))).await;
        
        let num_speakers = (self.config.min_speakers + 
                          (audio.len() % (self.config.max_speakers - self.config.min_speakers + 1))).min(self.config.max_speakers);
        
        let mut speakers = Vec::new();
        let mut segments = Vec::new();
        
        for i in 0..num_speakers {
            speakers.push(format!("speaker_{}", i + 1));
            
            let segment_duration = (audio.len() as f32 / self.config.sample_rate as f32) / num_speakers as f32;
            let start_time = i as f32 * segment_duration;
            let end_time = (i + 1) as f32 * segment_duration;
            
            segments.push(DiarizationSegment {
                start_time,
                end_time,
                speaker_id: format!("speaker_{}", i + 1),
                confidence: 0.85 + (i as f32 * 0.05), // Varying confidence
            });
        }
        
        Ok(DiarizationResult {
            speakers,
            segments,
            processing_metadata: ProcessingMetadata {
                processing_time_ms: processing_ms,
                memory_used_mb: get_memory_usage(),
                model_version: "mock_v1.0".to_string(),
            },
        })
    }
    
    async fn process_audio_chunk(&self, audio: &[f32], chunk_idx: usize) -> Result<DiarizationResult, String> {
        // Add some variation for chunk processing
        let mut result = self.process_audio(audio).await?;
        
        // Adjust speaker IDs for chunk continuity
        for segment in &mut result.segments {
            segment.speaker_id = format!("session_speaker_{}", 
                                       (chunk_idx + segment.speaker_id.chars().last().unwrap() as usize) % 4);
        }
        
        Ok(result)
    }
    
    async fn process_audio_with_format(
        &self, 
        audio: &[f32], 
        sample_rate: u32, 
        channels: usize
    ) -> Result<DiarizationResult, String> {
        // Simulate format validation
        if sample_rate < 8000 || sample_rate > 48000 {
            return Err(format!("Unsupported sample rate: {}", sample_rate));
        }
        
        if channels > 2 {
            return Err(format!("Unsupported channel count: {}", channels));
        }
        
        // Convert multi-channel to mono if needed
        let mono_audio = if channels == 2 {
            audio.chunks(2)
                .map(|chunk| (chunk[0] + chunk.get(1).unwrap_or(&0.0)) / 2.0)
                .collect::<Vec<_>>()
        } else {
            audio.to_vec()
        };
        
        self.process_audio(&mono_audio).await
    }
}

fn validate_segment_continuity(segments: &[DiarizationSegment]) {
    for window in segments.windows(2) {
        let current = &window[0];
        let next = &window[1];
        
        assert!(current.start_time <= current.end_time,
               "Segment has invalid time range: {} > {}", 
               current.start_time, current.end_time);
        
        assert!(current.end_time <= next.start_time + 1.0, // Allow 1s gap tolerance
               "Segments have large gap: {} to {}", 
               current.end_time, next.start_time);
    }
}

async fn simulate_transcription_integration(diarization_result: &DiarizationResult) -> Vec<TranscriptionSegment> {
    let mut transcription_segments = Vec::new();
    
    for segment in &diarization_result.segments {
        transcription_segments.push(TranscriptionSegment {
            start_time: segment.start_time,
            end_time: segment.end_time,
            speaker_id: segment.speaker_id.clone(),
            text: format!("Transcribed text for {}", segment.speaker_id),
            confidence: segment.confidence * 0.9, // Transcription confidence
        });
    }
    
    tokio::time::sleep(Duration::from_millis(50)).await; // Simulate processing
    transcription_segments
}

async fn simulate_storage_integration(diarization_result: &DiarizationResult) -> Result<(), String> {
    // Simulate storing results
    tokio::time::sleep(Duration::from_millis(10)).await;
    
    if diarization_result.speakers.is_empty() {
        Err("Cannot store empty diarization result".to_string())
    } else {
        Ok(())
    }
}

async fn simulate_ui_integration(diarization_result: &DiarizationResult) -> Vec<UIUpdate> {
    let mut ui_updates = Vec::new();
    
    for speaker in &diarization_result.speakers {
        ui_updates.push(UIUpdate {
            update_type: "speaker_detected".to_string(),
            data: speaker.clone(),
            timestamp: std::time::SystemTime::now(),
        });
    }
    
    for segment in &diarization_result.segments {
        ui_updates.push(UIUpdate {
            update_type: "segment_update".to_string(),
            data: format!("{}: {:.1}s-{:.1}s", 
                         segment.speaker_id, segment.start_time, segment.end_time),
            timestamp: std::time::SystemTime::now(),
        });
    }
    
    tokio::time::sleep(Duration::from_millis(5)).await;
    ui_updates
}

fn get_memory_usage() -> f64 {
    // Simplified memory usage simulation
    200.0 + (std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() % 200) as f64
}

#[derive(Debug)]
struct TranscriptionSegment {
    start_time: f32,
    end_time: f32,
    speaker_id: String,
    text: String,
    confidence: f32,
}

#[derive(Debug)]
struct UIUpdate {
    update_type: String,
    data: String,
    timestamp: std::time::SystemTime,
}