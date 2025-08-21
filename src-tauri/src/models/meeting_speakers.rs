use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Junction table connecting meetings with speakers and their statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeetingSpeaker {
    /// Unique identifier for this meeting-speaker relationship
    pub id: Uuid,
    /// Meeting identifier
    pub meeting_id: Uuid,
    /// Speaker profile identifier
    pub speaker_id: Uuid,
    /// Display name used in this specific meeting (may differ from profile name)
    pub display_name: String,
    /// Total speaking time in seconds
    pub speaking_time_seconds: f32,
    /// Number of segments spoken by this speaker
    pub segment_count: u32,
    /// Average confidence score for speaker identification
    pub average_confidence: f32,
    /// First time this speaker was identified in the meeting
    pub first_spoken_at: DateTime<Utc>,
    /// Last time this speaker was identified in the meeting
    pub last_spoken_at: DateTime<Utc>,
    /// Whether this speaker was manually verified by user
    pub is_verified: bool,
    /// Notes about this speaker in this meeting
    pub notes: Option<String>,
}

/// Statistics for a speaker across multiple meetings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeakerStats {
    pub speaker_id: Uuid,
    pub speaker_name: String,
    pub total_meetings: u32,
    pub total_speaking_time_seconds: f32,
    pub total_segments: u32,
    pub average_confidence: f32,
    pub first_meeting_date: DateTime<Utc>,
    pub last_meeting_date: DateTime<Utc>,
}

/// Meeting statistics with speaker breakdown
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeetingStats {
    pub meeting_id: Uuid,
    pub total_duration_seconds: f32,
    pub total_speakers: u32,
    pub speakers: Vec<MeetingSpeakerStats>,
    pub created_at: DateTime<Utc>,
}

/// Individual speaker statistics within a meeting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeetingSpeakerStats {
    pub speaker_id: Uuid,
    pub display_name: String,
    pub speaking_time_seconds: f32,
    pub speaking_percentage: f32,
    pub segment_count: u32,
    pub average_confidence: f32,
    pub is_verified: bool,
}

/// Request to create or update a meeting speaker relationship
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpsertMeetingSpeakerRequest {
    pub meeting_id: Uuid,
    pub speaker_id: Uuid,
    pub display_name: Option<String>,
    pub speaking_time_seconds: Option<f32>,
    pub segment_count: Option<u32>,
    pub confidence_score: Option<f32>,
    pub is_verified: Option<bool>,
    pub notes: Option<String>,
}

/// Response for speaker participation query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeakerParticipation {
    pub speaker: crate::models::SpeakerProfile,
    pub meeting_speaker: MeetingSpeaker,
    pub participation_percentage: f32,
}

impl MeetingSpeaker {
    /// Create a new meeting-speaker relationship
    pub fn new(
        meeting_id: Uuid,
        speaker_id: Uuid,
        display_name: String,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            meeting_id,
            speaker_id,
            display_name,
            speaking_time_seconds: 0.0,
            segment_count: 0,
            average_confidence: 0.0,
            first_spoken_at: now,
            last_spoken_at: now,
            is_verified: false,
            notes: None,
        }
    }

    /// Update speaking statistics
    pub fn update_speaking_stats(
        &mut self,
        additional_time: f32,
        confidence_score: f32,
    ) {
        self.speaking_time_seconds += additional_time;
        self.segment_count += 1;
        
        // Update average confidence using running average
        let total_segments = self.segment_count as f32;
        self.average_confidence = (
            (self.average_confidence * (total_segments - 1.0)) + confidence_score
        ) / total_segments;
        
        self.last_spoken_at = Utc::now();
    }

    /// Calculate speaking percentage within a meeting
    pub fn speaking_percentage(&self, total_meeting_duration: f32) -> f32 {
        if total_meeting_duration <= 0.0 {
            return 0.0;
        }
        (self.speaking_time_seconds / total_meeting_duration) * 100.0
    }

    /// Mark as verified by user
    pub fn verify(&mut self, notes: Option<String>) {
        self.is_verified = true;
        self.notes = notes;
    }
}

impl MeetingStats {
    /// Create meeting statistics from speaker data
    pub fn from_speakers(
        meeting_id: Uuid,
        total_duration: f32,
        meeting_speakers: Vec<MeetingSpeaker>,
    ) -> Self {
        let speakers: Vec<MeetingSpeakerStats> = meeting_speakers
            .iter()
            .map(|ms| MeetingSpeakerStats {
                speaker_id: ms.speaker_id,
                display_name: ms.display_name.clone(),
                speaking_time_seconds: ms.speaking_time_seconds,
                speaking_percentage: ms.speaking_percentage(total_duration),
                segment_count: ms.segment_count,
                average_confidence: ms.average_confidence,
                is_verified: ms.is_verified,
            })
            .collect();

        Self {
            meeting_id,
            total_duration_seconds: total_duration,
            total_speakers: speakers.len() as u32,
            speakers,
            created_at: Utc::now(),
        }
    }

    /// Get dominant speaker (highest speaking time)
    pub fn dominant_speaker(&self) -> Option<&MeetingSpeakerStats> {
        self.speakers
            .iter()
            .max_by(|a, b| a.speaking_time_seconds.partial_cmp(&b.speaking_time_seconds).unwrap())
    }

    /// Get speakers sorted by speaking time (descending)
    pub fn speakers_by_time(&self) -> Vec<&MeetingSpeakerStats> {
        let mut speakers: Vec<&MeetingSpeakerStats> = self.speakers.iter().collect();
        speakers.sort_by(|a, b| 
            b.speaking_time_seconds.partial_cmp(&a.speaking_time_seconds).unwrap()
        );
        speakers
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_meeting_speaker_creation() {
        let meeting_id = Uuid::new_v4();
        let speaker_id = Uuid::new_v4();
        let meeting_speaker = MeetingSpeaker::new(
            meeting_id,
            speaker_id,
            "John Doe".to_string(),
        );

        assert_eq!(meeting_speaker.meeting_id, meeting_id);
        assert_eq!(meeting_speaker.speaker_id, speaker_id);
        assert_eq!(meeting_speaker.display_name, "John Doe");
        assert_eq!(meeting_speaker.speaking_time_seconds, 0.0);
        assert_eq!(meeting_speaker.segment_count, 0);
        assert!(!meeting_speaker.is_verified);
    }

    #[test]
    fn test_speaking_stats_update() {
        let mut meeting_speaker = MeetingSpeaker::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            "Test Speaker".to_string(),
        );

        meeting_speaker.update_speaking_stats(30.0, 0.8);
        assert_eq!(meeting_speaker.speaking_time_seconds, 30.0);
        assert_eq!(meeting_speaker.segment_count, 1);
        assert_eq!(meeting_speaker.average_confidence, 0.8);

        meeting_speaker.update_speaking_stats(15.0, 0.9);
        assert_eq!(meeting_speaker.speaking_time_seconds, 45.0);
        assert_eq!(meeting_speaker.segment_count, 2);
        assert_eq!(meeting_speaker.average_confidence, 0.85); // (0.8 + 0.9) / 2
    }

    #[test]
    fn test_speaking_percentage() {
        let mut meeting_speaker = MeetingSpeaker::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            "Test Speaker".to_string(),
        );
        meeting_speaker.speaking_time_seconds = 120.0; // 2 minutes

        let percentage = meeting_speaker.speaking_percentage(600.0); // 10 minutes total
        assert!((percentage - 20.0).abs() < 0.01); // Should be 20%
    }

    #[test]
    fn test_meeting_stats_creation() {
        let meeting_id = Uuid::new_v4();
        let mut speaker1 = MeetingSpeaker::new(
            meeting_id,
            Uuid::new_v4(),
            "Speaker 1".to_string(),
        );
        speaker1.speaking_time_seconds = 180.0; // 3 minutes
        
        let mut speaker2 = MeetingSpeaker::new(
            meeting_id,
            Uuid::new_v4(),
            "Speaker 2".to_string(),
        );
        speaker2.speaking_time_seconds = 120.0; // 2 minutes

        let stats = MeetingStats::from_speakers(
            meeting_id,
            300.0, // 5 minutes total
            vec![speaker1, speaker2],
        );

        assert_eq!(stats.total_speakers, 2);
        assert_eq!(stats.total_duration_seconds, 300.0);
        
        let dominant = stats.dominant_speaker().unwrap();
        assert_eq!(dominant.display_name, "Speaker 1");
        assert!((dominant.speaking_percentage - 60.0).abs() < 0.01);
    }
}