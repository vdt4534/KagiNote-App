use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use std::collections::HashMap;

/// Speaker profile with voice characteristics and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeakerProfile {
    /// Unique identifier for the speaker
    pub id: Uuid,
    /// Display name for the speaker (user-assigned or auto-generated)
    pub name: String,
    /// Optional additional metadata about the speaker
    pub description: Option<String>,
    /// Color for UI visualization (hex code)
    pub color: String,
    /// Voice characteristics metadata
    pub voice_characteristics: VoiceCharacteristics,
    /// When this profile was created
    pub created_at: DateTime<Utc>,
    /// When this profile was last updated
    pub updated_at: DateTime<Utc>,
    /// Number of times this speaker has been identified
    pub identification_count: u32,
    /// Confidence threshold for automatic identification (0.0-1.0)
    pub confidence_threshold: f32,
    /// Whether this profile is active for identification
    pub is_active: bool,
}

/// Voice characteristics derived from audio analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceCharacteristics {
    /// Fundamental frequency range (Hz)
    pub pitch_range: (f32, f32),
    /// Average fundamental frequency (Hz)
    pub pitch_mean: f32,
    /// Speaking rate (words per minute)
    pub speaking_rate: Option<f32>,
    /// Voice quality indicators
    pub quality_features: HashMap<String, f32>,
    /// Gender classification (if detected)
    pub gender: Option<String>,
    /// Age range estimate (if available)
    pub age_range: Option<(u8, u8)>,
    /// Language/accent indicators
    pub language_markers: Vec<String>,
}

/// Voice embedding vector for speaker identification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceEmbedding {
    /// Unique identifier for the embedding
    pub id: Uuid,
    /// Speaker profile this embedding belongs to
    pub speaker_id: Uuid,
    /// The embedding vector (typically 512 or 256 dimensions)
    pub vector: Vec<f32>,
    /// Dimensionality of the vector
    pub dimensions: u16,
    /// Model/method used to generate this embedding
    pub model_name: String,
    /// Quality score of the audio used to generate embedding
    pub quality_score: f32,
    /// Duration of audio segment (seconds)
    pub duration_seconds: f32,
    /// When this embedding was created
    pub created_at: DateTime<Utc>,
}

/// Request for creating a new speaker profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSpeakerProfileRequest {
    pub name: String,
    pub description: Option<String>,
    pub color: Option<String>,
    pub confidence_threshold: Option<f32>,
}

/// Request for updating an existing speaker profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSpeakerProfileRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub color: Option<String>,
    pub confidence_threshold: Option<f32>,
    pub is_active: Option<bool>,
}

/// Response for speaker identification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeakerIdentification {
    /// Speaker profile if identified
    pub speaker: Option<SpeakerProfile>,
    /// Confidence score (0.0-1.0)
    pub confidence: f32,
    /// Whether this is a new speaker
    pub is_new_speaker: bool,
    /// Alternative candidates with scores
    pub alternatives: Vec<(SpeakerProfile, f32)>,
}

/// Speaker similarity search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimilarSpeaker {
    pub speaker: SpeakerProfile,
    pub similarity_score: f32,
    pub matching_embeddings: u32,
}

impl SpeakerProfile {
    /// Create a new speaker profile with default values
    pub fn new(name: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            description: None,
            color: Self::generate_color(),
            voice_characteristics: VoiceCharacteristics::default(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            identification_count: 0,
            confidence_threshold: 0.7,
            is_active: true,
        }
    }

    /// Generate a random color for speaker visualization
    fn generate_color() -> String {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let colors = [
            "#3B82F6", "#EF4444", "#10B981", "#F59E0B", 
            "#8B5CF6", "#EC4899", "#06B6D4", "#84CC16",
            "#F97316", "#6366F1", "#14B8A6", "#F472B6"
        ];
        colors[rng.gen_range(0..colors.len())].to_string()
    }

    /// Update the profile's updated_at timestamp
    pub fn touch(&mut self) {
        self.updated_at = Utc::now();
    }

    /// Increment identification count
    pub fn increment_identification(&mut self) {
        self.identification_count += 1;
        self.touch();
    }
}

impl Default for VoiceCharacteristics {
    fn default() -> Self {
        Self {
            pitch_range: (80.0, 300.0),
            pitch_mean: 150.0,
            speaking_rate: None,
            quality_features: HashMap::new(),
            gender: None,
            age_range: None,
            language_markers: Vec::new(),
        }
    }
}

impl VoiceEmbedding {
    /// Create a new voice embedding
    pub fn new(
        speaker_id: Uuid,
        vector: Vec<f32>,
        model_name: String,
        quality_score: f32,
        duration_seconds: f32,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            speaker_id,
            dimensions: vector.len() as u16,
            vector,
            model_name,
            quality_score,
            duration_seconds,
            created_at: Utc::now(),
        }
    }

    /// Calculate cosine similarity with another embedding
    pub fn cosine_similarity(&self, other: &VoiceEmbedding) -> f32 {
        if self.vector.len() != other.vector.len() {
            return 0.0;
        }

        let dot_product: f32 = self.vector
            .iter()
            .zip(other.vector.iter())
            .map(|(a, b)| a * b)
            .sum();

        let norm_a: f32 = self.vector.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = other.vector.iter().map(|x| x * x).sum::<f32>().sqrt();

        if norm_a == 0.0 || norm_b == 0.0 {
            return 0.0;
        }

        dot_product / (norm_a * norm_b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_speaker_profile_creation() {
        let profile = SpeakerProfile::new("John Doe".to_string());
        assert_eq!(profile.name, "John Doe");
        assert!(profile.is_active);
        assert_eq!(profile.identification_count, 0);
        assert_eq!(profile.confidence_threshold, 0.7);
        assert!(profile.color.starts_with('#'));
    }

    #[test]
    fn test_voice_embedding_similarity() {
        let speaker_id = Uuid::new_v4();
        let embedding1 = VoiceEmbedding::new(
            speaker_id,
            vec![1.0, 0.0, 0.0],
            "test_model".to_string(),
            0.9,
            3.0,
        );
        let embedding2 = VoiceEmbedding::new(
            speaker_id,
            vec![0.0, 1.0, 0.0],
            "test_model".to_string(),
            0.9,
            3.0,
        );

        let similarity = embedding1.cosine_similarity(&embedding2);
        assert!((similarity - 0.0).abs() < 0.01); // Should be orthogonal
    }

    #[test]
    fn test_profile_increment_identification() {
        let mut profile = SpeakerProfile::new("Test Speaker".to_string());
        let initial_count = profile.identification_count;
        let initial_time = profile.updated_at;
        
        std::thread::sleep(std::time::Duration::from_millis(10));
        profile.increment_identification();
        
        assert_eq!(profile.identification_count, initial_count + 1);
        assert!(profile.updated_at > initial_time);
    }
}