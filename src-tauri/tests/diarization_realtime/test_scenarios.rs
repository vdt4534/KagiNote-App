use std::collections::HashMap;
use std::time::Duration;
use serde::{Deserialize, Serialize};

/// Represents a ground truth segment with speaker identification and timing
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GroundTruthSegment {
    /// Unique speaker identifier (e.g., "speaker_0", "speaker_1")
    pub speaker_id: String,
    /// Start time in seconds
    pub start_time: f32,
    /// End time in seconds  
    pub end_time: f32,
    /// Optional transcript text for this segment
    pub text: Option<String>,
    /// Confidence level for this segment (0.0-1.0)
    pub confidence: f32,
}

impl GroundTruthSegment {
    pub fn new(speaker_id: String, start_time: f32, end_time: f32, text: Option<String>, confidence: f32) -> Self {
        Self {
            speaker_id,
            start_time,
            end_time,
            text,
            confidence,
        }
    }

    /// Duration of this segment in seconds
    pub fn duration(&self) -> f32 {
        self.end_time - self.start_time
    }

    /// Check if this segment overlaps with another segment
    pub fn overlaps_with(&self, other: &GroundTruthSegment) -> bool {
        self.start_time < other.end_time && self.end_time > other.start_time
    }

    /// Check if this segment is within a time tolerance of expected timing
    pub fn matches_timing(&self, expected_start: f32, expected_end: f32, tolerance: f32) -> bool {
        (self.start_time - expected_start).abs() <= tolerance 
            && (self.end_time - expected_end).abs() <= tolerance
    }
}

/// Complete ground truth data for a test scenario
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroundTruthData {
    /// Path to the audio file for this test case
    pub audio_file: String,
    /// Total duration of the audio in seconds
    pub duration: f32,
    /// All speaker segments in chronological order
    pub segments: Vec<GroundTruthSegment>,
    /// Total number of unique speakers
    pub total_speakers: usize,
    /// Additional metadata for this test case
    pub metadata: HashMap<String, String>,
}

impl GroundTruthData {
    pub fn new(audio_file: String, duration: f32) -> Self {
        Self {
            audio_file,
            duration,
            segments: Vec::new(),
            total_speakers: 0,
            metadata: HashMap::new(),
        }
    }

    /// Add a segment to the ground truth data
    pub fn add_segment(&mut self, segment: GroundTruthSegment) {
        self.segments.push(segment);
        self.update_speaker_count();
    }

    /// Update the total speaker count based on segments
    fn update_speaker_count(&mut self) {
        let unique_speakers: std::collections::HashSet<_> = 
            self.segments.iter().map(|s| &s.speaker_id).collect();
        self.total_speakers = unique_speakers.len();
    }

    /// Get all segments for a specific speaker
    pub fn segments_for_speaker(&self, speaker_id: &str) -> Vec<&GroundTruthSegment> {
        self.segments.iter().filter(|s| s.speaker_id == speaker_id).collect()
    }

    /// Get all unique speaker IDs
    pub fn speaker_ids(&self) -> Vec<String> {
        let mut speakers: Vec<_> = self.segments.iter().map(|s| s.speaker_id.clone()).collect();
        speakers.sort();
        speakers.dedup();
        speakers
    }

    /// Check for overlapping segments (indicating simultaneous speech)
    pub fn has_overlaps(&self) -> bool {
        for (i, segment1) in self.segments.iter().enumerate() {
            for segment2 in self.segments.iter().skip(i + 1) {
                if segment1.overlaps_with(segment2) {
                    return true;
                }
            }
        }
        false
    }

    /// Calculate total speaking time for each speaker
    pub fn speaking_time_per_speaker(&self) -> HashMap<String, f32> {
        let mut speaking_times = HashMap::new();
        for segment in &self.segments {
            let duration = segment.duration();
            *speaking_times.entry(segment.speaker_id.clone()).or_insert(0.0) += duration;
        }
        speaking_times
    }
}

/// Test scenario types for different diarization challenges
#[derive(Debug, Clone)]
pub enum TestScenarioType {
    /// Simple turn-taking conversation
    SimpleTurnTaking,
    /// Meeting with multiple speakers and interruptions
    MultiSpeakerMeeting,
    /// Overlapping speech scenarios
    OverlappingSpeech,
    /// Rapid speaker switching
    RapidSwitching,
    /// Long silence periods
    LongSilences,
    /// Single speaker monologue
    SingleSpeakerMonologue,
}

/// Configuration for generating test scenarios
#[derive(Debug, Clone)]
pub struct TestScenarioConfig {
    pub scenario_type: TestScenarioType,
    pub duration: f32,
    pub num_speakers: usize,
    pub silence_probability: f32,
    pub overlap_probability: f32,
    pub min_segment_duration: f32,
    pub max_segment_duration: f32,
}

impl Default for TestScenarioConfig {
    fn default() -> Self {
        Self {
            scenario_type: TestScenarioType::SimpleTurnTaking,
            duration: 60.0,
            num_speakers: 2,
            silence_probability: 0.1,
            overlap_probability: 0.0,
            min_segment_duration: 2.0,
            max_segment_duration: 8.0,
        }
    }
}

/// Test scenario generator for creating ground truth data
pub struct TestScenarioGenerator;

impl TestScenarioGenerator {
    /// Generate a simple 2-speaker turn-taking conversation
    pub fn simple_two_speaker_conversation() -> GroundTruthData {
        let mut data = GroundTruthData::new("test_2speakers_turntaking.wav".to_string(), 30.0);
        
        // Add metadata
        data.metadata.insert("scenario_type".to_string(), "simple_turn_taking".to_string());
        data.metadata.insert("description".to_string(), "Simple turn-taking between 2 speakers".to_string());
        
        // Create alternating segments
        let segments = vec![
            GroundTruthSegment::new("speaker_0".to_string(), 0.0, 5.0, Some("Hello, how are you today?".to_string()), 0.95),
            GroundTruthSegment::new("speaker_1".to_string(), 5.5, 10.0, Some("I'm doing well, thank you for asking.".to_string()), 0.92),
            GroundTruthSegment::new("speaker_0".to_string(), 10.5, 15.0, Some("That's great to hear.".to_string()), 0.94),
            GroundTruthSegment::new("speaker_1".to_string(), 15.5, 20.0, Some("How about you?".to_string()), 0.90),
            GroundTruthSegment::new("speaker_0".to_string(), 20.5, 25.0, Some("I'm having a good day too.".to_string()), 0.93),
            GroundTruthSegment::new("speaker_1".to_string(), 25.5, 30.0, Some("Wonderful to hear that.".to_string()), 0.91),
        ];
        
        for segment in segments {
            data.add_segment(segment);
        }
        
        data
    }

    /// Generate a 3-speaker meeting with natural interruptions
    pub fn multi_speaker_meeting() -> GroundTruthData {
        let mut data = GroundTruthData::new("test_3speakers_meeting.wav".to_string(), 45.0);
        
        data.metadata.insert("scenario_type".to_string(), "multi_speaker_meeting".to_string());
        data.metadata.insert("description".to_string(), "Meeting simulation with 3 speakers and interruptions".to_string());
        
        let segments = vec![
            GroundTruthSegment::new("speaker_0".to_string(), 0.0, 6.0, Some("Welcome everyone to today's meeting.".to_string()), 0.96),
            GroundTruthSegment::new("speaker_1".to_string(), 6.5, 12.0, Some("Thanks for organizing this.".to_string()), 0.94),
            GroundTruthSegment::new("speaker_2".to_string(), 12.5, 18.0, Some("I have some important updates to share.".to_string()), 0.93),
            // Interruption scenario
            GroundTruthSegment::new("speaker_1".to_string(), 17.0, 22.0, Some("Sorry to interrupt, but this is urgent.".to_string()), 0.89),
            GroundTruthSegment::new("speaker_2".to_string(), 22.5, 28.0, Some("No problem, what's the issue?".to_string()), 0.92),
            GroundTruthSegment::new("speaker_0".to_string(), 28.5, 35.0, Some("Let's address this concern first.".to_string()), 0.95),
            GroundTruthSegment::new("speaker_1".to_string(), 35.5, 41.0, Some("I appreciate your understanding.".to_string()), 0.91),
            GroundTruthSegment::new("speaker_2".to_string(), 41.5, 45.0, Some("Shall we continue with the agenda?".to_string()), 0.94),
        ];
        
        for segment in segments {
            data.add_segment(segment);
        }
        
        data
    }

    /// Generate overlapping speech scenario
    pub fn overlapping_speech_scenario() -> GroundTruthData {
        let mut data = GroundTruthData::new("test_overlapping_speech.wav".to_string(), 25.0);
        
        data.metadata.insert("scenario_type".to_string(), "overlapping_speech".to_string());
        data.metadata.insert("description".to_string(), "Multiple speakers talking simultaneously".to_string());
        
        let segments = vec![
            GroundTruthSegment::new("speaker_0".to_string(), 0.0, 5.0, Some("I think we should consider all options.".to_string()), 0.93),
            GroundTruthSegment::new("speaker_1".to_string(), 4.0, 9.0, Some("Actually, I disagree with that approach.".to_string()), 0.88), // Overlap
            GroundTruthSegment::new("speaker_0".to_string(), 8.0, 13.0, Some("But we need to think about the budget.".to_string()), 0.85), // Overlap
            GroundTruthSegment::new("speaker_1".to_string(), 13.5, 18.0, Some("The budget is exactly why we can't do this.".to_string()), 0.91),
            GroundTruthSegment::new("speaker_0".to_string(), 17.0, 22.0, Some("Let me explain my reasoning.".to_string()), 0.87), // Overlap
            GroundTruthSegment::new("speaker_1".to_string(), 22.5, 25.0, Some("I'm listening.".to_string()), 0.94),
        ];
        
        for segment in segments {
            data.add_segment(segment);
        }
        
        data
    }

    /// Generate rapid speaker switching scenario
    pub fn rapid_speaker_switching() -> GroundTruthData {
        let mut data = GroundTruthData::new("test_rapid_switching.wav".to_string(), 20.0);
        
        data.metadata.insert("scenario_type".to_string(), "rapid_switching".to_string());
        data.metadata.insert("description".to_string(), "Very fast speaker changes every 1-2 seconds".to_string());
        
        let segments = vec![
            GroundTruthSegment::new("speaker_0".to_string(), 0.0, 1.5, Some("Yes.".to_string()), 0.85),
            GroundTruthSegment::new("speaker_1".to_string(), 1.8, 3.2, Some("No.".to_string()), 0.83),
            GroundTruthSegment::new("speaker_0".to_string(), 3.5, 5.0, Some("Maybe.".to_string()), 0.81),
            GroundTruthSegment::new("speaker_1".to_string(), 5.3, 6.8, Some("Exactly.".to_string()), 0.86),
            GroundTruthSegment::new("speaker_0".to_string(), 7.1, 8.5, Some("I see.".to_string()), 0.82),
            GroundTruthSegment::new("speaker_1".to_string(), 8.8, 10.5, Some("Right.".to_string()), 0.84),
            GroundTruthSegment::new("speaker_0".to_string(), 10.8, 12.3, Some("Okay.".to_string()), 0.80),
            GroundTruthSegment::new("speaker_1".to_string(), 12.6, 14.2, Some("Sure.".to_string()), 0.87),
            GroundTruthSegment::new("speaker_0".to_string(), 14.5, 16.0, Some("Got it.".to_string()), 0.83),
            GroundTruthSegment::new("speaker_1".to_string(), 16.3, 18.0, Some("Perfect.".to_string()), 0.89),
            GroundTruthSegment::new("speaker_0".to_string(), 18.3, 20.0, Some("Thanks.".to_string()), 0.85),
        ];
        
        for segment in segments {
            data.add_segment(segment);
        }
        
        data
    }

    /// Generate scenario with long silence periods
    pub fn long_silences_scenario() -> GroundTruthData {
        let mut data = GroundTruthData::new("test_long_silences.wav".to_string(), 40.0);
        
        data.metadata.insert("scenario_type".to_string(), "long_silences".to_string());
        data.metadata.insert("description".to_string(), "Speakers with long pauses between utterances".to_string());
        
        let segments = vec![
            GroundTruthSegment::new("speaker_0".to_string(), 0.0, 5.0, Some("Let me think about this for a moment.".to_string()), 0.94),
            // 8 seconds of silence
            GroundTruthSegment::new("speaker_1".to_string(), 13.0, 18.0, Some("Take your time, no rush.".to_string()), 0.92),
            // 7 seconds of silence
            GroundTruthSegment::new("speaker_0".to_string(), 25.0, 31.0, Some("Okay, I think I have an answer now.".to_string()), 0.95),
            // 4 seconds of silence
            GroundTruthSegment::new("speaker_1".to_string(), 35.0, 40.0, Some("I'm ready to hear it.".to_string()), 0.93),
        ];
        
        for segment in segments {
            data.add_segment(segment);
        }
        
        data
    }

    /// Generate single speaker monologue
    pub fn single_speaker_monologue() -> GroundTruthData {
        let mut data = GroundTruthData::new("test_monologue.wav".to_string(), 30.0);
        
        data.metadata.insert("scenario_type".to_string(), "single_speaker_monologue".to_string());
        data.metadata.insert("description".to_string(), "Single speaker delivering a continuous speech".to_string());
        
        let segments = vec![
            GroundTruthSegment::new("speaker_0".to_string(), 0.0, 30.0, 
                Some("Today I want to talk about the importance of artificial intelligence in modern society. It has transformed how we work, communicate, and solve complex problems. From healthcare to transportation, AI is making significant impacts across all industries.".to_string()), 
                0.97),
        ];
        
        for segment in segments {
            data.add_segment(segment);
        }
        
        data
    }

    /// Generate test scenario based on configuration
    pub fn generate_scenario(config: TestScenarioConfig) -> GroundTruthData {
        match config.scenario_type {
            TestScenarioType::SimpleTurnTaking => Self::simple_two_speaker_conversation(),
            TestScenarioType::MultiSpeakerMeeting => Self::multi_speaker_meeting(),
            TestScenarioType::OverlappingSpeech => Self::overlapping_speech_scenario(),
            TestScenarioType::RapidSwitching => Self::rapid_speaker_switching(),
            TestScenarioType::LongSilences => Self::long_silences_scenario(),
            TestScenarioType::SingleSpeakerMonologue => Self::single_speaker_monologue(),
        }
    }
}

/// Synthetic audio generator for creating test audio files
pub struct SyntheticAudioGenerator;

impl SyntheticAudioGenerator {
    /// Generate synthetic multi-speaker audio using different frequencies
    /// Each speaker gets a unique frequency to simulate different voices
    pub fn generate_multi_frequency_audio(ground_truth: &GroundTruthData, sample_rate: u32) -> Vec<f32> {
        let total_samples = (ground_truth.duration * sample_rate as f32) as usize;
        let mut audio_buffer = vec![0.0f32; total_samples];
        
        // Assign frequencies to speakers (C4, E4, G4, A4, etc.)
        let speaker_frequencies = vec![261.63, 329.63, 392.0, 440.0, 523.25, 659.25, 783.99, 880.0];
        let mut speaker_freq_map = HashMap::new();
        
        for (i, speaker_id) in ground_truth.speaker_ids().iter().enumerate() {
            let freq = speaker_frequencies[i % speaker_frequencies.len()];
            speaker_freq_map.insert(speaker_id.clone(), freq);
        }
        
        // Generate audio for each segment
        for segment in &ground_truth.segments {
            let start_sample = (segment.start_time * sample_rate as f32) as usize;
            let end_sample = (segment.end_time * sample_rate as f32) as usize;
            let frequency = speaker_freq_map.get(&segment.speaker_id).unwrap_or(&440.0);
            
            for i in start_sample..end_sample.min(total_samples) {
                let t = i as f32 / sample_rate as f32;
                let amplitude = 0.3 * segment.confidence; // Use confidence as amplitude
                audio_buffer[i] += amplitude * (2.0 * std::f32::consts::PI * frequency * t).sin();
            }
        }
        
        audio_buffer
    }

    /// Add realistic background noise to synthetic audio
    pub fn add_background_noise(audio: &mut [f32], noise_level: f32) {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        
        for sample in audio.iter_mut() {
            let noise = rng.gen_range(-noise_level..noise_level);
            *sample += noise;
            *sample = sample.clamp(-1.0, 1.0); // Prevent clipping
        }
    }

    /// Apply voice-like filtering to make synthetic audio more realistic
    pub fn apply_voice_filter(audio: &mut [f32]) {
        // Simple low-pass filter to simulate human voice characteristics
        let mut prev_sample = 0.0f32;
        let alpha = 0.8; // Filter coefficient
        
        for sample in audio.iter_mut() {
            *sample = alpha * (*sample) + (1.0 - alpha) * prev_sample;
            prev_sample = *sample;
        }
    }
}

/// Scenario validator for checking diarization results against ground truth
pub struct ScenarioValidator;

impl ScenarioValidator {
    /// Check if detected speakers match ground truth within tolerance
    pub fn validate_speaker_detection(
        detected_segments: &[DetectedSegment],
        ground_truth: &GroundTruthData,
        time_tolerance: f32,
    ) -> ValidationResult {
        let mut correct_detections = 0;
        let mut total_detections = detected_segments.len();
        let mut speaker_mapping = HashMap::new();
        
        // Try to map detected speakers to ground truth speakers
        for detected in detected_segments {
            if let Some(ground_truth_segment) = Self::find_matching_segment(detected, ground_truth, time_tolerance) {
                // Update speaker mapping
                speaker_mapping.entry(detected.speaker_id.clone())
                    .or_insert_with(HashMap::new)
                    .entry(ground_truth_segment.speaker_id.clone())
                    .and_modify(|e| *e += 1)
                    .or_insert(1);
            }
        }
        
        // Find best speaker mapping
        let mut final_mapping = HashMap::new();
        for (detected_speaker, ground_truth_votes) in speaker_mapping {
            if let Some((best_ground_truth_speaker, _)) = ground_truth_votes.iter()
                .max_by_key(|(_, count)| *count) {
                final_mapping.insert(detected_speaker, best_ground_truth_speaker.clone());
            }
        }
        
        // Count correct detections using best mapping
        for detected in detected_segments {
            if let Some(mapped_speaker) = final_mapping.get(&detected.speaker_id) {
                if let Some(ground_truth_segment) = Self::find_matching_segment(detected, ground_truth, time_tolerance) {
                    if mapped_speaker == &ground_truth_segment.speaker_id {
                        correct_detections += 1;
                    }
                }
            }
        }
        
        let accuracy = if total_detections > 0 {
            correct_detections as f32 / total_detections as f32
        } else {
            0.0
        };
        
        ValidationResult {
            accuracy,
            correct_detections,
            total_detections,
            speaker_mapping: final_mapping,
            temporal_alignment_score: Self::calculate_temporal_alignment(detected_segments, ground_truth, time_tolerance),
        }
    }

    /// Find the best matching ground truth segment for a detected segment
    fn find_matching_segment(
        detected: &DetectedSegment,
        ground_truth: &GroundTruthData,
        tolerance: f32,
    ) -> Option<&GroundTruthSegment> {
        ground_truth.segments.iter()
            .filter(|segment| {
                // Check temporal overlap
                let overlap_start = detected.start_time.max(segment.start_time);
                let overlap_end = detected.end_time.min(segment.end_time);
                overlap_end > overlap_start && (overlap_end - overlap_start) >= tolerance
            })
            .max_by(|a, b| {
                // Select segment with maximum temporal overlap
                let overlap_a = detected.end_time.min(a.end_time) - detected.start_time.max(a.start_time);
                let overlap_b = detected.end_time.min(b.end_time) - detected.start_time.max(b.start_time);
                overlap_a.partial_cmp(&overlap_b).unwrap_or(std::cmp::Ordering::Equal)
            })
    }

    /// Calculate temporal alignment score between detected and ground truth
    fn calculate_temporal_alignment(
        detected_segments: &[DetectedSegment],
        ground_truth: &GroundTruthData,
        tolerance: f32,
    ) -> f32 {
        let mut total_overlap = 0.0f32;
        let total_duration = ground_truth.duration;
        
        for ground_truth_segment in &ground_truth.segments {
            let mut best_overlap = 0.0f32;
            
            for detected in detected_segments {
                let overlap_start = detected.start_time.max(ground_truth_segment.start_time);
                let overlap_end = detected.end_time.min(ground_truth_segment.end_time);
                let overlap = (overlap_end - overlap_start).max(0.0);
                best_overlap = best_overlap.max(overlap);
            }
            
            total_overlap += best_overlap;
        }
        
        total_overlap / total_duration
    }

    /// Validate speaker consistency across segments
    pub fn validate_speaker_consistency(
        detected_segments: &[DetectedSegment],
        min_consistency_score: f32,
    ) -> bool {
        if detected_segments.len() < 2 {
            return true;
        }
        
        let mut speaker_segments: HashMap<String, Vec<&DetectedSegment>> = HashMap::new();
        for segment in detected_segments {
            speaker_segments.entry(segment.speaker_id.clone()).or_insert_with(Vec::new).push(segment);
        }
        
        // Check that each speaker's segments don't have large temporal gaps that might indicate inconsistent labeling
        for (_, segments) in speaker_segments {
            if segments.len() > 1 {
                let mut sorted_segments = segments;
                sorted_segments.sort_by(|a, b| a.start_time.partial_cmp(&b.start_time).unwrap());
                
                for window in sorted_segments.windows(2) {
                    let gap = window[1].start_time - window[0].end_time;
                    if gap > 30.0 { // Large gap might indicate inconsistent speaker labeling
                        return false;
                    }
                }
            }
        }
        
        true
    }
}

/// Represents a detected segment from the diarization system
#[derive(Debug, Clone)]
pub struct DetectedSegment {
    pub speaker_id: String,
    pub start_time: f32,
    pub end_time: f32,
    pub confidence: f32,
}

/// Result of validation against ground truth
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// Overall accuracy (0.0-1.0)
    pub accuracy: f32,
    /// Number of correctly detected segments
    pub correct_detections: usize,
    /// Total number of detected segments
    pub total_detections: usize,
    /// Mapping from detected speaker IDs to ground truth speaker IDs
    pub speaker_mapping: HashMap<String, String>,
    /// Temporal alignment score (0.0-1.0)
    pub temporal_alignment_score: f32,
}

impl ValidationResult {
    /// Check if the validation meets minimum quality thresholds
    pub fn meets_quality_threshold(&self, min_accuracy: f32, min_temporal_alignment: f32) -> bool {
        self.accuracy >= min_accuracy && self.temporal_alignment_score >= min_temporal_alignment
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ground_truth_segment_creation() {
        let segment = GroundTruthSegment::new(
            "speaker_0".to_string(),
            0.0,
            5.0,
            Some("Hello world".to_string()),
            0.95,
        );
        
        assert_eq!(segment.speaker_id, "speaker_0");
        assert_eq!(segment.duration(), 5.0);
        assert!(segment.matches_timing(0.0, 5.0, 0.1));
        assert!(!segment.matches_timing(0.5, 5.5, 0.1));
    }

    #[test]
    fn test_segment_overlap_detection() {
        let segment1 = GroundTruthSegment::new("speaker_0".to_string(), 0.0, 5.0, None, 0.9);
        let segment2 = GroundTruthSegment::new("speaker_1".to_string(), 4.0, 9.0, None, 0.9);
        let segment3 = GroundTruthSegment::new("speaker_2".to_string(), 10.0, 15.0, None, 0.9);
        
        assert!(segment1.overlaps_with(&segment2));
        assert!(!segment1.overlaps_with(&segment3));
    }

    #[test]
    fn test_ground_truth_data_speaker_counting() {
        let mut data = GroundTruthData::new("test.wav".to_string(), 10.0);
        
        data.add_segment(GroundTruthSegment::new("speaker_0".to_string(), 0.0, 5.0, None, 0.9));
        data.add_segment(GroundTruthSegment::new("speaker_1".to_string(), 5.0, 10.0, None, 0.9));
        data.add_segment(GroundTruthSegment::new("speaker_0".to_string(), 10.0, 15.0, None, 0.9));
        
        assert_eq!(data.total_speakers, 2);
        assert_eq!(data.speaker_ids().len(), 2);
    }

    #[test]
    fn test_scenario_generator_simple_conversation() {
        let scenario = TestScenarioGenerator::simple_two_speaker_conversation();
        
        assert_eq!(scenario.total_speakers, 2);
        assert_eq!(scenario.duration, 30.0);
        assert!(scenario.segments.len() > 0);
        assert!(scenario.metadata.contains_key("scenario_type"));
    }

    #[test]
    fn test_scenario_generator_overlapping_speech() {
        let scenario = TestScenarioGenerator::overlapping_speech_scenario();
        
        assert!(scenario.has_overlaps());
        assert_eq!(scenario.total_speakers, 2);
    }

    #[test]
    fn test_synthetic_audio_generation() {
        let scenario = TestScenarioGenerator::simple_two_speaker_conversation();
        let audio = SyntheticAudioGenerator::generate_multi_frequency_audio(&scenario, 16000);
        
        // Should have correct length
        let expected_samples = (scenario.duration * 16000.0) as usize;
        assert_eq!(audio.len(), expected_samples);
        
        // Should have non-zero audio where segments exist
        let first_segment_start = (scenario.segments[0].start_time * 16000.0) as usize;
        let first_segment_end = (scenario.segments[0].end_time * 16000.0) as usize;
        
        // Check that there's audio in the first segment
        let segment_audio: Vec<f32> = audio[first_segment_start..first_segment_end].to_vec();
        let has_audio = segment_audio.iter().any(|&sample| sample.abs() > 0.01);
        assert!(has_audio, "Generated audio should have signal in active segments");
    }

    #[test]
    fn test_validation_speaker_consistency() {
        let segments = vec![
            DetectedSegment {
                speaker_id: "speaker_0".to_string(),
                start_time: 0.0,
                end_time: 5.0,
                confidence: 0.9,
            },
            DetectedSegment {
                speaker_id: "speaker_0".to_string(),
                start_time: 10.0,
                end_time: 15.0,
                confidence: 0.8,
            },
        ];
        
        assert!(ScenarioValidator::validate_speaker_consistency(&segments, 0.8));
    }
}