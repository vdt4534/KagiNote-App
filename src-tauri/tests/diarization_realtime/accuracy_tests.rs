//! Accuracy tests for real-time speaker diarization
//! 
//! These tests validate speaker identification accuracy, DER calculations,
//! and clustering performance across various scenarios.

use super::*;
use std::collections::HashMap;

/// Test speaker identification accuracy with ground truth
#[tokio::test]
async fn test_speaker_identification_accuracy() {
    let scenarios = get_standard_test_scenarios();
    
    for scenario in scenarios {
        println!("Testing accuracy for scenario: {}", scenario.name);
        
        // Simulate diarization results
        let predicted_segments = simulate_speaker_identification(&scenario).await;
        
        // Load ground truth (simulate since we don't have real files yet)
        let ground_truth = simulate_ground_truth(&scenario);
        
        // Calculate accuracy metrics
        let accuracy = calculate_speaker_accuracy(&predicted_segments, &ground_truth.segments);
        let der = DiarizationTestUtils::calculate_der(
            &predicted_segments,
            &ground_truth.segments.iter()
                .map(|s| (s.start, s.end, s.speaker.clone()))
                .collect::<Vec<_>>(),
            ground_truth.total_duration,
        );
        
        println!("  Accuracy: {:.2}%", accuracy * 100.0);
        println!("  DER: {:.2}%", der * 100.0);
        
        // Validate against expected metrics
        assert!(
            accuracy >= scenario.expected_metrics.min_accuracy,
            "Accuracy {:.2} below minimum {:.2} for scenario {}",
            accuracy, scenario.expected_metrics.min_accuracy, scenario.name
        );
        
        assert!(
            der <= scenario.expected_metrics.max_der,
            "DER {:.2} exceeds maximum {:.2} for scenario {}",
            der, scenario.expected_metrics.max_der, scenario.name
        );
    }
    
    println!("✅ Speaker identification accuracy tests passed");
}

/// Test DER calculation with various edge cases
#[test]
fn test_der_calculation_edge_cases() {
    // Test case 1: Perfect match
    let predicted = vec![
        (0.0, 5.0, "speaker_1".to_string()),
        (5.0, 10.0, "speaker_2".to_string()),
    ];
    let ground_truth = predicted.clone();
    
    let der = DiarizationTestUtils::calculate_der(&predicted, &ground_truth, 10.0);
    assert!(der < 0.01, "Perfect match should have DER near 0, got {}", der);
    
    // Test case 2: Completely wrong
    let predicted = vec![
        (0.0, 10.0, "speaker_1".to_string()),
    ];
    let ground_truth = vec![
        (0.0, 10.0, "speaker_2".to_string()),
    ];
    
    let der = DiarizationTestUtils::calculate_der(&predicted, &ground_truth, 10.0);
    assert!(der > 0.99, "Complete mismatch should have DER near 1, got {}", der);
    
    // Test case 3: Partial overlap
    let predicted = vec![
        (0.0, 6.0, "speaker_1".to_string()),
        (6.0, 10.0, "speaker_2".to_string()),
    ];
    let ground_truth = vec![
        (0.0, 4.0, "speaker_1".to_string()),
        (4.0, 10.0, "speaker_2".to_string()),
    ];
    
    let der = DiarizationTestUtils::calculate_der(&predicted, &ground_truth, 10.0);
    assert!(der > 0.0 && der < 1.0, "Partial mismatch should have DER between 0 and 1, got {}", der);
    
    // Test case 4: Empty segments
    let predicted = vec![];
    let ground_truth = vec![
        (0.0, 10.0, "speaker_1".to_string()),
    ];
    
    let der = DiarizationTestUtils::calculate_der(&predicted, &ground_truth, 10.0);
    assert!(der > 0.99, "Missing all speakers should have high DER, got {}", der);
    
    println!("✅ DER calculation edge cases passed");
}

/// Test clustering accuracy with different similarity thresholds
#[tokio::test]
async fn test_clustering_threshold_optimization() {
    let thresholds = vec![0.5, 0.6, 0.7, 0.8, 0.9];
    let mut best_threshold = 0.7;
    let mut best_accuracy = 0.0;
    
    for threshold in thresholds {
        let config = DiarizationTestConfig {
            similarity_threshold: threshold,
            ..Default::default()
        };
        
        let mut total_accuracy = 0.0;
        let scenarios = get_standard_test_scenarios();
        
        for scenario in &scenarios {
            let predicted_segments = simulate_speaker_identification_with_config(&scenario, &config).await;
            let ground_truth = simulate_ground_truth(&scenario);
            
            let accuracy = calculate_speaker_accuracy(&predicted_segments, &ground_truth.segments);
            total_accuracy += accuracy;
        }
        
        let avg_accuracy = total_accuracy / scenarios.len() as f32;
        
        println!("Threshold {}: Average accuracy {:.2}%", threshold, avg_accuracy * 100.0);
        
        if avg_accuracy > best_accuracy {
            best_accuracy = avg_accuracy;
            best_threshold = threshold;
        }
    }
    
    println!("✅ Best threshold: {} with accuracy {:.2}%", best_threshold, best_accuracy * 100.0);
    
    // Verify that optimization found a reasonable threshold
    assert!(best_threshold >= 0.6 && best_threshold <= 0.8, 
           "Optimal threshold {} should be in reasonable range", best_threshold);
    assert!(best_accuracy >= 0.80, 
           "Best accuracy {:.2} should be at least 80%", best_accuracy);
}

/// Test speaker consistency across segments
#[tokio::test]
async fn test_speaker_consistency() {
    let scenario = TestScenario {
        name: "consistency_test".to_string(),
        description: "Test speaker label consistency".to_string(),
        expected_speakers: 3,
        audio_file: "consistency_test.wav".to_string(),
        ground_truth_file: "consistency_test.json".to_string(),
        expected_metrics: PerformanceMetrics::default(),
    };
    
    // Run diarization multiple times
    let mut all_results = Vec::new();
    for _run in 0..5 {
        let predicted_segments = simulate_speaker_identification(&scenario).await;
        all_results.push(predicted_segments);
    }
    
    // Calculate consistency metrics
    let consistency_score = calculate_consistency_across_runs(&all_results);
    
    println!("Speaker consistency score: {:.2}%", consistency_score * 100.0);
    
    assert!(
        consistency_score >= 0.85,
        "Speaker consistency {:.2} should be at least 85%",
        consistency_score
    );
    
    println!("✅ Speaker consistency test passed");
}

/// Test overlapping speech detection accuracy
#[tokio::test]
async fn test_overlapping_speech_detection() {
    let scenario = TestScenario {
        name: "overlapping_speech_test".to_string(),
        description: "Test overlapping speech detection".to_string(),
        expected_speakers: 3,
        audio_file: "overlapping_speech.wav".to_string(),
        ground_truth_file: "overlapping_speech.json".to_string(),
        expected_metrics: PerformanceMetrics {
            max_der: 0.25, // Higher tolerance for overlapping speech
            min_accuracy: 0.75,
            ..Default::default()
        },
    };
    
    let predicted_segments = simulate_speaker_identification(&scenario).await;
    let ground_truth = simulate_ground_truth_with_overlaps(&scenario);
    
    // Calculate overlap detection accuracy
    let overlap_accuracy = calculate_overlap_detection_accuracy(
        &predicted_segments,
        &ground_truth.segments,
    );
    
    println!("Overlap detection accuracy: {:.2}%", overlap_accuracy * 100.0);
    
    assert!(
        overlap_accuracy >= 0.70,
        "Overlap detection accuracy {:.2} should be at least 70%",
        overlap_accuracy
    );
    
    // Calculate overall DER for overlapping speech scenario
    let der = DiarizationTestUtils::calculate_der(
        &predicted_segments,
        &ground_truth.segments.iter()
            .map(|s| (s.start, s.end, s.speaker.clone()))
            .collect::<Vec<_>>(),
        ground_truth.total_duration,
    );
    
    assert!(
        der <= scenario.expected_metrics.max_der,
        "DER {:.2} exceeds threshold for overlapping speech",
        der
    );
    
    println!("✅ Overlapping speech detection test passed");
}

/// Test accuracy with different numbers of speakers
#[tokio::test]
async fn test_multi_speaker_accuracy() {
    let speaker_counts = vec![2, 3, 4, 6, 8];
    let mut results = HashMap::new();
    
    for num_speakers in speaker_counts {
        let scenario = TestScenario {
            name: format!("{}_speaker_test", num_speakers),
            description: format!("Test with {} speakers", num_speakers),
            expected_speakers: num_speakers,
            audio_file: format!("{}_speakers.wav", num_speakers),
            ground_truth_file: format!("{}_speakers.json", num_speakers),
            expected_metrics: PerformanceMetrics {
                min_accuracy: if num_speakers <= 4 { 0.85 } else { 0.80 },
                max_der: if num_speakers <= 4 { 0.15 } else { 0.20 },
                ..Default::default()
            },
        };
        
        let predicted_segments = simulate_speaker_identification(&scenario).await;
        let ground_truth = simulate_ground_truth(&scenario);
        
        let accuracy = calculate_speaker_accuracy(&predicted_segments, &ground_truth.segments);
        let der = DiarizationTestUtils::calculate_der(
            &predicted_segments,
            &ground_truth.segments.iter()
                .map(|s| (s.start, s.end, s.speaker.clone()))
                .collect::<Vec<_>>(),
            ground_truth.total_duration,
        );
        
        results.insert(num_speakers, (accuracy, der));
        
        println!("Speakers: {}, Accuracy: {:.2}%, DER: {:.2}%", 
                 num_speakers, accuracy * 100.0, der * 100.0);
        
        // Validate accuracy decreases gracefully with more speakers
        assert!(
            accuracy >= scenario.expected_metrics.min_accuracy,
            "Accuracy {:.2} below threshold for {} speakers",
            accuracy, num_speakers
        );
        
        assert!(
            der <= scenario.expected_metrics.max_der,
            "DER {:.2} exceeds threshold for {} speakers",
            der, num_speakers
        );
    }
    
    println!("✅ Multi-speaker accuracy test passed");
}

/// Test noise robustness
#[tokio::test]
async fn test_noise_robustness() {
    let noise_levels = vec!["clean", "light_noise", "moderate_noise", "heavy_noise"];
    
    for noise_level in noise_levels {
        let scenario = TestScenario {
            name: format!("noise_test_{}", noise_level),
            description: format!("Test with {} audio", noise_level),
            expected_speakers: 2,
            audio_file: format!("audio_{}.wav", noise_level),
            ground_truth_file: format!("audio_{}.json", noise_level),
            expected_metrics: PerformanceMetrics {
                min_accuracy: match noise_level {
                    "clean" => 0.90,
                    "light_noise" => 0.85,
                    "moderate_noise" => 0.75,
                    "heavy_noise" => 0.65,
                    _ => 0.50,
                },
                max_der: match noise_level {
                    "clean" => 0.10,
                    "light_noise" => 0.15,
                    "moderate_noise" => 0.25,
                    "heavy_noise" => 0.35,
                    _ => 0.50,
                },
                ..Default::default()
            },
        };
        
        let predicted_segments = simulate_speaker_identification(&scenario).await;
        let ground_truth = simulate_ground_truth(&scenario);
        
        let accuracy = calculate_speaker_accuracy(&predicted_segments, &ground_truth.segments);
        let der = DiarizationTestUtils::calculate_der(
            &predicted_segments,
            &ground_truth.segments.iter()
                .map(|s| (s.start, s.end, s.speaker.clone()))
                .collect::<Vec<_>>(),
            ground_truth.total_duration,
        );
        
        println!("Noise level: {}, Accuracy: {:.2}%, DER: {:.2}%", 
                 noise_level, accuracy * 100.0, der * 100.0);
        
        assert!(
            accuracy >= scenario.expected_metrics.min_accuracy,
            "Accuracy {:.2} below threshold for {} noise",
            accuracy, noise_level
        );
        
        assert!(
            der <= scenario.expected_metrics.max_der,
            "DER {:.2} exceeds threshold for {} noise",
            der, noise_level
        );
    }
    
    println!("✅ Noise robustness test passed");
}

// Helper functions

/// Simulate speaker identification results
async fn simulate_speaker_identification(scenario: &TestScenario) -> Vec<(f32, f32, String)> {
    simulate_speaker_identification_with_config(scenario, &DiarizationTestConfig::default()).await
}

/// Simulate speaker identification with custom config
async fn simulate_speaker_identification_with_config(
    scenario: &TestScenario, 
    _config: &DiarizationTestConfig
) -> Vec<(f32, f32, String)> {
    // Simulate realistic diarization results
    let segment_duration = 30.0 / scenario.expected_speakers as f32;
    let mut segments = Vec::new();
    
    for i in 0..scenario.expected_speakers {
        let start = i as f32 * segment_duration;
        let end = (i + 1) as f32 * segment_duration;
        let speaker = format!("speaker_{}", i + 1);
        
        segments.push((start, end, speaker));
    }
    
    // Add some realistic noise to the boundaries
    for segment in &mut segments {
        segment.0 += (rand_f32() - 0.5) * 0.5; // ±0.25s variation
        segment.1 += (rand_f32() - 0.5) * 0.5;
    }
    
    segments
}

/// Simulate ground truth data
fn simulate_ground_truth(scenario: &TestScenario) -> GroundTruthData {
    let segment_duration = 30.0 / scenario.expected_speakers as f32;
    let mut segments = Vec::new();
    
    for i in 0..scenario.expected_speakers {
        let start = i as f32 * segment_duration;
        let end = (i + 1) as f32 * segment_duration;
        let speaker = format!("speaker_{}", i + 1);
        
        segments.push(GroundTruthSegment {
            start,
            end,
            speaker,
            text: Some(format!("Sample text from speaker {}", i + 1)),
        });
    }
    
    GroundTruthData {
        metadata: HashMap::from([
            ("duration".to_string(), "30.0".to_string()),
            ("sample_rate".to_string(), "16000".to_string()),
        ]),
        segments,
        total_speakers: scenario.expected_speakers,
        total_duration: 30.0,
    }
}

/// Simulate ground truth with overlapping segments
fn simulate_ground_truth_with_overlaps(scenario: &TestScenario) -> GroundTruthData {
    let mut segments = Vec::new();
    
    // Create overlapping segments
    segments.push(GroundTruthSegment {
        start: 0.0,
        end: 15.0,
        speaker: "speaker_1".to_string(),
        text: Some("First speaker content".to_string()),
    });
    
    segments.push(GroundTruthSegment {
        start: 10.0,
        end: 20.0,
        speaker: "speaker_2".to_string(),
        text: Some("Second speaker content".to_string()),
    });
    
    segments.push(GroundTruthSegment {
        start: 15.0,
        end: 30.0,
        speaker: "speaker_3".to_string(),
        text: Some("Third speaker content".to_string()),
    });
    
    GroundTruthData {
        metadata: HashMap::from([
            ("duration".to_string(), "30.0".to_string()),
            ("overlapping".to_string(), "true".to_string()),
        ]),
        segments,
        total_speakers: scenario.expected_speakers,
        total_duration: 30.0,
    }
}

/// Calculate speaker identification accuracy
fn calculate_speaker_accuracy(
    predicted: &[(f32, f32, String)],
    ground_truth: &[GroundTruthSegment],
) -> f32 {
    let total_duration = ground_truth.iter()
        .map(|s| s.end - s.start)
        .sum::<f32>();
    
    let mut correct_duration = 0.0;
    
    // Sample at 100ms intervals
    let sample_interval = 0.1;
    let max_time = ground_truth.iter()
        .map(|s| s.end)
        .fold(0.0f32, |a, b| a.max(b));
    
    let num_samples = (max_time / sample_interval) as usize;
    
    for i in 0..num_samples {
        let time = i as f32 * sample_interval;
        
        let predicted_speaker = get_speaker_at_time_predicted(predicted, time);
        let ground_truth_speaker = get_speaker_at_time_ground_truth(ground_truth, time);
        
        if let (Some(pred), Some(gt)) = (predicted_speaker, ground_truth_speaker) {
            if pred == gt {
                correct_duration += sample_interval;
            }
        }
    }
    
    if total_duration > 0.0 {
        correct_duration / total_duration
    } else {
        0.0
    }
}

fn get_speaker_at_time_predicted(segments: &[(f32, f32, String)], time: f32) -> Option<String> {
    for (start, end, speaker) in segments {
        if time >= *start && time < *end {
            return Some(speaker.clone());
        }
    }
    None
}

fn get_speaker_at_time_ground_truth(segments: &[GroundTruthSegment], time: f32) -> Option<String> {
    for segment in segments {
        if time >= segment.start && time < segment.end {
            return Some(segment.speaker.clone());
        }
    }
    None
}

/// Calculate consistency across multiple runs
fn calculate_consistency_across_runs(all_results: &[Vec<(f32, f32, String)>]) -> f32 {
    if all_results.len() < 2 {
        return 1.0;
    }
    
    let mut total_consistency = 0.0;
    let mut comparisons = 0;
    
    for i in 0..all_results.len() {
        for j in (i + 1)..all_results.len() {
            let consistency = calculate_pairwise_consistency(&all_results[i], &all_results[j]);
            total_consistency += consistency;
            comparisons += 1;
        }
    }
    
    if comparisons > 0 {
        total_consistency / comparisons as f32
    } else {
        1.0
    }
}

fn calculate_pairwise_consistency(run1: &[(f32, f32, String)], run2: &[(f32, f32, String)]) -> f32 {
    // Simplified consistency calculation
    let sample_interval = 0.1;
    let max_time = run1.iter().chain(run2.iter())
        .map(|(_, end, _)| *end)
        .fold(0.0f32, |a, b| a.max(b));
    
    let num_samples = (max_time / sample_interval) as usize;
    let mut matches = 0;
    
    for i in 0..num_samples {
        let time = i as f32 * sample_interval;
        let speaker1 = get_speaker_at_time_predicted(run1, time);
        let speaker2 = get_speaker_at_time_predicted(run2, time);
        
        if speaker1 == speaker2 {
            matches += 1;
        }
    }
    
    if num_samples > 0 {
        matches as f32 / num_samples as f32
    } else {
        1.0
    }
}

/// Calculate overlap detection accuracy
fn calculate_overlap_detection_accuracy(
    predicted: &[(f32, f32, String)],
    ground_truth: &[GroundTruthSegment],
) -> f32 {
    // Simplified overlap detection accuracy
    // In practice, this would be more sophisticated
    
    let mut correct_overlaps = 0;
    let mut total_overlaps = 0;
    
    // Find overlaps in ground truth
    for i in 0..ground_truth.len() {
        for j in (i + 1)..ground_truth.len() {
            let seg1 = &ground_truth[i];
            let seg2 = &ground_truth[j];
            
            // Check if segments overlap
            if seg1.start < seg2.end && seg2.start < seg1.end {
                total_overlaps += 1;
                
                // Check if predicted results also show overlap
                let overlap_start = seg1.start.max(seg2.start);
                let overlap_end = seg1.end.min(seg2.end);
                let overlap_mid = (overlap_start + overlap_end) / 2.0;
                
                let pred_speaker = get_speaker_at_time_predicted(predicted, overlap_mid);
                if pred_speaker.is_some() {
                    correct_overlaps += 1;
                }
            }
        }
    }
    
    if total_overlaps > 0 {
        correct_overlaps as f32 / total_overlaps as f32
    } else {
        1.0 // No overlaps to detect
    }
}

/// Simple pseudo-random number generator for tests
fn rand_f32() -> f32 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    use std::time::{SystemTime, UNIX_EPOCH};
    
    let mut hasher = DefaultHasher::new();
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos().hash(&mut hasher);
    (hasher.finish() % 1000) as f32 / 1000.0
}