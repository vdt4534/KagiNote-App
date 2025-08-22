use std::collections::{HashMap, VecDeque};
use std::hash::{DefaultHasher, Hash, Hasher};

/// Advanced content hasher for semantic deduplication of transcription segments
pub struct ContentHasher {
    /// Recent semantic hashes for duplicate detection
    recent_hashes: VecDeque<(u64, String, f32)>, // (hash, text, timestamp)
    /// Word frequency cache for context analysis
    word_frequencies: HashMap<String, f32>,
    /// Configuration parameters
    max_cache_size: usize,
    similarity_threshold: f32,
}

impl ContentHasher {
    pub fn new(max_cache_size: usize, similarity_threshold: f32) -> Self {
        Self {
            recent_hashes: VecDeque::with_capacity(max_cache_size),
            word_frequencies: HashMap::new(),
            max_cache_size,
            similarity_threshold,
        }
    }

    /// Check if content is semantically duplicate of recent segments
    pub fn is_duplicate(&mut self, text: &str, timestamp: f32) -> bool {
        let normalized = self.normalize_text(text);
        if normalized.is_empty() || normalized.len() < 5 {
            return false;
        }

        // Generate semantic hash
        let semantic_hash = self.generate_semantic_hash(&normalized);
        
        // Check against recent hashes
        for (hash, cached_text, _ts) in &self.recent_hashes {
            // Fast hash comparison first
            if *hash == semantic_hash {
                return true;
            }
            
            // Fallback to detailed similarity analysis
            if self.calculate_semantic_similarity(&normalized, cached_text) >= self.similarity_threshold {
                return true;
            }
        }

        // Not a duplicate - add to cache
        self.add_to_cache(semantic_hash, normalized, timestamp);
        false
    }

    /// Generate semantic hash considering word order and context
    fn generate_semantic_hash(&self, text: &str) -> u64 {
        let mut hasher = DefaultHasher::new();
        
        let words: Vec<&str> = text.split_whitespace().collect();
        
        // Hash the core semantic content
        for window in words.windows(3) {
            // Create semantic trigrams that capture meaning
            let trigram = format!("{} {} {}", window[0], window[1], window[2]);
            trigram.hash(&mut hasher);
        }
        
        // Include word count and length as structural features
        words.len().hash(&mut hasher);
        (text.len() / 10).hash(&mut hasher); // Rounded length feature
        
        hasher.finish()
    }

    /// Calculate detailed semantic similarity between texts
    fn calculate_semantic_similarity(&self, text1: &str, text2: &str) -> f32 {
        let words1: Vec<&str> = text1.split_whitespace().collect();
        let words2: Vec<&str> = text2.split_whitespace().collect();

        if words1.is_empty() || words2.is_empty() {
            return 0.0;
        }

        // Multi-layered similarity calculation
        let word_overlap = self.calculate_word_overlap(&words1, &words2);
        let sequence_similarity = self.calculate_sequence_similarity(&words1, &words2);
        let length_similarity = self.calculate_length_similarity(&words1, &words2);
        
        // Weighted combination of similarity metrics
        (word_overlap * 0.5 + sequence_similarity * 0.3 + length_similarity * 0.2).min(1.0)
    }

    /// Calculate word overlap with frequency weighting
    fn calculate_word_overlap(&self, words1: &[&str], words2: &[&str]) -> f32 {
        let mut common_weighted_score = 0.0;
        let mut total_weight = 0.0;

        for word in words1 {
            if words2.contains(word) {
                // Weight rare words higher than common words
                let weight = self.get_word_rarity_weight(word);
                common_weighted_score += weight;
            }
            total_weight += self.get_word_rarity_weight(word);
        }

        if total_weight == 0.0 {
            return 0.0;
        }

        common_weighted_score / total_weight
    }

    /// Calculate sequence similarity using longest common subsequence
    fn calculate_sequence_similarity(&self, words1: &[&str], words2: &[&str]) -> f32 {
        let lcs_length = self.longest_common_subsequence(words1, words2);
        let max_len = words1.len().max(words2.len());
        
        if max_len == 0 {
            return 0.0;
        }
        
        lcs_length as f32 / max_len as f32
    }

    /// Calculate length-based similarity
    fn calculate_length_similarity(&self, words1: &[&str], words2: &[&str]) -> f32 {
        let len1 = words1.len() as f32;
        let len2 = words2.len() as f32;
        
        if len1 == 0.0 || len2 == 0.0 {
            return 0.0;
        }
        
        1.0 - (len1 - len2).abs() / len1.max(len2)
    }

    /// Get word rarity weight (higher for less common words)
    pub fn get_word_rarity_weight(&self, word: &str) -> f32 {
        // Common words get lower weight, rare words get higher weight
        match word.len() {
            1..=2 => 0.1,  // Very short words (articles, etc.)
            3..=4 => match word {
                "the" | "and" | "are" | "but" | "not" | "you" | "all" | "can" | "had" | "her" | "was" | "one" | "our" | "out" | "day" | "get" | "has" | "him" | "how" | "may" | "new" | "now" | "old" | "see" | "two" | "way" | "who" | "boy" | "did" | "its" | "let" | "put" | "say" | "she" | "too" | "use" => 0.2,
                _ => 0.8,
            },
            5..=7 => 1.0,   // Medium length words
            _ => 1.2,       // Longer, potentially more meaningful words
        }
    }

    /// Calculate longest common subsequence between two word arrays
    fn longest_common_subsequence(&self, words1: &[&str], words2: &[&str]) -> usize {
        let m = words1.len();
        let n = words2.len();
        
        if m == 0 || n == 0 {
            return 0;
        }

        let mut dp = vec![vec![0; n + 1]; m + 1];

        for i in 1..=m {
            for j in 1..=n {
                if words1[i - 1] == words2[j - 1] {
                    dp[i][j] = dp[i - 1][j - 1] + 1;
                } else {
                    dp[i][j] = dp[i - 1][j].max(dp[i][j - 1]);
                }
            }
        }

        dp[m][n]
    }

    /// Normalize text for consistent processing
    fn normalize_text(&self, text: &str) -> String {
        text.trim()
            .to_lowercase()
            .chars()
            .filter(|c| c.is_alphanumeric() || c.is_whitespace())
            .collect::<String>()
            .split_whitespace()
            .collect::<Vec<&str>>()
            .join(" ")
    }

    /// Add content to internal cache
    fn add_to_cache(&mut self, hash: u64, text: String, timestamp: f32) {
        self.recent_hashes.push_back((hash, text.clone(), timestamp));
        
        // Update word frequencies
        for word in text.split_whitespace() {
            *self.word_frequencies.entry(word.to_string()).or_insert(0.0) += 1.0;
        }

        // Maintain cache size
        if self.recent_hashes.len() > self.max_cache_size {
            if let Some((_, old_text, _)) = self.recent_hashes.pop_front() {
                // Decay word frequencies from removed text
                for word in old_text.split_whitespace() {
                    if let Some(freq) = self.word_frequencies.get_mut(word) {
                        *freq = (*freq - 1.0).max(0.0);
                        if *freq <= 0.0 {
                            self.word_frequencies.remove(word);
                        }
                    }
                }
            }
        }
    }

    /// Clear all cached content (useful for new sessions)
    pub fn clear_cache(&mut self) {
        self.recent_hashes.clear();
        self.word_frequencies.clear();
    }

    /// Get cache statistics for debugging
    pub fn get_cache_stats(&self) -> (usize, usize) {
        (self.recent_hashes.len(), self.word_frequencies.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exact_duplicate_detection() {
        let mut hasher = ContentHasher::new(5, 0.6);
        
        assert!(!hasher.is_duplicate("Hello world", 1.0));
        assert!(hasher.is_duplicate("Hello world", 2.0));
    }

    #[test]
    fn test_semantic_similarity() {
        let mut hasher = ContentHasher::new(5, 0.6);
        
        assert!(!hasher.is_duplicate("Hello everyone, welcome to our meeting", 1.0));
        assert!(hasher.is_duplicate("Hello everyone, welcome to the meeting", 2.0));
    }

    #[test]
    fn test_word_rarity_weighting() {
        let hasher = ContentHasher::new(5, 0.6);
        
        // Common words should have low weight
        assert!(hasher.get_word_rarity_weight("the") < 0.5);
        
        // Longer words should have higher weight  
        assert!(hasher.get_word_rarity_weight("transcription") > 1.0);
    }

    #[test]
    fn test_cache_size_limit() {
        let mut hasher = ContentHasher::new(3, 0.6);
        
        hasher.is_duplicate("First text", 1.0);
        hasher.is_duplicate("Second text", 2.0);
        hasher.is_duplicate("Third text", 3.0);
        hasher.is_duplicate("Fourth text", 4.0);
        
        let (cache_size, _) = hasher.get_cache_stats();
        assert!(cache_size <= 3);
    }
}