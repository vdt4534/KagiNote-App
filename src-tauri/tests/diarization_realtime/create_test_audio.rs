use std::path::{Path, PathBuf};
use std::fs;
use std::collections::HashMap;
use hound::{WavWriter, WavSpec, SampleFormat};
use rand::Rng;
use serde_json;

use super::test_scenarios::{TestScenarioGenerator, GroundTruthData, GroundTruthSegment};

/// Advanced synthetic audio generator for realistic speaker diarization testing
pub struct TestAudioGenerator {
    pub output_dir: PathBuf,
    pub sample_rate: u32,
    pub noise_level: f32,
}

impl TestAudioGenerator {
    /// Create a new test audio generator
    pub fn new<P: AsRef<Path>>(output_dir: P, sample_rate: u32) -> Self {
        let output_dir = output_dir.as_ref().to_path_buf();
        fs::create_dir_all(&output_dir).unwrap_or_else(|e| {
            eprintln!("Warning: Failed to create output directory: {}", e);
        });

        Self {
            output_dir,
            sample_rate,
            noise_level: 0.02, // 2% background noise
        }
    }

    /// Generate all standard test audio files
    pub fn generate_all_test_audio(&self) -> Result<Vec<GeneratedAudioFile>, Box<dyn std::error::Error>> {
        println!("🎵 Generating comprehensive test audio files...");
        
        let scenarios = vec![
            ("2speaker_conversation", TestScenarioGenerator::simple_two_speaker_conversation()),
            ("3speaker_meeting", TestScenarioGenerator::multi_speaker_meeting()),
            ("overlapping_speech", TestScenarioGenerator::overlapping_speech_scenario()),
            ("rapid_switching", TestScenarioGenerator::rapid_speaker_switching()),
            ("long_silences", TestScenarioGenerator::long_silences_scenario()),
            ("single_speaker", TestScenarioGenerator::single_speaker_monologue()),
        ];

        let mut generated_files = Vec::new();

        for (name, ground_truth) in scenarios {
            println!("  📁 Generating '{}' - {} speakers, {:.1}s duration", 
                name, ground_truth.total_speakers, ground_truth.duration);
            
            let audio_file = self.generate_scenario_audio(&ground_truth, name)?;
            let ground_truth_file = self.save_ground_truth(&ground_truth, name)?;
            
            generated_files.push(GeneratedAudioFile {
                name: name.to_string(),
                audio_path: audio_file,
                ground_truth_path: ground_truth_file,
                duration: ground_truth.duration,
                num_speakers: ground_truth.total_speakers,
                scenario_type: ground_truth.metadata.get("scenario_type").cloned()
                    .unwrap_or_else(|| "unknown".to_string()),
            });
        }

        // Generate challenging synthetic scenarios
        generated_files.extend(self.generate_challenging_scenarios()?);

        println!("✅ Generated {} test audio files successfully", generated_files.len());
        self.print_generation_summary(&generated_files);

        Ok(generated_files)
    }

    /// Generate audio for a specific ground truth scenario
    pub fn generate_scenario_audio(&self, ground_truth: &GroundTruthData, name: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let audio_path = self.output_dir.join(format!("{}.wav", name));
        
        // Generate multi-frequency synthetic audio
        let audio_data = self.generate_multi_frequency_audio(ground_truth)?;
        
        // Apply realistic voice processing
        let processed_audio = self.apply_voice_processing(audio_data)?;
        
        // Save as WAV file
        self.save_wav_file(&audio_path, &processed_audio)?;
        
        Ok(audio_path)
    }

    /// Generate advanced multi-frequency audio with realistic voice characteristics
    fn generate_multi_frequency_audio(&self, ground_truth: &GroundTruthData) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
        let total_samples = (ground_truth.duration * self.sample_rate as f32) as usize;
        let mut audio_buffer = vec![0.0f32; total_samples];
        
        // Enhanced speaker frequency mapping - more realistic voice ranges
        let speaker_frequencies = vec![
            110.0,  // Bass voice (Male)
            130.0,  // Baritone (Male)
            160.0,  // Tenor (Male)
            220.0,  // Alto (Female)
            260.0,  // Soprano (Female)
            300.0,  // High Soprano (Female)
            350.0,  // Child voice
            400.0,  // High pitch
        ];
        
        let mut speaker_freq_map = HashMap::new();
        let speaker_ids = ground_truth.speaker_ids();
        
        for (i, speaker_id) in speaker_ids.iter().enumerate() {
            let base_freq = speaker_frequencies[i % speaker_frequencies.len()];
            speaker_freq_map.insert(speaker_id.clone(), base_freq);
        }

        // Generate audio for each segment with realistic voice synthesis
        for segment in &ground_truth.segments {
            let start_sample = (segment.start_time * self.sample_rate as f32) as usize;
            let end_sample = (segment.end_time * self.sample_rate as f32) as usize;
            let base_frequency = speaker_freq_map.get(&segment.speaker_id).unwrap_or(&220.0);
            
            self.synthesize_voice_segment(
                &mut audio_buffer,
                start_sample,
                end_sample.min(total_samples),
                *base_frequency,
                segment.confidence,
            )?;
        }
        
        Ok(audio_buffer)
    }

    /// Synthesize realistic voice segment with harmonics and modulation
    fn synthesize_voice_segment(
        &self,
        audio_buffer: &mut [f32],
        start_sample: usize,
        end_sample: usize,
        base_frequency: f32,
        confidence: f32,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut rng = rand::thread_rng();
        let segment_duration = (end_sample - start_sample) as f32 / self.sample_rate as f32;
        
        for i in start_sample..end_sample {
            let t = (i - start_sample) as f32 / self.sample_rate as f32;
            let global_t = i as f32 / self.sample_rate as f32;
            
            // Base amplitude with envelope
            let envelope = self.create_voice_envelope(t, segment_duration);
            let base_amplitude = 0.25 * confidence * envelope;
            
            // Fundamental frequency with slight vibrato
            let vibrato = 1.0 + 0.02 * (5.0 * 2.0 * std::f32::consts::PI * global_t).sin();
            let fundamental_freq = base_frequency * vibrato;
            
            // Generate harmonic series for more realistic voice
            let mut sample = 0.0f32;
            
            // Fundamental (strongest component)
            sample += base_amplitude * (2.0 * std::f32::consts::PI * fundamental_freq * global_t).sin();
            
            // Second harmonic (60% strength)
            sample += base_amplitude * 0.6 * (2.0 * std::f32::consts::PI * fundamental_freq * 2.0 * global_t).sin();
            
            // Third harmonic (30% strength) 
            sample += base_amplitude * 0.3 * (2.0 * std::f32::consts::PI * fundamental_freq * 3.0 * global_t).sin();
            
            // Fourth harmonic (15% strength)
            sample += base_amplitude * 0.15 * (2.0 * std::f32::consts::PI * fundamental_freq * 4.0 * global_t).sin();
            
            // Add slight formant filtering effect
            let formant_filter = 0.8 + 0.2 * (2.0 * std::f32::consts::PI * 800.0 * global_t).sin();
            sample *= formant_filter;
            
            // Add micro-variations for naturalness
            let micro_variation = rng.gen_range(-0.05..0.05);
            sample += base_amplitude * micro_variation;
            
            audio_buffer[i] += sample;
        }
        
        Ok(())
    }

    /// Create realistic voice envelope (attack, sustain, decay)
    fn create_voice_envelope(&self, t: f32, duration: f32) -> f32 {
        let attack_time = 0.1f32.min(duration * 0.1);  // 10% of segment or 100ms max
        let decay_time = 0.1f32.min(duration * 0.1);   // 10% of segment or 100ms max
        
        if t < attack_time {
            // Attack phase - gradual increase
            t / attack_time
        } else if t > duration - decay_time {
            // Decay phase - gradual decrease
            (duration - t) / decay_time
        } else {
            // Sustain phase - full amplitude with slight variation
            0.9 + 0.1 * (10.0 * 2.0 * std::f32::consts::PI * t).sin()
        }
    }

    /// Apply comprehensive voice processing pipeline
    fn apply_voice_processing(&self, mut audio: Vec<f32>) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
        // 1. Apply voice-like filtering
        self.apply_voice_filter(&mut audio);
        
        // 2. Add realistic background noise
        self.add_realistic_background_noise(&mut audio);
        
        // 3. Apply subtle compression (voice characteristic)
        self.apply_compression(&mut audio, 0.7, 0.8);
        
        // 4. Final normalization
        self.normalize_audio(&mut audio);
        
        Ok(audio)
    }

    /// Apply voice-like filtering to simulate human vocal characteristics
    fn apply_voice_filter(&self, audio: &mut [f32]) {
        // Simple bandpass filter for human voice range (85Hz - 8kHz)
        let mut prev_sample = 0.0f32;
        let mut prev_prev_sample = 0.0f32;
        
        // Low-pass filter (removes high frequency noise above ~8kHz)
        let low_pass_alpha = 0.85;
        
        // High-pass filter (removes low frequency rumble below ~85Hz) 
        let high_pass_alpha = 0.95;
        
        for sample in audio.iter_mut() {
            // Low-pass filtering
            *sample = low_pass_alpha * (*sample) + (1.0 - low_pass_alpha) * prev_sample;
            
            // High-pass filtering
            let high_pass_output = high_pass_alpha * (prev_sample - prev_prev_sample) + high_pass_alpha * (*sample);
            *sample = high_pass_output;
            
            prev_prev_sample = prev_sample;
            prev_sample = *sample;
        }
    }

    /// Add realistic background noise (room tone, HVAC, etc.)
    fn add_realistic_background_noise(&self, audio: &mut [f32]) {
        let mut rng = rand::thread_rng();
        
        for (i, sample) in audio.iter_mut().enumerate() {
            // White noise component
            let white_noise = rng.gen_range(-self.noise_level..self.noise_level);
            
            // Pink noise component (more realistic for room environments)
            let pink_noise = rng.gen_range(-self.noise_level * 0.5..self.noise_level * 0.5);
            
            // Very subtle 60Hz hum (electrical interference)
            let time_position = i as f32 / self.sample_rate as f32;
            let hum_component = 0.005 * (2.0 * std::f32::consts::PI * 60.0 * time_position).sin();
            
            *sample += white_noise + pink_noise + hum_component;
            *sample = sample.clamp(-1.0, 1.0); // Prevent clipping
        }
    }

    /// Apply subtle audio compression for voice-like dynamics
    fn apply_compression(&self, audio: &mut [f32], threshold: f32, ratio: f32) {
        for sample in audio.iter_mut() {
            let abs_sample = sample.abs();
            if abs_sample > threshold {
                let excess = abs_sample - threshold;
                let compressed_excess = excess / ratio;
                let new_amplitude = threshold + compressed_excess;
                *sample = (*sample / abs_sample) * new_amplitude;
            }
        }
    }

    /// Normalize audio to optimal levels
    fn normalize_audio(&self, audio: &mut [f32]) {
        let max_amplitude = audio.iter().map(|&s| s.abs()).fold(0.0f32, f32::max);
        
        if max_amplitude > 0.0 {
            let normalization_factor = 0.8 / max_amplitude; // Target 80% of max
            for sample in audio.iter_mut() {
                *sample *= normalization_factor;
            }
        }
    }

    /// Generate challenging edge-case scenarios
    fn generate_challenging_scenarios(&self) -> Result<Vec<GeneratedAudioFile>, Box<dyn std::error::Error>> {
        println!("  🔥 Generating challenging edge-case scenarios...");
        
        let mut challenging_files = Vec::new();
        
        // Very noisy environment (high noise floor)
        let noisy_scenario = self.generate_noisy_environment_scenario()?;
        challenging_files.push(noisy_scenario);
        
        // Many speakers scenario (8 speakers)
        let many_speakers_scenario = self.generate_many_speakers_scenario()?;
        challenging_files.push(many_speakers_scenario);
        
        // Whisper-like speech (very quiet)
        let whisper_scenario = self.generate_whisper_speech_scenario()?;
        challenging_files.push(whisper_scenario);
        
        // Mixed gender scenario
        let mixed_gender_scenario = self.generate_mixed_gender_scenario()?;
        challenging_files.push(mixed_gender_scenario);
        
        Ok(challenging_files)
    }

    /// Generate noisy environment test case
    fn generate_noisy_environment_scenario(&self) -> Result<GeneratedAudioFile, Box<dyn std::error::Error>> {
        let mut ground_truth = GroundTruthData::new("noisy_environment.wav".to_string(), 30.0);
        ground_truth.metadata.insert("scenario_type".to_string(), "noisy_environment".to_string());
        ground_truth.metadata.insert("description".to_string(), "High noise environment test".to_string());
        
        // Add segments with lower confidence due to noise
        let segments = vec![
            GroundTruthSegment::new("speaker_0".to_string(), 0.0, 7.0, Some("Can you hear me over this noise?".to_string()), 0.75),
            GroundTruthSegment::new("speaker_1".to_string(), 8.0, 15.0, Some("Yes, but it's quite loud in here.".to_string()), 0.72),
            GroundTruthSegment::new("speaker_0".to_string(), 16.0, 23.0, Some("We should find a quieter place.".to_string()), 0.78),
            GroundTruthSegment::new("speaker_1".to_string(), 24.0, 30.0, Some("Agreed, this is challenging.".to_string()), 0.74),
        ];
        
        for segment in segments {
            ground_truth.add_segment(segment);
        }
        
        // Generate with higher noise level
        let mut generator = TestAudioGenerator::new(&self.output_dir, self.sample_rate);
        generator.noise_level = 0.15; // Much higher noise
        
        let audio_path = generator.generate_scenario_audio(&ground_truth, "noisy_environment")?;
        let ground_truth_path = self.save_ground_truth(&ground_truth, "noisy_environment")?;
        
        Ok(GeneratedAudioFile {
            name: "noisy_environment".to_string(),
            audio_path,
            ground_truth_path,
            duration: ground_truth.duration,
            num_speakers: ground_truth.total_speakers,
            scenario_type: "challenging_noise".to_string(),
        })
    }

    /// Generate many speakers scenario (8 speakers)
    fn generate_many_speakers_scenario(&self) -> Result<GeneratedAudioFile, Box<dyn std::error::Error>> {
        let mut ground_truth = GroundTruthData::new("many_speakers.wav".to_string(), 60.0);
        ground_truth.metadata.insert("scenario_type".to_string(), "many_speakers".to_string());
        ground_truth.metadata.insert("description".to_string(), "8 speakers in conference call".to_string());
        
        // Create segments for 8 different speakers
        let segments = vec![
            GroundTruthSegment::new("speaker_0".to_string(), 0.0, 5.0, Some("Welcome everyone to the call.".to_string()), 0.92),
            GroundTruthSegment::new("speaker_1".to_string(), 5.5, 9.0, Some("Thanks for organizing this.".to_string()), 0.89),
            GroundTruthSegment::new("speaker_2".to_string(), 9.5, 13.0, Some("I have the quarterly report.".to_string()), 0.91),
            GroundTruthSegment::new("speaker_3".to_string(), 13.5, 17.0, Some("Great, we're ready to hear it.".to_string()), 0.88),
            GroundTruthSegment::new("speaker_4".to_string(), 17.5, 21.0, Some("Can everyone see the screen?".to_string()), 0.90),
            GroundTruthSegment::new("speaker_5".to_string(), 21.5, 25.0, Some("Yes, looks good from here.".to_string()), 0.87),
            GroundTruthSegment::new("speaker_6".to_string(), 25.5, 29.0, Some("I have a question about slide three.".to_string()), 0.89),
            GroundTruthSegment::new("speaker_7".to_string(), 29.5, 33.0, Some("Which part specifically?".to_string()), 0.86),
            GroundTruthSegment::new("speaker_0".to_string(), 33.5, 37.0, Some("Let's address that after the presentation.".to_string()), 0.92),
            GroundTruthSegment::new("speaker_2".to_string(), 37.5, 41.0, Some("Continuing with the next section.".to_string()), 0.91),
            GroundTruthSegment::new("speaker_1".to_string(), 41.5, 45.0, Some("This shows significant improvement.".to_string()), 0.89),
            GroundTruthSegment::new("speaker_3".to_string(), 45.5, 49.0, Some("Excellent results this quarter.".to_string()), 0.88),
            GroundTruthSegment::new("speaker_4".to_string(), 49.5, 53.0, Some("Credit to the entire team.".to_string()), 0.90),
            GroundTruthSegment::new("speaker_5".to_string(), 53.5, 57.0, Some("Any questions before we wrap up?".to_string()), 0.87),
            GroundTruthSegment::new("speaker_6".to_string(), 57.5, 60.0, Some("Thank you for the update.".to_string()), 0.89),
        ];
        
        for segment in segments {
            ground_truth.add_segment(segment);
        }
        
        let audio_path = self.generate_scenario_audio(&ground_truth, "many_speakers")?;
        let ground_truth_path = self.save_ground_truth(&ground_truth, "many_speakers")?;
        
        Ok(GeneratedAudioFile {
            name: "many_speakers".to_string(),
            audio_path,
            ground_truth_path,
            duration: ground_truth.duration,
            num_speakers: ground_truth.total_speakers,
            scenario_type: "challenging_many_speakers".to_string(),
        })
    }

    /// Generate whisper speech scenario (very quiet speech)
    fn generate_whisper_speech_scenario(&self) -> Result<GeneratedAudioFile, Box<dyn std::error::Error>> {
        let mut ground_truth = GroundTruthData::new("whisper_speech.wav".to_string(), 20.0);
        ground_truth.metadata.insert("scenario_type".to_string(), "whisper_speech".to_string());
        ground_truth.metadata.insert("description".to_string(), "Very quiet whisper-like speech".to_string());
        
        // Whisper scenarios have much lower confidence
        let segments = vec![
            GroundTruthSegment::new("speaker_0".to_string(), 0.0, 6.0, Some("We need to keep our voices down.".to_string()), 0.45),
            GroundTruthSegment::new("speaker_1".to_string(), 7.0, 13.0, Some("I understand, this is sensitive.".to_string()), 0.42),
            GroundTruthSegment::new("speaker_0".to_string(), 14.0, 20.0, Some("Exactly, we can't be overheard.".to_string()), 0.48),
        ];
        
        for segment in segments {
            ground_truth.add_segment(segment);
        }
        
        let audio_path = self.generate_scenario_audio(&ground_truth, "whisper_speech")?;
        let ground_truth_path = self.save_ground_truth(&ground_truth, "whisper_speech")?;
        
        Ok(GeneratedAudioFile {
            name: "whisper_speech".to_string(),
            audio_path,
            ground_truth_path,
            duration: ground_truth.duration,
            num_speakers: ground_truth.total_speakers,
            scenario_type: "challenging_whisper".to_string(),
        })
    }

    /// Generate mixed gender scenario with different voice characteristics
    fn generate_mixed_gender_scenario(&self) -> Result<GeneratedAudioFile, Box<dyn std::error::Error>> {
        let mut ground_truth = GroundTruthData::new("mixed_gender.wav".to_string(), 35.0);
        ground_truth.metadata.insert("scenario_type".to_string(), "mixed_gender".to_string());
        ground_truth.metadata.insert("description".to_string(), "Mixed male and female speakers".to_string());
        
        let segments = vec![
            GroundTruthSegment::new("male_speaker_0".to_string(), 0.0, 6.0, Some("Good morning everyone.".to_string()), 0.93),
            GroundTruthSegment::new("female_speaker_0".to_string(), 6.5, 12.0, Some("Thank you for joining the call.".to_string()), 0.91),
            GroundTruthSegment::new("male_speaker_1".to_string(), 12.5, 18.0, Some("I have the agenda prepared.".to_string()), 0.89),
            GroundTruthSegment::new("female_speaker_1".to_string(), 18.5, 24.0, Some("Perfect, let's begin with item one.".to_string()), 0.92),
            GroundTruthSegment::new("male_speaker_0".to_string(), 24.5, 30.0, Some("Here are the latest metrics.".to_string()), 0.93),
            GroundTruthSegment::new("female_speaker_0".to_string(), 30.5, 35.0, Some("These numbers look very promising.".to_string()), 0.91),
        ];
        
        for segment in segments {
            ground_truth.add_segment(segment);
        }
        
        let audio_path = self.generate_scenario_audio(&ground_truth, "mixed_gender")?;
        let ground_truth_path = self.save_ground_truth(&ground_truth, "mixed_gender")?;
        
        Ok(GeneratedAudioFile {
            name: "mixed_gender".to_string(),
            audio_path,
            ground_truth_path,
            duration: ground_truth.duration,
            num_speakers: ground_truth.total_speakers,
            scenario_type: "mixed_gender".to_string(),
        })
    }

    /// Save ground truth data as JSON file
    fn save_ground_truth(&self, ground_truth: &GroundTruthData, name: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let ground_truth_path = self.output_dir.join("../ground_truth").join(format!("{}.json", name));
        
        // Ensure ground truth directory exists
        if let Some(parent) = ground_truth_path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        let json_content = serde_json::to_string_pretty(ground_truth)?;
        fs::write(&ground_truth_path, json_content)?;
        
        Ok(ground_truth_path)
    }

    /// Save audio data as WAV file with proper specification
    fn save_wav_file(&self, path: &Path, audio_data: &[f32]) -> Result<(), Box<dyn std::error::Error>> {
        let spec = WavSpec {
            channels: 1,
            sample_rate: self.sample_rate,
            bits_per_sample: 16,
            sample_format: SampleFormat::Int,
        };
        
        let mut writer = WavWriter::create(path, spec)?;
        
        for &sample in audio_data {
            let sample_i16 = (sample.clamp(-1.0, 1.0) * i16::MAX as f32) as i16;
            writer.write_sample(sample_i16)?;
        }
        
        writer.finalize()?;
        Ok(())
    }

    /// Print generation summary
    fn print_generation_summary(&self, files: &[GeneratedAudioFile]) {
        println!("\n📊 Test Audio Generation Summary:");
        println!("================================");
        
        let total_duration: f32 = files.iter().map(|f| f.duration).sum();
        let total_speakers: usize = files.iter().map(|f| f.num_speakers).sum();
        
        println!("Total files generated: {}", files.len());
        println!("Total audio duration: {:.1} seconds", total_duration);
        println!("Total unique speakers: {}", total_speakers);
        println!("Sample rate: {} Hz", self.sample_rate);
        println!("Output directory: {}", self.output_dir.display());
        
        println!("\nGenerated files:");
        for file in files {
            println!("  • {} - {} speakers, {:.1}s ({})", 
                file.name, file.num_speakers, file.duration, file.scenario_type);
        }
        
        println!("\n✨ Ready for comprehensive diarization testing!");
    }
}

/// Information about a generated audio file
#[derive(Debug, Clone)]
pub struct GeneratedAudioFile {
    pub name: String,
    pub audio_path: PathBuf,
    pub ground_truth_path: PathBuf,
    pub duration: f32,
    pub num_speakers: usize,
    pub scenario_type: String,
}

impl GeneratedAudioFile {
    /// Get a summary string for this file
    pub fn summary(&self) -> String {
        format!(
            "{} - {} speakers, {:.1}s duration, type: {}",
            self.name, self.num_speakers, self.duration, self.scenario_type
        )
    }

    /// Check if the generated files exist
    pub fn files_exist(&self) -> bool {
        self.audio_path.exists() && self.ground_truth_path.exists()
    }
}

/// Utility functions for test audio generation
pub struct TestAudioUtils;

impl TestAudioUtils {
    /// Generate all test audio files in a standard location
    pub fn generate_standard_test_audio(base_dir: &Path) -> Result<Vec<GeneratedAudioFile>, Box<dyn std::error::Error>> {
        let test_audio_dir = base_dir.join("test_audio");
        let generator = TestAudioGenerator::new(&test_audio_dir, 16000);
        generator.generate_all_test_audio()
    }

    /// Verify that all generated files are valid
    pub fn verify_generated_files(files: &[GeneratedAudioFile]) -> Result<(), Box<dyn std::error::Error>> {
        println!("🔍 Verifying generated audio files...");
        
        for file in files {
            // Check that files exist
            if !file.files_exist() {
                return Err(format!("Missing files for scenario '{}': audio={}, ground_truth={}", 
                    file.name, file.audio_path.exists(), file.ground_truth_path.exists()).into());
            }
            
            // Verify WAV file can be read
            let reader = hound::WavReader::open(&file.audio_path)?;
            let spec = reader.spec();
            
            if spec.channels != 1 {
                return Err(format!("Invalid channel count for '{}': expected 1, got {}", file.name, spec.channels).into());
            }
            
            if spec.sample_rate != 16000 {
                return Err(format!("Invalid sample rate for '{}': expected 16000, got {}", file.name, spec.sample_rate).into());
            }
            
            // Verify JSON can be parsed
            let json_content = fs::read_to_string(&file.ground_truth_path)?;
            let _ground_truth: GroundTruthData = serde_json::from_str(&json_content)?;
            
            println!("  ✅ {}", file.name);
        }
        
        println!("✅ All {} audio files verified successfully", files.len());
        Ok(())
    }

    /// Clean up generated test files
    pub fn cleanup_generated_files(base_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let test_audio_dir = base_dir.join("test_audio");
        
        if test_audio_dir.exists() {
            fs::remove_dir_all(&test_audio_dir)?;
            println!("🧹 Cleaned up test audio directory: {}", test_audio_dir.display());
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_audio_generator_creation() {
        let temp_dir = env::temp_dir().join("kaginote_test_audio");
        let generator = TestAudioGenerator::new(&temp_dir, 16000);
        
        assert_eq!(generator.sample_rate, 16000);
        assert_eq!(generator.output_dir, temp_dir);
        assert!(generator.noise_level > 0.0);
    }

    #[test]
    fn test_generate_scenario_audio() {
        let temp_dir = env::temp_dir().join("kaginote_test_audio_scenario");
        let generator = TestAudioGenerator::new(&temp_dir, 16000);
        
        let ground_truth = TestScenarioGenerator::simple_two_speaker_conversation();
        let result = generator.generate_scenario_audio(&ground_truth, "test_scenario");
        
        assert!(result.is_ok());
        let audio_path = result.unwrap();
        assert!(audio_path.exists());
        
        // Verify the WAV file
        let reader = hound::WavReader::open(&audio_path);
        assert!(reader.is_ok());
        
        let reader = reader.unwrap();
        let spec = reader.spec();
        assert_eq!(spec.channels, 1);
        assert_eq!(spec.sample_rate, 16000);
        
        // Clean up
        let _ = fs::remove_file(&audio_path);
    }

    #[test]
    fn test_voice_envelope_creation() {
        let generator = TestAudioGenerator::new("/tmp", 16000);
        
        // Test attack phase
        let envelope_attack = generator.create_voice_envelope(0.05, 2.0);
        assert!(envelope_attack > 0.0 && envelope_attack < 1.0);
        
        // Test sustain phase
        let envelope_sustain = generator.create_voice_envelope(1.0, 2.0);
        assert!(envelope_sustain > 0.8);
        
        // Test decay phase
        let envelope_decay = generator.create_voice_envelope(1.95, 2.0);
        assert!(envelope_decay > 0.0 && envelope_decay < 1.0);
    }

    #[test]
    fn test_multi_frequency_audio_generation() {
        let temp_dir = env::temp_dir().join("kaginote_test_multifreq");
        let generator = TestAudioGenerator::new(&temp_dir, 16000);
        
        let ground_truth = TestScenarioGenerator::simple_two_speaker_conversation();
        let result = generator.generate_multi_frequency_audio(&ground_truth);
        
        assert!(result.is_ok());
        let audio = result.unwrap();
        
        // Should have correct length
        let expected_samples = (ground_truth.duration * 16000.0) as usize;
        assert_eq!(audio.len(), expected_samples);
        
        // Should have non-zero audio content
        let has_content = audio.iter().any(|&sample| sample.abs() > 0.01);
        assert!(has_content);
    }

    #[test]
    fn test_voice_processing_pipeline() {
        let generator = TestAudioGenerator::new("/tmp", 16000);
        
        // Create simple test audio
        let test_audio = vec![0.5f32; 1000];
        
        let result = generator.apply_voice_processing(test_audio);
        assert!(result.is_ok());
        
        let processed_audio = result.unwrap();
        assert_eq!(processed_audio.len(), 1000);
        
        // Should be normalized (max amplitude should be reasonable)
        let max_amplitude = processed_audio.iter().map(|&s| s.abs()).fold(0.0f32, f32::max);
        assert!(max_amplitude <= 1.0);
        assert!(max_amplitude > 0.0);
    }

    #[test]
    fn test_generated_audio_file_validation() {
        let temp_dir = env::temp_dir().join("kaginote_test_validation");
        fs::create_dir_all(&temp_dir).unwrap();
        
        // Create dummy files
        let audio_path = temp_dir.join("test.wav");
        let ground_truth_path = temp_dir.join("test.json");
        
        fs::write(&audio_path, b"dummy").unwrap();
        fs::write(&ground_truth_path, b"{}").unwrap();
        
        let file = GeneratedAudioFile {
            name: "test".to_string(),
            audio_path,
            ground_truth_path,
            duration: 30.0,
            num_speakers: 2,
            scenario_type: "test".to_string(),
        };
        
        assert!(file.files_exist());
        assert!(file.summary().contains("test"));
        assert!(file.summary().contains("2 speakers"));
        
        // Clean up
        let _ = fs::remove_dir_all(&temp_dir);
    }
}