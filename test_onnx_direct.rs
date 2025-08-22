use std::path::Path;
use ort::{Environment, Session, SessionBuilder};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Testing ONNX model loading directly...");
    
    // Initialize ORT environment
    let environment = Environment::builder()
        .with_name("test")
        .build()?;
    
    // Test paths
    let models_dir = dirs::data_dir()
        .unwrap()
        .join("KagiNote")
        .join("models");
    
    let segmentation_path = models_dir.join("segmentation-3.0.onnx");
    let wespeaker_path = models_dir.join("wespeaker-voxceleb-resnet34-LM.onnx");
    
    println!("📁 Models directory: {:?}", models_dir);
    println!("📁 Segmentation model: {:?}", segmentation_path);
    println!("📁 WeSpeaker model: {:?}", wespeaker_path);
    
    // Check if files exist
    if !segmentation_path.exists() {
        println!("❌ Segmentation model not found!");
        return Ok(());
    }
    
    if !wespeaker_path.exists() {
        println!("❌ WeSpeaker model not found!");
        return Ok(());
    }
    
    // Test loading segmentation model
    println!("🔍 Testing segmentation model loading...");
    match SessionBuilder::new(&environment)?.create_from_file(&segmentation_path) {
        Ok(_session) => {
            println!("✅ Segmentation model loaded successfully!");
        }
        Err(e) => {
            println!("❌ Failed to load segmentation model: {}", e);
            return Err(e.into());
        }
    }
    
    // Test loading WeSpeaker model
    println!("🔍 Testing WeSpeaker model loading...");
    match SessionBuilder::new(&environment)?.create_from_file(&wespeaker_path) {
        Ok(_session) => {
            println!("✅ WeSpeaker model loaded successfully!");
        }
        Err(e) => {
            println!("❌ Failed to load WeSpeaker model: {}", e);
            return Err(e.into());
        }
    }
    
    println!("🎉 All ONNX models loaded successfully!");
    Ok(())
}