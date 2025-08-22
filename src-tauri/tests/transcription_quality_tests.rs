use std::collections::VecDeque;
use kaginote_lib::transcription::{ContentHasher, TemporalAnalyzer, BoundaryDetector};
use kaginote_lib::transcription::temporal_analyzer::TemporalSegment;
use kaginote_lib::transcription::boundary_detector::{AudioChunk, BoundaryConfig, BoundaryType};

#[tokio::test]
async fn test_content_hasher_duplicate_detection() {
    let mut hasher = ContentHasher::new(10, 0.6);
    
    // Test exact duplicate detection
    assert!(!hasher.is_duplicate("Hello world, this is a test.", 1.0));
    assert!(hasher.is_duplicate("Hello world, this is a test.", 2.0));
    
    // Test semantic similarity
    assert!(!hasher.is_duplicate("Welcome everyone to our meeting today.", 3.0));
    assert!(hasher.is_duplicate("Welcome everyone to the meeting today.", 4.0));
    
    // Test false positives (different content)
    assert!(!hasher.is_duplicate("Completely different content here.", 5.0));
    assert!(!hasher.is_duplicate("Another unrelated sentence entirely.", 6.0));
}

#[tokio::test]
async fn test_content_hasher_word_rarity_weighting() {
    let hasher = ContentHasher::new(10, 0.6);
    
    // Common words should have lower weights
    assert!(hasher.get_word_rarity_weight("the") < 0.5);
    assert!(hasher.get_word_rarity_weight("and") < 0.5);
    assert!(hasher.get_word_rarity_weight("are") < 0.5);
    
    // Longer, more meaningful words should have higher weights
    assert!(hasher.get_word_rarity_weight("transcription") > 1.0);
    assert!(hasher.get_word_rarity_weight("algorithm") > 1.0);
    assert!(hasher.get_word_rarity_weight("implementation") > 1.0);
}

#[tokio::test]
async fn test_temporal_analyzer_conflict_detection() {
    let mut analyzer = TemporalAnalyzer::new(10, 0.3, 0.1);
    
    // Add a segment
    let seg1 = create_test_segment("Hello everyone", 1.0, 3.0, 0.9, "speaker_1");
    analyzer.add_segment(seg1);
    
    // Test overlapping segment (should detect conflict)
    let seg2 = create_test_segment("Welcome to the meeting", 2.0, 4.0, 0.8, "speaker_2");
    assert!(analyzer.has_temporal_conflict(&seg2));
    
    // Test non-overlapping segment (should not detect conflict)
    let seg3 = create_test_segment("Let's get started", 5.0, 7.0, 0.9, "speaker_1");
    assert!(!analyzer.has_temporal_conflict(&seg3));
}

#[tokio::test]
async fn test_temporal_analyzer_timing_validation() {
    let analyzer = TemporalAnalyzer::new(10, 0.3, 0.1);
    
    // Valid segment
    let valid_seg = create_test_segment("Valid segment", 1.0, 3.0, 0.9, "speaker_1");
    assert!(analyzer.is_valid_timing(&valid_seg));
    
    // Invalid segment (end before start)
    let invalid_seg1 = create_test_segment("Invalid segment", 3.0, 1.0, 0.9, "speaker_1");
    assert!(!analyzer.is_valid_timing(&invalid_seg1));
    
    // Invalid segment (negative start)
    let invalid_seg2 = create_test_segment("Invalid segment", -1.0, 2.0, 0.9, "speaker_1");
    assert!(!analyzer.is_valid_timing(&invalid_seg2));
    
    // Invalid segment (too long)
    let invalid_seg3 = create_test_segment("Too long segment", 1.0, 70.0, 0.9, "speaker_1");
    assert!(!analyzer.is_valid_timing(&invalid_seg3));
}

#[tokio::test]
async fn test_temporal_analyzer_segment_merging() {
    let mut analyzer = TemporalAnalyzer::new(10, 0.3, 0.1);
    
    // Add initial segment
    let seg1 = create_test_segment("Hello", 1.0, 3.0, 0.9, "speaker_1");
    analyzer.add_segment(seg1);
    
    // Add overlapping segment that should be merged
    let seg2 = create_test_segment("world", 2.0, 4.0, 0.8, "speaker_1");
    let merged_segments = analyzer.merge_overlapping_segments(seg2);
    
    assert_eq!(merged_segments.len(), 1);
    let merged = &merged_segments[0];
    assert_eq!(merged.start_time, 1.0);
    assert_eq!(merged.end_time, 4.0);
    assert!(merged.text.contains("Hello") && merged.text.contains("world"));
}

#[tokio::test]
async fn test_boundary_detector_silence_detection() {
    let config = BoundaryConfig {
        silence_threshold: 0.02,
        soft_boundary_ms: 400,
        hard_boundary_ms: 800,
        max_chunks: 50,
        min_speech_duration_ms: 2000,
        energy_variance_threshold: 0.05,
        spectral_analysis_enabled: true,
    };
    let mut detector = BoundaryDetector::new(config);

    let start_time = std::time::SystemTime::now();
    
    // Add speech chunks first
    for i in 0..25 {
        let chunk = create_audio_chunk(0.1, start_time + std::time::Duration::from_millis(i * 100));
        detector.process_chunk(chunk);
    }
    
    // Add silence chunks - should eventually detect boundary
    let mut detected_boundary = BoundaryType::None;
    for i in 25..35 {
        let chunk = create_audio_chunk(0.001, start_time + std::time::Duration::from_millis(i * 100));
        let boundary = detector.process_chunk(chunk);
        if matches!(boundary, BoundaryType::HardBoundary) {
            detected_boundary = boundary;
            break;
        }
    }
    
    assert_eq!(detected_boundary, BoundaryType::HardBoundary);
}

#[tokio::test]
async fn test_boundary_detector_sentence_ending() {
    let mut config = BoundaryConfig::default();
    config.spectral_analysis_enabled = true;
    config.min_speech_duration_ms = 1000; // Reduce for test
    let mut detector = BoundaryDetector::new(config);

    let start_time = std::time::SystemTime::now();
    
    // Add initial speech to meet minimum duration
    for i in 0..15 {
        let chunk = create_audio_chunk(0.1, start_time + std::time::Duration::from_millis(i * 100));
        detector.process_chunk(chunk);
    }
    
    // Create descending energy pattern (sentence ending)
    let energies = [0.2, 0.18, 0.16, 0.14, 0.12, 0.10, 0.08, 0.06, 0.04, 0.02];
    let mut detected_sentence_end = false;
    
    for (i, &energy) in energies.iter().enumerate() {
        let chunk = create_audio_chunk(energy, start_time + std::time::Duration::from_millis((15 + i) as u64 * 100));
        let boundary = detector.process_chunk(chunk);
        
        if matches!(boundary, BoundaryType::SentenceEnd) {
            detected_sentence_end = true;
            break;
        }
    }
    
    assert!(detected_sentence_end, "Should detect sentence ending pattern");
}

#[tokio::test]
async fn test_boundary_detector_buffering_decisions() {
    let config = BoundaryConfig::default();
    let mut detector = BoundaryDetector::new(config);
    
    // Should continue buffering for minimum duration
    assert!(detector.should_continue_buffering(2000)); // 2 seconds < 4.5s minimum
    
    // Should stop buffering after maximum duration
    assert!(!detector.should_continue_buffering(25000)); // 25 seconds > 20s maximum
}

#[tokio::test]
async fn test_integrated_transcription_quality_flow() {
    // Test the complete flow of content hashing, temporal analysis, and boundary detection
    let mut content_hasher = ContentHasher::new(8, 0.6);
    let mut temporal_analyzer = TemporalAnalyzer::new(10, 0.3, 0.1);
    let mut boundary_detector = BoundaryDetector::new(BoundaryConfig::default());
    
    let start_time = std::time::SystemTime::now();
    
    // Simulate a realistic transcription scenario
    let test_transcripts = vec![
        ("Hello everyone, welcome to our meeting today.", 1.0, 4.0, 0.9),
        ("Thank you for joining us this morning.", 5.0, 8.0, 0.85),
        ("Let's start with the agenda for today.", 9.0, 12.0, 0.9),
        ("Hello everyone, welcome to our meeting today.", 13.0, 16.0, 0.8), // Duplicate
        ("First item is the quarterly review.", 17.0, 20.0, 0.88),
    ];
    
    let mut processed_count = 0;
    let mut duplicate_count = 0;
    let mut valid_segments = Vec::new();
    
    for (text, start, end, confidence) in test_transcripts {
        // Test content hashing
        if !content_hasher.is_duplicate(text, start) {
            // Create temporal segment
            let temporal_segment = TemporalSegment {
                text: text.to_string(),
                start_time: start,
                end_time: end,
                confidence,
                speaker_id: "speaker_1".to_string(),
            };
            
            // Test temporal analysis
            if temporal_analyzer.is_valid_timing(&temporal_segment) {
                if !temporal_analyzer.has_temporal_conflict(&temporal_segment) {
                    temporal_analyzer.add_segment(temporal_segment.clone());
                    valid_segments.push(temporal_segment);
                    processed_count += 1;
                }
            }
        } else {
            duplicate_count += 1;
        }
        
        // Test boundary detection
        let chunk = create_audio_chunk(0.05, start_time + std::time::Duration::from_secs(start as u64));
        let boundary_type = boundary_detector.process_chunk(chunk);
        
        // Boundary detection should provide meaningful feedback
        assert!(matches!(boundary_type, BoundaryType::None | BoundaryType::SoftBoundary | BoundaryType::HardBoundary | BoundaryType::SentenceEnd));
    }
    
    // Verify that duplicates were properly detected
    assert_eq!(duplicate_count, 1, "Should detect exactly one duplicate");
    assert_eq!(processed_count, 4, "Should process 4 unique segments");
    assert_eq!(valid_segments.len(), 4, "Should have 4 valid segments");
    
    // Test statistics
    let (hasher_cache_size, hasher_vocab_size) = content_hasher.get_cache_stats();
    assert!(hasher_cache_size > 0, "Content hasher should have cached segments");
    assert!(hasher_vocab_size > 0, "Content hasher should have vocabulary");
    
    let (temporal_count, temporal_duration, temporal_confidence) = temporal_analyzer.get_stats();
    assert_eq!(temporal_count, 4, "Temporal analyzer should track 4 segments");
    assert!(temporal_duration > 0.0, "Total duration should be positive");
    assert!(temporal_confidence > 0.7, "Average confidence should be reasonable");
    
    let (speech_chunks, silence_chunks, avg_energy) = boundary_detector.get_stats();
    assert!(avg_energy > 0.0, "Average energy should be positive");
}

#[tokio::test]
async fn test_performance_and_memory_efficiency() {
    // Test that the quality systems don't consume excessive memory
    let mut content_hasher = ContentHasher::new(1000, 0.6); // Large cache
    let mut temporal_analyzer = TemporalAnalyzer::new(1000, 0.3, 0.1); // Large capacity
    let mut boundary_detector = BoundaryDetector::new(BoundaryConfig::default());
    
    let start_time = std::time::SystemTime::now();
    
    // Process many segments to test memory management
    for i in 0..2000 {
        let text = format!("This is test segment number {} with unique content.", i);
        let start = i as f32;
        let end = start + 2.0;
        
        // Test content hasher memory management
        let is_duplicate = content_hasher.is_duplicate(&text, start);
        assert!(!is_duplicate, "Unique segments should not be duplicates");
        
        // Test temporal analyzer memory management
        let segment = create_test_segment(&text, start, end, 0.9, "speaker_1");
        if !temporal_analyzer.has_temporal_conflict(&segment) {
            temporal_analyzer.add_segment(segment);
        }
        
        // Test boundary detector memory management
        let chunk = create_audio_chunk(0.05 + (i as f32 % 10.0) / 100.0, 
                                      start_time + std::time::Duration::from_secs(i));
        boundary_detector.process_chunk(chunk);
        
        // Check memory usage periodically
        if i % 500 == 0 {
            let (cache_size, vocab_size) = content_hasher.get_cache_stats();
            assert!(cache_size <= 1000, "Content hasher cache should not exceed limit");
            
            let (segment_count, _, _) = temporal_analyzer.get_stats();
            assert!(segment_count <= 1000, "Temporal analyzer should not exceed capacity");
            
            // Memory should be bounded
            assert!(vocab_size < 50000, "Vocabulary should not grow unbounded");
        }
    }
    
    let processing_time = start_time.elapsed().unwrap();
    println!("Processed 2000 segments in {:?}", processing_time);
    
    // Should complete in reasonable time (under 1 second for 2000 segments)
    assert!(processing_time.as_millis() < 1000, "Processing should be efficient");
}

// Helper functions for test setup

fn create_test_segment(text: &str, start: f32, end: f32, confidence: f32, speaker: &str) -> TemporalSegment {
    TemporalSegment {
        text: text.to_string(),
        start_time: start,
        end_time: end,
        confidence,
        speaker_id: speaker.to_string(),
    }
}

fn create_audio_chunk(energy: f32, timestamp: std::time::SystemTime) -> AudioChunk {
    AudioChunk {
        samples: vec![energy; 1600], // Simulated 100ms at 16kHz
        sample_rate: 16000,
        timestamp,
        energy_level: energy,
    }
}