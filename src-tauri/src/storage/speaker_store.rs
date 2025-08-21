use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use rusqlite::Row;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::task;
use uuid::Uuid;

use crate::models::{
    SpeakerProfile, VoiceCharacteristics, VoiceEmbedding, MeetingSpeaker,
    SpeakerStats, MeetingStats, SimilarSpeaker,
    CreateSpeakerProfileRequest, UpdateSpeakerProfileRequest,
};
use crate::storage::{Database, vector_to_blob, blob_to_vector, uuid_to_string, string_to_uuid};

/// Speaker storage operations
pub struct SpeakerStore {
    db: Database,
}

impl SpeakerStore {
    /// Create a new speaker store
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    /// Create a new speaker profile
    pub async fn create_speaker_profile(
        &self,
        request: CreateSpeakerProfileRequest,
    ) -> Result<SpeakerProfile> {
        let mut profile = SpeakerProfile::new(request.name);
        
        if let Some(description) = request.description {
            profile.description = Some(description);
        }
        if let Some(color) = request.color {
            profile.color = color;
        }
        if let Some(threshold) = request.confidence_threshold {
            profile.confidence_threshold = threshold;
        }

        let connection = Arc::clone(&self.db.connection);
        let profile_clone = profile.clone();

        task::spawn_blocking(move || -> Result<()> {
            let conn = connection.lock().unwrap();
            
            conn.execute(
                "INSERT INTO speaker_profiles (
                    id, name, description, color, created_at, updated_at,
                    identification_count, confidence_threshold, is_active,
                    pitch_range_min, pitch_range_max, pitch_mean, speaking_rate,
                    quality_features, gender, age_range_min, age_range_max,
                    language_markers
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18)",
                [
                    &uuid_to_string(&profile_clone.id),
                    &profile_clone.name,
                    &profile_clone.description.unwrap_or_default(),
                    &profile_clone.color,
                    &profile_clone.created_at.to_rfc3339(),
                    &profile_clone.updated_at.to_rfc3339(),
                    &profile_clone.identification_count.to_string(),
                    &profile_clone.confidence_threshold.to_string(),
                    &(profile_clone.is_active as i32).to_string(),
                    &profile_clone.voice_characteristics.pitch_range.0.to_string(),
                    &profile_clone.voice_characteristics.pitch_range.1.to_string(),
                    &profile_clone.voice_characteristics.pitch_mean.to_string(),
                    &profile_clone.voice_characteristics.speaking_rate
                        .map(|r| r.to_string())
                        .unwrap_or_default(),
                    &serde_json::to_string(&profile_clone.voice_characteristics.quality_features)
                        .unwrap_or_default(),
                    &profile_clone.voice_characteristics.gender.unwrap_or_default(),
                    &profile_clone.voice_characteristics.age_range
                        .map(|r| r.0.to_string())
                        .unwrap_or_default(),
                    &profile_clone.voice_characteristics.age_range
                        .map(|r| r.1.to_string())
                        .unwrap_or_default(),
                    &serde_json::to_string(&profile_clone.voice_characteristics.language_markers)
                        .unwrap_or_default(),
                ],
            ).context("Failed to insert speaker profile")?;

            Ok(())
        }).await??;

        Ok(profile)
    }

    /// Get speaker profile by ID
    pub async fn get_speaker_profile(&self, speaker_id: Uuid) -> Result<Option<SpeakerProfile>> {
        let connection = Arc::clone(&self.db.connection);
        let speaker_id_str = uuid_to_string(&speaker_id);

        task::spawn_blocking(move || -> Result<Option<SpeakerProfile>> {
            let conn = connection.lock().unwrap();
            
            let mut stmt = conn.prepare(
                "SELECT id, name, description, color, created_at, updated_at,
                        identification_count, confidence_threshold, is_active,
                        pitch_range_min, pitch_range_max, pitch_mean, speaking_rate,
                        quality_features, gender, age_range_min, age_range_max,
                        language_markers
                 FROM speaker_profiles WHERE id = ?1"
            )?;

            let profile = stmt.query_row([&speaker_id_str], |row| {
                Ok(row_to_speaker_profile(row)?)
            });

            match profile {
                Ok(p) => Ok(Some(p)),
                Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                Err(e) => Err(e.into()),
            }
        }).await?
    }

    /// Update speaker profile
    pub async fn update_speaker_profile(
        &self,
        speaker_id: Uuid,
        request: UpdateSpeakerProfileRequest,
    ) -> Result<Option<SpeakerProfile>> {
        let connection = Arc::clone(&self.db.connection);
        let speaker_id_str = uuid_to_string(&speaker_id);

        task::spawn_blocking(move || -> Result<Option<SpeakerProfile>> {
            let conn = connection.lock().unwrap();
            
            let mut updates = Vec::new();
            let mut params: Vec<String> = Vec::new();

            if let Some(name) = request.name {
                updates.push("name = ?");
                params.push(name);
            }
            if let Some(description) = request.description {
                updates.push("description = ?");
                params.push(description);
            }
            if let Some(color) = request.color {
                updates.push("color = ?");
                params.push(color);
            }
            if let Some(threshold) = request.confidence_threshold {
                updates.push("confidence_threshold = ?");
                params.push(threshold.to_string());
            }
            if let Some(is_active) = request.is_active {
                updates.push("is_active = ?");
                params.push((is_active as i32).to_string());
            }

            if updates.is_empty() {
                // No updates requested, just return current profile
                return Ok(None);
            }

            updates.push("updated_at = ?");
            params.push(Utc::now().to_rfc3339());
            params.push(speaker_id_str.clone());

            let sql = format!(
                "UPDATE speaker_profiles SET {} WHERE id = ?",
                updates.join(", ")
            );

            let rows_affected = conn.execute(&sql, rusqlite::params_from_iter(params))?;
            
            if rows_affected == 0 {
                return Ok(None);
            }

            // Return updated profile
            let mut stmt = conn.prepare(
                "SELECT id, name, description, color, created_at, updated_at,
                        identification_count, confidence_threshold, is_active,
                        pitch_range_min, pitch_range_max, pitch_mean, speaking_rate,
                        quality_features, gender, age_range_min, age_range_max,
                        language_markers
                 FROM speaker_profiles WHERE id = ?1"
            )?;

            let profile = stmt.query_row([&speaker_id_str], |row| {
                Ok(row_to_speaker_profile(row)?)
            })?;

            Ok(Some(profile))
        }).await?
    }

    /// List all speaker profiles
    pub async fn list_speaker_profiles(&self, active_only: bool) -> Result<Vec<SpeakerProfile>> {
        let connection = Arc::clone(&self.db.connection);

        task::spawn_blocking(move || -> Result<Vec<SpeakerProfile>> {
            let conn = connection.lock().unwrap();
            
            let sql = if active_only {
                "SELECT id, name, description, color, created_at, updated_at,
                        identification_count, confidence_threshold, is_active,
                        pitch_range_min, pitch_range_max, pitch_mean, speaking_rate,
                        quality_features, gender, age_range_min, age_range_max,
                        language_markers
                 FROM speaker_profiles WHERE is_active = 1 ORDER BY name"
            } else {
                "SELECT id, name, description, color, created_at, updated_at,
                        identification_count, confidence_threshold, is_active,
                        pitch_range_min, pitch_range_max, pitch_mean, speaking_rate,
                        quality_features, gender, age_range_min, age_range_max,
                        language_markers
                 FROM speaker_profiles ORDER BY name"
            };

            let mut stmt = conn.prepare(sql)?;
            let profiles = stmt.query_map([], |row| {
                Ok(row_to_speaker_profile(row)?)
            })?
            .collect::<Result<Vec<_>, _>>()?;

            Ok(profiles)
        }).await?
    }

    /// Delete speaker profile and all associated data
    pub async fn delete_speaker_profile(&self, speaker_id: Uuid) -> Result<bool> {
        let speaker_id_str = uuid_to_string(&speaker_id);
        let rows_affected = self.db.execute(
            "DELETE FROM speaker_profiles WHERE id = ?1",
            [&speaker_id_str],
        ).await?;

        Ok(rows_affected > 0)
    }

    /// Add voice embedding for a speaker
    pub async fn add_voice_embedding(
        &self,
        embedding: VoiceEmbedding,
    ) -> Result<()> {
        let connection = Arc::clone(&self.db.connection);
        let embedding_clone = embedding.clone();

        task::spawn_blocking(move || -> Result<()> {
            let conn = connection.lock().unwrap();
            
            let vector_blob = vector_to_blob(&embedding_clone.vector);
            
            conn.execute(
                "INSERT INTO voice_embeddings (
                    id, speaker_id, vector, dimensions, model_name,
                    quality_score, duration_seconds, created_at
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                [
                    &uuid_to_string(&embedding_clone.id),
                    &uuid_to_string(&embedding_clone.speaker_id),
                    &vector_blob,
                    &embedding_clone.dimensions.to_string(),
                    &embedding_clone.model_name,
                    &embedding_clone.quality_score.to_string(),
                    &embedding_clone.duration_seconds.to_string(),
                    &embedding_clone.created_at.to_rfc3339(),
                ],
            ).context("Failed to insert voice embedding")?;

            Ok(())
        }).await?
    }

    /// Get voice embeddings for a speaker
    pub async fn get_voice_embeddings(&self, speaker_id: Uuid) -> Result<Vec<VoiceEmbedding>> {
        let connection = Arc::clone(&self.db.connection);
        let speaker_id_str = uuid_to_string(&speaker_id);

        task::spawn_blocking(move || -> Result<Vec<VoiceEmbedding>> {
            let conn = connection.lock().unwrap();
            
            let mut stmt = conn.prepare(
                "SELECT id, speaker_id, vector, dimensions, model_name,
                        quality_score, duration_seconds, created_at
                 FROM voice_embeddings WHERE speaker_id = ?1
                 ORDER BY quality_score DESC, created_at DESC"
            )?;

            let embeddings = stmt.query_map([&speaker_id_str], |row| {
                Ok(row_to_voice_embedding(row)?)
            })?
            .collect::<Result<Vec<_>, _>>()?;

            Ok(embeddings)
        }).await?
    }

    /// Search for similar speakers based on embedding
    pub async fn find_similar_speakers(
        &self,
        query_vector: Vec<f32>,
        threshold: f32,
        limit: usize,
    ) -> Result<Vec<SimilarSpeaker>> {
        let connection = Arc::clone(&self.db.connection);

        task::spawn_blocking(move || -> Result<Vec<SimilarSpeaker>> {
            let conn = connection.lock().unwrap();
            
            // Get all embeddings with speaker profiles
            let mut stmt = conn.prepare(
                "SELECT e.vector, e.speaker_id, e.quality_score,
                        p.id, p.name, p.description, p.color, p.created_at, p.updated_at,
                        p.identification_count, p.confidence_threshold, p.is_active,
                        p.pitch_range_min, p.pitch_range_max, p.pitch_mean, p.speaking_rate,
                        p.quality_features, p.gender, p.age_range_min, p.age_range_max,
                        p.language_markers
                 FROM voice_embeddings e
                 JOIN speaker_profiles p ON e.speaker_id = p.id
                 WHERE p.is_active = 1
                 ORDER BY e.quality_score DESC"
            )?;

            let mut results: HashMap<Uuid, (SpeakerProfile, f32, u32)> = HashMap::new();

            stmt.query_map([], |row| {
                let vector_blob: Vec<u8> = row.get("vector")?;
                let embedding_vector = blob_to_vector(&vector_blob)
                    .map_err(|e| rusqlite::Error::InvalidColumnType(0, "vector".to_string(), rusqlite::types::Type::Blob))?;
                
                // Calculate cosine similarity
                let similarity = cosine_similarity(&query_vector, &embedding_vector);
                
                if similarity >= threshold {
                    let speaker_id = string_to_uuid(&row.get::<_, String>("speaker_id")?)
                        .map_err(|_| rusqlite::Error::InvalidColumnType(0, "speaker_id".to_string(), rusqlite::types::Type::Text))?;
                    
                    let profile = row_to_speaker_profile_from_join(row)?;
                    
                    // Aggregate results per speaker (keep best similarity)
                    results.entry(speaker_id)
                        .and_modify(|(_, best_sim, count)| {
                            *best_sim = best_sim.max(similarity);
                            *count += 1;
                        })
                        .or_insert((profile, similarity, 1));
                }
                
                Ok(())
            })?.collect::<Result<Vec<_>, _>>()?;

            // Convert to SimilarSpeaker results and sort by similarity
            let mut similar_speakers: Vec<SimilarSpeaker> = results
                .into_iter()
                .map(|(_, (speaker, similarity, matching_embeddings))| SimilarSpeaker {
                    speaker,
                    similarity_score: similarity,
                    matching_embeddings,
                })
                .collect();

            similar_speakers.sort_by(|a, b| 
                b.similarity_score.partial_cmp(&a.similarity_score).unwrap_or(std::cmp::Ordering::Equal)
            );

            similar_speakers.truncate(limit);
            Ok(similar_speakers)
        }).await?
    }
}

/// Convert database row to SpeakerProfile
fn row_to_speaker_profile(row: &Row) -> Result<SpeakerProfile, rusqlite::Error> {
    let quality_features_str: String = row.get("quality_features")?;
    let quality_features: HashMap<String, f32> = serde_json::from_str(&quality_features_str)
        .unwrap_or_default();

    let language_markers_str: String = row.get("language_markers")?;
    let language_markers: Vec<String> = serde_json::from_str(&language_markers_str)
        .unwrap_or_default();

    let age_range = {
        let age_min: String = row.get("age_range_min")?;
        let age_max: String = row.get("age_range_max")?;
        if age_min.is_empty() || age_max.is_empty() {
            None
        } else {
            Some((age_min.parse().unwrap_or(0), age_max.parse().unwrap_or(100)))
        }
    };

    let speaking_rate: String = row.get("speaking_rate")?;
    let speaking_rate = if speaking_rate.is_empty() {
        None
    } else {
        speaking_rate.parse().ok()
    };

    Ok(SpeakerProfile {
        id: string_to_uuid(&row.get::<_, String>("id")?)
            .map_err(|_| rusqlite::Error::InvalidColumnType(0, "id".to_string(), rusqlite::types::Type::Text))?,
        name: row.get("name")?,
        description: {
            let desc: String = row.get("description")?;
            if desc.is_empty() { None } else { Some(desc) }
        },
        color: row.get("color")?,
        created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>("created_at")?)
            .map_err(|_| rusqlite::Error::InvalidColumnType(0, "created_at".to_string(), rusqlite::types::Type::Text))?
            .with_timezone(&Utc),
        updated_at: DateTime::parse_from_rfc3339(&row.get::<_, String>("updated_at")?)
            .map_err(|_| rusqlite::Error::InvalidColumnType(0, "updated_at".to_string(), rusqlite::types::Type::Text))?
            .with_timezone(&Utc),
        identification_count: row.get::<_, String>("identification_count")?.parse().unwrap_or(0),
        confidence_threshold: row.get::<_, String>("confidence_threshold")?.parse().unwrap_or(0.7),
        is_active: row.get::<_, String>("is_active")?.parse::<i32>().unwrap_or(1) != 0,
        voice_characteristics: VoiceCharacteristics {
            pitch_range: (
                row.get::<_, String>("pitch_range_min")?.parse().unwrap_or(80.0),
                row.get::<_, String>("pitch_range_max")?.parse().unwrap_or(300.0),
            ),
            pitch_mean: row.get::<_, String>("pitch_mean")?.parse().unwrap_or(150.0),
            speaking_rate,
            quality_features,
            gender: {
                let gender: String = row.get("gender")?;
                if gender.is_empty() { None } else { Some(gender) }
            },
            age_range,
            language_markers,
        },
    })
}

/// Convert database row from joined query to SpeakerProfile
fn row_to_speaker_profile_from_join(row: &Row) -> Result<SpeakerProfile, rusqlite::Error> {
    // This is the same as row_to_speaker_profile but with joined column names
    row_to_speaker_profile(row)
}

/// Convert database row to VoiceEmbedding
fn row_to_voice_embedding(row: &Row) -> Result<VoiceEmbedding, rusqlite::Error> {
    let vector_blob: Vec<u8> = row.get("vector")?;
    let vector = blob_to_vector(&vector_blob)
        .map_err(|_| rusqlite::Error::InvalidColumnType(0, "vector".to_string(), rusqlite::types::Type::Blob))?;

    Ok(VoiceEmbedding {
        id: string_to_uuid(&row.get::<_, String>("id")?)
            .map_err(|_| rusqlite::Error::InvalidColumnType(0, "id".to_string(), rusqlite::types::Type::Text))?,
        speaker_id: string_to_uuid(&row.get::<_, String>("speaker_id")?)
            .map_err(|_| rusqlite::Error::InvalidColumnType(0, "speaker_id".to_string(), rusqlite::types::Type::Text))?,
        vector,
        dimensions: row.get::<_, String>("dimensions")?.parse().unwrap_or(0),
        model_name: row.get("model_name")?,
        quality_score: row.get::<_, String>("quality_score")?.parse().unwrap_or(0.0),
        duration_seconds: row.get::<_, String>("duration_seconds")?.parse().unwrap_or(0.0),
        created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>("created_at")?)
            .map_err(|_| rusqlite::Error::InvalidColumnType(0, "created_at".to_string(), rusqlite::types::Type::Text))?
            .with_timezone(&Utc),
    })
}

/// Calculate cosine similarity between two vectors
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() {
        return 0.0;
    }

    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }

    dot_product / (norm_a * norm_b)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::Database;
    use tempfile::NamedTempFile;

    async fn create_test_store() -> (SpeakerStore, NamedTempFile) {
        let temp_file = NamedTempFile::new().unwrap();
        let db = Database::new(temp_file.path()).await.unwrap();
        db.migrate().await.unwrap();
        let store = SpeakerStore::new(db);
        (store, temp_file)
    }

    #[tokio::test]
    async fn test_create_and_get_speaker_profile() {
        let (store, _temp_file) = create_test_store().await;

        let request = CreateSpeakerProfileRequest {
            name: "John Doe".to_string(),
            description: Some("Test speaker".to_string()),
            color: Some("#FF5733".to_string()),
            confidence_threshold: Some(0.8),
        };

        let profile = store.create_speaker_profile(request).await.unwrap();
        assert_eq!(profile.name, "John Doe");
        assert_eq!(profile.description, Some("Test speaker".to_string()));
        assert_eq!(profile.color, "#FF5733");
        assert_eq!(profile.confidence_threshold, 0.8);

        let retrieved = store.get_speaker_profile(profile.id).await.unwrap();
        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.id, profile.id);
        assert_eq!(retrieved.name, profile.name);
    }

    #[tokio::test]
    async fn test_voice_embedding_operations() {
        let (store, _temp_file) = create_test_store().await;

        // Create speaker first
        let request = CreateSpeakerProfileRequest {
            name: "Test Speaker".to_string(),
            description: None,
            color: None,
            confidence_threshold: None,
        };
        let profile = store.create_speaker_profile(request).await.unwrap();

        // Add embedding
        let embedding = VoiceEmbedding::new(
            profile.id,
            vec![1.0, 0.5, -0.3, 2.1],
            "test_model".to_string(),
            0.9,
            5.0,
        );

        store.add_voice_embedding(embedding.clone()).await.unwrap();

        // Retrieve embeddings
        let embeddings = store.get_voice_embeddings(profile.id).await.unwrap();
        assert_eq!(embeddings.len(), 1);
        assert_eq!(embeddings[0].speaker_id, profile.id);
        assert_eq!(embeddings[0].vector, vec![1.0, 0.5, -0.3, 2.1]);
    }

    #[tokio::test]
    async fn test_similarity_search() {
        let (store, _temp_file) = create_test_store().await;

        // Create test speaker
        let request = CreateSpeakerProfileRequest {
            name: "Similar Speaker".to_string(),
            description: None,
            color: None,
            confidence_threshold: None,
        };
        let profile = store.create_speaker_profile(request).await.unwrap();

        // Add similar embedding
        let embedding = VoiceEmbedding::new(
            profile.id,
            vec![1.0, 0.0, 0.0], // Unit vector along x-axis
            "test_model".to_string(),
            0.9,
            5.0,
        );

        store.add_voice_embedding(embedding).await.unwrap();

        // Search with similar vector
        let query_vector = vec![0.9, 0.1, 0.0]; // Very similar vector
        let results = store.find_similar_speakers(query_vector, 0.5, 10).await.unwrap();

        assert!(!results.is_empty());
        assert_eq!(results[0].speaker.id, profile.id);
        assert!(results[0].similarity_score > 0.8);
    }
}