//! Speaker Diarization Module
//! 
//! This module provides speaker diarization functionality for KagiNote.
//! It identifies different speakers in audio streams and assigns speaker IDs 
//! to transcription segments.

pub mod types;
pub mod service;
pub mod pipeline;
pub mod embedder;
pub mod clustering;
pub mod buffer_manager;
pub mod segment_merger;

// Re-export main types and service
pub use types::*;
pub use service::DiarizationService;
pub use pipeline::DiarizationPipeline;

// Re-export for backward compatibility with test expectations
pub use service::DiarizationService as DiarizationEngine;

use anyhow::Result;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Global diarization service instance
static DIARIZATION_SERVICE: tokio::sync::OnceCell<Arc<Mutex<Option<DiarizationService>>>> = 
    tokio::sync::OnceCell::const_new();

/// Initialize the global diarization service
pub async fn initialize() -> Result<()> {
    let service_container = Arc::new(Mutex::new(None));
    let _ = DIARIZATION_SERVICE.set(service_container).map_err(|_| {
        anyhow::anyhow!("Diarization service already initialized")
    })?;
    
    tracing::info!("Diarization module initialized successfully");
    Ok(())
}

/// Get the global diarization service instance
pub async fn get_service() -> Result<Arc<Mutex<Option<DiarizationService>>>> {
    DIARIZATION_SERVICE
        .get()
        .ok_or_else(|| anyhow::anyhow!("Diarization service not initialized"))
        .map(|s| s.clone())
}

/// Create a new diarization service with the given configuration
pub async fn create_service(config: DiarizationConfig) -> Result<DiarizationService> {
    DiarizationService::new(config).await
        .map_err(|e| anyhow::anyhow!("Failed to create diarization service: {:?}", e))
}