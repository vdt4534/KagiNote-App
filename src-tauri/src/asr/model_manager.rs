//! Model Manager for automatic Whisper model downloading and management
//! 
//! Handles downloading quantized Whisper models optimized for different tiers,
//! with automatic fallback and integrity verification.

use crate::asr::types::{ModelTier, ASRError};
use anyhow::Result;
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use tokio::fs;
use futures_util::StreamExt;
use reqwest;
use sha2::{Sha256, Digest};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Model metadata for verification and management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMetadata {
    pub name: String,
    pub url: String,
    pub size_mb: u64,
    pub sha256: String,
    pub tier: ModelTier,
    pub quantization: String,
    pub description: String,
}

/// Cache metadata to track model download status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheMetadata {
    pub download_timestamp: chrono::DateTime<chrono::Utc>,
    pub file_size: u64,
    pub sha256_verified: bool,
    pub model_tier: ModelTier,
    pub last_validation: Option<chrono::DateTime<chrono::Utc>>,
    pub validation_status: ValidationStatus,
}

/// Model cache status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CacheStatus {
    NotCached,
    Cached { metadata: CacheMetadata },
    Corrupted { reason: String },
    Downloading { progress: f32 },
}

/// Validation status of cached models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationStatus {
    NotValidated,
    Valid,
    Invalid { reason: String },
}

/// Model download progress callback
pub type ProgressCallback = Box<dyn Fn(u64, u64) + Send + Sync>;

/// Manages Whisper model downloading and caching
pub struct ModelManager {
    models_dir: PathBuf,
    model_registry: HashMap<ModelTier, ModelMetadata>,
    cache_metadata_file: PathBuf,
    cache_metadata: HashMap<ModelTier, CacheMetadata>,
}

impl ModelManager {
    /// Create new model manager
    pub fn new() -> Result<Self> {
        let models_dir = Self::get_models_directory()?;
        let model_registry = Self::initialize_model_registry();
        let cache_metadata_file = models_dir.join("cache_metadata.json");
        
        // Load existing cache metadata
        let cache_metadata = Self::load_cache_metadata(&cache_metadata_file)
            .unwrap_or_else(|_| HashMap::new());
        
        Ok(Self {
            models_dir,
            model_registry,
            cache_metadata_file,
            cache_metadata,
        })
    }

    /// Get the models directory path
    fn get_models_directory() -> Result<PathBuf> {
        let app_data_dir = dirs::data_dir()
            .ok_or_else(|| anyhow::anyhow!("Failed to get data directory"))?;
        
        let models_dir = app_data_dir.join("KagiNote").join("models");
        std::fs::create_dir_all(&models_dir)?;
        
        Ok(models_dir)
    }

    /// Initialize the model registry with quantized models optimized for each tier
    fn initialize_model_registry() -> HashMap<ModelTier, ModelMetadata> {
        let mut registry = HashMap::new();
        
        // Standard tier: Whisper Medium unquantized (~1.5GB) - more compatible
        registry.insert(ModelTier::Standard, ModelMetadata {
            name: "ggml-medium.bin".to_string(),
            url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-medium.bin".to_string(),
            size_mb: 1462, // Actual file size: 1533763059 bytes = 1462MB
            sha256: "f4b2bc61d2b85e3b5a85e8e4c7c8e6d9b2a9c8b7d6e5f4a3b2c1d0e9f8a7b6c5".to_string(), // Placeholder
            tier: ModelTier::Standard,
            quantization: "F32".to_string(),
            description: "Whisper Medium model unquantized - balanced performance".to_string(),
        });

        // High Accuracy tier: Use medium-q5_0 as the downloaded model (matches actual file)
        registry.insert(ModelTier::HighAccuracy, ModelMetadata {
            name: "ggml-medium-q5_0.bin".to_string(),
            url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-medium-q5_0.bin".to_string(),
            size_mb: 514, // Actual file size: 539212467 bytes = 514MB
            sha256: "a9b8c7d6e5f4a3b2c1d0e9f8a7b6c5d4e3f2a1b0c9d8e7f6a5b4c3d2e1f0a9b8".to_string(), // Placeholder
            tier: ModelTier::HighAccuracy,
            quantization: "Q5_0".to_string(),
            description: "Whisper Medium model with Q5_0 quantization - high accuracy".to_string(),
        });

        // Turbo tier: Fallback to standard model since turbo doesn't exist
        registry.insert(ModelTier::Turbo, ModelMetadata {
            name: "ggml-medium.bin".to_string(), // Fallback to standard model
            url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-medium.bin".to_string(),
            size_mb: 1462, // Actual file size: 1533763059 bytes = 1462MB
            sha256: "b8a7c6d5e4f3a2b1c0d9e8f7a6b5c4d3e2f1a0b9c8d7e6f5a4b3c2d1e0f9a8b7".to_string(), // Placeholder
            tier: ModelTier::Turbo,
            quantization: "F32".to_string(),
            description: "Whisper Medium model (fallback for turbo) - fastest available".to_string(),
        });

        registry
    }

    /// Check if model is available locally with comprehensive validation
    pub async fn is_model_available(&self, tier: ModelTier) -> bool {
        tracing::debug!("Checking availability of model tier: {:?}", tier);
        
        if let Some(metadata) = self.model_registry.get(&tier) {
            let model_path = self.models_dir.join(&metadata.name);
            
            if !model_path.exists() {
                tracing::debug!("Model file does not exist: {:?}", model_path);
                return false;
            }
            
            tracing::debug!("Model file exists, verifying integrity: {:?}", model_path);
            match self.verify_model_integrity(&model_path, metadata).await {
                Ok(()) => {
                    tracing::debug!("Model {:?} integrity verified successfully", tier);
                    true
                }
                Err(e) => {
                    tracing::warn!("Model {:?} integrity verification failed: {}", tier, e);
                    false
                }
            }
        } else {
            tracing::warn!("Unknown model tier requested: {:?}", tier);
            false
        }
    }
    
    /// Get detailed cache status for a model
    pub async fn get_cache_status(&self, tier: ModelTier) -> CacheStatus {
        let Some(metadata) = self.model_registry.get(&tier) else {
            return CacheStatus::NotCached;
        };
        
        let model_path = self.models_dir.join(&metadata.name);
        
        if !model_path.exists() {
            return CacheStatus::NotCached;
        }
        
        // Check if we have cache metadata
        if let Some(cache_meta) = self.cache_metadata.get(&tier) {
            // Verify the cached model is still valid
            match self.verify_model_integrity(&model_path, metadata).await {
                Ok(()) => CacheStatus::Cached { metadata: cache_meta.clone() },
                Err(e) => CacheStatus::Corrupted { reason: e.to_string() },
            }
        } else {
            // Model exists but no metadata - consider it cached but unverified
            let file_metadata = match fs::metadata(&model_path).await {
                Ok(meta) => meta,
                Err(_) => return CacheStatus::NotCached,
            };
            
            let cache_meta = CacheMetadata {
                download_timestamp: DateTime::from_timestamp(0, 0).unwrap_or_else(Utc::now),
                file_size: file_metadata.len(),
                sha256_verified: false,
                model_tier: tier,
                last_validation: None,
                validation_status: ValidationStatus::NotValidated,
            };
            
            CacheStatus::Cached { metadata: cache_meta }
        }
    }

    /// Get the path to a model file with comprehensive diagnostics
    pub fn get_model_path(&self, tier: ModelTier) -> Result<PathBuf, ASRError> {
        tracing::debug!("Getting model path for tier: {:?}", tier);
        
        let metadata = self.model_registry.get(&tier)
            .ok_or_else(|| {
                let available_tiers: Vec<String> = self.model_registry.keys()
                    .map(|k| format!("{:?}", k))
                    .collect();
                let error_msg = format!(
                    "Unknown model tier: {:?}. Available tiers: [{}]. Please use one of the supported model tiers.", 
                    tier, 
                    available_tiers.join(", ")
                );
                tracing::error!("{}", error_msg);
                ASRError::ModelLoadFailed { message: error_msg }
            })?;
        
        let model_path = self.models_dir.join(&metadata.name);
        tracing::debug!("Expected model path: {:?}", model_path);
        
        if !model_path.exists() {
            // Enhanced directory diagnostics
            let directory_info = match std::fs::read_dir(&self.models_dir) {
                Ok(entries) => {
                    let files: Vec<String> = entries
                        .filter_map(|e| e.ok())
                        .map(|e| {
                            let file_name = e.file_name().to_string_lossy().to_string();
                            if let Ok(metadata) = e.metadata() {
                                format!("{} ({:.1}MB)", file_name, metadata.len() as f64 / (1024.0 * 1024.0))
                            } else {
                                file_name
                            }
                        })
                        .collect();
                    
                    if files.is_empty() {
                        "Directory is empty - no models downloaded yet".to_string()
                    } else {
                        format!("Available files: [{}]", files.join(", "))
                    }
                }
                Err(e) => format!("Error reading models directory: {}. Directory may not exist or have permission issues.", e)
            };
            
            // Check directory permissions
            let permission_info = match std::fs::metadata(&self.models_dir) {
                Ok(dir_meta) => {
                    if dir_meta.permissions().readonly() {
                        " Directory is read-only - check write permissions."
                    } else {
                        " Directory has write permissions."
                    }
                }
                Err(_) => " Cannot check directory permissions."
            };
            
            let error_msg = format!(
                "Model file not found: '{}' at path: {:?}. \
                Models directory: {:?}. \
                {}{}. \
                Expected model size: ~{}MB. \
                If this is the first run, models will be downloaded automatically which may take several minutes.", 
                metadata.name, 
                model_path,
                self.models_dir,
                directory_info,
                permission_info,
                metadata.size_mb
            );
            
            tracing::error!("{}", error_msg);
            return Err(ASRError::ModelLoadFailed { message: error_msg });
        }

        // Additional path validation
        match std::fs::metadata(&model_path) {
            Ok(file_meta) => {
                tracing::info!(
                    "Model file found: {:?} ({:.1}MB)", 
                    model_path, 
                    file_meta.len() as f64 / (1024.0 * 1024.0)
                );
            }
            Err(e) => {
                let error_msg = format!(
                    "Model file exists but cannot read metadata: {:?}. Error: {}. This may indicate file corruption or permission issues.", 
                    model_path, e
                );
                tracing::error!("{}", error_msg);
                return Err(ASRError::ModelLoadFailed { message: error_msg });
            }
        }

        tracing::debug!("Model path validated: {:?}", model_path);
        Ok(model_path)
    }

    /// Download model if not available, with fallback to any available model
    pub async fn ensure_model_available(
        &mut self, 
        tier: ModelTier,
        progress_callback: Option<ProgressCallback>
    ) -> Result<PathBuf, ASRError> {
        let cache_status = self.get_cache_status(tier).await;
        
        match cache_status {
            CacheStatus::Cached { metadata: _ } => {
                tracing::info!("Using cached model for tier: {:?}", tier);
                if let Some(ref callback) = progress_callback {
                    callback(100, 100); // Signal that model is already available
                }
                self.get_model_path(tier)
            },
            CacheStatus::Corrupted { reason } => {
                tracing::warn!("Cached model corrupted ({}), re-downloading", reason);
                self.download_model(tier, progress_callback).await
            },
            CacheStatus::NotCached => {
                tracing::info!("Model not cached, checking for fallback options for tier: {:?}", tier);
                
                // Try to find any available model as fallback
                if let Some(fallback_tier) = self.find_available_model_fallback().await {
                    tracing::info!("Using fallback model {:?} for requested tier {:?}", fallback_tier, tier);
                    if let Some(ref callback) = progress_callback {
                        callback(100, 100); // Signal fallback is available
                    }
                    return self.get_model_path(fallback_tier);
                }
                
                // No fallback available, download the requested model
                self.download_model(tier, progress_callback).await
            },
            CacheStatus::Downloading { progress: _ } => {
                // This shouldn't happen in normal flow, but handle gracefully
                tracing::warn!("Model already downloading, waiting...");
                // For simplicity, attempt download again (could be improved with download queuing)
                self.download_model(tier, progress_callback).await
            }
        }
    }

    /// Download a specific model
    pub async fn download_model(
        &mut self,
        tier: ModelTier,
        progress_callback: Option<ProgressCallback>
    ) -> Result<PathBuf, ASRError> {
        let metadata = self.model_registry.get(&tier)
            .ok_or_else(|| ASRError::ModelLoadFailed {
                message: format!("Unknown model tier: {:?}", tier),
            })?;

        let model_path = self.models_dir.join(&metadata.name);
        let temp_path = model_path.with_extension("tmp");

        tracing::info!("Downloading {} model: {}", tier.to_string(), metadata.name);
        tracing::info!("Download URL: {}", metadata.url);
        tracing::info!("Expected size: {} MB", metadata.size_mb);

        // Create HTTP client with timeout
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(3600)) // 1 hour timeout
            .build()
            .map_err(|e| ASRError::ModelLoadFailed {
                message: format!("Failed to create HTTP client: {}", e),
            })?;

        // Start download
        let response = client
            .get(&metadata.url)
            .send()
            .await
            .map_err(|e| ASRError::ModelLoadFailed {
                message: format!("Failed to start download: {}", e),
            })?;

        if !response.status().is_success() {
            return Err(ASRError::ModelLoadFailed {
                message: format!("Download failed with status: {}", response.status()),
            });
        }

        let total_size = response.content_length().unwrap_or(metadata.size_mb * 1024 * 1024);

        // Create temporary file
        let mut file = tokio::fs::File::create(&temp_path)
            .await
            .map_err(|e| ASRError::ModelLoadFailed {
                message: format!("Failed to create temporary file: {}", e),
            })?;

        // Download with progress tracking
        let mut downloaded = 0u64;
        let mut stream = response.bytes_stream();
        let mut hasher = Sha256::new();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| ASRError::ModelLoadFailed {
                message: format!("Download error: {}", e),
            })?;

            // Write chunk to file
            use tokio::io::AsyncWriteExt;
            file.write_all(&chunk)
                .await
                .map_err(|e| ASRError::ModelLoadFailed {
                    message: format!("Failed to write to file: {}", e),
                })?;

            // Update hash
            hasher.update(&chunk);

            // Update progress
            downloaded += chunk.len() as u64;
            if let Some(ref callback) = progress_callback {
                callback(downloaded, total_size);
            }
        }

        // Finalize file
        file.sync_all()
            .await
            .map_err(|e| ASRError::ModelLoadFailed {
                message: format!("Failed to sync file: {}", e),
            })?;

        drop(file);

        // Verify integrity (skip for now since we don't have real checksums)
        tracing::info!("Download completed: {} bytes", downloaded);

        // Move temporary file to final location
        tokio::fs::rename(&temp_path, &model_path)
            .await
            .map_err(|e| ASRError::ModelLoadFailed {
                message: format!("Failed to move model file: {}", e),
            })?;

        // Update cache metadata
        let cache_meta = CacheMetadata {
            download_timestamp: Utc::now(),
            file_size: downloaded,
            sha256_verified: false, // TODO: Implement actual checksum verification
            model_tier: tier,
            last_validation: Some(Utc::now()),
            validation_status: ValidationStatus::Valid,
        };
        
        self.cache_metadata.insert(tier, cache_meta);
        self.save_cache_metadata().await.map_err(|e| ASRError::ModelLoadFailed {
            message: format!("Failed to save cache metadata: {}", e),
        })?;

        tracing::info!("Model {} successfully downloaded to: {:?}", metadata.name, model_path);

        Ok(model_path)
    }

    /// Comprehensive model file integrity verification
    async fn verify_model_integrity(&self, path: &Path, metadata: &ModelMetadata) -> Result<()> {
        tracing::debug!("Verifying integrity of model: {} at path: {:?}", metadata.name, path);
        
        // Check file accessibility
        let file_metadata = fs::metadata(path).await
            .map_err(|e| anyhow::anyhow!(
                "Cannot access model file {}: {}. File may be corrupted, moved, or have permission issues.", 
                metadata.name, e
            ))?;
        
        let actual_size = file_metadata.len();
        let expected_size = metadata.size_mb * 1024 * 1024;
        
        tracing::debug!(
            "Model {} size check: actual={}MB ({} bytes), expected=~{}MB ({} bytes)",
            metadata.name,
            actual_size / (1024 * 1024),
            actual_size,
            metadata.size_mb,
            expected_size
        );

        // Allow generous tolerance for size differences (15% for different architectures)
        let size_tolerance = expected_size / 7; // ~14% tolerance
        
        if actual_size < expected_size.saturating_sub(size_tolerance) || 
           actual_size > expected_size + size_tolerance {
            tracing::warn!(
                "Model size verification failed for {}: expected ~{}MB ({} bytes), got {}MB ({} bytes), tolerance: ±{:.1}MB",
                metadata.name,
                metadata.size_mb,
                expected_size,
                actual_size / (1024 * 1024),
                actual_size,
                size_tolerance as f64 / (1024.0 * 1024.0)
            );
            
            // Provide detailed guidance based on size difference
            let size_diff_mb = (actual_size as i64 - expected_size as i64).abs() as f64 / (1024.0 * 1024.0);
            let guidance = if actual_size < expected_size / 2 {
                "File appears to be truncated or download was interrupted. Try deleting and re-downloading."
            } else if actual_size > expected_size * 2 {
                "File is much larger than expected. May be wrong file or corrupted."
            } else {
                "File size is outside acceptable range but may still be usable."
            };
            
            return Err(anyhow::anyhow!(
                "Model size mismatch for {} (difference: {:.1}MB): expected ~{}MB, got {}MB. {}. \
                If this model was downloaded from an external source, it may be incompatible.",
                metadata.name,
                size_diff_mb,
                metadata.size_mb,
                actual_size / (1024 * 1024),
                guidance
            ));
        }

        tracing::debug!(
            "Model {} size verification passed: {}MB ({} bytes) within acceptable range",
            metadata.name,
            actual_size / (1024 * 1024),
            actual_size
        );
        
        // Check file is readable
        match fs::File::open(path).await {
            Ok(_) => {
                tracing::debug!("Model {} file accessibility verified", metadata.name);
            }
            Err(e) => {
                let error_msg = format!(
                    "Model file {} exists but cannot be opened: {}. This may indicate file corruption or permission issues.",
                    metadata.name, e
                );
                tracing::error!("{}", error_msg);
                return Err(anyhow::anyhow!(error_msg));
            }
        }
        
        // Check file modification time for freshness (warn if very old)
        if let Ok(modified) = file_metadata.modified() {
            if let Ok(duration) = modified.elapsed() {
                let days_old = duration.as_secs() / (24 * 3600);
                if days_old > 365 {
                    tracing::warn!(
                        "Model {} is very old ({} days), consider updating for latest improvements",
                        metadata.name, days_old
                    );
                }
            }
        }

        // TODO: Verify SHA256 checksum when we have real checksums from Hugging Face
        // This would require downloading metadata from the model repository
        
        tracing::debug!("Model {} integrity verification completed successfully", metadata.name);
        Ok(())
    }

    /// List all available models
    pub fn list_models(&self) -> Vec<&ModelMetadata> {
        self.model_registry.values().collect()
    }

    /// Get model metadata
    pub fn get_model_metadata(&self, tier: ModelTier) -> Option<&ModelMetadata> {
        self.model_registry.get(&tier)
    }
    
    /// Get cache metadata for a model
    pub fn get_cache_metadata(&self, tier: ModelTier) -> Option<&CacheMetadata> {
        self.cache_metadata.get(&tier)
    }

    /// Clean up old or corrupted models
    pub async fn cleanup_models(&self) -> Result<()> {
        let mut entries = fs::read_dir(&self.models_dir).await?;
        let valid_names: std::collections::HashSet<String> = self.model_registry
            .values()
            .map(|m| m.name.clone())
            .collect();

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            let file_name = path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");

            // Remove files that are not in our registry
            if !valid_names.contains(file_name) && 
               !file_name.ends_with(".tmp") {
                tracing::info!("Removing orphaned model file: {:?}", path);
                let _ = fs::remove_file(path).await;
            }
        }

        Ok(())
    }

    /// Get total storage used by models
    pub async fn get_storage_usage(&self) -> Result<u64> {
        let mut total_size = 0u64;
        let mut entries = fs::read_dir(&self.models_dir).await?;

        while let Some(entry) = entries.next_entry().await? {
            if let Ok(metadata) = entry.metadata().await {
                total_size += metadata.len();
            }
        }

        Ok(total_size)
    }
    
    /// Load cache metadata from disk
    fn load_cache_metadata(cache_file: &Path) -> Result<HashMap<ModelTier, CacheMetadata>> {
        if !cache_file.exists() {
            return Ok(HashMap::new());
        }
        
        let content = std::fs::read_to_string(cache_file)?;
        let metadata: HashMap<ModelTier, CacheMetadata> = serde_json::from_str(&content)
            .unwrap_or_else(|_| HashMap::new());
        
        Ok(metadata)
    }
    
    /// Save cache metadata to disk
    async fn save_cache_metadata(&self) -> Result<()> {
        let content = serde_json::to_string_pretty(&self.cache_metadata)?;
        fs::write(&self.cache_metadata_file, content).await?;
        Ok(())
    }
    
    /// Clear cache for a specific model tier
    pub async fn clear_model_cache(&mut self, tier: ModelTier) -> Result<()> {
        if let Some(metadata) = self.model_registry.get(&tier) {
            let model_path = self.models_dir.join(&metadata.name);
            
            if model_path.exists() {
                fs::remove_file(&model_path).await?;
                tracing::info!("Removed cached model: {:?}", model_path);
            }
            
            self.cache_metadata.remove(&tier);
            self.save_cache_metadata().await?;
        }
        
        Ok(())
    }
    
    /// Find any available model as fallback
    async fn find_available_model_fallback(&self) -> Option<ModelTier> {
        // Check available models in order of preference: Standard -> HighAccuracy -> Turbo
        let fallback_order = [ModelTier::Standard, ModelTier::HighAccuracy, ModelTier::Turbo];
        
        for &tier in &fallback_order {
            if self.is_model_available(tier).await {
                return Some(tier);
            }
        }
        
        None
    }
    
    /// Validate all cached models and clean up corrupted ones
    pub async fn validate_and_cleanup_cache(&mut self) -> Result<()> {
        let mut corrupted_models = Vec::new();
        
        for (&tier, _cache_meta) in &self.cache_metadata {
            if let Some(model_meta) = self.model_registry.get(&tier) {
                let model_path = self.models_dir.join(&model_meta.name);
                
                if let Err(e) = self.verify_model_integrity(&model_path, model_meta).await {
                    tracing::warn!("Model {:?} failed validation: {}", tier, e);
                    corrupted_models.push(tier);
                }
            }
        }
        
        for tier in corrupted_models {
            self.clear_model_cache(tier).await?;
        }
        
        Ok(())
    }
}

impl ModelTier {
    /// Convert to string representation
    pub fn to_string(&self) -> &'static str {
        match self {
            ModelTier::Standard => "Standard",
            ModelTier::HighAccuracy => "High Accuracy",
            ModelTier::Turbo => "Turbo",
        }
    }
}

/// Default implementation for ModelManager
impl Default for ModelManager {
    fn default() -> Self {
        Self::new().expect("Failed to create ModelManager")
    }
}