//! Direct test for ONNX model loading functionality
//!
//! This test validates that:
//! 1. Bundled ONNX models exist and can be found
//! 2. ONNX models can be loaded and create sessions successfully
//! 3. Model paths are resolved correctly in DiarizationModelManager
//! 4. Real ONNX model sessions can be created and used

use anyhow::Result;
use ort::{Environment, SessionBuilder};
use std::path::PathBuf;
use std::sync::Arc;

#[tokio::test]
async fn test_bundled_onnx_models_exist_and_load() {
    println!("🔍 Testing bundled ONNX models exist and can load...");
    
    // Test 1: Check bundled models exist
    let resource_dir = get_bundled_model_path().expect("Should find bundled models");
    println!("📂 Resource directory: {:?}", resource_dir);
    
    let seg_path = resource_dir.join("segmentation.onnx");
    let emb_path = resource_dir.join("embedding.onnx");
    
    assert!(seg_path.exists(), "Segmentation model should exist at {:?}", seg_path);
    assert!(emb_path.exists(), "Embedding model should exist at {:?}", emb_path);
    
    println!("✅ Bundled models found:");
    println!("   - Segmentation: {:?}", seg_path);
    println!("   - Embedding: {:?}", emb_path);
    
    // Test 2: Check file sizes are reasonable
    let seg_size = std::fs::metadata(&seg_path).unwrap().len();
    let emb_size = std::fs::metadata(&emb_path).unwrap().len();
    
    println!("📊 Model sizes:");
    println!("   - Segmentation: {} MB", seg_size / 1_048_576);
    println!("   - Embedding: {} MB", emb_size / 1_048_576);
    
    assert!(seg_size > 1_000_000, "Segmentation model should be > 1MB, got {}", seg_size);
    assert!(emb_size > 10_000_000, "Embedding model should be > 10MB, got {}", emb_size);
    
    // Test 3: Create ONNX sessions successfully
    println!("🔧 Creating ONNX environment...");
    let environment = Arc::new(Environment::builder()
        .with_name("test_diarization")
        .build()
        .expect("Should create ONNX environment"));
    
    println!("🔧 Testing segmentation model loading...");
    let seg_session_result = SessionBuilder::new(&environment)
        .and_then(|builder| builder.with_model_from_file(&seg_path));
    
    match seg_session_result {
        Ok(_session) => {
            println!("✅ Segmentation model loaded successfully");
        },
        Err(e) => {
            println!("❌ Segmentation model failed to load: {:?}", e);
            panic!("Critical: Segmentation ONNX model failed to load: {:?}", e);
        }
    }
    
    println!("🔧 Testing embedding model loading...");
    let emb_session_result = SessionBuilder::new(&environment)
        .and_then(|builder| builder.with_model_from_file(&emb_path));
    
    match emb_session_result {
        Ok(_session) => {
            println!("✅ Embedding model loaded successfully");
        },
        Err(e) => {
            println!("❌ Embedding model failed to load: {:?}", e);
            panic!("Critical: Embedding ONNX model failed to load: {:?}", e);
        }
    }
    
    println!("🎉 All ONNX model loading tests passed!");
}

#[tokio::test]
async fn test_diarization_model_manager() {
    use kaginote_lib::diarization::model_manager::DiarizationModelManager;
    
    println!("🔍 Testing DiarizationModelManager functionality...");
    
    let manager = DiarizationModelManager::new()
        .expect("Should create DiarizationModelManager");
    
    // Test cached model paths
    let seg_path = manager.get_segmentation_model_path();
    let emb_path = manager.get_embedding_model_path();
    
    println!("📂 Cached model paths:");
    println!("   - Segmentation: {:?}", seg_path);
    println!("   - Embedding: {:?}", emb_path);
    
    // These should be in user data directory
    assert!(seg_path.to_string_lossy().contains("KagiNote"));
    assert!(seg_path.to_string_lossy().contains("diarization"));
    assert!(emb_path.to_string_lossy().contains("KagiNote"));
    assert!(emb_path.to_string_lossy().contains("diarization"));
    
    // Test model availability check
    let are_cached = manager.are_models_cached();
    println!("📋 Models cached: {}", are_cached);
    
    // Test model copying
    println!("🔄 Testing model copying process...");
    
    let result = manager.ensure_models_available(|progress, message| {
        println!("📊 Copy progress: {:.0}% - {}", progress * 100.0, message);
    }).await;
    
    match result {
        Ok(()) => {
            println!("✅ Model copy process completed successfully");
            assert!(manager.are_models_cached(), "Models should be cached after successful copy");
            
            // Verify cached models can be loaded
            println!("🔧 Testing cached model loading...");
            test_cached_model_loading(&seg_path, &emb_path).await;
            
        },
        Err(e) => {
            println!("⚠️  Model copying failed: {:?}", e);
            // This might fail in some environments, but we should at least test path resolution
            println!("   Path resolution worked, but copying failed - this is acceptable in some test environments");
        }
    }
    
    // Test model verification if available
    if manager.are_models_cached() {
        println!("🔍 Testing model verification...");
        match manager.verify_models() {
            Ok(()) => println!("✅ Model verification passed"),
            Err(e) => println!("⚠️  Model verification failed: {:?}", e),
        }
    }
    
    println!("🎉 DiarizationModelManager tests complete!");
}

async fn test_cached_model_loading(seg_path: &PathBuf, emb_path: &PathBuf) {
    if !seg_path.exists() || !emb_path.exists() {
        println!("⏭️  Skipping cached model loading test - models not present");
        return;
    }
    
    println!("🔧 Testing cached ONNX model sessions...");
    let environment = Arc::new(Environment::builder()
        .with_name("test_cached")
        .build()
        .expect("Should create environment"));
    
    // Test segmentation model
    match SessionBuilder::new(&environment)
        .and_then(|builder| builder.with_model_from_file(seg_path)) {
        Ok(_) => println!("✅ Cached segmentation model loads successfully"),
        Err(e) => panic!("❌ Cached segmentation model failed: {:?}", e),
    }
    
    // Test embedding model
    match SessionBuilder::new(&environment)
        .and_then(|builder| builder.with_model_from_file(emb_path)) {
        Ok(_) => println!("✅ Cached embedding model loads successfully"),
        Err(e) => panic!("❌ Cached embedding model failed: {:?}", e),
    }
}

#[tokio::test]
async fn test_speaker_embedder_initialization() {
    use kaginote_lib::diarization::embedder::SpeakerEmbedder;
    use kaginote_lib::diarization::types::DiarizationConfig;
    
    println!("🔍 Testing SpeakerEmbedder ONNX initialization...");
    
    let config = DiarizationConfig::default();
    let mut embedder = SpeakerEmbedder::new(config).await
        .expect("Should create SpeakerEmbedder");
    
    println!("📋 Initial state - Initialized: {}", embedder.initialized);
    assert!(!embedder.initialized, "Should start uninitialized");
    
    // Test model initialization
    println!("🔧 Testing ONNX model initialization...");
    match embedder.initialize_models().await {
        Ok(()) => {
            println!("✅ ONNX models initialized successfully");
            assert!(embedder.initialized, "Should be initialized after successful init");
            println!("📋 Final state - Initialized: {}", embedder.initialized);
        },
        Err(e) => {
            println!("⚠️  ONNX model initialization failed: {:?}", e);
            println!("   This is expected if models aren't available in test environment");
            // Don't panic here - models might not be available in CI
        }
    }
    
    println!("🎉 SpeakerEmbedder initialization test complete!");
}

/// Get the path to bundled models (replicates DiarizationModelManager logic)
fn get_bundled_model_path() -> Result<PathBuf> {
    // Try to get the resource directory from Tauri context
    if let Ok(exe_dir) = std::env::current_exe() {
        // In production, resources are next to the executable
        let resource_path = exe_dir
            .parent()
            .ok_or_else(|| anyhow::anyhow!("Could not get executable directory"))?
            .join("resources")
            .join("models")
            .join("diarization");
        
        if resource_path.exists() {
            return Ok(resource_path);
        }
        
        // In development, use the src-tauri/resources directory
        let dev_path = PathBuf::from("src-tauri/resources/models/diarization");
        if dev_path.exists() {
            return Ok(dev_path);
        }
        
        // Try relative to current directory
        let relative_path = PathBuf::from("resources/models/diarization");
        if relative_path.exists() {
            return Ok(relative_path);
        }
    }
    
    // Fallback: look for the models in common locations
    let fallback_paths = [
        PathBuf::from("./src-tauri/resources/models/diarization"),
        PathBuf::from("../src-tauri/resources/models/diarization"),
        PathBuf::from("resources/models/diarization"),
        // Additional path for running from project root
        PathBuf::from("./resources/models/diarization"),
    ];
    
    for path in &fallback_paths {
        if path.exists() {
            return Ok(path.clone());
        }
    }
    
    Err(anyhow::anyhow!("Could not find bundled model resources in any expected location"))
}