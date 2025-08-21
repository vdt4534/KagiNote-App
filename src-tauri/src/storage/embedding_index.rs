use anyhow::{Context, Result};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use uuid::Uuid;

use crate::models::{VoiceEmbedding, SimilarSpeaker};

/// Fast in-memory index for voice embedding similarity search
/// Uses a simplified LSH (Locality Sensitive Hashing) approach for performance
pub struct EmbeddingIndex {
    /// Mapping from speaker ID to embeddings
    embeddings: Arc<RwLock<HashMap<Uuid, Vec<VoiceEmbedding>>>>,
    /// LSH buckets for fast approximate search
    lsh_buckets: Arc<RwLock<HashMap<String, Vec<Uuid>>>>,
    /// Number of hash functions for LSH
    num_hashes: usize,
    /// Dimension of embeddings (assumed consistent)
    embedding_dimension: usize,
}

impl EmbeddingIndex {
    /// Create a new embedding index
    pub fn new(embedding_dimension: usize, num_hashes: usize) -> Self {
        Self {
            embeddings: Arc::new(RwLock::new(HashMap::new())),
            lsh_buckets: Arc::new(RwLock::new(HashMap::new())),
            num_hashes,
            embedding_dimension,
        }
    }

    /// Add an embedding to the index
    pub fn add_embedding(&self, embedding: VoiceEmbedding) -> Result<()> {
        let speaker_id = embedding.speaker_id;
        
        // Add to embeddings map
        {
            let mut embeddings = self.embeddings.write()
                .map_err(|_| anyhow::anyhow!("Failed to acquire write lock on embeddings"))?;
            
            embeddings.entry(speaker_id)
                .or_insert_with(Vec::new)
                .push(embedding.clone());
        }

        // Add to LSH buckets
        let hash_values = self.compute_lsh_hashes(&embedding.vector);
        {
            let mut buckets = self.lsh_buckets.write()
                .map_err(|_| anyhow::anyhow!("Failed to acquire write lock on LSH buckets"))?;
            
            for hash_value in hash_values {
                buckets.entry(hash_value)
                    .or_insert_with(Vec::new)
                    .push(speaker_id);
            }
        }

        Ok(())
    }

    /// Remove all embeddings for a speaker
    pub fn remove_speaker(&self, speaker_id: Uuid) -> Result<()> {
        // Remove from embeddings
        let embeddings_to_remove = {
            let mut embeddings = self.embeddings.write()
                .map_err(|_| anyhow::anyhow!("Failed to acquire write lock on embeddings"))?;
            
            embeddings.remove(&speaker_id).unwrap_or_default()
        };

        // Remove from LSH buckets
        {
            let mut buckets = self.lsh_buckets.write()
                .map_err(|_| anyhow::anyhow!("Failed to acquire write lock on LSH buckets"))?;
            
            for bucket_speakers in buckets.values_mut() {
                bucket_speakers.retain(|&id| id != speaker_id);
            }
            
            // Clean up empty buckets
            buckets.retain(|_, speakers| !speakers.is_empty());
        }

        Ok(())
    }

    /// Update embeddings for a speaker (replace all)
    pub fn update_speaker_embeddings(
        &self, 
        speaker_id: Uuid, 
        new_embeddings: Vec<VoiceEmbedding>
    ) -> Result<()> {
        // Remove existing embeddings
        self.remove_speaker(speaker_id)?;
        
        // Add new embeddings
        for embedding in new_embeddings {
            self.add_embedding(embedding)?;
        }

        Ok(())
    }

    /// Fast similarity search using LSH + exact verification
    pub fn find_similar_embeddings(
        &self,
        query_vector: &[f32],
        threshold: f32,
        max_results: usize,
    ) -> Result<Vec<(Uuid, f32)>> {
        if query_vector.len() != self.embedding_dimension {
            return Err(anyhow::anyhow!(
                "Query vector dimension {} doesn't match index dimension {}",
                query_vector.len(),
                self.embedding_dimension
            ));
        }

        // Step 1: Find candidate speakers using LSH
        let candidate_speakers = self.find_lsh_candidates(query_vector)?;

        // Step 2: Exact similarity calculation for candidates
        let mut similarities = Vec::new();
        
        {
            let embeddings = self.embeddings.read()
                .map_err(|_| anyhow::anyhow!("Failed to acquire read lock on embeddings"))?;
            
            for speaker_id in candidate_speakers {
                if let Some(speaker_embeddings) = embeddings.get(&speaker_id) {
                    // Calculate best similarity for this speaker
                    let mut best_similarity = 0.0;
                    
                    for embedding in speaker_embeddings {
                        let similarity = cosine_similarity(query_vector, &embedding.vector);
                        if similarity > best_similarity {
                            best_similarity = similarity;
                        }
                    }
                    
                    if best_similarity >= threshold {
                        similarities.push((speaker_id, best_similarity));
                    }
                }
            }
        }

        // Step 3: Sort by similarity and limit results
        similarities.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        similarities.truncate(max_results);

        Ok(similarities)
    }

    /// Get current index statistics
    pub fn get_stats(&self) -> Result<IndexStats> {
        let embeddings = self.embeddings.read()
            .map_err(|_| anyhow::anyhow!("Failed to acquire read lock on embeddings"))?;
        
        let buckets = self.lsh_buckets.read()
            .map_err(|_| anyhow::anyhow!("Failed to acquire read lock on LSH buckets"))?;

        let total_embeddings: usize = embeddings.values()
            .map(|emb_vec| emb_vec.len())
            .sum();

        let total_bucket_entries: usize = buckets.values()
            .map(|speakers| speakers.len())
            .sum();

        Ok(IndexStats {
            total_speakers: embeddings.len(),
            total_embeddings,
            total_buckets: buckets.len(),
            total_bucket_entries,
            embedding_dimension: self.embedding_dimension,
            num_hashes: self.num_hashes,
        })
    }

    /// Clear all data from the index
    pub fn clear(&self) -> Result<()> {
        {
            let mut embeddings = self.embeddings.write()
                .map_err(|_| anyhow::anyhow!("Failed to acquire write lock on embeddings"))?;
            embeddings.clear();
        }
        
        {
            let mut buckets = self.lsh_buckets.write()
                .map_err(|_| anyhow::anyhow!("Failed to acquire write lock on LSH buckets"))?;
            buckets.clear();
        }

        Ok(())
    }

    /// Rebuild the entire index from a fresh set of embeddings
    pub fn rebuild(&self, all_embeddings: Vec<VoiceEmbedding>) -> Result<()> {
        self.clear()?;
        
        for embedding in all_embeddings {
            self.add_embedding(embedding)?;
        }

        Ok(())
    }

    // Private methods

    /// Compute LSH hash values for a vector
    fn compute_lsh_hashes(&self, vector: &[f32]) -> Vec<String> {
        let mut hashes = Vec::with_capacity(self.num_hashes);
        
        for i in 0..self.num_hashes {
            // Simple random projection hash
            // In practice, you'd want to use pre-computed random vectors
            let hash_value = self.simple_hash(vector, i);
            hashes.push(format!("{}_{}", i, hash_value));
        }
        
        hashes
    }

    /// Simple hash function for LSH (simplified implementation)
    fn simple_hash(&self, vector: &[f32], seed: usize) -> i32 {
        let mut sum = 0.0;
        for (j, &value) in vector.iter().enumerate() {
            // Simple pseudo-random coefficient based on seed and index
            let coeff = ((seed * 1000 + j) as f32).sin();
            sum += value * coeff;
        }
        
        if sum >= 0.0 { 1 } else { 0 }
    }

    /// Find candidate speakers using LSH buckets
    fn find_lsh_candidates(&self, query_vector: &[f32]) -> Result<Vec<Uuid>> {
        let query_hashes = self.compute_lsh_hashes(query_vector);
        let mut candidates = std::collections::HashSet::new();
        
        {
            let buckets = self.lsh_buckets.read()
                .map_err(|_| anyhow::anyhow!("Failed to acquire read lock on LSH buckets"))?;
            
            for hash_value in query_hashes {
                if let Some(speaker_ids) = buckets.get(&hash_value) {
                    candidates.extend(speaker_ids);
                }
            }
        }

        // If no LSH candidates found, fall back to checking all speakers
        // This ensures we don't miss results due to LSH approximation
        if candidates.is_empty() {
            let embeddings = self.embeddings.read()
                .map_err(|_| anyhow::anyhow!("Failed to acquire read lock on embeddings"))?;
            candidates.extend(embeddings.keys());
        }

        Ok(candidates.into_iter().collect())
    }
}

/// Statistics about the embedding index
#[derive(Debug, Clone)]
pub struct IndexStats {
    pub total_speakers: usize,
    pub total_embeddings: usize,
    pub total_buckets: usize,
    pub total_bucket_entries: usize,
    pub embedding_dimension: usize,
    pub num_hashes: usize,
}

/// Calculate cosine similarity between two vectors
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() {
        return 0.0;
    }

    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }

    dot_product / (norm_a * norm_b)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::VoiceEmbedding;

    fn create_test_embedding(speaker_id: Uuid, vector: Vec<f32>) -> VoiceEmbedding {
        VoiceEmbedding::new(
            speaker_id,
            vector,
            "test_model".to_string(),
            0.9,
            3.0,
        )
    }

    #[test]
    fn test_index_creation() {
        let index = EmbeddingIndex::new(128, 8);
        let stats = index.get_stats().unwrap();
        
        assert_eq!(stats.embedding_dimension, 128);
        assert_eq!(stats.num_hashes, 8);
        assert_eq!(stats.total_speakers, 0);
        assert_eq!(stats.total_embeddings, 0);
    }

    #[test]
    fn test_add_and_search_embeddings() {
        let index = EmbeddingIndex::new(3, 4);
        
        let speaker1 = Uuid::new_v4();
        let speaker2 = Uuid::new_v4();
        
        // Add embeddings
        let embedding1 = create_test_embedding(speaker1, vec![1.0, 0.0, 0.0]);
        let embedding2 = create_test_embedding(speaker2, vec![0.0, 1.0, 0.0]);
        
        index.add_embedding(embedding1).unwrap();
        index.add_embedding(embedding2).unwrap();
        
        // Check stats
        let stats = index.get_stats().unwrap();
        assert_eq!(stats.total_speakers, 2);
        assert_eq!(stats.total_embeddings, 2);
        
        // Search for similar vectors
        let query = vec![0.9, 0.1, 0.0]; // Similar to speaker1
        let results = index.find_similar_embeddings(&query, 0.5, 10).unwrap();
        
        assert!(!results.is_empty());
        // Should find speaker1 with high similarity
        let (found_speaker, similarity) = &results[0];
        assert!(similarity > &0.8);
    }

    #[test]
    fn test_remove_speaker() {
        let index = EmbeddingIndex::new(3, 4);
        
        let speaker1 = Uuid::new_v4();
        let speaker2 = Uuid::new_v4();
        
        // Add embeddings
        index.add_embedding(create_test_embedding(speaker1, vec![1.0, 0.0, 0.0])).unwrap();
        index.add_embedding(create_test_embedding(speaker2, vec![0.0, 1.0, 0.0])).unwrap();
        
        assert_eq!(index.get_stats().unwrap().total_speakers, 2);
        
        // Remove one speaker
        index.remove_speaker(speaker1).unwrap();
        
        let stats = index.get_stats().unwrap();
        assert_eq!(stats.total_speakers, 1);
        assert_eq!(stats.total_embeddings, 1);
    }

    #[test]
    fn test_clear_index() {
        let index = EmbeddingIndex::new(3, 4);
        
        // Add some embeddings
        index.add_embedding(create_test_embedding(Uuid::new_v4(), vec![1.0, 0.0, 0.0])).unwrap();
        index.add_embedding(create_test_embedding(Uuid::new_v4(), vec![0.0, 1.0, 0.0])).unwrap();
        
        assert_eq!(index.get_stats().unwrap().total_embeddings, 2);
        
        // Clear the index
        index.clear().unwrap();
        
        let stats = index.get_stats().unwrap();
        assert_eq!(stats.total_speakers, 0);
        assert_eq!(stats.total_embeddings, 0);
        assert_eq!(stats.total_buckets, 0);
    }

    #[test]
    fn test_cosine_similarity() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![0.0, 1.0, 0.0];
        let c = vec![1.0, 0.0, 0.0];
        
        // Orthogonal vectors should have 0 similarity
        assert!((cosine_similarity(&a, &b) - 0.0).abs() < 0.001);
        
        // Identical vectors should have 1.0 similarity
        assert!((cosine_similarity(&a, &c) - 1.0).abs() < 0.001);
        
        // Test with non-unit vectors
        let d = vec![2.0, 0.0, 0.0];
        assert!((cosine_similarity(&a, &d) - 1.0).abs() < 0.001);
    }
}