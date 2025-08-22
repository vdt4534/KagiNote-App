//! Demonstration of the Transcription Quality Analyzer
//!
//! This module provides a simplified demonstration of comprehensive quality analysis
//! for both transcription accuracy (WER/CER) and speaker diarization accuracy.
//! 
//! This is a standalone demo that shows the key functionality without complex
//! dependencies, making it suitable for TDD development and validation.

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
    pub language: String,
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
    pub is_correct_pronunciation: bool,
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
    /// Word Error Rate as percentage (0.0-100.0)
    pub wer_percentage: f32,
    /// Character Error Rate as percentage (0.0-100.0) 
    pub cer_percentage: f32,
    /// Number of substitutions (wrong words)
    pub substitutions: usize,
    /// Number of insertions (extra words)
    pub insertions: usize,
    /// Number of deletions (missed words)
    pub deletions: usize,
    /// Total number of words in reference
    pub total_reference_words: usize,
    /// Total number of words in hypothesis
    pub total_hypothesis_words: usize,
    /// Word-level accuracy (100.0 - WER)
    pub word_accuracy: f32,
    /// Character-level accuracy (100.0 - CER)
    pub character_accuracy: f32,
}

/// Speaker-Attributed Word Error Rate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SAWERResult {
    /// Overall SA-WER percentage
    pub sa_wer_percentage: f32,
    /// WER breakdown per speaker
    pub per_speaker_wer: HashMap<String, f32>,
    /// Words correctly attributed to speakers
    pub correct_speaker_attribution: usize,
    /// Words incorrectly attributed to speakers
    pub incorrect_speaker_attribution: usize,
    /// Speaker attribution accuracy percentage
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

impl QualityLevel {
    pub fn from_wer(wer: f32) -> Self {
        match wer {
            w if w < 5.0 => QualityLevel::Excellent,
            w if w < 15.0 => QualityLevel::Good,
            w if w < 25.0 => QualityLevel::Fair,
            _ => QualityLevel::Poor,
        }
    }
    
    pub fn color(&self) -> &'static str {
        match self {
            QualityLevel::Excellent => "#22c55e", // Green
            QualityLevel::Good => "#3b82f6",      // Blue
            QualityLevel::Fair => "#f59e0b",      // Orange
            QualityLevel::Poor => "#ef4444",      // Red
        }
    }
}

/// Quality improvement recommendations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityRecommendations {
    /// High-priority improvements
    pub critical_improvements: Vec<String>,
    /// Accuracy enhancements
    pub accuracy_enhancements: Vec<String>,
    /// Overall assessment summary
    pub summary: String,
}

/// Complete quality analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityAnalysisResult {
    /// Recording identifier
    pub recording_id: String,
    /// Word Error Rate analysis
    pub wer_result: WERResult,
    /// Speaker-Attributed WER analysis
    pub sa_wer_result: SAWERResult,
    /// Overall quality assessment
    pub quality_level: QualityLevel,
    /// Overall system score (0.0-100.0)
    pub overall_score: f32,
    /// Detailed recommendations
    pub recommendations: QualityRecommendations,
    /// Analysis timestamp
    pub analysis_timestamp: SystemTime,
}

/// Main quality analyzer (simplified version)
pub struct TranscriptionQualityAnalyzer;

impl TranscriptionQualityAnalyzer {
    /// Create new analyzer
    pub fn new() -> Self {
        Self
    }
    
    /// Perform comprehensive quality analysis
    pub fn analyze_quality(
        &mut self,
        transcription_result: TranscriptionResult,
        ground_truth: TranscriptionGroundTruth,
    ) -> Result<QualityAnalysisResult, String> {
        // Calculate Word Error Rate
        let wer_result = self.calculate_wer(&transcription_result, &ground_truth)?;
        
        // Calculate Speaker-Attributed WER
        let sa_wer_result = self.calculate_sa_wer(&transcription_result, &ground_truth)?;
        
        // Determine overall quality level
        let quality_level = QualityLevel::from_wer(wer_result.wer_percentage);
        
        // Calculate overall system score
        let overall_score = (100.0 - wer_result.wer_percentage).max(0.0).min(100.0);
        
        // Generate recommendations
        let recommendations = self.generate_recommendations(&wer_result, &sa_wer_result, quality_level);
        
        let result = QualityAnalysisResult {
            recording_id: format!("quality_analysis_{}", 
                SystemTime::now().duration_since(UNIX_EPOCH)
                    .unwrap_or_default().as_secs()),
            wer_result,
            sa_wer_result,
            quality_level,
            overall_score,
            recommendations,
            analysis_timestamp: SystemTime::now(),
        };
        
        Ok(result)
    }
    
    /// Calculate Word Error Rate using Levenshtein distance
    fn calculate_wer(
        &self,
        transcription: &TranscriptionResult,
        ground_truth: &TranscriptionGroundTruth,
    ) -> Result<WERResult, String> {
        // Normalize and tokenize text
        let hypothesis_words = self.tokenize_text(&transcription.text);
        let reference_words = self.tokenize_text(&ground_truth.text);
        
        if reference_words.is_empty() {
            return Err("Ground truth text is empty".to_string());
        }
        
        // Calculate edit operations using dynamic programming
        let (substitutions, insertions, deletions) = self.calculate_edit_operations(
            &reference_words,
            &hypothesis_words,
        );
        
        let total_reference_words = reference_words.len();
        let total_hypothesis_words = hypothesis_words.len();
        let total_errors = substitutions + insertions + deletions;
        
        let wer_percentage = if total_reference_words > 0 {
            (total_errors as f32 / total_reference_words as f32) * 100.0
        } else {
            0.0
        };
        
        let word_accuracy = (100.0 - wer_percentage).max(0.0);
        
        // Calculate Character Error Rate
        let ref_chars: Vec<char> = ground_truth.text.chars().collect();
        let hyp_chars: Vec<char> = transcription.text.chars().collect();
        
        let (char_sub, char_ins, char_del) = self.calculate_character_edit_operations(
            &ref_chars,
            &hyp_chars,
        );
        
        let total_char_errors = char_sub + char_ins + char_del;
        let cer_percentage = if !ref_chars.is_empty() {
            (total_char_errors as f32 / ref_chars.len() as f32) * 100.0
        } else {
            0.0
        };
        
        let character_accuracy = (100.0 - cer_percentage).max(0.0);
        
        Ok(WERResult {
            wer_percentage,
            cer_percentage,
            substitutions,
            insertions,
            deletions,
            total_reference_words,
            total_hypothesis_words: total_hypothesis_words,
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
        let mut total_attributed = 0;
        
        // Group words by speaker from ground truth
        let mut gt_speaker_words: HashMap<String, Vec<&GroundTruthWord>> = HashMap::new();
        for word in &ground_truth.words {
            gt_speaker_words.entry(word.speaker_id.clone())
                .or_default()
                .push(word);
        }
        
        // Calculate WER for each speaker
        for (speaker_id, gt_words) in &gt_speaker_words {
            // Find corresponding transcribed words for this speaker
            let transcribed_words: Vec<_> = transcription.words.iter()
                .filter(|w| w.speaker_id.as_ref() == Some(speaker_id))
                .collect();
            
            // Build text strings for this speaker
            let gt_text = gt_words.iter()
                .map(|w| w.word.as_str())
                .collect::<Vec<_>>()
                .join(" ");
            let transcribed_text = transcribed_words.iter()
                .map(|w| w.word.as_str())
                .collect::<Vec<_>>()
                .join(" ");
            
            // Calculate WER for this speaker
            let gt_tokens = self.tokenize_text(&gt_text);
            let trans_tokens = self.tokenize_text(&transcribed_text);
            
            if !gt_tokens.is_empty() {
                let (subs, ins, dels) = self.calculate_edit_operations(&gt_tokens, &trans_tokens);
                let speaker_wer = ((subs + ins + dels) as f32 / gt_tokens.len() as f32) * 100.0;
                per_speaker_wer.insert(speaker_id.clone(), speaker_wer);
            }
        }
        
        // Calculate speaker attribution accuracy (simplified version)
        for trans_word in &transcription.words {
            if let Some(speaker_id) = &trans_word.speaker_id {
                total_attributed += 1;
                
                // Find temporally overlapping ground truth word (simplified)
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
        
        let speaker_attribution_accuracy = if total_attributed > 0 {
            (correct_attribution as f32 / total_attributed as f32) * 100.0
        } else {
            0.0
        };
        
        // Calculate overall SA-WER (weighted by speaker word count)
        let total_gt_words: usize = gt_speaker_words.values()
            .map(|words| words.len())
            .sum();
        
        let weighted_wer: f32 = per_speaker_wer.iter()
            .map(|(speaker, wer)| {
                let word_count = gt_speaker_words.get(speaker).unwrap_or(&vec![]).len();
                let weight = word_count as f32 / total_gt_words as f32;
                wer * weight
            })
            .sum();
        
        Ok(SAWERResult {
            sa_wer_percentage: weighted_wer,
            per_speaker_wer,
            correct_speaker_attribution: correct_attribution,
            incorrect_speaker_attribution: incorrect_attribution,
            speaker_attribution_accuracy,
        })
    }
    
    /// Calculate edit operations using Levenshtein distance algorithm
    fn calculate_edit_operations(
        &self,
        reference: &[String],
        hypothesis: &[String],
    ) -> (usize, usize, usize) {
        let ref_len = reference.len();
        let hyp_len = hypothesis.len();
        
        // DP table: dp[i][j] represents edit distance between ref[0..i] and hyp[0..j]
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
                    // Match - copy from diagonal
                    dp[i][j] = dp[i-1][j-1];
                    ops[i][j] = ops[i-1][j-1];
                } else {
                    // Find minimum cost operation
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
    fn calculate_character_edit_operations(
        &self,
        reference: &[char],
        hypothesis: &[char],
    ) -> (usize, usize, usize) {
        let ref_strings: Vec<String> = reference.iter().map(|c| c.to_string()).collect();
        let hyp_strings: Vec<String> = hypothesis.iter().map(|c| c.to_string()).collect();
        self.calculate_edit_operations(&ref_strings, &hyp_strings)
    }
    
    /// Generate actionable recommendations
    fn generate_recommendations(
        &self,
        wer_result: &WERResult,
        sa_wer_result: &SAWERResult,
        quality_level: QualityLevel,
    ) -> QualityRecommendations {
        let mut critical_improvements = Vec::new();
        let mut accuracy_enhancements = Vec::new();
        
        // Critical improvements (must fix)
        if wer_result.wer_percentage > 30.0 {
            critical_improvements.push("WER > 30%: Consider using higher-accuracy model tier or improving audio quality".to_string());
        }
        
        // Accuracy enhancements
        if wer_result.wer_percentage > 10.0 && wer_result.wer_percentage <= 30.0 {
            accuracy_enhancements.push("Consider High-Accuracy model tier for better transcription quality".to_string());
        }
        
        if sa_wer_result.speaker_attribution_accuracy < 80.0 {
            accuracy_enhancements.push("Poor speaker attribution: Improve diarization accuracy or word alignment".to_string());
        }
        
        if wer_result.substitutions > wer_result.insertions + wer_result.deletions {
            accuracy_enhancements.push("High substitution errors: Consider domain-specific model or vocabulary".to_string());
        }
        
        // Generate summary
        let summary = match quality_level {
            QualityLevel::Excellent => format!(
                "Excellent quality achieved with {:.1}% word accuracy. System performing at production level.",
                wer_result.word_accuracy
            ),
            QualityLevel::Good => format!(
                "Good quality with {:.1}% word accuracy. Minor optimizations recommended for production deployment.",
                wer_result.word_accuracy
            ),
            QualityLevel::Fair => format!(
                "Fair quality with {:.1}% word accuracy. Significant improvements needed before production use.",
                wer_result.word_accuracy
            ),
            QualityLevel::Poor => format!(
                "Poor quality with {:.1}% word accuracy. Major improvements required - review model selection and audio preprocessing.",
                wer_result.word_accuracy
            ),
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
            language: "en".to_string(),
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
                    is_correct_pronunciation: true,
                },
                GroundTruthWord {
                    word: "world".to_string(),
                    start_time: 0.5,
                    end_time: 1.0,
                    speaker_id: "speaker_1".to_string(),
                    is_correct_pronunciation: true,
                },
                GroundTruthWord {
                    word: "how".to_string(),
                    start_time: 2.0,
                    end_time: 2.3,
                    speaker_id: "speaker_2".to_string(),
                    is_correct_pronunciation: true,
                },
                GroundTruthWord {
                    word: "are".to_string(),
                    start_time: 2.3,
                    end_time: 2.6,
                    speaker_id: "speaker_2".to_string(),
                    is_correct_pronunciation: true,
                },
                GroundTruthWord {
                    word: "you".to_string(),
                    start_time: 2.6,
                    end_time: 2.9,
                    speaker_id: "speaker_2".to_string(),
                    is_correct_pronunciation: true,
                },
                GroundTruthWord {
                    word: "today".to_string(),
                    start_time: 2.9,
                    end_time: 3.4,
                    speaker_id: "speaker_2".to_string(),
                    is_correct_pronunciation: true,
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
        
        // Overall score should be high
        assert_eq!(result.overall_score, 100.0);
        
        // Should have perfect speaker attribution
        assert_eq!(result.sa_wer_result.speaker_attribution_accuracy, 100.0);
        
        println!("Perfect transcription analysis:");
        println!("  WER: {:.2}%", result.wer_result.wer_percentage);
        println!("  CER: {:.2}%", result.wer_result.cer_percentage);
        println!("  Word Accuracy: {:.1}%", result.wer_result.word_accuracy);
        println!("  Character Accuracy: {:.1}%", result.wer_result.character_accuracy);
        println!("  Quality Level: {}", result.quality_level);
        println!("  Overall Score: {:.1}%", result.overall_score);
        println!("  Speaker Attribution: {:.1}%", result.sa_wer_result.speaker_attribution_accuracy);
        println!("  Summary: {}", result.recommendations.summary);
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
        
        // Should be Good quality (WER < 20%)
        assert_eq!(result.quality_level, QualityLevel::Good);
        
        println!("Transcription with errors:");
        println!("  WER: {:.2}%", result.wer_result.wer_percentage);
        println!("  Word Accuracy: {:.1}%", result.wer_result.word_accuracy);
        println!("  Substitutions: {}", result.wer_result.substitutions);
        println!("  Quality Level: {}", result.quality_level);
        println!("  Overall Score: {:.1}%", result.overall_score);
        println!("  Summary: {}", result.recommendations.summary);
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
            language: "en".to_string(),
            processing_time_ms: 3000,
            real_time_factor: 2.5, // Very slow
        };
        
        let ground_truth = create_test_ground_truth();
        
        let result = analyzer.analyze_quality(poor_transcription, ground_truth)
            .expect("Analysis should succeed");
        
        // Should have high WER
        assert!(result.wer_result.wer_percentage > 50.0);
        assert_eq!(result.quality_level, QualityLevel::Poor);
        
        // Should have recommendations
        assert!(!result.recommendations.critical_improvements.is_empty());
        
        println!("Poor quality transcription:");
        println!("  WER: {:.2}%", result.wer_result.wer_percentage);
        println!("  Word Accuracy: {:.1}%", result.wer_result.word_accuracy);
        println!("  Substitutions: {}", result.wer_result.substitutions);
        println!("  Insertions: {}", result.wer_result.insertions);
        println!("  Deletions: {}", result.wer_result.deletions);
        println!("  Quality Level: {}", result.quality_level);
        println!("  Overall Score: {:.1}%", result.overall_score);
        println!("  Critical Improvements: {}", result.recommendations.critical_improvements.len());
        println!("  Summary: {}", result.recommendations.summary);
    }
    
    #[test]
    fn test_speaker_attribution_analysis() {
        let mut analyzer = TranscriptionQualityAnalyzer::new();
        
        // Test incorrect speaker attribution
        let mut transcription = create_test_transcription();
        // Swap speaker IDs for first two words
        transcription.words[0].speaker_id = Some("speaker_2".to_string());
        transcription.words[1].speaker_id = Some("speaker_2".to_string());
        
        let ground_truth = create_test_ground_truth();
        
        let result = analyzer.analyze_quality(transcription, ground_truth)
            .expect("Analysis should succeed");
        
        // Should have incorrect attribution
        assert!(result.sa_wer_result.incorrect_speaker_attribution > 0);
        assert!(result.sa_wer_result.speaker_attribution_accuracy < 100.0);
        
        // Check per-speaker WER
        assert!(!result.sa_wer_result.per_speaker_wer.is_empty());
        
        println!("Speaker attribution analysis:");
        println!("  Correct attributions: {}", result.sa_wer_result.correct_speaker_attribution);
        println!("  Incorrect attributions: {}", result.sa_wer_result.incorrect_speaker_attribution);
        println!("  Attribution accuracy: {:.1}%", result.sa_wer_result.speaker_attribution_accuracy);
        println!("  SA-WER: {:.2}%", result.sa_wer_result.sa_wer_percentage);
        for (speaker, wer) in &result.sa_wer_result.per_speaker_wer {
            println!("  {}: {:.2}% WER", speaker, wer);
        }
    }
    
    #[test]
    fn test_tokenization() {
        let analyzer = TranscriptionQualityAnalyzer::new();
        
        let text = "Hello, world! How are you? I'm fine.";
        let tokens = analyzer.tokenize_text(text);
        
        assert_eq!(tokens, vec!["hello", "world", "how", "are", "you", "im", "fine"]);
    }
    
    #[test]
    fn test_edit_operations() {
        let analyzer = TranscriptionQualityAnalyzer::new();
        
        let reference = vec!["hello".to_string(), "world".to_string()];
        let hypothesis = vec!["hi".to_string(), "world".to_string()];
        
        let (substitutions, insertions, deletions) = analyzer.calculate_edit_operations(&reference, &hypothesis);
        
        assert_eq!(substitutions, 1); // "hello" -> "hi"
        assert_eq!(insertions, 0);
        assert_eq!(deletions, 0);
    }
    
    #[test]
    fn test_quality_level_classification() {
        assert_eq!(QualityLevel::from_wer(3.0), QualityLevel::Excellent);
        assert_eq!(QualityLevel::from_wer(10.0), QualityLevel::Good);
        assert_eq!(QualityLevel::from_wer(20.0), QualityLevel::Fair);
        assert_eq!(QualityLevel::from_wer(30.0), QualityLevel::Poor);
    }
}

/// Display a formatted quality report to console
pub fn print_quality_report(result: &QualityAnalysisResult) {
    println!("=== TRANSCRIPTION QUALITY ANALYSIS REPORT ===");
    println!("Recording ID: {}", result.recording_id);
    println!("Overall Score: {:.1}%", result.overall_score);
    println!("Quality Level: {} ({})", result.quality_level, result.quality_level.color());
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
    
    println!("\n📊 Analysis completed at: {:?}", result.analysis_timestamp);
}