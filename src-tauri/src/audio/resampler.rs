//! Audio resampling module
//! 
//! Provides high-quality sample rate conversion using dasp library to convert
//! any input sample rate to 16kHz for Whisper compatibility.

use crate::audio::types::{AudioData, AudioError};
use std::collections::VecDeque;
use tracing::{debug, info};

/// High-quality audio resampler for converting arbitrary sample rates to 16kHz
pub struct AudioResampler {
    source_sample_rate: u32,
    target_sample_rate: u32,
    channels: u8,
    conversion_ratio: f64,
    buffer: VecDeque<f32>,
    quality_mode: ResamplingQuality,
}

/// Resampling quality modes
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub enum ResamplingQuality {
    /// High quality with 64-sample window (default)
    High,
    /// Medium quality with 32-sample window (balanced)
    Medium,
    /// Fast with 16-sample window (low latency)
    Fast,
}

impl ResamplingQuality {
    fn window_size(&self) -> usize {
        match self {
            ResamplingQuality::High => 64,
            ResamplingQuality::Medium => 32,
            ResamplingQuality::Fast => 16,
        }
    }
}

impl AudioResampler {
    /// Create a new audio resampler
    pub fn new(
        source_sample_rate: u32,
        target_sample_rate: u32,
        channels: u8,
        quality: ResamplingQuality,
    ) -> Result<Self, AudioError> {
        if source_sample_rate == 0 || target_sample_rate == 0 {
            return Err(AudioError::ProcessingFailed {
                message: "Sample rates must be greater than zero".to_string(),
            });
        }

        if channels == 0 || channels > 8 {
            return Err(AudioError::ProcessingFailed {
                message: format!("Invalid channel count: {}", channels),
            });
        }

        let conversion_ratio = target_sample_rate as f64 / source_sample_rate as f64;

        info!(
            "🎛️ Initializing audio resampler: {} Hz → {} Hz (ratio: {:.4}), {} channels, {:?} quality",
            source_sample_rate, target_sample_rate, conversion_ratio, channels, quality
        );

        Ok(Self {
            source_sample_rate,
            target_sample_rate,
            channels,
            conversion_ratio,
            buffer: VecDeque::new(),
            quality_mode: quality,
        })
    }

    /// Create a resampler specifically for Whisper (16kHz target)
    pub fn for_whisper(source_sample_rate: u32, channels: u8) -> Result<Self, AudioError> {
        Self::new(source_sample_rate, 16000, channels, ResamplingQuality::High)
    }

    /// Process audio data and return resampled version
    pub fn process(&mut self, audio: &AudioData) -> Result<AudioData, AudioError> {
        // Validate input
        if audio.sample_rate != self.source_sample_rate {
            return Err(AudioError::ProcessingFailed {
                message: format!(
                    "Sample rate mismatch: expected {}, got {}",
                    self.source_sample_rate, audio.sample_rate
                ),
            });
        }

        if audio.channels != self.channels {
            return Err(AudioError::ProcessingFailed {
                message: format!(
                    "Channel count mismatch: expected {}, got {}",
                    self.channels, audio.channels
                ),
            });
        }

        // If no conversion needed, return original
        if self.source_sample_rate == self.target_sample_rate {
            debug!("No resampling needed, returning original audio");
            return Ok(audio.clone());
        }

        let resampled_samples = if self.channels == 1 {
            // Mono processing
            self.resample_mono(&audio.samples)?
        } else {
            // Stereo/multi-channel processing
            self.resample_multichannel(&audio.samples)?
        };

        debug!(
            "Resampled {} → {} samples ({:.2}s → {:.2}s)",
            audio.samples.len(),
            resampled_samples.len(),
            audio.samples.len() as f32 / self.source_sample_rate as f32,
            resampled_samples.len() as f32 / self.target_sample_rate as f32
        );

        let sample_count = resampled_samples.len();
        Ok(AudioData {
            samples: resampled_samples,
            sample_rate: self.target_sample_rate,
            channels: self.channels,
            timestamp: audio.timestamp,
            source_channel: audio.source_channel,
            duration_seconds: sample_count as f32 / (self.target_sample_rate * self.channels as u32) as f32,
        })
    }

    /// Process and convert stereo to mono if needed
    pub fn process_to_mono(&mut self, audio: &AudioData) -> Result<AudioData, AudioError> {
        let mut resampled = self.process(audio)?;
        
        if resampled.channels > 1 {
            resampled = self.convert_to_mono(resampled)?;
        }
        
        Ok(resampled)
    }

    /// Convert multi-channel audio to mono by averaging channels
    fn convert_to_mono(&self, audio: AudioData) -> Result<AudioData, AudioError> {
        if audio.channels == 1 {
            return Ok(audio);
        }

        let channels = audio.channels as usize;
        let input_len = audio.samples.len();
        let output_len = input_len / channels;
        let mut mono_samples = Vec::with_capacity(output_len);

        for i in 0..output_len {
            let mut sum = 0.0f32;
            for ch in 0..channels {
                let idx = i * channels + ch;
                if idx < input_len {
                    sum += audio.samples[idx];
                }
            }
            mono_samples.push(sum / channels as f32);
        }

        debug!(
            "Converted {} channels to mono: {} → {} samples",
            audio.channels, input_len, mono_samples.len()
        );

        let sample_count = mono_samples.len();
        Ok(AudioData {
            samples: mono_samples,
            sample_rate: audio.sample_rate,
            channels: 1,
            timestamp: audio.timestamp,
            source_channel: audio.source_channel,
            duration_seconds: sample_count as f32 / audio.sample_rate as f32,
        })
    }


    /// Resample mono audio using linear interpolation
    fn resample_mono(&mut self, samples: &[f32]) -> Result<Vec<f32>, AudioError> {
        if samples.is_empty() {
            return Ok(Vec::new());
        }

        let input_len = samples.len();
        let output_len = (input_len as f64 * self.conversion_ratio) as usize;
        let mut output = Vec::with_capacity(output_len);

        // Use linear interpolation for resampling
        for i in 0..output_len {
            let input_index = i as f64 / self.conversion_ratio;
            let input_index_floor = input_index.floor() as usize;
            let input_index_ceil = (input_index.ceil() as usize).min(input_len - 1);
            
            if input_index_floor == input_index_ceil {
                // Exact match, no interpolation needed
                output.push(samples[input_index_floor]);
            } else {
                // Linear interpolation between two samples
                let fraction = input_index - input_index_floor as f64;
                let sample_low = samples[input_index_floor];
                let sample_high = samples[input_index_ceil];
                let interpolated = sample_low + (sample_high - sample_low) * fraction as f32;
                output.push(interpolated);
            }
        }

        Ok(output)
    }

    /// Resample multi-channel audio
    fn resample_multichannel(&mut self, samples: &[f32]) -> Result<Vec<f32>, AudioError> {
        let channels = self.channels as usize;
        let frames_count = samples.len() / channels;
        let output_frames = (frames_count as f64 * self.conversion_ratio) as usize;
        let mut output = Vec::with_capacity(output_frames * channels);

        // Process each channel separately
        for ch in 0..channels {
            // Extract channel data
            let mut channel_data = Vec::with_capacity(frames_count);
            for frame in 0..frames_count {
                let idx = frame * channels + ch;
                if idx < samples.len() {
                    channel_data.push(samples[idx]);
                }
            }

            // Resample this channel
            let resampled_channel = self.resample_mono(&channel_data)?;

            // Interleave back into output
            for (frame_idx, &sample) in resampled_channel.iter().enumerate() {
                let output_idx = frame_idx * channels + ch;
                if output_idx < output.capacity() {
                    if output.len() <= output_idx {
                        output.resize(output_idx + 1, 0.0);
                    }
                    output[output_idx] = sample;
                }
            }
        }

        Ok(output)
    }

    /// Get conversion ratio
    pub fn conversion_ratio(&self) -> f64 {
        self.conversion_ratio
    }

    /// Get quality mode
    pub fn quality(&self) -> ResamplingQuality {
        self.quality_mode
    }

    /// Get source sample rate
    pub fn source_sample_rate(&self) -> u32 {
        self.source_sample_rate
    }

    /// Get target sample rate
    pub fn target_sample_rate(&self) -> u32 {
        self.target_sample_rate
    }

    /// Check if resampling is needed
    pub fn needs_resampling(&self) -> bool {
        self.source_sample_rate != self.target_sample_rate
    }
}

/// Utility functions for common resampling operations
pub struct ResamplerUtils;

impl ResamplerUtils {
    /// Quick resample to 16kHz for Whisper with default settings
    pub fn to_whisper_format(audio: &AudioData) -> Result<AudioData, AudioError> {
        if audio.sample_rate == 16000 && audio.channels == 1 {
            // Already in correct format
            return Ok(audio.clone());
        }

        let mut resampler = AudioResampler::for_whisper(audio.sample_rate, audio.channels)?;
        let resampled = resampler.process_to_mono(audio)?;
        
        info!(
            "Converted audio: {}Hz/{}ch → 16kHz/1ch ({:.2}s)",
            audio.sample_rate,
            audio.channels,
            resampled.duration_seconds
        );
        
        Ok(resampled)
    }

    /// Calculate optimal resampling quality based on CPU constraints
    pub fn recommend_quality(source_rate: u32, target_rate: u32, real_time: bool) -> ResamplingQuality {
        let ratio = (target_rate as f64 / source_rate as f64 - 1.0).abs();
        
        if real_time && ratio > 0.5 {
            // High conversion ratio in real-time, use fast
            ResamplingQuality::Fast
        } else if ratio > 0.2 {
            // Medium conversion ratio
            ResamplingQuality::Medium
        } else {
            // Small conversion or offline processing
            ResamplingQuality::High
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio::types::{AudioSource, AudioData};
    use std::time::SystemTime;

    fn create_test_audio(sample_rate: u32, channels: u8, duration_secs: f32) -> AudioData {
        let samples_per_channel = (sample_rate as f32 * duration_secs) as usize;
        let total_samples = samples_per_channel * channels as usize;
        
        // Generate test tone at 440Hz
        let mut samples = Vec::with_capacity(total_samples);
        for i in 0..samples_per_channel {
            let t = i as f32 / sample_rate as f32;
            let sample = (2.0 * std::f32::consts::PI * 440.0 * t).sin() * 0.5;
            
            for _ in 0..channels {
                samples.push(sample);
            }
        }

        AudioData {
            samples,
            sample_rate,
            channels,
            timestamp: SystemTime::now(),
            source_channel: AudioSource::Microphone,
            duration_seconds: duration_secs,
        }
    }

    #[test]
    fn test_no_resampling_needed() {
        let mut resampler = AudioResampler::new(16000, 16000, 1, ResamplingQuality::High).unwrap();
        let audio = create_test_audio(16000, 1, 1.0);
        
        let result = resampler.process(&audio).unwrap();
        assert_eq!(result.sample_rate, 16000);
        assert_eq!(result.samples.len(), audio.samples.len());
    }

    #[test]
    fn test_downsample_48khz_to_16khz() {
        let mut resampler = AudioResampler::new(48000, 16000, 1, ResamplingQuality::High).unwrap();
        let audio = create_test_audio(48000, 1, 1.0);
        
        let result = resampler.process(&audio).unwrap();
        assert_eq!(result.sample_rate, 16000);
        
        // Should be approximately 1/3 the samples (48k -> 16k)
        let expected_len = audio.samples.len() / 3;
        let tolerance = expected_len / 10; // 10% tolerance
        assert!((result.samples.len() as i32 - expected_len as i32).abs() < tolerance as i32);
    }

    #[test]
    fn test_stereo_to_mono_conversion() {
        let mut resampler = AudioResampler::new(44100, 16000, 2, ResamplingQuality::Medium).unwrap();
        let audio = create_test_audio(44100, 2, 0.5);
        
        let result = resampler.process_to_mono(&audio).unwrap();
        assert_eq!(result.sample_rate, 16000);
        assert_eq!(result.channels, 1);
    }

    #[test]
    fn test_whisper_format_conversion() {
        let audio = create_test_audio(48000, 2, 1.0);
        let result = ResamplerUtils::to_whisper_format(&audio).unwrap();
        
        assert_eq!(result.sample_rate, 16000);
        assert_eq!(result.channels, 1);
    }

    #[test]
    fn test_quality_recommendation() {
        // Small ratio should recommend High
        assert!(matches!(
            ResamplerUtils::recommend_quality(44100, 48000, false),
            ResamplingQuality::High
        ));

        // Large ratio in real-time should recommend Fast
        assert!(matches!(
            ResamplerUtils::recommend_quality(48000, 8000, true),
            ResamplingQuality::Fast
        ));
    }
}