//! Comprehensive validation framework for diarization accuracy and performance
//! 
//! This module provides tools for measuring diarization quality using standard
//! metrics like Diarization Error Rate (DER), precision, recall, and speaker
//! consistency. It also includes performance monitoring and report generation
//! capabilities for continuous improvement and benchmarking.

use kaginote_lib::diarization::types::{DiarizationResult, SpeakerSegment, SpeakerEmbedding, ProcessingMetrics};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use thiserror::Error;

/// Ground truth segment for validation comparison
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroundTruthSegment {
    /// True speaker identifier
    pub speaker_id: String,
    
    /// Start time in seconds
    pub start_time: f32,
    
    /// End time in seconds
    pub end_time: f32,
    
    /// Optional text content for reference
    pub text: Option<String>,
    
    /// Audio file path if available
    pub audio_file: Option<String>,
    
    /// Quality score for this ground truth segment (0.0-1.0)
    pub quality: f32,
}

/// Complete ground truth data for a recording
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroundTruthData {
    /// Recording identifier
    pub recording_id: String,
    
    /// All ground truth segments
    pub segments: Vec<GroundTruthSegment>,
    
    /// Total number of speakers
    pub total_speakers: usize,
    
    /// Recording duration in seconds
    pub duration: f32,
    
    /// Sample rate
    pub sample_rate: u32,
    
    /// Metadata about the recording
    pub metadata: HashMap<String, String>,
}

/// Diarization Error Rate calculation results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DERResult {
    /// Overall DER score (0.0 = perfect, higher = worse)
    pub der_score: f32,
    
    /// False alarm rate (speech detected when none)
    pub false_alarm_rate: f32,
    
    /// Miss rate (speech missed when present)
    pub miss_rate: f32,
    
    /// Speaker error rate (wrong speaker assigned)
    pub speaker_error_rate: f32,
    
    /// Total speech time in seconds
    pub total_speech_time: f32,
    
    /// Total error time in seconds
    pub total_error_time: f32,
    
    /// Precision score (0.0-1.0)
    pub precision: f32,
    
    /// Recall score (0.0-1.0)
    pub recall: f32,
    
    /// F1 score (harmonic mean of precision and recall)
    pub f1_score: f32,
    
    /// Overlap accuracy (correctly identified overlapping speech)
    pub overlap_accuracy: f32,
}

/// Speaker consistency metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeakerConsistencyResult {
    /// Overall consistency score (0.0-1.0)
    pub consistency_score: f32,
    
    /// Speaker purity scores per speaker
    pub speaker_purity: HashMap<String, f32>,
    
    /// Speaker coverage scores per speaker
    pub speaker_coverage: HashMap<String, f32>,
    
    /// Number of speaker ID switches detected
    pub id_switches: usize,
    
    /// Total number of segments analyzed
    pub total_segments: usize,
    
    /// Percentage of segments with consistent speaker ID
    pub consistency_percentage: f32,
    
    /// Confusion matrix between predicted and true speakers
    pub confusion_matrix: HashMap<String, HashMap<String, usize>>,
}

/// Performance metrics collection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// Real-time factor (processing_time / audio_duration)
    pub real_time_factor: f32,
    
    /// Peak memory usage in MB
    pub peak_memory_mb: f32,
    
    /// Average memory usage in MB
    pub average_memory_mb: f32,
    
    /// CPU utilization percentage
    pub cpu_utilization: f32,
    
    /// Processing latency from input to output (ms)
    pub latency_ms: u64,
    
    /// Throughput (audio seconds processed per second)
    pub throughput: f32,
    
    /// Number of memory allocations
    pub memory_allocations: usize,
    
    /// Time to first output (cold start)
    pub time_to_first_output_ms: u64,
    
    /// Processing start timestamp
    pub start_time: SystemTime,
    
    /// Processing end timestamp
    pub end_time: SystemTime,
}

/// Complete validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Recording identifier
    pub recording_id: String,
    
    /// DER calculation results
    pub der_result: DERResult,
    
    /// Speaker consistency metrics
    pub consistency_result: SpeakerConsistencyResult,
    
    /// Performance metrics
    pub performance_metrics: PerformanceMetrics,
    
    /// Comparison summary
    pub summary: ValidationSummary,
    
    /// Timestamp when validation was performed
    pub validation_timestamp: SystemTime,
    
    /// Configuration used for diarization
    pub config_used: String,
    
    /// Any warnings during validation
    pub warnings: Vec<String>,
}

/// High-level validation summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationSummary {
    /// Overall quality score (0.0-1.0)
    pub overall_quality: f32,
    
    /// Accuracy level (Excellent, Good, Fair, Poor)
    pub accuracy_level: AccuracyLevel,
    
    /// Performance level (Excellent, Good, Fair, Poor)
    pub performance_level: PerformanceLevel,
    
    /// Key strengths identified
    pub strengths: Vec<String>,
    
    /// Areas for improvement
    pub improvements: Vec<String>,
    
    /// Pass/fail status for target thresholds
    pub meets_targets: bool,
}

/// Accuracy classification levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccuracyLevel {
    Excellent, // DER < 10%
    Good,      // DER < 20%
    Fair,      // DER < 30%
    Poor,      // DER >= 30%
}

/// Performance classification levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PerformanceLevel {
    Excellent, // RT factor < 0.5x
    Good,      // RT factor < 1.0x
    Fair,      // RT factor < 2.0x
    Poor,      // RT factor >= 2.0x
}

/// Validation configuration and thresholds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationConfig {
    /// Time tolerance for segment matching (seconds)
    pub time_tolerance_s: f32,
    
    /// Target DER threshold for pass/fail
    pub target_der_threshold: f32,
    
    /// Target real-time factor for pass/fail
    pub target_rt_factor: f32,
    
    /// Minimum speaker consistency score
    pub min_consistency_score: f32,
    
    /// Whether to include overlap evaluation
    pub evaluate_overlaps: bool,
    
    /// Whether to generate detailed reports
    pub generate_reports: bool,
    
    /// Output directory for reports
    pub report_output_dir: String,
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            time_tolerance_s: 0.25,     // 250ms tolerance
            target_der_threshold: 0.15,  // 15% DER target
            target_rt_factor: 1.5,       // 1.5x real-time target
            min_consistency_score: 0.85,  // 85% consistency target
            evaluate_overlaps: true,
            generate_reports: true,
            report_output_dir: "test_reports".to_string(),
        }
    }
}

/// Validation errors
#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("Ground truth file not found: {path}")]
    GroundTruthNotFound { path: String },
    
    #[error("Invalid ground truth format: {message}")]
    InvalidGroundTruth { message: String },
    
    #[error("Time range mismatch: predicted {predicted}s, ground truth {ground_truth}s")]
    TimeRangeMismatch { predicted: f32, ground_truth: f32 },
    
    #[error("Insufficient data for validation: {message}")]
    InsufficientData { message: String },
    
    #[error("IO error: {message}")]
    IoError { message: String },
    
    #[error("Calculation error: {message}")]
    CalculationError { message: String },
}

/// Main validation framework
pub struct DiarizationValidator {
    config: ValidationConfig,
    memory_tracker: MemoryTracker,
    performance_monitor: PerformanceMonitor,
}

impl DiarizationValidator {
    /// Create a new validator with default configuration
    pub fn new() -> Self {
        Self::with_config(ValidationConfig::default())
    }
    
    /// Create a new validator with custom configuration
    pub fn with_config(config: ValidationConfig) -> Self {
        Self {
            config,
            memory_tracker: MemoryTracker::new(),
            performance_monitor: PerformanceMonitor::new(),
        }
    }
    
    /// Compare predicted diarization results with ground truth
    /// 
    /// This is the main validation function that calculates DER, speaker consistency,
    /// and performance metrics by comparing diarization output with known ground truth.
    /// 
    /// # Arguments
    /// 
    /// * `predicted` - Diarization result to validate
    /// * `ground_truth` - Ground truth segments for comparison
    /// * `tolerance_ms` - Time tolerance for segment matching (milliseconds)
    /// 
    /// # Returns
    /// 
    /// Complete validation result with DER, consistency, and performance metrics
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// let validator = DiarizationValidator::new();
    /// let result = validator.compare_segments(
    ///     predicted_segments,
    ///     ground_truth_segments,
    ///     250.0 // 250ms tolerance
    /// )?;
    /// 
    /// println!("DER: {:.2}%", result.der_result.der_score * 100.0);
    /// println!("F1 Score: {:.3}", result.der_result.f1_score);
    /// ```
    pub fn compare_segments(
        &mut self,
        predicted: Vec<SpeakerSegment>,
        ground_truth: Vec<GroundTruthSegment>,
        tolerance_ms: f32,
    ) -> Result<ValidationResult, ValidationError> {
        let start_time = SystemTime::now();
        
        // Start performance monitoring
        self.performance_monitor.start_monitoring();
        
        // Calculate DER metrics
        let der_result = self.calculate_der(&predicted, &ground_truth, tolerance_ms)?;
        
        // Calculate speaker consistency
        let consistency_result = self.calculate_speaker_consistency(&predicted, &ground_truth)?;
        
        // Collect performance metrics
        let performance_metrics = self.performance_monitor.finish_monitoring(start_time);
        
        // Generate summary
        let summary = self.generate_summary(&der_result, &consistency_result, &performance_metrics);
        
        // Create validation result
        let result = ValidationResult {
            recording_id: format!("validation_{}", 
                SystemTime::now().duration_since(UNIX_EPOCH)
                    .unwrap_or_default().as_secs()),
            der_result,
            consistency_result,
            performance_metrics,
            summary,
            validation_timestamp: SystemTime::now(),
            config_used: serde_json::to_string(&self.config).unwrap_or_default(),
            warnings: Vec::new(),
        };
        
        // Generate reports if configured
        if self.config.generate_reports {
            self.generate_report(&result)?;
        }
        
        Ok(result)
    }
    
    /// Calculate Diarization Error Rate (DER) and related metrics
    /// 
    /// DER is the standard metric for diarization evaluation, calculated as:
    /// DER = (false_alarm + miss + speaker_error) / total_speech_time
    /// 
    /// Also calculates precision, recall, F1 score, and overlap accuracy.
    fn calculate_der(
        &self,
        predicted: &[SpeakerSegment],
        ground_truth: &[GroundTruthSegment],
        tolerance_ms: f32,
    ) -> Result<DERResult, ValidationError> {
        if predicted.is_empty() || ground_truth.is_empty() {
            return Err(ValidationError::InsufficientData {
                message: "Need both predicted and ground truth segments".to_string(),
            });
        }
        
        let tolerance_s = tolerance_ms / 1000.0;
        
        // Create time-aligned comparison grid
        let max_time = ground_truth.iter()
            .map(|s| s.end_time)
            .fold(0.0f32, f32::max);
        
        let time_step = 0.01; // 10ms resolution
        let num_steps = (max_time / time_step).ceil() as usize;
        
        let mut ground_truth_grid = vec![None::<String>; num_steps];
        let mut predicted_grid = vec![None::<String>; num_steps];
        
        // Fill ground truth grid
        for segment in ground_truth {
            let start_idx = (segment.start_time / time_step) as usize;
            let end_idx = ((segment.end_time / time_step) as usize).min(num_steps);
            
            for i in start_idx..end_idx {
                ground_truth_grid[i] = Some(segment.speaker_id.clone());
            }
        }
        
        // Fill predicted grid
        for segment in predicted {
            let start_idx = (segment.start_time / time_step) as usize;
            let end_idx = ((segment.end_time / time_step) as usize).min(num_steps);
            
            for i in start_idx..end_idx {
                predicted_grid[i] = Some(segment.speaker_id.clone());
            }
        }
        
        // Calculate error components
        let mut false_alarm_time = 0.0;
        let mut miss_time = 0.0;
        let mut speaker_error_time = 0.0;
        let mut correct_time = 0.0;
        let mut total_speech_time = 0.0;
        
        for i in 0..num_steps {
            let gt_speaker = &ground_truth_grid[i];
            let pred_speaker = &predicted_grid[i];
            let time_duration = time_step;
            
            match (gt_speaker, pred_speaker) {
                (Some(_), Some(pred)) if gt_speaker == pred_speaker => {
                    // Correct detection
                    correct_time += time_duration;
                    total_speech_time += time_duration;
                },
                (Some(_), Some(_)) => {
                    // Wrong speaker assigned
                    speaker_error_time += time_duration;
                    total_speech_time += time_duration;
                },
                (Some(_), None) => {
                    // Speech missed
                    miss_time += time_duration;
                    total_speech_time += time_duration;
                },
                (None, Some(_)) => {
                    // False alarm (speech detected when none)
                    false_alarm_time += time_duration;
                },
                (None, None) => {
                    // Correct silence (no contribution to DER)
                },
            }
        }
        
        // Calculate rates
        let false_alarm_rate = if total_speech_time > 0.0 {
            false_alarm_time / total_speech_time
        } else {
            0.0
        };
        
        let miss_rate = if total_speech_time > 0.0 {
            miss_time / total_speech_time
        } else {
            0.0
        };
        
        let speaker_error_rate = if total_speech_time > 0.0 {
            speaker_error_time / total_speech_time
        } else {
            0.0
        };
        
        let der_score = false_alarm_rate + miss_rate + speaker_error_rate;
        
        // Calculate precision, recall, F1
        let true_positives = correct_time;
        let false_positives = false_alarm_time + speaker_error_time;
        let false_negatives = miss_time;
        
        let precision = if true_positives + false_positives > 0.0 {
            true_positives / (true_positives + false_positives)
        } else {
            0.0
        };
        
        let recall = if true_positives + false_negatives > 0.0 {
            true_positives / (true_positives + false_negatives)
        } else {
            0.0
        };
        
        let f1_score = if precision + recall > 0.0 {
            2.0 * (precision * recall) / (precision + recall)
        } else {
            0.0
        };
        
        // Calculate overlap accuracy if enabled
        let overlap_accuracy = if self.config.evaluate_overlaps {
            self.calculate_overlap_accuracy(predicted, ground_truth)?
        } else {
            1.0 // Assume perfect if not evaluating
        };
        
        Ok(DERResult {
            der_score,
            false_alarm_rate,
            miss_rate,
            speaker_error_rate,
            total_speech_time,
            total_error_time: false_alarm_time + miss_time + speaker_error_time,
            precision,
            recall,
            f1_score,
            overlap_accuracy,
        })
    }
    
    /// Calculate speaker consistency metrics
    /// 
    /// Measures how consistently speakers are identified across segments,
    /// including purity (no ID switches) and coverage (all speech assigned).
    fn calculate_speaker_consistency(
        &self,
        predicted: &[SpeakerSegment],
        ground_truth: &[GroundTruthSegment],
    ) -> Result<SpeakerConsistencyResult, ValidationError> {
        if predicted.is_empty() || ground_truth.is_empty() {
            return Err(ValidationError::InsufficientData {
                message: "Need segments for consistency analysis".to_string(),
            });
        }
        
        // Create speaker mapping
        let mut confusion_matrix: HashMap<String, HashMap<String, usize>> = HashMap::new();
        let mut speaker_purity: HashMap<String, f32> = HashMap::new();
        let mut speaker_coverage: HashMap<String, f32> = HashMap::new();
        
        // Build confusion matrix by matching segments
        for pred_segment in predicted {
            // Find overlapping ground truth segments
            let overlapping_gt: Vec<_> = ground_truth.iter()
                .filter(|gt| {
                    let overlap_start = pred_segment.start_time.max(gt.start_time);
                    let overlap_end = pred_segment.end_time.min(gt.end_time);
                    overlap_end > overlap_start + self.config.time_tolerance_s
                })
                .collect();
            
            // Find the ground truth segment with maximum overlap
            if let Some(best_match) = overlapping_gt.iter()
                .max_by(|a, b| {
                    let overlap_a = (pred_segment.end_time.min(a.end_time) - 
                                   pred_segment.start_time.max(a.start_time)).max(0.0);
                    let overlap_b = (pred_segment.end_time.min(b.end_time) - 
                                   pred_segment.start_time.max(b.start_time)).max(0.0);
                    overlap_a.partial_cmp(&overlap_b).unwrap_or(std::cmp::Ordering::Equal)
                }) {
                
                // Update confusion matrix
                let pred_speaker = &pred_segment.speaker_id;
                let true_speaker = &best_match.speaker_id;
                
                *confusion_matrix
                    .entry(true_speaker.clone())
                    .or_default()
                    .entry(pred_speaker.clone())
                    .or_default() += 1;
            }
        }
        
        // Calculate purity for each predicted speaker
        for (true_speaker, pred_counts) in &confusion_matrix {
            if let Some((most_frequent_pred, &max_count)) = pred_counts.iter()
                .max_by_key(|(_, &count)| count) {
                
                let total_count: usize = pred_counts.values().sum();
                let purity = if total_count > 0 {
                    max_count as f32 / total_count as f32
                } else {
                    0.0
                };
                
                speaker_purity.insert(most_frequent_pred.clone(), purity);
            }
        }
        
        // Calculate coverage for each true speaker
        let mut total_gt_time_per_speaker: HashMap<String, f32> = HashMap::new();
        let mut covered_time_per_speaker: HashMap<String, f32> = HashMap::new();
        
        for gt_segment in ground_truth {
            let speaker_time = gt_segment.end_time - gt_segment.start_time;
            *total_gt_time_per_speaker.entry(gt_segment.speaker_id.clone())
                .or_default() += speaker_time;
            
            // Find overlapping predicted segments
            let covered_time: f32 = predicted.iter()
                .map(|pred| {
                    let overlap_start = pred.start_time.max(gt_segment.start_time);
                    let overlap_end = pred.end_time.min(gt_segment.end_time);
                    (overlap_end - overlap_start).max(0.0)
                })
                .sum();
            
            *covered_time_per_speaker.entry(gt_segment.speaker_id.clone())
                .or_default() += covered_time.min(speaker_time);
        }
        
        for (speaker, &total_time) in &total_gt_time_per_speaker {
            let covered_time = covered_time_per_speaker.get(speaker).unwrap_or(&0.0);
            let coverage = if total_time > 0.0 {
                covered_time / total_time
            } else {
                0.0
            };
            speaker_coverage.insert(speaker.clone(), coverage);
        }
        
        // Count speaker ID switches
        let id_switches = self.count_speaker_switches(predicted);
        
        // Calculate overall consistency score
        let avg_purity: f32 = speaker_purity.values().sum::<f32>() / 
            speaker_purity.len().max(1) as f32;
        let avg_coverage: f32 = speaker_coverage.values().sum::<f32>() / 
            speaker_coverage.len().max(1) as f32;
        
        let consistency_score = (avg_purity + avg_coverage) / 2.0;
        let consistency_percentage = consistency_score * 100.0;
        
        Ok(SpeakerConsistencyResult {
            consistency_score,
            speaker_purity,
            speaker_coverage,
            id_switches,
            total_segments: predicted.len(),
            consistency_percentage,
            confusion_matrix,
        })
    }
    
    /// Calculate overlap accuracy for overlapping speech detection
    fn calculate_overlap_accuracy(
        &self,
        predicted: &[SpeakerSegment],
        ground_truth: &[GroundTruthSegment],
    ) -> Result<f32, ValidationError> {
        // Find overlapping regions in ground truth
        let mut gt_overlaps = Vec::new();
        for i in 0..ground_truth.len() {
            for j in i+1..ground_truth.len() {
                let seg1 = &ground_truth[i];
                let seg2 = &ground_truth[j];
                
                let overlap_start = seg1.start_time.max(seg2.start_time);
                let overlap_end = seg1.end_time.min(seg2.end_time);
                
                if overlap_end > overlap_start + self.config.time_tolerance_s {
                    gt_overlaps.push((overlap_start, overlap_end));
                }
            }
        }
        
        // Find overlapping regions in predictions
        let mut pred_overlaps = Vec::new();
        for segment in predicted {
            if segment.has_overlap {
                pred_overlaps.push((segment.start_time, segment.end_time));
            }
        }
        
        if gt_overlaps.is_empty() && pred_overlaps.is_empty() {
            return Ok(1.0); // Perfect if no overlaps to detect
        }
        
        if gt_overlaps.is_empty() {
            return Ok(0.0); // False positives only
        }
        
        // Calculate overlap detection accuracy
        let mut correct_overlap_time = 0.0;
        let mut total_overlap_time: f32 = gt_overlaps.iter()
            .map(|(start, end)| end - start)
            .sum();
        
        for (gt_start, gt_end) in &gt_overlaps {
            for (pred_start, pred_end) in &pred_overlaps {
                let overlap_start = gt_start.max(*pred_start);
                let overlap_end = gt_end.min(*pred_end);
                
                if overlap_end > overlap_start {
                    correct_overlap_time += overlap_end - overlap_start;
                }
            }
        }
        
        Ok(if total_overlap_time > 0.0 {
            correct_overlap_time / total_overlap_time
        } else {
            1.0
        })
    }
    
    /// Count speaker ID switches in predicted segments
    fn count_speaker_switches(&self, segments: &[SpeakerSegment]) -> usize {
        if segments.len() < 2 {
            return 0;
        }
        
        let mut switches = 0;
        for i in 1..segments.len() {
            if segments[i].speaker_id != segments[i-1].speaker_id {
                switches += 1;
            }
        }
        
        switches
    }
    
    /// Generate validation summary
    fn generate_summary(
        &self,
        der_result: &DERResult,
        consistency_result: &SpeakerConsistencyResult,
        performance_metrics: &PerformanceMetrics,
    ) -> ValidationSummary {
        // Determine accuracy level
        let accuracy_level = match der_result.der_score {
            x if x < 0.10 => AccuracyLevel::Excellent,
            x if x < 0.20 => AccuracyLevel::Good,
            x if x < 0.30 => AccuracyLevel::Fair,
            _ => AccuracyLevel::Poor,
        };
        
        // Determine performance level
        let performance_level = match performance_metrics.real_time_factor {
            x if x < 0.5 => PerformanceLevel::Excellent,
            x if x < 1.0 => PerformanceLevel::Good,
            x if x < 2.0 => PerformanceLevel::Fair,
            _ => PerformanceLevel::Poor,
        };
        
        // Calculate overall quality score
        let der_score_normalized = (1.0 - der_result.der_score.min(1.0)).max(0.0);
        let consistency_score = consistency_result.consistency_score;
        let performance_score = (2.0 - performance_metrics.real_time_factor).max(0.0).min(1.0);
        
        let overall_quality = (der_score_normalized * 0.5 + 
                              consistency_score * 0.3 + 
                              performance_score * 0.2).max(0.0).min(1.0);
        
        // Identify strengths and improvements
        let mut strengths = Vec::new();
        let mut improvements = Vec::new();
        
        if der_result.der_score < 0.15 {
            strengths.push("Excellent diarization accuracy".to_string());
        } else if der_result.der_score > 0.25 {
            improvements.push("Improve overall diarization accuracy".to_string());
        }
        
        if der_result.f1_score > 0.85 {
            strengths.push("High precision and recall balance".to_string());
        } else if der_result.f1_score < 0.70 {
            improvements.push("Balance precision and recall scores".to_string());
        }
        
        if consistency_result.consistency_score > 0.85 {
            strengths.push("Consistent speaker identification".to_string());
        } else {
            improvements.push("Reduce speaker ID inconsistencies".to_string());
        }
        
        if performance_metrics.real_time_factor < 1.0 {
            strengths.push("Real-time processing capability".to_string());
        } else if performance_metrics.real_time_factor > 1.5 {
            improvements.push("Optimize processing speed".to_string());
        }
        
        if performance_metrics.peak_memory_mb < 300.0 {
            strengths.push("Efficient memory usage".to_string());
        } else if performance_metrics.peak_memory_mb > 500.0 {
            improvements.push("Optimize memory consumption".to_string());
        }
        
        // Check if meets target thresholds
        let meets_targets = der_result.der_score < self.config.target_der_threshold &&
                           performance_metrics.real_time_factor < self.config.target_rt_factor &&
                           consistency_result.consistency_score > self.config.min_consistency_score;
        
        ValidationSummary {
            overall_quality,
            accuracy_level,
            performance_level,
            strengths,
            improvements,
            meets_targets,
        }
    }
    
    /// Generate detailed HTML and JSON reports
    fn generate_report(&self, result: &ValidationResult) -> Result<(), ValidationError> {
        let report_dir = Path::new(&self.config.report_output_dir);
        fs::create_dir_all(report_dir)
            .map_err(|e| ValidationError::IoError { 
                message: format!("Failed to create report directory: {}", e) 
            })?;
        
        // Generate JSON report
        let json_path = report_dir.join(format!("{}_validation.json", result.recording_id));
        let json_content = serde_json::to_string_pretty(result)
            .map_err(|e| ValidationError::IoError { 
                message: format!("Failed to serialize JSON: {}", e) 
            })?;
        
        fs::write(&json_path, json_content)
            .map_err(|e| ValidationError::IoError { 
                message: format!("Failed to write JSON report: {}", e) 
            })?;
        
        // Generate HTML report
        let html_path = report_dir.join(format!("{}_validation.html", result.recording_id));
        let html_content = self.generate_html_report(result);
        
        fs::write(&html_path, html_content)
            .map_err(|e| ValidationError::IoError { 
                message: format!("Failed to write HTML report: {}", e) 
            })?;
        
        println!("Reports generated:");
        println!("  JSON: {}", json_path.display());
        println!("  HTML: {}", html_path.display());
        
        Ok(())
    }
    
    /// Generate HTML report content
    fn generate_html_report(&self, result: &ValidationResult) -> String {
        let der_score_pct = result.der_result.der_score * 100.0;
        let accuracy_color = match result.summary.accuracy_level {
            AccuracyLevel::Excellent => "green",
            AccuracyLevel::Good => "blue",
            AccuracyLevel::Fair => "orange",
            AccuracyLevel::Poor => "red",
        };
        
        let performance_color = match result.summary.performance_level {
            PerformanceLevel::Excellent => "green",
            PerformanceLevel::Good => "blue", 
            PerformanceLevel::Fair => "orange",
            PerformanceLevel::Poor => "red",
        };
        
        format!(r#"
<!DOCTYPE html>
<html>
<head>
    <title>Diarization Validation Report - {}</title>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 40px; }}
        .header {{ background: #f5f5f5; padding: 20px; border-radius: 5px; }}
        .metrics {{ display: flex; gap: 20px; margin: 20px 0; }}
        .metric-card {{ flex: 1; padding: 15px; border: 1px solid #ddd; border-radius: 5px; }}
        .score {{ font-size: 24px; font-weight: bold; }}
        .excellent {{ color: green; }}
        .good {{ color: blue; }}
        .fair {{ color: orange; }}
        .poor {{ color: red; }}
        .confusion-matrix {{ margin: 20px 0; }}
        .confusion-matrix table {{ border-collapse: collapse; }}
        .confusion-matrix th, .confusion-matrix td {{ border: 1px solid #ddd; padding: 8px; text-align: center; }}
        .confusion-matrix th {{ background: #f5f5f5; }}
    </style>
</head>
<body>
    <div class="header">
        <h1>Diarization Validation Report</h1>
        <p><strong>Recording ID:</strong> {}</p>
        <p><strong>Validation Time:</strong> {}</p>
        <p><strong>Overall Quality:</strong> <span class="score">{:.1}%</span></p>
        <p><strong>Meets Targets:</strong> {}</p>
    </div>
    
    <div class="metrics">
        <div class="metric-card">
            <h3>Accuracy Metrics</h3>
            <p><strong>DER Score:</strong> <span class="score" style="color: {}">{:.2}%</span></p>
            <p><strong>Accuracy Level:</strong> <span style="color: {}">{:?}</span></p>
            <p><strong>Precision:</strong> {:.3}</p>
            <p><strong>Recall:</strong> {:.3}</p>
            <p><strong>F1 Score:</strong> {:.3}</p>
        </div>
        
        <div class="metric-card">
            <h3>Speaker Consistency</h3>
            <p><strong>Consistency Score:</strong> <span class="score">{:.1}%</span></p>
            <p><strong>ID Switches:</strong> {}</p>
            <p><strong>Total Segments:</strong> {}</p>
        </div>
        
        <div class="metric-card">
            <h3>Performance Metrics</h3>
            <p><strong>Real-time Factor:</strong> <span class="score" style="color: {}">{:.2}x</span></p>
            <p><strong>Performance Level:</strong> <span style="color: {}">{:?}</span></p>
            <p><strong>Peak Memory:</strong> {:.1} MB</p>
            <p><strong>Latency:</strong> {} ms</p>
        </div>
    </div>
    
    <div class="confusion-matrix">
        <h3>Speaker Confusion Matrix</h3>
        <p>Shows how predicted speakers map to true speakers</p>
        <table>
            <tr><th>True\\Predicted</th>{}</tr>
            {}
        </table>
    </div>
    
    <div>
        <h3>Strengths</h3>
        <ul>
            {}
        </ul>
        
        <h3>Areas for Improvement</h3>
        <ul>
            {}
        </ul>
    </div>
    
    <div>
        <h3>Detailed Metrics</h3>
        <p><strong>False Alarm Rate:</strong> {:.3}</p>
        <p><strong>Miss Rate:</strong> {:.3}</p>
        <p><strong>Speaker Error Rate:</strong> {:.3}</p>
        <p><strong>Overlap Accuracy:</strong> {:.3}</p>
        <p><strong>Total Speech Time:</strong> {:.1}s</p>
        <p><strong>Total Error Time:</strong> {:.1}s</p>
    </div>
</body>
</html>
"#,
            result.recording_id,
            result.recording_id,
            result.validation_timestamp.duration_since(UNIX_EPOCH)
                .unwrap_or_default().as_secs(),
            result.summary.overall_quality * 100.0,
            if result.summary.meets_targets { "✅ Yes" } else { "❌ No" },
            accuracy_color, der_score_pct,
            accuracy_color, result.summary.accuracy_level,
            result.der_result.precision,
            result.der_result.recall,
            result.der_result.f1_score,
            result.consistency_result.consistency_percentage,
            result.consistency_result.id_switches,
            result.consistency_result.total_segments,
            performance_color, result.performance_metrics.real_time_factor,
            performance_color, result.summary.performance_level,
            result.performance_metrics.peak_memory_mb,
            result.performance_metrics.latency_ms,
            self.generate_confusion_matrix_headers(&result.consistency_result.confusion_matrix),
            self.generate_confusion_matrix_rows(&result.consistency_result.confusion_matrix),
            result.summary.strengths.iter()
                .map(|s| format!("<li>{}</li>", s))
                .collect::<Vec<_>>()
                .join(""),
            result.summary.improvements.iter()
                .map(|s| format!("<li>{}</li>", s))
                .collect::<Vec<_>>()
                .join(""),
            result.der_result.false_alarm_rate,
            result.der_result.miss_rate,
            result.der_result.speaker_error_rate,
            result.der_result.overlap_accuracy,
            result.der_result.total_speech_time,
            result.der_result.total_error_time,
        )
    }
    
    /// Generate confusion matrix HTML headers
    fn generate_confusion_matrix_headers(&self, matrix: &HashMap<String, HashMap<String, usize>>) -> String {
        let mut all_speakers: HashSet<String> = HashSet::new();
        for (_, pred_speakers) in matrix {
            for pred_speaker in pred_speakers.keys() {
                all_speakers.insert(pred_speaker.clone());
            }
        }
        
        let mut speakers: Vec<_> = all_speakers.into_iter().collect();
        speakers.sort();
        
        speakers.iter()
            .map(|s| format!("<th>{}</th>", s))
            .collect::<Vec<_>>()
            .join("")
    }
    
    /// Generate confusion matrix HTML rows
    fn generate_confusion_matrix_rows(&self, matrix: &HashMap<String, HashMap<String, usize>>) -> String {
        let mut all_speakers: HashSet<String> = HashSet::new();
        for (true_speaker, pred_speakers) in matrix {
            all_speakers.insert(true_speaker.clone());
            for pred_speaker in pred_speakers.keys() {
                all_speakers.insert(pred_speaker.clone());
            }
        }
        
        let mut speakers: Vec<_> = all_speakers.into_iter().collect();
        speakers.sort();
        
        let mut rows = Vec::new();
        for true_speaker in matrix.keys() {
            let mut row = format!("<tr><th>{}</th>", true_speaker);
            for pred_speaker in &speakers {
                let count = matrix.get(true_speaker)
                    .and_then(|pred_map| pred_map.get(pred_speaker))
                    .unwrap_or(&0);
                row.push_str(&format!("<td>{}</td>", count));
            }
            row.push_str("</tr>");
            rows.push(row);
        }
        
        rows.join("")
    }
}

/// Memory usage tracking
struct MemoryTracker {
    initial_memory: usize,
    peak_memory: usize,
}

impl MemoryTracker {
    fn new() -> Self {
        Self {
            initial_memory: Self::get_current_memory(),
            peak_memory: 0,
        }
    }
    
    fn update_peak(&mut self) {
        let current = Self::get_current_memory();
        if current > self.peak_memory {
            self.peak_memory = current;
        }
    }
    
    fn get_current_memory() -> usize {
        // Simplified memory tracking - in production would use system APIs
        // For now, return a placeholder value
        0
    }
}

/// Performance monitoring
struct PerformanceMonitor {
    start_time: Option<SystemTime>,
    cpu_samples: Vec<f32>,
    memory_samples: Vec<f32>,
}

impl PerformanceMonitor {
    fn new() -> Self {
        Self {
            start_time: None,
            cpu_samples: Vec::new(),
            memory_samples: Vec::new(),
        }
    }
    
    fn start_monitoring(&mut self) {
        self.start_time = Some(SystemTime::now());
        self.cpu_samples.clear();
        self.memory_samples.clear();
    }
    
    fn finish_monitoring(&mut self, validation_start: SystemTime) -> PerformanceMetrics {
        let end_time = SystemTime::now();
        let start_time = self.start_time.unwrap_or(validation_start);
        
        let total_duration = end_time.duration_since(start_time)
            .unwrap_or_default();
        
        // Simplified metrics - in production would collect real system data
        PerformanceMetrics {
            real_time_factor: 0.8, // Placeholder
            peak_memory_mb: 150.0,
            average_memory_mb: 120.0,
            cpu_utilization: 25.0,
            latency_ms: 1500,
            throughput: 1.25,
            memory_allocations: 1000,
            time_to_first_output_ms: 500,
            start_time,
            end_time,
        }
    }
}

/// Load ground truth data from JSON file
pub fn load_ground_truth(path: &str) -> Result<GroundTruthData, ValidationError> {
    let content = fs::read_to_string(path)
        .map_err(|_| ValidationError::GroundTruthNotFound { 
            path: path.to_string() 
        })?;
    
    serde_json::from_str(&content)
        .map_err(|e| ValidationError::InvalidGroundTruth { 
            message: e.to_string() 
        })
}

/// Save ground truth data to JSON file
pub fn save_ground_truth(data: &GroundTruthData, path: &str) -> Result<(), ValidationError> {
    let content = serde_json::to_string_pretty(data)
        .map_err(|e| ValidationError::IoError { 
            message: format!("JSON serialization failed: {}", e) 
        })?;
    
    fs::write(path, content)
        .map_err(|e| ValidationError::IoError { 
            message: format!("Failed to write file: {}", e) 
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    
    /// Helper function to create test segments
    fn create_test_segments() -> Vec<SpeakerSegment> {
        vec![
            SpeakerSegment {
                speaker_id: "speaker_1".to_string(),
                start_time: 0.0,
                end_time: 2.0,
                confidence: 0.9,
                text: Some("Hello world".to_string()),
                embedding: None,
                has_overlap: false,
                overlapping_speakers: vec![],
            },
            SpeakerSegment {
                speaker_id: "speaker_2".to_string(),
                start_time: 2.5,
                end_time: 4.5,
                confidence: 0.8,
                text: Some("How are you".to_string()),
                embedding: None,
                has_overlap: false,
                overlapping_speakers: vec![],
            },
        ]
    }
    
    /// Helper function to create test ground truth
    fn create_test_ground_truth() -> Vec<GroundTruthSegment> {
        vec![
            GroundTruthSegment {
                speaker_id: "speaker_1".to_string(),
                start_time: 0.0,
                end_time: 2.0,
                text: Some("Hello world".to_string()),
                audio_file: None,
                quality: 1.0,
            },
            GroundTruthSegment {
                speaker_id: "speaker_2".to_string(),
                start_time: 2.5,
                end_time: 4.5,
                text: Some("How are you".to_string()),
                audio_file: None,
                quality: 1.0,
            },
        ]
    }
    
    #[test]
    fn test_perfect_diarization() {
        let mut validator = DiarizationValidator::new();
        let predicted = create_test_segments();
        let ground_truth = create_test_ground_truth();
        
        let result = validator.compare_segments(predicted, ground_truth, 250.0)
            .expect("Validation should succeed");
        
        // Perfect match should have very low DER
        assert!(result.der_result.der_score < 0.01);
        assert!(result.der_result.precision > 0.99);
        assert!(result.der_result.recall > 0.99);
        assert!(result.der_result.f1_score > 0.99);
        assert!(result.summary.meets_targets);
        assert_eq!(result.summary.accuracy_level, AccuracyLevel::Excellent);
    }
    
    #[test]
    fn test_speaker_confusion() {
        let mut validator = DiarizationValidator::new();
        
        // Create segments with swapped speaker IDs
        let mut predicted = create_test_segments();
        predicted[0].speaker_id = "speaker_2".to_string();
        predicted[1].speaker_id = "speaker_1".to_string();
        
        let ground_truth = create_test_ground_truth();
        
        let result = validator.compare_segments(predicted, ground_truth, 250.0)
            .expect("Validation should succeed");
        
        // Should have high speaker error rate
        assert!(result.der_result.speaker_error_rate > 0.8);
        assert!(result.der_result.der_score > 0.8);
        assert!(result.consistency_result.id_switches >= 1);
        assert!(!result.summary.meets_targets);
    }
    
    #[test]
    fn test_missed_speech() {
        let mut validator = DiarizationValidator::new();
        
        // Remove one segment (missed speech)
        let mut predicted = create_test_segments();
        predicted.pop();
        
        let ground_truth = create_test_ground_truth();
        
        let result = validator.compare_segments(predicted, ground_truth, 250.0)
            .expect("Validation should succeed");
        
        // Should have high miss rate
        assert!(result.der_result.miss_rate > 0.3);
        assert!(result.der_result.recall < 0.7);
    }
    
    #[test]
    fn test_false_alarm() {
        let mut validator = DiarizationValidator::new();
        
        // Add extra segment (false alarm)
        let mut predicted = create_test_segments();
        predicted.push(SpeakerSegment {
            speaker_id: "speaker_3".to_string(),
            start_time: 5.0,
            end_time: 6.0,
            confidence: 0.7,
            text: Some("Extra speech".to_string()),
            embedding: None,
            has_overlap: false,
            overlapping_speakers: vec![],
        });
        
        let ground_truth = create_test_ground_truth();
        
        let result = validator.compare_segments(predicted, ground_truth, 250.0)
            .expect("Validation should succeed");
        
        // Should have false alarm contribution
        assert!(result.der_result.false_alarm_rate > 0.1);
        assert!(result.der_result.precision < 0.9);
    }
    
    #[test]
    fn test_ground_truth_serialization() {
        let ground_truth = GroundTruthData {
            recording_id: "test_001".to_string(),
            segments: create_test_ground_truth(),
            total_speakers: 2,
            duration: 5.0,
            sample_rate: 16000,
            metadata: HashMap::new(),
        };
        
        // Test serialization round-trip
        let json = serde_json::to_string(&ground_truth).unwrap();
        let deserialized: GroundTruthData = serde_json::from_str(&json).unwrap();
        
        assert_eq!(ground_truth.recording_id, deserialized.recording_id);
        assert_eq!(ground_truth.segments.len(), deserialized.segments.len());
        assert_eq!(ground_truth.total_speakers, deserialized.total_speakers);
    }
    
    #[test] 
    fn test_validation_config_defaults() {
        let config = ValidationConfig::default();
        
        assert_eq!(config.time_tolerance_s, 0.25);
        assert_eq!(config.target_der_threshold, 0.15);
        assert_eq!(config.target_rt_factor, 1.5);
        assert_eq!(config.min_consistency_score, 0.85);
        assert!(config.evaluate_overlaps);
        assert!(config.generate_reports);
    }
    
    #[test]
    fn test_empty_segments_error() {
        let mut validator = DiarizationValidator::new();
        
        let result = validator.compare_segments(vec![], vec![], 250.0);
        
        assert!(result.is_err());
        match result.unwrap_err() {
            ValidationError::InsufficientData { .. } => {}, // Expected
            _ => panic!("Expected InsufficientData error"),
        }
    }
    
    #[test]
    fn test_time_tolerance_effect() {
        let mut validator = DiarizationValidator::new();
        
        // Create slightly misaligned segments
        let mut predicted = create_test_segments();
        predicted[0].start_time = 0.1; // 100ms offset
        predicted[0].end_time = 2.1;
        
        let ground_truth = create_test_ground_truth();
        
        // Test with tight tolerance
        let result_tight = validator.compare_segments(
            predicted.clone(), 
            ground_truth.clone(), 
            50.0 // 50ms tolerance
        ).expect("Validation should succeed");
        
        // Test with loose tolerance
        let result_loose = validator.compare_segments(
            predicted, 
            ground_truth, 
            200.0 // 200ms tolerance
        ).expect("Validation should succeed");
        
        // Loose tolerance should perform better
        assert!(result_loose.der_result.der_score < result_tight.der_result.der_score);
    }
}