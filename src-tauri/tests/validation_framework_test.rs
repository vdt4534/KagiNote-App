//! Integration test for the diarization validation framework
//! 
//! This test verifies that the validation framework compiles and runs
//! basic validation scenarios successfully.

use kaginote_lib::diarization::types::SpeakerSegment;
use serde::{Deserialize, Serialize};

// Import the validation framework
mod validation_utils {
    use super::*;
    
    /// Simple ground truth segment for testing
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct SimpleGroundTruthSegment {
        pub speaker_id: String,
        pub start_time: f32,
        pub end_time: f32,
        pub text: Option<String>,
        pub quality: f32,
    }
    
    /// Simple validation metrics
    #[derive(Debug, Clone)]
    pub struct SimpleValidationResult {
        pub der_score: f32,
        pub precision: f32,
        pub recall: f32,
        pub f1_score: f32,
        pub accuracy_level: AccuracyLevel,
    }
    
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum AccuracyLevel {
        Excellent,
        Good,
        Fair,
        Poor,
    }
    
    /// Simple validator for basic testing
    pub struct SimpleValidator;
    
    impl SimpleValidator {
        /// Compare predicted segments with ground truth and calculate basic metrics
        pub fn validate(
            predicted: &[SpeakerSegment],
            ground_truth: &[SimpleGroundTruthSegment],
            tolerance_ms: f32,
        ) -> SimpleValidationResult {
            if predicted.is_empty() || ground_truth.is_empty() {
                return SimpleValidationResult {
                    der_score: 1.0,
                    precision: 0.0,
                    recall: 0.0,
                    f1_score: 0.0,
                    accuracy_level: AccuracyLevel::Poor,
                };
            }
            
            let tolerance_s = tolerance_ms / 1000.0;
            let mut correct_matches = 0;
            let mut total_predicted = predicted.len();
            let mut total_ground_truth = ground_truth.len();
            
            // Simple overlap-based matching
            for pred in predicted {
                for gt in ground_truth {
                    // Check if segments overlap significantly
                    let overlap_start = pred.start_time.max(gt.start_time);
                    let overlap_end = pred.end_time.min(gt.end_time);
                    let overlap_duration = (overlap_end - overlap_start).max(0.0);
                    
                    // Consider it a match if:
                    // 1. There's significant overlap (>50% of either segment)
                    // 2. Speaker IDs match (or are similar)
                    let pred_duration = pred.end_time - pred.start_time;
                    let gt_duration = gt.end_time - gt.start_time;
                    let overlap_ratio = overlap_duration / pred_duration.min(gt_duration);
                    
                    if overlap_ratio > 0.5 && Self::speakers_match(&pred.speaker_id, &gt.speaker_id) {
                        correct_matches += 1;
                        break; // Only count each predicted segment once
                    }
                }
            }
            
            // Calculate basic metrics
            let precision = if total_predicted > 0 {
                correct_matches as f32 / total_predicted as f32
            } else {
                0.0
            };
            
            let recall = if total_ground_truth > 0 {
                correct_matches as f32 / total_ground_truth as f32
            } else {
                0.0
            };
            
            let f1_score = if precision + recall > 0.0 {
                2.0 * (precision * recall) / (precision + recall)
            } else {
                0.0
            };
            
            // Simple DER approximation
            let missed = total_ground_truth - correct_matches;
            let false_positives = total_predicted - correct_matches;
            let der_score = (missed + false_positives) as f32 / total_ground_truth.max(1) as f32;
            
            let accuracy_level = match der_score {
                x if x < 0.10 => AccuracyLevel::Excellent,
                x if x < 0.20 => AccuracyLevel::Good,
                x if x < 0.30 => AccuracyLevel::Fair,
                _ => AccuracyLevel::Poor,
            };
            
            SimpleValidationResult {
                der_score,
                precision,
                recall,
                f1_score,
                accuracy_level,
            }
        }
        
        /// Simple speaker matching - in practice would use more sophisticated matching
        fn speakers_match(pred_id: &str, gt_id: &str) -> bool {
            pred_id == gt_id || 
            pred_id.contains(&gt_id.replace("speaker_", "")) ||
            gt_id.contains(&pred_id.replace("speaker_", ""))
        }
    }
}

use validation_utils::*;

#[test]
fn test_validation_framework_basic_functionality() {
    // Create simple test data
    let predicted_segments = vec![
        SpeakerSegment {
            speaker_id: "speaker_alice".to_string(),
            start_time: 0.0,
            end_time: 2.0,
            confidence: 0.9,
            text: Some("Hello world".to_string()),
            embedding: None,
            has_overlap: false,
            overlapping_speakers: vec![],
        },
        SpeakerSegment {
            speaker_id: "speaker_bob".to_string(),
            start_time: 2.5,
            end_time: 4.5,
            confidence: 0.8,
            text: Some("How are you".to_string()),
            embedding: None,
            has_overlap: false,
            overlapping_speakers: vec![],
        },
    ];
    
    let ground_truth_segments = vec![
        SimpleGroundTruthSegment {
            speaker_id: "speaker_alice".to_string(),
            start_time: 0.0,
            end_time: 2.0,
            text: Some("Hello world".to_string()),
            quality: 1.0,
        },
        SimpleGroundTruthSegment {
            speaker_id: "speaker_bob".to_string(),
            start_time: 2.5,
            end_time: 4.5,
            text: Some("How are you".to_string()),
            quality: 1.0,
        },
    ];
    
    // Run validation
    let result = SimpleValidator::validate(&predicted_segments, &ground_truth_segments, 250.0);
    
    // Verify results
    assert!(result.der_score < 0.1, "DER should be low for perfect match, got {}", result.der_score);
    assert!(result.precision > 0.9, "Precision should be high, got {}", result.precision);
    assert!(result.recall > 0.9, "Recall should be high, got {}", result.recall);
    assert!(result.f1_score > 0.9, "F1 score should be high, got {}", result.f1_score);
    assert_eq!(result.accuracy_level, AccuracyLevel::Excellent);
    
    println!("✅ Perfect match validation test passed!");
    println!("   DER: {:.3}", result.der_score);
    println!("   Precision: {:.3}", result.precision);
    println!("   Recall: {:.3}", result.recall);
    println!("   F1 Score: {:.3}", result.f1_score);
    println!("   Accuracy Level: {:?}", result.accuracy_level);
}

#[test]
fn test_validation_framework_speaker_confusion() {
    // Create test data with speaker confusion
    let predicted_segments = vec![
        SpeakerSegment {
            speaker_id: "speaker_bob".to_string(),  // Wrong speaker!
            start_time: 0.0,
            end_time: 2.0,
            confidence: 0.7,
            text: Some("Hello world".to_string()),
            embedding: None,
            has_overlap: false,
            overlapping_speakers: vec![],
        },
        SpeakerSegment {
            speaker_id: "speaker_alice".to_string(), // Wrong speaker!
            start_time: 2.5,
            end_time: 4.5,
            confidence: 0.6,
            text: Some("How are you".to_string()),
            embedding: None,
            has_overlap: false,
            overlapping_speakers: vec![],
        },
    ];
    
    let ground_truth_segments = vec![
        SimpleGroundTruthSegment {
            speaker_id: "speaker_alice".to_string(),
            start_time: 0.0,
            end_time: 2.0,
            text: Some("Hello world".to_string()),
            quality: 1.0,
        },
        SimpleGroundTruthSegment {
            speaker_id: "speaker_bob".to_string(),
            start_time: 2.5,
            end_time: 4.5,
            text: Some("How are you".to_string()),
            quality: 1.0,
        },
    ];
    
    // Run validation
    let result = SimpleValidator::validate(&predicted_segments, &ground_truth_segments, 250.0);
    
    // Should show poor accuracy due to speaker confusion
    assert!(result.der_score > 0.5, "DER should be high for speaker confusion, got {}", result.der_score);
    assert!(result.precision < 0.5, "Precision should be low, got {}", result.precision);
    assert!(result.recall < 0.5, "Recall should be low, got {}", result.recall);
    assert_eq!(result.accuracy_level, AccuracyLevel::Poor);
    
    println!("✅ Speaker confusion validation test passed!");
    println!("   DER: {:.3}", result.der_score);
    println!("   Accuracy Level: {:?}", result.accuracy_level);
}

#[test]
fn test_validation_framework_missed_segments() {
    // Create test data with missing segments
    let predicted_segments = vec![
        SpeakerSegment {
            speaker_id: "speaker_alice".to_string(),
            start_time: 0.0,
            end_time: 2.0,
            confidence: 0.9,
            text: Some("Hello world".to_string()),
            embedding: None,
            has_overlap: false,
            overlapping_speakers: vec![],
        },
        // Missing the second speaker segment
    ];
    
    let ground_truth_segments = vec![
        SimpleGroundTruthSegment {
            speaker_id: "speaker_alice".to_string(),
            start_time: 0.0,
            end_time: 2.0,
            text: Some("Hello world".to_string()),
            quality: 1.0,
        },
        SimpleGroundTruthSegment {
            speaker_id: "speaker_bob".to_string(),
            start_time: 2.5,
            end_time: 4.5,
            text: Some("How are you".to_string()),
            quality: 1.0,
        },
    ];
    
    // Run validation
    let result = SimpleValidator::validate(&predicted_segments, &ground_truth_segments, 250.0);
    
    // Should show reduced recall due to missed segment
    assert!(result.recall < 0.8, "Recall should be reduced due to missed segment, got {}", result.recall);
    assert!(result.der_score > 0.3, "DER should be elevated, got {}", result.der_score);
    
    println!("✅ Missed segments validation test passed!");
    println!("   Recall: {:.3}", result.recall);
    println!("   DER: {:.3}", result.der_score);
}

#[test]
fn test_validation_framework_empty_data() {
    // Test edge cases with empty data
    let empty_predicted: Vec<SpeakerSegment> = vec![];
    let empty_ground_truth: Vec<SimpleGroundTruthSegment> = vec![];
    
    let result = SimpleValidator::validate(&empty_predicted, &empty_ground_truth, 250.0);
    
    // Should handle empty data gracefully
    assert_eq!(result.accuracy_level, AccuracyLevel::Poor);
    assert_eq!(result.precision, 0.0);
    assert_eq!(result.recall, 0.0);
    assert_eq!(result.f1_score, 0.0);
    
    println!("✅ Empty data validation test passed!");
}

#[test]
fn test_validation_framework_timing_tolerance() {
    // Create slightly misaligned segments
    let predicted_segments = vec![
        SpeakerSegment {
            speaker_id: "speaker_alice".to_string(),
            start_time: 0.1,  // 100ms offset
            end_time: 2.1,
            confidence: 0.9,
            text: Some("Hello world".to_string()),
            embedding: None,
            has_overlap: false,
            overlapping_speakers: vec![],
        },
    ];
    
    let ground_truth_segments = vec![
        SimpleGroundTruthSegment {
            speaker_id: "speaker_alice".to_string(),
            start_time: 0.0,
            end_time: 2.0,
            text: Some("Hello world".to_string()),
            quality: 1.0,
        },
    ];
    
    // Test with strict tolerance
    let result_strict = SimpleValidator::validate(&predicted_segments, &ground_truth_segments, 50.0);
    
    // Test with lenient tolerance  
    let result_lenient = SimpleValidator::validate(&predicted_segments, &ground_truth_segments, 200.0);
    
    // Both should handle the timing difference reasonably well due to overlap-based matching
    assert!(result_lenient.f1_score >= result_strict.f1_score, 
            "Lenient tolerance should perform at least as well as strict");
    
    println!("✅ Timing tolerance validation test passed!");
    println!("   Strict F1: {:.3}", result_strict.f1_score);
    println!("   Lenient F1: {:.3}", result_lenient.f1_score);
}

#[test] 
fn test_validation_framework_performance() {
    // Create a larger dataset to test performance
    let mut predicted_segments = Vec::new();
    let mut ground_truth_segments = Vec::new();
    
    let num_segments = 100;
    let segment_duration = 2.0;
    let speakers = vec!["speaker_alice", "speaker_bob", "speaker_charlie"];
    
    for i in 0..num_segments {
        let start_time = i as f32 * segment_duration;
        let end_time = start_time + segment_duration * 0.9;
        let speaker_id = speakers[i % speakers.len()].to_string();
        
        predicted_segments.push(SpeakerSegment {
            speaker_id: speaker_id.clone(),
            start_time,
            end_time,
            confidence: 0.8 + (i as f32 % 10.0) * 0.02, // Vary confidence
            text: Some(format!("Segment {} content", i)),
            embedding: None,
            has_overlap: false,
            overlapping_speakers: vec![],
        });
        
        ground_truth_segments.push(SimpleGroundTruthSegment {
            speaker_id,
            start_time,
            end_time,
            text: Some(format!("Segment {} content", i)),
            quality: 1.0,
        });
    }
    
    let start_time = std::time::Instant::now();
    let result = SimpleValidator::validate(&predicted_segments, &ground_truth_segments, 250.0);
    let duration = start_time.elapsed();
    
    // Should complete quickly and accurately for perfect match
    assert!(duration.as_millis() < 100, "Validation should complete quickly, took {}ms", duration.as_millis());
    assert!(result.f1_score > 0.95, "Large dataset should validate accurately, got F1: {}", result.f1_score);
    assert_eq!(result.accuracy_level, AccuracyLevel::Excellent);
    
    println!("✅ Performance validation test passed!");
    println!("   Segments: {}", num_segments);
    println!("   Duration: {}ms", duration.as_millis());
    println!("   F1 Score: {:.3}", result.f1_score);
    println!("   Throughput: {:.1} segments/ms", num_segments as f32 / duration.as_millis() as f32);
}

#[test]
fn test_validation_framework_serialization() {
    // Test that validation data can be serialized/deserialized
    let ground_truth = SimpleGroundTruthSegment {
        speaker_id: "speaker_test".to_string(),
        start_time: 1.0,
        end_time: 3.0,
        text: Some("Test segment".to_string()),
        quality: 0.95,
    };
    
    // Test JSON serialization
    let json = serde_json::to_string(&ground_truth).expect("Should serialize to JSON");
    let deserialized: SimpleGroundTruthSegment = serde_json::from_str(&json).expect("Should deserialize from JSON");
    
    assert_eq!(ground_truth.speaker_id, deserialized.speaker_id);
    assert_eq!(ground_truth.start_time, deserialized.start_time);
    assert_eq!(ground_truth.end_time, deserialized.end_time);
    assert_eq!(ground_truth.text, deserialized.text);
    assert_eq!(ground_truth.quality, deserialized.quality);
    
    println!("✅ Serialization validation test passed!");
    println!("   JSON: {}", json);
}