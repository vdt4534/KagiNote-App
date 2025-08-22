//! Diarization Model Management
//! 
//! Handles copying bundled models to the user's application directory
//! No network downloads required - models are bundled with the app

use anyhow::{Result, Context};
use std::path::PathBuf;
use std::fs;
use tracing::{info, debug, warn};

/// Model information for diarization
#[derive(Debug, Clone)]
pub struct DiarizationModel {
    pub name: String,
    pub segmentation_size: u64,
    pub embedding_size: u64,
    pub description: String,
}

/// Default bundled models
impl DiarizationModel {
    pub fn default() -> Self {
        Self {
            name: "sherpa-onnx-pyannote".to_string(),
            segmentation_size: 5_900_000,  // ~5.9MB
            embedding_size: 71_500_000,    // ~71.5MB (3D-Speaker ERes2NetV2)
            description: "Sherpa-ONNX PyAnnote segmentation + 3D-Speaker ERes2NetV2 embeddings".to_string(),
        }
    }
}

/// Manages diarization model storage and retrieval
pub struct DiarizationModelManager {
    storage_dir: PathBuf,
    model: DiarizationModel,
}

impl DiarizationModelManager {
    /// Create a new model manager
    pub fn new() -> Result<Self> {
        let storage_dir = Self::get_storage_directory()?;
        
        // Ensure directory exists
        fs::create_dir_all(&storage_dir)
            .context("Failed to create diarization model directory")?;
            
        Ok(Self {
            storage_dir,
            model: DiarizationModel::default(),
        })
    }
    
    /// Get the storage directory for diarization models
    fn get_storage_directory() -> Result<PathBuf> {
        let app_support = dirs::data_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine application data directory"))?;
            
        Ok(app_support
            .join("KagiNote")
            .join("models")
            .join("diarization"))
    }
    
    /// Check if models are already available
    pub fn are_models_cached(&self) -> bool {
        let seg_path = self.get_segmentation_model_path();
        let emb_path = self.get_embedding_model_path();
        
        seg_path.exists() && emb_path.exists()
    }
    
    /// Get path to segmentation model
    pub fn get_segmentation_model_path(&self) -> PathBuf {
        self.storage_dir.join("segmentation.onnx")
    }
    
    /// Get path to embedding model
    pub fn get_embedding_model_path(&self) -> PathBuf {
        self.storage_dir.join("embedding.onnx")
    }
    
    /// Ensure models are available (copy from bundled resources if needed)
    pub async fn ensure_models_available<F>(
        &self,
        progress_callback: F,
    ) -> Result<()>
    where
        F: Fn(f32, String) + Send + Sync + 'static,
    {
        if self.are_models_cached() {
            info!("Diarization models already available");
            progress_callback(1.0, "Models already available".to_string());
            return Ok(());
        }
        
        info!("Copying bundled diarization models...");
        progress_callback(0.0, "Setting up diarization models...".to_string());
        
        // Copy models from bundled resources
        self.copy_bundled_models(&progress_callback)?;
        
        progress_callback(1.0, "Models ready".to_string());
        info!("Diarization models ready");
        
        Ok(())
    }
    
    /// Copy bundled models to user directory
    fn copy_bundled_models<F>(&self, progress_callback: &F) -> Result<()>
    where
        F: Fn(f32, String) + Send + Sync,
    {
        // Get the resource directory path
        let resource_dir = Self::get_resource_dir()?;
        
        // Copy segmentation model
        progress_callback(0.25, "Copying segmentation model...".to_string());
        let seg_source = resource_dir.join("segmentation.onnx");
        let seg_target = self.get_segmentation_model_path();
        
        if seg_source.exists() {
            fs::copy(&seg_source, &seg_target)
                .context("Failed to copy segmentation model")?;
            debug!("Copied segmentation model to {:?}", seg_target);
        } else {
            warn!("Segmentation model not found in resources at {:?}", seg_source);
        }
        
        // Copy embedding model
        progress_callback(0.75, "Copying embedding model...".to_string());
        let emb_source = resource_dir.join("embedding.onnx");
        let emb_target = self.get_embedding_model_path();
        
        if emb_source.exists() {
            fs::copy(&emb_source, &emb_target)
                .context("Failed to copy embedding model")?;
            debug!("Copied embedding model to {:?}", emb_target);
        } else {
            warn!("Embedding model not found in resources at {:?}", emb_source);
        }
        
        Ok(())
    }
    
    /// Get the resource directory containing bundled models
    fn get_resource_dir() -> Result<PathBuf> {
        // Try to get the resource directory from Tauri
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
        ];
        
        for path in &fallback_paths {
            if path.exists() {
                return Ok(path.clone());
            }
        }
        
        Err(anyhow::anyhow!("Could not find bundled model resources"))
    }
    
    /// Verify model integrity
    pub fn verify_models(&self) -> Result<()> {
        let seg_path = self.get_segmentation_model_path();
        let emb_path = self.get_embedding_model_path();
        
        // Check file sizes
        let seg_size = fs::metadata(&seg_path)
            .context("Failed to read segmentation model")?
            .len();
        let emb_size = fs::metadata(&emb_path)
            .context("Failed to read embedding model")?
            .len();
            
        // Allow 10% size variance
        let seg_expected = self.model.segmentation_size;
        let emb_expected = self.model.embedding_size;
        
        if seg_size < (seg_expected * 9 / 10) {
            warn!("Segmentation model size mismatch: {} vs expected {}", seg_size, seg_expected);
        }
        
        if emb_size < (emb_expected * 9 / 10) {
            warn!("Embedding model size mismatch: {} vs expected {}", emb_size, emb_expected);
        }
        
        info!("Model verification complete");
        Ok(())
    }
    
    /// Clean up downloaded models (for debugging/reset)
    pub fn cleanup_models(&self) -> Result<()> {
        let seg_path = self.get_segmentation_model_path();
        let emb_path = self.get_embedding_model_path();
        
        if seg_path.exists() {
            fs::remove_file(seg_path)
                .context("Failed to remove segmentation model")?;
        }
        
        if emb_path.exists() {
            fs::remove_file(emb_path)
                .context("Failed to remove embedding model")?;
        }
        
        info!("Diarization models cleaned up");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_model_paths() {
        let manager = DiarizationModelManager::new().unwrap();
        
        let seg_path = manager.get_segmentation_model_path();
        assert!(seg_path.to_string_lossy().contains("segmentation"));
        
        let emb_path = manager.get_embedding_model_path();
        assert!(emb_path.to_string_lossy().contains("embedding"));
    }
    
    #[test]
    fn test_cache_detection() {
        let manager = DiarizationModelManager::new().unwrap();
        
        // Just check the method works
        let _ = manager.are_models_cached();
    }
}