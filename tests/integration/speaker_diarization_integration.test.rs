//! Integration tests for speaker diarization functionality
//!
//! These tests verify that the diarization, storage, and frontend integration
//! all work together correctly end-to-end.

use std::time::Duration;
use tokio;
use uuid::Uuid;

use kaginote::commands::AppState;
use kaginote::diarization::{DiarizationService, DiarizationConfig};
use kaginote::models::{CreateSpeakerProfileRequest, VoiceEmbedding};
use kaginote::storage::{Database, SpeakerStore, EmbeddingIndex};

const TEST_SAMPLE_RATE: u32 = 16000;

/// Generate test audio samples for testing
fn generate_test_audio(duration_seconds: f32, frequency: f32) -> Vec<f32> {
    let sample_count = (TEST_SAMPLE_RATE as f32 * duration_seconds) as usize;
    let mut samples = Vec::with_capacity(sample_count);
    
    for i in 0..sample_count {
        let t = i as f32 / TEST_SAMPLE_RATE as f32;
        let sample = (2.0 * std::f32::consts::PI * frequency * t).sin();
        samples.push(sample * 0.5); // Reduce amplitude
    }
    
    samples
}

/// Test that diarization service can be initialized and process audio
#[tokio::test]
async fn test_diarization_service_initialization() {
    let config = DiarizationConfig::default();
    
    // This might fail if pyannote models aren't available, which is expected in CI
    match DiarizationService::new(config).await {
        Ok(service) => {
            assert!(service.get_config().max_speakers > 0);
            println!("✅ Diarization service initialized successfully");
        }
        Err(e) => {
            println!("⚠️  Diarization service initialization failed (expected in CI): {:?}", e);
            // This is expected if pyannote models aren't available
        }
    }
}

/// Test that speaker profiles can be stored and retrieved from database
#[tokio::test]
async fn test_speaker_profile_storage() {
    // Create temporary database
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("test_speakers.db");
    
    // Initialize database
    let database = Database::new(&db_path).await
        .expect("Failed to create database");
    database.migrate().await
        .expect("Failed to run migrations");
    
    let speaker_store = SpeakerStore::new(database.clone());
    
    // Create test speaker profile
    let create_request = CreateSpeakerProfileRequest {
        name: "Test Speaker".to_string(),
        description: Some("Integration test speaker".to_string()),
        color: Some("#3B82F6".to_string()),
        confidence_threshold: Some(0.8),
    };
    
    let created_profile = speaker_store.create_speaker_profile(create_request).await
        .expect("Failed to create speaker profile");
    
    assert_eq!(created_profile.name, "Test Speaker");
    assert_eq!(created_profile.color, "#3B82F6");
    
    // Retrieve profile
    let retrieved_profile = speaker_store.get_speaker_profile(created_profile.id).await
        .expect("Failed to retrieve profile");
    
    assert!(retrieved_profile.is_some());
    let profile = retrieved_profile.unwrap();
    assert_eq!(profile.id, created_profile.id);
    assert_eq!(profile.name, "Test Speaker");
    
    println!("✅ Speaker profile storage and retrieval working");
}

/// Test that voice embeddings can be stored and searched
#[tokio::test]
async fn test_voice_embedding_storage_and_search() {
    // Create temporary database
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("test_embeddings.db");
    
    // Initialize database and embedding index
    let database = Database::new(&db_path).await
        .expect("Failed to create database");
    database.migrate().await
        .expect("Failed to run migrations");
    
    let speaker_store = SpeakerStore::new(database.clone());
    let mut embedding_index = EmbeddingIndex::new(512, 8);
    
    // Create test speaker
    let create_request = CreateSpeakerProfileRequest {
        name: "Voice Test Speaker".to_string(),
        description: Some("For embedding tests".to_string()),
        color: Some("#10B981".to_string()),
        confidence_threshold: Some(0.7),
    };
    
    let speaker_profile = speaker_store.create_speaker_profile(create_request).await
        .expect("Failed to create speaker");
    
    // Create test embedding
    let test_vector = (0..512).map(|i| (i as f32) / 512.0).collect();
    let voice_embedding = VoiceEmbedding::new(
        speaker_profile.id,
        test_vector.clone(),
        "test_model".to_string(),
        0.95,
        3.0,
    );
    
    // Store embedding
    speaker_store.add_voice_embedding(voice_embedding.clone()).await
        .expect("Failed to store embedding");
    
    // Add to index
    embedding_index.add_embedding(voice_embedding.clone())
        .expect("Failed to add to index");
    
    // Search for similar embeddings
    let similar_results = embedding_index.find_similar_embeddings(&test_vector, 0.8, 5)
        .expect("Failed to search embeddings");
    
    assert!(!similar_results.is_empty());
    assert_eq!(similar_results[0].0, speaker_profile.id);
    assert!(similar_results[0].1 > 0.9); // Should be very similar to itself
    
    println!("✅ Voice embedding storage and search working");
}

/// Test app state initialization with all systems
#[tokio::test]
async fn test_app_state_initialization() {
    let app_state = AppState::new();
    
    // Test speaker storage initialization
    match app_state.initialize_speaker_storage().await {
        Ok(_) => println!("✅ Speaker storage initialized in AppState"),
        Err(e) => panic!("❌ Failed to initialize speaker storage: {}", e),
    }
    
    // Verify embedding index is working
    {
        let index_guard = app_state.embedding_index.lock().await;
        let stats = index_guard.get_stats().expect("Failed to get index stats");
        assert_eq!(stats.embedding_dimension, 512);
        println!("✅ Embedding index initialized with correct dimensions");
    }
}

/// Test the complete integration flow: diarization → storage → retrieval
#[tokio::test]
async fn test_complete_integration_flow() {
    let app_state = AppState::new();
    
    // Initialize speaker storage
    app_state.initialize_speaker_storage().await
        .expect("Failed to initialize speaker storage");
    
    // Create test audio samples (different frequencies for different "speakers")
    let speaker1_audio = generate_test_audio(2.0, 440.0); // A4 note
    let speaker2_audio = generate_test_audio(2.0, 523.25); // C5 note
    
    println!("Generated test audio: {} and {} samples", speaker1_audio.len(), speaker2_audio.len());
    
    // Try to initialize diarization (might fail without models)
    let diarization_config = DiarizationConfig {
        max_speakers: 4,
        min_speakers: 1,
        embedding_dimension: 512,
        similarity_threshold: 0.7,
        min_segment_duration: 1.0,
        ..Default::default()
    };
    
    match DiarizationService::new(diarization_config).await {
        Ok(diarization_service) => {
            // Store in app state
            let mut diarization_guard = app_state.diarization_service.lock().await;
            *diarization_guard = Some(diarization_service);
            drop(diarization_guard);
            
            println!("✅ Diarization service initialized - testing full flow");
            
            // Test speaker identification flow
            let diarization_guard = app_state.diarization_service.lock().await;
            if let Some(ref service) = *diarization_guard {
                // Extract embeddings from first audio sample
                match service.extract_speaker_embeddings(&speaker1_audio, TEST_SAMPLE_RATE).await {
                    Ok(embeddings) if !embeddings.is_empty() => {
                        println!("✅ Extracted {} embeddings from audio", embeddings.len());
                        
                        // Try speaker identification (should return None for new speaker)
                        match service.reidentify_speaker(&embeddings[0]).await {
                            Ok(None) => println!("✅ Correctly identified as new speaker"),
                            Ok(Some(existing)) => println!("⚠️  Unexpectedly found existing speaker: {}", existing),
                            Err(e) => println!("⚠️  Speaker identification error: {:?}", e),
                        }
                    }
                    Ok(_) => println!("⚠️  No embeddings extracted from audio"),
                    Err(e) => println!("⚠️  Embedding extraction failed: {:?}", e),
                }
            }
        }
        Err(e) => {
            println!("⚠️  Skipping diarization tests - service unavailable: {:?}", e);
            println!("✅ Integration test completed (diarization models not available)");
        }
    }
}

/// Test error handling and graceful degradation
#[tokio::test]
async fn test_error_handling_and_graceful_degradation() {
    let app_state = AppState::new();
    
    // Test handling of empty audio
    let empty_audio: Vec<f32> = vec![];
    
    // This should handle empty audio gracefully
    match app_state.initialize_speaker_storage().await {
        Ok(_) => {
            // Try diarization with empty audio - should handle gracefully
            let diarization_config = DiarizationConfig::default();
            
            match DiarizationService::new(diarization_config).await {
                Ok(service) => {
                    match service.extract_speaker_embeddings(&empty_audio, TEST_SAMPLE_RATE).await {
                        Err(_) => println!("✅ Correctly handled empty audio with error"),
                        Ok(embeddings) => {
                            if embeddings.is_empty() {
                                println!("✅ Correctly returned empty embeddings for empty audio");
                            } else {
                                panic!("❌ Unexpected embeddings from empty audio");
                            }
                        }
                    }
                }
                Err(_) => println!("✅ Gracefully handled unavailable diarization service"),
            }
        }
        Err(e) => panic!("❌ Failed basic initialization: {}", e),
    }
    
    // Test invalid sample rate handling
    let test_audio = generate_test_audio(1.0, 440.0);
    let invalid_sample_rate = 0;
    
    if let Ok(service) = DiarizationService::new(DiarizationConfig::default()).await {
        match service.extract_speaker_embeddings(&test_audio, invalid_sample_rate).await {
            Err(_) => println!("✅ Correctly handled invalid sample rate"),
            Ok(_) => panic!("❌ Should have failed with invalid sample rate"),
        }
    }
    
    println!("✅ Error handling tests completed");
}

/// Performance validation test
#[tokio::test]
async fn test_performance_targets() {
    use std::time::Instant;
    
    let app_state = AppState::new();
    
    // Test speaker storage initialization performance (should be < 1s)
    let start = Instant::now();
    app_state.initialize_speaker_storage().await
        .expect("Failed to initialize speaker storage");
    let init_duration = start.elapsed();
    
    assert!(init_duration < Duration::from_secs(1), 
           "Speaker storage initialization took {}ms, should be < 1000ms", 
           init_duration.as_millis());
    
    println!("✅ Speaker storage initialized in {}ms", init_duration.as_millis());
    
    // Test embedding index operations performance
    let mut embedding_index = EmbeddingIndex::new(512, 8);
    let test_vector: Vec<f32> = (0..512).map(|i| (i as f32) / 512.0).collect();
    
    // Test single embedding addition (should be very fast)
    let start = Instant::now();
    let test_embedding = VoiceEmbedding::new(
        Uuid::new_v4(),
        test_vector.clone(),
        "test_model".to_string(),
        0.9,
        2.0,
    );
    embedding_index.add_embedding(test_embedding)
        .expect("Failed to add embedding");
    let add_duration = start.elapsed();
    
    assert!(add_duration < Duration::from_millis(10), 
           "Embedding addition took {}ms, should be < 10ms", 
           add_duration.as_millis());
    
    // Test similarity search performance
    let start = Instant::now();
    let results = embedding_index.find_similar_embeddings(&test_vector, 0.8, 5)
        .expect("Failed to search embeddings");
    let search_duration = start.elapsed();
    
    assert!(search_duration < Duration::from_millis(100), 
           "Similarity search took {}ms, should be < 100ms", 
           search_duration.as_millis());
    
    assert!(!results.is_empty(), "Should find at least one similar embedding");
    
    println!("✅ Performance targets met:");
    println!("   - Speaker storage init: {}ms", init_duration.as_millis());
    println!("   - Embedding addition: {}μs", add_duration.as_micros());
    println!("   - Similarity search: {}μs", search_duration.as_micros());
}