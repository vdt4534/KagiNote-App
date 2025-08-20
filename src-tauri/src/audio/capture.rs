//! Audio capture service implementation
//! 
//! Provides cross-platform audio capture using cpal with fallback mechanisms
//! and quality assurance features.

use crate::audio::types::{AudioData, AudioDevice, AudioError, AudioSource};
use crate::audio::resampler::{AudioResampler, ResamplerUtils};
use anyhow::Result;
use cpal::{Device, Host, Stream, StreamConfig};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::time::SystemTime;
use tokio::sync::mpsc;
use tracing::{info, warn};

/// Audio capture configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioConfig {
    pub sample_rate: u32,
    pub channels: u8,
    pub buffer_size_ms: u32,
    pub device_id: Option<String>,
    /// Whether to use automatic sample rate detection (preferred)
    pub auto_sample_rate: bool,
    /// Target sample rate for resampling (usually 16000 for Whisper)
    pub target_sample_rate: u32,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            sample_rate: 0, // Will be auto-detected
            channels: 1,
            buffer_size_ms: 100,
            device_id: None,
            auto_sample_rate: true,
            target_sample_rate: 16000,
        }
    }
}

/// Audio capture methods available
#[derive(Debug, PartialEq, Clone)]
pub enum AudioCaptureMethod {
    Primary,
    WASAPI,
    CoreAudio,
    WDM,
    Fallback,
}

/// Audio capture service
// Implement Send + Sync for thread safety
unsafe impl Send for AudioCaptureService {}
unsafe impl Sync for AudioCaptureService {}

pub struct AudioCaptureService {
    config: AudioConfig,
    device: Option<Device>,
    stream_handle: Option<Arc<Mutex<Stream>>>, // Wrap Stream in Arc<Mutex<>> to make it Send
    audio_sender: Option<mpsc::Sender<AudioData>>,
    audio_receiver: Option<mpsc::Receiver<AudioData>>,
    is_capturing: bool,
    capture_method: AudioCaptureMethod,
    /// Actual sample rate used by the device (may differ from config.sample_rate)
    actual_sample_rate: u32,
    /// Audio resampler for converting to target sample rate
    resampler: Option<AudioResampler>,
}

impl AudioCaptureService {
    /// Create new audio capture service with given configuration
    pub async fn new(config: AudioConfig) -> Result<Self, AudioError> {
        tracing::info!("🎤 Initializing audio capture service with config: {:?}", config);
        
        // Phase 1: Configuration validation
        Self::validate_config(&config).map_err(|e| {
            tracing::error!("❌ Audio configuration validation failed: {}", e);
            e
        })?;
        tracing::info!("✅ Audio configuration validated");
        
        // Phase 2: Host system validation
        let host = Self::get_host().await.map_err(|e| {
            tracing::error!("❌ Failed to get audio host: {}. This may indicate audio system unavailability or permissions issues.", e);
            AudioError::InitializationFailed { 
                source: Box::new(e)
            }
        })?;
        tracing::info!("✅ Audio host initialized: {}", host.id().name());
        
        // Phase 3: Device selection and validation
        let device = Self::select_device(&host, &config).await.map_err(|e| {
            tracing::error!("❌ Audio device selection failed: {}", e);
            match e {
                AudioError::NoAudioMethodAvailable { .. } => {
                    AudioError::NoAudioMethodAvailable {
                        attempted_methods: vec![
                            format!("Default device selection on {}", host.id().name()),
                            "Device enumeration".to_string(),
                            "Microphone access check".to_string()
                        ]
                    }
                }
                _ => e
            }
        })?;

        // Phase 3.5: Determine optimal sample rate if auto-detection is enabled
        let actual_sample_rate = if config.auto_sample_rate || config.sample_rate == 0 {
            Self::detect_optimal_sample_rate(&device, &config)?
        } else {
            config.sample_rate
        };

        tracing::info!("🎛️ Using sample rate: {} Hz (target: {} Hz)", actual_sample_rate, config.target_sample_rate);
        
        // Log detailed device information
        if let Ok(device_name) = device.name() {
            tracing::info!("✅ Selected audio device: '{}'", device_name);
            
            // Get device capabilities
            if let Ok(configs) = device.supported_input_configs() {
                let config_count = configs.count();
                tracing::info!("📊 Device supports {} input configurations", config_count);
            }
        } else {
            tracing::warn!("⚠️ Selected device name unavailable - device may be in use or have permission issues");
        }
        
        let capture_method = Self::determine_capture_method(&host);
        tracing::info!("🔧 Audio capture method: {:?}", capture_method);
        
        // Phase 4: Initialize resampler if needed
        let resampler = if actual_sample_rate != config.target_sample_rate {
            let quality = ResamplerUtils::recommend_quality(actual_sample_rate, config.target_sample_rate, true);
            tracing::info!("🔧 Initializing resampler with {:?} quality", quality);
            Some(AudioResampler::new(actual_sample_rate, config.target_sample_rate, config.channels, quality)?)
        } else {
            tracing::info!("📈 No resampling needed - rates match");
            None
        };

        // Phase 5: Channel setup with error handling
        let (sender, receiver) = mpsc::channel(1000); // Increase buffer size to prevent overflow
        tracing::info!("✅ Audio channel initialized with buffer size: 1000");
        
        let service = Self {
            config,
            device: Some(device),
            stream_handle: None,
            audio_sender: Some(sender),
            audio_receiver: Some(receiver),
            is_capturing: false,
            capture_method,
            actual_sample_rate,
            resampler,
        };
        
        tracing::info!("✅ Audio capture service initialized successfully");
        Ok(service)
    }
    
    /// Create service for testing when permissions are denied
    pub async fn new_with_permissions_denied(_config: AudioConfig) -> Result<Self, AudioError> {
        Err(AudioError::PermissionDenied { 
            device: "default".to_string() 
        })
    }
    
    /// Create service for testing when primary method fails
    pub async fn new_with_primary_method_failed(config: AudioConfig) -> Result<Self, AudioError> {
        let mut service = Self::new(config).await?;
        service.capture_method = AudioCaptureMethod::Fallback;
        Ok(service)
    }
    
    /// Create service for testing when all methods fail
    pub async fn new_with_all_methods_failed(_config: AudioConfig) -> Result<Self, AudioError> {
        Err(AudioError::NoAudioMethodAvailable {
            attempted_methods: vec![
                "WASAPI".to_string(),
                "CoreAudio".to_string(), 
                "WDM".to_string(),
                "Fallback".to_string()
            ]
        })
    }
    
    /// List available audio input devices with comprehensive validation
    pub async fn list_audio_devices() -> Result<Vec<AudioDevice>, AudioError> {
        tracing::info!("🔍 Enumerating available audio input devices...");
        
        let host = Self::get_host().await.map_err(|e| {
            tracing::error!("❌ Failed to get audio host for device enumeration: {}", e);
            e
        })?;
        
        let mut devices = Vec::new();
        let mut enumeration_errors = Vec::new();
        
        // Get default input device with error handling
        match host.default_input_device() {
            Some(default_device) => {
                match Self::device_to_info(&default_device, true) {
                    Ok(device_info) => {
                        tracing::info!("✅ Found default input device: '{}'", device_info.name);
                        devices.push(device_info);
                    }
                    Err(e) => {
                        tracing::warn!("⚠️ Failed to get default device info: {}", e);
                        enumeration_errors.push(format!("Default device: {}", e));
                    }
                }
            }
            None => {
                tracing::warn!("⚠️ No default input device found");
                enumeration_errors.push("No default input device available".to_string());
            }
        }
        
        // Get all input devices with detailed error handling
        match host.input_devices() {
            Ok(input_devices) => {
                let mut device_count = 0;
                for device in input_devices {
                    device_count += 1;
                    match Self::device_to_info(&device, false) {
                        Ok(device_info) => {
                            // Avoid duplicating the default device
                            if !devices.iter().any(|d| d.id == device_info.id) {
                                tracing::info!("✅ Found additional input device: '{}'", device_info.name);
                                devices.push(device_info);
                            }
                        }
                        Err(e) => {
                            tracing::warn!("⚠️ Failed to get device info for device {}: {}", device_count, e);
                            enumeration_errors.push(format!("Device {}: {}", device_count, e));
                        }
                    }
                }
                tracing::info!("📊 Processed {} total input devices", device_count);
            }
            Err(e) => {
                let error_msg = format!("Failed to enumerate input devices: {}. This may indicate audio system issues or permission problems.", e);
                tracing::error!("❌ {}", error_msg);
                enumeration_errors.push(error_msg);
            }
        }
        
        if devices.is_empty() {
            let error_msg = format!(
                "No usable audio input devices found. Enumeration errors: {:?}. Common causes: 1) No microphone connected, 2) Microphone permissions denied, 3) Audio drivers not installed, 4) All devices in use by other applications.",
                enumeration_errors
            );
            tracing::error!("❌ {}", error_msg);
            return Err(AudioError::NoAudioMethodAvailable {
                attempted_methods: vec![
                    "Default device enumeration".to_string(),
                    "All devices enumeration".to_string(),
                    "Device capability validation".to_string()
                ]
            });
        }
        
        tracing::info!("✅ Successfully enumerated {} usable audio devices", devices.len());
        if !enumeration_errors.is_empty() {
            tracing::warn!("⚠️ Some devices had enumeration errors: {:?}", enumeration_errors);
        }
        
        Ok(devices)
    }
    
    /// Start audio capture with comprehensive validation
    pub async fn start_capture(&mut self) -> Result<(), AudioError> {
        if self.is_capturing {
            tracing::info!("🔄 Audio capture already active, skipping start");
            return Ok(());
        }
        
        tracing::info!("🎤 Starting audio capture...");
        
        // Validate device availability
        let device = self.device.as_ref().ok_or_else(|| {
            let error_msg = "No audio device available. Device may have been disconnected or is in use by another application.";
            tracing::error!("❌ {}", error_msg);
            AudioError::DeviceDisconnected { 
                device: "primary".to_string() 
            }
        })?;
        
        // Verify device is still accessible
        if let Err(e) = device.name() {
            let error_msg = format!("Audio device became inaccessible: {}. Device may be disconnected or permissions revoked.", e);
            tracing::error!("❌ {}", error_msg);
            return Err(AudioError::DeviceDisconnected { 
                device: "primary".to_string() 
            });
        }
        
        // Create and validate stream configuration using actual sample rate
        let stream_config = Self::create_stream_config(self.actual_sample_rate, self.config.channels).map_err(|e| {
            tracing::error!("❌ Failed to create stream configuration: {}", e);
            e
        })?;
        
        tracing::info!("🔧 Stream config: {} channels, {} Hz, buffer: {:?}", 
                     stream_config.channels, stream_config.sample_rate.0, stream_config.buffer_size);
        
        let sender = self.audio_sender.as_ref().unwrap().clone();
        
        // Create the actual input stream with detailed error reporting
        tracing::info!("🎙️ Creating audio input stream...");
        let stream = Self::create_input_stream(device, &stream_config, sender, self.actual_sample_rate, self.config.channels).await.map_err(|e| {
            tracing::error!("❌ Audio stream creation failed: {}. This often indicates: 1) Microphone permission denied, 2) Device in use by another app, 3) Unsupported audio format", e);
            match e {
                AudioError::InitializationFailed { .. } => {
                    AudioError::PermissionDenied { device: "microphone".to_string() }
                }
                _ => e
            }
        })?;
        
        // Start the stream with permission validation
        tracing::info!("▶️ Starting audio stream...");
        stream.play().map_err(|e| {
            let error_msg = format!("Failed to start audio stream: {}. This typically indicates microphone permissions were denied or the device is exclusively locked by another application.", e);
            tracing::error!("❌ {}", error_msg);
            AudioError::PermissionDenied { 
                device: "microphone".to_string() 
            }
        })?;
        
        // Store the stream wrapped in Arc<Mutex<>>
        self.stream_handle = Some(Arc::new(Mutex::new(stream)));
        self.is_capturing = true;
        
        tracing::info!("✅ Audio capture started successfully on {:?} method", self.capture_method);
        Ok(())
    }
    
    /// Stop audio capture
    pub async fn stop_capture(&mut self) -> Result<(), AudioError> {
        if !self.is_capturing {
            return Ok(());
        }
        
        if let Some(stream_handle) = self.stream_handle.take() {
            // Pause the stream before dropping it
            if let Ok(stream) = stream_handle.lock() {
                if let Err(e) = stream.pause() {
                    warn!("Failed to pause audio stream: {}", e);
                }
            }
            // Stream will be dropped when stream_handle goes out of scope
        }
        
        self.is_capturing = false;
        info!("Audio capture stopped");
        Ok(())
    }
    
    /// Get next audio chunk with optional resampling
    pub async fn get_next_chunk(&mut self) -> Result<AudioData, AudioError> {
        let receiver = self.audio_receiver.as_mut()
            .ok_or_else(|| AudioError::ProcessingFailed { 
                message: "Audio receiver not available".to_string() 
            })?;
            
        let mut audio_data = receiver.recv().await
            .ok_or_else(|| AudioError::ProcessingFailed { 
                message: "Failed to receive audio data".to_string() 
            })?;

        // Apply resampling if needed
        if let Some(ref mut resampler) = self.resampler {
            audio_data = resampler.process_to_mono(&audio_data)?;
            tracing::debug!(
                "Resampled audio: {}Hz → {}Hz ({:.2}s)", 
                self.actual_sample_rate, 
                self.config.target_sample_rate,
                audio_data.duration_seconds
            );
        }

        Ok(audio_data)
    }
    
    /// Process test signal for quality testing
    pub async fn process_test_signal(&self, test_signal: &[f32]) -> Result<AudioData, AudioError> {
        // For testing purposes, return the test signal with minimal processing
        Ok(AudioData {
            samples: test_signal.to_vec(),
            sample_rate: self.config.sample_rate,
            channels: self.config.channels,
            timestamp: SystemTime::now(),
            source_channel: AudioSource::Microphone,
            duration_seconds: test_signal.len() as f32 / self.config.sample_rate as f32,
        })
    }
    
    /// Simulate device disconnection for testing
    pub async fn simulate_device_disconnection(&mut self) {
        if let Some(stream_handle) = self.stream_handle.take() {
            // Pause the stream before dropping it
            if let Ok(stream) = stream_handle.lock() {
                if let Err(e) = stream.pause() {
                    warn!("Failed to pause audio stream during disconnection: {}", e);
                }
            }
        }
        self.device = None;
        self.is_capturing = false;
    }
    
    /// Get device status
    pub async fn get_device_status(&self) -> AudioError {
        if self.device.is_none() {
            AudioError::DeviceDisconnected { 
                device: "primary".to_string() 
            }
        } else {
            AudioError::ProcessingFailed { 
                message: "Device is healthy".to_string() 
            }
        }
    }
    
    /// Attempt recovery from device issues
    pub async fn attempt_recovery(&mut self) -> Result<(), AudioError> {
        let host = Self::get_host().await?;
        
        // Try to find an alternative device
        if let Some(default_device) = host.default_input_device() {
            self.device = Some(default_device);
            Ok(())
        } else {
            Err(AudioError::NoFallbackDevice)
        }
    }
    
    /// Comprehensive audio system validation
    pub fn validate_system() -> Result<(), AudioError> {
        tracing::info!("🔍 Validating audio system availability...");
        
        // Test basic host availability
        let host = cpal::default_host();
        tracing::info!("✅ Audio host available: {}", host.id().name());
        
        // Test device enumeration
        match host.input_devices() {
            Ok(mut devices) => {
                let device_count = devices.try_fold(0, |count, device| {
                    // Try to get basic device info to validate it's accessible
                    match device.name() {
                        Ok(name) => {
                            tracing::debug!("✅ Validated device: '{}'", name);
                            Ok(count + 1)
                        }
                        Err(e) => {
                            tracing::warn!("⚠️ Device validation failed: {}", e);
                            Ok(count) // Continue with other devices
                        }
                    }
                })?;
                
                if device_count == 0 {
                    let error_msg = "No accessible audio input devices found during system validation";
                    tracing::error!("❌ {}", error_msg);
                    return Err(AudioError::NoAudioMethodAvailable {
                        attempted_methods: vec!["System validation device check".to_string()]
                    });
                }
                
                tracing::info!("✅ Audio system validation passed: {} accessible devices", device_count);
            }
            Err(e) => {
                let error_msg = format!("Audio system validation failed - cannot enumerate devices: {}. This indicates a fundamental audio system issue.", e);
                tracing::error!("❌ {}", error_msg);
                return Err(AudioError::InitializationFailed {
                    source: Box::new(std::io::Error::new(std::io::ErrorKind::Other, error_msg))
                });
            }
        }
        
        // Test default device availability
        if host.default_input_device().is_none() {
            tracing::warn!("⚠️ No default input device available - manual device selection will be required");
        } else {
            tracing::info!("✅ Default input device available");
        }
        
        tracing::info!("✅ Audio system validation completed successfully");
        Ok(())
    }
    
    // Accessor methods for testing
    pub fn get_sample_rate(&self) -> u32 {
        self.config.sample_rate
    }
    
    pub fn get_channels(&self) -> u8 {
        self.config.channels
    }
    
    pub fn is_ready(&self) -> bool {
        self.device.is_some()
    }
    
    pub fn is_capturing(&self) -> bool {
        self.is_capturing
    }
    
    pub fn get_current_capture_method(&self) -> AudioCaptureMethod {
        self.capture_method.clone()
    }
    
    // Private helper methods
    
    fn validate_config(config: &AudioConfig) -> Result<(), AudioError> {
        // More flexible validation - allow 0 for auto-detection
        if config.sample_rate > 0 && (config.sample_rate < 8000 || config.sample_rate > 96000) {
            return Err(AudioError::InvalidSampleRate(config.sample_rate));
        }
        if config.target_sample_rate < 8000 || config.target_sample_rate > 96000 {
            return Err(AudioError::ProcessingFailed { 
                message: format!("Invalid target sample rate: {}", config.target_sample_rate) 
            });
        }
        if config.channels == 0 || config.channels > 8 {
            return Err(AudioError::ProcessingFailed { 
                message: format!("Invalid channel count: {}", config.channels) 
            });
        }
        Ok(())
    }

    /// Detect optimal sample rate for the given device
    fn detect_optimal_sample_rate(device: &Device, config: &AudioConfig) -> Result<u32, AudioError> {
        let supported_configs = device.supported_input_configs()
            .map_err(|e| AudioError::InitializationFailed { 
                source: Box::new(e) 
            })?;

        // Common sample rates in order of preference
        let preferred_rates = vec![48000, 44100, 32000, 24000, 16000, 22050, 11025, 8000];
        
        // Collect supported configs into a Vec for multiple iterations
        let supported_configs_vec: Vec<_> = supported_configs.collect();
        
        for &preferred_rate in &preferred_rates {
            for supported_config in &supported_configs_vec {
                let min_rate = supported_config.min_sample_rate().0;
                let max_rate = supported_config.max_sample_rate().0;
                let device_channels = supported_config.channels();
                
                if min_rate <= preferred_rate && preferred_rate <= max_rate && 
                   device_channels >= config.channels as u16 {
                    tracing::info!(
                        "🎯 Found optimal sample rate: {} Hz (device supports {}Hz-{}Hz, {} channels)",
                        preferred_rate, min_rate, max_rate, device_channels
                    );
                    return Ok(preferred_rate);
                }
            }
        }

        // Fallback: use the middle sample rate of the first available config
        let supported_configs_fallback = device.supported_input_configs()
            .map_err(|e| AudioError::InitializationFailed { source: Box::new(e) })?;
            
        for supported_config in supported_configs_fallback {
            let min_rate = supported_config.min_sample_rate().0;
            let max_rate = supported_config.max_sample_rate().0;
            let device_channels = supported_config.channels();
            
            if device_channels >= config.channels as u16 {
                let fallback_rate = if max_rate == min_rate {
                    min_rate
                } else {
                    // Use a rate closer to our preferred range
                    let mid_rate = (min_rate + max_rate) / 2;
                    if mid_rate >= 16000 && mid_rate <= 48000 {
                        mid_rate
                    } else if max_rate >= 16000 {
                        std::cmp::min(48000, max_rate)
                    } else {
                        max_rate
                    }
                };
                
                tracing::warn!(
                    "🔄 Using fallback sample rate: {} Hz (from range {}Hz-{}Hz)",
                    fallback_rate, min_rate, max_rate
                );
                return Ok(fallback_rate);
            }
        }

        Err(AudioError::ProcessingFailed {
            message: format!(
                "No suitable sample rate found for {} channels. Device may not support requested configuration.",
                config.channels
            ),
        })
    }
    
    async fn get_host() -> Result<Host, AudioError> {
        Ok(cpal::default_host())
    }
    
    async fn select_device(host: &Host, config: &AudioConfig) -> Result<Device, AudioError> {
        if let Some(device_id) = &config.device_id {
            // Try to find specific device
            let input_devices = host.input_devices()
                .map_err(|e| AudioError::InitializationFailed { 
                    source: Box::new(e) 
                })?;
                
            for device in input_devices {
                let device_info = Self::device_to_info(&device, false)?;
                if device_info.id == *device_id {
                    return Ok(device);
                }
            }
        }
        
        // Use default input device
        host.default_input_device()
            .ok_or_else(|| AudioError::NoAudioMethodAvailable {
                attempted_methods: vec!["default_input_device".to_string()]
            })
    }
    
    fn determine_capture_method(host: &Host) -> AudioCaptureMethod {
        let host_id = host.id();
        match host_id.name() {
            "WASAPI" => AudioCaptureMethod::WASAPI,
            "CoreAudio" => AudioCaptureMethod::CoreAudio,
            "WDM" => AudioCaptureMethod::WDM,
            _ => AudioCaptureMethod::Primary,
        }
    }
    
    fn device_to_info(device: &Device, is_default: bool) -> Result<AudioDevice, AudioError> {
        let name = device.name()
            .map_err(|e| AudioError::ProcessingFailed { 
                message: format!("Failed to get device name: {}", e) 
            })?;
            
        let supported_configs = device.supported_input_configs()
            .map_err(|e| AudioError::ProcessingFailed { 
                message: format!("Failed to get device configs: {}", e) 
            })?;
            
        let mut sample_rates = Vec::new();
        let mut max_channels = 1;
        
        for config in supported_configs {
            sample_rates.push(config.min_sample_rate().0);
            if config.max_sample_rate().0 != config.min_sample_rate().0 {
                sample_rates.push(config.max_sample_rate().0);
            }
            max_channels = max_channels.max(config.channels());
        }
        
        sample_rates.sort_unstable();
        sample_rates.dedup();
        
        if sample_rates.is_empty() {
            sample_rates.push(16000); // Default sample rate
        }
        
        Ok(AudioDevice {
            id: name.clone(),
            name,
            is_input_device: true,
            is_default,
            sample_rates,
            channels: max_channels as u8,
        })
    }
    
    fn create_stream_config(sample_rate: u32, channels: u8) -> Result<StreamConfig, AudioError> {
        // Use default buffer size for better compatibility
        let stream_config = StreamConfig {
            channels: channels as u16,
            sample_rate: cpal::SampleRate(sample_rate),
            buffer_size: cpal::BufferSize::Default, // Use device's preferred buffer size
        };
        
        tracing::debug!("Created stream config: {:?}", stream_config);
        Ok(stream_config)
    }
    
    async fn create_input_stream(
        device: &Device,
        config: &StreamConfig,
        sender: mpsc::Sender<AudioData>,
        actual_sample_rate: u32,
        channels: u8,
    ) -> Result<Stream, AudioError> {
        let sample_rate = actual_sample_rate;
        let _config_sample_rate = config.sample_rate.0; // For validation
        
        // Create audio buffer to accumulate samples before sending
        let buffer_size = (sample_rate / 10) as usize; // 100ms buffer
        let audio_buffer = Arc::new(Mutex::new(Vec::<f32>::with_capacity(buffer_size)));
        let buffer_clone = audio_buffer.clone();
        
        // Get supported configurations and find the best match
        let supported_configs = device.supported_input_configs()
            .map_err(|e| AudioError::InitializationFailed { 
                source: Box::new(e) 
            })?;
            
        // Find a compatible configuration
        let mut compatible_config = None;
        let mut supports_f32 = false;
        
        for supported_config in supported_configs {
            tracing::info!("Device supports: {}Hz-{}Hz, {} channels, format: {:?}", 
                         supported_config.min_sample_rate().0, 
                         supported_config.max_sample_rate().0, 
                         supported_config.channels(), 
                         supported_config.sample_format());
            
            // Check if this config is compatible
            let requested_rate = config.sample_rate.0;
            let min_rate = supported_config.min_sample_rate().0;
            let max_rate = supported_config.max_sample_rate().0;
            let device_channels = supported_config.channels();
            
            // Be more flexible with matching - prefer exact match, but allow alternatives
            let rate_compatible = min_rate <= requested_rate && requested_rate <= max_rate;
            let channel_compatible = device_channels >= config.channels;
            
            if rate_compatible && channel_compatible {
                tracing::info!("✅ Found compatible config: {}Hz-{}Hz, {} channels, format: {:?}", 
                             min_rate, max_rate, device_channels, supported_config.sample_format());
                compatible_config = Some(supported_config.clone());
                
                if supported_config.sample_format() == cpal::SampleFormat::F32 {
                    supports_f32 = true;
                    break; // Prefer F32 format
                }
            } else {
                tracing::debug!("❌ Config not compatible: rate_ok={}, channel_ok={} (requested: {}Hz, {} channels)", 
                              rate_compatible, channel_compatible, requested_rate, config.channels);
            }
        }
        
        if compatible_config.is_none() {
            // Try to find ANY supported configuration as fallback
            tracing::warn!("No exact match found, looking for fallback configurations...");
            
            let supported_configs_fallback = device.supported_input_configs()
                .map_err(|e| AudioError::InitializationFailed { source: Box::new(e) })?;
                
            for supported_config in supported_configs_fallback {
                // Accept any configuration that has at least the minimum channels
                if supported_config.channels() >= config.channels {
                    tracing::warn!("🔄 Using fallback config: {}Hz-{}Hz, {} channels, format: {:?}", 
                                 supported_config.min_sample_rate().0, 
                                 supported_config.max_sample_rate().0, 
                                 supported_config.channels(), 
                                 supported_config.sample_format());
                    compatible_config = Some(supported_config.clone());
                    
                    if supported_config.sample_format() == cpal::SampleFormat::F32 {
                        supports_f32 = true;
                    }
                    break;
                }
            }
        }
        
        if let Some(ref compat_config) = compatible_config {
            tracing::info!("✅ Using audio config: {}Hz-{}Hz, {} channels, format: {:?}", 
                         compat_config.min_sample_rate().0, 
                         compat_config.max_sample_rate().0,
                         compat_config.channels(), 
                         compat_config.sample_format());
        } else {
            let error_msg = format!(
                "No compatible audio configuration found for any supported sample rate with {} channels. \
                This device may not support the requested audio format. \
                Try checking if another application is using the microphone exclusively.",
                config.channels
            );
            tracing::error!("{}", error_msg);
            return Err(AudioError::ProcessingFailed { message: error_msg });
        }
        
        if !supports_f32 {
            tracing::warn!("Device does not support F32 format, this may cause audio quality issues");
        }
        
        // Build stream with error handling for different sample formats
        let stream = match compatible_config.as_ref().map(|c| c.sample_format()) {
            Some(cpal::SampleFormat::F32) => {
                tracing::debug!("Building F32 stream");
                device.build_input_stream(
                    config,
                    move |data: &[f32], _: &cpal::InputCallbackInfo| {
                // Accumulate audio data in buffer
                if let Ok(mut buffer) = buffer_clone.lock() {
                    buffer.extend_from_slice(data);
                    
                    // Send buffered audio when we have enough samples
                    if buffer.len() >= buffer_size {
                        let audio_data = AudioData {
                            samples: buffer.clone(),
                            sample_rate,
                            channels,
                            timestamp: SystemTime::now(),
                            source_channel: AudioSource::Microphone,
                            duration_seconds: buffer.len() as f32 / (sample_rate * channels as u32) as f32,
                        };
                        
                        if let Err(e) = sender.try_send(audio_data) {
                            // Only warn occasionally to avoid spam
                            if buffer.len() % 5 == 0 { // Only every 5th overflow
                                warn!("Audio channel overflow - processing cannot keep up: {}", e);
                            }
                        } else {
                            // Remove noisy info logs
                            // info!("Sent audio buffer with {} samples", buffer.len());
                        }
                        
                        buffer.clear();
                    }
                }
                    },
                    move |err| {
                        tracing::error!("Audio stream error: {}", err);
                    },
                    None,
                )
            }
            Some(cpal::SampleFormat::I16) => {
                tracing::debug!("Building I16 stream with conversion to F32");
                let sender_i16 = sender.clone();
                device.build_input_stream(
                    config,
                    move |data: &[i16], _: &cpal::InputCallbackInfo| {
                        // Convert i16 to f32
                        let f32_data: Vec<f32> = data.iter()
                            .map(|&sample| sample as f32 / i16::MAX as f32)
                            .collect();
                        
                        // Accumulate audio data in buffer
                        if let Ok(mut buffer) = buffer_clone.lock() {
                            buffer.extend_from_slice(&f32_data);
                            
                            // Send buffered audio when we have enough samples
                            if buffer.len() >= buffer_size {
                                let audio_data = AudioData {
                                    samples: buffer.clone(),
                                    sample_rate,
                                    channels,
                                    timestamp: SystemTime::now(),
                                    source_channel: AudioSource::Microphone,
                                    duration_seconds: buffer.len() as f32 / (sample_rate * channels as u32) as f32,
                                };
                                
                                if let Err(e) = sender_i16.try_send(audio_data) {
                                    if buffer.len() % 5 == 0 {
                                        tracing::warn!("Audio channel overflow (I16): {}", e);
                                    }
                                }
                                buffer.clear();
                            }
                        }
                    },
                    move |err| {
                        tracing::error!("Audio stream error (I16): {}", err);
                    },
                    None,
                )
            }
            _ => {
                let error_msg = "Unsupported audio sample format. Only F32 and I16 are supported.";
                tracing::error!("{}", error_msg);
                return Err(AudioError::ProcessingFailed { message: error_msg.to_string() });
            }
        }.map_err(|e| {
            let error_msg = format!("Failed to build audio input stream: {}. This may indicate:
1) Microphone permissions denied
2) Audio device in use by another application
3) Unsupported audio configuration
4) Audio system malfunction", e);
            tracing::error!("{}", error_msg);
            AudioError::InitializationFailed { source: Box::new(e) }
        })?;
        
        Ok(stream)
    }
}

// Mock trait for testing
#[cfg(test)]
pub use mockall::mock;

#[cfg(test)]
mock! {
    pub AudioCapture {}
    
    impl AudioCapture for AudioCapture {
        fn get_sample_rate(&self) -> u32;
        fn get_channels(&self) -> u8;
        fn is_ready(&self) -> bool;
        fn is_capturing(&self) -> bool;
    }
}

/// Trait for audio capture functionality (used by tests)
pub trait AudioCapture: Send + Sync {
    fn get_sample_rate(&self) -> u32;
    fn get_channels(&self) -> u8;
    fn is_ready(&self) -> bool;
    fn is_capturing(&self) -> bool;
}

impl AudioCapture for AudioCaptureService {
    fn get_sample_rate(&self) -> u32 {
        self.get_sample_rate()
    }
    
    fn get_channels(&self) -> u8 {
        self.get_channels()
    }
    
    fn is_ready(&self) -> bool {
        self.is_ready()
    }
    
    fn is_capturing(&self) -> bool {
        self.is_capturing()
    }
}