use std::collections::VecDeque;

/// Audio chunk with basic characteristics for boundary detection
#[derive(Debug, Clone)]
pub struct AudioChunk {
    pub samples: Vec<f32>,
    pub sample_rate: u32,
    pub timestamp: std::time::SystemTime,
    pub energy_level: f32,
}

/// Speech boundary detection result
#[derive(Debug, Clone, PartialEq)]
pub enum BoundaryType {
    None,           // No boundary detected
    SoftBoundary,   // Possible natural pause
    HardBoundary,   // Clear speech segment boundary
    SentenceEnd,    // End of complete sentence/thought
}

/// Advanced boundary detector for identifying natural speech pauses
pub struct BoundaryDetector {
    /// Recent audio chunks for analysis
    recent_chunks: VecDeque<AudioChunk>,
    /// Energy level history for trend analysis
    energy_history: VecDeque<f32>,
    /// Configuration parameters
    config: BoundaryConfig,
    /// Internal state tracking
    consecutive_silence_chunks: usize,
    consecutive_speech_chunks: usize,
    last_boundary_time: Option<std::time::SystemTime>,
    speech_pattern_buffer: Vec<f32>,
}

#[derive(Debug, Clone)]
pub struct BoundaryConfig {
    /// Energy threshold for silence detection
    pub silence_threshold: f32,
    /// Minimum silence duration for soft boundary (ms)
    pub soft_boundary_ms: u64,
    /// Minimum silence duration for hard boundary (ms)
    pub hard_boundary_ms: u64,
    /// Maximum chunks to analyze
    pub max_chunks: usize,
    /// Minimum speech duration before considering boundaries (ms)
    pub min_speech_duration_ms: u64,
    /// Energy variance threshold for detecting speech patterns
    pub energy_variance_threshold: f32,
    /// Spectral analysis parameters
    pub spectral_analysis_enabled: bool,
}

impl Default for BoundaryConfig {
    fn default() -> Self {
        Self {
            silence_threshold: 0.015,
            soft_boundary_ms: 400,
            hard_boundary_ms: 800,
            max_chunks: 50,
            min_speech_duration_ms: 2000,
            energy_variance_threshold: 0.05,
            spectral_analysis_enabled: true,
        }
    }
}

impl BoundaryDetector {
    pub fn new(config: BoundaryConfig) -> Self {
        Self {
            recent_chunks: VecDeque::with_capacity(config.max_chunks),
            energy_history: VecDeque::with_capacity(config.max_chunks),
            config,
            consecutive_silence_chunks: 0,
            consecutive_speech_chunks: 0,
            last_boundary_time: None,
            speech_pattern_buffer: Vec::new(),
        }
    }

    /// Process new audio chunk and detect speech boundaries
    pub fn process_chunk(&mut self, chunk: AudioChunk) -> BoundaryType {
        let chunk_energy = chunk.energy_level;
        
        // Add to history
        self.recent_chunks.push_back(chunk.clone());
        self.energy_history.push_back(chunk_energy);
        
        // Maintain size limits
        if self.recent_chunks.len() > self.config.max_chunks {
            self.recent_chunks.pop_front();
            self.energy_history.pop_front();
        }

        // Update speech/silence counters
        let is_speech = chunk_energy > self.config.silence_threshold;
        
        if is_speech {
            self.consecutive_speech_chunks += 1;
            self.consecutive_silence_chunks = 0;
            self.speech_pattern_buffer.push(chunk_energy);
            
            // Maintain pattern buffer size
            if self.speech_pattern_buffer.len() > 20 {
                self.speech_pattern_buffer.remove(0);
            }
        } else {
            self.consecutive_silence_chunks += 1;
            self.consecutive_speech_chunks = 0;
        }

        // Detect boundaries based on analysis
        self.detect_boundary(&chunk)
    }

    /// Main boundary detection logic
    fn detect_boundary(&mut self, current_chunk: &AudioChunk) -> BoundaryType {
        // Need sufficient speech history before detecting boundaries
        if self.get_total_speech_duration() < self.config.min_speech_duration_ms {
            return BoundaryType::None;
        }

        // Calculate silence duration in milliseconds (assuming ~100ms chunks)
        let silence_duration_ms = self.consecutive_silence_chunks as u64 * 100;

        // Check for hard boundary (long silence)
        if silence_duration_ms >= self.config.hard_boundary_ms {
            // Additional validation for hard boundaries
            if self.validate_hard_boundary(current_chunk) {
                self.last_boundary_time = Some(current_chunk.timestamp);
                return BoundaryType::HardBoundary;
            }
        }

        // Check for soft boundary (medium silence)
        if silence_duration_ms >= self.config.soft_boundary_ms {
            if self.validate_soft_boundary(current_chunk) {
                return BoundaryType::SoftBoundary;
            }
        }

        // Check for sentence ending patterns
        if self.detect_sentence_ending_pattern() {
            return BoundaryType::SentenceEnd;
        }

        BoundaryType::None
    }

    /// Validate hard boundary with additional criteria
    fn validate_hard_boundary(&self, _chunk: &AudioChunk) -> bool {
        // Ensure we had significant speech before the silence
        if self.speech_pattern_buffer.len() < 5 {
            return false;
        }

        // Check if there was a clear energy drop pattern
        if let Some(recent_energy) = self.energy_history.iter().rev().take(10).collect::<Vec<_>>().first() {
            let avg_speech_energy = self.speech_pattern_buffer.iter().sum::<f32>() / self.speech_pattern_buffer.len() as f32;
            
            // Significant energy drop indicates natural pause
            if **recent_energy < avg_speech_energy * 0.2 {
                return true;
            }
        }

        // Check for consistent silence pattern
        let recent_silence_count = self.energy_history
            .iter()
            .rev()
            .take(8)
            .filter(|&&e| e <= self.config.silence_threshold)
            .count();
        
        recent_silence_count >= 6
    }

    /// Validate soft boundary with pattern analysis
    fn validate_soft_boundary(&self, _chunk: &AudioChunk) -> bool {
        // Don't create soft boundaries too close to the last boundary
        if let Some(last_time) = self.last_boundary_time {
            if let Ok(elapsed) = last_time.elapsed() {
                if elapsed.as_millis() < 1000 { // Less than 1 second
                    return false;
                }
            }
        }

        // Check for natural speech patterns
        self.has_natural_speech_pattern()
    }

    /// Detect sentence ending patterns in speech characteristics
    fn detect_sentence_ending_pattern(&self) -> bool {
        if !self.config.spectral_analysis_enabled || self.speech_pattern_buffer.len() < 10 {
            return false;
        }

        // Analyze energy pattern for sentence-ending characteristics
        let recent_pattern: Vec<f32> = self.speech_pattern_buffer.iter().rev().take(10).cloned().collect();
        
        // Look for descending energy pattern (typical of sentence endings)
        let mut descending_count = 0;
        for window in recent_pattern.windows(2) {
            if window[0] > window[1] {
                descending_count += 1;
            }
        }

        // Sentence endings typically show 70%+ descending energy
        descending_count >= 7
    }

    /// Check if current speech pattern indicates natural pausing
    fn has_natural_speech_pattern(&self) -> bool {
        if self.energy_history.len() < 10 {
            return false;
        }

        // Calculate energy variance over recent history
        let recent_energies: Vec<f32> = self.energy_history.iter().rev().take(10).cloned().collect();
        let mean_energy = recent_energies.iter().sum::<f32>() / recent_energies.len() as f32;
        
        let variance = recent_energies
            .iter()
            .map(|&e| (e - mean_energy).powi(2))
            .sum::<f32>() / recent_energies.len() as f32;

        // Natural speech has moderate energy variance
        variance >= self.config.energy_variance_threshold && variance <= 0.5
    }

    /// Get total speech duration from current session
    fn get_total_speech_duration(&self) -> u64 {
        // Approximate based on speech chunks (assuming ~100ms per chunk)
        self.recent_chunks
            .iter()
            .filter(|chunk| chunk.energy_level > self.config.silence_threshold)
            .count() as u64 * 100
    }

    /// Check if we should wait for more audio before making transcription
    pub fn should_continue_buffering(&mut self, current_buffer_duration_ms: u64) -> bool {
        // Always buffer for minimum duration
        if current_buffer_duration_ms < self.config.min_speech_duration_ms {
            return true;
        }

        // If we're in the middle of speech, continue buffering
        if self.consecutive_speech_chunks > 0 && self.consecutive_silence_chunks < 3 {
            return true;
        }

        // If no clear boundary detected and not too long, continue
        if current_buffer_duration_ms < 15000 { // Max 15 seconds
            // Check if we have recent chunks to analyze
            if let Some(last_chunk) = self.recent_chunks.back().cloned() {
                let recent_boundary = self.detect_boundary(&last_chunk);
                matches!(recent_boundary, BoundaryType::None | BoundaryType::SoftBoundary)
            } else {
                true // Continue if no chunks yet
            }
        } else {
            false // Stop buffering after 15 seconds
        }
    }

    /// Get optimal transcription trigger point
    pub fn get_optimal_transcription_point(&self) -> Option<std::time::SystemTime> {
        // Find the most recent hard boundary
        for (i, chunk) in self.recent_chunks.iter().rev().enumerate() {
            if chunk.energy_level <= self.config.silence_threshold {
                // Check if this represents a good boundary
                let silence_duration = i * 100; // Approximate ms
                if silence_duration >= self.config.hard_boundary_ms as usize {
                    return Some(chunk.timestamp);
                }
            }
        }

        None
    }

    /// Reset detector state for new session
    pub fn reset(&mut self) {
        self.recent_chunks.clear();
        self.energy_history.clear();
        self.consecutive_silence_chunks = 0;
        self.consecutive_speech_chunks = 0;
        self.last_boundary_time = None;
        self.speech_pattern_buffer.clear();
    }

    /// Get detector statistics for debugging
    pub fn get_stats(&self) -> (usize, usize, f32) {
        let avg_energy = if self.energy_history.is_empty() {
            0.0
        } else {
            self.energy_history.iter().sum::<f32>() / self.energy_history.len() as f32
        };

        (
            self.consecutive_speech_chunks,
            self.consecutive_silence_chunks,
            avg_energy,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, Duration};

    fn create_audio_chunk(energy: f32, timestamp: SystemTime) -> AudioChunk {
        AudioChunk {
            samples: vec![energy; 1600], // Simulated 100ms at 16kHz
            sample_rate: 16000,
            timestamp,
            energy_level: energy,
        }
    }

    #[test]
    fn test_silence_detection() {
        let config = BoundaryConfig::default();
        let mut detector = BoundaryDetector::new(config);

        let start_time = SystemTime::now();

        // Add some speech chunks
        for i in 0..5 {
            let chunk = create_audio_chunk(0.1, start_time + Duration::from_millis(i * 100));
            detector.process_chunk(chunk);
        }

        // Add silence chunks
        for i in 5..15 {
            let chunk = create_audio_chunk(0.001, start_time + Duration::from_millis(i * 100));
            let boundary = detector.process_chunk(chunk);
            
            if i >= 13 { // Should detect hard boundary after 800ms silence
                assert_eq!(boundary, BoundaryType::HardBoundary);
                break;
            }
        }
    }

    #[test]
    fn test_minimum_speech_duration() {
        let config = BoundaryConfig::default();
        let mut detector = BoundaryDetector::new(config);

        let start_time = SystemTime::now();

        // Add only short speech - should not detect boundaries
        for i in 0..3 {
            let speech_chunk = create_audio_chunk(0.1, start_time + Duration::from_millis(i * 100));
            detector.process_chunk(speech_chunk);
        }

        // Add silence - should not detect boundary due to insufficient speech
        for i in 3..13 {
            let silence_chunk = create_audio_chunk(0.001, start_time + Duration::from_millis(i * 100));
            let boundary = detector.process_chunk(silence_chunk);
            assert_eq!(boundary, BoundaryType::None);
        }
    }

    #[test]
    fn test_sentence_ending_pattern() {
        let mut config = BoundaryConfig::default();
        config.spectral_analysis_enabled = true;
        let mut detector = BoundaryDetector::new(config);

        let start_time = SystemTime::now();

        // Create descending energy pattern (simulating sentence ending)
        let energies = [0.2, 0.18, 0.16, 0.14, 0.12, 0.10, 0.08, 0.06, 0.04, 0.02];
        
        for (i, &energy) in energies.iter().enumerate() {
            let chunk = create_audio_chunk(energy, start_time + Duration::from_millis(i as u64 * 100));
            detector.process_chunk(chunk);
        }

        // Add sufficient speech history first
        for i in 0..25 {
            let chunk = create_audio_chunk(0.1, start_time + Duration::from_millis(i * 100));
            detector.process_chunk(chunk);
        }

        // Now test the descending pattern
        for (i, &energy) in energies.iter().enumerate() {
            let chunk = create_audio_chunk(energy, start_time + Duration::from_millis((25 + i) as u64 * 100));
            let boundary = detector.process_chunk(chunk);
            
            if i == energies.len() - 1 {
                // Should detect sentence ending pattern
                assert_eq!(boundary, BoundaryType::SentenceEnd);
            }
        }
    }

    #[test]
    fn test_buffering_decision() {
        let config = BoundaryConfig::default();
        let mut detector = BoundaryDetector::new(config);

        // Should continue buffering for minimum duration
        assert!(detector.should_continue_buffering(1000)); // 1 second < minimum

        // Should stop buffering after maximum duration
        assert!(!detector.should_continue_buffering(20000)); // 20 seconds > maximum
    }
}