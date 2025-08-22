//! Integration tests for KagiNote
//! 
//! This file enables the test modules to access the main crate

use kaginote_lib;

// Re-export modules for tests to use
pub use kaginote_lib::*;

// Test modules
pub mod diarization_realtime;
pub mod transcription_quality_analyzer;
pub mod quality_analyzer_integration_test;
pub mod quality_analyzer_demo;