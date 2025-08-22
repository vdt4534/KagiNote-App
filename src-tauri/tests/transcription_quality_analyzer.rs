//! Comprehensive Quality Analyzer for Transcription and Diarization
//!
//! This module provides detailed analysis of both transcription accuracy (WER) and
//! speaker diarization accuracy (DER) using real system output. It implements
//! industry-standard metrics and provides actionable insights for system improvement.

use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};
use thiserror::Error;

// Import main library and validation framework
use kaginote_lib;

// Import types from the diarization validation framework
use crate::diarization_realtime::validation::{
    DiarizationValidator, ValidationConfig, GroundTruthSegment,
    ValidationResult as DiarizationValidationResult
};

/// Word-level transcription result for WER calculation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionWord {
    pub word: String,
    pub start_time: f32,
    pub end_time: f32,
    pub confidence: f32,
    pub speaker_id: Option<String>,
}

/// Complete transcription result with speaker attribution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionResult {
    pub text: String,
    pub words: Vec<TranscriptionWord>,
    pub confidence: f32,
    pub language: String,
    pub processing_time_ms: u64,
    pub real_time_factor: f32,
}

/// Ground truth transcription for comparison
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionGroundTruth {
    pub text: String,
    pub words: Vec<GroundTruthWord>,
    pub speaker_segments: Vec<GroundTruthSpeakerSegment>,
    pub total_duration: f32,
    pub num_speakers: usize,
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

/// Ground truth speaker segment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroundTruthSpeakerSegment {
    pub speaker_id: String,
    pub start_time: f32,
    pub end_time: f32,
    pub text: String,
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
    /// Total words with speaker labels
    pub total_attributed_words: usize,
    /// Speaker attribution accuracy percentage
    pub speaker_attribution_accuracy: f32,
}

/// Combined system performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemPerformanceMetrics {
    /// Real-time processing factor
    pub real_time_factor: f32,
    /// Memory usage in MB
    pub memory_usage_mb: f32,
    /// CPU utilization percentage
    pub cpu_utilization: f32,
    /// Processing latency in milliseconds
    pub latency_ms: u64,
    /// Throughput (audio seconds per real second)
    pub throughput: f32,
    /// Model loading time in milliseconds
    pub model_load_time_ms: u64,
}

/// Overall quality assessment level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QualityLevel {
    Excellent, // WER < 5%, DER < 10%
    Good,      // WER < 15%, DER < 20%
    Fair,      // WER < 25%, DER < 30%
    Poor,      // WER >= 25% or DER >= 30%
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
    pub fn from_wer_der(wer: f32, der: f32) -> Self {
        match (wer, der) {
            (w, d) if w < 5.0 && d < 10.0 => QualityLevel::Excellent,
            (w, d) if w < 15.0 && d < 20.0 => QualityLevel::Good,
            (w, d) if w < 25.0 && d < 30.0 => QualityLevel::Fair,
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

/// Comprehensive quality analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityAnalysisResult {
    /// Recording identifier
    pub recording_id: String,
    /// Word Error Rate analysis
    pub wer_result: WERResult,
    /// Speaker-Attributed WER analysis
    pub sa_wer_result: SAWERResult,
    /// Diarization validation result
    pub diarization_result: DiarizationValidationResult,
    /// System performance metrics
    pub performance_metrics: SystemPerformanceMetrics,
    /// Overall quality assessment
    pub quality_level: QualityLevel,
    /// Overall system score (0.0-100.0)
    pub overall_score: f32,
    /// Detailed recommendations
    pub recommendations: QualityRecommendations,
    /// Analysis timestamp
    pub analysis_timestamp: SystemTime,
    /// Processing duration
    pub analysis_duration_ms: u64,
}

/// Quality improvement recommendations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityRecommendations {
    /// High-priority improvements
    pub critical_improvements: Vec<String>,
    /// Performance optimizations
    pub performance_optimizations: Vec<String>,
    /// Accuracy enhancements
    pub accuracy_enhancements: Vec<String>,
    /// Configuration suggestions
    pub config_suggestions: Vec<String>,
    /// Overall assessment summary
    pub summary: String,
}

/// Quality analysis configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityAnalysisConfig {
    /// Include character-level error analysis
    pub include_character_analysis: bool,
    /// Include per-speaker WER breakdown
    pub include_per_speaker_analysis: bool,
    /// Include detailed word alignment
    pub include_word_alignment: bool,
    /// Generate HTML report
    pub generate_html_report: bool,
    /// Generate JSON report
    pub generate_json_report: bool,
    /// Output directory for reports
    pub report_output_dir: String,
    /// Time tolerance for word alignment (seconds)
    pub word_alignment_tolerance_s: f32,
}

impl Default for QualityAnalysisConfig {
    fn default() -> Self {
        Self {
            include_character_analysis: true,
            include_per_speaker_analysis: true,
            include_word_alignment: false, // Can be expensive
            generate_html_report: true,
            generate_json_report: true,
            report_output_dir: "quality_reports".to_string(),
            word_alignment_tolerance_s: 0.2, // 200ms tolerance
        }
    }
}

/// Quality analysis errors
#[derive(Debug, Error)]
pub enum QualityAnalysisError {
    #[error("Ground truth not found: {path}")]
    GroundTruthNotFound { path: String },
    
    #[error("Invalid transcription format: {message}")]
    InvalidTranscription { message: String },
    
    #[error("Mismatched audio lengths: transcription {transcription}s, ground_truth {ground_truth}s")]
    AudioLengthMismatch { transcription: f32, ground_truth: f32 },
    
    #[error("Insufficient data: {message}")]
    InsufficientData { message: String },
    
    #[error("IO error: {message}")]
    IoError { message: String },
    
    #[error("Analysis calculation error: {message}")]
    CalculationError { message: String },
}

/// Main quality analyzer
pub struct TranscriptionQualityAnalyzer {
    config: QualityAnalysisConfig,
    diarization_validator: DiarizationValidator,
}

impl TranscriptionQualityAnalyzer {
    /// Create new analyzer with default configuration
    pub fn new() -> Self {
        Self::with_config(QualityAnalysisConfig::default())
    }
    
    /// Create new analyzer with custom configuration
    pub fn with_config(config: QualityAnalysisConfig) -> Self {
        // Configure diarization validator to match quality analysis settings
        let validation_config = ValidationConfig {
            generate_reports: false, // We'll generate our own combined reports
            report_output_dir: config.report_output_dir.clone(),
            ..ValidationConfig::default()
        };
        
        Self {
            config,
            diarization_validator: DiarizationValidator::with_config(validation_config),
        }
    }
    
    /// Perform comprehensive quality analysis
    /// 
    /// This is the main analysis function that evaluates both transcription accuracy
    /// (WER/CER) and speaker diarization accuracy (DER) against ground truth data.
    /// 
    /// # Arguments
    /// 
    /// * `transcription_result` - System output to analyze
    /// * `ground_truth` - Ground truth transcription and speaker data
    /// * `performance_metrics` - System performance data
    /// 
    /// # Returns
    /// 
    /// Complete quality analysis with detailed metrics and recommendations
    pub fn analyze_quality(
        &mut self,
        transcription_result: TranscriptionResult,
        ground_truth: TranscriptionGroundTruth,
        performance_metrics: SystemPerformanceMetrics,
    ) -> Result<QualityAnalysisResult, QualityAnalysisError> {
        let analysis_start = SystemTime::now();
        
        // Validate input data
        self.validate_inputs(&transcription_result, &ground_truth)?;
        
        // Calculate Word Error Rate
        let wer_result = self.calculate_wer(&transcription_result, &ground_truth)?;
        
        // Calculate Speaker-Attributed WER
        let sa_wer_result = if self.config.include_per_speaker_analysis {
            self.calculate_sa_wer(&transcription_result, &ground_truth)?
        } else {
            SAWERResult {
                sa_wer_percentage: wer_result.wer_percentage,
                per_speaker_wer: HashMap::new(),
                correct_speaker_attribution: 0,
                incorrect_speaker_attribution: 0,
                total_attributed_words: 0,
                speaker_attribution_accuracy: 0.0,
            }
        };
        
        // Convert ground truth to diarization format for DER calculation
        let diarization_ground_truth = self.convert_to_diarization_ground_truth(&ground_truth);
        let predicted_segments = self.convert_to_speaker_segments(&transcription_result);
        
        // Calculate diarization metrics
        let diarization_result = self.diarization_validator.compare_segments(
            predicted_segments,
            diarization_ground_truth,
            self.config.word_alignment_tolerance_s * 1000.0, // Convert to milliseconds
        ).map_err(|e| QualityAnalysisError::CalculationError { 
            message: format!("Diarization validation failed: {}", e) 
        })?;
        
        // Determine overall quality level
        let quality_level = QualityLevel::from_wer_der(
            wer_result.wer_percentage,
            diarization_result.der_result.der_score * 100.0,
        );
        
        // Calculate overall system score
        let overall_score = self.calculate_overall_score(
            &wer_result,
            &sa_wer_result,
            &diarization_result,
            &performance_metrics,
        );
        
        // Generate recommendations
        let recommendations = self.generate_recommendations(
            &wer_result,
            &sa_wer_result,
            &diarization_result,
            &performance_metrics,
            quality_level,
        );
        
        let analysis_end = SystemTime::now();
        let analysis_duration_ms = analysis_end.duration_since(analysis_start)
            .unwrap_or_default().as_millis() as u64;
        
        let result = QualityAnalysisResult {
            recording_id: format!("quality_analysis_{}", 
                SystemTime::now().duration_since(UNIX_EPOCH)
                    .unwrap_or_default().as_secs()),
            wer_result,
            sa_wer_result,
            diarization_result,
            performance_metrics,
            quality_level,
            overall_score,
            recommendations,
            analysis_timestamp: SystemTime::now(),
            analysis_duration_ms,
        };
        
        // Generate reports if configured
        if self.config.generate_html_report || self.config.generate_json_report {
            self.generate_reports(&result)?;
        }
        
        Ok(result)
    }
    
    /// Calculate Word Error Rate using Levenshtein distance
    fn calculate_wer(
        &self,
        transcription: &TranscriptionResult,
        ground_truth: &TranscriptionGroundTruth,
    ) -> Result<WERResult, QualityAnalysisError> {
        // Normalize and tokenize text
        let hypothesis_words = self.tokenize_text(&transcription.text);
        let reference_words = self.tokenize_text(&ground_truth.text);
        
        if reference_words.is_empty() {
            return Err(QualityAnalysisError::InsufficientData {
                message: "Ground truth text is empty".to_string(),
            });
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
        
        // Calculate Character Error Rate if enabled
        let (cer_percentage, character_accuracy) = if self.config.include_character_analysis {
            let ref_chars: Vec<char> = ground_truth.text.chars().collect();
            let hyp_chars: Vec<char> = transcription.text.chars().collect();
            
            let (char_sub, char_ins, char_del) = self.calculate_character_edit_operations(
                &ref_chars,
                &hyp_chars,
            );
            
            let total_char_errors = char_sub + char_ins + char_del;
            let cer = if ref_chars.len() > 0 {
                (total_char_errors as f32 / ref_chars.len() as f32) * 100.0
            } else {
                0.0
            };
            
            (cer, (100.0 - cer).max(0.0))
        } else {
            (0.0, 100.0)
        };
        
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
    ) -> Result<SAWERResult, QualityAnalysisError> {
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
        
        // Calculate speaker attribution accuracy
        for trans_word in &transcription.words {
            if let Some(speaker_id) = &trans_word.speaker_id {
                total_attributed += 1;
                
                // Find temporally overlapping ground truth word
                let overlapping_gt_word = ground_truth.words.iter()
                    .find(|gt_word| {
                        let overlap_start = trans_word.start_time.max(gt_word.start_time);
                        let overlap_end = trans_word.end_time.min(gt_word.end_time);
                        overlap_end > overlap_start + self.config.word_alignment_tolerance_s
                    });
                
                if let Some(gt_word) = overlapping_gt_word {
                    if gt_word.speaker_id == *speaker_id {
                        correct_attribution += 1;
                    } else {
                        incorrect_attribution += 1;
                    }
                } else {
                    incorrect_attribution += 1; // No overlapping ground truth = error
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
            total_attributed_words: total_attributed,
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
    
    /// Calculate overall system score (0-100)
    fn calculate_overall_score(
        &self,
        wer_result: &WERResult,
        sa_wer_result: &SAWERResult,
        diarization_result: &DiarizationValidationResult,
        performance_metrics: &SystemPerformanceMetrics,
    ) -> f32 {
        // Weight factors for different components
        let wer_weight = 0.4;       // 40% - transcription accuracy
        let diarization_weight = 0.3; // 30% - speaker diarization
        let performance_weight = 0.2;  // 20% - processing performance
        let attribution_weight = 0.1;  // 10% - speaker attribution
        
        // Convert metrics to 0-100 scores (higher is better)
        let wer_score = (100.0 - wer_result.wer_percentage).max(0.0).min(100.0);
        let der_score = (100.0f32 - diarization_result.der_result.der_score * 100.0).max(0.0).min(100.0);
        let performance_score = ((2.0 - performance_metrics.real_time_factor).max(0.0) * 50.0).min(100.0);
        let attribution_score = sa_wer_result.speaker_attribution_accuracy;
        
        // Calculate weighted average
        (wer_score * wer_weight + 
         der_score * diarization_weight + 
         performance_score * performance_weight + 
         attribution_score * attribution_weight).max(0.0).min(100.0)
    }
    
    /// Generate actionable recommendations
    fn generate_recommendations(
        &self,
        wer_result: &WERResult,
        sa_wer_result: &SAWERResult,
        diarization_result: &DiarizationValidationResult,
        performance_metrics: &SystemPerformanceMetrics,
        quality_level: QualityLevel,
    ) -> QualityRecommendations {
        let mut critical_improvements = Vec::new();
        let mut performance_optimizations = Vec::new();
        let mut accuracy_enhancements = Vec::new();
        let mut config_suggestions = Vec::new();
        
        // Critical improvements (must fix)
        if wer_result.wer_percentage > 30.0 {
            critical_improvements.push("WER > 30%: Consider using higher-accuracy model tier or improving audio quality".to_string());
        }
        if diarization_result.der_result.der_score > 0.3 {
            critical_improvements.push("DER > 30%: Review speaker diarization configuration and audio preprocessing".to_string());
        }
        if performance_metrics.real_time_factor > 3.0 {
            critical_improvements.push("Processing too slow: Consider using faster model tier or optimizing hardware".to_string());
        }
        
        // Performance optimizations
        if performance_metrics.real_time_factor > 1.5 {
            performance_optimizations.push(format!(
                "Real-time factor {:.2}x: Consider Turbo model tier for faster processing", 
                performance_metrics.real_time_factor
            ));
        }
        if performance_metrics.memory_usage_mb > 2000.0 {
            performance_optimizations.push("High memory usage: Consider smaller model or batch processing optimization".to_string());
        }
        if performance_metrics.latency_ms > 3000 {
            performance_optimizations.push("High latency: Optimize model loading and audio buffering".to_string());
        }
        
        // Accuracy enhancements
        if wer_result.wer_percentage > 10.0 && wer_result.wer_percentage <= 30.0 {
            accuracy_enhancements.push("Consider High-Accuracy model tier for better transcription quality".to_string());
        }
        if diarization_result.der_result.precision < 0.8 {
            accuracy_enhancements.push("Low diarization precision: Adjust speaker detection thresholds".to_string());
        }
        if sa_wer_result.speaker_attribution_accuracy < 80.0 {
            accuracy_enhancements.push("Poor speaker attribution: Improve diarization accuracy or word alignment".to_string());
        }
        if wer_result.substitutions > wer_result.insertions + wer_result.deletions {
            accuracy_enhancements.push("High substitution errors: Consider domain-specific model or vocabulary".to_string());
        }
        
        // Configuration suggestions
        if diarization_result.der_result.miss_rate > 0.2 {
            config_suggestions.push("High miss rate: Lower VAD threshold to capture more speech".to_string());
        }
        if diarization_result.der_result.false_alarm_rate > 0.2 {
            config_suggestions.push("High false alarm rate: Raise VAD threshold to reduce noise detection".to_string());
        }
        
        // Generate summary
        let summary = match quality_level {
            QualityLevel::Excellent => format!(
                "Excellent quality achieved with {:.1}% word accuracy and {:.1}% diarization accuracy. System performing at production level.",
                wer_result.word_accuracy,
                (1.0 - diarization_result.der_result.der_score) * 100.0
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
            performance_optimizations,
            accuracy_enhancements,
            config_suggestions,
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
    
    /// Convert ground truth to diarization format
    fn convert_to_diarization_ground_truth(
        &self,
        ground_truth: &TranscriptionGroundTruth,
    ) -> Vec<GroundTruthSegment> {
        ground_truth.speaker_segments.iter()
            .map(|seg| GroundTruthSegment {
                speaker_id: seg.speaker_id.clone(),
                start_time: seg.start_time,
                end_time: seg.end_time,
                text: Some(seg.text.clone()),
                audio_file: None,
                quality: 1.0,
            })
            .collect()
    }
    
    /// Convert transcription result to speaker segments
    fn convert_to_speaker_segments(
        &self,
        transcription: &TranscriptionResult,
    ) -> Vec<kaginote_lib::diarization::types::SpeakerSegment> {
        let mut segments = Vec::new();
        let mut current_speaker = None;
        let mut current_start = 0.0f32;
        let mut current_text = String::new();
        let mut current_end = 0.0f32;
        
        for word in &transcription.words {
            if current_speaker.as_ref() != word.speaker_id.as_ref() {
                // Speaker change - finish current segment
                if let Some(speaker) = current_speaker.take() {
                    segments.push(kaginote_lib::diarization::types::SpeakerSegment {
                        speaker_id: speaker,
                        start_time: current_start,
                        end_time: current_end,
                        confidence: 0.9, // Use average confidence
                        text: Some(current_text.trim().to_string()),
                        embedding: None,
                        has_overlap: false,
                        overlapping_speakers: vec![],
                    });
                }
                
                // Start new segment
                if let Some(speaker_id) = &word.speaker_id {
                    current_speaker = Some(speaker_id.clone());
                    current_start = word.start_time;
                    current_text = word.word.clone();
                } else {
                    current_text.clear();
                }
            } else if word.speaker_id.is_some() {
                // Continue current segment
                if !current_text.is_empty() {
                    current_text.push(' ');
                }
                current_text.push_str(&word.word);
            }
            
            current_end = word.end_time;
        }
        
        // Finish final segment
        if let Some(speaker) = current_speaker {
            segments.push(kaginote_lib::diarization::types::SpeakerSegment {
                speaker_id: speaker,
                start_time: current_start,
                end_time: current_end,
                confidence: 0.9,
                text: Some(current_text.trim().to_string()),
                embedding: None,
                has_overlap: false,
                overlapping_speakers: vec![],
            });
        }
        
        segments
    }
    
    /// Validate input data consistency
    fn validate_inputs(
        &self,
        transcription: &TranscriptionResult,
        ground_truth: &TranscriptionGroundTruth,
    ) -> Result<(), QualityAnalysisError> {
        // Check for empty data
        if transcription.text.is_empty() {
            return Err(QualityAnalysisError::InsufficientData {
                message: "Transcription text is empty".to_string(),
            });
        }
        
        if ground_truth.text.is_empty() {
            return Err(QualityAnalysisError::InsufficientData {
                message: "Ground truth text is empty".to_string(),
            });
        }
        
        // Check audio duration consistency (allow 10% tolerance)
        let transcription_duration = transcription.words.last()
            .map(|w| w.end_time)
            .unwrap_or(0.0);
        
        let duration_diff = (transcription_duration - ground_truth.total_duration).abs();
        let tolerance = ground_truth.total_duration * 0.1; // 10% tolerance
        
        if duration_diff > tolerance {
            return Err(QualityAnalysisError::AudioLengthMismatch {
                transcription: transcription_duration,
                ground_truth: ground_truth.total_duration,
            });
        }
        
        Ok(())
    }
    
    /// Generate HTML and JSON reports
    fn generate_reports(
        &self,
        result: &QualityAnalysisResult,
    ) -> Result<(), QualityAnalysisError> {
        let report_dir = Path::new(&self.config.report_output_dir);
        fs::create_dir_all(report_dir)
            .map_err(|e| QualityAnalysisError::IoError {
                message: format!("Failed to create report directory: {}", e),
            })?;
        
        // Generate JSON report
        if self.config.generate_json_report {
            let json_path = report_dir.join(format!("{}_quality_analysis.json", result.recording_id));
            let json_content = serde_json::to_string_pretty(result)
                .map_err(|e| QualityAnalysisError::IoError {
                    message: format!("Failed to serialize JSON: {}", e),
                })?;
            
            fs::write(&json_path, json_content)
                .map_err(|e| QualityAnalysisError::IoError {
                    message: format!("Failed to write JSON report: {}", e),
                })?;
            
            println!("JSON report: {}", json_path.display());
        }
        
        // Generate HTML report
        if self.config.generate_html_report {
            let html_path = report_dir.join(format!("{}_quality_analysis.html", result.recording_id));
            let html_content = self.generate_html_report(result);
            
            fs::write(&html_path, html_content)
                .map_err(|e| QualityAnalysisError::IoError {
                    message: format!("Failed to write HTML report: {}", e),
                })?;
            
            println!("HTML report: {}", html_path.display());
        }
        
        Ok(())
    }
    
    /// Generate comprehensive HTML report
    fn generate_html_report(&self, result: &QualityAnalysisResult) -> String {
        let quality_color = result.quality_level.color();
        
        format!(r#"
<!DOCTYPE html>
<html>
<head>
    <title>Transcription Quality Analysis - {}</title>
    <style>
        body {{ font-family: 'Segoe UI', -apple-system, BlinkMacSystemFont, sans-serif; margin: 0; padding: 20px; background: #f8fafc; }}
        .container {{ max-width: 1200px; margin: 0 auto; background: white; border-radius: 12px; box-shadow: 0 4px 6px rgba(0,0,0,0.1); }}
        .header {{ background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); color: white; padding: 30px; border-radius: 12px 12px 0 0; }}
        .header h1 {{ margin: 0; font-size: 28px; font-weight: 600; }}
        .header .subtitle {{ opacity: 0.9; margin-top: 8px; font-size: 16px; }}
        .overall-score {{ text-align: center; padding: 30px; background: #f8fafc; border-bottom: 1px solid #e2e8f0; }}
        .score-circle {{ width: 120px; height: 120px; border-radius: 50%; margin: 0 auto 20px; display: flex; align-items: center; justify-content: center; color: white; font-size: 28px; font-weight: bold; background: {}; }}
        .score-label {{ font-size: 18px; color: #475569; margin-bottom: 8px; }}
        .quality-level {{ font-size: 20px; font-weight: 600; color: {}; }}
        .metrics-grid {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(300px, 1fr)); gap: 30px; padding: 30px; }}
        .metric-card {{ background: white; border: 1px solid #e2e8f0; border-radius: 8px; padding: 20px; }}
        .metric-card h3 {{ margin: 0 0 16px 0; color: #1e293b; font-size: 18px; font-weight: 600; }}
        .metric-row {{ display: flex; justify-content: space-between; align-items: center; margin-bottom: 12px; }}
        .metric-label {{ color: #64748b; }}
        .metric-value {{ font-weight: 600; }}
        .metric-value.good {{ color: #059669; }}
        .metric-value.fair {{ color: #d97706; }}
        .metric-value.poor {{ color: #dc2626; }}
        .recommendations {{ padding: 30px; background: #f8fafc; }}
        .recommendations h3 {{ margin: 0 0 20px 0; color: #1e293b; }}
        .rec-section {{ margin-bottom: 24px; }}
        .rec-section h4 {{ color: #475569; margin: 0 0 12px 0; }}
        .rec-list {{ list-style: none; padding: 0; }}
        .rec-list li {{ background: white; padding: 12px 16px; margin-bottom: 8px; border-left: 4px solid #3b82f6; border-radius: 4px; }}
        .critical {{ border-left-color: #dc2626 !important; }}
        .performance {{ border-left-color: #d97706 !important; }}
        .accuracy {{ border-left-color: #059669 !important; }}
        .config {{ border-left-color: #6366f1 !important; }}
        .summary {{ background: #eff6ff; border: 1px solid #dbeafe; padding: 20px; border-radius: 8px; margin-top: 20px; }}
        .summary p {{ margin: 0; color: #1e40af; font-weight: 500; }}
        .details-table {{ width: 100%; border-collapse: collapse; margin-top: 16px; }}
        .details-table th, .details-table td {{ padding: 8px 12px; text-align: left; border-bottom: 1px solid #e2e8f0; }}
        .details-table th {{ background: #f1f5f9; font-weight: 600; color: #475569; }}
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>Transcription Quality Analysis Report</h1>
            <div class="subtitle">Recording: {} • Generated: {}</div>
        </div>
        
        <div class="overall-score">
            <div class="score-circle">{:.0}%</div>
            <div class="score-label">Overall System Score</div>
            <div class="quality-level">{} Quality</div>
        </div>
        
        <div class="metrics-grid">
            <div class="metric-card">
                <h3>📝 Transcription Accuracy</h3>
                <div class="metric-row">
                    <span class="metric-label">Word Error Rate (WER)</span>
                    <span class="metric-value {}">{:.2}%</span>
                </div>
                <div class="metric-row">
                    <span class="metric-label">Word Accuracy</span>
                    <span class="metric-value {}">{:.1}%</span>
                </div>
                <div class="metric-row">
                    <span class="metric-label">Character Accuracy</span>
                    <span class="metric-value">{:.1}%</span>
                </div>
                <table class="details-table">
                    <tr><th>Error Type</th><th>Count</th></tr>
                    <tr><td>Substitutions</td><td>{}</td></tr>
                    <tr><td>Insertions</td><td>{}</td></tr>
                    <tr><td>Deletions</td><td>{}</td></tr>
                </table>
            </div>
            
            <div class="metric-card">
                <h3>👥 Speaker Diarization</h3>
                <div class="metric-row">
                    <span class="metric-label">Diarization Error Rate (DER)</span>
                    <span class="metric-value {}">{:.2}%</span>
                </div>
                <div class="metric-row">
                    <span class="metric-label">Speaker Attribution</span>
                    <span class="metric-value">{:.1}%</span>
                </div>
                <div class="metric-row">
                    <span class="metric-label">F1 Score</span>
                    <span class="metric-value">{:.3}</span>
                </div>
                <table class="details-table">
                    <tr><th>Metric</th><th>Rate</th></tr>
                    <tr><td>False Alarm Rate</td><td>{:.3}</td></tr>
                    <tr><td>Miss Rate</td><td>{:.3}</td></tr>
                    <tr><td>Speaker Error Rate</td><td>{:.3}</td></tr>
                </table>
            </div>
            
            <div class="metric-card">
                <h3>⚡ Performance Metrics</h3>
                <div class="metric-row">
                    <span class="metric-label">Real-time Factor</span>
                    <span class="metric-value {}">{:.2}x</span>
                </div>
                <div class="metric-row">
                    <span class="metric-label">Memory Usage</span>
                    <span class="metric-value">{:.0} MB</span>
                </div>
                <div class="metric-row">
                    <span class="metric-label">Latency</span>
                    <span class="metric-value">{} ms</span>
                </div>
                <div class="metric-row">
                    <span class="metric-label">CPU Usage</span>
                    <span class="metric-value">{:.1}%</span>
                </div>
            </div>
            
            <div class="metric-card">
                <h3>🎯 Speaker-Attributed WER</h3>
                <div class="metric-row">
                    <span class="metric-label">SA-WER</span>
                    <span class="metric-value">{:.2}%</span>
                </div>
                <div class="metric-row">
                    <span class="metric-label">Attribution Accuracy</span>
                    <span class="metric-value">{:.1}%</span>
                </div>
                <table class="details-table">
                    <tr><th>Speaker</th><th>WER</th></tr>
                    {}
                </table>
            </div>
        </div>
        
        <div class="recommendations">
            <h3>🎯 Quality Improvement Recommendations</h3>
            
            {}
            
            <div class="summary">
                <p>{}</p>
            </div>
        </div>
    </div>
</body>
</html>
"#,
            result.recording_id,
            result.recording_id,
            result.analysis_timestamp.duration_since(UNIX_EPOCH)
                .unwrap_or_default().as_secs(),
            result.overall_score,
            result.quality_level,
            
            // WER metrics
            self.get_quality_class(result.wer_result.wer_percentage, 5.0, 15.0, 25.0),
            result.wer_result.wer_percentage,
            self.get_quality_class(100.0 - result.wer_result.word_accuracy, 25.0, 15.0, 5.0),
            result.wer_result.word_accuracy,
            result.wer_result.character_accuracy,
            result.wer_result.substitutions,
            result.wer_result.insertions,
            result.wer_result.deletions,
            
            // DER metrics
            self.get_quality_class(result.diarization_result.der_result.der_score * 100.0, 10.0, 20.0, 30.0),
            result.diarization_result.der_result.der_score * 100.0,
            result.sa_wer_result.speaker_attribution_accuracy,
            result.diarization_result.der_result.f1_score,
            result.diarization_result.der_result.false_alarm_rate,
            result.diarization_result.der_result.miss_rate,
            result.diarization_result.der_result.speaker_error_rate,
            
            // Performance metrics
            self.get_performance_class(result.performance_metrics.real_time_factor),
            result.performance_metrics.real_time_factor,
            result.performance_metrics.memory_usage_mb,
            result.performance_metrics.latency_ms,
            result.performance_metrics.cpu_utilization,
            
            // SA-WER metrics
            result.sa_wer_result.sa_wer_percentage,
            result.sa_wer_result.speaker_attribution_accuracy,
            self.generate_speaker_table(&result.sa_wer_result.per_speaker_wer),
            
            // Recommendations
            self.generate_recommendations_html(&result.recommendations),
            result.recommendations.summary,
        )
    }
    
    /// Get CSS class for quality metrics
    fn get_quality_class(&self, value: f32, excellent: f32, good: f32, fair: f32) -> &'static str {
        if value < excellent { "good" }
        else if value < good { "fair" }
        else if value < fair { "fair" }
        else { "poor" }
    }
    
    /// Get CSS class for performance metrics
    fn get_performance_class(&self, rt_factor: f32) -> &'static str {
        if rt_factor < 0.5 { "good" }
        else if rt_factor < 1.0 { "good" }
        else if rt_factor < 2.0 { "fair" }
        else { "poor" }
    }
    
    /// Generate speaker WER table HTML
    fn generate_speaker_table(&self, per_speaker_wer: &HashMap<String, f32>) -> String {
        per_speaker_wer.iter()
            .map(|(speaker, wer)| format!("<tr><td>{}</td><td>{:.2}%</td></tr>", speaker, wer))
            .collect::<Vec<_>>()
            .join("")
    }
    
    /// Generate recommendations HTML
    fn generate_recommendations_html(&self, recommendations: &QualityRecommendations) -> String {
        let mut html = String::new();
        
        if !recommendations.critical_improvements.is_empty() {
            html.push_str(r#"<div class="rec-section"><h4>🚨 Critical Improvements</h4><ul class="rec-list">"#);
            for item in &recommendations.critical_improvements {
                html.push_str(&format!(r#"<li class="critical">{}</li>"#, item));
            }
            html.push_str("</ul></div>");
        }
        
        if !recommendations.performance_optimizations.is_empty() {
            html.push_str(r#"<div class="rec-section"><h4>⚡ Performance Optimizations</h4><ul class="rec-list">"#);
            for item in &recommendations.performance_optimizations {
                html.push_str(&format!(r#"<li class="performance">{}</li>"#, item));
            }
            html.push_str("</ul></div>");
        }
        
        if !recommendations.accuracy_enhancements.is_empty() {
            html.push_str(r#"<div class="rec-section"><h4>🎯 Accuracy Enhancements</h4><ul class="rec-list">"#);
            for item in &recommendations.accuracy_enhancements {
                html.push_str(&format!(r#"<li class="accuracy">{}</li>"#, item));
            }
            html.push_str("</ul></div>");
        }
        
        if !recommendations.config_suggestions.is_empty() {
            html.push_str(r#"<div class="rec-section"><h4>⚙️ Configuration Suggestions</h4><ul class="rec-list">"#);
            for item in &recommendations.config_suggestions {
                html.push_str(&format!(r#"<li class="config">{}</li>"#, item));
            }
            html.push_str("</ul></div>");
        }
        
        html
    }
}

/// Analyze LibriSpeech results from integration tests
pub fn analyze_librispeech_results(
    _audio_file_path: &str,
    _ground_truth_path: &str,
) -> Result<QualityAnalysisResult, QualityAnalysisError> {
    // This function would integrate with the diarization_realtime integration tests
    // to analyze real LibriSpeech results
    todo!("Implement LibriSpeech results analysis")
}

/// Load ground truth from LibriSpeech format
pub fn load_librispeech_ground_truth(
    _librispeech_transcript_path: &str,
) -> Result<TranscriptionGroundTruth, QualityAnalysisError> {
    // Parse LibriSpeech transcript format
    todo!("Implement LibriSpeech ground truth parsing")
}

#[cfg(test)]
mod tests {
    use super::*;
    
    /// Create test transcription result
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
    
    /// Create test ground truth
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
            speaker_segments: vec![
                GroundTruthSpeakerSegment {
                    speaker_id: "speaker_1".to_string(),
                    start_time: 0.0,
                    end_time: 1.0,
                    text: "Hello world".to_string(),
                },
                GroundTruthSpeakerSegment {
                    speaker_id: "speaker_2".to_string(),
                    start_time: 2.0,
                    end_time: 3.4,
                    text: "how are you today".to_string(),
                },
            ],
            total_duration: 3.4,
            num_speakers: 2,
        }
    }
    
    /// Create test performance metrics
    fn create_test_performance_metrics() -> SystemPerformanceMetrics {
        SystemPerformanceMetrics {
            real_time_factor: 0.4,
            memory_usage_mb: 150.0,
            cpu_utilization: 25.0,
            latency_ms: 800,
            throughput: 2.5,
            model_load_time_ms: 1200,
        }
    }
    
    #[test]
    fn test_perfect_transcription_analysis() {
        let mut analyzer = TranscriptionQualityAnalyzer::new();
        let transcription = create_test_transcription();
        let ground_truth = create_test_ground_truth();
        let performance = create_test_performance_metrics();
        
        let result = analyzer.analyze_quality(transcription, ground_truth, performance)
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
        assert!(result.overall_score > 90.0);
        
        // Should have perfect speaker attribution
        assert_eq!(result.sa_wer_result.speaker_attribution_accuracy, 100.0);
    }
    
    #[test]
    fn test_wer_calculation() {
        let analyzer = TranscriptionQualityAnalyzer::new();
        
        // Test substitution error
        let mut transcription = create_test_transcription();
        transcription.words[0].word = "Hi".to_string(); // "Hello" -> "Hi"
        transcription.text = "Hi world how are you today".to_string();
        
        let ground_truth = create_test_ground_truth();
        
        let wer_result = analyzer.calculate_wer(&transcription, &ground_truth)
            .expect("WER calculation should succeed");
        
        // Should have 1 substitution out of 6 words = 16.67% WER
        assert_eq!(wer_result.substitutions, 1);
        assert_eq!(wer_result.insertions, 0);
        assert_eq!(wer_result.deletions, 0);
        assert!((wer_result.wer_percentage - 16.67).abs() < 0.1);
        assert!((wer_result.word_accuracy - 83.33).abs() < 0.1);
    }
    
    #[test]
    fn test_speaker_attribution_analysis() {
        let analyzer = TranscriptionQualityAnalyzer::new();
        
        // Test incorrect speaker attribution
        let mut transcription = create_test_transcription();
        // Swap speaker IDs for first two words
        transcription.words[0].speaker_id = Some("speaker_2".to_string());
        transcription.words[1].speaker_id = Some("speaker_2".to_string());
        
        let ground_truth = create_test_ground_truth();
        
        let sa_wer_result = analyzer.calculate_sa_wer(&transcription, &ground_truth)
            .expect("SA-WER calculation should succeed");
        
        // Should have incorrect attribution for 2 out of 6 words
        assert_eq!(sa_wer_result.incorrect_speaker_attribution, 2);
        assert_eq!(sa_wer_result.correct_speaker_attribution, 4);
        assert!((sa_wer_result.speaker_attribution_accuracy - 66.67).abs() < 0.1);
    }
    
    #[test]
    fn test_quality_level_classification() {
        // Test excellent quality
        assert_eq!(QualityLevel::from_wer_der(3.0, 8.0), QualityLevel::Excellent);
        
        // Test good quality
        assert_eq!(QualityLevel::from_wer_der(10.0, 15.0), QualityLevel::Good);
        
        // Test fair quality
        assert_eq!(QualityLevel::from_wer_der(20.0, 25.0), QualityLevel::Fair);
        
        // Test poor quality
        assert_eq!(QualityLevel::from_wer_der(30.0, 35.0), QualityLevel::Poor);
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
    fn test_input_validation() {
        let analyzer = TranscriptionQualityAnalyzer::new();
        
        // Test empty transcription
        let empty_transcription = TranscriptionResult {
            text: "".to_string(),
            words: vec![],
            confidence: 0.0,
            language: "en".to_string(),
            processing_time_ms: 0,
            real_time_factor: 0.0,
        };
        let ground_truth = create_test_ground_truth();
        
        let result = analyzer.validate_inputs(&empty_transcription, &ground_truth);
        assert!(result.is_err());
        
        // Test duration mismatch
        let mut transcription = create_test_transcription();
        transcription.words.last_mut().unwrap().end_time = 10.0; // Much longer than ground truth
        
        let result = analyzer.validate_inputs(&transcription, &ground_truth);
        assert!(result.is_err());
    }
}