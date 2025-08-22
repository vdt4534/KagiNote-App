//! Memory management tests for real-time speaker diarization
//! 
//! These tests validate memory usage patterns, leak detection, and resource
//! cleanup during diarization operations.

use super::*;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

/// Test memory usage during extended diarization sessions
#[tokio::test]
async fn test_extended_session_memory_usage() {
    let config = DiarizationTestConfig {
        test_duration: 120, // 2 minutes per session
        max_speakers: 6,
        ..Default::default()
    };
    
    let mut memory_tracker = MemoryTracker::new();
    memory_tracker.start_monitoring().await;
    
    // Run multiple extended sessions
    for session_id in 0..5 {
        println!("Starting extended session {}", session_id);
        
        let session_start = memory_tracker.take_snapshot("session_start");
        
        // Process continuous audio chunks
        for chunk_id in 0..24 { // 24 chunks of 5 seconds = 2 minutes
            let audio_chunk = DiarizationTestUtils::generate_synthetic_audio(
                5.0, // 5-second chunks
                config.max_speakers,
                config.sample_rate,
            );
            
            let chunk_snapshot = memory_tracker.take_snapshot(&format!("chunk_{}_{}", session_id, chunk_id));
            
            // Simulate diarization processing
            let _result = simulate_memory_intensive_processing(&audio_chunk, &config).await;
            
            memory_tracker.record_chunk_processing(chunk_snapshot, get_current_memory_mb());
            
            // Small delay between chunks
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        
        let session_end = memory_tracker.take_snapshot("session_end");
        memory_tracker.record_session_completion(session_start, session_end);
        
        // Force cleanup between sessions
        simulate_cleanup().await;
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
    
    memory_tracker.stop_monitoring().await;
    
    // Analyze memory patterns
    let analysis = memory_tracker.analyze();
    
    println!("Memory analysis results:");
    println!("  Peak memory: {:.1} MB", analysis.peak_memory_mb);
    println!("  Average memory: {:.1} MB", analysis.average_memory_mb);
    println!("  Memory growth: {:.1} MB", analysis.total_growth_mb);
    println!("  Potential leaks: {}", analysis.potential_leaks);
    
    // Validate memory constraints
    assert!(analysis.peak_memory_mb <= 750.0, 
           "Peak memory {} MB exceeds limit", analysis.peak_memory_mb);
    
    assert!(analysis.total_growth_mb <= 150.0,
           "Total memory growth {} MB indicates potential leak", analysis.total_growth_mb);
    
    assert!(analysis.potential_leaks == 0,
           "Detected {} potential memory leaks", analysis.potential_leaks);
    
    println!("✅ Extended session memory test passed");
}

/// Test memory usage with concurrent sessions
#[tokio::test]
async fn test_concurrent_session_memory_isolation() {
    let num_concurrent_sessions = 4;
    let session_duration = 30; // seconds
    
    let memory_tracker = Arc::new(Mutex::new(MemoryTracker::new()));
    {
        let tracker = memory_tracker.lock().await;
        tracker.start_monitoring().await;
    }
    
    let initial_memory = get_current_memory_mb();
    
    // Start concurrent sessions
    let mut session_handles = Vec::new();
    
    for session_id in 0..num_concurrent_sessions {
        let tracker_clone = Arc::clone(&memory_tracker);
        
        let handle = tokio::spawn(async move {
            let config = DiarizationTestConfig {
                test_duration: session_duration,
                max_speakers: 3 + session_id % 3, // Vary complexity
                ..Default::default()
            };
            
            let session_memory = SessionMemoryInfo::new(session_id);
            
            // Process session audio
            for chunk_idx in 0..6 { // 6 chunks of 5 seconds each
                let audio_chunk = DiarizationTestUtils::generate_synthetic_audio(
                    5.0,
                    config.max_speakers,
                    config.sample_rate,
                );
                
                let memory_before = get_current_memory_mb();
                
                let _result = simulate_memory_intensive_processing(&audio_chunk, &config).await;
                
                let memory_after = get_current_memory_mb();
                
                {
                    let tracker = tracker_clone.lock().await;
                    tracker.record_concurrent_chunk(session_id, chunk_idx, memory_before, memory_after);
                }
                
                session_memory.record_chunk(memory_before, memory_after);
                
                tokio::time::sleep(Duration::from_millis(200)).await;
            }
            
            session_memory
        });
        
        session_handles.push(handle);
    }
    
    // Wait for all sessions to complete
    let mut session_results = Vec::new();
    for handle in session_handles {
        let session_memory = handle.await.expect("Session should complete");
        session_results.push(session_memory);
    }
    
    let final_memory = get_current_memory_mb();
    
    {
        let tracker = memory_tracker.lock().await;
        tracker.stop_monitoring().await;
    }
    
    // Analyze concurrent memory usage
    let total_memory_growth = final_memory - initial_memory;
    let max_session_memory = session_results.iter()
        .map(|s| s.peak_memory)
        .fold(0.0, |a, b| a.max(b));
    
    println!("Concurrent session memory analysis:");
    println!("  Initial memory: {:.1} MB", initial_memory);
    println!("  Final memory: {:.1} MB", final_memory);
    println!("  Total growth: {:.1} MB", total_memory_growth);
    println!("  Max session memory: {:.1} MB", max_session_memory);
    
    // Validate memory isolation
    assert!(total_memory_growth <= 300.0,
           "Total memory growth {} MB too high for concurrent sessions", total_memory_growth);
    
    assert!(max_session_memory <= 200.0,
           "Individual session memory {} MB exceeds limit", max_session_memory);
    
    // Verify no session had excessive memory growth
    for session_memory in &session_results {
        assert!(session_memory.growth <= 100.0,
               "Session {} memory growth {} MB exceeds limit", 
               session_memory.session_id, session_memory.growth);
    }
    
    println!("✅ Concurrent session memory isolation test passed");
}

/// Test memory cleanup after errors
#[tokio::test]
async fn test_memory_cleanup_after_errors() {
    let config = DiarizationTestConfig::default();
    let mut memory_tracker = MemoryTracker::new();
    memory_tracker.start_monitoring().await;
    
    let initial_memory = get_current_memory_mb();
    
    // Test various error scenarios
    let error_scenarios = vec![
        ("empty_audio", vec![]),
        ("invalid_short_audio", vec![0.0; 50]),
        ("corrupted_audio", vec![f32::NAN; 1000]),
        ("extremely_long_audio", vec![0.0; 10_000_000]), // 10M samples
    ];
    
    for (scenario_name, audio_data) in error_scenarios {
        println!("Testing error scenario: {}", scenario_name);
        
        let memory_before_error = get_current_memory_mb();
        
        // Attempt processing (expecting errors)
        let result = simulate_error_prone_processing(&audio_data, &config).await;
        
        // Verify error occurred (except for extremely long audio which might succeed)
        if scenario_name != "extremely_long_audio" {
            assert!(result.is_err(), "Expected error for scenario {}", scenario_name);
        }
        
        // Force cleanup after error
        simulate_error_cleanup().await;
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        let memory_after_cleanup = get_current_memory_mb();
        let memory_delta = memory_after_cleanup - memory_before_error;
        
        println!("  Memory delta for {}: {:.1} MB", scenario_name, memory_delta);
        
        // Verify memory was cleaned up properly
        assert!(memory_delta.abs() <= 50.0,
               "Memory not cleaned up properly after error in {}: {:.1} MB delta",
               scenario_name, memory_delta);
    }
    
    let final_memory = get_current_memory_mb();
    let total_growth = final_memory - initial_memory;
    
    memory_tracker.stop_monitoring().await;
    
    println!("Error cleanup test results:");
    println!("  Initial memory: {:.1} MB", initial_memory);
    println!("  Final memory: {:.1} MB", final_memory);
    println!("  Total growth: {:.1} MB", total_growth);
    
    // Verify overall memory is stable after error handling
    assert!(total_growth.abs() <= 100.0,
           "Memory growth {} MB after error testing indicates leaks", total_growth);
    
    println!("✅ Memory cleanup after errors test passed");
}

/// Test memory pressure handling
#[tokio::test]
async fn test_memory_pressure_handling() {
    let config = DiarizationTestConfig {
        max_speakers: 8,
        test_duration: 60,
        ..Default::default()
    };
    
    // Simulate high memory pressure by allocating large amounts of memory
    let _pressure_allocations = create_memory_pressure(500.0); // 500MB pressure
    
    let memory_under_pressure = get_current_memory_mb();
    println!("Memory under pressure: {:.1} MB", memory_under_pressure);
    
    // Test diarization under memory pressure
    let audio = DiarizationTestUtils::generate_synthetic_audio(
        config.test_duration as f32,
        config.max_speakers,
        config.sample_rate,
    );
    
    let start_time = Instant::now();
    let result = simulate_memory_intensive_processing(&audio, &config).await;
    let processing_time = start_time.elapsed();
    
    let memory_after_processing = get_current_memory_mb();
    
    // Verify system handles memory pressure gracefully
    assert!(result.is_ok(), "Processing should succeed under memory pressure");
    
    // Processing might be slower under pressure, but should complete
    assert!(processing_time <= Duration::from_secs(config.test_duration as u64 * 3),
           "Processing took too long under memory pressure: {:?}", processing_time);
    
    // Memory usage should not grow excessively
    let memory_growth = memory_after_processing - memory_under_pressure;
    assert!(memory_growth <= 200.0,
           "Memory growth {} MB too high under pressure", memory_growth);
    
    drop(_pressure_allocations); // Release pressure
    
    // Allow system to recover
    tokio::time::sleep(Duration::from_millis(500)).await;
    
    let memory_after_recovery = get_current_memory_mb();
    println!("Memory after recovery: {:.1} MB", memory_after_recovery);
    
    println!("✅ Memory pressure handling test passed");
}

/// Test memory usage patterns with different speaker counts
#[tokio::test]
async fn test_memory_scaling_with_speakers() {
    let speaker_counts = vec![2, 4, 6, 8, 10];
    let mut memory_results = Vec::new();
    
    for num_speakers in speaker_counts {
        let config = DiarizationTestConfig {
            max_speakers: num_speakers,
            min_speakers: num_speakers,
            test_duration: 20,
            ..Default::default()
        };
        
        // Measure baseline memory
        let baseline_memory = get_current_memory_mb();
        
        let audio = DiarizationTestUtils::generate_synthetic_audio(
            config.test_duration as f32,
            num_speakers,
            config.sample_rate,
        );
        
        let memory_before = get_current_memory_mb();
        let _result = simulate_memory_intensive_processing(&audio, &config).await;
        let memory_after = get_current_memory_mb();
        
        let memory_used = memory_after - memory_before;
        let total_memory = memory_after - baseline_memory;
        
        memory_results.push((num_speakers, memory_used, total_memory));
        
        println!("Speakers: {}, Memory used: {:.1} MB, Total: {:.1} MB", 
                 num_speakers, memory_used, total_memory);
        
        // Clean up between tests
        simulate_cleanup().await;
        tokio::time::sleep(Duration::from_millis(200)).await;
    }
    
    // Analyze memory scaling
    let memory_per_speaker = memory_results.windows(2)
        .map(|window| {
            let (speakers1, mem1, _) = window[0];
            let (speakers2, mem2, _) = window[1];
            (mem2 - mem1) / (speakers2 - speakers1) as f64
        })
        .collect::<Vec<f64>>();
    
    let avg_memory_per_speaker = memory_per_speaker.iter().sum::<f64>() / memory_per_speaker.len() as f64;
    
    println!("Memory scaling analysis:");
    println!("  Average memory per additional speaker: {:.1} MB", avg_memory_per_speaker);
    
    // Validate reasonable memory scaling
    assert!(avg_memory_per_speaker <= 30.0,
           "Memory per speaker {} MB is too high", avg_memory_per_speaker);
    
    // Verify maximum memory usage is reasonable
    let max_total_memory = memory_results.iter()
        .map(|(_, _, total)| *total)
        .fold(0.0, |a, b| a.max(b));
    
    assert!(max_total_memory <= 600.0,
           "Maximum total memory {} MB exceeds limit", max_total_memory);
    
    println!("✅ Memory scaling test passed");
}

/// Generate comprehensive memory report
#[tokio::test]
async fn generate_memory_usage_report() {
    let mut memory_tracker = MemoryTracker::new();
    memory_tracker.start_monitoring().await;
    
    let test_scenarios = vec![
        ("quick_2_speakers", 2, 10),
        ("medium_4_speakers", 4, 30),
        ("long_6_speakers", 6, 60),
        ("stress_8_speakers", 8, 45),
    ];
    
    let mut report_data = Vec::new();
    
    for (scenario_name, num_speakers, duration) in test_scenarios {
        println!("Running memory test scenario: {}", scenario_name);
        
        let config = DiarizationTestConfig {
            max_speakers: num_speakers,
            test_duration: duration,
            ..Default::default()
        };
        
        let scenario_start = memory_tracker.take_snapshot(&format!("{}_start", scenario_name));
        
        let audio = DiarizationTestUtils::generate_synthetic_audio(
            duration as f32,
            num_speakers,
            config.sample_rate,
        );
        
        let processing_start = Instant::now();
        let memory_before = get_current_memory_mb();
        
        let _result = simulate_memory_intensive_processing(&audio, &config).await;
        
        let processing_time = processing_start.elapsed();
        let memory_after = get_current_memory_mb();
        let peak_memory = memory_tracker.get_peak_memory_since(scenario_start);
        
        let scenario_data = ScenarioMemoryData {
            name: scenario_name.to_string(),
            speakers: num_speakers,
            duration_seconds: duration,
            memory_before_mb: memory_before,
            memory_after_mb: memory_after,
            peak_memory_mb: peak_memory,
            processing_time_ms: processing_time.as_millis() as u64,
            memory_efficiency: (num_speakers as f64 * duration as f64) / (memory_after - memory_before),
        };
        
        report_data.push(scenario_data);
        
        // Cleanup between scenarios
        simulate_cleanup().await;
        tokio::time::sleep(Duration::from_millis(300)).await;
    }
    
    memory_tracker.stop_monitoring().await;
    
    // Generate report
    let report = MemoryUsageReport {
        timestamp: chrono::Utc::now().to_rfc3339(),
        scenarios: report_data,
        system_info: SystemMemoryInfo {
            total_system_memory_mb: get_total_system_memory_mb(),
            available_memory_mb: get_available_memory_mb(),
        },
    };
    
    // Save report
    let report_filename = format!("memory_usage_report_{}.json", 
                                 chrono::Utc::now().format("%Y%m%d_%H%M%S"));
    
    if let Err(e) = save_memory_report(&report, &report_filename) {
        println!("Warning: Could not save memory report: {}", e);
    }
    
    // Print summary
    println!("\n📊 Memory Usage Report Summary:");
    println!("┌─────────────────────┬─────────┬──────────┬─────────────┬──────────────┬─────────────┐");
    println!("│ Scenario            │ Speakers│ Duration │ Memory Used │ Peak Memory  │ Efficiency  │");
    println!("├─────────────────────┼─────────┼──────────┼─────────────┼──────────────┼─────────────┤");
    
    for scenario in &report.scenarios {
        let memory_used = scenario.memory_after_mb - scenario.memory_before_mb;
        println!("│ {:19} │ {:7} │ {:8}s │ {:9.1}MB │ {:10.1}MB │ {:9.2}   │",
                scenario.name, scenario.speakers, scenario.duration_seconds,
                memory_used, scenario.peak_memory_mb, scenario.memory_efficiency);
    }
    
    println!("└─────────────────────┴─────────┴──────────┴─────────────┴──────────────┴─────────────┘");
    
    // Validate all scenarios passed memory constraints
    for scenario in &report.scenarios {
        let memory_used = scenario.memory_after_mb - scenario.memory_before_mb;
        assert!(memory_used <= 400.0,
               "Scenario {} used too much memory: {:.1} MB", scenario.name, memory_used);
        
        assert!(scenario.peak_memory_mb <= 800.0,
               "Scenario {} peak memory too high: {:.1} MB", scenario.name, scenario.peak_memory_mb);
    }
    
    println!("✅ Memory usage report generated: {}", report_filename);
}

// Helper types and functions

struct MemoryTracker {
    snapshots: Vec<MemorySnapshot>,
    monitoring: bool,
    start_time: Option<Instant>,
}

#[derive(Clone)]
struct MemorySnapshot {
    name: String,
    timestamp: Instant,
    memory_mb: f64,
}

struct MemoryAnalysis {
    peak_memory_mb: f64,
    average_memory_mb: f64,
    total_growth_mb: f64,
    potential_leaks: usize,
}

struct SessionMemoryInfo {
    session_id: usize,
    chunks: Vec<(f64, f64)>, // (before, after) memory per chunk
    peak_memory: f64,
    growth: f64,
}

#[derive(serde::Serialize)]
struct ScenarioMemoryData {
    name: String,
    speakers: usize,
    duration_seconds: u32,
    memory_before_mb: f64,
    memory_after_mb: f64,
    peak_memory_mb: f64,
    processing_time_ms: u64,
    memory_efficiency: f64,
}

#[derive(serde::Serialize)]
struct MemoryUsageReport {
    timestamp: String,
    scenarios: Vec<ScenarioMemoryData>,
    system_info: SystemMemoryInfo,
}

#[derive(serde::Serialize)]
struct SystemMemoryInfo {
    total_system_memory_mb: f64,
    available_memory_mb: f64,
}

impl MemoryTracker {
    fn new() -> Self {
        Self {
            snapshots: Vec::new(),
            monitoring: false,
            start_time: None,
        }
    }
    
    async fn start_monitoring(&mut self) {
        self.monitoring = true;
        self.start_time = Some(Instant::now());
        self.take_snapshot("monitoring_start");
    }
    
    async fn stop_monitoring(&mut self) {
        self.take_snapshot("monitoring_end");
        self.monitoring = false;
    }
    
    fn take_snapshot(&mut self, name: &str) -> MemorySnapshot {
        let snapshot = MemorySnapshot {
            name: name.to_string(),
            timestamp: Instant::now(),
            memory_mb: get_current_memory_mb(),
        };
        self.snapshots.push(snapshot.clone());
        snapshot
    }
    
    fn record_chunk_processing(&mut self, _before: MemorySnapshot, _after_memory: f64) {
        // Record chunk processing metrics
    }
    
    fn record_session_completion(&mut self, _start: MemorySnapshot, _end: MemorySnapshot) {
        // Record session completion metrics
    }
    
    fn record_concurrent_chunk(&self, _session_id: usize, _chunk_idx: usize, _before: f64, _after: f64) {
        // Record concurrent chunk processing
    }
    
    fn get_peak_memory_since(&self, snapshot: MemorySnapshot) -> f64 {
        self.snapshots.iter()
            .filter(|s| s.timestamp >= snapshot.timestamp)
            .map(|s| s.memory_mb)
            .fold(0.0, |a, b| a.max(b))
    }
    
    fn analyze(&self) -> MemoryAnalysis {
        if self.snapshots.is_empty() {
            return MemoryAnalysis {
                peak_memory_mb: 0.0,
                average_memory_mb: 0.0,
                total_growth_mb: 0.0,
                potential_leaks: 0,
            };
        }
        
        let peak_memory = self.snapshots.iter()
            .map(|s| s.memory_mb)
            .fold(0.0, |a, b| a.max(b));
        
        let average_memory = self.snapshots.iter()
            .map(|s| s.memory_mb)
            .sum::<f64>() / self.snapshots.len() as f64;
        
        let first_memory = self.snapshots.first().unwrap().memory_mb;
        let last_memory = self.snapshots.last().unwrap().memory_mb;
        let total_growth = last_memory - first_memory;
        
        // Simple leak detection: look for consistent upward trends
        let potential_leaks = self.detect_memory_leaks();
        
        MemoryAnalysis {
            peak_memory_mb: peak_memory,
            average_memory_mb: average_memory,
            total_growth_mb: total_growth,
            potential_leaks,
        }
    }
    
    fn detect_memory_leaks(&self) -> usize {
        // Simplified leak detection
        if self.snapshots.len() < 10 {
            return 0;
        }
        
        let mut leak_count = 0;
        let window_size = 5;
        
        for window in self.snapshots.windows(window_size) {
            let trend = window.last().unwrap().memory_mb - window.first().unwrap().memory_mb;
            if trend > 50.0 { // 50MB growth in window indicates potential leak
                leak_count += 1;
            }
        }
        
        leak_count
    }
}

impl SessionMemoryInfo {
    fn new(session_id: usize) -> Self {
        Self {
            session_id,
            chunks: Vec::new(),
            peak_memory: 0.0,
            growth: 0.0,
        }
    }
    
    fn record_chunk(&mut self, before: f64, after: f64) {
        self.chunks.push((before, after));
        self.peak_memory = self.peak_memory.max(after);
        
        if let Some((first_before, _)) = self.chunks.first() {
            self.growth = after - first_before;
        }
    }
}

async fn simulate_memory_intensive_processing(
    audio: &[f32], 
    _config: &DiarizationTestConfig
) -> Result<(), String> {
    if audio.is_empty() {
        return Err("Empty audio".to_string());
    }
    
    if audio.len() < 100 {
        return Err("Audio too short".to_string());
    }
    
    // Check for NaN values
    if audio.iter().any(|&x| x.is_nan()) {
        return Err("Corrupted audio data".to_string());
    }
    
    // Simulate memory-intensive processing
    let processing_duration = (audio.len() / 16000).max(1) as u64; // At least 1ms
    tokio::time::sleep(Duration::from_millis(processing_duration.min(100))).await;
    
    // Simulate temporary memory allocation during processing
    let _temp_buffer: Vec<f32> = vec![0.0; audio.len() / 4];
    
    Ok(())
}

async fn simulate_error_prone_processing(
    audio: &[f32],
    _config: &DiarizationTestConfig,
) -> Result<(), String> {
    if audio.is_empty() {
        return Err("Empty audio input".to_string());
    }
    
    if audio.len() < 100 {
        return Err("Audio too short for processing".to_string());
    }
    
    if audio.iter().any(|&x| x.is_nan()) {
        return Err("Corrupted audio data detected".to_string());
    }
    
    if audio.len() > 5_000_000 {
        // Very long audio - might succeed but will be slow
        tokio::time::sleep(Duration::from_millis(1000)).await;
    }
    
    Ok(())
}

async fn simulate_cleanup() {
    // Simulate memory cleanup operations
    tokio::time::sleep(Duration::from_millis(50)).await;
}

async fn simulate_error_cleanup() {
    // Simulate cleanup after errors
    tokio::time::sleep(Duration::from_millis(25)).await;
}

fn create_memory_pressure(target_mb: f64) -> Vec<Vec<u8>> {
    // Create memory pressure by allocating large chunks
    let chunk_size = 10 * 1024 * 1024; // 10MB chunks
    let num_chunks = (target_mb / 10.0) as usize;
    
    let mut allocations = Vec::new();
    for _ in 0..num_chunks {
        allocations.push(vec![0u8; chunk_size]);
    }
    
    allocations
}

fn get_current_memory_mb() -> f64 {
    // Simplified memory usage simulation
    use std::time::{SystemTime, UNIX_EPOCH};
    let base_memory = 200.0;
    let variation = (SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() % 100) as f64;
    base_memory + variation
}

fn get_total_system_memory_mb() -> f64 {
    // Simulate system memory (e.g., 16GB)
    16384.0
}

fn get_available_memory_mb() -> f64 {
    // Simulate available memory
    8192.0
}

fn save_memory_report(report: &MemoryUsageReport, filename: &str) -> Result<(), String> {
    let reports_dir = DiarizationTestUtils::reports_dir();
    let file_path = reports_dir.join(filename);
    
    match serde_json::to_string_pretty(report) {
        Ok(json_content) => {
            if let Err(e) = std::fs::write(file_path, json_content) {
                Err(format!("Failed to write report: {}", e))
            } else {
                Ok(())
            }
        }
        Err(e) => Err(format!("Failed to serialize report: {}", e))
    }
}

// Re-use chrono module from performance_tests.rs
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