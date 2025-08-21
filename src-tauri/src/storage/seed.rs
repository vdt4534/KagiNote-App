use anyhow::{Context, Result};
use std::sync::Arc;
use tokio::task;
use rand::{Rng, SeedableRng};

use crate::storage::Database;

/// Seed data management for testing and development
pub struct SeedManager {
    db: Database,
}

impl SeedManager {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    /// Load seed data for testing
    pub async fn load_test_data(&self) -> Result<String> {
        let connection = Arc::clone(&self.db.connection);
        
        task::spawn_blocking(move || -> Result<String> {
            let conn = connection.lock().unwrap();
            
            // Load test speaker seed data
            let seed_sql = include_str!("../../seeds/001_test_speakers.sql");
            
            conn.execute_batch(seed_sql)
                .context("Failed to load test seed data")?;
            
            // Get counts to verify data was loaded
            let profile_count: i64 = conn
                .prepare("SELECT COUNT(*) FROM speaker_profiles")?
                .query_row([], |row| row.get(0))?;
                
            let embedding_count: i64 = conn
                .prepare("SELECT COUNT(*) FROM voice_embeddings")?
                .query_row([], |row| row.get(0))?;
                
            let meeting_speaker_count: i64 = conn
                .prepare("SELECT COUNT(*) FROM meeting_speakers")?
                .query_row([], |row| row.get(0))?;

            let summary = format!(
                "Seed data loaded successfully:\n- {} speaker profiles\n- {} voice embeddings\n- {} meeting speakers",
                profile_count, embedding_count, meeting_speaker_count
            );

            tracing::info!("{}", summary);
            Ok(summary)
        }).await?
    }

    /// Clear all data (for testing)
    pub async fn clear_all_data(&self) -> Result<String> {
        let connection = Arc::clone(&self.db.connection);
        
        task::spawn_blocking(move || -> Result<String> {
            let conn = connection.lock().unwrap();
            
            // Clear all tables in dependency order
            conn.execute("DELETE FROM meeting_speakers", [])?;
            conn.execute("DELETE FROM voice_embeddings", [])?;
            conn.execute("DELETE FROM speaker_profiles", [])?;
            
            // Reset auto-increment counters if any
            conn.execute("DELETE FROM sqlite_sequence WHERE name IN ('speaker_profiles', 'voice_embeddings', 'meeting_speakers')", [])?;
            
            let summary = "All speaker data cleared successfully".to_string();
            tracing::info!("{}", summary);
            Ok(summary)
        }).await?
    }

    /// Load custom seed data from SQL string
    pub async fn load_custom_seed(&self, sql: &str, description: &str) -> Result<String> {
        let connection = Arc::clone(&self.db.connection);
        let sql = sql.to_string();
        let description = description.to_string();
        
        task::spawn_blocking(move || -> Result<String> {
            let conn = connection.lock().unwrap();
            
            conn.execute_batch(&sql)
                .with_context(|| format!("Failed to load custom seed data: {}", description))?;
            
            let summary = format!("Custom seed data loaded successfully: {}", description);
            tracing::info!("{}", summary);
            Ok(summary)
        }).await?
    }

    /// Generate random test embeddings for a speaker
    pub fn generate_test_embedding(speaker_id: &str, dimensions: usize, quality: f32) -> Vec<u8> {
        let mut rng = rand::thread_rng();
        let mut vector_bytes = Vec::with_capacity(dimensions * 4);
        
        // Generate normalized random vector
        let mut vector: Vec<f32> = (0..dimensions)
            .map(|_| rng.gen_range(-1.0..1.0))
            .collect();
        
        // Normalize the vector
        let magnitude: f32 = vector.iter().map(|x| x * x).sum::<f32>().sqrt();
        if magnitude > 0.0 {
            for value in &mut vector {
                *value /= magnitude;
            }
        }
        
        // Add some speaker-specific bias to make embeddings more realistic
        let speaker_seed = speaker_id.chars().map(|c| c as u32).sum::<u32>();
        let mut bias_rng = rand::rngs::StdRng::seed_from_u64(speaker_seed as u64);
        let bias_strength = quality * 0.3; // Higher quality = more consistent embeddings
        
        for (i, value) in vector.iter_mut().enumerate() {
            let bias = bias_rng.gen_range(-bias_strength..bias_strength);
            *value = (*value + bias).clamp(-1.0, 1.0);
        }
        
        // Convert to bytes
        for value in vector {
            vector_bytes.extend_from_slice(&value.to_le_bytes());
        }
        
        vector_bytes
    }

    /// Create a comprehensive test dataset
    pub async fn create_comprehensive_test_dataset(&self) -> Result<String> {
        let connection = Arc::clone(&self.db.connection);
        
        task::spawn_blocking(move || -> Result<String> {
            let conn = connection.lock().unwrap();
            let tx = conn.unchecked_transaction()?;
            
            // Clear existing data first
            tx.execute("DELETE FROM meeting_speakers", [])?;
            tx.execute("DELETE FROM voice_embeddings", [])?;
            tx.execute("DELETE FROM speaker_profiles", [])?;
            
            let mut profile_count = 0;
            let mut embedding_count = 0;
            let mut meeting_count = 0;
            
            // Create diverse speaker profiles
            let speakers = vec![
                ("Alice Cooper", "Executive", "#3B82F6", "female", (25, 35), vec!["en-US", "professional"]),
                ("Bob Wilson", "Engineer", "#EF4444", "male", (30, 40), vec!["en-US", "technical"]),
                ("Carlos Rodriguez", "Designer", "#10B981", "male", (28, 35), vec!["es-ES", "en-US", "creative"]),
                ("Diana Prince", "Manager", "#F59E0B", "female", (35, 45), vec!["en-GB", "authoritative"]),
                ("Erik Johansson", "Analyst", "#8B5CF6", "male", (26, 32), vec!["sv-SE", "en-US", "analytical"]),
                ("Fiona MacLeod", "Consultant", "#EC4899", "female", (40, 50), vec!["en-GB", "advisory"]),
                ("George Kim", "Developer", "#06B6D4", "male", (24, 30), vec!["ko-KR", "en-US", "technical"]),
                ("Hannah Torres", "Writer", "#84CC16", "female", (29, 38), vec!["en-US", "creative"]),
                ("Ivan Petrov", "Researcher", "#F97316", "male", (35, 45), vec!["ru-RU", "en-US", "academic"]),
                ("Julia Nakamura", "Product Manager", "#6366F1", "female", (32, 42), vec!["ja-JP", "en-US", "strategic"]),
            ];
            
            for (i, (name, description, color, gender, age_range, languages)) in speakers.iter().enumerate() {
                let speaker_id = format!("test-speaker-{:03}", i + 1);
                
                // Insert speaker profile
                tx.execute(
                    "INSERT INTO speaker_profiles (
                        id, name, description, color, created_at, updated_at,
                        identification_count, confidence_threshold, is_active,
                        pitch_range_min, pitch_range_max, pitch_mean, speaking_rate,
                        quality_features, gender, age_range_min, age_range_max, language_markers
                    ) VALUES (?1, ?2, ?3, ?4, datetime('now'), datetime('now'), ?5, ?6, 1, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
                    rusqlite::params![
                        speaker_id,
                        name,
                        format!("{} - Generated test profile", description),
                        color,
                        rand::thread_rng().gen_range(1..20), // identification_count
                        0.7 + (rand::thread_rng().gen::<f32>() * 0.2), // confidence_threshold 0.7-0.9
                        if gender == "male" { 80.0 } else { 150.0 }, // pitch_range_min
                        if gender == "male" { 200.0 } else { 350.0 }, // pitch_range_max
                        if gender == "male" { 130.0 } else { 220.0 }, // pitch_mean
                        120.0 + (rand::thread_rng().gen::<f32>() * 80.0), // speaking_rate 120-200 WPM
                        format!(r#"{{"clarity": {:.2}, "stability": {:.2}, "brightness": {:.2}}}"#,
                            0.7 + (rand::thread_rng().gen::<f32>() * 0.3),
                            0.7 + (rand::thread_rng().gen::<f32>() * 0.3),
                            0.5 + (rand::thread_rng().gen::<f32>() * 0.4)
                        ),
                        gender,
                        age_range.0,
                        age_range.1,
                        format!(r#"[{}]"#, languages.iter().map(|l| format!(r#""{}""#, l)).collect::<Vec<_>>().join(", "))
                    ],
                )?;
                profile_count += 1;
                
                // Generate 2-4 embeddings per speaker
                let num_embeddings = rand::thread_rng().gen_range(2..=4);
                for j in 0..num_embeddings {
                    let embedding_id = format!("embed-{:03}-{:03}", i + 1, j + 1);
                    let quality = 0.8 + (rand::thread_rng().gen::<f32>() * 0.2);
                    let vector_blob = Self::generate_test_embedding(&speaker_id, 512, quality);
                    
                    tx.execute(
                        "INSERT INTO voice_embeddings (
                            id, speaker_id, vector, dimensions, model_name,
                            quality_score, duration_seconds, created_at
                        ) VALUES (?1, ?2, ?3, 512, 'comprehensive_test_model_v1', ?4, ?5, datetime('now', ?6))",
                        rusqlite::params![
                            embedding_id,
                            speaker_id,
                            vector_blob,
                            quality,
                            3.0 + (rand::thread_rng().gen::<f32>() * 8.0), // duration 3-11 seconds
                            format!("-{} hours", rand::thread_rng().gen_range(1..72)) // created 1-72 hours ago
                        ],
                    )?;
                    embedding_count += 1;
                }
            }
            
            // Create sample meetings with speaker participation
            let meetings = vec![
                ("weekly-standup", "Weekly Team Standup", vec![0, 1, 2]),
                ("quarterly-review", "Quarterly Business Review", vec![0, 3, 5, 9]),
                ("design-critique", "Design Review Session", vec![2, 4, 6, 7]),
                ("tech-talk", "Technical Architecture Discussion", vec![1, 4, 6, 8]),
                ("strategy-planning", "Strategic Planning Meeting", vec![0, 3, 5, 9]),
                ("all-hands", "All-Hands Company Meeting", vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9]),
            ];
            
            for (meeting_id, meeting_name, participant_indices) in meetings {
                for &speaker_idx in &participant_indices {
                    let speaker_id = format!("test-speaker-{:03}", speaker_idx + 1);
                    let speaker_name = speakers[speaker_idx].0;
                    
                    let speaking_time = 30.0 + (rand::thread_rng().gen::<f32>() * 300.0); // 30-330 seconds
                    let segment_count = (speaking_time / 15.0) as u32 + rand::thread_rng().gen_range(0..5); // ~15s per segment
                    let confidence = 0.6 + (rand::thread_rng().gen::<f32>() * 0.35); // 0.6-0.95
                    
                    tx.execute(
                        "INSERT INTO meeting_speakers (
                            id, meeting_id, speaker_id, display_name,
                            speaking_time_seconds, segment_count, average_confidence,
                            first_spoken_at, last_spoken_at, is_verified, notes
                        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 
                                 datetime('now', ?8), datetime('now', ?9), ?10, ?11)",
                        rusqlite::params![
                            format!("ms-{}-{:03}", meeting_id, speaker_idx + 1),
                            meeting_id,
                            speaker_id,
                            speaker_name,
                            speaking_time,
                            segment_count,
                            confidence,
                            format!("-{} days", rand::thread_rng().gen_range(1..30)), // first_spoken_at
                            format!("-{} days", rand::thread_rng().gen_range(1..30)), // last_spoken_at
                            if rand::thread_rng().gen::<f32>() > 0.3 { 1 } else { 0 }, // 70% verified
                            if rand::thread_rng().gen::<f32>() > 0.5 {
                                Some(format!("Participant in {}", meeting_name))
                            } else { None }
                        ],
                    )?;
                    meeting_count += 1;
                }
            }
            
            tx.commit()?;
            
            let summary = format!(
                "Comprehensive test dataset created:\n- {} speaker profiles\n- {} voice embeddings\n- {} meeting participations\n- {} unique meetings",
                profile_count, embedding_count, meeting_count, meetings.len()
            );

            tracing::info!("{}", summary);
            Ok(summary)
        }).await?
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::Database;
    use tempfile::NamedTempFile;

    async fn create_test_database() -> Database {
        let temp_file = NamedTempFile::new().unwrap();
        let db = Database::new(temp_file.path()).await.unwrap();
        db.migrate().await.unwrap();
        db
    }

    #[tokio::test]
    async fn test_load_test_data() {
        let db = create_test_database().await;
        let seed_manager = SeedManager::new(db);

        let result = seed_manager.load_test_data().await.unwrap();
        assert!(result.contains("speaker profiles"));
        assert!(result.contains("voice embeddings"));
        assert!(result.contains("meeting speakers"));
    }

    #[tokio::test]
    async fn test_clear_all_data() {
        let db = create_test_database().await;
        let seed_manager = SeedManager::new(db);

        // Load data first
        seed_manager.load_test_data().await.unwrap();

        // Then clear it
        let result = seed_manager.clear_all_data().await.unwrap();
        assert!(result.contains("cleared successfully"));
    }

    #[tokio::test]
    async fn test_comprehensive_dataset() {
        let db = create_test_database().await;
        let seed_manager = SeedManager::new(db);

        let result = seed_manager.create_comprehensive_test_dataset().await.unwrap();
        assert!(result.contains("10 speaker profiles"));
        assert!(result.contains("voice embeddings"));
        assert!(result.contains("6 unique meetings"));
    }

    #[test]
    fn test_generate_test_embedding() {
        let embedding_bytes = SeedManager::generate_test_embedding("test-speaker-001", 512, 0.9);
        assert_eq!(embedding_bytes.len(), 512 * 4); // 4 bytes per f32

        // Convert back to floats to verify normalization
        let mut embedding_floats = Vec::new();
        for chunk in embedding_bytes.chunks_exact(4) {
            let bytes = [chunk[0], chunk[1], chunk[2], chunk[3]];
            embedding_floats.push(f32::from_le_bytes(bytes));
        }

        // Check that values are in reasonable range
        assert!(embedding_floats.iter().all(|&x| x >= -1.0 && x <= 1.0));
        
        // Check that vector is approximately normalized
        let magnitude: f32 = embedding_floats.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((magnitude - 1.0).abs() < 0.1); // Should be close to 1.0
    }
}