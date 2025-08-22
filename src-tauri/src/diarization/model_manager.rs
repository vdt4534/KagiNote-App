//! Diarization Model Management
//! 
//! Handles downloading, caching, and loading of pyannote diarization models.

use anyhow::{Result, Context};
use std::path::{Path, PathBuf};
use std::fs;
use reqwest;
use futures_util::StreamExt;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tracing::{info, debug, warn};

/// Model information for diarization
#[derive(Debug, Clone)]
pub struct DiarizationModel {
    pub name: String,
    pub segmentation_url: String,
    pub embedding_url: String,
    pub segmentation_size: u64,
    pub embedding_size: u64,
    pub description: String,
}

/// Default pyannote models
impl DiarizationModel {
    pub fn default() -> Self {
        Self {
            name: "pyannote-3.0".to_string(),
            // Using Hugging Face model URLs
            segmentation_url: "https://huggingface.co/pyannote/segmentation-3.0/resolve/main/pytorch_model.bin".to_string(),
            embedding_url: "https://huggingface.co/pyannote/wespeaker-voxceleb-resnet34-LM/resolve/main/pytorch_model.bin".to_string(),
            segmentation_size: 5_900_000, // ~5.9MB
            embedding_size: 24_500_000,   // ~24.5MB
            description: "PyAnnote 3.0 segmentation + WeSpeaker embeddings".to_string(),
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
    
    /// Check if models are already downloaded
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
    
    /// Download models if needed
    pub async fn ensure_models_available<F>(
        &self,
        progress_callback: F,
    ) -> Result<()>
    where
        F: Fn(f32, String) + Send + Sync + 'static,
    {
        if self.are_models_cached() {
            info!("Diarization models already cached");
            progress_callback(1.0, "Models already available".to_string());
            return Ok(());
        }
        
        info!("Downloading diarization models...");
        
        // Download segmentation model
        progress_callback(0.0, "Downloading segmentation model...".to_string());
        self.download_model(
            &self.model.segmentation_url,
            &self.get_segmentation_model_path(),
            self.model.segmentation_size,
            |p| progress_callback(p * 0.5, format!("Segmentation: {:.0}%", p * 100.0)),
        ).await?;
        
        // Download embedding model
        progress_callback(0.5, "Downloading embedding model...".to_string());
        self.download_model(
            &self.model.embedding_url,
            &self.get_embedding_model_path(),
            self.model.embedding_size,
            |p| progress_callback(0.5 + p * 0.5, format!("Embedding: {:.0}%", p * 100.0)),
        ).await?;
        
        progress_callback(1.0, "Models downloaded successfully".to_string());
        info!("Diarization models downloaded successfully");
        
        Ok(())
    }
    
    /// Download a single model file
    async fn download_model<F>(
        &self,
        url: &str,
        target_path: &Path,
        expected_size: u64,
        progress_callback: F,
    ) -> Result<()>
    where
        F: Fn(f32) + Send + Sync,
    {
        debug!("Downloading from {} to {:?}", url, target_path);
        
        // Create a client
        let client = reqwest::Client::new();
        let response = client
            .get(url)
            .send()
            .await
            .context("Failed to start download")?;
            
        let total_size = response
            .content_length()
            .unwrap_or(expected_size);
            
        // Create the file
        let mut file = File::create(target_path)
            .await
            .context("Failed to create model file")?;
            
        // Download with progress
        let mut downloaded = 0u64;
        let mut stream = response.bytes_stream();
        
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.context("Failed to download chunk")?;
            file.write_all(&chunk).await
                .context("Failed to write to file")?;
                
            downloaded += chunk.len() as u64;
            let progress = downloaded as f32 / total_size as f32;
            progress_callback(progress);
        }
        
        file.flush().await.context("Failed to flush file")?;
        
        Ok(())
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
        
        if seg_size < (seg_expected * 9 / 10) || seg_size > (seg_expected * 11 / 10) {
            warn!("Segmentation model size mismatch: {} vs expected {}", seg_size, seg_expected);
        }
        
        if emb_size < (emb_expected * 9 / 10) || emb_size > (emb_expected * 11 / 10) {
            warn!("Embedding model size mismatch: {} vs expected {}", emb_size, emb_expected);
        }
        
        info!("Model verification complete");
        Ok(())
    }
    
    /// Clean up downloaded models
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
    use tempfile::TempDir;
    
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
        
        // Initially not cached (unless already downloaded)
        // Just check the method works
        let _ = manager.are_models_cached();
    }
}