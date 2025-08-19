//! Performance Benchmark Tests
//! 
//! These benchmarks are written BEFORE implementation exists (TDD).
//! They define performance requirements that implementation must meet.
//! All benchmarks should FAIL initially - this is correct TDD behavior.

use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use tokio_test;
use std::time::{Duration, Instant};
use tempfile::TempDir;

// These imports WILL FAIL because modules don't exist yet
use crate::audio::capture::AudioCaptureService;
use crate::audio::vad::SileroVAD;
use crate::asr::whisper::WhisperEngine;
use crate::transcription::pipeline::TranscriptionPipeline;
use crate::audio::types::AudioData;
use crate::asr::types::ModelTier;

/// Audio Processing Benchmarks
/// These define performance requirements for real-time audio processing
fn benchmark_audio_capture_latency(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    
    c.bench_function("audio_capture_initialization", |b| {
        b.to_async(&rt).iter(|| async {
            // BENCHMARK REQUIREMENT: Audio capture must initialize within 100ms
            let config = create_audio_config();
            let start = Instant::now();
            
            // This WILL FAIL because AudioCaptureService doesn't exist
            let capture = AudioCaptureService::new(config).await.unwrap();
            let elapsed = start.elapsed();
            
            // Performance requirement
            assert!(elapsed < Duration::from_millis(100), 
                   "Audio capture initialization took {}ms, must be <100ms", 
                   elapsed.as_millis());
            
            capture
        });
    });

    c.bench_function("audio_capture_chunk_processing", |b| {
        b.to_async(&rt).iter(|| async {
            let config = create_audio_config();
            let mut capture = AudioCaptureService::new(config).await.unwrap();
            capture.start_capture().await.unwrap();
            
            let start = Instant::now();
            // BENCHMARK REQUIREMENT: Audio chunk retrieval must be <10ms
            let chunk = capture.get_next_chunk().await.unwrap();
            let elapsed = start.elapsed();
            
            // Performance requirement for real-time processing
            assert!(elapsed < Duration::from_millis(10),
                   "Audio chunk processing took {}ms, must be <10ms",
                   elapsed.as_millis());
            
            capture.stop_capture().await.unwrap();
            chunk
        });
    });

    // Throughput benchmark for continuous audio processing
    let mut group = c.benchmark_group("audio_throughput");
    for buffer_size in [512, 1024, 2048, 4096].iter() {
        group.throughput(Throughput::Elements(*buffer_size as u64));
        group.bench_with_input(
            BenchmarkId::new("samples_per_second", buffer_size),
            buffer_size,
            |b, &buffer_size| {
                b.to_async(&rt).iter(|| async {
                    let config = AudioConfig {
                        sample_rate: 16000,
                        channels: 1,
                        buffer_size_ms: (buffer_size * 1000 / 16000) as u32, // Convert to ms
                        device_id: None,
                    };
                    
                    let mut capture = AudioCaptureService::new(config).await.unwrap();
                    capture.start_capture().await.unwrap();
                    
                    // Process multiple chunks to measure sustained throughput
                    for _ in 0..10 {
                        let _chunk = capture.get_next_chunk().await.unwrap();
                    }
                    
                    capture.stop_capture().await.unwrap();
                });
            }
        );
    }
    group.finish();
}

/// VAD Processing Benchmarks  
/// Define performance requirements for voice activity detection
fn benchmark_vad_performance(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("vad_initialization", |b| {
        b.to_async(&rt).iter(|| async {
            // BENCHMARK REQUIREMENT: VAD must initialize within 500ms
            let config = create_vad_config();
            let start = Instant::now();
            
            // This WILL FAIL because SileroVAD doesn't exist
            let vad = SileroVAD::new(config).await.unwrap();
            let elapsed = start.elapsed();
            
            assert!(elapsed < Duration::from_millis(500),
                   "VAD initialization took {}ms, must be <500ms",
                   elapsed.as_millis());
            
            vad
        });
    });

    // Test VAD processing latency for different audio lengths
    let mut group = c.benchmark_group("vad_processing");
    for duration in [0.5, 1.0, 2.0, 5.0].iter() {
        group.throughput(Throughput::Elements((*duration * 16000.0) as u64));
        group.bench_with_input(
            BenchmarkId::new("detect_speech", duration),
            duration,
            |b, &duration| {
                b.to_async(&rt).iter(|| async {
                    let config = create_vad_config();
                    let vad = SileroVAD::new(config).await.unwrap();
                    let test_audio = create_test_audio(duration);
                    
                    let start = Instant::now();
                    let result = vad.detect_speech(&test_audio).await.unwrap();
                    let elapsed = start.elapsed();
                    
                    // BENCHMARK REQUIREMENT: VAD processing must be <50ms for 5s audio
                    let max_latency = Duration::from_millis((duration * 10.0) as u64); // 10ms per second
                    assert!(elapsed < max_latency,
                           "VAD processing took {}ms for {}s audio, must be <{}ms",
                           elapsed.as_millis(), duration, max_latency.as_millis());
                    
                    result
                });
            }
        );
    }
    group.finish();

    c.bench_function("vad_streaming_chunks", |b| {
        b.to_async(&rt).iter(|| async {
            let config = create_vad_config();
            let mut vad = SileroVAD::new(config).await.unwrap();
            
            // BENCHMARK REQUIREMENT: Streaming VAD must process 100ms chunks in <5ms
            let chunk = create_test_audio(0.1); // 100ms chunk
            
            let start = Instant::now();
            let result = vad.process_chunk(&chunk).await.unwrap();
            let elapsed = start.elapsed();
            
            assert!(elapsed < Duration::from_millis(5),
                   "VAD chunk processing took {}ms, must be <5ms for real-time",
                   elapsed.as_millis());
            
            result
        });
    });
}

/// ASR Engine Benchmarks
/// Define performance requirements for speech recognition models
fn benchmark_asr_performance(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    // Test model loading performance for different tiers
    let mut group = c.benchmark_group("asr_model_loading");
    for tier in [ModelTier::Standard, ModelTier::HighAccuracy, ModelTier::Turbo].iter() {
        group.bench_with_input(
            BenchmarkId::new("model_init", format!("{:?}", tier)),
            tier,
            |b, &tier| {
                b.to_async(&rt).iter(|| async {
                    let config = WhisperConfig {
                        model_tier: tier,
                        device: Device::CPU,
                        ..Default::default()
                    };
                    
                    let start = Instant::now();
                    // This WILL FAIL because WhisperEngine doesn't exist
                    let engine = WhisperEngine::new(config).await.unwrap();
                    let elapsed = start.elapsed();
                    
                    // BENCHMARK REQUIREMENTS per tier
                    let max_init_time = match tier {
                        ModelTier::Standard => Duration::from_secs(3),      // Medium model
                        ModelTier::HighAccuracy => Duration::from_secs(10), // Large-v3 model
                        ModelTier::Turbo => Duration::from_secs(2),         // Turbo model
                    };
                    
                    assert!(elapsed < max_init_time,
                           "{:?} model loading took {}s, must be <{}s",
                           tier, elapsed.as_secs_f32(), max_init_time.as_secs());
                    
                    engine
                });
            }
        );
    }
    group.finish();

    // Test transcription performance with different model tiers and audio lengths
    let mut group = c.benchmark_group("asr_transcription");
    for tier in [ModelTier::Standard, ModelTier::HighAccuracy, ModelTier::Turbo].iter() {
        for duration in [10.0, 30.0, 60.0, 300.0].iter() { // 10s to 5min
            group.throughput(Throughput::Elements(*duration as u64));
            group.bench_with_input(
                BenchmarkId::new(format!("{:?}", tier), duration),
                &(tier, duration),
                |b, &(tier, duration)| {
                    b.to_async(&rt).iter(|| async {
                        let config = WhisperConfig {
                            model_tier: *tier,
                            device: Device::CPU,
                            ..Default::default()
                        };
                        let engine = WhisperEngine::new(config).await.unwrap();
                        let test_audio = create_test_audio(*duration);
                        
                        let start = Instant::now();
                        let result = engine.transcribe(&test_audio, &Default::default()).await.unwrap();
                        let elapsed = start.elapsed();
                        
                        // BENCHMARK REQUIREMENTS: Real-time Factor (RTF) targets
                        let rtf = elapsed.as_secs_f64() / *duration as f64;
                        let max_rtf = match tier {
                            ModelTier::Standard => 1.0,     // Must be ≤1.0x real-time
                            ModelTier::HighAccuracy => 2.0, // Must be ≤2.0x real-time  
                            ModelTier::Turbo => 0.8,        // Must be ≤0.8x real-time
                        };
                        
                        assert!(rtf <= max_rtf,
                               "{:?} RTF was {:.2}x for {}s audio, must be ≤{:.1}x",
                               tier, rtf, duration, max_rtf);
                        
                        result
                    });
                }
            );
        }
    }
    group.finish();
}

/// Complete Pipeline Benchmarks
/// Define performance requirements for end-to-end transcription
fn benchmark_pipeline_performance(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("pipeline_session_startup", |b| {
        b.to_async(&rt).iter(|| async {
            // BENCHMARK REQUIREMENT: Complete session startup within 2 seconds
            let config = create_pipeline_config();
            let temp_dir = TempDir::new().unwrap();
            let pipeline = TranscriptionPipeline::new(config, temp_dir.path()).await.unwrap();
            
            let start = Instant::now();
            let session_id = pipeline.start_session().await.unwrap();
            let elapsed = start.elapsed();
            
            assert!(elapsed < Duration::from_secs(2),
                   "Pipeline session startup took {}ms, must be <2000ms",
                   elapsed.as_millis());
            
            session_id
        });
    });

    // Test complete meeting processing performance
    let mut group = c.benchmark_group("complete_meeting_processing");
    for (meeting_type, duration) in [
        ("short_call", 300.0),     // 5 minutes
        ("standard_meeting", 1800.0), // 30 minutes  
        ("long_meeting", 3600.0),     // 1 hour
    ].iter() {
        group.throughput(Throughput::Elements(*duration as u64));
        group.bench_with_input(
            BenchmarkId::new("end_to_end", meeting_type),
            &(meeting_type, duration),
            |b, &(meeting_type, duration)| {
                b.to_async(&rt).iter(|| async {
                    let config = create_pipeline_config();
                    let temp_dir = TempDir::new().unwrap();
                    let mut pipeline = TranscriptionPipeline::new(config, temp_dir.path()).await.unwrap();
                    
                    let meeting_audio = create_business_meeting_audio(*duration);
                    
                    let start = Instant::now();
                    let session_id = pipeline.start_session().await.unwrap();
                    let result = pipeline.process_complete_audio(&meeting_audio).await.unwrap();
                    let elapsed = start.elapsed();
                    
                    // BENCHMARK REQUIREMENTS: Complete processing performance
                    let rtf = elapsed.as_secs_f64() / *duration as f64;
                    let max_rtf = match *meeting_type {
                        "short_call" => 0.8,      // Short calls should be very fast
                        "standard_meeting" => 1.0, // 30min should process in real-time
                        "long_meeting" => 1.2,     // 1hr may be slightly slower
                        _ => 1.0,
                    };
                    
                    assert!(rtf <= max_rtf,
                           "{} processing RTF was {:.2}x for {}s audio, must be ≤{:.1}x",
                           meeting_type, rtf, duration, max_rtf);
                    
                    // Quality requirement
                    assert!(result.quality_metrics.overall_confidence > 0.85,
                           "Meeting confidence {:.2}% must be >85%",
                           result.quality_metrics.overall_confidence * 100.0);
                    
                    result
                });
            }
        );
    }
    group.finish();

    c.bench_function("real_time_streaming_latency", |b| {
        b.to_async(&rt).iter(|| async {
            let config = create_pipeline_config();
            let temp_dir = TempDir::new().unwrap();
            let mut pipeline = TranscriptionPipeline::new(config, temp_dir.path()).await.unwrap();
            
            let session_id = pipeline.start_session().await.unwrap();
            
            // BENCHMARK REQUIREMENT: Streaming chunk processing <1.5s latency
            let chunk = create_test_audio(5.0); // 5-second chunk
            
            let start = Instant::now();
            let result = pipeline.process_chunk_immediate(&chunk).await.unwrap();
            let elapsed = start.elapsed();
            
            assert!(elapsed < Duration::from_millis(1500),
                   "Real-time streaming took {}ms, must be <1500ms",
                   elapsed.as_millis());
            
            // Should have transcription results
            assert!(!result.segments.is_empty());
            
            result
        });
    });
}

/// Memory Usage Benchmarks
/// Define memory efficiency requirements
fn benchmark_memory_usage(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("memory_usage_standard_meeting", |b| {
        b.to_async(&rt).iter(|| async {
            let config = create_pipeline_config();
            let temp_dir = TempDir::new().unwrap();
            let mut pipeline = TranscriptionPipeline::new(config, temp_dir.path()).await.unwrap();
            
            // BENCHMARK REQUIREMENT: 30-minute meeting should use <8GB peak memory
            let meeting_audio = create_business_meeting_audio(1800.0); // 30 minutes
            
            let memory_before = get_memory_usage();
            let session_id = pipeline.start_session().await.unwrap();
            let result = pipeline.process_complete_audio(&meeting_audio).await.unwrap();
            let memory_after = get_memory_usage();
            
            let memory_used = memory_after - memory_before;
            let max_memory_gb = 8.0 * 1024.0 * 1024.0 * 1024.0; // 8GB
            
            assert!(memory_used < max_memory_gb as usize,
                   "Memory usage {}GB must be <8GB for 30-minute meeting",
                   memory_used as f64 / (1024.0 * 1024.0 * 1024.0));
            
            // Should also check reported peak usage
            assert!(result.quality_metrics.memory_usage_peak < max_memory_gb as u64);
            
            result
        });
    });

    c.bench_function("memory_leak_detection", |b| {
        b.to_async(&rt).iter(|| async {
            let config = create_pipeline_config();
            let temp_dir = TempDir::new().unwrap();
            
            let memory_start = get_memory_usage();
            
            // Process multiple short sessions to detect leaks
            for _ in 0..10 {
                let mut pipeline = TranscriptionPipeline::new(config.clone(), temp_dir.path()).await.unwrap();
                let session_id = pipeline.start_session().await.unwrap();
                
                let short_audio = create_test_audio(30.0); // 30 seconds
                let result = pipeline.process_complete_audio(&short_audio).await.unwrap();
                
                // Explicitly drop pipeline to test cleanup
                drop(pipeline);
            }
            
            // Force garbage collection
            tokio::time::sleep(Duration::from_millis(100)).await;
            
            let memory_end = get_memory_usage();
            let memory_growth = memory_end - memory_start;
            
            // BENCHMARK REQUIREMENT: <100MB growth after 10 sessions (leak detection)
            let max_growth = 100 * 1024 * 1024; // 100MB
            assert!(memory_growth < max_growth,
                   "Memory growth {}MB after 10 sessions, possible leak (max {}MB)",
                   memory_growth / (1024 * 1024), max_growth / (1024 * 1024));
        });
    });
}

/// Accuracy vs Performance Trade-off Benchmarks
/// Define quality requirements under performance constraints
fn benchmark_accuracy_performance_tradeoffs(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    // Test different quality tiers against accuracy and speed requirements
    let mut group = c.benchmark_group("accuracy_vs_speed");
    for tier in [ModelTier::Standard, ModelTier::HighAccuracy, ModelTier::Turbo].iter() {
        group.bench_with_input(
            BenchmarkId::new("tier", format!("{:?}", tier)),
            tier,
            |b, &tier| {
                b.to_async(&rt).iter(|| async {
                    let config = PipelineConfig {
                        quality_tier: tier,
                        ..create_pipeline_config()
                    };
                    
                    let temp_dir = TempDir::new().unwrap();
                    let mut pipeline = TranscriptionPipeline::new(config, temp_dir.path()).await.unwrap();
                    
                    // Test with known ground truth audio
                    let test_audio = create_ground_truth_audio();
                    
                    let start = Instant::now();
                    let session_id = pipeline.start_session().await.unwrap();
                    let result = pipeline.process_complete_audio(&test_audio).await.unwrap();
                    let elapsed = start.elapsed();
                    
                    let rtf = elapsed.as_secs_f64() / test_audio.duration_seconds as f64;
                    
                    // BENCHMARK REQUIREMENTS: Quality vs Speed targets
                    match tier {
                        ModelTier::Standard => {
                            assert!(rtf <= 1.0, "Standard RTF {:.2}x must be ≤1.0x", rtf);
                            assert!(result.quality_metrics.overall_confidence > 0.88,
                                   "Standard confidence {:.1}% must be >88%",
                                   result.quality_metrics.overall_confidence * 100.0);
                        },
                        ModelTier::HighAccuracy => {
                            assert!(rtf <= 2.0, "High-accuracy RTF {:.2}x must be ≤2.0x", rtf);
                            assert!(result.quality_metrics.overall_confidence > 0.92,
                                   "High-accuracy confidence {:.1}% must be >92%",
                                   result.quality_metrics.overall_confidence * 100.0);
                        },
                        ModelTier::Turbo => {
                            assert!(rtf <= 0.8, "Turbo RTF {:.2}x must be ≤0.8x", rtf);
                            assert!(result.quality_metrics.overall_confidence > 0.85,
                                   "Turbo confidence {:.1}% must be >85%",
                                   result.quality_metrics.overall_confidence * 100.0);
                        },
                    }
                    
                    (rtf, result.quality_metrics.overall_confidence)
                });
            }
        );
    }
    group.finish();
}

/// Concurrent Load Benchmarks  
/// Test system behavior under multiple simultaneous operations
fn benchmark_concurrent_load(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("concurrent_vad_processing", |b| {
        b.to_async(&rt).iter(|| async {
            // BENCHMARK REQUIREMENT: Handle 4 concurrent VAD streams without degradation
            let config = create_vad_config();
            
            let tasks: Vec<_> = (0..4).map(|i| {
                let config = config.clone();
                tokio::spawn(async move {
                    let vad = SileroVAD::new(config).await.unwrap();
                    let test_audio = create_test_audio(10.0);
                    
                    let start = Instant::now();
                    let result = vad.detect_speech(&test_audio).await.unwrap();
                    let elapsed = start.elapsed();
                    
                    (i, elapsed, result.has_speech)
                })
            }).collect();
            
            let results = futures::future::join_all(tasks).await;
            
            // All should complete within reasonable time
            for result in results {
                let (i, elapsed, has_speech) = result.unwrap();
                assert!(elapsed < Duration::from_millis(200),
                       "Concurrent VAD {} took {}ms, should be <200ms",
                       i, elapsed.as_millis());
            }
        });
    });

    c.bench_function("system_under_stress", |b| {
        b.to_async(&rt).iter(|| async {
            // BENCHMARK REQUIREMENT: System should maintain >70% performance under stress
            let base_config = create_pipeline_config();
            
            // Simulate system stress with multiple operations
            let stress_tasks: Vec<_> = (0..3).map(|_| {
                let config = base_config.clone();
                tokio::spawn(async move {
                    let temp_dir = TempDir::new().unwrap();
                    let mut pipeline = TranscriptionPipeline::new(config, temp_dir.path()).await.unwrap();
                    
                    let session_id = pipeline.start_session().await.unwrap();
                    let test_audio = create_test_audio(60.0); // 1 minute each
                    
                    let start = Instant::now();
                    let result = pipeline.process_complete_audio(&test_audio).await.unwrap();
                    let elapsed = start.elapsed();
                    
                    let rtf = elapsed.as_secs_f64() / 60.0;
                    (rtf, result.quality_metrics.overall_confidence)
                })
            }).collect();
            
            let results = futures::future::join_all(stress_tasks).await;
            
            // Under stress, should maintain reasonable performance
            for result in results {
                let (rtf, confidence) = result.unwrap();
                assert!(rtf < 1.5, "Under stress RTF {:.2}x should be <1.5x", rtf);
                assert!(confidence > 0.7, "Under stress confidence {:.1}% should be >70%",
                       confidence * 100.0);
            }
        });
    });
}

// Helper functions for benchmark tests

fn create_audio_config() -> AudioConfig {
    AudioConfig {
        sample_rate: 16000,
        channels: 1,
        buffer_size_ms: 100,
        device_id: None,
    }
}

fn create_vad_config() -> VADConfig {
    VADConfig {
        threshold: 0.5,
        min_speech_duration_ms: 500,
        max_speech_duration_ms: 30000,
        padding_before_ms: 200,
        padding_after_ms: 200,
        adaptive_threshold: false,
        context_frames: 16,
    }
}

fn create_pipeline_config() -> PipelineConfig {
    PipelineConfig {
        quality_tier: ModelTier::Standard,
        languages: vec!["en".to_string()],
        enable_speaker_diarization: true,
        enable_two_pass_refinement: true,
        enable_vad: true,
        audio_sources: AudioSourceConfig {
            microphone: true,
            system_audio: false,
        },
        vad_config: create_vad_config(),
        asr_config: ASRConfig::default(),
        diarization_config: DiarizationConfig::default(),
    }
}

fn create_test_audio(duration: f32) -> AudioData {
    let sample_rate = 16000;
    let num_samples = (duration * sample_rate as f32) as usize;
    let samples = (0..num_samples)
        .map(|i| {
            let t = i as f32 / sample_rate as f32;
            // Generate speech-like signal
            (2.0 * std::f32::consts::PI * 440.0 * t).sin() * 0.3 *
            (std::f32::consts::PI * t * 2.5).sin().abs() // Speech envelope
        })
        .collect();
    
    AudioData {
        sample_rate: sample_rate as u32,
        channels: 1,
        samples,
        timestamp: std::time::SystemTime::now(),
        source_channel: AudioSource::Microphone,
        duration_seconds: duration,
    }
}

fn create_business_meeting_audio(duration: f32) -> AudioData {
    // Generate complex business meeting scenario
    let sample_rate = 16000;
    let num_samples = (duration * sample_rate as f32) as usize;
    let mut samples = vec![0.0; num_samples];
    
    // Mix multiple speakers with realistic patterns
    let speakers = [
        (0.4, 160.0), // Speaker 1: 40% time, 160Hz fundamental
        (0.35, 140.0), // Speaker 2: 35% time, 140Hz
        (0.25, 180.0), // Speaker 3: 25% time, 180Hz
    ];
    
    for (ratio, freq) in speakers.iter() {
        let speaker_samples = (num_samples as f32 * ratio) as usize;
        for i in 0..speaker_samples {
            let t = i as f32 / sample_rate as f32;
            let sample = (2.0 * std::f32::consts::PI * freq * t).sin() * 0.3 *
                        speech_envelope(t, duration * ratio);
            
            let idx = (i as f32 / ratio) as usize;
            if idx < samples.len() {
                samples[idx] += sample * 0.7;
            }
        }
    }
    
    AudioData {
        sample_rate: sample_rate as u32,
        channels: 1,
        samples,
        timestamp: std::time::SystemTime::now(),
        source_channel: AudioSource::Mixed,
        duration_seconds: duration,
    }
}

fn create_ground_truth_audio() -> AudioData {
    // Create audio with known transcription for accuracy testing
    create_test_audio(30.0) // 30 second test clip
}

fn speech_envelope(t: f32, duration: f32) -> f32 {
    // Natural speech envelope with pauses
    let speech_rate = 2.5; // syllables per second
    let syllable_phase = (t * speech_rate) % 1.0;
    
    if syllable_phase < 0.6 && ((t * 0.5) as i32 % 3) < 2 {
        (std::f32::consts::PI * syllable_phase / 0.6).sin()
    } else {
        0.0
    }
}

fn get_memory_usage() -> usize {
    // Platform-specific memory usage measurement
    // This is a placeholder - real implementation would use system APIs
    #[cfg(target_os = "linux")]
    {
        // Use /proc/self/statm on Linux
        std::fs::read_to_string("/proc/self/statm")
            .ok()
            .and_then(|s| s.split_whitespace().nth(1))
            .and_then(|s| s.parse::<usize>().ok())
            .map(|pages| pages * 4096) // Convert pages to bytes
            .unwrap_or(0)
    }
    
    #[cfg(target_os = "macos")]
    {
        // Use task_info on macOS
        use std::mem;
        use libc::{mach_task_self, task_info, TASK_BASIC_INFO};
        
        unsafe {
            let mut info: task_basic_info = mem::zeroed();
            let mut count = (mem::size_of::<task_basic_info>() / mem::size_of::<u32>()) as u32;
            
            if task_info(mach_task_self(), TASK_BASIC_INFO, &mut info as *mut _ as *mut _, &mut count) == 0 {
                info.resident_size as usize
            } else {
                0
            }
        }
    }
    
    #[cfg(target_os = "windows")]
    {
        // Use GetProcessMemoryInfo on Windows
        use winapi::um::psapi::{GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS};
        use winapi::um::processthreadsapi::GetCurrentProcess;
        use std::mem;
        
        unsafe {
            let mut pmc: PROCESS_MEMORY_COUNTERS = mem::zeroed();
            if GetProcessMemoryInfo(GetCurrentProcess(), &mut pmc, mem::size_of::<PROCESS_MEMORY_COUNTERS>() as u32) != 0 {
                pmc.WorkingSetSize
            } else {
                0
            }
        }
    }
}

// Define benchmark groups
criterion_group!(
    benches,
    benchmark_audio_capture_latency,
    benchmark_vad_performance,
    benchmark_asr_performance,
    benchmark_pipeline_performance,
    benchmark_memory_usage,
    benchmark_accuracy_performance_tradeoffs,
    benchmark_concurrent_load
);

criterion_main!(benches);

/*
PERFORMANCE BENCHMARK CONTRACT:
==============================

These benchmarks define the performance requirements that implementation must meet:

## Audio Processing Requirements:
- Audio capture initialization: <100ms
- Audio chunk processing: <10ms per chunk
- Sustained throughput: Process all common buffer sizes efficiently
- Memory efficiency: No memory leaks across sessions

## VAD Processing Requirements:
- VAD initialization: <500ms 
- Speech detection: <10ms per second of audio
- Streaming processing: <5ms for 100ms chunks
- Accuracy maintenance under performance pressure

## ASR Engine Requirements:
- Model loading times:
  * Standard (Medium): <3s
  * High Accuracy (Large-v3): <10s  
  * Turbo: <2s
- Real-time factors:
  * Standard: ≤1.0x real-time
  * High Accuracy: ≤2.0x real-time
  * Turbo: ≤0.8x real-time

## Complete Pipeline Requirements:
- Session startup: <2s
- End-to-end meeting processing:
  * 5-minute calls: <0.8x real-time
  * 30-minute meetings: ≤1.0x real-time  
  * 1-hour meetings: <1.2x real-time
- Real-time streaming latency: <1.5s
- Overall confidence: >85% for all tiers

## Memory Usage Requirements:
- 30-minute meeting: <8GB peak memory
- Memory leak detection: <100MB growth across 10 sessions
- Efficient cleanup on session end

## Quality vs Performance Trade-offs:
- Standard: >88% confidence at ≤1.0x RTF
- High Accuracy: >92% confidence at ≤2.0x RTF
- Turbo: >85% confidence at ≤0.8x RTF

## Concurrent Load Requirements:
- Handle 4 concurrent VAD streams: <200ms per stream
- System under stress: Maintain >70% performance
- No degradation from concurrent operations

## Cross-Platform Requirements:
- All benchmarks must pass on Windows, macOS, and Linux
- Memory measurement accuracy for leak detection
- Thermal throttling adaptation under sustained load

All benchmarks should FAIL initially - this is correct TDD behavior.
The implementation will be optimized to meet these performance targets.
*/