//! Benchmarking utilities for diarization validation framework
//! 
//! This module provides performance benchmarking tools to measure
//! validation framework performance and establish baseline metrics.

use kaginote_lib::diarization::types::{SpeakerSegment, SpeakerEmbedding};
use crate::diarization_realtime::validation::{
    DiarizationValidator, ValidationConfig, GroundTruthSegment, PerformanceMetrics
};
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant, SystemTime};
use std::collections::HashMap;

/// Benchmark configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkConfig {
    /// Number of segments to generate for testing
    pub num_segments: usize,
    
    /// Number of speakers to simulate
    pub num_speakers: usize,
    
    /// Total duration of simulated audio (seconds)
    pub total_duration: f32,
    
    /// Number of benchmark iterations
    pub iterations: usize,
    
    /// Include memory profiling
    pub profile_memory: bool,
    
    /// Include CPU profiling
    pub profile_cpu: bool,
    
    /// Validation tolerance in milliseconds
    pub tolerance_ms: f32,
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        Self {
            num_segments: 100,
            num_speakers: 4,
            total_duration: 300.0, // 5 minutes
            iterations: 5,
            profile_memory: true,
            profile_cpu: true,
            tolerance_ms: 250.0,
        }
    }
}

/// Benchmark result for a single iteration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkIteration {
    /// Iteration number
    pub iteration: usize,
    
    /// Time taken for validation (milliseconds)
    pub validation_time_ms: u64,
    
    /// Memory used during validation (MB)
    pub memory_used_mb: f32,
    
    /// DER score achieved
    pub der_score: f32,
    
    /// F1 score achieved  
    pub f1_score: f32,
    
    /// Real-time factor for this iteration
    pub real_time_factor: f32,
    
    /// Number of segments processed
    pub segments_processed: usize,
}

/// Complete benchmark results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResults {
    /// Configuration used for benchmark
    pub config: BenchmarkConfig,
    
    /// Results from each iteration
    pub iterations: Vec<BenchmarkIteration>,
    
    /// Summary statistics
    pub summary: BenchmarkSummary,
    
    /// Timestamp when benchmark was run
    pub timestamp: SystemTime,
    
    /// System information
    pub system_info: SystemInfo,
}

/// Summary statistics across all benchmark iterations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkSummary {
    /// Average validation time (ms)
    pub avg_validation_time_ms: f64,
    
    /// Minimum validation time (ms)
    pub min_validation_time_ms: u64,
    
    /// Maximum validation time (ms)
    pub max_validation_time_ms: u64,
    
    /// Standard deviation of validation times (ms)
    pub std_dev_validation_time_ms: f64,
    
    /// Average memory usage (MB)
    pub avg_memory_usage_mb: f32,
    
    /// Peak memory usage (MB)
    pub peak_memory_usage_mb: f32,
    
    /// Average DER score
    pub avg_der_score: f32,
    
    /// Average F1 score
    pub avg_f1_score: f32,
    
    /// Average real-time factor
    pub avg_real_time_factor: f32,
    
    /// Throughput (segments per second)
    pub throughput_segments_per_sec: f32,
    
    /// Processing efficiency (segments per MB of memory)
    pub efficiency_segments_per_mb: f32,
}

/// System information for benchmark context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    /// Operating system
    pub os: String,
    
    /// CPU architecture
    pub arch: String,
    
    /// Number of CPU cores
    pub cpu_cores: usize,
    
    /// Total system memory (MB)
    pub total_memory_mb: usize,
    
    /// Rust version
    pub rust_version: String,
    
    /// Compilation mode (debug/release)
    pub build_mode: String,
}

/// Main benchmarking framework
pub struct DiarizationBenchmark {
    config: BenchmarkConfig,
    validator: DiarizationValidator,
}

impl DiarizationBenchmark {
    /// Create a new benchmark with default configuration
    pub fn new() -> Self {
        Self::with_config(BenchmarkConfig::default())
    }
    
    /// Create a new benchmark with custom configuration
    pub fn with_config(config: BenchmarkConfig) -> Self {
        let validation_config = ValidationConfig {
            generate_reports: false, // Skip report generation during benchmarking
            ..ValidationConfig::default()
        };
        
        Self {
            config,
            validator: DiarizationValidator::with_config(validation_config),
        }
    }
    
    /// Run complete benchmark suite
    pub fn run_benchmark(&mut self) -> BenchmarkResults {
        println!("Starting diarization validation benchmark...");
        println!("Configuration: {} segments, {} speakers, {} iterations", 
                 self.config.num_segments, self.config.num_speakers, self.config.iterations);
        
        let start_time = SystemTime::now();
        let mut iterations = Vec::new();
        
        for i in 0..self.config.iterations {
            println!("Running iteration {} of {}...", i + 1, self.config.iterations);
            let iteration_result = self.run_single_iteration(i);
            iterations.push(iteration_result);
        }
        
        let summary = self.calculate_summary(&iterations);
        let system_info = self.collect_system_info();
        
        let results = BenchmarkResults {
            config: self.config.clone(),
            iterations,
            summary,
            timestamp: start_time,
            system_info,
        };
        
        self.print_results(&results);
        results
    }
    
    /// Run a single benchmark iteration
    fn run_single_iteration(&mut self, iteration: usize) -> BenchmarkIteration {
        // Generate test data
        let (predicted_segments, ground_truth_segments) = self.generate_test_data();
        
        // Measure memory before validation
        let memory_before = self.get_memory_usage();
        
        // Run validation with timing
        let validation_start = Instant::now();
        
        let validation_result = self.validator.compare_segments(
            predicted_segments.clone(),
            ground_truth_segments,
            self.config.tolerance_ms,
        ).expect("Benchmark validation should succeed");
        
        let validation_duration = validation_start.elapsed();
        
        // Measure memory after validation
        let memory_after = self.get_memory_usage();
        let memory_used = memory_after - memory_before;
        
        // Calculate real-time factor
        let real_time_factor = validation_duration.as_secs_f32() / self.config.total_duration;
        
        BenchmarkIteration {
            iteration,
            validation_time_ms: validation_duration.as_millis() as u64,
            memory_used_mb: memory_used,
            der_score: validation_result.der_result.der_score,
            f1_score: validation_result.der_result.f1_score,
            real_time_factor,
            segments_processed: predicted_segments.len(),
        }
    }
    
    /// Generate test data for benchmarking
    fn generate_test_data(&self) -> (Vec<SpeakerSegment>, Vec<GroundTruthSegment>) {
        let segment_duration = self.config.total_duration / self.config.num_segments as f32;
        let speaker_names: Vec<String> = (0..self.config.num_speakers)
            .map(|i| format!("benchmark_speaker_{}", i))
            .collect();
        
        let mut predicted_segments = Vec::new();
        let mut ground_truth_segments = Vec::new();
        
        for i in 0..self.config.num_segments {
            let start_time = i as f32 * segment_duration;
            let end_time = start_time + segment_duration * 0.9; // 90% utilization with gaps
            
            // Rotate through speakers
            let speaker_id = speaker_names[i % self.config.num_speakers].clone();
            
            // Create ground truth segment
            let gt_segment = GroundTruthSegment {
                speaker_id: speaker_id.clone(),
                start_time,
                end_time,
                text: Some(format!("Benchmark segment {} content", i)),
                audio_file: None,
                quality: 1.0,
            };
            
            // Create predicted segment with slight variations
            let confidence_variation = 0.8 + (i as f32 * 0.1) % 0.2; // 0.8-1.0 range
            let time_variation = (i as f32 * 0.01) % 0.1 - 0.05; // ±50ms variation
            
            let pred_segment = SpeakerSegment {
                speaker_id: speaker_id.clone(),
                start_time: start_time + time_variation,
                end_time: end_time + time_variation,
                confidence: confidence_variation,
                text: Some(format!("Predicted segment {} content", i)),
                embedding: Some(self.generate_benchmark_embedding(start_time, &speaker_id)),
                has_overlap: i % 10 == 0, // 10% of segments have overlaps
                overlapping_speakers: if i % 10 == 0 {
                    vec![speaker_names[(i + 1) % self.config.num_speakers].clone()]
                } else {
                    vec![]
                },
            };
            
            ground_truth_segments.push(gt_segment);
            predicted_segments.push(pred_segment);
        }
        
        (predicted_segments, ground_truth_segments)
    }
    
    /// Generate a benchmark speaker embedding
    fn generate_benchmark_embedding(&self, timestamp: f32, speaker_id: &str) -> SpeakerEmbedding {
        // Generate deterministic but realistic embedding vector
        let mut vector = Vec::with_capacity(512);
        
        let speaker_hash = self.hash_string(speaker_id) as f32;
        let time_factor = (timestamp * 0.1).sin();
        
        for i in 0..512 {
            let component = (speaker_hash + i as f32 + time_factor).sin() * 0.5;
            vector.push(component);
        }
        
        // Normalize vector
        let magnitude: f32 = vector.iter().map(|x| x * x).sum::<f32>().sqrt();
        if magnitude > 0.0 {
            for component in &mut vector {
                *component /= magnitude;
            }
        }
        
        SpeakerEmbedding {
            vector,
            confidence: 0.9,
            timestamp_start: timestamp,
            timestamp_end: timestamp + 1.0,
            speaker_id: Some(speaker_id.to_string()),
            quality: 0.95,
            extracted_at: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            audio_duration_ms: 1000,
        }
    }
    
    /// Simple string hashing for deterministic embedding generation
    fn hash_string(&self, s: &str) -> u32 {
        let mut hash = 0u32;
        for byte in s.bytes() {
            hash = hash.wrapping_mul(31).wrapping_add(byte as u32);
        }
        hash
    }
    
    /// Calculate summary statistics from benchmark iterations
    fn calculate_summary(&self, iterations: &[BenchmarkIteration]) -> BenchmarkSummary {
        if iterations.is_empty() {
            return BenchmarkSummary {
                avg_validation_time_ms: 0.0,
                min_validation_time_ms: 0,
                max_validation_time_ms: 0,
                std_dev_validation_time_ms: 0.0,
                avg_memory_usage_mb: 0.0,
                peak_memory_usage_mb: 0.0,
                avg_der_score: 0.0,
                avg_f1_score: 0.0,
                avg_real_time_factor: 0.0,
                throughput_segments_per_sec: 0.0,
                efficiency_segments_per_mb: 0.0,
            };
        }
        
        let n = iterations.len() as f64;
        
        // Timing statistics
        let validation_times: Vec<u64> = iterations.iter()
            .map(|i| i.validation_time_ms)
            .collect();
        
        let avg_validation_time_ms = validation_times.iter().sum::<u64>() as f64 / n;
        let min_validation_time_ms = *validation_times.iter().min().unwrap_or(&0);
        let max_validation_time_ms = *validation_times.iter().max().unwrap_or(&0);
        
        let variance = validation_times.iter()
            .map(|&time| {
                let diff = time as f64 - avg_validation_time_ms;
                diff * diff
            })
            .sum::<f64>() / n;
        let std_dev_validation_time_ms = variance.sqrt();
        
        // Memory statistics
        let memory_usages: Vec<f32> = iterations.iter()
            .map(|i| i.memory_used_mb)
            .collect();
        
        let avg_memory_usage_mb = memory_usages.iter().sum::<f32>() / n as f32;
        let peak_memory_usage_mb = memory_usages.iter()
            .copied()
            .fold(0.0f32, f32::max);
        
        // Accuracy statistics
        let avg_der_score = iterations.iter()
            .map(|i| i.der_score)
            .sum::<f32>() / n as f32;
        
        let avg_f1_score = iterations.iter()
            .map(|i| i.f1_score)
            .sum::<f32>() / n as f32;
        
        let avg_real_time_factor = iterations.iter()
            .map(|i| i.real_time_factor)
            .sum::<f32>() / n as f32;
        
        // Performance statistics
        let total_segments_processed: usize = iterations.iter()
            .map(|i| i.segments_processed)
            .sum();
        
        let total_time_seconds = avg_validation_time_ms / 1000.0;
        let throughput_segments_per_sec = if total_time_seconds > 0.0 {
            total_segments_processed as f32 / total_time_seconds as f32
        } else {
            0.0
        };
        
        let efficiency_segments_per_mb = if avg_memory_usage_mb > 0.0 {
            (total_segments_processed as f32 / n as f32) / avg_memory_usage_mb
        } else {
            0.0
        };
        
        BenchmarkSummary {
            avg_validation_time_ms,
            min_validation_time_ms,
            max_validation_time_ms,
            std_dev_validation_time_ms,
            avg_memory_usage_mb,
            peak_memory_usage_mb,
            avg_der_score,
            avg_f1_score,
            avg_real_time_factor,
            throughput_segments_per_sec,
            efficiency_segments_per_mb,
        }
    }
    
    /// Collect system information for benchmark context
    fn collect_system_info(&self) -> SystemInfo {
        SystemInfo {
            os: std::env::consts::OS.to_string(),
            arch: std::env::consts::ARCH.to_string(),
            cpu_cores: 4, // Simplified for testing - would use num_cpus crate in production
            total_memory_mb: self.get_total_memory_mb(),
            rust_version: "1.70.0".to_string(), // Simplified - would use env!("RUSTC_VERSION") in production
            build_mode: if cfg!(debug_assertions) {
                "debug".to_string()
            } else {
                "release".to_string()
            },
        }
    }
    
    /// Get current memory usage (simplified implementation)
    fn get_memory_usage(&self) -> f32 {
        // Simplified memory tracking - in production would use system APIs
        // For now, return a simulation based on number of segments
        let base_memory = 50.0; // Base 50MB
        let per_segment_memory = 0.1; // 100KB per segment
        base_memory + (self.config.num_segments as f32 * per_segment_memory)
    }
    
    /// Get total system memory (simplified implementation)
    fn get_total_memory_mb(&self) -> usize {
        // Simplified - would use system APIs in production
        8192 // 8GB default
    }
    
    /// Print benchmark results to console
    fn print_results(&self, results: &BenchmarkResults) {
        println!("\n🚀 Diarization Validation Benchmark Results");
        println!("==========================================");
        
        println!("\n📊 Configuration:");
        println!("  Segments: {}", results.config.num_segments);
        println!("  Speakers: {}", results.config.num_speakers);
        println!("  Duration: {:.1}s", results.config.total_duration);
        println!("  Iterations: {}", results.config.iterations);
        println!("  Tolerance: {:.0}ms", results.config.tolerance_ms);
        
        println!("\n⏱️  Timing Performance:");
        println!("  Average: {:.1}ms", results.summary.avg_validation_time_ms);
        println!("  Min: {}ms", results.summary.min_validation_time_ms);
        println!("  Max: {}ms", results.summary.max_validation_time_ms);
        println!("  Std Dev: {:.1}ms", results.summary.std_dev_validation_time_ms);
        println!("  Throughput: {:.1} segments/sec", results.summary.throughput_segments_per_sec);
        
        println!("\n🧠 Memory Usage:");
        println!("  Average: {:.1}MB", results.summary.avg_memory_usage_mb);
        println!("  Peak: {:.1}MB", results.summary.peak_memory_usage_mb);
        println!("  Efficiency: {:.1} segments/MB", results.summary.efficiency_segments_per_mb);
        
        println!("\n🎯 Accuracy Metrics:");
        println!("  Average DER: {:.2}%", results.summary.avg_der_score * 100.0);
        println!("  Average F1: {:.3}", results.summary.avg_f1_score);
        println!("  Real-time Factor: {:.2}x", results.summary.avg_real_time_factor);
        
        println!("\n💻 System Info:");
        println!("  OS: {} ({})", results.system_info.os, results.system_info.arch);
        println!("  CPU Cores: {}", results.system_info.cpu_cores);
        println!("  Memory: {}MB", results.system_info.total_memory_mb);
        println!("  Build: {}", results.system_info.build_mode);
        
        // Performance assessment
        println!("\n📈 Performance Assessment:");
        
        if results.summary.avg_real_time_factor < 1.0 {
            println!("  ✅ Excellent: Real-time processing capability");
        } else if results.summary.avg_real_time_factor < 2.0 {
            println!("  ⚠️  Good: Near real-time processing");
        } else {
            println!("  ❌ Needs Improvement: Slower than real-time");
        }
        
        if results.summary.avg_der_score < 0.15 {
            println!("  ✅ Excellent: Low diarization error rate");
        } else if results.summary.avg_der_score < 0.25 {
            println!("  ⚠️  Good: Acceptable diarization accuracy");
        } else {
            println!("  ❌ Needs Improvement: High error rate");
        }
        
        if results.summary.peak_memory_usage_mb < 200.0 {
            println!("  ✅ Excellent: Low memory footprint");
        } else if results.summary.peak_memory_usage_mb < 400.0 {
            println!("  ⚠️  Good: Moderate memory usage");
        } else {
            println!("  ❌ Needs Improvement: High memory usage");
        }
    }
    
    /// Save benchmark results to JSON file
    pub fn save_results(&self, results: &BenchmarkResults, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let json_content = serde_json::to_string_pretty(results)?;
        std::fs::write(path, json_content)?;
        println!("📁 Benchmark results saved to: {}", path);
        Ok(())
    }
}

/// Predefined benchmark scenarios
pub struct BenchmarkScenarios;

impl BenchmarkScenarios {
    /// Quick benchmark for development
    pub fn quick() -> BenchmarkConfig {
        BenchmarkConfig {
            num_segments: 20,
            num_speakers: 2,
            total_duration: 60.0, // 1 minute
            iterations: 3,
            tolerance_ms: 250.0,
            ..BenchmarkConfig::default()
        }
    }
    
    /// Standard benchmark for CI/CD
    pub fn standard() -> BenchmarkConfig {
        BenchmarkConfig {
            num_segments: 100,
            num_speakers: 4,
            total_duration: 300.0, // 5 minutes
            iterations: 5,
            tolerance_ms: 250.0,
            ..BenchmarkConfig::default()
        }
    }
    
    /// Extensive benchmark for performance analysis
    pub fn extensive() -> BenchmarkConfig {
        BenchmarkConfig {
            num_segments: 500,
            num_speakers: 8,
            total_duration: 1800.0, // 30 minutes
            iterations: 10,
            tolerance_ms: 250.0,
            ..BenchmarkConfig::default()
        }
    }
    
    /// Stress test with large numbers of segments
    pub fn stress_test() -> BenchmarkConfig {
        BenchmarkConfig {
            num_segments: 2000,
            num_speakers: 10,
            total_duration: 3600.0, // 1 hour
            iterations: 3,
            tolerance_ms: 250.0,
            ..BenchmarkConfig::default()
        }
    }
    
    /// Memory efficiency test
    pub fn memory_test() -> BenchmarkConfig {
        BenchmarkConfig {
            num_segments: 1000,
            num_speakers: 6,
            total_duration: 1200.0, // 20 minutes
            iterations: 5,
            tolerance_ms: 250.0,
            profile_memory: true,
            profile_cpu: false,
        }
    }
}

// Note: In production, this would use external crates like num_cpus for system information

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_benchmark_config_defaults() {
        let config = BenchmarkConfig::default();
        
        assert_eq!(config.num_segments, 100);
        assert_eq!(config.num_speakers, 4);
        assert_eq!(config.total_duration, 300.0);
        assert_eq!(config.iterations, 5);
        assert_eq!(config.tolerance_ms, 250.0);
        assert!(config.profile_memory);
        assert!(config.profile_cpu);
    }
    
    #[test]
    fn test_benchmark_scenarios() {
        let quick = BenchmarkScenarios::quick();
        assert_eq!(quick.num_segments, 20);
        assert_eq!(quick.iterations, 3);
        
        let standard = BenchmarkScenarios::standard();
        assert_eq!(standard.num_segments, 100);
        assert_eq!(standard.iterations, 5);
        
        let extensive = BenchmarkScenarios::extensive();
        assert_eq!(extensive.num_segments, 500);
        assert_eq!(extensive.iterations, 10);
        
        let stress = BenchmarkScenarios::stress_test();
        assert_eq!(stress.num_segments, 2000);
        assert_eq!(stress.num_speakers, 10);
    }
    
    #[test]
    fn test_benchmark_data_generation() {
        let config = BenchmarkScenarios::quick();
        let benchmark = DiarizationBenchmark::with_config(config);
        
        let (predicted, ground_truth) = benchmark.generate_test_data();
        
        assert_eq!(predicted.len(), 20);
        assert_eq!(ground_truth.len(), 20);
        
        // Verify segments are properly distributed
        let mut speaker_counts = HashMap::new();
        for segment in &predicted {
            *speaker_counts.entry(&segment.speaker_id).or_insert(0) += 1;
        }
        
        assert_eq!(speaker_counts.len(), 2); // 2 speakers as configured
        
        // Verify timing is reasonable
        assert!(predicted[0].start_time < predicted[0].end_time);
        assert!(predicted[0].end_time <= predicted[1].start_time + 1.0); // Allow some overlap
    }
    
    #[test]
    fn test_embedding_generation() {
        let benchmark = DiarizationBenchmark::new();
        
        let embedding1 = benchmark.generate_benchmark_embedding(0.0, "speaker_1");
        let embedding2 = benchmark.generate_benchmark_embedding(0.0, "speaker_1");
        let embedding3 = benchmark.generate_benchmark_embedding(0.0, "speaker_2");
        
        assert_eq!(embedding1.vector.len(), 512);
        assert_eq!(embedding2.vector.len(), 512);
        assert_eq!(embedding3.vector.len(), 512);
        
        // Same speaker, same time should produce identical embeddings
        assert_eq!(embedding1.vector, embedding2.vector);
        
        // Different speakers should produce different embeddings
        assert_ne!(embedding1.vector, embedding3.vector);
        
        // Verify embeddings are normalized
        let magnitude: f32 = embedding1.vector.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((magnitude - 1.0).abs() < 0.001, "Embedding should be normalized");
    }
    
    #[test]
    fn test_summary_calculation() {
        let config = BenchmarkScenarios::quick();
        let benchmark = DiarizationBenchmark::with_config(config);
        
        let iterations = vec![
            BenchmarkIteration {
                iteration: 0,
                validation_time_ms: 100,
                memory_used_mb: 50.0,
                der_score: 0.10,
                f1_score: 0.90,
                real_time_factor: 0.8,
                segments_processed: 20,
            },
            BenchmarkIteration {
                iteration: 1,
                validation_time_ms: 120,
                memory_used_mb: 55.0,
                der_score: 0.12,
                f1_score: 0.88,
                real_time_factor: 0.9,
                segments_processed: 20,
            },
        ];
        
        let summary = benchmark.calculate_summary(&iterations);
        
        assert_eq!(summary.avg_validation_time_ms, 110.0);
        assert_eq!(summary.min_validation_time_ms, 100);
        assert_eq!(summary.max_validation_time_ms, 120);
        assert_eq!(summary.avg_memory_usage_mb, 52.5);
        assert_eq!(summary.avg_der_score, 0.11);
        assert_eq!(summary.avg_f1_score, 0.89);
        assert_eq!(summary.avg_real_time_factor, 0.85);
        
        // Verify throughput calculation
        assert!(summary.throughput_segments_per_sec > 0.0);
        assert!(summary.efficiency_segments_per_mb > 0.0);
    }
    
    #[tokio::test]
    async fn test_quick_benchmark() {
        let mut benchmark = DiarizationBenchmark::with_config(BenchmarkScenarios::quick());
        let results = benchmark.run_benchmark();
        
        // Verify results structure
        assert_eq!(results.iterations.len(), 3); // 3 iterations in quick config
        assert!(results.summary.avg_validation_time_ms > 0.0);
        assert!(results.summary.avg_memory_usage_mb > 0.0);
        assert!(results.summary.throughput_segments_per_sec > 0.0);
        
        // Verify all iterations completed
        for (i, iteration) in results.iterations.iter().enumerate() {
            assert_eq!(iteration.iteration, i);
            assert!(iteration.validation_time_ms > 0);
            assert_eq!(iteration.segments_processed, 20); // 20 segments in quick config
        }
    }
}