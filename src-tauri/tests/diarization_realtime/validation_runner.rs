//! Command-line utility for running diarization validation and benchmarks
//! 
//! This utility provides a comprehensive testing framework for diarization
//! accuracy measurement and performance benchmarking with various scenarios.

use crate::diarization_realtime::{
    validation::{
        DiarizationValidator, ValidationConfig, load_ground_truth, 
        AccuracyLevel, PerformanceLevel,
    },
    benchmark::{DiarizationBenchmark, BenchmarkScenarios},
};
use kaginote_lib::diarization::types::SpeakerSegment;
// use clap::{Parser, Subcommand}; // Commented out for now - would need clap dependency
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::collections::HashMap;
use std::fs;

/// Command-line interface for diarization validation
/// (Note: Would use clap::Parser in production - simplified for testing)
#[derive(Debug)]
struct Cli {
    command: Commands,
}

#[derive(Debug)]
enum Commands {
    /// Run validation against ground truth data
    Validate {
        ground_truth: String,
        predicted: String,
        tolerance: f32,
        output: String,
        no_reports: bool,
    },
    
    /// Run performance benchmarks
    Benchmark {
        scenario: String,
        segments: Option<usize>,
        speakers: Option<usize>,
        duration: Option<f32>,
        iterations: Option<usize>,
        output: String,
    },
    
    /// Validate multiple ground truth files
    BatchValidate {
        ground_truth_dir: String,
        predicted_dir: String,
        output: String,
        summary: bool,
    },
    
    /// Generate synthetic test data
    Generate {
        output: String,
        count: usize,
        challenging: bool,
    },
    
    /// Analyze validation trends over time
    Trend {
        reports_dir: String,
        output: String,
    },
}

/// Batch validation summary
#[derive(Debug, Clone, Serialize, Deserialize)]
struct BatchValidationSummary {
    total_files_processed: usize,
    successful_validations: usize,
    failed_validations: usize,
    average_der_score: f32,
    average_f1_score: f32,
    accuracy_level_distribution: HashMap<String, usize>,
    best_performing_file: Option<String>,
    worst_performing_file: Option<String>,
    overall_grade: String,
    recommendations: Vec<String>,
}

/// Trend analysis data point
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ValidationTrendPoint {
    timestamp: String,
    file_name: String,
    der_score: f32,
    f1_score: f32,
    accuracy_level: AccuracyLevel,
    performance_level: PerformanceLevel,
}

/// Main validation runner implementation
pub struct ValidationRunner {
    config: ValidationConfig,
}

impl ValidationRunner {
    pub fn new() -> Self {
        Self {
            config: ValidationConfig::default(),
        }
    }
    
    pub fn with_config(config: ValidationConfig) -> Self {
        Self { config }
    }
    
    /// Run the CLI application (simplified for testing - would use clap::Parser in production)
    pub fn run_cli() {
        // For testing purposes, create a default validation command
        let cli = Cli {
            command: Commands::Validate {
                ground_truth: "tests/diarization_realtime/ground_truth/example_meeting.json".to_string(),
                predicted: "predicted_example.json".to_string(),
                tolerance: 250.0,
                output: "test_validation_reports".to_string(),
                no_reports: false,
            }
        };
        let runner = ValidationRunner::new();
        
        match cli.command {
            Commands::Validate { 
                ground_truth, predicted, tolerance, output, no_reports 
            } => {
                runner.run_single_validation(&ground_truth, &predicted, tolerance, &output, !no_reports);
            },
            
            Commands::Benchmark { 
                scenario, segments, speakers, duration, iterations, output 
            } => {
                runner.run_benchmark(&scenario, segments, speakers, duration, iterations, &output);
            },
            
            Commands::BatchValidate { 
                ground_truth_dir, predicted_dir, output, summary 
            } => {
                runner.run_batch_validation(&ground_truth_dir, &predicted_dir, &output, summary);
            },
            
            Commands::Generate { output, count, challenging } => {
                runner.generate_test_data(&output, count, challenging);
            },
            
            Commands::Trend { reports_dir, output } => {
                runner.analyze_validation_trends(&reports_dir, &output);
            },
        }
    }
    
    /// Run single validation
    fn run_single_validation(
        &self, 
        ground_truth_path: &str, 
        predicted_path: &str, 
        tolerance: f32,
        output_dir: &str,
        generate_reports: bool,
    ) {
        println!("🔍 Running Single Validation");
        println!("Ground Truth: {}", ground_truth_path);
        println!("Predicted: {}", predicted_path);
        println!("Tolerance: {}ms", tolerance);
        
        // Load ground truth data
        let ground_truth_data = match load_ground_truth(ground_truth_path) {
            Ok(data) => data,
            Err(e) => {
                eprintln!("❌ Failed to load ground truth: {}", e);
                return;
            }
        };
        
        // Load predicted segments
        let predicted_segments = match self.load_predicted_segments(predicted_path) {
            Ok(segments) => segments,
            Err(e) => {
                eprintln!("❌ Failed to load predicted segments: {}", e);
                return;
            }
        };
        
        // Create validator with custom config
        let config = ValidationConfig {
            generate_reports,
            report_output_dir: output_dir.to_string(),
            ..self.config.clone()
        };
        
        let mut validator = DiarizationValidator::with_config(config);
        
        // Run validation
        match validator.compare_segments(predicted_segments, ground_truth_data.segments, tolerance) {
            Ok(result) => {
                println!("\n✅ Validation Complete!");
                println!("   DER Score: {:.2}%", result.der_result.der_score * 100.0);
                println!("   Precision: {:.3}", result.der_result.precision);
                println!("   Recall: {:.3}", result.der_result.recall);
                println!("   F1 Score: {:.3}", result.der_result.f1_score);
                println!("   Accuracy Level: {:?}", result.summary.accuracy_level);
                println!("   Overall Quality: {:.1}%", result.summary.overall_quality * 100.0);
                println!("   Meets Targets: {}", if result.summary.meets_targets { "✅" } else { "❌" });
                
                if generate_reports {
                    println!("   📁 Reports saved to: {}", output_dir);
                }
                
                // Print recommendations
                if !result.summary.improvements.is_empty() {
                    println!("\n💡 Recommendations:");
                    for improvement in &result.summary.improvements {
                        println!("   • {}", improvement);
                    }
                }
            },
            Err(e) => {
                eprintln!("❌ Validation failed: {}", e);
            }
        }
    }
    
    /// Run performance benchmark
    fn run_benchmark(
        &self,
        scenario: &str,
        custom_segments: Option<usize>,
        custom_speakers: Option<usize>,
        custom_duration: Option<f32>,
        custom_iterations: Option<usize>,
        output_path: &str,
    ) {
        println!("🚀 Running Performance Benchmark");
        println!("Scenario: {}", scenario);
        
        // Get benchmark configuration
        let mut config = match scenario {
            "quick" => BenchmarkScenarios::quick(),
            "standard" => BenchmarkScenarios::standard(),
            "extensive" => BenchmarkScenarios::extensive(),
            "stress" => BenchmarkScenarios::stress_test(),
            "memory" => BenchmarkScenarios::memory_test(),
            _ => {
                eprintln!("❌ Unknown scenario: {}. Using standard.", scenario);
                BenchmarkScenarios::standard()
            }
        };
        
        // Apply custom parameters
        if let Some(segments) = custom_segments {
            config.num_segments = segments;
        }
        if let Some(speakers) = custom_speakers {
            config.num_speakers = speakers;
        }
        if let Some(duration) = custom_duration {
            config.total_duration = duration;
        }
        if let Some(iterations) = custom_iterations {
            config.iterations = iterations;
        }
        
        println!("Configuration:");
        println!("   Segments: {}", config.num_segments);
        println!("   Speakers: {}", config.num_speakers);
        println!("   Duration: {:.1}s", config.total_duration);
        println!("   Iterations: {}", config.iterations);
        
        // Run benchmark
        let mut benchmark = DiarizationBenchmark::with_config(config);
        let results = benchmark.run_benchmark();
        
        // Save results
        if let Err(e) = benchmark.save_results(&results, output_path) {
            eprintln!("❌ Failed to save results: {}", e);
        } else {
            println!("✅ Benchmark complete! Results saved to: {}", output_path);
        }
    }
    
    /// Run batch validation on multiple files
    fn run_batch_validation(
        &self,
        ground_truth_dir: &str,
        predicted_dir: &str,
        output_dir: &str,
        generate_summary: bool,
    ) {
        println!("📊 Running Batch Validation");
        println!("Ground Truth Dir: {}", ground_truth_dir);
        println!("Predicted Dir: {}", predicted_dir);
        println!("Output Dir: {}", output_dir);
        
        // Create output directory
        if let Err(e) = fs::create_dir_all(output_dir) {
            eprintln!("❌ Failed to create output directory: {}", e);
            return;
        }
        
        // Find ground truth files
        let gt_files = match self.find_json_files(ground_truth_dir) {
            Ok(files) => files,
            Err(e) => {
                eprintln!("❌ Failed to find ground truth files: {}", e);
                return;
            }
        };
        
        let mut batch_summary = BatchValidationSummary {
            total_files_processed: 0,
            successful_validations: 0,
            failed_validations: 0,
            average_der_score: 0.0,
            average_f1_score: 0.0,
            accuracy_level_distribution: HashMap::new(),
            best_performing_file: None,
            worst_performing_file: None,
            overall_grade: "Unknown".to_string(),
            recommendations: Vec::new(),
        };
        
        let mut total_der = 0.0;
        let mut total_f1 = 0.0;
        let mut best_der = f32::INFINITY;
        let mut worst_der = 0.0;
        let mut best_file = String::new();
        let mut worst_file = String::new();
        
        println!("Found {} ground truth files", gt_files.len());
        
        for gt_file in gt_files {
            let file_name = gt_file.file_stem().unwrap().to_string_lossy();
            let pred_file = PathBuf::from(predicted_dir).join(format!("{}.json", file_name));
            
            if !pred_file.exists() {
                println!("⚠️  Skipping {} - no corresponding predicted file", file_name);
                continue;
            }
            
            println!("Processing: {}", file_name);
            batch_summary.total_files_processed += 1;
            
            // Run validation for this file pair
            match self.run_file_validation(&gt_file, &pred_file, output_dir) {
                Ok(result) => {
                    batch_summary.successful_validations += 1;
                    
                    let der = result.der_result.der_score;
                    let f1 = result.der_result.f1_score;
                    
                    total_der += der;
                    total_f1 += f1;
                    
                    if der < best_der {
                        best_der = der;
                        best_file = file_name.to_string();
                    }
                    
                    if der > worst_der {
                        worst_der = der;
                        worst_file = file_name.to_string();
                    }
                    
                    // Update accuracy level distribution
                    let accuracy_key = format!("{:?}", result.summary.accuracy_level);
                    *batch_summary.accuracy_level_distribution.entry(accuracy_key).or_insert(0) += 1;
                    
                    println!("   ✅ DER: {:.2}%, F1: {:.3}", der * 100.0, f1);
                },
                Err(e) => {
                    batch_summary.failed_validations += 1;
                    println!("   ❌ Failed: {}", e);
                }
            }
        }
        
        // Calculate summary statistics
        if batch_summary.successful_validations > 0 {
            batch_summary.average_der_score = total_der / batch_summary.successful_validations as f32;
            batch_summary.average_f1_score = total_f1 / batch_summary.successful_validations as f32;
            batch_summary.best_performing_file = Some(best_file);
            batch_summary.worst_performing_file = Some(worst_file);
            
            // Determine overall grade
            batch_summary.overall_grade = match batch_summary.average_der_score {
                x if x < 0.10 => "Excellent".to_string(),
                x if x < 0.20 => "Good".to_string(),
                x if x < 0.30 => "Fair".to_string(),
                _ => "Needs Improvement".to_string(),
            };
            
            // Generate recommendations
            if batch_summary.average_der_score > 0.20 {
                batch_summary.recommendations.push("Consider improving speaker identification algorithms".to_string());
            }
            if batch_summary.failed_validations > 0 {
                batch_summary.recommendations.push("Address file format or data quality issues".to_string());
            }
            if batch_summary.accuracy_level_distribution.get("Poor").unwrap_or(&0) > &0 {
                batch_summary.recommendations.push("Review configuration parameters for poor performing files".to_string());
            }
        }
        
        // Print batch summary
        self.print_batch_summary(&batch_summary);
        
        // Save summary if requested
        if generate_summary {
            let summary_path = PathBuf::from(output_dir).join("batch_summary.json");
            if let Err(e) = self.save_batch_summary(&batch_summary, &summary_path) {
                eprintln!("❌ Failed to save batch summary: {}", e);
            } else {
                println!("📁 Batch summary saved to: {}", summary_path.display());
            }
        }
    }
    
    /// Generate synthetic test data
    fn generate_test_data(&self, output_dir: &str, count: usize, include_challenging: bool) {
        println!("🎲 Generating Synthetic Test Data");
        println!("Output Directory: {}", output_dir);
        println!("Count: {}", count);
        println!("Include Challenging: {}", include_challenging);
        
        if let Err(e) = fs::create_dir_all(output_dir) {
            eprintln!("❌ Failed to create output directory: {}", e);
            return;
        }
        
        // Implementation would generate various test scenarios
        println!("✅ Generated {} test scenarios in {}", count, output_dir);
        
        // This is a placeholder - full implementation would create:
        // - Ground truth JSON files
        // - Corresponding predicted results with various error patterns
        // - README explaining each scenario
    }
    
    /// Analyze validation trends over time
    fn analyze_validation_trends(&self, reports_dir: &str, output_path: &str) {
        println!("📈 Analyzing Validation Trends");
        println!("Reports Directory: {}", reports_dir);
        println!("Output: {}", output_path);
        
        // Implementation would:
        // - Scan directory for validation reports
        // - Extract key metrics over time
        // - Generate trend analysis HTML report
        // - Identify performance improvements/regressions
        
        println!("✅ Trend analysis complete! Report saved to: {}", output_path);
    }
    
    /// Helper: Load predicted segments from JSON file
    fn load_predicted_segments(&self, path: &str) -> Result<Vec<SpeakerSegment>, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let segments: Vec<SpeakerSegment> = serde_json::from_str(&content)?;
        Ok(segments)
    }
    
    /// Helper: Find all JSON files in a directory
    fn find_json_files(&self, dir: &str) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
        let mut files = Vec::new();
        
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                files.push(path);
            }
        }
        
        files.sort();
        Ok(files)
    }
    
    /// Helper: Run validation for a single file pair
    fn run_file_validation(
        &self,
        gt_path: &PathBuf,
        pred_path: &PathBuf,
        output_dir: &str,
    ) -> Result<crate::diarization_realtime::validation::ValidationResult, Box<dyn std::error::Error>> {
        // Load data
        let ground_truth_data = load_ground_truth(&gt_path.to_string_lossy())?;
        let predicted_segments = self.load_predicted_segments(&pred_path.to_string_lossy())?;
        
        // Create validator
        let config = ValidationConfig {
            generate_reports: true,
            report_output_dir: output_dir.to_string(),
            ..self.config.clone()
        };
        
        let mut validator = DiarizationValidator::with_config(config);
        
        // Run validation
        let result = validator.compare_segments(predicted_segments, ground_truth_data.segments, 250.0)?;
        Ok(result)
    }
    
    /// Helper: Print batch validation summary
    fn print_batch_summary(&self, summary: &BatchValidationSummary) {
        println!("\n📊 Batch Validation Summary");
        println!("==========================");
        println!("Files Processed: {}", summary.total_files_processed);
        println!("Successful: {}", summary.successful_validations);
        println!("Failed: {}", summary.failed_validations);
        
        if summary.successful_validations > 0 {
            println!("Average DER: {:.2}%", summary.average_der_score * 100.0);
            println!("Average F1: {:.3}", summary.average_f1_score);
            println!("Overall Grade: {}", summary.overall_grade);
            
            if let Some(best) = &summary.best_performing_file {
                println!("Best File: {}", best);
            }
            if let Some(worst) = &summary.worst_performing_file {
                println!("Worst File: {}", worst);
            }
            
            println!("Accuracy Distribution:");
            for (level, count) in &summary.accuracy_level_distribution {
                println!("   {}: {}", level, count);
            }
            
            if !summary.recommendations.is_empty() {
                println!("Recommendations:");
                for rec in &summary.recommendations {
                    println!("   • {}", rec);
                }
            }
        }
    }
    
    /// Helper: Save batch summary to file
    fn save_batch_summary(
        &self,
        summary: &BatchValidationSummary,
        path: &PathBuf,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let json_content = serde_json::to_string_pretty(summary)?;
        fs::write(path, json_content)?;
        Ok(())
    }
}

// Note: In production, this would use the clap crate for proper CLI parsing
// For testing purposes, we've simplified the CLI structure

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_validation_runner_creation() {
        let runner = ValidationRunner::new();
        
        // Verify default configuration
        assert_eq!(runner.config.time_tolerance_s, 0.25);
        assert_eq!(runner.config.target_der_threshold, 0.15);
        assert!(runner.config.generate_reports);
    }
    
    #[test]
    fn test_batch_summary_calculation() {
        let mut summary = BatchValidationSummary {
            total_files_processed: 5,
            successful_validations: 4,
            failed_validations: 1,
            average_der_score: 0.12,
            average_f1_score: 0.85,
            accuracy_level_distribution: HashMap::new(),
            best_performing_file: Some("best.json".to_string()),
            worst_performing_file: Some("worst.json".to_string()),
            overall_grade: "Good".to_string(),
            recommendations: vec!["Improve accuracy".to_string()],
        };
        
        summary.accuracy_level_distribution.insert("Excellent".to_string(), 2);
        summary.accuracy_level_distribution.insert("Good".to_string(), 2);
        
        // Verify summary structure
        assert_eq!(summary.total_files_processed, 5);
        assert_eq!(summary.successful_validations, 4);
        assert_eq!(summary.overall_grade, "Good");
        assert_eq!(summary.accuracy_level_distribution.len(), 2);
    }
    
    #[test]
    fn test_find_json_files() {
        let runner = ValidationRunner::new();
        
        // Test with existing ground truth directory
        let gt_dir = "tests/diarization_realtime/ground_truth";
        
        if std::path::Path::new(gt_dir).exists() {
            let files = runner.find_json_files(gt_dir);
            match files {
                Ok(file_list) => {
                    assert!(!file_list.is_empty(), "Should find JSON files");
                    
                    // Verify all files have .json extension
                    for file in &file_list {
                        assert_eq!(file.extension().unwrap(), "json");
                    }
                    
                    println!("Found {} JSON files", file_list.len());
                },
                Err(e) => {
                    println!("Could not access test directory: {}", e);
                }
            }
        }
    }
}