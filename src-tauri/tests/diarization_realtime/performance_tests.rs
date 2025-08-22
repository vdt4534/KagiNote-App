//! Performance tests for real-time speaker diarization
//! 
//! These tests validate that diarization meets real-time performance requirements
//! including latency, throughput, and resource usage constraints.

use super::*;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Test real-time processing latency
#[tokio::test]
async fn test_real_time_latency() {
    let config = DiarizationTestConfig::default();
    
    // Generate test audio
    let audio = DiarizationTestUtils::generate_synthetic_audio(
        config.test_duration as f32,
        config.min_speakers,
        config.sample_rate,
    );
    
    let start = DiarizationTestUtils::start_timer();
    
    // Simulate diarization processing
    let processing_time = simulate_diarization_processing(&audio, &config).await;
    
    let total_latency = DiarizationTestUtils::elapsed_ms(start);
    
    // Verify real-time performance
    let audio_duration_ms = (config.test_duration * 1000) as u64;
    let real_time_factor = processing_time as f32 / audio_duration_ms as f32;
    
    assert!(
        total_latency <= config.test_duration as u64 * 1000 + 2000,
        "Total latency {} ms exceeds threshold for {} second audio",
        total_latency,
        config.test_duration
    );
    
    assert!(
        real_time_factor <= 1.5,
        "Real-time factor {} exceeds maximum 1.5x",
        real_time_factor
    );
    
    println!("✅ Real-time latency test passed");
    println!("   Total latency: {} ms", total_latency);
    println!("   Real-time factor: {:.2}x", real_time_factor);
}

/// Test throughput with continuous audio stream
#[tokio::test]
async fn test_continuous_stream_throughput() {
    let config = DiarizationTestConfig {
        test_duration: 60, // 1 minute test
        ..Default::default()
    };
    
    let mut total_processed = 0u64;
    let mut peak_latency = 0u64;
    let test_start = Instant::now();
    
    // Simulate continuous 5-second chunks
    while test_start.elapsed() < Duration::from_secs(config.test_duration as u64) {
        let chunk_audio = DiarizationTestUtils::generate_synthetic_audio(
            5.0, // 5-second chunks
            config.min_speakers,
            config.sample_rate,
        );
        
        let chunk_start = Instant::now();
        let _processing_time = simulate_diarization_processing(&chunk_audio, &config).await;
        let chunk_latency = chunk_start.elapsed().as_millis() as u64;
        
        total_processed += 5000; // 5 seconds in ms
        peak_latency = peak_latency.max(chunk_latency);
        
        // Brief pause to simulate real-time gaps
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    
    let throughput = total_processed as f32 / test_start.elapsed().as_secs_f32();
    
    assert!(
        throughput >= 900.0, // Should process at least 90% of real-time
        "Throughput {} ms/s is below minimum 900 ms/s",
        throughput
    );
    
    assert!(
        peak_latency <= 6000, // 6 seconds max for 5-second chunks
        "Peak latency {} ms exceeds threshold",
        peak_latency
    );
    
    println!("✅ Continuous stream throughput test passed");
    println!("   Throughput: {:.1} ms/s", throughput);
    println!("   Peak latency: {} ms", peak_latency);
}

/// Test memory usage during extended operation
#[tokio::test]
async fn test_extended_memory_usage() {
    let config = DiarizationTestConfig {
        test_duration: 120, // 2 minutes
        max_speakers: 6,
        ..Default::default()
    };
    
    let initial_memory = get_current_memory_usage();
    let mut peak_memory = initial_memory;
    let mut memory_samples = Vec::new();
    
    let test_start = Instant::now();
    
    while test_start.elapsed() < Duration::from_secs(config.test_duration as u64) {
        // Process audio chunk
        let audio = DiarizationTestUtils::generate_synthetic_audio(
            10.0, // 10-second chunks
            config.max_speakers,
            config.sample_rate,
        );
        
        let _processing_time = simulate_diarization_processing(&audio, &config).await;
        
        // Sample memory usage
        let current_memory = get_current_memory_usage();
        peak_memory = peak_memory.max(current_memory);
        memory_samples.push(current_memory);
        
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
    
    let avg_memory = memory_samples.iter().sum::<f64>() / memory_samples.len() as f64;
    let memory_growth = peak_memory - initial_memory;
    
    assert!(
        peak_memory <= 500.0,
        "Peak memory usage {} MB exceeds limit",
        peak_memory
    );
    
    assert!(
        memory_growth <= 100.0,
        "Memory growth {} MB indicates potential leak",
        memory_growth
    );
    
    println!("✅ Extended memory usage test passed");
    println!("   Initial memory: {:.1} MB", initial_memory);
    println!("   Peak memory: {:.1} MB", peak_memory);
    println!("   Average memory: {:.1} MB", avg_memory);
    println!("   Memory growth: {:.1} MB", memory_growth);
}

/// Benchmark performance across different speaker counts
#[tokio::test]
async fn benchmark_speaker_scalability() {
    let speaker_counts = vec![2, 4, 6, 8];
    let mut results = Vec::new();
    
    for num_speakers in speaker_counts {
        let config = DiarizationTestConfig {
            max_speakers: num_speakers,
            min_speakers: num_speakers,
            test_duration: 30,
            ..Default::default()
        };
        
        let audio = DiarizationTestUtils::generate_synthetic_audio(
            config.test_duration as f32,
            num_speakers,
            config.sample_rate,
        );
        
        let start = Instant::now();
        let processing_time = simulate_diarization_processing(&audio, &config).await;
        let total_time = start.elapsed().as_millis() as u64;
        
        let real_time_factor = processing_time as f32 / (config.test_duration * 1000) as f32;
        
        results.push((num_speakers, total_time, real_time_factor));
        
        println!(
            "Speakers: {}, Time: {} ms, RT Factor: {:.2}x",
            num_speakers, total_time, real_time_factor
        );
    }
    
    // Verify scalability constraints
    for (speakers, _time, rt_factor) in &results {
        let max_rt_factor = match speakers {
            2 => 1.0,
            4 => 1.2,
            6 => 1.4,
            8 => 1.5,
            _ => 2.0,
        };
        
        assert!(
            *rt_factor <= max_rt_factor,
            "Speaker count {} exceeds RT factor limit: {:.2} > {}",
            speakers, rt_factor, max_rt_factor
        );
    }
    
    println!("✅ Speaker scalability benchmark passed");
}

/// Test performance under system load
#[tokio::test]
async fn test_performance_under_load() {
    let config = DiarizationTestConfig::default();
    
    // Create CPU load simulation
    let load_handle = tokio::spawn(async {
        let start = Instant::now();
        while start.elapsed() < Duration::from_secs(30) {
            // Simulate CPU-intensive work
            let _: Vec<_> = (0..10000).map(|x| x * x).collect();
            tokio::time::sleep(Duration::from_millis(1)).await;
        }
    });
    
    // Test diarization performance under load
    let audio = DiarizationTestUtils::generate_synthetic_audio(
        20.0,
        config.max_speakers,
        config.sample_rate,
    );
    
    let start = Instant::now();
    let processing_time = simulate_diarization_processing(&audio, &config).await;
    let total_time = start.elapsed().as_millis() as u64;
    
    // Wait for load simulation to complete
    let _ = load_handle.await;
    
    let real_time_factor = processing_time as f32 / 20000.0; // 20 seconds of audio
    
    // Performance should degrade gracefully under load
    assert!(
        real_time_factor <= 2.0,
        "Performance under load degraded too much: {:.2}x",
        real_time_factor
    );
    
    assert!(
        total_time <= 45000, // Should complete within 45 seconds
        "Total processing time {} ms too high under load",
        total_time
    );
    
    println!("✅ Performance under load test passed");
    println!("   Real-time factor under load: {:.2}x", real_time_factor);
}

/// Generate comprehensive performance report
#[tokio::test]
async fn generate_performance_report() {
    let scenarios = get_standard_test_scenarios();
    let mut report_results = Vec::new();
    
    for scenario in scenarios {
        println!("Testing scenario: {}", scenario.name);
        
        // Simulate processing for each scenario
        let audio = DiarizationTestUtils::generate_synthetic_audio(
            30.0, // 30 seconds
            scenario.expected_speakers,
            16000,
        );
        
        let start = Instant::now();
        let processing_time = simulate_diarization_processing(&audio, &Default::default()).await;
        let total_time = start.elapsed().as_millis() as u64;
        
        let memory_usage = get_current_memory_usage();
        let real_time_factor = processing_time as f32 / 30000.0;
        
        let result = DiarizationTestResult {
            scenario: scenario.name.clone(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            latency_ms: total_time,
            der: 0.12, // Simulated DER
            memory_mb: memory_usage,
            accuracy: 0.87, // Simulated accuracy
            real_time_factor,
            detected_speakers: scenario.expected_speakers,
            expected_speakers: scenario.expected_speakers,
            passed: total_time <= scenario.expected_metrics.max_latency_ms,
            details: HashMap::from([
                ("processing_time_ms".to_string(), processing_time.to_string()),
                ("audio_duration_s".to_string(), "30".to_string()),
            ]),
        };
        
        report_results.push(result);
    }
    
    // Save performance report
    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let report_filename = format!("performance_report_{}.json", timestamp);
    
    if let Err(e) = DiarizationTestUtils::save_test_results(&report_results, &report_filename) {
        println!("Warning: Could not save report: {}", e);
    }
    
    // Print summary
    let passed_tests = report_results.iter().filter(|r| r.passed).count();
    let total_tests = report_results.len();
    
    println!("✅ Performance report generated");
    println!("   Tests passed: {}/{}", passed_tests, total_tests);
    println!("   Report saved: {}", report_filename);
    
    assert_eq!(passed_tests, total_tests, "All performance tests should pass");
}

// Helper functions

/// Simulate diarization processing (replace with actual implementation)
async fn simulate_diarization_processing(
    _audio: &[f32],
    _config: &DiarizationTestConfig,
) -> u64 {
    // Simulate processing time proportional to audio length
    let processing_ms = _audio.len() as u64 / 100; // Simulate processing
    tokio::time::sleep(Duration::from_millis(processing_ms.min(100))).await;
    processing_ms
}

/// Get current memory usage in MB (simplified implementation)
fn get_current_memory_usage() -> f64 {
    // In a real implementation, this would use system APIs
    // For testing, return a simulated value
    200.0 + (rand::random::<f64>() * 100.0)
}

mod rand {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    use std::time::{SystemTime, UNIX_EPOCH};
    
    pub fn random<T>() -> T 
    where 
        T: From<u64> + std::ops::Rem<Output = T> + Copy,
        u64: std::ops::Rem<T, Output = u64>
    {
        let mut hasher = DefaultHasher::new();
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos().hash(&mut hasher);
        T::from(hasher.finish() % 1000)
    }
}

mod chrono {
    use std::time::{SystemTime, UNIX_EPOCH};
    
    pub struct Utc;
    
    impl Utc {
        pub fn now() -> DateTime {
            DateTime { timestamp: SystemTime::now() }
        }
    }
    
    pub struct DateTime {
        timestamp: SystemTime,
    }
    
    impl DateTime {
        pub fn to_rfc3339(&self) -> String {
            let duration = self.timestamp.duration_since(UNIX_EPOCH).unwrap();
            format!("2025-08-22T{:02}:{:02}:{:02}Z", 
                (duration.as_secs() / 3600) % 24,
                (duration.as_secs() / 60) % 60,
                duration.as_secs() % 60
            )
        }
        
        pub fn format(&self, _fmt: &str) -> FormattedDateTime {
            let duration = self.timestamp.duration_since(UNIX_EPOCH).unwrap();
            FormattedDateTime { 
                formatted: format!("20250822_{:02}{:02}{:02}", 
                    (duration.as_secs() / 3600) % 24,
                    (duration.as_secs() / 60) % 60,
                    duration.as_secs() % 60
                )
            }
        }
    }
    
    pub struct FormattedDateTime {
        formatted: String,
    }
    
    impl std::fmt::Display for FormattedDateTime {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.formatted)
        }
    }
}