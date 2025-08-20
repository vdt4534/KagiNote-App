//! Simple test to identify the transcription issue

use kaginote_lib::audio::types::{AudioData, AudioSource};
use kaginote_lib::asr::types::TranscriptionContext;

#[tokio::test]
async fn test_whisper_engine_initialization() {
    println!("🔍 Testing Whisper engine initialization...");
    
    let whisper_config = kaginote_lib::asr::whisper::WhisperConfig::default();
    
    match kaginote_lib::asr::whisper::WhisperEngine::new(whisper_config).await {
        Ok(engine) => {
            println!("✅ Whisper engine initialized successfully");
            
            // Create simple test audio
            let mut samples = Vec::new();
            for _ in 0..16000 {
                samples.push(0.1);
            }
            let test_audio = AudioData {
                samples,
                sample_rate: 16000,
                channels: 1,
                timestamp: std::time::SystemTime::now(),
                source_channel: AudioSource::Microphone,
                duration_seconds: 1.0,
            };
            
            let context = TranscriptionContext::default();
            
            match engine.transcribe(&test_audio, &context).await {
                Ok(result) => {
                    println!("✅ Transcription completed");
                    println!("📝 Text: '{}'", result.text);
                    println!("🎯 Confidence: {:.2}", result.confidence);
                    
                    if result.text.is_empty() {
                        println!("⚠️ WARNING: Empty transcription result");
                    }
                }
                Err(e) => {
                    println!("❌ Transcription failed: {}", e);
                }
            }
        }
        Err(e) => {
            println!("❌ Whisper engine initialization failed: {}", e);
            
            // Analyze the error to understand the root cause
            let error_msg = e.to_string();
            
            if error_msg.contains("model") || error_msg.contains("file") || error_msg.contains("download") {
                println!("🔍 ROOT CAUSE: Model file issue");
                println!("   - Model may not be downloaded");
                println!("   - Model file may be corrupted");
                println!("   - Model path may be incorrect");
            }
            
            if error_msg.contains("memory") || error_msg.contains("allocation") {
                println!("🔍 ROOT CAUSE: Memory issue");
                println!("   - Insufficient memory for model");
                println!("   - Memory allocation failed");
            }
            
            if error_msg.contains("whisper") || error_msg.contains("context") {
                println!("🔍 ROOT CAUSE: Whisper library issue");
                println!("   - whisper.cpp integration problem");
                println!("   - Metal acceleration issue");
                println!("   - Library initialization failed");
            }
            
            // This is the failing condition we expect
            panic!("Whisper engine initialization failed: {}", e);
        }
    }
}

#[tokio::test]
async fn test_model_manager_download() {
    println!("🔍 Testing model manager download functionality...");
    
    use kaginote_lib::asr::model_manager::ModelManager;
    use kaginote_lib::asr::types::ModelTier;
    
    let mut model_manager = match ModelManager::new() {
        Ok(mm) => {
            println!("✅ Model manager created successfully");
            mm
        }
        Err(e) => {
            println!("❌ Failed to create model manager: {}", e);
            panic!("Model manager creation failed: {}", e);
        }
    };
    
    // Test downloading the standard model
    println!("🔍 Testing model download for Standard tier...");
    
    match model_manager.ensure_model_available(ModelTier::Standard, None).await {
        Ok(path) => {
            println!("✅ Model available at: {:?}", path);
            
            if path.exists() {
                let metadata = std::fs::metadata(&path).unwrap();
                let size_mb = metadata.len() / (1024 * 1024);
                println!("📁 Model file size: {}MB", size_mb);
                
                if size_mb < 100 {
                    println!("⚠️ WARNING: Model file seems too small ({}MB)", size_mb);
                    println!("   Expected: ~800MB for medium model");
                }
                
                if size_mb > 100 {
                    println!("✅ Model file size looks correct");
                }
            } else {
                println!("❌ BUG FOUND: Model path returned but file doesn't exist!");
                panic!("Model file missing at path: {:?}", path);
            }
        }
        Err(e) => {
            println!("❌ Model download failed: {}", e);
            
            let error_msg = e.to_string();
            
            if error_msg.contains("network") || error_msg.contains("download") || error_msg.contains("request") {
                println!("🔍 ROOT CAUSE: Network/download issue");
                println!("   - Check internet connection");
                println!("   - Check download URLs");
                println!("   - Check firewall settings");
            }
            
            if error_msg.contains("space") || error_msg.contains("disk") {
                println!("🔍 ROOT CAUSE: Disk space issue");
                println!("   - Free up disk space");
                println!("   - Check available storage");
            }
            
            if error_msg.contains("permission") {
                println!("🔍 ROOT CAUSE: Permission issue");
                println!("   - Check file permissions");
                println!("   - Check directory write access");
            }
            
            panic!("Model download failed: {}", e);
        }
    }
}