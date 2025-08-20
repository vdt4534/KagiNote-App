//! Audio processing module
//! 
//! Provides audio capture, voice activity detection, resampling, device profiles, and related functionality.

pub mod capture;
pub mod types;
pub mod vad;
pub mod resampler;
pub mod device_profiles;

pub use types::*;
pub use resampler::{AudioResampler, ResamplerUtils, ResamplingQuality};
pub use device_profiles::{DeviceProfile, DeviceProfileManager, ProfileStats};