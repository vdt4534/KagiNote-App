//! Integration tests for the diarization validation framework
//! 
//! This module demonstrates how to use the validation framework to measure
//! diarization accuracy and performance using ground truth data.

use crate::diarization::types::{SpeakerSegment, SpeakerEmbedding};
use crate::diarization_realtime::validation::{
    DiarizationValidator, ValidationConfig, GroundTruthData, GroundTruthSegment,
    load_ground_truth, save_ground_truth, AccuracyLevel, PerformanceLevel,
};
use std::collections::HashMap;
use std::path::Path;

/// Test the complete validation workflow with ground truth data
#[tokio::test]
async fn test_complete_validation_workflow() {
    // Load ground truth data
    let ground_truth_path = "tests/diarization_realtime/ground_truth/example_meeting.json";
    let ground_truth_data = load_ground_truth(ground_truth_path)
        .expect("Should load ground truth data");
    
    // Create predicted segments that match the ground truth (perfect scenario)
    let predicted_segments = ground_truth_data.segments.iter().map(|gt| {
        SpeakerSegment {
            speaker_id: gt.speaker_id.clone(),
            start_time: gt.start_time,
            end_time: gt.end_time,
            confidence: 0.9,
            text: gt.text.clone(),
            embedding: None,
            has_overlap: false,
            overlapping_speakers: vec![],
        }
    }).collect();
    
    // Create validator with custom configuration
    let config = ValidationConfig {
        time_tolerance_s: 0.25,
        target_der_threshold: 0.10,
        target_rt_factor: 1.0,
        min_consistency_score: 0.90,
        evaluate_overlaps: true,
        generate_reports: true,
        report_output_dir: "tests/diarization_realtime/reports".to_string(),
    };
    
    let mut validator = DiarizationValidator::with_config(config);
    
    // Run validation
    let result = validator.compare_segments(
        predicted_segments,
        ground_truth_data.segments,
        250.0,
    ).expect("Validation should succeed");
    
    // Verify perfect diarization results
    assert!(result.der_result.der_score < 0.05, "DER should be very low for perfect match");
    assert!(result.der_result.precision > 0.95, "Precision should be very high");
    assert!(result.der_result.recall > 0.95, "Recall should be very high");
    assert!(result.der_result.f1_score > 0.95, "F1 score should be very high");
    assert_eq!(result.summary.accuracy_level, AccuracyLevel::Excellent);
    assert!(result.summary.meets_targets, "Should meet all target thresholds");
    assert!(result.summary.overall_quality > 0.9, "Overall quality should be excellent");
    
    println!("Perfect diarization validation results:");
    println!("  DER: {:.3}%", result.der_result.der_score * 100.0);
    println!("  Precision: {:.3}", result.der_result.precision);
    println!("  Recall: {:.3}", result.der_result.recall);
    println!("  F1 Score: {:.3}", result.der_result.f1_score);
    println!("  Overall Quality: {:.1}%", result.summary.overall_quality * 100.0);
}

/// Test validation with speaker confusion errors
#[tokio::test]
async fn test_speaker_confusion_validation() {
    let ground_truth_path = "tests/diarization_realtime/ground_truth/example_meeting.json";
    let ground_truth_data = load_ground_truth(ground_truth_path)
        .expect("Should load ground truth data");
    
    // Create predicted segments with speaker IDs swapped
    let mut predicted_segments: Vec<SpeakerSegment> = ground_truth_data.segments.iter().map(|gt| {
        SpeakerSegment {
            speaker_id: gt.speaker_id.clone(),
            start_time: gt.start_time,
            end_time: gt.end_time,
            confidence: 0.8,
            text: gt.text.clone(),
            embedding: None,
            has_overlap: false,
            overlapping_speakers: vec![],
        }
    }).collect();
    
    // Swap speaker IDs to simulate confusion
    for segment in &mut predicted_segments {
        segment.speaker_id = match segment.speaker_id.as_str() {
            "speaker_alice" => "speaker_bob".to_string(),
            "speaker_bob" => "speaker_charlie".to_string(),
            "speaker_charlie" => "speaker_alice".to_string(),
            _ => segment.speaker_id.clone(),
        };
    }
    
    let mut validator = DiarizationValidator::new();
    
    let result = validator.compare_segments(
        predicted_segments,
        ground_truth_data.segments,
        250.0,
    ).expect("Validation should succeed");
    
    // Verify high speaker error rate
    assert!(result.der_result.speaker_error_rate > 0.8, "Should have high speaker error rate");
    assert!(result.der_result.der_score > 0.8, "DER should be high due to confusion");
    assert!(result.consistency_result.id_switches > 0, "Should detect speaker switches");
    assert!(!result.summary.meets_targets, "Should not meet targets with confusion");
    assert_eq!(result.summary.accuracy_level, AccuracyLevel::Poor);
    
    println!("Speaker confusion validation results:");
    println!("  DER: {:.1}%", result.der_result.der_score * 100.0);
    println!("  Speaker Error Rate: {:.1}%", result.der_result.speaker_error_rate * 100.0);
    println!("  ID Switches: {}", result.consistency_result.id_switches);
    println!("  Consistency: {:.1}%", result.consistency_result.consistency_percentage);
}

/// Test validation with overlapping speech detection
#[tokio::test] 
async fn test_overlap_detection_validation() {
    let ground_truth_path = "tests/diarization_realtime/ground_truth/challenging_meeting.json";
    let ground_truth_data = load_ground_truth(ground_truth_path)
        .expect("Should load challenging ground truth data");
    
    // Create predicted segments with some overlaps detected
    let mut predicted_segments: Vec<SpeakerSegment> = ground_truth_data.segments.iter().map(|gt| {
        SpeakerSegment {
            speaker_id: gt.speaker_id.clone(),
            start_time: gt.start_time,
            end_time: gt.end_time,
            confidence: 0.85,
            text: gt.text.clone(),
            embedding: None,
            has_overlap: false,
            overlapping_speakers: vec![],
        }
    }).collect();
    
    // Mark some segments as having overlaps based on metadata
    if let Some(overlaps) = ground_truth_data.metadata.get("overlaps_detected") {
        // Simulate overlap detection by marking segments that have timing overlaps
        for i in 0..predicted_segments.len() {
            for j in i+1..predicted_segments.len() {
                let seg1 = &predicted_segments[i];
                let seg2 = &predicted_segments[j];
                
                let overlap_start = seg1.start_time.max(seg2.start_time);
                let overlap_end = seg1.end_time.min(seg2.end_time);
                
                if overlap_end > overlap_start + 0.2 { // 200ms overlap threshold
                    predicted_segments[i].has_overlap = true;
                    predicted_segments[i].overlapping_speakers.push(seg2.speaker_id.clone());
                    predicted_segments[j].has_overlap = true;
                    predicted_segments[j].overlapping_speakers.push(seg1.speaker_id.clone());
                }
            }
        }
    }
    
    let config = ValidationConfig {
        evaluate_overlaps: true,
        time_tolerance_s: 0.3, // More lenient for overlapping speech
        ..ValidationConfig::default()
    };
    
    let mut validator = DiarizationValidator::with_config(config);
    
    let result = validator.compare_segments(
        predicted_segments,
        ground_truth_data.segments,
        300.0,
    ).expect("Validation should succeed");
    
    // Check overlap accuracy
    assert!(result.der_result.overlap_accuracy >= 0.5, "Should detect some overlaps correctly");
    println!("Overlap detection validation results:");
    println!("  Overlap Accuracy: {:.1}%", result.der_result.overlap_accuracy * 100.0);
    println!("  DER with overlaps: {:.1}%", result.der_result.der_score * 100.0);
}

/// Test validation with missing segments (missed speech)
#[tokio::test]
async fn test_missed_speech_validation() {
    let ground_truth_path = "tests/diarization_realtime/ground_truth/example_meeting.json";
    let ground_truth_data = load_ground_truth(ground_truth_path)
        .expect("Should load ground truth data");
    
    // Create predicted segments but remove every third segment to simulate missed speech
    let mut predicted_segments: Vec<SpeakerSegment> = ground_truth_data.segments.iter()
        .enumerate()
        .filter(|(i, _)| i % 3 != 0) // Remove every third segment
        .map(|(_, gt)| {
            SpeakerSegment {
                speaker_id: gt.speaker_id.clone(),
                start_time: gt.start_time,
                end_time: gt.end_time,
                confidence: 0.9,
                text: gt.text.clone(),
                embedding: None,
                has_overlap: false,
                overlapping_speakers: vec![],
            }
        })
        .collect();
    
    let mut validator = DiarizationValidator::new();
    
    let result = validator.compare_segments(
        predicted_segments,
        ground_truth_data.segments,
        250.0,
    ).expect("Validation should succeed");
    
    // Should have high miss rate and low recall
    assert!(result.der_result.miss_rate > 0.2, "Should have significant miss rate");
    assert!(result.der_result.recall < 0.8, "Recall should be reduced due to missed segments");
    assert!(result.der_result.der_score > 0.2, "DER should be elevated due to misses");
    
    println!("Missed speech validation results:");
    println!("  Miss Rate: {:.1}%", result.der_result.miss_rate * 100.0);
    println!("  Recall: {:.3}", result.der_result.recall);
    println!("  DER: {:.1}%", result.der_result.der_score * 100.0);
}

/// Test validation with false alarms (extra segments)
#[tokio::test] 
async fn test_false_alarm_validation() {
    let ground_truth_path = "tests/diarization_realtime/ground_truth/example_meeting.json";
    let ground_truth_data = load_ground_truth(ground_truth_path)
        .expect("Should load ground truth data");
    
    // Create predicted segments with additional false alarm segments
    let mut predicted_segments: Vec<SpeakerSegment> = ground_truth_data.segments.iter().map(|gt| {
        SpeakerSegment {
            speaker_id: gt.speaker_id.clone(),
            start_time: gt.start_time,
            end_time: gt.end_time,
            confidence: 0.9,
            text: gt.text.clone(),
            embedding: None,
            has_overlap: false,
            overlapping_speakers: vec![],
        }
    }).collect();
    
    // Add false alarm segments
    predicted_segments.extend(vec![
        SpeakerSegment {
            speaker_id: "speaker_noise".to_string(),
            start_time: 36.0,
            end_time: 38.5,
            confidence: 0.6,
            text: Some("Background noise classified as speech".to_string()),
            embedding: None,
            has_overlap: false,
            overlapping_speakers: vec![],
        },
        SpeakerSegment {
            speaker_id: "speaker_false".to_string(),
            start_time: 39.0,
            end_time: 41.2,
            confidence: 0.5,
            text: Some("False detection of speech".to_string()),
            embedding: None,
            has_overlap: false,
            overlapping_speakers: vec![],
        },
    ]);
    
    let mut validator = DiarizationValidator::new();
    
    let result = validator.compare_segments(
        predicted_segments,
        ground_truth_data.segments,
        250.0,
    ).expect("Validation should succeed");
    
    // Should have false alarm rate and reduced precision
    assert!(result.der_result.false_alarm_rate > 0.1, "Should have false alarm rate");
    assert!(result.der_result.precision < 0.9, "Precision should be reduced due to false alarms");
    assert!(result.der_result.der_score > 0.1, "DER should be elevated due to false alarms");
    
    println!("False alarm validation results:");
    println!("  False Alarm Rate: {:.1}%", result.der_result.false_alarm_rate * 100.0);
    println!("  Precision: {:.3}", result.der_result.precision);
    println!("  DER: {:.1}%", result.der_result.der_score * 100.0);
}

/// Test performance metrics collection
#[tokio::test]
async fn test_performance_metrics_validation() {
    let ground_truth_path = "tests/diarization_realtime/ground_truth/example_meeting.json";
    let ground_truth_data = load_ground_truth(ground_truth_path)
        .expect("Should load ground truth data");
    
    let predicted_segments: Vec<SpeakerSegment> = ground_truth_data.segments.iter().map(|gt| {
        SpeakerSegment {
            speaker_id: gt.speaker_id.clone(),
            start_time: gt.start_time,
            end_time: gt.end_time,
            confidence: 0.9,
            text: gt.text.clone(),
            embedding: None,
            has_overlap: false,
            overlapping_speakers: vec![],
        }
    }).collect();
    
    let config = ValidationConfig {
        target_rt_factor: 0.5, // Very strict performance target
        generate_reports: false, // Skip report generation for this test
        ..ValidationConfig::default()
    };
    
    let mut validator = DiarizationValidator::with_config(config);
    
    let result = validator.compare_segments(
        predicted_segments,
        ground_truth_data.segments,
        250.0,
    ).expect("Validation should succeed");
    
    // Check performance metrics are collected
    assert!(result.performance_metrics.real_time_factor > 0.0, "Should have real-time factor");
    assert!(result.performance_metrics.peak_memory_mb > 0.0, "Should track memory usage");
    assert!(result.performance_metrics.latency_ms > 0, "Should measure latency");
    
    // Performance level should be determined based on real-time factor
    let expected_performance_level = match result.performance_metrics.real_time_factor {
        x if x < 0.5 => PerformanceLevel::Excellent,
        x if x < 1.0 => PerformanceLevel::Good,
        x if x < 2.0 => PerformanceLevel::Fair,
        _ => PerformanceLevel::Poor,
    };
    
    assert_eq!(result.summary.performance_level, expected_performance_level);
    
    println!("Performance metrics validation results:");
    println!("  Real-time Factor: {:.2}x", result.performance_metrics.real_time_factor);
    println!("  Peak Memory: {:.1} MB", result.performance_metrics.peak_memory_mb);
    println!("  Latency: {} ms", result.performance_metrics.latency_ms);
    println!("  Performance Level: {:?}", result.summary.performance_level);
}

/// Test ground truth data serialization and loading
#[tokio::test]
async fn test_ground_truth_serialization() {
    // Create test ground truth data
    let original_data = GroundTruthData {
        recording_id: "test_serialization_001".to_string(),
        segments: vec![
            GroundTruthSegment {
                speaker_id: "test_speaker_1".to_string(),
                start_time: 0.0,
                end_time: 5.0,
                text: Some("Test segment 1".to_string()),
                audio_file: Some("test_audio_1.wav".to_string()),
                quality: 1.0,
            },
            GroundTruthSegment {
                speaker_id: "test_speaker_2".to_string(),
                start_time: 5.5,
                end_time: 10.0,
                text: Some("Test segment 2".to_string()),
                audio_file: Some("test_audio_2.wav".to_string()),
                quality: 0.95,
            },
        ],
        total_speakers: 2,
        duration: 10.0,
        sample_rate: 16000,
        metadata: {
            let mut map = HashMap::new();
            map.insert("test_key".to_string(), "test_value".to_string());
            map.insert("quality".to_string(), "high".to_string());
            map
        },
    };
    
    // Save to temporary file
    let temp_path = "tests/diarization_realtime/ground_truth/temp_test.json";
    save_ground_truth(&original_data, temp_path)
        .expect("Should save ground truth data");
    
    // Load back from file
    let loaded_data = load_ground_truth(temp_path)
        .expect("Should load ground truth data");
    
    // Verify data integrity
    assert_eq!(original_data.recording_id, loaded_data.recording_id);
    assert_eq!(original_data.segments.len(), loaded_data.segments.len());
    assert_eq!(original_data.total_speakers, loaded_data.total_speakers);
    assert_eq!(original_data.duration, loaded_data.duration);
    assert_eq!(original_data.sample_rate, loaded_data.sample_rate);
    
    // Verify first segment
    let orig_seg = &original_data.segments[0];
    let loaded_seg = &loaded_data.segments[0];
    assert_eq!(orig_seg.speaker_id, loaded_seg.speaker_id);
    assert_eq!(orig_seg.start_time, loaded_seg.start_time);
    assert_eq!(orig_seg.end_time, loaded_seg.end_time);
    assert_eq!(orig_seg.text, loaded_seg.text);
    assert_eq!(orig_seg.quality, loaded_seg.quality);
    
    // Clean up temp file
    std::fs::remove_file(temp_path).ok();
    
    println!("Ground truth serialization test passed");
}

/// Test validation configuration and thresholds
#[tokio::test]
async fn test_validation_configuration() {
    let ground_truth_path = "tests/diarization_realtime/ground_truth/example_meeting.json";
    let ground_truth_data = load_ground_truth(ground_truth_path)
        .expect("Should load ground truth data");
    
    let predicted_segments: Vec<SpeakerSegment> = ground_truth_data.segments.iter().map(|gt| {
        SpeakerSegment {
            speaker_id: gt.speaker_id.clone(),
            start_time: gt.start_time + 0.1, // Small time offset
            end_time: gt.end_time + 0.1,
            confidence: 0.9,
            text: gt.text.clone(),
            embedding: None,
            has_overlap: false,
            overlapping_speakers: vec![],
        }
    }).collect();
    
    // Test with strict configuration
    let strict_config = ValidationConfig {
        time_tolerance_s: 0.05, // Very strict timing
        target_der_threshold: 0.05, // Very low DER target
        target_rt_factor: 0.8, // Strict performance target
        min_consistency_score: 0.95, // High consistency requirement
        evaluate_overlaps: true,
        generate_reports: false,
        report_output_dir: "test_reports".to_string(),
    };
    
    let mut strict_validator = DiarizationValidator::with_config(strict_config);
    
    let strict_result = strict_validator.compare_segments(
        predicted_segments.clone(),
        ground_truth_data.segments.clone(),
        50.0, // 50ms tolerance matching config
    ).expect("Validation should succeed");
    
    // Test with lenient configuration
    let lenient_config = ValidationConfig {
        time_tolerance_s: 0.5, // Very lenient timing
        target_der_threshold: 0.30, // High DER tolerance
        target_rt_factor: 2.0, // Lenient performance target
        min_consistency_score: 0.70, // Lower consistency requirement
        evaluate_overlaps: false,
        generate_reports: false,
        report_output_dir: "test_reports".to_string(),
    };
    
    let mut lenient_validator = DiarizationValidator::with_config(lenient_config);
    
    let lenient_result = lenient_validator.compare_segments(
        predicted_segments,
        ground_truth_data.segments,
        500.0, // 500ms tolerance matching config
    ).expect("Validation should succeed");
    
    // Lenient validation should perform better than strict
    assert!(lenient_result.der_result.der_score <= strict_result.der_result.der_score);
    assert!(lenient_result.summary.overall_quality >= strict_result.summary.overall_quality);
    
    // Strict validation might not meet targets, lenient should
    if !strict_result.summary.meets_targets {
        println!("Strict validation correctly identified areas for improvement");
    }
    
    if lenient_result.summary.meets_targets {
        println!("Lenient validation correctly passed with reasonable tolerances");
    }
    
    println!("Configuration comparison results:");
    println!("  Strict DER: {:.2}% (meets targets: {})", 
             strict_result.der_result.der_score * 100.0,
             strict_result.summary.meets_targets);
    println!("  Lenient DER: {:.2}% (meets targets: {})", 
             lenient_result.der_result.der_score * 100.0,
             lenient_result.summary.meets_targets);
}

/// Test error handling and edge cases
#[tokio::test]
async fn test_validation_error_handling() {
    let mut validator = DiarizationValidator::new();
    
    // Test with empty segments
    let result = validator.compare_segments(vec![], vec![], 250.0);
    assert!(result.is_err(), "Should fail with empty segments");
    
    // Test loading non-existent ground truth file
    let load_result = load_ground_truth("non_existent_file.json");
    assert!(load_result.is_err(), "Should fail loading non-existent file");
    
    // Test with mismatched time ranges
    let predicted = vec![
        SpeakerSegment {
            speaker_id: "speaker_1".to_string(),
            start_time: 0.0,
            end_time: 10.0, // Much longer than ground truth
            confidence: 0.9,
            text: Some("Long segment".to_string()),
            embedding: None,
            has_overlap: false,
            overlapping_speakers: vec![],
        }
    ];
    
    let ground_truth = vec![
        GroundTruthSegment {
            speaker_id: "speaker_1".to_string(),
            start_time: 0.0,
            end_time: 2.0, // Much shorter
            text: Some("Short segment".to_string()),
            audio_file: None,
            quality: 1.0,
        }
    ];
    
    // This should still work but show poor accuracy
    let validation_result = validator.compare_segments(predicted, ground_truth, 250.0);
    assert!(validation_result.is_ok(), "Should handle time range mismatches");
    
    if let Ok(result) = validation_result {
        // Should show poor accuracy due to timing issues
        assert!(result.der_result.der_score > 0.3, "Should show poor accuracy for mismatched timing");
    }
    
    println!("Error handling tests completed");
}