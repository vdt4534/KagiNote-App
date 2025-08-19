//! Audio processing module
//! 
//! Provides audio capture, voice activity detection, and related functionality.

pub mod capture;
pub mod types;
pub mod vad;

pub use types::*;