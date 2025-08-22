//! Real-time Audio Playback Simulator for Testing Speaker Diarization
//!
//! This module provides a comprehensive audio playback simulator that mimics real microphone
//! capture behavior for testing speaker diarization systems. It supports loading WAV and MP3
//! files using symphonia/rodio and streaming them in real-time chunks to simulate live audio capture.
//!
//! ## Key Features
//! - Loads audio files in WAV, MP3, FLAC, and other formats using symphonia
//! - Converts audio to f32 samples at 16kHz (matching Whisper requirements)
//! - Real-time streaming with configurable chunk sizes (default 100ms)
//! - Accurate timing using tokio timers to match real-time playback speed
//! - Pause, resume, and stop controls for flexible testing
//! - Integration points for diarization pipeline testing
//! - Performance metrics collection and monitoring
//! - Configurable silence injection for testing edge cases

use kaginote_lib::audio::types::{AudioData, AudioError, AudioSource};
use anyhow::{Context, Result};
use rodio::{Decoder, Source};
use std::collections::VecDeque;
use std::io::BufReader;
use std::path::Path;
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, SystemTime};
use tokio::sync::mpsc;
use tokio::time::{interval, sleep, Instant};
use tracing::{debug, info, warn, error};

/// Configuration for the audio playback simulator
#[derive(Debug, Clone)]
pub struct AudioPlaybackConfig {
    /// Target sample rate for output (typically 16000 for Whisper)
    pub target_sample_rate: u32,
    /// Target channels (1 for mono, 2 for stereo)
    pub target_channels: u8,
    /// Chunk size in milliseconds for real-time streaming
    pub chunk_duration_ms: u64,
    /// Buffer size for audio chunks (number of chunks to pre-buffer)
    pub buffer_size: usize,
    /// Whether to loop the audio file when it reaches the end
    pub loop_playback: bool,
    /// Silence duration to inject between loops (milliseconds)
    pub silence_between_loops_ms: u64,
    /// Real-time speed multiplier (1.0 = real-time, 2.0 = 2x speed)
    pub speed_multiplier: f32,
    /// Whether to enable performance metrics collection
    pub enable_metrics: bool,
}

impl Default for AudioPlaybackConfig {
    fn default() -> Self {
        Self {
            target_sample_rate: 16000,
            target_channels: 1,
            chunk_duration_ms: 100, // 100ms chunks for responsive testing
            buffer_size: 10,
            loop_playback: false,
            silence_between_loops_ms: 1000,
            speed_multiplier: 1.0,
            enable_metrics: true,
        }
    }
}

/// Playback state for controlling the simulator
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaybackState {
    /// Stopped - no playback active
    Stopped,
    /// Playing - streaming audio chunks
    Playing,
    /// Paused - playback suspended but can be resumed
    Paused,
    /// Finished - reached end of audio (if not looping)
    Finished,
    /// Error state - playback failed
    Error,
}

/// Performance metrics collected during playback
#[derive(Debug, Clone, Default)]
pub struct PlaybackMetrics {
    /// Total number of chunks sent
    pub chunks_sent: u64,
    /// Total audio duration processed (seconds)
    pub total_duration_processed: f64,
    /// Average chunk processing time (microseconds)
    pub avg_chunk_processing_time_us: f64,
    /// Maximum chunk processing time (microseconds)
    pub max_chunk_processing_time_us: u64,
    /// Number of timing violations (chunks sent late)
    pub timing_violations: u64,
    /// Total playback time (wall clock time)
    pub total_playback_time: Duration,
    /// Playback start time
    pub playback_start_time: Option<Instant>,
    /// Current playback position (seconds)
    pub current_position: f64,
}

impl PlaybackMetrics {
    /// Calculate real-time factor (how fast we're processing vs real-time)
    pub fn real_time_factor(&self) -> f64 {
        if self.total_playback_time.as_secs_f64() > 0.0 {
            self.total_duration_processed / self.total_playback_time.as_secs_f64()
        } else {
            0.0
        }
    }

    /// Check if playback is keeping up with real-time
    pub fn is_keeping_up(&self) -> bool {
        self.real_time_factor() >= 0.95 // Allow 5% tolerance
    }
}

/// Main audio playback simulator
pub struct AudioPlaybackSimulator {
    config: AudioPlaybackConfig,
    audio_samples: Arc<RwLock<Vec<f32>>>,
    sample_rate: u32,
    channels: u8,
    state: Arc<RwLock<PlaybackState>>,
    position: Arc<RwLock<usize>>, // Current sample position
    audio_sender: Option<mpsc::Sender<AudioData>>,
    metrics: Arc<RwLock<PlaybackMetrics>>,
    chunk_buffer: Arc<Mutex<VecDeque<Vec<f32>>>>,
}

impl AudioPlaybackSimulator {
    /// Create a new audio playback simulator
    pub fn new(config: AudioPlaybackConfig) -> Self {
        Self {
            config,
            audio_samples: Arc::new(RwLock::new(Vec::new())),
            sample_rate: 0,
            channels: 0,
            state: Arc::new(RwLock::new(PlaybackState::Stopped)),
            position: Arc::new(RwLock::new(0)),
            audio_sender: None,
            metrics: Arc::new(RwLock::new(PlaybackMetrics::default())),
            chunk_buffer: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    /// Load an audio file and prepare for playback
    pub async fn load_audio_file<P: AsRef<Path>>(&mut self, file_path: P) -> Result<(), AudioError> {
        let path = file_path.as_ref();
        info!("Loading audio file: {}", path.display());

        // Validate file exists
        if !path.exists() {
            let error_msg = format!("Audio file not found: {}", path.display());
            error!("{}", error_msg);
            return Err(AudioError::ProcessingFailed { message: error_msg });
        }

        // Load and decode audio file using rodio/symphonia
        let file = std::fs::File::open(path)
            .with_context(|| format!("Failed to open audio file: {}", path.display()))
            .map_err(|e| AudioError::ProcessingFailed { message: e.to_string() })?;

        let buf_reader = BufReader::new(file);
        let decoder = Decoder::new(buf_reader)
            .with_context(|| format!("Failed to decode audio file: {}", path.display()))
            .map_err(|e| AudioError::ProcessingFailed { message: e.to_string() })?;

        // Get original audio properties
        let original_sample_rate = decoder.sample_rate();
        let original_channels = decoder.channels();
        
        info!("Original audio: {}Hz, {} channels", original_sample_rate, original_channels);
        info!("Target audio: {}Hz, {} channels", self.config.target_sample_rate, self.config.target_channels);

        // Convert to f32 samples and collect all samples
        let samples: Vec<f32> = decoder.convert_samples().collect();
        
        if samples.is_empty() {
            return Err(AudioError::ProcessingFailed { 
                message: "Audio file contains no samples".to_string() 
            });
        }

        info!("Loaded {} samples ({:.2}s duration)", 
              samples.len(), 
              samples.len() as f32 / (original_sample_rate * original_channels as u32) as f32);

        // Resample and convert channels if needed
        let processed_samples = self.process_audio_samples(
            samples,
            original_sample_rate,
            original_channels as u8,
        )?;

        // Store processed audio
        {
            let mut audio_guard = self.audio_samples.write().unwrap();
            *audio_guard = processed_samples;
        }

        self.sample_rate = self.config.target_sample_rate;
        self.channels = self.config.target_channels;

        // Reset state
        {
            let mut state_guard = self.state.write().unwrap();
            *state_guard = PlaybackState::Stopped;
        }
        {
            let mut pos_guard = self.position.write().unwrap();
            *pos_guard = 0;
        }

        info!("Audio file loaded successfully");
        Ok(())
    }

    /// Process audio samples (resample and convert channels)
    fn process_audio_samples(
        &self,
        samples: Vec<f32>,
        source_sample_rate: u32,
        source_channels: u8,
    ) -> Result<Vec<f32>, AudioError> {
        let mut processed = samples;

        // Convert channels (stereo to mono or vice versa)
        if source_channels != self.config.target_channels {
            processed = self.convert_channels(processed, source_channels, self.config.target_channels)?;
        }

        // Resample if needed
        if source_sample_rate != self.config.target_sample_rate {
            processed = self.resample_audio(
                processed,
                source_sample_rate,
                self.config.target_sample_rate,
                self.config.target_channels,
            )?;
        }

        Ok(processed)
    }

    /// Convert between channel configurations
    fn convert_channels(&self, samples: Vec<f32>, from_channels: u8, to_channels: u8) -> Result<Vec<f32>, AudioError> {
        match (from_channels, to_channels) {
            (2, 1) => {
                // Stereo to mono - average left and right channels
                let mono_samples: Vec<f32> = samples
                    .chunks_exact(2)
                    .map(|stereo_frame| (stereo_frame[0] + stereo_frame[1]) / 2.0)
                    .collect();
                Ok(mono_samples)
            }
            (1, 2) => {
                // Mono to stereo - duplicate mono channel
                let stereo_samples: Vec<f32> = samples
                    .into_iter()
                    .flat_map(|mono_sample| [mono_sample, mono_sample])
                    .collect();
                Ok(stereo_samples)
            }
            (from, to) if from == to => Ok(samples), // No conversion needed
            (from, to) => {
                return Err(AudioError::ProcessingFailed { 
                    message: format!("Unsupported channel conversion: {} -> {}", from, to) 
                });
            }
        }
    }

    /// Simple linear interpolation resampling
    fn resample_audio(
        &self,
        samples: Vec<f32>,
        from_rate: u32,
        to_rate: u32,
        channels: u8,
    ) -> Result<Vec<f32>, AudioError> {
        if from_rate == to_rate {
            return Ok(samples);
        }

        let ratio = to_rate as f64 / from_rate as f64;
        let input_frames = samples.len() / channels as usize;
        let output_frames = (input_frames as f64 * ratio) as usize;
        let mut resampled = Vec::with_capacity(output_frames * channels as usize);

        for output_frame in 0..output_frames {
            let input_position = output_frame as f64 / ratio;
            let input_frame = input_position.floor() as usize;
            let fraction = input_position.fract() as f32;

            for channel in 0..channels as usize {
                let current_idx = input_frame * channels as usize + channel;
                let next_idx = ((input_frame + 1) * channels as usize + channel).min(samples.len() - 1);

                if current_idx < samples.len() {
                    let current_sample = samples[current_idx];
                    let next_sample = if next_idx < samples.len() { 
                        samples[next_idx] 
                    } else { 
                        current_sample 
                    };

                    // Linear interpolation
                    let interpolated = current_sample + (next_sample - current_sample) * fraction;
                    resampled.push(interpolated);
                } else {
                    resampled.push(0.0);
                }
            }
        }

        debug!("Resampled audio: {}Hz -> {}Hz ({} -> {} frames)",
               from_rate, to_rate, input_frames, output_frames);

        Ok(resampled)
    }

    /// Create audio sender channel for streaming chunks
    pub fn create_audio_channel(&mut self) -> mpsc::Receiver<AudioData> {
        let (sender, receiver) = mpsc::channel(self.config.buffer_size);
        self.audio_sender = Some(sender);
        receiver
    }

    /// Start real-time audio streaming
    pub async fn start_playback(&mut self) -> Result<(), AudioError> {
        // Check if audio is loaded
        let audio_len = {
            let audio_guard = self.audio_samples.read().unwrap();
            audio_guard.len()
        };

        if audio_len == 0 {
            return Err(AudioError::ProcessingFailed { 
                message: "No audio loaded. Call load_audio_file() first.".to_string() 
            });
        }

        // Check if sender is available
        let sender = self.audio_sender.as_ref().ok_or_else(|| {
            AudioError::ProcessingFailed { 
                message: "Audio channel not created. Call create_audio_channel() first.".to_string() 
            }
        })?.clone();

        // Update state to playing
        {
            let mut state_guard = self.state.write().unwrap();
            *state_guard = PlaybackState::Playing;
        }

        // Initialize metrics
        if self.config.enable_metrics {
            let mut metrics_guard = self.metrics.write().unwrap();
            *metrics_guard = PlaybackMetrics::default();
            metrics_guard.playback_start_time = Some(Instant::now());
        }

        info!("Starting real-time audio playback ({}ms chunks)", self.config.chunk_duration_ms);

        // Spawn the streaming task
        let config = self.config.clone();
        let audio_samples = Arc::clone(&self.audio_samples);
        let state = Arc::clone(&self.state);
        let position = Arc::clone(&self.position);
        let metrics = Arc::clone(&self.metrics);

        tokio::spawn(async move {
            Self::stream_audio_chunks(
                config,
                audio_samples,
                state,
                position,
                metrics,
                sender,
            ).await;
        });

        Ok(())
    }

    /// Internal method to stream audio chunks in real-time
    async fn stream_audio_chunks(
        config: AudioPlaybackConfig,
        audio_samples: Arc<RwLock<Vec<f32>>>,
        state: Arc<RwLock<PlaybackState>>,
        position: Arc<RwLock<usize>>,
        metrics: Arc<RwLock<PlaybackMetrics>>,
        sender: mpsc::Sender<AudioData>,
    ) {
        let chunk_duration = Duration::from_millis(config.chunk_duration_ms);
        let adjusted_interval = if config.speed_multiplier > 0.0 {
            Duration::from_millis((config.chunk_duration_ms as f32 / config.speed_multiplier) as u64)
        } else {
            chunk_duration
        };

        let mut timer = interval(adjusted_interval);
        let samples_per_chunk = (config.target_sample_rate as u64 * config.chunk_duration_ms / 1000) as usize * config.target_channels as usize;

        loop {
            timer.tick().await;

            // Check current state
            let current_state = {
                let state_guard = state.read().unwrap();
                *state_guard
            };

            match current_state {
                PlaybackState::Stopped => {
                    debug!("Playback stopped, ending stream");
                    break;
                }
                PlaybackState::Paused => {
                    debug!("Playback paused, waiting...");
                    sleep(Duration::from_millis(10)).await;
                    continue;
                }
                PlaybackState::Error | PlaybackState::Finished => {
                    debug!("Playback finished or error, ending stream");
                    break;
                }
                PlaybackState::Playing => {
                    // Continue with chunk processing
                }
            }

            let chunk_start_time = Instant::now();

            // Get current position and audio samples
            let (current_pos, audio_data, total_samples) = {
                let pos_guard = position.read().unwrap();
                let audio_guard = audio_samples.read().unwrap();
                (*pos_guard, audio_guard.clone(), audio_guard.len())
            };

            // Check if we've reached the end
            if current_pos >= total_samples {
                if config.loop_playback {
                    // Reset position for looping
                    {
                        let mut pos_guard = position.write().unwrap();
                        *pos_guard = 0;
                    }

                    // Inject silence between loops if configured
                    if config.silence_between_loops_ms > 0 {
                        let silence_samples = vec![0.0f32; samples_per_chunk];
                        let silence_chunk = AudioData {
                            samples: silence_samples,
                            sample_rate: config.target_sample_rate,
                            channels: config.target_channels,
                            timestamp: SystemTime::now(),
                            source_channel: AudioSource::File,
                            duration_seconds: config.chunk_duration_ms as f32 / 1000.0,
                        };

                        if let Err(e) = sender.try_send(silence_chunk) {
                            warn!("Failed to send silence chunk: {}", e);
                        }

                        // Sleep for silence duration
                        sleep(Duration::from_millis(config.silence_between_loops_ms)).await;
                    }
                    continue;
                } else {
                    // Mark as finished
                    {
                        let mut state_guard = state.write().unwrap();
                        *state_guard = PlaybackState::Finished;
                    }
                    info!("Audio playback finished");
                    break;
                }
            }

            // Extract chunk
            let end_pos = (current_pos + samples_per_chunk).min(total_samples);
            let chunk_samples = if end_pos > current_pos {
                audio_data[current_pos..end_pos].to_vec()
            } else {
                vec![0.0f32; samples_per_chunk] // Silence if no data
            };

            // Pad with silence if chunk is shorter than expected
            let mut final_chunk = chunk_samples;
            while final_chunk.len() < samples_per_chunk {
                final_chunk.push(0.0);
            }

            // Create AudioData chunk
            let audio_chunk = AudioData {
                samples: final_chunk,
                sample_rate: config.target_sample_rate,
                channels: config.target_channels,
                timestamp: SystemTime::now(),
                source_channel: AudioSource::File,
                duration_seconds: config.chunk_duration_ms as f32 / 1000.0,
            };

            // Send chunk
            if let Err(e) = sender.try_send(audio_chunk) {
                warn!("Failed to send audio chunk: {}. Receiver may be full or disconnected.", e);
                
                // Check if receiver is disconnected
                if sender.is_closed() {
                    info!("Audio receiver disconnected, stopping playback");
                    {
                        let mut state_guard = state.write().unwrap();
                        *state_guard = PlaybackState::Stopped;
                    }
                    break;
                }
            }

            // Update position
            {
                let mut pos_guard = position.write().unwrap();
                *pos_guard = end_pos;
            }

            // Update metrics
            if config.enable_metrics {
                let processing_time = chunk_start_time.elapsed();
                let mut metrics_guard = metrics.write().unwrap();
                
                metrics_guard.chunks_sent += 1;
                metrics_guard.total_duration_processed += config.chunk_duration_ms as f64 / 1000.0;
                metrics_guard.current_position = current_pos as f64 / (config.target_sample_rate as f64 * config.target_channels as f64);
                
                let processing_time_us = processing_time.as_micros() as u64;
                if processing_time_us > metrics_guard.max_chunk_processing_time_us {
                    metrics_guard.max_chunk_processing_time_us = processing_time_us;
                }

                // Update average processing time
                let total_processing_time = metrics_guard.avg_chunk_processing_time_us * (metrics_guard.chunks_sent - 1) as f64;
                metrics_guard.avg_chunk_processing_time_us = 
                    (total_processing_time + processing_time_us as f64) / metrics_guard.chunks_sent as f64;

                // Check for timing violations
                if processing_time > chunk_duration {
                    metrics_guard.timing_violations += 1;
                    debug!("Timing violation: chunk took {}ms to process", processing_time.as_millis());
                }

                if let Some(start_time) = metrics_guard.playback_start_time {
                    metrics_guard.total_playback_time = start_time.elapsed();
                }
            }
        }

        info!("Audio streaming completed");
    }

    /// Pause playback (can be resumed)
    pub async fn pause(&mut self) -> Result<(), AudioError> {
        let mut state_guard = self.state.write().unwrap();
        match *state_guard {
            PlaybackState::Playing => {
                *state_guard = PlaybackState::Paused;
                info!("Audio playback paused");
                Ok(())
            }
            _ => {
                warn!("Cannot pause - playback not in playing state");
                Err(AudioError::ProcessingFailed { 
                    message: "Playback is not currently playing".to_string() 
                })
            }
        }
    }

    /// Resume paused playback
    pub async fn resume(&mut self) -> Result<(), AudioError> {
        let mut state_guard = self.state.write().unwrap();
        match *state_guard {
            PlaybackState::Paused => {
                *state_guard = PlaybackState::Playing;
                info!("Audio playback resumed");
                Ok(())
            }
            _ => {
                warn!("Cannot resume - playback not in paused state");
                Err(AudioError::ProcessingFailed { 
                    message: "Playback is not currently paused".to_string() 
                })
            }
        }
    }

    /// Stop playback completely
    pub async fn stop(&mut self) -> Result<(), AudioError> {
        let mut state_guard = self.state.write().unwrap();
        *state_guard = PlaybackState::Stopped;

        // Reset position
        {
            let mut pos_guard = self.position.write().unwrap();
            *pos_guard = 0;
        }

        info!("Audio playback stopped");
        Ok(())
    }

    /// Get current playback state
    pub fn get_state(&self) -> PlaybackState {
        let state_guard = self.state.read().unwrap();
        *state_guard
    }

    /// Get current playback position in seconds
    pub fn get_position_seconds(&self) -> f64 {
        let pos_guard = self.position.read().unwrap();
        let sample_position = *pos_guard;
        
        if self.sample_rate > 0 && self.channels > 0 {
            sample_position as f64 / (self.sample_rate as f64 * self.channels as f64)
        } else {
            0.0
        }
    }

    /// Get total duration of loaded audio in seconds
    pub fn get_total_duration_seconds(&self) -> f64 {
        let audio_guard = self.audio_samples.read().unwrap();
        let total_samples = audio_guard.len();
        
        if self.sample_rate > 0 && self.channels > 0 {
            total_samples as f64 / (self.sample_rate as f64 * self.channels as f64)
        } else {
            0.0
        }
    }

    /// Get current performance metrics
    pub fn get_metrics(&self) -> PlaybackMetrics {
        let metrics_guard = self.metrics.read().unwrap();
        metrics_guard.clone()
    }

    /// Seek to a specific position in seconds
    pub async fn seek_to_position(&mut self, position_seconds: f64) -> Result<(), AudioError> {
        let total_duration = self.get_total_duration_seconds();
        
        if position_seconds < 0.0 || position_seconds > total_duration {
            return Err(AudioError::ProcessingFailed { 
                message: format!("Invalid seek position: {}s (valid range: 0.0 - {:.2}s)", 
                                position_seconds, total_duration) 
            });
        }

        let sample_position = (position_seconds * self.sample_rate as f64 * self.channels as f64) as usize;
        
        {
            let mut pos_guard = self.position.write().unwrap();
            *pos_guard = sample_position;
        }

        info!("Seeked to position: {:.2}s", position_seconds);
        Ok(())
    }

    /// Check if the simulator is ready for playback
    pub fn is_ready(&self) -> bool {
        let audio_guard = self.audio_samples.read().unwrap();
        !audio_guard.is_empty() && self.sample_rate > 0
    }

    /// Get audio file information
    pub fn get_audio_info(&self) -> (u32, u8, usize) {
        let audio_guard = self.audio_samples.read().unwrap();
        (self.sample_rate, self.channels, audio_guard.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    /// Create a simple test WAV file
    async fn create_test_wav_file() -> Result<NamedTempFile, Box<dyn std::error::Error>> {
        let mut temp_file = NamedTempFile::new()?;
        
        // Create a simple WAV file with a sine wave
        let sample_rate = 44100u32;
        let duration_seconds = 2.0;
        let frequency = 440.0; // A4 note
        let samples_count = (sample_rate as f64 * duration_seconds) as usize;

        // WAV header
        temp_file.write_all(b"RIFF")?;
        temp_file.write_all(&(36 + samples_count * 2).to_le_bytes())?; // File size - 8
        temp_file.write_all(b"WAVE")?;
        temp_file.write_all(b"fmt ")?;
        temp_file.write_all(&16u32.to_le_bytes())?; // PCM header size
        temp_file.write_all(&1u16.to_le_bytes())?; // PCM format
        temp_file.write_all(&1u16.to_le_bytes())?; // Mono
        temp_file.write_all(&sample_rate.to_le_bytes())?; // Sample rate
        temp_file.write_all(&(sample_rate * 2).to_le_bytes())?; // Byte rate
        temp_file.write_all(&2u16.to_le_bytes())?; // Block align
        temp_file.write_all(&16u16.to_le_bytes())?; // Bits per sample
        temp_file.write_all(b"data")?;
        temp_file.write_all(&(samples_count * 2).to_le_bytes())?; // Data size

        // Generate sine wave
        for i in 0..samples_count {
            let t = i as f64 / sample_rate as f64;
            let sample = (2.0 * std::f64::consts::PI * frequency * t).sin();
            let sample_i16 = (sample * i16::MAX as f64) as i16;
            temp_file.write_all(&sample_i16.to_le_bytes())?;
        }

        temp_file.flush()?;
        // Ensure the file is fully written by syncing
        temp_file.as_file().sync_all()?;
        Ok(temp_file)
    }

    #[tokio::test]
    async fn test_load_audio_file() {
        let temp_wav = create_test_wav_file().await.expect("Failed to create test WAV");
        
        let config = AudioPlaybackConfig::default();
        let mut simulator = AudioPlaybackSimulator::new(config);
        
        let result = simulator.load_audio_file(temp_wav.path()).await;
        assert!(result.is_ok(), "Failed to load audio file: {:?}", result);
        assert!(simulator.is_ready());
        
        let total_duration = simulator.get_total_duration_seconds();
        assert!(total_duration > 1.8 && total_duration < 2.2, 
                "Expected ~2 seconds duration, got {}", total_duration);
    }

    #[tokio::test]
    async fn test_playback_control() {
        let temp_wav = create_test_wav_file().await.expect("Failed to create test WAV");
        
        let config = AudioPlaybackConfig::default();
        let mut simulator = AudioPlaybackSimulator::new(config);
        
        simulator.load_audio_file(temp_wav.path()).await.unwrap();
        let _receiver = simulator.create_audio_channel();
        
        // Test initial state
        assert_eq!(simulator.get_state(), PlaybackState::Stopped);
        
        // Start playback
        simulator.start_playback().await.unwrap();
        
        // Wait a bit for the stream to start
        tokio::time::sleep(Duration::from_millis(50)).await;
        assert_eq!(simulator.get_state(), PlaybackState::Playing);
        
        // Pause
        simulator.pause().await.unwrap();
        assert_eq!(simulator.get_state(), PlaybackState::Paused);
        
        // Resume
        simulator.resume().await.unwrap();
        assert_eq!(simulator.get_state(), PlaybackState::Playing);
        
        // Stop
        simulator.stop().await.unwrap();
        assert_eq!(simulator.get_state(), PlaybackState::Stopped);
    }

    #[tokio::test]
    async fn test_audio_streaming() {
        let temp_wav = create_test_wav_file().await.expect("Failed to create test WAV");
        
        let config = AudioPlaybackConfig {
            chunk_duration_ms: 50, // 50ms chunks for faster testing
            ..Default::default()
        };
        let mut simulator = AudioPlaybackSimulator::new(config);
        
        simulator.load_audio_file(temp_wav.path()).await.unwrap();
        let mut receiver = simulator.create_audio_channel();
        
        simulator.start_playback().await.unwrap();
        
        // Receive a few chunks
        let mut chunks_received = 0;
        let max_chunks = 10;
        
        while chunks_received < max_chunks {
            match tokio::time::timeout(Duration::from_millis(200), receiver.recv()).await {
                Ok(Some(audio_data)) => {
                    assert_eq!(audio_data.sample_rate, 16000);
                    assert_eq!(audio_data.channels, 1);
                    assert!(!audio_data.samples.is_empty());
                    assert_eq!(audio_data.source_channel, AudioSource::File);
                    chunks_received += 1;
                }
                Ok(None) => {
                    break; // Channel closed
                }
                Err(_) => {
                    panic!("Timeout waiting for audio chunk after receiving {} chunks", chunks_received);
                }
            }
        }
        
        simulator.stop().await.unwrap();
        assert!(chunks_received > 0, "Should have received at least one chunk");
    }

    #[tokio::test]
    async fn test_metrics_collection() {
        let temp_wav = create_test_wav_file().await.expect("Failed to create test WAV");
        
        let config = AudioPlaybackConfig {
            chunk_duration_ms: 50,
            enable_metrics: true,
            ..Default::default()
        };
        let mut simulator = AudioPlaybackSimulator::new(config);
        
        simulator.load_audio_file(temp_wav.path()).await.unwrap();
        let _receiver = simulator.create_audio_channel();
        
        simulator.start_playback().await.unwrap();
        
        // Let it play for a bit
        tokio::time::sleep(Duration::from_millis(300)).await;
        
        let metrics = simulator.get_metrics();
        assert!(metrics.chunks_sent > 0);
        assert!(metrics.total_duration_processed > 0.0);
        assert!(metrics.real_time_factor() > 0.0);
        
        simulator.stop().await.unwrap();
    }

    #[tokio::test]
    async fn test_seek_functionality() {
        let temp_wav = create_test_wav_file().await.expect("Failed to create test WAV");
        
        let config = AudioPlaybackConfig::default();
        let mut simulator = AudioPlaybackSimulator::new(config);
        
        simulator.load_audio_file(temp_wav.path()).await.unwrap();
        
        // Test seeking to middle
        simulator.seek_to_position(1.0).await.unwrap();
        let position = simulator.get_position_seconds();
        assert!((position - 1.0).abs() < 0.1, "Seek position incorrect: {}", position);
        
        // Test invalid seek positions
        assert!(simulator.seek_to_position(-1.0).await.is_err());
        assert!(simulator.seek_to_position(10.0).await.is_err());
    }
}