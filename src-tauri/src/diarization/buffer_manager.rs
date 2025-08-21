//! Audio Buffer Manager
//! 
//! Thread-safe audio buffer management for shared access between
//! diarization and transcription components.

use super::types::*;
use anyhow::Result;
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use tracing;

/// Thread-safe audio buffer for shared access
pub struct AudioBufferManager {
    buffer: Arc<RwLock<VecDeque<f32>>>,
    metadata: Arc<Mutex<BufferMetadata>>,
    consumers: Arc<Mutex<HashMap<String, ConsumerState>>>,
    config: DiarizationConfig,
}

/// Buffer metadata
#[derive(Debug, Clone)]
struct BufferMetadata {
    sample_rate: u32,
    capacity: usize,
    total_samples_written: u64,
    start_timestamp: std::time::SystemTime,
}

/// Consumer state tracking
#[derive(Debug, Clone)]
struct ConsumerState {
    id: String,
    read_position: u64,
    last_access: std::time::SystemTime,
}

impl AudioBufferManager {
    /// Create a new audio buffer manager
    pub async fn new(config: DiarizationConfig, sample_rate: u32, capacity_seconds: f32) -> Result<Self> {
        let capacity_samples = (capacity_seconds * sample_rate as f32) as usize;
        
        tracing::info!("Initializing AudioBufferManager: {}Hz, {:.1}s capacity ({} samples)", 
                      sample_rate, capacity_seconds, capacity_samples);
        
        let metadata = BufferMetadata {
            sample_rate,
            capacity: capacity_samples,
            total_samples_written: 0,
            start_timestamp: std::time::SystemTime::now(),
        };
        
        Ok(Self {
            buffer: Arc::new(RwLock::new(VecDeque::with_capacity(capacity_samples))),
            metadata: Arc::new(Mutex::new(metadata)),
            consumers: Arc::new(Mutex::new(HashMap::new())),
            config,
        })
    }
    
    /// Write audio samples to the buffer
    pub async fn write_samples(&self, samples: &[f32]) -> Result<usize> {
        if samples.is_empty() {
            return Ok(0);
        }
        
        let mut buffer = self.buffer.write().await;
        let mut metadata = self.metadata.lock().await;
        
        let mut samples_written = 0;
        
        for &sample in samples {
            // If buffer is full, remove oldest sample
            if buffer.len() >= metadata.capacity {
                buffer.pop_front();
            }
            
            buffer.push_back(sample);
            samples_written += 1;
        }
        
        metadata.total_samples_written += samples_written as u64;
        
        tracing::trace!("Wrote {} samples to buffer (total: {})", 
                       samples_written, metadata.total_samples_written);
        
        Ok(samples_written)
    }
    
    /// Register a new consumer
    pub async fn register_consumer(&self, consumer_id: String) -> Result<()> {
        let mut consumers = self.consumers.lock().await;
        let metadata = self.metadata.lock().await;
        
        let consumer_state = ConsumerState {
            id: consumer_id.clone(),
            read_position: metadata.total_samples_written,
            last_access: std::time::SystemTime::now(),
        };
        
        consumers.insert(consumer_id.clone(), consumer_state);
        
        tracing::info!("Registered consumer: {}", consumer_id);
        Ok(())
    }
    
    /// Unregister a consumer
    pub async fn unregister_consumer(&self, consumer_id: &str) -> Result<()> {
        let mut consumers = self.consumers.lock().await;
        
        if consumers.remove(consumer_id).is_some() {
            tracing::info!("Unregistered consumer: {}", consumer_id);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Consumer not found: {}", consumer_id))
        }
    }
    
    /// Read samples for a specific consumer
    pub async fn read_samples(&self, consumer_id: &str, max_samples: usize) -> Result<Vec<f32>> {
        let buffer = self.buffer.read().await;
        let mut consumers = self.consumers.lock().await;
        let metadata = self.metadata.lock().await;
        
        let consumer = consumers.get_mut(consumer_id)
            .ok_or_else(|| anyhow::anyhow!("Consumer not registered: {}", consumer_id))?;
        
        // Calculate available samples for this consumer
        let available_samples = metadata.total_samples_written.saturating_sub(consumer.read_position);
        let samples_to_read = max_samples.min(available_samples as usize).min(buffer.len());
        
        if samples_to_read == 0 {
            return Ok(vec![]);
        }
        
        // Calculate start position in circular buffer
        let buffer_start = if buffer.len() < metadata.capacity {
            0
        } else {
            buffer.len().saturating_sub(available_samples as usize)
        };
        
        let samples: Vec<f32> = buffer.range(buffer_start..buffer_start + samples_to_read)
            .cloned()
            .collect();
        
        // Update consumer state
        consumer.read_position += samples.len() as u64;
        consumer.last_access = std::time::SystemTime::now();
        
        tracing::trace!("Consumer {} read {} samples", consumer_id, samples.len());
        
        Ok(samples)
    }
    
    /// Get samples in a specific time range
    pub async fn get_samples_in_range(&self, start_time: f32, end_time: f32) -> Result<Vec<f32>> {
        let buffer = self.buffer.read().await;
        let metadata = self.metadata.lock().await;
        
        let start_sample = (start_time * metadata.sample_rate as f32) as usize;
        let end_sample = (end_time * metadata.sample_rate as f32) as usize;
        
        if start_sample >= end_sample || buffer.is_empty() {
            return Ok(vec![]);
        }
        
        // Calculate the position in the circular buffer
        let buffer_duration_samples = buffer.len();
        let current_position = metadata.total_samples_written as usize;
        
        // Check if the requested range is still available in the buffer
        let oldest_available = current_position.saturating_sub(buffer_duration_samples);
        
        if start_sample < oldest_available {
            return Err(anyhow::anyhow!("Requested samples are no longer available in buffer"));
        }
        
        let buffer_start_idx = start_sample - oldest_available;
        let buffer_end_idx = (end_sample - oldest_available).min(buffer.len());
        
        if buffer_start_idx >= buffer.len() {
            return Ok(vec![]);
        }
        
        let samples: Vec<f32> = buffer.range(buffer_start_idx..buffer_end_idx)
            .cloned()
            .collect();
        
        Ok(samples)
    }
    
    /// Get buffer state information
    pub async fn get_buffer_state(&self) -> Result<BufferState> {
        let buffer = self.buffer.read().await;
        let metadata = self.metadata.lock().await;
        let consumers = self.consumers.lock().await;
        
        let mut read_positions = HashMap::new();
        for (id, consumer) in consumers.iter() {
            read_positions.insert(id.clone(), consumer.read_position as usize);
        }
        
        let state = BufferState {
            write_position: metadata.total_samples_written as usize,
            read_positions,
            capacity: metadata.capacity,
            utilization: buffer.len() as f32 / metadata.capacity as f32,
            sample_rate: metadata.sample_rate,
        };
        
        Ok(state)
    }
    
    /// Get buffer statistics
    pub async fn get_statistics(&self) -> Result<HashMap<String, f32>> {
        let buffer = self.buffer.read().await;
        let metadata = self.metadata.lock().await;
        let consumers = self.consumers.lock().await;
        
        let mut stats = HashMap::new();
        stats.insert("capacity_samples".to_string(), metadata.capacity as f32);
        stats.insert("current_size".to_string(), buffer.len() as f32);
        stats.insert("utilization".to_string(), buffer.len() as f32 / metadata.capacity as f32);
        stats.insert("total_samples_written".to_string(), metadata.total_samples_written as f32);
        stats.insert("active_consumers".to_string(), consumers.len() as f32);
        stats.insert("sample_rate".to_string(), metadata.sample_rate as f32);
        
        let duration_seconds = metadata.start_timestamp.elapsed()
            .unwrap_or_default()
            .as_secs_f32();
        stats.insert("uptime_seconds".to_string(), duration_seconds);
        
        // Calculate average read lag
        if !consumers.is_empty() {
            let avg_lag = consumers.values()
                .map(|c| metadata.total_samples_written.saturating_sub(c.read_position))
                .sum::<u64>() as f32 / consumers.len() as f32;
            stats.insert("average_consumer_lag".to_string(), avg_lag);
        }
        
        Ok(stats)
    }
    
    /// Clear the buffer
    pub async fn clear(&self) -> Result<()> {
        let mut buffer = self.buffer.write().await;
        let mut metadata = self.metadata.lock().await;
        let mut consumers = self.consumers.lock().await;
        
        buffer.clear();
        metadata.total_samples_written = 0;
        metadata.start_timestamp = std::time::SystemTime::now();
        
        // Reset all consumer positions
        for consumer in consumers.values_mut() {
            consumer.read_position = 0;
        }
        
        tracing::info!("Buffer cleared");
        Ok(())
    }
    
    /// Cleanup inactive consumers
    pub async fn cleanup_inactive_consumers(&self, timeout_seconds: u64) -> Result<usize> {
        let mut consumers = self.consumers.lock().await;
        let threshold = std::time::SystemTime::now() - std::time::Duration::from_secs(timeout_seconds);
        
        let initial_count = consumers.len();
        consumers.retain(|id, consumer| {
            let keep = consumer.last_access > threshold;
            if !keep {
                tracing::info!("Removing inactive consumer: {}", id);
            }
            keep
        });
        
        let removed_count = initial_count - consumers.len();
        
        if removed_count > 0 {
            tracing::info!("Cleaned up {} inactive consumers", removed_count);
        }
        
        Ok(removed_count)
    }
    
    /// Check if buffer has enough data for processing
    pub async fn has_sufficient_data(&self, required_seconds: f32) -> Result<bool> {
        let buffer = self.buffer.read().await;
        let metadata = self.metadata.lock().await;
        
        let required_samples = (required_seconds * metadata.sample_rate as f32) as usize;
        Ok(buffer.len() >= required_samples)
    }
    
    /// Get the current buffer duration in seconds
    pub async fn get_duration_seconds(&self) -> Result<f32> {
        let buffer = self.buffer.read().await;
        let metadata = self.metadata.lock().await;
        
        let duration = buffer.len() as f32 / metadata.sample_rate as f32;
        Ok(duration)
    }
}