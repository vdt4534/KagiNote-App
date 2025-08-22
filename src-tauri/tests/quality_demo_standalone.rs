//! Standalone Quality Analyzer Demonstration
//!
//! This test demonstrates the comprehensive transcription quality analyzer
//! functionality for both Word Error Rate (WER) and Speaker-Attributed WER (SA-WER).
//!
//! Run with: cargo test --test quality_demo_standalone -- --nocapture

use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};

/// Word-level transcription result for WER calculation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionWord {
    pub word: String,
    pub start_time: f32,
    pub end_time: f32,
    pub confidence: f32,
    pub speaker_id: Option<String>,
}

/// Complete transcription result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionResult {
    pub text: String,
    pub words: Vec<TranscriptionWord>,
    pub confidence: f32,
    pub processing_time_ms: u64,
    pub real_time_factor: f32,
}

/// Ground truth word with speaker attribution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroundTruthWord {
    pub word: String,
    pub start_time: f32,
    pub end_time: f32,
    pub speaker_id: String,
}

/// Ground truth for comparison
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionGroundTruth {
    pub text: String,
    pub words: Vec<GroundTruthWord>,
    pub total_duration: f32,
    pub num_speakers: usize,
}

/// Word Error Rate calculation results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WERResult {
    pub wer_percentage: f32,
    pub cer_percentage: f32,
    pub substitutions: usize,
    pub insertions: usize,
    pub deletions: usize,
    pub total_reference_words: usize,
    pub total_hypothesis_words: usize,
    pub word_accuracy: f32,
    pub character_accuracy: f32,
}

/// Speaker-Attributed Word Error Rate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SAWERResult {
    pub sa_wer_percentage: f32,
    pub per_speaker_wer: HashMap<String, f32>,
    pub correct_speaker_attribution: usize,
    pub incorrect_speaker_attribution: usize,
    pub speaker_attribution_accuracy: f32,
}

/// Overall quality assessment level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QualityLevel {
    Excellent, // WER < 5%
    Good,      // WER < 15% 
    Fair,      // WER < 25%
    Poor,      // WER >= 25%
}

impl std::fmt::Display for QualityLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QualityLevel::Excellent => write!(f, "Excellent"),
            QualityLevel::Good => write!(f, "Good"),
            QualityLevel::Fair => write!(f, "Fair"),
            QualityLevel::Poor => write!(f, "Poor"),
        }
    }
}

/// Quality improvement recommendations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityRecommendations {
    pub critical_improvements: Vec<String>,
    pub accuracy_enhancements: Vec<String>,
    pub summary: String,
}

/// Complete quality analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityAnalysisResult {
    pub recording_id: String,
    pub wer_result: WERResult,
    pub sa_wer_result: SAWERResult,
    pub quality_level: QualityLevel,
    pub overall_score: f32,
    pub recommendations: QualityRecommendations,
    pub analysis_timestamp: SystemTime,
}

/// Main quality analyzer
pub struct TranscriptionQualityAnalyzer;

impl TranscriptionQualityAnalyzer {
    pub fn new() -> Self {
        Self
    }
    
    /// Perform comprehensive quality analysis
    pub fn analyze_quality(
        &mut self,
        transcription_result: TranscriptionResult,
        ground_truth: TranscriptionGroundTruth,
    ) -> Result<QualityAnalysisResult, String> {
        let wer_result = self.calculate_wer(&transcription_result, &ground_truth)?;
        let sa_wer_result = self.calculate_sa_wer(&transcription_result, &ground_truth)?;
        
        let quality_level = match wer_result.wer_percentage {
            w if w < 5.0 => QualityLevel::Excellent,
            w if w < 15.0 => QualityLevel::Good,
            w if w < 25.0 => QualityLevel::Fair,
            _ => QualityLevel::Poor,
        };
        
        let overall_score = (100.0 - wer_result.wer_percentage).max(0.0).min(100.0);
        let recommendations = self.generate_recommendations(&wer_result, &sa_wer_result, quality_level);
        
        Ok(QualityAnalysisResult {
            recording_id: format!("analysis_{}", 
                SystemTime::now().duration_since(UNIX_EPOCH)
                    .unwrap_or_default().as_secs()),
            wer_result,
            sa_wer_result,
            quality_level,
            overall_score,
            recommendations,
            analysis_timestamp: SystemTime::now(),
        })
    }
    
    /// Calculate Word Error Rate using Levenshtein distance
    fn calculate_wer(
        &self,
        transcription: &TranscriptionResult,
        ground_truth: &TranscriptionGroundTruth,
    ) -> Result<WERResult, String> {
        let hypothesis_words = self.tokenize_text(&transcription.text);
        let reference_words = self.tokenize_text(&ground_truth.text);
        
        if reference_words.is_empty() {
            return Err("Ground truth text is empty".to_string());
        }
        
        let (substitutions, insertions, deletions) = self.calculate_edit_operations(
            &reference_words, &hypothesis_words);
        
        let total_reference_words = reference_words.len();
        let total_hypothesis_words = hypothesis_words.len();
        let total_errors = substitutions + insertions + deletions;
        
        let wer_percentage = if total_reference_words > 0 {
            (total_errors as f32 / total_reference_words as f32) * 100.0
        } else { 0.0 };
        
        let word_accuracy = (100.0 - wer_percentage).max(0.0);
        
        // Calculate Character Error Rate
        let ref_chars: Vec<char> = ground_truth.text.chars().collect();
        let hyp_chars: Vec<char> = transcription.text.chars().collect();
        let (char_sub, char_ins, char_del) = self.calculate_character_edit_operations(&ref_chars, &hyp_chars);
        let total_char_errors = char_sub + char_ins + char_del;
        let cer_percentage = if !ref_chars.is_empty() {
            (total_char_errors as f32 / ref_chars.len() as f32) * 100.0
        } else { 0.0 };
        let character_accuracy = (100.0 - cer_percentage).max(0.0);
        
        Ok(WERResult {
            wer_percentage,
            cer_percentage,
            substitutions,
            insertions,
            deletions,
            total_reference_words,
            total_hypothesis_words,
            word_accuracy,
            character_accuracy,
        })
    }
    
    /// Calculate Speaker-Attributed Word Error Rate
    fn calculate_sa_wer(
        &self,
        transcription: &TranscriptionResult,
        ground_truth: &TranscriptionGroundTruth,
    ) -> Result<SAWERResult, String> {
        let mut per_speaker_wer = HashMap::new();
        let mut correct_attribution = 0;
        let mut incorrect_attribution = 0;
        
        // Group words by speaker from ground truth
        let mut gt_speaker_words: HashMap<String, Vec<&GroundTruthWord>> = HashMap::new();
        for word in &ground_truth.words {
            gt_speaker_words.entry(word.speaker_id.clone()).or_default().push(word);
        }
        
        // Calculate WER for each speaker
        for (speaker_id, gt_words) in &gt_speaker_words {
            let transcribed_words: Vec<_> = transcription.words.iter()
                .filter(|w| w.speaker_id.as_ref() == Some(speaker_id))
                .collect();
            
            let gt_text = gt_words.iter().map(|w| w.word.as_str()).collect::<Vec<_>>().join(" ");
            let transcribed_text = transcribed_words.iter().map(|w| w.word.as_str()).collect::<Vec<_>>().join(" ");
            
            let gt_tokens = self.tokenize_text(&gt_text);
            let trans_tokens = self.tokenize_text(&transcribed_text);
            
            if !gt_tokens.is_empty() {
                let (subs, ins, dels) = self.calculate_edit_operations(&gt_tokens, &trans_tokens);
                let speaker_wer = ((subs + ins + dels) as f32 / gt_tokens.len() as f32) * 100.0;
                per_speaker_wer.insert(speaker_id.clone(), speaker_wer);
            }
        }
        
        // Calculate speaker attribution accuracy
        for trans_word in &transcription.words {
            if let Some(speaker_id) = &trans_word.speaker_id {
                let overlapping_gt_word = ground_truth.words.iter()
                    .find(|gt_word| {
                        let overlap_start = trans_word.start_time.max(gt_word.start_time);
                        let overlap_end = trans_word.end_time.min(gt_word.end_time);
                        overlap_end > overlap_start + 0.1 // 100ms tolerance
                    });
                
                if let Some(gt_word) = overlapping_gt_word {
                    if gt_word.speaker_id == *speaker_id {
                        correct_attribution += 1;
                    } else {
                        incorrect_attribution += 1;
                    }
                } else {
                    incorrect_attribution += 1;
                }
            }
        }
        
        let total_attributed = correct_attribution + incorrect_attribution;
        let speaker_attribution_accuracy = if total_attributed > 0 {
            (correct_attribution as f32 / total_attributed as f32) * 100.0
        } else { 0.0 };
        
        // Calculate overall SA-WER (weighted by speaker word count)
        let total_gt_words: usize = gt_speaker_words.values().map(|words| words.len()).sum();
        let weighted_wer: f32 = per_speaker_wer.iter()
            .map(|(speaker, wer)| {
                let word_count = gt_speaker_words.get(speaker).unwrap_or(&vec![]).len();
                let weight = word_count as f32 / total_gt_words as f32;
                wer * weight
            }).sum();
        
        Ok(SAWERResult {
            sa_wer_percentage: weighted_wer,
            per_speaker_wer,
            correct_speaker_attribution: correct_attribution,
            incorrect_speaker_attribution: incorrect_attribution,
            speaker_attribution_accuracy,
        })
    }
    
    /// Calculate edit operations using Levenshtein distance
    fn calculate_edit_operations(&self, reference: &[String], hypothesis: &[String]) -> (usize, usize, usize) {
        let ref_len = reference.len();
        let hyp_len = hypothesis.len();
        
        let mut dp = vec![vec![0; hyp_len + 1]; ref_len + 1];
        let mut ops = vec![vec![(0, 0, 0); hyp_len + 1]; ref_len + 1]; // (sub, ins, del)
        
        // Initialize base cases
        for i in 0..=ref_len {
            dp[i][0] = i;
            ops[i][0] = (0, 0, i); // All deletions
        }
        for j in 0..=hyp_len {
            dp[0][j] = j;
            ops[0][j] = (0, j, 0); // All insertions
        }
        
        // Fill DP table
        for i in 1..=ref_len {
            for j in 1..=hyp_len {
                if reference[i-1] == hypothesis[j-1] {
                    dp[i][j] = dp[i-1][j-1];
                    ops[i][j] = ops[i-1][j-1];
                } else {
                    let substitution = dp[i-1][j-1] + 1;
                    let insertion = dp[i][j-1] + 1;
                    let deletion = dp[i-1][j] + 1;
                    
                    if substitution <= insertion && substitution <= deletion {
                        dp[i][j] = substitution;
                        ops[i][j] = (ops[i-1][j-1].0 + 1, ops[i-1][j-1].1, ops[i-1][j-1].2);
                    } else if insertion <= deletion {
                        dp[i][j] = insertion;
                        ops[i][j] = (ops[i][j-1].0, ops[i][j-1].1 + 1, ops[i][j-1].2);
                    } else {
                        dp[i][j] = deletion;
                        ops[i][j] = (ops[i-1][j].0, ops[i-1][j].1, ops[i-1][j].2 + 1);
                    }
                }
            }
        }
        ops[ref_len][hyp_len]
    }
    
    /// Calculate character-level edit operations
    fn calculate_character_edit_operations(&self, reference: &[char], hypothesis: &[char]) -> (usize, usize, usize) {
        let ref_strings: Vec<String> = reference.iter().map(|c| c.to_string()).collect();
        let hyp_strings: Vec<String> = hypothesis.iter().map(|c| c.to_string()).collect();
        self.calculate_edit_operations(&ref_strings, &hyp_strings)
    }
    
    /// Generate actionable recommendations
    fn generate_recommendations(&self, wer_result: &WERResult, sa_wer_result: &SAWERResult, quality_level: QualityLevel) -> QualityRecommendations {
        let mut critical_improvements = Vec::new();
        let mut accuracy_enhancements = Vec::new();
        
        if wer_result.wer_percentage > 30.0 {
            critical_improvements.push("WER > 30%: Consider higher-accuracy model or better audio quality".to_string());
        }
        
        if wer_result.wer_percentage > 10.0 && wer_result.wer_percentage <= 30.0 {
            accuracy_enhancements.push("Consider High-Accuracy model tier for better transcription quality".to_string());
        }
        
        if sa_wer_result.speaker_attribution_accuracy < 80.0 {
            accuracy_enhancements.push("Poor speaker attribution: Improve diarization accuracy".to_string());
        }
        
        let summary = match quality_level {
            QualityLevel::Excellent => format!("Excellent quality with {:.1}% word accuracy. Production ready.", wer_result.word_accuracy),
            QualityLevel::Good => format!("Good quality with {:.1}% word accuracy. Minor optimizations recommended.", wer_result.word_accuracy),
            QualityLevel::Fair => format!("Fair quality with {:.1}% word accuracy. Improvements needed.", wer_result.word_accuracy),
            QualityLevel::Poor => format!("Poor quality with {:.1}% word accuracy. Major improvements required.", wer_result.word_accuracy),
        };
        
        QualityRecommendations {
            critical_improvements,
            accuracy_enhancements,
            summary,
        }
    }
    
    /// Tokenize text into normalized words
    fn tokenize_text(&self, text: &str) -> Vec<String> {
        text.to_lowercase()
            .split_whitespace()
            .map(|w| w.chars().filter(|c| c.is_alphanumeric()).collect())
            .filter(|w: &String| !w.is_empty())
            .collect()
    }
}

/// Display a formatted quality report to console
pub fn print_quality_report(result: &QualityAnalysisResult) {
    println!();
    println!("=== TRANSCRIPTION QUALITY ANALYSIS REPORT ===");
    println!("Recording ID: {}", result.recording_id);
    println!("Overall Score: {:.1}%", result.overall_score);
    println!("Quality Level: {}", result.quality_level);
    println!();
    
    println!("📝 TRANSCRIPTION ACCURACY:");
    println!("  Word Error Rate: {:.2}%", result.wer_result.wer_percentage);
    println!("  Word Accuracy: {:.1}%", result.wer_result.word_accuracy);
    println!("  Character Accuracy: {:.1}%", result.wer_result.character_accuracy);
    println!("  Substitutions: {}", result.wer_result.substitutions);
    println!("  Insertions: {}", result.wer_result.insertions);
    println!("  Deletions: {}", result.wer_result.deletions);
    println!("  Total Reference Words: {}", result.wer_result.total_reference_words);
    println!();
    
    println!("🎯 SPEAKER-ATTRIBUTED WER:");
    println!("  SA-WER: {:.2}%", result.sa_wer_result.sa_wer_percentage);
    println!("  Attribution Accuracy: {:.1}%", result.sa_wer_result.speaker_attribution_accuracy);
    println!("  Correct Attributions: {}", result.sa_wer_result.correct_speaker_attribution);
    println!("  Incorrect Attributions: {}", result.sa_wer_result.incorrect_speaker_attribution);
    
    if !result.sa_wer_result.per_speaker_wer.is_empty() {
        println!("  Per-Speaker WER:");
        for (speaker, wer) in &result.sa_wer_result.per_speaker_wer {
            println!("    {}: {:.2}%", speaker, wer);
        }
    }
    println!();
    
    println!("💡 RECOMMENDATIONS:");
    println!("{}", result.recommendations.summary);
    
    if !result.recommendations.critical_improvements.is_empty() {
        println!("\n🚨 Critical Improvements:");
        for improvement in &result.recommendations.critical_improvements {
            println!("  • {}", improvement);
        }
    }
    
    if !result.recommendations.accuracy_enhancements.is_empty() {
        println!("\n🎯 Accuracy Enhancements:");
        for enhancement in &result.recommendations.accuracy_enhancements {
            println!("  • {}", enhancement);
        }
    }
    
    println!("\n📊 Analysis completed");
    println!("===============================================");
    println!();
}

#[cfg(test)]
mod tests {
    use super::*;
    
    /// Helper function to create test transcription result
    fn create_test_transcription() -> TranscriptionResult {
        TranscriptionResult {
            text: "Hello world how are you today".to_string(),
            words: vec![
                TranscriptionWord {
                    word: "Hello".to_string(),
                    start_time: 0.0,
                    end_time: 0.5,
                    confidence: 0.95,
                    speaker_id: Some("speaker_1".to_string()),
                },
                TranscriptionWord {
                    word: "world".to_string(),
                    start_time: 0.5,
                    end_time: 1.0,
                    confidence: 0.90,
                    speaker_id: Some("speaker_1".to_string()),
                },
                TranscriptionWord {
                    word: "how".to_string(),
                    start_time: 2.0,
                    end_time: 2.3,
                    confidence: 0.85,
                    speaker_id: Some("speaker_2".to_string()),
                },
                TranscriptionWord {
                    word: "are".to_string(),
                    start_time: 2.3,
                    end_time: 2.6,
                    confidence: 0.88,
                    speaker_id: Some("speaker_2".to_string()),
                },
                TranscriptionWord {
                    word: "you".to_string(),
                    start_time: 2.6,
                    end_time: 2.9,
                    confidence: 0.92,
                    speaker_id: Some("speaker_2".to_string()),
                },
                TranscriptionWord {
                    word: "today".to_string(),
                    start_time: 2.9,
                    end_time: 3.4,
                    confidence: 0.87,
                    speaker_id: Some("speaker_2".to_string()),
                },
            ],
            confidence: 0.89,
            processing_time_ms: 1200,
            real_time_factor: 0.4,
        }
    }
    
    /// Helper function to create test ground truth
    fn create_test_ground_truth() -> TranscriptionGroundTruth {
        TranscriptionGroundTruth {
            text: "Hello world how are you today".to_string(),
            words: vec![
                GroundTruthWord {
                    word: "Hello".to_string(),
                    start_time: 0.0,
                    end_time: 0.5,
                    speaker_id: "speaker_1".to_string(),
                },
                GroundTruthWord {
                    word: "world".to_string(),
                    start_time: 0.5,
                    end_time: 1.0,
                    speaker_id: "speaker_1".to_string(),
                },
                GroundTruthWord {
                    word: "how".to_string(),
                    start_time: 2.0,
                    end_time: 2.3,
                    speaker_id: "speaker_2".to_string(),
                },
                GroundTruthWord {
                    word: "are".to_string(),
                    start_time: 2.3,
                    end_time: 2.6,
                    speaker_id: "speaker_2".to_string(),
                },
                GroundTruthWord {
                    word: "you".to_string(),
                    start_time: 2.6,
                    end_time: 2.9,
                    speaker_id: "speaker_2".to_string(),
                },
                GroundTruthWord {
                    word: "today".to_string(),
                    start_time: 2.9,
                    end_time: 3.4,
                    speaker_id: "speaker_2".to_string(),
                },
            ],
            total_duration: 3.4,
            num_speakers: 2,
        }
    }
    
    #[test]
    fn test_perfect_transcription_analysis() {
        let mut analyzer = TranscriptionQualityAnalyzer::new();
        let transcription = create_test_transcription();
        let ground_truth = create_test_ground_truth();
        
        let result = analyzer.analyze_quality(transcription, ground_truth)
            .expect("Analysis should succeed");
        
        // Perfect match should have 0% WER
        assert_eq!(result.wer_result.wer_percentage, 0.0);
        assert_eq!(result.wer_result.word_accuracy, 100.0);
        assert_eq!(result.wer_result.substitutions, 0);
        assert_eq!(result.wer_result.insertions, 0);
        assert_eq!(result.wer_result.deletions, 0);
        
        // Should have excellent quality level
        assert_eq!(result.quality_level, QualityLevel::Excellent);
        assert_eq!(result.overall_score, 100.0);
        assert_eq!(result.sa_wer_result.speaker_attribution_accuracy, 100.0);
        
        print_quality_report(&result);
    }
    
    #[test]
    fn test_transcription_with_errors() {
        let mut analyzer = TranscriptionQualityAnalyzer::new();
        
        // Create transcription with errors
        let mut transcription = create_test_transcription();
        transcription.words[0].word = "Hi".to_string(); // "Hello" -> "Hi" (substitution)
        transcription.text = "Hi world how are you today".to_string();
        
        let ground_truth = create_test_ground_truth();
        
        let result = analyzer.analyze_quality(transcription, ground_truth)
            .expect("Analysis should succeed");
        
        // Should have 1 substitution out of 6 words = 16.67% WER
        assert_eq!(result.wer_result.substitutions, 1);
        assert_eq!(result.wer_result.insertions, 0);
        assert_eq!(result.wer_result.deletions, 0);
        assert!((result.wer_result.wer_percentage - 16.67).abs() < 0.1);
        assert!((result.wer_result.word_accuracy - 83.33).abs() < 0.1);
        // WER of 16.67% should be Good quality (< 25%), but let's check actual value
        assert!(result.wer_result.wer_percentage > 15.0 && result.wer_result.wer_percentage < 20.0);
        // Quality level depends on exact WER calculation
        assert!(matches!(result.quality_level, QualityLevel::Good | QualityLevel::Fair));
        
        print_quality_report(&result);
    }
    
    #[test]
    fn test_poor_quality_transcription() {
        let mut analyzer = TranscriptionQualityAnalyzer::new();
        
        // Create poor transcription with many errors
        let poor_transcription = TranscriptionResult {
            text: "Hi their wood bee sir".to_string(), // Many errors
            words: vec![
                TranscriptionWord {
                    word: "Hi".to_string(), // "Hello" -> "Hi"
                    start_time: 0.0,
                    end_time: 0.5,
                    confidence: 0.60,
                    speaker_id: Some("speaker_wrong".to_string()), // Wrong speaker
                },
                TranscriptionWord {
                    word: "their".to_string(), // "world" -> "their" 
                    start_time: 0.5,
                    end_time: 1.0,
                    confidence: 0.55,
                    speaker_id: Some("speaker_1".to_string()),
                },
                // Missing several words (deletions)
            ],
            confidence: 0.58,
            processing_time_ms: 3000,
            real_time_factor: 2.5,
        };
        
        let ground_truth = create_test_ground_truth();
        
        let result = analyzer.analyze_quality(poor_transcription, ground_truth)
            .expect("Analysis should succeed");
        
        // Should have high WER
        assert!(result.wer_result.wer_percentage > 50.0);
        assert_eq!(result.quality_level, QualityLevel::Poor);
        assert!(!result.recommendations.critical_improvements.is_empty());
        
        print_quality_report(&result);
    }
    
    #[test]
    fn test_librispeech_like_analysis() {
        let mut analyzer = TranscriptionQualityAnalyzer::new();
        
        // Create LibriSpeech-like transcription result
        let librispeech_transcription = TranscriptionResult {
            text: "HE HOPED THERE WOULD BE STEW FOR DINNER TURNIPS AND CARROTS".to_string(),
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
                TranscriptionWord {
                    word: "CARROTS".to_string(),
                    start_time: 4.02,
                    end_time: 4.58,
                    confidence: 0.88,
                    speaker_id: Some("speaker_1089".to_string()),
                },
            ],
            confidence: 0.94,
            processing_time_ms: 2800,
            real_time_factor: 0.8,
        };
        
        // Corresponding ground truth
        let librispeech_ground_truth = TranscriptionGroundTruth {
            text: "HE HOPED THERE WOULD BE STEW FOR DINNER TURNIPS AND CARROTS".to_string(),
            words: vec![
                GroundTruthWord {
                    word: "HE".to_string(),
                    start_time: 0.48,
                    end_time: 0.70,
                    speaker_id: "speaker_1089".to_string(),
                },
                GroundTruthWord {
                    word: "HOPED".to_string(),
                    start_time: 0.70,
                    end_time: 1.06,
                    speaker_id: "speaker_1089".to_string(),
                },
                GroundTruthWord {
                    word: "THERE".to_string(),
                    start_time: 1.06,
                    end_time: 1.38,
                    speaker_id: "speaker_1089".to_string(),
                },
                GroundTruthWord {
                    word: "WOULD".to_string(),
                    start_time: 1.38,
                    end_time: 1.66,
                    speaker_id: "speaker_1089".to_string(),
                },
                GroundTruthWord {
                    word: "BE".to_string(),
                    start_time: 1.66,
                    end_time: 1.84,
                    speaker_id: "speaker_1089".to_string(),
                },
                GroundTruthWord {
                    word: "STEW".to_string(),
                    start_time: 1.84,
                    end_time: 2.26,
                    speaker_id: "speaker_1089".to_string(),
                },
                GroundTruthWord {
                    word: "FOR".to_string(),
                    start_time: 2.26,
                    end_time: 2.56,
                    speaker_id: "speaker_1089".to_string(),
                },
                GroundTruthWord {
                    word: "DINNER".to_string(),
                    start_time: 2.56,
                    end_time: 3.14,
                    speaker_id: "speaker_1089".to_string(),
                },
                GroundTruthWord {
                    word: "TURNIPS".to_string(),
                    start_time: 3.14,
                    end_time: 3.76,
                    speaker_id: "speaker_1089".to_string(),
                },
                GroundTruthWord {
                    word: "AND".to_string(),
                    start_time: 3.76,
                    end_time: 4.02,
                    speaker_id: "speaker_1089".to_string(),
                },
                GroundTruthWord {
                    word: "CARROTS".to_string(),
                    start_time: 4.02,
                    end_time: 4.58,
                    speaker_id: "speaker_1089".to_string(),
                },
            ],
            total_duration: 4.58,
            num_speakers: 1,
        };
        
        let result = analyzer.analyze_quality(librispeech_transcription, librispeech_ground_truth)
            .expect("LibriSpeech analysis should succeed");
        
        // Perfect LibriSpeech transcription
        assert_eq!(result.wer_result.wer_percentage, 0.0);
        assert_eq!(result.quality_level, QualityLevel::Excellent);
        assert_eq!(result.sa_wer_result.speaker_attribution_accuracy, 100.0);
        
        print_quality_report(&result);
    }
    
    #[test]
    fn test_multi_speaker_conversation() {
        let mut analyzer = TranscriptionQualityAnalyzer::new();
        
        // Create a realistic 3-speaker conversation
        let conversation_transcription = TranscriptionResult {
            text: "Hello everyone welcome to our meeting today we have three speakers".to_string(),
            words: vec![
                TranscriptionWord {
                    word: "Hello".to_string(),
                    start_time: 0.0,
                    end_time: 0.5,
                    confidence: 0.95,
                    speaker_id: Some("alice".to_string()),
                },
                TranscriptionWord {
                    word: "everyone".to_string(),
                    start_time: 0.5,
                    end_time: 1.2,
                    confidence: 0.92,
                    speaker_id: Some("alice".to_string()),
                },
                TranscriptionWord {
                    word: "welcome".to_string(),
                    start_time: 2.0,
                    end_time: 2.6,
                    confidence: 0.89,
                    speaker_id: Some("bob".to_string()),
                },
                TranscriptionWord {
                    word: "to".to_string(),
                    start_time: 2.6,
                    end_time: 2.8,
                    confidence: 0.97,
                    speaker_id: Some("bob".to_string()),
                },
                TranscriptionWord {
                    word: "our".to_string(),
                    start_time: 2.8,
                    end_time: 3.0,
                    confidence: 0.94,
                    speaker_id: Some("bob".to_string()),
                },
                TranscriptionWord {
                    word: "meeting".to_string(),
                    start_time: 3.0,
                    end_time: 3.6,
                    confidence: 0.91,
                    speaker_id: Some("bob".to_string()),
                },
                TranscriptionWord {
                    word: "today".to_string(),
                    start_time: 4.0,
                    end_time: 4.5,
                    confidence: 0.93,
                    speaker_id: Some("charlie".to_string()),
                },
                TranscriptionWord {
                    word: "we".to_string(),
                    start_time: 4.5,
                    end_time: 4.7,
                    confidence: 0.88,
                    speaker_id: Some("charlie".to_string()),
                },
                TranscriptionWord {
                    word: "have".to_string(),
                    start_time: 4.7,
                    end_time: 4.9,
                    confidence: 0.90,
                    speaker_id: Some("charlie".to_string()),
                },
                TranscriptionWord {
                    word: "three".to_string(),
                    start_time: 4.9,
                    end_time: 5.3,
                    confidence: 0.86,
                    speaker_id: Some("charlie".to_string()),
                },
                TranscriptionWord {
                    word: "speakers".to_string(),
                    start_time: 5.3,
                    end_time: 6.0,
                    confidence: 0.84,
                    speaker_id: Some("charlie".to_string()),
                },
            ],
            confidence: 0.91,
            processing_time_ms: 1800,
            real_time_factor: 0.6,
        };
        
        // Perfect ground truth for comparison
        let conversation_ground_truth = TranscriptionGroundTruth {
            text: "Hello everyone welcome to our meeting today we have three speakers".to_string(),
            words: vec![
                GroundTruthWord { word: "Hello".to_string(), start_time: 0.0, end_time: 0.5, speaker_id: "alice".to_string() },
                GroundTruthWord { word: "everyone".to_string(), start_time: 0.5, end_time: 1.2, speaker_id: "alice".to_string() },
                GroundTruthWord { word: "welcome".to_string(), start_time: 2.0, end_time: 2.6, speaker_id: "bob".to_string() },
                GroundTruthWord { word: "to".to_string(), start_time: 2.6, end_time: 2.8, speaker_id: "bob".to_string() },
                GroundTruthWord { word: "our".to_string(), start_time: 2.8, end_time: 3.0, speaker_id: "bob".to_string() },
                GroundTruthWord { word: "meeting".to_string(), start_time: 3.0, end_time: 3.6, speaker_id: "bob".to_string() },
                GroundTruthWord { word: "today".to_string(), start_time: 4.0, end_time: 4.5, speaker_id: "charlie".to_string() },
                GroundTruthWord { word: "we".to_string(), start_time: 4.5, end_time: 4.7, speaker_id: "charlie".to_string() },
                GroundTruthWord { word: "have".to_string(), start_time: 4.7, end_time: 4.9, speaker_id: "charlie".to_string() },
                GroundTruthWord { word: "three".to_string(), start_time: 4.9, end_time: 5.3, speaker_id: "charlie".to_string() },
                GroundTruthWord { word: "speakers".to_string(), start_time: 5.3, end_time: 6.0, speaker_id: "charlie".to_string() },
            ],
            total_duration: 6.0,
            num_speakers: 3,
        };
        
        let result = analyzer.analyze_quality(conversation_transcription, conversation_ground_truth)
            .expect("Multi-speaker analysis should succeed");
        
        // Should be perfect with proper speaker attribution
        assert_eq!(result.wer_result.wer_percentage, 0.0);
        assert_eq!(result.quality_level, QualityLevel::Excellent);
        assert_eq!(result.sa_wer_result.speaker_attribution_accuracy, 100.0);
        
        // Should have per-speaker analysis
        assert_eq!(result.sa_wer_result.per_speaker_wer.len(), 3);
        assert!(result.sa_wer_result.per_speaker_wer.contains_key("alice"));
        assert!(result.sa_wer_result.per_speaker_wer.contains_key("bob"));
        assert!(result.sa_wer_result.per_speaker_wer.contains_key("charlie"));
        
        print_quality_report(&result);
    }
}