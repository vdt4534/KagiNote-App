//! Device-specific audio configuration profiles
//! 
//! Provides optimized configurations for common audio devices, especially
//! Apple devices, and caches successful configurations to improve startup time.

use crate::audio::types::{AudioError, AudioDevice};
use crate::audio::resampler::ResamplingQuality;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tracing::{debug, info, warn};

/// Device profile containing optimal audio configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceProfile {
    pub device_name: String,
    pub device_id: String,
    pub preferred_sample_rate: u32,
    pub supported_sample_rates: Vec<u32>,
    pub max_channels: u8,
    pub optimal_buffer_size_ms: u32,
    pub resampling_quality: ResamplingQuality,
    pub notes: String,
    /// Timestamp when this profile was last validated
    pub last_validated: u64,
    /// Number of successful uses
    pub success_count: u32,
}

impl DeviceProfile {
    /// Create a new device profile
    pub fn new(
        device_name: String,
        device_id: String,
        preferred_sample_rate: u32,
        supported_sample_rates: Vec<u32>,
        max_channels: u8,
    ) -> Self {
        let resampling_quality = match preferred_sample_rate {
            44100 | 48000 => ResamplingQuality::High, // Common rates, good quality
            88200 | 96000 => ResamplingQuality::Medium, // High rates, balance quality/perf
            _ => ResamplingQuality::Fast, // Other rates, prioritize performance
        };

        Self {
            device_name: device_name.clone(),
            device_id,
            preferred_sample_rate,
            supported_sample_rates,
            max_channels,
            optimal_buffer_size_ms: 100,
            resampling_quality,
            notes: format!("Auto-generated profile for {}", device_name),
            last_validated: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            success_count: 0,
        }
    }

    /// Update profile after successful use
    pub fn record_success(&mut self) {
        self.success_count += 1;
        self.last_validated = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
    }

    /// Check if this profile is still valid (not too old)
    pub fn is_valid(&self) -> bool {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        // Profile is valid for 30 days
        now - self.last_validated < 30 * 24 * 60 * 60
    }
}

/// Device profile manager for caching and retrieving optimal configurations
pub struct DeviceProfileManager {
    profiles: HashMap<String, DeviceProfile>,
    cache_file: Option<PathBuf>,
    built_in_profiles: HashMap<String, DeviceProfile>,
}

impl DeviceProfileManager {
    /// Create new device profile manager
    pub fn new() -> Result<Self, AudioError> {
        let mut manager = Self {
            profiles: HashMap::new(),
            cache_file: Self::get_cache_file_path(),
            built_in_profiles: HashMap::new(),
        };

        manager.initialize_built_in_profiles();
        manager.load_cached_profiles()?;

        Ok(manager)
    }

    /// Initialize built-in profiles for common Apple devices
    fn initialize_built_in_profiles(&mut self) {
        // MacBook Pro built-in microphone
        self.built_in_profiles.insert(
            "MacBook Pro Microphone".to_string(),
            DeviceProfile {
                device_name: "MacBook Pro Microphone".to_string(),
                device_id: "macbook_pro_mic".to_string(),
                preferred_sample_rate: 48000,
                supported_sample_rates: vec![48000, 44100, 32000, 24000, 16000],
                max_channels: 1,
                optimal_buffer_size_ms: 100,
                resampling_quality: ResamplingQuality::High,
                notes: "Built-in MacBook Pro microphone - typically supports 48kHz best".to_string(),
                last_validated: 0,
                success_count: 0,
            }
        );

        // MacBook Air built-in microphone
        self.built_in_profiles.insert(
            "MacBook Air Microphone".to_string(),
            DeviceProfile {
                device_name: "MacBook Air Microphone".to_string(),
                device_id: "macbook_air_mic".to_string(),
                preferred_sample_rate: 48000,
                supported_sample_rates: vec![48000, 44100, 24000, 16000],
                max_channels: 1,
                optimal_buffer_size_ms: 100,
                resampling_quality: ResamplingQuality::High,
                notes: "Built-in MacBook Air microphone - typically supports 48kHz best".to_string(),
                last_validated: 0,
                success_count: 0,
            }
        );

        // iMac built-in microphone
        self.built_in_profiles.insert(
            "iMac Microphone".to_string(),
            DeviceProfile {
                device_name: "iMac Microphone".to_string(),
                device_id: "imac_mic".to_string(),
                preferred_sample_rate: 48000,
                supported_sample_rates: vec![48000, 44100, 32000, 24000, 16000],
                max_channels: 1,
                optimal_buffer_size_ms: 100,
                resampling_quality: ResamplingQuality::High,
                notes: "Built-in iMac microphone - typically supports 48kHz best".to_string(),
                last_validated: 0,
                success_count: 0,
            }
        );

        // Generic USB microphones
        self.built_in_profiles.insert(
            "USB Audio Device".to_string(),
            DeviceProfile {
                device_name: "USB Audio Device".to_string(),
                device_id: "usb_generic".to_string(),
                preferred_sample_rate: 44100,
                supported_sample_rates: vec![44100, 48000, 32000, 24000, 16000],
                max_channels: 2,
                optimal_buffer_size_ms: 100,
                resampling_quality: ResamplingQuality::Medium,
                notes: "Generic USB audio device - commonly supports 44.1kHz".to_string(),
                last_validated: 0,
                success_count: 0,
            }
        );

        info!("Initialized {} built-in device profiles", self.built_in_profiles.len());
    }

    /// Get profile for a device, creating one if needed
    pub fn get_or_create_profile(&mut self, device: &AudioDevice) -> DeviceProfile {
        // First check cached profiles
        if let Some(profile) = self.profiles.get(&device.id) {
            if profile.is_valid() {
                debug!("Using cached profile for device: {}", device.name);
                return profile.clone();
            } else {
                warn!("Cached profile for {} is expired, will recreate", device.name);
            }
        }

        // Check built-in profiles by name matching
        for (name_pattern, built_in_profile) in &self.built_in_profiles {
            if device.name.contains(name_pattern) || name_pattern.contains(&device.name) {
                info!("Using built-in profile for device: {} (matched: {})", device.name, name_pattern);
                let mut profile = built_in_profile.clone();
                profile.device_id = device.id.clone();
                profile.device_name = device.name.clone();
                return profile;
            }
        }

        // Create new profile from device capabilities
        info!("Creating new profile for unknown device: {}", device.name);
        let preferred_rate = Self::determine_preferred_sample_rate(&device.sample_rates);
        
        DeviceProfile::new(
            device.name.clone(),
            device.id.clone(),
            preferred_rate,
            device.sample_rates.clone(),
            device.channels,
        )
    }

    /// Cache a successful profile
    pub fn cache_profile(&mut self, mut profile: DeviceProfile) -> Result<(), AudioError> {
        profile.record_success();
        self.profiles.insert(profile.device_id.clone(), profile);
        self.save_cached_profiles()
    }

    /// Determine the best sample rate from a list of supported rates
    fn determine_preferred_sample_rate(supported_rates: &[u32]) -> u32 {
        // Priority order: 48kHz > 44.1kHz > 32kHz > other rates > 16kHz
        let priority_rates = vec![48000, 44100, 32000, 24000, 22050];
        
        for &priority_rate in &priority_rates {
            if supported_rates.contains(&priority_rate) {
                return priority_rate;
            }
        }

        // Fallback to the highest supported rate, or 16kHz if list is empty
        supported_rates.iter().max().copied().unwrap_or(16000)
    }

    /// Get cache file path
    fn get_cache_file_path() -> Option<PathBuf> {
        if let Some(mut cache_dir) = dirs::cache_dir() {
            cache_dir.push("KagiNote");
            cache_dir.push("device_profiles.json");
            std::fs::create_dir_all(cache_dir.parent()?).ok()?;
            Some(cache_dir)
        } else {
            warn!("Could not determine cache directory for device profiles");
            None
        }
    }

    /// Load cached profiles from disk
    fn load_cached_profiles(&mut self) -> Result<(), AudioError> {
        if let Some(ref cache_file) = self.cache_file {
            if cache_file.exists() {
                match std::fs::read_to_string(cache_file) {
                    Ok(contents) => {
                        match serde_json::from_str::<HashMap<String, DeviceProfile>>(&contents) {
                            Ok(profiles) => {
                                let valid_count = profiles.values().filter(|p| p.is_valid()).count();
                                info!("Loaded {} device profiles ({} valid)", profiles.len(), valid_count);
                                self.profiles = profiles;
                                // Remove expired profiles
                                self.profiles.retain(|_, profile| profile.is_valid());
                            }
                            Err(e) => {
                                warn!("Failed to parse device profiles cache: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        warn!("Failed to read device profiles cache: {}", e);
                    }
                }
            }
        }
        Ok(())
    }

    /// Save cached profiles to disk
    fn save_cached_profiles(&self) -> Result<(), AudioError> {
        if let Some(ref cache_file) = self.cache_file {
            match serde_json::to_string_pretty(&self.profiles) {
                Ok(json) => {
                    if let Err(e) = std::fs::write(cache_file, json) {
                        warn!("Failed to save device profiles cache: {}", e);
                    } else {
                        debug!("Saved {} device profiles to cache", self.profiles.len());
                    }
                }
                Err(e) => {
                    warn!("Failed to serialize device profiles: {}", e);
                }
            }
        }
        Ok(())
    }

    /// Get troubleshooting suggestions for a device
    pub fn get_troubleshooting_suggestions(&self, device_name: &str) -> Vec<String> {
        let mut suggestions = Vec::new();

        if device_name.contains("MacBook") || device_name.contains("iMac") {
            suggestions.extend(vec![
                "This appears to be an Apple built-in microphone".to_string(),
                "These devices typically work best with 48kHz sample rate".to_string(),
                "Check System Preferences > Security & Privacy > Microphone permissions".to_string(),
                "Ensure no other applications are using the microphone".to_string(),
            ]);
        } else if device_name.contains("USB") {
            suggestions.extend(vec![
                "USB audio devices may need driver installation".to_string(),
                "Try unplugging and reconnecting the USB device".to_string(),
                "Check if the device works with other applications first".to_string(),
            ]);
        } else {
            suggestions.extend(vec![
                "This appears to be an external audio device".to_string(),
                "Verify the device is properly connected and recognized by the system".to_string(),
                "Check if device-specific drivers are needed".to_string(),
            ]);
        }

        suggestions.push("If problems persist, try restarting the application".to_string());
        suggestions
    }

    /// Get statistics about cached profiles
    pub fn get_stats(&self) -> ProfileStats {
        ProfileStats {
            total_profiles: self.profiles.len(),
            valid_profiles: self.profiles.values().filter(|p| p.is_valid()).count(),
            built_in_profiles: self.built_in_profiles.len(),
            most_successful_device: self.profiles.values()
                .max_by_key(|p| p.success_count)
                .map(|p| p.device_name.clone()),
        }
    }
}

/// Statistics about device profiles
#[derive(Debug)]
pub struct ProfileStats {
    pub total_profiles: usize,
    pub valid_profiles: usize,
    pub built_in_profiles: usize,
    pub most_successful_device: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio::types::AudioSource;

    fn create_test_device(name: &str, sample_rates: Vec<u32>) -> AudioDevice {
        AudioDevice {
            id: format!("test_{}", name.to_lowercase().replace(" ", "_")),
            name: name.to_string(),
            is_input_device: true,
            is_default: false,
            sample_rates,
            channels: 1,
        }
    }

    #[test]
    fn test_built_in_profile_matching() {
        let mut manager = DeviceProfileManager::new().unwrap();
        
        let macbook_device = create_test_device("MacBook Pro Microphone", vec![48000, 44100]);
        let profile = manager.get_or_create_profile(&macbook_device);
        
        assert_eq!(profile.preferred_sample_rate, 48000);
        assert_eq!(profile.resampling_quality as u8, ResamplingQuality::High as u8);
    }

    #[test]
    fn test_profile_creation_for_unknown_device() {
        let mut manager = DeviceProfileManager::new().unwrap();
        
        let unknown_device = create_test_device("Unknown Microphone", vec![44100, 32000]);
        let profile = manager.get_or_create_profile(&unknown_device);
        
        assert_eq!(profile.preferred_sample_rate, 44100);
        assert_eq!(profile.device_name, "Unknown Microphone");
    }

    #[test]
    fn test_sample_rate_priority() {
        assert_eq!(DeviceProfileManager::determine_preferred_sample_rate(&[16000, 44100, 48000]), 48000);
        assert_eq!(DeviceProfileManager::determine_preferred_sample_rate(&[16000, 22050, 32000]), 32000);
        assert_eq!(DeviceProfileManager::determine_preferred_sample_rate(&[16000]), 16000);
        assert_eq!(DeviceProfileManager::determine_preferred_sample_rate(&[]), 16000);
    }

    #[test]
    fn test_profile_validity() {
        let mut profile = DeviceProfile::new(
            "Test Device".to_string(),
            "test_device".to_string(),
            48000,
            vec![48000],
            1,
        );
        
        assert!(profile.is_valid());
        
        // Set last validated to 40 days ago (should be invalid)
        profile.last_validated = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() - (40 * 24 * 60 * 60);
        
        assert!(!profile.is_valid());
    }
}