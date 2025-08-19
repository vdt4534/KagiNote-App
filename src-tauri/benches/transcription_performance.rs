//! Transcription Performance Benchmarks
//! 
//! This is a placeholder benchmark file.
//! The actual benchmarks are in tests/benchmarks/performance_benchmarks.rs
//! This file exists to satisfy Cargo.toml requirements.

use criterion::{criterion_group, criterion_main, Criterion};

fn placeholder_transcription_benchmark(c: &mut Criterion) {
    c.bench_function("transcription_placeholder", |b| b.iter(|| {
        // Placeholder benchmark - will be replaced by actual implementation
        "transcription"
    }));
}

criterion_group!(benches, placeholder_transcription_benchmark);
criterion_main!(benches);