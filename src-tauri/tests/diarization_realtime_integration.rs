use std::path::PathBuf;

mod diarization_realtime;
use diarization_realtime::{DiarizationTestRunner, TestScenarioGenerator};
use diarization_realtime::test_scenarios::*;

/// Integration tests for the complete diarization testing infrastructure
/// These tests validate that our test framework works correctly

#[test]
fn test_complete_testing_infrastructure() {
    let runner = DiarizationTestRunner::new();
    
    // Test that we can create the test runner
    assert!(runner.scenarios_path.to_string_lossy().contains("ground_truth"));
    assert!(runner.audio_output_path.to_string_lossy().contains("generated_audio"));
    
    println!("✅ Test runner created successfully");
}

#[test]
fn test_ground_truth_data_structures() {
    // Test all scenario generators work
    let scenarios = vec![
        ("simple_turn_taking", TestScenarioGenerator::simple_two_speaker_conversation()),
        ("multi_speaker_meeting", TestScenarioGenerator::multi_speaker_meeting()),
        ("overlapping_speech", TestScenarioGenerator::overlapping_speech_scenario()),
        ("rapid_switching", TestScenarioGenerator::rapid_speaker_switching()),
        ("long_silences", TestScenarioGenerator::long_silences_scenario()),
        ("single_speaker", TestScenarioGenerator::single_speaker_monologue()),
    ];
    
    for (name, scenario) in scenarios {
        // Validate basic structure
        assert!(scenario.duration > 0.0, "Scenario {} should have positive duration", name);
        assert!(scenario.total_speakers > 0, "Scenario {} should have speakers", name);
        assert!(!scenario.segments.is_empty(), "Scenario {} should have segments", name);
        
        // Validate metadata
        assert!(scenario.metadata.contains_key("scenario_type"), "Scenario {} should have scenario_type", name);
        
        // Validate segments are in chronological order
        for window in scenario.segments.windows(2) {
            assert!(window[0].start_time <= window[1].start_time, 
                "Segments in {} should be chronologically ordered", name);
        }
        
        // Validate segment timing makes sense
        for segment in &scenario.segments {
            assert!(segment.start_time >= 0.0, "Start time should be non-negative in {}", name);
            assert!(segment.end_time > segment.start_time, "End time should be after start time in {}", name);
            assert!(segment.end_time <= scenario.duration, "Segment should not exceed total duration in {}", name);
            assert!(segment.confidence >= 0.0 && segment.confidence <= 1.0, "Confidence should be 0-1 in {}", name);
        }
        
        println!("✅ Scenario {} validated: {} speakers, {:.1}s duration, {} segments", 
                 name, scenario.total_speakers, scenario.duration, scenario.segments.len());
    }
}

#[test]
fn test_synthetic_audio_generation() {
    let scenario = TestScenarioGenerator::simple_two_speaker_conversation();
    let sample_rate = 16000;
    
    // Generate synthetic audio
    let audio_data = SyntheticAudioGenerator::generate_multi_frequency_audio(&scenario, sample_rate);
    
    // Validate audio properties
    let expected_samples = (scenario.duration * sample_rate as f32) as usize;
    assert_eq!(audio_data.len(), expected_samples, "Audio should have correct length");
    
    // Check that audio is generated where segments exist
    let first_segment = &scenario.segments[0];
    let start_sample = (first_segment.start_time * sample_rate as f32) as usize;
    let end_sample = (first_segment.end_time * sample_rate as f32) as usize;
    
    let segment_audio: Vec<f32> = audio_data[start_sample..end_sample.min(audio_data.len())].to_vec();
    let max_amplitude = segment_audio.iter().map(|x| x.abs()).fold(0.0f32, f32::max);
    
    assert!(max_amplitude > 0.01, "Audio should have signal in active segments");
    assert!(max_amplitude <= 1.0, "Audio should not clip");
    
    println!("✅ Audio generation: {} samples, max amplitude: {:.3}", audio_data.len(), max_amplitude);
}

#[test]
fn test_audio_noise_and_filtering() {
    let scenario = TestScenarioGenerator::simple_two_speaker_conversation();
    let mut audio_data = SyntheticAudioGenerator::generate_multi_frequency_audio(&scenario, 16000);
    
    let original_max = audio_data.iter().map(|x| x.abs()).fold(0.0f32, f32::max);
    
    // Add background noise
    SyntheticAudioGenerator::add_background_noise(&mut audio_data, 0.05);
    
    let noise_max = audio_data.iter().map(|x| x.abs()).fold(0.0f32, f32::max);
    assert!(noise_max >= original_max, "Noise should increase amplitude");
    
    // Apply voice filter
    SyntheticAudioGenerator::apply_voice_filter(&mut audio_data);
    
    let filtered_max = audio_data.iter().map(|x| x.abs()).fold(0.0f32, f32::max);
    assert!(filtered_max <= 1.0, "Filtered audio should not clip");
    
    println!("✅ Audio processing: original {:.3}, with noise {:.3}, filtered {:.3}", 
             original_max, noise_max, filtered_max);
}

#[test]
fn test_scenario_validation_logic() {
    let ground_truth = TestScenarioGenerator::simple_two_speaker_conversation();
    
    // Create perfect detection (should score highly)
    let perfect_segments = ground_truth.segments.iter().map(|segment| {
        DetectedSegment {
            speaker_id: segment.speaker_id.clone(),
            start_time: segment.start_time,
            end_time: segment.end_time,
            confidence: segment.confidence,
        }
    }).collect::<Vec<_>>();
    
    let perfect_validation = ScenarioValidator::validate_speaker_detection(
        &perfect_segments,
        &ground_truth,
        0.1, // 100ms tolerance
    );
    
    assert!(perfect_validation.accuracy >= 0.8, "Perfect detection should have high accuracy");
    assert!(perfect_validation.temporal_alignment_score >= 0.8, "Perfect detection should have high temporal alignment");
    
    // Create poor detection (should score poorly)
    let poor_segments = vec![
        DetectedSegment {
            speaker_id: "wrong_speaker".to_string(),
            start_time: 100.0, // Way off timing
            end_time: 105.0,
            confidence: 0.5,
        }
    ];
    
    let poor_validation = ScenarioValidator::validate_speaker_detection(
        &poor_segments,
        &ground_truth,
        0.1,
    );
    
    assert!(poor_validation.accuracy < perfect_validation.accuracy, "Poor detection should have lower accuracy");
    
    println!("✅ Validation logic: perfect accuracy {:.2}, poor accuracy {:.2}", 
             perfect_validation.accuracy, poor_validation.accuracy);
}

#[test]
fn test_overlapping_speech_detection() {
    let scenario = TestScenarioGenerator::overlapping_speech_scenario();
    
    // Verify overlaps are detected correctly
    assert!(scenario.has_overlaps(), "Overlapping speech scenario should detect overlaps");
    
    // Count actual overlaps
    let mut overlap_count = 0;
    for (i, segment1) in scenario.segments.iter().enumerate() {
        for segment2 in scenario.segments.iter().skip(i + 1) {
            if segment1.overlaps_with(segment2) {
                overlap_count += 1;
                println!("Overlap detected: {} ({:.1}-{:.1}) overlaps with {} ({:.1}-{:.1})", 
                         segment1.speaker_id, segment1.start_time, segment1.end_time,
                         segment2.speaker_id, segment2.start_time, segment2.end_time);
            }
        }
    }
    
    assert!(overlap_count > 0, "Should detect actual overlaps");
    println!("✅ Overlap detection: {} overlapping pairs found", overlap_count);
}

#[test]
fn test_speaker_consistency_validation() {
    // Test consistent speaker labeling (should pass)
    let consistent_segments = vec![
        DetectedSegment {
            speaker_id: "speaker_A".to_string(),
            start_time: 0.0,
            end_time: 5.0,
            confidence: 0.9,
        },
        DetectedSegment {
            speaker_id: "speaker_A".to_string(),
            start_time: 10.0,
            end_time: 15.0,
            confidence: 0.8,
        },
    ];
    
    assert!(ScenarioValidator::validate_speaker_consistency(&consistent_segments, 0.8), 
            "Consistent labeling should pass validation");
    
    // Test inconsistent speaker labeling (should fail)
    let inconsistent_segments = vec![
        DetectedSegment {
            speaker_id: "speaker_A".to_string(),
            start_time: 0.0,
            end_time: 5.0,
            confidence: 0.9,
        },
        DetectedSegment {
            speaker_id: "speaker_A".to_string(),
            start_time: 100.0, // Large gap suggests inconsistent labeling
            end_time: 105.0,
            confidence: 0.8,
        },
    ];
    
    assert!(!ScenarioValidator::validate_speaker_consistency(&inconsistent_segments, 0.8), 
            "Inconsistent labeling should fail validation");
    
    println!("✅ Speaker consistency validation working correctly");
}

#[test]
fn test_rapid_switching_scenario_properties() {
    let scenario = TestScenarioGenerator::rapid_speaker_switching();
    
    // Verify rapid switching properties
    let avg_duration: f32 = scenario.segments.iter()
        .map(|s| s.duration())
        .sum::<f32>() / scenario.segments.len() as f32;
    
    assert!(avg_duration < 2.5, "Rapid switching should have short average segment duration");
    assert!(scenario.segments.len() >= 8, "Rapid switching should have many segments");
    
    // Check that speakers alternate reasonably
    let mut speaker_changes = 0;
    for window in scenario.segments.windows(2) {
        if window[0].speaker_id != window[1].speaker_id {
            speaker_changes += 1;
        }
    }
    
    let change_rate = speaker_changes as f32 / (scenario.segments.len() - 1) as f32;
    assert!(change_rate > 0.5, "Rapid switching should have frequent speaker changes");
    
    println!("✅ Rapid switching: avg {:.1}s segments, {:.0}% change rate", avg_duration, change_rate * 100.0);
}

#[test]
fn test_ground_truth_json_serialization() {
    let runner = DiarizationTestRunner::new();
    
    // Test that we can load the pre-created JSON files
    let json_files = vec![
        "example_2speakers.json",
        "example_3speakers_meeting.json", 
        "example_overlapping_speech.json",
        "example_rapid_switching.json",
    ];
    
    for json_file in json_files {
        // This will fail initially until implementation is complete
        match runner.load_ground_truth(json_file) {
            Ok(ground_truth) => {
                assert!(ground_truth.total_speakers > 0, "Loaded ground truth should have speakers");
                assert!(ground_truth.duration > 0.0, "Loaded ground truth should have duration");
                assert!(!ground_truth.segments.is_empty(), "Loaded ground truth should have segments");
                
                println!("✅ Loaded {}: {} speakers, {:.1}s", json_file, ground_truth.total_speakers, ground_truth.duration);
            },
            Err(e) => {
                // Expected to fail initially in TDD
                panic!("Failed to load {} (expected in TDD): {}", json_file, e);
            }
        }
    }
}

#[test]
fn test_complete_workflow_integration() {
    // This test demonstrates the complete workflow from scenario generation to validation
    let runner = DiarizationTestRunner::new();
    
    // Step 1: Generate scenario
    let ground_truth = TestScenarioGenerator::simple_two_speaker_conversation();
    println!("📋 Generated scenario: {} speakers, {:.1}s duration", 
             ground_truth.total_speakers, ground_truth.duration);
    
    // Step 2: Generate synthetic audio
    let audio_result = runner.generate_test_audio(&ground_truth, 16000);
    
    match audio_result {
        Ok(audio_path) => {
            println!("🎵 Generated audio: {:?}", audio_path);
            
            // Step 3: Simulate diarization results (in real implementation, this would come from the actual diarization engine)
            let simulated_results = vec![
                DetectedSegment {
                    speaker_id: "detected_0".to_string(),
                    start_time: 0.1,
                    end_time: 4.9,
                    confidence: 0.92,
                },
                DetectedSegment {
                    speaker_id: "detected_1".to_string(),
                    start_time: 5.6,
                    end_time: 9.9,
                    confidence: 0.88,
                },
            ];
            
            // Step 4: Validate results
            let validation = ScenarioValidator::validate_speaker_detection(
                &simulated_results,
                &ground_truth,
                0.5, // 500ms tolerance
            );
            
            println!("📊 Validation results:");
            println!("   Accuracy: {:.2}", validation.accuracy);
            println!("   Temporal alignment: {:.2}", validation.temporal_alignment_score);
            println!("   Correct detections: {}/{}", validation.correct_detections, validation.total_detections);
            
            // Step 5: Check quality thresholds
            let meets_quality = validation.meets_quality_threshold(0.7, 0.6);
            println!("   Meets quality threshold: {}", meets_quality);
            
            println!("✅ Complete workflow tested successfully");
        },
        Err(e) => {
            // Expected to fail initially in TDD - this drives implementation
            panic!("Complete workflow failed (expected in TDD): {}", e);
        }
    }
}

/// Performance benchmark test for the testing infrastructure itself
#[test]
fn test_performance_benchmarks() {
    use std::time::Instant;
    
    // Benchmark scenario generation
    let start = Instant::now();
    let scenario = TestScenarioGenerator::simple_two_speaker_conversation();
    let generation_time = start.elapsed();
    
    // Benchmark audio synthesis
    let start = Instant::now();
    let audio_data = SyntheticAudioGenerator::generate_multi_frequency_audio(&scenario, 16000);
    let synthesis_time = start.elapsed();
    
    // Benchmark validation
    let detected_segments = vec![
        DetectedSegment {
            speaker_id: "test_speaker".to_string(),
            start_time: 0.0,
            end_time: 5.0,
            confidence: 0.9,
        }
    ];
    
    let start = Instant::now();
    let _validation = ScenarioValidator::validate_speaker_detection(&detected_segments, &scenario, 0.5);
    let validation_time = start.elapsed();
    
    // Performance assertions (these define our performance requirements)
    assert!(generation_time.as_millis() < 100, "Scenario generation should be fast (<100ms)");
    assert!(synthesis_time.as_millis() < 1000, "Audio synthesis should be reasonable (<1s)");
    assert!(validation_time.as_millis() < 50, "Validation should be very fast (<50ms)");
    
    println!("🚀 Performance benchmarks:");
    println!("   Scenario generation: {:?}", generation_time);
    println!("   Audio synthesis: {:?} for {} samples", synthesis_time, audio_data.len());
    println!("   Validation: {:?}", validation_time);
    
    println!("✅ All performance benchmarks passed");
}