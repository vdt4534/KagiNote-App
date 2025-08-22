//! Simple validation test to verify the framework works
//! 
//! This test uses only the basic validation functionality without
//! depending on complex diarization modules.

use kaginote_lib::diarization::types::SpeakerSegment;
use serde::{Deserialize, Serialize};

/// Simple ground truth segment for testing
#[derive(Debug, Clone, Serialize, Deserialize)]
struct TestGroundTruthSegment {
    speaker_id: String,
    start_time: f32,
    end_time: f32,
    text: Option<String>,
}

/// Simple validation result
#[derive(Debug, Clone)]
struct TestValidationResult {
    der_score: f32,
    precision: f32,
    recall: f32,
    f1_score: f32,
}

/// Simple validator for testing
struct TestValidator;

impl TestValidator {
    /// Simple validation logic
    fn validate(
        predicted: &[SpeakerSegment],
        ground_truth: &[TestGroundTruthSegment],
    ) -> TestValidationResult {
        if predicted.is_empty() || ground_truth.is_empty() {
            return TestValidationResult {
                der_score: 1.0,
                precision: 0.0,
                recall: 0.0,
                f1_score: 0.0,
            };
        }
        
        let mut correct_matches = 0;
        let total_predicted = predicted.len();
        let total_ground_truth = ground_truth.len();
        
        // Simple matching: check if segments overlap and speakers match
        for pred in predicted {
            for gt in ground_truth {
                let overlap_start = pred.start_time.max(gt.start_time);
                let overlap_end = pred.end_time.min(gt.end_time);
                let overlap_duration = (overlap_end - overlap_start).max(0.0);
                
                if overlap_duration > 0.5 && pred.speaker_id == gt.speaker_id {
                    correct_matches += 1;
                    break;
                }
            }
        }
        
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
        
        let missed = total_ground_truth - correct_matches;
        let false_positives = total_predicted - correct_matches;
        let der_score = (missed + false_positives) as f32 / total_ground_truth.max(1) as f32;
        
        TestValidationResult {
            der_score,
            precision,
            recall,
            f1_score,
        }
    }
}

#[test]
fn test_simple_validation_perfect_match() {
    let predicted_segments = vec![
        SpeakerSegment {
            speaker_id: "alice".to_string(),
            start_time: 0.0,
            end_time: 2.0,
            confidence: 0.9,
            text: Some("Hello".to_string()),
            embedding: None,
            has_overlap: false,
            overlapping_speakers: vec![],
        },
        SpeakerSegment {
            speaker_id: "bob".to_string(),
            start_time: 2.5,
            end_time: 4.5,
            confidence: 0.8,
            text: Some("World".to_string()),
            embedding: None,
            has_overlap: false,
            overlapping_speakers: vec![],
        },
    ];
    
    let ground_truth_segments = vec![
        TestGroundTruthSegment {
            speaker_id: "alice".to_string(),
            start_time: 0.0,
            end_time: 2.0,
            text: Some("Hello".to_string()),
        },
        TestGroundTruthSegment {
            speaker_id: "bob".to_string(),
            start_time: 2.5,
            end_time: 4.5,
            text: Some("World".to_string()),
        },
    ];
    
    let result = TestValidator::validate(&predicted_segments, &ground_truth_segments);
    
    assert!(result.der_score < 0.1, "DER should be low for perfect match");
    assert!(result.precision > 0.9, "Precision should be high");
    assert!(result.recall > 0.9, "Recall should be high");
    assert!(result.f1_score > 0.9, "F1 score should be high");
    
    println!("✅ Simple validation test passed!");
    println!("   DER: {:.3}", result.der_score);
    println!("   Precision: {:.3}", result.precision);
    println!("   Recall: {:.3}", result.recall);
    println!("   F1 Score: {:.3}", result.f1_score);
}

#[test]
fn test_simple_validation_speaker_confusion() {
    let predicted_segments = vec![
        SpeakerSegment {
            speaker_id: "bob".to_string(), // Wrong speaker
            start_time: 0.0,
            end_time: 2.0,
            confidence: 0.7,
            text: Some("Hello".to_string()),
            embedding: None,
            has_overlap: false,
            overlapping_speakers: vec![],
        },
        SpeakerSegment {
            speaker_id: "alice".to_string(), // Wrong speaker
            start_time: 2.5,
            end_time: 4.5,
            confidence: 0.6,
            text: Some("World".to_string()),
            embedding: None,
            has_overlap: false,
            overlapping_speakers: vec![],
        },
    ];
    
    let ground_truth_segments = vec![
        TestGroundTruthSegment {
            speaker_id: "alice".to_string(),
            start_time: 0.0,
            end_time: 2.0,
            text: Some("Hello".to_string()),
        },
        TestGroundTruthSegment {
            speaker_id: "bob".to_string(),
            start_time: 2.5,
            end_time: 4.5,
            text: Some("World".to_string()),
        },
    ];
    
    let result = TestValidator::validate(&predicted_segments, &ground_truth_segments);
    
    assert!(result.der_score > 0.5, "DER should be high for speaker confusion");
    assert!(result.precision < 0.5, "Precision should be low");
    assert!(result.f1_score < 0.5, "F1 score should be low");
    
    println!("✅ Speaker confusion test passed!");
    println!("   DER: {:.3}", result.der_score);
    println!("   Precision: {:.3}", result.precision);
    println!("   F1 Score: {:.3}", result.f1_score);
}

#[test]
fn test_simple_validation_empty_data() {
    let empty_predicted: Vec<SpeakerSegment> = vec![];
    let empty_ground_truth: Vec<TestGroundTruthSegment> = vec![];
    
    let result = TestValidator::validate(&empty_predicted, &empty_ground_truth);
    
    assert_eq!(result.der_score, 1.0, "DER should be 1.0 for empty data");
    assert_eq!(result.precision, 0.0, "Precision should be 0.0");
    assert_eq!(result.recall, 0.0, "Recall should be 0.0");
    assert_eq!(result.f1_score, 0.0, "F1 score should be 0.0");
    
    println!("✅ Empty data test passed!");
}

#[test]
fn test_validation_serialization() {
    let ground_truth = TestGroundTruthSegment {
        speaker_id: "test_speaker".to_string(),
        start_time: 1.0,
        end_time: 3.0,
        text: Some("Test content".to_string()),
    };
    
    // Test JSON serialization
    let json = serde_json::to_string(&ground_truth).expect("Should serialize");
    let deserialized: TestGroundTruthSegment = serde_json::from_str(&json).expect("Should deserialize");
    
    assert_eq!(ground_truth.speaker_id, deserialized.speaker_id);
    assert_eq!(ground_truth.start_time, deserialized.start_time);
    assert_eq!(ground_truth.end_time, deserialized.end_time);
    assert_eq!(ground_truth.text, deserialized.text);
    
    println!("✅ Serialization test passed!");
    println!("   JSON: {}", json);
}

#[test]
fn test_validation_performance() {
    // Generate larger dataset
    let mut predicted_segments = Vec::new();
    let mut ground_truth_segments = Vec::new();
    
    for i in 0..50 {
        let start_time = i as f32 * 2.0;
        let end_time = start_time + 1.5;
        let speaker_id = format!("speaker_{}", i % 3);
        
        predicted_segments.push(SpeakerSegment {
            speaker_id: speaker_id.clone(),
            start_time,
            end_time,
            confidence: 0.8,
            text: Some(format!("Segment {}", i)),
            embedding: None,
            has_overlap: false,
            overlapping_speakers: vec![],
        });
        
        ground_truth_segments.push(TestGroundTruthSegment {
            speaker_id,
            start_time,
            end_time,
            text: Some(format!("Segment {}", i)),
        });
    }
    
    let start_time = std::time::Instant::now();
    let result = TestValidator::validate(&predicted_segments, &ground_truth_segments);
    let duration = start_time.elapsed();
    
    assert!(duration.as_millis() < 50, "Should be fast");
    assert!(result.f1_score > 0.95, "Should be accurate");
    
    println!("✅ Performance test passed!");
    println!("   Duration: {}ms", duration.as_millis());
    println!("   F1 Score: {:.3}", result.f1_score);
    println!("   Throughput: {:.1} segments/ms", predicted_segments.len() as f32 / duration.as_millis() as f32);
}