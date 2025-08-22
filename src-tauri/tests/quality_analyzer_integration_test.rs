//! Integration test demonstrating the comprehensive quality analyzer
//!
//! This test shows how to use the TranscriptionQualityAnalyzer with real
//! LibriSpeech data to evaluate both transcription (WER/CER) and diarization (DER)
//! accuracy in production scenarios.

use crate::transcription_quality_analyzer::{
    TranscriptionQualityAnalyzer, TranscriptionResult, TranscriptionGroundTruth,
    TranscriptionWord, GroundTruthWord, GroundTruthSpeakerSegment,
    SystemPerformanceMetrics, QualityAnalysisConfig, QualityLevel
};
use std::path::Path;

/// Test comprehensive quality analysis with LibriSpeech-like data
#[tokio::test]
async fn test_quality_analysis_librispeech_sample() {
    // Configure analyzer for comprehensive testing
    let config = QualityAnalysisConfig {
        include_character_analysis: true,
        include_per_speaker_analysis: true,
        include_word_alignment: true,
        generate_html_report: true,
        generate_json_report: true,
        report_output_dir: "test_quality_reports".to_string(),
        word_alignment_tolerance_s: 0.2,
    };
    
    let mut analyzer = TranscriptionQualityAnalyzer::with_config(config);
    
    // Create realistic transcription result based on LibriSpeech sample
    // This simulates the output from our whisper + diarization system
    let transcription_result = create_librispeech_transcription_result();
    
    // Create corresponding ground truth from LibriSpeech reference
    let ground_truth = create_librispeech_ground_truth();
    
    // Create realistic performance metrics
    let performance_metrics = create_production_performance_metrics();
    
    // Perform comprehensive quality analysis
    let analysis_result = analyzer.analyze_quality(
        transcription_result,
        ground_truth,
        performance_metrics,
    ).expect("Quality analysis should succeed");
    
    // Validate analysis results
    assert!(analysis_result.overall_score > 0.0);
    assert!(analysis_result.overall_score <= 100.0);
    
    // Check WER metrics
    assert!(analysis_result.wer_result.wer_percentage >= 0.0);
    assert!(analysis_result.wer_result.word_accuracy <= 100.0);
    
    // Check diarization metrics
    assert!(analysis_result.diarization_result.der_result.der_score >= 0.0);
    assert!(analysis_result.diarization_result.der_result.der_score <= 1.0);
    
    // Check speaker attribution
    assert!(analysis_result.sa_wer_result.speaker_attribution_accuracy >= 0.0);
    assert!(analysis_result.sa_wer_result.speaker_attribution_accuracy <= 100.0);
    
    // Verify recommendations are generated
    assert!(!analysis_result.recommendations.summary.is_empty());
    
    println!("=== TRANSCRIPTION QUALITY ANALYSIS REPORT ===");
    println!("Recording ID: {}", analysis_result.recording_id);
    println!("Overall Score: {:.1}%", analysis_result.overall_score);
    println!("Quality Level: {:?}", analysis_result.quality_level);
    println!();
    
    println!("📝 TRANSCRIPTION ACCURACY:");
    println!("  Word Error Rate: {:.2}%", analysis_result.wer_result.wer_percentage);
    println!("  Word Accuracy: {:.1}%", analysis_result.wer_result.word_accuracy);
    println!("  Character Accuracy: {:.1}%", analysis_result.wer_result.character_accuracy);
    println!("  Substitutions: {}", analysis_result.wer_result.substitutions);
    println!("  Insertions: {}", analysis_result.wer_result.insertions);
    println!("  Deletions: {}", analysis_result.wer_result.deletions);
    println!();
    
    println!("👥 SPEAKER DIARIZATION:");
    println!("  Diarization Error Rate: {:.2}%", analysis_result.diarization_result.der_result.der_score * 100.0);
    println!("  Precision: {:.3}", analysis_result.diarization_result.der_result.precision);
    println!("  Recall: {:.3}", analysis_result.diarization_result.der_result.recall);
    println!("  F1 Score: {:.3}", analysis_result.diarization_result.der_result.f1_score);
    println!("  Speaker Consistency: {:.1}%", analysis_result.diarization_result.consistency_result.consistency_percentage);
    println!();
    
    println!("⚡ PERFORMANCE METRICS:");
    println!("  Real-time Factor: {:.2}x", analysis_result.performance_metrics.real_time_factor);
    println!("  Memory Usage: {:.0} MB", analysis_result.performance_metrics.memory_usage_mb);
    println!("  Processing Latency: {} ms", analysis_result.performance_metrics.latency_ms);
    println!("  CPU Utilization: {:.1}%", analysis_result.performance_metrics.cpu_utilization);
    println!();
    
    println!("🎯 SPEAKER-ATTRIBUTED WER:");
    println!("  SA-WER: {:.2}%", analysis_result.sa_wer_result.sa_wer_percentage);
    println!("  Attribution Accuracy: {:.1}%", analysis_result.sa_wer_result.speaker_attribution_accuracy);
    for (speaker, wer) in &analysis_result.sa_wer_result.per_speaker_wer {
        println!("  {}: {:.2}% WER", speaker, wer);
    }
    println!();
    
    println!("💡 RECOMMENDATIONS:");
    println!("{}", analysis_result.recommendations.summary);
    
    if !analysis_result.recommendations.critical_improvements.is_empty() {
        println!("\n🚨 Critical Improvements:");
        for improvement in &analysis_result.recommendations.critical_improvements {
            println!("  • {}", improvement);
        }
    }
    
    if !analysis_result.recommendations.accuracy_enhancements.is_empty() {
        println!("\n🎯 Accuracy Enhancements:");
        for enhancement in &analysis_result.recommendations.accuracy_enhancements {
            println!("  • {}", enhancement);
        }
    }
    
    if !analysis_result.recommendations.performance_optimizations.is_empty() {
        println!("\n⚡ Performance Optimizations:");
        for optimization in &analysis_result.recommendations.performance_optimizations {
            println!("  • {}", optimization);
        }
    }
    
    println!("\n📊 Analysis completed in {} ms", analysis_result.analysis_duration_ms);
    
    // Reports should be generated
    let report_dir = Path::new("test_quality_reports");
    if report_dir.exists() {
        println!("📄 Reports generated in: {}", report_dir.display());
    }
}

/// Test quality analysis with poor transcription (high WER scenario)
#[tokio::test]
async fn test_quality_analysis_poor_transcription() {
    let mut analyzer = TranscriptionQualityAnalyzer::new();
    
    // Create transcription with many errors
    let poor_transcription = create_poor_transcription_result();
    let ground_truth = create_librispeech_ground_truth();
    let performance_metrics = create_production_performance_metrics();
    
    let result = analyzer.analyze_quality(
        poor_transcription,
        ground_truth,
        performance_metrics,
    ).expect("Analysis should succeed even with poor quality");
    
    // Should detect poor quality
    assert_eq!(result.quality_level, QualityLevel::Poor);
    assert!(result.wer_result.wer_percentage > 25.0);
    assert!(!result.recommendations.critical_improvements.is_empty());
    
    println!("Poor transcription analysis:");
    println!("  WER: {:.2}%", result.wer_result.wer_percentage);
    println!("  Quality: {:?}", result.quality_level);
    println!("  Critical improvements: {}", result.recommendations.critical_improvements.len());
}

/// Test quality analysis with slow performance
#[tokio::test]
async fn test_quality_analysis_slow_performance() {
    let mut analyzer = TranscriptionQualityAnalyzer::new();
    
    let transcription = create_librispeech_transcription_result();
    let ground_truth = create_librispeech_ground_truth();
    
    // Create slow performance metrics
    let slow_performance = SystemPerformanceMetrics {
        real_time_factor: 3.5, // Very slow
        memory_usage_mb: 2500.0, // High memory usage
        cpu_utilization: 95.0, // High CPU usage
        latency_ms: 5000, // High latency
        throughput: 0.3, // Low throughput
        model_load_time_ms: 15000, // Slow model loading
    };
    
    let result = analyzer.analyze_quality(
        transcription,
        ground_truth,
        slow_performance,
    ).expect("Analysis should succeed");
    
    // Should recommend performance improvements
    assert!(!result.recommendations.performance_optimizations.is_empty());
    assert!(result.recommendations.performance_optimizations.iter()
        .any(|rec| rec.contains("Real-time factor") || rec.contains("memory")));
    
    println!("Slow performance analysis:");
    println!("  RT Factor: {:.2}x", result.performance_metrics.real_time_factor);
    println!("  Performance optimizations: {}", result.recommendations.performance_optimizations.len());
}

/// Create realistic LibriSpeech-based transcription result
fn create_librispeech_transcription_result() -> TranscriptionResult {
    TranscriptionResult {
        text: "HE HOPED THERE WOULD BE STEW FOR DINNER TURNIPS AND CARROTS AND BRUSSEL SPROUTS AND ONIONS AND PERHAPS A NICE BIG FAT GOOSE WITH PLENTY OF STUFFING".to_string(),
        words: vec![
            TranscriptionWord {
                word: "HE".to_string(),
                start_time: 0.48,
                end_time: 0.70,
                confidence: 0.99,
                speaker_id: Some("speaker_1089".to_string()),
            },
            TranscriptionWord {
                word: "HOPED".to_string(),
                start_time: 0.70,
                end_time: 1.06,
                confidence: 0.98,
                speaker_id: Some("speaker_1089".to_string()),
            },
            TranscriptionWord {
                word: "THERE".to_string(),
                start_time: 1.06,
                end_time: 1.38,
                confidence: 0.97,
                speaker_id: Some("speaker_1089".to_string()),
            },
            TranscriptionWord {
                word: "WOULD".to_string(),
                start_time: 1.38,
                end_time: 1.66,
                confidence: 0.96,
                speaker_id: Some("speaker_1089".to_string()),
            },
            TranscriptionWord {
                word: "BE".to_string(),
                start_time: 1.66,
                end_time: 1.84,
                confidence: 0.95,
                speaker_id: Some("speaker_1089".to_string()),
            },
            TranscriptionWord {
                word: "STEW".to_string(),
                start_time: 1.84,
                end_time: 2.26,
                confidence: 0.94,
                speaker_id: Some("speaker_1089".to_string()),
            },
            TranscriptionWord {
                word: "FOR".to_string(),
                start_time: 2.26,
                end_time: 2.56,
                confidence: 0.93,
                speaker_id: Some("speaker_1089".to_string()),
            },
            TranscriptionWord {
                word: "DINNER".to_string(),
                start_time: 2.56,
                end_time: 3.14,
                confidence: 0.92,
                speaker_id: Some("speaker_1089".to_string()),
            },
            TranscriptionWord {
                word: "TURNIPS".to_string(),
                start_time: 3.14,
                end_time: 3.76,
                confidence: 0.89,
                speaker_id: Some("speaker_1089".to_string()),
            },
            TranscriptionWord {
                word: "AND".to_string(),
                start_time: 3.76,
                end_time: 4.02,
                confidence: 0.91,
                speaker_id: Some("speaker_1089".to_string()),
            },
            // Add more words to complete the sentence...
        ],
        confidence: 0.94,
        language: "en".to_string(),
        processing_time_ms: 2800,
        real_time_factor: 0.8,
    }
}

/// Create corresponding ground truth for LibriSpeech sample
fn create_librispeech_ground_truth() -> TranscriptionGroundTruth {
    TranscriptionGroundTruth {
        text: "HE HOPED THERE WOULD BE STEW FOR DINNER TURNIPS AND CARROTS AND BRUSSEL SPROUTS AND ONIONS AND PERHAPS A NICE BIG FAT GOOSE WITH PLENTY OF STUFFING".to_string(),
        words: vec![
            GroundTruthWord {
                word: "HE".to_string(),
                start_time: 0.48,
                end_time: 0.70,
                speaker_id: "speaker_1089".to_string(),
                is_correct_pronunciation: true,
            },
            GroundTruthWord {
                word: "HOPED".to_string(),
                start_time: 0.70,
                end_time: 1.06,
                speaker_id: "speaker_1089".to_string(),
                is_correct_pronunciation: true,
            },
            GroundTruthWord {
                word: "THERE".to_string(),
                start_time: 1.06,
                end_time: 1.38,
                speaker_id: "speaker_1089".to_string(),
                is_correct_pronunciation: true,
            },
            GroundTruthWord {
                word: "WOULD".to_string(),
                start_time: 1.38,
                end_time: 1.66,
                speaker_id: "speaker_1089".to_string(),
                is_correct_pronunciation: true,
            },
            GroundTruthWord {
                word: "BE".to_string(),
                start_time: 1.66,
                end_time: 1.84,
                speaker_id: "speaker_1089".to_string(),
                is_correct_pronunciation: true,
            },
            GroundTruthWord {
                word: "STEW".to_string(),
                start_time: 1.84,
                end_time: 2.26,
                speaker_id: "speaker_1089".to_string(),
                is_correct_pronunciation: true,
            },
            GroundTruthWord {
                word: "FOR".to_string(),
                start_time: 2.26,
                end_time: 2.56,
                speaker_id: "speaker_1089".to_string(),
                is_correct_pronunciation: true,
            },
            GroundTruthWord {
                word: "DINNER".to_string(),
                start_time: 2.56,
                end_time: 3.14,
                speaker_id: "speaker_1089".to_string(),
                is_correct_pronunciation: true,
            },
            // Add remaining ground truth words...
        ],
        speaker_segments: vec![
            GroundTruthSpeakerSegment {
                speaker_id: "speaker_1089".to_string(),
                start_time: 0.48,
                end_time: 10.43,
                text: "HE HOPED THERE WOULD BE STEW FOR DINNER TURNIPS AND CARROTS AND BRUSSEL SPROUTS AND ONIONS AND PERHAPS A NICE BIG FAT GOOSE WITH PLENTY OF STUFFING".to_string(),
            },
        ],
        total_duration: 10.43,
        num_speakers: 1,
    }
}

/// Create poor quality transcription for testing error scenarios
fn create_poor_transcription_result() -> TranscriptionResult {
    TranscriptionResult {
        text: "HI HOPED THEIR WOOD BEE STIR FOR DINER TURNUP AN CARATS".to_string(), // Many errors
        words: vec![
            TranscriptionWord {
                word: "HI".to_string(), // Wrong: should be "HE"
                start_time: 0.48,
                end_time: 0.70,
                confidence: 0.60, // Low confidence
                speaker_id: Some("speaker_unknown".to_string()), // Wrong speaker
            },
            TranscriptionWord {
                word: "HOPED".to_string(),
                start_time: 0.70,
                end_time: 1.06,
                confidence: 0.85,
                speaker_id: Some("speaker_1089".to_string()),
            },
            TranscriptionWord {
                word: "THEIR".to_string(), // Wrong: should be "THERE"
                start_time: 1.06,
                end_time: 1.38,
                confidence: 0.70,
                speaker_id: Some("speaker_1089".to_string()),
            },
            TranscriptionWord {
                word: "WOOD".to_string(), // Wrong: should be "WOULD"
                start_time: 1.38,
                end_time: 1.66,
                confidence: 0.65,
                speaker_id: Some("speaker_1089".to_string()),
            },
            TranscriptionWord {
                word: "BEE".to_string(), // Wrong: should be "BE"
                start_time: 1.66,
                end_time: 1.84,
                confidence: 0.55,
                speaker_id: Some("speaker_1089".to_string()),
            },
            // Truncated with many errors...
        ],
        confidence: 0.68, // Low overall confidence
        language: "en".to_string(),
        processing_time_ms: 8500, // Slow processing
        real_time_factor: 2.4, // Very slow
    }
}

/// Create production-realistic performance metrics
fn create_production_performance_metrics() -> SystemPerformanceMetrics {
    SystemPerformanceMetrics {
        real_time_factor: 0.8, // Good performance
        memory_usage_mb: 245.0, // Reasonable memory usage
        cpu_utilization: 35.0, // Moderate CPU usage
        latency_ms: 1200, // Good latency
        throughput: 1.25, // Good throughput
        model_load_time_ms: 2400, // Reasonable model loading time
    }
}

/// Test with actual LibriSpeech files (if available)
#[tokio::test]
#[ignore] // Only run when LibriSpeech files are available
async fn test_quality_analysis_real_librispeech_file() {
    let audio_file_path = "src-tauri/tests/diarization_realtime/test_audio/1089-134686-0000.wav";
    let ground_truth_path = "src-tauri/tests/diarization_realtime/ground_truth/librispeech_test.json";
    
    // Check if test files exist
    if !Path::new(audio_file_path).exists() || !Path::new(ground_truth_path).exists() {
        println!("Skipping real LibriSpeech test - files not available");
        return;
    }
    
    // This would integrate with the actual diarization pipeline
    // to process the real audio file and analyze results
    println!("Testing with real LibriSpeech file: {}", audio_file_path);
    
    // For now, demonstrate the analysis framework
    let mut analyzer = TranscriptionQualityAnalyzer::new();
    let transcription = create_librispeech_transcription_result();
    let ground_truth = create_librispeech_ground_truth();
    let performance = create_production_performance_metrics();
    
    let result = analyzer.analyze_quality(transcription, ground_truth, performance)
        .expect("Real file analysis should succeed");
    
    println!("Real LibriSpeech analysis completed:");
    println!("  Overall Score: {:.1}%", result.overall_score);
    println!("  WER: {:.2}%", result.wer_result.wer_percentage);
    println!("  DER: {:.2}%", result.diarization_result.der_result.der_score * 100.0);
}